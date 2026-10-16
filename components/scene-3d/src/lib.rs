//! scene-3d: the 3D starter kit. Scene node types (nodes.rs) each draw on their own: a `World` is the 3D viewport
//! and holds `Sky` (with `Palette`s), `Sea`, `Terrain`, `Plant`, `Hut`, `Dock`, `Boat` and `Camera` nodes. The
//! World's builder (lua/nodes/scene_3d.luau) gathers them into a `scene_3d` paint node; this component parses it
//! into typed data, assembles the frame (world.rs) and draws it cel-shaded with wgpu offscreen (render.rs,
//! world.wgsl) into the node's box.
//!
//! Luau: `scene_3d.snapshot(world, items, w, h, time, file, camera?)` for stills at exact times, and
//! `scene_3d.height_at(terrain, x, z)` for walking on the ground (Rust: `Terrain::height_at`).
//!
//! Other packages plug into a World by its node path (hooks.rs): extra meshes drawn with it, a camera pose, and
//! what it last drew, whose `Ground` answers walkable height (decks included) and colliders.
pub use atelico_render_3d::{gi, look, material, mesh, part, render, world_part};
pub use component_toon_character as character;
pub mod generators;
pub mod hooks;
pub mod nodes;
pub mod parts;
pub mod probes;
pub mod uniforms;
pub mod props;
pub mod scene_file;
// a scene drawn through a look graph, for the Shader Preview panel
pub mod look_preview;
mod picture;
pub mod stills;
pub use component_water::sdf;
/// The swell and boats on it (water's and island-props' own, under the old path).
pub mod sea {
    pub use component_island_props::boat::{afloat, boat_at};
    pub use component_water::{float_on, waves, WAVES};
}
pub use component_sky as sky;
pub use component_terrain_generation::world as terrain;
pub mod world;
#[cfg(test)]
mod pixel_lens;

use std::collections::BTreeMap;
use std::sync::Mutex;

use atelico_component::mlua::{self, Lua, LuaSerdeExt, Value as LuaValue};
use atelico_component::tiny_skia::{FilterQuality, Pixmap, PixmapPaint};
use atelico_component::{register, Component, Surface, Value};
use serde::Deserialize;
// linked for their kinds (palm, bush, tree; sea-cliff, sea-stack, boulders), which register with markov-junior
use component_plant_generation as _;
use component_rock_generation as _;

pub use nodes::{Boat, Camera, Character, Dock, Grade, Hut, Item, Palette, Plant, Rgb, Rock, Sea, Sky, Terrain, World};

register!(Component { name: "scene-3d", lua: install, paint, draw, ..Component::NONE });

/// `{kind = "scene_3d", id, world, items = [{type, ...props}], camera?, time?}`: what a World node builds.
#[derive(Deserialize)]
struct PaintNode {
    #[serde(default)]
    id: String,
    #[serde(default)]
    world: World,
    /// (parsed apart, once per change: `items_of`)
    #[serde(skip_deserializing)]
    items: Vec<Item>,
    /// The World's environment at this moment (`{type = "Environment", ...}`), beside its items so they stay the same
    /// table, and the same JSON, while only the time moves.
    environment: Option<Value>,
    /// Where its items were handed over (`scene_3d.keep_items(at, items)`, once each time they change) and which
    /// handing that was: the node then carries no items at all.
    kept: Option<(String, f64)>,
    /// The Camera the World's `camera` names, when it names one.
    camera: Option<Camera>,
    time: Option<f32>,
    /// The scene's `pixel`: drawn as pixel art, that many of the scene's pixels to one (1: not).
    pixel: Option<f32>,
}

/// A list, or an empty table (Luau writes `{}` for an empty list).
fn list<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Item>, D::Error> {
    use serde::de::Error;
    // Luau writes an empty list as an empty table; a real list says which item is wrong and why
    match Value::deserialize(d)? {
        Value::Array(items) => items.into_iter().enumerate().map(|(i, v)| {
            let kind = v.get("type").and_then(Value::as_str).unwrap_or("?").to_string();
            serde_json::from_value(v).map_err(|e| D::Error::custom(format!("item {i} ({kind}): {e}")))
        }).collect(),
        Value::Object(m) if m.is_empty() => Ok(Vec::new()),
        other => Err(D::Error::custom(format!("items: a list, not {other}"))),
    }
}

/// A JSON value hashed (the change detection of a World's node: its items, camera, time, look).
fn hash_value(v: &Value, h: &mut std::collections::hash_map::DefaultHasher) {
    use std::hash::Hash;
    match v {
        Value::Null => 0u8.hash(h),
        Value::Bool(b) => b.hash(h),
        Value::Number(n) => n.as_f64().map(f64::to_bits).hash(h),
        Value::String(s) => s.hash(h),
        Value::Array(a) => {
            a.len().hash(h);
            a.iter().for_each(|x| hash_value(x, h));
        }
        Value::Object(o) => {
            o.len().hash(h);
            for (k, x) in o {
                k.hash(h);
                hash_value(x, h);
            }
        }
    }
}

/// A World's items as last parsed, by node id: the hash of their JSON and the typed list. A playing frame hands the
/// same items again (only the time and the environment move): they are hashed, not parsed again.
static ITEMS: Mutex<BTreeMap<String, (u64, std::sync::Arc<Vec<Item>>)>> = Mutex::new(BTreeMap::new());

/// Items handed over from Luau when they change (`scene_3d.keep_items`), by where: the handing's number and the typed
/// list. A World's node names them instead of carrying 800 items as JSON every frame.
static KEPT: Mutex<BTreeMap<String, (u64, std::sync::Arc<Vec<Item>>)>> = Mutex::new(BTreeMap::new());

fn kept_items(at: &str, n: u64) -> Result<std::sync::Arc<Vec<Item>>, String> {
    match KEPT.lock().unwrap().get(at) {
        Some((k, items)) if *k == n => Ok(items.clone()),
        _ => Err(format!("no items kept at {at} (handing {n})")),
    }
}

fn items_of(id: &str, v: Option<&Value>) -> Result<(u64, std::sync::Arc<Vec<Item>>), String> {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    if let Some(v) = v {
        hash_value(v, &mut h);
    }
    let key = h.finish();
    let mut cache = ITEMS.lock().unwrap();
    if let Some((k, items)) = cache.get(id) {
        if *k == key {
            return Ok((key, items.clone()));
        }
    }
    let items = std::sync::Arc::new(match v {
        Some(v) => list(v.clone()).map_err(|e| e.to_string())?,
        None => Vec::new(),
    });
    cache.insert(id.to_string(), (key, items.clone()));
    Ok((key, items))
}

#[derive(PartialEq)]
struct FrameKey { root: std::path::PathBuf, layer: u32, id: String, revision: u64, gpu: bool }
impl picture::ViewKey for FrameKey { fn same_view(&self, other:&Self)->bool { self.root==other.root && self.layer==other.layer && self.id==other.id && self.gpu==other.gpu } }
static FRAMES: Mutex<picture::Cache<FrameKey>> = Mutex::new(picture::Cache::new());
struct Prepared { picture: std::sync::Arc<picture::Picture>, place: Placement, pixel_art: bool }

fn prepare(node: &Value, rect: atelico_component::Rect, scale: f32, time: f64, gpu: bool) -> Result<Prepared, String> {
    let parsed = PaintNode::deserialize(node).map_err(|e| e.to_string()).and_then(|mut n| {
        let (key, items) = match &n.kept {
            // (a Luau number: whole, but a float in JSON)
            Some((at, k)) => (*k as u64, kept_items(at, *k as u64)?),
            None => items_of(&n.id, node.get("items"))?,
        };
        let mut all = (*items).clone();
        if let Some(env) = &n.environment {
            all.push(serde_json::from_value(env.clone()).map_err(|e| format!("environment: {e}"))?);
        }
        n.items = all;
        Ok((n, key))
    });
    let (n, items_key) = parsed?;
    let time = n.camera.as_ref().filter(|c| c.time >= 0.0).map(|c| c.time as f32).or(n.time).unwrap_or(time as f32);
    let pixel_art = n.pixel.is_some_and(|p| p > 1.0);
    let place = Placement::of(rect, scale, pixel_art);
    let (w, h) = place.size;
    let (items, extras, driven) = hooks::driven(&n.id, &n.items);
    // a grid of light probes baked in the background draws once it is ready
    let baked = probes::READY.load(std::sync::atomic::Ordering::Relaxed);
    // (a hash of the node as it came, not its typed items printed: printing them every frame cost milliseconds)
    let key = {
        use std::hash::{Hash, Hasher};
        let mut hs = std::collections::hash_map::DefaultHasher::new();
        // (the items by their hash, the rest of the node as it came)
        items_key.hash(&mut hs);
        for (k, v) in node.as_object().into_iter().flatten().filter(|(k, _)| k.as_str() != "items") {
            k.hash(&mut hs);
            hash_value(v, &mut hs);
        }
        (format!("{:?}", hooks::pose(&n.id)), driven, baked, time.to_bits(), w, h, rect.w.to_bits(), rect.h.to_bits()).hash(&mut hs);
        hs.finish()
    };
    let mut frames = FRAMES.lock().unwrap();
    let cache_key = FrameKey { root: picture::project_root(), layer: atelico_component::layer(), id: n.id.clone(), revision: key, gpu };
    let picture = frames.get_or_render(cache_key, || {
        let mut frame = world::frame(&n.world, &items, n.camera.as_ref(), hooks::pose(&n.id), &extras, time, w as f32 / h as f32);
        // pixel art at the scene's own resolution over the pixel size, whatever this surface is scaled to
        frame.pixel = world::pixel_look(&items, n.camera.as_ref(), n.pixel.unwrap_or(1.0), rect.w, rect.h);
        let light = if frame.moment.sun.y > -0.05 { frame.moment.sun } else { frame.moment.moon };
        // (the ground of the World's items, found again only when they change: not every playing frame)
        let mut ground = {
            static GROUNDS: Mutex<BTreeMap<String, (u64, hooks::Ground)>> = Mutex::new(BTreeMap::new());
            let mut grounds = GROUNDS.lock().unwrap();
            match grounds.get(&n.id) {
                Some((k, g)) if *k == items_key => g.clone(),
                _ => {
                    let g = hooks::Ground::of(&n.items);
                    grounds.insert(n.id.clone(), (items_key, g.clone()));
                    g
                }
            }
        };
        ground.colliders.extend(frame.built.canopies.iter().map(|(c, r, _)| hooks::Collider::Ball { centre: *c, radius: *r }));
        let seen = hooks::Seen { items: n.items.clone(), ground, light: frame.moment.tint.light, shadow: frame.moment.tint.shadow, towards_light: light, aspect: w as f32 / h.max(1) as f32 };
        // (on the GPU, the picture stays there when the host can show it as it is: no readback)
        if gpu {
            if let Some(on) = world::render_texture_in(&n.id, &frame, w, h)? {
                let mut picture = picture::Picture::snapshot(on, w, h)?;
                picture.seen = Some(seen);
                return Ok(picture);
            }
        }
        let mut picture = world::render_in(&n.id, &frame, w, h).and_then(|px| pixels_of(&px, w, h)).map(picture::Picture::pixels_only)?;
        picture.seen = Some(seen);
        Ok(picture)
    })?;
    // A cache hit can follow another project or capture using the same hook id.
    // Restore the pictured scene's typed ground/light, not the last producer's state.
    if let Some(seen) = &picture.seen { hooks::remember(&n.id, seen.clone()); }
    Ok(Prepared { picture, place, pixel_art })
}

fn source(node: &Value) -> String { format!("scene_3d {}", node.get("id").and_then(Value::as_str).unwrap_or("?")) }
fn paint(node: &Value, s: &mut Surface) -> bool {
    if node.get("kind").and_then(Value::as_str) != Some("scene_3d") { return false; }
    let result = prepare(node, s.rect, s.scale, s.time, atelico_component::overlay::enabled()).and_then(|p| p.paint(s));
    match result { Ok(()) => atelico_component::clear(&source(node)), Err(e) => atelico_component::error(&source(node), e) }
    true
}
fn draw(node: &Value, on: &mut atelico_component::On) -> bool {
    if node.get("kind").and_then(Value::as_str) != Some("scene_3d") { return false; }
    let result = prepare(node, on.rect, on.canvas.scale(), on.time, true).and_then(|p| p.draw(on));
    match result { Ok(()) => atelico_component::clear(&source(node)), Err(e) => atelico_component::error(&source(node), e) }
    true
}
impl Prepared {
    fn fits(&self, rect: atelico_component::Rect, scale: f32) -> bool {
        (rect.w * scale - self.place.size.0 as f32).abs() < 1.5 && (rect.h * scale - self.place.size.1 as f32).abs() < 1.5
    }
    fn paint(&self, s: &mut Surface) -> Result<(), String> {
        let (w, h) = self.place.size;
        let fits = self.fits(s.rect, s.scale);
        if let Some(on) = &self.picture.gpu {
            if atelico_component::overlay::enabled() {
                let (x0, y0) = ((s.rect.x * s.scale).round() as i32, (s.rect.y * s.scale).round() as i32);
                let at = if fits || self.pixel_art { [x0, y0, w as i32, h as i32] } else { [x0, y0, ((s.rect.x + s.rect.w) * s.scale).round() as i32 - x0, ((s.rect.y + s.rect.h) * s.scale).round() as i32 - y0] };
                let (fx, fy) = (at[2] as f32 / w as f32, at[3] as f32 / h as f32);
                let d = on.art;
                let dst = [at[0] + (d[0] as f32 * fx).round() as i32, at[1] + (d[1] as f32 * fy).round() as i32, (d[2] as f32 * fx).round() as i32, (d[3] as f32 * fy).round() as i32];
                if dst != at {
                    if let Some(r) = atelico_component::tiny_skia::Rect::from_xywh(at[0] as f32, at[1] as f32, at[2] as f32, at[3] as f32) {
                        let black = atelico_component::tiny_skia::Paint { shader: atelico_component::tiny_skia::Shader::SolidColor(atelico_component::tiny_skia::Color::BLACK), ..Default::default() };
                        s.pixmap.fill_rect(r, &black, atelico_component::tiny_skia::Transform::identity(), s.clip);
                    }
                }
                let src = [d[0] as f32 / w as f32, d[1] as f32 / h as f32, d[2] as f32 / w as f32, d[3] as f32 / h as f32];
                if atelico_component::overlay::offer_part(s.pixmap, s.clip, &on.texture, src, dst, on.nearest) { return Ok(()); }
            }
        }
        let pm = self.picture.pixels()?;
        if fits { copy_in(s.pixmap, ((s.rect.x * s.scale).round() as i32, (s.rect.y * s.scale).round() as i32), &pm, s.clip); }
        else { s.pixmap.draw_pixmap(0, 0, pm.as_ref().as_ref(), &PixmapPaint { quality: if self.pixel_art { FilterQuality::Nearest } else { FilterQuality::Bilinear }, ..Default::default() }, self.place.transform, s.clip); }
        Ok(())
    }
    fn draw(&self, on: &mut atelico_component::On) -> Result<(), String> {
        let scale = on.canvas.scale();
        let transform = if self.fits(on.rect, scale) { atelico_component::tiny_skia::Transform::from_translate((on.rect.x * scale).round(), (on.rect.y * scale).round()) } else { self.place.transform };
        let filter = if self.pixel_art { atelico_component::atelico_paint::Filter::Nearest } else { atelico_component::atelico_paint::Filter::Bilinear };
        if let Some(gpu) = &self.picture.gpu {
            if on.canvas.draw_texture(&gpu.texture, transform, filter, 1.) { return Ok(()); }
        }
        // The editor's offscreen scene can use a raster Canvas even when its window is GPU-backed.
        // Offer the retained texture to that surface's collector before materializing CPU pixels.
        // This also preserves the Surface path's rounded placement, pixel bars and clip coverage.
        if let Some((pixmap, clip)) = on.canvas.legacy(on.rect) {
            return self.paint(&mut Surface { pixmap, clip, scale, rect: on.rect, assets: on.assets, time: on.time });
        }
        self.picture.draw(on.canvas, transform, filter)
    }
}

/// RGBA8 rows (w x h) as a picture.
fn pixels_of(px: &[u8], w: u32, h: u32) -> Result<Pixmap, String> {
    let mut pm = Pixmap::new(w, h).ok_or("no picture of that size")?;
    pm.data_mut().copy_from_slice(px);
    Ok(pm)
}

/// Where a World's picture goes on a surface: its size in pixels, and the transform that lays it on the rect.
/// `src` (opaque: a 3D picture fills its every pixel) copied into `dst` with its top left at `at`, clipped to `dst`
/// and to `clip` (a pixel the clip covers in part is mixed by its coverage).
fn copy_in(dst: &mut Pixmap, at: (i32, i32), src: &Pixmap, clip: Option<&atelico_component::tiny_skia::Mask>) {
    let (dw, dh, sw, sh) = (dst.width() as i32, dst.height() as i32, src.width() as i32, src.height() as i32);
    let (x0, y0, x1, y1) = (at.0.max(0), at.1.max(0), (at.0 + sw).min(dw), (at.1 + sh).min(dh));
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let n = ((x1 - x0) * 4) as usize;
    let mask = clip.filter(|m| m.width() as i32 == dw && m.height() as i32 == dh).map(|m| m.data());
    if clip.is_some() && mask.is_none() {
        return;
    }
    let (from, to) = (src.data(), dst.data_mut());
    for row in y0..y1 {
        let s = (((row - at.1) * sw + (x0 - at.0)) * 4) as usize;
        let d = ((row * dw + x0) * 4) as usize;
        match mask {
            None => to[d..d + n].copy_from_slice(&from[s..s + n]),
            Some(m) => {
                let mrow = &m[(row * dw + x0) as usize..(row * dw + x1) as usize];
                for (i, &c) in mrow.iter().enumerate() {
                    let (a, b) = (s + i * 4, d + i * 4);
                    match c {
                        0 => {}
                        255 => to[b..b + 4].copy_from_slice(&from[a..a + 4]),
                        c => {
                            for k in 0..4 {
                                to[b + k] = ((from[a + k] as u32 * c as u32 + to[b + k] as u32 * (255 - c as u32) + 127) / 255) as u8;
                            }
                        }
                    }
                }
            }
        }
    }
}

struct Placement {
    size: (u32, u32),
    transform: atelico_component::tiny_skia::Transform,
}

impl Placement {
    /// A 3D picture is drawn at the surface's scale (0.5 to 2, at most 1600 pixels wide) and stretched to the rect.
    /// Pixel art is drawn at the rect's own size in screen pixels, uncapped, and laid on whole screen pixels one to
    /// one, so the lens's whole-number scale is what the screen shows (never resampled by a fraction).
    fn of(rect: atelico_component::Rect, scale: f32, pixel_art: bool) -> Self {
        use atelico_component::tiny_skia::Transform;
        if pixel_art {
            let (x, y) = ((rect.x * scale).round(), (rect.y * scale).round());
            let size = (((rect.w * scale).round() as u32).max(1), ((rect.h * scale).round() as u32).max(1));
            return Self { size, transform: Transform::from_translate(x, y) };
        }
        let res = scale.clamp(0.5, 2.0);
        let (mut w, mut h) = ((rect.w * res).max(1.0), (rect.h * res).max(1.0));
        let cap = 1600.0 / w.max(h);
        if cap < 1.0 {
            (w, h) = (w * cap, h * cap);
        }
        let (w, h) = (w as u32, h as u32);
        let transform = Transform::from_translate(rect.x, rect.y).post_scale(scale, scale).pre_scale(rect.w / w as f32, rect.h / h as f32);
        Self { size: (w, h), transform }
    }
}

/// The particle stage (scenes/particle-stage.scene.luau: the island's west beach at dusk) drawing particle graph
/// `graph` (graphs/<graph>.graph.yaml; one node of it alone, or the whole when `node` is empty) at `t` seconds into
/// it, through the stage's preview camera: RGBA8 pixels, `w` x `h`. The same stage the graph editor previews each
/// node on, so a capture at a keyframe is what the editor shows at that moment.
pub fn particle_frame(root: &std::path::Path, graph: &str, node: &str, t: f32, w: u32, h: u32) -> Result<Vec<u8>, String> {
    component_particles::part::use_root(root);
    let mut stage = scene_file::load_world(root, std::path::Path::new("scenes/particle-stage.scene.luau"))?;
    stage.items.push(Item::part(component_particles::part::ParticleEffect { graph: graph.into(), node: node.into(), x: -9.7, z: -7.0, ..Default::default() }));
    let mut camera = stage.cameras.get("preview").cloned().ok_or("the particle stage has no preview camera")?;
    camera.time = t as f64;
    let frame = world::frame(&stage.world, &stage.items, Some(&camera), None, &[], t, w as f32 / h as f32);
    world::render(&frame, w, h)
}

/// A still of a world at `time`, as a PNG file.
pub fn snapshot(world: &World, items: &[Item], camera: Option<&Camera>, w: u32, h: u32, time: f32, file: &std::path::Path) -> Result<(), String> {
    snapshot_as(world, items, camera, w, h, time, 1.0, file)
}

/// A still of a world at `time` as a PNG file, drawn as pixel art with `pixel` of its pixels to one (1: not).
#[allow(clippy::too_many_arguments)]
pub fn snapshot_as(world: &World, items: &[Item], camera: Option<&Camera>, w: u32, h: u32, time: f32, pixel: f32, file: &std::path::Path) -> Result<(), String> {
    let mut frame = world::frame(world, items, camera, None, &[], time, w as f32 / h as f32);
    frame.pixel = world::pixel_look(items, camera, pixel, w as f32, h as f32);
    let png = world::render(&frame, w, h).and_then(|px| atelico_gpu::png(&px, w, h))?;
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(file, png).map_err(|e| e.to_string())
}

fn install(lua: &Lua) -> mlua::Result<()> {
    // keeps the Plant and Rock generators linked into the host
    std::hint::black_box((&generators::PLANT, &generators::ROCK));
    component_fire::link();
    component_arrow::link();
    component_particles::preview::link();
    component_creatures::link();
    component_founding::link();
    let t = lua.create_table()?;
    t.set(
        "snapshot",
        lua.create_function(|lua, (world, items, w, h, time, file, camera): (LuaValue, LuaValue, u32, u32, f32, String, Option<LuaValue>)| {
            let world: World = lua.from_value(world)?;
            let items: Vec<Item> = lua.from_value(items)?;
            let camera: Option<Camera> = camera.map(|c| lua.from_value(c)).transpose()?;
            snapshot(&world, &items, camera.as_ref(), w, h, time, std::path::Path::new(&file)).map_err(mlua::Error::external)
        })?,
    )?;
    // scene_3d.still({file, camera, hour, t, width, height}): a still of a scene file's World at a moment, as a job
    // ({state = "running", what} | {state = "done", file} | {state = "failed", error}; ask again until done)
    t.set(
        "still",
        lua.create_function(|lua, want: LuaValue| {
            let want: stills::Want = lua.from_value(want)?;
            lua.to_value(&stills::still(&std::env::current_dir().unwrap_or_default(), &want))
        })?,
    )?;
    // the World's environment at this moment (its clock, sky, palettes and looks, a camera's hour): what the World
    // draws with and what the host writes to runtime.environment.<world> for clocks and HUDs to read
    t.set(
        "environment",
        lua.create_function(|lua, (items, time, camera): (LuaValue, f32, Option<LuaValue>)| {
            let items: Vec<Item> = lua.from_value(items)?;
            let camera: Option<Camera> = camera.map(|c| lua.from_value(c)).transpose()?;
            let t = camera.as_ref().filter(|c| c.time >= 0.0).map_or(time, |c| c.time as f32);
            lua.to_value(&world::environment_of(&items, camera.as_ref(), t))
        })?,
    )?;
    t.set(
        "height_at",
        lua.create_function(|lua, (terrain, x, z): (LuaValue, f32, f32)| {
            let terrain: Terrain = lua.from_value(terrain)?;
            Ok(terrain.height_at(x, z))
        })?,
    )?;
    // (the paint node takes the environment beside its items: a host built before that reads only `items`, and the
    // Luau appends it there for it)
    t.set("environment_beside", true)?;
    // scene_3d.keep_items(at, items) -> n: a World's items typed once, when they change, kept by `at`; its paint node
    // names them ({kept = {at, n}}) instead of carrying them
    t.set(
        "keep_items",
        lua.create_function(|lua, (at, items): (String, LuaValue)| {
            static HANDED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            let items: Vec<Item> = lua.from_value(items)?;
            let n = HANDED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            KEPT.lock().unwrap().insert(at, (n, std::sync::Arc::new(items)));
            Ok(n)
        })?,
    )?;
    lua.globals().set("scene_3d", t)
}

#[cfg(test)]
mod placement_tests {
    use super::*;

    /// Pixel art in the editor is laid one picture pixel to one screen pixel at any surface scale and any rect (a
    /// retina window, a fractional panel, a view wider than 1600), so the lens's whole-number scale is exactly what
    /// the screen shows; a 3D picture is still stretched to its rect.
    #[test]
    fn pixel_art_lands_on_whole_screen_pixels_in_the_editor() {
        for (scale, rect) in [(1.0, (12.3, 40.6, 1003.7, 611.3)), (1.5, (0.0, 33.0, 1200.0, 675.0)), (2.0, (8.0, 8.0, 1800.0, 1000.0)), (3.0, (1.0, 2.0, 640.0, 360.0))] {
            let r = atelico_component::Rect { x: rect.0, y: rect.1, w: rect.2, h: rect.3 };
            let p = Placement::of(r, scale, true);
            let t = p.transform;
            assert_eq!((t.sx, t.sy, t.kx, t.ky), (1.0, 1.0, 0.0, 0.0), "one picture pixel to one screen pixel at scale {scale}");
            assert_eq!((t.tx.fract(), t.ty.fract()), (0.0, 0.0), "on whole screen pixels at scale {scale}");
            assert_eq!(p.size, ((rect.2 * scale).round() as u32, (rect.3 * scale).round() as u32), "the rect's own size in screen pixels, uncapped");
        }
        let p = Placement::of(atelico_component::Rect { x: 0.0, y: 0.0, w: 1800.0, h: 1000.0 }, 2.0, false);
        assert!(p.size.0 <= 1600 && (p.transform.sx * p.size.0 as f32 - 3600.0).abs() < 0.5, "3D stays capped and stretched");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atelico_data::Data;

    /// What each pixel of a `w` x `h` view of `frame` shows, by ray-marching the terrain: 0 sand (and paths), 1 grass,
    /// 2 rock, 3 sea, None the sky or too far (what stands on the ground is not told apart: most pixels are ground).
    fn ground_regions(frame: &world::Frame, items: &[Item], w: u32, h: u32) -> Vec<Option<u8>> {
        let terrain = items.iter().find_map(|i| if let Item::Terrain(t) = i { Some(t.clone()) } else { None }).unwrap();
        let inv = frame.view_proj.inverse();
        let mut out = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                let ndc = glam::vec2((x as f32 + 0.5) / w as f32 * 2.0 - 1.0, 1.0 - (y as f32 + 0.5) / h as f32 * 2.0);
                let far = inv.project_point3(ndc.extend(1.0));
                let dir = (far - frame.eye).normalize();
                let (mut t, mut hit) = (0.5f32, None);
                while t < 160.0 {
                    let p = frame.eye + dir * t;
                    let ground = terrain.height_at(p.x, p.z);
                    if p.y <= ground.max(0.0) {
                        hit = Some(if ground < 0.02 { 3 } else { match terrain.ground_kind(p.x, p.z) { Some(1) => 1, Some(2) => 2, _ => 0 } });
                        break;
                    }
                    t += (t * 0.01).max(0.05);
                }
                out.push(hit);
            }
        }
        out
    }

    /// sRGB bytes to OKLab.
    fn oklab(c: [u8; 3]) -> glam::Vec3 {
        let l = |x: u8| { let x = x as f32 / 255.0; if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) } };
        let (r, g, b) = (l(c[0]), l(c[1]), l(c[2]));
        let lm = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
        let mm = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
        let sm = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
        glam::vec3(0.2104542553 * lm + 0.7936177850 * mm - 0.0040720468 * sm, 1.9779984951 * lm - 2.4285922050 * mm + 0.4505937099 * sm, 0.0259040371 * lm + 0.7827717662 * mm - 0.8086757660 * sm)
    }

    /// The island's back, away from the sun, by day and at dusk: sand, grass, rock and sea each keep their own value
    /// and hue (their mean colours stay apart in OKLab: every pair at least 0.045, lightness or hue (the closest, sand and grass lit by day, 0.05; at dusk sand and grass went from 0.158 to 0.197, grass and sea from 0.106 to 0.115, grass and rock stayed 0.06)), so the far side
    /// reads. The views are saved as target/scene-3d/back-*.png.
    #[test]
    fn the_islands_back_keeps_its_grounds_apart() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
        // the looks (looks/*.cube) are read from the project
        std::env::set_current_dir(&root).unwrap();
        let w = scene_file::load_world(&root, std::path::Path::new("scenes/island.scene.luau")).unwrap();
        let (pw, ph) = (320u32, 180u32);
        let names = ["sand", "grass", "rock", "sea"];
        let mut report = Vec::new();
        let mut worst = f32::MAX;
        for cam in ["back-side", "back-side-dusk"] {
            let camera = w.cameras[cam].clone();
            let frame = world::frame(&w.world, &w.items, Some(&camera), None, &[], 0.0, pw as f32 / ph as f32);
            let px = world::render(&frame, pw, ph).unwrap();
            let dir = root.join("target/scene-3d");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("{cam}.png")), atelico_gpu::png(&px, pw, ph).unwrap()).unwrap();
            let regions = ground_regions(&frame, &w.items, pw, ph);
            let mask: Vec<u8> = regions.iter().flat_map(|r| match r { Some(0) => [240, 200, 120, 255], Some(1) => [60, 180, 60, 255], Some(2) => [120, 90, 140, 255], Some(3) => [40, 80, 200, 255], _ => [0, 0, 0, 255] }).collect();
            std::fs::write(dir.join(format!("{cam}-grounds.png")), atelico_gpu::png(&mask, pw, ph).unwrap()).unwrap();
            let mut sums = [(glam::Vec3::ZERO, 0usize); 4];
            for (i, r) in regions.iter().enumerate() {
                if let Some(k) = r {
                    sums[*k as usize].0 += oklab([px[i * 4], px[i * 4 + 1], px[i * 4 + 2]]);
                    sums[*k as usize].1 += 1;
                }
            }
            let means: Vec<Option<glam::Vec3>> = sums.iter().map(|(s, n)| (*n > 40).then(|| *s / *n as f32)).collect();
            for a in 0..4 {
                for b in a + 1..4 {
                    if let (Some(x), Some(y)) = (means[a], means[b]) {
                        let d = x.distance(y);
                        worst = worst.min(d);
                        report.push(format!("{cam} {}-{}: {d:.3} ({:.2} {:.2} {:.2} / {:.2} {:.2} {:.2})", names[a], names[b], x.x, x.y, x.z, y.x, y.y, y.z));
                    }
                }
            }
        }
        eprintln!("{}", report.join("\n"));
        assert!(worst >= 0.045, "grounds that blend on the island's back:\n{}", report.join("\n"));
    }

    /// A particle graph on its stage, isolated and testable: the same frame every time at the same moment; the whole
    /// effect is exactly its nodes' particles together (what each node's preview shows is its share of the whole);
    /// and a capture at a keyframe is the frame playback shows at that moment.
    #[test]
    fn a_particle_graph_draws_the_same_at_the_same_moment_and_its_nodes_make_the_whole() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
        let (w, h) = (160, 90);
        let a = particle_frame(&root, "bomb-explosion", "", 0.6, w, h).unwrap();
        assert_eq!(a, particle_frame(&root, "bomb-explosion", "", 0.6, w, h).unwrap(), "deterministic at t");
        assert_ne!(a, particle_frame(&root, "bomb-explosion", "", 1.8, w, h).unwrap(), "and it moves");
        // the whole is its nodes together
        let path = root.join("graphs/bomb-explosion.graph.yaml");
        let whole = component_particles::graph::effect_in(&path, None).unwrap();
        let g: atelico_graph::Graph = atelico_graph::Graph::from_value(&serde_yaml_ng::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()).unwrap();
        for t in [0.05f32, 0.6, 1.8, 2.8] {
            let mut parts: Vec<_> = g.nodes.iter().filter(|(_, n)| n.ty == "emitter").flat_map(|(id, _)| component_particles::graph::effect_in(&path, Some(id)).unwrap().sprites(t, glam::Vec3::ZERO, 1.0)).collect();
            let mut all = whole.sprites(t, glam::Vec3::ZERO, 1.0);
            let key = |s: &component_particles::Sprite| (s.at.x.to_bits(), s.at.y.to_bits(), s.at.z.to_bits(), s.cell);
            parts.sort_by_key(key);
            all.sort_by_key(key);
            assert_eq!(parts, all, "at {t} s the nodes' particles are the whole's");
        }
        // a capture at each keyframe is playback's frame then
        let timeline: atelico_timeline::TimelineFile = atelico_data::file::load(&root.join("graphs/bomb-explosion.timeline.yaml")).unwrap();
        for (name, k) in &timeline.keys {
            let playback = particle_frame(&root, "bomb-explosion", "", k.t as f32, w, h).unwrap();
            let stage = scene_file::load_world(&root, std::path::Path::new("scenes/particle-stage.scene.luau")).unwrap();
            let mut items = stage.items.clone();
            items.push(Item::part(component_particles::part::ParticleEffect { graph: "bomb-explosion".into(), x: -9.7, z: -7.0, ..Default::default() }));
            let mut cam = stage.cameras["preview"].clone();
            cam.time = k.t;
            let f = world::frame(&stage.world, &items, Some(&cam), None, &[], k.t as f32, w as f32 / h as f32);
            assert_eq!(world::render(&f, w, h).unwrap(), playback, "keyframe {name}");
        }
    }

    /// Pixel art holds still while the camera slides by a fraction of a pixel a frame (the grid of pixels is the
    /// world's, drawn without antialiasing): once the picture's whole-pixel moves are taken out, under a tenth as
    /// many pixels change from frame to frame as in the same shot without the look, and under a quarter as many change
    /// and change back; and its colours come from its palette. Frames 5 and 6: target/scene-3d/pixel-art-*.png.
    #[test]
    fn pixel_art_does_not_swim_as_the_camera_slides() {
        let world = World { stage: true, bands: 3, outline: 1.0, ..World::default() };
        let items = vec![
            Item::TimeOfDay(crate::nodes::TimeOfDay { hour: 13.0, ..Default::default() }),
            Item::Sky(Sky::default()),
            Item::part(crate::nodes::Swatch { material: "stone".into(), x: -1.2, z: 0.0, ..Default::default() }),
            Item::part(crate::nodes::Swatch { material: "wood-painted".into(), x: 1.2, z: 0.4, ..Default::default() }),
        ];
        let (w, h) = (160u32, 90u32);
        let shots = |pixel: bool| -> Vec<Vec<u8>> {
            (0..24)
                .map(|k| {
                    // about a fifth of a pixel a frame
                    let x = 0.4 + k as f64 * 0.009;
                    let cam = Camera { x, y: 2.2, z: 6.0, look_x: x, look_y: 0.4, look_z: 0.0, fov: 40.0, time: 1.0, ..Camera::default() };
                    let mut f = world::frame(&world, &items, Some(&cam), None, &[], 1.0, w as f32 / h as f32);
                    f.pixel = pixel.then(|| world::pixel_look(&items, None, 4.0, (w * 4) as f32, (h * 4) as f32).unwrap());
                    world::render(&f, w, h).unwrap()
                })
                .collect()
        };
        // pixels that change from one frame to the next once the whole picture's move (whole pixels, the best of a
        // few) is taken out: what swims against the world, not the world sliding past; and pixels that change and
        // change back (the shimmer)
        let (wi, hi) = (w as i32, h as i32);
        let px = |f: &[u8], x: i32, y: i32| -> [u8; 3] {
            let k = ((y * wi + x) * 4) as usize;
            [f[k], f[k + 1], f[k + 2]]
        };
        let moved = |a: &[u8], b: &[u8]| -> usize {
            let mut best = usize::MAX;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let mut n = 0;
                    for y in 2..hi - 2 {
                        for x in 2..wi - 2 {
                            n += (px(a, x, y) != px(b, x + dx, y + dy)) as usize;
                        }
                    }
                    best = best.min(n);
                }
            }
            best
        };
        let changes = |s: &[Vec<u8>]| -> (usize, usize) {
            let n = s.windows(2).map(|p| moved(&p[0], &p[1])).sum();
            let back = s.windows(3).map(|p| (0..w * h).filter(|i| {
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                px(&p[0], x, y) == px(&p[2], x, y) && px(&p[0], x, y) != px(&p[1], x, y)
            }).count()).sum();
            (n, back)
        };
        let (art, plain) = (shots(true), shots(false));
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scene-3d");
        std::fs::create_dir_all(&dir).unwrap();
        for k in [5usize, 6] {
            std::fs::write(dir.join(format!("pixel-art-{k}.png")), atelico_gpu::png(&art[k], w, h).unwrap()).unwrap();
        }
        let ((am, ab), (pm, pb)) = (changes(&art), changes(&plain));
        eprintln!("pixel art: {am} changes, {ab} back and forth; without the look: {pm}, {pb}");
        assert!(am * 10 < pm && ab * 4 < pb, "pixel art: {am} changes, {ab} back and forth; without the look: {pm}, {pb}");
        let colours: std::collections::BTreeSet<[u8; 3]> = art[0].chunks(4).map(|c| [c[0], c[1], c[2]]).collect();
        assert!(colours.len() <= 32, "every pixel one of the palette's 32 colours: {}", colours.len());
    }

    #[test]
    fn props_parse_into_typed_nodes() {
        let items: Vec<Item> = serde_json::from_value(serde_json::json!([
            { "type": "Plant", "kind": "palm", "seed": 4, "x": 1.5 },
            { "type": "Sea", "sea": "#102030" },
        ]))
        .unwrap();
        assert!(items[0].get::<Plant>().is_some_and(|p| p.seed == 4 && p.x == 1.5 && p.scale == 1.0));
        assert!(matches!(&items[1], Item::Sea(s) if s.sea == nodes::hex(0x102030)));
        assert!(serde_json::from_value::<Sea>(serde_json::json!({ "sea": "blue-ish" })).is_err());
        assert_eq!(serde_json::to_value(Sea::default()).unwrap()["sea"], "#1690ee");
        for name in ["World", "Sky", "Palette", "Grade", "Sea", "Terrain", "Plant", "Rock", "Hut", "Material", "Swatch", "Character", "Campfire", "Dock", "Boat", "Camera"] {
            assert!(atelico_data::registered().any(|s| s.name == name), "{name} is a registered data type");
        }
        assert!(World::schema().fields.iter().any(|f| f.name == "stage"));
    }

    /// The Luau builder's list of what a World gathers, and the package's declared data types, are exactly the
    /// World's settings, the registered parts and the Camera: adding a part without listing it fails here.
    #[test]
    fn the_luau_and_manifest_lists_match_the_registered_parts() {
        let mut rust: Vec<String> = nodes::SETTINGS.iter().map(|s| s.to_string()).chain(part::types().map(String::from)).collect();
        rust.sort();
        let list = |text: &str, key: &str| -> Vec<String> {
            let at = text.find(key).unwrap_or_else(|| panic!("no {key}"));
            let line = &text[at..text[at..].find('}').or(text[at..].find(']')).map(|e| at + e).unwrap()];
            let mut v: Vec<String> = line.split('"').skip(1).step_by(2).map(String::from).collect();
            v.sort();
            v
        };
        let luau = list(include_str!("../lua/nodes/scene_3d.luau"), "local ITEMS = {");
        assert_eq!(luau, rust, "lua/nodes/scene_3d.luau ITEMS");
        let mut manifest = list(include_str!("../component.toml"), "data_types = [");
        manifest.retain(|t| t != "World" && t != "Camera");
        assert_eq!(manifest, rust, "component.toml data_types (besides World and Camera)");
    }

    /// The fire reads from any angle: from four sides, a fire on the stage fills a flame-shaped area with its
    /// bands (reds, oranges, a yellow core), with the core's yellow among them.
    #[test]
    fn the_fire_reads_from_every_side() {
        let world = World { stage: true, ..World::default() };
        let fire = component_fire::Fire { size: 1.2, ..component_fire::Fire::default() };
        let items = vec![Item::TimeOfDay(crate::nodes::TimeOfDay { hour: 20.0, ..Default::default() }), Item::Sky(Sky { ..Sky::default() }), Item::part(fire)];
        for (k, yaw) in [0.0f32, 90.0, 180.0, 270.0].into_iter().enumerate() {
            let a = yaw.to_radians();
            let cam = Camera { x: (a.sin() * 3.0) as f64, y: 1.0, z: (a.cos() * 3.0) as f64, look_x: 0.0, look_y: 0.5, look_z: 0.0, fov: 40.0, time: 1.3, ..Camera::default() };
            let (w, h) = (240u32, 240u32);
            let f = world::frame(&world, &items, Some(&cam), None, &[], 1.3 + k as f32 * 0.7, 1.0);
            let px = world::render(&f, w, h).unwrap();
            let (mut fire, mut core) = (0, 0);
            for c in px.chunks(4) {
                let (r, g, b) = (c[0] as i32, c[1] as i32, c[2] as i32);
                if r > 200 && r - b > 120 && g < r {
                    fire += 1;
                    if g > 200 {
                        core += 1;
                    }
                }
            }
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scene-3d");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("fire-{yaw}.png")), atelico_gpu::png(&px, w, h).unwrap()).unwrap();
            let share = fire as f32 / (w * h) as f32;
            assert!(share > 0.03, "from {yaw} degrees the fire covers {share} of the view (target/scene-3d/fire-{yaw}.png)");
            assert!(core > 20, "from {yaw} degrees its yellow core shows ({core} pixels)");
        }
    }

    /// A fire's light on the ground has no hard edge: walking out from the fire toward the eye, the ground's
    /// brightness changes by at most 6% from one 5 cm step to the next, and it falls off (the pool fades out).
    #[test]
    fn the_firelight_pool_has_no_hard_edge() {
        use glam::{vec3, Vec3};
        let world = World { stage: true, outline: 0.0, ground: nodes::hex(0xa08a62), ..World::default() };
        let fire = component_fire::Fire { size: 1.2, intensity: 3.6, reach: 7.0, ..component_fire::Fire::default() };
        // swatches far off widen the stage's disc past the light's reach
        let items = vec![
            Item::TimeOfDay(nodes::TimeOfDay { hour: 22.0, ..Default::default() }),
            Item::Sky(Sky::default()),
            Item::part(fire),
            Item::part(nodes::Swatch { x: 9.0, ..nodes::Swatch::default() }),
            Item::part(nodes::Swatch { x: -9.0, ..nodes::Swatch::default() }),
        ];
        let cam = Camera { x: 0.0, y: 5.0, z: 7.0, look_x: 0.0, look_y: 0.0, look_z: 2.0, fov: 50.0, time: 0.0, ..Camera::default() };
        let (w, h) = (640u32, 480u32);
        let f = world::frame(&world, &items, Some(&cam), None, &[], 0.0, w as f32 / h as f32);
        let px = world::render(&f, w, h).unwrap();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scene-3d");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("firelight-pool.png"), atelico_gpu::png(&px, w, h).unwrap()).unwrap();
        let luma = |p: Vec3| {
            let c = f.view_proj.project_point3(p);
            let (x, y) = (((c.x * 0.5 + 0.5) * w as f32) as u32, ((0.5 - c.y * 0.5) * h as f32) as u32);
            let i = ((y.min(h - 1) * w + x.min(w - 1)) * 4) as usize;
            0.2126 * px[i] as f32 + 0.7152 * px[i + 1] as f32 + 0.0722 * px[i + 2] as f32
        };
        let samples: Vec<f32> = (0..80).map(|k| luma(vec3(0.0, 0.0, 0.9 + k as f32 * 0.05))).collect();
        let worst = samples.windows(2).map(|w| (w[1] - w[0]).abs() / w[0].max(w[1]).max(8.0)).fold(0.0, f32::max);
        assert!(worst < 0.06, "a step of {:.0}% along the pool: {samples:.0?} (target/scene-3d/firelight-pool.png)", worst * 100.0);
        assert!(samples[0] > samples[79] * 1.8, "the pool fades out: {} near, {} far", samples[0], samples[79]);
    }

    /// At dusk and at night on the island the fire and its pool are the brightest thing in the frame: the brightest
    /// 40-pixel square (of 640 x 360) lies within 2.5 m of the campfire.
    #[test]
    fn the_campfire_is_the_brightest_thing_at_dusk_and_night() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let w = scene_file::load_world(&root, std::path::Path::new("scenes/island.scene.luau")).unwrap();
        let fire = w.items.iter().find_map(|i| i.get::<component_island_props::Campfire>()).cloned().unwrap();
        for camera in ["campfire-dusk", "night"] {
            let cam = w.cameras[camera].clone();
            let (width, height) = (640u32, 360u32);
            let f = world::frame(&w.world, &w.items, Some(&cam), None, &[], cam.time.max(0.0) as f32, width as f32 / height as f32);
            let px = world::render(&f, width, height).unwrap();
            let dir = root.join("target/scene-3d");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("brightest-{camera}.png")), atelico_gpu::png(&px, width, height).unwrap()).unwrap();
            let luma = |x: u32, y: u32| {
                let i = ((y * width + x) * 4) as usize;
                0.2126 * px[i] as f32 + 0.7152 * px[i + 1] as f32 + 0.0722 * px[i + 2] as f32
            };
            let n = 40;
            let (mut best, mut at) = (0.0, (0, 0));
            for y in (0..height - n).step_by(4) {
                for x in (0..width - n).step_by(4) {
                    let mut sum = 0.0;
                    for j in (0..n).step_by(2) {
                        for i in (0..n).step_by(2) {
                            sum += luma(x + i, y + j);
                        }
                    }
                    if sum > best {
                        (best, at) = (sum, (x + n / 2, y + n / 2));
                    }
                }
            }
            let ground = hooks::Ground::of(&w.items);
            let screen = |p: glam::Vec3| {
                let c = f.view_proj.project_point3(p);
                ((c.x * 0.5 + 0.5) * width as f32, (0.5 - c.y * 0.5) * height as f32)
            };
            let foot = glam::vec3(fire.x as f32, ground.height_at(fire.x as f32, fire.z as f32) + 0.4, fire.z as f32);
            let fire_at = screen(foot);
            // the fire and its pool: within 2.5 m of it, across the view
            let eye = glam::vec3(cam.x as f32, cam.y as f32, cam.z as f32);
            let right = (foot - eye).cross(glam::Vec3::Y).normalize();
            let edge = screen(foot + right * 2.5);
            let pool = ((edge.0 - fire_at.0).powi(2) + (edge.1 - fire_at.1).powi(2)).sqrt();
            let off = ((at.0 as f32 - fire_at.0).powi(2) + (at.1 as f32 - fire_at.1).powi(2)).sqrt();
            assert!(off < pool, "{camera}: the brightest square is at {at:?}, the fire at {fire_at:.0?}: {off:.0} px off, its pool reaches {pool:.0} px (target/scene-3d/brightest-{camera}.png)");
        }
    }

    /// The bomb's explosion is at most 1.2 s long and its fireball is the one fire in the bomb-explosion graph's look
    /// (Shader Forge in function mode, joined to the renderer's shader): while it goes off, a fire entry names a
    /// graph look and the shader holds the graph's function; after 1.2 s nothing of it is drawn.
    #[test]
    fn the_explosion_is_particles_for_4_s() {
        let world = World { stage: true, ..World::default() };
        let boom = component_fire::explosion::Explosion { start: 0.0, every: 0.0, ..Default::default() };
        let items = vec![Item::part(boom.clone())];
        let during = world::frame(&world, &items, None, None, &[], 0.4, 1.0);
        let sprites = during.listed(component_particles::PARTICLES).len();
        assert!(sprites > 30, "its puffs, tongues and bits: {sprites}");
        assert!(during.listed(crate::render::FIRES).is_empty(), "no fireball");
        world::render(&during, 96, 96).unwrap();
        let after = world::frame(&world, &items, None, None, &[], 4.05, 1.0);
        assert!(after.listed(component_particles::PARTICLES).is_empty() && after.lamps.is_empty(), "gone after 4 s");
        assert!(boom.age(4.0).is_some() && boom.age(4.01).is_none());
    }

    /// Every package's material presets, linked together, fit the Material schema and none is defined twice.
    #[test]
    fn every_linked_preset_fits_the_schema() {
        crate::material::presets_fit_their_schema().unwrap();
        for name in ["plain", "sand", "thatch", "skin", "leaf"] {
            assert!(crate::material::presets().iter().any(|m| m.name == name), "{name}");
        }
    }

    /// Fractional fields keep their fractions in the store (an integer range makes a field whole: 18.6 became 19).
    #[test]
    fn fractional_fields_are_not_whole_numbers() {
        use atelico_data::Kind;
        for (schema, field) in [(nodes::TimeOfDay::schema(), "hour"), (Camera::schema(), "hour"), (component_sky::Look::schema(), "strength"), (Grade::schema(), "look_strength"), (Grade::schema(), "exposure"), (World::schema(), "outline"), (Boat::schema(), "speed"), (Plant::schema(), "scale")] {
            let f = schema.fields.iter().find(|f| f.name == field).unwrap();
            assert!(matches!(f.kind, Kind::Number { whole: false, .. }), "{}.{field} is {:?}", schema.name, f.kind);
        }
        assert!(matches!(Terrain::schema().fields.iter().find(|f| f.name == "seed").unwrap().kind, Kind::Number { whole: true, .. }));
    }

    /// A pole on raised ground under a known sun: the ground behind it (away from the sun, at the distance its
    /// shadow reaches) is darker than the ground in front of it, seen from straight above.
    #[test]
    fn a_pole_on_raised_ground_casts_its_shadow_away_from_the_sun() {
        let terrain = Terrain { tufts: 0.0, ..Terrain::default() };
        let (x, z) = flattest(&terrain, 2.0, 4.0);
        assert!(terrain.height_at(x, z) > 2.0, "raised ground");
        let (behind, before, _) = pole_shadow(terrain, (x, z), 40.0, "pole-shadow");
        assert!(behind < before * 0.85, "behind the pole {behind:.0}, in front {before:.0} (target/scene-3d/pole-shadow.png)");
    }

    /// On flat ground, a pole's shadow is its height over the tangent of the sun's elevation long, at 30, 45 and 60
    /// degrees (within 15%).
    #[test]
    fn a_shadow_is_as_long_as_the_sun_is_low() {
        let flat = Terrain { tufts: 0.0, ..Terrain::default() };
        let spot = flattest(&flat, 0.3, 6.0);
        for elevation in [30.0f32, 45.0, 60.0] {
            let (_, _, length) = pole_shadow(flat.clone(), spot, elevation, &format!("pole-shadow-{elevation}"));
            let expected = POLE / elevation.to_radians().tan();
            assert!((length - expected).abs() < expected * 0.15, "at {elevation}°: shadow {length:.2} m, expected {expected:.2} m");
        }
    }

    const POLE: f32 = 3.0;

    /// The flattest place on the island at least `above` metres up: the one whose ground within `radius` varies
    /// least in height.
    fn flattest(t: &Terrain, above: f32, radius: f32) -> (f32, f32) {
        let reach = t.extent() * 0.8;
        let mut best = (f32::MAX, (0.0, 0.0));
        for i in 0..40 {
            for j in 0..40 {
                let (x, z) = ((i as f32 / 39.0 * 2.0 - 1.0) * reach, (j as f32 / 39.0 * 2.0 - 1.0) * reach);
                let h = t.height_at(x, z);
                if h < above {
                    continue;
                }
                let spread = (0..16).map(|k| {
                    let a = k as f32 / 16.0 * std::f32::consts::TAU;
                    (1..=3).map(|r| (t.height_at(x + a.cos() * radius * r as f32 / 3.0, z + a.sin() * radius * r as f32 / 3.0) - h).abs()).fold(0.0, f32::max)
                });
                let worst = spread.fold(0.0, f32::max);
                if worst < best.0 {
                    best = (worst, (x, z));
                }
            }
        }
        best.1
    }

    /// A pole at (x, z) under the noon sun at `elevation`, seen from straight above: brightness behind and in front
    /// of it, and how far its shadow reaches (metres). Saves target/scene-3d/<name>.png.
    fn pole_shadow(terrain: Terrain, (x, z): (f32, f32), elevation: f32, name: &str) -> (f32, f32, f32) {
        use glam::{vec3, Mat4, Vec3};
        let ground = terrain.height_at(x, z);
        let sky = Sky { noon_elevation: elevation as f64, sun_bearing: 99.0, ..Sky::default() };
        let clock = Item::TimeOfDay(nodes::TimeOfDay { hour: 12.0, ..Default::default() });
        let sun = sky.sun_at(12.0);
        let pole = hooks::Extra {
            vertices: {
                let mut m = mesh::Mesh::default();
                m.cuboid(vec3(0.0, POLE / 2.0, 0.0), vec3(0.15, POLE / 2.0, 0.15), 0.0, 0.0);
                m.0
            },
            colours: vec![(nodes::hex(0x8f5e36), world::Style::Plain)],
            transform: Mat4::from_translation(vec3(x, ground, z)),
        };
        let away = -Vec3::new(sun.x, 0.0, sun.z).normalize();
        let reach = POLE * (1.0 - sun.y * sun.y).sqrt() / sun.y;
        let base = vec3(x, ground, z);
        let centre = base + away * reach * 0.5;
        let above = Camera { x: centre.x as f64, y: (ground + 14.0) as f64, z: centre.z as f64 + 0.01, look_x: centre.x as f64, look_y: ground as f64, look_z: centre.z as f64, fov: 50.0, ..Camera::default() };
        let items = vec![clock, Item::Sky(sky), Item::Terrain(terrain.clone())];
        let world = World { outline: 0.0, ..World::default() };
        let (w, h) = (640, 640);
        let frame = world::frame(&world, &items, Some(&above), None, &[pole], 0.0, 1.0);
        let px = world::render(&frame, w, h).unwrap();
        let bright = |p: Vec3| {
            let p = vec3(p.x, terrain.height_at(p.x, p.z), p.z);
            let c = frame.view_proj.project_point3(p);
            let (sx, sy) = (((c.x * 0.5 + 0.5) * w as f32) as i32, ((0.5 - c.y * 0.5) * h as f32) as i32);
            let mut sum = 0.0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let i = (((sy + dy).clamp(0, h as i32 - 1) as u32 * w + (sx + dx).clamp(0, w as i32 - 1) as u32) * 4) as usize;
                    sum += px[i] as f32 + px[i + 1] as f32 + px[i + 2] as f32;
                }
            }
            sum / 27.0
        };
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scene-3d");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{name}.png")), atelico_gpu::png(&px, w, h).unwrap()).unwrap();
        let lit = bright(base - away * 1.5);
        // walk out from the pole's foot along the shadow until the ground is lit again
        let mut length = 0.0;
        let mut d = 0.3;
        while d < reach * 2.0 + 2.0 {
            if bright(base + away * d) < lit * 0.85 {
                length = d;
            } else if d > length + 0.4 {
                break;
            }
            d += 0.05;
        }
        // the tip of a 0.3 m pole's shadow reaches half its width past the axis
        (bright(base + away * reach * 0.6), bright(base - away * reach * 0.6), length - 0.15)
    }

    /// Shadow-side readability: a boat seen from its dark side (the sun behind it) still shows its parts apart. The
    /// hull and the sail in shade differ in luminance by at least a fifth of the brighter one.
    #[test]
    fn a_back_lit_boat_reads_on_its_dark_side() {
        use glam::vec3;
        let boat = Boat { x: 0.0, z: 0.0, start: 0.0, ..Boat::default() };
        let sky = Sky { sun_bearing: 99.0, ..Sky::default() };
        let clock = Item::TimeOfDay(nodes::TimeOfDay { hour: 16.5, ..Default::default() });
        let sun = sky.sun_at(16.5);
        // the eye opposite the sun, looking at the boat's side
        let eye = -vec3(sun.x, 0.0, sun.z).normalize() * 4.5 + vec3(0.0, 1.2, 0.0);
        let cam = Camera { x: eye.x as f64, y: eye.y as f64, z: eye.z as f64, look_x: 0.0, look_y: 0.4, look_z: 0.0, fov: 45.0, ..Camera::default() };
        let items = vec![clock, Item::Sky(sky), Item::Sea(Sea::default()), Item::part(boat.clone())];
        let world = World::default();
        let (w, h) = (640, 360);
        let frame = world::frame(&world, &items, Some(&cam), None, &[], 0.0, w as f32 / h as f32);
        let px = world::render(&frame, w, h).unwrap();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scene-3d");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("boat-dark-side.png"), atelico_gpu::png(&px, w, h).unwrap()).unwrap();
        let at = sea::afloat(&boat, 0.0, 1.0, 0.0);
        let luma = |local: glam::Vec3| {
            let c = frame.view_proj.project_point3(at.transform_point3(local));
            let (sx, sy) = (((c.x * 0.5 + 0.5) * w as f32) as u32, ((0.5 - c.y * 0.5) * h as f32) as u32);
            let i = ((sy * w + sx) * 4) as usize;
            0.2126 * px[i] as f32 + 0.7152 * px[i + 1] as f32 + 0.0722 * px[i + 2] as f32
        };
        // points on the hull's side and on the sail, both on the side facing the eye (away from the sun)
        let side = if (at.transform_vector3(glam::Vec3::Z)).dot(eye) > 0.0 { 1.0 } else { -1.0 };
        let (hull, sail) = (luma(vec3(-0.2, 0.05, 0.43 * side)), luma(vec3(-0.25, 0.9, 0.12 * side)));
        assert!(hull > 12.0 && sail > 12.0, "not crushed: hull {hull:.0}, sail {sail:.0}");
        assert!((hull - sail).abs() > hull.max(sail) * 0.2, "hull {hull:.0} and sail {sail:.0} read apart (target/scene-3d/boat-dark-side.png)");
    }

    /// Wood in full sun keeps its detail: a hut wall facing the noon sun stays below near-white and its plank seams
    /// read (the column of pixels up the wall varies by more than a seam's worth).
    #[test]
    fn sunlit_wood_is_not_blown_out_and_shows_its_planks() {
        use glam::vec3;
        let sky = Sky { sun_bearing: 99.0, ..Sky::default() };
        let clock = Item::TimeOfDay(nodes::TimeOfDay { hour: 12.8, ..Default::default() });
        let sun = sky.sun_at(12.8);
        // the wall whose outward normal is closest to the sun (the door is on +z)
        let normal = [vec3(1.0, 0.0, 0.0), vec3(-1.0, 0.0, 0.0), vec3(0.0, 0.0, -1.0)].into_iter().max_by(|a, b| a.dot(sun).total_cmp(&b.dot(sun))).unwrap();
        let half = if normal.x != 0.0 { 0.85 } else { 0.7 };
        let across = vec3(normal.z, 0.0, -normal.x);
        let eye = normal * 4.0 + vec3(0.0, 1.2, 0.0);
        let cam = Camera { x: eye.x as f64, y: eye.y as f64, z: eye.z as f64, look_x: 0.0, look_y: 1.2, look_z: 0.0, fov: 40.0, ..Camera::default() };
        let items = vec![clock, Item::Sky(sky), Item::part(Hut::default())];
        let world = World { stage: true, outline: 0.0, ..World::default() };
        let (w, h) = (640, 640);
        let frame = world::frame(&world, &items, Some(&cam), None, &[], 0.0, 1.0);
        let px = world::render(&frame, w, h).unwrap();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scene-3d");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("wood-in-sun.png"), atelico_gpu::png(&px, w, h).unwrap()).unwrap();
        let column: Vec<f32> = (0..45)
            .map(|i| {
                let p = normal * (half + 0.01) + across * 0.45 + vec3(0.0, 0.75 + i as f32 * 0.02, 0.0);
                let c = frame.view_proj.project_point3(p);
                let (sx, sy) = (((c.x * 0.5 + 0.5) * w as f32) as u32, ((0.5 - c.y * 0.5) * h as f32) as u32);
                let i = ((sy * w + sx) * 4) as usize;
                0.2126 * px[i] as f32 + 0.7152 * px[i + 1] as f32 + 0.0722 * px[i + 2] as f32
            })
            .collect();
        let (lo, hi) = column.iter().fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        assert!(hi < 190.0, "sunlit wood stays rich, not near-white: brightest {hi:.0} (target/scene-3d/wood-in-sun.png)");
        assert!(hi - lo > 12.0, "plank seams read: the wall varies {lo:.0}..{hi:.0}");
    }

    /// Stills to target/scene-3d/ (look at them): each node alone on the stage, then the island.
    #[test]
    fn renders_each_node_alone_and_the_island() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scene-3d");
        let stage = World { stage: true, ..World::default() };
        let sea = Item::Sea(Sea::default());
        let cases: Vec<(&str, World, Vec<Item>)> = vec![
            ("palm-tree", stage.clone(), vec![Item::part(Plant::default())]),
            ("hut", stage.clone(), vec![Item::part(Hut::default())]),
            ("dock", stage.clone(), vec![Item::part(Dock::default())]),
            ("boats", World { stage: true, ..World::default() }, vec![sea.clone(), Item::part(Boat { x: -1.5, ..Boat::default() }), Item::part(Boat { kind: crate::nodes::BoatKind::Row, x: 1.5, ..Boat::default() })]),
            ("terrain", World { distance: 40.0, height: 16.0, ..World::default() }, vec![Item::Terrain(Terrain::default())]),
            ("sky-dusk", World::default(), vec![Item::TimeOfDay(crate::nodes::TimeOfDay { hour: 18.2, ..Default::default() }), Item::Sky(Sky { ..Sky::default() }), sea.clone()]),
            ("sky-night", World::default(), vec![Item::TimeOfDay(crate::nodes::TimeOfDay { hour: 23.0, ..Default::default() }), Item::Sky(Sky { ..Sky::default() }), sea.clone()]),
        ];
        let boats_cam = Camera { x: 0.0, y: 2.0, z: 6.0, look_x: 0.0, look_y: 0.2, look_z: 0.0, fov: 45.0, ..Camera::default() };
        for (name, world, items) in cases {
            let cam = (name == "boats").then_some(&boats_cam);
            let file = dir.join(format!("{name}.png"));
            snapshot(&world, &items, cam, 960, 540, 2.0, &file).unwrap();
            assert!(std::fs::metadata(&file).unwrap().len() > 20_000, "{name}");
        }
    }
}

#[cfg(test)]
mod character_tests {
    use crate::character::*;
    use crate::nodes::Character;
    use glam::{vec3, Vec3};

    /// Faces are lit flat: from the front under a high sun, the skin round the eyes, the cheeks and the chin all take
    /// the lit band (within a small spread), no cast or self shadow across them.
    #[test]
    fn the_face_stays_in_the_lit_band() {
        use crate::nodes::{Camera, Item, Sky, World};
        let c = Character::default();
        let head = head_at("idle", 0.0);
        let centre = head.transform_point3(Vec3::ZERO);
        let cam = Camera { x: 0.0, y: centre.y as f64, z: 1.4, look_x: 0.0, look_y: centre.y as f64, look_z: 0.0, fov: 30.0, ..Camera::default() };
        for hour in [9.5, 12.8, 16.0] {
            let items = vec![Item::TimeOfDay(crate::nodes::TimeOfDay { hour: hour, ..Default::default() }), Item::Sky(Sky { sun_bearing: 99.0, ..Sky::default() }), Item::part(c.clone())];
            let world = World { stage: true, outline: 0.0, ..World::default() };
            let (w, h) = (400, 400);
            let f = crate::world::frame(&world, &items, Some(&cam), None, &[], 0.0, 1.0);
            let px = crate::world::render(&f, w, h).unwrap();
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scene-3d");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("face-{hour}.png")), atelico_gpu::png(&px, w, h).unwrap()).unwrap();
            // points on the skin: cheeks, chin, between the eyes, the temples
            let skin: Vec<f32> = [vec3(-0.15, -0.1, 0.18), vec3(0.15, -0.1, 0.18), vec3(0.0, -0.17, 0.16), vec3(0.0, 0.03, 0.235), vec3(-0.19, 0.02, 0.15), vec3(0.19, 0.02, 0.15)]
                .iter()
                .map(|p| {
                    let c = f.view_proj.project_point3(head.transform_point3(*p));
                    let (sx, sy) = (((c.x * 0.5 + 0.5) * w as f32) as u32, ((0.5 - c.y * 0.5) * h as f32) as u32);
                    let i = ((sy * w + sx) * 4) as usize;
                    0.2126 * px[i] as f32 + 0.7152 * px[i + 1] as f32 + 0.0722 * px[i + 2] as f32
                })
                .collect();
            let (lo, hi) = skin.iter().fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
            assert!(hi - lo < 24.0, "at {hour}h the face's skin spans {lo:.0}..{hi:.0}: {skin:?} (target/scene-3d/face-{hour}.png)");
        }
    }

    #[test]
    fn a_big_headed_figure_that_moves() {
        let idle = figure("idle", "jacket", "goggles", "", 0.0);
        let (lo, hi) = idle.bounds().unwrap();
        assert!((hi.y - lo.y - 1.2).abs() < 0.2, "about 1.2 m tall: {}", hi.y - lo.y);
        let throw_a = figure("throw", "jacket", "goggles", "bomb", 0.3);
        let throw_b = figure("throw", "jacket", "goggles", "bomb", 0.8);
        assert_ne!(throw_a.0[0..2000], throw_b.0[0..2000], "the throw moves");
        assert!(throw_a.0.iter().any(|v| v.material == slot::BOMB));
    }
}
