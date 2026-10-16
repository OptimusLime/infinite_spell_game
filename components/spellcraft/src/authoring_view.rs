//! Flat, normalized native authoring records. Runtime definitions and catalog JSON never enter Luau.
use crate::{
    authoring::{Direction, Draft, Phase},
    definition,
};
use atelico_data::{Data, DataKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ROOT: &str = "runtime.spell_authoring";
pub const DASHBOARD: &str = "runtime.spell_authoring.dashboard";
pub const REQUEST: &str = "runtime.spell_authoring.request";
pub const UI: &str = "spell_authoring_ui";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, DataKind)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    #[default]
    New,
    Load,
    Reload,
    Save,
    SaveAs,
    SetMetadata,
    AddStep,
    SetStep,
    MoveStep,
    RemoveStep,
    SetParameter,
    ClearParameter,
    SetContact,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, DataKind)]
#[serde(rename_all = "lowercase")]
pub enum SpellKind {
    #[default]
    Spell,
    Building,
    Creature,
    Seed,
}
impl From<SpellKind> for definition::Kind {
    fn from(v: SpellKind) -> Self {
        match v {
            SpellKind::Spell => Self::Spell,
            SpellKind::Building => Self::Building,
            SpellKind::Creature => Self::Creature,
            SpellKind::Seed => Self::Seed,
        }
    }
}
impl From<definition::Kind> for SpellKind {
    fn from(v: definition::Kind) -> Self {
        match v {
            definition::Kind::Spell => Self::Spell,
            definition::Kind::Building => Self::Building,
            definition::Kind::Creature => Self::Creature,
            definition::Kind::Seed => Self::Seed,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, DataKind)]
#[serde(rename_all = "lowercase")]
pub enum ManaSource {
    #[default]
    Wand,
    Town,
    Moon,
    Seed,
}
impl From<ManaSource> for definition::Source {
    fn from(v: ManaSource) -> Self {
        match v {
            ManaSource::Wand => Self::Wand,
            ManaSource::Town => Self::Town,
            ManaSource::Moon => Self::Moon,
            ManaSource::Seed => Self::Seed,
        }
    }
}
impl From<definition::Source> for ManaSource {
    fn from(v: definition::Source) -> Self {
        match v {
            definition::Source::Wand => Self::Wand,
            definition::Source::Town => Self::Town,
            definition::Source::Moon => Self::Moon,
            definition::Source::Seed => Self::Seed,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, DataKind)]
#[serde(rename_all = "lowercase")]
pub enum ParameterKind {
    #[default]
    Number,
    Text,
    Choice,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, DataKind)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    #[default]
    Steps,
    Metadata,
    Catalog,
    File,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellAuthoringUi {
    pub input_error: String,
    #[data(choice)]
    pub section: Section,
    pub connections_open: bool,
    #[data(choice)]
    pub selected_phase: Phase,
    pub selected_step: String,
    pub selected_component: String,
    pub selected_parameter: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellAuthoringRequest {
    pub sequence: u64,
    pub expected_generation: u64,
    #[data(choice)]
    pub operation: Operation,
    pub file: String,
    pub step_id: String,
    #[data(choice)]
    pub phase: Phase,
    pub component: String,
    #[data(choice)]
    pub direction: Direction,
    pub name: String,
    pub title: String,
    #[data(choice)]
    pub kind: SpellKind,
    pub shape: String,
    pub element: String,
    #[data(choice)]
    pub source: ManaSource,
    pub container: String,
    pub allocation: f64,
    pub target: String,
    pub input: String,
    pub output: String,
    pub has_every: bool,
    pub every: i64,
    pub parameter: String,
    #[data(choice)]
    pub parameter_kind: ParameterKind,
    pub value_number: f64,
    pub value_text: String,
    pub contact_target: String,
    pub contact_every: i64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellAuthoringDashboard {
    pub metadata: String,
    pub phases: Vec<String>,
    pub steps: Vec<String>,
    pub selected_step: String,
    pub components: Vec<String>,
    pub selected_component: String,
    pub selected_parameter: String,
    pub file: String,
    pub base_revision: String,
    pub error: String,
    pub validation: String,
    pub notice: String,
    pub generation: u64,
    pub dirty: bool,
    pub valid: bool,
    pub loaded: bool,
    pub new_file: bool,
    pub conflict: bool,
    pub contact_target: String,
    pub contact_every: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellMetadataView {
    pub name: String,
    pub title: String,
    #[data(choice)]
    pub kind: SpellKind,
    pub shape: String,
    pub element: String,
    #[data(choice)]
    pub source: ManaSource,
    pub container: String,
    pub allocation: f64,
    pub containers: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellPhaseView {
    #[data(choice)]
    pub key: Phase,
    pub title: String,
    pub steps: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellStepView {
    pub id: String,
    #[data(choice)]
    pub phase: Phase,
    pub component: String,
    pub title: String,
    pub description: String,
    pub target: String,
    pub input: String,
    pub output: String,
    pub has_every: bool,
    pub every: u64,
    pub parameters: Vec<String>,
    pub index: usize,
    pub total: usize,
    pub takes_target: bool,
    pub takes_input: bool,
    pub takes_output: bool,
    pub target_type: String,
    pub input_type: String,
    pub output_type: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellParameterView {
    pub key: String,
    #[data(choice)]
    pub kind: ParameterKind,
    pub number: f64,
    pub text: String,
    pub choices: Vec<String>,
    pub has_value: bool,
    pub required: bool,
    pub has_min: bool,
    pub min: f64,
    pub has_max: bool,
    pub max: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellCatalogComponentView {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub description: String,
    pub phases: Vec<String>,
    pub parameters: Vec<String>,
}
#[derive(Default)]
pub struct Projection {
    pub dashboard: SpellAuthoringDashboard,
    pub metadata: SpellMetadataView,
    pub phases: BTreeMap<String, SpellPhaseView>,
    pub steps: BTreeMap<String, SpellStepView>,
    pub components: BTreeMap<String, SpellCatalogComponentView>,
    pub parameters: BTreeMap<String, SpellParameterView>,
}
fn segment(id: &str) -> String {
    id.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn path(kind: &str, id: &str) -> String {
    format!("{ROOT}.{kind}.{}", segment(id))
}
fn parameter(
    key: &str,
    p: &definition::Parameter,
    value: Option<&serde_json::Value>,
) -> SpellParameterView {
    let v = value.or(p.default.as_ref());
    SpellParameterView {
        key: key.into(),
        kind: match p.value_type.as_str() {
            "number" => ParameterKind::Number,
            "choice" => ParameterKind::Choice,
            _ => ParameterKind::Text,
        },
        number: v.and_then(|v| v.as_f64()).unwrap_or_default(),
        text: v.and_then(|v| v.as_str()).unwrap_or_default().into(),
        choices: p.choices.clone(),
        has_value: value.is_some(),
        required: p.required,
        has_min: p.min.is_some(),
        min: p.min.unwrap_or_default(),
        has_max: p.max.is_some(),
        max: p.max.unwrap_or_default(),
    }
}
impl Projection {
    pub fn build(draft: Option<&Draft>, ui: &SpellAuthoringUi, error: &str, notice: &str) -> Self {
        let mut p = Self::default();
        p.dashboard.error = error.into();
        p.dashboard.notice = notice.into();
        for c in definition::catalog() {
            let at = path("components", &c.id);
            let mut row = SpellCatalogComponentView {
                id: c.id.clone(),
                title: c.title.clone(),
                kind: c.kind.clone(),
                description: c.description.clone(),
                phases: c.phases.clone(),
                parameters: Vec::new(),
            };
            for (key, param) in &c.params {
                let ref_path = path("parameters", &format!("catalog/{}/{}", c.id, key));
                row.parameters.push(ref_path.clone());
                p.parameters.insert(ref_path, parameter(key, param, None));
            }
            if c.phases
                .iter()
                .any(|v| v == ui.selected_phase.catalog_key())
            {
                p.dashboard.components.push(at.clone());
            }
            if c.id == ui.selected_component {
                p.dashboard.selected_component = at.clone();
            }
            p.components.insert(at, row);
        }
        if !p
            .dashboard
            .components
            .contains(&p.dashboard.selected_component)
        {
            p.dashboard.selected_component =
                p.dashboard.components.first().cloned().unwrap_or_default();
        }
        let Some(draft) = draft else { return p };
        let d = draft.definition();
        let validation = draft.validation();
        p.dashboard.loaded = true;
        p.dashboard.valid = validation.is_ok();
        p.dashboard.validation = validation.err().unwrap_or_default();
        p.dashboard.file = draft
            .path()
            .map(|v| v.display().to_string())
            .unwrap_or_default();
        p.dashboard.base_revision = draft.base_revision();
        p.dashboard.generation = draft.generation();
        p.dashboard.dirty = draft.dirty();
        p.dashboard.new_file = draft.new_file();
        p.dashboard.conflict = draft.conflicted();
        p.dashboard.metadata = format!("{ROOT}.metadata");
        p.dashboard.contact_target = d.phases.on_contact.target.clone();
        p.dashboard.contact_every = d.phases.on_contact.every;
        p.metadata = SpellMetadataView {
            name: d.name.clone(),
            title: d.title.clone().unwrap_or_default(),
            kind: d.kind.into(),
            shape: d.shape.clone().unwrap_or_default(),
            element: d.element.clone().unwrap_or_default(),
            source: d.mana.source.into(),
            container: d.container.clone(),
            allocation: d.mana.allocation,
            containers: vec!["seed".into(), "field".into(), "grand".into()],
        };
        for phase in Phase::ALL {
            let mut row = SpellPhaseView {
                key: phase,
                title: phase.title().into(),
                steps: Vec::new(),
            };
            let steps = phase.steps(d);
            for (index, s) in steps.iter().enumerate() {
                let c = definition::component(&s.component);
                let mut sr = SpellStepView {
                    id: s.id.clone(),
                    phase,
                    component: s.component.clone(),
                    title: c
                        .map(|v| v.title.clone())
                        .unwrap_or_else(|| s.component.clone()),
                    description: c.map(|v| v.description.clone()).unwrap_or_default(),
                    target: s.target.clone().unwrap_or_default(),
                    input: s.input.clone().unwrap_or_default(),
                    output: s.output.clone().unwrap_or_default(),
                    has_every: s.every.is_some(),
                    every: s.every.unwrap_or_default(),
                    parameters: Vec::new(),
                    index,
                    total: steps.len(),
                    takes_target: c.is_some_and(|v| v.target.is_some()),
                    takes_input: c.is_some_and(|v| v.input.is_some()),
                    takes_output: c.is_some_and(|v| v.output.is_some()),
                    target_type: c
                        .and_then(|v| v.target.as_ref())
                        .map(|v| v.value_type.clone())
                        .unwrap_or_default(),
                    input_type: c
                        .and_then(|v| v.input.as_ref())
                        .map(|v| v.value_type.clone())
                        .unwrap_or_default(),
                    output_type: c
                        .and_then(|v| v.output.as_ref())
                        .map(|v| v.value_type.clone())
                        .unwrap_or_default(),
                };
                if let Some(c) = c {
                    for (key, param) in &c.params {
                        let at = path("parameters", &format!("step/{}/{}", s.id, key));
                        sr.parameters.push(at.clone());
                        p.parameters
                            .insert(at, parameter(key, param, s.params.get(key)));
                    }
                }
                let at = path("steps", &s.id);
                row.steps.push(at.clone());
                if phase == ui.selected_phase && s.id == ui.selected_step {
                    p.dashboard.selected_step = at.clone();
                }
                p.steps.insert(at, sr);
            }
            if phase == ui.selected_phase {
                p.dashboard.steps = row.steps.clone();
            }
            let at = path("phases", phase.catalog_key());
            p.dashboard.phases.push(at.clone());
            p.phases.insert(at, row);
        }
        if p.dashboard.selected_step.is_empty() {
            p.dashboard.selected_step = p.dashboard.steps.first().cloned().unwrap_or_default();
        }
        if let Some(s) = p.steps.get(&p.dashboard.selected_step) {
            p.dashboard.selected_parameter = s
                .parameters
                .iter()
                .find(|at| p.parameters[*at].key == ui.selected_parameter)
                .or(s.parameters.first())
                .cloned()
                .unwrap_or_default();
        }
        p
    }
}
