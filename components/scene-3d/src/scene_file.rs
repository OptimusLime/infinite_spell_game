//! A World read straight from a scene file (instances resolved by atelico-scene), as the World node's builder
//! gathers it in the editor: its props, every 3D node under it depth first, and its Cameras by name. For rendering
//! without an editor: comparison sheets, looks, stills.
use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use crate::nodes::{Camera, Item, World};

#[derive(Clone)]
pub struct SceneWorld {
    pub world: World,
    pub items: Vec<Item>,
    pub cameras: BTreeMap<String, Camera>,
    /// The scene's `pixel`: drawn as pixel art, that many of its pixels to one (1: not).
    pub pixel: f32,
}


/// The first World in the scene file `rel` (relative to the project `root`).
pub fn load_world(root: &Path, rel: &Path) -> Result<SceneWorld, String> {
    // (the parts that register themselves from crates nothing here otherwise calls)
    component_creatures::link();
    component_founding::link();
    let loaded = atelico_scene::load(root, rel).map_err(|e| format!("{}: {e:#}", rel.display()))?;
    let nodes = loaded.value.get("nodes").and_then(Value::as_object).ok_or("the scene has no nodes")?;
    let kind = |path: &str| nodes.get(path).and_then(|n| n.get("type")).and_then(Value::as_str).unwrap_or("");
    let (path, node) = nodes.iter().find(|(p, _)| kind(p) == "World").ok_or("the scene has no World")?;
    let props = |n: &Value| n.get("props").cloned().filter(Value::is_object).unwrap_or_else(|| serde_json::json!({}));
    let world: World = serde_json::from_value(props(node)).map_err(|e| format!("{path}: {e}"))?;
    let (mut items, mut cameras) = (Vec::new(), BTreeMap::new());
    let mut stack: Vec<String> = children(node).into_iter().rev().collect();
    while let Some(p) = stack.pop() {
        let Some(n) = nodes.get(&p) else { continue };
        let ty = kind(&p);
        if ty == "Camera" {
            let name = n.get("name").and_then(Value::as_str).unwrap_or(&p).to_string();
            cameras.insert(name, serde_json::from_value(props(n)).map_err(|e| format!("{p}: {e}"))?);
        } else if Item::is_type(ty) {
            let mut v = props(n);
            v["type"] = Value::String(ty.into());
            // a Character is found by its node's name (third-person drives the one it is given), as the Luau builder does
            if ty == "Character" && v.get("name").and_then(Value::as_str).is_none_or(str::is_empty) {
                v["name"] = n.get("name").cloned().unwrap_or(Value::Null);
            }
            items.push(serde_json::from_value(v).map_err(|e| format!("{p}: {e}"))?);
        }
        stack.extend(children(n).into_iter().rev());
    }
    let pixel = loaded.value.get("pixel").and_then(Value::as_f64).map_or(1.0, |p| p.round().max(1.0) as f32);
    Ok(SceneWorld { world, items, cameras, pixel })
}

fn children(n: &Value) -> Vec<String> {
    n.get("children").and_then(Value::as_array).map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect()).unwrap_or_default()
}

#[cfg(test)]
fn item_json(i: &Item) -> serde_json::Value {
    i.json()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Grade really runs on every path: with an inverting look at full strength, each pixel of the editor's live
    /// paint and of a still comes out as 255 minus what it is without the look; and one stop more exposure brightens.
    #[test]
    fn the_grade_is_applied_not_just_plausible() {
        use crate::nodes::{Grade, Sky, World};
        use atelico_component::tiny_skia::Pixmap;
        let dir = std::env::temp_dir().join("scene-3d-grade-test");
        let invert = dir.join("invert.cube");
        let mut lut = crate::look::Lut::identity(17);
        for c in &mut lut.table {
            *c = c.map(|v| 1.0 - v);
        }
        lut.write(&invert, "invert").unwrap();
        let world = World { stage: true, ..World::default() };
        let items = |look: f64, exposure: f64| {
            vec![
                Item::TimeOfDay(crate::nodes::TimeOfDay { hour: 12.0, ..Default::default() }), Item::Sky(Sky { ..Sky::default() }),
                Item::Grade(Grade { look: invert.display().to_string(), look_strength: look, exposure, bloom: 0.0, ..Grade::default() }),
                Item::part(crate::nodes::Hut::default()),
            ]
        };
        let cam = Camera { x: 0.0, y: 2.0, z: 6.0, look_x: 0.0, look_y: 1.0, look_z: 0.0, fov: 45.0, time: 0.0, ..Camera::default() };
        let (w, h) = (160u32, 90u32);
        let still = |look, exposure| crate::world::render(&crate::world::frame(&world, &items(look, exposure), Some(&cam), None, &[], 0.0, w as f32 / h as f32), w, h).unwrap();
        let live = |look: f64| {
            let mut node = serde_json::json!({ "kind": "scene_3d", "id": format!("grade {look}"), "world": world, "camera": cam, "time": 0.0 });
            node["items"] = serde_json::to_value(items(look, 0.0).iter().map(item_json).collect::<Vec<_>>()).unwrap();
            let mut pm = Pixmap::new(w, h).unwrap();
            let assets: Vec<std::path::PathBuf> = Vec::new();
            let mut surface = atelico_component::Surface { pixmap: &mut pm, scale: 1.0, rect: atelico_component::Rect { x: 0.0, y: 0.0, w: w as f32, h: h as f32 }, clip: None, assets: &assets, time: 0.0 };
            atelico_component::paint(&node, &mut surface);
            pm.data().to_vec()
        };
        for (plain, looked, path) in [(still(0.0, 0.0), still(1.0, 0.0), "still"), (live(0.0), live(1.0), "live")] {
            let worst = plain.chunks(4).zip(looked.chunks(4)).map(|(a, b)| (0..3).map(|k| (255 - a[k] as i32 - b[k] as i32).abs()).max().unwrap()).max().unwrap();
            assert!(worst <= 12, "{path}: an inverting look should give 255 - v, worst error {worst}");
        }
        let mean = |px: &[u8]| px.iter().map(|&v| v as f32).sum::<f32>() / px.len() as f32;
        assert!(mean(&still(0.0, 1.0)) > mean(&still(0.0, 0.0)) + 10.0, "one stop more exposure brightens");
    }

    /// The grade cannot be skipped: the same view drawn through every path (the editor's live paint of a World node,
    /// a still from render-scene, a camera pose from another package) goes through the post pass and comes out alike,
    /// in a scene with a Grade and in one without.
    #[test]
    fn every_render_path_is_graded_alike() {
        use crate::nodes::World;
        use atelico_component::tiny_skia::Pixmap;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        std::env::set_current_dir(&root).unwrap();
        let white = |px: &[u8]| px.chunks(4).filter(|p| p[0] >= 245 && p[1] >= 245 && p[2] >= 245).count() as f32 / (px.len() / 4) as f32;
        for scene in ["scenes/island.scene.luau", "scenes/island-walk.scene.luau"] {
            let w = load_world(&root, Path::new(scene)).unwrap();
            let cam = Camera { x: -21.0, y: 1.8, z: 3.5, look_x: -12.0, look_y: 1.2, look_z: 0.0, fov: 48.0, time: 0.0, ..Camera::default() };
            let (wd, ht) = (480u32, 270u32);
            // a still, as render-scene makes it
            let still = crate::world::render(&crate::world::frame(&w.world, &w.items, Some(&cam), None, &[], 0.0, wd as f32 / ht as f32), wd, ht).unwrap();
            // a camera pose from another package (a follow camera)
            let pose = crate::hooks::Pose { eye: glam::vec3(-21.0, 1.8, 3.5), look: glam::vec3(-12.0, 1.2, 0.0), fov: 48.0 };
            let posed = crate::world::render(&crate::world::frame(&World { camera: String::new(), ..w.world.clone() }, &w.items, None, Some(pose), &[], 0.0, wd as f32 / ht as f32), wd, ht).unwrap();
            // the editor's live paint of the World node
            let mut node = serde_json::json!({ "kind": "scene_3d", "id": format!("test {scene}"), "world": w.world, "items": [], "camera": cam, "time": 0.0 });
            node["items"] = serde_json::to_value(w.items.iter().map(item_json).collect::<Vec<_>>()).unwrap();
            let mut pm = Pixmap::new(wd, ht).unwrap();
            let assets: Vec<std::path::PathBuf> = Vec::new();
            let mut surface = atelico_component::Surface { pixmap: &mut pm, scale: 1.0, rect: atelico_component::Rect { x: 0.0, y: 0.0, w: wd as f32, h: ht as f32 }, clip: None, assets: &assets, time: 0.0 };
            assert!(atelico_component::paint(&node, &mut surface));
            let live = pm.data().to_vec();
            let shares = [white(&still), white(&posed), white(&live)];
            let mean = |px: &[u8]| px.iter().map(|&v| v as f32).sum::<f32>() / px.len() as f32;
            assert!(shares.iter().all(|s| *s < 0.06), "{scene}: near-white share through still, pose, live: {shares:?}");
            let (a, b, c) = (mean(&still), mean(&posed), mean(&live));
            assert!((a - b).abs() < 2.0 && (a - c).abs() < 3.0, "{scene}: mean brightness through still, pose, live: {a:.1} {b:.1} {c:.1}");
        }
    }

    /// Blowout: through every island camera, the share of clipped or near-white pixels (every channel at 245 or
    /// above) stays under 6%: bright sand, foam and clouds keep their shading.
    #[test]
    fn no_island_camera_is_blown_out() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        std::env::set_current_dir(&root).unwrap();
        let w = load_world(&root, Path::new("scenes/island.scene.luau")).unwrap();
        let mut report = Vec::new();
        for (name, cam) in &w.cameras {
            let frame = crate::world::frame(&w.world, &w.items, Some(cam), None, &[], cam.time.max(0.0) as f32, 16.0 / 9.0);
            let px = crate::world::render(&frame, 640, 360).unwrap();
            let white = px.chunks(4).filter(|p| p[0] >= 245 && p[1] >= 245 && p[2] >= 245).count() as f32 / (640.0 * 360.0);
            report.push((name.clone(), white));
        }
        let worst = report.iter().cloned().fold((String::new(), 0.0f32), |a, b| if b.1 > a.1 { b } else { a });
        assert!(worst.1 < 0.06, "near-white share per camera: {report:?}");
    }

    #[test]
    fn the_island_scene_gathers_its_parts_and_cameras() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let w = load_world(&root, Path::new("scenes/island.scene.luau")).unwrap();
        assert!(w.items.iter().any(|i| matches!(i, Item::Terrain(_))));
        assert!(w.items.iter().filter(|i| i.get::<crate::nodes::Plant>().is_some()).count() >= 8);
        assert!(w.items.iter().any(|i| matches!(i, Item::Palette(_))), "the sky's palettes, instanced two levels down");
        let campfire = load_world(&root, Path::new("scenes/campfire.scene.luau")).unwrap();
        assert!(campfire.items.iter().any(|i| i.get::<crate::nodes::Character>().is_some_and(|c| c.name == "hero")), "a Character is named after its node");
        assert!(w.cameras.contains_key("hero-aerial"));
    }

    /// A World that cannot be drawn says why where proofs look (the editor's console, `runtime.errors`), once, not
    /// only on stderr.
    #[test]
    fn a_bad_prop_is_reported_once() {
        use atelico_component::tiny_skia::Pixmap;
        let node = serde_json::json!({ "kind": "scene_3d", "id": "test/bad-prop", "world": {}, "items": [{ "type": "Grade", "output": "sepia" }] });
        let mut pm = Pixmap::new(32, 32).unwrap();
        let assets: Vec<std::path::PathBuf> = Vec::new();
        for _ in 0..3 {
            let mut surface = atelico_component::Surface { pixmap: &mut pm, scale: 1.0, rect: atelico_component::Rect { x: 0.0, y: 0.0, w: 32.0, h: 32.0 }, clip: None, assets: &assets, time: 0.0 };
            atelico_component::paint(&node, &mut surface);
        }
        let mine: Vec<_> = atelico_component::take_errors().into_iter().filter(|(s, _)| s == "scene_3d test/bad-prop").collect();
        assert_eq!(mine.len(), 1, "{mine:?}");
        assert!(mine[0].1.contains("sepia"), "{}", mine[0].1);
    }
}
#[cfg(test)]
mod probe_two_shadows {
    #[test]
    #[ignore]
    fn second_light() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for f in ["scenes/two-shadows-pixel.scene.luau", "scenes/two-shadows-ink.scene.luau"] {
            let w = super::load_world(&root, std::path::Path::new(f)).unwrap();
            let cam = w.cameras.get("two-shadows").cloned();
            let e = crate::world::environment_of(&w.items, cam.as_ref(), 0.0);
            let skies: Vec<String> = w.items.iter().filter_map(|i| if let crate::nodes::Item::Sky(s) = i { Some(format!("seed {} day {} second {}", s.moon_seed, s.moon_day, s.second_shadow)) } else { None }).collect();
            eprintln!("{f}: hour {} day {:.3} second {:?} colour {:?} shadow {} skies {skies:?} purple {:?} orange {:?}", e.hour, e.moon_days, e.second, e.second_colour, e.second_shadow, e.purple, e.orange);
        }
    }
}
