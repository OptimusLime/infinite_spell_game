use atelico_equipment::Equipment;
use atelico_inventory::{Inventory, ItemId, Location, SlotId};
use component_spellcraft::{
    casting::{AuthoredSource, AuthoredSpell, Casting},
    player::Player,
    player_world::{BodySpec, PlayerWorld, WorldConfig},
    supply::{Supply, SupplySnapshot},
};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellFile {
    pub id: String,
    pub definition: PathBuf,
    #[serde(default)]
    pub brain: Option<PathBuf>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReturnLocation {
    pub item: ItemId,
    pub location: Location,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub title: String,
    pub description: String,
    pub actor: String,
    pub equipment_slot: SlotId,
    pub muzzle_offset: f32,
    pub floor: f32,
    pub inventory: Inventory,
    pub equipment: Equipment,
    pub supply: SupplySnapshot,
    pub world: WorldConfig,
    pub bodies: Vec<BodySpec>,
    pub spells: Vec<SpellFile>,
    pub returns: Vec<ReturnLocation>,
    #[serde(default)]
    pub keys: Vec<crate::view::PlayerKeyBinding>,
}
pub struct Loaded {
    pub config: Config,
    pub source: String,
    pub sources: BTreeMap<String, BTreeMap<PathBuf, String>>,
    pub spells: BTreeMap<String, AuthoredSpell>,
}
pub fn load(path: &Path) -> Result<Loaded, String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&source).map_err(|e| e.to_string())?;
    if config.title.trim().is_empty()
        || !config.floor.is_finite()
        || !config.muzzle_offset.is_finite()
        || config.muzzle_offset <= 0.
    {
        return Err("title, finite floor and positive muzzle offset are required".into());
    }
    if !config
        .bodies
        .iter()
        .any(|b| b.id == config.actor && b.role == component_spellcraft::player_world::Role::Actor)
    {
        return Err("actor must name a configured actor body".into());
    }
    let mut spells = BTreeMap::new();
    let mut sources = BTreeMap::new();
    for spec in &config.spells {
        if spec.id.trim().is_empty()
            || spec.id.chars().any(char::is_control)
            || spells.contains_key(&spec.id)
        {
            return Err("spell IDs must be nonempty and unique".into());
        }
        let authored = read_spell(path, spec)?;
        sources.insert(spec.id.clone(), authored.sources);
        spells.insert(spec.id.clone(), authored.spell);
    }
    if spells.is_empty() {
        return Err("at least one authored spell is required".into());
    }
    let mut returned = std::collections::BTreeSet::new();
    for r in &config.returns {
        if config.inventory.item(&r.item).is_none()
            || !returned.insert(&r.item)
            || !matches!(r.location, Location::Grid { .. })
        {
            return Err("return locations need distinct live items and grid destinations".into());
        }
    }
    let mut keys = std::collections::BTreeSet::new();
    for binding in &config.keys {
        use crate::view::Command;
        let key = atelico_component::input::normalize_key(&binding.key);
        if key.is_empty() || key.chars().any(char::is_control) || !keys.insert(key) {
            return Err("keyboard bindings require nonempty distinct keys".into());
        }
        match binding.command {
            Command::Select if !spells.contains_key(&binding.spell_id) => {
                return Err("key binding names an unknown authored spell".into())
            }
            Command::Equip | Command::Load
                if config
                    .inventory
                    .item(&ItemId::new(&binding.item_id).map_err(|e| e.to_string())?)
                    .is_none() =>
            {
                return Err("key binding names an unknown item".into())
            }
            Command::Aim
                if !config.bodies.iter().any(|b| {
                    b.id == binding.target_id
                        && b.role == component_spellcraft::player_world::Role::Target
                }) =>
            {
                return Err("key binding names an unknown aim target".into())
            }
            _ => {}
        }
    }
    Ok(Loaded {
        config,
        source,
        sources,
        spells,
    })
}
pub struct AuthoredFile {
    pub spell: AuthoredSpell,
    pub sources: BTreeMap<PathBuf, String>,
}
/// Read the configured source files as one exact candidate. No session or inventory construction.
pub fn read_spell(config: &Path, spec: &SpellFile) -> Result<AuthoredFile, String> {
    let base = config.parent().ok_or("config has no parent directory")?;
    let definition_path = base
        .join(&spec.definition)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let definition = std::fs::read_to_string(&definition_path).map_err(|e| e.to_string())?;
    let mut sources = BTreeMap::from([(definition_path, definition.clone())]);
    let brain = spec
        .brain
        .as_ref()
        .map(|b| {
            let path = base.join(b).canonicalize().map_err(|e| e.to_string())?;
            let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            sources.insert(path, text.clone());
            Ok::<_, String>(text)
        })
        .transpose()?;
    Ok(AuthoredFile {
        spell: AuthoredSpell::new(AuthoredSource { definition, brain })?,
        sources,
    })
}
pub struct Flat(pub f32);
impl atelico_items::physics::Terrain for Flat {
    fn height(&self, _: f32, _: f32) -> f32 {
        self.0
    }
    fn blocked(&self, _: glam::Vec3, _: f32) -> bool {
        false
    }
}
impl Loaded {
    pub fn player(&self) -> Result<Player, String> {
        let c = &self.config;
        let world = PlayerWorld::new(c.world.clone(), c.bodies.clone(), Box::new(Flat(c.floor)))?;
        for s in self.spells.values() {
            world.validate_spell(s)?;
        }
        Player::new(
            c.inventory.clone(),
            c.equipment.clone(),
            Casting::new(Supply::restore(&c.inventory, c.supply.clone())?),
            world,
            c.equipment_slot.clone(),
        )
    }
}
