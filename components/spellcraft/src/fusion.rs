//! Cast-order fusion: two spells cast one after the other become one. The first gives the **shape**, the second the
//! **element**, and the first's element rides along as the carrier's lesser effect. Fire then wind: fire's wall,
//! wind's push, still burning: a Wall of Flame. Wind then fire: wind's orb, fire's burst: a Fireball. The same spell
//! twice is a dud (fire then fire: Hearthglow, a warm glow that harms nothing).
//!
//! The parts are closed sets and Rust builds the definition from them, so a fused spell is always valid: the
//! on-device model (item-synthesis) only picks among `choices` (and names the result); `build` makes the spell.
//! Names for known pairs are typed data (spells/fusion.toml).
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::definition::{validate, ContactPhase, Kind, Mana, Phases, Source, SpellDefinition, Step};

/// spells/fusion.toml: the names of known pairs and the dud's.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Names {
    /// The name of a spell cast twice in a row.
    pub dud: BTreeMap<String, String>,
    pub named: Vec<Named>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Named {
    pub first: String,
    pub second: String,
    pub name: String,
}

pub const NAMES: &str = include_str!("../../../spells/fusion.toml");

impl Names {
    pub fn shipped() -> Names {
        toml::from_str(NAMES).expect("spells/fusion.toml parses")
    }
    pub fn parse(text: &str) -> Result<Names, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }
}

pub const SHAPES: [&str; 3] = ["bolt", "wall", "orb"];
pub const ELEMENTS: [&str; 3] = ["fire", "wind", "frost"];

/// What the model may pick for a pair: the shape and element are fixed by the cast order; it picks the name (and,
/// for an unknown pair, whether the carrier's lesser effect is kept).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub shape: String,
    pub element: String,
    pub carrier: Option<String>,
    pub name: String,
}

/// A spell's shape and element (its two visible parts).
pub fn parts(d: &SpellDefinition) -> Result<(String, String), String> {
    match (&d.shape, &d.element) {
        (Some(s), Some(e)) => Ok((s.clone(), e.clone())),
        _ => Err(format!("{} has no shape and element: only spells fuse", d.name)),
    }
}

/// The legal choices for `first` then `second`, as a JSON schema for the model (only a legal `Choice` can come back).
pub fn schema(first: &SpellDefinition, second: &SpellDefinition) -> Result<Value, String> {
    let (shape, carrier) = parts(first)?;
    let (_, element) = parts(second)?;
    let carriers: Vec<Value> = if carrier == element { vec![Value::Null] } else { vec![json!(carrier), Value::Null] };
    Ok(json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["shape", "element", "carrier", "name"],
        "properties": {
            "shape": { "enum": [shape] },
            "element": { "enum": [element] },
            "carrier": { "enum": carriers },
            "name": { "type": "string", "minLength": 3, "maxLength": 20, "pattern": "^[A-Z][A-Za-z' ]+$" },
        }
    }))
}

/// The choice Rust makes alone: the shape and element by cast order, the carrier kept, the name from the table (or
/// "Element Shape": "Frost Bolt").
pub fn default_choice(first: &SpellDefinition, second: &SpellDefinition, names: &Names) -> Result<Choice, String> {
    let (shape, carrier) = parts(first)?;
    let (_, element) = parts(second)?;
    let dud = first.name == second.name;
    let name = if dud {
        names.dud.get(&first.name).cloned().unwrap_or_else(|| format!("Dull {}", title(&shape)))
    } else {
        names.named.iter().find(|n| n.first == first.name && n.second == second.name).map(|n| n.name.clone()).unwrap_or_else(|| format!("{} {}", title(&element), title(&shape)))
    };
    Ok(Choice { shape, element, carrier: (!dud && carrier != parts(second)?.1).then_some(carrier), name })
}

fn title(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn step(id: &str, component: &str, target: Option<&str>, output: Option<&str>, params: Value) -> Step {
    let params = params.as_object().map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default();
    Step { id: id.into(), component: component.into(), target: target.map(String::from), input: None, output: output.map(String::from), every: None, params }
}

/// The effect steps an element brings on contact (`main`: the second spell's element, full strength; else the
/// carrier's, lesser).
fn element_steps(element: &str, main: bool, shape: &str) -> Vec<Step> {
    let k = if main { 1.0 } else { 0.5 };
    let tag = if main { "" } else { "_carried" };
    match element {
        "fire" => {
            let mut v = vec![step(&format!("burn{tag}"), "effect.burn", Some("event.other"), None, json!({ "per_second": 1.5 * k, "seconds": 3.0 }))];
            if main && shape == "orb" {
                v.push(step("burst", "fire.burst", Some("form"), None, json!({ "radius": 2.5 })));
            }
            v
        }
        "wind" => vec![step(&format!("push{tag}"), "force.push", Some("event.other"), None, json!({ "strength": 3.0 * k }))],
        "frost" => vec![step(&format!("slow{tag}"), "effect.slow", Some("event.other"), None, json!({ "factor": if main { 0.25 } else { 0.6 }, "seconds": 2.0 }))],
        _ => vec![],
    }
}

/// The spell a base spell or a fusion is: its shape formed (and launched when it flies), a contact hook on it, the
/// element's effects (and the carrier's) on contact. A dud glows warm and harms nothing.
pub fn spell(name: &str, title_: Option<&str>, shape: &str, element: &str, carrier: Option<&str>, dud: bool) -> Result<SpellDefinition, String> {
    if !SHAPES.contains(&shape) || !ELEMENTS.contains(&element) || carrier.is_some_and(|c| !ELEMENTS.contains(&c)) {
        return Err(format!("unknown parts {shape} / {element} / {carrier:?}"));
    }
    let mut on_start = vec![step("form", &format!("body.{shape}"), None, Some("form"), if shape == "wall" { json!({ "length": 7.0 }) } else { json!({}) })];
    if shape != "wall" {
        on_start.push(step("launch", "motion.launch", Some("form"), None, json!({ "speed": if shape == "bolt" { 14.0 } else { 5.0 } })));
    }
    if dud {
        // a dud puffs warm the moment it forms, and does nothing else
        on_start.push(step("glow", "effect.warm", Some("form"), None, json!({ "radius": 1.0 })));
    }
    let steps = if dud {
        vec![]
    } else {
        let mut v = element_steps(element, true, shape);
        if let Some(c) = carrier {
            v.extend(element_steps(c, false, shape));
        }
        v
    };
    let d = SpellDefinition {
        version: 1,
        name: name.to_lowercase().replace(' ', "-").replace('\'', ""),
        title: Some(title_.unwrap_or(name).to_string()),
        kind: Kind::Spell,
        shape: Some(shape.into()),
        element: Some(element.into()),
        mana: Mana { source: Source::Wand, allocation: 20.0 },
        container: "field".into(),
        phases: Phases { on_start, on_tick: vec![], on_contact: if dud { ContactPhase::default() } else { ContactPhase { target: "form".into(), every: 1, steps } } },
    };
    validate(&d)?;
    Ok(d)
}

/// The fused spell for `first` then `second` and a choice (the model's, or `default_choice`): checked against the
/// legal choices, then built by Rust.
pub fn build(first: &SpellDefinition, second: &SpellDefinition, c: &Choice) -> Result<SpellDefinition, String> {
    let (shape, carrier) = parts(first)?;
    let (_, element) = parts(second)?;
    if c.shape != shape || c.element != element {
        return Err(format!("cast order gives {shape} and {element}, not {} and {}", c.shape, c.element));
    }
    if c.carrier.as_ref().is_some_and(|k| *k != carrier || carrier == element) {
        return Err(format!("the carrier can only be {carrier}"));
    }
    let n = c.name.trim();
    if n.len() < 3 || n.len() > 20 || !n.chars().all(|ch| ch.is_alphabetic() || ch == ' ' || ch == '\'') {
        return Err(format!("name {n:?}: 3-20 letters"));
    }
    spell(n, Some(n), &shape, &element, c.carrier.as_deref(), first.name == second.name)
}

/// `first` then `second` with Rust's own choice.
pub fn fuse(first: &SpellDefinition, second: &SpellDefinition, names: &Names) -> Result<SpellDefinition, String> {
    build(first, second, &default_choice(first, second, names)?)
}
