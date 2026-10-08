//! The moon town as a spell world: the `WorldAdapter` every spell in the town runs against, and the town that runs
//! them. Everything here is a spell (Paul): the watchtower's defence (sensor, brain, effect), each creature (its
//! health and how it goes for the seed: climbing over fences or digging under them), the town seed (its health; at
//! zero, "fall": the town is gone), and the player's casts, fused by cast order. The town is a pure function of time:
//! step it from 0 and the same seed, moons and casts give the same night.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::definition::{number, parse, text, Kind, SpellDefinition, Step};
use crate::fusion::{self, Names};
use crate::runtime::{self, Brain, Effect, Instance, Observation, Status, WorldAdapter};

/// spells/town.toml.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TownConfig {
    pub ticks_per_second: u32,
    pub refill: f64,
    pub seed: Place,
    pub hero: Point,
    pub buildings: Vec<Building>,
    pub walls: Vec<Wall>,
    pub spawns: Vec<Spawn>,
    pub waves: Waves,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: f64,
    pub z: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Place {
    pub x: f64,
    pub z: f64,
    pub spell: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Building {
    pub name: String,
    pub spell: String,
    pub x: f64,
    pub z: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Wall {
    pub x1: f64,
    pub z1: f64,
    pub x2: f64,
    pub z2: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub x: f64,
    pub z: f64,
    pub moon: Moon,
    pub spell: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Moon {
    Purple,
    Orange,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Waves {
    pub every: f64,
    pub alive: usize,
}

/// The spells a town runs: definitions by name, their brains, the fusion names.
pub struct Library {
    pub spells: BTreeMap<String, SpellDefinition>,
    pub brains: BTreeMap<String, Arc<Brain>>,
    pub names: Names,
    pub town: TownConfig,
}

/// The files shipped with the package (the project's spells/ dir is read first).
const SHIPPED: &[(&str, &str)] = &[
    ("wind.spell.yaml", include_str!("../../../spells/wind.spell.yaml")),
    ("fire.spell.yaml", include_str!("../../../spells/fire.spell.yaml")),
    ("watchtower.spell.yaml", include_str!("../../../spells/watchtower.spell.yaml")),
    ("watchtower.lua", include_str!("../../../spells/watchtower.lua")),
    ("climber.spell.yaml", include_str!("../../../spells/climber.spell.yaml")),
    ("digger.spell.yaml", include_str!("../../../spells/digger.spell.yaml")),
    ("town-seed.spell.yaml", include_str!("../../../spells/town-seed.spell.yaml")),
    ("fusion.toml", include_str!("../../../spells/fusion.toml")),
    ("town.toml", include_str!("../../../spells/town.toml")),
];

impl Library {
    /// The spells in `dir` (spells/: `<name>.spell.yaml`, a brain `<name>.lua`, fusion.toml, town.toml), each file
    /// the shipped one where the dir has none.
    pub fn load(dir: Option<&Path>) -> Result<Library, String> {
        let read = |file: &str| -> String {
            dir.and_then(|d| std::fs::read_to_string(d.join(file)).ok()).unwrap_or_else(|| SHIPPED.iter().find(|(f, _)| *f == file).map(|(_, t)| t.to_string()).unwrap_or_default())
        };
        let mut files: BTreeSet<String> = SHIPPED.iter().map(|(f, _)| f.to_string()).collect();
        if let Some(d) = dir {
            for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
                files.insert(e.file_name().to_string_lossy().into_owned());
            }
        }
        let mut spells = BTreeMap::new();
        let mut brains = BTreeMap::new();
        for f in &files {
            if let Some(name) = f.strip_suffix(".spell.yaml") {
                let d = parse(&read(f)).map_err(|e| format!("spells/{f}: {e}"))?;
                spells.insert(name.to_string(), d);
            } else if let Some(name) = f.strip_suffix(".lua") {
                brains.insert(name.to_string(), Arc::new(Brain::compile(&read(f)).map_err(|e| format!("spells/{f}: {e}"))?));
            }
        }
        let names = Names::parse(&read("fusion.toml")).map_err(|e| format!("spells/fusion.toml: {e}"))?;
        let town: TownConfig = toml::from_str(&read("town.toml")).map_err(|e| format!("spells/town.toml: {e}"))?;
        Ok(Library { spells, brains, names, town })
    }

    pub fn spell(&self, name: &str) -> Result<&SpellDefinition, String> {
        self.spells.get(name).ok_or_else(|| format!("no spell {name}"))
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BodyKind {
    Hero,
    Building,
    Seed,
    Creature,
    Bolt,
    Wall,
    Orb,
}

/// A body in the town.
#[derive(Clone, Debug, Serialize)]
pub struct Body {
    pub id: String,
    pub kind: BodyKind,
    pub x: f64,
    pub z: f64,
    #[serde(skip)]
    pub vx: f64,
    #[serde(skip)]
    pub vz: f64,
    pub health: f64,
    pub max: f64,
    pub alive: bool,
    pub moon: Option<Moon>,
    /// walk, climb or dig (creatures)
    pub way: String,
    pub underground: bool,
    pub climbing: bool,
    pub slowed: bool,
    pub burning: bool,
    #[serde(skip)]
    slow_until: u64,
    #[serde(skip)]
    slow_factor: f64,
    #[serde(skip)]
    burn_until: u64,
    #[serde(skip)]
    burn_rate: f64,
    #[serde(skip)]
    on_zero: String,
    #[serde(skip)]
    height: f64,
    #[serde(skip)]
    pub(crate) expires: u64,
    /// a wall's far end (its body is a segment from x, z)
    #[serde(skip)]
    pub(crate) x2: f64,
    #[serde(skip)]
    pub(crate) z2: f64,
    #[serde(skip)]
    touched: BTreeMap<String, u64>,
    /// A formed body's spell: its look (wind-orb, fire-wall, flame-wall, fireball, dud, or shape-element) and when it
    /// was formed (tick), for the effects (fx.rs).
    pub look: String,
    pub born: u64,
    /// Knockback: a shove's velocity, dying away (m/s).
    #[serde(skip)]
    kvx: f64,
    #[serde(skip)]
    kvz: f64,
}

impl Body {
    fn new(id: &str, kind: BodyKind, x: f64, z: f64) -> Body {
        Body {
            id: id.into(), kind, x, z, vx: 0., vz: 0., health: 1., max: 1., alive: true, moon: None, way: String::new(), underground: false, climbing: false, slowed: false, burning: false,
            slow_until: 0, slow_factor: 1., burn_until: 0, burn_rate: 0., on_zero: "vanish".into(), height: 0., expires: u64::MAX, x2: x, z2: z, touched: BTreeMap::new(),
            look: String::new(), born: 0, kvx: 0., kvz: 0.,
        }
    }
}

/// Something that happened, for the HUD: a hit (its number or word), a fusion, a creature gone, fire caught, the town
/// fallen.
#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub tick: u64,
    pub kind: String,
    pub text: String,
    pub x: f64,
    pub z: f64,
}

/// The world the town's spells act on.
pub struct World {
    pub bodies: BTreeMap<String, Body>,
    pub seed: String,
    pub tick: u64,
    pub dt: f64,
    pub walls: Vec<Wall>,
    /// Towards each moon (purple, orange); y up. A moon under the horizon throws no shadow.
    pub moons: [Option<[f64; 3]>; 2],
    pub events: Vec<Event>,
    next: u64,
    /// the bodies formed for each instance, so an exhausted spell's body can go
    formed: BTreeMap<String, Vec<String>>,
    /// each cast instance's look (what its formed bodies are drawn as)
    pub looks: BTreeMap<String, String>,
}

fn dist_to_segment(px: f64, pz: f64, w: &Wall) -> (f64, f64) {
    let (dx, dz) = (w.x2 - w.x1, w.z2 - w.z1);
    let l2 = (dx * dx + dz * dz).max(1e-9);
    let t = (((px - w.x1) * dx + (pz - w.z1) * dz) / l2).clamp(0., 1.);
    let (cx, cz) = (w.x1 + t * dx, w.z1 + t * dz);
    (((px - cx).powi(2) + (pz - cz).powi(2)).sqrt(), t)
}

impl World {
    fn id(&mut self, kind: &str) -> String {
        self.next += 1;
        format!("{kind}-{}", self.next)
    }

    fn event(&mut self, kind: &str, text: String, x: f64, z: f64) {
        self.events.push(Event { tick: self.tick, kind: kind.into(), text, x, z });
        if self.events.len() > 64 {
            self.events.remove(0);
        }
    }

    /// The nearest living creature above ground to (x, z).
    fn nearest_creature(&self, x: f64, z: f64) -> Option<&Body> {
        self.bodies.values().filter(|b| b.kind == BodyKind::Creature && b.alive && !b.underground).min_by(|a, b| {
            let da = (a.x - x).powi(2) + (a.z - z).powi(2);
            let db = (b.x - x).powi(2) + (b.z - z).powi(2);
            da.total_cmp(&db)
        })
    }

    /// Whether (x, z) is in the shadow a moon throws from a building of `height` at (bx, bz), within `reach`.
    fn in_shadow(&self, bx: f64, bz: f64, height: f64, x: f64, z: f64, reach: f64, moon: Option<Moon>) -> bool {
        self.moons.iter().enumerate().any(|(k, m)| {
            let Some(d) = m else { return false };
            if moon.is_some_and(|want| (want == Moon::Purple) != (k == 0)) || d[1] <= 0.02 {
                return false;
            }
            let flat = (d[0] * d[0] + d[2] * d[2]).sqrt().max(1e-6);
            let length = (height * flat / d[1]).min(reach);
            // the shadow runs away from the moon
            let (ux, uz) = (-d[0] / flat, -d[2] / flat);
            let (px, pz) = (x - bx, z - bz);
            let along = px * ux + pz * uz;
            let across = (px * uz - pz * ux).abs();
            along >= 0. && along <= length && across <= 1.2
        })
    }

    fn damage(&mut self, id: &str, amount: f64, word: Option<&str>) {
        let Some(b) = self.bodies.get_mut(id) else { return };
        if !b.alive {
            return;
        }
        b.health -= amount;
        let (x, z) = (b.x, b.z);
        let text = word.map(String::from).unwrap_or_else(|| format!("{}", amount.round().max(1.) as i64));
        self.event("hit", text, x, z);
    }
}

impl WorldAdapter for World {
    fn form(&mut self, instance: &str, caster: &str, step: &Step) -> Result<String, String> {
        let (cx, cz) = self.bodies.get(caster).map(|b| (b.x, b.z)).ok_or("the caster is gone")?;
        let (tx, tz) = self.nearest_creature(cx, cz).map(|b| (b.x, b.z)).unwrap_or((cx, cz - 4.));
        let (dx, dz) = (tx - cx, tz - cz);
        let l = (dx * dx + dz * dz).sqrt().max(1e-6);
        let (ux, uz) = (dx / l, dz / l);
        let ticks = 1.0 / self.dt;
        let (kind, ttl) = match step.component.as_str() {
            "body.bolt" => (BodyKind::Bolt, 2.0),
            "body.orb" => (BodyKind::Orb, 3.0),
            "body.wall" => (BodyKind::Wall, 4.0),
            other => return Err(format!("{other} forms nothing here")),
        };
        let id = self.id(match kind {
            BodyKind::Bolt => "bolt",
            BodyKind::Orb => "orb",
            _ => "wall",
        });
        let mut b = Body::new(&id, kind, cx, cz);
        b.expires = self.tick + (ttl * ticks) as u64;
        b.look = self.looks.get(instance).cloned().unwrap_or_default();
        b.born = self.tick;
        if b.look == "dud" {
            // a dud puffs at the caster's hands, in front of it (the town's cameras look down -z)
            // (beside the caster, to its side, not over its body)
            b.x = cx + 0.9;
            b.z = cz + 0.5;
        } else if kind == BodyKind::Wall {
            // across the way the nearest creature comes, 3.5 m out (clear of the caster)
            let half = number(step, "length", 5.) / 2.;
            let (mx, mz) = (cx + ux * 3.5, cz + uz * 3.5);
            b.x = mx - uz * half;
            b.z = mz + ux * half;
            b.x2 = mx + uz * half;
            b.z2 = mz - ux * half;
        } else {
            // a projectile leaves from in front of the caster, not inside it
            b.x = cx + ux * 1.6;
            b.z = cz + uz * 1.6;
            b.vx = ux;
            b.vz = uz;
        }
        self.bodies.insert(id.clone(), b);
        self.formed.entry(instance.into()).or_default().push(id.clone());
        Ok(id)
    }

    fn sense(&mut self, step: &Step, target: &str, tick: u64) -> Result<Observation, String> {
        let t = self.bodies.get(target).cloned().ok_or("the sensor's target is gone")?;
        let mut found: Vec<(f64, String)> = Vec::new();
        for b in self.bodies.values().filter(|b| b.kind == BodyKind::Creature && b.alive && !b.underground) {
            let d = ((b.x - t.x).powi(2) + (b.z - t.z).powi(2)).sqrt();
            let hit = match step.component.as_str() {
                "sense.nearby" => d <= number(step, "radius", 4.),
                "sense.shadow" => {
                    let moon = match text(step, "moon", "any").as_str() {
                        "purple" => Some(Moon::Purple),
                        "orange" => Some(Moon::Orange),
                        _ => None,
                    };
                    self.in_shadow(t.x, t.z, t.height, b.x, b.z, number(step, "reach", 9.), moon)
                }
                other => return Err(format!("{other} senses nothing here")),
            };
            if hit {
                found.push((d, b.id.clone()));
            }
        }
        found.sort_by(|a, b| a.0.total_cmp(&b.0));
        Ok(Observation { tick, target: target.into(), distances: found.iter().map(|f| f.0).collect(), bodies: found.into_iter().map(|f| f.1).collect() })
    }

    fn apply(&mut self, e: &Effect) -> Result<(), String> {
        let p = |k: &str, d: f64| e.params.get(k).and_then(|v| v.as_f64()).unwrap_or(d);
        let ticks = 1.0 / self.dt;
        let now = self.tick;
        match e.kind.as_str() {
            "motion.launch" => {
                for t in &e.targets {
                    if let Some(b) = self.bodies.get_mut(t) {
                        let s = p("speed", 10.);
                        b.vx *= s;
                        b.vz *= s;
                    }
                }
            }
            "move.toward" => {
                let (sx, sz) = self.bodies.get(&self.seed).map(|b| (b.x, b.z)).unwrap_or((0., 0.));
                let way = e.params.get("way").and_then(|v| v.as_str()).unwrap_or("walk").to_string();
                let walls = self.walls.clone();
                for t in &e.targets {
                    let Some(b) = self.bodies.get_mut(t) else { continue };
                    b.way = way.clone();
                    let (dx, dz) = (sx - b.x, sz - b.z);
                    let l = (dx * dx + dz * dz).sqrt();
                    let near_wall = walls.iter().any(|w| dist_to_segment(b.x, b.z, w).0 < if way == "dig" { 2.0 } else { 0.7 });
                    b.climbing = way == "climb" && near_wall;
                    b.underground = way == "dig" && near_wall;
                    if l < 0.9 {
                        continue;
                    }
                    let mut speed = p("speed", 1.2);
                    if b.climbing {
                        speed *= 0.5;
                    }
                    if b.slow_until > now {
                        speed *= b.slow_factor;
                    }
                    let step = (speed * self.dt).min(l - 0.8);
                    b.x += dx / l * step;
                    b.z += dz / l * step;
                }
            }
            "effect.slow" => {
                for t in &e.targets {
                    if let Some(b) = self.bodies.get_mut(t) {
                        let fresh = b.slow_until <= now;
                        b.slow_until = now + (p("seconds", 2.) * ticks) as u64;
                        b.slow_factor = p("factor", 0.25);
                        let (x, z) = (b.x, b.z);
                        if fresh {
                            self.event("hit", "SLOW".into(), x, z);
                        }
                    }
                }
            }
            "effect.burn" => {
                for t in &e.targets {
                    if let Some(b) = self.bodies.get_mut(t) {
                        let fresh = b.burn_until <= now;
                        b.burn_until = now + (p("seconds", 3.) * ticks) as u64;
                        b.burn_rate = b.burn_rate.max(p("per_second", 1.));
                        let (x, z) = (b.x, b.z);
                        if fresh {
                            self.event("hit", "BURN".into(), x, z);
                        }
                    }
                }
            }
            "effect.damage" => {
                for t in &e.targets {
                    self.damage(t, p("amount", 1.), None);
                }
            }
            "force.push" => {
                let from = self.bodies.get(e.targets.first().map(String::as_str).unwrap_or("")).map(|b| (b.x, b.z));
                let (sx, sz) = self.bodies.get(&self.seed).map(|b| (b.x, b.z)).unwrap_or((0., 0.));
                for t in &e.targets {
                    if let Some(b) = self.bodies.get_mut(t) {
                        // knocked back away from the seed (the way it came): a shove that dies away over a quarter
                        // second and carries it `strength` metres
                        let (dx, dz) = (b.x - sx, b.z - sz);
                        let l = (dx * dx + dz * dz).sqrt().max(1e-6);
                        let v = p("strength", 2.) / KNOCK_TAU;
                        b.kvx += dx / l * v;
                        b.kvz += dz / l * v;
                        let (x, z) = (b.x, b.z);
                        self.events.push(Event { tick: now, kind: "knock".into(), text: String::new(), x, z });
                    }
                }
                let _ = from;
            }
            "fire.burst" => {
                let Some(c) = e.targets.first().and_then(|t| self.bodies.get(t)).map(|b| (b.x, b.z)) else { return Ok(()) };
                let r = p("radius", 2.);
                let ids: Vec<String> = self.bodies.values().filter(|b| b.kind == BodyKind::Creature && b.alive && !b.underground && ((b.x - c.0).powi(2) + (b.z - c.1).powi(2)).sqrt() <= r).map(|b| b.id.clone()).collect();
                for id in ids {
                    if let Some(b) = self.bodies.get_mut(&id) {
                        b.burn_until = now + (3. * ticks) as u64;
                        b.burn_rate = b.burn_rate.max(1.5);
                    }
                }
                self.event("burst", "FIRE".into(), c.0, c.1);
            }
            "effect.warm" => {
                if let Some(b) = e.targets.first().and_then(|t| self.bodies.get(t)) {
                    let (x, z) = (b.x, b.z);
                    self.event("warm", "warm".into(), x, z);
                }
            }
            "vital.health" => {
                for t in &e.targets {
                    if let Some(b) = self.bodies.get_mut(t) {
                        b.max = p("max", 5.);
                        b.health = b.max;
                        b.on_zero = e.params.get("on_zero").and_then(|v| v.as_str()).unwrap_or("vanish").into();
                    }
                }
            }
            "vital.drain" => {
                let n = e.params.get("_count").and_then(|v| v.as_f64()).unwrap_or(0.);
                if n > 0. {
                    for t in &e.targets {
                        let amount = p("per_body", 1.) * n;
                        self.damage(t, amount, None);
                    }
                }
            }
            other => return Err(format!("{other} does nothing here")),
        }
        Ok(())
    }

    fn exists(&self, body: &str) -> bool {
        self.bodies.get(body).is_some_and(|b| b.alive)
    }

    fn take_contact(&mut self, body: &str) -> Option<String> {
        let b = self.bodies.get(body)?.clone();
        let now = self.tick;
        let hit = match b.kind {
            BodyKind::Wall => {
                let w = Wall { x1: b.x, z1: b.z, x2: b.x2, z2: b.z2 };
                self.bodies.values().filter(|c| c.kind == BodyKind::Creature && c.alive && !c.underground && dist_to_segment(c.x, c.z, &w).0 < 0.6 && b.touched.get(&c.id).is_none_or(|t| now >= *t + 10)).map(|c| c.id.clone()).next()
            }
            BodyKind::Bolt | BodyKind::Orb => self.bodies.values().filter(|c| c.kind == BodyKind::Creature && c.alive && !c.underground && ((c.x - b.x).powi(2) + (c.z - b.z).powi(2)).sqrt() < 0.9).map(|c| c.id.clone()).next(),
            _ => None,
        }?;
        if let Some(w) = self.bodies.get_mut(body) {
            w.touched.insert(hit.clone(), now);
            // a projectile is spent on what it hits (gone after this tick's contact steps)
            if matches!(w.kind, BodyKind::Bolt | BodyKind::Orb) {
                w.expires = now + 1;
            }
        }
        Some(hit)
    }
}

/// How a spell's formed bodies are drawn: the named looks the effects know, else "shape-element".
pub fn look_of(d: &SpellDefinition) -> String {
    let uses = |c: &str| d.steps().any(|(_, s)| s.component == c);
    let (shape, element) = (d.shape.as_deref().unwrap_or(""), d.element.as_deref().unwrap_or(""));
    if uses("effect.warm") {
        return "dud".into();
    }
    match (shape, element, uses("effect.burn")) {
        ("orb", "wind", _) => "wind-orb".into(),
        ("wall", "fire", _) => "fire-wall".into(),
        ("wall", "wind", true) => "flame-wall".into(),
        ("orb", "fire", _) => "fireball".into(),
        _ => format!("{shape}-{element}"),
    }
}

/// A fusion the player cast, for the HUD: the two in order, the result's name, when.
#[derive(Clone, Debug, Serialize)]
pub struct Fused {
    pub first: String,
    pub second: String,
    /// Its name now: the fallback until the on-device model's name arrives (`rename`).
    pub name: String,
    /// The name Rust gives alone (spells/fusion.toml, or "Element Shape").
    pub fallback: String,
    /// Whether the model has named it.
    pub named: bool,
    pub shape: String,
    pub element: String,
    pub tick: u64,
}

/// Seconds an armed spell waits for a second cast to fuse with.
pub const ARM: f64 = 1.2;
/// How fast a knockback dies away (s).
pub const KNOCK_TAU: f64 = 0.25;

/// The town: its world, the spells running in it, the player's slots.
pub struct Town {
    pub lib: Library,
    pub world: World,
    pub instances: Vec<Instance>,
    /// The player's spells in their slots (names in the library), fire once caught.
    pub slots: Vec<String>,
    pub fused: Option<(SpellDefinition, Fused)>,
    pub town_gone: bool,
    pub fire_caught: bool,
    /// The slot the player has selected (0-based).
    pub selected: usize,
    /// A spell cast and waiting `ARM` seconds for a second: cast again in time and the two fuse in cast order;
    /// else it goes alone.
    pub armed: Option<(String, u64)>,
    spawn_clock: f64,
    spawn_turn: usize,
    count: u64,
}

impl Town {
    pub fn new(lib: Library) -> Result<Town, String> {
        let dt = 1.0 / lib.town.ticks_per_second.max(1) as f64;
        let mut world = World { bodies: BTreeMap::new(), seed: "seed".into(), tick: 0, dt, walls: lib.town.walls.clone(), moons: [None, None], events: Vec::new(), next: 0, formed: BTreeMap::new(), looks: BTreeMap::new() };
        world.bodies.insert("hero".into(), Body::new("hero", BodyKind::Hero, lib.town.hero.x, lib.town.hero.z));
        world.bodies.insert("seed".into(), Body::new("seed", BodyKind::Seed, lib.town.seed.x, lib.town.seed.z));
        let mut town = Town { lib, world, instances: Vec::new(), slots: vec!["wind".into()], fused: None, town_gone: false, fire_caught: false, selected: 0, armed: None, spawn_clock: 0., spawn_turn: 0, count: 0 };
        let seed_spell = town.lib.town.seed.spell.clone();
        town.start(&seed_spell, "seed")?;
        for b in town.lib.town.buildings.clone() {
            let mut body = Body::new(&b.name, BodyKind::Building, b.x, b.z);
            body.height = b.height;
            town.world.bodies.insert(b.name.clone(), body);
            town.start(&b.spell, &b.name)?;
        }
        Ok(town)
    }

    fn start(&mut self, spell: &str, holder: &str) -> Result<(), String> {
        let d = self.lib.spell(spell)?.clone();
        self.start_definition(d, holder)
    }

    fn start_definition(&mut self, d: SpellDefinition, holder: &str) -> Result<(), String> {
        self.count += 1;
        let brain = self.lib.brains.get(&d.name).cloned();
        let mana = d.mana.allocation;
        let tick = self.world.tick;
        let id = format!("{}#{}", d.name, self.count);
        self.world.looks.insert(id.clone(), look_of(&d));
        let i = runtime::cast(&id, holder, &d, brain, mana, &mut self.world, tick)?;
        self.instances.push(i);
        Ok(())
    }

    /// Casts the spell in slot `name` from the hero.
    pub fn cast(&mut self, name: &str) -> Result<(), String> {
        if !self.slots.iter().any(|s| s == name) {
            return Err(format!("{name} is not in a slot"));
        }
        self.start(name, "hero")
    }

    /// Casts `first` then `second`: the fusion (shape from the first, element from the second), named by the table or
    /// by `choice` (the on-device model's pick, checked and built by Rust).
    pub fn cast_pair(&mut self, first: &str, second: &str, choice: Option<&fusion::Choice>) -> Result<String, String> {
        for n in [first, second] {
            if !self.slots.iter().any(|s| s == n) {
                return Err(format!("{n} is not in a slot"));
            }
        }
        let (a, b) = (self.lib.spell(first)?.clone(), self.lib.spell(second)?.clone());
        let d = match choice {
            Some(c) => fusion::build(&a, &b, c)?,
            None => fusion::fuse(&a, &b, &self.lib.names)?,
        };
        let name = d.title().to_string();
        self.fused = Some((d.clone(), Fused { first: first.into(), second: second.into(), name: name.clone(), fallback: name.clone(), named: choice.is_some(), shape: d.shape.clone().unwrap_or_default(), element: d.element.clone().unwrap_or_default(), tick: self.world.tick }));
        self.world.event("fusion", name.clone(), self.world.bodies["hero"].x, self.world.bodies["hero"].z);
        self.start_definition(d, "hero")?;
        Ok(name)
    }

    fn spawn(&mut self) -> Result<(), String> {
        let alive = self.world.bodies.values().filter(|b| b.kind == BodyKind::Creature && b.alive).count();
        if alive >= self.lib.town.waves.alive || self.lib.town.spawns.is_empty() {
            return Ok(());
        }
        for _ in 0..self.lib.town.spawns.len() {
            let s = self.lib.town.spawns[self.spawn_turn % self.lib.town.spawns.len()].clone();
            self.spawn_turn += 1;
            let up = self.world.moons[if s.moon == Moon::Purple { 0 } else { 1 }].is_some_and(|d| d[1] > 0.);
            if !up {
                continue;
            }
            let id = self.world.id("creature");
            let mut b = Body::new(&id, BodyKind::Creature, s.x, s.z);
            b.moon = Some(s.moon);
            // (when it rose, for whoever draws it climbing out of the ground)
            b.born = self.world.tick;
            self.world.bodies.insert(id.clone(), b);
            self.start(&s.spell, &id)?;
            return Ok(());
        }
        Ok(())
    }

    /// One tick: the town feeds its buildings and seed, creatures rise while their moon is up, every spell ticks,
    /// bodies move and burn, the dead go, and a seed at zero is the town gone.
    pub fn step(&mut self) {
        if self.town_gone {
            return;
        }
        // an armed spell that waited long enough for a second goes alone
        if let Some((name, at)) = self.armed.clone() {
            if (self.world.tick - at) as f64 * self.world.dt >= ARM {
                self.armed = None;
                let _ = self.start(&name, "hero");
            }
        }
        let w = &mut self.world;
        w.tick += 1;
        let dt = w.dt;
        let refill = self.lib.town.refill * dt;
        for i in &mut self.instances {
            if matches!(i.definition.mana.source, crate::definition::Source::Town | crate::definition::Source::Seed) && i.status == Status::Running {
                i.energy = (i.energy + refill).min(i.initial);
            }
        }
        self.spawn_clock += dt;
        if self.spawn_clock >= self.lib.town.waves.every {
            self.spawn_clock = 0.;
            let _ = self.spawn();
        }
        let tick = self.world.tick;
        for i in &mut self.instances {
            runtime::tick(i, &mut self.world, tick);
        }
        // bodies: projectiles fly (homing on the nearest creature), burns burn, the spent and the dead go
        let ids: Vec<String> = self.world.bodies.keys().cloned().collect();
        for id in &ids {
            let Some(b) = self.world.bodies.get(id) else { continue };
            if b.alive && matches!(b.kind, BodyKind::Bolt | BodyKind::Orb) {
                let speed = (b.vx * b.vx + b.vz * b.vz).sqrt();
                if let Some((tx, tz)) = self.world.nearest_creature(b.x, b.z).map(|c| (c.x, c.z)) {
                    let (dx, dz) = (tx - b.x, tz - b.z);
                    let l = (dx * dx + dz * dz).sqrt().max(1e-6);
                    let b = self.world.bodies.get_mut(id).unwrap();
                    b.vx = dx / l * speed;
                    b.vz = dz / l * speed;
                }
            }
        }
        for id in ids {
            let (x, z, burning, rate, kind) = {
                let b = self.world.bodies.get_mut(&id).unwrap();
                if !b.alive {
                    continue;
                }
                b.x += (b.vx + b.kvx) * dt;
                b.z += (b.vz + b.kvz) * dt;
                let fade = (-dt / KNOCK_TAU).exp();
                b.kvx *= fade;
                b.kvz *= fade;
                b.slowed = b.slow_until > tick;
                b.burning = b.burn_until > tick;
                if tick >= b.expires {
                    b.alive = false;
                }
                (b.x, b.z, b.burning, b.burn_rate, b.kind)
            };
            if burning && kind == BodyKind::Creature {
                if let Some(b) = self.world.bodies.get_mut(&id) {
                    b.health -= rate * dt;
                }
                if tick % (1.0 / dt) as u64 == 0 {
                    self.world.event("hit", format!("{}", rate.round().max(1.) as i64), x, z);
                }
            }
            let b = self.world.bodies.get(&id).unwrap();
            if b.alive && matches!(b.kind, BodyKind::Creature | BodyKind::Seed) && b.health <= 0. {
                let on_zero = b.on_zero.clone();
                self.world.bodies.get_mut(&id).unwrap().alive = false;
                if kind == BodyKind::Seed && on_zero == "fall" {
                    self.town_gone = true;
                    self.world.event("fall", "THE SEED BREAKS".into(), x, z);
                } else {
                    self.world.event("gone", "gone".into(), x, z);
                }
            }
        }
        // spells whose body is gone end; exhausted casts are dropped once their bodies are
        let bodies = &self.world.bodies;
        let alive = |id: &str| bodies.get(id).is_some_and(|b| b.alive);
        self.instances.retain(|i| {
            if i.definition.kind == Kind::Spell {
                // a cast lives while what it formed does
                i.status == Status::Running && i.bindings.get("form").is_some_and(|f| alive(f))
            } else {
                alive(&i.holder)
            }
        });
        self.world.bodies.retain(|_, b| b.alive || matches!(b.kind, BodyKind::Seed | BodyKind::Hero | BodyKind::Building));
    }

    /// The on-device model's name for the fusion cast at `tick` (a later fusion is not renamed by an old reply).
    pub fn rename(&mut self, tick: u64, name: &str) -> bool {
        let n = name.trim();
        if n.len() < 3 || n.len() > 20 || !n.chars().all(|c| c.is_alphabetic() || c == ' ' || c == '\'') {
            return false;
        }
        match &mut self.fused {
            Some((d, f)) if f.tick == tick => {
                f.name = n.to_string();
                f.named = true;
                d.title = Some(n.to_string());
                for i in &mut self.instances {
                    if i.definition.name == d.name {
                        i.definition.title = Some(n.to_string());
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Selects slot `slot` (0-based) when it holds a spell.
    pub fn select(&mut self, slot: usize) {
        if slot < self.slots.len() {
            self.selected = slot;
        }
    }

    /// The cast key: the selected spell is armed; if one is already armed (within `ARM` seconds) the two fuse, the
    /// armed one first (cast order).
    pub fn press_cast(&mut self) -> Result<(), String> {
        let name = self.slots.get(self.selected).cloned().ok_or("no spell selected")?;
        match self.armed.take() {
            Some((first, _)) => self.cast_pair(&first, &name, None).map(|_| ()),
            None => {
                self.armed = Some((name, self.world.tick));
                Ok(())
            }
        }
    }

    /// Weather: fire falling at the hero (0..1). The first time it falls the hero catches a flake: fire joins the slots.
    pub fn weather(&mut self, fire: f64) {
        if fire >= 0.25 && !self.fire_caught {
            self.fire_caught = true;
            self.slots.push("fire".into());
            let (x, z) = (self.world.bodies["hero"].x, self.world.bodies["hero"].z);
            self.world.event("caught", "FIRE CAUGHT".into(), x, z);
        }
    }

    pub fn seed(&self) -> &Body {
        &self.world.bodies["seed"]
    }
}
