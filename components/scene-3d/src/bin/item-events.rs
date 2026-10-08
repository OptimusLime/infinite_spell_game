//! item-events: what the shots in a scene's World raise and run (docs/composition.md): each Arrow's `impact` where
//! it comes down on the World's ground, then each of its item's effects there, with their times and places, as
//! JSON lines. `runtime.events` holds the same in the host; proofs read this without one. Run from the project root.
use component_scene_3d::nodes::Arrow;
use component_scene_3d::{hooks, scene_file};

fn main() -> Result<(), String> {
    let scene = std::env::args().nth(1).ok_or("item-events <scene>")?;
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let w = scene_file::load_world(&root, std::path::Path::new(&scene))?;
    let ground = hooks::Ground::of(&w.items);
    for arrow in w.items.iter().filter_map(|i| i.get::<Arrow>()) {
        for e in arrow.events(&|x, z| ground.height_at(x, z)) {
            println!("{}", serde_json::json!({ "item": arrow.item, "t": e.t, "name": e.name, "what": e.what, "at": e.at }));
        }
    }
    Ok(())
}
