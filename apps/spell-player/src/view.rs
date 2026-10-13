use atelico_data::{Data, DataKind};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SpellPlayer {
    pub config: String,
    pub session: String,
    pub world: String,
    pub show: String,
    /// Optional caller-owned body-to-part presentation; does not change the funded session's config.
    #[data(choice)]
    pub visuals: crate::visuals::Visuals,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, DataKind, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    #[default]
    Select,
    Equip,
    Load,
    Unload,
    Aim,
    Fire,
    ToggleInventory,
    ToggleEquipment,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerRequest {
    pub sequence: u64,
    #[data(choice)]
    pub command: Command,
    pub item_id: String,
    pub spell_id: String,
    pub target_id: String,
    pub expected_token: u64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerUi {
    pub inventory_open: bool,
    pub equipment_open: bool,
    pub sources_open: bool,
    pub capacity_open: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerDashboard {
    pub title: String,
    pub description: String,
    pub session: String,
    pub config: String,
    pub source_changed: bool,
    pub source_error: String,
    pub inventory: String,
    pub inventory_request: String,
    pub inventory_interaction: String,
    pub equipment: String,
    pub supply: String,
    pub capacity: String,
    pub capacity_request: String,
    pub request: String,
    pub revision_request: String,
    pub revision_review: String,
    pub ui: String,
    pub spells: Vec<String>,
    pub keys: Vec<String>,
    pub wands: Vec<String>,
    pub targets: Vec<String>,
    pub bodies: Vec<String>,
    pub selected_spell: String,
    pub aimed_target: String,
    pub aim_x: f32,
    pub aim_y: f32,
    pub aim_z: f32,
    pub has_aim: bool,
    pub token: u64,
    pub tick: u64,
    pub message: String,
    pub error: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerSpell {
    pub id: String,
    pub title: String,
    pub revision: String,
    pub allocation: f64,
    pub selected: bool,
}

/// Caller-authored key-to-command mapping; IDs are explicit rather than implicit cycling/nearest selection.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerKeyBinding {
    pub key: String,
    #[data(choice)]
    pub command: Command,
    pub item_id: String,
    pub spell_id: String,
    pub target_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerCapacityRequest {
    pub sequence: u64,
    pub item_id: String,
    pub source_id: String,
    pub expected_token: u64,
    pub expected_config: String,
    pub expected_remaining: f64,
}
