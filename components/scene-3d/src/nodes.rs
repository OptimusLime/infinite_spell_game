//! The 3D scene node types, each typed Rust data (fields, kinds, defaults, docs) and each drawable on its own:
//! a `World` is the 3D viewport; `Sky`, `Sea`, `Terrain`, `Plant`, `Hut`, `Dock` and `Boat` are what it holds;
//! `Camera` and `Palette` tune how it is seen. Scene files compose them, and instance each other.
use std::collections::BTreeMap;

use atelico_data::Data;
use serde::{Deserialize, Serialize};

pub use atelico_render_3d::color::{hex, Rgb};

/// A 3D viewport: draws the Sky, Sea, Terrain and things inside it, cel-shaded, seen through a Camera it holds, or
/// circling what it holds. On the isolation `stage` (no Sea or Terrain), things stand on a neutral ground disc
/// under a turntable camera framed on their bounds.
#[derive(Data, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct World {
    /// Flat light steps of the toon ramp (Wind Waker uses 2).
    #[data(range(2, 4), whole)]
    pub bands: u32,
    /// Ink outline width in pixels at 1280 wide; 0: none (Wind Waker paints its ink into textures).
    #[data(range(0.0, 6.0))]
    pub outline: f64,
    #[data(color)]
    pub ink: Rgb,
    /// A neutral cel-shaded ground disc and a turntable camera framed on what stands on it.
    pub stage: bool,
    /// Debug: a small sphere at each baked light probe, coloured by the light it gathered.
    pub show_probes: bool,
    #[data(color)]
    pub ground: Rgb,
    /// The child Camera to look through; empty: circle the world.
    pub camera: String,
    /// Degrees per second the circling camera turns.
    #[data(range(-90.0, 90.0))]
    pub orbit_speed: f64,
    /// Where the circling starts, degrees.
    #[data(range(-360.0, 360.0))]
    pub yaw: f64,
    /// The circling camera's distance and height (ignored on the stage, which frames itself).
    #[data(range(4.0, 200.0))]
    pub distance: f64,
    #[data(range(0.5, 100.0))]
    pub height: f64,
    /// Vertical field of view, degrees.
    #[data(range(15.0, 90.0))]
    pub fov: f64,
    /// Shown only when the app's value of this name is true ("!name": when false).
    pub show: String,
}

impl Default for World {
    fn default() -> Self {
        Self {
            bands: 2,
            outline: 1.0,
            ink: hex(0x1d2340),
            stage: false,
            show_probes: false,
            ground: hex(0xd9e4c8),
            camera: String::new(),
            orbit_speed: 6.0,
            yaw: 20.0,
            distance: 40.0,
            height: 9.0,
            fov: 42.0,
            show: String::new(),
        }
    }
}

pub use component_sky::{Environment, Look, Palette, Sky, TimeOfDay};

/// How the World's light becomes the picture (a child of the World), each stage inspectable through `output`:
/// the scene is lit in linear light, then exposed and white-balanced, tonemapped by a filmic curve that keeps
/// saturated colours saturated, bloomed from genuinely bright pixels, and graded last by a colour look (a 3D LUT).
#[derive(Data, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Grade {
    /// Stops brighter (+) or darker (-).
    #[data(range(-4.0, 4.0))]
    pub exposure: f64,
    /// White balance: cooler (-1) to warmer (+1).
    #[data(range(-1.0, 1.0))]
    pub temperature: f64,
    /// White balance: greener (-1) to more magenta (+1).
    #[data(range(-1.0, 1.0))]
    pub tint: f64,
    /// The filmic curve; off: linear light clipped at white.
    pub tonemap: bool,
    /// How much glow spreads from pixels brighter than `bloom_threshold` (linear light).
    #[data(range(0.0, 2.0))]
    pub bloom: f64,
    #[data(range(0.5, 8.0))]
    pub bloom_threshold: f64,
    /// A colour look (a .cube 3D LUT, e.g. looks/wind-waker-day.cube) applied last; empty: none. Palettes may
    /// pick their own per hour.
    pub look: String,
    #[data(range(0.0, 1.0))]
    pub look_strength: f64,
    /// Ambient occlusion: how far round each point it looks (metres), how strongly occlusion darkens, and how
    /// occluded a point must be to step into the shade band.
    #[data(range(0.1, 5.0))]
    pub ao_radius: f64,
    #[data(range(0.0, 2.0))]
    pub ao_strength: f64,
    #[data(range(0.0, 1.0))]
    pub ao_threshold: f64,
    /// A look graph that does the whole post pass instead (graphs/<graph>.graph.yaml, a Shader Forge graph of look
    /// nodes: ink, occlusion, bands, rim, sky, exposure, bloom, tonemap, the colour look, light steps, the palette;
    /// packages/world-looks); empty: the post pass's own stages with the settings above. Edits to the graph's file
    /// show on the next frame.
    pub graph: String,
    /// Which stage the World shows: the final picture, or linear light, exposed, or tonemapped (before the look), or
    /// the ambient occlusion alone.
    #[data(choice)]
    pub output: Output,
    pub show: String,
}

impl Default for Grade {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            temperature: 0.0,
            tint: 0.0,
            tonemap: true,
            bloom: 0.3,
            bloom_threshold: 1.4,
            look: String::new(),
            look_strength: 1.0,
            ao_radius: 1.2,
            ao_strength: 1.0,
            ao_threshold: 0.3,
            graph: String::new(),
            output: Output::Final,
            show: String::new(),
        }
    }
}

/// A stage of the picture a World can show (Grade.output).
#[derive(atelico_data::DataKind, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Output {
    #[default]
    Final,
    Linear,
    Exposed,
    Tonemapped,
    Ao,
}

impl Output {
    /// The shader's number for it (world.wgsl fs_post).
    pub fn stage(self) -> f32 {
        match self {
            Output::Linear => 0.0,
            Output::Exposed => 1.0,
            Output::Tonemapped => 2.0,
            Output::Final => 3.0,
            Output::Ao => 4.0,
        }
    }
}

pub use component_water::Sea;

pub use component_terrain_generation::world::Terrain;

/// A plant: a grown kind and a seed (the same plant every time, baked once into .atelico/baked/), or a pinned asset.
#[derive(Data, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Plant {
    /// A plant kind registered with markov-junior (plant-generation: palm, bush, tree).
    pub kind: String,
    #[data(whole, range(0, 100000))]
    pub seed: u32,
    /// A pinned plant (assets/plants/<name>.mesh) instead of kind and seed; empty: grow from them.
    pub asset: String,
    pub x: f64,
    pub z: f64,
    /// Degrees about the vertical.
    #[data(range(-360.0, 360.0))]
    pub yaw: f64,
    #[data(range(0.1, 10.0))]
    pub scale: f64,
    #[data(color)]
    pub leaf: Rgb,
    /// The lighter of a frond's two tones.
    #[data(color)]
    pub leaf_light: Rgb,
    #[data(color)]
    pub trunk: Rgb,
    #[data(color)]
    pub trunk_light: Rgb,
    #[data(color)]
    pub wood: Rgb,
    pub show: String,
}

impl Default for Plant {
    fn default() -> Self {
        Self {
            kind: "palm".into(),
            seed: 1,
            asset: String::new(),
            x: 0.0,
            z: 0.0,
            yaw: 0.0,
            scale: 1.0,
            leaf: hex(0x3faa3a),
            leaf_light: hex(0x7fcc4c),
            trunk: hex(0x8a6a44),
            trunk_light: hex(0xc4b27a),
            wood: hex(0x6e4526),
            show: String::new(),
        }
    }
}

/// A rock formation: a grown kind and a seed (rock-generation's rules), or a pinned asset.
#[derive(Data, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Rock {
    /// A rock kind registered with markov-junior (rock-generation: sea-cliff, sea-stack, boulders).
    pub kind: String,
    #[data(whole, range(0, 100000))]
    pub seed: u32,
    /// A pinned rock formation (assets/rocks/<name>.mesh) instead of kind and seed; empty: grow from them.
    pub asset: String,
    pub x: f64,
    pub z: f64,
    #[data(range(-360.0, 360.0))]
    pub yaw: f64,
    #[data(range(0.1, 20.0))]
    pub scale: f64,
    #[data(color)]
    pub stone: Rgb,
    #[data(color)]
    pub stone_light: Rgb,
    #[data(color)]
    pub moss: Rgb,
    pub show: String,
}

impl Default for Rock {
    fn default() -> Self {
        Self {
            kind: "sea-stack".into(),
            seed: 1,
            asset: String::new(),
            x: 0.0,
            z: 0.0,
            yaw: 0.0,
            scale: 1.0,
            stone: hex(0x8a8fa6),
            stone_light: hex(0xb8b8c4),
            moss: hex(0x7fb84a),
            show: String::new(),
        }
    }
}

/// A material sample for looking at a material alone: a sphere and a plank box in one of the presets (or a
/// scene's Material of that name), in its own albedo.
#[derive(Data, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Swatch {
    pub material: String,
    pub x: f64,
    pub z: f64,
    pub show: String,
}

impl Default for Swatch {
    fn default() -> Self {
        Self { material: "wood-weathered".into(), x: 0.0, z: 0.0, show: String::new() }
    }
}

pub use component_island_props::{Boat, BoatKind, Campfire, Dock, Hut};
pub use component_moon_town::{House, HouseKind, Lantern, Meadow, Plaza, Street, TownSeed, Well};
pub use component_toon_character::Character;
pub use component_fire::bomb::Bomb;
pub use component_arrow::Arrow;
pub use component_particles::part::ParticleEffect;
pub use component_weather::Weather;
pub use component_creatures::{CreatureModel, Creatures};
pub use component_founding::Founding;

/// A named viewpoint (a child of the World): where the eye is, what it looks at, the reference shot it is framed
/// to match.
#[derive(Data, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub look_x: f64,
    pub look_y: f64,
    pub look_z: f64,
    /// Vertical field of view, degrees.
    #[data(range(10.0, 100.0))]
    pub fov: f64,
    /// The reference image this view is compared with (a file in .atelico/references/wind-waker/).
    pub reference: String,
    /// What this view must match, in a few words.
    pub target: String,
    /// The moment this view shows, seconds; negative: the live clock.
    #[data(range(-1.0, 3600.0))]
    pub time: f64,
    /// The hour this view shows, 0 to 24, instead of the World's clock; negative: the clock's.
    #[data(range(-1.0, 24.0))]
    pub hour: f64,
    /// In a pixel scene, this view's pixel size instead of the scene's (0: the scene's): a wide shot takes smaller
    /// pixels than a close one, so each reads as art of about the same size.
    #[data(range(0.0, 16.0))]
    pub pixel: f64,
    /// Orthographic: no perspective, framed as the view at the point it looks at; in a pixel scene, at the
    /// PixelArt's texels per metre and snapped to the texel grid (the pixel lens).
    pub orthographic: bool,
    /// Metres per second the camera drifts along x and z while the world plays (a slow pan).
    #[data(range(-10.0, 10.0))]
    pub pan_x: f64,
    #[data(range(-10.0, 10.0))]
    pub pan_z: f64,
    /// Metres per second the camera moves back along its line of sight while the world plays, what it looks at
    /// held (a slow pull-back; negative: in).
    #[data(range(-10.0, 10.0))]
    pub dolly: f64,
    /// Keeps the part of this name (a driven character, a Walker) where the camera looks, at the same offset. Through
    /// the pixel lens it moves by whole texels with it, so the character holds still on screen.
    pub follow: String,
    /// Effects applied in order to the finished picture at screen resolution (after a pixel frame's whole-number
    /// scale): tilt-shift, vignette, bloom.
    #[data(list)]
    #[serde(deserialize_with = "list_or_empty")]
    pub lens: Vec<atelico_render_3d::lens::LensEffect>,
    pub show: String,
}

/// A list, or an empty table (Luau writes `{}` for an empty list).
fn list_or_empty<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Vec<T>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum List<T> {
        Some(Vec<T>),
        Empty(BTreeMap<String, serde_json::Value>),
    }
    match List::deserialize(d)? {
        List::Some(v) => Ok(v),
        List::Empty(m) if m.is_empty() => Ok(Vec::new()),
        List::Empty(_) => Err(serde::de::Error::custom("a list of lens effects")),
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self { x: 0.0, y: 4.0, z: 18.0, look_x: 0.0, look_y: 1.0, look_z: 0.0, fov: 42.0, reference: String::new(), target: String::new(), time: -1.0, hour: -1.0, pixel: 0.0, orthographic: false, pan_x: 0.0, pan_z: 0.0, dolly: 0.0, follow: String::new(), lens: Vec::new(), show: String::new() }
    }
}

/// How a World in a pixel scene (a scene's `pixel`) is drawn as pixel art: its palette (every pixel snapped to the
/// nearest of these colours as they look), how strongly gradients are dithered
/// between them (the sky, the sea's depth, a shading step; never a flat face), and a dark one-pixel outline round
/// silhouettes. Without one, a pixel scene takes these defaults: the island palette of 32 colours, dithered, outlined.
/// An empty palette leaves colours as drawn (sprite art in its own palette).
#[derive(Data, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct PixelArt {
    #[data(map(color))]
    pub palette: BTreeMap<String, Rgb>,
    #[data(range(0.0, 1.0))]
    pub dither: f64,
    pub outline: bool,
    /// Through an orthographic camera (the pixel lens): texels per metre, so a sprite's texel is a screen texel.
    #[data(range(4.0, 64.0))]
    pub texels_per_metre: f64,
    pub show: String,
}

impl Default for PixelArt {
    /// The island palette: our own 32 colours for pixel art (inks, sea and sky blues, greens, earth and wood, fire,
    /// dusk violets), each a step apart as they look.
    fn default() -> Self {
        let palette = [
            ("01-ink", "1a1423"),
            ("02-night", "2b2438"),
            ("03-dusk-shadow", "3e3552"),
            ("04-slate", "5d5a7a"),
            ("05-stone", "8a8aa3"),
            ("06-pale-sky", "a2d6f2"),
            ("07-white", "f4f1e8"),
            ("08-deep-sea", "0f2a5c"),
            ("09-sea", "1b4a8f"),
            ("10-shallows", "2a6fc9"),
            ("11-sky", "3f9be0"),
            ("12-light-sky", "7cc6ef"),
            ("13-haze", "bfe7f5"),
            ("14-lagoon", "2f8f8a"),
            ("15-forest", "1d3d2b"),
            ("16-leaf", "2f6b3a"),
            ("17-grass", "4c9a3f"),
            ("18-lime", "86c34a"),
            ("19-sunlit-grass", "c6e37a"),
            ("20-bark", "3b2418"),
            ("21-wood", "6b3e26"),
            ("22-plank", "9c6337"),
            ("23-thatch", "c99456"),
            ("24-sand", "e8c98a"),
            ("25-light-sand", "f5e4b5"),
            ("26-ember", "7a1f2b"),
            ("27-red", "c0392b"),
            ("28-orange", "e8702a"),
            ("29-gold", "f7b538"),
            ("30-cream", "fff1a8"),
            ("31-plum", "8e4a8c"),
            ("32-rose", "d98bb0"),
        ]
        .into_iter()
        .map(|(name, h)| (name.to_string(), hex(u32::from_str_radix(h, 16).unwrap_or(0))))
        .collect();
        Self { palette, dither: 0.6, outline: true, texels_per_metre: 16.0, show: String::new() }
    }
}

impl PixelArt {
    /// The renderer's pixel look for a picture `w` x `h` of the scene's pixels at `pixel` to one (None: not pixel art).
    pub fn look(&self, pixel: f32, w: f32, h: f32) -> Option<atelico_render_3d::pixel::PixelLook> {
        let mut look = atelico_render_3d::pixel::PixelLook::of(pixel, w, h)?;
        look.palette = self.palette.values().take(atelico_render_3d::pixel::MOST_COLOURS).map(|c| c.0).collect();
        look.dither = self.dither as f32;
        look.outline = self.outline;
        look.texels_per_metre = self.texels_per_metre as f32;
        Some(look)
    }
}

/// One thing a World holds: one of its settings (the sky and its palettes, the grade, the sea, the ground, the
/// scene's materials) or a part (any registered `WorldPart`: plants, rocks, huts, a character...). Written as its
/// node type (`type`) and props.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    TimeOfDay(TimeOfDay),
    Sky(Sky),
    Palette(Palette),
    Look(Look),
    /// The World's environment at this moment, derived by the host (not written in scenes): when present, the World
    /// draws with it instead of deriving its own.
    Environment(Environment),
    Grade(Grade),
    PixelArt(PixelArt),
    Sea(Sea),
    Terrain(Terrain),
    Material(crate::material::Material),
    Part(crate::part::Part),
}

/// The settings' node types; parts are whatever `part::types()` lists.
pub const SETTINGS: [&str; 9] = ["TimeOfDay", "Sky", "Palette", "Look", "Grade", "PixelArt", "Sea", "Terrain", "Material"];

impl Item {
    pub fn part<T: crate::part::WorldPart>(p: T) -> Item {
        Item::Part(crate::part::Part::of(p))
    }

    /// The part inside, if it is a `T`.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        match self {
            Item::Part(p) => p.get(),
            _ => None,
        }
    }

    /// Its node type and props, as a scene writes them.
    pub fn json(&self) -> serde_json::Value {
        let (ty, v) = match self {
            Item::TimeOfDay(x) => ("TimeOfDay", serde_json::to_value(x)),
            Item::Look(x) => ("Look", serde_json::to_value(x)),
            Item::Environment(x) => ("Environment", serde_json::to_value(x)),
            Item::Sky(x) => ("Sky", serde_json::to_value(x)),
            Item::Palette(x) => ("Palette", serde_json::to_value(x)),
            Item::Grade(x) => ("Grade", serde_json::to_value(x)),
            Item::PixelArt(x) => ("PixelArt", serde_json::to_value(x)),
            Item::Sea(x) => ("Sea", serde_json::to_value(x)),
            Item::Terrain(x) => ("Terrain", serde_json::to_value(x)),
            Item::Material(x) => ("Material", serde_json::to_value(x)),
            Item::Part(p) => (p.0.type_name(), Ok(p.0.props())),
        };
        let mut v = v.unwrap_or_default();
        v["type"] = ty.into();
        v
    }

    /// Whether a node type is something a World holds.
    pub fn is_type(name: &str) -> bool {
        SETTINGS.contains(&name) || crate::part::types().any(|t| t == name)
    }
}

impl<'de> Deserialize<'de> for Item {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Item, D::Error> {
        use serde::de::Error;
        let mut v = serde_json::Value::deserialize(d)?;
        let ty = v.get("type").and_then(|t| t.as_str()).ok_or_else(|| D::Error::custom("an item has a type"))?.to_string();
        if let Some(m) = v.as_object_mut() {
            m.remove("type");
        }
        let setting = |r: Result<Item, serde_json::Error>| r.map_err(D::Error::custom);
        match ty.as_str() {
            "TimeOfDay" => setting(serde_json::from_value(v).map(Item::TimeOfDay)),
            "Look" => setting(serde_json::from_value(v).map(Item::Look)),
            "Environment" => setting(serde_json::from_value(v).map(Item::Environment)),
            "Sky" => setting(serde_json::from_value(v).map(Item::Sky)),
            "Palette" => setting(serde_json::from_value(v).map(Item::Palette)),
            "Grade" => setting(serde_json::from_value(v).map(Item::Grade)),
            "PixelArt" => setting(serde_json::from_value(v).map(Item::PixelArt)),
            "Sea" => setting(serde_json::from_value(v).map(Item::Sea)),
            "Terrain" => setting(serde_json::from_value(v).map(Item::Terrain)),
            "Material" => setting(serde_json::from_value(v).map(Item::Material)),
            other => match crate::part::parse(other, v) {
                Some(p) => p.map(|p| Item::Part(crate::part::Part(p))).map_err(D::Error::custom),
                None => Err(D::Error::custom(format!("{other} is not something a World holds (settings: {}; parts: {})", SETTINGS.join(", "), crate::part::types().collect::<Vec<_>>().join(", ")))),
            },
        }
    }
}
