//! Grown Plants and Rocks and a material Swatch as `WorldPart`s. Each says, in one place, its colours, its mesh, what it adds to the World and what it stands on or blocks.
use std::any::Any;

use component_markov_junior::{encoding, kind, Shape};
use glam::{vec2, vec3, Vec3};
use serde_json::Value;

use crate::hooks::{Collider, Deck};
use crate::material::{self, Material};
use crate::mesh::{Mesh, Vertex};
use crate::nodes::{Plant, Rgb, Rock, Swatch};
use crate::part::{Ground, Piece, WorldPart};
use crate::{props, world_part};

atelico_render_3d::material_presets!("materials.toml");

fn json<T: serde::Serialize>(t: &T) -> Value {
    serde_json::to_value(t).unwrap_or_default()
}

fn named(c: &[(Rgb, &str)]) -> Vec<(Rgb, String)> {
    c.iter().map(|(rgb, m)| (*rgb, m.to_string())).collect()
}

/// A grown shape from markov-junior: a kept asset (`assets/<family>/<name>`, as the host's Keep writes it), else
/// `kind` from `seed` through the node cache (kept on disk, grown once per project), or grown directly without one.
fn grown(family: &str, name: &str, seed: u32, asset: &str) -> Option<Shape> {
    let got = if !asset.is_empty() {
        let root = atelico_node_cache::shared().map_or_else(|| std::env::current_dir().unwrap_or_default(), |c| c.root().to_path_buf());
        atelico_node_cache::keep::load::<Shape, Value>(&root, family, asset).map(|(s, _)| s)
    } else if let Some(cache) = atelico_node_cache::shared() {
        encoding::cached(&cache, name, seed as u64).map(|s| (*s).clone())
    } else {
        kind(name).and_then(|k| (k.grow)(seed as u64))
    };
    got.map_err(|e| eprintln!("scene-3d: {family} {name} {seed} {asset}: {e}")).ok()
}

/// A crown's ball, fitted to its leaves (grown-mesh slots 1 and 4 to 6): centred on their middle, wide enough to
/// hold nine in ten of them.
fn crown(mesh: &Mesh) -> Option<(Vec3, f32)> {
    let leaves: Vec<Vec3> = mesh.0.iter().filter(|v| [1.0, 4.0, 5.0, 6.0].contains(&v.material)).map(|v| Vec3::from(v.pos)).collect();
    if leaves.len() < 8 {
        return None;
    }
    let centre = leaves.iter().sum::<Vec3>() / leaves.len() as f32;
    let mut d: Vec<f32> = leaves.iter().map(|p| p.distance(centre)).collect();
    d.sort_by(f32::total_cmp);
    Some((centre, d[(d.len() * 9 / 10).min(d.len() - 1)]))
}

/// A grown shape stood on the ground at (x, z), turned and scaled, with a contact shade `base` wide at its foot.
fn grown_piece(shape: Option<Shape>, (x, z, yaw, scale): (f64, f64, f64, f64), ground: Ground, base: f32) -> Piece {
    let Some(shape) = shape else { return Piece::default() };
    let foot = vec3(x as f32, ground(x as f32, z as f32) - 0.05, z as f32);
    let placed = shape.placed(foot, (yaw as f32).to_radians(), scale as f32);
    let mesh = Mesh(placed.mesh.0.iter().map(|v| Vertex { pos: v.pos, normal: v.normal, material: v.material.clamp(0.0, 7.0), sway: v.sway }).collect());
    let canopy = (placed.canopy > 0.8).then(|| crown(&placed.mesh)).flatten();
    Piece { mesh, contacts: vec![(vec2(x as f32, z as f32), base * scale as f32)], framed: true, canopy: canopy.map(|(c, r)| (c, r, Rgb([0.0; 3]))), ..Piece::default() }
}

fn foot(x: f64, z: f64, under: Ground) -> Vec3 {
    vec3(x as f32, under(x as f32, z as f32), z as f32)
}

impl WorldPart for Plant {
    fn type_name(&self) -> &'static str {
        "Plant"
    }
    fn props(&self) -> Value {
        json(self)
    }
    /// By grown-mesh material number: 0 light bark, 1 light leaf, 2-3 wood, 4-6 leaf, 7 trunk.
    fn colours(&self, _: &[Material]) -> Vec<(Rgb, String)> {
        named(&[(self.trunk_light, "bark"), (self.leaf_light, "leaf"), (self.wood, "bark"), (self.wood, "bark"), (self.leaf, "leaf"), (self.leaf, "leaf"), (self.leaf, "leaf"), (self.trunk, "bark")])
    }
    fn shape(&self, ground: &str) -> Option<String> {
        Some(format!("P{} {} {} {} {} {} {} {ground}", self.kind, self.seed, self.asset, self.x, self.z, self.yaw, self.scale))
    }
    fn build(&self, ground: Ground) -> Piece {
        let mut p = grown_piece(grown("plants", &self.kind, self.seed, &self.asset), (self.x, self.z, self.yaw, self.scale), ground, 0.5);
        p.canopy = p.canopy.map(|(c, r, _)| (c, r, self.leaf));
        p
    }
    /// Trees and palms: their trunk blocks (their crowns are balls the World adds from its canopies).
    fn ground(&self, under: Ground) -> (Vec<Deck>, Vec<Collider>) {
        if self.kind != "palm" && self.kind != "tree" {
            return (Vec::new(), Vec::new());
        }
        (Vec::new(), vec![Collider::Post { foot: foot(self.x, self.z, under), radius: 0.22 * self.scale as f32, height: 3.0 * self.scale as f32 }])
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
world_part!(Plant);

impl WorldPart for Rock {
    fn type_name(&self) -> &'static str {
        "Rock"
    }
    fn props(&self) -> Value {
        json(self)
    }
    fn colours(&self, _: &[Material]) -> Vec<(Rgb, String)> {
        let (light, moss, stone) = ((self.stone_light, "stone"), (self.moss, "moss"), (self.stone, "stone"));
        named(&[light, moss, stone, stone, moss, stone, stone, stone])
    }
    fn shape(&self, ground: &str) -> Option<String> {
        Some(format!("R{} {} {} {} {} {} {} {ground}", self.kind, self.seed, self.asset, self.x, self.z, self.yaw, self.scale))
    }
    fn build(&self, ground: Ground) -> Piece {
        let mut piece = grown_piece(grown("rocks", &self.kind, self.seed, &self.asset), (self.x, self.z, self.yaw, self.scale), ground, 1.2);
        if ground(self.x as f32, self.z as f32) < 0.0 {
            let reach = piece.mesh.bounds().map_or(1.0, |(lo, hi)| ((hi - lo) * vec3(1.0, 0.0, 1.0)).length() * 0.5);
            piece.rings = vec![(vec2(self.x as f32, self.z as f32), reach)];
        }
        piece
    }
    fn ground(&self, under: Ground) -> (Vec<Deck>, Vec<Collider>) {
        (Vec::new(), vec![Collider::Post { foot: foot(self.x, self.z, under), radius: 1.0 * self.scale as f32, height: 1.5 * self.scale as f32 }])
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
world_part!(Rock);

impl WorldPart for Swatch {
    fn type_name(&self) -> &'static str {
        "Swatch"
    }
    fn props(&self) -> Value {
        json(self)
    }
    fn colours(&self, scene: &[Material]) -> Vec<(Rgb, String)> {
        vec![(material::named(&self.material, scene).albedo, self.material.clone())]
    }
    fn shape(&self, _: &str) -> Option<String> {
        Some(format!("W{} {}", self.x, self.z))
    }
    fn build(&self, _: Ground) -> Piece {
        Piece { mesh: props::swatch(vec2(self.x as f32, self.z as f32)), framed: true, ..Piece::default() }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
world_part!(Swatch);


/// Any registered grown kind as it grows, where no node of its own colours it (turntable sheets): the grown-shape
/// palette (sand, grass, rock, wood, leaf, ..., trunk, cliff) with the kind's own overrides.
#[derive(atelico_data::Data, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Grown {
    pub kind: String,
    pub seed: u32,
    pub x: f64,
    pub z: f64,
    pub yaw: f64,
    pub scale: f64,
}

impl Default for Grown {
    fn default() -> Self {
        Self { kind: "palm".into(), seed: 1, x: 0.0, z: 0.0, yaw: 0.0, scale: 1.0 }
    }
}

/// The grown-shape palette's colours and materials, by slot (markov-junior's SAND .. CLIFF).
const GROWN: [(u32, &str); 9] = [(0xf4ecc6, "sand"), (0x8cc43c, "grass"), (0xa8a39a, "stone"), (0x8f5e36, "bark"), (0x3faa3a, "leaf"), (0xe8423a, "plain"), (0xc9a25a, "plain"), (0x6e4526, "bark"), (0x676d82, "stone")];

impl WorldPart for Grown {
    fn type_name(&self) -> &'static str {
        "Grown"
    }
    fn props(&self) -> Value {
        json(self)
    }
    fn colours(&self, _: &[Material]) -> Vec<(Rgb, String)> {
        let overrides = kind(&self.kind).map(|k| k.colors).unwrap_or(&[]);
        GROWN.iter().enumerate().map(|(slot, (rgb, m))| (crate::nodes::hex(overrides.iter().find(|o| o.0 == slot).map_or(*rgb, |o| o.1)), m.to_string())).collect()
    }
    fn shape(&self, ground: &str) -> Option<String> {
        Some(format!("G{} {} {} {} {} {} {ground}", self.kind, self.seed, self.x, self.z, self.yaw, self.scale))
    }
    fn build(&self, ground: Ground) -> Piece {
        let family = kind(&self.kind).map_or("plants", |k| k.family);
        let Some(shape) = grown(family, &self.kind, self.seed, "") else { return Piece::default() };
        let foot = vec3(self.x as f32, ground(self.x as f32, self.z as f32), self.z as f32);
        let placed = shape.placed(foot, (self.yaw as f32).to_radians(), self.scale as f32);
        Piece { mesh: placed.mesh, framed: true, contacts: vec![(vec2(self.x as f32, self.z as f32), 0.5 * self.scale as f32)], ..Piece::default() }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
world_part!(Grown);

#[cfg(test)]
mod tests {
    use super::*;

    /// A palm's crown collider (and the probes' canopy) is the fronds' real spread: it holds nine in ten of the
    /// leaves' vertices and is no bigger than they reach.
    #[test]
    fn a_palm_crown_matches_its_fronds() {
        for seed in [1, 3, 7] {
            let plant = Plant { kind: "palm".into(), seed, ..Plant::default() };
            let piece = plant.build(&|_, _| 0.0);
            let (centre, radius, _) = piece.canopy.expect("a palm has a crown");
            let leaves: Vec<Vec3> = piece.mesh.0.iter().filter(|v| [1.0, 4.0, 5.0, 6.0].contains(&v.material)).map(|v| Vec3::from(v.pos)).collect();
            let inside = leaves.iter().filter(|p| p.distance(centre) <= radius + 0.01).count() as f32 / leaves.len() as f32;
            let reach = leaves.iter().map(|p| p.distance(centre)).fold(0.0, f32::max);
            assert!(inside >= 0.85, "seed {seed}: the crown holds {inside:.2} of the fronds");
            assert!(radius <= reach * 1.05, "seed {seed}: radius {radius} beyond the fronds' reach {reach}");
        }
    }
}
