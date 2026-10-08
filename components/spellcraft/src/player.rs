//! Small caller-owned player aggregate. Input transport and scene drawing remain outside it.
use crate::{
    casting::{AuthoredSpell, CastReceipt, CastRequest, Casting, CastingSnapshot},
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
        })
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
        }
        Ok(admitted.receipt)
    }
    pub fn update_actor(&mut self, id: &str, position: [f32; 3]) -> Result<(), String> {
        self.world.update_actor(id, position)
    }
    pub fn advance(&mut self) -> Result<(), String> {
        self.world.advance()?;
        let tick = self.world.tick();
        for instance in &mut self.instances {
            runtime::tick(instance, &mut self.world, tick);
        }
        Ok(())
    }
    /// Generic inventory UI can submit a checked operation without bypassing cross-system validity.
    pub fn edit_inventory<T>(
        &mut self,
        run: impl FnOnce(&mut Inventory) -> Result<Mutation<T>, atelico_inventory::Error>,
    ) -> Result<Mutation<T>, String> {
        let mut candidate = self.inventory.clone();
        let result = run(&mut candidate).map_err(|e| e.to_string())?;
        self.equipment
            .validate(&candidate)
            .map_err(|e| e.to_string())?;
        self.casting.supply().validate(&candidate)?;
        let changes = candidate.changes_from(&self.inventory);
        self.inventory = candidate;
        Ok(Mutation {
            value: result.value,
            changes,
        })
    }
    pub fn equip(&mut self, item: &ItemId, expected: &Location) -> Result<Mutation, String> {
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
