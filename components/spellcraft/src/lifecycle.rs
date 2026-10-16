//! Terminal accounting for finite casts. Unused transferred energy stays recorded, never refunded.
use crate::{
    definition::{component, Step},
    runtime::{Instance, WorldAdapter},
};
use atelico_data::{Data, DataKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, DataKind)]
#[serde(rename_all = "snake_case")]
pub enum TerminalStatus {
    #[default]
    Completed,
    Cancelled,
    Exhausted,
    Faulted,
}
impl TerminalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Exhausted => "exhausted",
            Self::Faulted => "faulted",
        }
    }
}
/// Compact retained result: no VM, schedules, observations or owned physical bodies remain active.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct TerminalCast {
    pub id: String,
    pub revision: String,
    pub title: String,
    #[data(choice)]
    pub status: TerminalStatus,
    pub reason: String,
    pub ended_tick: u64,
    pub initial: f64,
    /// Transferred but unused allocation. This is a historical amount, not a refundable balance.
    pub unused: f64,
    pub sensors: f64,
    pub logic: f64,
    pub rules: f64,
    pub effectors: f64,
    pub dissipation: f64,
}
impl TerminalCast {
    pub(crate) fn finish(i: Instance, status: TerminalStatus, reason: String, tick: u64) -> Self {
        Self {
            id: i.id.clone(),
            revision: i.revision.clone(),
            title: i.definition.title().into(),
            status,
            reason,
            ended_tick: tick,
            initial: i.initial,
            unused: i.energy,
            sensors: *i.spent.get("sensors").unwrap_or(&0.),
            logic: *i.spent.get("logic").unwrap_or(&0.),
            rules: *i.spent.get("rules").unwrap_or(&0.),
            effectors: *i.spent.get("effectors").unwrap_or(&0.),
            dissipation: *i.spent.get("dissipation").unwrap_or(&0.),
        }
    }
    pub fn spent_total(&self) -> f64 {
        self.sensors + self.logic + self.rules + self.effectors + self.dissipation
    }
}

fn direct_target_live(i: &Instance, selector: Option<&str>, world: &dyn WorldAdapter) -> bool {
    match selector {
        None => world.exists(&i.holder),
        Some(name) => {
            i.bindings.get(name).is_some_and(|body| world.exists(body))
                || i.observations
                    .get(name)
                    .is_some_and(|o| o.bodies.iter().any(|body| world.exists(body)))
        }
    }
}
fn scheduled(i: &Instance, step: &Step) -> bool {
    i.next.contains_key(&step.id)
}
/// Conservative future-work check over the existing schedule, including sensors whose next sample is empty now.
/// A scheduled brain can do work without a body; completion must not turn that authored computation off early.
pub(crate) fn has_live_work(i: &Instance, world: &dyn WorldAdapter) -> bool {
    let contact = &i.definition.phases.on_contact;
    if i.next.contains_key("contact") && direct_target_live(i, Some(&contact.target), world) {
        return true;
    }
    i.definition
        .phases
        .on_tick
        .iter()
        .filter(|s| scheduled(i, s))
        .any(|s| {
            if component(&s.component).is_some_and(|c| c.kind == "logic") {
                return true;
            }
            if direct_target_live(i, s.target.as_deref(), world) {
                return true;
            }
            // A currently empty observation may acquire targets at a later paid sensor sample.
            s.target.as_ref().is_some_and(|target| {
                i.definition.phases.on_tick.iter().any(|sensor| {
                    sensor.output.as_ref() == Some(target)
                        && scheduled(i, sensor)
                        && component(&sensor.component).is_some_and(|c| c.kind == "sensor")
                        && direct_target_live(i, sensor.target.as_deref(), world)
                })
            })
        })
}
