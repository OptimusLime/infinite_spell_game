//! A World and what it holds, turned into what the GPU draws: one vertex list, a table of colours, ground shadows,
//! the water's distance field, floating hulls, paths, the time of day and the camera.
//!
//! Each item's mesh is built once per shape in its own colour slots and cached; the world's buffer is those
//! meshes with slots mapped into the shared table, rebuilt (a copy, not a rebuild of any mesh) when a shape or the
//! table's layout changes. A colour edit usually changes only the table.
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use glam::{vec2, vec3, Mat4, Vec2, Vec3};

use crate::mesh::{Mesh, Vertex};
use crate::material::{self, Material, Slot};
use crate::nodes::{Camera, Environment, Grade, Item, Look, Palette, Rgb, Sea, Terrain, World};
use crate::part::{Afloat, PathEnd, Piece, Swell};
use crate::hooks::{Extra, Pose};
use crate::sky::Moment;
use crate::props;
use crate::render::{Draw, Things};

pub use atelico_render_3d::material::Style;

/// Everything static the world draws.
pub struct Built {
    pub key: u64,
    pub vertices: Arc<Vec<Vertex>>,
    /// What stands in the world, for the stage's framing.
    pub bounds: Option<(Vec3, Vec3)>,
    /// Where each object meets the ground, for a small soft contact shade (not a cast shadow).
    pub contacts: Vec<(Vec2, f32)>,
    pub paths: Vec<(Vec2, Vec2)>,
    /// The water's distance field (the shore, posts, rocks) and its key, when there is a sea.
    pub sdf: Option<(u64, Arc<crate::sdf::Field>)>,
    /// Plant canopies for the probes' rays: centre, radius, leaf colour.
    pub canopies: Vec<(Vec3, f32, Rgb)>,
    /// A sphere holding everything that casts or takes shadows: centre, radius.
    pub reach: (Vec3, f32),
    /// The ground's heights for the sea (terrain::MAP² over [-extent, extent]) and their key.
    pub heightmap: Option<(u64, f32, Arc<Vec<[f32; 2]>>)>,
}

/// One frame's worth of what moves or is coloured.
pub struct Frame {
    pub built: Arc<Built>,
    pub world: World,
    pub moment: Moment,
    pub sea: Option<Sea>,
    /// Every colour slot: an albedo with its material.
    pub materials: Vec<Slot>,
    pub floats: Vec<Mat4>,
    /// Floating hulls: where each is, which way it points, half its length, half its beam, its speed.
    pub hulls: Vec<(Vec2, f32, f32, f32, f32)>,
    /// Meshes other packages drew into this world this frame, in world space, slots in `materials`.
    pub extra: Vec<Vertex>,
    /// How light becomes the picture, and the looks for this hour: (first, second, how far to the second).
    pub grade: Grade,
    pub looks: (String, String, f32),
    pub view_proj: Mat4,
    /// World to camera, and whether the camera is orthographic (the pixel lens projects it at native scale).
    pub view: Mat4,
    pub orthographic: bool,
    /// Through the pixel lens, the snap's remainder glides the picture (off while following a character, which moves
    /// by whole texels itself); and the camera's lens stack.
    pub glide: bool,
    pub lens: Vec<atelico_render_3d::lens::LensEffect>,
    pub eye: Vec3,
    pub time: f32,
    /// The shadow map's view: an orthographic box from the light (the sun, or the moon at night) over `reach`.
    pub light_view_proj: Mat4,
    /// The second shadow map's view, from the second light (the other moon), when there is one.
    pub light2_view_proj: Option<Mat4>,
    /// Light from everywhere but the sun (sky above, bounce below), as SH irradiance over π, linear.
    pub ambient: [Vec3; 9],
    /// The baked probe grid (where the light arrives from, place by place).
    pub probes: Option<Arc<crate::probes::Probes>>,
    /// Point lights (campfires) at this hour.
    pub lamps: Vec<crate::probes::Lamp>,
    /// How strong the looks are at this hour (times the Grade's strength).
    pub look_strength: f32,
    /// What the parts added to the renderer's other lists (a fire to `FIRES`): (list, the entry's vec4s).
    pub listed: Vec<(usize, Vec<[f32; 4]>)>,
    /// Drawn as pixel art (a scene's `pixel`), or not.
    pub pixel: Option<atelico_render_3d::pixel::PixelLook>,
}

impl Frame {
    /// The parts' entries in list `k`.
    pub fn listed(&self, k: usize) -> Vec<Vec<[f32; 4]>> {
        self.listed.iter().filter(|(l, _)| *l == k).map(|(_, e)| e.clone()).collect()
    }

    /// What shadowed colour is multiplied by at this hour (for anything shaded outside the World).
    pub fn shadow(&self) -> Rgb {
        self.moment.tint.shadow
    }
}

/// Colour slots, shared by identical albedo, material and code.
struct Table {
    slots: Vec<Slot>,
    scene: Vec<Material>,
}

impl Table {
    /// The slot of `albedo` in the material `name`, running the material's own pattern.
    fn slot(&mut self, albedo: Rgb, name: &str) -> f32 {
        let material = material::named(name, &self.scene);
        let code = material::code(&material.pattern).0;
        self.put(Slot { albedo, material, code })
    }

    fn put(&mut self, s: Slot) -> f32 {
        let i = self.slots.iter().position(|e| *e == s).unwrap_or_else(|| {
            self.slots.push(s);
            self.slots.len() - 1
        });
        i as f32
    }
}

fn terrain_of(all: &[Item]) -> Option<&Terrain> {
    all.iter().find_map(|i| if let Item::Terrain(t) = i { Some(t) } else { None })
}

/// Every item's colours mapped to table slots: per item, its local slot → table slot; the stage's disc last.
fn colours(world: &World, all: &[Item], stage: bool) -> (Table, Vec<Vec<f32>>) {
    let scene: Vec<Material> = all.iter().filter_map(|i| if let Item::Material(m) = i { Some(m.clone()) } else { None }).collect();
    let mut t = Table { slots: Vec::new(), scene };
    let mut maps = Vec::new();
    for item in all {
        maps.push(match item {
            // the ground takes four adjacent slots (sand, grass, cliff, path: the shader picks per pixel), then tufts
            Item::Terrain(x) => {
                let ground = t.slots.len() as f32;
                for (i, (c, name)) in [(x.sand, "sand"), (x.grass, "grass"), (x.cliff, "stone"), (x.path, "path")].into_iter().enumerate() {
                    let material = material::named(name, &t.scene);
                    let code = if i == 0 { material::GROUND } else { material::code(&material.pattern).0 };
                    t.slots.push(Slot { albedo: c, material, code });
                }
                vec![ground, ground + 1.0, ground + 2.0, ground + 3.0, t.slot(x.grass, "tuft")]
            }
            Item::Part(p) => p.0.colours(&t.scene).into_iter().map(|(rgb, name)| t.slot(rgb, &name)).collect(),
            _ => vec![],
        });
    }
    if stage {
        let material = material::named("stage", &t.scene);
        maps.push(vec![t.put(Slot { albedo: world.ground, material, code: material::STAGE })]);
    }
    (t, maps)
}

/// An item's shape: what its static mesh depends on, never its colours. Things on the ground depend on the ground.
fn shape_of(item: &Item, ground: &str) -> Option<String> {
    match item {
        Item::Terrain(t) => Some(format!("T{} {}", t.seed, t.tufts)),
        Item::Part(p) => p.0.shape(ground).map(|s| format!("{}{s}", p.0.type_name())),
        _ => None,
    }
}

fn piece(item: &Item, ground: &dyn Fn(f32, f32) -> f32) -> Piece {
    match item {
        Item::Terrain(t) => Piece { mesh: t.mesh(), ..Piece::default() },
        Item::Part(p) => p.0.build(ground),
        _ => Piece::default(),
    }
}

static PIECES: Mutex<Option<HashMap<String, Arc<Piece>>>> = Mutex::new(None);
/// The Worlds built lately, the latest first: two Worlds drawn in turn (two windows, a slide and the editor) each
/// keep theirs.
static BUILT: Mutex<Vec<Arc<Built>>> = Mutex::new(Vec::new());
/// At most this many Worlds, and only while they hold fewer vertices than `BUILT_VERTICES` together (the latest is
/// always kept).
const BUILDS: usize = 64;
const BUILT_VERTICES: usize = 4_000_000;

fn ground_key(all: &[Item], stage: bool) -> String {
    terrain_of(all).map_or(format!("flat {stage}"), |t| format!("{}", t.seed))
}

fn build(all: &[Item], stage: bool, maps: &[Vec<f32>], key: u64) -> Built {
    let terrain = terrain_of(all).cloned();
    let gk = ground_key(all, stage);
    let ground = |x: f32, z: f32| terrain.as_ref().map_or(if stage { 0.0 } else { -3.0 }, |t| t.height_at(x, z));
    let (mut vertices, mut contacts, mut rings, mut paths): (Vec<Vertex>, Vec<(Vec2, f32)>, Vec<(Vec2, f32)>, Vec<(Vec2, Vec2)>) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut canopies = Vec::new();
    let mut things: Option<(Vec3, Vec3)> = None;
    let mut grow_bounds = |lo: Vec3, hi: Vec3| things = Some(things.map_or((lo, hi), |(a, b)| (a.min(lo), b.max(hi))));
    let (mut dock_start, mut huts) = (None, Vec::new());
    let mut floating = 0;
    {
        let mut guard = PIECES.lock().unwrap();
        let cache = guard.get_or_insert_with(HashMap::new);
        if cache.len() > 512 {
            cache.clear();
        }
        for (item, map) in all.iter().zip(maps) {
            let Some(shape) = shape_of(item, &gk) else { continue };
            let p = cache.entry(shape).or_insert_with(|| Arc::new(piece(item, &ground))).clone();
            // a floating part's mesh is placed on the swell each frame, by its index among the floating
            let floats = matches!(item, Item::Part(q) if q.0.afloat(0.0, Swell { height: 0.0, time: 0.0 }).is_some());
            let sway = floats.then(|| {
                floating += 1;
                -(floating as f32)
            });
            if p.framed {
                if let Some((lo, hi)) = p.bounds.or_else(|| p.mesh.bounds()) {
                    grow_bounds(lo, hi);
                }
            }
            contacts.extend(&p.contacts);
            rings.extend(&p.rings);
            canopies.extend(p.canopy);
            match p.path {
                Some(PathEnd::Start(d)) => {
                    dock_start.get_or_insert(d);
                }
                Some(PathEnd::Stop(h)) => huts.push(h),
                None => {}
            }
            vertices.extend(p.mesh.0.iter().map(|v| Vertex { material: map[(v.material as usize).min(map.len() - 1)], sway: sway.unwrap_or(v.sway), ..*v }));
        }
    }
    // paths: from the dock to the nearest hut, then on to the next nearest
    if let Some(mut at) = dock_start {
        let mut left = huts;
        while !left.is_empty() {
            let (i, _) = left.iter().enumerate().min_by(|a, b| a.1.distance(at).total_cmp(&b.1.distance(at))).unwrap();
            let next = left.remove(i);
            paths.push((at, next));
            at = next;
        }
    }
    if stage {
        // the isolation stage: a neutral ground disc under what stands on it
        let slot = maps.last().and_then(|m| m.first()).copied().unwrap_or(0.0);
        let (centre, reach) = things.map_or((Vec3::ZERO, 2.0), |(lo, hi)| ((lo + hi) * 0.5, ((hi - lo) * vec3(1.0, 0.0, 1.0)).length() * 0.5));
        let r = reach * 1.25 + 0.8;
        let mut m = Mesh::default();
        for i in 0..64 {
            let (a, b) = (props::circle(i, 64) * r, props::circle(i + 1, 64) * r);
            m.tri(vec3(centre.x, 0.0, centre.z), vec3(centre.x + b.x, 0.0, centre.z + b.y), vec3(centre.x + a.x, 0.0, centre.z + a.y), slot, [0.0; 3]);
        }
        vertices.extend(m.0);
    }
    let heightmap = terrain.as_ref().map(|t| {
        let mut h = DefaultHasher::new();
        gk.hash(&mut h);
        (h.finish(), t.extent(), Arc::new(t.heightmap()))
    });
    // everything that casts or takes a shadow: the island (its shore and a margin) or the things on the stage
    let reach = match &terrain {
        Some(t) => (Vec3::ZERO, t.extent() * 0.85),
        None => things.map_or((Vec3::ZERO, 4.0), |(lo, hi)| ((lo + hi) * 0.5, (hi - lo).length() * 0.8 + 1.5)),
    };
    let sdf = all.iter().any(|i| matches!(i, Item::Sea(_))).then(|| {
        let (centre, extent) = match &terrain {
            Some(t) => (Vec2::ZERO, t.extent()),
            None => (vec2(reach.0.x, reach.0.z), reach.1.max(8.0)),
        };
        let mut h = DefaultHasher::new();
        (gk.clone(), format!("{rings:?}")).hash(&mut h);
        (h.finish(), Arc::new(crate::sdf::build(centre, extent, &ground, &rings)))
    });
    Built { key, vertices: Arc::new(vertices), bounds: things, contacts, paths, reach, heightmap, sdf, canopies }
}

/// The frame's lists for the renderer: contact spots, floating hulls' transforms, hulls, paths, lamps.
pub fn things(f: &Frame) -> Things {
    Things::of([
        f.built.contacts.iter().map(|(at, r)| vec![[at.x, at.y, *r, 0.0]]).collect(),
        f.floats.iter().map(|m| m.to_cols_array_2d().to_vec()).collect(),
        f.hulls.iter().map(|(at, heading, half, beam, speed)| vec![[at.x, at.y, heading.cos(), heading.sin()], [*half, *beam, *speed, 1.0]]).collect(),
        f.built.paths.iter().map(|(a, b)| vec![[a.x, a.y, b.x, b.y]]).collect(),
        f.lamps.iter().map(|l| vec![[l.at.x, l.at.y, l.at.z, l.reach], [l.colour.x, l.colour.y, l.colour.z, 1.0]]).collect(),
        f.listed(5),
        f.listed(6),
        component_pixel_sprites::atlas::art(f.listed(component_pixel_sprites::MAP), f.listed(component_pixel_sprites::SPRITE), component_weather::part::cover(&f.listed)[0]),
    ])
}

/// The frame as the renderer takes it, drawn at w x h.
pub fn draw(f: &Frame, w: u32, h: u32) -> Draw<'_> {
    Draw {
        uniforms: crate::uniforms::of(f, w, h, crate::render::SHADOW),
        vertices: (f.built.key, &f.built.vertices),
        extra: &f.extra,
        heightmap: f.built.heightmap.as_ref().map(|(k, _, m)| (*k, m.as_slice())),
        field: f.built.sdf.as_ref().map(|(k, s)| (*k, s.values.as_slice())),
        materials: &f.materials,
        probes: f.probes.as_ref(),
        things: things(f),
        looks: (&f.looks.0, &f.looks.1),
        block: bytemuck::bytes_of(&crate::uniforms::water(f)).to_vec(),
        passes: ["sky"].into_iter().chain(f.sea.as_ref().map(|_| "sea")).chain((!f.listed(crate::render::FIRES).is_empty()).then_some("fire")).chain((!f.listed(component_particles::PARTICLES).is_empty()).then_some("particles"))
            .chain(component_pixel_sprites::passes(f.listed(component_pixel_sprites::MAP).len(), f.listed(component_pixel_sprites::SPRITE).len()))
            .collect(),
        crisp: f.pixel.is_some(),
    }
}

/// How the World draws as pixel art in a scene of `pixel` (the Camera's own when it names one, in a pixel scene) over
/// a picture `w` x `h` of the scene's pixels: its PixelArt, else the defaults (None: not pixel art).
pub fn pixel_look(all: &[Item], camera: Option<&crate::nodes::Camera>, pixel: f32, w: f32, h: f32) -> Option<atelico_render_3d::pixel::PixelLook> {
    let pixel = match camera.map(|c| c.pixel as f32) {
        Some(p) if pixel > 1.0 && p > 1.0 => p,
        _ => pixel,
    };
    let art = all.iter().find_map(|i| if let Item::PixelArt(p) = i { Some(p.clone()) } else { None }).unwrap_or_default();
    art.look(pixel, w, h)
}

/// The frame as w x h RGBA8 pixels, row-major, top row first, in a view of its own (stills, tests).
pub fn render(f: &Frame, w: u32, h: u32) -> Result<Vec<u8>, String> {
    render_in("", f, w, h)
}

/// The frame drawn in `view` (a World's node id), which keeps its GPU state between frames. A pixel-art frame is
/// drawn at its own resolution (or the picture's, if that is smaller already), on the world's grid of pixels, and
/// scaled up to w x h without smoothing.
pub fn render_in(view: &str, f: &Frame, w: u32, h: u32) -> Result<Vec<u8>, String> {
    let (w, h) = (w.max(1), h.max(1));
    let Some(p) = &f.pixel else {
        let mut picture = crate::render::render_in(view, &draw(f, w, h), w, h)?;
        atelico_render_3d::lens::apply(&f.lens, &mut picture, w, h);
        return Ok(picture);
    };
    if f.orthographic {
        // the pixel lens: native scale, snapped, a texel of margin, scaled up by a whole number, glided by the rest
        let (vp, grid, lens) = atelico_render_3d::pixel::Lens::of(f.view, p.texels_per_metre, p.resolution, w, h, f.glide);
        let (tw, th) = lens.texels;
        let mut d = draw(f, tw, th);
        d.uniforms.camera.view_proj = vp.to_cols_array_2d();
        d.uniforms.camera.inv_view_proj = vp.inverse().to_cols_array_2d();
        d.uniforms.post.grid = grid;
        let px = crate::render::render_in(view, &d, tw, th)?;
        let mut picture = lens.picture(&px, w, h);
        atelico_render_3d::lens::apply(&f.lens, &mut picture, w, h);
        return Ok(picture);
    }
    let (rw, rh) = (p.resolution.0.min(w).max(1), p.resolution.1.min(h).max(1));
    let mut d = draw(f, rw, rh);
    let vp = atelico_render_3d::pixel::snap(f.view_proj, rw, rh);
    d.uniforms.camera.view_proj = vp.to_cols_array_2d();
    d.uniforms.camera.inv_view_proj = vp.inverse().to_cols_array_2d();
    d.uniforms.post.grid = atelico_render_3d::pixel::grid(vp, rw, rh);
    let px = crate::render::render_in(view, &d, rw, rh)?;
    Ok(if (rw, rh) == (w, h) { px } else { atelico_render_3d::pixel::upscale(&px, rw, rh, w, h) })
}

/// A frame left on the GPU: its texture, and how it lays on the w x h picture `render_in` would have made: the part
/// `src` (texels: x, y, w, h) shown at `dst` (picture pixels), by nearest texel for pixel art. Outside `dst` the
/// picture is black (the bars round pixel art).
pub struct OnGpu {
    pub texture: atelico_gpu::wgpu::Texture,
    pub src: [f32; 4],
    pub dst: [u32; 4],
    pub nearest: bool,
}

/// The frame drawn in `view` and left on the GPU, laid as `render_in` lays its picture (the pixel lens and pixel-art
/// upscale as a nearest-texel part of the texture instead of a CPU copy): None when lens effects are to be applied
/// to its pixels on the CPU.
pub fn render_texture_in(view: &str, f: &Frame, w: u32, h: u32) -> Result<Option<OnGpu>, String> {
    if !f.lens.is_empty() {
        return Ok(None);
    }
    let (w, h) = (w.max(1), h.max(1));
    let Some(p) = &f.pixel else {
        let texture = crate::render::render_texture(view, &draw(f, w, h), w, h)?;
        return Ok(Some(OnGpu { texture, src: [0.0, 0.0, w as f32, h as f32], dst: [0, 0, w, h], nearest: false }));
    };
    if f.orthographic {
        // the pixel lens (Lens::picture): texel (x + origin) / scale at each picture pixel x of the art's area
        let (vp, grid, lens) = atelico_render_3d::pixel::Lens::of(f.view, p.texels_per_metre, p.resolution, w, h, f.glide);
        let (tw, th) = lens.texels;
        let mut d = draw(f, tw, th);
        d.uniforms.camera.view_proj = vp.to_cols_array_2d();
        d.uniforms.camera.inv_view_proj = vp.inverse().to_cols_array_2d();
        d.uniforms.post.grid = grid;
        let texture = crate::render::render_texture(view, &d, tw, th)?;
        // A later editor pixel pass amplifies this small preview. Materialize its discrete sample
        // grid first; stretching the original art directly would bypass Lens::picture's downsample.
        if lens.size.0 < lens.art.0 || lens.size.1 < lens.art.1 {
            static PREVIEWS: Mutex<Option<HashMap<(u32, String), atelico_gpu::region::NearestRegion>>> = Mutex::new(None);
            let mut cache = PREVIEWS.lock().unwrap();
            let cache = cache.get_or_insert_with(HashMap::new);
            let key = (atelico_component::layer(), view.to_string());
            if cache.get(&key).is_none_or(|target| target.size() != (w, h)) {
                cache.insert(key.clone(), atelico_gpu::region::NearestRegion::new(w, h)?);
            }
            let texture = cache[&key].draw(&texture, lens.source(), lens.destination())?;
            return Ok(Some(OnGpu { texture, src: [0.0, 0.0, w as f32, h as f32], dst: [0, 0, w, h], nearest: true }));
        }
        return Ok(Some(OnGpu { texture, src: lens.source(), dst: lens.destination(), nearest: true }));
    }
    let (rw, rh) = (p.resolution.0.min(w).max(1), p.resolution.1.min(h).max(1));
    let mut d = draw(f, rw, rh);
    let vp = atelico_render_3d::pixel::snap(f.view_proj, rw, rh);
    d.uniforms.camera.view_proj = vp.to_cols_array_2d();
    d.uniforms.camera.inv_view_proj = vp.inverse().to_cols_array_2d();
    d.uniforms.post.grid = atelico_render_3d::pixel::grid(vp, rw, rh);
    let texture = crate::render::render_texture(view, &d, rw, rh)?;
    Ok(Some(OnGpu { texture, src: [0.0, 0.0, rw as f32, rh as f32], dst: [0, 0, w, h], nearest: true }))
}

/// The World's environment at `t` seconds of play, derived from its TimeOfDay, Sky, Palettes and Looks (the
/// Grade's look without Looks), at `camera`'s hour when it names one.
pub fn environment_of(all: &[Item], camera: Option<&Camera>, t: f32) -> Environment {
    let sky = all.iter().find_map(|i| if let Item::Sky(s) = i { Some(s.clone()) } else { None }).unwrap_or_default();
    let clock = all.iter().find_map(|i| if let Item::TimeOfDay(c) = i { Some(c.clone()) } else { None }).unwrap_or_default();
    let palettes: Vec<Palette> = all.iter().filter_map(|i| if let Item::Palette(p) = i { Some(p.clone()) } else { None }).collect();
    let looks: Vec<Look> = all.iter().filter_map(|i| if let Item::Look(l) = i { Some(l.clone()) } else { None }).collect();
    let grade = all.iter().find_map(|i| if let Item::Grade(g) = i { Some(g.clone()) } else { None }).unwrap_or_default();
    let hour = camera.map(|c| c.hour as f32).filter(|h| *h >= 0.0);
    component_sky::environment(&sky, &clock, &palettes, &looks, &grade.look, t, hour)
}

/// The world at `time`, seen through `camera` (else `pose`, else circling or the stage's turntable), for an
/// `aspect` view, with other packages' `extras` drawn in it.
pub fn frame(world: &World, all: &[Item], camera: Option<&Camera>, pose: Option<Pose>, extras: &[Extra], time: f32, aspect: f32) -> Frame {
    let stage = world.stage && !all.iter().any(|i| matches!(i, Item::Sea(_) | Item::Terrain(_)));
    let (mut table, maps) = colours(world, all, stage);
    // the buffer depends on every shape and on which slot each part's colours landed in
    let key = {
        let mut h = DefaultHasher::new();
        let gk = ground_key(all, stage);
        gk.hash(&mut h);
        for item in all {
            shape_of(item, &gk).hash(&mut h);
        }
        for v in maps.iter().flatten() {
            v.to_bits().hash(&mut h);
        }
        h.finish()
    };
    let built = {
        let mut cache = BUILT.lock().unwrap();
        let b = match cache.iter().position(|b| b.key == key) {
            Some(i) => cache.remove(i),
            None => Arc::new(build(all, stage, &maps, key)),
        };
        cache.insert(0, b.clone());
        cache.truncate(BUILDS);
        while cache.len() > 1 && cache.iter().map(|b| b.vertices.len()).sum::<usize>() > BUILT_VERTICES {
            cache.pop();
        }
        b
    };
    // the host's environment for this moment when it gave one, else derived here (stills, tests)
    let environment = all.iter().find_map(|i| if let Item::Environment(e) = i { Some(e.clone()) } else { None }).unwrap_or_else(|| environment_of(all, camera, time));
    let moment = environment.moment();
    let sea = all.iter().find_map(|i| if let Item::Sea(s) = i { Some(s.clone()) } else { None });
    let swell = sea.as_ref().map_or(0.0, |s| s.swell as f32);
    let swell = Swell { height: swell, time: sea.as_ref().map_or(0.0, |s| time * (s.water_tempo * s.swell_rate) as f32) };
    let afloat: Vec<Afloat> = all.iter().filter_map(|i| if let Item::Part(p) = i { p.0.afloat(time, swell) } else { None }).collect();
    let floats = afloat.iter().map(|a| a.transform).collect();
    let hulls = afloat.iter().map(|a| (a.at, a.heading, a.half_length, a.beam, a.speed)).collect();
    let mut extra = Vec::new();
    let terrain = terrain_of(all).cloned();
    let stand = |x: f32, z: f32| terrain.as_ref().map_or(0.0, |t| t.height_at(x, z));
    // (what parts are told of the sky: the moons' schedule now and where they are; the camera's look comes later)
    let v3 = |p: &[f64]| vec3(p.first().copied().unwrap_or(0.0) as f32, p.get(1).copied().unwrap_or(0.0) as f32, p.get(2).copied().unwrap_or(0.0) as f32);
    let sky_around = crate::part::Around {
        moon_days: environment.twin.then_some(environment.moon_days),
        moon_seed: all.iter().find_map(|i| if let Item::Sky(s) = i { Some(s.moon_seed) } else { None }).unwrap_or(0),
        moons: environment.twin.then(|| [v3(&environment.purple), v3(&environment.orange)]),
        ..Default::default()
    };
    for (item, map) in all.iter().zip(&maps) {
        if let Some(m) = if let Item::Part(p) = item { p.0.animate_around(time, &stand, &sky_around) } else { None } {
            extra.extend(m.0.iter().map(|v| Vertex { material: map[(v.material as usize).min(map.len() - 1)], ..*v }));
        }
    }
    for e in extras {
        let slots: Vec<f32> = e.colours.iter().map(|(c, style)| table.slot(*c, style.preset())).collect();
        for v in &e.vertices {
            let pos = e.transform.transform_point3(Vec3::from(v.pos));
            let normal = e.transform.transform_vector3(Vec3::from(v.normal)).normalize_or_zero();
            let material = slots.get(v.material as usize).copied().unwrap_or(0.0);
            extra.push(Vertex { pos: pos.into(), normal: normal.into(), material, sway: v.sway.max(0.0) });
        }
    }
    let camera = camera.cloned().or(pose.map(|p| Camera {
        x: p.eye.x as f64,
        y: p.eye.y as f64,
        z: p.eye.z as f64,
        look_x: p.look.x as f64,
        look_y: p.look.y as f64,
        look_z: p.look.z as f64,
        fov: p.fov as f64,
        ..Camera::default()
    }));
    // a camera that follows a part looks at where it is now, from the same offset
    let camera = camera.map(|mut c| {
        let at = (!c.follow.is_empty()).then(|| all.iter().find_map(|i| if let Item::Part(p) = i { (p.0.name() == c.follow).then(|| p.0.position(time)).flatten() } else { None })).flatten();
        if let Some(p) = at {
            let (dx, dz) = (p.x as f64 - c.look_x, p.z as f64 - c.look_z);
            (c.x, c.z, c.look_x, c.look_z) = (c.x + dx, c.z + dz, c.look_x + dx, c.look_z + dz);
        }
        c
    });
    let (view_proj, eye, camera_view) = view(world, camera.as_ref(), built.bounds, stage, time, aspect);
    let orthographic = camera.as_ref().is_some_and(|c| c.orthographic);
    let glide = camera.as_ref().is_none_or(|c| c.follow.is_empty());
    let lens = camera.as_ref().map(|c| c.lens.clone()).unwrap_or_default();
    let grade = all.iter().find_map(|i| if let Item::Grade(g) = i { Some(g.clone()) } else { None }).unwrap_or_default();
    let looks = (environment.look_a.clone(), environment.look_b.clone(), environment.look_mix as f32);
    let look_strength = environment.look_strength as f32;
    let light_view_proj = light_view(&built, &moment);
    let light2_view_proj = moment.second.filter(|_| moment.sun.y <= -0.05).map(|(towards, _, _)| view_from(&built, towards));
    // what bounces from below: sand and sea round an island, the sea alone, or the stage's ground
    let below = match (terrain_of(all), &sea) {
        (Some(t), Some(s)) => t.sand.lerp(moment.tint.sea.map_or(s.sea_shallow, |c| c.1), 0.5),
        (Some(t), None) => t.sand,
        (None, Some(s)) => moment.tint.sea.map_or(s.sea, |c| c.0),
        (None, None) => world.ground,
    };
    let ambient = crate::gi::ambient(&moment.light(), below);
    let lamps: Vec<crate::probes::Lamp> = all.iter().flat_map(|i| if let Item::Part(p) = i { p.0.lamps(time, moment.day, &stand) } else { Vec::new() }).collect();
    // the lamps that stay where they are (a part's lamps half a second on stand where they stand now): they bake into
    // the light probes; a moving one (a walker's lantern) lights what it passes directly but not the grid, which would
    // bake again every frame it moved. Asked of the parts, not of the frames before, so a still is the same picture
    // whatever was drawn before it
    let still: Vec<crate::probes::Lamp> = all
        .iter()
        .filter_map(|i| if let Item::Part(p) = i { Some(p) } else { None })
        .flat_map(|p| {
            let now = p.0.lamps(time, moment.day, &stand);
            let later = p.0.lamps(time + 0.5, moment.day, &stand);
            let stays = now.len() == later.len() && now.iter().zip(&later).all(|(a, b)| a.at == b.at);
            if stays { now } else { Vec::new() }
        })
        .collect();
    // (parts that answer to the sky and the view, the weather, are told the moons' schedule now and where the camera looks)
    // (and where the characters stand: a part that says where it is, else a Character by its props)
    let mut characters = [None; 4];
    for (slot, at) in characters.iter_mut().zip(all.iter().filter_map(|i| {
        let Item::Part(p) = i else { return None };
        p.0.position(time).or_else(|| {
            (p.0.type_name() == "Character").then(|| {
                let v = p.0.props();
                let (x, z) = (v["x"].as_f64().unwrap_or(0.0) as f32, v["z"].as_f64().unwrap_or(0.0) as f32);
                vec3(x, stand(x, z).max(0.0), z)
            })
        })
    })) {
        *slot = Some(at);
    }
    let around = crate::part::Around { look: camera.as_ref().map_or(Vec3::ZERO, |c| vec3(c.look_x as f32, c.look_y as f32, c.look_z as f32)), eye, characters, ..sky_around };
    let listed: Vec<(usize, Vec<[f32; 4]>)> = all.iter().filter_map(|i| if let Item::Part(p) = i { Some(p.0.things_around(time, &stand, &around)) } else { None }).flatten().collect();
    let probes = bake_probes(all, &built, &moment, &still, stage);
    if let (true, Some(p)) = (world.show_probes, &probes) {
        // a flat sphere at each valid probe in the colour of the light it gathered, averaged over all directions
        let ball = props::swatch(Vec2::ZERO).0;
        let sphere: Vec<Vertex> = ball.iter().take(16 * 24 * 6).cloned().collect();
        for (i, (sh, ok)) in p.sh.iter().zip(&p.valid).enumerate() {
            if !ok {
                continue;
            }
            let (ix, iz, iy) = (i % crate::probes::NX, i / crate::probes::NX % crate::probes::NZ, i / (crate::probes::NX * crate::probes::NZ));
            let at = p.origin + p.spacing * vec3(ix as f32, iy as f32, iz as f32);
            let at = vec3(at.x, at.y.max(stand(at.x, at.z) + 0.3), at.z);
            let up = [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z].iter().map(|d| crate::gi::at(sh, *d)).sum::<Vec3>() / 6.0;
            let display = |v: f32| v.clamp(0.0, 1.0).powf(1.0 / 2.2);
            let slot = table.slot(Rgb([display(up.x), display(up.y), display(up.z)]), "decal");
            let centre = vec3(-0.55, 0.5, 0.0);
            extra.extend(sphere.iter().map(|v| Vertex { pos: (at + (Vec3::from(v.pos) - centre) * 0.8).into(), material: slot, ..*v }));
        }
    }
    Frame { built, world: world.clone(), moment, sea, floats, hulls, extra, grade, looks, view_proj, view: camera_view, orthographic, glide, lens, eye, time, light_view_proj, light2_view_proj, ambient, probes, lamps, listed, look_strength, materials: table.slots, pixel: None }
}

/// The probe grid for this world at this hour (baked, cached by geometry and hour).
fn bake_probes(all: &[Item], built: &Built, moment: &Moment, lamps: &[crate::probes::Lamp], stage: bool) -> Option<Arc<crate::probes::Probes>> {
    use crate::probes::lin;
    let heights = built.heightmap.as_ref().map(|(_, e, m)| (*e, m.clone()));
    let kinds = terrain_of(all).map(|t| [lin(t.sand), lin(t.grass), lin(t.cliff), lin(t.path)]);
    let (centre, radius) = built.reach;
    let top = match (&heights, built.bounds) {
        (Some((_, m)), _) => m.iter().map(|h| h[0]).fold(0.0f32, f32::max) + 3.0,
        (None, Some((_, hi))) => hi.y + 1.0,
        _ => 3.0,
    };
    let at = |l: &crate::probes::Lamp| (l.at.x.to_bits(), l.at.y.to_bits(), l.at.z.to_bits());
    let still = lamps;
    // (the colliders of the world's things, found once per geometry: not every frame)
    let colliders = {
        static COLLIDERS: Mutex<std::collections::BTreeMap<u64, Vec<crate::hooks::Collider>>> = Mutex::new(std::collections::BTreeMap::new());
        let mut kept = COLLIDERS.lock().unwrap();
        if kept.len() > 16 {
            kept.clear();
        }
        kept.entry(built.key).or_insert_with(|| crate::hooks::Ground::of(all).colliders).clone()
    };
    let bake = crate::probes::Bake {
        heights,
        floor: if stage { 0.0 } else { -3.0 },
        kinds,
        colliders,
        canopies: built.canopies.iter().map(|(c, r, a)| (*c, *r, lin(*a))).collect(),
        lamps: still.to_vec(),
        centre: vec3(centre.x, top * 0.5 - 0.25, centre.z),
        half: vec3(radius * 0.9, top * 0.5 + 0.25, radius * 0.9),
        light: moment.light(),
    };
    let mut h = DefaultHasher::new();
    built.key.hash(&mut h);
    ((moment.hour * 4.0).round() as i32).hash(&mut h);
    // (a lamp's colour to an eighth in a running host, where a flickering fire must not bake a grid every frame; exact
    // for a still, with no node cache, so a still is the same picture whatever grid another still baked meanwhile)
    let exact = atelico_node_cache::shared().is_none();
    for l in still {
        let colour = if exact { [l.colour.x.to_bits() as i64, l.colour.y.to_bits() as i64, l.colour.z.to_bits() as i64] } else { [(l.colour.x * 8.0).round() as i64, 0, 0] };
        (at(l), l.reach.to_bits(), colour).hash(&mut h);
    }
    crate::probes::cached(h.finish(), built.key, bake)
}

/// Looking from the light at everything in reach: the sun, or the moon once the sun is down.
fn light_view(built: &Built, moment: &Moment) -> Mat4 {
    view_from(built, if moment.sun.y > -0.05 { moment.sun } else { moment.moon })
}

/// Looking from a light `towards` at everything in reach (held a little above the horizon).
fn view_from(built: &Built, towards: Vec3) -> Mat4 {
    let towards = if towards.y < 0.08 { vec3(towards.x, 0.08, towards.z).normalize() } else { towards };
    let (centre, r) = built.reach;
    let up = if towards.y.abs() > 0.95 { Vec3::Z } else { Vec3::Y };
    Mat4::orthographic_rh(-r, r, -r, r, 0.1, 4.0 * r) * Mat4::look_at_rh(centre + towards * 2.0 * r, centre, up)
}

/// The view-projection, the eye, and world to camera.
fn view(world: &World, camera: Option<&Camera>, bounds: Option<(Vec3, Vec3)>, stage: bool, time: f32, aspect: f32) -> (Mat4, Vec3, Mat4) {
    let yaw = (world.yaw as f32 + world.orbit_speed as f32 * time).to_radians();
    let (eye, target, fov) = match (camera, stage, bounds) {
        (Some(c), _, _) => {
            let pan = vec3(c.pan_x as f32, 0.0, c.pan_z as f32) * time;
            let (eye, at) = (vec3(c.x as f32, c.y as f32, c.z as f32) + pan, vec3(c.look_x as f32, c.look_y as f32, c.look_z as f32) + pan);
            // (a dolly moves the eye back along its line of sight, what it looks at held)
            (eye + (eye - at).normalize_or_zero() * c.dolly as f32 * time, at, c.fov as f32)
        }
        (None, true, Some((lo, hi))) => {
            // the turntable: framed so the whole thing fits, looking slightly down
            let centre = (lo + hi) * 0.5;
            let radius = ((hi - lo) * 0.5).length().max(0.5);
            let fov = world.fov as f32;
            let distance = radius / (fov.to_radians() * 0.5).sin() * 0.9;
            let pitch = 24f32.to_radians();
            (centre + vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos()) * distance, centre, fov)
        }
        _ => (vec3(yaw.cos() * world.distance as f32, world.height as f32, yaw.sin() * world.distance as f32), vec3(0.0, 1.6, 0.0), world.fov as f32),
    };
    let look = Mat4::look_at_rh(eye, target, Vec3::Y);
    let proj = match camera {
        // framed as the perspective view would be at the point it looks at
        Some(c) if c.orthographic => {
            let half = (target - eye).length() * (fov.to_radians() * 0.5).tan();
            Mat4::orthographic_rh(-half * aspect, half * aspect, -half, half, 0.1, 1500.0)
        }
        _ => Mat4::perspective_rh(fov.to_radians(), aspect, 0.2, 1500.0),
    };
    (proj * look, eye, look)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::{hex, Boat, Plant};

    fn palm(seed: u32, x: f64) -> Item {
        Item::part(Plant { seed, x, ..Plant::default() })
    }

    #[test]
    fn one_plant_on_the_stage_is_framed_and_stands_on_a_disc() {
        let world = World { stage: true, ..World::default() };
        let f = frame(&world, &[palm(3, 0.0)], None, None, &[], 0.0, 16.0 / 9.0);
        let (lo, hi) = f.built.bounds.expect("the palm has bounds");
        assert!(hi.y - lo.y > 1.0);
        let centre = f.view_proj.project_point3((lo + hi) * 0.5);
        assert!(centre.x.abs() < 0.2 && centre.y.abs() < 0.2, "framed in the middle: {centre}");
        assert!(f.materials.iter().any(|s| s.code == material::STAGE));
    }

    #[test]
    fn same_colours_share_a_slot_and_a_recolour_points_at_its_own() {
        let world = World::default();
        let a = frame(&world, &[palm(1, 0.0), palm(2, 3.0)], None, None, &[], 0.0, 1.0);
        assert_eq!(a.materials.len(), 5, "two palms, one palette");
        let b = frame(&world, &[palm(1, 0.0), Item::part(Plant { seed: 2, x: 3.0, leaf: hex(0xff0000), ..Plant::default() })], None, None, &[], 0.0, 1.0);
        assert_eq!(b.materials.len(), 6);
        let red = b.materials.iter().position(|s| s.albedo == hex(0xff0000)).unwrap() as f32;
        assert!(b.built.vertices.iter().any(|v| v.material == red), "the recoloured leaves use the new slot");
    }

    /// Looks own the grade across the day: an hour between two looks without a file (dusk, night) is graded through
    /// no look, never the day's by way of the Grade; a World without Looks takes the Grade's; a Camera's hour wins
    /// over the clock.
    #[test]
    fn night_is_not_graded_through_the_day_look() {
        use crate::nodes::{Grade, Sky, TimeOfDay};
        let world = World { stage: true, ..World::default() };
        let grade = Item::Grade(Grade { look: "looks/grade.cube".into(), ..Grade::default() });
        let looks = [(0.0, ""), (9.0, "looks/day.cube"), (17.0, "looks/day.cube"), (19.0, "")].map(|(hour, file)| Item::Look(Look { hour, file: file.into(), ..Look::default() }));
        let at = |hour: f64, with_looks: bool, camera: Option<&Camera>| {
            let mut all = vec![Item::TimeOfDay(TimeOfDay { hour, ..TimeOfDay::default() }), Item::Sky(Sky::default()), grade.clone()];
            if with_looks {
                all.extend(looks.iter().cloned());
            }
            let (a, b, f) = frame(&world, &all, camera, None, &[], 0.0, 1.0).looks;
            (a, b, (f * 100.0).round() / 100.0)
        };
        assert_eq!(at(12.0, true, None), ("looks/day.cube".into(), "looks/day.cube".into(), at(12.0, true, None).2));
        assert_eq!(at(23.0, true, None), (String::new(), String::new(), at(23.0, true, None).2), "night: no look");
        assert_eq!(at(18.0, true, None).0, "looks/day.cube", "dusk blends from the day's look");
        assert_eq!(at(18.0, true, None).1, "", "to none");
        assert_eq!(at(23.0, false, None).0, "looks/grade.cube", "no looks: the Grade's look");
        let night = Camera { hour: 23.0, ..Camera::default() };
        assert_eq!(at(12.0, true, Some(&night)).0, "", "the camera's hour: night");
    }

    /// Nothing is dropped past a fixed count: the contact spots of twenty characters and seven campfires, eight boats (afloat and with wakes)
    /// and seven campfires all reach the shader, and the frame still draws.
    #[test]
    fn every_contact_hull_and_lamp_reaches_the_shader() {
        use crate::nodes::{Campfire, Character};
        use crate::render::{CONTACTS, FLOATS, HULLS, LAMPS};
        let world = World { stage: true, ..World::default() };
        let mut all: Vec<Item> = (0..20).map(|i| Item::part(Character { x: i as f64 * 2.0, ..Character::default() })).collect();
        all.extend((0..8).map(|i| Item::part(Boat { start: i as f64 * 45.0, ..Boat::default() })));
        all.extend((0..7).map(|i| Item::part(Campfire { x: i as f64 * 3.0, z: 6.0, ..Campfire::default() })));
        let f = frame(&world, &all, None, None, &[], 0.0, 1.0);
        let t = things(&f);
        assert_eq!([t.lists[CONTACTS][1], t.lists[FLOATS][1], t.lists[HULLS][1], t.lists[LAMPS][1]], [27, 8, 8, 14], "each character and campfire has a contact spot; each campfire two lights (its fire and its embers)");
        crate::world::render(&f, 64, 64).unwrap();
    }

    /// Two Worlds drawn in turn at different sizes (two windows, a slide beside the editor) each keep what they
    /// built and their own targets: nothing is rebuilt or reallocated for the other.
    #[test]
    fn two_worlds_drawn_in_turn_keep_their_own_state() {
        use crate::nodes::{Character, Hut};
        use crate::render::made;
        let world = World { stage: true, ..World::default() };
        let a = vec![Item::part(Hut { x: 0.25, ..Hut::default() })];
        let b = vec![Item::part(Character { x: 0.5, ..Character::default() })];
        let (va, vb) = ("test/two-worlds/a", "test/two-worlds/b");
        let (fa, fb) = (frame(&world, &a, None, None, &[], 0.0, 1.0), frame(&world, &b, None, None, &[], 0.0, 2.0));
        for _ in 0..3 {
            let (ga, gb) = (frame(&world, &a, None, None, &[], 0.0, 1.0), frame(&world, &b, None, None, &[], 0.0, 2.0));
            assert!(Arc::ptr_eq(&ga.built, &fa.built) && Arc::ptr_eq(&gb.built, &fb.built), "a World was rebuilt after the other drew");
            render_in(va, &ga, 64, 64).unwrap();
            render_in(vb, &gb, 96, 48).unwrap();
        }
        assert_eq!((made(va), made(vb)), (Some(1), Some(1)), "each view made its targets once");
    }
}