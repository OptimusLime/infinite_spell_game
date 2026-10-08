//! Shader Forge: shader node graphs compiled to WGSL (compile.rs) and drawn by wgpu offscreen (gpu.rs).
//! Luau gets `shader.compile(name, graph, catalog, target?)` (the graph's run: per-node status and the WGSL) and
//! `shader.snapshot(wgsl, w, h, time, file)`; the `shader_preview` node kind draws a compiled shader at a time.
pub mod check;
pub mod compile;
pub mod function;
pub mod gpu;

use std::collections::BTreeMap;
use std::sync::Mutex;

use atelico_component::mlua::{self, Lua, LuaSerdeExt, Value as LuaValue};
use atelico_component::tiny_skia::{FilterQuality, Pixmap, PixmapPaint};
use atelico_component::{num, register, Component, Surface, Value};
use atelico_graph::{Cache, Catalog, Graph};
use atelico_timeline::Track;
use serde_json::json;

use compile::{compile, Expr};

register!(Component { name: "shader-forge", lua: install, paint, inspect, ..Component::NONE });

/// Compile caches by graph name and target (the output, or a previewed node).
static CACHES: Mutex<BTreeMap<String, Cache<Expr>>> = Mutex::new(BTreeMap::new());
/// Each graph's last compile: per-node status, for `/scene` and proofs.
static RUNS: Mutex<BTreeMap<String, Value>> = Mutex::new(BTreeMap::new());
/// The last compile that changed something, per graph and target: (the target's key, its summary).
static LAST: Mutex<BTreeMap<String, (String, Value)>> = Mutex::new(BTreeMap::new());
/// The last frame drawn per preview id: (wgsl hash, time, w, h) → pixels, so an unchanged frame is not redrawn.
static FRAMES: Mutex<BTreeMap<String, (u64, u32, u32, u32, Pixmap)>> = Mutex::new(BTreeMap::new());

fn inspect() -> Value {
    json!({ "runs": *RUNS.lock().unwrap() })
}

fn typed<T: serde::de::DeserializeOwned>(lua: &Lua, v: LuaValue) -> mlua::Result<T> {
    let mut v: Value = lua.from_value(v)?;
    // Luau writes an empty table as an array; these are maps
    if let Some(m) = v.as_object_mut() {
        for (k, x) in m.iter_mut() {
            if ["nodes", "edges", "groups", "casts"].contains(&k.as_str()) && x.as_array().is_some_and(Vec::is_empty) {
                *x = json!({});
            }
        }
        if let Some(nodes) = m.get_mut("nodes").and_then(Value::as_object_mut) {
            for n in nodes.values_mut() {
                if n.get("params").is_some_and(|p| p.as_array().is_some_and(Vec::is_empty)) {
                    n["params"] = json!({});
                }
            }
        }
    }
    serde_json::from_value(v).map_err(mlua::Error::external)
}

/// Compiles a graph (as the store holds it) for its output or one node: the summary Luau and proofs read.
/// `animated`: the `node:param`s a timeline track drives (they compile to uniform slots).
pub fn compile_summary(name: &str, graph: &Graph, cat: &Catalog, target: Option<&str>, animated: &[String]) -> Value {
    let key = format!("{name}@{}", target.unwrap_or("output"));
    let mut cache = CACHES.lock().unwrap().remove(&key).unwrap_or_default();
    let result = compile(graph, cat, target, animated, &mut cache);
    CACHES.lock().unwrap().insert(key.clone(), cache);
    let nodes = |run: &atelico_graph::Run<Expr>| -> serde_json::Map<String, Value> {
        run.nodes.iter().map(|(k, n)| {
            let mut v = json!(n.status);
            v["key"] = json!(n.key);
            (k.clone(), v)
        }).collect()
    };
    // Several panels compile the same graph every frame; after the first, all is cached. What a node did is
    // what it did for the graph as it is now: the run that last changed the target's key.
    let summary = match &result {
        Ok(s) => {
            let target_key = s.run.nodes[&s.target].key.clone();
            let mut last = LAST.lock().unwrap();
            match last.get(&key) {
                Some((k, v)) if *k == target_key => v.clone(),
                _ => {
                    let v = json!({ "ok": true, "wgsl": s.wgsl, "slots": s.slots, "target": s.target, "order": s.run.order, "ran": s.run.ran, "cached": s.run.cached, "nodes": nodes(&s.run) });
                    last.insert(key, (target_key, v.clone()));
                    v
                }
            }
        }
        // an invalid edit keeps the last good shader drawing; the error says what to fix
        Err(f) => {
            let good = LAST.lock().unwrap().get(&key).map(|(_, v)| v.clone());
            let mut v = json!({ "ok": false, "error": f.error, "nodes": f.run.as_ref().map(nodes).unwrap_or_default() });
            if let Some(g) = good {
                v["last_good"] = json!({ "wgsl": g["wgsl"], "slots": g["slots"] });
            }
            v
        }
    };
    if target.is_none() {
        let mut brief = summary.clone();
        brief.as_object_mut().map(|m| {
            m.remove("wgsl");
            m.remove("last_good")
        });
        let mut runs = RUNS.lock().unwrap();
        // count edits that did not compile (a rejected edit), as the owner's recipe validation does
        let rejected = runs.get(name).map_or(0, |r| r["rejected"].as_u64().unwrap_or(0) + u64::from(r["ok"] == true && summary["ok"] == false));
        brief["rejected"] = json!(rejected);
        runs.insert(name.to_string(), brief);
    }
    summary
}

/// The animated params' values at `t`, in slot order.
pub fn sample(tracks: &BTreeMap<String, Track>, slots: &[String], t: f64) -> Vec<f32> {
    slots.iter().map(|s| tracks.get(s).map_or(0.0, |tr| tr.sample(t)) as f32).collect()
}

fn install(lua: &Lua) -> mlua::Result<()> {
    let s = lua.create_table()?;
    // shader.compile(name, graph, catalog, target?, tracks?): tracks by `node:param` (timeline.<name>.tracks)
    s.set(
        "compile",
        lua.create_function(|lua, (name, graph, cat, target, tracks): (String, LuaValue, LuaValue, Option<String>, Option<LuaValue>)| {
            let (graph, cat): (Graph, Catalog) = (typed(lua, graph)?, typed(lua, cat)?);
            let tracks: BTreeMap<String, Track> = match tracks {
                Some(t) if !t.is_nil() => typed(lua, t)?,
                _ => BTreeMap::new(),
            };
            let animated: Vec<String> = tracks.keys().cloned().collect();
            lua.to_value(&compile_summary(&name, &graph, &cat, target.as_deref(), &animated))
        })?,
    )?;
    // shader.sample(tracks, slots, t): the uniform values for a compile's slots at time t
    s.set(
        "sample",
        lua.create_function(|lua, (tracks, slots, t): (LuaValue, Vec<String>, f64)| {
            let tracks: BTreeMap<String, Track> = if tracks.is_nil() { BTreeMap::new() } else { typed(lua, tracks)? };
            Ok(sample(&tracks, &slots, t))
        })?,
    )?;
    s.set(
        "snapshot",
        lua.create_function(|_, (wgsl, w, h, time, file, params): (String, u32, u32, f32, String, Option<Vec<f32>>)| {
            let png = gpu::render(&wgsl, w, h, time, &params.unwrap_or_default()).and_then(|px| gpu::png(&px, w, h)).map_err(mlua::Error::external)?;
            if let Some(dir) = std::path::Path::new(&file).parent() {
                std::fs::create_dir_all(dir).map_err(mlua::Error::external)?;
            }
            std::fs::write(&file, png).map_err(mlua::Error::external)
        })?,
    )?;
    lua.globals().set("shader", s)
}

fn hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// `{kind = "shader_preview", id, wgsl, time, params?}`: the shader at `time` with its animated params,
/// filling the node's box.
fn paint(node: &Value, s: &mut Surface) -> bool {
    if node.get("kind").and_then(Value::as_str) != Some("shader_preview") {
        return false;
    }
    let Some(wgsl) = node.get("wgsl").and_then(Value::as_str) else { return true };
    let id = node.get("id").and_then(Value::as_str).unwrap_or("preview").to_string();
    let time = num(node, "time", s.time as f32);
    let params: Vec<f32> = node.get("params").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_f64).map(|v| v as f32).collect()).unwrap_or_default();
    // at most 1 device pixel per logical pixel and 640 wide: a preview, not a render
    let res = s.scale.min(1.0).max(0.5);
    let (w, h) = (((s.rect.w * res) as u32).clamp(1, 640), ((s.rect.h * res) as u32).clamp(1, 640));
    let key = (hash(&format!("{wgsl}{params:?}")), (time * 1000.0) as u32, w, h);
    let mut frames = FRAMES.lock().unwrap();
    let fresh = frames.get(&id).is_some_and(|f| (f.0, f.1, f.2, f.3) == key);
    if !fresh {
        let Ok(px) = gpu::render(wgsl, w, h, time, &params) else { return true };
        let Some(mut pm) = Pixmap::new(w, h) else { return true };
        pm.data_mut().copy_from_slice(&px);
        frames.insert(id.clone(), (key.0, key.1, w, h, pm));
    }
    let pm = &frames[&id].4;
    let t = s.transform().pre_scale(s.rect.w / w as f32, s.rect.h / h as f32);
    let paint = PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() };
    s.pixmap.draw_pixmap(0, 0, pm.as_ref(), &paint, t, s.clip);
    true
}

#[cfg(test)]
mod tests;
