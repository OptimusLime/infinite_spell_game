//! A shader graph → one WGSL fragment shader. Each output of a node class carries a WGSL expression template
//! whose `{name}`s are the node's inputs (the wired source's variable, cast to the input's type, else the port's
//! default) and params (WGSL literals, or `u.p[i].x` for a param a timeline track animates, so keyframes never
//! recompile). A bypassed node gives each output's declared identity. The graph runs in dependency order with
//! the node-graph cache: a node whose key is unchanged reuses its declaration, and identical subgraphs share a
//! variable (the key is the content). The graph becomes `fn graph(uv, time, seed) -> vec4`, drawn once per
//! instance (seed, phase, placement) and composited over.

use atelico_graph::{run_as, Cache, Catalog, Eval, Graph, Instance, Outputs, ParamValue, Run, Status, Style};

/// What one node output compiles to: the declaration that computes it and the variable holding it.
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub decl: String,
    pub var: String,
    /// float | vec2 | color | rgba
    pub ty: String,
}

pub struct Shader {
    pub wgsl: String,
    /// Where the shader's color comes from: the output node, or a node previewed on its own.
    pub target: String,
    pub run: Run<Expr>,
    /// The animated params, in uniform order (`u.p[i].x`), as `node:param`.
    pub slots: Vec<String>,
}

/// Why a graph did not compile, and the run as far as it got (each node's status).
#[derive(Debug)]
pub struct Failure {
    pub error: String,
    pub run: Option<Run<Expr>>,
}

impl From<String> for Failure {
    fn from(error: String) -> Self {
        Self { error, run: None }
    }
}

impl From<&str> for Failure {
    fn from(error: &str) -> Self {
        Self { error: error.into(), run: None }
    }
}

pub const PRELUDE: &str = include_str!("prelude.wgsl");

/// WGSL helpers another package gives every graph (its node types' code calls them): the fire package's flame
/// stages, say. `shader_forge::helpers!("flame.wgsl")` registers a file beside the package's Cargo.toml, as the
/// crate's `SHADER_FORGE_HELPERS`; a binary that compiles graphs keeps it linked with `black_box` on it.
pub struct Helpers {
    pub file: &'static str,
    pub wgsl: &'static str,
}

#[atelico_component::linkme::distributed_slice]
#[linkme(crate = atelico_component::linkme)]
pub static HELPERS: [Helpers];

#[macro_export]
macro_rules! helpers {
    ($file:literal) => {
        #[$crate::compile::linkme::distributed_slice($crate::compile::HELPERS)]
        #[linkme(crate = $crate::compile::linkme)]
        pub static SHADER_FORGE_HELPERS: $crate::compile::Helpers = $crate::compile::Helpers { file: concat!(env!("CARGO_PKG_NAME"), "/", $file), wgsl: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file)) };
    };
}
pub use atelico_component::linkme;

/// The look API's stand-ins (look graphs call the renderer's `wl_*` post stages; standing alone they draw a test
/// card): one line each, every line starting `fn wl_`.
pub const LOOK_API: &str = include_str!("look_api.wgsl");

/// The prelude with the look API's stand-ins and every linked package's helpers after it.
pub fn prelude() -> String {
    let mut all: Vec<&Helpers> = HELPERS.iter().collect();
    all.sort_by_key(|h| h.file);
    let mut out = PRELUDE.to_string();
    out.push_str(LOOK_API);
    for h in all {
        out.push_str(&format!("\n// helpers from {}\n{}", h.file, h.wgsl));
    }
    out
}
/// Animated params per shader (the uniform array's length).
pub const SLOTS: usize = 16;

fn wgsl_type(ty: &str) -> &'static str {
    match ty {
        "vec2" => "vec2<f32>",
        "color" => "vec3<f32>",
        "rgba" => "vec4<f32>",
        _ => "f32",
    }
}

/// `expr` of type `from` as a `to`; floats spread, vectors truncate or pad, opaque unless told.
fn cast(expr: &str, from: &str, to: &str) -> String {
    match (from, to) {
        (a, b) if a == b => expr.to_string(),
        ("float", "vec2") => format!("vec2<f32>({expr})"),
        ("float", "color") => format!("vec3<f32>({expr})"),
        ("float", "rgba") => format!("vec4<f32>(vec3<f32>({expr}), 1.0)"),
        ("vec2", "color") => format!("vec3<f32>({expr}, 0.0)"),
        ("vec2", "rgba") => format!("vec4<f32>({expr}, 0.0, 1.0)"),
        ("color", "rgba") => format!("vec4<f32>({expr}, 1.0)"),
        ("color", "vec2") => format!("({expr}).xy"),
        ("rgba", "color") => format!("({expr}).rgb"),
        ("rgba", "float") => format!("({expr}).a"),
        ("vec2", "float") => format!("({expr}).x"),
        ("color", "float") => format!("dot({expr}, vec3<f32>(0.299, 0.587, 0.114))"),
        _ => expr.to_string(),
    }
}

fn float(n: f64) -> String {
    let s = format!("{n}");
    if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("NaN") {
        s
    } else {
        format!("{s}.0")
    }
}

/// A param as a WGSL literal: numbers as floats, `#rrggbb` (sRGB, made linear) and HDR triples (linear) as vec3s,
/// 2-vectors as vec2s,
/// choices as written (they name prelude functions).
pub fn literal(v: &ParamValue) -> String {
    match v {
        ParamValue::Number(n) => float(*n),
        ParamValue::Bool(b) => b.to_string(),
        ParamValue::Text(t) if t.starts_with('#') => {
            let h = &t[1..];
            // a hex colour is what one sees: sRGB, into the graph's linear light
            let byte = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0) as f64 / 255.0;
            let linear = |x: f64| if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) };
            let c = |i| float((linear(byte(i)) * 10000.0).round() / 10000.0);
            format!("vec3<f32>({}, {}, {})", c(0), c(2), c(4))
        }
        ParamValue::Text(t) => t.clone(),
        ParamValue::Vector(v) => format!("vec{}<f32>({})", v.len(), v.iter().map(|x| float(*x)).collect::<Vec<_>>().join(", ")),
    }
}

/// Fills `{name}` holes from `lookup`.
fn fill(template: &str, lookup: impl Fn(&str) -> Option<String>) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = rest[open..].find('}').ok_or("unclosed { in code")? + open;
        let name = &rest[open + 1..close];
        out.push_str(&lookup(name).ok_or_else(|| format!("code names {{{name}}}, which is no input or param"))?);
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn var_name(id: &str, port: &str) -> String {
    let clean = |s: &str| s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>();
    format!("n_{}_{}", clean(id), clean(port))
}

/// The shader style of the graph core: a node compiles to a WGSL declaration of each of its outputs.
pub struct ShaderStyle;

impl Style for ShaderStyle {
    type Value = Expr;
    fn eval(&mut self, e: &Eval<Expr>) -> Result<Outputs<Expr>, String> {
        eval(e, e.bypass)
    }
}

fn eval(e: &Eval<Expr>, bypass: bool) -> Result<Outputs<Expr>, String> {
    let lookup = |name: &str| {
        if let Some(port) = e.ty.input(name) {
            return Some(match e.inputs.get(name) {
                Some(x) => cast(&x.var, &x.ty, &port.ty),
                None => port.default.clone().unwrap_or_else(|| cast("0.0", "float", &port.ty)),
            });
        }
        e.params.get(name).map(literal)
    };
    let mut out = Outputs::new();
    for (i, port) in e.ty.outputs.iter().enumerate() {
        let template = if bypass {
            port.identity.as_deref().ok_or_else(|| format!("{} cannot be bypassed: {} has no identity", e.ty.title, port.name))?
        } else {
            port.code.as_deref().or(if i == 0 { e.ty.code.as_deref() } else { None }).ok_or_else(|| format!("{} has no code for {}", e.ty.title, port.name))?
        };
        let expr = fill(template, lookup)?;
        let var = var_name(e.id, &port.name);
        out.insert(port.name.clone(), Expr { decl: format!("    let {var}: {} = {expr};", wgsl_type(&port.ty)), var, ty: port.ty.clone() });
    }
    Ok(out)
}

/// The graph with animated params as uniform reads; the slots in order. Only number params animate.
fn with_slots(graph: &Graph, cat: &Catalog, animated: &[String]) -> Result<(Graph, Vec<String>), String> {
    let mut g = graph.clone();
    let mut slots = Vec::new();
    for key in animated {
        let (node, param) = key.split_once(':').ok_or_else(|| format!("track {key}: want node:param"))?;
        let Some(n) = g.nodes.get_mut(node) else { continue };
        let kind = cat.get(&n.ty).and_then(|t| t.param(param)).map(|p| p.kind.clone());
        if kind.as_deref() != Some("number") {
            return Err(format!("track {key}: {} has no number param {param}", n.ty));
        }
        if slots.len() == SLOTS {
            return Err(format!("more than {SLOTS} animated params"));
        }
        n.params.insert(param.into(), ParamValue::Text(format!("u.p[{}].x", slots.len())));
        slots.push(key.clone());
    }
    Ok((g, slots))
}

/// Compiles the graph's output node (`target` None), or any node's first output (a preview of that stage).
/// `animated` lists the `node:param`s a timeline track drives.
pub fn compile(graph: &Graph, cat: &Catalog, target: Option<&str>, animated: &[String], cache: &mut Cache<Expr>) -> Result<Shader, Failure> {
    let target = match target {
        Some(t) => t.to_string(),
        None => graph.nodes.iter().find(|(_, n)| n.ty.ends_with("output")).map(|(id, _)| id.clone()).ok_or("the graph has no output node")?,
    };
    if !graph.nodes.contains_key(&target) {
        return Err(format!("no node {target}").into());
    }
    let (g, slots) = with_slots(graph, cat, animated)?;
    let run = run_as(&g, cat, Some(&[target.as_str()]), cache, &mut ShaderStyle);
    let result = &run.nodes[&target];
    let color = match &result.status {
        // the first output as the node type declares them (the outputs are kept by name)
        Status::Ran | Status::Cached => {
            let first = graph.nodes.get(&target).and_then(|n| cat.get(&n.ty)).and_then(|t| t.outputs.first()).map(|o| o.name.clone());
            first.and_then(|f| result.outputs.get(&f)).or_else(|| result.outputs.values().next()).map(|x| cast(&x.var, &x.ty, "rgba"))
        }
        _ => None,
    };
    let Some(color) = color else {
        let why = match &result.status {
            Status::Failed(e) | Status::Blocked(e) => format!("{target}: {e}"),
            Status::Cycle => format!("{target} is on a cycle"),
            _ => format!("{target} gives no output"),
        };
        return Err(Failure { error: why, run: Some(run) });
    };
    let mut body = Vec::new();
    for id in &run.order {
        for x in run.nodes[id].outputs.values() {
            if !body.contains(&x.decl) {
                body.push(x.decl.clone());
            }
        }
    }
    let one = [Instance::default()];
    let instances = if graph.instances.is_empty() { &one[..] } else { &graph.instances[..] };
    let draws: Vec<String> = instances
        .iter()
        .map(|i| {
            // placed by its base (bottom center) and scaled about it
            format!(
                "    acc = over(acc, graph((in.uv - vec2<f32>({}, {})) / {} + vec2<f32>(0.5, 0.0), u.time + {}, {}));",
                float(0.5 + i.x), float(i.y), float(i.scale.max(0.01)), float(i.phase), float(i.seed)
            )
        })
        .collect();
    let prelude = prelude();
    let wgsl = format!(
        "{prelude}\nfn graph(uv_in: vec2<f32>, time: f32, seed: f32) -> vec4<f32> {{\n    let uv = uv_in;\n{}\n    return {color};\n}}\n\n@fragment fn fs(in: VOut) -> @location(0) vec4<f32> {{\n    var acc = vec4<f32>(0.0);\n{}\n    return acc;\n}}\n",
        body.join("\n"),
        draws.join("\n")
    );
    if let Err(e) = validate(&wgsl) {
        return Err(Failure { error: e, run: Some(run) });
    }
    Ok(Shader { wgsl, target, run, slots })
}

/// naga's parse and validation, so a bad template is a node error, not a GPU panic.
pub fn validate(wgsl: &str) -> Result<(), String> {
    let module = naga::front::wgsl::parse_str(wgsl).map_err(|e| e.emit_to_string(wgsl))?;
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::default()).validate(&module).map_err(|e| format!("{e:?}"))?;
    Ok(())
}
