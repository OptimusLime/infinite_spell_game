//! The look preview: a scene file drawn through a look graph, for the Shader Preview panel (shader-forge's
//! `graph_renderers.shader` gives `{kind = "look_preview", id, graph, scene, camera, time}` for a look graph). The
//! scene (its World, items and Cameras, read again when its file changes) is drawn with its Grade's `graph` set to the
//! previewed graph, through the named Camera, at the panel's size; the graph is compiled again when its file changes
//! (packages/world-looks), so a dragged param shows here on the next frame (the node carries the compiled stage's
//! `version`, so a param change repaints it).
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use atelico_component::tiny_skia::{FilterQuality, Pixmap, PixmapPaint};
use atelico_component::{num, register, Component, Surface, Value};

use crate::nodes::Item;
use crate::scene_file::{load_world, SceneWorld};

register!(Component { name: "scene-3d-look-preview", paint, ..Component::NONE });

fn root() -> PathBuf {
    atelico_node_cache::shared().map_or_else(|| std::env::current_dir().unwrap_or_default(), |c| c.root().to_path_buf())
}

fn modified(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Scenes read, by path, with the time their file was read at.
static SCENES: Mutex<BTreeMap<String, (Option<SystemTime>, Result<SceneWorld, String>)>> = Mutex::new(BTreeMap::new());
/// The last picture per preview id and what it was drawn from.
static FRAMES: Mutex<BTreeMap<String, (String, Pixmap)>> = Mutex::new(BTreeMap::new());

fn scene(root: &Path, path: &str) -> Result<SceneWorld, String> {
    let file = root.join(path);
    let m = modified(&file);
    let mut scenes = SCENES.lock().unwrap();
    if scenes.get(path).is_none_or(|(when, _)| *when != m) {
        scenes.insert(path.to_string(), (m, load_world(root, Path::new(path))));
    }
    scenes[path].1.clone()
}

/// The scene `scene` (from the project `root`) through look graph `graph` and Camera `camera` at `time`, `w` x `h`
/// RGBA8 pixels.
pub fn render(root: &Path, scene_path: &str, graph: &str, camera: &str, time: f32, w: u32, h: u32) -> Result<Vec<u8>, String> {
    let sw = scene(root, scene_path)?;
    let mut items = sw.items.clone();
    let mut graded = false;
    for i in items.iter_mut() {
        if let Item::Grade(g) = i {
            g.graph = graph.to_string();
            graded = true;
        }
    }
    if !graded {
        items.push(Item::Grade(crate::nodes::Grade { graph: graph.to_string(), ..Default::default() }));
    }
    let cam = sw.cameras.get(camera).cloned();
    let mut frame = crate::world::frame(&sw.world, &items, cam.as_ref(), None, &[], time, w as f32 / h.max(1) as f32);
    frame.pixel = crate::world::pixel_look(&items, cam.as_ref(), sw.pixel, w as f32, h as f32);
    crate::world::render_in(&format!("look-preview/{scene_path}"), &frame, w, h)
}

fn paint(node: &Value, s: &mut Surface) -> bool {
    if node.get("kind").and_then(Value::as_str) != Some("look_preview") {
        return false;
    }
    let text = |k: &str| node.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let (id, graph, scene_path, camera) = (text("id"), text("graph"), text("scene"), text("camera"));
    let time = num(node, "time", 0.0);
    let (w, h) = (((s.rect.w * s.scale) as u32).clamp(1, 1280), ((s.rect.h * s.scale) as u32).clamp(1, 1280));
    let graph_file = root().join("graphs").join(format!("{graph}.graph.yaml"));
    let key = format!("{graph}{scene_path}{camera}{time}{w}x{h}{:?}{:?}{}", modified(&graph_file), modified(&root().join(&scene_path)), text("version"));
    let source = format!("look_preview {id}");
    let mut frames = FRAMES.lock().unwrap();
    if frames.get(&id).is_none_or(|f| f.0 != key) {
        match render(&root(), &scene_path, &graph, &camera, time, w, h) {
            Ok(px) => {
                atelico_component::clear(&source);
                let Some(mut pm) = Pixmap::new(w, h) else { return true };
                pm.data_mut().copy_from_slice(&px);
                frames.insert(id.clone(), (key, pm));
            }
            Err(e) => {
                atelico_component::error(&source, e);
                return true;
            }
        }
    }
    let pm = &frames[&id].1;
    let t = s.transform().pre_scale(s.rect.w / w as f32, s.rect.h / h as f32);
    s.pixmap.draw_pixmap(0, 0, pm.as_ref(), &PixmapPaint { quality: FilterQuality::Nearest, ..Default::default() }, t, s.clip);
    true
}

#[cfg(test)]
mod tests {
    /// The pixel town through its look graph renders.
    #[test]
    fn a_look_preview_draws_the_scene_through_its_graph() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let a = super::render(&root, "scenes/pixel-world.scene.luau", "pixel-world", "player", 0.0, 320, 180).unwrap();
        assert!(a.chunks(4).filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 60).count() > 2000, "a town drawn");
    }
}
