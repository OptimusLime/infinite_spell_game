//! Layout in isolation: flex sizing, percent and absolute placement (the picker's handles), hidden nodes,
//! per-layer boxes and hits, gradients and text painted where laid out.
use atelico_component::tiny_skia::Pixmap;
use atelico_component::{Rect, Surface};
use serde_json::{json, Value};

use super::*;

/// The layer being painted is process-wide: one test at a time.
fn one() -> std::sync::MutexGuard<'static, ()> {
    static ONE: Mutex<()> = Mutex::new(());
    ONE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Paints `node` into a `w`x`h` pixmap on `layer`; returns the pixmap and the boxes by id.
fn lay(node: Value, w: f32, h: f32, layer: u32) -> (Pixmap, Vec<(String, Rect)>) {
    let mut pm = Pixmap::new(w as u32, h as u32).unwrap();
    atelico_component::set_layer(layer);
    let mut s = Surface { pixmap: &mut pm, scale: 1.0, rect: Rect { x: 0.0, y: 0.0, w, h }, clip: None, assets: &[], time: 0.0 };
    assert!(paint(&node, &mut s));
    let boxes = PAINTED.lock().unwrap()[&layer].boxes.clone();
    (pm, boxes)
}

fn rect(boxes: &[(String, Rect)], id: &str) -> Rect {
    boxes.iter().find(|(i, _)| i == id).unwrap_or_else(|| panic!("no {id}")).1
}

fn px(pm: &Pixmap, x: u32, y: u32) -> [u8; 3] {
    let p = pm.pixel(x, y).unwrap().demultiply();
    [p.red(), p.green(), p.blue()]
}

#[test]
fn flex_rows_grow_and_gap() {
    let _one = one();
    let (_, b) = lay(json!({ "kind": "node", "style": { "width": 300, "height": 40, "gap": 10, "padding": 5 }, "children": [
        { "kind": "node", "id": "a", "style": { "width": 50, "shrink": 0 } },
        { "kind": "node", "id": "b", "style": { "grow": 1, "width": 0 } },
        { "kind": "node", "id": "c", "style": { "grow": 1, "width": 0 } },
    ] }), 300.0, 40.0, 11);
    let (a, bb, c) = (rect(&b, "a"), rect(&b, "b"), rect(&b, "c"));
    assert_eq!((a.x, a.w), (5.0, 50.0));
    assert_eq!((bb.x, bb.w), (65.0, 110.0));
    assert_eq!((c.x, c.w), (185.0, 110.0));
    assert_eq!(a.h, 30.0, "stretched to the padded height");
}

#[test]
fn absolute_percent_places_a_handle_centre() {
    let _one = one();
    // the picker's hue handle: left = h/360 %, margin -w/2 → its centre sits at that fraction of the bar
    for f in [0.0f32, 0.25, 0.6167, 1.0] {
        let (_, b) = lay(json!({ "kind": "node", "id": "bar", "style": { "width": 400, "height": 14 }, "children": [
            { "kind": "node", "id": "handle", "style": { "absolute": true, "left": format!("{}%", f * 100.0), "top": -2, "margin_left": -9, "width": 18, "height": 18 } },
        ] }), 400.0, 20.0, 12);
        let h = rect(&b, "handle");
        assert!((h.x + h.w / 2.0 - 400.0 * f).abs() <= 0.5, "{f}: centre {} (layout rounds to whole pixels)", h.x + h.w / 2.0);
        assert_eq!(h.y, -2.0);
    }
}

#[test]
fn hidden_nodes_take_no_space_and_leave_no_box() {
    let _one = one();
    let (_, b) = lay(json!({ "kind": "node", "style": { "width": 100, "height": 100, "direction": "column" }, "children": [
        { "kind": "node", "id": "gone", "style": { "height": 40, "hidden": true } },
        { "kind": "node", "id": "kept", "style": { "height": 40 } },
    ] }), 100.0, 100.0, 13);
    assert!(!b.iter().any(|(i, _)| i == "gone"));
    assert_eq!(rect(&b, "kept").y, 0.0);
}

#[test]
fn layers_keep_their_own_boxes_and_hits() {
    let _one = one();
    let press = json!({ "event": "route", "name": "go" });
    lay(json!({ "kind": "button", "id": "one", "on_press": press, "style": { "width": 50, "height": 50 } }), 100.0, 100.0, 14);
    lay(json!({ "kind": "node", "id": "two", "style": { "width": 80, "height": 80 } }), 100.0, 100.0, 15);
    atelico_component::set_layer(14);
    assert_eq!(hit(10.0, 10.0).map(|h| h.target["kind"].clone()), Some(json!("press")));
    assert!(hit(70.0, 70.0).is_none());
    atelico_component::set_layer(15);
    assert!(hit(10.0, 10.0).is_none(), "layer 15 painted nothing interactive");
    let layers = inspect()["layers"].clone();
    assert_eq!(layers["14"][0]["id"], "one");
    assert_eq!(layers["15"][0]["id"], "two");
}

#[test]
fn backgrounds_and_gradients_land_in_their_rects() {
    let _one = one();
    let (pm, _) = lay(json!({ "kind": "node", "style": { "width": 200, "height": 100 }, "children": [
        { "kind": "node", "style": { "width": 100, "height": 100, "background": "#1f4fbf" } },
        { "kind": "node", "style": { "width": 100, "height": 100, "gradient": ["#ffffff", "#ff0000"] } },
    ] }), 200.0, 100.0, 16);
    assert_eq!(px(&pm, 50, 50), [0x1f, 0x4f, 0xbf]);
    let left = px(&pm, 101, 50);
    let right = px(&pm, 198, 50);
    assert!(left[1] > 240 && right[1] < 15 && right[0] > 240, "white → red: {left:?} {right:?}");
}

#[test]
fn text_and_input_values_are_painted() {
    let _one = one();
    let (pm, _) = lay(json!({ "kind": "node", "style": { "width": 200, "height": 60, "background": "#000000", "direction": "column" }, "children": [
        { "kind": "text", "text": "Hello", "style": { "font_size": 20, "color": "#ffffff" } },
        { "kind": "input", "value": "", "style": { "height": 26, "font_size": 20, "color": "#ffffff" } },
    ] }), 200.0, 60.0, 17);
    let lit = |y0: u32, y1: u32| (0..200).flat_map(|x| (y0..y1).map(move |y| (x, y))).filter(|(x, y)| px(&pm, *x, *y)[0] > 128).count();
    assert!(lit(0, 26) > 40, "text drawn");
    assert_eq!(lit(30, 56), 0, "empty input draws nothing");
}
