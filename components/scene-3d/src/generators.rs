//! The Plant and Rock nodes as node-cache generators: the host finds them in scenes, keys each as the grown shape
//! is keyed (kind, its version and rules, seed; never placement or colour, so node and World share one entry), grows it in the background through markov-junior's kind cache, shows its
//! state in the Inspector, and can Keep it as a named asset (assets/plants/, assets/rocks/) that the node then
//! reads instead.
use std::path::Path;

use atelico_node_cache::{CachePolicy, Caching, Generator, Key};
use component_markov_junior::{encoding, Shape};

use crate::nodes::{Plant, Rock};

const PLACEMENT: [&str; 11] = ["x", "z", "yaw", "scale", "leaf", "leaf_light", "trunk", "trunk_light", "wood", "show", "moss"];

/// The grown shape's key: kind, the kind's version, its rules' hash, seed.
fn grown_key(kind: &str, seed: u32) -> Option<Result<Key, String>> {
    Some(component_markov_junior::kind(kind).and_then(|k| encoding::key(k, seed as u64)))
}

fn grow(project: &Path, kind: &str, seed: u32) -> Result<Shape, String> {
    match atelico_node_cache::shared() {
        Some(cache) => encoding::cached(&cache, kind, seed as u64).map(|s| (*s).clone()),
        None => component_markov_junior::kind(kind).and_then(|k| (k.grow)(seed as u64)).map_err(|e| format!("{}: {e}", project.display())),
    }
}

impl Generator for Plant {
    const NODE_TYPE: &'static str = "Plant";
    const FAMILY: &'static str = "plants";
    const ASSET_FIELD: Option<&'static str> = Some("asset");
    type Params = Plant;
    type Output = Shape;
    fn caching() -> Caching {
        Caching::new(CachePolicy::Disk, 1).ignoring(&PLACEMENT)
    }
    fn key(p: &Plant) -> Option<Result<Key, String>> {
        grown_key(&p.kind, p.seed)
    }
    fn asset(p: &Plant) -> Option<&str> {
        (!p.asset.is_empty()).then_some(p.asset.as_str())
    }
    fn generate(p: &Plant, project: &Path) -> Result<Shape, String> {
        grow(project, &p.kind, p.seed)
    }
}

impl Generator for Rock {
    const NODE_TYPE: &'static str = "Rock";
    const FAMILY: &'static str = "rocks";
    const ASSET_FIELD: Option<&'static str> = Some("asset");
    type Params = Rock;
    type Output = Shape;
    fn caching() -> Caching {
        Caching::new(CachePolicy::Disk, 1).ignoring(&["x", "z", "yaw", "scale", "stone", "stone_light", "moss", "show"])
    }
    fn key(p: &Rock) -> Option<Result<Key, String>> {
        grown_key(&p.kind, p.seed)
    }
    fn asset(p: &Rock) -> Option<&str> {
        (!p.asset.is_empty()).then_some(p.asset.as_str())
    }
    fn generate(p: &Rock, project: &Path) -> Result<Shape, String> {
        grow(project, &p.kind, p.seed)
    }
}

atelico_node_cache::generator!(PLANT, Plant);
atelico_node_cache::generator!(ROCK, Rock);

#[cfg(test)]
mod tests {
    use super::*;
    use atelico_node_cache::generator::key_of;

    /// A Plant or Rock node is keyed as the grown shape is (kind, the kind's version, its rules' hash, seed), so an
    /// edited or bumped kind is a new key and the node and the World share one cache entry; placement and colour
    /// are not in it.
    #[test]
    fn plant_and_rock_keys_are_the_grown_shapes_keys() {
        let root = std::env::temp_dir();
        let plant = Plant { kind: "palm".into(), seed: 7, ..Plant::default() };
        let (key, _) = key_of::<Plant>(&root, &Plant::caching(), &plant).unwrap();
        let palm = component_markov_junior::kind("palm").unwrap();
        assert_eq!(key, encoding::key(palm, 7).unwrap());
        let moved = Plant { x: 4.0, leaf: crate::nodes::hex(0xff0000), ..plant.clone() };
        assert_eq!(key_of::<Plant>(&root, &Plant::caching(), &moved).unwrap().0, key, "placement and colour are not in the key");
        let rock = Rock { seed: 3, ..Rock::default() };
        let (key, _) = key_of::<Rock>(&root, &Rock::caching(), &rock).unwrap();
        assert_eq!(key, encoding::key(component_markov_junior::kind(&rock.kind).unwrap(), 3).unwrap());
    }
}
