//! Spellcraft in the engine: everything in the game is a spell, made of sensors, effectors and an action (Paul). Ported
//! from creature_3d_studio's `crates/spellcraft` (the YAML phases, the component catalog, mana-metered logic, the
//! `WorldAdapter`), with Paul's two visible parts (shape and element), cast-order fusion, and the moon town as its
//! world. Modules: `definition` (the data and its validator), `runtime` (cast instances, mana, Luau brains),
//! `fusion` (first gives the shape, second the element), `town` (the town's world and the spells running in it).
//! docs/spells.md in the game repo is the design.
//!
//! Luau: `spellcraft.town(spec)` steps the town to the scene's time and returns what the HUD shows (the seed, the
//! slots, the last fusion, the creatures, recent hits, the buildings' spells); `spellcraft.spell(name)` a spell's
//! definition; `spellcraft.fuse(first, second)` a fusion's.
pub mod aim;
pub mod lifecycle;
pub mod player;
pub mod player_world;
pub mod casting;
pub mod funding;
pub mod capacity;
pub mod capacity_view;
pub mod supply;
pub mod supply_view;
pub mod authoring;
pub mod authoring_view;
pub mod definition;
pub mod fusion;
pub mod fx;
pub mod runtime;
pub mod town;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use atelico_component::mlua::{self, Lua, LuaSerdeExt, Table};
use atelico_component::{register, Component};
use serde::Serialize;
use serde_json::{json, Value};

use crate::town::{BodyKind, Library, Moon, Town};

register!(Component { name: "spellcraft", lua: install, ..Component::NONE });

/// The project's spells/ dir (the host runs in the project's root).
fn spells_dir() -> Option<PathBuf> {
    let d = std::env::current_dir().ok()?.join("spells");
    d.is_dir().then_some(d)
}

/// When any file in spells/ last changed (a change starts the town over with the new spells).
fn spells_stamp(dir: &Option<PathBuf>) -> u128 {
    let Some(d) = dir else { return 0 };
    std::fs::read_dir(d).into_iter().flatten().flatten().filter_map(|e| e.metadata().ok()?.modified().ok()).map(|t| t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)).sum()
}

/// A town kept per scene: stepped forward to each asked time, started over when time goes back or spells change.
struct Running {
    town: Town,
    /// Whether the town casts on its own (the last caller said: the HUD in the editor's preview, not in Play).
    autoplay: bool,
    t: f64,
    stamp: u128,
    next_cast: f64,
    turn: usize,
}

static TOWNS: Mutex<BTreeMap<String, Running>> = Mutex::new(BTreeMap::new());

/// The most scene time one call simulates (a jump further starts the town over, that far back).
const CATCH_UP: f64 = 30.0;

/// The one town of a scene, shared: every reader of the scene's town (the HUD, its World's SpellFx, packages/creatures)
/// goes through these, so they all see the same simulation. A town is kept per key (the scene's id).
///
/// `f` on the town kept under `key`, if it runs (read only: it is not stepped).
pub fn with_town<R>(key: &str, f: impl FnOnce(&Town) -> R) -> Option<R> {
    TOWNS.lock().ok()?.get(key).map(|r| f(&r.town))
}

/// Steps the scene's town to scene time `t` under the moons (towards each, y up) and the fire falling, then `f` on
/// it. Any reader may call it: a town already at `t` is not stepped again, so the HUD and the World's parts calling it
/// in the same frame advance it once. Input (keys) comes only through the HUD (`spellcraft.town`); a town no one plays
/// casts on its own as the editor's preview does.
pub fn town_at_time<R>(key: &str, t: f64, moons: [Option<[f64; 3]>; 2], fire: f64, f: impl FnOnce(&Town) -> R) -> Option<R> {
    let autoplay = TOWNS.lock().ok()?.get(key).map_or(true, |r| r.autoplay);
    let _ = town_at(Spec { key: key.into(), t, purple: moons[0], orange: moons[1], fire, autoplay, select: None, cast: false });
    with_town(key, f)
}

/// The town's own casts in the editor's preview (not playing), so a still town shows fusions: every 8 s, a fusion in
/// turn once fire is caught (fire then wind, wind then fire, fire then fire), else wind. In Play the keys cast.
const CAST_EVERY: f64 = 8.0;
fn autoplay(r: &mut Running) {
    while r.t >= r.next_cast {
        r.next_cast += CAST_EVERY;
        if r.town.fire_caught {
            let pairs = [("fire", "wind"), ("wind", "fire"), ("fire", "fire")];
            let (a, b) = pairs[r.turn % pairs.len()];
            r.turn += 1;
            let _ = r.town.cast_pair(a, b, None);
        } else {
            let _ = r.town.cast("wind");
        }
    }
}

#[derive(serde::Deserialize, Default)]
struct Spec {
    key: String,
    t: f64,
    #[serde(default)]
    purple: Option<[f64; 3]>,
    #[serde(default)]
    orange: Option<[f64; 3]>,
    #[serde(default)]
    fire: f64,
    #[serde(default = "yes")]
    autoplay: bool,
    /// Play's keys this frame: a slot chosen (1-based), the cast key newly down.
    #[serde(default)]
    select: Option<usize>,
    #[serde(default)]
    cast: bool,
}

fn yes() -> bool {
    true
}

#[derive(Serialize)]
struct Snapshot {
    t: f64,
    tick: u64,
    town_gone: bool,
    fire_caught: bool,
    seed: Value,
    slots: Vec<Value>,
    fused: Option<Value>,
    fusion: Option<Value>,
    armed: Option<Value>,
    selected: usize,
    creatures: Vec<Value>,
    events: Vec<Value>,
    buildings: Vec<Value>,
    error: Option<String>,
}

fn snapshot(r: &Running, error: Option<String>) -> Snapshot {
    let town = &r.town;
    let w = &town.world;
    let dt = w.dt;
    let s = town.seed();
    let last_hit = w.events.iter().rev().find(|e| e.kind == "hit" && (e.x - s.x).abs() < 1e-6 && (e.z - s.z).abs() < 1e-6).map(|e| (w.tick - e.tick) as f64 * dt);
    let card = |d: &definition::SpellDefinition| {
        json!({ "name": d.name, "title": d.title(), "shape": d.shape, "element": d.element })
    };
    Snapshot {
        t: r.t,
        tick: w.tick,
        town_gone: town.town_gone,
        fire_caught: town.fire_caught,
        seed: json!({ "health": s.health.max(0.).ceil(), "max": s.max, "x": s.x, "z": s.z, "hit_ago": last_hit }),
        slots: town.slots.iter().filter_map(|n| town.lib.spells.get(n)).map(card).collect(),
        fused: town.fused.as_ref().map(|(d, _)| card(d)),
        fusion: town.fused.as_ref().map(|(_, f)| json!({ "first": f.first, "second": f.second, "name": f.name, "fallback": f.fallback, "named": f.named, "tick": f.tick, "age": (w.tick - f.tick) as f64 * dt })),
        armed: town.armed.as_ref().map(|(n, at)| json!({ "name": n, "age": (w.tick - at) as f64 * dt })),
        selected: town.selected,
        creatures: w
            .bodies
            .values()
            .filter(|b| b.kind == BodyKind::Creature && b.alive)
            .map(|b| json!({ "id": b.id, "x": b.x, "z": b.z, "health": b.health.max(0.), "max": b.max, "moon": b.moon.map(|m| if m == Moon::Purple { "purple" } else { "orange" }), "way": b.way, "underground": b.underground, "slowed": b.slowed, "burning": b.burning }))
            .collect(),
        events: w.events.iter().filter(|e| (w.tick - e.tick) as f64 * dt <= 2.0).map(|e| json!({ "age": (w.tick - e.tick) as f64 * dt, "kind": e.kind, "text": e.text, "x": e.x, "z": e.z })).collect(),
        buildings: town
            .lib
            .town
            .buildings
            .iter()
            .filter_map(|b| {
                let d = town.lib.spells.get(&b.spell)?;
                let i = town.instances.iter().find(|i| i.holder == b.name);
                let title = |kind: &str| d.first_of(kind).map(|(_, c)| c.title.to_lowercase());
                Some(json!({
                    "name": b.name, "title": d.title(), "x": b.x, "z": b.z,
                    "sensor": title("sensor"), "logic": title("logic"), "effect": title("effector"),
                    "energy": i.map(|i| i.energy), "spent": i.map(|i| i.spent_total()), "work": i.map(|i| i.work),
                }))
            })
            .collect(),
        error,
    }
}

fn town_at(spec: Spec) -> Snapshot {
    let dir = spells_dir();
    let stamp = spells_stamp(&dir);
    let mut towns = TOWNS.lock().unwrap();
    let fresh = |stamp| -> Result<Running, String> { Ok(Running { town: Town::new(Library::load(dir.as_deref())?)?, autoplay: true, t: 0., stamp, next_cast: 2.0, turn: 0 }) };
    let restart = towns.get(&spec.key).is_none_or(|r| spec.t + 1e-6 < r.t || r.stamp != stamp || spec.t - r.t > CATCH_UP);
    if restart {
        match fresh(stamp) {
            Ok(r) => {
                towns.insert(spec.key.clone(), r);
            }
            Err(e) => {
                let mut s = Snapshot { t: spec.t, tick: 0, town_gone: false, fire_caught: false, seed: Value::Null, slots: vec![], fused: None, fusion: None, armed: None, selected: 0, creatures: vec![], events: vec![], buildings: vec![], error: Some(e.clone()) };
                s.error = Some(e);
                return s;
            }
        }
    }
    let r = towns.get_mut(&spec.key).unwrap();
    r.autoplay = spec.autoplay;
    // a town started late (or a clock that jumped) begins at most CATCH_UP seconds back: one call never simulates more
    if spec.t - r.t > CATCH_UP {
        r.t = spec.t - CATCH_UP;
        r.next_cast = r.t + 2.0;
    }
    r.town.world.moons = [spec.purple, spec.orange];
    r.town.weather(spec.fire);
    if let Some(n) = spec.select {
        r.town.select(n.saturating_sub(1));
    }
    if spec.cast {
        let _ = r.town.press_cast();
    }
    let dt = r.town.world.dt;
    while r.t + dt <= spec.t + 1e-9 {
        r.t += dt;
        if spec.autoplay {
            autoplay(r);
        }
        r.town.step();
    }
    snapshot(r, None)
}

/// One frame of the town under `key` from Rust (what `spellcraft.town` does for Luau): stepped to `t` under the moons
/// (towards each, y up) and the fire falling, with Play's keys this frame. For tools that render the town offscreen.
pub fn drive(key: &str, t: f64, moons: [Option<[f64; 3]>; 2], fire: f64, select: Option<usize>, cast: bool) -> Value {
    let spec = Spec { key: key.into(), t, purple: moons[0], orange: moons[1], fire, autoplay: false, select, cast };
    serde_json::to_value(town_at(spec)).unwrap_or(Value::Null)
}

fn install(lua: &Lua) -> mlua::Result<()> {
    let t = lua.create_table()?;
    t.set(
        "town",
        lua.create_function(|lua, spec: Table| {
            let spec: Spec = lua.from_value(mlua::Value::Table(spec))?;
            lua.to_value(&town_at(spec))
        })?,
    )?;
    // spellcraft.rename(key, tick, name): the on-device model's name for the fusion cast at `tick`
    t.set(
        "rename",
        lua.create_function(|_, (key, tick, name): (String, u64, String)| {
            let mut towns = TOWNS.lock().unwrap();
            Ok(towns.get_mut(&key).is_some_and(|r| r.town.rename(tick, &name)))
        })?,
    )?;
    t.set(
        "spell",
        lua.create_function(|lua, name: String| {
            let lib = Library::load(spells_dir().as_deref()).map_err(mlua::Error::runtime)?;
            let d = lib.spell(&name).map_err(mlua::Error::runtime)?;
            lua.to_value(d)
        })?,
    )?;
    t.set(
        "fuse",
        lua.create_function(|lua, (a, b): (String, String)| {
            let lib = Library::load(spells_dir().as_deref()).map_err(mlua::Error::runtime)?;
            let d = fusion::fuse(lib.spell(&a).map_err(mlua::Error::runtime)?, lib.spell(&b).map_err(mlua::Error::runtime)?, &lib.names).map_err(mlua::Error::runtime)?;
            lua.to_value(&d)
        })?,
    )?;
    lua.globals().set("spellcraft", t)
}

// ---- the spells seen in the world (fx.rs draws them). The part and its registrations live in this module: the one
// the host links by name, so their linkme statics are always kept.
use std::any::Any;
use atelico_data::Data;
use atelico_render_3d::color::{hex, Rgb};
use atelico_render_3d::material::Material;
use atelico_render_3d::mesh::Mesh;
use atelico_render_3d::part::{Around, Ground, WorldPart};
use atelico_render_3d::probes::Lamp;
use atelico_render_3d::world_part;
use glam::vec3;
use serde::Deserialize;
use crate::fx::{mesh, now, Fx};

atelico_render_3d::material_presets!("materials.toml");

/// The spells' effects in a World: which town's casts it draws (the scene's id: the Hud steps the same town).
#[derive(Data, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct SpellFx {
    /// The town whose casts it draws: the scene's id (town, town-ink).
    pub town: String,
    /// Effects drawn this much bigger (1: as the spells' sizes say).
    #[data(range(0.25, 4.0))]
    pub scale: f64,
    /// The world's look (pixel, ink): a graph spell-<name>-<look>.graph.yaml is used where there is one.
    pub look: String,
    pub show: String,
}

impl Default for SpellFx {
    fn default() -> Self {
        SpellFx { town: "town".into(), scale: 1.0, look: String::new(), show: String::new() }
    }
}

impl WorldPart for SpellFx {
    fn type_name(&self) -> &'static str {
        "SpellFx"
    }
    fn props(&self) -> Value {
        serde_json::to_value(self).unwrap_or_default()
    }
    fn colours(&self, _: &[Material]) -> Vec<(Rgb, String)> {
        vec![
            (hex(0xff3d0a), "spell-flame".into()),
            (hex(0xffc21a), "spell-flame".into()),
            (hex(0x9fdcff), "spell-wind".into()),
            (hex(0xff9a3c), "spell-glow".into()),
            (hex(0xfff4d6), "spell-glow".into()),
            (hex(0xa89a86), "spell-dust".into()),
            (hex(0x3f8fd8), "spell-wind".into()),
        ]
    }
    fn animate_around(&self, t: f32, ground: Ground, _around: &Around) -> Option<Mesh> {
        let fx = now(&self.town);
        if fx.is_empty() {
            return None;
        }
        Some(mesh(&fx, t, ground, self.scale as f32))
    }
    fn things_around(&self, t: f32, ground: Ground, _around: &Around) -> Vec<(usize, Vec<[f32; 4]>)> {
        let fx = now(&self.town);
        crate::fx::sprites(&fx, t, ground, self.scale as f32, &self.look).iter().map(|s| (component_particles::PARTICLES, s.entry())).collect()
    }
    fn lamps(&self, _t: f32, _day: f32, ground: Ground) -> Vec<Lamp> {
        // flames light the street round them
        now(&self.town)
            .iter()
            .filter_map(|f| match f {
                Fx::Body { look, at, .. } if look.ends_with("wall") || look == "flame-wall" => Some(Lamp { at: vec3(at.x, ground(at.x, at.z).max(0.0) + 1.0, at.z), colour: vec3(1.4, 0.45, 0.08), reach: 4.0 }),
                Fx::Moment { kind, at, age, .. } if kind == "burst" && *age < 0.35 => Some(Lamp { at: vec3(at.x, 1.0, at.z), colour: vec3(1.5, 0.5, 0.1) * (1.0 - age / 0.35), reach: 4.0 }),
                _ => None,
            })
            .take(6)
            .collect()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
world_part!(SpellFx);

#[cfg(test)]
mod tests;

#[cfg(test)]
mod supply_tests;

#[cfg(test)]
mod authoring_tests;

#[cfg(test)]
mod funding_tests;

#[cfg(test)]
mod capacity_tests;
