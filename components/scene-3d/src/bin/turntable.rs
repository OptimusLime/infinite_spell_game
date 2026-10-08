//! Grown shapes as a turntable sheet, drawn by the engine's renderer on the isolation stage: one row of seeds
//! close up, turning, above the same seeds in a row from further away (or each seed from straight above, or from
//! 40 degrees up, for kinds such as the island's terrain). Writes DIR/sheet.png (the first frame) and
//! DIR/frames/NNNN.png (one full turn).
//!
//! ```text
//! turntable <kind> [--view turntable|top|three-quarter] [--seeds 8] [--frames 96] --out DIR
//! ```
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use component_scene_3d::nodes::{Camera, Item, World};
use component_scene_3d::parts::Grown;
use component_scene_3d::world;
use glam::{vec3, Vec3};

#[derive(Clone, Copy, PartialEq, ValueEnum)]
enum View {
    /// Close up from the side, turning, above a row from further away.
    Turntable,
    /// Straight down: a map.
    Top,
    /// From 40 degrees up, like an aerial shot.
    ThreeQuarter,
}

#[derive(Parser)]
struct Args {
    /// A registered grown kind (palm, bush, tree, sea-cliff, sea-stack, boulders, island-terrain, ...).
    kind: String,
    #[arg(long, value_enum, default_value = "turntable")]
    view: View,
    #[arg(long, default_value_t = 8)]
    seeds: u32,
    #[arg(long, default_value_t = 96)]
    frames: u32,
    #[arg(long)]
    out: PathBuf,
}

const HERO: (u32, u32) = (360, 540);
const FAR_HEIGHT: u32 = 540;

fn paste(sheet: &mut [u8], sheet_w: u32, tile: &[u8], (w, h): (u32, u32), x: u32, y: u32) {
    for row in 0..h {
        let from = (row * w * 4) as usize;
        let to = (((y + row) * sheet_w + x) * 4) as usize;
        sheet[to..to + (w * 4) as usize].copy_from_slice(&tile[from..from + (w * 4) as usize]);
    }
}

/// `items` on the stage seen from `eye` looking at `look` with `fov`, at `time`, w x h.
fn still(items: Vec<Item>, (eye, look, fov): (Vec3, Vec3, f32), time: f32, (w, h): (u32, u32)) -> Result<Vec<u8>, String> {
    let camera = Camera { x: eye.x as f64, y: eye.y as f64, z: eye.z as f64, look_x: look.x as f64, look_y: look.y as f64, look_z: look.z as f64, fov: fov as f64, ..Camera::default() };
    let f = world::frame(&World { stage: true, ..World::default() }, &items, Some(&camera), None, &[], time, w as f32 / h as f32);
    world::render(&f, w, h)
}

fn main() -> Result<(), String> {
    let a = Args::parse();
    let kind = component_markov_junior::kind(&a.kind)?;
    let shapes: Vec<_> = (0..a.seeds as u64).map(kind.grow).collect::<Result<_, _>>()?;
    // frame on the biggest shape; wide ones (cliffs) get wider tiles, four to a row
    let top = shapes.iter().map(|p| p.mesh.top()).fold(0.0, f32::max);
    let reach = shapes.iter().map(|p| p.canopy).fold(0.0, f32::max);
    let wide = reach * 1.2 > top * 1.3 || a.view != View::Turntable;
    let (cols, tile) = if wide { (4, (HERO.0 * 2, HERO.1)) } else { (a.seeds, HERO) };
    let rows = a.seeds.div_ceil(cols);
    let far_height = if a.view == View::Turntable { FAR_HEIGHT } else { 0 };
    let (sheet_w, sheet_h) = (tile.0 * cols, tile.1 * rows + far_height);
    std::fs::create_dir_all(a.out.join("frames")).map_err(|e| e.to_string())?;
    let tan = 19f32.to_radians().tan();
    let near = (top * 0.72 / tan).max(reach * 0.95 / (tan * tile.0 as f32 / tile.1 as f32));
    let size = top.max(reach);
    let spacing = size * 3.5;
    let grown = |seed: u32, x: f32, yaw: f32| Item::part(Grown { kind: a.kind.clone(), seed, x: x as f64, yaw: yaw.to_degrees() as f64, ..Grown::default() });
    for frame in 0..a.frames {
        let t = frame as f32 / a.frames as f32;
        let yaw = std::f32::consts::TAU * t;
        let time = t * 4.0;
        let mut sheet = vec![0u8; (sheet_w * sheet_h * 4) as usize];
        for i in 0..a.seeds {
            let above = reach * 0.85 / tan;
            let camera = match a.view {
                View::Turntable => (vec3(0.0, top * 0.55 + near * 0.12, near), vec3(0.0, top * 0.5, 0.0), 38.0),
                View::Top => (vec3(0.0, above, above * 0.01), Vec3::ZERO, 38.0),
                View::ThreeQuarter => (vec3(0.0, above * 0.72, above * 0.86), vec3(0.0, top * 0.2, 0.0), 38.0),
            };
            let px = still(vec![grown(i, 0.0, yaw)], camera, time, tile)?;
            paste(&mut sheet, sheet_w, &px, tile, (i % cols) * tile.0, (i / cols) * tile.1);
        }
        if a.view == View::Turntable {
            let row = (0..a.seeds).map(|i| grown(i, (i as f32 - (a.seeds as f32 - 1.0) / 2.0) * spacing, yaw)).collect();
            let far = still(row, (vec3(0.0, size * 1.2, size * 7.4), vec3(0.0, size * 0.45, 0.0), 42.0), time, (sheet_w, FAR_HEIGHT))?;
            paste(&mut sheet, sheet_w, &far, (sheet_w, FAR_HEIGHT), 0, tile.1 * rows);
        }
        let png = atelico_gpu::png(&sheet, sheet_w, sheet_h)?;
        std::fs::write(a.out.join(format!("frames/{frame:04}.png")), &png).map_err(|e| e.to_string())?;
        if frame == 0 {
            std::fs::write(a.out.join("sheet.png"), &png).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
