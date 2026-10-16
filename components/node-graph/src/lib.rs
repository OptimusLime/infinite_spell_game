//! The node-graph component's native half: the Luau `graph` table over `atelico_graph` (checks, connection
//! rules, dependency-ordered runs with a cache per graph) and the `graph_wires` node kind: cables painted with
//! tiny-skia (Luau cannot draw paths), each a polyline at right angles with rounded elbows (`points`) or a bezier
//! between its ends, and a dot where a bundle splits (`dots`). The canvas itself is Luau: `ui.node_graph`.
pub mod routing;
mod routing_lua;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use atelico_data::file::{self, FileError};
use atelico_timeline::TimelineFile;

use atelico_component::mlua::{self, Function, Lua, LuaSerdeExt, Value as LuaValue};
use atelico_component::atelico_paint::{Paint, Stroke};
use atelico_component::tiny_skia::{self, Color, LineCap, PathBuilder};
use atelico_component::{color, num, register, Component, On, Surface, Value};
use atelico_graph::{run, Cache, Catalog, Graph, Halt};
use serde_json::json;

register!(Component { name: "node-graph", lua: install, paint, draw, inspect, ..Component::NONE });

/// Each graph's cache, by the name its runs pass.
/// Luau evaluators return Luau values, so this is the one run whose outputs are JSON: the Luau boundary.
static CACHES: Mutex<BTreeMap<String, Cache<Value>>> = Mutex::new(BTreeMap::new());
/// Each graph's last run (order, counts, per-node status), for `/scene`.
static RUNS: Mutex<BTreeMap<String, Value>> = Mutex::new(BTreeMap::new());

fn inspect() -> Value {
    json!({ "runs": *RUNS.lock().unwrap() })
}

/// Luau writes empty tables as arrays; the graph's maps want objects.
fn maps(mut v: Value, keys: &[&str]) -> Value {
    for k in keys {
        if v.get(k).is_some_and(|x| x.as_array().is_some_and(Vec::is_empty)) {
            v[*k] = json!({});
        }
    }
    if let Some(nodes) = v.get_mut("nodes").and_then(Value::as_object_mut) {
        for n in nodes.values_mut() {
            if n.get("params").is_some_and(|x| x.as_array().is_some_and(Vec::is_empty)) {
                n["params"] = json!({});
            }
        }
    }
    v
}

fn graph_of(lua: &Lua, t: LuaValue) -> mlua::Result<Graph> {
    let v: Value = lua.from_value(t)?;
    Graph::from_value(&maps(v, &["nodes", "edges", "groups"])).map_err(mlua::Error::external)
}

fn catalog_of(lua: &Lua, t: LuaValue) -> mlua::Result<Catalog> {
    let v: Value = lua.from_value(t)?;
    serde_json::from_value(maps(v, &["casts"])).map_err(mlua::Error::external)
}

/// A graph file's timeline: `x.graph.yaml` → `x.timeline.yaml`.
pub fn timeline_path(graph: &Path) -> PathBuf {
    let name = graph.file_name().and_then(|n| n.to_str()).unwrap_or("graph.graph.yaml");
    graph.with_file_name(format!("{}.timeline.yaml", name.strip_suffix(".graph.yaml").unwrap_or(name)))
}

/// Saves a graph and its timeline (when it has one) as typed files at `path` and beside it.
pub fn save(path: &Path, graph: &Graph, timeline: Option<&TimelineFile>) -> Result<(), FileError> {
    file::save(path, graph)?;
    match timeline {
        Some(t) => file::save(&timeline_path(path), t),
        None => Ok(()),
    }
}

/// Reads a graph file and its timeline, each straight into its type; any error (file:line) loads nothing.
pub fn load(path: &Path) -> Result<(Graph, Option<TimelineFile>), FileError> {
    let graph = file::load(path)?;
    let t = timeline_path(path);
    let timeline = if t.is_file() { Some(file::load(&t)?) } else { None };
    Ok((graph, timeline))
}

/// Loads `path` into the project as graph `name` (graphs/<name>.graph.yaml and its timeline), which the
/// editor mounts: the store follows the files. Checked first, so a broken file changes nothing.
pub fn load_into(root: &Path, path: &Path, name: &str) -> Result<(Graph, Option<TimelineFile>), FileError> {
    let (graph, timeline) = load(path)?;
    save(&root.join(format!("graphs/{name}.graph.yaml")), &graph, timeline.as_ref())?;
    Ok((graph, timeline))
}

fn install(lua: &Lua) -> mlua::Result<()> {
    let g = lua.create_table()?;
    routing_lua::install(lua, &g)?;
    // graph.save(path, graph, timeline?) and graph.load(path, name) → ok, error ("file:line: ...")
    g.set(
        "save",
        lua.create_function(|lua, (path, graph, timeline): (String, LuaValue, Option<LuaValue>)| {
            let graph = graph_of(lua, graph)?;
            let timeline: Option<TimelineFile> = match timeline {
                Some(t) if !t.is_nil() => Some(lua.from_value(t)?),
                _ => None,
            };
            Ok(match save(Path::new(&path), &graph, timeline.as_ref()) {
                Ok(()) => (true, None),
                Err(e) => (false, Some(e.to_string())),
            })
        })?,
    )?;
    // graph.read(path) → graph, or nil and "file:line: ..." (an app's own graph data, not in the store)
    g.set(
        "read",
        lua.create_function(|lua, path: String| {
            Ok(match file::load::<Graph>(Path::new(&path)) {
                Ok(graph) => (lua.to_value(&graph)?, None),
                Err(e) => (LuaValue::Nil, Some(e.to_string())),
            })
        })?,
    )?;
    g.set(
        "load",
        lua.create_function(|_, (path, name): (String, String)| {
            Ok(match load_into(Path::new("."), Path::new(&path), &name) {
                Ok(_) => (true, None),
                Err(e) => (false, Some(e.to_string())),
            })
        })?,
    )?;
    g.set(
        "check",
        lua.create_function(|lua, (graph, cat): (LuaValue, LuaValue)| {
            let issues = graph_of(lua, graph)?.issues(&catalog_of(lua, cat)?);
            lua.to_value(&issues)
        })?,
    )?;
    g.set(
        "can_connect",
        lua.create_function(|lua, (graph, cat, from, to): (LuaValue, LuaValue, String, String)| {
            Ok(match graph_of(lua, graph)?.can_connect(&catalog_of(lua, cat)?, &from, &to) {
                Ok(()) => (true, None),
                Err(e) => (false, Some(e)),
            })
        })?,
    )?;
    g.set(
        "order",
        lua.create_function(|lua, graph: LuaValue| {
            Ok(match graph_of(lua, graph)?.order() {
                Ok(o) => (lua.to_value(&o)?, LuaValue::Nil),
                Err(c) => (LuaValue::Nil, lua.to_value(&c)?),
            })
        })?,
    )?;
    // graph.run(name, graph, catalog, eval?, targets?): eval(e) gets {id, type, params, inputs} and returns the
    // node's outputs by port (error() fails the node), or nil and {running = "what is under way"} while its work
    // goes on (nothing is kept; the next run asks again); without eval a run only orders and keys.
    g.set(
        "run",
        lua.create_function(|lua, (name, graph, cat, eval, targets): (String, LuaValue, LuaValue, Option<Function>, Option<Vec<String>>)| {
            let (graph, cat) = (graph_of(lua, graph)?, catalog_of(lua, cat)?);
            let mut cache = CACHES.lock().unwrap().remove(&name).unwrap_or_default();
            let targets: Option<Vec<&str>> = targets.as_ref().map(|t| t.iter().map(String::as_str).collect());
            let r = run(&graph, &cat, targets.as_deref(), &mut cache, |e: &atelico_graph::Eval<Value>| -> Result<atelico_graph::Outputs<Value>, Halt> {
                let Some(f) = &eval else { return Ok(BTreeMap::new()) };
                let arg = lua.to_value(&json!({ "id": e.id, "type": e.ty.name, "params": e.params, "inputs": e.inputs })).map_err(|e| e.to_string())?;
                let (out, halt): (LuaValue, LuaValue) = f.call(arg).map_err(|e| e.to_string().lines().next().unwrap_or("").to_string())?;
                if out.is_nil() {
                    let halt: Value = lua.from_value(halt).map_err(|e| e.to_string())?;
                    if let Some(what) = halt.get("running").and_then(Value::as_str) {
                        return Err(Halt::Running(what.to_string()));
                    }
                }
                let out: Value = lua.from_value(out).map_err(|e| e.to_string())?;
                Ok(out.as_object().map(|m| m.clone().into_iter().collect()).unwrap_or_default())
            });
            CACHES.lock().unwrap().insert(name.clone(), cache);
            let at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64());
            let summary = json!({
                "order": r.order, "ran": r.ran, "cached": r.cached, "at": at,
                "nodes": r.nodes.iter().map(|(k, n)| {
                    let mut v = json!(n.status);
                    v["key"] = json!(n.key);
                    (k.clone(), v)
                }).collect::<serde_json::Map<_, _>>(),
            });
            RUNS.lock().unwrap().insert(name, summary);
            lua.to_value(&r)
        })?,
    )?;
    lua.globals().set("graph", g)
}

/// Control distance for a wire leaving rightwards and arriving from the left (xyflow's: half the gap going
/// forward, a loop-back that grows with the square root of the overlap going backward), at least `min`.
pub fn bend(dx: f32, min: f32) -> f32 {
    let off = if dx >= 0.0 { 0.5 * dx } else { 0.25 * 25.0 * (-dx).sqrt() * 2.0 };
    off.max(min)
}

fn point(v: &Value) -> Option<(f32, f32)> {
    Some((v.get("x")?.as_f64()? as f32, v.get("y")?.as_f64()? as f32))
}

/// A polyline through `pts` with each corner rounded by up to `r` (less where a segment is short).
pub fn elbows(pts: &[(f32, f32)], r: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let (first, last) = (*pts.first()?, *pts.last()?);
    pb.move_to(first.0, first.1);
    for w in pts.windows(3) {
        let (a, b, c) = (w[0], w[1], w[2]);
        let (d1, d2) = (((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt(), ((c.0 - b.0).powi(2) + (c.1 - b.1).powi(2)).sqrt());
        let k = r.min(d1 / 2.0).min(d2 / 2.0);
        if d1 == 0.0 || d2 == 0.0 || k <= 0.0 {
            pb.line_to(b.0, b.1);
            continue;
        }
        let p = (b.0 + (a.0 - b.0) * k / d1, b.1 + (a.1 - b.1) * k / d1);
        let q = (b.0 + (c.0 - b.0) * k / d2, b.1 + (c.1 - b.1) * k / d2);
        pb.line_to(p.0, p.1);
        pb.quad_to(b.0, b.1, q.0, q.1);
    }
    pb.line_to(last.0, last.1);
    pb.finish()
}

/// Painted on a CPU picture: the same drawing, on a CPU canvas over it (and its clip).
fn paint(node: &Value, s: &mut Surface) -> bool {
    let mut c = atelico_paint_cpu::CpuCanvas::new(&mut *s.pixmap, s.scale, s.clip);
    draw(node, &mut On { canvas: &mut c, rect: s.rect, assets: s.assets, time: s.time })
}

/// Drawn through the paint interface (any backend).
fn draw(node: &Value, on: &mut On) -> bool {
    match node.get("kind").and_then(Value::as_str) {
        Some("graph_wires") => wires(node, on),
        Some("graph_grid") => grid(node, on),
        _ => return false,
    }
    true
}

/// The canvas's grid (`graph_grid`): lines every `cell` graph px, placed by the view (`x`, `y`, `zoom`: screen =
/// graph * zoom + offset, from the canvas's corner), a `major_color` one every `major` cells; one px wide, or two
/// for a major line (logical px). Cells under 6 px on screen are left out (too dense to read); the major lines stay.
fn grid(node: &Value, s: &mut On) {
    // (painted in the canvas's own space: `transform` places it)
    let r = atelico_component::Rect { x: 0.0, y: 0.0, w: s.rect.w, h: s.rect.h };
    let (ox, oy, zoom) = (num(node, "x", 0.0), num(node, "y", 0.0), num(node, "zoom", 1.0).max(0.01));
    let (cell, major) = (num(node, "cell", 24.0).max(1.0), num(node, "major", 5.0).max(1.0).round() as i64);
    let (minor_c, major_c) = (color(node, "minor_color").unwrap_or(Color::from_rgba8(0, 0, 0, 20)), color(node, "major_color").unwrap_or(Color::from_rgba8(0, 0, 0, 40)));
    let step = cell * zoom;
    let dense = step < 6.0;
    let transform = s.transform();
    let line = |pb: &mut PathBuilder, a: (f32, f32), b: (f32, f32)| {
        pb.move_to(a.0, a.1);
        pb.line_to(b.0, b.1);
    };
    let (mut minor, mut majors) = (PathBuilder::new(), PathBuilder::new());
    let first = ((r.x - ox) / step).floor() as i64;
    let last = ((r.x + r.w - ox) / step).ceil() as i64;
    for k in first..=last {
        let x = ox + k as f32 * step;
        let strong = k.rem_euclid(major) == 0;
        if strong {
            line(&mut majors, (x, r.y), (x, r.y + r.h));
        } else if !dense {
            line(&mut minor, (x, r.y), (x, r.y + r.h));
        }
    }
    let first = ((r.y - oy) / step).floor() as i64;
    let last = ((r.y + r.h - oy) / step).ceil() as i64;
    for k in first..=last {
        let y = oy + k as f32 * step;
        let strong = k.rem_euclid(major) == 0;
        if strong {
            line(&mut majors, (r.x, y), (r.x + r.w, y));
        } else if !dense {
            line(&mut minor, (r.x, y), (r.x + r.w, y));
        }
    }
    for (pb, c, width) in [(minor, minor_c, 1.0), (majors, major_c, 2.0)] {
        if let Some(path) = pb.finish() {
            s.canvas.stroke_path(&path, &Paint::Solid(c), &Stroke { width, ..Stroke::default() }, transform, true);
        }
    }
}

fn wires(node: &Value, s: &mut On) {
    let transform = s.transform();
    let zoom = num(node, "zoom", 1.0);
    let corner = num(node, "corner", 8.0) * zoom;
    for w in node.get("wires").and_then(Value::as_array).into_iter().flatten() {
        let pts: Vec<(f32, f32)> = w.get("points").and_then(Value::as_array).into_iter().flatten().filter_map(point).collect();
        let path = if pts.len() >= 2 {
            elbows(&pts, corner)
        } else {
            let (Some(a), Some(b)) = (w.get("from").and_then(point), w.get("to").and_then(point)) else { continue };
            let off = bend(b.0 - a.0, 30.0 * zoom);
            let mut pb = PathBuilder::new();
            pb.move_to(a.0, a.1);
            pb.cubic_to(a.0 + off, a.1, b.0 - off, b.1, b.0, b.1);
            pb.finish()
        };
        let Some(path) = path else { continue };
        let width = num(w, "width", 2.0) * zoom.max(0.5);
        let c = color(w, "color").unwrap_or(Color::WHITE);
        let mut stroke = Stroke { width, line_cap: LineCap::Butt, ..Stroke::default() };
        if w.get("dash").and_then(Value::as_bool) == Some(true) {
            stroke.dash = Some((vec![6.0 * zoom, 5.0 * zoom], 0.0));
        }
        if w.get("glow").and_then(Value::as_bool) == Some(true) {
            let mut halo = c;
            halo.set_alpha(0.28);
            let wide = Stroke { width: width * 3.5, line_cap: LineCap::Round, ..Stroke::default() };
            s.canvas.stroke_path(&path, &Paint::Solid(halo), &wide, transform, true);
        }
        s.canvas.stroke_path(&path, &Paint::Solid(c), &stroke, transform, true);
    }
    // a split's dot, in the first cable's color, a little wider than a cable
    let dot = node.get("wires").and_then(Value::as_array).and_then(|w| w.first()).and_then(|w| color(w, "color")).unwrap_or(Color::BLACK);
    for d in node.get("dots").and_then(Value::as_array).into_iter().flatten() {
        let Some((x, y)) = point(d) else { continue };
        if let Some(circle) = PathBuilder::from_circle(x, y, 3.5 * zoom.max(0.5)) {
            s.canvas.fill_path(&circle, &Paint::Solid(dot), transform, true);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use atelico_component::mlua::Table;

    #[test]
    fn wires_bend_forward_by_half_the_gap_and_loop_back_when_reversed() {
        assert_eq!(bend(200.0, 30.0), 100.0);
        assert_eq!(bend(20.0, 30.0), 30.0);
        assert!(bend(-100.0, 30.0) > 60.0, "a backward wire loops out");
    }

    #[test]
    fn an_elbow_is_rounded_but_never_past_half_a_short_segment() {
        let path = elbows(&[(0.0, 0.0), (10.0, 0.0), (10.0, 100.0)], 8.0).unwrap();
        let b = path.bounds();
        assert_eq!((b.left(), b.top(), b.right(), b.bottom()), (0.0, 0.0, 10.0, 100.0));
        assert!(elbows(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0)], 8.0).is_some(), "a short segment still draws");
    }

    #[test]
    fn luau_empty_tables_become_maps() {
        let v = maps(json!({ "nodes": { "a": { "type": "x", "params": [] } }, "edges": [], "groups": [] }), &["nodes", "edges", "groups"]);
        assert!(Graph::from_value(&v).is_ok());
    }

    #[test]
    fn a_run_from_luau_uses_the_cache_and_reports_status() {
        let lua = Lua::new();
        install(&lua).unwrap();
        let r: Table = lua
            .load(
                r#"
                local cat = { name = "t", types = { { name = "n", outputs = { { name = "out", type = "float" } }, params = { { name = "v", kind = "number", default = 1 } } },
                    { name = "add", inputs = { { name = "a", type = "float" } }, outputs = { { name = "out", type = "float" } } } } }
                local g = { nodes = { a = { type = "n", x = 0, y = 0 }, s = { type = "add", x = 1, y = 0 } }, edges = { ["s:a"] = "a:out" }, groups = {} }
                local function eval(e) if e.type == "n" then return { out = e.params.v * 2 } else return { out = e.inputs.a + 1 } end end
                graph.run("t", g, cat, eval)
                return graph.run("t", g, cat, eval)
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(r.get::<i64>("cached").unwrap(), 2);
        let s: Table = r.get::<Table>("nodes").unwrap().get("s").unwrap();
        assert_eq!(s.get::<String>("status").unwrap(), "cached");
        assert_eq!(s.get::<Table>("outputs").unwrap().get::<f64>("out").unwrap(), 3.0);
    }
}
