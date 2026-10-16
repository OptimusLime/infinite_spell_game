//! Explicit shot pose captured in the idempotent command, independent of later camera/target movement.
use glam::Vec3;
use serde::{Deserialize, Deserializer, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Aim {
    origin: [f32; 3],
    direction: [f32; 3],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AimData {
    origin: [f32; 3],
    direction: [f32; 3],
}
impl Aim {
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Result<Self, String> {
        let (p, d) = (Vec3::from(origin), Vec3::from(direction));
        if !p.is_finite()
            || !p.length_squared().is_finite()
            || !d.is_finite()
            || !d.length_squared().is_finite()
            || d.length_squared() <= 0.0
        {
            return Err("aim requires finite origin and nonzero finite direction".into());
        }
        // Preserve the submitted payload exactly; normalization happens only when using the direction.
        Ok(Self { origin, direction })
    }
    pub fn origin(self) -> Vec3 {
        self.origin.into()
    }
    pub fn direction(self) -> Vec3 {
        Vec3::from(self.direction).normalize()
    }
}
impl<'de> Deserialize<'de> for Aim {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let d = AimData::deserialize(d)?;
        Self::new(d.origin, d.direction).map_err(serde::de::Error::custom)
    }
}
