//! The material swatch's mesh (a sphere and a plank box), for the Swatch part and the probes' debug view.
use std::f32::consts::TAU;

use glam::{vec3, Vec2};

use crate::mesh::Mesh;
pub use crate::mesh::circle;

/// A material swatch at `at`: a sphere and, beside it, a box of boards with seams (slot 0).
pub fn swatch(at: Vec2) -> Mesh {
    let mut m = Mesh::default();
    let (c, r) = (vec3(at.x - 0.55, 0.5, at.y), 0.45);
    let (rows, cols) = (16, 24);
    let point = |i: usize, j: usize| {
        let (th, ph) = (std::f32::consts::PI * i as f32 / rows as f32, TAU * j as f32 / cols as f32);
        vec3(th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin())
    };
    for i in 0..rows {
        for j in 0..cols {
            let q = [point(i, j), point(i + 1, j), point(i + 1, j + 1), point(i, j + 1)];
            for k in [0, 1, 2, 0, 2, 3] {
                m.vertex(c + q[k] * r, q[k], 0.0, 0.0);
            }
        }
    }
    m.cuboid(vec3(at.x + 0.55, 0.45, at.y), vec3(0.4, 0.45, 0.4), 0.5, 0.0);
    m
}

