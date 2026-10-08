//! Deliberate admission of one exact authored revision from one finite physical cartridge.
//! This state is owned by the caller, never by the clock/file-resettable legacy Town demo.
use crate::{
    definition::{self, Kind, Source, SpellDefinition},
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
}

/// This is an admission ledger, not a simulation snapshot. Persist it together with Inventory.
/// Completed/pending runtime instances and the world's own persistence remain the caller's concern.
#[derive(Debug)]
pub struct Casting {
    supply: Supply,
    receipts: Vec<CastReceipt>,
    requests: BTreeMap<String, usize>,
}
impl Casting {
    pub fn new(supply: Supply) -> Self {
        Self {
            supply,
            receipts: Vec::new(),
            requests: BTreeMap::new(),
        }
    }
    pub fn restore(inventory: &Inventory, snapshot: CastingSnapshot) -> Result<Self, String> {
        let supply = Supply::restore(inventory, snapshot.supply)?;
        let mut this = Self::new(supply);
        let mut remaining = BTreeMap::new();
        for receipt in snapshot.receipts {
            receipt.request.validate()?;
            let spell = AuthoredSpell::new(receipt.source.clone())?;
            let cartridge = this
                .supply
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
            if remaining
                .insert(cartridge.item.clone(), receipt.after)
                .is_some_and(|previous| previous != receipt.before)
            {
                return Err("cast receipt funding chain is discontinuous".into());
            }
            if this
                .requests
                .insert(receipt.request.id.clone(), this.receipts.len())
                .is_some()
            {
                return Err("duplicate cast request id in snapshot".into());
            }
            this.receipts.push(receipt);
        }
        for (item, last) in remaining {
            if this.supply.cartridge(&item).unwrap().remaining != last {
                return Err("cartridge balance disagrees with admission history".into());
            }
        }
        Ok(this)
    }
    pub fn snapshot(&self) -> CastingSnapshot {
        CastingSnapshot {
            supply: self.supply.snapshot(),
            receipts: self.receipts.clone(),
        }
    }
    pub fn supply(&self) -> &Supply {
        &self.supply
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
        self.supply.validate(inventory)?;
        if equipment.owner() != &request.owner {
            return Err("cast owner differs from equipment owner".into());
        }
        if spell.revision() != request.expected_revision {
            return Err("selected authored revision changed".into());
        }
        if self
            .supply
            .equipped_wand(inventory, equipment, &request.equipment_slot)?
            != Some(&request.expected_wand)
        {
            return Err("equipped wand changed".into());
        }
        if self.supply.loaded(inventory, &request.expected_wand)?
            != Some(&request.expected_cartridge)
        {
            return Err("loaded cartridge changed".into());
        }
        let cartridge = self
            .supply
            .cartridge(&request.expected_cartridge)
            .ok_or("loaded item is not a cartridge")?;
        if cartridge.remaining != request.expected_remaining {
            return Err("loaded cartridge balance changed".into());
        }
        if spell.allocation() > cartridge.remaining {
            return Err("loaded cartridge cannot fund the selected spell allocation".into());
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
        let (before, after) = self
            .supply
            .debit(&request.expected_cartridge, spell.allocation())?;
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
