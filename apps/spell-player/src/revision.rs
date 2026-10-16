//! Deliberate revision review/admission. This controller never reconstructs Inventory, Casting or the world.
use crate::config::{self, AuthoredFile, Loaded};
use atelico_data::{Data, DataKind};
use component_spellcraft::player::Player;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, DataKind)]
#[serde(rename_all = "snake_case")]
pub enum RevisionOperation {
    #[default]
    Preview,
    Admit,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerRevisionRequest {
    pub sequence: u64,
    #[data(choice)]
    pub operation: RevisionOperation,
    pub spell_id: String,
    pub expected_current_revision: String,
    pub expected_candidate_revision: String,
    pub expected_preview_token: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerRevisionView {
    pub spell_id: String,
    pub current_revision: String,
    pub candidate_revision: String,
    pub current_title: String,
    pub candidate_title: String,
    pub current_allocation: f64,
    pub candidate_allocation: f64,
    pub source_files: Vec<String>,
    pub preview_token: u64,
    pub valid: bool,
    pub changed: bool,
    pub error: String,
    pub notice: String,
}
struct Candidate {
    spell_id: String,
    old_revision: String,
    authored: AuthoredFile,
}
#[derive(Default)]
pub struct Review {
    seen: u64,
    view: PlayerRevisionView,
    candidate: Option<Candidate>,
}
impl Review {
    pub fn seen(&self) -> u64 {
        self.seen
    }
    pub fn view(&self) -> &PlayerRevisionView {
        &self.view
    }
    /// Returns true only for a newly admitted revision. Duplicate requests never replay.
    pub fn handle(
        &mut self,
        r: &PlayerRevisionRequest,
        path: &Path,
        loaded: &mut Loaded,
        player: &mut Player,
        selected: &str,
    ) -> bool {
        if r.sequence <= self.seen {
            return false;
        }
        self.seen = r.sequence;
        self.view.error.clear();
        match self.apply(r, path, loaded, player, selected) {
            Ok(admitted) => admitted,
            Err(error) => {
                self.view.error = error;
                self.view.valid = false;
                false
            }
        }
    }
    fn apply(
        &mut self,
        r: &PlayerRevisionRequest,
        path: &Path,
        loaded: &mut Loaded,
        player: &mut Player,
        selected: &str,
    ) -> Result<bool, String> {
        let current = loaded
            .spells
            .get(&r.spell_id)
            .ok_or("unknown configured authored spell")?;
        if current.revision() != r.expected_current_revision {
            return Err("The active authored revision changed. Preview it again.".into());
        }
        let spec = loaded
            .config
            .spells
            .iter()
            .find(|s| s.id == r.spell_id)
            .ok_or("unknown configured source files")?;
        match r.operation {
            RevisionOperation::Preview => {
                let token = self
                    .view
                    .preview_token
                    .checked_add(1)
                    .ok_or("revision preview token exhausted")?;
                self.candidate = None;
                self.view = PlayerRevisionView {
                    spell_id: r.spell_id.clone(),
                    current_revision: current.revision().into(),
                    current_title: current.definition().title().into(),
                    current_allocation: current.allocation().get(),
                    preview_token: token,
                    ..Default::default()
                };
                let authored = config::read_spell(path, spec)?;
                player.world().validate_spell(&authored.spell)?;
                self.view.candidate_revision = authored.spell.revision().into();
                self.view.candidate_title = authored.spell.definition().title().into();
                self.view.candidate_allocation = authored.spell.allocation().get();
                self.view.source_files = authored
                    .sources
                    .keys()
                    .map(|p| p.display().to_string())
                    .collect();
                self.view.changed = authored.spell.revision() != current.revision();
                self.view.valid = true;
                self.view.notice = if self.view.changed {
                    "Preview only. Running spells and finite state have not changed."
                } else {
                    "The source matches the active authored revision."
                }
                .into();
                self.candidate = Some(Candidate {
                    spell_id: r.spell_id.clone(),
                    old_revision: current.revision().into(),
                    authored,
                });
                Ok(false)
            }
            RevisionOperation::Admit => {
                let candidate = self
                    .candidate
                    .as_ref()
                    .ok_or("Preview this authored source before admitting it.")?;
                if !self.view.valid
                    || !self.view.changed
                    || r.expected_preview_token != self.view.preview_token
                    || candidate.spell_id != r.spell_id
                    || candidate.old_revision != r.expected_current_revision
                    || candidate.authored.spell.revision() != r.expected_candidate_revision
                {
                    return Err(
                        "The displayed revision preview is stale or unchanged. Preview again."
                            .into(),
                    );
                }
                let authored = config::read_spell(path, spec)?;
                if authored.spell.revision() != r.expected_candidate_revision
                    || authored.sources != candidate.authored.sources
                {
                    return Err("The authored files changed after preview. Preview the exact new source again.".into());
                }
                player.world().validate_spell(&authored.spell)?;
                // All fallible validation precedes replacing the map. select changes only future admissions.
                if selected == r.spell_id {
                    player.select(authored.spell.clone())?;
                }
                loaded.sources.insert(r.spell_id.clone(), authored.sources);
                loaded.spells.insert(r.spell_id.clone(), authored.spell);
                self.candidate = None;
                self.view.valid = false;
                self.view.notice = "Authored revision admitted for future casts. Inventory, cartridge balances, physical world and earlier receipts were preserved.".into();
                Ok(true)
            }
        }
    }
}
