//! What other packages (a character, a follow camera) plug into a World, by the World's node path (its paint id,
//! e.g. "root/world"): meshes drawn with it, the camera that looks at it, and what it last drew: its things, its
//! ground and the light of its hour.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use glam::{vec2, Vec3};

use crate::nodes::{Item, Rgb, Terrain};
pub use atelico_render_3d::part::{Deck, Drive, Extra};

/// Where a camera is: eye, the point it looks at, vertical field of view in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub eye: Vec3,
    pub look: Vec3,
    pub fov: f32,
}

/// What a World last drew.
#[derive(Clone, Debug)]
pub struct Seen {
    pub items: Vec<Item>,
    /// Its ground: walkable height (decks included) and colliders (huts, posts, trunks, rocks, the campfire, and
    /// the crowns of its plants).
    pub ground: Ground,
    /// What lit and shadowed colour are multiplied by at this hour, and where the light comes from.
    pub light: Rgb,
    pub shadow: Rgb,
    pub towards_light: Vec3,
    /// The picture's width over its height, as the World last drew it (a game's camera frames for it).
    pub aspect: f32,
}

#[derive(Default)]
struct Hooks {
    extras: HashMap<String, Vec<Extra>>,
    items: HashMap<String, HashMap<String, Vec<Item>>>,
    drives: HashMap<String, HashMap<String, Drive>>,
    poses: HashMap<String, Pose>,
    seen: HashMap<String, Seen>,
}

fn hooks() -> &'static Mutex<Hooks> {
    static H: OnceLock<Mutex<Hooks>> = OnceLock::new();
    H.get_or_init(Default::default)
}

/// Meshes to draw in the World at `world` from now on (replacing the previous set).
pub fn set_extras(world: &str, extras: Vec<Extra>) {
    hooks().lock().unwrap().extras.insert(world.into(), extras);
}

/// Things `owner` adds to the World at `world` from now on, beside its own (replacing what `owner` added before): a
/// thrown bomb, say. Each owner keeps its own set.
pub fn set_items(world: &str, owner: &str, items: Vec<Item>) {
    let mut h = hooks().lock().unwrap();
    let all = h.items.entry(world.into()).or_default();
    if items.is_empty() {
        all.remove(owner);
    } else {
        all.insert(owner.into(), items);
    }
}

/// Look at the World at `world` through `pose` (None: back to its own Camera or circling). A Camera the World
/// names still wins.
pub fn set_pose(world: &str, pose: Option<Pose>) {
    let mut h = hooks().lock().unwrap();
    match pose {
        Some(p) => h.poses.insert(world.into(), p),
        None => h.poses.remove(world),
    };
}

/// Drive the Character named `name` in the World at `world` (None: back to standing where its node puts it). Its
/// look stays the node's: outfit, headwear, colours and scale are edited in the scene, not here.
pub fn drive(world: &str, name: &str, drive: Option<Drive>) {
    let mut h = hooks().lock().unwrap();
    let drives = h.drives.entry(world.into()).or_default();
    match drive {
        Some(d) => drives.insert(name.into(), d),
        None => drives.remove(name),
    };
}

/// The World's things with its driven Characters taken out and what packages added put in, its extras with those Characters added where they are
/// driven to, and what changes the picture: the extras' placements and the drives.
pub(crate) fn driven(world: &str, items: &[Item]) -> (Vec<Item>, Vec<Extra>, String) {
    let h = hooks().lock().unwrap();
    let drives = h.drives.get(world);
    let mut extras = h.extras.get(world).cloned().unwrap_or_default();
    let added: Vec<&Item> = h.items.get(world).into_iter().flat_map(|m| m.values().flatten()).collect();
    let mut kept = Vec::with_capacity(items.len() + added.len());
    for item in items.iter().chain(added.iter().copied()) {
        match (item, drives) {
            (Item::Part(p), Some(d)) if d.contains_key(p.0.name()) => {
                if let Some(e) = p.0.drive(&d[p.0.name()]) {
                    extras.push(e);
                } else if let Some(s) = p.0.stand_in(&d[p.0.name()]) {
                    kept.push(Item::Part(crate::part::Part(s)));
                }
            }
            _ => kept.push(item.clone()),
        }
    }
    let mut key: Vec<String> = drives.into_iter().flatten().map(|(n, d)| format!("{n}{d:?}")).collect();
    key.sort();
    key.extend(extras.iter().map(|e| format!("{:?}", e.transform)));
    key.extend(added.iter().map(|i| format!("{i:?}")));
    (kept, extras, key.concat())
}

/// What the World at `world` last drew.
pub fn seen(world: &str) -> Option<Seen> {
    hooks().lock().unwrap().seen.get(world).cloned()
}

pub(crate) fn pose(world: &str) -> Option<Pose> {
    hooks().lock().unwrap().poses.get(world).copied()
}

pub(crate) fn remember(world: &str, seen: Seen) {
    hooks().lock().unwrap().seen.insert(world.into(), seen);
}

pub use atelico_render_3d::collider::Collider;

/// The ground of a World's things: the Terrain's surface, the dock's and huts' decks on top of it, and colliders.
#[derive(Clone, Debug, Default)]
pub struct Ground {
    terrain: Option<Terrain>,
    decks: Vec<Deck>,
    pub colliders: Vec<Collider>,
}

impl Ground {
    pub fn of(items: &[Item]) -> Ground {
        let terrain = items.iter().find_map(|i| if let Item::Terrain(t) = i { Some(t.clone()) } else { None });
        let under = |x: f32, z: f32| terrain.as_ref().map_or(0.0, |t| t.height_at(x, z));
        let (mut decks, mut colliders) = (Vec::new(), Vec::new());
        for item in items {
            if let Item::Part(p) = item {
                let (d, c) = p.0.ground(&under);
                decks.extend(d);
                colliders.extend(c);
            }
        }
        Ground { terrain, decks, colliders }
    }

    /// The walkable tops over the ground: dock planks, hut decks.
    pub fn decks(&self) -> &[Deck] {
        &self.decks
    }

    /// Where one can stand at (x, z): the highest of the ground and any deck over it.
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let ground = self.terrain.as_ref().map_or(0.0, |t| t.height_at(x, z));
        self.decks.iter().filter(|d| d.holds(vec2(x, z))).map(|d| d.top).fold(ground, f32::max)
    }

    /// Whether a ball of radius `r` at `p` touches any collider.
    pub fn solid(&self, p: Vec3, r: f32) -> bool {
        self.colliders.iter().any(|c| c.hits(p, r))
    }

    /// The sea's level (its swell aside).
    pub fn water_level(&self) -> f32 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec3;
    use crate::nodes::{Dock, Hut};

    #[test]
    fn decks_are_walkable_above_the_ground_and_huts_collide() {
        let items = vec![
            Item::Terrain(Terrain::default()),
            Item::part(Dock { x: -18.0, z: 0.0, heading: 180.0, length: 6.0, ..Dock::default() }),
            Item::part(Hut { x: -5.0, z: 2.0, ..Hut::default() }),
        ];
        let g = Ground::of(&items);
        let t = Terrain::default();
        // out on the dock, over the sea: the planks, not the sea floor
        assert!(g.height_at(-22.0, 0.0) > 0.3, "{}", g.height_at(-22.0, 0.0));
        assert!(t.height_at(-22.0, 0.0) < 0.3);
        // beside the dock: the ground
        assert_eq!(g.height_at(-22.0, 3.0), t.height_at(-22.0, 3.0));
        assert!(g.height_at(-5.0, 2.2) > t.height_at(-5.0, 2.2) + 0.4, "the hut's deck");
        assert!(g.colliders.iter().any(|c| matches!(c, Collider::Box { .. })), "the hut's walls");
        // the walkable strips are there to ask for: the dock's and the hut's
        assert_eq!(g.decks().len(), 2);
        // the hut's roof and stilts block, not only its walls
        let hut = Hut { x: -5.0, z: 2.0, ..Hut::default() };
        let under = |x: f32, z: f32| t.height_at(x, z);
        let base = component_island_props::props::hut_base(glam::vec2(-5.0, 2.0), 0.0, 1.0, &under);
        assert!(g.solid(base + vec3(1.2, 1.35, 0.0), 0.05), "the eaves");
        let (sx, sz) = component_island_props::props::STILTS[0];
        let stilt = vec3(-5.0 + sx, base.y - 0.3, 2.0 + sz);
        assert!(g.solid(stilt, 0.05), "a stilt under the deck");
        let _ = hut;
        // the dock's edge posts and its crates block; the middle of its planks is clear
        let deck_top = g.height_at(-21.0, 0.0);
        // posts every 1 m from 0.3 m along (a 6 m dock), half a metre to each side
        assert!(g.solid(vec3(-21.3, deck_top + 0.1, 0.5), 0.05) && g.solid(vec3(-21.3, deck_top + 0.1, -0.5), 0.05), "the edge posts");
        assert!(!g.solid(vec3(-21.0, deck_top + 0.5, 0.0), 0.2), "the middle of the planks is walkable");
        assert!(g.solid(vec3(-18.0 - 6.0 + 0.7, deck_top + 0.2, 0.25), 0.05), "a crate at the end");
    }

    #[test]
    fn packages_add_things_each_their_own_set() {
        use crate::nodes::Bomb;
        let items = vec![Item::Terrain(Terrain::default())];
        set_items("test/items", "bombs", vec![Item::part(Bomb::default())]);
        set_items("test/items", "other", vec![Item::part(Bomb { thrown: 2.0, ..Bomb::default() })]);
        let (kept, _, key) = driven("test/items", &items);
        assert_eq!(kept.len(), 3, "the World's own and both packages' things");
        set_items("test/items", "bombs", Vec::new());
        let (kept, _, after) = driven("test/items", &items);
        assert_eq!(kept.len(), 2, "one owner's clearing leaves the other's");
        assert_ne!(key, after, "a change of things redraws");
    }

    #[test]
    fn a_driven_character_leaves_its_node_and_follows_the_drive() {
        use crate::nodes::Character;
        let hero = Character { name: "hero".into(), x: 1.0, z: 2.0, ..Character::default() };
        let items = vec![Item::Terrain(Terrain::default()), Item::part(hero.clone())];
        let (kept, extras, _) = driven("test/drive", &items);
        assert_eq!((kept.len(), extras.len()), (2, 0), "undriven: the node draws itself");
        let at = vec3(-4.0, 1.5, 3.0);
        drive("test/drive", "hero", Some(Drive { at, facing: 1.0, pose: "walk", t: 0.3, shown: true }));
        let (kept, extras, key) = driven("test/drive", &items);
        assert_eq!((kept.len(), extras.len()), (1, 1), "driven: drawn once, as an extra");
        assert_eq!(extras[0].colours, crate::character::colours(&hero), "in the node's own look");
        assert!(extras[0].transform.transform_point3(Vec3::ZERO).distance(at) < 1e-4);
        drive("test/drive", "hero", Some(Drive { at, facing: 1.0, pose: "walk", t: 0.4, shown: true }));
        assert_ne!(driven("test/drive", &items).2, key, "a new step redraws");
        drive("test/drive", "hero", None);
        assert_eq!(driven("test/drive", &items).1.len(), 0);
    }

    #[test]
    fn every_collider_kind_blocks() {
        let at = vec3(0.0, 1.0, 0.0);
        for c in [
            Collider::Box { centre: at, half: Vec3::splat(0.5), yaw: 0.7 },
            Collider::Post { foot: Vec3::ZERO, radius: 0.3, height: 2.0 },
            Collider::Ball { centre: at, radius: 0.5 },
        ] {
            assert!(c.hits(at + vec3(0.45, 0.0, 0.0), 0.2), "{c:?}");
            assert!(!c.hits(at + vec3(3.0, 0.0, 0.0), 0.2), "{c:?}");
        }
    }
}