//! Read-only normalized capacity/grant and shared funding evidence. Inventory owns all item/grid rows.
use crate::{
    capacity::{Capacity, Prepared, Status},
    funding::{Account, DebitPurpose, DebitReceipt},
};
use atelico_data::{Data, DataKind};
use atelico_inventory::{view::Paths, Inventory};
use atelico_store::{Op, Store};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, DataKind)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Inactive,
    Paid,
    PolicyRequired,
    ClockStopped,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct CapacityView {
    pub fixture: bool,
    pub clock_running: bool,
    pub tick: u64,
    pub expanders: Vec<String>,
    pub debits: Vec<String>,
    pub note: String,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct ExpanderView {
    pub item: String,
    pub item_id: String,
    pub grid: String,
    pub grid_id: String,
    pub configuration: String,
    pub baseline_width: u16,
    pub baseline_height: u16,
    pub expanded_width: u16,
    pub expanded_height: u16,
    pub activation_cost: f64,
    pub renewal_cost: f64,
    pub cadence: u64,
    #[data(choice)]
    pub phase: Phase,
    pub can_activate: bool,
    pub revision: u64,
    pub paid_until: u64,
    pub overflow_items: Vec<String>,
    pub reason: String,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Data)]
#[serde(deny_unknown_fields)]
pub struct FundingDebitView {
    pub id: String,
    #[data(choice)]
    pub purpose: DebitPurpose,
    pub cartridge: String,
    pub cartridge_id: String,
    pub before: f64,
    pub after: f64,
    pub amount: f64,
    pub payload_revision: String,
}
pub struct Projection {
    pub dashboard: CapacityView,
    pub expanders: BTreeMap<String, ExpanderView>,
    pub debits: BTreeMap<String, FundingDebitView>,
}
#[derive(Clone)]
pub struct Binding {
    root: Paths,
    inventory: Paths,
}
impl Binding {
    pub fn new(root: impl Into<String>, inventory: Paths) -> Result<Self, String> {
        let root = Paths::new(root)?;
        if atelico_store::overlaps(root.root(), inventory.root()) {
            return Err("Capacity and Inventory paths overlap".into());
        }
        Ok(Self { root, inventory })
    }
    pub fn dashboard(&self) -> String {
        self.root.dashboard()
    }
    fn path(&self, kind: &str, id: &str) -> String {
        format!(
            "{}.{}.{}",
            self.root.root(),
            kind,
            id.as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
    }
    pub fn install(&self, s: &Store) -> Result<(), String> {
        for (path, schema) in [
            (self.dashboard(), CapacityView::schema()),
            (
                format!("{}.expanders.*", self.root.root()),
                ExpanderView::schema(),
            ),
            (
                format!("{}.debits.*", self.root.root()),
                FundingDebitView::schema(),
            ),
        ] {
            let name = schema.name.clone();
            let mut errors = s.define(schema);
            errors.extend(s.bind(&path, &name));
            if !errors.is_empty() {
                return Err(errors.join("; "));
            }
        }
        Ok(())
    }
    pub fn project(
        &self,
        capacity: &Capacity,
        inventory: &Inventory,
        account: &Account,
        tick: u64,
        clock_running: bool,
        pending: Option<&Prepared>,
    ) -> Result<Projection, String> {
        let inventory = pending.map_or(inventory, |p| p.inventory());
        let mut p=Projection{dashboard:CapacityView{fixture:true,clock_running,tick,note:"Engineering fixture only. Prices and dimensions are unreviewed. Expiry stops capacity-dependent commands pending the no-power decision.".into(),..Default::default()},expanders:BTreeMap::new(),debits:BTreeMap::new()};
        for spec in capacity.specs() {
            let new_grant = pending
                .filter(|p| p.receipt().grant.item == spec.item)
                .map(|p| &p.receipt().grant);
            let grant = new_grant.or_else(|| capacity.grant(&spec.item));
            let mut row = ExpanderView {
                item: self.inventory.item(&spec.item),
                item_id: spec.item.as_str().into(),
                grid: self.inventory.grid(&spec.grid),
                grid_id: spec.grid.as_str().into(),
                configuration: spec.revision(),
                baseline_width: spec.baseline.width.get(),
                baseline_height: spec.baseline.height.get(),
                expanded_width: spec.expanded.width.get(),
                expanded_height: spec.expanded.height.get(),
                activation_cost: spec.activation_cost.get(),
                renewal_cost: spec.renewal_cost.get(),
                cadence: spec.cadence,
                ..Default::default()
            };
            if let Some(g) = grant {
                row.revision = g.revision;
                row.paid_until = g.paid_until;
                row.phase = Phase::Paid;
            }
            if new_grant.is_none() {
                if let Some(Status::PolicyRequired(policy)) = capacity
                    .status(inventory, &spec.item, tick)
                    .map_err(|e| e.to_string())?
                {
                    row.phase = Phase::PolicyRequired;
                    row.overflow_items = policy
                        .overflow_items
                        .iter()
                        .map(|id| self.inventory.item(id))
                        .collect();
                    row.reason="Policy required: paid space expired. Contents preserved; fixture commands locked.".into();
                }
            }
            if row.phase != Phase::PolicyRequired && !clock_running {
                row.phase = Phase::ClockStopped;
                row.reason = "World clock stopped or unavailable; capacity commands locked.".into();
            }
            row.can_activate = grant.is_none() && clock_running;
            let path = self.path("expanders", spec.item.as_str());
            p.dashboard.expanders.push(path.clone());
            p.expanders.insert(path, row);
        }
        for debit in account.journal() {
            self.debit(&mut p, debit, inventory);
        }
        if let Some(prepared) = pending {
            if !prepared.replayed() {
                self.debit(&mut p, prepared.debit(), inventory);
            }
        }
        Ok(p)
    }
    fn debit(&self, p: &mut Projection, r: &DebitReceipt, inventory: &Inventory) {
        let purpose = match r.request.key.purpose {
            DebitPurpose::Cast => "cast",
            DebitPurpose::CapacityActivation => "capacity_activation",
            DebitPurpose::CapacityRenewal => "capacity_renewal",
        };
        let path = self.path("debits", &format!("{purpose}:{}", r.request.key.id));
        if !p.debits.contains_key(&path) {
            p.dashboard.debits.push(path.clone());
        }
        p.debits.insert(
            path,
            FundingDebitView {
                id: r.request.key.id.clone(),
                purpose: r.request.key.purpose,
                cartridge: inventory
                    .item(&r.request.cartridge)
                    .map(|_| self.inventory.item(&r.request.cartridge))
                    .unwrap_or_default(),
                cartridge_id: r.request.cartridge.as_str().into(),
                before: r.before.get(),
                after: r.after.get(),
                amount: r.request.amount.get(),
                payload_revision: r.request.payload_revision.clone(),
            },
        );
    }
    pub fn operations(&self, s: &Store, p: &Projection) -> Result<Vec<Op>, String> {
        let mut ops = vec![];
        fn row<T: Serialize>(
            s: &Store,
            path: &str,
            row: &T,
            ops: &mut Vec<Op>,
        ) -> Result<(), String> {
            let value = serde_json::to_value(row).map_err(|e| e.to_string())?;
            ops.extend(atelico_store::diff(path, s.get(path).as_ref(), &value));
            Ok(())
        }
        row(s, &self.dashboard(), &p.dashboard, &mut ops)?;
        for (path, v) in &p.expanders {
            row(s, path, v, &mut ops)?;
        }
        for (path, v) in &p.debits {
            row(s, path, v, &mut ops)?;
        }
        Ok(ops)
    }
}
