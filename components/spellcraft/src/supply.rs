//! Finite cartridge contents compose Inventory custody and Equipment rules. There is no clock,
//! recharge, implicit item creation, or second location/owner registry here.
use atelico_equipment::Equipment;
use atelico_inventory::{
    DefinitionId, EntityId, Holder, Inventory, ItemId, Location, Mutation, SlotId, Transfer,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Existing spellcraft energy units, without a new price or capacity policy.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Energy(f64);
impl Energy {
    pub fn new(value: f64) -> Result<Self, String> {
        if !value.is_finite() || value < 0.0 {
            return Err("energy must be finite and nonnegative".into());
        }
        Ok(Self(if value == 0.0 { 0.0 } else { value }))
    }
    pub fn get(self) -> f64 {
        self.0
    }
}
impl<'de> Deserialize<'de> for Energy {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(f64::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cartridge {
    pub item: ItemId,
    pub capacity: Energy,
    pub remaining: Energy,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wand {
    pub item: ItemId,
    pub cartridge_slot: SlotId,
    pub accepts: Vec<DefinitionId>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupplySnapshot {
    pub cartridges: Vec<Cartridge>,
    pub wands: Vec<Wand>,
}

/// Deserialize the DTO, then call `restore` with the same authoritative Inventory snapshot.
/// Private maps prohibit admitting invalid/duplicate/stale item identities through serde.
#[derive(Debug, Clone, Default)]
pub struct Supply {
    cartridges: BTreeMap<ItemId, Cartridge>,
    wands: BTreeMap<ItemId, Wand>,
}
impl Supply {
    pub fn restore(inventory: &Inventory, snapshot: SupplySnapshot) -> Result<Self, String> {
        let mut this = Self::default();
        for cartridge in snapshot.cartridges {
            if this
                .cartridges
                .insert(cartridge.item.clone(), cartridge)
                .is_some()
            {
                return Err("duplicate cartridge item".into());
            }
        }
        for wand in snapshot.wands {
            if this.wands.insert(wand.item.clone(), wand).is_some() {
                return Err("duplicate wand item".into());
            }
        }
        this.validate(inventory)?;
        Ok(this)
    }
    pub fn snapshot(&self) -> SupplySnapshot {
        SupplySnapshot {
            cartridges: self.cartridges.values().cloned().collect(),
            wands: self.wands.values().cloned().collect(),
        }
    }
    pub fn cartridges(&self) -> impl Iterator<Item = &Cartridge> {
        self.cartridges.values()
    }
    pub fn cartridge(&self, item: &ItemId) -> Option<&Cartridge> {
        self.cartridges.get(item)
    }
    pub fn wand(&self, item: &ItemId) -> Option<&Wand> {
        self.wands.get(item)
    }
    pub fn validate(&self, inventory: &Inventory) -> Result<(), String> {
        for c in self.cartridges.values() {
            if inventory.item(&c.item).is_none() {
                return Err(format!("cartridge {} has no live Inventory item", c.item));
            }
            if c.remaining > c.capacity {
                return Err(format!("cartridge {} exceeds capacity", c.item));
            }
        }
        for w in self.wands.values() {
            if inventory.item(&w.item).is_none() {
                return Err(format!("wand {} has no live Inventory item", w.item));
            }
            if self.cartridges.contains_key(&w.item) {
                return Err("an item cannot be both a wand and its energy cartridge".into());
            }
            if w.accepts.is_empty()
                || w.accepts.iter().collect::<BTreeSet<_>>().len() != w.accepts.len()
            {
                return Err(format!(
                    "wand {} needs a nonempty unique cartridge definition allowlist",
                    w.item
                ));
            }
            if let Some(item) = self.loaded(inventory, &w.item)? {
                self.accepts(inventory, w, item)?;
            }
        }
        Ok(())
    }
    fn accepts(&self, inventory: &Inventory, wand: &Wand, item: &ItemId) -> Result<(), String> {
        let c = self
            .cartridge(item)
            .ok_or_else(|| format!("{} is not a registered cartridge", item))?;
        let record = inventory
            .item(&c.item)
            .ok_or("cartridge Inventory item is missing")?;
        if !wand.accepts.contains(&record.item.definition) {
            return Err(format!(
                "{} cannot load cartridge definition {}",
                wand.item, record.item.definition
            ));
        }
        Ok(())
    }
    pub fn location(&self, wand: &ItemId) -> Result<Location, String> {
        let w = self
            .wand(wand)
            .ok_or("equipped item is not a configured wand")?;
        Ok(Location::Slot {
            holder: Holder::Item(w.item.clone()),
            slot: w.cartridge_slot.clone(),
        })
    }
    pub fn loaded<'a>(
        &self,
        inventory: &'a Inventory,
        wand: &ItemId,
    ) -> Result<Option<&'a ItemId>, String> {
        let location = self.location(wand)?;
        Ok(inventory
            .items()
            .find(|r| r.location == location)
            .map(|r| &r.item.id))
    }
    pub fn equipped_wand<'a>(
        &self,
        inventory: &'a Inventory,
        equipment: &Equipment,
        slot: &SlotId,
    ) -> Result<Option<&'a ItemId>, String> {
        equipment.validate(inventory).map_err(|e| e.to_string())?;
        let item = equipment
            .equipped(inventory, slot)
            .map_err(|e| e.to_string())?;
        if let Some(item) = item {
            self.wand(item)
                .ok_or("equipped item is not a configured wand")?;
        }
        Ok(item)
    }
    pub fn owned<'a>(
        &'a self,
        inventory: &'a Inventory,
        owner: &EntityId,
    ) -> Result<Vec<&'a Cartridge>, String> {
        self.validate(inventory)?;
        // Inventory's acyclic authoritative custody traversal yields every instance exactly once.
        Ok(inventory
            .descendants(&Holder::Entity(owner.clone()))
            .filter_map(|r| self.cartridge(&r.item.id))
            .collect())
    }
    pub fn total(&self, inventory: &Inventory, owner: &EntityId) -> Result<Energy, String> {
        Energy::new(
            self.owned(inventory, owner)?
                .into_iter()
                .map(|c| c.remaining.get())
                .sum(),
        )
    }
    /// Replaces the current cartridge in one Inventory transaction. The old cartridge needs an
    /// explicit same-owner grid destination; full grids or stale expected sources roll back both moves.
    pub fn load(
        &self,
        inventory: &mut Inventory,
        equipment: &Equipment,
        equipment_slot: &SlotId,
        expected_wand: &ItemId,
        item: &ItemId,
        expected_source: &Location,
        expected_loaded: Option<&ItemId>,
        return_to: Option<Location>,
    ) -> Result<Mutation, String> {
        self.validate(inventory)?;
        if self.equipped_wand(inventory, equipment, equipment_slot)? != Some(expected_wand) {
            return Err("equipped wand changed".into());
        }
        if self.loaded(inventory, expected_wand)? != expected_loaded {
            return Err("loaded cartridge changed".into());
        }
        if inventory.owner_of(item) != Some(equipment.owner()) {
            return Err("cartridge belongs to another owner".into());
        }
        let wand = self.wand(expected_wand).expect("checked wand");
        self.accepts(inventory, wand, item)?;
        if expected_loaded == Some(item) {
            return Err("cartridge is already loaded".into());
        }
        let mut moves = vec![Transfer {
            item: item.clone(),
            from: expected_source.clone(),
            to: self.location(expected_wand)?,
        }];
        match (expected_loaded, return_to) {
            (Some(old), Some(to)) => {
                self.return_grid(inventory, equipment.owner(), &to)?;
                moves.push(Transfer {
                    item: old.clone(),
                    from: self.location(expected_wand)?,
                    to,
                });
            }
            (None, None) => {}
            _ => {
                return Err(
                    "replacement requires exactly one explicit old-cartridge return destination"
                        .into(),
                )
            }
        }
        inventory.transfer_all(&moves).map_err(|e| e.to_string())
    }
    pub fn unload(
        &self,
        inventory: &mut Inventory,
        equipment: &Equipment,
        equipment_slot: &SlotId,
        expected_wand: &ItemId,
        expected_cartridge: &ItemId,
        destination: Location,
    ) -> Result<Mutation, String> {
        self.validate(inventory)?;
        if self.equipped_wand(inventory, equipment, equipment_slot)? != Some(expected_wand) {
            return Err("equipped wand changed".into());
        }
        if self.loaded(inventory, expected_wand)? != Some(expected_cartridge) {
            return Err("loaded cartridge changed".into());
        }
        self.return_grid(inventory, equipment.owner(), &destination)?;
        inventory
            .transfer(
                expected_cartridge,
                &self.location(expected_wand)?,
                destination,
            )
            .map_err(|e| e.to_string())
    }
    fn return_grid(
        &self,
        inventory: &Inventory,
        owner: &EntityId,
        destination: &Location,
    ) -> Result<(), String> {
        let Location::Grid { grid, .. } = destination else {
            return Err("cartridge return destination must be a carried grid".into());
        };
        let grid = inventory.grid(grid).ok_or("return grid does not exist")?;
        if inventory.owner_of_holder(&grid.holder).as_ref() != Some(owner) {
            return Err("return grid belongs to another owner".into());
        }
        Ok(())
    }
    pub(crate) fn debit(
        &mut self,
        item: &ItemId,
        amount: Energy,
    ) -> Result<(Energy, Energy), String> {
        let c = self
            .cartridges
            .get_mut(item)
            .ok_or("source is not a registered cartridge")?;
        if amount > c.remaining {
            return Err("loaded cartridge cannot fund the selected spell allocation".into());
        }
        let before = c.remaining;
        c.remaining = Energy::new(before.get() - amount.get())?;
        Ok((before, c.remaining))
    }
}
