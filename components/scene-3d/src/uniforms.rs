//! The renderer's uniform blocks (atelico-render-3d's `uniforms`) filled from a World's Frame, in one place.
use glam::{Mat4, Vec3};

use crate::world::Frame;

pub use atelico_render_3d::uniforms::*;

pub use component_water::Water;

type V4 = [f32; 4];
type M4 = [[f32; 4]; 4];

fn linear(c: [f32; 3]) -> [f32; 3] {
    c.map(crate::look::to_linear)
}

/// A colour for the shader: in linear light.
fn rgb(c: [f32; 3], a: f32) -> V4 {
    let [r, g, b] = linear(c);
    [r, g, b, a]
}

fn v(p: Vec3, w: f32) -> V4 {
    [p.x, p.y, p.z, w]
}

fn m(m: Mat4) -> M4 {
    m.to_cols_array_2d()
}

/// The blocks for `f` drawn at w x h, with the shadow map `shadow` texels a side (the lists' places are the
/// renderer's to fill).
pub fn of(f: &Frame, w: u32, h: u32, shadow: u32) -> U {
    let mo = &f.moment;
    let t = &mo.tint;
    // the light comes from the sun by day and from the moon by night
    let light = if mo.sun.y > -0.05 { mo.sun } else { mo.moon };
    let (extent, has_map) = f.built.heightmap.as_ref().map_or((1.0, 0.0), |(_, e, _)| (*e, 1.0));
    let g = &f.grade;
    let mut ambient = [[0.0; 4]; 9];
    for (a, c) in ambient.iter_mut().zip(f.ambient) {
        *a = v(c, 0.0);
    }
    U {
        camera: Camera {
            view_proj: m(f.view_proj),
            inv_view_proj: m(f.view_proj.inverse()),
            eye: v(f.eye, f.time),
            screen: [w as f32, h as f32, f.world.outline as f32 * w as f32 / 1280.0, f.world.bands as f32],
        },
        env: Environment {
            light: v(light, mo.day),
            sun: v(mo.sun, 0.0),
            sun_colour: rgb(mo.sun_colour.0, 1.0),
            moon: v(mo.moon, 0.0),
            moon_colour: rgb(mo.moon_colour.0, 1.0),
            sky_top: rgb(t.sky_top.0, 1.0),
            sky_horizon: rgb(t.sky_horizon.0, 1.0),
            light_colour: rgb(t.light.0, 1.0),
            shadow: rgb(t.shadow.0, 1.0),
            twin_a: mo.twin.map_or([0.0; 4], |(a, _)| v(a, 1.0)),
            twin_b: mo.twin.map_or([0.0; 4], |(_, b)| v(b, 1.0)),
            light2: mo.second.filter(|_| f.light2_view_proj.is_some()).map_or([0.0; 4], |(d, _, _)| v(d, 1.0)),
            light2_colour: mo.second.map_or([0.0; 4], |(_, c, shadow)| rgb(c.0, shadow)),
            ambient,
        },
        // (z: the snow lying, from the weather; w: its fire, 0 to 1)
        ground: Ground { map: [extent, has_map, component_weather::part::cover(&f.listed)[0], component_weather::part::cover(&f.listed)[3]] },
        post: Post {
            ink: rgb(f.world.ink.0, if f.pixel.as_ref().map_or(f.world.outline > 0.0, |p| p.outline) { 1.0 } else { 0.0 }),
            grade: [2f32.powf(g.exposure as f32), g.temperature as f32, g.tint as f32, if g.tonemap { 1.0 } else { 0.0 }],
            bloom: [g.bloom as f32, g.bloom_threshold as f32, g.look_strength as f32 * f.look_strength, f.looks.2],
            output: [g.output.stage(), component_world_looks::look(&g.graph).0 as f32, 0.0, 0.0],
            ao: [g.ao_radius as f32, g.ao_strength as f32, g.ao_threshold as f32, 0.0],
            pixel: atelico_render_3d::pixel::PixelLook::post(f.pixel.as_ref(), w).0,
            palette: atelico_render_3d::pixel::PixelLook::post(f.pixel.as_ref(), w).1,
            grid: [0.0; 4],
        },
        lighting: Lighting {
            light_view_proj: m(f.light_view_proj),
            shadow_map_info: [1.0 / shadow as f32, 1.0, if f.light2_view_proj.is_some() { 1.0 } else { 0.0 }, 0.0],
            probe_origin: f.probes.as_ref().map_or([0.0; 4], |p| v(p.origin, 1.0)),
            probe_spacing: f.probes.as_ref().map_or([0.0; 4], |p| v(p.spacing, 0.0)),
            light2_view_proj: m(f.light2_view_proj.unwrap_or(Mat4::IDENTITY)),
        },
        lists: [[0; 4]; 4],
    }
}


/// The water's block for `f`.
pub fn water(f: &Frame) -> Water {
    let t = &f.moment.tint;
    let (sea, shallow, foam, swell) = match &f.sea {
        Some(s) => {
            let (a, b) = t.sea.unwrap_or((s.sea, s.sea_shallow));
            (a.0, b.0, s.foam.0, s.swell as f32)
        }
        None => ([0.0; 3], [0.0; 3], [1.0; 3], 0.0),
    };
    let s = f.sea.clone().unwrap_or_default();
    let has_sea = if f.sea.is_some() { 1.0 } else { 0.0 };
    let wt = f.time * s.water_tempo as f32;
    // the sea's own colours are daylight ones: the shader dims them by the light unless a palette set them
    Water {
        sea: rgb(sea, if t.sea.is_some() { 1.0 } else { 0.0 }),
        sea_shallow: rgb(shallow, 1.0),
        foam: rgb(foam, 1.0),
        swell: [swell, 0.0, 0.0, 0.0],
        shore: [s.wave_period as f32, s.wave_reach as f32, s.wave_spread as f32, s.wave_lines as f32],
        wash: [s.wave_runup as f32 * has_sea, 0.0, 0.0, 0.0],
        field: f.built.sdf.as_ref().map_or([0.0; 4], |(_, s)| [s.centre.x, s.centre.y, s.extent, 1.0]),
        clocks: [wt * s.swell_rate as f32, wt * s.drift_rate as f32, wt * s.foam_rate as f32, wt],
    }
}

