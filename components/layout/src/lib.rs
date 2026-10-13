//! Flexbox layout (taffy) and painting of ui.* trees, plus hit testing for what it painted.
mod cache;
mod centring;
mod text;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;

use atelico_component::input::ScrollRegion;
use atelico_component::tiny_skia::{Color, Pixmap, Transform};
use atelico_component::{parse_color, register, Component, Hit, Rect, Surface, Value};
use atelico_paint::{Canvas, Image, Paint as Fill, Stroke};
use atelico_paint_cpu::CpuCanvas;
use serde_json::json;
use taffy::prelude::*;
use taffy::{Overflow, Point};

use text::{Fonts, TextSpec};

register!(Component { name: "layout", paint, hit, clear_layer, inspect, ..Component::NONE });

const KINDS: [&str; 5] = ["node", "text", "button", "input", "image"];

#[derive(Default)]
struct Painted {
    hits: Vec<Hit>,
    boxes: Vec<(String, Rect)>,
    scrolls: Vec<ScrollRegion>,
    /// What does not fit: a node or a text line reaching past its parent's content box where nothing
    /// scrolls (vertically) or scales it to fit. (where: the nearest id, then the kind; by: left, top, right, bottom)
    overflow: Vec<(String, [f32; 4])>,
    /// Every text-in-a-box and how far its ink is from the box's centre (see `centring`).
    centring: Vec<centring::Centring>,
    /// Texts set smaller than `SMALLEST` px: (where, text, face, px).
    small: Vec<(String, String, String, f32)>,
    /// Texts cut short to fit, ending in "…": (where, the whole text).
    elided: Vec<(String, String)>,
    /// Texts drawn with a template's `{name}` still in them, never filled: (where, the text).
    unfilled: Vec<(String, String)>,
    /// The cached layers still settling (a zoom in progress, shown from a picture drawn at another scale), by id:
    /// each asks to be painted again once the zoom holds (`cache::SETTLE`).
    settle: Vec<String>,
}

/// A `{name}` placeholder left in a text (a word of letters, digits and _ between braces).
pub fn placeholder(text: &str) -> bool {
    let mut rest = text;
    while let Some(i) = rest.find('{') {
        rest = &rest[i + 1..];
        if let Some(j) = rest.find('}') {
            let name = &rest[..j];
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return true;
            }
        } else {
            break;
        }
    }
    false
}

/// The smallest text anyone should have to read, in logical px (1080p at UI scale 1).
pub const SMALLEST: f32 = 21.0;

/// What each layer (see `atelico_component::layer`) last painted.
static PAINTED: Mutex<BTreeMap<u32, Painted>> = Mutex::new(BTreeMap::new());

thread_local! {
    static FONTS: RefCell<Fonts> = RefCell::new(Fonts::default());
    static IMAGES: RefCell<HashMap<PathBuf, CachedImage>> = RefCell::new(HashMap::new());
    /// Nodes (by address in the tree being painted) whose `breakpoints` apply at their laid-out width.
    static BROKEN: RefCell<HashMap<usize, usize>> = RefCell::new(HashMap::new());
    /// The last layouts, by layer and the rect and scale they were made for (a layer can paint more than one tree a
    /// frame, each in its own rect: a scene's picture and its HUD drawn crisp over it): the taffy tree and its root,
    /// and each taffy node's own fingerprint (its node's fields but its children: `own`), so the next frame changes
    /// only the nodes that changed (`relay`).
    static LAID: RefCell<HashMap<(u32, [u32; 5]), Laid>> = RefCell::new(HashMap::new());
}

/// A document can replace an image at the same path. Its content revision invalidates both the
/// decoded pixels and the backend upload ID; unchanged revisions keep their cached upload.
struct CachedImage {
    revision: String,
    image: Option<Image>,
}

enum Leaf {
    Text(TextSpec),
    Image(f32, f32),
}

struct Laid {
    /// When it was last used (`USES`): the least recently used go once there are more than `KEEP`.
    used: u64,
    tree: TaffyTree<Leaf>,
    root: NodeId,
    own: HashMap<NodeId, u64>,
}

// Retire only what was painted. Geometry/font/image caches remain reusable, and other
// windows keep their own metadata until their complete layer is replaced.
fn clear_layer(layer: u32) {
    PAINTED.lock().unwrap().remove(&layer);
}

fn hit(x: f32, y: f32) -> Option<Hit> {
    PAINTED.lock().unwrap().get(&atelico_component::layer())?.hits.iter().rev().find(|h| h.rect.contains(x, y)).cloned()
}

/// `{layers = {[layer] = {{id, rect}}}, overflow = {[layer] = {{at, by}}}, centring = {[layer] = {{at, text,
/// content, ink, by, limit, ok}}}}`: laid-out boxes with ids, in each layer's logical coordinates, what did not fit,
/// how centred each text-in-a-box is (`by` and `limit` in frame pixels, null on an axis not meant centred), and
/// `small = {[layer] = {{at, text, font, px}}}`: texts set under `SMALLEST` px.
fn inspect() -> Value {
    let p = PAINTED.lock().unwrap();
    let rect = |r: &Rect| json!([r.x, r.y, r.w, r.h]);
    let layers: serde_json::Map<String, Value> = p.iter().map(|(l, p)| (l.to_string(), p.boxes.iter().map(|(id, r)| json!({ "id": id, "rect": rect(r) })).collect())).collect();
    let overflow: serde_json::Map<String, Value> = p.iter().map(|(l, p)| (l.to_string(), p.overflow.iter().map(|(at, by)| json!({ "at": at, "by": by })).collect())).collect();
    let centring: serde_json::Map<String, Value> = p
        .iter()
        .map(|(l, p)| {
            let all = p.centring.iter().map(|c| json!({ "at": c.at, "text": c.text, "font": c.font, "content": rect(&c.content), "ink": rect(&c.ink), "by": c.by, "limit": c.limit, "ok": c.ok() }));
            (l.to_string(), all.collect())
        })
        .collect();
    let small: serde_json::Map<String, Value> = p.iter().map(|(l, p)| (l.to_string(), p.small.iter().map(|(at, text, font, px)| json!({ "at": at, "text": text, "font": font, "px": px })).collect())).collect();
    let elided: serde_json::Map<String, Value> = p.iter().map(|(l, p)| (l.to_string(), p.elided.iter().map(|(at, text)| json!({ "at": at, "text": text })).collect())).collect();
    let unfilled: serde_json::Map<String, Value> = p.iter().map(|(l, p)| (l.to_string(), p.unfilled.iter().map(|(at, text)| json!({ "at": at, "text": text })).collect())).collect();
    let settle: serde_json::Map<String, Value> = p.iter().filter(|(_, p)| !p.settle.is_empty()).map(|(l, p)| (l.to_string(), p.settle.iter().map(|id| json!(id)).collect())).collect();
    let scrolls: BTreeMap<_, _> = p.iter().map(|(l, p)| (l.to_string(), &p.scrolls)).collect();
    json!({ "layers": layers, "scrolls": scrolls, "overflow": overflow, "centring": centring, "small": small, "elided": elided, "unfilled": unfilled, "settle": settle })
}

/// Where a tree is laid out and painted: its rect (logical px), the device scale, where relative asset paths resolve,
/// and the app's time.
#[derive(Clone, Copy)]
pub struct Place<'a> {
    pub rect: Rect,
    pub scale: f32,
    pub assets: &'a [PathBuf],
    pub time: f64,
}

/// Lays out `node` in the place's rect; returns the tree and its root.
fn lay_out(node: &Value, s: &Place) -> (TaffyTree<Leaf>, NodeId) {
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
    (tree, root)
}

/// Which of a node's `breakpoints` = {{below = <width>, style = {...}}, ...} applies at its laid-out width
/// (the narrowest one it is below), for every node that has them; true when that changed anything.
fn breakpoints(node: &Value, tree: &TaffyTree<Leaf>, id: NodeId, found: &mut HashMap<usize, usize>) {
    if let (Some(bps), Ok(l)) = (style_of(node).get("breakpoints").and_then(Value::as_array), tree.layout(id)) {
        let pick = bps.iter().enumerate().filter(|(_, bp)| f(bp, "below").is_some_and(|w| l.size.width < w)).min_by(|a, b| f(a.1, "below").partial_cmp(&f(b.1, "below")).unwrap_or(std::cmp::Ordering::Equal));
        if let Some((i, _)) = pick {
            found.insert(node as *const Value as usize, i);
        }
    }
    let ids = tree.children(id).unwrap_or_default();
    for (c, cid) in children(node).iter().zip(ids) {
        breakpoints(c, tree, cid, found);
    }
}

fn paint(node: &Value, s: &mut Surface) -> bool {
    let place = Place { rect: s.rect, scale: s.scale, assets: s.assets, time: s.time };
    // (on the backend the host chose for this window; tiny-skia unless it chose another)
    match atelico_paint::canvas_over() {
        Some(over) => paint_on(node, &place, &mut *over(&mut *s.pixmap, s.scale)),
        None => paint_on(node, &place, &mut CpuCanvas::new(&mut *s.pixmap, s.scale, None)),
    }
}

/// Lays out `node` at `s` and paints it on `c` (any backend's canvas; the component's own `paint` uses the CPU one);
/// false when it is not a layout node.
pub fn paint_on(node: &Value, s: &Place, c: &mut dyn Canvas) -> bool {
    if !KINDS.contains(&kind(node)) {
        return false;
    }
    // the last layout of this layer, in the same rect, changed where the tree changed: a node whose own fields are the
    // same keeps its style and measure, so taffy lays out again only what a change reaches (a HUD's moving dial, a
    // clock's words); a tree of another shape (nodes added, removed, of another kind) is laid out afresh
    let layer = atelico_component::layer();
    let place = [s.rect.x, s.rect.y, s.rect.w, s.rect.h, s.scale].map(f32::to_bits);
    let kept = LAID.with_borrow_mut(|laid| laid.remove(&(layer, place)));
    let relaid = kept.and_then(|mut l| {
        let mut own = HashMap::with_capacity(l.own.len());
        let changed = FONTS.with_borrow_mut(|fonts| relay(&mut l.tree, node, l.root, &l.own, &mut own, fonts, s, true))?;
        if changed {
            let available = Size { width: AvailableSpace::Definite(s.rect.w), height: AvailableSpace::Definite(s.rect.h) };
            FONTS.with_borrow_mut(|fonts| {
                let _ = l.tree.compute_layout_with_measure(l.root, available, |known, avail, _, leaf, _| measure(fonts, known, avail, leaf));
            });
            // (a breakpoint that now applies: laid out afresh, with it)
            let mut found = HashMap::new();
            breakpoints(node, &l.tree, l.root, &mut found);
            if !found.is_empty() {
                return None;
            }
        }
        l.own = own;
        Some(l)
    });
    BROKEN.with_borrow_mut(|b| b.clear());
    let (tree, root, own, reusable) = match relaid {
        Some(l) => (l.tree, l.root, l.own, true),
        None => {
            let (mut tree, mut root) = lay_out(node, s);
            // breakpoints: a second pass with the overrides that apply at the widths the first pass gave
            let mut found = HashMap::new();
            breakpoints(node, &tree, root, &mut found);
            let none = found.is_empty();
            if !none {
                BROKEN.with_borrow_mut(|b| *b = found);
                (tree, root) = lay_out(node, s);
            }
            let mut own = HashMap::new();
            if none {
                prints(node, &tree, root, &mut own);
            }
            (tree, root, own, none)
        }
    };
    let mut out = Painted::default();
    let (frame, assets, time) = (s.rect, s.assets, s.time);
    FONTS.with_borrow_mut(|fonts| {
        let mut p = Painter { tree: &tree, fonts, out: &mut out, at: vec![String::from("root")], zoom: 1.0, view: None, frame, rounded: false, assets, time };
        p.node(node, root, c, frame.x, frame.y);
    });
    PAINTED.lock().unwrap().insert(layer, out);
    if reusable {
        static USES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        const KEEP: usize = 16;
        let used = USES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        LAID.with_borrow_mut(|laid| {
            laid.insert((layer, place), Laid { used, tree, root, own });
            if laid.len() > KEEP {
                if let Some(old) = laid.iter().min_by_key(|(_, l)| l.used).map(|(k, _)| *k) {
                    laid.remove(&old);
                }
            }
        });
    }
    true
}

/// A node's own fingerprint: its fields but its children (and its UI address state), as they decide its style and
/// measure.
fn own_print(node: &Value) -> u64 {
    use std::hash::{Hash, Hasher};
    fn go(v: &Value, h: &mut std::collections::hash_map::DefaultHasher) {
        match v {
            Value::Null => 0u8.hash(h),
            Value::Bool(b) => b.hash(h),
            Value::Number(n) => n.as_f64().map(f64::to_bits).hash(h),
            Value::String(s) => s.hash(h),
            Value::Array(a) => {
                a.len().hash(h);
                a.iter().for_each(|x| go(x, h));
            }
            Value::Object(o) => {
                o.len().hash(h);
                for (k, x) in o {
                    k.hash(h);
                    go(x, h);
                }
            }
        }
    }
    let mut h = std::collections::hash_map::DefaultHasher::new();
    match node {
        Value::Object(o) => {
            for (k, x) in o.iter().filter(|(k, _)| k.as_str() != "children" && k.as_str() != "ui_state") {
                k.hash(&mut h);
                go(x, &mut h);
            }
        }
        other => go(other, &mut h),
    }
    h.finish()
}

/// Every taffy node's own fingerprint, for a tree just laid out.
fn prints(node: &Value, tree: &TaffyTree<Leaf>, id: NodeId, own: &mut HashMap<NodeId, u64>) {
    own.insert(id, own_print(node));
    let ids = tree.children(id).unwrap_or_default();
    for (c, cid) in children(node).iter().zip(ids) {
        prints(c, tree, cid, own);
    }
}

/// Brings a kept layout up to `node`: each node whose own fingerprint changed gets its style and measure again
/// (taffy marks it and its ancestors to be laid out again; the rest keep their layout). None when the tree has another
/// shape than the kept one; else whether anything changed.
#[allow(clippy::too_many_arguments)]
fn relay(tree: &mut TaffyTree<Leaf>, node: &Value, id: NodeId, was: &HashMap<NodeId, u64>, now: &mut HashMap<NodeId, u64>, fonts: &mut Fonts, s: &Place, root: bool) -> Option<bool> {
    let print = own_print(node);
    now.insert(id, print);
    let kids = children(node);
    let ids = tree.children(id).unwrap_or_default();
    let leaf = matches!(kind(node), "text" | "image");
    if (leaf && !ids.is_empty()) || (!leaf && kids.len() != ids.len()) || leaf != tree.get_node_context(id).is_some() {
        return None;
    }
    let mut changed = false;
    if was.get(&id) != Some(&print) {
        let mut st = style(node);
        if root {
            // (as lay_out sizes the root: the rect, where the tree does not size it)
            if st.size.width.is_auto() {
                st.size.width = length(s.rect.w);
            }
            if st.size.height.is_auto() {
                st.size.height = length(s.rect.h);
            }
        }
        if tree.style(id).ok() != Some(&st) {
            let _ = tree.set_style(id, st);
            changed = true;
        }
        match kind(node) {
            "text" => {
                let spec = TextSpec::of(node, fonts, s.assets);
                // the same room: the same words and face, or (a ticking clock, a countdown) new words that measure the
                // same, unwrapped, as the old ones did
                let same = matches!(tree.get_node_context(id), Some(Leaf::Text(t)) if t.measures_as(&spec) || (!t.wrap && !spec.wrap && t.same_face(&spec) && fonts.wrap(t, f32::INFINITY) .iter().map(|l| l.1.to_bits()).eq(fonts.wrap(&spec, f32::INFINITY).iter().map(|l| l.1.to_bits()))));
                if same {
                    // (the same room: the new colour or alignment set without laying out again)
                    if let Some(c) = tree.get_node_context_mut(id) {
                        *c = Leaf::Text(spec);
                    }
                } else {
                    let _ = tree.set_node_context(id, Some(Leaf::Text(spec)));
                    changed = true;
                }
            }
            "image" => {
                let (w, h) = image(node, s.assets).map_or((0.0, 0.0), |(_, r)| (r.w, r.h));
                if !matches!(tree.get_node_context(id), Some(Leaf::Image(a, b)) if *a == w && *b == h) {
                    let _ = tree.set_node_context(id, Some(Leaf::Image(w, h)));
                    changed = true;
                }
            }
            _ => {}
        }
    }
    for (i, (c, cid)) in kids.iter().zip(ids).enumerate() {
        match relay(tree, c, cid, was, now, fonts, s, false) {
            Some(ch) => changed |= ch,
            None => {
                // (another shape below here, a pixel label's new words: this child alone is built afresh in its place,
                // not the whole layer)
                let fresh = build(tree, c, fonts, s);
                let _ = tree.replace_child_at_index(id, i, fresh);
                remove_all(tree, cid);
                prints(c, tree, fresh, now);
                changed = true;
            }
        }
    }
    Some(changed)
}

/// Removes `id` and everything under it from `tree`.
fn remove_all(tree: &mut TaffyTree<Leaf>, id: NodeId) {
    for c in tree.children(id).unwrap_or_default() {
        remove_all(tree, c);
    }
    let _ = tree.remove(id);
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

/// A node's style with the breakpoint override that applies, if any.
fn effective_style(node: &Value) -> std::borrow::Cow<'_, Value> {
    let st = style_of(node);
    let Some(i) = BROKEN.with_borrow(|b| b.get(&(node as *const Value as usize)).copied()) else { return std::borrow::Cow::Borrowed(st) };
    let mut merged = st.clone();
    if let (Some(m), Some(over)) = (merged.as_object_mut(), st["breakpoints"][i]["style"].as_object()) {
        for (k, v) in over {
            m.insert(k.clone(), v.clone());
        }
    }
    std::borrow::Cow::Owned(merged)
}

fn style(node: &Value) -> Style {
    let st = &*effective_style(node);
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

fn build(tree: &mut TaffyTree<Leaf>, node: &Value, fonts: &mut Fonts, s: &Place) -> NodeId {
    let st = style(node);
    match kind(node) {
        "text" => {
            let spec = TextSpec::of(node, fonts, s.assets);
            tree.new_leaf_with_context(st, Leaf::Text(spec)).expect("leaf")
        }
        "image" => {
            let (w, h) = image(node, s.assets).map_or((0.0, 0.0), |(_, r)| (r.w, r.h));
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
fn image(node: &Value, assets: &[PathBuf]) -> Option<(PathBuf, Rect)> {
    let path = atelico_component::asset(assets, node.get("texture")?.as_str()?)?;
    let revision = node.get("texture_revision").and_then(Value::as_str).unwrap_or("");
    let (w, h) = IMAGES.with_borrow_mut(|images| {
        if images.get(&path).is_none_or(|cached| cached.revision != revision) {
            let image = std::fs::read(&path).ok().and_then(|bytes| decode_image(&bytes));
            images.insert(path.clone(), CachedImage { revision: revision.into(), image });
        }
        images.get(&path)?.image.as_ref().map(|p| (p.width as f32, p.height as f32))
    })?;
    let atlas: Vec<f32> = node.get("atlas").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_f64).map(|v| v as f32).collect()).unwrap_or_default();
    let r = if atlas.len() == 4 { Rect { x: atlas[0], y: atlas[1], w: atlas[2] - atlas[0], h: atlas[3] - atlas[1] } } else { Rect { x: 0.0, y: 0.0, w, h } };
    Some((path, r))
}

/// Keep PNG on the existing tiny-skia decoder: its premultiplied bytes are part of the
/// renderer contract. JPEG is opaque, so decoded RGBA is already valid premultiplied data.
/// Explicit format and default decoder limits avoid enabling unrelated image formats here.
fn decode_image(bytes: &[u8]) -> Option<Image> {
    if let Ok(pm) = Pixmap::decode_png(bytes) {
        return Some(Image { id: images_read(), width: pm.width(), height: pm.height(), data: pm.data().into() });
    }
    if !bytes.starts_with(&[0xff, 0xd8]) {
        return None;
    }
    let rgba = image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Jpeg)
        .decode().ok()?.into_rgba8();
    Some(Image { id: images_read(), width: rgba.width(), height: rgba.height(), data: rgba.into_raw().into() })
}

/// A fresh id for each image read.
fn images_read() -> u64 {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

struct Painter<'a> {
    tree: &'a TaffyTree<Leaf>,
    fonts: &'a mut Fonts,
    out: &'a mut Painted,
    /// The ids of the nodes being painted, outermost first: where an overflow is.
    at: Vec<String>,
    /// The zoom of the canvases being painted (their `zoom` styles multiplied).
    zoom: f32,
    /// What the clipping and scrolling ancestors leave in view (None: the whole frame); a text partly out of it is
    /// not measured for centring.
    view: Option<Rect>,
    /// The whole layer: where a declared overlay (`overlay = true`) must stay.
    frame: Rect,
    /// A clipping ancestor has round corners: its clip is not just `view`'s rectangle.
    rounded: bool,
    /// Where relative asset paths resolve, and the app's time (for the components it hands nodes to).
    assets: &'a [PathBuf],
    time: f64,
}

impl Painter<'_> {
    fn node(&mut self, node: &Value, id: NodeId, c: &mut dyn Canvas, px: f32, py: f32) {
        let Ok(l) = self.tree.layout(id) else { return };
        let st = &*effective_style(node);
        if b(st, "hidden") {
            return;
        }
        let r = Rect { x: px + l.location.x, y: py + l.location.y, w: l.size.width, h: l.size.height };
        let named = node.get("id").and_then(Value::as_str);
        if let Some(id) = named {
            self.out.boxes.push((id.to_string(), r));
        }
        let scale = c.scale();
        let hovered = node.get("hover").filter(|h| h.is_object()).filter(|_| atelico_component::pointer().is_some_and(|(x, y)| r.contains(x, y)));
        let over = |k: &str| hovered.and_then(|h| s_color(h, k)).or_else(|| s_color(st, k));
        let radius = f(st, "radius").unwrap_or(0.0);
        let fill = match (gradient(st, r), over("background")) {
            (Some(g), _) => Some(g),
            (None, Some(bg)) => Some(Fill::Solid(bg)),
            _ => None,
        };
        let square = radius <= 0.0 && f(st, "rotation").is_none();
        if let (Some(fill), true) = (fill.as_ref(), square) {
            // (a plain box filled as a rectangle: a path's scan of a panel-sized box costs far more for the same pixels)
            c.fill_box(r, fill);
        } else if let Some(fill) = fill {
            let t = match f(st, "rotation") {
                Some(a) => Transform::from_rotate_at(a.to_degrees(), r.x + r.w / 2.0, r.y + r.h / 2.0),
                None => Transform::identity(),
            };
            c.fill_rounded(r, radius, &fill, t);
        }
        let border = f(st, "border").unwrap_or(0.0);
        if border > 0.0 {
            let focused = node.get("focused").and_then(Value::as_bool) == Some(true);
            let color = if focused { node.get("focus_border").and_then(Value::as_str).and_then(parse_color) } else { None };
            if let Some(color) = color.or_else(|| over("border_color")) {
                let inset = Rect { x: r.x + border / 2.0, y: r.y + border / 2.0, w: r.w - border, h: r.h - border };
                c.stroke_rounded(inset, (radius - border / 2.0).max(0.0), &Fill::Solid(color), &Stroke { width: border, ..Stroke::default() });
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
                let mut spec = TextSpec::of(node, self.fonts, self.assets);
                self.size(node, &spec);
                if placeholder(&spec.text) && r.w > 0.5 && r.h > 0.5 {
                    self.out.unfilled.push((named.unwrap_or(self.at.last().map_or("", String::as_str)).to_string(), spec.text.clone()));
                }
                if spec.ellipsis && !spec.wrap {
                    let cut = spec.text.split('\n').map(|l| self.fonts.ellipsize(&spec, l, content.w)).collect::<Vec<_>>().join("\n");
                    // (only what can be seen: a line scrolled away or in a collapsed panel is not read)
                    let seen = r.w > 0.5 && r.h > 0.5 && self.view.is_none_or(|v| r.x < v.x + v.w && r.x + r.w > v.x && r.y < v.y + v.h && r.y + r.h > v.y);
                    if cut != spec.text && seen {
                        self.out.elided.push((named.unwrap_or(self.at.last().map_or("", String::as_str)).to_string(), spec.text.clone()));
                    }
                    // a line cut to fit fills its box: set in its middle, as a line that fits exactly would be
                    spec.centre_cut(cut);
                }
                // a line wider than its box spills out (or is cut off)
                let widest = self.fonts.wrap(&spec, if spec.wrap { content.w } else { f32::INFINITY }).iter().map(|l| l.1).fold(0.0, f32::max);
                if widest > content.w + 1.5 {
                    let text = spec.text.chars().take(24).collect::<String>();
                    self.out.overflow.push((format!("{} > text {text:?}", named.unwrap_or(self.at.last().map_or("", String::as_str))), [0.0, 0.0, widest - content.w, 0.0]));
                }
                self.fonts.draw(c, &spec, content, false);
                return;
            }
            "image" => {
                self.image(node, c, r, radius);
                return;
            }
            "input" => {
                let value = node.get("value").and_then(Value::as_str).unwrap_or("");
                let mut spec = TextSpec::of(node, self.fonts, self.assets);
                spec.text = value.to_string();
                self.size(node, &spec);
                spec.editable = true;
                let focused = node.get("focused").and_then(Value::as_bool) == Some(true);
                self.fonts.draw(c, &spec, content, focused);
            }
            "node" | "button" => {}
            _ => {
                // (a component draws into the backend's picture through the clip: not yet through the canvas)
                // (through the canvas when the component draws that way; else on the backend's CPU picture)
                if !atelico_component::draw(node, &mut atelico_component::On { canvas: &mut *c, rect: r, assets: self.assets, time: self.time }) {
                    if let Some((pixmap, clip)) = c.legacy(r) {
                        atelico_component::paint(node, &mut Surface { pixmap, scale, rect: r, clip, assets: self.assets, time: self.time });
                    }
                }
            }
        }
        let clips = b(st, "clip") || b(st, "scroll");
        if clips {
            c.push_clip(r, radius);
        }
        // scroll_to = "end" stays at the bottom; scroll_to = <id> keeps that descendant a quarter of the way down
        // the view, where it was laid out (what leads to it shows above it); else scroll_y
        let most = (l.content_size.height - l.size.height).max(0.0);
        let scroll = if !b(st, "scroll") {
            0.0
        } else {
            let wanted = match st.get("scroll_to").and_then(Value::as_str) {
                Some("end") => Some(most),
                Some(want) => laid_out(self.tree, node, id, want).map(|(y, _)| y - l.size.height / 4.0),
                None => None,
            };
            wanted.or(f(st, "scroll_y")).unwrap_or(0.0).clamp(0.0, most)
        };
        if let Some(id) = named.filter(|_| b(st, "scroll")) {
            let region = ScrollRegion { id: id.into(), rect: [r.x, r.y, r.w, r.h], offset: scroll,
                maximum: most, scale: 1.0, follow_end: st.get("scroll_to").and_then(Value::as_str) == Some("end") };
            if let Some(region) = region.clipped(self.view.unwrap_or(self.frame)) {
                self.out.scrolls.push(region);
            }
        }
        let ids = self.tree.children(id).unwrap_or_default();
        // fit = "contain": content bigger than the box is drawn scaled down to fit it (about the box's origin),
        // and what it records (boxes, hits) is scaled the same way
        let k = if st.get("fit").and_then(Value::as_str) == Some("contain") && l.content_size.width > 0.0 && l.content_size.height > 0.0 {
            (l.size.width / l.content_size.width).min(l.size.height / l.content_size.height).min(1.0)
        } else {
            1.0
        };
        // containment: what paints past this node's content box (a placed child: its padding box, as it is placed
        // in), unless it scrolls, is scaled to fit, is cut to this node's box, or is a declared overlay (which must
        // stay inside the whole layer instead, unless a clipping ancestor cuts it)
        if b(st, "overlay") && self.view.is_none() {
            let f = self.frame;
            let by = [f.x - r.x, f.y - r.y, r.x + r.w - f.x - f.w, r.y + r.h - f.y - f.h].map(|v| v.max(0.0));
            if by.iter().any(|v| *v > 1.5) {
                self.out.overflow.push((format!("{} (overlay, past the window)", named.unwrap_or(self.at.last().map_or("", String::as_str))), by));
            }
        }
        if k >= 1.0 {
            let scrolls = b(st, "scroll");
            let cuts = b(st, "clip") || scrolls;
            let inner = Rect { x: r.x + l.border.left, y: r.y + l.border.top, w: l.size.width - l.border.left - l.border.right, h: l.size.height - l.border.top - l.border.bottom };
            for (child, cid) in children(node).iter().zip(self.tree.children(id).unwrap_or_default()) {
                let cs = style_of(child);
                let Ok(cl) = self.tree.layout(cid) else { continue };
                let placed = b(cs, "absolute");
                if b(cs, "hidden") || b(cs, "overlay") || (placed && cuts) {
                    continue;
                }
                let bx = if placed { inner } else { content };
                let (cx, cy) = (r.x + cl.location.x, r.y + cl.location.y - if placed { 0.0 } else { scroll });
                let mut by = [bx.x - cx, bx.y - cy, cx + cl.size.width - bx.x - bx.w, cy + cl.size.height - bx.y - bx.h].map(|v| v.max(0.0));
                if scrolls {
                    (by[1], by[3]) = (0.0, 0.0);
                }
                if by.iter().any(|v| *v > 1.5) {
                    let what = child.get("id").and_then(Value::as_str).map(String::from).unwrap_or_else(|| format!("{} > {}", named.unwrap_or(self.at.last().map_or("", String::as_str)), kind(child)));
                    self.out.overflow.push((what, by));
                }
            }
        }
        if let Some(n) = named {
            self.at.push(n.to_string());
        }
        let zoom = self.zoom;
        if let Some(z) = f(st, "zoom").filter(|z| *z > 0.0) {
            self.zoom *= z;
        }
        let boxed = st.get("gradient").is_some() || over("background").is_some() || border > 0.0;
        let shown: Vec<&Value> = children(node).iter().filter(|c| !b(style_of(c), "hidden")).collect();
        // (a zoomed canvas places its boxes at fractions of a pixel: centring is checked where it is drawn 1:1)
        let in_view = self.view.is_none_or(|v| r.x >= v.x - 0.5 && r.y >= v.y - 0.5 && r.x + r.w <= v.x + v.w + 0.5 && r.y + r.h <= v.y + v.h + 0.5);
        let text_in_box = boxed && k >= 1.0 && self.zoom == 1.0 && in_view && shown.len() == 1 && kind(shown[0]) == "text" && !b(style_of(shown[0]), "absolute");
        // the children are seen through this node when it clips or scrolls
        let (view, rounded) = (self.view, self.rounded);
        if b(st, "clip") || b(st, "scroll") {
            self.rounded |= radius > 0.0;
            self.view = Some(match view {
                Some(v) => {
                    let (x0, y0) = (r.x.max(v.x), r.y.max(v.y));
                    Rect { x: x0, y: y0, w: ((r.x + r.w).min(v.x + v.w) - x0).max(0.0), h: ((r.y + r.h).min(v.y + v.h) - y0).max(0.0) }
                }
                None => r,
            });
        }
        let before = if text_in_box { c.pixels().and_then(|pm| centring::before(pm, r, scale)) } else { None };
        if let Some(name) = named.filter(|_| b(st, "cache")) {
            // a cached layer: its children from a picture of their own, laid in at its place and scale (cache.rs)
            cache::paint(self, node, id, c, r, name);
        } else if k < 1.0 {
            let (boxes, hits, scrolls) = (self.out.boxes.len(), self.out.hits.len(), self.out.scrolls.len());
            // (inside, boxes are in the scaled content's own units: no view to hold them to)
            let outer_view = self.view;
            self.view = None;
            let outer_frame = self.frame;
            self.frame = Rect { x: outer_frame.x / k, y: outer_frame.y / k, w: outer_frame.w / k, h: outer_frame.h / k };
            c.set_scale(scale * k);
            for (child, cid) in children(node).iter().zip(ids) {
                self.node(child, cid, c, r.x / k, (r.y - scroll) / k);
            }
            c.set_scale(scale);
            self.frame = outer_frame;
            let fix = |q: &mut Rect| *q = Rect { x: q.x * k, y: q.y * k, w: q.w * k, h: q.h * k };
            self.out.boxes[boxes..].iter_mut().for_each(|(_, q)| fix(q));
            self.out.hits[hits..].iter_mut().for_each(|h| fix(&mut h.rect));
            let regions: Vec<_> = self.out.scrolls.drain(scrolls..)
                .filter_map(|r| r.mapped(0.0, 0.0, k).clipped(outer_view.unwrap_or(self.frame))).collect();
            self.out.scrolls.extend(regions);
        } else {
            for (child, cid) in children(node).iter().zip(ids) {
                self.node(child, cid, c, r.x, r.y - scroll);
            }
        }
        if let Some(ink) = before.as_ref().and_then(|b| centring::ink(c.pixels()?, b)) {
            let spec = TextSpec::of(shown[0], self.fonts, self.assets);
            let lines = self.fonts.wrap(&spec, if spec.wrap { content.w } else { f32::INFINITY });
            let widest = lines.iter().map(|l| l.1).fold(0.0, f32::max);
            let centred = style_of(shown[0]).get("text_align").and_then(Value::as_str) == Some("center");
            let axes = centring::axes(st, content, (widest, lines.len() as f32 * spec.line_height()), centred);
            let at = shown[0].get("id").and_then(Value::as_str).or(named).or(self.at.last().map(String::as_str)).unwrap_or("").to_string();
            let font = style_of(shown[0]).get("font").and_then(Value::as_str).unwrap_or("").to_string();
            self.out.centring.push(centring::measure(at, spec.text.clone(), font, content, ink, scale, axes));
        }
        if clips {
            c.pop_clip();
        }
        (self.view, self.rounded) = (view, rounded);
        self.zoom = zoom;
        if named.is_some() {
            self.at.pop();
        }
    }

    /// Records text set under `SMALLEST` px. A view that zooms (fit = "contain", or `zoom` = the zoom its content
    /// is drawn at, as a canvas's is) shows its content smaller on purpose, as a map does: its text counts at its
    /// own size.
    fn size(&mut self, node: &Value, spec: &TextSpec) {
        let px = spec.size() / self.zoom;
        if px < SMALLEST - 0.01 && !spec.text.trim().is_empty() {
            let at = node.get("id").and_then(Value::as_str).or(self.at.last().map(String::as_str)).unwrap_or("").to_string();
            let font = style_of(node).get("font").and_then(Value::as_str).unwrap_or("").to_string();
            self.out.small.push((at, spec.text.chars().take(40).collect(), font, px));
        }
    }

    fn interactive(&mut self, node: &Value, r: Rect) {
        let disabled = node.get("disabled").and_then(Value::as_bool) == Some(true);
        let target = if let Some(a) = node.get("on_press").filter(|a| a.is_object() && !disabled) {
            json!({ "kind": "press", "action": a, "id": node.get("id").cloned().unwrap_or(Value::Null) })
        } else if let Some(a) = node.get("on_pointer").filter(|a| a.is_object()) {
            json!({ "kind": "pointer", "action": a })
        } else if kind(node) == "input" {
            let bind = node.get("bind").or(node.get("id")).cloned().unwrap_or(Value::Null);
            json!({ "kind": "input", "bind": bind, "value": node.get("value").cloned().unwrap_or(Value::Null), "on_submit": node.get("on_submit").cloned().unwrap_or(Value::Null) })
        } else {
            return;
        };
        // only what the clipping and scrolling ancestors leave in view takes the pointer: a row scrolled up under a
        // panel's header is not clicked through it
        let r = match self.view {
            Some(v) => {
                let (x0, y0) = (r.x.max(v.x), r.y.max(v.y));
                Rect { x: x0, y: y0, w: (r.x + r.w).min(v.x + v.w) - x0, h: (r.y + r.h).min(v.y + v.h) - y0 }
            }
            None => r,
        };
        if r.w > 0.0 && r.h > 0.0 {
            self.out.hits.push(Hit { rect: r, target });
        }
    }

    fn image(&mut self, node: &Value, c: &mut dyn Canvas, r: Rect, radius: f32) {
        let Some((path, src)) = image(node, self.assets) else { return };
        let Some(img) = IMAGES.with_borrow(|images| images.get(&path).and_then(|cached| cached.image.clone())) else { return };
        if radius > 0.0 {
            c.push_clip(r, radius);
        }
        c.image(&img, src, r);
        if radius > 0.0 {
            c.pop_clip();
        }
    }
}

/// The top and height of `node`'s descendant with id `want`, from the top of `node`'s box, as laid out.
fn laid_out(tree: &TaffyTree<Leaf>, node: &Value, id: NodeId, want: &str) -> Option<(f32, f32)> {
    for (c, cid) in children(node).iter().zip(tree.children(id).unwrap_or_default()) {
        let l = tree.layout(cid).ok()?;
        if c.get("id").and_then(Value::as_str) == Some(want) {
            return Some((l.location.y, l.size.height));
        }
        if let Some((y, h)) = laid_out(tree, c, cid, want) {
            return Some((l.location.y + y, h));
        }
    }
    None
}

/// `gradient = {"#rrggbbaa", ...}` spread evenly along `gradient_dir` ("x", the default, or "y").
fn gradient(st: &Value, r: Rect) -> Option<Fill> {
    let colors: Vec<Color> = st.get("gradient")?.as_array()?.iter().filter_map(|c| parse_color(c.as_str()?)).collect();
    let n = colors.len().checked_sub(1).filter(|n| *n > 0)?;
    let stops = colors.into_iter().enumerate().map(|(i, c)| (i as f32 / n as f32, c)).collect();
    let end = if s(st, "gradient_dir") == Some("y") { (r.x, r.y + r.h) } else { (r.x + r.w, r.y) };
    Some(Fill::Linear { start: (r.x, r.y), end, stops })
}

fn s_color(st: &Value, k: &str) -> Option<Color> {
    parse_color(st.get(k)?.as_str()?)
}

#[cfg(test)]
mod tests;
