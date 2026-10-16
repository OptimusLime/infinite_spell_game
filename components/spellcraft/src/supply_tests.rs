use super::{
    casting::*,
    definition::Step,
    runtime::{self, Effect, Observation, WorldAdapter},
    supply::*,
    supply_view,
};
use atelico_equipment::{Acceptance, Config, Equipment, SlotSpec};
use atelico_inventory::*;
use atelico_store::Store;

fn item(s: &str) -> ItemId {
    ItemId::new(s).unwrap()
}
fn definition(s: &str) -> DefinitionId {
    DefinitionId::new(s).unwrap()
}
fn slot(s: &str) -> SlotId {
    SlotId::new(s).unwrap()
}
fn owner() -> EntityId {
    EntityId::new("fixture-actor").unwrap()
}
fn energy(v: f64) -> Energy {
    Energy::new(v).unwrap()
}
fn at(column: u16) -> Location {
    Location::Grid {
        grid: GridId::new("pack").unwrap(),
        column,
        row: 0,
        orientation: Orientation::Normal,
    }
}
pub(super) fn fixture() -> (Inventory, Equipment, Supply) {
    let mut inv = Inventory::new();
    inv.add_grid(Grid {
        id: GridId::new("pack").unwrap(),
        holder: Holder::Entity(owner()),
        size: Size::new(5, 2).unwrap(),
    })
    .unwrap();
    for (n, d, x) in [
        ("wand", "wand-type", 0),
        ("a", "cartridge-type", 1),
        ("b", "cartridge-type", 2),
    ] {
        inv.register_at(
            ItemInstance {
                id: item(n),
                definition: definition(d),
                footprint: Footprint {
                    size: Size::new(1, 1).unwrap(),
                    rotatable: false,
                },
            },
            at(x),
        )
        .unwrap();
    }
    let eq = Equipment::try_from(Config {
        owner: owner(),
        slots: vec![SlotSpec {
            id: slot("hand"),
            title: "Hand".into(),
            accepts: Acceptance::Definitions {
                definitions: vec![definition("wand-type")],
            },
        }],
    })
    .unwrap();
    eq.equip(&mut inv, &item("wand"), &at(0), &slot("hand"))
        .unwrap();
    let supply = Supply::restore(
        &inv,
        SupplySnapshot {
            cartridges: vec![
                Cartridge {
                    item: item("a"),
                    capacity: energy(40.0),
                    remaining: energy(40.0),
                },
                Cartridge {
                    item: item("b"),
                    capacity: energy(25.0),
                    remaining: energy(25.0),
                },
            ],
            wands: vec![Wand {
                item: item("wand"),
                cartridge_slot: slot("cell"),
                accepts: vec![definition("cartridge-type")],
            }],
        },
    )
    .unwrap();
    supply
        .load(
            &mut inv,
            &eq,
            &slot("hand"),
            &item("wand"),
            &item("a"),
            &at(1),
            None,
            None,
        )
        .unwrap();
    (inv, eq, supply)
}
pub(super) fn spell() -> AuthoredSpell {
    AuthoredSpell::new(AuthoredSource {
        definition: include_str!("../../../spells/wind.spell.yaml").into(),
        brain: None,
    })
    .unwrap()
}
pub(super) fn request(casting: &Casting, spell: &AuthoredSpell, id: &str, source: &str) -> CastRequest {
    CastRequest {
        aim: crate::aim::Aim::new([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]).unwrap(),
        id: id.into(),
        owner: owner(),
        equipment_slot: slot("hand"),
        expected_wand: item("wand"),
        expected_cartridge: item(source),
        expected_remaining: casting.supply().cartridge(&item(source)).unwrap().remaining,
        expected_revision: spell.revision().into(),
        holder: "hero-body".into(),
        tick: 4,
    }
}
#[derive(Default)]
struct World {
    forms: usize,
    effects: Vec<Effect>,
    fail_form: bool,
}
impl WorldAdapter for World {
    fn form(&mut self, instance: &str, _: &str, _: &Step) -> Result<String, String> {
        self.forms += 1;
        if self.fail_form {
            return Err("fixture form fault".into());
        }
        Ok(format!("{instance}:body"))
    }
    fn sense(&mut self, _: &Step, target: &str, tick: u64) -> Result<Observation, String> {
        Ok(Observation {
            target: target.into(),
            tick,
            bodies: vec![],
            distances: vec![],
        })
    }
    fn apply(&mut self, effect: &Effect) -> Result<(), String> {
        self.effects.push(effect.clone());
        Ok(())
    }
    fn exists(&self, _: &str) -> bool {
        true
    }
    fn take_contact(&mut self, _: &str) -> Option<String> {
        None
    }
}

#[test]
fn finite_energy_and_supply_restore_validate_physical_identities() {
    for value in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Energy::new(value).is_err());
    }
    assert!(serde_json::from_str::<Energy>("-0.01").is_err());
    let (mut inv, _, supply) = fixture();
    let original = supply.snapshot();
    let mut bad = original.clone();
    bad.cartridges.push(bad.cartridges[0].clone());
    assert!(Supply::restore(&inv, bad)
        .unwrap_err()
        .contains("duplicate"));
    let mut bad = original.clone();
    bad.cartridges[0].remaining = energy(41.0);
    assert!(Supply::restore(&inv, bad).is_err());
    let mut bad = original.clone();
    bad.wands[0].accepts.clear();
    assert!(Supply::restore(&inv, bad).is_err());
    inv.remove_item(&item("b"), &at(2)).unwrap();
    assert!(Supply::restore(&inv, original)
        .unwrap_err()
        .contains("no live Inventory item"));
}
#[test]
fn owner_total_counts_nested_equipped_and_loaded_ids_once() {
    let (mut inv, eq, supply) = fixture();
    assert_eq!(supply.total(&inv, &owner()).unwrap(), energy(65.0));
    assert_eq!(supply.owned(&inv, &owner()).unwrap().len(), 2);
    inv.add_grid(Grid {
        id: GridId::new("nested").unwrap(),
        holder: Holder::Item(item("wand")),
        size: Size::new(1, 1).unwrap(),
    })
    .unwrap();
    inv.transfer(
        &item("b"),
        &at(2),
        Location::Grid {
            grid: GridId::new("nested").unwrap(),
            column: 0,
            row: 0,
            orientation: Orientation::Normal,
        },
    )
    .unwrap();
    assert_eq!(supply.total(&inv, &owner()).unwrap(), energy(65.0));
    eq.unequip(&mut inv, &slot("hand"), &item("wand"), at(0))
        .unwrap();
    assert_eq!(supply.total(&inv, &owner()).unwrap(), energy(65.0));
    inv.transfer(
        &item("wand"),
        &at(0),
        Location::Slot {
            holder: Holder::Entity(EntityId::new("other").unwrap()),
            slot: slot("custody"),
        },
    )
    .unwrap();
    assert_eq!(supply.total(&inv, &owner()).unwrap(), energy(0.0));
    assert_eq!(
        supply
            .total(&inv, &EntityId::new("other").unwrap())
            .unwrap(),
        energy(65.0)
    );
}
#[test]
fn replacement_is_atomic_and_preserves_old_cartridge() {
    let (mut inv, eq, supply) = fixture();
    let before = inv.snapshot();
    // Both incoming and outgoing would target cells occupied in the final candidate.
    assert!(supply
        .load(
            &mut inv,
            &eq,
            &slot("hand"),
            &item("wand"),
            &item("b"),
            &at(3),
            Some(&item("a")),
            Some(at(1))
        )
        .is_err());
    assert_eq!(inv.snapshot(), before);
    supply
        .load(
            &mut inv,
            &eq,
            &slot("hand"),
            &item("wand"),
            &item("b"),
            &at(2),
            Some(&item("a")),
            Some(at(2)),
        )
        .unwrap();
    assert_eq!(
        supply.loaded(&inv, &item("wand")).unwrap(),
        Some(&item("b"))
    );
    assert_eq!(inv.item(&item("a")).unwrap().location, at(2));
    assert_eq!(supply.total(&inv, &owner()).unwrap(), energy(65.0));
    let before = inv.snapshot();
    assert!(supply
        .unload(
            &mut inv,
            &eq,
            &slot("hand"),
            &item("wand"),
            &item("b"),
            at(2)
        )
        .is_err());
    assert_eq!(inv.snapshot(), before);
}
#[test]
fn admission_debits_once_replay_has_no_effect_and_different_payload_rejects() {
    let (inv, eq, supply) = fixture();
    let mut casts = Casting::new(supply);
    let spell = spell();
    let r = request(&casts, &spell, "input-event-1", "a");
    let admitted = casts.admit(&inv, &eq, &spell, r.clone()).unwrap();
    assert_eq!(admitted.receipt.before, energy(40.0));
    assert_eq!(admitted.receipt.after, energy(20.0));
    let mut world = World::default();
    assert_eq!(world.forms, 0, "admission cannot touch a world");
    let instance = admitted.funded.unwrap().start(&mut world);
    assert_eq!(world.forms, 1);
    assert_eq!(instance.initial, 20.0);
    assert_eq!(instance.revision, spell.revision());
    let replay = casts.admit(&inv, &eq, &spell, r.clone()).unwrap();
    assert!(replay.funded.is_none());
    assert_eq!(casts.receipts().len(), 1);
    assert_eq!(world.forms, 1);
    let mut different = r;
    different.holder = "another-body".into();
    assert!(casts
        .admit(&inv, &eq, &spell, different)
        .err()
        .unwrap()
        .contains("different payload"));
    assert_eq!(
        casts.supply().cartridge(&item("a")).unwrap().remaining,
        energy(20.0)
    );
}
#[test]
fn depletion_rejects_without_effect_replacement_is_deliberate_and_time_does_not_refill() {
    let (mut inv, eq, supply) = fixture();
    let mut casts = Casting::new(supply);
    let spell = spell();
    let mut world = World::default();
    let mut live = Vec::new();
    for id in ["one", "two"] {
        let r = request(&casts, &spell, id, "a");
        live.push(
            casts
                .admit(&inv, &eq, &spell, r)
                .unwrap()
                .funded
                .unwrap()
                .start(&mut world),
        );
    }
    let r = request(&casts, &spell, "too-much", "a");
    let snapshot = casts.snapshot();
    assert!(casts.admit(&inv, &eq, &spell, r).is_err());
    assert_eq!(casts.snapshot(), snapshot);
    assert_eq!(world.forms, 2);
    casts
        .supply()
        .unload(
            &mut inv,
            &eq,
            &slot("hand"),
            &item("wand"),
            &item("a"),
            at(1),
        )
        .unwrap();
    for tick in 5..100 {
        for i in &mut live {
            runtime::tick(i, &mut world, tick);
        }
    }
    assert_eq!(
        casts.supply().cartridge(&item("a")).unwrap().remaining,
        energy(0.0)
    );
    assert!(
        live.iter().all(|i| i.initial == 20.0),
        "disconnect does not remove transferred instance funding"
    );
    casts
        .supply()
        .load(
            &mut inv,
            &eq,
            &slot("hand"),
            &item("wand"),
            &item("b"),
            &at(2),
            None,
            None,
        )
        .unwrap();
    assert_eq!(world.forms, 2, "loading does not cast");
    let r = request(&casts, &spell, "replacement", "b");
    casts
        .admit(&inv, &eq, &spell, r)
        .unwrap()
        .funded
        .unwrap()
        .start(&mut world);
    assert_eq!(world.forms, 3);
    assert_eq!(casts.supply().total(&inv, &owner()).unwrap(), energy(5.0));
    assert!(inv.item(&item("a")).is_some());
}
#[test]
fn revision_source_and_runtime_faults_do_not_refund_or_replay_paid_work() {
    let (inv, eq, supply) = fixture();
    let mut casts = Casting::new(supply);
    let original = spell();
    let mut source = original.source().clone();
    source.definition.push_str("\n# a new authored revision\n");
    let changed = AuthoredSpell::new(source).unwrap();
    assert_ne!(changed.revision(), original.revision());
    let old_request = request(&casts, &original, "stale", "a");
    assert!(casts.admit(&inv, &eq, &changed, old_request).is_err());
    let r = request(&casts, &original, "fault", "a");
    let mut world = World {
        fail_form: true,
        ..World::default()
    };
    let instance = casts
        .admit(&inv, &eq, &original, r.clone())
        .unwrap()
        .funded
        .unwrap()
        .start(&mut world);
    assert_eq!(instance.status, runtime::Status::Faulted);
    assert_eq!(instance.definition, *original.definition());
    assert_eq!(instance.revision, original.revision());
    assert_eq!(
        casts.supply().cartridge(&item("a")).unwrap().remaining,
        energy(20.0)
    );
    assert!(casts
        .admit(&inv, &eq, &original, r)
        .unwrap()
        .funded
        .is_none());
    assert_eq!(world.forms, 1);
}
#[test]
fn brain_entries_are_checked_before_admission_and_each_cast_is_isolated() {
    let yaml = "version: 1\nname: brain-probe\nkind: spell\nmana: {source: wand, allocation: 20}\ncontainer: field\nphases:\n  on_start:\n    - {id: self, use: body.self, output: self}\n  on_tick:\n    - {id: look, use: sense.nearby, target: self, output: seen}\n    - {id: think, use: logic.lua, input: seen, params: {entry: decide}}\n";
    assert!(AuthoredSpell::new(AuthoredSource {
        definition: yaml.into(),
        brain: None
    })
    .is_err());
    assert!(AuthoredSpell::new(AuthoredSource {
        definition: yaml.into(),
        brain: Some("return {wrong=function() return true end}".into())
    })
    .unwrap_err()
    .contains("missing Lua entry"));
    let spell = AuthoredSpell::new(AuthoredSource {
        definition: yaml.into(),
        brain: Some(
            "local calls=0; return {decide=function() calls+=1; return calls==1 end}".into(),
        ),
    })
    .unwrap();
    let (inv, eq, supply) = fixture();
    let mut casts = Casting::new(supply);
    let mut world = World::default();
    let mut instances = Vec::new();
    for id in ["a", "b"] {
        let r = request(&casts, &spell, id, "a");
        instances.push(
            casts
                .admit(&inv, &eq, &spell, r)
                .unwrap()
                .funded
                .unwrap()
                .start(&mut world),
        );
    }
    for instance in &mut instances {
        runtime::tick(instance, &mut world, 5);
        assert_eq!(instance.decision, Some(true));
    }
}
#[test]
fn persisted_receipts_validate_funding_chain_and_replay_after_restore() {
    let (inv, eq, supply) = fixture();
    let mut casts = Casting::new(supply);
    let spell = spell();
    let first = request(&casts, &spell, "first", "a");
    casts.admit(&inv, &eq, &spell, first.clone()).unwrap();
    let second = request(&casts, &spell, "second", "a");
    casts.admit(&inv, &eq, &spell, second).unwrap();
    let snapshot: CastingSnapshot =
        serde_json::from_slice(&serde_json::to_vec(&casts.snapshot()).unwrap()).unwrap();
    let mut restored = Casting::restore(&inv, snapshot.clone()).unwrap();
    assert!(restored
        .admit(&inv, &eq, &spell, first)
        .unwrap()
        .funded
        .is_none());
    let mut invalid = snapshot.clone();
    invalid.receipts.push(invalid.receipts[0].clone());
    assert!(Casting::restore(&inv, invalid).is_err());
    let mut invalid = snapshot.clone();
    invalid.receipts[0].after = energy(21.0);
    assert!(Casting::restore(&inv, invalid).is_err());
    let mut invalid = snapshot;
    invalid.supply.cartridges[0].remaining = energy(10.0);
    assert!(Casting::restore(&inv, invalid).is_err());
}
#[test]
fn normalized_hud_reads_authoritative_total_and_publishes_atomic_field_updates() {
    let (mut inv, eq, supply) = fixture();
    let mut casts = Casting::new(supply);
    let spell = spell();
    let store = Store::new();
    let inventory_binding = atelico_inventory::binding::Binding::new("runtime.inventory").unwrap();
    inventory_binding.install(&store).unwrap();
    inventory_binding.publish(&store, &inv).unwrap();
    let binding =
        supply_view::Binding::new("runtime.supply", inventory_binding.paths().clone()).unwrap();
    binding.install(&store).unwrap();
    let projection = binding
        .project(&casts, &inv, &eq, &slot("hand"), Some(&spell), &[])
        .unwrap();
    binding.publish(&store, &projection).unwrap();
    let before = store
        .read::<supply_view::SupplyView>(&binding.dashboard())
        .unwrap();
    assert_eq!(before.total, 65.0);
    assert_eq!(before.cartridges.len(), 2);
    assert!(before.can_cast);
    let bpath = binding.cartridge(&item("b"));
    let (bid, _) = store.object(&bpath).unwrap();
    let b_revision = store.object_rev(bid);
    let r = request(&casts, &spell, "hud", "a");
    let admitted = casts.admit(&inv, &eq, &spell, r).unwrap();
    let mut world = World::default();
    let live = vec![admitted.funded.unwrap().start(&mut world)];
    binding
        .publish(
            &store,
            &binding
                .project(&casts, &inv, &eq, &slot("hand"), Some(&spell), &live)
                .unwrap(),
        )
        .unwrap();
    let after = store
        .read::<supply_view::SupplyView>(&binding.dashboard())
        .unwrap();
    assert_eq!(after.total, 45.0);
    assert_eq!(after.receipts.len(), 1);
    assert_eq!(after.instances.len(), 1);
    assert_eq!(
        store.object_rev(bid),
        b_revision,
        "unaffected cartridge userdata is not invalidated"
    );
    let running = store
        .read::<supply_view::CastView>(&after.instances[0])
        .unwrap();
    assert_eq!(running.revision, spell.revision());
    assert!((running.initial - running.remaining - running.effectors).abs() < 1e-8);
    let snapshot = casts.snapshot();
    let projection = binding
        .project(&casts, &inv, &eq, &slot("hand"), None, &live)
        .unwrap();
    binding.publish(&store, &projection).unwrap();
    assert_eq!(
        casts.snapshot(),
        snapshot,
        "reading HUD never spends or refills energy"
    );
    assert!(
        !store
            .read::<supply_view::SupplyView>(&binding.dashboard())
            .unwrap()
            .can_cast
    );
    casts
        .supply()
        .unload(
            &mut inv,
            &eq,
            &slot("hand"),
            &item("wand"),
            &item("a"),
            at(1),
        )
        .unwrap();
    inv.transfer(
        &item("a"),
        &at(1),
        Location::Slot {
            holder: Holder::Entity(EntityId::new("other").unwrap()),
            slot: slot("carry"),
        },
    )
    .unwrap();
    let projection = binding
        .project(&casts, &inv, &eq, &slot("hand"), Some(&spell), &live)
        .unwrap();
    assert_eq!(projection.dashboard.total, 25.0);
    assert_eq!(
        projection.receipts.values().next().unwrap().cartridge,
        "",
        "historical receipt does not point at an absent scoped cartridge row"
    );
}

struct Flat;
impl atelico_items::physics::Terrain for Flat {
    fn height(&self, _: f32, _: f32) -> f32 {
        0.0
    }
    fn blocked(&self, _: glam::Vec3, _: f32) -> bool {
        false
    }
}
fn player_fixture() -> crate::player::Player {
    use crate::player_world::{BodySpec, Form, PlayerWorld, Role, Shape, WorldConfig};
    let (inv, eq, supply) = fixture();
    // Engineering fixtures only. These dimensions, contents, masses and health are not game balance.
    let world = PlayerWorld::new(
        WorldConfig {
            tick_seconds: 1.0 / 60.0,
            forms: vec![
                Form {
                    shape: Shape::Orb,
                    radius: 0.1,
                    mass: 1.0,
                    lifetime_ticks: 360,
                },
                Form {
                    shape: Shape::Bolt,
                    radius: 0.08,
                    mass: 1.0,
                    lifetime_ticks: 360,
                },
            ],
        },
        vec![
            BodySpec {
                id: "hero-body".into(),
                role: Role::Actor,
                position: [0.0, 2.0, 0.0],
                radius: 0.3,
                mass: 1.0,
                health: 10.0,
            },
            BodySpec {
                id: "aimed-target".into(),
                role: Role::Target,
                position: [0.0, 1.8, 2.1],
                radius: 0.4,
                mass: 1.0,
                health: 10.0,
            },
            BodySpec {
                id: "nearer-side-target".into(),
                role: Role::Target,
                position: [0.9, 2.0, 1.0],
                radius: 0.2,
                mass: 1.0,
                health: 10.0,
            },
        ],
        Box::new(Flat),
    )
    .unwrap();
    crate::player::Player::new(inv, eq, Casting::new(supply), world, slot("hand")).unwrap()
}
fn player_request(p: &crate::player::Player, id: &str, direction: [f32; 3]) -> CastRequest {
    let mut r = request(p.casting(), p.selected().unwrap(), id, "a");
    r.tick = p.world().tick();
    r.aim = crate::aim::Aim::new([0.0, 2.0, 1.0], direction).unwrap();
    r
}
#[test]
fn player_explicit_aim_misses_nearer_target_and_never_homes_or_refills() {
    let mut p = player_fixture();
    p.select(spell()).unwrap();
    let r = player_request(&p, "aim-straight", [0.0, 0.0, 1.0]);
    p.fire(r.clone()).unwrap();
    p.update_actor("hero-body", [2.0, 2.0, 0.0]).unwrap();
    for _ in 0..4 {
        p.advance().unwrap();
    }
    let body = p
        .world()
        .bodies()
        .into_iter()
        .find(|b| !b.instance.is_empty())
        .unwrap();
    assert!(body.z > 1.0 && body.y < 2.0);
    assert_eq!(body.x, 0.0);
    assert_eq!(body.vx, 0.0, "nearer side target cannot steer the shot");
    for _ in 0..18 {
        p.advance().unwrap();
    }
    let push = p
        .world()
        .effects()
        .iter()
        .find(|e| e.kind == "force.push")
        .expect("authored paid contact effect ran");
    assert_eq!(push.targets, vec!["aimed-target"]);
    assert!(!p
        .world()
        .effects()
        .iter()
        .any(|e| e.targets.iter().any(|t| t == "nearer-side-target")));
    let balance = p
        .casting()
        .supply()
        .cartridge(&item("a"))
        .unwrap()
        .remaining;
    for _ in 0..400 {
        p.advance().unwrap();
    }
    assert_eq!(
        p.casting()
            .supply()
            .cartridge(&item("a"))
            .unwrap()
            .remaining,
        balance
    );
    let count = p.world().effects().len();
    p.fire(r.clone()).unwrap();
    assert_eq!(p.world().effects().len(), count);
    let mut changed = r;
    changed.aim = crate::aim::Aim::new([0.0, 2.0, 1.0], [1.0, 0.0, 0.0]).unwrap();
    assert!(p.fire(changed).unwrap_err().contains("different payload"));
}
#[test]
fn player_selected_authored_revision_changes_shape_and_physical_effect() {
    let mut p = player_fixture();
    let source = AuthoredSource {
        definition: include_str!("../fixtures/projectile-hit.spell.yaml").into(),
        brain: None,
    };
    AuthoredSpell::new(AuthoredSource {
        definition: include_str!("../fixtures/projectile-push.spell.yaml").into(),
        brain: None,
    })
    .unwrap();
    let authored = AuthoredSpell::new(source).unwrap();
    let revision = authored.revision().to_string();
    p.select(authored).unwrap();
    let r = player_request(&p, "authored-bolt", [0.0, 0.0, 1.0]);
    p.fire(r.clone()).unwrap();
    p.select(spell()).unwrap(); // selection does not rewrite the active instance
    p.fire(r).unwrap(); // an exact retry resolves its prior revision, not the now-selected one
    for _ in 0..22 {
        p.advance().unwrap();
    }
    assert_eq!(p.instances().len(), 1);
    assert_eq!(p.instances()[0].revision, revision);
    let shot = p
        .world()
        .bodies()
        .into_iter()
        .find(|b| !b.instance.is_empty())
        .unwrap();
    assert_eq!(shot.shape, "body.bolt");
    let target = p
        .world()
        .bodies()
        .into_iter()
        .find(|b| b.id == "aimed-target")
        .unwrap();
    assert_eq!(target.health, 7.0);
    assert!(!p.world().effects().iter().any(|e| e.kind == "force.push"));
}
#[test]
fn player_invalid_aim_unsupported_wall_and_stale_tick_never_debit() {
    assert!(crate::aim::Aim::new([0.0; 3], [0.0; 3]).is_err());
    assert!(crate::aim::Aim::new([f32::NAN, 0.0, 0.0], [0.0, 0.0, 1.0]).is_err());
    assert!(
        serde_json::from_str::<crate::aim::Aim>(r#"{"origin":[0,0,0],"direction":[0,0,0]}"#)
            .is_err()
    );
    let mut p = player_fixture();
    let before = p.snapshot();
    let wall = AuthoredSpell::new(AuthoredSource {
        definition: include_str!("../../../spells/fire.spell.yaml").into(),
        brain: None,
    })
    .unwrap();
    assert!(p.select(wall).unwrap_err().contains("body.wall"));
    assert_eq!(p.snapshot(), before);
    p.select(spell()).unwrap();
    let r = player_request(&p, "stale-tick", [0.0, 0.0, 1.0]);
    p.advance().unwrap();
    assert!(p.fire(r).unwrap_err().contains("tick changed"));
    assert_eq!(p.snapshot(), before);
    assert!(p.world().effects().is_empty());
    let mut r = player_request(&p, "wrong-body", [0.0, 0.0, 1.0]);
    r.holder = "aimed-target".into();
    assert!(p.fire(r).is_err());
    assert_eq!(p.snapshot(), before);
}
#[test]
fn player_aimed_miss_depletion_and_explicit_cartridge_replacement() {
    let mut p = player_fixture();
    p.select(spell()).unwrap();
    for id in ["miss-one", "miss-two"] {
        let r = player_request(&p, id, [-1.0, 0.0, 0.0]);
        p.fire(r).unwrap();
    }
    let r = player_request(&p, "empty", [0.0, 0.0, 1.0]);
    assert!(p.fire(r).unwrap_err().contains("cannot fund"));
    for _ in 0..60 {
        p.advance().unwrap();
    }
    assert!(
        !p.world().effects().iter().any(|e| e.kind == "force.push"
            && e.targets
                .iter()
                .any(|t| t == "aimed-target" || t == "nearer-side-target")),
        "aimed misses never pick a configured target automatically"
    );
    p.load(
        &item("wand"),
        &item("b"),
        &at(2),
        Some(&item("a")),
        Some(at(2)),
    )
    .unwrap();
    assert_eq!(
        p.casting()
            .supply()
            .cartridge(&item("a"))
            .unwrap()
            .remaining,
        energy(0.0)
    );
    let mut r = player_request(&p, "replacement-shot", [0.0, 0.0, 1.0]);
    r.expected_cartridge = item("b");
    r.expected_remaining = energy(25.0);
    p.fire(r).unwrap();
    assert_eq!(
        p.casting().supply().total(p.inventory(), &owner()).unwrap(),
        energy(5.0)
    );
    assert_eq!(p.instances().len(), 1);
    assert_eq!(p.finished().len(), 2);
    assert!(p
        .finished()
        .iter()
        .all(|i| i.status == crate::lifecycle::TerminalStatus::Exhausted));
    assert_eq!(p.casting().receipts().len(), 3);
}

#[test]
fn funded_token_retains_aim_independently_of_receipt_copy() {
    let (inv, eq, supply) = fixture();
    let mut casting = Casting::new(supply);
    let spell = spell();
    let request = request(&casting, &spell, "captured-pose", "a");
    let expected = request.aim;
    let mut admission = casting.admit(&inv, &eq, &spell, request).unwrap();
    admission.receipt.request.aim =
        crate::aim::Aim::new([12.0, 1.0, 0.0], [1.0, 0.0, 0.0]).unwrap();
    assert_eq!(admission.funded.unwrap().aim(), expected);
    assert_eq!(casting.receipts()[0].request.aim, expected);
}

fn lifecycle_conserved(t: &crate::lifecycle::TerminalCast) {
    assert!(
        (t.initial - t.unused - t.spent_total()).abs() < 1e-8,
        "terminal allocation is fully accounted: {t:?}"
    );
}
#[test]
fn lifecycle_expiry_retires_runtime_and_owned_resources_without_rewriting_admission() {
    use crate::lifecycle::TerminalStatus;
    let mut p = player_fixture();
    let baseline = p.world().resource_counts();
    p.select(spell()).unwrap();
    let r = player_request(&p, "expires", [-1., 0., 0.]);
    let receipt = p.fire(r.clone()).unwrap();
    let ledger = p.snapshot();
    for _ in 0..359 {
        p.advance().unwrap();
    }
    assert_eq!(
        p.instances().len(),
        1,
        "a still-live owned form keeps the cast active"
    );
    p.advance().unwrap();
    assert!(p.instances().is_empty());
    assert_eq!(p.finished().len(), 1);
    let done = &p.finished()[0];
    assert_eq!(done.status, TerminalStatus::Completed);
    assert_eq!(done.ended_tick, 360);
    assert!(done.unused > 0.);
    lifecycle_conserved(done);
    assert_eq!(
        p.world().resource_counts(),
        baseline,
        "physics, contact and aim storage must be released"
    );
    let terminal = done.clone();
    for _ in 0..200 {
        p.advance().unwrap();
    }
    assert_eq!(p.snapshot(), ledger);
    assert_eq!(p.finished(), &[terminal]);
    assert_eq!(
        p.fire(r).unwrap(),
        receipt,
        "immutable retry returns receipt without restarting terminal work"
    );
    assert!(p.instances().is_empty());
    assert_eq!(p.world().resource_counts(), baseline);
}
fn self_spell(tick: &str, brain: Option<&str>) -> AuthoredSpell {
    AuthoredSpell::new(AuthoredSource { definition: format!(
        "version: 1\nname: lifecycle-self\nkind: spell\nmana: {{source: wand, allocation: 20}}\ncontainer: field\nphases:\n  on_start:\n    - {{id: self, use: body.self, output: self}}\n{tick}"), brain: brain.map(str::to_string) }).unwrap()
}
#[test]
fn lifecycle_one_shot_completes_but_future_sensor_and_self_work_remain_active() {
    use crate::lifecycle::TerminalStatus;
    let mut p = player_fixture();
    let baseline = p.world().resource_counts();
    p.select(self_spell("", None)).unwrap();
    let r = player_request(&p, "one-shot", [0., 0., 1.]);
    p.fire(r).unwrap();
    assert!(p.instances().is_empty());
    assert_eq!(p.finished()[0].status, TerminalStatus::Completed);
    assert_eq!(p.finished()[0].unused, 20.);
    lifecycle_conserved(&p.finished()[0]);
    assert_eq!(
        p.world().resource_counts(),
        baseline,
        "body.self is borrowed, never removed"
    );
    p.select(self_spell("  on_tick:\n    - {id: sensor, use: sense.nearby, target: self, output: seen, every: 120, params: {radius: 0.1}}\n    - {id: warm, use: effect.warm, target: seen, every: 120}\n", None)).unwrap();
    let r = player_request(&p, "continuous", [0., 0., 1.]);
    let receipt = p.fire(r).unwrap();
    for _ in 0..240 {
        p.advance().unwrap();
    }
    assert_eq!(
        p.instances().len(),
        1,
        "empty observations and a waiting cadence are not completion"
    );
    assert_eq!(p.instances()[0].spent["sensors"], 0.36);
    let ledger = p.snapshot();
    let terminal = p
        .cancel(&receipt.instance, &receipt.request.expected_revision)
        .unwrap()
        .clone();
    assert_eq!(terminal.status, TerminalStatus::Cancelled);
    lifecycle_conserved(&terminal);
    assert_eq!(p.world().resource_counts(), baseline);
    assert_eq!(p.snapshot(), ledger);
}
#[test]
fn lifecycle_cancel_is_explicit_revision_checked_and_does_not_undo_effects_or_refund() {
    use crate::lifecycle::TerminalStatus;
    let mut p = player_fixture();
    p.select(
        AuthoredSpell::new(AuthoredSource {
            definition: include_str!("../fixtures/projectile-hit.spell.yaml").into(),
            brain: None,
        })
        .unwrap(),
    )
    .unwrap();
    let r = player_request(&p, "cancel-after-contact", [0., 0., 1.]);
    let receipt = p.fire(r.clone()).unwrap();
    for _ in 0..22 {
        p.advance().unwrap();
    }
    let health = p
        .world()
        .bodies()
        .into_iter()
        .find(|b| b.id == "aimed-target")
        .unwrap()
        .health;
    assert!(health < 10.);
    let ledger = p.snapshot();
    let effects = p.world().effects().to_vec();
    assert!(p.cancel(&receipt.instance, "wrong revision").is_err());
    assert_eq!(p.instances().len(), 1);
    let done = p
        .cancel(&receipt.instance, &receipt.request.expected_revision)
        .unwrap()
        .clone();
    assert_eq!(done.status, TerminalStatus::Cancelled);
    lifecycle_conserved(&done);
    assert!(p.instances().is_empty());
    assert_eq!(
        p.cancel(&receipt.instance, &receipt.request.expected_revision)
            .unwrap(),
        &done
    );
    assert!(p
        .cancel("unknown", &receipt.request.expected_revision)
        .is_err());
    for _ in 0..30 {
        p.advance().unwrap();
    }
    assert_eq!(
        p.world()
            .bodies()
            .into_iter()
            .find(|b| b.id == "aimed-target")
            .unwrap()
            .health,
        health
    );
    assert_eq!(p.world().effects(), effects);
    assert_eq!(p.snapshot(), ledger);
    assert_eq!(p.fire(r).unwrap(), receipt);
    assert!(p.instances().is_empty());
    assert_eq!(p.finished(), &[done]);
}
#[test]
fn lifecycle_exhaustion_keeps_existing_dissipation_and_fault_keeps_unused_allocation() {
    use crate::lifecycle::TerminalStatus;
    let mut p = player_fixture();
    let baseline = p.world().resource_counts();
    let mut src = spell().source().clone();
    src.definition = src.definition.replace("allocation: 20", "allocation: 2.05");
    p.select(AuthoredSpell::new(src).unwrap()).unwrap();
    let r = player_request(&p, "exhaust-on-launch", [0., 0., 1.]);
    p.fire(r).unwrap();
    assert!(p.instances().is_empty());
    let t = &p.finished()[0];
    assert_eq!(t.status, TerminalStatus::Exhausted);
    assert_eq!(t.unused, 0.);
    assert!((t.dissipation - 0.05).abs() < 1e-8);
    lifecycle_conserved(t);
    assert_eq!(p.world().resource_counts(), baseline);
    p.select(self_spell("  on_tick:\n    - {id: see, use: sense.nearby, target: self, output: seen}\n    - {id: fail, use: logic.lua, input: seen}\n",Some("return {decide=function(ctx) error('authored failure') end}"))).unwrap();
    let r = player_request(&p, "fault-after-admission", [0., 0., 1.]);
    p.fire(r).unwrap();
    let ledger = p.snapshot();
    p.advance().unwrap();
    assert!(p.instances().is_empty());
    let t = &p.finished()[1];
    assert_eq!(t.status, TerminalStatus::Faulted);
    assert!(t.reason.contains("authored failure"));
    assert!(t.unused > 0.);
    assert_eq!(t.dissipation, 0.);
    lifecycle_conserved(t);
    assert_eq!(p.snapshot(), ledger);
    assert_eq!(p.world().resource_counts(), baseline);
}
#[test]
fn lifecycle_projection_retains_terminal_receipt_links_and_checked_accounting() {
    let mut p = player_fixture();
    p.select(spell()).unwrap();
    let r = player_request(&p, "typed-terminal", [-1., 0., 0.]);
    let receipt = p.fire(r).unwrap();
    let inv = atelico_inventory::binding::Binding::new("lifecycle.inventory").unwrap();
    let binding = supply_view::Binding::new("lifecycle.supply", inv.paths().clone()).unwrap();
    let store = Store::default();
    binding.install(&store).unwrap();
    let project = |p: &crate::player::Player| {
        binding
            .project_with_history(
                p.casting(),
                p.inventory(),
                p.equipment(),
                p.equipment_slot(),
                p.selected(),
                p.instances(),
                p.finished(),
            )
            .unwrap()
    };
    binding.publish(&store, &project(&p)).unwrap();
    let before = store
        .read::<supply_view::SupplyView>(&binding.dashboard())
        .unwrap();
    let path = before.instances[0].clone();
    p.cancel(&receipt.instance, &receipt.request.expected_revision)
        .unwrap();
    binding.publish(&store, &project(&p)).unwrap();
    let after = store
        .read::<supply_view::SupplyView>(&binding.dashboard())
        .unwrap();
    assert!(after.instances.is_empty());
    assert_eq!(after.finished, vec![path.clone()]);
    assert_eq!(before.total, after.total);
    let row = store.read::<supply_view::CastView>(&path).unwrap();
    assert!(row.terminal);
    assert_eq!(row.remaining, 0.);
    assert!(row.unused > 0.);
    assert!(
        (row.initial
            - row.unused
            - row.sensors
            - row.logic
            - row.rules
            - row.effectors
            - row.dissipation)
            .abs()
            < 1e-8
    );
    let receipt = store
        .read::<supply_view::ReceiptView>(&after.receipts[0])
        .unwrap();
    assert_eq!(
        receipt.instance, path,
        "terminal accounting remains linked from immutable admission evidence"
    );
    assert!(store
        .set(
            &format!("{path}.status"),
            serde_json::json!("invented"),
            atelico_store::Origin::Gui
        )
        .is_err());
}

#[test]
fn lifecycle_expiry_before_a_due_sensor_completes_without_spurious_fault() {
    let mut p = player_fixture();
    let baseline = p.world().resource_counts();
    let spell = AuthoredSpell::new(AuthoredSource {
        definition: "version: 1\nname: expired-sensor\nkind: spell\nshape: orb\nelement: wind\nmana: {source: wand, allocation: 20}\ncontainer: field\nphases:\n  on_start:\n    - {id: form, use: body.orb, output: body}\n  on_tick:\n    - {id: see, use: sense.nearby, target: body, output: seen, every: 120, params: {radius: 0.1}}\n".into(),
        brain: None,
    }).unwrap();
    p.select(spell).unwrap();
    let request = player_request(&p, "expires-on-sample", [-1., 0., 0.]);
    p.fire(request).unwrap();
    let ledger = p.snapshot();
    for _ in 0..359 {
        p.advance().unwrap();
    }
    assert_eq!(p.instances().len(), 1);
    p.advance().unwrap();
    assert!(p.instances().is_empty());
    let terminal = &p.finished()[0];
    assert_eq!(terminal.status, crate::lifecycle::TerminalStatus::Completed);
    assert_eq!(terminal.ended_tick, 360);
    assert_eq!(terminal.sensors, 0.36);
    lifecycle_conserved(terminal);
    assert_eq!(p.snapshot(), ledger);
    assert_eq!(p.world().resource_counts(), baseline);
}
