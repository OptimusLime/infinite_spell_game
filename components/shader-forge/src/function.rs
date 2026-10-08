//! Function mode: a graph compiled not to a whole preview shader but to one plain WGSL function,
//! `fn sf_<name>(uv: vec2<f32>, time: f32, seed: f32) -> vec4<f32>` (straight rgba, uv (0,0) bottom-left to
//! (1,1) top-right), with the prelude's helpers it may call, every name prefixed so it sits beside anything else:
//! no bindings, no entry points. Another shader (the renderer's) calls it: a fire's look is authored as a graph.
//! Params compile to literals (an edit recompiles the function; the renderer rebuilds its shader).
use std::path::Path;

use atelico_graph::{Cache, Catalog, Graph};

use crate::compile::{compile, prelude, validate, Failure};

/// The function's name for graph `name`.
pub fn name_of(graph: &str) -> String {
    format!("sf_{}", graph.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>())
}

/// The graph's output as a plain function named `name_of(name)`, with its prefixed helpers.
pub fn function(graph: &Graph, cat: &Catalog, name: &str) -> Result<String, Failure> {
    let mut cache = Cache::default();
    let shader = compile(graph, cat, None, &[], &mut cache)?;
    let wgsl = &shader.wgsl;
    let start = wgsl.find("fn graph(").ok_or("the compiled shader has no graph function")?;
    let end = wgsl[start..].find("\n@fragment").map_or(wgsl.len(), |e| start + e);
    let body = &wgsl[start..end];
    // the prelude's plain helpers: everything after its vertex entry point, but the look API's stand-ins (the
    // renderer that calls this function has the real `wl_*` stages; calls to them keep their names)
    let prelude = prelude();
    let all = &prelude[prelude.find("\nfn ").ok_or("the prelude has no helpers")?..];
    let stand_ins: String = all.lines().filter(|l| l.starts_with("fn wl_")).map(|l| format!("{l}\n")).collect();
    let helpers: String = all.lines().filter(|l| !l.starts_with("fn wl_")).map(|l| format!("{l}\n")).collect();
    let helpers = helpers.as_str();
    let names: Vec<&str> = helpers.lines().filter_map(|l| l.strip_prefix("fn ")).filter_map(|l| l.split('(').next()).chain(["graph"]).collect();
    let prefix = name_of(name);
    let rename = |text: &str| {
        let mut out = String::with_capacity(text.len());
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < text.len() {
            let word_start = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            let hit = word_start.then(|| names.iter().find(|n| text[i..].starts_with(&format!("{n}(")))).flatten();
            match hit {
                Some(n) if *n == "graph" => {
                    out.push_str(&prefix);
                    i += n.len();
                }
                Some(n) => {
                    out.push_str(&format!("{prefix}_{n}"));
                    i += n.len();
                }
                None => {
                    let c = text[i..].chars().next().unwrap();
                    out.push(c);
                    i += c.len_utf8();
                }
            }
        }
        out
    };
    let out = format!("// the {name} graph (Shader Forge, function mode)\n{}\n{}\n", rename(helpers), rename(body));
    if out.contains("u.") {
        return Err(format!("{name}: a graph in function mode cannot read the preview's uniforms (animated params)").into());
    }
    // (checked with the stand-ins beside it, as the renderer's own stand in for them)
    let uses_look = out.contains("wl_");
    validate(&if uses_look { format!("{out}\n{stand_ins}") } else { out.clone() }).map_err(|e| Failure { error: e, run: None })?;
    Ok(out)
}

/// Shader Forge's own node catalog (nodes/shader.nodes.yaml).
pub fn catalog() -> Result<Catalog, String> {
    serde_yaml_ng::from_str(include_str!("../nodes/shader.nodes.yaml")).map_err(|e| format!("shader.nodes.yaml: {e}"))
}

/// A graph file (graphs/<name>.graph.yaml).
pub fn load(path: &Path) -> Result<Graph, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: serde_json::Value = serde_yaml_ng::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Graph::from_value(&v).map_err(|e| format!("{}: {e}", path.display()))
}
