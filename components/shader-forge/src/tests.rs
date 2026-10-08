//! Shader Forge on a small glow graph of its own generic nodes (compiles, renders, recompiles only what an edit
//! changes, previews one node, reports a bad template). The one fire's graph (graphs/fire.graph.yaml, whose stage
//! nodes are the fire package's) is tested there: packages/fire/src/tests.rs.
use atelico_graph::{Cache, Catalog, Graph, ParamValue, Status};

use crate::compile::{compile, Expr};
use crate::gpu;

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn catalog() -> Catalog {
    serde_yaml_ng::from_str(&std::fs::read_to_string(root().join("nodes/shader.nodes.yaml")).unwrap()).unwrap()
}

/// A soft glow rising through turbulence: every node Shader Forge's own.
const GLOW: &str = r#"
name: Glow
types: shader
nodes:
  uv: {type: uv, x: 0, y: 150}
  time: {type: time, x: 0, y: 330, params: {speed: 1}}
  rise: {type: scroll, x: 230, y: 20, params: {dy: -1.2}}
  turbulence: {type: noise, x: 460, y: 0, params: {scale: 3, stretch: 1.5, octaves: 5}}
  sway: {type: warp, x: 690, y: 60, params: {strength: 0.2}}
  shape: {type: gradient, x: 230, y: 640, params: {direction: radial}}
  body: {type: multiply, x: 460, y: 640, params: {gain: 1.5}}
  cut: {type: smoothstep, x: 690, y: 640, params: {low: 0.05, high: 0.6}}
  colors: {type: ramp, x: 920, y: 640}
  out: {type: output, x: 1150, y: 640}
edges:
  rise:uv: uv:uv
  rise:t: time:t
  turbulence:uv: rise:uv
  sway:uv: uv:uv
  sway:amount: turbulence:value
  shape:uv: sway:uv
  body:a: shape:value
  body:b: turbulence:value
  cut:x: body:out
  colors:value: cut:out
  out:color: colors:color
"#;

fn graph() -> Graph {
    let v: serde_json::Value = serde_yaml_ng::from_str(GLOW).unwrap();
    Graph::from_value(&v).unwrap()
}

fn render(wgsl: &str, w: u32, h: u32, t: f32, params: &[f32]) -> Option<Vec<u8>> {
    match gpu::render(wgsl, w, h, t, params) {
        Ok(px) => Some(px),
        Err(e) if e.contains("no GPU") => {
            eprintln!("skipped: {e}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

/// Mean brightness (premultiplied, so alpha counts) of a region given in fractions of the image.
fn luma(px: &[u8], w: u32, h: u32, x: (f32, f32), y: (f32, f32)) -> f32 {
    let (mut sum, mut n) = (0.0, 0.0);
    for j in (y.0 * h as f32) as u32..(y.1 * h as f32) as u32 {
        for i in (x.0 * w as f32) as u32..(x.1 * w as f32) as u32 {
            let k = ((j * w + i) * 4) as usize;
            sum += (px[k] as f32 * 0.3 + px[k + 1] as f32 * 0.59 + px[k + 2] as f32 * 0.11) / 255.0;
            n += 1.0;
        }
    }
    sum / n
}

#[test]
fn the_glow_graph_compiles_to_valid_wgsl() {
    let (g, cat) = (graph(), catalog());
    assert!(g.issues(&cat).is_empty(), "{:?}", g.issues(&cat));
    let s = compile(&g, &cat, None, &[], &mut Cache::<Expr>::default()).unwrap();
    assert!(s.wgsl.contains("fbm(") && s.wgsl.contains("grad_radial("), "{}", s.wgsl);
    assert_eq!(s.run.ran, g.nodes.len());
}

#[test]
fn a_param_edit_recompiles_only_its_descendants() {
    let (mut g, cat) = (graph(), catalog());
    let mut cache = Cache::<Expr>::default();
    compile(&g, &cat, None, &[], &mut cache).unwrap();
    g.nodes.get_mut("colors").unwrap().params.insert("c2".into(), ParamValue::Text("#22aaff".into()));
    let s = compile(&g, &cat, None, &[], &mut cache).unwrap();
    let ran: Vec<&str> = s.run.nodes.iter().filter(|(_, n)| n.status == Status::Ran).map(|(k, _)| k.as_str()).collect();
    assert_eq!(ran, ["colors", "out"]);
}

#[test]
fn a_node_compiles_on_its_own_for_a_preview() {
    let (g, cat) = (graph(), catalog());
    let s = compile(&g, &cat, Some("turbulence"), &[], &mut Cache::<Expr>::default()).unwrap();
    assert_eq!(s.run.order, ["time", "uv", "rise", "turbulence"]);
}

#[test]
fn a_bad_template_is_a_compile_error_not_a_panic() {
    let (g, mut cat) = (graph(), catalog());
    cat.types.iter_mut().find(|t| t.name == "noise").unwrap().code = Some("fbm({uv} *** 2.0, 4.0)".into());
    assert!(compile(&g, &cat, None, &[], &mut Cache::<Expr>::default()).is_err());
}

#[test]
fn the_glow_renders_bright_in_its_middle_and_dark_at_its_corners() {
    let (g, cat) = (graph(), catalog());
    let s = compile(&g, &cat, None, &[], &mut Cache::<Expr>::default()).unwrap();
    let (w, h) = (160, 240);
    for t in [0.0, 1.3, 2.7] {
        let Some(px) = render(&s.wgsl, w, h, t, &[]) else { return };
        assert!(luma(&px, w, h, (0.45, 0.55), (0.45, 0.55)) > 0.3, "t={t}: middle");
        assert!(luma(&px, w, h, (0.0, 0.05), (0.0, 0.05)) < 0.08, "t={t}: corner");
    }
}
