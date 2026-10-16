use crate::{capacity::*, funding::Account, supply::Energy, supply_tests::fixture};
use atelico_inventory::*;
fn item(s: &str) -> ItemId {
    ItemId::new(s).unwrap()
}
fn grid() -> GridId {
    GridId::new("pack").unwrap()
}
fn energy(v: f64) -> Energy {
    Energy::new(v).unwrap()
}
fn location(x: u16, y: u16) -> Location {
    Location::Grid {
        grid: grid(),
        column: x,
        row: y,
        orientation: Orientation::Normal,
    }
}
fn fixture_capacity(renewal: f64) -> (Inventory, Account, Capacity) {
    let (mut inv, _, supply) = fixture();
    inv.register_at(
        ItemInstance {
            id: item("expander"),
            definition: DefinitionId::new("fixture-expander").unwrap(),
            footprint: Footprint {
                size: Size::new(1, 1).unwrap(),
                rotatable: false,
            },
        },
        location(3, 0),
    )
    .unwrap();
    let capacity = Capacity::new(
        &inv,
        vec![Spec {
            item: item("expander"),
            grid: grid(),
            baseline: Size::new(5, 2).unwrap(),
            expanded: Size::new(7, 3).unwrap(),
            activation_cost: energy(3.),
            renewal_cost: energy(renewal),
            cadence: 10,
        }],
    )
    .unwrap();
    (inv, Account::new(supply), capacity)
}
fn request(
    inv: &Inventory,
    account: &Account,
    capacity: &Capacity,
    id: &str,
    tick: u64,
) -> Request {
    let spec = capacity.spec(&item("expander")).unwrap();
    let grant = capacity.grant(&spec.item);
    Request {
        id: id.into(),
        operation: if grant.is_some() {
            Operation::Renew
        } else {
            Operation::Activate
        },
        item: spec.item.clone(),
        owner: EntityId::new("fixture-actor").unwrap(),
        expected_config: spec.revision(),
        expected_location: inv.item(&spec.item).unwrap().location.clone(),
        expected_size: inv.grid(&spec.grid).unwrap().size,
        expected_revision: grant.map(|g| g.revision).unwrap_or(0),
        expected_paid_until: grant.map(|g| g.paid_until).unwrap_or(0),
        source: item("a"),
        expected_remaining: account.supply().cartridge(&item("a")).unwrap().remaining,
        tick,
    }
}
fn state(inv: &Inventory, account: &Account, capacity: &Capacity) -> CandidateSnapshot {
    CandidateSnapshot {
        inventory: inv.snapshot(),
        funding: account.snapshot(),
        capacity: capacity.snapshot(),
    }
}
#[test]
fn activation_and_renewal_are_atomic_exact_and_replay_safe() {
    let (mut inv, mut account, mut capacity) = fixture_capacity(4.);
    let r = request(&inv, &account, &capacity, "activate", 4);
    let before = state(&inv, &account, &capacity);
    let prepared = capacity.prepare(&inv, &account, r.clone()).unwrap();
    assert_eq!(state(&inv, &account, &capacity), before);
    let durable = capacity
        .candidate_snapshot(&inv, &account, &prepared)
        .unwrap();
    Capacity::restore_candidate(durable.clone()).unwrap();
    let outcome = capacity.commit(&mut inv, &mut account, prepared).unwrap();
    assert!(!outcome.replayed);
    assert_eq!(state(&inv, &account, &capacity), durable);
    assert_eq!(inv.grid(&grid()).unwrap().size, Size::new(7, 3).unwrap());
    assert_eq!(
        account.supply().cartridge(&item("a")).unwrap().remaining,
        energy(37.)
    );
    inv.transfer(&item("b"), &location(2, 0), location(6, 2))
        .unwrap();
    let replay = capacity.prepare(&inv, &account, r.clone()).unwrap();
    inv.transfer(&item("b"), &location(6, 2), location(5, 2))
        .unwrap();
    let replay_export = capacity
        .candidate_snapshot(&inv, &account, &replay)
        .unwrap();
    assert_eq!(replay_export.inventory, inv.snapshot());
    Capacity::restore_candidate(replay_export).unwrap();
    capacity
        .commit_with(&mut inv, &mut account, replay, |_| {
            panic!("a replay must not publish an old grid")
        })
        .unwrap();
    assert_eq!(inv.item(&item("b")).unwrap().location, location(5, 2));
    let mut changed = r;
    changed.tick = 5;
    assert!(capacity.prepare(&inv, &account, changed).is_err());
    let early = request(&inv, &account, &capacity, "early", 13);
    assert!(capacity.prepare(&inv, &account, early).is_err());
    let renewal = request(&inv, &account, &capacity, "renew", 14);
    let prepared = capacity.prepare(&inv, &account, renewal.clone()).unwrap();
    assert!(prepared.changes().is_empty());
    capacity.commit(&mut inv, &mut account, prepared).unwrap();
    assert_eq!(
        account.supply().cartridge(&item("a")).unwrap().remaining,
        energy(33.)
    );
    assert_eq!(capacity.grant(&item("expander")).unwrap().paid_until, 24);
    let again = capacity.prepare(&inv, &account, renewal).unwrap();
    capacity.commit(&mut inv, &mut account, again).unwrap();
    let saved = state(&inv, &account, &capacity);
    Capacity::restore_candidate(saved.clone()).unwrap();
    let late = request(&inv, &account, &capacity, "late", 25);
    assert!(matches!(
        capacity.prepare(&inv, &account, late),
        Err(crate::capacity::Error::PolicyRequired(_))
    ));
    assert_eq!(state(&inv, &account, &capacity), saved);
}
#[test]
fn unpaid_occupied_extension_requires_a_policy_without_collapsing_or_refilling() {
    let (mut inv, mut account, mut capacity) = fixture_capacity(40.);
    let p = capacity
        .prepare(
            &inv,
            &account,
            request(&inv, &account, &capacity, "activate", 4),
        )
        .unwrap();
    capacity.commit(&mut inv, &mut account, p).unwrap();
    inv.transfer(&item("b"), &location(2, 0), location(6, 2))
        .unwrap();
    let before = state(&inv, &account, &capacity);
    assert!(matches!(
        capacity.status(&inv, &item("expander"), 13).unwrap(),
        Some(Status::Paid(_))
    ));
    assert!(capacity.status(&inv, &item("expander"), 3).is_err());
    let r = request(&inv, &account, &capacity, "unfunded", 14);
    let Err(crate::capacity::Error::Funding {
        policy: Some(policy),
        ..
    }) = capacity.prepare(&inv, &account, r)
    else {
        panic!("explicit missing policy expected")
    };
    assert_eq!(policy.overflow_items, vec![item("b")]);
    for tick in [14, 15, 10000] {
        assert!(matches!(
            capacity.status(&inv, &item("expander"), tick).unwrap(),
            Some(Status::PolicyRequired(_))
        ));
    }
    assert_eq!(state(&inv, &account, &capacity), before);
}
#[test]
fn stale_inventory_owner_dimensions_and_configuration_cannot_spend() {
    let (mut inv, mut account, mut capacity) = fixture_capacity(4.);
    let original = state(&inv, &account, &capacity);
    let r = request(&inv, &account, &capacity, "one", 4);
    let mut wrong = r.clone();
    wrong.expected_size = Size::new(4, 2).unwrap();
    assert!(capacity.prepare(&inv, &account, wrong).is_err());
    let mut wrong = r.clone();
    wrong.owner = EntityId::new("foreign").unwrap();
    assert!(capacity.prepare(&inv, &account, wrong).is_err());
    let mut wrong = r.clone();
    wrong.expected_config = "changed".into();
    assert!(capacity.prepare(&inv, &account, wrong).is_err());
    assert_eq!(state(&inv, &account, &capacity), original);
    let p = capacity.prepare(&inv, &account, r).unwrap();
    inv.transfer(&item("b"), &location(2, 0), location(2, 1))
        .unwrap();
    let moved = inv.snapshot();
    assert!(capacity.commit(&mut inv, &mut account, p).is_err());
    assert_eq!(inv.snapshot(), moved);
    assert_eq!(account.snapshot(), original.funding);
    assert_eq!(capacity.snapshot(), original.capacity);
    let p = capacity
        .prepare(
            &inv,
            &account,
            request(&inv, &account, &capacity, "other-state", 4),
        )
        .unwrap();
    let mut alternative = capacity.spec(&item("expander")).unwrap().clone();
    alternative.activation_cost = energy(5.);
    let mut other = Capacity::new(&inv, vec![alternative]).unwrap();
    assert!(other.commit(&mut inv, &mut account, p).is_err());
    assert_eq!(account.snapshot(), original.funding);
}
#[test]
fn capacity_restoration_rejects_missing_or_forged_grants_and_payment_links() {
    let (mut inv, mut account, mut capacity) = fixture_capacity(4.);
    let p = capacity
        .prepare(&inv, &account, request(&inv, &account, &capacity, "one", 4))
        .unwrap();
    capacity.commit(&mut inv, &mut account, p).unwrap();
    let good = state(&inv, &account, &capacity);
    for case in 0..5 {
        let mut bad = good.clone();
        match case {
            0 => bad.capacity.grants.clear(),
            1 => bad.capacity.receipts[0].grant.paid_until += 1,
            2 => bad.capacity.specs[0].activation_cost = energy(2.),
            3 => bad.capacity.receipts.push(bad.capacity.receipts[0].clone()),
            _ => bad.funding.journal.debits[0].request.payload_revision = "ff".repeat(32),
        };
        assert!(Capacity::restore_candidate(bad).is_err());
    }
    let mut missing = good;
    missing
        .inventory
        .items
        .retain(|i| i.item.id != item("expander"));
    assert!(Capacity::restore_candidate(missing).is_err());
}
#[test]
fn rejected_store_batch_keeps_grid_funding_and_grant_unchanged() {
    use atelico_data::Data;
    use atelico_store::{Op, Origin, Store};
    use serde::{Deserialize, Serialize};
    #[derive(Data, Serialize, Deserialize, Default)]
    struct Projection {
        remaining: f64,
        revision: u64,
    }
    let (mut inv, mut account, mut capacity) = fixture_capacity(4.);
    let store = Store::new();
    let binding = atelico_inventory::binding::Binding::new("fixture.inventory").unwrap();
    binding.install(&store).unwrap();
    binding.publish(&store, &inv).unwrap();
    let schema = Projection::schema();
    let name = schema.name.clone();
    assert!(store.define(schema).is_empty());
    assert!(store.bind("fixture.capacity", &name).is_empty());
    store
        .set(
            "fixture.capacity",
            serde_json::to_value(Projection {
                remaining: 40.,
                revision: 0,
            })
            .unwrap(),
            Origin::Host,
        )
        .unwrap();
    let reject = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let flag = reject.clone();
    store.rule("fixture.capacity.revision", move |v| {
        if flag.load(std::sync::atomic::Ordering::SeqCst) {
            Err("fixture publication rejected".into())
        } else {
            Ok(v.clone())
        }
    });
    let publish = |p: &Prepared| {
        let mut ops = binding.operations(&store, p.inventory(), p.changes())?;
        ops.push(Op::Set(
            "fixture.capacity.remaining".into(),
            serde_json::json!(p.debit().after.get()),
        ));
        ops.push(Op::Set(
            "fixture.capacity.revision".into(),
            serde_json::json!(p.receipt().grant.revision),
        ));
        store.batch(ops, Origin::Host).map(|_| ())
    };
    let r = request(&inv, &account, &capacity, "one", 4);
    let before = state(&inv, &account, &capacity);
    let revision = store.rev();
    let p = capacity.prepare(&inv, &account, r.clone()).unwrap();
    assert!(matches!(
        capacity.commit_with(&mut inv, &mut account, p, publish),
        Err(crate::capacity::Error::Publication(_))
    ));
    assert_eq!(state(&inv, &account, &capacity), before);
    assert_eq!(store.rev(), revision);
    reject.store(false, std::sync::atomic::Ordering::SeqCst);
    let p = capacity.prepare(&inv, &account, r).unwrap();
    capacity
        .commit_with(&mut inv, &mut account, p, publish)
        .unwrap();
    let projected: Projection = store.read("fixture.capacity").unwrap();
    assert_eq!(projected.remaining, 37.);
    assert_eq!(projected.revision, 1);
    let grid: atelico_inventory::view::GridView =
        store.read(&binding.paths().grid(&grid())).unwrap();
    assert_eq!(grid.width, 7);
}
#[test]
fn capacity_candidate_preserves_cast_authored_history_in_the_callers_aggregate() {
    use crate::{
        casting::{AuthoredSource, AuthoredSpell, Casting},
        supply_tests::{request as cast_request, spell},
    };
    let (mut inv, account, mut capacity) = fixture_capacity(4.);
    let (_, eq, _) = fixture();
    let mut casts =
        Casting::new(crate::supply::Supply::restore(&inv, account.snapshot().supply).unwrap());
    let source = spell();
    let authored = AuthoredSpell::new(AuthoredSource {
        definition: source
            .source()
            .definition
            .replace("allocation: 20", "allocation: 8"),
        brain: None,
    })
    .unwrap();
    let original = cast_request(&casts, &authored, "cast-before", "a");
    casts.admit(&inv, &eq, &authored, original.clone()).unwrap();
    let p = capacity
        .prepare(
            &inv,
            casts.funding(),
            request(&inv, casts.funding(), &capacity, "grow", 4),
        )
        .unwrap();
    let durable = capacity
        .candidate_snapshot(&inv, casts.funding(), &p)
        .unwrap();
    let candidate_inv = Inventory::try_from(durable.inventory.clone()).unwrap();
    let cast_snapshot = casts
        .candidate_snapshot(&candidate_inv, durable.funding.clone())
        .unwrap();
    capacity.commit(&mut inv, casts.funding_mut(), p).unwrap();
    assert_eq!(casts.snapshot(), cast_snapshot);
    let mut restored = Casting::restore(&candidate_inv, cast_snapshot).unwrap();
    Capacity::restore(&candidate_inv, restored.funding(), durable.capacity).unwrap();
    assert!(restored
        .admit(&candidate_inv, &eq, &authored, original)
        .unwrap()
        .funded
        .is_none());
    let next = cast_request(&restored, &authored, "cast-after", "a");
    restored
        .admit(&candidate_inv, &eq, &authored, next)
        .unwrap();
    assert_eq!(
        restored.supply().cartridge(&item("a")).unwrap().remaining,
        energy(21.)
    );
}
