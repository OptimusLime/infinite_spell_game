//! Engineering geometry for the actual physical bodies, handed to the existing scene-3d hook.
use atelico_render_3d::{
    color::{hex, Rgb},
    material::Material,
    mesh::Mesh,
    part::{Ground, Piece, WorldPart},
};
use component_spellcraft::player_world::BodyView;
use glam::Vec3;
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
struct Body {
    row: BodyView,
    aimed: bool,
}
impl WorldPart for Body {
    fn type_name(&self) -> &'static str {
        "SpellPlayerBody"
    }
    fn props(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
    fn colours(&self, _: &[Material]) -> Vec<(Rgb, String)> {
        let color = match self.row.shape.as_str() {
            "actor" => 0x977df5,
            "target" => {
                if self.aimed {
                    0x73dca0
                } else if self.row.health > 0. {
                    0xf0a74e
                } else {
                    0x756b62
                }
            }
            "body.orb" => 0x80d6e8,
            _ => 0xe6f0ff,
        };
        vec![(hex(color), "plain".into())]
    }
    fn shape(&self, _: &str) -> Option<String> {
        Some(format!("player-body:{:?}:{}", self.row, self.aimed))
    }
    fn build(&self, _: Ground) -> Piece {
        let mut mesh = Mesh::default();
        let center = self.row.position();
        let size = Vec3::splat(self.row.radius);
        mesh.ball(center, size, 8, 12, 0.);
        Piece {
            mesh,
            framed: true,
            bounds: Some((center - size, center + size)),
            ..Default::default()
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
pub fn publish(world: &str, session: &str, bodies: Vec<BodyView>, aimed: &str) {
    component_scene_3d::hooks::set_items(
        world,
        session,
        bodies
            .into_iter()
            .filter(|b| b.alive)
            .map(|row| {
                component_scene_3d::Item::part(Body {
                    aimed: row.id == aimed,
                    row,
                })
            })
            .collect(),
    );
}
