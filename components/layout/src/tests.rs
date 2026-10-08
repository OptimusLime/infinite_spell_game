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
fn changed_texture_revision_reloads_same_path_pixels_and_backend_upload() {
    let _one = one();
    let path = std::env::temp_dir().join(format!("atelico-layout-image-revision-{}.png", std::process::id()));
    let mut source = Pixmap::new(8, 8).unwrap();
    source.fill(Color::from_rgba8(255, 0, 0, 255));
    source.save_png(&path).unwrap();
    let mut node = json!({"kind":"image","texture":path,"texture_revision":"first","style":{"width":8,"height":8}});
    let (first, _) = lay(node.clone(), 8., 8., 95);
    assert_eq!(px(&first, 4, 4), [255, 0, 0]);
    let first_id = IMAGES.with_borrow(|cache| cache[&path].image.as_ref().unwrap().id);
    source.fill(Color::from_rgba8(0, 0, 255, 255));
    source.save_png(&path).unwrap();
    node["texture_revision"] = json!("second");
    let (second, _) = lay(node.clone(), 8., 8., 95);
    assert_eq!(px(&second, 4, 4), [0, 0, 255]);
    let second_id = IMAGES.with_borrow(|cache| cache[&path].image.as_ref().unwrap().id);
    assert_ne!(first_id, second_id, "GPU uploads must also see the new bytes");
    lay(node, 8., 8., 95);
    assert_eq!(second_id, IMAGES.with_borrow(|cache| cache[&path].image.as_ref().unwrap().id));
    std::fs::remove_file(&path).unwrap();
    IMAGES.with_borrow_mut(|cache| cache.remove(&path));
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

#[test]
fn fit_contain_scales_bigger_content_down_to_its_box_and_records_it_scaled() {
    let _one = one();
    // a 400-wide grid in a 200-wide box: drawn at half size, its boxes and hits halved too
    let (pm, b) = lay(json!({ "kind": "node", "style": { "width": 200, "height": 300, "fit": "contain" }, "children": [
        { "kind": "node", "id": "grid", "style": { "width": 400, "height": 400, "shrink": 0, "background": "#ff0000" }, "children": [
            { "kind": "button", "id": "tile", "on_press": { "event": "route", "name": "t" }, "style": { "width": 100, "height": 100, "background": "#00ff00" } },
        ] },
    ] }), 200.0, 300.0, 31);
    let g = rect(&b, "grid");
    assert_eq!((g.w, g.h), (200.0, 200.0), "{g:?}");
    assert_eq!(rect(&b, "tile").w, 50.0);
    assert_eq!(px(&pm, 190, 190), [255, 0, 0], "the scaled grid fills the box");
    assert_eq!(px(&pm, 190, 250), [0, 0, 0], "and no further");
    let hit = PAINTED.lock().unwrap()[&31].hits[0].rect;
    assert_eq!((hit.w, hit.h), (50.0, 50.0), "hits follow the drawing");
}

#[test]
fn content_that_fits_is_not_scaled() {
    let _one = one();
    let (_, b) = lay(json!({ "kind": "node", "style": { "width": 200, "height": 200, "fit": "contain" }, "children": [
        { "kind": "node", "id": "small", "style": { "width": 80, "height": 80 } },
    ] }), 200.0, 200.0, 32);
    assert_eq!(rect(&b, "small").w, 80.0);
}

#[test]
fn breakpoints_restyle_a_node_below_a_width() {
    let _one = one();
    let row = |w: f32| json!({ "kind": "node", "id": "row", "style": { "width": w, "gap": 0, "breakpoints": [{ "below": 300, "style": { "direction": "column" } }] }, "children": [
        { "kind": "node", "id": "a", "style": { "width": 100, "height": 20, "shrink": 0 } },
        { "kind": "node", "id": "b", "style": { "width": 100, "height": 20, "shrink": 0 } },
    ] });
    let (_, wide) = lay(row(400.0), 400.0, 100.0, 33);
    assert_eq!(rect(&wide, "b").y, rect(&wide, "a").y, "side by side at 400");
    let (_, narrow) = lay(row(250.0), 400.0, 100.0, 33);
    assert_eq!(rect(&narrow, "b").y, rect(&narrow, "a").y + 20.0, "stacked below 300");
}

#[test]
fn what_does_not_fit_is_recorded_unless_it_scrolls_fits_is_cut_or_is_an_overlay() {
    let _one = one();
    lay(json!({ "kind": "node", "id": "panel", "style": { "width": 100, "height": 100, "direction": "column" }, "children": [
        { "kind": "node", "id": "wide", "style": { "width": 180, "height": 10, "shrink": 0 } },
        { "kind": "node", "id": "list", "style": { "width": 100, "height": 40, "scroll": true, "direction": "column" }, "children": [
            { "kind": "node", "style": { "width": 100, "height": 300, "shrink": 0 } } ] },
        { "kind": "node", "style": { "width": 100, "height": 20, "fit": "contain" }, "children": [
            { "kind": "node", "style": { "width": 400, "height": 80, "shrink": 0 } } ] },
        { "kind": "node", "id": "badge", "style": { "absolute": true, "overlay": true, "left": 90, "top": 90, "width": 20, "height": 20 } },
        { "kind": "node", "id": "tag", "style": { "absolute": true, "left": 90, "top": 20, "width": 20, "height": 20 } },
        { "kind": "node", "style": { "width": 100, "height": 20, "clip": true }, "children": [
            { "kind": "node", "style": { "absolute": true, "left": 80, "top": 0, "width": 60, "height": 20 } } ] },
        { "kind": "text", "text": "a line far too long to fit in a hundred pixels", "style": { "text_wrap": "none", "font_size": 14, "width": 100, "shrink": 1 } },
    ] }), 200.0, 200.0, 34);
    let o = PAINTED.lock().unwrap()[&34].overflow.clone();
    let at: Vec<&str> = o.iter().map(|(a, _)| a.as_str()).collect();
    assert_eq!(o.len(), 3, "{o:?}");
    assert_eq!(at[0], "wide");
    assert_eq!(at[1], "tag", "a placed child is held to its parent unless it is a declared overlay or cut to it");
    assert!(at[2].starts_with("panel > text"), "{at:?}");
    assert_eq!(o[0].1, [0.0, 0.0, 80.0, 0.0]);
    assert_eq!(o[1].1, [0.0, 0.0, 10.0, 0.0]);
}

#[test]
fn ellipsis_cuts_a_long_line_to_fit_and_counts_as_fitting() {
    let _one = one();
    lay(json!({ "kind": "node", "id": "row", "style": { "width": 100, "height": 30 }, "children": [
        { "kind": "text", "text": "a line far too long to fit in a hundred pixels", "style": { "text_wrap": "none", "text_overflow": "ellipsis", "font_size": 14, "width": 100, "shrink": 1 } },
    ] }), 200.0, 100.0, 35);
    assert!(PAINTED.lock().unwrap()[&35].overflow.is_empty());
}

/// Every face the editor and the ui-kit set text in, at small to display sizes, in a hugging chip and in a fixed
/// button that centres its label: the ink (the pixels painted) sits in the middle of the box, within a pixel or 3%.
#[test]
fn text_ink_is_centred_in_its_box_in_every_face_and_size() {
    let _one = one();
    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let roots = [here.join("../../editor"), here.join("../ui-kit/lua")];
    let faces = ["fonts/Roobert-Regular.ttf", "fonts/Roobert-SemiBold.ttf", "fonts/AtelicoTypeKern-Black.ttf", "themes/atelico/atelico-type.ttf", "fonts/VT323-Regular.ttf", "fonts/FiraMono-Medium.ttf", "fonts/Inter-SemiBold.ttf", ""];
    let mut report = Vec::new();
    for face in faces {
        for size in [11.0, 16.0, 28.0, 64.0] {
            for words in ["Build it", "Apply recipe", "WHAT ARE YOU BUILDING?"] {
                let label = json!({ "kind": "text", "text": words, "style": { "font": face, "font_size": size, "color": "#ffffff", "text_wrap": "none" } });
                let chip = json!({ "kind": "node", "style": { "background": "#1a1100", "border": 2, "border_color": "#fdf8ef", "padding_x": (size * 0.6f32).round(), "padding_y": (size * 0.3f32).round() }, "children": [label.clone()] });
                let button = json!({ "kind": "button", "style": { "height": (size * 2.2f32).round(), "padding_x": size * 3.0, "align": "center", "justify": "center", "background": "#aa96ff" }, "children": [label] });
                let root = json!({ "kind": "node", "style": { "width": 1600, "height": 400, "gap": 20, "align": "start", "padding": 10, "direction": "column" }, "children": [chip, button] });
                let mut pm = Pixmap::new(1600, 400).unwrap();
                atelico_component::set_layer(40);
                assert!(paint(&root, &mut Surface { pixmap: &mut pm, scale: 1.0, rect: Rect { x: 0.0, y: 0.0, w: 1600.0, h: 400.0 }, clip: None, assets: &roots, time: 0.0 }));
                let found = PAINTED.lock().unwrap()[&40].centring.clone();
                assert_eq!(found.len(), 2, "{face} {size} {words:?}: both boxes are text-in-a-box");
                for (c, what) in found.iter().zip(["chip", "button"]) {
                    assert_eq!(c.by.map(|b| b.is_some()), [true, true], "{what}: both axes meant centred");
                    if !c.ok() {
                        report.push(format!("{face:32} {size:>4} {what:6} {words:24} off by x {:+.1} y {:+.1} (limit {:.1}, {:.1}) content {:?} ink {:?}", c.by[0].unwrap(), c.by[1].unwrap(), c.limit[0], c.limit[1], c.content, c.ink));
                    }
                }
            }
        }
    }
    assert!(report.is_empty(), "{} of {} off-centre:\n{}", report.len(), faces.len() * 4 * 3 * 2, report.join("\n"));
}

#[test]
fn scroll_to_keeps_a_row_in_view_or_the_end() {
    let _one = one();
    let rows: Vec<Value> = (0..40).map(|i| json!({ "kind": "node", "id": format!("row-{i}"), "style": { "width": 100, "height": 20, "shrink": 0 } })).collect();
    let list = |to: &str| json!({ "kind": "node", "style": { "width": 100, "height": 100, "direction": "column", "scroll": true, "scroll_to": to }, "children": rows.clone() });
    let (_, b) = lay(list("row-20"), 100.0, 100.0, 36);
    let r = rect(&b, "row-20");
    assert_eq!(r.y, 25.0, "the row sits a quarter of the way down the view: {r:?}");
    let (_, b) = lay(list("end"), 100.0, 100.0, 36);
    assert_eq!(rect(&b, "row-39").y, 80.0, "the last row at the bottom");
    let (_, b) = lay(list("row-1"), 100.0, 100.0, 36);
    assert_eq!(rect(&b, "row-0").y, 0.0, "never scrolled past the top");
}


/// A cached layer (`cache = true`): painted from its picture, the same as painting its children; panned, its picture
/// laid in at the new place; its boxes and hits placed with it.
fn board(cache: bool, left: f32, scale: f32) -> Value {
    json!({ "kind": "node", "style": { "width": 200, "height": 100, "clip": true, "background": "#101010" }, "children": [
        { "kind": "node", "id": format!("board-{cache}"), "style": { "absolute": true, "left": left, "top": 5, "width": 400, "height": 300, "scale": scale, "cache": cache, "background": "#202020" }, "children": [
            { "kind": "node", "id": format!("card-{cache}"), "on_press": { "event": "route", "name": "pick" }, "style": { "absolute": true, "left": 20, "top": 10, "width": 40, "height": 20, "background": "#ff0000" } },
        ] },
    ] })
}

#[test]
fn a_cached_layer_paints_as_it_would_and_pans_from_its_picture() {
    let _one = one();
    for left in [10.0, 37.0, -12.0] {
        let (with, wb) = lay(board(true, left, 1.0), 200.0, 100.0, 77);
        let (without, nb) = lay(board(false, left, 1.0), 200.0, 100.0, 77);
        assert!(with.data() == without.data(), "panned to {left}: the same picture");
        assert_eq!(rect(&wb, "card-true"), rect(&nb, "card-false"), "its card's box where it is drawn");
    }
    let (_, wb) = lay(board(true, 30.0, 1.0), 200.0, 100.0, 77);
    let r = rect(&wb, "card-true");
    assert_eq!((r.x, r.y), (50.0, 15.0));
}

#[test]
fn a_zoomed_cached_layer_is_drawn_scaled_and_places_its_boxes() {
    let _one = one();
    lay(board(true, 10.0, 1.0), 200.0, 100.0, 78);
    let (pm, wb) = lay(board(true, 10.0, 2.0), 200.0, 100.0, 78);
    let r = rect(&wb, "card-true");
    assert_eq!((r.x, r.y, r.w, r.h), (50.0, 25.0, 80.0, 40.0));
    assert_eq!(px(&pm, 90, 45), [255, 0, 0], "the card drawn at twice the size");
}

/// What the painter draws, recorded into a display list and played on the CPU canvas afterwards, is the same picture
/// pixel for pixel as painting it straight onto that canvas: the list carries everything the painter draws (boxes,
/// round and rotated boxes, borders, gradients, clips and scrolls, text with a caret, a scaled-to-fit box, a cached
/// layer's picture).
#[test]
fn a_recorded_frame_plays_back_to_the_same_pixels() {
    let _one = one();
    let tree = json!({ "kind": "node", "style": { "width": "100%", "height": "100%", "padding": 12, "gap": 8, "direction": "column", "background": "#202530" }, "children": [
        { "kind": "node", "style": { "height": 40, "radius": 9, "background": "#3a6df0", "border": 2, "border_color": "#ffffffaa", "padding_x": 10, "align": "center" }, "children": [
            { "kind": "text", "text": "Recorded, then played", "style": { "font_size": 18, "color": "#ffffff" } } ] },
        { "kind": "node", "style": { "height": 30, "gradient": ["#ff8800", "#2244ff", "#22cc66"], "gradient_dir": "x" } },
        { "kind": "node", "style": { "height": 26, "width": 80, "background": "#e0c060", "rotation": 0.2 } },
        { "kind": "node", "style": { "height": 70, "radius": 14, "clip": true, "background": "#101010" }, "children": [
            { "kind": "node", "style": { "absolute": true, "left": -10, "top": 20, "width": 400, "height": 90, "background": "#c04060" } } ] },
        { "kind": "node", "style": { "height": 60, "scroll": true, "scroll_y": 25, "direction": "column", "background": "#303848" }, "children": [
            { "kind": "text", "text": "one\ntwo\nthree\nfour\nfive", "style": { "font_size": 16, "color": "#d0d8e0" } } ] },
        { "kind": "input", "value": "typing", "focused": true, "style": { "height": 28, "font_size": 16, "color": "#ffffff", "border": 1, "border_color": "#888888" } },
        { "kind": "node", "style": { "height": 50, "fit": "contain" }, "children": [
            { "kind": "node", "style": { "width": 600, "height": 100, "background": "#40a0a0" } } ] },
        { "kind": "node", "style": { "height": 80, "clip": true }, "children": [
            { "kind": "node", "id": "layer", "style": { "cache": true, "absolute": true, "left": 7, "top": 3, "width": 300, "height": 70 }, "children": [
                { "kind": "node", "style": { "width": 120, "height": 40, "radius": 6, "background": "#f0f0f0" } } ] } ] },
    ] });
    for scale in [1.0, 2.0] {
        let (w, h) = (320.0, 480.0);
        let place = Place { rect: Rect { x: 0.0, y: 0.0, w, h }, scale, assets: &[], time: 0.0 };
        let size = ((w * scale) as u32, (h * scale) as u32);
        atelico_component::set_layer(41);
        let mut direct = Pixmap::new(size.0, size.1).unwrap();
        assert!(paint_on(&tree, &place, &mut atelico_paint_cpu::CpuCanvas::new(&mut direct, scale, None)));
        atelico_component::set_layer(42);
        let mut list = atelico_paint::DisplayList::new(size, scale);
        assert!(paint_on(&tree, &place, &mut list));
        let mut played = Pixmap::new(size.0, size.1).unwrap();
        list.play(&mut atelico_paint_cpu::CpuCanvas::new(&mut played, scale, None));
        let differ = direct.data().chunks_exact(4).zip(played.data().chunks_exact(4)).filter(|(a, b)| a != b).count();
        assert!(list.commands.len() > 20, "the list holds what was drawn ({} commands)", list.commands.len());
        assert_eq!(differ, 0, "at scale {scale}: {differ} pixels differ between painting and playing the recording");
    }
}

/// Editing a live HUD reuses its layout. Changed labels, styles, and child shapes must produce the same pixels and
/// hit boxes as a fresh layout; otherwise the consolidated cache can leave old geometry after a hot reload.
#[test]
fn incremental_layout_edits_match_fresh_layout() {
    let _one = one();
    let mut tree = json!({ "kind": "node", "style": { "width": 240, "height": 100, "gap": 4, "background": "#202020" }, "children": [
        { "kind": "text", "id": "label", "text": "1", "style": { "font_size": 18, "color": "#ffffff" } },
        { "kind": "button", "id": "action", "on_press": { "event": "route", "name": "go" }, "style": { "width": 40, "height": 24, "background": "#ff0000" } }
    ] });
    lay(tree.clone(), 240.0, 100.0, 91);
    for edit in 0..5 {
        match edit {
            0 => tree["children"][0]["text"] = json!("a longer label"),
            1 => tree["children"][1]["style"]["width"] = json!(70),
            2 => tree["children"][0] = json!({ "kind": "node", "id": "group", "children": [
                { "kind": "text", "id": "label", "text": "nested", "style": { "font_size": 18, "color": "#ffffff" } }
            ] }),
            3 => tree["children"].as_array_mut().unwrap().push(json!({ "kind": "node", "id": "added", "style": { "width": 20, "height": 20, "background": "#00ff00" } })),
            _ => { tree["children"].as_array_mut().unwrap().remove(0); }
        }
        let (reused, reused_boxes) = lay(tree.clone(), 240.0, 100.0, 91);
        LAID.with_borrow_mut(|laid| laid.clear());
        let (fresh, fresh_boxes) = lay(tree.clone(), 240.0, 100.0, 91);
        assert_eq!(reused.data(), fresh.data(), "edit {edit}: cached layout pixels");
        assert_eq!(reused_boxes, fresh_boxes, "edit {edit}: cached layout boxes");
    }
}

#[test]
fn bounded_cached_panel_matches_direct_painting() {
    let _one = one();
    for left in [10.0, 37.0, -12.0] {
        let mut cached = board(true, left, 1.0);
        cached["children"][0]["style"]["cache_bounded"] = json!(true);
        let (with, wb) = lay(cached, 200.0, 100.0, 92);
        let (without, nb) = lay(board(false, left, 1.0), 200.0, 100.0, 92);
        assert_eq!(with.data(), without.data(), "bounded panel panned to {left}");
        assert_eq!(rect(&wb, "card-true"), rect(&nb, "card-false"));
    }
}

#[test]
fn scroll_geometry_uses_layout_ranges_clips_and_cached_or_fitted_coordinates() {
    let _one = one();
    let inner = json!({ "id":"inner", "kind":"node", "style":{"scroll":true,"direction":"column","width":80,"height":60,"shrink":0},
        "children":[{"kind":"node","style":{"height":500,"shrink":0}}] });
    let mut outer = json!({"id":"outer","kind":"node","style":{"scroll":true,"direction":"column","width":100,"height":100,"padding":10},
        "children":[inner,{"kind":"node","style":{"height":50,"shrink":0}}]});
    lay(outer.clone(),100.,100.,98);
    let regions: Vec<ScrollRegion> = serde_json::from_value(inspect()["scrolls"]["98"].clone()).unwrap();
    assert_eq!(regions.len(),2);
    assert_eq!(regions[0].maximum,30.,"outer content includes padding but not the nested 500px scroll body");
    assert_eq!(regions[1].maximum,440.);
    outer["style"]["scroll_y"] = json!(1000);
    lay(outer,100.,100.,98);
    let regions = PAINTED.lock().unwrap()[&98].scrolls.clone();
    assert_eq!(regions[0].offset,30.,"published offset is the actual clamped painted offset");
    assert_eq!(regions[1].rect,[10.,0.,80.,40.],"hidden portion cannot own wheel input");

    let pane = json!({"id":"pane","kind":"node","style":{"scroll":true,"direction":"column","width":200,"height":200,"shrink":0},
        "children":[{"kind":"node","style":{"height":600,"shrink":0}}]});
    let fitted = json!({"kind":"node","style":{"width":100,"height":100,"fit":"contain"},"children":[pane.clone()]});
    lay(fitted,100.,100.,99);
    let region = PAINTED.lock().unwrap()[&99].scrolls[0].clone();
    assert_eq!(region.rect,[0.,0.,100.,100.]);
    assert_eq!((region.scale,region.maximum),(0.5,400.));

    let mut cached = json!({"kind":"node","style":{"width":400,"height":300},"children":[
        {"id":"scroll-cache","kind":"node","style":{"cache":true,"absolute":true,"left":20,"top":20,"width":200,"height":200},"children":[pane]}]});
    for x in [20.,50.] {
        cached["children"][0]["style"]["left"]=json!(x);
        lay(cached.clone(),400.,300.,97);
        let region = PAINTED.lock().unwrap()[&97].scrolls[0].clone();
        assert_eq!(region.rect,[x,20.,200.,200.],"cached picture scroll regions move with its controls");
        assert_eq!((region.scale,region.maximum),(1.,400.));
    }
}
