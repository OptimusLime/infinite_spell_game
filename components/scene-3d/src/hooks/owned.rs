//! View-owned presentation drives. The caller supplies the App's explicit render identity.
use super::{Drive, Extra, Item};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldView {
    pub app: u64,
    pub layer: u32,
    pub path: String,
}
impl WorldView {
    pub fn new(
        view: atelico_component::AppView,
        path: impl Into<String>,
    ) -> Result<Self, DriveError> {
        let path = path.into();
        if !valid_name(&path) {
            return Err(DriveError::Invalid("world path"));
        }
        Ok(Self {
            app: view.id,
            layer: view.layer,
            path,
        })
    }
}
#[derive(Clone, Debug)]
pub struct VisualDrive {
    pub part: String,
    pub drive: Drive,
    /// Kept visible if the current scene does not contain exactly one drivable part.
    pub fallback: Option<Item>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DriveError {
    Invalid(&'static str),
    Conflict { part: String, owner: String },
    Released,
}
impl std::fmt::Display for DriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(what) => write!(f, "invalid visual drive {what}"),
            Self::Conflict { part, owner } => {
                write!(f, "visual part {part} is already driven by {owner}")
            }
            Self::Released => write!(f, "visual drive owner has been released"),
        }
    }
}
impl std::error::Error for DriveError {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    Applied,
    Missing,
    Ambiguous,
    Unsupported,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisualStatus {
    pub part: String,
    pub resolution: Resolution,
}
/// A driven mesh is already in `Extra`; this part retains only follow-position and carried-light
/// metadata. It contributes no static/animated geometry, colliders or a second lamp source.
struct DrivenMetadata {
    source: std::sync::Arc<dyn crate::part::WorldPart>,
    drive: Drive,
}
impl crate::part::WorldPart for DrivenMetadata {
    fn type_name(&self) -> &'static str {
        "DrivenMetadata"
    }
    fn props(&self) -> serde_json::Value {
        serde_json::json!({"source": self.source.props(), "at": self.drive.at.to_array(),
            "facing": self.drive.facing, "shown": self.drive.shown})
    }
    fn colours(&self, _: &[crate::material::Material]) -> Vec<(crate::nodes::Rgb, String)> {
        vec![]
    }
    fn name(&self) -> &str {
        self.source.name()
    }
    fn position(&self, _: f32) -> Option<glam::Vec3> {
        Some(self.drive.at)
    }
    fn lamps(
        &self,
        t: f32,
        day: f32,
        _: crate::part::Ground,
    ) -> Vec<atelico_render_3d::probes::Lamp> {
        self.source.driven_lamps(&self.drive, t, day)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Default)]
pub(super) struct Registry {
    entries: BTreeMap<WorldView, BTreeMap<u64, Batch>>,
}
#[derive(Clone)]
struct Batch {
    owner: String,
    drives: Vec<VisualDrive>,
    items: Vec<Item>,
    status: Vec<VisualStatus>,
}
fn valid_name(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(char::is_control)
}
impl Registry {
    pub(super) fn claim(
        &self,
        view: &WorldView,
        part: &str,
        except: Option<u64>,
    ) -> Option<String> {
        self.entries.get(view)?.iter().find_map(|(token, b)| {
            (Some(*token) != except && b.drives.iter().any(|d| d.part == part))
                .then(|| b.owner.clone())
        })
    }
    /// Apply only unique supported targets. Missing/unsupported mappings retain their debug fallback.
    pub(super) fn resolve(
        &mut self,
        view: &WorldView,
        items: &mut Vec<Item>,
        extras: &mut Vec<Extra>,
        key: &mut String,
    ) {
        let Some(batches) = self.entries.get_mut(view) else {
            return;
        };
        for batch in batches.values() {
            items.extend(batch.items.iter().cloned());
        }
        let mut names = BTreeMap::<String, usize>::new();
        for item in items.iter() {
            if let Item::Part(p) = item {
                *names.entry(p.0.name().into()).or_default() += 1;
            }
        }
        let mut removed = BTreeSet::new();
        let mut replacement = Vec::new();
        for batch in batches.values_mut() {
            batch.status.clear();
            key.push_str(&format!("{:?}{:?}", batch.items, batch.drives));
            for d in &batch.drives {
                let resolution = match names.get(&d.part).copied().unwrap_or(0) {
                    0 => Resolution::Missing,
                    1 => {
                        let (index, part) = items
                            .iter()
                            .enumerate()
                            .find_map(|(i, item)| match item {
                                Item::Part(p) if p.0.name() == d.part => Some((i, p)),
                                _ => None,
                            })
                            .expect("counted part");
                        let driven = if let Some(extra) = part.0.drive(&d.drive) {
                            extras.push(extra);
                            replacement.push(Item::part(DrivenMetadata {
                                source: part.0.clone(),
                                drive: d.drive,
                            }));
                            true
                        } else if let Some(stand_in) = part.0.stand_in(&d.drive) {
                            replacement.push(Item::Part(crate::part::Part(stand_in)));
                            true
                        } else if !d.drive.shown {
                            // Mesh implementations return None for a hidden body. Probe support without
                            // displaying that mesh; this preserves the existing WorldPart contract.
                            part.0
                                .drive(&Drive {
                                    shown: true,
                                    ..d.drive
                                })
                                .is_some()
                        } else {
                            false
                        };
                        if driven {
                            removed.insert(index);
                            Resolution::Applied
                        } else {
                            Resolution::Unsupported
                        }
                    }
                    _ => Resolution::Ambiguous,
                };
                if resolution != Resolution::Applied {
                    replacement.extend(d.fallback.iter().cloned());
                }
                batch.status.push(VisualStatus {
                    part: d.part.clone(),
                    resolution,
                });
            }
        }
        let mut index = 0;
        items.retain(|_| {
            let keep = !removed.contains(&index);
            index += 1;
            keep
        });
        items.extend(replacement);
    }
}
/// A unique presentation lifetime, independent of funded sessions. It cannot release another owner's batch.
pub struct DriveLease {
    view: WorldView,
    token: u64,
}
impl DriveLease {
    pub fn new(view: WorldView, owner: impl Into<String>) -> Result<Self, DriveError> {
        let owner = owner.into();
        if !valid_name(&owner) {
            return Err(DriveError::Invalid("owner"));
        }
        // Public fields allow ergonomic callers, so revalidate the supplied key at the boundary.
        WorldView::new(
            atelico_component::AppView {
                id: view.app,
                layer: view.layer,
            },
            view.path.clone(),
        )?;
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let token = NEXT.fetch_add(1, Ordering::Relaxed);
        super::hooks()
            .lock()
            .unwrap()
            .owned
            .entries
            .entry(view.clone())
            .or_default()
            .insert(
                token,
                Batch {
                    owner,
                    drives: vec![],
                    items: vec![],
                    status: vec![],
                },
            );
        Ok(Self { view, token })
    }
    pub fn replace(&self, drives: Vec<VisualDrive>, items: Vec<Item>) -> Result<(), DriveError> {
        let mut names = BTreeSet::new();
        for d in &drives {
            if !valid_name(&d.part) || !names.insert(&d.part) {
                return Err(DriveError::Invalid("distinct part names"));
            }
            if !d.drive.at.is_finite()
                || !d.drive.facing.is_finite()
                || !d.drive.t.is_finite()
                || !valid_name(d.drive.pose)
            {
                return Err(DriveError::Invalid("finite position, facing and pose time"));
            }
        }
        let mut h = super::hooks().lock().unwrap();
        for d in &drives {
            if let Some(owner) = h.owned.claim(&self.view, &d.part, Some(self.token)) {
                return Err(DriveError::Conflict {
                    part: d.part.clone(),
                    owner,
                });
            }
            if h.drives
                .get(&self.view)
                .is_some_and(|m| m.contains_key(&d.part))
            {
                return Err(DriveError::Conflict {
                    part: d.part.clone(),
                    owner: "legacy drive".into(),
                });
            }
        }
        let batch = h
            .owned
            .entries
            .get_mut(&self.view)
            .and_then(|b| b.get_mut(&self.token))
            .ok_or(DriveError::Released)?;
        batch.drives = drives;
        batch.items = items;
        batch.status.clear();
        Ok(())
    }
    pub fn status(&self) -> Vec<VisualStatus> {
        super::hooks()
            .lock()
            .unwrap()
            .owned
            .entries
            .get(&self.view)
            .and_then(|b| b.get(&self.token))
            .map_or_else(Vec::new, |b| b.status.clone())
    }
}
impl Drop for DriveLease {
    fn drop(&mut self) {
        let mut h = super::hooks().lock().unwrap();
        if let Some(batches) = h.owned.entries.get_mut(&self.view) {
            batches.remove(&self.token);
            if batches.is_empty() {
                h.owned.entries.remove(&self.view);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::{Bomb, Character};
    use glam::vec3;
    fn view(name: &str, layer: u32) -> WorldView {
        WorldView::new(
            atelico_component::AppView {
                id: layer as u64,
                layer,
            },
            format!("test/owned/{name}"),
        )
        .unwrap()
    }
    fn hero(name: &str) -> Item {
        Item::part(Character {
            name: name.into(),
            ..Default::default()
        })
    }
    fn mapping(part: &str, x: f32) -> VisualDrive {
        VisualDrive {
            part: part.into(),
            drive: Drive {
                at: vec3(x, 1., 3.),
                facing: 0.4,
                pose: "idle",
                t: 0.,
                shown: true,
            },
            fallback: Some(Item::part(Bomb::default())),
        }
    }
    fn render(v: &WorldView, items: &[Item]) -> (Vec<Item>, Vec<Extra>, String) {
        super::super::driven_in(v, items)
    }
    #[test]
    fn atomic_replace_conflict_and_owner_drop_do_not_steal_or_clear_other_parts() {
        let v = view("atomic", 8101);
        let first = DriveLease::new(v.clone(), "first").unwrap();
        let second = DriveLease::new(v.clone(), "second").unwrap();
        first.replace(vec![mapping("hero", 4.)], vec![]).unwrap();
        second.replace(vec![mapping("other", 8.)], vec![]).unwrap();
        let original = [hero("hero"), hero("other")];
        let before = render(&v, &original).2;
        assert!(matches!(
            second.replace(vec![mapping("other", 9.), mapping("hero", 10.)], vec![]),
            Err(DriveError::Conflict { .. })
        ));
        assert_eq!(
            before,
            render(&v, &original).2,
            "a rejected batch changes no part"
        );
        assert!(matches!(
            super::super::try_drive(&v, "hero", None),
            Err(DriveError::Conflict { .. })
        ));
        drop(first);
        let third = DriveLease::new(v.clone(), "third").unwrap();
        third.replace(vec![mapping("hero", 12.)], vec![]).unwrap();
        drop(second);
        let (kept, extras, _) = render(&v, &original);
        assert_eq!((kept.len(), extras.len()), (2, 1));
        assert_eq!(
            extras[0].transform.transform_point3(glam::Vec3::ZERO),
            vec3(12., 1., 3.)
        );
        drop(third);
        assert_eq!(render(&v, &original).0.len(), 2);
        assert!(render(&v, &original).1.is_empty());
    }
    #[test]
    fn explicit_view_keys_isolate_same_named_worlds_and_legacy_drive() {
        let a = view("scopes", 8102);
        let mut b = view("scopes", 8102);
        b.app += 100;
        assert_eq!(
            (a.layer, &a.path),
            (b.layer, &b.path),
            "two projects may use identical local layers and paths"
        );
        let owned = DriveLease::new(a.clone(), "same-name").unwrap();
        owned.replace(vec![mapping("hero", 5.)], vec![]).unwrap();
        super::super::try_drive(&b, "hero", Some(mapping("hero", 10.).drive)).unwrap();
        let items = [hero("hero")];
        assert_eq!(
            render(&a, &items).1[0]
                .transform
                .transform_point3(glam::Vec3::ZERO)
                .x,
            5.
        );
        assert_eq!(
            render(&b, &items).1[0]
                .transform
                .transform_point3(glam::Vec3::ZERO)
                .x,
            10.
        );
        let blocked = DriveLease::new(b.clone(), "new").unwrap();
        assert!(matches!(
            blocked.replace(vec![mapping("hero", 30.)], vec![]),
            Err(DriveError::Conflict { .. })
        ));
        super::super::try_drive(&b, "hero", None).unwrap();
        assert_eq!(render(&b, &items).0.len(), 1);
        drop(owned);
        assert_eq!(render(&a, &items).0.len(), 1);
    }
    #[test]
    fn invalid_replacement_retains_last_good_and_missing_ambiguous_unsupported_keep_fallback() {
        let v = view("validation", 8104);
        let lease = DriveLease::new(v.clone(), "actor").unwrap();
        lease.replace(vec![mapping("hero", 3.)], vec![]).unwrap();
        let key = render(&v, &[hero("hero")]).2;
        assert!(lease
            .replace(vec![mapping("hero", f32::NAN)], vec![])
            .is_err());
        assert!(lease
            .replace(vec![mapping("hero", 1.), mapping("hero", 2.)], vec![])
            .is_err());
        assert_eq!(key, render(&v, &[hero("hero")]).2);
        let (kept, extras, _) = render(&v, &[]);
        assert_eq!((kept.len(), extras.len()), (1, 0));
        assert_eq!(lease.status()[0].resolution, Resolution::Missing);
        let (kept, extras, _) = render(&v, &[hero("hero"), hero("hero")]);
        assert_eq!((kept.len(), extras.len()), (3, 0));
        assert_eq!(lease.status()[0].resolution, Resolution::Ambiguous);
        // A named part may have no drive implementation. Keep it and the physical fallback.
        #[derive(Debug)]
        struct Static;
        impl crate::part::WorldPart for Static {
            fn type_name(&self) -> &'static str {
                "Static"
            }
            fn name(&self) -> &str {
                "hero"
            }
            fn props(&self) -> serde_json::Value {
                serde_json::json!({})
            }
            fn colours(&self, _: &[crate::material::Material]) -> Vec<(crate::nodes::Rgb, String)> {
                vec![]
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
        let (kept, extras, _) = render(&v, &[Item::part(Static)]);
        assert_eq!((kept.len(), extras.len()), (2, 0));
        assert_eq!(lease.status()[0].resolution, Resolution::Unsupported);
        let mut hidden = mapping("hero", 2.);
        hidden.drive.shown = false;
        lease.replace(vec![hidden], vec![]).unwrap();
        let (kept, extras, _) = render(&v, &[hero("hero")]);
        assert!(kept.is_empty() && extras.is_empty());
        assert_eq!(lease.status()[0].resolution, Resolution::Applied);
    }
    #[test]
    fn driven_mesh_has_one_carried_lamp_and_follow_position_without_duplicate_geometry() {
        use crate::part::WorldPart;
        let v = view("mesh-metadata", 8106);
        let lease = DriveLease::new(v.clone(), "mesh").unwrap();
        let character = Character {
            name: "hero".into(),
            x: 90.,
            z: -20.,
            reach: 3.,
            intensity: 2.,
            ..Default::default()
        };
        let original_light = character.lamps(1.25, 0.3, &|_, _| 1.);
        let d = mapping("hero", 6.);
        lease.replace(vec![d.clone()], vec![]).unwrap();
        let original = [Item::part(character)];
        let (items, extras, _) = render(&v, &original);
        assert_eq!((items.len(), extras.len()), (1, 1));
        let Item::Part(p) = &items[0] else {
            panic!("metadata")
        };
        assert_eq!(p.0.type_name(), "DrivenMetadata");
        assert_eq!(p.0.position(0.), Some(d.drive.at));
        assert_eq!(p.0.name(), "hero");
        assert!(p.0.shape("flat").is_none());
        assert!(p.0.build(&|_, _| 100.).mesh.0.is_empty());
        assert!(p.0.animate(0., &|_, _| 100.).is_none());
        assert!(p.0.things(0., &|_, _| 100.).is_empty());
        let lamps = p.0.lamps(1.25, 0.3, &|_, _| 100.);
        assert_eq!(lamps.len(), 1);
        assert_eq!(lamps[0].at, d.drive.at + vec3(0., 0.35, 0.));
        assert_eq!(lamps[0].colour, original_light[0].colour);
        assert_eq!(lamps[0].reach, original_light[0].reach);
        lease
            .replace(
                vec![VisualDrive {
                    drive: Drive {
                        shown: false,
                        ..d.drive
                    },
                    ..d
                }],
                vec![],
            )
            .unwrap();
        let (items, extras, _) = render(&v, &original);
        assert!(
            items.is_empty() && extras.is_empty(),
            "hidden mesh has no light or mesh"
        );
        drop(lease);
        let (items, extras, _) = render(&v, &original);
        assert_eq!(
            (items.len(), extras.len()),
            (1, 0),
            "unmapped original returns unchanged"
        );
        let Item::Part(p) = &items[0] else {
            panic!("original")
        };
        assert_eq!(p.0.type_name(), "Character");
        assert_eq!(p.0.lamps(1.25, 0.3, &|_, _| 1.)[0].at, original_light[0].at);
    }
    #[test]
    fn walker_uses_existing_stand_in_with_exact_position_and_carried_light() {
        let v = view("walker", 8105);
        let lease = DriveLease::new(v.clone(), "sprite").unwrap();
        let walker = component_pixel_sprites::parts::Walker {
            name: "hero".into(),
            reach: 3.,
            intensity: 2.,
            ..Default::default()
        };
        let light = walker.light;
        let d = mapping("hero", 6.);
        lease.replace(vec![d.clone()], vec![]).unwrap();
        let (items, extras, _) = render(&v, &[Item::part(walker)]);
        assert!(extras.is_empty());
        assert_eq!(items.len(), 1);
        let Item::Part(p) = &items[0] else {
            panic!("stand-in")
        };
        assert_eq!(p.0.position(0.), Some(d.drive.at));
        let lamps = p.0.lamps(0., 0., &|_, _| 99.);
        assert_eq!(lamps.len(), 1);
        assert_eq!(lamps[0].reach, 3.);
        assert_eq!(p.0.props()["light"], serde_json::to_value(light).unwrap());
    }
}
