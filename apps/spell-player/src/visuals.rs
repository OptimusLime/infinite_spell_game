//! Optional presentation bindings. Item custody, funded sessions and physical bodies stay elsewhere.
use atelico_component::AppView;
use atelico_data::{DataKind, Kind};
use component_scene_3d::hooks::{Drive, DriveLease, Resolution, VisualDrive, WorldView};
use component_spellcraft::player_world::BodyView;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize, DataKind, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BodyVisual {
    Part {
        body: String,
        part: String,
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
        yaw_offset: f32,
    },
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Visuals(pub Vec<BodyVisual>);
impl DataKind for Visuals {
    fn kind() -> Kind {
        Kind::ListOf(Box::new(BodyVisual::kind()))
    }
}
impl Visuals {
    pub fn validate(&self, bodies: &BTreeSet<String>) -> Result<(), String> {
        let mut mapped = BTreeSet::new();
        let mut parts = BTreeSet::new();
        for visual in &self.0 {
            let BodyVisual::Part {
                body,
                part,
                offset_x,
                offset_y,
                offset_z,
                yaw_offset,
            } = visual;
            if !bodies.contains(body) {
                return Err(format!(
                    "Visual mapping names unknown configured body {body}"
                ));
            }
            if !mapped.insert(body) || !parts.insert(part) {
                return Err("Visual mappings need distinct body and part names".into());
            }
            if part.trim().is_empty() || part.chars().any(char::is_control) {
                return Err("Visual part needs a nonempty name".into());
            }
            if ![offset_x, offset_y, offset_z, yaw_offset]
                .iter()
                .all(|x| x.is_finite())
            {
                return Err("Visual offsets and yaw must be finite".into());
            }
        }
        Ok(())
    }
}
pub(crate) struct Presentation {
    world: String,
    session: String,
    lease: DriveLease,
    accepted: Visuals,
}
impl Presentation {
    pub fn new(view: AppView, world: &str, session: &str, node: &str) -> Result<Self, String> {
        Ok(Self {
            world: world.into(),
            session: session.into(),
            lease: DriveLease::new(
                WorldView::new(view, world).map_err(|e| e.to_string())?,
                format!("{session}/{node}"),
            )
            .map_err(|e| e.to_string())?,
            accepted: Visuals::default(),
        })
    }
    pub fn matches(&self, world: &str, session: &str) -> bool {
        self.world == world && self.session == session
    }
    pub fn refresh(
        &mut self,
        requested: Visuals,
        configured: &BTreeSet<String>,
        bodies: &[BodyView],
        aimed: &str,
        t: f32,
    ) -> String {
        let resolved = self.lease.status();
        let same = requested == self.accepted;
        let candidate = requested.validate(configured).and_then(|_| {
            let (drives, items) = batch(&requested, bodies, aimed, t);
            self.lease.replace(drives, items).map_err(|e| e.to_string())
        });
        match candidate {
            Ok(()) => {
                self.accepted = requested;
                if !same {
                    return String::new();
                }
                resolved
                    .iter()
                    .filter(|s| s.resolution != Resolution::Applied)
                    .map(|s| {
                        format!(
                            "{}: {:?}; physical debug body retained",
                            s.part, s.resolution
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" · ")
            }
            Err(error) => {
                // Keep last accepted presentation choices while physical positions continue to update.
                // A first invalid request has empty accepted mappings, so every body remains visible.
                let (drives, items) = batch(&self.accepted, bodies, aimed, t);
                let _ = self.lease.replace(drives, items);
                error
            }
        }
    }
}
fn batch(
    visuals: &Visuals,
    bodies: &[BodyView],
    aimed: &str,
    t: f32,
) -> (Vec<VisualDrive>, Vec<component_scene_3d::Item>) {
    let mut mapped = BTreeSet::new();
    let mut drives = Vec::new();
    for visual in &visuals.0 {
        let BodyVisual::Part {
            body,
            part,
            offset_x,
            offset_y,
            offset_z,
            yaw_offset,
        } = visual;
        mapped.insert(body.as_str());
        let row = bodies.iter().find(|b| &b.id == body);
        let at = row.map_or(glam::Vec3::ZERO, |b| b.position())
            + glam::vec3(*offset_x, *offset_y, *offset_z);
        let facing = row.map_or(0., |b| b.dx.atan2(b.dz)) + yaw_offset;
        drives.push(VisualDrive {
            part: part.clone(),
            drive: Drive {
                at,
                facing,
                pose: "idle",
                t,
                shown: row.is_some_and(|b| b.alive),
            },
            fallback: row
                .filter(|b| b.alive)
                .map(|b| crate::render::body(b.clone(), aimed)),
        });
    }
    let items = bodies
        .iter()
        .filter(|b| b.alive && !mapped.contains(b.id.as_str()))
        .map(|b| crate::render::body(b.clone(), aimed))
        .collect();
    (drives, items)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bindings_are_typed_optional_and_reject_unknown_duplicate_or_nonfinite_inputs() {
        let ids = BTreeSet::from(["actor".into(), "target".into()]);
        let one = BodyVisual::Part {
            body: "actor".into(),
            part: "hero".into(),
            offset_x: 0.,
            offset_y: -0.5,
            offset_z: 0.,
            yaw_offset: 0.,
        };
        Visuals::default().validate(&ids).unwrap();
        Visuals(vec![one.clone()]).validate(&ids).unwrap();
        assert!(Visuals(vec![one.clone(), one]).validate(&ids).is_err());
        assert!(Visuals(vec![BodyVisual::Part {
            body: "actor".into(),
            part: "hero".into(),
            offset_x: f32::NAN,
            offset_y: 0.,
            offset_z: 0.,
            yaw_offset: 0.
        }])
        .validate(&ids)
        .is_err());
        assert!(Visuals(vec![BodyVisual::Part {
            body: "other".into(),
            part: "hero".into(),
            offset_x: 0.,
            offset_y: 0.,
            offset_z: 0.,
            yaw_offset: 0.
        }])
        .validate(&ids)
        .is_err());
        let Kind::ListOf(k) = Visuals::kind() else {
            panic!("typed list")
        };
        assert!(matches!(*k, Kind::Tagged(_)));
        assert!(Visuals::kind().check(&serde_json::json!([{ "kind":"part", "body":"actor", "part":"hero", "offset_x":"banana", "offset_y":0, "offset_z":0, "yaw_offset":0 }])).is_err());
    }
    #[test]
    fn older_hosts_never_read_the_new_prop_without_both_schema_and_callback() {
        use atelico_component::mlua::Lua;
        for (schema, callback, expected) in [
            (false, false, "legacy"),
            (false, true, "legacy"),
            (true, false, "legacy"),
            (true, true, "owned"),
        ] {
            let lua = Lua::new();
            lua.globals().set("has_schema", schema).unwrap();
            lua.globals().set("has_callback", callback).unwrap();
            lua.load(r#"
                nodes={types={},builders={}}
                data={schema=function() return {fields=if has_schema then {{name="visuals"}} else {}} end}
                native_spell_player={frame=function() return "legacy" end}
                if has_callback then native_spell_player.frame_visuals=function(_,_,_,_,v) assert(#v==0);return "owned","" end end
                local hud={render=function(ref) return ref end,error=function(err) error(err) end}
                function require(name) return if name=="nodes" then nodes else hud end
                props=setmetatable({config="fixture",session="test",world="world"},{__index=function(_,key)
                    assert(key=="visuals" and has_schema and has_callback,"old host read new field")
                    return {}
                end})
            "#).exec().unwrap();
            lua.load(include_str!("../lua/nodes/spell_player.luau"))
                .exec()
                .unwrap();
            let actual:String=lua.load("return nodes.builders.SpellPlayer({env={width=100,height=100}},'root/player',props)").eval().unwrap();
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn presentation_edits_do_not_steal_targets_and_invalid_edits_keep_current_physical_positions() {
        let app = AppView::new(8201);
        let ids = BTreeSet::from(["actor".into()]);
        let visual = Visuals(vec![BodyVisual::Part {
            body: "actor".into(),
            part: "hero".into(),
            offset_x: 0.,
            offset_y: -0.5,
            offset_z: 0.,
            yaw_offset: 0.,
        }]);
        let mut rows = vec![BodyView {
            id: "actor".into(),
            shape: "actor".into(),
            x: 2.,
            y: 0.5,
            z: 3.,
            dz: 1.,
            radius: 0.5,
            alive: true,
            ..Default::default()
        }];
        let mut one = Presentation::new(app, "test/visuals", "session", "first").unwrap();
        assert!(one.refresh(visual.clone(), &ids, &rows, "", 0.).is_empty());
        let mut two = Presentation::new(app, "test/visuals", "session", "second").unwrap();
        assert!(two
            .refresh(visual.clone(), &ids, &rows, "", 0.)
            .contains("already driven"));
        assert!(
            two.accepted.0.is_empty(),
            "conflicting first request uses debug fallback"
        );
        rows[0].x = 8.;
        let mut bad = visual.clone();
        let BodyVisual::Part { part, .. } = &mut bad.0[0];
        *part = String::new();
        assert!(!one.refresh(bad, &ids, &rows, "", 1.).is_empty());
        assert_eq!(
            one.accepted, visual,
            "invalid edit keeps last valid mapping"
        );
        let (drives, items) = batch(&one.accepted, &rows, "", 1.);
        assert!(items.is_empty());
        assert_eq!(drives[0].drive.at, glam::vec3(8., 0., 3.));
        assert_eq!(
            drives[0].fallback.as_ref().unwrap().json()["row"]["x"],
            8.
        );
        drop(one);
        assert!(
            two.refresh(visual, &ids, &rows, "", 1.).is_empty(),
            "owner release permits next owner"
        );
    }
}
