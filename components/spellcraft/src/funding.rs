//! Shared finite funding for independent consumers. Inventory owns identity/custody; Supply owns
//! balances; this account owns the one interleaved debit journal. It never runs a spell or a clock.
use crate::supply::{Energy, Supply, SupplySnapshot};
use atelico_inventory::{EntityId, Inventory, ItemId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Default,
    atelico_data::DataKind,
)]
#[serde(rename_all = "snake_case")]
pub enum DebitPurpose {
    #[default]
    Cast,
    CapacityActivation,
    CapacityRenewal,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebitKey {
    pub purpose: DebitPurpose,
    pub id: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebitRequest {
    pub key: DebitKey,
    pub owner: EntityId,
    pub cartridge: ItemId,
    pub expected_remaining: Energy,
    pub amount: Energy,
    /// Exact consumer payload fingerprint. Consumers retain and validate their own typed payload.
    pub payload_revision: String,
}
impl DebitRequest {
    fn validate(&self) -> Result<(), String> {
        if self.key.id.trim().is_empty() || self.key.id.chars().any(char::is_control) {
            return Err("debit request ID must be nonempty without control characters".into());
        }
        if self.payload_revision.len() != 64
            || !self.payload_revision.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("debit requires an exact consumer payload fingerprint".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebitReceipt {
    pub request: DebitRequest,
    pub before: Energy,
    pub after: Energy,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalSnapshot {
    pub version: u32,
    pub debits: Vec<DebitReceipt>,
}
impl Default for JournalSnapshot {
    fn default() -> Self {
        Self {
            version: 1,
            debits: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountSnapshot {
    pub supply: SupplySnapshot,
    pub journal: JournalSnapshot,
}

/// A small proposed delta: no balances, histories, running instances or brains are cloned to prepare.
#[derive(Debug)]
pub struct DebitPlan {
    receipt: DebitReceipt,
    revision: usize,
    replay: bool,
}
impl DebitPlan {
    pub fn receipt(&self) -> &DebitReceipt {
        &self.receipt
    }
    pub fn replayed(&self) -> bool {
        self.replay
    }
}
#[derive(Debug)]
pub struct DebitOutcome {
    pub receipt: DebitReceipt,
    pub replayed: bool,
}
#[derive(Debug)]
pub struct Account {
    supply: Supply,
    journal: Vec<DebitReceipt>,
    keys: BTreeMap<DebitKey, usize>,
}
impl Account {
    pub fn new(supply: Supply) -> Self {
        Self {
            supply,
            journal: Vec::new(),
            keys: BTreeMap::new(),
        }
    }
    pub fn supply(&self) -> &Supply {
        &self.supply
    }
    pub fn journal(&self) -> &[DebitReceipt] {
        &self.journal
    }
    pub fn receipt(&self, key: &DebitKey) -> Option<&DebitReceipt> {
        self.keys.get(key).map(|&i| &self.journal[i])
    }
    pub fn snapshot(&self) -> AccountSnapshot {
        AccountSnapshot {
            supply: self.supply.snapshot(),
            journal: JournalSnapshot {
                version: 1,
                debits: self.journal.clone(),
            },
        }
    }
    pub fn restore(inventory: &Inventory, snapshot: AccountSnapshot) -> Result<Self, String> {
        if snapshot.journal.version != 1 {
            return Err("unsupported funding journal version".into());
        }
        let mut this = Self::new(Supply::restore(inventory, snapshot.supply)?);
        let mut remaining = BTreeMap::new();
        for receipt in snapshot.journal.debits {
            receipt.request.validate()?;
            let cartridge = this
                .supply
                .cartridge(&receipt.request.cartridge)
                .ok_or("debit source cartridge is missing")?;
            if receipt.before != receipt.request.expected_remaining
                || receipt.before > cartridge.capacity
                || receipt.request.amount > receipt.before
                || (receipt.request.amount.get() > 0.0 && receipt.after >= receipt.before)
                || Energy::new(receipt.before.get() - receipt.request.amount.get())?
                    != receipt.after
            {
                return Err("invalid funding journal arithmetic".into());
            }
            if remaining
                .insert(cartridge.item.clone(), receipt.after)
                .is_some_and(|old| old != receipt.before)
            {
                return Err("funding journal chain is discontinuous".into());
            }
            if this
                .keys
                .insert(receipt.request.key.clone(), this.journal.len())
                .is_some()
            {
                return Err("duplicate funding request ID".into());
            }
            this.journal.push(receipt);
        }
        for (id, remaining) in remaining {
            if this
                .supply
                .cartridge(&id)
                .expect("checked source")
                .remaining
                != remaining
            {
                return Err("cartridge balance disagrees with funding history".into());
            }
        }
        Ok(this)
    }
    pub fn prepare(
        &self,
        inventory: &Inventory,
        request: DebitRequest,
    ) -> Result<DebitPlan, String> {
        request.validate()?;
        if let Some(receipt) = self.receipt(&request.key) {
            if receipt.request != request {
                return Err("debit request ID was reused with a different payload".into());
            }
            return Ok(DebitPlan {
                receipt: receipt.clone(),
                revision: self.journal.len(),
                replay: true,
            });
        }
        self.supply.validate(inventory)?;
        if inventory.owner_of(&request.cartridge) != Some(&request.owner) {
            return Err("funding cartridge belongs to another owner".into());
        }
        let source = self
            .supply
            .cartridge(&request.cartridge)
            .ok_or("source is not a registered cartridge")?;
        if source.remaining != request.expected_remaining {
            return Err("funding cartridge balance changed".into());
        }
        if request.amount > source.remaining {
            return Err("cartridge cannot fund the requested amount".into());
        }
        let before = source.remaining;
        let after = Energy::new(before.get() - request.amount.get())?;
        if request.amount.get() > 0.0 && after >= before {
            return Err("positive debit is too small to reduce the represented balance".into());
        }
        Ok(DebitPlan {
            receipt: DebitReceipt {
                request,
                before,
                after,
            },
            revision: self.journal.len(),
            replay: false,
        })
    }
    pub fn verify(&self, inventory: &Inventory, plan: &DebitPlan) -> Result<(), String> {
        if !plan.replay && self.journal.len() != plan.revision {
            return Err("funding account changed after preparation".into());
        }
        let current = self.prepare(inventory, plan.receipt.request.clone())?;
        if current.receipt != plan.receipt || current.replay != plan.replay {
            return Err("prepared funding changed".into());
        }
        Ok(())
    }
    /// Commit a verified funding delta. Composition callers validate their other domain deltas first.
    pub fn commit(
        &mut self,
        inventory: &Inventory,
        plan: DebitPlan,
    ) -> Result<DebitOutcome, String> {
        self.verify(inventory, &plan)?;
        Ok(self.apply_verified(plan))
    }
    /// Only the checked transaction composer uses this after verification while holding exclusive
    /// account access across its Store publication. The remaining domain commit cannot fail.
    pub(crate) fn apply_verified(&mut self, plan: DebitPlan) -> DebitOutcome {
        if !plan.replay {
            let r = &plan.receipt;
            self.supply
                .debit(&r.request.cartridge, r.request.amount)
                .expect("verified debit under exclusive borrow");
            self.keys.insert(r.request.key.clone(), self.journal.len());
            self.journal.push(r.clone());
        }
        DebitOutcome {
            receipt: plan.receipt,
            replayed: plan.replay,
        }
    }
    /// Explicit durable export, not part of normal preparation. History is copied only when the
    /// caller asks for a complete persistence snapshot, never to resize or pay one request.
    pub fn candidate_snapshot(
        &self,
        inventory: &Inventory,
        plan: &DebitPlan,
    ) -> Result<AccountSnapshot, String> {
        self.verify(inventory, plan)?;
        let mut snapshot = self.snapshot();
        if !plan.replay {
            snapshot
                .supply
                .cartridges
                .iter_mut()
                .find(|c| c.item == plan.receipt.request.cartridge)
                .expect("verified source")
                .remaining = plan.receipt.after;
            snapshot.journal.debits.push(plan.receipt.clone());
        }
        Ok(snapshot)
    }
}
