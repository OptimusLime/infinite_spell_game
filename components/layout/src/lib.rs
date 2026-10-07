//! Flexbox layout (taffy) and painting of ui.* trees, plus hit testing for what it painted.
mod text;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;

use atelico_component::tiny_skia::{self, Color, FillRule, FilterQuality, GradientStop, LinearGradient, Mask, Paint, Path, PathBuilder, Pattern, Pixmap, Point as SkPoint, Shader, SpreadMode, Stroke, Transform};
use atelico_component::{parse_color, register, Component, Hit, Rect, Surface, Value};
use serde_json::json;
use taffy::prelude::*;
use taffy::{Overflow, Point};

use text::{Fonts, TextSpec};

register!(Component { name: "layout", paint, hit, inspect, ..Component::NONE });

const KINDS: [&str; 5] = ["node", "text", "button", "input", "image"];

#[derive(Default)]
struct Painted {
    hits: Vec<Hit>,
    boxes: Vec<(String, Rect)>,
}

/// What each layer (see `atelico_component::layer`) last painted.
static PAINTED: Mutex<BTreeMap<u32, Painted>> = Mutex::new(BTreeMap::new());

thread_local! {
    static FONTS: RefCell<Fonts> = RefCell::new(Fonts::default());
    static IMAGES: RefCell<HashMap<PathBuf, Option<Pixmap>>> = RefCell::new(HashMap::new());
}

enum Leaf {
    Text(TextSpec),
    Image(f32, f32),
}

fn hit(x: f32, y: f32) -> Option<Hit> {
    PAINTED.lock().unwrap().get(&atelico_component::layer())?.hits.iter().rev().find(|h| h.rect.contains(x, y)).cloned()
}

/// `{layers = {[layer] = {{id, rect}}}}`: laid-out boxes with ids, in each layer's logical coordinates.
fn inspect() -> Value {
    let p = PAINTED.lock().unwrap();
    let layers: serde_json::Map<String, Value> =
        p.iter().map(|(l, p)| (l.to_string(), p.boxes.iter().map(|(id, r)| json!({ "id": id, "rect": [r.x, r.y, r.w, r.h] })).collect())).collect();
    json!({ "layers": layers })
}

fn paint(node: &Value, s: &mut Surface) -> bool {
    if !KINDS.contains(&kind(node)) {
        return false;
    }
    let mut tree: TaffyTree<Leaf> = TaffyTree::new();
    let root = FONTS.with_borrow_mut(|fonts| build(&mut tree, node, fonts, s));
    let mut root_style = tree.style(root).cloned().unwrap_or_default();
    if root_style.size.width.is_auto() {
        root_style.size.width = length(s.rect.w);
    }
    if root_style.size.height.is_auto() {
        root_style.size.height = length(s.rect.h);
    }
    let _ = tree.set_style(root, root_style);
    let available = Size { width: AvailableSpace::Definite(s.rect.w), height: AvailableSpace::Definite(s.rect.h) };
    FONTS.with_borrow_mut(|fonts| {
        let _ = tree.compute_layout_with_measure(root, available, |known, avail, _, leaf, _| measure(fonts, known, avail, leaf));
    });
    let mut out = Painted::default();
    FONTS.with_borrow_mut(|fonts| {
        let mut p = Painter { tree: &tree, fonts, out: &mut out };
        p.node(node, root, s, s.rect.x, s.rect.y, None);
    });
    PAINTED.lock().unwrap().insert(atelico_component::layer(), out);
    true
}

fn kind(node: &Value) -> &str {
    node.get("kind").and_then(Value::as_str).unwrap_or("")
}

fn style_of(node: &Value) -> &Value {
    static EMPTY: Value = Value::Null;
    node.get("style").filter(|v| v.is_object()).unwrap_or(&EMPTY)
}

fn f(st: &Value, k: &str) -> Option<f32> {
    st.get(k).and_then(Value::as_f64).map(|v| v as f32)
}

fn b(st: &Value, k: &str) -> bool {
    st.get(k).and_then(Value::as_bool).unwrap_or(false)
}

fn s<'a>(st: &'a Value, k: &str) -> Option<&'a str> {
    st.get(k).and_then(Value::as_str)
}

fn children(node: &Value) -> &[Value] {
    node.get("children").and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn dim(v: Option<&Value>) -> Dimension {
    match v {
        Some(Value::Number(n)) => length(n.as_f64().unwrap_or(0.0) as f32),
        Some(Value::String(p)) if p.ends_with('%') => percent(p.trim_end_matches('%').parse::<f32>().unwrap_or(0.0) / 100.0),
        _ => auto(),
    }
}

fn lp(st: &Value, k: &str) -> Option<LengthPercentage> {
    match st.get(k) {
        Some(Value::Number(n)) => Some(length(n.as_f64().unwrap_or(0.0) as f32)),
        Some(Value::String(p)) if p.ends_with('%') => Some(percent(p.trim_end_matches('%').parse::<f32>().unwrap_or(0.0) / 100.0)),
        _ => None,
    }
}

fn lpa(st: &Value, k: &str) -> LengthPercentageAuto {
    match lp(st, k) {
        Some(v) => v.into(),
        None => LengthPercentageAuto::auto(),
    }
}

fn style(node: &Value) -> Style {
    let st = style_of(node);
    let mut out = Style { display: if b(st, "hidden") { Display::None } else { Display::Flex }, ..Style::default() };
    if s(st, "direction") == Some("column") {
        out.flex_direction = FlexDirection::Column;
    }
    out.size = Size { width: dim(st.get("width")), height: dim(st.get("height")) };
    out.min_size = Size { width: dim(st.get("min_width")), height: dim(st.get("min_height")) };
    out.max_size = Size { width: dim(st.get("max_width")), height: dim(st.get("max_height")) };
    out.aspect_ratio = f(st, "aspect");
    out.flex_grow = f(st, "grow").unwrap_or(0.0);
    out.flex_shrink = f(st, "shrink").unwrap_or(1.0);
    if let Some(g) = lp(st, "gap") {
        out.gap = Size { width: g, height: g };
    }
    let pad = |side: &str, axis: &str| lp(st, &format!("padding_{side}")).or_else(|| lp(st, &format!("padding_{axis}"))).or_else(|| lp(st, "padding")).unwrap_or(zero());
    out.padding = taffy::Rect { left: pad("left", "x"), right: pad("right", "x"), top: pad("top", "y"), bottom: pad("bottom", "y") };
    let m = |side: &str| lp(st, &format!("margin_{side}")).or_else(|| lp(st, "margin")).map_or(LengthPercentageAuto::length(0.0), Into::into);
    out.margin = taffy::Rect { left: m("left"), right: m("right"), top: m("top"), bottom: m("bottom") };
    let border = f(st, "border").unwrap_or(0.0);
    out.border = taffy::Rect { left: length(border), right: length(border), top: length(border), bottom: length(border) };
    out.align_items = match s(st, "align") {
        Some("start") => Some(AlignItems::FlexStart),
        Some("center") => Some(AlignItems::Center),
        Some("end") => Some(AlignItems::FlexEnd),
        Some("baseline") => Some(AlignItems::Baseline),
        _ => None,
    };
    out.justify_content = match s(st, "justify") {
        Some("center") => Some(JustifyContent::Center),
        Some("end") => Some(JustifyContent::FlexEnd),
        Some("space_between") => Some(JustifyContent::SpaceBetween),
        Some("space_around") => Some(JustifyContent::SpaceAround),
        Some("space_evenly") => Some(JustifyContent::SpaceEvenly),
        _ => None,
    };
    if b(st, "wrap") {
        out.flex_wrap = FlexWrap::Wrap;
    }
    if b(st, "absolute") {
        out.position = Position::Absolute;
        out.inset = taffy::Rect { left: lpa(st, "left"), right: lpa(st, "right"), top: lpa(st, "top"), bottom: lpa(st, "bottom") };
    }
    if b(st, "scroll") {
        out.overflow = Point { x: Overflow::Clip, y: Overflow::Scroll };
    } else if b(st, "clip") {
        out.overflow = Point { x: Overflow::Clip, y: Overflow::Clip };
    }
    out
}

fn build(tree: &mut TaffyTree<Leaf>, node: &Value, fonts: &mut Fonts, s: &Surface) -> NodeId {
    let st = style(node);
    match kind(node) {
        "text" => {
            let spec = TextSpec::of(node, fonts, s);
            tree.new_leaf_with_context(st, Leaf::Text(spec)).expect("leaf")
        }
        "image" => {
            let (w, h) = image(node, s).map_or((0.0, 0.0), |(_, r)| (r.w, r.h));
            tree.new_leaf_with_context(st, Leaf::Image(w, h)).expect("leaf")
        }
        _ => {
            let ids: Vec<NodeId> = children(node).iter().map(|c| build(tree, c, fonts, s)).collect();
            tree.new_with_children(st, &ids).expect("node")
        }
    }
}

fn measure(fonts: &mut Fonts, known: Size<Option<f32>>, avail: Size<AvailableSpace>, leaf: Option<&mut Leaf>) -> Size<f32> {
    match leaf {
        Some(Leaf::Text(t)) => {
            let max = known.width.unwrap_or(match avail.width {
                AvailableSpace::Definite(w) => w,
                AvailableSpace::MinContent => 0.0,
                AvailableSpace::MaxContent => f32::INFINITY,
            });
            let lines = fonts.wrap(t, max);
            let w = lines.iter().map(|l| l.1).fold(0.0, f32::max);
            Size { width: known.width.unwrap_or(w.ceil()), height: known.height.unwrap_or(lines.len().max(1) as f32 * t.line_height()) }
        }
        Some(Leaf::Image(w, h)) => match (known.width, known.height) {
            (Some(kw), None) if *w > 0.0 => Size { width: kw, height: kw * *h / *w },
            (None, Some(kh)) if *h > 0.0 => Size { width: kh * *w / *h, height: kh },
            (kw, kh) => Size { width: kw.unwrap_or(*w), height: kh.unwrap_or(*h) },
        },
        None => Size::ZERO,
    }
}

/// The decoded texture and the atlas cell to draw, in texture pixels.
fn image(node: &Value, s: &Surface) -> Option<(PathBuf, Rect)> {
    let path = s.asset(node.get("texture")?.as_str()?)?;
    let (w, h) = IMAGES.with_borrow_mut(|images| {
        let img = images.entry(path.clone()).or_insert_with(|| std::fs::read(&path).ok().and_then(|b| Pixmap::decode_png(&b).ok()));
        img.as_ref().map(|p| (p.width() as f32, p.height() as f32))
    })?;
    let atlas: Vec<f32> = node.get("atlas").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_f64).map(|v| v as f32).collect()).unwrap_or_default();
    let r = if atlas.len() == 4 { Rect { x: atlas[0], y: atlas[1], w: atlas[2] - atlas[0], h: atlas[3] - atlas[1] } } else { Rect { x: 0.0, y: 0.0, w, h } };
    Some((path, r))
}

fn rounded(r: Rect, radius: f32) -> Option<Path> {
    let radius = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
    if radius <= 0.0 {
        return Some(PathBuilder::from_rect(tiny_skia::Rect::from_xywh(r.x, r.y, r.w, r.h)?));
    }
    let (x, y, w, h, k) = (r.x, r.y, r.w, r.h, radius);
    let mut pb = PathBuilder::new();
    pb.move_to(x + k, y);
    pb.line_to(x + w - k, y);
    pb.quad_to(x + w, y, x + w, y + k);
    pb.line_to(x + w, y + h - k);
    pb.quad_to(x + w, y + h, x + w - k, y + h);
    pb.line_to(x + k, y + h);
    pb.quad_to(x, y + h, x, y + h - k);
    pb.line_to(x, y + k);
    pb.quad_to(x, y, x + k, y);
    pb.close();
    pb.finish()
}

fn solid(c: Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(c);
    p.anti_alias = true;
    p
}

struct Painter<'a> {
    tree: &'a TaffyTree<Leaf>,
    fonts: &'a mut Fonts,
    out: &'a mut Painted,
}

impl Painter<'_> {
    fn node(&mut self, node: &Value, id: NodeId, s: &mut Surface, px: f32, py: f32, clip: Option<&Mask>) {
        let Ok(l) = self.tree.layout(id) else { return };
        let st = style_of(node);
        if b(st, "hidden") {
            return;
        }
        let r = Rect { x: px + l.location.x, y: py + l.location.y, w: l.size.width, h: l.size.height };
        if let Some(id) = node.get("id").and_then(Value::as_str) {
            self.out.boxes.push((id.to_string(), r));
        }
        let scale = s.scale;
        let t = Transform::from_scale(scale, scale);
        let hovered = node.get("hover").filter(|h| h.is_object()).filter(|_| atelico_component::pointer().is_some_and(|(x, y)| r.contains(x, y)));
        let over = |k: &str| hovered.and_then(|h| s_color(h, k)).or_else(|| s_color(st, k));
        let radius = f(st, "radius").unwrap_or(0.0);
        let fill = match (gradient(st, r), over("background")) {
            (Some(g), _) => Some(Paint { shader: g, anti_alias: true, ..Paint::default() }),
            (None, Some(bg)) => Some(solid(bg)),
            _ => None,
        };
        if let (Some(fill), Some(path)) = (fill, rounded(r, radius)) {
            let t = match f(st, "rotation") {
                Some(a) => Transform::from_rotate_at(a.to_degrees(), r.x + r.w / 2.0, r.y + r.h / 2.0).post_concat(t),
                None => t,
            };
            s.pixmap.fill_path(&path, &fill, FillRule::Winding, t, clip);
        }
        let border = f(st, "border").unwrap_or(0.0);
        if border > 0.0 {
            let focused = node.get("focused").and_then(Value::as_bool) == Some(true);
            let color = if focused { node.get("focus_border").and_then(Value::as_str).and_then(parse_color) } else { None };
            if let Some(c) = color.or_else(|| over("border_color")) {
                let inset = Rect { x: r.x + border / 2.0, y: r.y + border / 2.0, w: r.w - border, h: r.h - border };
                if let Some(path) = rounded(inset, (radius - border / 2.0).max(0.0)) {
                    s.pixmap.stroke_path(&path, &solid(c), &Stroke { width: border, ..Stroke::default() }, t, clip);
                }
            }
        }
        let content = Rect {
            x: r.x + l.padding.left + l.border.left,
            y: r.y + l.padding.top + l.border.top,
            w: l.size.width - l.padding.left - l.padding.right - l.border.left - l.border.right,
            h: l.size.height - l.padding.top - l.padding.bottom - l.border.top - l.border.bottom,
        };
        self.interactive(node, r);
        match kind(node) {
            "text" => {
                let spec = TextSpec::of(node, self.fonts, s);
                self.fonts.draw(s.pixmap, scale, &spec, content, clip, false);
                return;
            }
            "image" => {
                self.image(node, s, r, radius, clip);
                return;
            }
            "input" => {
                let value = node.get("value").and_then(Value::as_str).unwrap_or("");
                let mut spec = TextSpec::of(node, self.fonts, s);
                spec.text = value.to_string();
                let focused = node.get("focused").and_then(Value::as_bool) == Some(true);
                self.fonts.draw(s.pixmap, scale, &spec, content, clip, focused);
            }
            "node" | "button" => {}
            _ => {
                atelico_component::paint(node, &mut Surface { pixmap: &mut *s.pixmap, scale, rect: r, clip, assets: s.assets, time: s.time });
            }
        }
        let own;
        let clip = if b(st, "clip") || b(st, "scroll") {
            own = self.mask(s.pixmap, scale, r, radius, clip);
            own.as_ref().or(clip)
        } else {
            clip
        };
        let scroll = if b(st, "scroll") { f(st, "scroll_y").unwrap_or(0.0).clamp(0.0, (l.content_size.height - l.size.height).max(0.0)) } else { 0.0 };
        let ids = self.tree.children(id).unwrap_or_default();
        for (child, cid) in children(node).iter().zip(ids) {
            self.node(child, cid, s, r.x, r.y - scroll, clip);
        }
    }

    fn mask(&self, pm: &Pixmap, scale: f32, r: Rect, radius: f32, parent: Option<&Mask>) -> Option<Mask> {
        let path = rounded(r, radius)?;
        let t = Transform::from_scale(scale, scale);
        match parent {
            Some(p) => {
                let mut m = p.clone();
                m.intersect_path(&path, FillRule::Winding, true, t);
                Some(m)
            }
            None => {
                let mut m = Mask::new(pm.width(), pm.height())?;
                m.fill_path(&path, FillRule::Winding, true, t);
                Some(m)
            }
        }
    }

    fn interactive(&mut self, node: &Value, r: Rect) {
        let disabled = node.get("disabled").and_then(Value::as_bool) == Some(true);
        let target = if let Some(a) = node.get("on_press").filter(|a| a.is_object() && !disabled) {
            json!({ "kind": "press", "action": a })
        } else if let Some(a) = node.get("on_pointer").filter(|a| a.is_object()) {
            json!({ "kind": "pointer", "action": a })
        } else if kind(node) == "input" {
            let bind = node.get("bind").or(node.get("id")).cloned().unwrap_or(Value::Null);
            json!({ "kind": "input", "bind": bind, "on_submit": node.get("on_submit").cloned().unwrap_or(Value::Null) })
        } else {
            return;
        };
        if r.w > 0.0 && r.h > 0.0 {
            self.out.hits.push(Hit { rect: r, target });
        }
    }

    fn image(&mut self, node: &Value, s: &mut Surface, r: Rect, radius: f32, clip: Option<&Mask>) {
        let Some((path, src)) = image(node, s) else { return };
        let scale = s.scale;
        let own = if radius > 0.0 { self.mask(s.pixmap, scale, r, radius, clip) } else { None };
        let clip = own.as_ref().or(clip);
        IMAGES.with_borrow(|images| {
            let Some(Some(img)) = images.get(&path) else { return };
            let (sx, sy) = (r.w / src.w, r.h / src.h);
            let t = Transform::from_translate(-src.x, -src.y).post_scale(sx, sy).post_translate(r.x, r.y);
            let pattern = Pattern::new(img.as_ref(), SpreadMode::Pad, FilterQuality::Bilinear, 1.0, t);
            let paint = Paint { shader: pattern, anti_alias: true, ..Paint::default() };
            if let Some(rect) = tiny_skia::Rect::from_xywh(r.x, r.y, r.w, r.h) {
                s.pixmap.fill_rect(rect, &paint, Transform::from_scale(scale, scale), clip);
            }
        });
    }
}

/// `gradient = {"#rrggbbaa", ...}` spread evenly along `gradient_dir` ("x", the default, or "y").
fn gradient(st: &Value, r: Rect) -> Option<Shader<'static>> {
    let colors: Vec<Color> = st.get("gradient")?.as_array()?.iter().filter_map(|c| parse_color(c.as_str()?)).collect();
    let n = colors.len().checked_sub(1).filter(|n| *n > 0)?;
    let stops = colors.into_iter().enumerate().map(|(i, c)| GradientStop::new(i as f32 / n as f32, c)).collect();
    let end = if s(st, "gradient_dir") == Some("y") { SkPoint::from_xy(r.x, r.y + r.h) } else { SkPoint::from_xy(r.x + r.w, r.y) };
    LinearGradient::new(SkPoint::from_xy(r.x, r.y), end, stops, SpreadMode::Pad, Transform::identity())
}

fn s_color(st: &Value, k: &str) -> Option<Color> {
    parse_color(st.get(k)?.as_str()?)
}

#[cfg(test)]
mod tests;
