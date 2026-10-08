use crate::{
    config::{self, Loaded},
    view::*,
};
use atelico_data::Data;
use atelico_inventory::{
    binding::Binding as InventoryBinding,
    interaction::{Controller, InteractionRequest},
    ItemId, Mutation,
};
use atelico_store::{Op, Origin, Store};
use component_spellcraft::{
    aim::Aim, casting::CastRequest, player::Player, player_world::BodyView,
    supply_view::Binding as SupplyBinding,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
fn encode(id: &str) -> String {
    id.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn record<T: serde::Serialize>(
    store: &Store,
    path: &str,
    row: &T,
    ops: &mut Vec<Op>,
) -> Result<(), String> {
    let value = serde_json::to_value(row).map_err(|e| e.to_string())?;
    ops.extend(atelico_store::diff(path, store.get(path).as_ref(), &value));
    Ok(())
}
fn bind<T: Data>(s: &Store, path: &str) -> Result<(), String> {
    let schema = T::schema();
    let name = schema.name.clone();
    let mut e = s.define(schema);
    e.extend(s.bind(path, &name));
    if e.is_empty() {
        Ok(())
    } else {
        Err(e.join("; "))
    }
}
fn write<T: serde::Serialize>(s: &Store, path: &str, row: &T) -> Result<(), String> {
    let mut ops = vec![];
    record(s, path, row, &mut ops)?;
    s.batch(ops, Origin::Host)?;
    Ok(())
}
pub fn canonical(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .or_else(|_| {
            path.parent()
                .unwrap_or(Path::new("."))
                .canonicalize()
                .map(|p| p.join(path.file_name().unwrap_or_default()))
        })
        .map_err(|e| e.to_string())
}
pub fn identity(path: &Path, name: &str) -> Result<String, String> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err("an explicit nonempty session ID is required".into());
    }
    let mut h = Sha256::new();
    h.update(path.to_string_lossy().as_bytes());
    h.update([0]);
    h.update(name.as_bytes());
    Ok(format!("runtime.spell_player.sessions.p{:x}", h.finalize()))
}
pub struct Session {
    pub player: Player,
    loaded: Loaded,
    path: PathBuf,
    name: String,
    pub root: String,
    inventory: InventoryBinding,
    equipment: atelico_equipment::binding::Binding,
    supply: SupplyBinding,
    controller: Option<Controller>,
    revision: crate::revision::Review,
    seen: u64,
    token: u64,
    selected: String,
    aimed: String,
    aim: Option<Aim>,
    message: String,
    error: String,
    last_clock: Option<f64>,
    accumulated: f64,
    checked: Instant,
    source_changed: bool,
    source_error: String,
}
impl Session {
    pub fn new(path: PathBuf, name: String) -> Result<Self, String> {
        let root = identity(&path, &name)?;
        let loaded = config::load(&path)?;
        let player = loaded.player()?;
        let inventory = InventoryBinding::new(format!("{root}.inventory"))?;
        let equipment = atelico_equipment::binding::Binding::new(
            format!("{root}.equipment"),
            inventory.clone(),
        )?;
        let supply = SupplyBinding::new(format!("{root}.supply"), inventory.paths().clone())?;
        Ok(Self {
            player,
            loaded,
            path,
            name,
            root,
            inventory,
            equipment,
            supply,
            controller: None,
            revision: Default::default(),
            seen: 0,
            token: 1,
            selected: String::new(),
            aimed: String::new(),
            aim: None,
            message: "Equip a wand, load a cartridge, select a spell, aim, then fire.".into(),
            error: String::new(),
            last_clock: None,
            accumulated: 0.,
            checked: Instant::now() - Duration::from_secs(1),
            source_changed: false,
            source_error: String::new(),
        })
    }
    pub fn aimed_target(&self) -> &str {
        &self.aimed
    }
    pub fn dashboard_path(&self) -> String {
        format!("{}.dashboard", self.root)
    }
    pub fn request_path(&self) -> String {
        format!("{}.request", self.root)
    }
    pub fn revision_request_path(&self) -> String {
        format!("{}.revision_request", self.root)
    }
    pub fn revision_review_path(&self) -> String {
        format!("{}.revision_review", self.root)
    }
    fn path(&self, kind: &str, id: &str) -> String {
        format!("{}.{kind}.{}", self.root, encode(id))
    }
    fn ensure(&mut self, s: &Store) -> Result<(), String> {
        // Installation retypes the Store; perform it once per bound session, not on every paint.
        if s.type_of(&self.dashboard_path()).as_deref() != Some("PlayerDashboard") {
            self.equipment.install(s)?;
            self.supply.install(s)?;
            bind::<PlayerDashboard>(s, &self.dashboard_path())?;
            bind::<PlayerRequest>(s, &self.request_path())?;
            bind::<crate::revision::PlayerRevisionRequest>(s, &self.revision_request_path())?;
            bind::<crate::revision::PlayerRevisionView>(s, &self.revision_review_path())?;
            bind::<PlayerUi>(s, &format!("{}.ui", self.root))?;
            bind::<PlayerSpell>(s, &format!("{}.spells.*", self.root))?;
            bind::<PlayerKeyBinding>(s, &format!("{}.keys.*", self.root))?;
            bind::<BodyView>(s, &format!("{}.bodies.*", self.root))?;
        }
        if s.object(&self.request_path()).is_none() {
            write(
                s,
                &self.request_path(),
                &PlayerRequest {
                    sequence: self.seen,
                    ..Default::default()
                },
            )?;
        }
        if s.object(&self.revision_request_path()).is_none() {
            write(
                s,
                &self.revision_request_path(),
                &crate::revision::PlayerRevisionRequest {
                    sequence: self.revision.seen(),
                    ..Default::default()
                },
            )?;
        }
        if s.object(&format!("{}.ui", self.root)).is_none() {
            write(s, &format!("{}.ui", self.root), &PlayerUi::default())?;
        }
        if self.controller.is_none()
            || s.object(&Controller::request_path(&self.inventory))
                .is_none()
        {
            self.controller = Some(Controller::install(s, &self.inventory)?);
        }
        Ok(())
    }
    fn source_check(&mut self) {
        if self.checked.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.checked = Instant::now();
        let changed = std::fs::read_to_string(&self.path).ok().as_deref()
            != Some(self.loaded.source.as_str())
            || self
                .loaded
                .sources
                .values()
                .flat_map(|files| files.iter())
                .any(|(p, text)| std::fs::read_to_string(p).ok().as_deref() != Some(text));
        self.source_changed = changed;
        self.source_error = if changed {
            match config::load(&self.path).and_then(|l|l.player().map(|_|())){Ok(())=>"Configuration or authored sources changed. Current finite state is preserved. Review and explicitly admit authored spell edits in Sources; configuration changes require a new session.".into(),Err(e)=>format!("Edited configuration is invalid; current session preserved: {e}")}
        } else {
            String::new()
        };
    }
    fn apply(&mut self, r: &PlayerRequest, s: &Store) -> Result<(), String> {
        if r.expected_token != self.token {
            return Err(
                "The displayed player state changed. Inspect the controls and try again.".into(),
            );
        }
        match r.command {
            Command::ToggleInventory | Command::ToggleEquipment => {
                let path = format!("{}.ui", self.root);
                let mut ui: PlayerUi = s.read(&path).ok_or("invalid typed player view state")?;
                if r.command == Command::ToggleInventory {
                    ui.inventory_open = !ui.inventory_open;
                    if ui.inventory_open {
                        ui.sources_open = false;
                    }
                } else {
                    ui.equipment_open = !ui.equipment_open;
                    if ui.equipment_open {
                        ui.sources_open = false;
                    }
                }
                write(s, &path, &ui)?;
            }
            Command::Select => {
                let spell = self
                    .loaded
                    .spells
                    .get(&r.spell_id)
                    .ok_or("unknown authored spell")?
                    .clone();
                self.player.select(spell)?;
                self.selected = r.spell_id.clone();
                self.message = "Spell selected. Selection does not fire.".into();
            }
            Command::Equip => {
                let id = ItemId::new(&r.item_id).map_err(|e| e.to_string())?;
                let from = self
                    .player
                    .inventory()
                    .item(&id)
                    .ok_or("unknown item")?
                    .location
                    .clone();
                self.player.equip(&id, &from)?;
                self.message = "Wand equipped from Inventory.".into();
            }
            Command::Load => {
                let id = ItemId::new(&r.item_id).map_err(|e| e.to_string())?;
                let supply = self.player.casting().supply();
                let wand = supply
                    .equipped_wand(
                        self.player.inventory(),
                        self.player.equipment(),
                        self.player.equipment_slot(),
                    )?
                    .ok_or("equip a wand first")?
                    .clone();
                let loaded = supply.loaded(self.player.inventory(), &wand)?.cloned();
                let from = self
                    .player
                    .inventory()
                    .item(&id)
                    .ok_or("unknown cartridge item")?
                    .location
                    .clone();
                let to = loaded
                    .as_ref()
                    .map(|item| {
                        self.loaded
                            .config
                            .returns
                            .iter()
                            .find(|r| &r.item == item)
                            .map(|r| r.location.clone())
                            .ok_or("loaded cartridge needs an explicit return location")
                    })
                    .transpose()?;
                self.player.load(&wand, &id, &from, loaded.as_ref(), to)?;
                self.message="Cartridge loaded; any previous cartridge returned to its configured Inventory cells.".into();
            }
            Command::Unload => {
                let supply = self.player.casting().supply();
                let wand = supply
                    .equipped_wand(
                        self.player.inventory(),
                        self.player.equipment(),
                        self.player.equipment_slot(),
                    )?
                    .ok_or("equip a wand first")?
                    .clone();
                let cartridge = supply
                    .loaded(self.player.inventory(), &wand)?
                    .ok_or("no loaded cartridge")?
                    .clone();
                let to = self
                    .loaded
                    .config
                    .returns
                    .iter()
                    .find(|r| r.item == cartridge)
                    .ok_or("cartridge needs an explicit return location")?
                    .location
                    .clone();
                self.player.unload(&wand, &cartridge, to)?;
                self.message = "Cartridge returned to Inventory.".into();
            }
            Command::Aim => {
                if !self.loaded.config.bodies.iter().any(|b| {
                    b.id == r.target_id
                        && b.role == component_spellcraft::player_world::Role::Target
                }) {
                    return Err("choose a configured target".into());
                }
                let bodies = self.player.world().bodies();
                let actor = bodies
                    .iter()
                    .find(|b| b.id == self.loaded.config.actor && b.alive)
                    .ok_or("actor is unavailable")?;
                let target = bodies
                    .iter()
                    .find(|b| b.id == r.target_id && b.alive)
                    .ok_or("target is unavailable")?;
                let direction = (target.position() - actor.position()).normalize_or_zero();
                let origin = actor.position() + direction * self.loaded.config.muzzle_offset;
                self.aim = Some(Aim::new(origin.to_array(), direction.to_array())?);
                self.aimed = r.target_id.clone();
                self.message="Aim set to the target's current position. Fire is explicit; shots do not home.".into();
            }
            Command::Fire => {
                let aim = self.aim.ok_or("aim at a target first")?;
                let selected = self.player.selected().ok_or("select a spell first")?;
                let supply = self.player.casting().supply();
                let wand = supply
                    .equipped_wand(
                        self.player.inventory(),
                        self.player.equipment(),
                        self.player.equipment_slot(),
                    )?
                    .ok_or("equip a wand first")?
                    .clone();
                let cartridge = supply
                    .loaded(self.player.inventory(), &wand)?
                    .ok_or("load a cartridge first")?
                    .clone();
                let remaining = supply
                    .cartridge(&cartridge)
                    .ok_or("cartridge contents missing")?
                    .remaining;
                let receipt = self.player.fire(CastRequest {
                    aim,
                    id: format!("{}:{}", self.root, r.sequence),
                    owner: self.player.equipment().owner().clone(),
                    equipment_slot: self.player.equipment_slot().clone(),
                    expected_wand: wand,
                    expected_cartridge: cartridge,
                    expected_remaining: remaining,
                    expected_revision: selected.revision().into(),
                    holder: self.loaded.config.actor.clone(),
                    tick: self.player.world().tick(),
                })?;
                self.message = format!(
                    "Fired {} energy from {}. Loaded cartridge: {} → {}.",
                    receipt.allocation.get(),
                    receipt.request.expected_cartridge,
                    receipt.before.get(),
                    receipt.after.get()
                );
            }
        }
        Ok(())
    }
    pub fn frame(&mut self, s: &Store, now: f64) -> Result<String, String> {
        self.frame_input(s, now, &BTreeSet::new())
    }
    pub fn frame_input(
        &mut self,
        s: &Store,
        now: f64,
        pressed: &BTreeSet<String>,
    ) -> Result<String, String> {
        self.ensure(s)?;
        self.source_check();
        let mut request = s
            .read::<PlayerRequest>(&self.request_path())
            .ok_or("invalid typed player request")?;
        let bindings: Vec<_> = self
            .loaded
            .config
            .keys
            .iter()
            .filter(|b| pressed.contains(&atelico_component::input::normalize_key(&b.key)))
            .collect();
        if bindings.len() > 1 {
            self.error =
                "Press one configured action at a time; no keyboard action was applied.".into();
        } else if let Some(binding) = bindings.first() {
            if request.sequence > self.seen {
                // A completed pointer request already owns this slot; never overwrite its admission.
                self.error = "A pointer action is pending; release the key and press again.".into();
            } else {
                let displayed: PlayerDashboard = s
                    .read(&self.dashboard_path())
                    .ok_or("player controls have not been displayed")?;
                request = PlayerRequest {
                    sequence: request
                        .sequence
                        .checked_add(1)
                        .ok_or("request sequence exhausted")?,
                    command: binding.command,
                    item_id: binding.item_id.clone(),
                    spell_id: binding.spell_id.clone(),
                    target_id: binding.target_id.clone(),
                    expected_token: displayed.token,
                };
                s.write(&self.request_path(), &request, Origin::Gui)?;
            }
        }
        if request.sequence > self.seen {
            self.seen = request.sequence;
            self.error.clear();
            if let Err(e) = self.apply(&request, s) {
                self.error = e;
            }
            self.token = self.token.checked_add(1).ok_or("request token exhausted")?;
        }
        let revision_request = s
            .read::<crate::revision::PlayerRevisionRequest>(&self.revision_request_path())
            .ok_or("invalid typed revision request")?;
        if self.revision.handle(
            &revision_request,
            &self.path,
            &mut self.loaded,
            &mut self.player,
            &self.selected,
        ) {
            self.token = self.token.checked_add(1).ok_or("request token exhausted")?;
            self.checked = Instant::now() - Duration::from_secs(1);
            self.source_check();
        }
        let mut candidate = self.player.inventory().clone();
        let inv_req = s
            .read::<InteractionRequest>(&Controller::request_path(&self.inventory))
            .ok_or("invalid typed Inventory request")?;
        let controller = self.controller.as_mut().unwrap();
        controller.refresh(s, &self.inventory, &candidate)?;
        let outcome =
            controller.handle_checked(s, &self.inventory, &mut candidate, &inv_req, |inv| {
                self.player
                    .equipment()
                    .validate(inv)
                    .map_err(|e| e.to_string())?;
                self.player.casting().supply().validate(inv)
            })?;
        if outcome.changed {
            self.player.edit_inventory(|inv| {
                let changes = candidate.changes_from(inv);
                *inv = candidate;
                Ok(Mutation { value: (), changes })
            })?;
            self.token = self.token.checked_add(1).ok_or("request token exhausted")?;
        }
        if now.is_finite() {
            if let Some(last) = self.last_clock {
                self.accumulated += (now - last).clamp(0., 0.1);
            }
            self.last_clock = Some(now);
            let dt = self.player.world().tick_seconds() as f64;
            let mut steps = 0;
            while self.accumulated >= dt && steps < 8 {
                self.player.advance()?;
                self.accumulated -= dt;
                steps += 1;
            }
        }
        self.publish(s)?;
        Ok(self.dashboard_path())
    }
    pub fn publish(&self, s: &Store) -> Result<(), String> {
        self.equipment
            .publish(s, self.player.inventory(), self.player.equipment())
            .map_err(|e| e.to_string())?;
        let supply = self.supply.project(
            self.player.casting(),
            self.player.inventory(),
            self.player.equipment(),
            self.player.equipment_slot(),
            self.player.selected(),
            self.player.instances(),
        )?;
        let mut ops = self.supply.operations(s, &supply)?;
        let mut view = PlayerDashboard {
            title: self.loaded.config.title.clone(),
            description: self.loaded.config.description.clone(),
            session: self.name.clone(),
            config: self.path.display().to_string(),
            source_changed: self.source_changed,
            source_error: self.source_error.clone(),
            inventory: self.inventory.paths().dashboard(),
            inventory_request: Controller::request_path(&self.inventory),
            inventory_interaction: Controller::view_path(&self.inventory),
            equipment: self.equipment.paths().dashboard(),
            supply: self.supply.dashboard(),
            request: self.request_path(),
            revision_request: self.revision_request_path(),
            revision_review: self.revision_review_path(),
            ui: format!("{}.ui", self.root),
            selected_spell: self.selected.clone(),
            aimed_target: self.aimed.clone(),
            has_aim: self.aim.is_some(),
            token: self.token,
            tick: self.player.world().tick(),
            message: self.message.clone(),
            error: self.error.clone(),
            ..Default::default()
        };
        if let Some(aim) = self.aim {
            let d = aim.direction();
            view.aim_x = d.x;
            view.aim_y = d.y;
            view.aim_z = d.z;
        }
        for binding in &self.loaded.config.keys {
            let path = self.path("keys", &binding.key);
            record(s, &path, binding, &mut ops)?;
            view.keys.push(path);
        }
        for (id, spell) in &self.loaded.spells {
            let path = self.path("spells", id);
            record(
                s,
                &path,
                &PlayerSpell {
                    id: id.clone(),
                    title: spell.definition().title().into(),
                    revision: spell.revision().into(),
                    allocation: spell.allocation().get(),
                    selected: id == &self.selected,
                },
                &mut ops,
            )?;
            view.spells.push(path);
        }
        for w in &self.loaded.config.supply.wands {
            view.wands.push(self.inventory.paths().item(&w.item));
        }
        for b in self.player.world().bodies() {
            let path = self.path("bodies", &b.id);
            record(s, &path, &b, &mut ops)?;
            if self.loaded.config.bodies.iter().any(|spec| {
                spec.id == b.id && spec.role == component_spellcraft::player_world::Role::Target
            }) {
                view.targets.push(path.clone());
            }
            view.bodies.push(path);
        }
        record(
            s,
            &self.revision_review_path(),
            self.revision.view(),
            &mut ops,
        )?;
        record(s, &self.dashboard_path(), &view, &mut ops)?;
        s.batch(ops, Origin::Host)?;
        Ok(())
    }
}
#[derive(Default)]
pub struct Registry {
    pub sessions: BTreeMap<String, Session>,
}
impl Registry {
    pub fn frame(
        &mut self,
        s: &Store,
        config: &Path,
        name: &str,
        now: f64,
    ) -> Result<(String, String), String> {
        self.frame_input(s, config, name, now, &BTreeSet::new())
    }
    pub fn frame_input(
        &mut self,
        s: &Store,
        config: &Path,
        name: &str,
        now: f64,
        pressed: &BTreeSet<String>,
    ) -> Result<(String, String), String> {
        let path = canonical(config)?;
        let key = identity(&path, name)?;
        if !self.sessions.contains_key(&key) {
            self.sessions
                .insert(key.clone(), Session::new(path, name.into())?);
        }
        let session = self.sessions.get_mut(&key).unwrap();
        let view = session.frame_input(s, now, pressed)?;
        Ok((key, view))
    }
}
