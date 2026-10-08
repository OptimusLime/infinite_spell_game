//! Spell data: what a spell, a building, a creature or the town seed *is* (Paul: "everything is quote-unquote the
//! spell system ... sensors effectors and an action"). Ported from creature_3d_studio's
//! `crates/spellcraft/src/definition.rs`: the same versioned YAML (`on_start`, `on_tick`, `on_contact` phases of
//! ordered steps, each one catalog component with typed target/input/output handles and bounded params), the same
//! containers and the same validator checks (container capacity, mana bounds, phase compatibility, no forward or
//! mistyped references). Added for the game: `kind` (what holds the spell), `title`, and Paul's two visible parts,
//! `shape` and `element`.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CATALOG_YAML: &str = include_str!("../catalog.yaml");
pub const MAX_SOURCE_BYTES: usize = 64 * 1024;

/// What holds the spell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// A spell the player casts.
    #[default]
    Spell,
    /// A building's spell (its defences).
    Building,
    /// A creature's spell (how it moves and what it does).
    Creature,
    /// The town seed's spell (its health and what happens when it breaks).
    Seed,
}

/// Where its mana comes from: the player's wand, the town (buildings are fed by the seed), a moon (creatures), the
/// seed itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Wand,
    Town,
    Moon,
    Seed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mana {
    pub source: Source,
    pub allocation: f64,
}

/// A spell definition (spellcraft `SpellDefinition`, plus kind, title, shape and element).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellDefinition {
    pub version: u32,
    pub name: String,
    /// The name a player reads ("Wall of Flame"); `name` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub kind: Kind,
    /// Paul's two visible parts: the shape it takes (bolt, wall, orb) and its element (fire, wind, frost).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    pub mana: Mana,
    /// seed (4 steps, 40 mana), field (8, 80), grand (16, 140): spellcraft's containers.
    pub container: String,
    pub phases: Phases,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Phases {
    #[serde(default)]
    pub on_start: Vec<Step>,
    #[serde(default)]
    pub on_tick: Vec<Step>,
    #[serde(default)]
    pub on_contact: ContactPhase,
}

/// The contact hook: bound to a body made at start, read every `every` ticks, its steps run on each contact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ContactPhase {
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub every: u64,
    #[serde(default)]
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    #[serde(rename = "use")]
    pub component: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Ticks between runs in on_tick (1: every tick; 0: sleeps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every: Option<u64>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

impl SpellDefinition {
    pub fn title(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.name)
    }
    pub fn steps(&self) -> impl Iterator<Item = (&'static str, &Step)> {
        self.phases.on_start.iter().map(|s| ("on_start", s)).chain(self.phases.on_tick.iter().map(|s| ("on_tick", s))).chain(self.phases.on_contact.steps.iter().map(|s| ("on_contact", s)))
    }
    /// Its first step of a kind (sensor, effector, ...), for a card: "Sensor: creature in my shadow".
    pub fn first_of(&self, kind: &str) -> Option<(&Step, &'static Component)> {
        self.steps().filter_map(|(_, s)| component(&s.component).map(|c| (s, c))).find(|(_, c)| c.kind == kind)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    #[serde(rename = "type")]
    pub value_type: String,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub choices: Vec<String>,
    pub required: bool,
}

/// A catalog component (spellcraft's `Component`, with its price as a number).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub description: String,
    pub phases: Vec<String>,
    #[serde(default)]
    pub target: Option<Port>,
    #[serde(default)]
    pub input: Option<Port>,
    #[serde(default)]
    pub output: Option<Port>,
    pub params: BTreeMap<String, Parameter>,
    /// Mana per run (sensors per sample; with `unit: work`, per VM work unit).
    pub mana: f64,
    #[serde(default)]
    pub unit: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    version: u32,
    components: Vec<Component>,
}

pub fn catalog() -> &'static [Component] {
    static CATALOG: OnceLock<Vec<Component>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let c: Catalog = serde_yaml_ng::from_str(CATALOG_YAML).expect("catalog.yaml parses");
        assert_eq!(c.version, 1, "catalog version");
        c.components
    })
}

pub fn component(id: &str) -> Option<&'static Component> {
    catalog().iter().find(|c| c.id == id)
}

/// A step's number param, else the catalog default, else `fallback`.
pub fn number(step: &Step, key: &str, fallback: f64) -> f64 {
    step.params
        .get(key)
        .and_then(Value::as_f64)
        .or_else(|| component(&step.component).and_then(|c| c.params.get(key)).and_then(|p| p.default.as_ref()).and_then(Value::as_f64))
        .unwrap_or(fallback)
}

/// A step's choice/text param, else the catalog default.
pub fn text(step: &Step, key: &str, fallback: &str) -> String {
    step.params
        .get(key)
        .and_then(Value::as_str)
        .map(String::from)
        .or_else(|| component(&step.component).and_then(|c| c.params.get(key)).and_then(|p| p.default.as_ref()).and_then(Value::as_str).map(String::from))
        .unwrap_or_else(|| fallback.to_string())
}

pub fn parse(text: &str) -> Result<SpellDefinition, String> {
    if text.len() > MAX_SOURCE_BYTES {
        return Err(format!("spell source over {MAX_SOURCE_BYTES} bytes"));
    }
    let d: SpellDefinition = serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?;
    validate(&d)?;
    Ok(d)
}

pub fn to_yaml(d: &SpellDefinition) -> Result<String, String> {
    serde_yaml_ng::to_string(d).map_err(|e| e.to_string())
}

/// A container's step capacity and most mana (spellcraft's).
pub fn container(name: &str) -> Option<(usize, f64)> {
    match name {
        "seed" => Some((4, 40.)),
        "field" => Some((8, 80.)),
        "grand" => Some((16, 140.)),
        _ => None,
    }
}

/// Spellcraft's checks, before a definition is accepted.
pub fn validate(d: &SpellDefinition) -> Result<(), String> {
    if d.version != 1 {
        return Err(format!("unsupported spell version {}; expected 1", d.version));
    }
    if d.name.trim().is_empty() || d.name.chars().count() > 80 {
        return Err("a spell's name has 1-80 characters".into());
    }
    let (capacity, most) = container(&d.container).ok_or_else(|| format!("unknown container {}", d.container))?;
    if !d.mana.allocation.is_finite() || d.mana.allocation <= 0. || d.mana.allocation > most {
        return Err(format!("mana allocation must be positive and at most {most} for a {} container", d.container));
    }
    let count = d.phases.on_start.len() + d.phases.on_tick.len() + d.phases.on_contact.steps.len();
    let hook = usize::from(!d.phases.on_contact.target.is_empty());
    if count + hook > capacity {
        return Err(format!("container capacity exceeded: {count} steps and {hook} contact hook need {} slots, a {} container has {capacity}", count + hook, d.container));
    }
    if d.phases.on_contact.every > 3600 {
        return Err("the contact hook's schedule is 0-3600 ticks".into());
    }
    let mut ids = BTreeSet::new();
    let mut symbols: BTreeMap<String, String> = BTreeMap::new();
    phase("on_start", &d.phases.on_start, &mut ids, &mut symbols)?;
    if d.phases.on_contact.target.is_empty() {
        if !d.phases.on_contact.steps.is_empty() {
            return Err("contact steps need a contact hook target".into());
        }
    } else {
        reference(&d.phases.on_contact.target, "body", "the contact hook's target", "on_start", &symbols)?;
    }
    phase("on_tick", &d.phases.on_tick, &mut ids, &mut symbols)?;
    phase("on_contact", &d.phases.on_contact.steps, &mut ids, &mut symbols)?;
    Ok(())
}

fn valid_id(v: &str) -> bool {
    !v.is_empty() && v.len() <= 64 && v.bytes().next().is_some_and(|c| c.is_ascii_alphabetic() || c == b'_') && v.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

fn reference(r: &str, want: &str, field: &str, phase: &str, symbols: &BTreeMap<String, String>) -> Result<(), String> {
    if r == "event.other" {
        return if phase == "on_contact" && (want == "body" || want == "targets") { Ok(()) } else { Err(format!("{field}: event.other is a body only on contact")) };
    }
    match symbols.get(r) {
        Some(found) if found == want || (want == "targets" && (found == "body" || found == "observations")) => Ok(()),
        Some(found) => Err(format!("{field}: {r} is a {found}; expected {want}")),
        None => Err(format!("{field}: unknown or forward reference {r}; make the {want} earlier")),
    }
}

fn phase(name: &str, steps: &[Step], ids: &mut BTreeSet<String>, symbols: &mut BTreeMap<String, String>) -> Result<(), String> {
    for s in steps {
        if !valid_id(&s.id) || !ids.insert(s.id.clone()) {
            return Err(format!("step id {:?} is not a fresh identifier", s.id));
        }
        let c = component(&s.component).ok_or_else(|| format!("{}: unknown component {}", s.id, s.component))?;
        if !c.phases.iter().any(|p| p == name) {
            return Err(format!("{}: {} cannot run in {name}", s.id, s.component));
        }
        if let Some(every) = s.every {
            if name != "on_tick" || every > 3600 {
                return Err(format!("{}: a schedule is for on_tick steps, 0-3600 ticks", s.id));
            }
        }
        for (port, value, field) in [(&c.target, &s.target, "target"), (&c.input, &s.input, "input")] {
            match (port, value) {
                (Some(p), Some(v)) => reference(v, &p.value_type, &format!("{}: {field}", s.id), name, symbols)?,
                (Some(p), None) if p.required => return Err(format!("{}: needs a {field}", s.id)),
                (None, Some(_)) => return Err(format!("{}: {} takes no {field}", s.id, s.component)),
                _ => {}
            }
        }
        for (key, v) in &s.params {
            let p = c.params.get(key).ok_or_else(|| format!("{}: {} has no param {key}", s.id, s.component))?;
            match p.value_type.as_str() {
                "number" => {
                    let n = v.as_f64().ok_or_else(|| format!("{}: {key} is a number", s.id))?;
                    if p.min.is_some_and(|m| n < m) || p.max.is_some_and(|m| n > m) || !n.is_finite() {
                        return Err(format!("{}: {key} {n} is out of bounds {:?}..{:?}", s.id, p.min, p.max));
                    }
                }
                "choice" => {
                    let t = v.as_str().ok_or_else(|| format!("{}: {key} is a choice", s.id))?;
                    if !p.choices.iter().any(|c| c == t) {
                        return Err(format!("{}: {key} {t:?} is not one of {:?}", s.id, p.choices));
                    }
                }
                _ => {
                    v.as_str().ok_or_else(|| format!("{}: {key} is text", s.id))?;
                }
            }
        }
        match (&c.output, &s.output) {
            (Some(p), Some(o)) => {
                if !valid_id(o) || symbols.contains_key(o) {
                    return Err(format!("{}: output {o:?} is not a fresh name", s.id));
                }
                symbols.insert(o.clone(), p.value_type.clone());
            }
            (Some(p), None) if p.required => return Err(format!("{}: needs an output name", s.id)),
            (None, Some(_)) => return Err(format!("{}: {} has no output", s.id, s.component)),
            _ => {}
        }
    }
    Ok(())
}
