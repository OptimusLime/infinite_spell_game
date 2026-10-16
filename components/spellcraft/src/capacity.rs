//! Paid grid expansion over the generic Inventory and shared funding Account. There is no Player,
//! spell execution, automatic billing, depletion policy, spill, deletion or recharge in this module.
use crate::{
    funding::{
        Account, AccountSnapshot, DebitKey, DebitPlan, DebitPurpose, DebitReceipt, DebitRequest,
    },
    supply::Energy,
};
use atelico_inventory::{
    ChangeSet, EntityId, GridId, Inventory, ItemId, Location, Orientation, Size,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub item: ItemId,
    pub grid: GridId,
    pub baseline: Size,
    pub expanded: Size,
    pub activation_cost: Energy,
    pub renewal_cost: Energy,
    pub cadence: u64,
}
impl Spec {
    pub fn revision(&self) -> String {
        fingerprint(self)
    }
    fn validate(&self) -> Result<(), Error> {
        if self.cadence == 0
            || self.expanded.width < self.baseline.width
            || self.expanded.height < self.baseline.height
            || self.expanded == self.baseline
        {
            return Err(Error::Invalid("Expansion must grow at least one dimension without shrinking another, with an explicit positive cadence.".into()));
        }
        Ok(())
    }
}
fn fingerprint<T: Serialize>(value: &T) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("finite typed capacity payload"))
    )
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Activate,
    Renew,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub id: String,
    pub operation: Operation,
    pub item: ItemId,
    pub owner: EntityId,
    pub expected_config: String,
    pub expected_location: Location,
    pub expected_size: Size,
    pub expected_revision: u64,
    pub expected_paid_until: u64,
    pub source: ItemId,
    pub expected_remaining: Energy,
    pub tick: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub item: ItemId,
    pub grid: GridId,
    pub owner: EntityId,
    pub revision: u64,
    pub activated_at: u64,
    pub last_paid_at: u64,
    pub paid_until: u64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub request: Request,
    pub grant: Grant,
    pub debit: DebitKey,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    pub specs: Vec<Spec>,
    pub grants: Vec<Grant>,
    pub receipts: Vec<Receipt>,
}
/// Persist this one aggregate, or a caller-owned atomic journal entry, before committing prepared
/// state. Three independent files and Store+filesystem writes are not a cross-system transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateSnapshot {
    pub inventory: atelico_inventory::Snapshot,
    pub funding: AccountSnapshot,
    pub capacity: Snapshot,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRequired {
    pub item: ItemId,
    pub grid: GridId,
    pub paid_until: u64,
    pub baseline: Size,
    pub expanded: Size,
    pub overflow_items: Vec<ItemId>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Paid(Grant),
    PolicyRequired(PolicyRequired),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Invalid(String),
    Funding {
        required: Energy,
        available: Energy,
        policy: Option<PolicyRequired>,
    },
    PolicyRequired(PolicyRequired),
    Publication(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(s)|Self::Publication(s)=>f.write_str(s),
            Self::Funding{required,available,..}=>write!(f,"Capacity needs {} energy; source has {}",required.get(),available.get()),
            Self::PolicyRequired(p)=>write!(f,"Grid {} reached its paid boundary {}; the caller must resolve its expired-space policy",p.grid,p.paid_until),
        }
    }
}
impl std::error::Error for Error {}
#[derive(Debug)]
pub struct Prepared {
    basis: Inventory,
    candidate: Inventory,
    state_revision: usize,
    debit: DebitPlan,
    receipt: Receipt,
    changes: ChangeSet,
    replayed: bool,
}
impl Prepared {
    pub fn inventory(&self) -> &Inventory {
        &self.candidate
    }
    pub fn changes(&self) -> &ChangeSet {
        &self.changes
    }
    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }
    pub fn debit(&self) -> &DebitReceipt {
        self.debit.receipt()
    }
    pub fn replayed(&self) -> bool {
        self.replayed
    }
}
#[derive(Debug)]
pub struct Outcome {
    pub receipt: Receipt,
    pub changes: ChangeSet,
    pub replayed: bool,
}
#[derive(Debug)]
pub struct Capacity {
    specs: BTreeMap<ItemId, Spec>,
    grants: BTreeMap<ItemId, Grant>,
    receipts: Vec<Receipt>,
    requests: BTreeMap<String, usize>,
}
impl Capacity {
    pub fn new(inventory: &Inventory, specs: Vec<Spec>) -> Result<Self, Error> {
        let mut this = Self {
            specs: BTreeMap::new(),
            grants: BTreeMap::new(),
            receipts: Vec::new(),
            requests: BTreeMap::new(),
        };
        let mut grids = BTreeSet::new();
        for spec in specs {
            spec.validate()?;
            if inventory.item(&spec.item).is_none() || inventory.grid(&spec.grid).is_none() {
                return Err(Error::Invalid(
                    "Expander and target grid must be live Inventory identities.".into(),
                ));
            }
            if !grids.insert(spec.grid.clone())
                || this.specs.insert(spec.item.clone(), spec).is_some()
            {
                return Err(Error::Invalid("Each physical expander and target grid needs one explicit capacity configuration; stacking is not configured.".into()));
            }
        }
        Ok(this)
    }
    pub fn specs(&self) -> impl Iterator<Item = &Spec> {
        self.specs.values()
    }
    pub fn spec(&self, item: &ItemId) -> Option<&Spec> {
        self.specs.get(item)
    }
    pub fn grant(&self, item: &ItemId) -> Option<&Grant> {
        self.grants.get(item)
    }
    pub fn receipts(&self) -> &[Receipt] {
        &self.receipts
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            version: 1,
            specs: self.specs.values().cloned().collect(),
            grants: self.grants.values().cloned().collect(),
            receipts: self.receipts.clone(),
        }
    }
    pub fn validate(&self, inventory: &Inventory) -> Result<(), Error> {
        inventory
            .validate()
            .map_err(|e| Error::Invalid(e.to_string()))?;
        for spec in self.specs.values() {
            if inventory.item(&spec.item).is_none() || inventory.grid(&spec.grid).is_none() {
                return Err(Error::Invalid(
                    "Capacity configuration has a missing or retired Inventory identity.".into(),
                ));
            }
        }
        for grant in self.grants.values() {
            let spec = &self.specs[&grant.item];
            self.owner(inventory, spec, &grant.owner)?;
            if inventory.grid(&grant.grid).map(|g| g.size) != Some(spec.expanded) {
                return Err(Error::Invalid(
                    "An active capacity grant's grid dimensions changed outside its transaction."
                        .into(),
                ));
            }
        }
        Ok(())
    }
    fn owner(&self, inventory: &Inventory, spec: &Spec, owner: &EntityId) -> Result<(), Error> {
        let grid = inventory
            .grid(&spec.grid)
            .ok_or_else(|| Error::Invalid("Target grid is missing.".into()))?;
        if inventory.owner_of(&spec.item) != Some(owner)
            || inventory.owner_of_holder(&grid.holder).as_ref() != Some(owner)
        {
            return Err(Error::Invalid(
                "Expander and target grid must belong to the requested owner.".into(),
            ));
        }
        Ok(())
    }
    fn policy(&self, inventory: &Inventory, grant: &Grant) -> PolicyRequired {
        let spec = &self.specs[&grant.item];
        let overflow_items = inventory
            .items_in_grid(&grant.grid)
            .filter_map(|r| {
                let Location::Grid {
                    column,
                    row,
                    orientation,
                    ..
                } = &r.location
                else {
                    return None;
                };
                let size = r.item.footprint.size;
                let (w, h) = match orientation {
                    Orientation::Normal => (size.width.get(), size.height.get()),
                    Orientation::Rotated => (size.height.get(), size.width.get()),
                };
                (u32::from(*column) + u32::from(w) > u32::from(spec.baseline.width.get())
                    || u32::from(*row) + u32::from(h) > u32::from(spec.baseline.height.get()))
                .then(|| r.item.id.clone())
            })
            .collect();
        PolicyRequired {
            item: grant.item.clone(),
            grid: grant.grid.clone(),
            paid_until: grant.paid_until,
            baseline: spec.baseline,
            expanded: spec.expanded,
            overflow_items,
        }
    }
    /// Read-only status. Past paid_until does not infer whether occupied space may be used, collapsed,
    /// or spilled. A caller cannot treat PolicyRequired as an active paid grant.
    pub fn status(
        &self,
        inventory: &Inventory,
        item: &ItemId,
        tick: u64,
    ) -> Result<Option<Status>, Error> {
        self.validate(inventory)?;
        if self.grants.get(item).is_some_and(|g| tick < g.last_paid_at) {
            return Err(Error::Invalid(
                "Caller tick predates the current paid interval.".into(),
            ));
        }
        Ok(self.grants.get(item).map(|g| {
            if tick >= g.paid_until {
                Status::PolicyRequired(self.policy(inventory, g))
            } else {
                Status::Paid(g.clone())
            }
        }))
    }
    fn derived(spec: &Spec, old: Option<&Grant>, r: &Request) -> Result<Grant, Error> {
        if r.id.trim().is_empty()
            || r.id.chars().any(char::is_control)
            || r.expected_config != spec.revision()
        {
            return Err(Error::Invalid(
                "Capacity request needs a fresh valid ID and the displayed configuration revision."
                    .into(),
            ));
        }
        let activated_at=match (r.operation,old) {
            (Operation::Activate,None) if r.expected_revision==0 && r.expected_paid_until==0 && r.expected_size==spec.baseline=>r.tick,
            (Operation::Renew,Some(g)) if r.expected_revision==g.revision && r.expected_paid_until==g.paid_until && r.tick==g.paid_until && r.expected_size==spec.expanded && r.owner==g.owner=>g.activated_at,
            _=>return Err(Error::Invalid("Grant, dimensions or exact renewal boundary changed; inspect the current capacity state.".into())),
        };
        Ok(Grant {
            item: r.item.clone(),
            grid: spec.grid.clone(),
            owner: r.owner.clone(),
            revision: r
                .expected_revision
                .checked_add(1)
                .ok_or_else(|| Error::Invalid("Capacity revision exhausted.".into()))?,
            activated_at,
            last_paid_at: r.tick,
            paid_until: r.tick.checked_add(spec.cadence).ok_or_else(|| {
                Error::Invalid("Capacity interval overflows the caller's tick clock.".into())
            })?,
        })
    }
    fn payment(spec: &Spec, r: &Request) -> DebitRequest {
        DebitRequest {
            key: DebitKey {
                purpose: match r.operation {
                    Operation::Activate => DebitPurpose::CapacityActivation,
                    Operation::Renew => DebitPurpose::CapacityRenewal,
                },
                id: r.id.clone(),
            },
            owner: r.owner.clone(),
            cartridge: r.source.clone(),
            expected_remaining: r.expected_remaining,
            amount: match r.operation {
                Operation::Activate => spec.activation_cost,
                Operation::Renew => spec.renewal_cost,
            },
            payload_revision: fingerprint(&(spec, r)),
        }
    }
    pub fn prepare(
        &self,
        inventory: &Inventory,
        account: &Account,
        r: Request,
    ) -> Result<Prepared, Error> {
        if let Some(&index) = self.requests.get(&r.id) {
            let receipt = &self.receipts[index];
            if receipt.request != r {
                return Err(Error::Invalid(
                    "Capacity request ID was reused with a different payload.".into(),
                ));
            }
            let spec = &self.specs[&r.item];
            let debit = account
                .prepare(inventory, Self::payment(spec, &r))
                .map_err(Error::Invalid)?;
            if !debit.replayed() {
                return Err(Error::Invalid(
                    "Capacity receipt has no matching funding record.".into(),
                ));
            }
            return Ok(Prepared {
                basis: inventory.clone(),
                candidate: inventory.clone(),
                state_revision: self.receipts.len(),
                debit,
                receipt: receipt.clone(),
                changes: ChangeSet::default(),
                replayed: true,
            });
        }
        self.validate(inventory)?;
        account
            .supply()
            .validate(inventory)
            .map_err(Error::Invalid)?;
        let spec = self
            .specs
            .get(&r.item)
            .ok_or_else(|| Error::Invalid("Unknown physical expander configuration.".into()))?;
        self.owner(inventory, spec, &r.owner)?;
        if inventory.item(&r.item).map(|v| &v.location) != Some(&r.expected_location) {
            return Err(Error::Invalid(
                "Expander moved after the request was prepared.".into(),
            ));
        }
        let old = self.grants.get(&r.item);
        if r.operation == Operation::Renew && old.is_some_and(|g| r.tick > g.paid_until) {
            return Err(Error::PolicyRequired(self.policy(inventory, old.unwrap())));
        }
        let grant = Self::derived(spec, old, &r)?;
        let mut candidate = inventory.clone();
        let changes = candidate
            .resize_grid(&spec.grid, r.expected_size, spec.expanded)
            .map_err(|e| Error::Invalid(e.to_string()))?
            .changes;
        let payment = Self::payment(spec, &r);
        let source = account
            .supply()
            .cartridge(&r.source)
            .ok_or_else(|| Error::Invalid("Funding cartridge is not registered.".into()))?;
        // Check stale source/ownership before classifying a shortage as a policy decision.
        if source.remaining != r.expected_remaining
            || inventory.owner_of(&r.source) != Some(&r.owner)
        {
            return Err(Error::Invalid(
                "Funding source owner or balance changed.".into(),
            ));
        }
        if payment.amount > source.remaining {
            return Err(Error::Funding {
                required: payment.amount,
                available: source.remaining,
                policy: old.map(|g| self.policy(inventory, g)),
            });
        }
        let debit = account
            .prepare(inventory, payment)
            .map_err(Error::Invalid)?;
        if debit.replayed() {
            return Err(Error::Invalid(
                "Funding history has no matching capacity receipt.".into(),
            ));
        }
        let receipt = Receipt {
            request: r,
            grant,
            debit: debit.receipt().request.key.clone(),
        };
        Ok(Prepared {
            basis: inventory.clone(),
            candidate,
            state_revision: self.receipts.len(),
            debit,
            receipt,
            changes,
            replayed: false,
        })
    }
    fn verify(&self, inventory: &Inventory, account: &Account, p: &Prepared) -> Result<(), Error> {
        if !p.replayed && (inventory != &p.basis || self.receipts.len() != p.state_revision) {
            return Err(Error::Invalid(
                "Inventory or capacity state changed after preparation.".into(),
            ));
        }
        if p.replayed
            && self
                .requests
                .get(&p.receipt.request.id)
                .map(|&i| &self.receipts[i])
                != Some(&p.receipt)
        {
            return Err(Error::Invalid("Capacity replay history changed.".into()));
        }
        let spec = self
            .specs
            .get(&p.receipt.request.item)
            .ok_or_else(|| Error::Invalid("Prepared expander configuration disappeared.".into()))?;
        if !p.replayed
            && Self::derived(spec, self.grants.get(&spec.item), &p.receipt.request)?
                != p.receipt.grant
        {
            return Err(Error::Invalid(
                "Prepared capacity configuration or grant changed.".into(),
            ));
        }
        account.verify(inventory, &p.debit).map_err(Error::Invalid)
    }
    /// Publication receives only typed prepared values and holds no mutable domain references.
    /// It may issue one checked Store batch. No domain delta is applied if that batch rejects.
    pub fn commit_with(
        &mut self,
        inventory: &mut Inventory,
        account: &mut Account,
        p: Prepared,
        publish: impl FnOnce(&Prepared) -> Result<(), String>,
    ) -> Result<Outcome, Error> {
        self.verify(inventory, account, &p)?;
        if !p.replayed {
            publish(&p).map_err(Error::Publication)?;
            account.apply_verified(p.debit);
            *inventory = p.candidate;
            self.grants
                .insert(p.receipt.grant.item.clone(), p.receipt.grant.clone());
            self.requests
                .insert(p.receipt.request.id.clone(), self.receipts.len());
            self.receipts.push(p.receipt.clone());
        }
        Ok(Outcome {
            receipt: p.receipt,
            changes: p.changes,
            replayed: p.replayed,
        })
    }
    pub fn commit(
        &mut self,
        inventory: &mut Inventory,
        account: &mut Account,
        p: Prepared,
    ) -> Result<Outcome, Error> {
        self.commit_with(inventory, account, p, |_| Ok(()))
    }
    pub fn candidate_snapshot(
        &self,
        inventory: &Inventory,
        account: &Account,
        p: &Prepared,
    ) -> Result<CandidateSnapshot, Error> {
        self.verify(inventory, account, p)?;
        let mut capacity = self.snapshot();
        if !p.replayed {
            capacity.grants.retain(|g| g.item != p.receipt.grant.item);
            capacity.grants.push(p.receipt.grant.clone());
            capacity.grants.sort_by(|a, b| a.item.cmp(&b.item));
            capacity.receipts.push(p.receipt.clone());
        }
        Ok(CandidateSnapshot {
            inventory: if p.replayed {
                inventory.snapshot()
            } else {
                p.candidate.snapshot()
            },
            funding: account
                .candidate_snapshot(inventory, &p.debit)
                .map_err(Error::Invalid)?,
            capacity,
        })
    }
    pub fn restore(
        inventory: &Inventory,
        account: &Account,
        snapshot: Snapshot,
    ) -> Result<Self, Error> {
        if snapshot.version != 1 {
            return Err(Error::Invalid(
                "Unsupported capacity snapshot version.".into(),
            ));
        }
        let mut this = Self::new(inventory, snapshot.specs)?;
        for receipt in snapshot.receipts {
            let spec = this
                .specs
                .get(&receipt.request.item)
                .ok_or_else(|| Error::Invalid("Receipt expander has no configuration.".into()))?;
            let expected = Self::derived(spec, this.grants.get(&spec.item), &receipt.request)?;
            let payment = Self::payment(spec, &receipt.request);
            if receipt.grant != expected
                || receipt.debit != payment.key
                || account.receipt(&receipt.debit).map(|r| &r.request) != Some(&payment)
            {
                return Err(Error::Invalid("Capacity receipt does not match its exact configuration, grant or funding history.".into()));
            }
            if this
                .requests
                .insert(receipt.request.id.clone(), this.receipts.len())
                .is_some()
            {
                return Err(Error::Invalid("Duplicate capacity request ID.".into()));
            }
            this.grants.insert(expected.item.clone(), expected);
            this.receipts.push(receipt);
        }
        let mut grants = BTreeMap::new();
        for grant in snapshot.grants {
            if grants.insert(grant.item.clone(), grant).is_some() {
                return Err(Error::Invalid("Duplicate capacity grant.".into()));
            }
        }
        if grants != this.grants {
            return Err(Error::Invalid(
                "Capacity grants disagree with their history.".into(),
            ));
        }
        if account
            .journal()
            .iter()
            .filter(|r| {
                matches!(
                    r.request.key.purpose,
                    DebitPurpose::CapacityActivation | DebitPurpose::CapacityRenewal
                )
            })
            .count()
            != this.receipts.len()
        {
            return Err(Error::Invalid(
                "Funding history contains an unrecorded capacity payment.".into(),
            ));
        }
        this.validate(inventory)?;
        Ok(this)
    }
    pub fn restore_candidate(
        snapshot: CandidateSnapshot,
    ) -> Result<(Inventory, Account, Self), Error> {
        let inventory =
            Inventory::try_from(snapshot.inventory).map_err(|e| Error::Invalid(e.to_string()))?;
        let funding = Account::restore(&inventory, snapshot.funding).map_err(Error::Invalid)?;
        let capacity = Self::restore(&inventory, &funding, snapshot.capacity)?;
        Ok((inventory, funding, capacity))
    }
}
