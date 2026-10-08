use std::collections::BTreeMap;
use std::sync::Arc;

use super::definition::{parse, validate, Step};
use super::fusion::{self, Choice, Names};
use super::runtime::{self, Brain, Effect, Observation, Status, WorldAdapter};
use super::town::{Library, Town};

/// A world for testing spells alone: a few bodies, a sensor that always finds `found`.
#[derive(Default)]
struct Fixture {
    found: Vec<String>,
    applied: Vec<Effect>,
}

impl WorldAdapter for Fixture {
    fn form(&mut self, instance: &str, _caster: &str, step: &Step) -> Result<String, String> {
        Ok(format!("{instance}:{}", step.id))
    }
    fn sense(&mut self, _step: &Step, target: &str, tick: u64) -> Result<Observation, String> {
        Ok(Observation { tick, target: target.into(), bodies: self.found.clone(), distances: vec![1.0; self.found.len()] })
    }
    fn apply(&mut self, e: &Effect) -> Result<(), String> {
        self.applied.push(e.clone());
        Ok(())
    }
    fn exists(&self, _body: &str) -> bool {
        true
    }
    fn take_contact(&mut self, _body: &str) -> Option<String> {
        Some("creature".into())
    }
}

fn lib() -> Library {
    Library::load(None).expect("the shipped spells load")
}

/// Every shipped spell parses and passes spellcraft's validator; the catalog parses.
#[test]
fn the_shipped_spells_are_valid() {
    let l = lib();
    for name in ["wind", "fire", "watchtower", "climber", "digger", "town-seed"] {
        assert!(l.spells.contains_key(name), "{name}");
        validate(&l.spells[name]).unwrap();
    }
    assert!(l.brains.contains_key("watchtower"));
    assert!(super::definition::catalog().len() >= 16);
    // the watchtower reads as Paul's card: sensor "creature in my shadow", effect "slow"
    let t = &l.spells["watchtower"];
    assert_eq!(t.first_of("sensor").unwrap().1.title, "Creature in my shadow");
    assert_eq!(t.first_of("effector").unwrap().1.title, "Slow");
}

/// The validator refuses a forward reference, too many steps for the container, a param out of bounds.
#[test]
fn the_validator_refuses_bad_spells() {
    let base = include_str!("../../../spells/watchtower.spell.yaml");
    assert!(parse(&base.replace("target: in_shadow, every: 6, params: {factor", "target: later, every: 6, params: {factor")).unwrap_err().contains("forward reference"));
    assert!(parse(&base.replace("container: grand", "container: seed").replace("allocation: 140", "allocation: 40")).is_ok(), "4 steps fit a seed container");
    let five = base.replace("    - {id: slow,", "    - {id: slow2, use: effect.slow, target: in_shadow}\n    - {id: slow,").replace("container: grand", "container: seed").replace("allocation: 140", "allocation: 40");
    assert!(parse(&five).unwrap_err().contains("capacity"));
    assert!(parse(&base.replace("factor: 0.25", "factor: 7")).unwrap_err().contains("out of bounds"));
}

/// Cast order: fire then wind is a Wall of Flame (fire's wall, wind's push, still burning); wind then fire a Fireball
/// (wind's orb, fire's burst); the same twice a dud (Hearthglow: a warm glow, no harm).
#[test]
fn fusion_follows_cast_order() {
    let l = lib();
    let (fire, wind) = (&l.spells["fire"], &l.spells["wind"]);
    let wall = fusion::fuse(fire, wind, &l.names).unwrap();
    assert_eq!((wall.title(), wall.shape.as_deref(), wall.element.as_deref()), ("Wall of Flame", Some("wall"), Some("wind")));
    let uses: Vec<&str> = wall.steps().map(|(_, s)| s.component.as_str()).collect();
    assert!(uses.contains(&"body.wall") && uses.contains(&"force.push") && uses.contains(&"effect.burn"), "{uses:?}");

    let ball = fusion::fuse(wind, fire, &l.names).unwrap();
    assert_eq!((ball.title(), ball.shape.as_deref(), ball.element.as_deref()), ("Fireball", Some("orb"), Some("fire")));
    let uses: Vec<&str> = ball.steps().map(|(_, s)| s.component.as_str()).collect();
    assert!(uses.contains(&"body.orb") && uses.contains(&"fire.burst") && uses.contains(&"motion.launch"), "{uses:?}");

    let dud = fusion::fuse(fire, fire, &l.names).unwrap();
    assert_eq!(dud.title(), "Hearthglow");
    let uses: Vec<&str> = dud.steps().map(|(_, s)| s.component.as_str()).collect();
    assert!(uses.contains(&"effect.warm") && !uses.iter().any(|u| u.starts_with("effect.burn") || *u == "fire.burst" || *u == "force.push"), "{uses:?}");

    // a pair no table names: Element Shape
    let frost = fusion::spell("frost", None, "bolt", "frost", None, false).unwrap();
    assert_eq!(fusion::fuse(&frost, fire, &l.names).unwrap().title(), "Fire Bolt");
}

/// The model only picks: a choice that breaks the cast order is refused; its name is used; the schema only offers
/// the legal parts.
#[test]
fn a_choice_is_checked_and_built_by_rust() {
    let l = lib();
    let (fire, wind) = (&l.spells["fire"], &l.spells["wind"]);
    let good = Choice { shape: "wall".into(), element: "wind".into(), carrier: Some("fire".into()), name: "Gale of Embers".into() };
    assert_eq!(fusion::build(fire, wind, &good).unwrap().title(), "Gale of Embers");
    let wrong = Choice { shape: "orb".into(), ..good.clone() };
    assert!(fusion::build(fire, wind, &wrong).is_err());
    let bad_name = Choice { name: "x".into(), ..good };
    assert!(fusion::build(fire, wind, &bad_name).is_err());
    let s = fusion::schema(fire, wind).unwrap();
    assert_eq!(s["properties"]["shape"]["enum"], serde_json::json!(["wall"]));
    assert_eq!(s["properties"]["element"]["enum"], serde_json::json!(["wind"]));
    let names: Names = Names::shipped();
    assert_eq!(names.dud["fire"], "Hearthglow");
}

/// Mana pays for computation: a brain that loops spends more than one that reads a summary; the same brain and
/// observation spend the same; an instance that cannot pay is exhausted with the rest dissipated.
#[test]
fn mana_meters_sensing_logic_and_effects() {
    let yaml = |entry: &str, allocation: f64| {
        format!(
            "version: 1\nname: probe\nkind: building\nmana: {{source: town, allocation: {allocation}}}\ncontainer: field\nphases:\n  on_start:\n    - {{id: me, use: body.self, output: me}}\n  on_tick:\n    - {{id: look, use: sense.nearby, target: me, output: seen}}\n    - {{id: think, use: logic.lua, input: seen, params: {{entry: {entry}}}}}\n    - {{id: slow, use: effect.slow, target: seen}}\n"
        )
    };
    let brain = Arc::new(
        Brain::compile("return { full = function(ctx) local n = 0 for i = 1, 200 do n = n + 1 end return #ctx.observation.bodies > 0 end, lean = function(ctx) return #ctx.observation.bodies > 0 end }").unwrap(),
    );
    let run = |entry: &str| {
        let d = parse(&yaml(entry, 40.)).unwrap();
        let mut w = Fixture { found: vec!["c1".into()], ..Default::default() };
        let mut i = runtime::cast("p", "tower", &d, Some(brain.clone()), 40., &mut w, 0).unwrap();
        runtime::tick(&mut i, &mut w, 1);
        (i.spent["logic"], i.spent["sensors"], i.spent["effectors"], i.work, w.applied.len())
    };
    let (full, sensed, acted, full_work, applied) = run("full");
    let (lean, _, _, lean_work, _) = run("lean");
    assert!(full > lean * 5.0, "a loop costs more: {full} vs {lean} ({full_work} vs {lean_work} units)");
    assert!((sensed - 0.18).abs() < 1e-9 && (acted - 0.6).abs() < 1e-9 && applied == 1, "{sensed} {acted} {applied}");
    assert_eq!(run("full").3, full_work, "work units are deterministic");

    // too little mana: the sensor is paid, the brain's work runs out, the rest dissipates
    let d = parse(&yaml("full", 0.3)).unwrap();
    let mut w = Fixture { found: vec!["c1".into()], ..Default::default() };
    let mut i = runtime::cast("p", "tower", &d, Some(brain.clone()), 0.3, &mut w, 0).unwrap();
    runtime::tick(&mut i, &mut w, 1);
    assert_eq!(i.status, Status::Exhausted);
    assert!(w.applied.is_empty(), "nothing acted without mana");
    let total: f64 = i.spent.values().sum();
    assert!((total - 0.3).abs() < 1e-9, "every unit accounted for: {:?}", i.spent);

    // nothing sensed: the effect sleeps and costs nothing
    let d = parse(&yaml("lean", 40.)).unwrap();
    let mut w = Fixture::default();
    let mut i = runtime::cast("p", "tower", &d, Some(brain), 40., &mut w, 0).unwrap();
    runtime::tick(&mut i, &mut w, 1);
    assert_eq!(i.spent["effectors"], 0.);
}

fn up(elevation_deg: f64, bearing_deg: f64) -> [f64; 3] {
    let (e, b) = (elevation_deg.to_radians(), bearing_deg.to_radians());
    [e.cos() * b.sin(), e.sin(), -e.cos() * b.cos()]
}

/// Creatures rise while their moon is up and go for the seed; each one there takes a health a second; at zero the
/// seed breaks and the town is gone.
#[test]
fn the_seed_breaks_and_the_town_is_gone() {
    let mut town = Town::new(lib()).unwrap();
    town.world.moons = [Some(up(40., 0.)), Some(up(30., 90.))];
    let start = town.seed().health;
    assert_eq!(start, 24.);
    let mut ticks = 0;
    while !town.town_gone && ticks < 20 * 600 {
        town.step();
        ticks += 1;
    }
    assert!(town.town_gone, "the seed fell within ten minutes (health {})", town.seed().health);
    assert!(town.world.events.iter().any(|e| e.kind == "fall"));
    // once gone, nothing more happens
    let tick = town.world.tick;
    town.step();
    assert_eq!(town.world.tick, tick);
    // with no moon up no creature rises, and the seed holds
    let mut calm = Town::new(lib()).unwrap();
    for _ in 0..20 * 60 {
        calm.step();
    }
    assert_eq!(calm.seed().health, 24.);
}

/// The watchtower's spell slows a creature in its shadow (and pays for it), and the player's fusions hurt creatures.
#[test]
fn the_tower_slows_and_fusions_hurt() {
    let mut town = Town::new(lib()).unwrap();
    // the purple moon low in the north: the tower (at z -6.6) throws its shadow south, over the square the
    // climbers come into
    town.world.moons = [Some(up(25., 0.)), None];
    let mut slowed = false;
    for _ in 0..20 * 40 {
        town.step();
        slowed |= town.world.bodies.values().any(|b| b.slowed);
        if slowed {
            break;
        }
    }
    assert!(slowed, "a creature crossed the tower's shadow and was slowed");
    let tower = town.instances.iter().find(|i| i.holder == "watchtower").unwrap();
    assert!(tower.spent["sensors"] > 0. && tower.spent["effectors"] > 0. && tower.work > 0);

    town.weather(1.0);
    assert!(town.slots.iter().any(|s| s == "fire"), "fire caught from the falling flakes");
    let hp = |t: &Town| t.world.bodies.values().filter(|b| b.alive && b.kind == super::town::BodyKind::Creature).map(|b| b.health).sum::<f64>();
    let before = hp(&town);
    assert_eq!(town.cast_pair("wind", "fire", None).unwrap(), "Fireball");
    for _ in 0..20 * 3 {
        town.step();
    }
    assert!(hp(&town) < before || town.world.events.iter().any(|e| e.text == "BURN" || e.kind == "burst"), "the fireball hit something");
    let _: BTreeMap<String, f64> = BTreeMap::new();
}


/// The cast key: a spell cast alone goes after the arming window; a second cast inside it fuses the two in cast
/// order; the model's name for that fusion replaces the fallback, and an old reply cannot rename a later fusion.
#[test]
fn keys_arm_then_fuse_in_cast_order() {
    let mut town = Town::new(lib()).unwrap();
    town.world.moons = [Some(up(40., 0.)), None];
    town.weather(1.0);
    assert_eq!(town.slots, vec!["wind".to_string(), "fire".to_string()]);
    // alone: wind armed, then cast by itself when the window closes
    town.select(0);
    town.press_cast().unwrap();
    assert!(town.armed.is_some());
    for _ in 0..(super::town::ARM * 20.0) as usize + 2 {
        town.step();
    }
    assert!(town.armed.is_none() && town.fused.is_none(), "wind went alone");
    // a pair: fire, then wind inside the window: a fusion, fire first
    town.select(1);
    town.press_cast().unwrap();
    town.step();
    town.select(0);
    town.press_cast().unwrap();
    let (d, f) = town.fused.clone().unwrap();
    assert_eq!((f.first.as_str(), f.second.as_str(), f.fallback.as_str(), f.named), ("fire", "wind", "Wall of Flame", false));
    assert_eq!(d.shape.as_deref(), Some("wall"));
    assert!(!town.rename(f.tick + 99, "Blazing Curtain"), "a reply for another fusion");
    assert!(town.rename(f.tick, "Blazing Curtain"));
    let (d, f) = town.fused.clone().unwrap();
    assert_eq!((d.title(), f.name.as_str(), f.named), ("Blazing Curtain", "Blazing Curtain", true));
    assert!(!town.rename(f.tick, "x"), "a name the schema would not allow");
}

/// The effects part is registered (its data type and its World part), and each spell's body draws something.
#[test]
fn the_effects_are_registered_and_draw() {
    assert!(atelico_data::registered().any(|s| s.name == "SpellFx"));
    assert!(atelico_render_3d::part::parse("SpellFx", serde_json::json!({ "town": "x" })).is_some());
    let flat = |_: f32, _: f32| 0.0;
    // the silhouettes are meshes, the fire, smoke and wind are particles from graphs/spell-*.graph.yaml
    for look in ["wind-orb", "fire-wall", "flame-wall"] {
        let fx = vec![super::fx::Fx::Body { look: look.into(), at: glam::Vec3::ZERO, to: glam::vec3(4.0, 0.0, 0.0), vel: glam::vec3(8.0, 0.0, 0.0), age: 0.5, left: 2.0 }];
        assert!(!super::fx::mesh(&fx, 1.0, &flat, 1.0).0.is_empty(), "{look}");
    }
    component_particles::part::use_root(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    for look in ["wind-orb", "fireball", "fire-wall", "flame-wall"] {
        let fx = vec![super::fx::Fx::Body { look: look.into(), at: glam::Vec3::ZERO, to: glam::vec3(4.0, 0.0, 0.0), vel: glam::vec3(8.0, 0.0, 0.0), age: 0.5, left: 2.0 }];
        assert!(!super::fx::sprites(&fx, 1.0, &flat, 1.0, "").is_empty(), "{look} has particles");
    }
    for kind in ["burst", "hit", "warm", "knock"] {
        let fx = vec![super::fx::Fx::Moment { kind: kind.into(), at: glam::Vec3::ZERO, age: 0.05, dir: glam::Vec3::X }];
        assert!(!super::fx::sprites(&fx, 1.0, &flat, 1.0, "").is_empty(), "{kind} has particles");
    }
}
