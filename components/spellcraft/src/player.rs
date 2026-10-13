//! Small caller-owned player aggregate. Input transport and scene drawing remain outside it.
use crate::{
    capacity::{self, Capacity},
    casting::{AuthoredSpell, CastReceipt, CastRequest, Casting, CastingSnapshot},
    lifecycle::{self, TerminalCast, TerminalStatus},
    player_world::PlayerWorld,
    runtime::{self, Instance},
};
use atelico_equipment::Equipment;
use atelico_inventory::{Inventory, ItemId, Location, Mutation, SlotId};
pub struct Player {
    inventory: Inventory,
    equipment: Equipment,
    casting: Casting,
    world: PlayerWorld,
    equipment_slot: SlotId,
    selected: Option<AuthoredSpell>,
    instances: Vec<Instance>,
    finished: Vec<TerminalCast>,
    capacity: Option<Capacity>,
    capacity_tick: Option<u64>,
}
impl Player {
    pub fn new(
        inventory: Inventory,
        equipment: Equipment,
        casting: Casting,
        world: PlayerWorld,
        equipment_slot: SlotId,
    ) -> Result<Self, String> {
        equipment.slot(&equipment_slot).map_err(|e| e.to_string())?;
        equipment.validate(&inventory).map_err(|e| e.to_string())?;
        casting.supply().validate(&inventory)?;
        Ok(Self {
            inventory,
            equipment,
            casting,
            world,
            equipment_slot,
            selected: None,
            instances: Vec::new(),
            finished: Vec::new(),
            capacity: None,
            capacity_tick: None,
        })
    }
    /// Explicit caller opt-in. The native adapter permits this only for labeled engineering fixtures
    /// until a gameplay depletion policy has been selected. There are no default specs or payments.
    pub fn configure_capacity_fixture(&mut self, specs: Vec<capacity::Spec>) -> Result<(), String> {
        if self.capacity.is_some() || !self.casting.funding().journal().is_empty() {
            return Err("Capacity configuration requires a fresh explicit fixture session".into());
        }
        self.capacity = Some(Capacity::new(&self.inventory, specs).map_err(|e| e.to_string())?);
        Ok(())
    }
    pub fn capacity(&self) -> Option<&Capacity> {
        self.capacity.as_ref()
    }
    /// The caller must observe an advancing world clock. None locks fixture admission; it does not
    /// substitute tick zero, refresh a grant, or choose an offline/depleted-space rule.
    pub fn set_capacity_clock(&mut self, tick: Option<u64>) -> Result<(), String> {
        if tick.is_some_and(|t| t != self.world.tick()) {
            return Err("Capacity clock differs from the authoritative world tick".into());
        }
        self.capacity_tick = tick;
        Ok(())
    }
    pub fn capacity_clock_running(&self) -> bool {
        self.capacity_tick == Some(self.world.tick())
    }
    fn capacity_clock(&self) -> Result<(), String> {
        if !self.capacity_clock_running() {
            return Err(
                "Capacity fixture clock stopped or unavailable; commands are locked".into(),
            );
        }
        Ok(())
    }
    pub fn capacity_admission(&self) -> Result<(), String> {
        let Some(capacity) = &self.capacity else {
            return Ok(());
        };
        for spec in capacity.specs() {
            match capacity.status(&self.inventory, &spec.item, self.world.tick()).map_err(|e| e.to_string())? {
                Some(capacity::Status::PolicyRequired(_)) => return Err("Capacity policy required: paid space expired; fixture commands are locked without moving or deleting contents".into()),
                Some(capacity::Status::Paid(_)) => self.capacity_clock()?,
                None => {}
            }
        }
        Ok(())
    }
    pub fn validate_inventory_candidate(&self, inventory: &Inventory) -> Result<(), String> {
        self.capacity_admission()?;
        self.equipment
            .validate(inventory)
            .map_err(|e| e.to_string())?;
        self.casting.supply().validate(inventory)?;
        if let Some(capacity) = &self.capacity {
            capacity.validate(inventory).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn prepare_capacity(
        &self,
        request: capacity::Request,
    ) -> Result<capacity::Prepared, String> {
        let capacity = self.capacity.as_ref().ok_or("Capacity is disabled")?;
        // Exact historical replay is read-only even after the clock stops. Changed replay payloads
        // are still rejected by Capacity, and never regain permission to spend.
        let replay = capacity.receipts().iter().any(|r| r.request == request);
        if !replay {
            self.capacity_clock()?;
            self.capacity_admission()?;
            if request.tick != self.world.tick() || &request.owner != self.equipment.owner() {
                return Err("Capacity request has a stale world tick or different owner".into());
            }
        }
        capacity
            .prepare(&self.inventory, self.casting.funding(), request)
            .map_err(|e| e.to_string())
    }
    pub fn commit_capacity_with(
        &mut self,
        prepared: capacity::Prepared,
        publish: impl FnOnce(&capacity::Prepared) -> Result<(), String>,
    ) -> Result<capacity::Outcome, String> {
        if !prepared.replayed() {
            self.capacity_clock()?;
            self.capacity_admission()?;
            if prepared.receipt().request.tick != self.world.tick() {
                return Err("World advanced after capacity preparation".into());
            }
            self.equipment
                .validate(prepared.inventory())
                .map_err(|e| e.to_string())?;
        }
        self.capacity
            .as_mut()
            .ok_or("Capacity is disabled")?
            .commit_with(
                &mut self.inventory,
                self.casting.funding_mut(),
                prepared,
                publish,
            )
            .map_err(|e| e.to_string())
    }
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
    pub fn equipment(&self) -> &Equipment {
        &self.equipment
    }
    pub fn casting(&self) -> &Casting {
        &self.casting
    }
    pub fn world(&self) -> &PlayerWorld {
        &self.world
    }
    pub fn instances(&self) -> &[Instance] {
        &self.instances
    }
    /// Retained terminal summaries, separate from scheduled active instances.
    pub fn finished(&self) -> &[TerminalCast] {
        &self.finished
    }
    pub fn selected(&self) -> Option<&AuthoredSpell> {
        self.selected.as_ref()
    }
    pub fn equipment_slot(&self) -> &SlotId {
        &self.equipment_slot
    }
    pub fn snapshot(&self) -> (atelico_inventory::Snapshot, CastingSnapshot) {
        (self.inventory.snapshot(), self.casting.snapshot())
    }
    /// Explicit selection does not fire and does not replace any already funded instance revision.
    pub fn select(&mut self, spell: AuthoredSpell) -> Result<(), String> {
        self.world.validate_spell(&spell)?;
        self.selected = Some(spell);
        Ok(())
    }
    /// In-memory application boundary. A persistent caller should use Casting::admit + persist +
    /// consuming FundedCast::start directly, alongside its own durable world transaction.
    pub fn fire(&mut self, request: CastRequest) -> Result<CastReceipt, String> {
        if request.equipment_slot != self.equipment_slot {
            return Err("request names another player equipment slot".into());
        }
        // A retry names the original immutable request even if selection or simulation has moved on.
        if let Some(receipt) = self
            .casting
            .receipts()
            .iter()
            .find(|r| r.request.id == request.id)
        {
            let original = AuthoredSpell::new(receipt.source.clone())?;
            return Ok(self
                .casting
                .admit(&self.inventory, &self.equipment, &original, request)?
                .receipt);
        }
        self.capacity_admission()?;
        let selected = self
            .selected
            .as_ref()
            .ok_or("select an authored spell first")?;
        self.world.validate_request(selected, &request)?;
        let admitted = self
            .casting
            .admit(&self.inventory, &self.equipment, selected, request)?;
        if let Some(funded) = admitted.funded {
            self.instances.push(self.world.start_funded(funded));
            self.settle();
        }
        Ok(admitted.receipt)
    }
    pub fn update_actor(&mut self, id: &str, position: [f32; 3]) -> Result<(), String> {
        self.world.update_actor(id, position)
    }
    pub fn advance(&mut self) -> Result<(), String> {
        self.world.advance()?;
        // Expiration may retire the last live form before a due sensor would read it.
        self.settle();
        let tick = self.world.tick();
        for instance in &mut self.instances {
            runtime::tick(instance, &mut self.world, tick);
        }
        self.settle();
        Ok(())
    }
    fn archive(&mut self, instance: Instance, status: TerminalStatus, reason: String) {
        self.world.release_instance(&instance.id);
        self.finished.push(TerminalCast::finish(
            instance,
            status,
            reason,
            self.world.tick(),
        ));
    }
    fn settle(&mut self) {
        for instance in std::mem::take(&mut self.instances) {
            let state = match instance.status {
                runtime::Status::Exhausted => Some((
                    TerminalStatus::Exhausted,
                    instance
                        .why
                        .clone()
                        .unwrap_or_else(|| "Funded runtime exhausted its allocation".into()),
                )),
                runtime::Status::Faulted => Some((
                    TerminalStatus::Faulted,
                    instance
                        .why
                        .clone()
                        .unwrap_or_else(|| "Funded runtime faulted".into()),
                )),
                runtime::Status::Running
                    if !self.world.has_live_forms(&instance.id)
                        && !lifecycle::has_live_work(&instance, &self.world) =>
                {
                    Some((
                        TerminalStatus::Completed,
                        "No owned form or live scheduled work remains".into(),
                    ))
                }
                runtime::Status::Running => None,
            };
            if let Some((status, reason)) = state {
                self.archive(instance, status, reason);
            } else {
                self.instances.push(instance);
            }
        }
    }
    /// Explicit cancellation is revision-checked and idempotent. Already terminal casts keep their result.
    /// It removes owned forms/schedules but never rewrites admission receipts or replenishes a cartridge.
    pub fn cancel(
        &mut self,
        instance: &str,
        expected_revision: &str,
    ) -> Result<&TerminalCast, String> {
        if let Some(n) = self.finished.iter().position(|i| i.id == instance) {
            if self.finished[n].revision != expected_revision {
                return Err("cancel revision changed".into());
            }
            return Ok(&self.finished[n]);
        }
        let n = self
            .instances
            .iter()
            .position(|i| i.id == instance)
            .ok_or("unknown funded instance")?;
        if self.instances[n].revision != expected_revision {
            return Err("cancel revision changed".into());
        }
        let i = self.instances.remove(n);
        self.archive(
            i,
            TerminalStatus::Cancelled,
            "Explicitly cancelled by caller".into(),
        );
        Ok(self.finished.last().expect("archived cancellation"))
    }
    /// Generic inventory UI can submit a checked operation without bypassing cross-system validity.
    pub fn edit_inventory<T>(
        &mut self,
        run: impl FnOnce(&mut Inventory) -> Result<Mutation<T>, atelico_inventory::Error>,
    ) -> Result<Mutation<T>, String> {
        self.capacity_admission()?;
        let mut candidate = self.inventory.clone();
        let result = run(&mut candidate).map_err(|e| e.to_string())?;
        self.validate_inventory_candidate(&candidate)?;
        let changes = candidate.changes_from(&self.inventory);
        self.inventory = candidate;
        Ok(Mutation {
            value: result.value,
            changes,
        })
    }
    pub fn equip(&mut self, item: &ItemId, expected: &Location) -> Result<Mutation, String> {
        self.capacity_admission()?;
        if self.casting.supply().wand(item).is_none() {
            return Err("item is not a configured wand".into());
        }
        self.equipment
            .equip(&mut self.inventory, item, expected, &self.equipment_slot)
            .map_err(|e| e.to_string())
    }
    pub fn load(
        &mut self,
        expected_wand: &ItemId,
        item: &ItemId,
        expected_source: &Location,
        expected_loaded: Option<&ItemId>,
        return_to: Option<Location>,
    ) -> Result<Mutation, String> {
        self.capacity_admission()?;
        self.casting.supply().load(
            &mut self.inventory,
            &self.equipment,
            &self.equipment_slot,
            expected_wand,
            item,
            expected_source,
            expected_loaded,
            return_to,
        )
    }
    pub fn unload(
        &mut self,
        expected_wand: &ItemId,
        expected_cartridge: &ItemId,
        destination: Location,
    ) -> Result<Mutation, String> {
        self.capacity_admission()?;
        self.casting.supply().unload(
            &mut self.inventory,
            &self.equipment,
            &self.equipment_slot,
            expected_wand,
            expected_cartridge,
            destination,
        )
    }
}
