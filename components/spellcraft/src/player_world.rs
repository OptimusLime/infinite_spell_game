//! Projectile-only player world, using the existing Items ballistic physics. No Town, homing,
//! spawn waves, automatic funding, source-name fusion or model requests occur in this adapter.
use crate::{
    aim::Aim,
    casting::{AuthoredSpell, CastRequest, FundedCast},
    definition::{self, Step},
    runtime::{Effect, Observation, WorldAdapter},
};
use atelico_data::Data;
use atelico_items::{
    physics::{SimWorld, Terrain},
    runtime::{BodyId, World as PhysicsWorld},
};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    Bolt,
    Orb,
}
impl Shape {
    pub fn component(self) -> &'static str {
        match self {
            Self::Bolt => "body.bolt",
            Self::Orb => "body.orb",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Form {
    pub shape: Shape,
    /// Radius at the catalog default size; an authored size scales it proportionally.
    pub radius: f32,
    pub mass: f32,
    pub lifetime_ticks: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldConfig {
    pub tick_seconds: f32,
    pub forms: Vec<Form>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Actor,
    Target,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodySpec {
    pub id: String,
    pub role: Role,
    pub position: [f32; 3],
    pub radius: f32,
    pub mass: f32,
    pub health: f64,
}
/// Typed rendering/query snapshot. Physical body IDs are not Inventory item IDs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Data)]
#[serde(default, deny_unknown_fields)]
pub struct BodyView {
    pub id: String,
    pub instance: String,
    pub shape: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    pub radius: f32,
    pub has_health: bool,
    pub health: f64,
    pub alive: bool,
}
impl BodyView {
    pub fn position(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
    pub fn velocity(&self) -> Vec3 {
        Vec3::new(self.vx, self.vy, self.vz)
    }
}

struct Body {
    physical: BodyId,
    instance: String,
    kind: String,
    role: Option<Role>,
    health: Option<f64>,
    expires: Option<u64>,
}
pub struct PlayerWorld {
    physics: SimWorld,
    config: WorldConfig,
    forms: BTreeMap<String, Form>,
    bodies: BTreeMap<String, Body>,
    physical: BTreeMap<BodyId, String>,
    aims: BTreeMap<String, Aim>,
    contacts: BTreeMap<String, Option<String>>,
    effects: Vec<Effect>,
    tick: u64,
}
fn positive(value: f32) -> bool {
    value.is_finite() && value > 0.0
}
impl PlayerWorld {
    pub fn new(
        config: WorldConfig,
        bodies: Vec<BodySpec>,
        terrain: Box<dyn Terrain + Send>,
    ) -> Result<Self, String> {
        if !positive(config.tick_seconds) {
            return Err("tick duration must be finite and positive".into());
        }
        let mut forms = BTreeMap::new();
        for form in &config.forms {
            if !positive(form.radius) || !positive(form.mass) || form.lifetime_ticks == 0 {
                return Err("form needs finite positive radius/mass and nonzero lifetime".into());
            }
            if forms
                .insert(form.shape.component().into(), form.clone())
                .is_some()
            {
                return Err("duplicate configured form".into());
            }
        }
        let mut world = Self {
            physics: SimWorld::new(0.0),
            config,
            forms,
            bodies: BTreeMap::new(),
            physical: BTreeMap::new(),
            aims: BTreeMap::new(),
            contacts: BTreeMap::new(),
            effects: Vec::new(),
            tick: 0,
        };
        world.physics.set_terrain(terrain);
        for body in bodies {
            if body.id.trim().is_empty()
                || body.id.chars().any(char::is_control)
                || world.bodies.contains_key(&body.id)
            {
                return Err("body IDs must be nonempty and unique".into());
            }
            let at = Vec3::from(body.position);
            if !at.is_finite()
                || !at.length_squared().is_finite()
                || !positive(body.radius)
                || !positive(body.mass)
                || !body.health.is_finite()
                || body.health <= 0.0
            {
                return Err("invalid configured body dimensions, position or health".into());
            }
            let physical = world.physics.add(
                if body.role == Role::Actor {
                    "actor"
                } else {
                    "target"
                },
                at,
                body.radius,
                body.mass,
            );
            world.physical.insert(physical, body.id.clone());
            world.bodies.insert(
                body.id,
                Body {
                    physical,
                    instance: String::new(),
                    kind: if body.role == Role::Actor {
                        "actor"
                    } else {
                        "target"
                    }
                    .into(),
                    role: Some(body.role),
                    health: Some(body.health),
                    expires: None,
                },
            );
        }
        Ok(world)
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn tick_seconds(&self) -> f32 {
        self.config.tick_seconds
    }
    /// The caller's locomotion controller supplies the authoritative actor pose. Existing shot aim
    /// stays pinned; moving the caster cannot redirect an already formed projectile.
    pub fn update_actor(&mut self, id: &str, position: [f32; 3]) -> Result<(), String> {
        let at = Vec3::from(position);
        if !at.is_finite() || !at.length_squared().is_finite() {
            return Err("actor position must be finite".into());
        }
        let body = self
            .bodies
            .get(id)
            .filter(|b| b.role == Some(Role::Actor))
            .ok_or("unknown actor")?;
        let physical = self
            .physics
            .bodies
            .iter_mut()
            .find(|b| b.id == body.physical && !b.consumed)
            .ok_or("actor is no longer live")?;
        physical.at = at;
        Ok(())
    }
    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }
    pub fn bodies(&self) -> Vec<BodyView> {
        self.bodies
            .iter()
            .filter_map(|(id, b)| {
                self.physics.body(b.physical).map(|p| BodyView {
                    id: id.clone(),
                    instance: b.instance.clone(),
                    shape: b.kind.clone(),
                    x: p.at.x,
                    y: p.at.y,
                    z: p.at.z,
                    vx: p.velocity.x,
                    vy: p.velocity.y,
                    vz: p.velocity.z,
                    dx: p.dir.x,
                    dy: p.dir.y,
                    dz: p.dir.z,
                    radius: p.radius,
                    has_health: b.health.is_some(),
                    health: b.health.unwrap_or_default(),
                    alive: !p.consumed,
                })
            })
            .collect()
    }
    pub fn validate_spell(&self, spell: &AuthoredSpell) -> Result<(), String> {
        for (_, s) in spell.definition().steps() {
            match s.component.as_str() {
                "body.bolt" | "body.orb" => {
                    self.form_radius(s)?;
                }
                "body.self" | "motion.launch" | "sense.nearby" | "logic.lua" | "effect.damage"
                | "force.push" | "effect.warm" => {}
                other => {
                    return Err(format!(
                        "player projectile adapter does not support {other}"
                    ))
                }
            }
        }
        Ok(())
    }
    fn form_radius(&self, step: &Step) -> Result<f32, String> {
        let form = self
            .forms
            .get(&step.component)
            .ok_or_else(|| format!("no configured physical form for {}", step.component))?;
        let standard = definition::component(&step.component)
            .and_then(|c| c.params.get("size"))
            .and_then(|p| p.default.as_ref())
            .and_then(|v| v.as_f64())
            .ok_or("form has no catalog size")?;
        let radius =
            (form.radius as f64 * definition::number(step, "size", standard) / standard) as f32;
        if !positive(radius) {
            return Err("authored form size overflows configured radius".into());
        }
        Ok(radius)
    }
    pub fn validate_request(
        &self,
        spell: &AuthoredSpell,
        request: &CastRequest,
    ) -> Result<(), String> {
        self.validate_spell(spell)?;
        if request.tick != self.tick {
            return Err("player simulation tick changed".into());
        }
        if !self
            .bodies
            .get(&request.holder)
            .is_some_and(|b| b.role == Some(Role::Actor))
            || !self.exists(&request.holder)
        {
            return Err("cast holder is not a live configured actor".into());
        }
        Ok(())
    }
    /// External durable controllers may validate_request, admit, persist, then consume this token.
    /// The aim comes from the immutable funded token, never from an editable receipt copy.
    pub fn start_funded(&mut self, funded: FundedCast) -> crate::runtime::Instance {
        self.aims.insert(funded.instance().id.clone(), funded.aim());
        funded.start(self)
    }
    /// One monotonic fixed step. The caller then ticks funded instances to consume these contacts.
    pub fn advance(&mut self) -> Result<(), String> {
        let tick = self.tick.checked_add(1).ok_or("player tick overflow")?;
        let now = tick as f64 * self.config.tick_seconds as f64;
        if !now.is_finite() || now > f32::MAX as f64 {
            return Err("player physics clock overflow".into());
        }
        self.tick = tick;
        for b in self
            .bodies
            .values()
            .filter(|b| b.expires.is_some_and(|until| self.tick >= until))
        {
            self.physics.consume(b.physical);
        }
        self.physics.step(self.config.tick_seconds, now as f32);
        for (id, b) in &self.bodies {
            if let Some(c) = self.physics.contact(b.physical) {
                // A contact remains until its paid sensor reads it; no homing or target substitution.
                self.contacts.entry(id.clone()).or_insert_with(|| {
                    c.other.and_then(|other| self.physical.get(&other).cloned())
                });
            }
        }
        Ok(())
    }
    pub(crate) fn has_live_forms(&self, instance: &str) -> bool {
        self.bodies
            .values()
            .any(|b| b.instance == instance && self.physics.read(b.physical).is_some())
    }
    /// Release only this cast's owned resources. Prior impulses/damage to other bodies are not undone.
    pub(crate) fn release_instance(&mut self, instance: &str) {
        let owned: std::collections::BTreeSet<_> = self
            .bodies
            .iter()
            .filter(|(_, b)| b.instance == instance)
            .map(|(id, b)| (id.clone(), b.physical))
            .collect();
        let ids: std::collections::BTreeSet<_> = owned.iter().map(|(_, id)| *id).collect();
        for (id, physical) in &owned {
            self.physics.contact(*physical);
            self.bodies.remove(id);
            self.physical.remove(physical);
            self.contacts.remove(id);
        }
        self.contacts.retain(|_, target| {
            !target
                .as_ref()
                .is_some_and(|t| owned.iter().any(|(id, _)| id == t))
        });
        self.physics.bodies.retain(|b| !ids.contains(&b.id));
        self.aims.remove(instance);
    }
    #[cfg(test)]
    pub(crate) fn resource_counts(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.bodies.len(),
            self.physics.bodies.len(),
            self.physical.len(),
            self.aims.len(),
            self.contacts.len(),
        )
    }
    fn physical(&self, id: &str) -> Result<BodyId, String> {
        self.bodies
            .get(id)
            .map(|b| b.physical)
            .ok_or_else(|| format!("unknown world body {id}"))
    }
    fn numeric(&self, e: &Effect, key: &str) -> Result<f64, String> {
        e.params
            .get(key)
            .and_then(|v| v.as_f64())
            .or_else(|| {
                definition::component(&e.kind)
                    .and_then(|c| c.params.get(key))
                    .and_then(|p| p.default.as_ref())
                    .and_then(|v| v.as_f64())
            })
            .filter(|n| n.is_finite())
            .ok_or_else(|| format!("{} missing numeric {key}", e.kind))
    }
}
impl WorldAdapter for PlayerWorld {
    fn form(&mut self, instance: &str, caster: &str, step: &Step) -> Result<String, String> {
        if !self.exists(caster) {
            return Err("caster is gone".into());
        }
        let aim = *self
            .aims
            .get(instance)
            .ok_or("instance has no admitted aim")?;
        let form = self
            .forms
            .get(&step.component)
            .ok_or("unsupported physical form")?;
        let id = format!("{instance}/{}", step.id);
        if self.bodies.contains_key(&id) {
            return Err("physical body already formed".into());
        }
        let expires = self
            .tick
            .checked_add(form.lifetime_ticks)
            .ok_or("form lifetime overflow")?;
        let radius = self.form_radius(step)?;
        let physical = self
            .physics
            .add(&step.component, aim.origin(), radius, form.mass);
        self.physical.insert(physical, id.clone());
        self.bodies.insert(
            id.clone(),
            Body {
                physical,
                instance: instance.into(),
                kind: step.component.clone(),
                role: None,
                health: None,
                expires: Some(expires),
            },
        );
        Ok(id)
    }
    fn sense(&mut self, step: &Step, target: &str, tick: u64) -> Result<Observation, String> {
        if step.component != "sense.nearby" {
            return Err("unsupported player sensor".into());
        }
        let source = self
            .physics
            .read(self.physical(target)?)
            .ok_or("sensor body is gone")?;
        let radius = definition::number(step, "radius", 0.0) as f32;
        let mut found: Vec<_> = self
            .bodies
            .iter()
            .filter(|(id, b)| id.as_str() != target && b.role == Some(Role::Target))
            .filter_map(|(id, b)| {
                self.physics.read(b.physical).and_then(|p| {
                    let d = p.at.distance(source.at);
                    (d <= radius).then(|| (d, id.clone()))
                })
            })
            .collect();
        found.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        Ok(Observation {
            tick,
            target: target.into(),
            bodies: found.iter().map(|(_, id)| id.clone()).collect(),
            distances: found.iter().map(|(d, _)| *d as f64).collect(),
        })
    }
    fn apply(&mut self, e: &Effect) -> Result<(), String> {
        match e.kind.as_str() {
            "motion.launch" => {
                let speed = self.numeric(e, "speed")? as f32;
                let aim = *self
                    .aims
                    .get(&e.instance)
                    .ok_or("instance has no admitted aim")?;
                for target in &e.targets {
                    self.physics
                        .set_velocity(self.physical(target)?, aim.direction() * speed);
                }
            }
            "effect.damage" => {
                let amount = self.numeric(e, "amount")?;
                for target in &e.targets {
                    if let Some(body) = self.bodies.get_mut(target) {
                        if let Some(health) = &mut body.health {
                            *health = (*health - amount).max(0.0);
                            if *health == 0.0 {
                                self.physics.consume(body.physical);
                            }
                        }
                    }
                }
            }
            "force.push" => {
                let strength = self.numeric(e, "strength")? as f32;
                let aim = *self
                    .aims
                    .get(&e.instance)
                    .ok_or("instance has no admitted aim")?;
                for target in &e.targets {
                    self.physics
                        .impulse(self.physical(target)?, aim.direction() * strength);
                }
            }
            "effect.warm" => {} // The paid effect receipt carries authored radius and target for rendering.
            other => return Err(format!("unsupported player effect {other}")),
        }
        self.effects.push(e.clone());
        Ok(())
    }
    fn exists(&self, body: &str) -> bool {
        self.bodies
            .get(body)
            .is_some_and(|b| self.physics.read(b.physical).is_some())
    }
    fn take_contact(&mut self, body: &str) -> Option<String> {
        self.contacts.remove(body).flatten()
    }
}
