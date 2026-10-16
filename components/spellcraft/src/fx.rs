//! The spells seen in the world: `SpellFx`, a World part that draws every live cast and what it does, from the town's
//! real state (town.rs). Meshes, so each look dresses them its own way: the pixel world draws them at 1/pixel on the
//! art-pixel grid, the ink world shades them flat with bold ink outlines. Fast things smear (a ball stretched along
//! where it flies, its trail behind it); moments punch (a burst snaps out in a few frames and is gone). No drama: every
//! effect is short and says what the spell did.
//!
//! - wind orb: a pale ball spinning with streaks trailing behind it;
//! - fire wall: a row of flame tongues along the wall, flickering, rising in its first fifth of a second;
//! - wall of flame (fire then wind): the wall's flames twice as tall, leaning with the wind, a crest running its
//!   length at roof height;
//! - fireball: an orange core in a flame shell, smeared along its flight, a trail of shrinking blobs; on impact a
//!   burst of spikes and a flash, 0.35 s;
//! - Hearthglow (a dud): a little warm puff that rises, sags and fades in a second;
//! - a hit: a white star that pops over the creature (0.15 s); a shove: a small dust kick.
//! The flames light the street (lamps).

use atelico_render_3d::mesh::Mesh;
use atelico_render_3d::part::Ground;
use glam::{vec3, Vec3};

// colour slots (0 flame, 3 warm, 4 flash, 5 dust are the meshes' former slots; the colours list keeps them)
const CORE: f32 = 1.0;
const WIND: f32 = 2.0;
const GUST: f32 = 6.0;

/// A thing to draw, read from the town: a formed body (by its look) or a moment (an event).
#[derive(Clone, Debug)]
pub enum Fx {
    Body { look: String, at: Vec3, to: Vec3, vel: Vec3, age: f32, left: f32 },
    /// `dir`: where it pushes (a knockback: away from the seed), flat.
    Moment { kind: String, at: Vec3, age: f32, dir: Vec3 },
}

/// The town's effects now (its formed bodies and its moments in the last second), in world space.
pub fn now(town: &str) -> Vec<Fx> {
    crate::with_town(town, |t| {
        let w = &t.world;
        let dt = w.dt as f32;
        let mut out = Vec::new();
        for b in w.bodies.values().filter(|b| b.alive && !b.look.is_empty()) {
            let age = (w.tick - b.born) as f32 * dt;
            let left = b.expires.saturating_sub(w.tick) as f32 * dt;
            out.push(Fx::Body { look: b.look.clone(), at: vec3(b.x as f32, 0.0, b.z as f32), to: vec3(b.x2 as f32, 0.0, b.z2 as f32), vel: vec3(b.vx as f32, 0.0, b.vz as f32), age, left });
        }
        // a shove landing with a burst is part of the burst (the fireball's carried wind): not drawn on its own
        let bursts: Vec<(f64, f64, u64)> = w.events.iter().filter(|e| e.kind == "burst").map(|e| (e.x, e.z, e.tick)).collect();
        let in_burst = |e: &crate::town::Event| bursts.iter().any(|(x, z, tick)| e.tick.abs_diff(*tick) <= 2 && (e.x - x).hypot(e.z - z) < 2.5);
        for e in w.events.iter().filter(|e| !(e.kind == "knock" && in_burst(e))) {
            let age = (w.tick - e.tick) as f32 * dt;
            if age <= 2.2 && matches!(e.kind.as_str(), "burst" | "hit" | "warm" | "knock") {
                let seed = w.bodies.get(&w.seed).map(|b| vec3(b.x as f32, 0.0, b.z as f32)).unwrap_or(Vec3::ZERO);
                let at = vec3(e.x as f32, 0.0, e.z as f32);
                out.push(Fx::Moment { kind: e.kind.clone(), at, age, dir: (at - seed).normalize_or_zero() });
            }
        }
        out
    })
    .unwrap_or_default()
}

fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x) * (1.0 - x)
}


/// The meshes for the town's effects at `t` (flicker), standing on `ground`.
pub fn mesh(fx: &[Fx], t: f32, ground: Ground, scale: f32) -> Mesh {
    let mut m = Mesh::default();
    let lift = |p: Vec3, y: f32| vec3(p.x, ground(p.x, p.z).max(0.0) + y, p.z);
    for (n, f) in fx.iter().enumerate() {
        let seed = n as u32 * 7919;
        match f {
            Fx::Body { look, at, to, vel, age, left } => match look.as_str() {
                "wind-orb" => {
                    // the swirl's silhouette: three spiral arcs round an empty middle (the particles add the cloud and
                    // the streaks)
                    let scale = scale * 1.6;
                    let c = lift(*at, 1.2 * scale / 1.6);
                    let dir = vel.normalize_or_zero();
                    let dir = if dir == Vec3::ZERO { Vec3::X } else { dir };
                    let side = Vec3::Y.cross(dir).normalize_or_zero();
                    for arm in 0..3 {
                        let a0 = t * 12.0 + arm as f32 * std::f32::consts::TAU / 3.0;
                        let mut prev = None;
                        for k in 0..=8 {
                            let u = k as f32 / 8.0;
                            let a = a0 + u * 2.2;
                            let r = (0.25 + 0.4 * u) * scale;
                            let p = c + (side * a.cos() + Vec3::Y * a.sin()) * r - dir * u * 0.3 * scale;
                            if let Some(q) = prev {
                                let w = (0.09 * (1.0 - u) + 0.025) * scale;
                                let n = (p - c).normalize_or_zero() * w;
                                m.quad(q - n, p - n * 0.8, p + n * 0.8, q + n, if arm == 0 { WIND } else { GUST });
                            }
                            prev = Some(p);
                        }
                    }
                }
                "fire-wall" | "flame-wall" => {
                    // the wall's silhouette: its glowing bed along the ground (the particles are its flames)
                    let rise = ease_out(age / 0.3) * ease_out(*left / 0.4);
                    let len = (*to - *at).length().max(0.5);
                    let dir = (*to - *at) / len;
                    let side = Vec3::Y.cross(dir).normalize_or_zero();
                    let (a0, a1) = (lift(*at, 0.03), lift(*to, 0.03));
                    let wide = side * 0.4 * scale * rise;
                    m.quad(a0 - wide, a1 - wide, a1 + wide, a0 + wide, CORE);
                }
                _ => {}
            },
            Fx::Moment { .. } => {}
        }
    }
    m
}



/// The spells' particles (graphs/spell-*.graph.yaml, read again when they change): each formed body's and moment's
/// effect at its age, where it is.
pub fn sprites(fx: &[Fx], t: f32, ground: atelico_render_3d::part::Ground, scale: f32, look: &str) -> Vec<component_particles::Sprite> {
    use component_particles::{graph, part::graphs_dir, Effect};
    // the look's own graph where there is one (spell-fire-wall-ink), else the shared one
    let load = |name: &str, node: Option<&str>| -> Effect {
        let own = graphs_dir().join(format!("spell-{name}-{look}.graph.yaml"));
        let path = if !look.is_empty() && own.is_file() { own } else { graphs_dir().join(format!("spell-{name}.graph.yaml")) };
        graph::effect_in(&path, node).map_err(|e| eprintln!("spellfx: {e}")).unwrap_or_default()
    };
    let lift = |p: Vec3, y: f32| vec3(p.x, ground(p.x, p.z).max(0.0) + y, p.z);
    let look_ink = look == "ink";
    let mut out = Vec::new();
    let emit = |e: &Effect, age: f32, at: Vec3, s: f32, dir: Option<Vec3>, seed: u32, out: &mut Vec<component_particles::Sprite>| {
        for em in &e.emitters {
            let mut em = em.clone();
            em.seed = em.seed.wrapping_add(seed);
            if let Some(d) = dir {
                em.direction = d;
            }
            out.extend(em.sprites(age, at, s));
        }
    };
    for (n, f) in fx.iter().enumerate() {
        let seed = n as u32 * 977;
        match f {
            Fx::Body { look, at, to, vel, age, .. } => match look.as_str() {
                "fireball" => {
                    let back = -vel.normalize_or_zero();
                    emit(&load("fireball", None), *age, lift(*at, 1.1 * scale), scale, Some(if back == Vec3::ZERO { Vec3::Y } else { back }), seed, &mut out);
                }
                "wind-orb" => {
                    let back = -vel.normalize_or_zero();
                    emit(&load("wind", Some("out")), *age, lift(*at, 1.2 * scale), scale * 1.4, Some(if back == Vec3::ZERO { Vec3::Y } else { back }), seed, &mut out);
                }
                "fire-wall" | "flame-wall" => {
                    let gust = look == "flame-wall";
                    let len = (*to - *at).length().max(0.5);
                    let dir = (*to - *at) / len;
                    let side = Vec3::Y.cross(dir).normalize_or_zero();
                    let mut wall = load("fire-wall", None);
                    let age = *age;
                    if gust {
                        // (the wind blows its smoke away: no smoke emitter)
                        wall.emitters.retain(|e| e.colour.at(0.0).x > 0.6);
                    }
                    let count = (len / 0.5).ceil() as usize;
                    // it rises in half a second from low flames
                    let rise = 0.35 + 0.65 * ease_out((age - 0.1) / 0.5);
                    // the wall of flame: first a crest runs once along it, upright, peaking over the roofs (0.6 to 1.3 s);
                    // then the wind bends the whole wall over (from 1.3 s): the flames are blown, not tilted, so their
                    // tongues curl downwind instead of turning into wedges
                    let bend = if gust { ease_out((age - 1.3) / 0.35) } else { 0.0 };
                    if gust && bend > 0.0 {
                        for em in &mut wall.emitters {
                            em.gravity += side * 9.0 * bend;
                            // and drawn along their flight, so each tongue sweeps downwind
                            em.stretch = em.stretch.max(0.7 * bend);
                            em.speed = component_particles::Range(em.speed.0 * (1.0 + 0.4 * bend), em.speed.1 * (1.0 + 0.4 * bend));
                        }
                        // the gust that does it: speed lines sweeping along the wall's foot from the caster's side
                        let gusts = load("wind", Some("gust"));
                        for g in 0..3 {
                            let p = lift(*at + dir * len * (0.2 + 0.3 * g as f32), 0.6 * scale);
                            emit(&gusts, age - 1.25 - 0.05 * g as f32, p - side * 0.8 * scale, scale, Some(side), seed + 501 + g, &mut out);
                        }
                    }
                    let crest = ((age - 0.6) / 0.7).clamp(0.0, 1.0);
                    if gust && look_ink && age > 0.55 && age < 1.5 {
                        // in ink the crest is a few big tongues, one tip leading, running along the wall
                        let p = lift(*at + dir * len * crest, 0.0);
                        let swell = ease_out((age - 0.55) / 0.2) * ease_out((1.5 - age) / 0.2);
                        emit(&load("fire-wall", Some("crest")), age, p, scale * (0.4 + 0.6 * swell), Some(Vec3::Y), seed + 71, &mut out);
                    }
                    for k in 0..=count {
                        let u = k as f32 / count.max(1) as f32;
                        let p = lift(*at + dir * len * u, 0.0);
                        let s = scale * rise * if gust { 1.15 } else { 1.0 };
                        let mut stack = 0;
                        if gust && !look_ink && age > 0.6 && age < 1.45 {
                            // the crest: more flames stacked up the wall where it runs (each tier the same size, so the
                            // flames stay crisp), a dip behind it
                            let near = (-((u - crest) * 5.0).powi(2)).exp();
                            stack = (near * 3.0).round() as usize;
                        }
                        for tier in 0..=stack {
                            emit(&wall, age, p + Vec3::Y * 0.9 * scale * tier as f32, s, Some(Vec3::Y), seed + k as u32 * 31 + tier as u32 * 7, &mut out);
                        }
                    }
                }
                _ => {}
            },
            Fx::Moment { kind, at, age, dir } => match kind.as_str() {
                "burst" => emit(&load("impact", None), *age, lift(*at, 0.6 * scale), scale, None, seed, &mut out),
                "hit" => emit(&load("hit", None), *age, lift(*at, 1.4 * scale), scale, None, seed, &mut out),
                "knock" => emit(&load("wind", Some("knock")), *age, lift(*at, 0.35), scale * 1.3, Some((*dir + Vec3::Y * 0.05).normalize_or_zero()), seed, &mut out),
                "warm" => emit(&load("dud", None), *age, lift(*at, 1.1 * scale), scale, None, seed, &mut out),
                _ => {}
            },
        }
    }
    out.truncate(component_particles::MOST as usize / 2);
    out
}
