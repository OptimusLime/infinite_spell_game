//! A scene rendered through a look graph. Typed cache keys retain source/project identity;
//! native Canvas frames own immutable GPU pictures while public `render` remains an explicit capture.
use crate::{
    nodes::Item,
    picture::{self, Picture},
    scene_file::{load_world, SceneWorld},
};
use atelico_component::{
    register,
    tiny_skia::{FilterQuality, PixmapPaint},
    Component, On, Rect, Surface, Value,
};
use serde::Deserialize;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};
register!(Component {
    name: "scene-3d-look-preview",
    paint,
    draw,
    ..Component::NONE
});
#[derive(Deserialize, Default)]
#[serde(default)]
struct Node {
    id: String,
    graph: String,
    scene: String,
    camera: String,
    time: f32,
    version: String,
}
fn modified(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}
fn canonical(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}
type SceneEntry = (PathBuf, Option<SystemTime>, Result<SceneWorld, String>);
static SCENES: Mutex<VecDeque<SceneEntry>> = Mutex::new(VecDeque::new());
fn scene(root: &Path, path: &str) -> Result<SceneWorld, String> {
    let file = canonical(root.join(path));
    let stamp = modified(&file);
    let mut scenes = SCENES.lock().unwrap();
    if let Some(i) = scenes
        .iter()
        .position(|(p, m, _)| *p == file && *m == stamp)
    {
        let e = scenes.remove(i).unwrap();
        let result = e.2.clone();
        scenes.push_back(e);
        return result;
    }
    let loaded = load_world(root, Path::new(path));
    scenes.retain(|(p, _, _)| *p != file);
    while scenes.len() >= 8 {
        scenes.pop_front();
    }
    scenes.push_back((file, stamp, loaded.clone()));
    loaded
}
fn graph_path(root: &Path, graph: &str) -> PathBuf {
    canonical(root.join("graphs").join(format!(
        "{}.graph.yaml",
        graph.split('@').next().unwrap_or(graph)
    )))
}
fn frame(
    root: &Path,
    scene_path: &str,
    graph: &str,
    camera: &str,
    time: f32,
    w: u32,
    h: u32,
) -> Result<crate::world::Frame, String> {
    let sw = scene(root, scene_path)?;
    let mut items = sw.items.clone();
    let mut graded = false;
    // Make the source explicit for public callers whose root differs from the process project.
    let graph = if graph.is_empty() {
        String::new()
    } else {
        let (name, stage) = graph
            .split_once('@')
            .map_or((graph, None), |(n, s)| (n, Some(s)));
        let file = graph_path(root, name);
        let stem = file
            .to_string_lossy()
            .trim_end_matches(".graph.yaml")
            .to_string();
        stage.map_or_else(|| stem.clone(), |s| format!("{stem}@{s}"))
    };
    for i in &mut items {
        if let Item::Grade(g) = i {
            g.graph = graph.clone();
            graded = true;
        }
    }
    if !graded {
        items.push(Item::Grade(crate::nodes::Grade {
            graph,
            ..Default::default()
        }));
    }
    let cam = sw.cameras.get(camera).cloned();
    let mut frame = crate::world::frame(
        &sw.world,
        &items,
        cam.as_ref(),
        None,
        &[],
        time,
        w as f32 / h.max(1) as f32,
    );
    frame.pixel = crate::world::pixel_look(&items, cam.as_ref(), sw.pixel, w as f32, h as f32);
    Ok(frame)
}
/// The scene and graph in the caller's project as RGBA8 pixels. This intentionally reads the GPU.
pub fn render(
    root: &Path,
    scene_path: &str,
    graph: &str,
    camera: &str,
    time: f32,
    w: u32,
    h: u32,
) -> Result<Vec<u8>, String> {
    let f = frame(root, scene_path, graph, camera, time, w, h)?;
    crate::world::render_in(
        &format!("look-preview/{}", root.join(scene_path).display()),
        &f,
        w,
        h,
    )
}
#[derive(PartialEq)]
struct FrameKey {
    root: PathBuf,
    layer: u32,
    id: String,
    scene: PathBuf,
    graph: PathBuf,
    graph_name: String,
    camera: String,
    time: u32,
    size: (u32, u32),
    scene_modified: Option<SystemTime>,
    graph_modified: Option<SystemTime>,
    version: String,
    gpu: bool,
}
impl picture::ViewKey for FrameKey {
    fn same_view(&self, other: &Self) -> bool {
        self.root == other.root
            && self.layer == other.layer
            && self.id == other.id
            && self.gpu == other.gpu
    }
}
static FRAMES: Mutex<picture::Cache<FrameKey>> = Mutex::new(picture::Cache::new());
fn prepare(
    node: &Value,
    rect: Rect,
    scale: f32,
    gpu: bool,
) -> Result<std::sync::Arc<Picture>, String> {
    let n = Node::deserialize(node).map_err(|e| e.to_string())?;
    let root = picture::project_root();
    let size = (
        ((rect.w * scale) as u32).clamp(1, 1280),
        ((rect.h * scale) as u32).clamp(1, 1280),
    );
    let scene = canonical(root.join(&n.scene));
    let graph = graph_path(&root, &n.graph);
    let key = FrameKey {
        root: root.clone(),
        layer: atelico_component::layer(),
        id: n.id.clone(),
        scene_modified: modified(&scene),
        graph_modified: modified(&graph),
        scene,
        graph,
        graph_name: n.graph.clone(),
        camera: n.camera.clone(),
        time: n.time.to_bits(),
        size,
        version: n.version,
        gpu,
    };
    FRAMES.lock().unwrap().get_or_render(key, || {
        let f = frame(&root, &n.scene, &n.graph, &n.camera, n.time, size.0, size.1)?;
        let view = format!("look-preview/{}/{}", root.display(), n.id);
        if gpu {
            if let Some(on) = crate::world::render_texture_in(&view, &f, size.0, size.1)? {
                return Picture::snapshot(on, size.0, size.1);
            }
        }
        crate::world::render_in(&view, &f, size.0, size.1)
            .and_then(|px| crate::pixels_of(&px, size.0, size.1))
            .map(Picture::pixels_only)
    })
}
fn source(node: &Value) -> String {
    format!(
        "look_preview {}",
        node.get("id").and_then(Value::as_str).unwrap_or("")
    )
}
fn placement(rect: Rect, scale: f32) -> atelico_component::tiny_skia::Transform {
    let w = ((rect.w * scale) as u32).clamp(1, 1280);
    let h = ((rect.h * scale) as u32).clamp(1, 1280);
    atelico_component::tiny_skia::Transform::from_translate(rect.x, rect.y)
        .post_scale(scale, scale)
        .pre_scale(rect.w / w as f32, rect.h / h as f32)
}
fn paint(node: &Value, s: &mut Surface) -> bool {
    if node.get("kind").and_then(Value::as_str) != Some("look_preview") {
        return false;
    }
    let result = prepare(node, s.rect, s.scale, false)
        .and_then(|p| p.pixels())
        .map(|pm| {
            s.pixmap.draw_pixmap(
                0,
                0,
                pm.as_ref().as_ref(),
                &PixmapPaint {
                    quality: FilterQuality::Nearest,
                    ..Default::default()
                },
                placement(s.rect, s.scale),
                s.clip,
            );
        });
    match result {
        Ok(()) => atelico_component::clear(&source(node)),
        Err(e) => atelico_component::error(&source(node), e),
    }
    true
}
fn draw(node: &Value, on: &mut On) -> bool {
    if node.get("kind").and_then(Value::as_str) != Some("look_preview") {
        return false;
    }
    let result = prepare(node, on.rect, on.canvas.scale(), true).and_then(|p| {
        let t = placement(on.rect, on.canvas.scale());
        p.draw(
            on.canvas,
            t,
            atelico_component::atelico_paint::Filter::Nearest,
        )
    });
    match result {
        Ok(()) => atelico_component::clear(&source(node)),
        Err(e) => atelico_component::error(&source(node), e),
    }
    true
}
#[cfg(test)]
mod tests {
    #[test]
    fn a_look_preview_draws_the_scene_through_its_graph() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let a = super::render(
            &root,
            "scenes/pixel-world.scene.luau",
            "pixel-world",
            "player",
            0.,
            320,
            180,
        )
        .unwrap();
        assert!(
            a.chunks(4)
                .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 60)
                .count()
                > 2000,
            "a town drawn"
        );
    }
}
