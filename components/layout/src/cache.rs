//! Cached layers. A node with `cache = true` and an `id` draws its children once into a picture of its own and lays
//! that picture in at its place each frame; `scale = <n>` draws its children at that scale. A whiteboard that pans
//! and zooms (its contents in board units, the pan as the node's place, the zoom as its scale) then costs a copy per
//! frame instead of painting every card again:
//!
//! - the picture is drawn again only when what is in the layer changes (its children, its own style apart from where
//!   it is and its scale), when the screen's scale changes, or when what shows reaches past what was drawn (a pan far
//!   enough: half a view of margin is drawn round what shows);
//! - a zoom is shown by drawing the picture larger or smaller, and the picture is drawn again at the new zoom once the
//!   zoom has not changed for `SETTLE` (the layer says where it is still settling, `settle` in the layout's inspect,
//!   so the editor paints it once more then);
//! - the boxes and hits of what is in the layer are kept with the picture and placed with it each frame, so its cards
//!   are still found and clicked.
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use atelico_component::{input::ScrollRegion, Hit, Rect, Value};
use atelico_paint::{Canvas, Picture};
use taffy::prelude::*;

use super::{children, f, style_of, Painter};

/// How long a zoom must hold before the picture is drawn again at it.
pub const SETTLE: Duration = Duration::from_millis(150);
/// The largest side of a layer's picture, in device pixels.
const MOST: f32 = 4096.0;
/// Style keys that place the layer or scale it (not what is in it).
const PLACING: [&str; 6] = ["left", "top", "right", "bottom", "scale", "cache"];

struct Layer {
    /// What the picture shows: a fingerprint of the layer's children and its own style apart from `PLACING`.
    content: u64,
    /// The screen's scale and the layer's `scale` it was drawn at.
    dev: f32,
    zoom: f32,
    /// What it covers, in the layer's own units (from its origin).
    area: Rect,
    picture: Picture,
    /// The picture's size in device pixels.
    size: (u32, u32),
    /// The boxes and hits of what is in it, in its own units.
    boxes: Vec<(String, Rect)>,
    hits: Vec<Hit>,
    scrolls: Vec<ScrollRegion>,
    /// The last scale it was shown at, and since when (a zoom settles).
    shown: f32,
    since: Instant,
}

thread_local! {
    static LAYERS: RefCell<HashMap<String, Layer>> = RefCell::new(HashMap::new());
}

fn content_of(node: &Value) -> u64 {
    // (hashed as it stands, not printed: printing a HUD panel's thousands of boxes every frame cost more than the
    // picture saved)
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
    let st = style_of(node);
    if let Some(m) = st.as_object() {
        for (k, v) in m {
            if !PLACING.contains(&k.as_str()) {
                k.hash(&mut h);
                go(v, &mut h);
            }
        }
    }
    for c in children(node) {
        go(c, &mut h);
    }
    h.finish()
}

/// How far past its own box a bounded layer's picture reaches (a frame's hard shadow, a label's outline).
const BOUNDED_MARGIN: f32 = 24.0;

fn inside(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x - 0.5 && inner.y >= outer.y - 0.5 && inner.x + inner.w <= outer.x + outer.w + 0.5 && inner.y + inner.h <= outer.y + outer.h + 0.5
}

/// Paints the layer `node` (laid out as `id`, at `r`, named `name`) from its picture, drawing the picture first when
/// it is missing or out of date.
pub(super) fn paint(p: &mut Painter, node: &Value, id: NodeId, c: &mut dyn Canvas, r: Rect, name: &str) {
    let st = style_of(node);
    let zoom = f(st, "scale").filter(|z| *z > 0.0).unwrap_or(1.0);
    let dev = c.scale();
    // what of it shows (the clipping ancestors' view, else the whole layer), in its own units
    let view = p.view.unwrap_or(p.frame);
    let mut shows = Rect { x: (view.x - r.x) / zoom, y: (view.y - r.y) / zoom, w: view.w / zoom, h: view.h / zoom };
    // `cache_bounded`: a layer the size of its own box (a HUD panel), not a board panned past: its picture covers its
    // box and a margin, not half a view round what shows
    let bounded = super::b(st, "cache_bounded");
    if bounded {
        let m = BOUNDED_MARGIN;
        let own = Rect { x: -m, y: -m, w: r.w / zoom + 2.0 * m, h: r.h / zoom + 2.0 * m };
        let (x0, y0) = (shows.x.max(own.x), shows.y.max(own.y));
        let (x1, y1) = ((shows.x + shows.w).min(own.x + own.w), (shows.y + shows.h).min(own.y + own.h));
        shows = Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
    }
    if shows.w <= 0.0 || shows.h <= 0.0 {
        return;
    }
    let content = content_of(node);
    let now = Instant::now();
    LAYERS.with_borrow_mut(|layers| {
        let old = layers.remove(name);
        let (shown, since) = match &old {
            Some(l) if l.shown == zoom => (l.shown, l.since),
            _ => (zoom, now),
        };
        let settled = now.duration_since(since) >= SETTLE;
        let draw = match &old {
            None => true,
            Some(l) => l.content != content || l.dev != dev || (l.zoom != zoom && settled) || !inside(l.area, shows) || !c.owns(&l.picture),
        };
        let mut layer = if draw {
            draw_layer(p, node, id, c, content, dev, zoom, shows, !bounded)
        } else {
            old.expect("kept")
        };
        layer.shown = shown;
        layer.since = since;
        show(p, c, &layer, r, zoom, view);
        if layer.zoom != zoom {
            // still zooming: drawn larger or smaller for now, and again at the zoom once it settles
            p.out.settle.push(name.to_string());
        }
        layers.insert(name.to_string(), layer);
    });
}

/// Draws the layer's children into a picture covering what shows and half a view round it (as far as `MOST` allows).
#[allow(clippy::too_many_arguments)]
fn draw_layer(p: &mut Painter, node: &Value, id: NodeId, c: &mut dyn Canvas, content: u64, dev: f32, zoom: f32, shows: Rect, margin: bool) -> Layer {
    let k = dev * zoom;
    let (mx, my) = if margin { ((shows.w * 0.5).min(((MOST / k) - shows.w).max(0.0) / 2.0), (shows.h * 0.5).min(((MOST / k) - shows.h).max(0.0) / 2.0)) } else { (0.0, 0.0) };
    let area = Rect { x: shows.x - mx, y: shows.y - my, w: (shows.w + 2.0 * mx).min(MOST / k), h: (shows.h + 2.0 * my).min(MOST / k) };
    let (pw, ph) = ((area.w * k).ceil().max(1.0) as u32, (area.h * k).ceil().max(1.0) as u32);
    // (clear: the layer's own background is painted as any node's, under the picture)
    // (drawn with no view or clip of the window's, from the area's corner; what it records is kept in its own units)
    let (view, frame) = (p.view, p.frame);
    let marks = (p.out.boxes.len(), p.out.hits.len(), p.out.overflow.len(), p.out.centring.len(), p.out.small.len(), p.out.elided.len(), p.out.unfilled.len(), p.out.settle.len(), p.out.scrolls.len());
    p.view = None;
    p.frame = Rect { x: -area.x, y: -area.y, w: area.w, h: area.h };
    let ids = p.tree.children(id).unwrap_or_default();
    let picture = c.picture(pw, ph, k, &mut |off| {
        for (child, cid) in children(node).iter().zip(ids.iter().copied()) {
            p.node(child, cid, off, -area.x, -area.y);
        }
    });
    (p.view, p.frame) = (view, frame);
    let own = |q: Rect| Rect { x: q.x + area.x, y: q.y + area.y, w: q.w, h: q.h };
    let boxes = p.out.boxes.drain(marks.0..).map(|(i, q)| (i, own(q))).collect();
    let hits = p.out.hits.drain(marks.1..).map(|h| Hit { rect: own(h.rect), target: h.target }).collect();
    let scrolls = p.out.scrolls.drain(marks.8..).map(|r| r.mapped(area.x, area.y, 1.0)).collect();
    p.out.overflow.truncate(marks.2);
    p.out.centring.truncate(marks.3);
    p.out.small.truncate(marks.4);
    p.out.elided.truncate(marks.5);
    p.out.unfilled.truncate(marks.6);
    p.out.settle.truncate(marks.7);
    Layer { content, dev, zoom, area, picture, size: (pw.max(1), ph.max(1)), boxes, hits, scrolls, shown: zoom, since: Instant::now() }
}

/// Lays the picture in at the layer's place `r` and scale `zoom`, inside `view` (and `clip`), and records the boxes
/// and hits of what is in it where they now are.
fn show(p: &mut Painter, c: &mut dyn Canvas, l: &Layer, r: Rect, zoom: f32, view: Rect) {
    let dev = c.scale();
    let (cw, ch) = c.size();
    let k = zoom / l.zoom;
    let (tx, ty) = ((r.x + l.area.x * zoom) * dev, (r.y + l.area.y * zoom) * dev);
    // (at its own scale on whole pixels, laid in pixel for pixel)
    let whole = (k - 1.0).abs() < 1e-4;
    let (tx, ty) = if whole { (tx.round(), ty.round()) } else { (tx, ty) };
    // (only what shows: the picture runs half a view past it on every side)
    let (pw, ph) = (l.size.0 as f32 * k, l.size.1 as f32 * k);
    let (x0, y0) = (tx.max(view.x * dev).max(0.0), ty.max(view.y * dev).max(0.0));
    let (x1, y1) = ((tx + pw).min((view.x + view.w) * dev).min(cw as f32), (ty + ph).min((view.y + view.h) * dev).min(ch as f32));
    // laid over what is there (the picture is clear where nothing is drawn), through the clip: pixel for pixel at its
    // own scale, nearest pixel while a zoom settles (it is drawn again sharp once it holds)
    c.draw_picture(&l.picture, (tx, ty), k, [x0.floor(), y0.floor(), x1.ceil(), y1.ceil()]);
    place(p, l, r, zoom);
}

/// The boxes and hits of what is in the layer, where it now is.
fn place(p: &mut Painter, l: &Layer, r: Rect, zoom: f32) {
    let placed = |q: Rect| Rect { x: r.x + q.x * zoom, y: r.y + q.y * zoom, w: q.w * zoom, h: q.h * zoom };
    p.out.boxes.extend(l.boxes.iter().map(|(i, q)| (i.clone(), placed(*q))));
    p.out.hits.extend(l.hits.iter().map(|h| Hit { rect: placed(h.rect), target: h.target.clone() }));
    p.out.scrolls.extend(l.scrolls.iter().cloned().filter_map(|region| region.mapped(r.x, r.y, zoom).clipped(p.view.unwrap_or(p.frame))));
}
