//! Flat typed HUD records. Rust computes ownership, totals and readiness; Luau reads data.object refs.
use crate::{
    casting::{AuthoredSpell, Casting},
    runtime::{Instance, Status},
};
use atelico_data::Data;
use atelico_equipment::Equipment;
use atelico_inventory::{view::Paths as InventoryPaths, Inventory, ItemId, SlotId};
use atelico_store::{Op, Origin, Store};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct SupplyView {
    pub owner: String,
    pub cartridges: Vec<String>,
    pub total: f64,
    pub wand: String,
    pub loaded: String,
    pub selected_name: String,
    pub selected_title: String,
    pub selected_revision: String,
    pub allocation: f64,
    pub can_cast: bool,
    pub reason: String,
    pub receipts: Vec<String>,
    pub instances: Vec<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct CartridgeView {
    pub item: String,
    pub item_id: String,
    pub definition: String,
    pub capacity: f64,
    pub remaining: f64,
    pub loaded: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct ReceiptView {
    pub request_id: String,
    pub instance: String,
    pub cartridge: String,
    pub cartridge_id: String,
    pub wand_id: String,
    pub revision: String,
    pub allocation: f64,
    pub before: f64,
    pub after: f64,
    pub tick: u64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct CastView {
    pub id: String,
    pub revision: String,
    pub title: String,
    pub status: String,
    pub reason: String,
    pub initial: f64,
    pub remaining: f64,
    pub sensors: f64,
    pub logic: f64,
    pub rules: f64,
    pub effectors: f64,
    pub dissipation: f64,
}
/// Each map contains full safe Store paths, never nested records embedded inside dashboard fields.
pub struct Projection {
    pub dashboard: SupplyView,
    pub cartridges: BTreeMap<String, CartridgeView>,
    pub receipts: BTreeMap<String, ReceiptView>,
    pub instances: BTreeMap<String, CastView>,
}
#[derive(Clone)]
pub struct Binding {
    paths: InventoryPaths,
    inventory: InventoryPaths,
}
impl Binding {
    pub fn new(root: impl Into<String>, inventory: InventoryPaths) -> Result<Self, String> {
        let paths = InventoryPaths::new(root)?;
        if atelico_store::overlaps(paths.root(), inventory.root()) {
            return Err("supply and Inventory namespaces must not overlap".into());
        }
        Ok(Self { paths, inventory })
    }
    pub fn dashboard(&self) -> String {
        self.paths.dashboard()
    }
    fn path(&self, kind: &str, id: &str) -> String {
        format!(
            "{}.{kind}.{}",
            self.paths.root(),
            id.as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
    }
    pub fn cartridge(&self, item: &ItemId) -> String {
        self.path("cartridges", item.as_str())
    }
    pub fn project(
        &self,
        casting: &Casting,
        inventory: &Inventory,
        equipment: &Equipment,
        equipment_slot: &SlotId,
        selected: Option<&AuthoredSpell>,
        instances: &[Instance],
    ) -> Result<Projection, String> {
        let supply = casting.supply();
        supply.validate(inventory)?;
        let wand = supply.equipped_wand(inventory, equipment, equipment_slot)?;
        let loaded = wand
            .map(|id| supply.loaded(inventory, id))
            .transpose()?
            .flatten();
        let mut cartridges = BTreeMap::new();
        for c in supply.owned(inventory, equipment.owner())? {
            cartridges.insert(
                self.cartridge(&c.item),
                CartridgeView {
                    item: self.inventory.item(&c.item),
                    item_id: c.item.as_str().into(),
                    definition: inventory
                        .item(&c.item)
                        .expect("validated item")
                        .item
                        .definition
                        .as_str()
                        .into(),
                    capacity: c.capacity.get(),
                    remaining: c.remaining.get(),
                    loaded: loaded == Some(&c.item),
                },
            );
        }
        let mut dashboard = SupplyView {
            owner: equipment.owner().as_str().into(),
            cartridges: cartridges.keys().cloned().collect(),
            total: supply.total(inventory, equipment.owner())?.get(),
            wand: wand.map(|id| self.inventory.item(id)).unwrap_or_default(),
            loaded: loaded.map(|id| self.cartridge(id)).unwrap_or_default(),
            ..SupplyView::default()
        };
        if let Some(spell) = selected {
            dashboard.selected_name = spell.definition().name.clone();
            dashboard.selected_title = spell.definition().title().into();
            dashboard.selected_revision = spell.revision().into();
            dashboard.allocation = spell.allocation().get();
        }
        dashboard.reason = match (selected, wand, loaded) {
            (None, _, _) => "Select an authored spell revision".into(),
            (_, None, _) => "Equip a wand".into(),
            (_, _, None) => "Load a cartridge".into(),
            (Some(spell), _, Some(id))
                if supply.cartridge(id).expect("validated cartridge").remaining
                    < spell.allocation() =>
            {
                "Loaded cartridge cannot fund the selected allocation".into()
            }
            _ => {
                dashboard.can_cast = true;
                String::new()
            }
        };
        let mut receipts = BTreeMap::new();
        for r in casting
            .receipts()
            .iter()
            .filter(|r| &r.request.owner == equipment.owner())
        {
            let path = self.path("receipts", &r.request.id);
            dashboard.receipts.push(path.clone());
            receipts.insert(
                path,
                ReceiptView {
                    request_id: r.request.id.clone(),
                    instance: self.path("instances", &r.instance),
                    cartridge: cartridges
                        .contains_key(&self.cartridge(&r.request.expected_cartridge))
                        .then(|| self.cartridge(&r.request.expected_cartridge))
                        .unwrap_or_default(),
                    cartridge_id: r.request.expected_cartridge.as_str().into(),
                    wand_id: r.request.expected_wand.as_str().into(),
                    revision: r.request.expected_revision.clone(),
                    allocation: r.allocation.get(),
                    before: r.before.get(),
                    after: r.after.get(),
                    tick: r.request.tick,
                },
            );
        }
        let mut rows = BTreeMap::new();
        for i in instances {
            if !casting
                .receipts()
                .iter()
                .any(|r| r.instance == i.id && &r.request.owner == equipment.owner())
            {
                continue;
            }
            let path = self.path("instances", &i.id);
            rows.insert(
                path.clone(),
                CastView {
                    id: i.id.clone(),
                    revision: i.revision.clone(),
                    title: i.definition.title().into(),
                    status: match i.status {
                        Status::Running => "running",
                        Status::Exhausted => "exhausted",
                        Status::Faulted => "faulted",
                    }
                    .into(),
                    reason: i.why.clone().unwrap_or_default(),
                    initial: i.initial,
                    remaining: i.energy,
                    sensors: *i.spent.get("sensors").unwrap_or(&0.0),
                    logic: *i.spent.get("logic").unwrap_or(&0.0),
                    rules: *i.spent.get("rules").unwrap_or(&0.0),
                    effectors: *i.spent.get("effectors").unwrap_or(&0.0),
                    dissipation: *i.spent.get("dissipation").unwrap_or(&0.0),
                },
            );
            dashboard.instances.push(path);
        }
        // An admission receipt is meaningful even before start or after a runtime instance is retired.
        for r in receipts.values_mut() {
            if !rows.contains_key(&r.instance) {
                r.instance.clear();
            }
        }
        Ok(Projection {
            dashboard,
            cartridges,
            receipts,
            instances: rows,
        })
    }
    pub fn install(&self, store: &Store) -> Result<(), String> {
        if store.has(self.paths.root())
            && store.type_of(&self.dashboard()).as_deref()
                != Some(SupplyView::schema().name.as_str())
        {
            return Err("supply namespace already contains other data".into());
        }
        for (path, schema) in [
            (self.dashboard(), SupplyView::schema()),
            (
                format!("{}.cartridges.*", self.paths.root()),
                CartridgeView::schema(),
            ),
            (
                format!("{}.receipts.*", self.paths.root()),
                ReceiptView::schema(),
            ),
            (
                format!("{}.instances.*", self.paths.root()),
                CastView::schema(),
            ),
        ] {
            let name = schema.name.clone();
            let errors = store.define(schema);
            if !errors.is_empty() {
                return Err(errors.join("; "));
            }
            let errors = store.bind(&path, &name);
            if !errors.is_empty() {
                return Err(errors.join("; "));
            }
        }
        if store.object(&self.dashboard()).is_none() {
            store.set(
                &self.dashboard(),
                serde_json::to_value(SupplyView::default()).map_err(|e| e.to_string())?,
                Origin::Host,
            )?;
        }
        Ok(())
    }
    /// Read-only transport: no cast/load action is triggered by projection or publication.
    /// `operations` can join Inventory/Equipment updates in a caller-owned checked Store batch.
    pub fn operations(&self, store: &Store, projection: &Projection) -> Result<Vec<Op>, String> {
        if store.type_of(&self.dashboard()).as_deref() != Some(SupplyView::schema().name.as_str()) {
            return Err("install supply binding before publication".into());
        }
        let mut ops = Vec::new();
        record(store, &self.dashboard(), &projection.dashboard, &mut ops)?;
        macro_rules! rows {
            ($rows:expr, $ty:ty, $kind:literal) => {{
                let present: BTreeSet<_> = $rows.keys().cloned().collect();
                for (path, row) in $rows {
                    record(store, path, row, &mut ops)?;
                }
                let prefix = format!("{}.{}.", self.paths.root(), $kind);
                for path in store.objects_of(&<$ty>::schema().name) {
                    if path.starts_with(&prefix) && !present.contains(&path) {
                        ops.push(Op::Remove(path));
                    }
                }
            }};
        }
        rows!(&projection.cartridges, CartridgeView, "cartridges");
        rows!(&projection.receipts, ReceiptView, "receipts");
        rows!(&projection.instances, CastView, "instances");
        Ok(ops)
    }
    pub fn publish(&self, store: &Store, projection: &Projection) -> Result<usize, String> {
        Ok(store
            .batch(self.operations(store, projection)?, Origin::Host)?
            .len())
    }
}
fn record<T: Serialize>(
    store: &Store,
    path: &str,
    row: &T,
    ops: &mut Vec<Op>,
) -> Result<(), String> {
    let value = serde_json::to_value(row).map_err(|e| e.to_string())?;
    ops.extend(atelico_store::diff(path, store.get(path).as_ref(), &value));
    Ok(())
}
