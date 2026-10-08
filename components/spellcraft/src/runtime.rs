//! Spells at work: a cast instance owns the mana transferred to it and spends it on sensing, logic, rules and
//! effects, and stops when it runs out (spellcraft `lib.rs`: `Instance`, `charge`, the ordered phases, the contact hook
//! read on its cadence, scheduled sensors that cost nothing while they sleep). It acts on a world only through
//! `WorldAdapter` (spellcraft's trait, narrowed to what the steps need), so the same spell runs in the moon town
//! (town.rs), a test fixture, or a future physics world.
//!
//! Logic runs as Luau brains (`logic.lua`): spellcraft meters Lua 5.4 per VM instruction with a hook; one binary can
//! link one Lua, and the engine's is Luau, whose VM calls an interrupt at every function call and loop turn. Each
//! interrupt is one work unit and costs the catalog's `logic.lua` price, so a brain that loops pays more than one that
//! reads a cached summary, as in spellcraft. Work units are deterministic for a given brain and observation.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use atelico_component::mlua::{self, Function, Lua, LuaSerdeExt, Table, Value as LuaValue, VmState};
use serde::Serialize;
use serde_json::Value;

use crate::definition::{component, SpellDefinition, Step};

const EPS: f64 = 1e-9;
/// A contact hook read and a contact rule's activation (spellcraft's tariffs).
pub const CONTACT_READ: f64 = 0.025;
pub const CONTACT_RULE: f64 = 0.03;
/// Work units one brain call may use.
pub const WORK_LIMIT: u64 = 20_000;

/// A sensor's cached sample.
#[derive(Clone, Debug, Default, Serialize, PartialEq)]
pub struct Observation {
    pub tick: u64,
    pub target: String,
    pub bodies: Vec<String>,
    pub distances: Vec<f64>,
}

/// An effect handed to the world, already paid for.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Effect {
    /// The component id: effect.slow, fire.burst, vital.health, move.toward, ...
    pub kind: String,
    pub step: String,
    pub instance: String,
    pub targets: Vec<String>,
    pub tick: u64,
    pub mana: f64,
    pub params: BTreeMap<String, Value>,
}

/// What a spell needs from a world (spellcraft's `WorldAdapter`).
pub trait WorldAdapter {
    /// Forms a body of `shape` (body.bolt, body.wall, body.orb) for `instance`, cast by `caster`; its id.
    fn form(&mut self, instance: &str, caster: &str, step: &Step) -> Result<String, String>;
    /// A sensor sample (sense.nearby, sense.shadow) round `target`.
    fn sense(&mut self, step: &Step, target: &str, tick: u64) -> Result<Observation, String>;
    /// Applies a paid effect.
    fn apply(&mut self, effect: &Effect) -> Result<(), String>;
    fn exists(&self, body: &str) -> bool;
    /// The body `body` touched since it was last asked, if any.
    fn take_contact(&mut self, body: &str) -> Option<String>;
}

/// One spell's brain: its Luau source compiled once, its functions by name.
pub struct Brain {
    lua: Lua,
    functions: Table,
    work: Arc<AtomicU64>,
    limit: Arc<Mutex<u64>>,
}

impl Brain {
    pub fn compile(source: &str) -> Result<Brain, String> {
        if source.len() > crate::definition::MAX_SOURCE_BYTES {
            return Err("brain source too large".into());
        }
        let lua = Lua::new();
        lua.sandbox(true).map_err(|e| e.to_string())?;
        let work = Arc::new(AtomicU64::new(0));
        let limit = Arc::new(Mutex::new(WORK_LIMIT));
        let (w, l) = (work.clone(), limit.clone());
        lua.set_interrupt(move |_| {
            let n = w.fetch_add(1, Ordering::Relaxed) + 1;
            if n > *l.lock().unwrap() {
                return Err(mlua::Error::runtime("out of mana or work"));
            }
            Ok(VmState::Continue)
        });
        let functions: Table = lua.load(source).set_name("brain").eval().map_err(|e| e.to_string())?;
        Ok(Brain { lua, functions, work, limit })
    }

    /// Validate the named entry without executing any spell work.
    pub fn validate_entry(&self, entry: &str) -> Result<(), String> {
        self.functions.get::<Function>(entry).map(|_| ()).map_err(|_| format!("missing Lua entry: {entry}"))
    }

    /// Calls `entry` with the observation; returns its decision and the work units it used. `budget` caps the units
    /// (the mana the instance can still pay for).
    pub fn decide(&self, entry: &str, observation: Option<&Observation>, budget: u64) -> (Result<bool, String>, u64) {
        self.work.store(0, Ordering::Relaxed);
        *self.limit.lock().unwrap() = budget.min(WORK_LIMIT);
        let result = (|| -> mlua::Result<bool> {
            let f: Function = self.functions.get(entry)?;
            let ctx = self.lua.create_table()?;
            match observation {
                Some(o) => ctx.set("observation", self.lua.to_value(o)?)?,
                None => ctx.set("observation", LuaValue::Nil)?,
            }
            let v: LuaValue = f.call(ctx)?;
            Ok(match v {
                LuaValue::Boolean(b) => b,
                LuaValue::Nil => false,
                _ => true,
            })
        })();
        (result.map_err(|e| e.to_string()), self.work.load(Ordering::Relaxed))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Running,
    Exhausted,
    Faulted,
}

#[derive(Clone, Debug, Serialize)]
pub struct Trace {
    pub tick: u64,
    pub phase: &'static str,
    pub step: String,
    pub mana: f64,
}

/// A cast instance (spellcraft's `Instance`, trimmed to what runs here).
#[derive(Serialize)]
pub struct Instance {
    pub id: String,
    /// The body that holds it (its `self`), or the caster for a cast spell.
    pub holder: String,
    pub definition: SpellDefinition,
    /// Exact authored-source fingerprint for finite player admissions; empty for legacy demo casts.
    pub revision: String,
    pub initial: f64,
    pub energy: f64,
    pub status: Status,
    pub why: Option<String>,
    /// Spent by category: sensors, logic, rules, effectors, dissipation.
    pub spent: BTreeMap<&'static str, f64>,
    pub bindings: BTreeMap<String, String>,
    pub observations: BTreeMap<String, Observation>,
    pub next: BTreeMap<String, u64>,
    /// Work units its brain has used in all.
    pub work: u64,
    pub decision: Option<bool>,
    pub trace: Vec<Trace>,
    #[serde(skip)]
    brain: Option<Arc<Brain>>,
}

impl Instance {
    /// The amount, if the instance can pay it; else what it had goes to dissipation and it is exhausted
    /// (spellcraft's `charge`). Work already done is never refunded.
    pub fn charge(&mut self, category: &'static str, amount: f64) -> bool {
        if self.status != Status::Running {
            return false;
        }
        if self.energy + EPS < amount {
            *self.spent.entry("dissipation").or_default() += self.energy;
            self.energy = 0.;
            self.status = Status::Exhausted;
            return false;
        }
        self.energy = (self.energy - amount).max(0.);
        *self.spent.entry(category).or_default() += amount;
        if self.energy < EPS {
            self.energy = 0.;
            self.status = Status::Exhausted;
        }
        true
    }

    fn resolve(&self, selector: &str, other: Option<&str>) -> Result<Vec<String>, String> {
        if selector == "event.other" {
            return other.map(|o| vec![o.to_string()]).ok_or_else(|| "no contacted body".to_string());
        }
        if let Some(b) = self.bindings.get(selector) {
            return Ok(vec![b.clone()]);
        }
        if let Some(o) = self.observations.get(selector) {
            return Ok(o.bodies.clone());
        }
        // an observation not yet sampled is empty
        Ok(Vec::new())
    }

    fn fault(&mut self, why: String) {
        if self.status == Status::Running {
            self.status = Status::Faulted;
        }
        self.why = Some(why);
    }

    pub fn spent_total(&self) -> f64 {
        self.spent.values().sum()
    }
}

/// Casts `definition` held by `holder` with `mana` transferred once (at most its allocation): validates, runs
/// `on_start` in order. `brain`: its Luau brain, when it has a `logic.lua` step.
/// Checks every entry before a cartridge can be debited or any on-start action can run.
pub(crate) fn validate_brain(definition: &SpellDefinition, brain: Option<&Brain>) -> Result<(), String> {
    for (_, step) in definition.steps().filter(|(_, s)| s.component == "logic.lua") {
        let brain = brain.ok_or_else(|| format!("{}: a logic.lua step needs a brain", definition.name))?;
        brain.validate_entry(&crate::definition::text(step, "entry", "decide"))?;
    }
    Ok(())
}
pub fn cast(id: &str, holder: &str, definition: &SpellDefinition, brain: Option<Arc<Brain>>, mana: f64, world: &mut dyn WorldAdapter, tick: u64) -> Result<Instance, String> {
    let mut instance = prepare(id, holder, definition, brain, mana, tick)?;
    start(&mut instance, world, tick);
    Ok(instance)
}
pub(crate) fn prepare(id: &str, holder: &str, definition: &SpellDefinition, brain: Option<Arc<Brain>>, mana: f64, tick: u64) -> Result<Instance, String> {
    crate::definition::validate(definition)?;
    validate_brain(definition, brain.as_deref())?;
    if !mana.is_finite() || mana < 0.0 { return Err("funding must be finite and nonnegative".into()); }
    let energy = mana.min(definition.mana.allocation).max(0.);
    let mut i = Instance {
        id: id.into(),
        holder: holder.into(),
        definition: definition.clone(),
        revision: String::new(),
        initial: energy,
        energy,
        status: Status::Running,
        why: None,
        spent: ["sensors", "logic", "rules", "effectors", "dissipation"].into_iter().map(|k| (k, 0.)).collect(),
        bindings: BTreeMap::new(),
        observations: BTreeMap::new(),
        next: BTreeMap::new(),
        work: 0,
        decision: None,
        trace: Vec::new(),
        brain,
    };
    for s in &definition.phases.on_tick {
        i.next.insert(s.id.clone(), tick + s.every.unwrap_or(1).max(1));
    }
    if !definition.phases.on_contact.target.is_empty() {
        i.next.insert("contact".into(), tick + definition.phases.on_contact.every.max(1));
    }
    Ok(i)
}
pub(crate) fn start(i: &mut Instance, world: &mut dyn WorldAdapter, tick: u64) {
    let steps = i.definition.phases.on_start.clone();
    run(i, "on_start", &steps, world, tick, None);
}

/// One tick: the contact hook if it is due (a paid read; a contact runs `on_contact`), then every due `on_tick` step
/// in order. A `logic.lua` step that decides false skips the rest of that tick's steps. Sleeping steps cost nothing.
pub fn tick(i: &mut Instance, world: &mut dyn WorldAdapter, tick: u64) {
    if i.status != Status::Running {
        return;
    }
    if i.next.get("contact").is_some_and(|n| tick >= *n) {
        let every = i.definition.phases.on_contact.every;
        if every == 0 {
            i.next.remove("contact");
        } else {
            i.next.insert("contact".into(), tick + every);
        }
        let hook = i.definition.phases.on_contact.target.clone();
        if let Some(body) = i.bindings.get(&hook).cloned() {
            if world.exists(&body) && i.charge("sensors", CONTACT_READ) {
                if let Some(other) = world.take_contact(&body) {
                    if i.charge("rules", CONTACT_RULE) {
                        let steps = i.definition.phases.on_contact.steps.clone();
                        run(i, "on_contact", &steps, world, tick, Some(&other));
                    }
                }
            }
        }
    }
    let steps: Vec<Step> = i.definition.phases.on_tick.iter().filter(|s| i.next.get(&s.id).is_some_and(|n| tick >= *n)).cloned().collect();
    for s in &steps {
        let every = s.every.unwrap_or(1);
        if every == 0 {
            i.next.remove(&s.id);
        } else {
            i.next.insert(s.id.clone(), tick + every);
        }
    }
    run(i, "on_tick", &steps, world, tick, None);
}

fn run(i: &mut Instance, phase: &'static str, steps: &[Step], world: &mut dyn WorldAdapter, tick: u64, other: Option<&str>) {
    for s in steps {
        if i.status != Status::Running {
            break;
        }
        let before = i.energy;
        match step(i, phase, s, world, tick, other) {
            Ok(go_on) => {
                let cost = before - i.energy;
                if cost > 0. {
                    i.trace.push(Trace { tick, phase, step: s.id.clone(), mana: cost });
                    if i.trace.len() > 32 {
                        i.trace.remove(0);
                    }
                }
                if !go_on {
                    break;
                }
            }
            Err(e) => {
                i.fault(format!("{}: {e}", s.id));
                break;
            }
        }
    }
}

/// One step; Ok(false) stops the rest of this run (a brain decided no).
fn step(i: &mut Instance, _phase: &'static str, s: &Step, world: &mut dyn WorldAdapter, tick: u64, other: Option<&str>) -> Result<bool, String> {
    let c = component(&s.component).ok_or_else(|| format!("unknown component {}", s.component))?;
    match c.kind.as_str() {
        "construction" => {
            if !i.charge("effectors", c.mana) {
                return Ok(false);
            }
            let body = if s.component == "body.self" { i.holder.clone() } else { world.form(&i.id, &i.holder, s)? };
            i.bindings.insert(s.output.clone().ok_or("no output name")?, body);
        }
        "sensor" => {
            let target = i.resolve(s.target.as_deref().ok_or("no target")?, other)?;
            let Some(target) = target.into_iter().next().filter(|t| world.exists(t)) else { return Err("the sensor's target is gone".into()) };
            if !i.charge("sensors", c.mana) {
                return Ok(false);
            }
            let o = world.sense(s, &target, tick)?;
            i.observations.insert(s.output.clone().ok_or("no output name")?, o);
        }
        "logic" => {
            let brain = i.brain.clone().ok_or("no brain")?;
            let budget = (i.energy / c.mana).floor().max(0.) as u64;
            let input = s.input.as_deref().and_then(|k| i.observations.get(k)).cloned();
            let entry = crate::definition::text(s, "entry", "decide");
            let (result, work) = brain.decide(&entry, input.as_ref(), budget);
            i.work += work;
            // the work done is paid for whether or not the brain finished
            let paid = i.charge("logic", work as f64 * c.mana);
            let decision = result?;
            if !paid {
                return Ok(false);
            }
            i.decision = Some(decision);
            return Ok(decision);
        }
        _ => {
            // effectors, motion and vitals: act on the target body, or on every body an observation holds
            let targets = match &s.target {
                Some(t) => i.resolve(t, other)?,
                None => vec![i.holder.clone()],
            };
            let targets: Vec<String> = targets.into_iter().filter(|t| world.exists(t)).collect();
            if targets.is_empty() {
                // nothing to act on: skipped work costs nothing
                return Ok(true);
            }
            let mut params = s.params.clone();
            if let Some(input) = s.input.as_deref() {
                let n = i.observations.get(input).map_or(0, |o| o.bodies.len());
                params.insert("_count".into(), Value::from(n));
            }
            let e = Effect { kind: s.component.clone(), step: s.id.clone(), instance: i.id.clone(), targets, tick, mana: c.mana, params };
            if !i.charge("effectors", c.mana) {
                return Ok(false);
            }
            world.apply(&e)?;
        }
    }
    Ok(true)
}
