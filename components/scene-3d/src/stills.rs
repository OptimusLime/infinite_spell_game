//! Stills of a scene file's World through one of its Cameras at fixed moments (`scene_3d.still` in Luau): what the
//! video workflow's Capture node takes. Each still is a job, keyed by the scene file (its size and modification
//! time), the camera, the hour, the moment and the size, rendered once on a worker thread into
//! `.atelico/post/stills/<key>.png`; Luau asks again until it is done.
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// One still: the scene file, its Camera by name (empty: the World's own), the hour (0 or less: the camera's), the
/// moment (seconds) and the picture's size.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Want {
    pub file: String,
    #[serde(default)]
    pub camera: String,
    #[serde(default)]
    pub hour: f64,
    pub t: f64,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum Job {
    Running { what: String },
    Done { file: String },
    Failed { error: String },
}

pub const DIR: &str = ".atelico/post/stills";

static JOBS: Mutex<BTreeMap<String, Job>> = Mutex::new(BTreeMap::new());
/// One still is rendered at a time (the GPU is shared with the frame being drawn).
static WORKER: Mutex<()> = Mutex::new(());

fn key(root: &Path, w: &Want) -> Result<String, String> {
    let meta = std::fs::metadata(root.join(&w.file)).map_err(|e| format!("{}: {e}", w.file))?;
    let mut h = DefaultHasher::new();
    (&w.file, meta.len(), meta.modified().ok(), &w.camera, w.hour.to_bits(), w.t.to_bits(), w.width, w.height).hash(&mut h);
    let id = Path::new(&w.file).file_name().and_then(|f| f.to_str()).and_then(|f| f.split('.').next()).unwrap_or("scene");
    Ok(format!("{id}-{:016x}", h.finish()))
}

/// The still `w` of a scene file in the project `root`: done when its file is there, else rendered once on a thread.
pub fn still(root: &Path, w: &Want) -> Job {
    let name = match key(root, w) {
        Ok(k) => k,
        Err(error) => return Job::Failed { error },
    };
    let rel = PathBuf::from(DIR).join(format!("{name}.png"));
    // (the file by its whole path: an image finds it whatever roots its view looks in)
    if root.join(&rel).is_file() {
        return Job::Done { file: root.join(&rel).display().to_string() };
    }
    let mut jobs = JOBS.lock().unwrap();
    if let Some(j) = jobs.get(&name).filter(|j| !matches!(j, Job::Done { .. })) {
        return j.clone();
    }
    let running = Job::Running { what: format!("still at {:.1} s", w.t) };
    jobs.insert(name.clone(), running.clone());
    let (root, w) = (root.to_path_buf(), w.clone());
    std::thread::spawn(move || {
        let _one = WORKER.lock().unwrap_or_else(|e| e.into_inner());
        let file = root.join(&rel);
        let part = file.with_extension("part.png");
        let done = render(&root, &w, &part).and_then(|_| std::fs::rename(&part, &file).map_err(|e| e.to_string()));
        JOBS.lock().unwrap().insert(name, match done {
            Ok(()) => Job::Done { file: root.join(&rel).display().to_string() },
            Err(error) => Job::Failed { error },
        });
    });
    running
}

fn render(root: &Path, w: &Want, out: &Path) -> Result<(), String> {
    let s = crate::scene_file::load_world(root, Path::new(&w.file))?;
    let mut camera = if w.camera.is_empty() { None } else { Some(s.cameras.get(&w.camera).cloned().ok_or_else(|| format!("{} has no camera {}", w.file, w.camera))?) };
    if let Some(c) = camera.as_mut() {
        c.time = w.t;
        if w.hour > 0.0 {
            c.hour = w.hour;
        }
    }
    crate::snapshot_as(&s.world, &s.items, camera.as_ref(), w.width, w.height, w.t as f32, s.pixel, out)
}
