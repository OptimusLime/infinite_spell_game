use crate::{
    session::Registry,
    view::{Command, PlayerDashboard, PlayerRequest},
};
use atelico_store::{Origin, Store};
use component_spellcraft::supply_view::SupplyView;
use std::{fs, path::PathBuf};
fn fixture() -> (PathBuf, Store, Registry, String) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "spell-player-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::create_dir_all(dir.join("players")).unwrap();
    fs::create_dir_all(dir.join("packages/spellcraft/fixtures")).unwrap();
    fs::write(
        dir.join("players/basic.player.toml"),
        include_str!("../fixtures/basic.player.toml"),
    )
    .unwrap();
    for (name, text) in [
        (
            "projectile-hit.spell.yaml",
            include_str!("../fixtures/projectile-hit.spell.yaml"),
        ),
        (
            "projectile-push.spell.yaml",
            include_str!("../fixtures/projectile-push.spell.yaml"),
        ),
    ] {
        fs::write(dir.join("packages/spellcraft/fixtures").join(name), text).unwrap();
    }
    let store = Store::new();
    let mut registry = Registry::default();
    let (key, _) = registry
        .frame(&store, &dir.join("players/basic.player.toml"), "proof", 0.)
        .unwrap();
    (dir, store, registry, key)
}
fn view(s: &Store, r: &Registry, key: &str) -> PlayerDashboard {
    s.read(&r.sessions[key].dashboard_path()).unwrap()
}
fn supply(s: &Store, r: &Registry, key: &str) -> SupplyView {
    s.read(&view(s, r, key).supply).unwrap()
}
fn command(s: &Store, r: &mut Registry, key: &str, kind: Command, id: &str, now: f64) {
    let v = view(s, r, key);
    let mut q: PlayerRequest = s.read(&v.request).unwrap();
    q.sequence += 1;
    q.command = kind;
    q.item_id = id.into();
    q.spell_id = id.into();
    q.target_id = id.into();
    q.expected_token = v.token;
    s.write(&v.request, &q, Origin::Gui).unwrap();
    r.sessions.get_mut(key).unwrap().frame(s, now).unwrap();
}
#[test]
fn deliberate_equipment_aim_and_fire_debit_finite_cartridges_and_move_real_bodies() {
    let (dir, s, mut r, key) = fixture();
    assert_eq!(supply(&s, &r, &key).total, 80.);
    command(&s, &mut r, &key, Command::Fire, "", 0.);
    assert!(!view(&s, &r, &key).error.is_empty());
    assert_eq!(supply(&s, &r, &key).total, 80.);
    command(&s, &mut r, &key, Command::Equip, "wand", 0.);
    command(&s, &mut r, &key, Command::Load, "amber", 0.);
    command(&s, &mut r, &key, Command::Select, "bolt", 0.);
    command(&s, &mut r, &key, Command::Aim, "front", 0.);
    assert!(
        view(&s, &r, &key).error.is_empty(),
        "{:?}",
        view(&s, &r, &key)
    );
    assert!(r.sessions[&key].player.casting().receipts().is_empty());
    command(&s, &mut r, &key, Command::Fire, "", 0.);
    assert_eq!(supply(&s, &r, &key).total, 60.);
    assert_eq!(r.sessions[&key].player.casting().receipts().len(), 1);
    let first = r.sessions[&key]
        .player
        .world()
        .bodies()
        .into_iter()
        .find(|b| b.shape == "body.bolt")
        .unwrap();
    for n in 1..36 {
        r.sessions
            .get_mut(&key)
            .unwrap()
            .frame(&s, n as f64 / 60.)
            .unwrap();
    }
    let bodies = r.sessions[&key].player.world().bodies();
    assert!(bodies.iter().find(|b| b.id == first.id).unwrap().z > first.z);
    assert!(
        bodies.iter().find(|b| b.id == "front").unwrap().health < 12.,
        "actual physical contact must damage the aimed target: {bodies:?}"
    );
    command(&s, &mut r, &key, Command::Fire, "", 0.6);
    assert_eq!(supply(&s, &r, &key).total, 40.);
    command(&s, &mut r, &key, Command::Fire, "", 0.6);
    assert_eq!(supply(&s, &r, &key).total, 40.);
    assert!(!view(&s, &r, &key).error.is_empty());
    assert_eq!(r.sessions[&key].player.casting().receipts().len(), 2);
    command(&s, &mut r, &key, Command::Load, "blue", 0.6);
    assert!(view(&s, &r, &key).error.is_empty());
    assert_eq!(supply(&s, &r, &key).total, 40.);
    let inv = r.sessions[&key].player.inventory();
    assert!(matches!(
        inv.item(&atelico_inventory::ItemId::new("amber").unwrap())
            .unwrap()
            .location,
        atelico_inventory::Location::Grid { .. }
    ));
    command(&s, &mut r, &key, Command::Select, "orb", 0.6);
    command(&s, &mut r, &key, Command::Aim, "side", 0.6);
    command(&s, &mut r, &key, Command::Fire, "", 0.6);
    assert_eq!(supply(&s, &r, &key).total, 20.);
    assert!(r.sessions[&key]
        .player
        .world()
        .bodies()
        .iter()
        .any(|b| b.shape == "body.orb"));
    command(&s, &mut r, &key, Command::Unload, "", 0.6);
    assert!(supply(&s, &r, &key).loaded.is_empty());
    assert_eq!(supply(&s, &r, &key).total, 20.);
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn aliases_reloads_backward_time_and_invalid_source_edits_never_reset_or_refill() {
    let (dir, s, mut r, key) = fixture();
    for (cmd, id) in [
        (Command::Equip, "wand"),
        (Command::Load, "amber"),
        (Command::Select, "bolt"),
        (Command::Aim, "front"),
        (Command::Fire, ""),
    ] {
        command(&s, &mut r, &key, cmd, id, 2.);
    }
    let before = r.sessions[&key].player.snapshot();
    let tick = r.sessions[&key].player.world().tick();
    let (same, _) = r
        .frame(&s, &dir.join("players/./basic.player.toml"), "proof", 0.)
        .unwrap();
    assert_eq!(same, key);
    assert_eq!(r.sessions.len(), 1);
    assert_eq!(r.sessions[&key].player.snapshot(), before);
    assert_eq!(r.sessions[&key].player.world().tick(), tick);
    let again = view(&s, &r, &key);
    let request: PlayerRequest = s.read(&again.request).unwrap();
    r.frame(&s, &dir.join("players/basic.player.toml"), "proof", 0.)
        .unwrap();
    assert_eq!(
        r.sessions[&key].player.casting().receipts().len(),
        1,
        "repeated frame must not replay Fire"
    );
    let mut stale = request;
    stale.sequence += 1;
    stale.expected_token = 1;
    s.write(&again.request, &stale, Origin::Gui).unwrap();
    r.frame(&s, &dir.join("players/basic.player.toml"), "proof", 0.)
        .unwrap();
    assert_eq!(r.sessions[&key].player.snapshot(), before);
    assert!(view(&s, &r, &key).error.contains("changed"));
    fs::write(dir.join("players/basic.player.toml"), "invalid source [").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(510));
    r.frame(&s, &dir.join("players/basic.player.toml"), "proof", 0.)
        .unwrap();
    assert_eq!(r.sessions[&key].player.snapshot(), before);
    assert!(view(&s, &r, &key).source_changed);
    assert!(view(&s, &r, &key).source_error.contains("invalid"));
    assert!(s
        .set(
            &format!("{}.request.command", key),
            serde_json::json!("auto_cast"),
            Origin::Gui
        )
        .is_err());
    assert!(s
        .set(
            &format!("{}.request.mana", key),
            serde_json::json!(999),
            Origin::Gui
        )
        .is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn caller_key_bindings_reject_duplicates_and_unknown_authored_targets() {
    let (dir, _, _, _) = fixture();
    let path = dir.join("players/basic.player.toml");
    let source = fs::read_to_string(&path).unwrap();
    for (extra, message) in [
        (
            "\n[[keys]]\nkey='W'\ncommand='fire'\n[[keys]]\nkey='w'\ncommand='fire'\n",
            "distinct keys",
        ),
        (
            "\n[[keys]]\nkey='F12'\ncommand='select'\nspell_id='missing'\n",
            "unknown authored spell",
        ),
        (
            "\n[[keys]]\nkey='F12'\ncommand='aim'\ntarget_id='nearest'\n",
            "unknown aim target",
        ),
    ] {
        fs::write(&path, format!("{source}{extra}")).unwrap();
        let error = crate::config::load(&path)
            .err()
            .expect("invalid caller binding rejected");
        assert!(error.contains(message), "{error}");
    }
    fs::remove_dir_all(dir).unwrap();
}

fn revision_request(
    s: &Store,
    r: &mut Registry,
    key: &str,
    operation: crate::revision::RevisionOperation,
    old: &str,
    candidate: &str,
    token: u64,
) {
    let path = r.sessions[key].revision_request_path();
    let prior: crate::revision::PlayerRevisionRequest = s.read(&path).unwrap();
    s.write(
        &path,
        &crate::revision::PlayerRevisionRequest {
            sequence: prior.sequence + 1,
            operation,
            spell_id: "bolt".into(),
            expected_current_revision: old.into(),
            expected_candidate_revision: candidate.into(),
            expected_preview_token: token,
        },
        Origin::Gui,
    )
    .unwrap();
    r.sessions.get_mut(key).unwrap().frame(s, 0.).unwrap();
}
fn review(s: &Store, r: &Registry, key: &str) -> crate::revision::PlayerRevisionView {
    s.read(&r.sessions[key].revision_review_path()).unwrap()
}
#[test]
fn exact_authored_revision_admission_preserves_finite_state_world_and_old_receipts() {
    use crate::revision::RevisionOperation::{Admit, Preview};
    let (dir, s, mut r, key) = fixture();
    for (op, id) in [
        (Command::Equip, "wand"),
        (Command::Load, "amber"),
        (Command::Select, "bolt"),
        (Command::Aim, "front"),
        (Command::Fire, ""),
    ] {
        command(&s, &mut r, &key, op, id, 0.);
    }
    assert_eq!(supply(&s, &r, &key).total, 60.);
    let old = r.sessions[&key]
        .player
        .selected()
        .unwrap()
        .revision()
        .to_string();
    let snapshot = serde_json::to_value(r.sessions[&key].player.snapshot()).unwrap();
    let equipment = serde_json::to_value(r.sessions[&key].player.equipment()).unwrap();
    let bodies = serde_json::to_value(r.sessions[&key].player.world().bodies()).unwrap();
    let source = dir.join("packages/spellcraft/fixtures/projectile-hit.spell.yaml");
    let original = fs::read_to_string(&source).unwrap();
    let changed = original
        .replace(
            "title: Engineering bolt",
            "title: Reviewed engineering bolt",
        )
        .replace("allocation: 20", "allocation: 10");
    fs::write(&source, &changed).unwrap();
    revision_request(&s, &mut r, &key, Preview, &old, "", 0);
    let shown = review(&s, &r, &key);
    assert!(shown.valid && shown.changed, "{shown:?}");
    assert_eq!(
        (shown.current_allocation, shown.candidate_allocation),
        (20., 10.)
    );
    assert_eq!(
        serde_json::to_value(r.sessions[&key].player.snapshot()).unwrap(),
        snapshot,
        "preview cannot alter finite state"
    );
    // Exact source changed between preview and admission: no mutation, including existing physical bodies.
    fs::write(&source, format!("{changed}\n# changed after review\n")).unwrap();
    revision_request(
        &s,
        &mut r,
        &key,
        Admit,
        &old,
        &shown.candidate_revision,
        shown.preview_token,
    );
    assert!(review(&s, &r, &key).error.contains("changed after preview"));
    assert_eq!(
        serde_json::to_value(r.sessions[&key].player.snapshot()).unwrap(),
        snapshot
    );
    assert_eq!(
        serde_json::to_value(r.sessions[&key].player.world().bodies()).unwrap(),
        bodies
    );
    revision_request(&s, &mut r, &key, Preview, &old, "", 0);
    let shown = review(&s, &r, &key);
    revision_request(
        &s,
        &mut r,
        &key,
        Admit,
        &old,
        &shown.candidate_revision,
        shown.preview_token + 1,
    );
    assert!(review(&s, &r, &key).error.contains("stale"));
    revision_request(&s, &mut r, &key, Preview, &old, "", 0);
    let shown = review(&s, &r, &key);
    revision_request(
        &s,
        &mut r,
        &key,
        Admit,
        &old,
        &shown.candidate_revision,
        shown.preview_token,
    );
    assert!(review(&s, &r, &key).error.is_empty());
    assert_eq!(
        r.sessions[&key].player.selected().unwrap().revision(),
        shown.candidate_revision
    );
    assert_eq!(
        serde_json::to_value(r.sessions[&key].player.snapshot()).unwrap(),
        snapshot,
        "admission cannot change Inventory, balances or old receipts"
    );
    assert_eq!(
        serde_json::to_value(r.sessions[&key].player.equipment()).unwrap(),
        equipment
    );
    assert_eq!(
        serde_json::to_value(r.sessions[&key].player.world().bodies()).unwrap(),
        bodies
    );
    assert_eq!(r.sessions[&key].player.world().tick(), 0);
    assert_eq!(r.sessions[&key].player.instances().len(), 1);
    r.sessions.get_mut(&key).unwrap().frame(&s, 0.).unwrap();
    assert_eq!(
        serde_json::to_value(r.sessions[&key].player.snapshot()).unwrap(),
        snapshot,
        "same request sequence cannot replay"
    );
    command(&s, &mut r, &key, Command::Fire, "", 0.);
    assert_eq!(
        supply(&s, &r, &key).total,
        50.,
        "future cast uses new10 allocation against existing20 remaining"
    );
    let receipts = r.sessions[&key].player.casting().receipts();
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0].request.expected_revision, old);
    assert_eq!(
        receipts[1].request.expected_revision,
        shown.candidate_revision
    );
    let after = serde_json::to_value(r.sessions[&key].player.snapshot()).unwrap();
    revision_request(
        &s,
        &mut r,
        &key,
        Admit,
        &old,
        &shown.candidate_revision,
        shown.preview_token,
    );
    assert!(review(&s, &r, &key)
        .error
        .contains("active authored revision changed"));
    fs::write(&source, "not valid: [").unwrap();
    revision_request(&s, &mut r, &key, Preview, &shown.candidate_revision, "", 0);
    assert!(!review(&s, &r, &key).valid);
    assert!(!review(&s, &r, &key).error.is_empty());
    assert_eq!(
        serde_json::to_value(r.sessions[&key].player.snapshot()).unwrap(),
        after
    );
    fs::remove_dir_all(dir).unwrap();
}
