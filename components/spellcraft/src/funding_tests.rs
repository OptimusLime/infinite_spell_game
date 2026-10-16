use crate::{
    casting::{AuthoredSource, AuthoredSpell, Casting, CastingSnapshot},
    funding::*,
    supply::Energy,
    supply_tests::{fixture, request, spell},
};
use atelico_inventory::{EntityId, ItemId};
fn energy(v: f64) -> Energy {
    Energy::new(v).unwrap()
}
fn item(v: &str) -> ItemId {
    ItemId::new(v).unwrap()
}
fn debit(account: &Account, id: &str, amount: f64) -> DebitRequest {
    DebitRequest {
        key: DebitKey {
            purpose: DebitPurpose::CapacityActivation,
            id: id.into(),
        },
        owner: EntityId::new("fixture-actor").unwrap(),
        cartridge: item("a"),
        expected_remaining: account.supply().cartridge(&item("a")).unwrap().remaining,
        amount: energy(amount),
        payload_revision: "ab".repeat(32),
    }
}
#[test]
fn funding_prepares_without_mutation_and_commits_once_with_exact_replay() {
    let (inv, _, supply) = fixture();
    let mut a = Account::new(supply);
    let before = a.snapshot();
    let r = debit(&a, "one", 3.);
    let plan = a.prepare(&inv, r.clone()).unwrap();
    assert_eq!(a.snapshot(), before);
    let candidate = a.candidate_snapshot(&inv, &plan).unwrap();
    assert_eq!(candidate.supply.cartridges[0].remaining, energy(37.));
    Account::restore(&inv, candidate.clone()).unwrap();
    let paid = a.commit(&inv, plan).unwrap();
    assert!(!paid.replayed);
    assert_eq!(a.snapshot(), candidate);
    let replay = a.prepare(&inv, r.clone()).unwrap();
    assert!(a.commit(&inv, replay).unwrap().replayed);
    assert_eq!(a.snapshot(), candidate);
    let mut changed = r;
    changed.payload_revision = "cd".repeat(32);
    assert!(a.prepare(&inv, changed).is_err());
    assert_eq!(a.snapshot(), candidate);
}
#[test]
fn funding_checks_owner_affordability_and_stale_plans_before_any_debit() {
    let (inv, _, supply) = fixture();
    let mut a = Account::new(supply);
    let original = a.snapshot();
    let mut foreign = debit(&a, "foreign", 1.);
    foreign.owner = EntityId::new("another").unwrap();
    assert!(a.prepare(&inv, foreign).is_err());
    assert!(a.prepare(&inv, debit(&a, "too-much", 41.)).is_err());
    assert_eq!(a.snapshot(), original);
    let stale = a.prepare(&inv, debit(&a, "stale", 1.)).unwrap();
    let newer = a.prepare(&inv, debit(&a, "new", 2.)).unwrap();
    a.commit(&inv, newer).unwrap();
    let paid = a.snapshot();
    assert!(a.commit(&inv, stale).is_err());
    assert_eq!(a.snapshot(), paid);
    for case in 0..4 {
        let mut bad = paid.clone();
        match case {
            0 => bad.journal.debits.push(bad.journal.debits[0].clone()),
            1 => bad.journal.debits[0].after = energy(10.),
            2 => bad.supply.cartridges[0].remaining = energy(39.),
            _ => bad.journal.version = 99,
        };
        assert!(Account::restore(&inv, bad).is_err());
    }
}
#[test]
fn cast_capacity_cast_history_restores_against_one_balance_and_exact_authored_sources() {
    let (inv, eq, supply) = fixture();
    let mut casts = Casting::new(supply);
    let original = spell();
    let authored = AuthoredSpell::new(AuthoredSource {
        definition: original
            .source()
            .definition
            .replace("allocation: 20", "allocation: 8"),
        brain: None,
    })
    .unwrap();
    let first = request(&casts, &authored, "first", "a");
    casts.admit(&inv, &eq, &authored, first.clone()).unwrap();
    let expansion = debit(casts.funding(), "expansion", 2.);
    let plan = casts.funding().prepare(&inv, expansion.clone()).unwrap();
    casts.funding_mut().commit(&inv, plan).unwrap();
    let last = request(&casts, &authored, "last", "a");
    casts.admit(&inv, &eq, &authored, last).unwrap();
    assert_eq!(
        casts.supply().cartridge(&item("a")).unwrap().remaining,
        energy(22.)
    );
    let snapshot = casts.snapshot();
    assert_eq!(snapshot.funding.as_ref().unwrap().debits.len(), 3);
    let mut restored = Casting::restore(
        &inv,
        serde_json::from_slice(&serde_json::to_vec(&snapshot).unwrap()).unwrap(),
    )
    .unwrap();
    assert!(restored
        .admit(&inv, &eq, &authored, first)
        .unwrap()
        .funded
        .is_none());
    let replay = restored.funding().prepare(&inv, expansion).unwrap();
    assert!(
        restored
            .funding_mut()
            .commit(&inv, replay)
            .unwrap()
            .replayed
    );
    assert_eq!(restored.snapshot(), snapshot);
    let mut bad = snapshot.clone();
    bad.receipts[0].source.definition = bad.receipts[0]
        .source
        .definition
        .replace("title: Wind", "title: Changed");
    assert!(Casting::restore(&inv, bad).is_err());
    let mut bad = snapshot;
    bad.funding.as_mut().unwrap().debits[0]
        .request
        .payload_revision = "ff".repeat(32);
    assert!(Casting::restore(&inv, bad).is_err());
}
#[test]
fn legacy_cast_only_snapshots_migrate_but_new_missing_links_are_rejected() {
    let (inv, eq, supply) = fixture();
    let mut casts = Casting::new(supply);
    let s = spell();
    let r = request(&casts, &s, "old", "a");
    casts.admit(&inv, &eq, &s, r.clone()).unwrap();
    let mut old = casts.snapshot();
    old.funding = None;
    let serialized = serde_json::to_string(&old).unwrap();
    assert!(!serialized.contains("funding"));
    let mut restored = Casting::restore(
        &inv,
        serde_json::from_str::<CastingSnapshot>(&serialized).unwrap(),
    )
    .unwrap();
    assert!(restored.admit(&inv, &eq, &s, r).unwrap().funded.is_none());
    assert_eq!(restored.snapshot(), casts.snapshot());
    let mut broken = casts.snapshot();
    broken.funding = Some(JournalSnapshot::default());
    assert!(Casting::restore(&inv, broken).is_err());
}

#[test]
fn positive_debits_must_reduce_the_represented_balance_including_on_restore() {
    let (inv, _, supply) = fixture();
    let mut snapshot = supply.snapshot();
    let source = snapshot
        .cartridges
        .iter_mut()
        .find(|c| c.item == item("a"))
        .unwrap();
    source.capacity = energy(1e300);
    source.remaining = energy(1e300);
    let mut account = Account::new(crate::supply::Supply::restore(&inv, snapshot).unwrap());
    let original = account.snapshot();
    let tiny = debit(&account, "tiny-positive", 1.);
    assert_eq!(
        tiny.expected_remaining.get() - tiny.amount.get(),
        tiny.expected_remaining.get()
    );
    assert!(account.prepare(&inv, tiny.clone()).is_err());
    assert_eq!(account.snapshot(), original);

    let mut forged = original.clone();
    forged.journal.debits.push(DebitReceipt {
        before: tiny.expected_remaining,
        after: tiny.expected_remaining,
        request: tiny,
    });
    assert!(Account::restore(&inv, forged).is_err());

    let zero = account
        .prepare(&inv, debit(&account, "explicit-zero", 0.))
        .unwrap();
    let paid = account.commit(&inv, zero).unwrap();
    assert_eq!(paid.receipt.before, paid.receipt.after);
    Account::restore(&inv, account.snapshot()).unwrap();
    let represented = account
        .prepare(&inv, debit(&account, "represented-positive", 1e299))
        .unwrap();
    let paid = account.commit(&inv, represented).unwrap();
    assert!(paid.receipt.after < paid.receipt.before);
    Account::restore(&inv, account.snapshot()).unwrap();
}
