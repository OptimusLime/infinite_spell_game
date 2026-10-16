//! Deliberate admission of one exact authored revision from one finite physical cartridge.
//! This state is owned by the caller, never by the clock/file-resettable legacy Town demo.
use crate::{
    definition::{self, Kind, Source, SpellDefinition},
    funding::{
        Account, AccountSnapshot, DebitKey, DebitPurpose, DebitReceipt, DebitRequest,
        JournalSnapshot,
    },
    runtime::{self, Brain, Instance, WorldAdapter},
    supply::{Energy, Supply, SupplySnapshot},
};
use atelico_equipment::Equipment;
use atelico_inventory::{EntityId, Inventory, ItemId, SlotId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredSource {
    pub definition: String,
    pub brain: Option<String>,
}
/// An exact source revision, not a mutable name lookup. Changing either source creates a new revision.
#[derive(Debug, Clone)]
pub struct AuthoredSpell {
    source: AuthoredSource,
    definition: SpellDefinition,
    revision: String,
}
impl AuthoredSpell {
    pub fn new(source: AuthoredSource) -> Result<Self, String> {
        let definition = definition::parse(&source.definition)?;
        if definition.kind != Kind::Spell || definition.mana.source != Source::Wand {
            return Err("player admission requires an authored wand spell".into());
        }
        if definition.steps().next().is_none() {
            return Err("cannot admit an empty spell".into());
        }
        let revision = digest(&serde_json::to_vec(&source).map_err(|e| e.to_string())?);
        let this = Self {
            source,
            definition,
            revision,
        };
        this.brain()?; // Validate now, then instantiate isolated state again for each admission.
        Ok(this)
    }
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn definition(&self) -> &SpellDefinition {
        &self.definition
    }
    pub fn source(&self) -> &AuthoredSource {
        &self.source
    }
    pub fn allocation(&self) -> Energy {
        Energy::new(self.definition.mana.allocation).expect("validated definition")
    }
    fn brain(&self) -> Result<Option<Arc<Brain>>, String> {
        let brain = self
            .source
            .brain
            .as_deref()
            .map(Brain::compile)
            .transpose()?
            .map(Arc::new);
        runtime::validate_brain(&self.definition, brain.as_deref())?;
        Ok(brain)
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn text(value: &str, name: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(format!(
            "{name} must be nonempty without control characters"
        ));
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CastRequest {
    pub aim: crate::aim::Aim,
    /// Unique within this persisted Casting state. Retrying must submit this exact same payload.
    pub id: String,
    pub owner: EntityId,
    pub equipment_slot: SlotId,
    pub expected_wand: ItemId,
    pub expected_cartridge: ItemId,
    pub expected_remaining: Energy,
    pub expected_revision: String,
    /// World body identifier, separate from the physical Inventory owner identity.
    pub holder: String,
    pub tick: u64,
}
impl CastRequest {
    fn validate(&self) -> Result<(), String> {
        text(&self.id, "request id")?;
        text(&self.holder, "world holder")?;
        if self.expected_revision.len() != 64
            || !self
                .expected_revision
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err("invalid authored revision fingerprint".into());
        }
        Ok(())
    }
    fn instance(&self) -> String {
        format!("cast:{}", digest(self.id.as_bytes()))
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CastReceipt {
    pub request: CastRequest,
    pub instance: String,
    pub source: AuthoredSource,
    pub allocation: Energy,
    pub before: Energy,
    pub after: Energy,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CastingSnapshot {
    pub supply: SupplySnapshot,
    pub receipts: Vec<CastReceipt>,
    /// Absent only in legacy cast-only snapshots. New snapshots include the shared debit history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding: Option<JournalSnapshot>,
}

/// This is an admission ledger, not a simulation snapshot. Persist it together with Inventory.
/// Completed/pending runtime instances and the world's own persistence remain the caller's concern.
#[derive(Debug)]
pub struct Casting {
    account: Account,
    receipts: Vec<CastReceipt>,
    requests: BTreeMap<String, usize>,
}
impl Casting {
    pub fn new(supply: Supply) -> Self {
        Self {
            account: Account::new(supply),
            receipts: Vec::new(),
            requests: BTreeMap::new(),
        }
    }
    pub fn restore(inventory: &Inventory, snapshot: CastingSnapshot) -> Result<Self, String> {
        // Validate exact authored history before accepting or migrating any funding journal.
        let supply = Supply::restore(inventory, snapshot.supply.clone())?;
        let mut requests = BTreeMap::new();
        let mut expected = Vec::new();
        for (index, receipt) in snapshot.receipts.iter().enumerate() {
            receipt.request.validate()?;
            let spell = AuthoredSpell::new(receipt.source.clone())?;
            let cartridge = supply
                .cartridge(&receipt.request.expected_cartridge)
                .ok_or("receipt source cartridge is missing")?;
            if receipt.request.expected_revision != spell.revision()
                || receipt.allocation != spell.allocation()
                || receipt.instance != receipt.request.instance()
                || receipt.before != receipt.request.expected_remaining
                || receipt.before > cartridge.capacity
                || receipt.allocation > receipt.before
                || Energy::new(receipt.before.get() - receipt.allocation.get())? != receipt.after
            {
                return Err("invalid cast receipt funding or authored revision".into());
            }
            if requests.insert(receipt.request.id.clone(), index).is_some() {
                return Err("duplicate cast request id in snapshot".into());
            }
            expected.push(DebitReceipt {
                request: funding_request(&receipt.request, &receipt.source, receipt.allocation)?,
                before: receipt.before,
                after: receipt.after,
            });
        }
        let journal = snapshot.funding.unwrap_or_else(|| JournalSnapshot {
            version: 1,
            debits: expected.clone(),
        });
        let account = Account::restore(
            inventory,
            AccountSnapshot {
                supply: snapshot.supply,
                journal,
            },
        )?;
        for receipt in &expected {
            if account.receipt(&receipt.request.key) != Some(receipt) {
                return Err("cast receipt does not match its exact funding history".into());
            }
        }
        if account
            .journal()
            .iter()
            .filter(|r| r.request.key.purpose == DebitPurpose::Cast)
            .count()
            != expected.len()
        {
            return Err("funding history contains an unrecorded cast".into());
        }
        Ok(Self {
            account,
            receipts: snapshot.receipts,
            requests,
        })
    }
    pub fn snapshot(&self) -> CastingSnapshot {
        let account = self.account.snapshot();
        CastingSnapshot {
            supply: account.supply,
            receipts: self.receipts.clone(),
            funding: Some(account.journal),
        }
    }
    /// A capacity/funding transaction does not own this consumer's authored receipts. Include
    /// them in the caller's durable aggregate, validated against the candidate shared account.
    pub fn candidate_snapshot(
        &self,
        inventory: &Inventory,
        account: AccountSnapshot,
    ) -> Result<CastingSnapshot, String> {
        let snapshot = CastingSnapshot {
            supply: account.supply,
            receipts: self.receipts.clone(),
            funding: Some(account.journal),
        };
        Self::restore(inventory, snapshot.clone())?;
        Ok(snapshot)
    }
    pub fn supply(&self) -> &Supply {
        self.account.supply()
    }
    /// Compatibility ownership: independent consumers use Account directly. A player keeping this
    /// existing Casting aggregate can pass its checked account to those consumers without a copy.
    pub fn funding(&self) -> &Account {
        &self.account
    }
    pub fn funding_mut(&mut self) -> &mut Account {
        &mut self.account
    }
    pub fn receipts(&self) -> &[CastReceipt] {
        &self.receipts
    }
    /// All rejection paths precede debit. No world is accepted here, so validation or persistence
    /// failures cannot leave unfunded physical effects. Replay returns a receipt without a start token.
    pub fn admit(
        &mut self,
        inventory: &Inventory,
        equipment: &Equipment,
        spell: &AuthoredSpell,
        request: CastRequest,
    ) -> Result<Admission, String> {
        request.validate()?;
        if let Some(index) = self.requests.get(&request.id) {
            let receipt = &self.receipts[*index];
            if receipt.request != request || receipt.source != *spell.source() {
                return Err("cast request id was reused with a different payload".into());
            }
            return Ok(Admission {
                receipt: receipt.clone(),
                funded: None,
            });
        }
        self.account.supply().validate(inventory)?;
        if equipment.owner() != &request.owner {
            return Err("cast owner differs from equipment owner".into());
        }
        if spell.revision() != request.expected_revision {
            return Err("selected authored revision changed".into());
        }
        if self
            .account
            .supply()
            .equipped_wand(inventory, equipment, &request.equipment_slot)?
            != Some(&request.expected_wand)
        {
            return Err("equipped wand changed".into());
        }
        if self
            .account
            .supply()
            .loaded(inventory, &request.expected_wand)?
            != Some(&request.expected_cartridge)
        {
            return Err("loaded cartridge changed".into());
        }
        let cartridge = self
            .account
            .supply()
            .cartridge(&request.expected_cartridge)
            .ok_or("loaded item is not a cartridge")?;
        if cartridge.remaining != request.expected_remaining {
            return Err("loaded cartridge balance changed".into());
        }
        if spell.allocation() > cartridge.remaining {
            return Err("loaded cartridge cannot fund the selected spell allocation".into());
        }
        let funding = self.account.prepare(
            inventory,
            funding_request(&request, spell.source(), spell.allocation())?,
        )?;
        if funding.replayed() {
            return Err("funding history has no matching cast receipt".into());
        }
        // A fresh brain prevents mutable Lua closure state from leaking between instances.
        let mut instance = runtime::prepare(
            &request.instance(),
            &request.holder,
            spell.definition(),
            spell.brain()?,
            spell.allocation().get(),
            request.tick,
        )?;
        instance.revision = spell.revision().into();
        let paid = self.account.commit(inventory, funding)?.receipt;
        let (before, after) = (paid.before, paid.after);
        let receipt = CastReceipt {
            instance: instance.id.clone(),
            source: spell.source().clone(),
            allocation: spell.allocation(),
            before,
            after,
            request,
        };
        self.requests
            .insert(receipt.request.id.clone(), self.receipts.len());
        self.receipts.push(receipt.clone());
        let tick = receipt.request.tick;
        let aim = receipt.request.aim;
        Ok(Admission {
            receipt,
            funded: Some(FundedCast {
                instance,
                tick,
                aim,
            }),
        })
    }
}
fn funding_request(
    request: &CastRequest,
    source: &AuthoredSource,
    amount: Energy,
) -> Result<DebitRequest, String> {
    Ok(DebitRequest {
        key: DebitKey {
            purpose: DebitPurpose::Cast,
            id: request.id.clone(),
        },
        owner: request.owner.clone(),
        cartridge: request.expected_cartridge.clone(),
        expected_remaining: request.expected_remaining,
        amount,
        payload_revision: digest(
            &serde_json::to_vec(&(request, source)).map_err(|e| e.to_string())?,
        ),
    })
}
/// Exactly one consuming start token exists for a successful new admission; a replay has none.
pub struct Admission {
    pub receipt: CastReceipt,
    pub funded: Option<FundedCast>,
}
pub struct FundedCast {
    instance: Instance,
    tick: u64,
    aim: crate::aim::Aim,
}
impl FundedCast {
    pub fn aim(&self) -> crate::aim::Aim {
        self.aim
    }
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
    /// Persist admission before calling if crash durability is required. On-start runtime faults
    /// consume paid work normally; they do not refund the cartridge or permit replaying the command.
    pub fn start(mut self, world: &mut dyn WorldAdapter) -> Instance {
        runtime::start(&mut self.instance, world, self.tick);
        self.instance
    }
}
