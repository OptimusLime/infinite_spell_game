use super::authoring::*;
use super::definition::{self, Kind, Source};
use std::{fs, path::PathBuf};
const SOURCE: &str = r#"version: 1
name: fixture
title: Engineering fixture
kind: spell
mana: {source: wand, allocation: 16}
container: field
phases:
  on_start:
    - {id: form, use: body.bolt, output: bolt}
    - {id: launch, use: motion.launch, target: bolt, params: {speed: 12}}
  on_contact:
    target: bolt
    every: 1
    steps:
      - {id: hit, use: effect.damage, target: event.other, params: {amount: 2}}
"#;
struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "spell-authoring-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn source(&self, name: &str) -> PathBuf {
        let p = self.0.join(format!("{name}.spell.yaml"));
        fs::write(&p, SOURCE).unwrap();
        p
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn title(d: &mut Draft, text: &str) {
    let old = d.definition();
    let m = Metadata {
        name: old.name.clone(),
        title: text.into(),
        kind: old.kind,
        shape: old.shape.clone().unwrap_or_default(),
        element: old.element.clone().unwrap_or_default(),
        source: old.mana.source,
        container: old.container.clone(),
        allocation: old.mana.allocation,
    };
    d.apply(d.generation(), Command::Metadata(m)).unwrap();
}
#[test]
fn a_blank_draft_is_created_explicitly_and_saved_only_after_validation() {
    let files = Files::new();
    let path = files.0.join("created.spell.yaml");
    let mut d = Draft::new();
    assert!(d.validation().is_err());
    assert!(d.save_as(d.generation(), &path).is_err());
    assert!(!path.exists());
    d.apply(
        d.generation(),
        Command::Metadata(Metadata {
            name: "authored".into(),
            title: "Authored fixture".into(),
            kind: Kind::Spell,
            shape: String::new(),
            element: String::new(),
            source: Source::Wand,
            container: "seed".into(),
            allocation: 8.,
        }),
    )
    .unwrap();
    d.apply(
        d.generation(),
        Command::Add {
            phase: Phase::Start,
            id: "form".into(),
            component: "body.bolt".into(),
        },
    )
    .unwrap();
    assert!(
        d.validation().is_err(),
        "the required output is still missing"
    );
    d.apply(
        d.generation(),
        Command::Ports {
            id: "form".into(),
            ports: Ports {
                target: String::new(),
                input: String::new(),
                output: "body".into(),
                every: None,
            },
        },
    )
    .unwrap();
    d.apply(
        d.generation(),
        Command::Add {
            phase: Phase::Start,
            id: "launch".into(),
            component: "motion.launch".into(),
        },
    )
    .unwrap();
    d.apply(
        d.generation(),
        Command::Ports {
            id: "launch".into(),
            ports: Ports {
                target: "body".into(),
                input: String::new(),
                output: String::new(),
                every: None,
            },
        },
    )
    .unwrap();
    d.apply(
        d.generation(),
        Command::Parameter {
            id: "launch".into(),
            key: "speed".into(),
            value: Some(ParameterValue::Number(9.)),
        },
    )
    .unwrap();
    d.save_as(d.generation(), &path).unwrap();
    assert!(!d.dirty());
    assert!(!d.new_file());
    assert_eq!(
        definition::parse(&fs::read_to_string(&path).unwrap()).unwrap(),
        *d.definition()
    );
    assert_eq!(
        definition::number(&d.definition().phases.on_start[1], "speed", 0.),
        9.
    );
    assert!(
        d.save_as(d.generation(), &path).is_err(),
        "Save-as never replaces an existing file"
    );
}
#[test]
fn invalid_parameters_and_stale_commands_leave_the_draft_unchanged() {
    let files = Files::new();
    let path = files.source("params");
    let mut d = Draft::load(&path).unwrap();
    let before = d.definition().clone();
    let generation = d.generation();
    for value in [
        ParameterValue::Number(f64::NAN),
        ParameterValue::Number(f64::INFINITY),
        ParameterValue::Number(-1.),
        ParameterValue::Number(1000.),
        ParameterValue::Text("fast".into()),
    ] {
        assert!(d
            .apply(
                generation,
                Command::Parameter {
                    id: "launch".into(),
                    key: "speed".into(),
                    value: Some(value)
                }
            )
            .is_err());
        assert_eq!(d.definition(), &before);
        assert_eq!(d.generation(), generation);
    }
    assert!(d
        .apply(
            generation,
            Command::Add {
                phase: Phase::Contact,
                id: "form".into(),
                component: "body.bolt".into()
            }
        )
        .is_err());
    title(&mut d, "Changed");
    let changed = d.definition().clone();
    assert!(d
        .apply(generation, Command::Remove { id: "hit".into() })
        .is_err());
    assert_eq!(d.definition(), &changed);
    assert!(d
        .apply(
            d.generation(),
            Command::Add {
                phase: Phase::Contact,
                id: "new_form".into(),
                component: "body.bolt".into()
            }
        )
        .is_err());
    d.replace(d.generation(), Draft::new()).unwrap();
    assert!(d.generation() > generation);
    assert!(
        d.apply(generation, Command::Remove { id: "form".into() })
            .is_err(),
        "file/new replacement invalidates old generations"
    );
}
#[test]
fn order_and_port_errors_remain_editable_but_cannot_overwrite_saved_source() {
    let files = Files::new();
    let path = files.source("order");
    let mut d = Draft::load(&path).unwrap();
    d.apply(
        d.generation(),
        Command::Move {
            id: "launch".into(),
            direction: Direction::Up,
        },
    )
    .unwrap();
    assert!(d.validation().unwrap_err().contains("forward reference"));
    assert!(d.save(d.generation()).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
    d.apply(
        d.generation(),
        Command::Move {
            id: "launch".into(),
            direction: Direction::Down,
        },
    )
    .unwrap();
    assert!(d.validation().is_ok());
    d.apply(d.generation(), Command::Remove { id: "form".into() })
        .unwrap();
    assert!(d.validation().is_err());
    d.reload(d.generation()).unwrap();
    assert!(!d.dirty());
    assert_eq!(d.definition().phases.on_start.len(), 2);
}
#[test]
fn competing_drafts_and_external_changes_are_detected_without_losing_the_draft() {
    let files = Files::new();
    let path = files.source("shared");
    let mut a = Draft::load(&path).unwrap();
    let mut b = Draft::load(&path).unwrap();
    title(&mut a, "First writer");
    title(&mut b, "Other writer");
    a.save(a.generation()).unwrap();
    let saved = fs::read_to_string(&path).unwrap();
    assert!(b.conflicted());
    assert!(b.save(b.generation()).unwrap_err().contains("changed"));
    assert_eq!(fs::read_to_string(&path).unwrap(), saved);
    assert_eq!(b.definition().title(), "Other writer");
    assert!(b.dirty());
    let fork = files.0.join("fork.spell.yaml");
    b.save_as(b.generation(), &fork).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), saved);
    title(&mut a, "Unsaved draft");
    fs::write(&path, "not valid YAML: [").unwrap();
    assert!(a.reload(a.generation()).is_err());
    assert_eq!(a.definition().title(), "Unsaved draft");
    assert!(a.save(a.generation()).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "not valid YAML: [");
}
#[test]
fn projection_is_catalog_typed_and_keeps_definition_validity_separate_from_player_support() {
    use super::authoring_view::*;
    let files = Files::new();
    let mut d = Draft::load(&files.source("projection")).unwrap();
    d.apply(
        d.generation(),
        Command::Parameter {
            id: "launch".into(),
            key: "speed".into(),
            value: None,
        },
    )
    .unwrap();
    let ui = SpellAuthoringUi {
        selected_step: "launch".into(),
        selected_parameter: "speed".into(),
        ..Default::default()
    };
    let p = Projection::build(Some(&d), &ui, "", "");
    assert!(p.dashboard.valid);
    let step = &p.steps[&p.dashboard.selected_step];
    assert_eq!(step.id, "launch");
    assert_eq!(step.target_type, "body");
    let param = &p.parameters[&p.dashboard.selected_parameter];
    assert_eq!(param.kind, ParameterKind::Number);
    assert!(!param.has_value);
    assert_eq!(param.number, 10.);
    assert_eq!((param.min, param.max), (0., 40.));
    assert!(p.dashboard.components.iter().any(|r|p.components[r].id=="body.wall"),"generic authoring exposes catalog members even when this Player adapter does not support them");
    assert!(p
        .steps
        .keys()
        .all(|p| !p.rsplit('.').next().unwrap().contains('-')));
}
