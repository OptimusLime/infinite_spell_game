//! Editable authored definitions. Drafts can be incomplete; only validated drafts cross an explicit save boundary.
//! This module never executes a spell or changes a running player's admitted revision.
use crate::definition::{self, ContactPhase, Kind, Mana, Phases, Source, SpellDefinition, Step};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, atelico_data::DataKind,
)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Start,
    Tick,
    Contact,
}
impl Phase {
    pub fn title(self) -> &'static str {
        match self {
            Self::Start => "On start",
            Self::Tick => "On tick",
            Self::Contact => "On contact",
        }
    }
    pub fn catalog_key(self) -> &'static str {
        match self {
            Self::Start => "on_start",
            Self::Tick => "on_tick",
            Self::Contact => "on_contact",
        }
    }
    pub fn steps(self, d: &SpellDefinition) -> &[Step] {
        match self {
            Self::Start => &d.phases.on_start,
            Self::Tick => &d.phases.on_tick,
            Self::Contact => &d.phases.on_contact.steps,
        }
    }
    fn steps_mut(self, d: &mut SpellDefinition) -> &mut Vec<Step> {
        match self {
            Self::Start => &mut d.phases.on_start,
            Self::Tick => &mut d.phases.on_tick,
            Self::Contact => &mut d.phases.on_contact.steps,
        }
    }
    pub const ALL: [Self; 3] = [Self::Start, Self::Tick, Self::Contact];
}
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, atelico_data::DataKind,
)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Up,
    Down,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ParameterValue {
    Number(f64),
    Text(String),
}
#[derive(Clone, Debug)]
pub struct Metadata {
    pub name: String,
    pub title: String,
    pub kind: Kind,
    pub shape: String,
    pub element: String,
    pub source: Source,
    pub container: String,
    pub allocation: f64,
}
#[derive(Clone, Debug)]
pub struct Ports {
    pub target: String,
    pub input: String,
    pub output: String,
    pub every: Option<u64>,
}
#[derive(Clone, Debug)]
pub enum Command {
    Metadata(Metadata),
    Add {
        phase: Phase,
        id: String,
        component: String,
    },
    Ports {
        id: String,
        ports: Ports,
    },
    Move {
        id: String,
        direction: Direction,
    },
    Remove {
        id: String,
    },
    Parameter {
        id: String,
        key: String,
        value: Option<ParameterValue>,
    },
    Contact {
        target: String,
        every: u64,
    },
}
fn optional(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}
pub fn revision(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))
}

#[derive(Clone)]
pub struct Draft {
    definition: SpellDefinition,
    path: Option<PathBuf>,
    base: Option<String>,
    generation: u64,
    dirty: bool,
}
impl Draft {
    /// No allocation or spell behavior is invented. The empty draft is invalid until its author fills it in.
    pub fn new() -> Self {
        Self {
            definition: SpellDefinition {
                version: 1,
                name: String::new(),
                title: None,
                kind: Kind::Spell,
                shape: None,
                element: None,
                mana: Mana {
                    source: Source::Wand,
                    allocation: 0.,
                },
                container: String::new(),
                phases: Phases::default(),
            },
            path: None,
            base: None,
            generation: 1,
            dirty: true,
        }
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let definition = definition::parse(&source)?;
        Ok(Self {
            definition,
            path: Some(path),
            base: Some(source),
            generation: 1,
            dirty: false,
        })
    }
    /// A new/load operation invalidates every previously displayed command, even across file switches.
    pub fn replace(
        &mut self,
        expected_generation: u64,
        mut replacement: Draft,
    ) -> Result<(), String> {
        self.expect(expected_generation)?;
        replacement.generation = self.next()?;
        *self = replacement;
        Ok(())
    }
    pub fn definition(&self) -> &SpellDefinition {
        &self.definition
    }
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn dirty(&self) -> bool {
        self.dirty
    }
    pub fn new_file(&self) -> bool {
        self.base.is_none()
    }
    pub fn base_revision(&self) -> String {
        self.base.as_deref().map(revision).unwrap_or_default()
    }
    pub fn validation(&self) -> Result<(), String> {
        definition::validate(&self.definition)
    }
    pub fn conflicted(&self) -> bool {
        self.path
            .as_ref()
            .zip(self.base.as_ref())
            .is_some_and(|(path, base)| fs::read_to_string(path).ok().as_ref() != Some(base))
    }
    fn expect(&self, generation: u64) -> Result<(), String> {
        if generation != self.generation {
            return Err(
                "The displayed draft changed. Inspect its current revision and try again.".into(),
            );
        }
        Ok(())
    }
    fn next(&self) -> Result<u64, String> {
        self.generation
            .checked_add(1)
            .ok_or_else(|| "draft generation exhausted".into())
    }
    pub fn apply(&mut self, expected_generation: u64, command: Command) -> Result<(), String> {
        self.expect(expected_generation)?;
        let mut d = self.definition.clone();
        match command {
            Command::Metadata(m) => {
                if !m.allocation.is_finite() || m.allocation < 0. {
                    return Err("Allocation must be finite and nonnegative; a saved spell needs a positive allocation.".into());
                }
                d.name = m.name;
                d.title = optional(m.title);
                d.kind = m.kind;
                d.shape = optional(m.shape);
                d.element = optional(m.element);
                d.mana = Mana {
                    source: m.source,
                    allocation: m.allocation,
                };
                d.container = m.container;
            }
            Command::Add {
                phase,
                id,
                component,
            } => {
                if !identifier(&id) || d.steps().any(|(_, s)| s.id == id) {
                    return Err(
                        "A step needs a fresh identifier (letters, digits, underscore or hyphen)."
                            .into(),
                    );
                }
                let c = definition::component(&component).ok_or("Unknown catalog component.")?;
                if !c.phases.iter().any(|p| p == phase.catalog_key()) {
                    return Err("That component cannot run in this phase.".into());
                }
                phase.steps_mut(&mut d).push(Step {
                    id,
                    component,
                    target: None,
                    input: None,
                    output: None,
                    every: None,
                    params: BTreeMap::new(),
                });
            }
            Command::Ports { id, ports } => {
                let (phase, index) = locate(&d, &id)?;
                if ports
                    .every
                    .is_some_and(|n| phase != Phase::Tick || n > 3600)
                {
                    return Err("Only on-tick steps have a schedule, from 0 to 3600 ticks.".into());
                }
                let s = &mut phase.steps_mut(&mut d)[index];
                s.target = optional(ports.target);
                s.input = optional(ports.input);
                s.output = optional(ports.output);
                s.every = ports.every;
            }
            Command::Move { id, direction } => {
                let (phase, index) = locate(&d, &id)?;
                let steps = phase.steps_mut(&mut d);
                let other = match direction {
                    Direction::Up => index.checked_sub(1),
                    Direction::Down => (index + 1 < steps.len()).then_some(index + 1),
                }
                .ok_or("The step is already at this end of its phase.")?;
                steps.swap(index, other);
            }
            Command::Remove { id } => {
                let (phase, index) = locate(&d, &id)?;
                phase.steps_mut(&mut d).remove(index);
            }
            Command::Parameter { id, key, value } => {
                let (phase, index) = locate(&d, &id)?;
                let step = &mut phase.steps_mut(&mut d)[index];
                let c = definition::component(&step.component).ok_or("Unknown component.")?;
                let p = c.params.get(&key).ok_or("Unknown catalog parameter.")?;
                match value {
                    None => {
                        step.params.remove(&key);
                    }
                    Some(v) => {
                        let value=match (p.value_type.as_str(),v) {
                            ("number",ParameterValue::Number(n)) if n.is_finite() && p.min.is_none_or(|v|n>=v) && p.max.is_none_or(|v|n<=v)=>serde_json::Value::from(n),
                            ("choice",ParameterValue::Text(t)) if p.choices.contains(&t)=>serde_json::Value::String(t),
                            ("text",ParameterValue::Text(t)) if t.len()<=4096=>serde_json::Value::String(t),
                            _=>return Err("The parameter has the wrong type or is outside its catalog bounds/choices.".into()),
                        };
                        step.params.insert(key, value);
                    }
                }
            }
            Command::Contact { target, every } => {
                if every > 3600 {
                    return Err("Contact schedule must be 0 to 3600 ticks.".into());
                }
                d.phases.on_contact = ContactPhase {
                    target,
                    every,
                    steps: d.phases.on_contact.steps,
                };
            }
        }
        if d != self.definition {
            self.generation = self.next()?;
            self.definition = d;
            self.dirty = true;
        }
        Ok(())
    }
    /// Reload is deliberate and discards the unsaved draft only after reading a valid replacement.
    pub fn reload(&mut self, expected_generation: u64) -> Result<(), String> {
        self.expect(expected_generation)?;
        let mut loaded = Self::load(
            self.path
                .as_deref()
                .ok_or("This draft has no saved file.")?,
        )?;
        loaded.generation = self.next()?;
        *self = loaded;
        Ok(())
    }
    pub fn save(&mut self, expected_generation: u64) -> Result<PathBuf, String> {
        self.expect(expected_generation)?;
        let path = self.path.clone().ok_or("Choose Save as for a new draft.")?;
        self.save_to(&path, false)
    }
    /// Save-as creates a distinct file, never overwriting another author's file or the current source.
    pub fn save_as(&mut self, expected_generation: u64, path: &Path) -> Result<PathBuf, String> {
        self.expect(expected_generation)?;
        self.save_to(path, true)
    }
    fn save_to(&mut self, path: &Path, create: bool) -> Result<PathBuf, String> {
        self.validation()?;
        let text = definition::to_yaml(&self.definition)?;
        if text.len() > definition::MAX_SOURCE_BYTES {
            return Err("Saved spell exceeds the source size limit.".into());
        }
        let generation = self.next()?;
        let path = save_path(path)?;
        let lock_path = path.with_file_name(format!(
            "{}.lock",
            path.file_name().unwrap().to_string_lossy()
        ));
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|e| e.to_string())?;
        lock.lock().map_err(|e| e.to_string())?;
        if create {
            if path.exists() {
                return Err("Save-as target already exists. Choose a new file; existing source was preserved.".into());
            }
        } else if fs::read_to_string(&path).ok().as_ref() != self.base.as_ref() {
            return Err(
                "The source file changed. Reload it or save this draft under a new name.".into(),
            );
        }
        atomic_write(&path, &text, create, self.base.as_deref())?;
        self.path = Some(path.clone());
        self.base = Some(text);
        self.generation = generation;
        self.dirty = false;
        Ok(path)
    }
}
impl Default for Draft {
    fn default() -> Self {
        Self::new()
    }
}
fn locate(d: &SpellDefinition, id: &str) -> Result<(Phase, usize), String> {
    Phase::ALL
        .into_iter()
        .find_map(|p| p.steps(d).iter().position(|s| s.id == id).map(|i| (p, i)))
        .ok_or_else(|| "Unknown draft step.".into())
}
fn save_path(path: &Path) -> Result<PathBuf, String> {
    if !path
        .file_name()
        .and_then(|v| v.to_str())
        .is_some_and(|v| v.ends_with(".spell.yaml"))
    {
        return Err("Choose a .spell.yaml file.".into());
    }
    let parent = path
        .parent()
        .ok_or("The file needs a parent directory.")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let result = parent.join(path.file_name().unwrap());
    if fs::symlink_metadata(&result).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err("Save through the source's canonical path, not a symbolic link.".into());
    }
    Ok(result)
}
fn atomic_write(
    path: &Path,
    text: &str,
    create: bool,
    expected: Option<&str>,
) -> Result<(), String> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let temp = path.with_file_name(format!(
        "{}.tmp-{}-{}",
        path.file_name().unwrap().to_string_lossy(),
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let result = (|| {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        f.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
        if create {
            // The final name appears atomically and only if absent, even against a non-cooperating creator.
            fs::hard_link(&temp, path)
                .map_err(|e| format!("Could not create without replacing an existing file: {e}"))?;
            fs::remove_file(&temp).map_err(|e| e.to_string())?;
        } else {
            if fs::read_to_string(path).ok().as_deref() != expected {
                return Err("Source changed during save; existing content was preserved.".into());
            }
            fs::rename(&temp, path).map_err(|e| e.to_string())?;
        }
        fs::File::open(path.parent().unwrap())
            .and_then(|d| d.sync_all())
            .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
