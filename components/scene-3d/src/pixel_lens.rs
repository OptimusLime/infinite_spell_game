//! The pixel lens held to its promise on the pixel island (scenes/parts/pixel/island.scene.luau): a camera panning in
//! steps smaller than a texel moves the picture rigidly, with nothing crawling or swimming, where the same camera
//! with perspective does not.
use std::path::{Path, PathBuf};

use crate::nodes::{Camera, Item, TimeOfDay};
use crate::scene_file::load_world;
use crate::world::{frame, pixel_look, render_in};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The island's World with its sheets read from the repository and its clock stopped.
fn island() -> (crate::nodes::World, Vec<Item>, Camera, f32) {
    let root = root();
    let w = load_world(&root, Path::new("scenes/pixel-island.scene.luau")).unwrap();
    let items = w
        .items
        .iter()
        .map(|i| {
            let mut v = i.json();
            if let Some(sheet) = v.get("sheet").and_then(|s| s.as_str()).filter(|s| !s.is_empty()) {
                v["sheet"] = root.join(sheet).to_string_lossy().into_owned().into();
            }
            let mut item: Item = serde_json::from_value(v).unwrap();
            if let Item::TimeOfDay(t) = &mut item {
                *t = TimeOfDay { running: false, hour: 16.5, ..t.clone() };
            }
            item
        })
        .collect();
    (w.world, items, w.cameras["pan"].clone(), w.pixel)
}

/// Pixels of `b` that differ from `a` moved by (dx, dy), over where both are drawn, and how many were compared.
fn differ(a: &[u8], b: &[u8], w: usize, h: usize, dx: i32, dy: i32) -> (usize, usize) {
    let (mut bad, mut seen) = (0, 0);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let (sx, sy) = (x + dx, y + dy);
            if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
                continue;
            }
            let (i, j) = ((y as usize * w + x as usize) * 4, (sy as usize * w + sx as usize) * 4);
            seen += 1;
            if a[i..i + 3] != b[j..j + 3] {
                bad += 1;
            }
        }
    }
    (bad, seen)
}

/// Pans `camera` in `steps` of `step` metres, the world held still; for each picture, the fewest pixels that differ
/// from the first moved by any whole-pixel shift near where the pan took it, as a share of those compared.
fn pan(orthographic: bool, steps: usize, step: f32) -> Vec<f64> {
    let (world, items, cam, pixel) = island();
    let (w, h) = (640u32, 360u32);
    let time = 2.0;
    let at = |k: usize| {
        let dx = k as f64 * step as f64;
        let c = Camera { orthographic, x: cam.x + dx, look_x: cam.look_x + dx, pan_x: 0.0, fov: 30.0, ..cam.clone() };
        let mut f = frame(&world, &items, Some(&c), None, &[], time, w as f32 / h as f32);
        // the scene's pixel for a 1920-wide picture, here at a third of that
        f.pixel = pixel_look(&items, Some(&c), pixel / 3.0, w as f32, h as f32);
        render_in(&format!("test/pixel-lens/{orthographic}"), &f, w, h).unwrap()
    };
    let first = at(0);
    let px_per_metre = 16.0 * (pixel / 3.0);
    (1..=steps)
        .map(|k| {
            let b = at(k);
            let expect = (k as f32 * step * px_per_metre).round() as i32;
            let mut best = f64::MAX;
            for dx in expect - 3..=expect + 3 {
                for dy in -2..=2 {
                    let (bad, seen) = differ(&first, &b, w as usize, h as usize, -dx, dy);
                    best = best.min(bad as f64 / seen.max(1) as f64);
                }
            }
            best
        })
        .collect()
}

/// A pan of 0.25 m/s at 30 frames a second (a sixth of a texel a frame), for two seconds: through the pixel lens
/// every picture is the first moved by whole pixels (no pixel differs); the same pan with perspective changes a large
/// share of the picture beyond any shift (crawl). Prints both for the proof's report.
#[test]
fn a_pan_moves_the_pixel_island_rigidly() {
    let step = 0.25 / 30.0;
    let lens = pan(true, 60, step);
    let perspective = pan(false, 60, step);
    let worst = |v: &[f64]| v.iter().cloned().fold(0.0, f64::max);
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    println!("pixel lens: pictures differing from the first moved by whole pixels: worst {:.4}%, mean {:.4}%", worst(&lens) * 100.0, mean(&lens) * 100.0);
    println!("perspective: worst {:.2}%, mean {:.2}%", worst(&perspective) * 100.0, mean(&perspective) * 100.0);
    println!("per step: {}", serde_json::json!({ "metres_per_step": step, "lens": lens, "perspective": perspective }));
    assert_eq!(worst(&lens), 0.0, "the lens: every picture a whole-pixel move of the first");
    assert!(mean(&perspective) > 0.01, "perspective crawls: {:.4}", mean(&perspective));
}

/// The island at `time` (its clock stopped at 16:30) through camera `name` (its lens stack taken off), `w` x `h`.
fn shot(name: &str, time: f32, w: u32, h: u32) -> Vec<u8> {
    shot_of(name, time, (w, h), w, h)
}

/// The same, its art sized for a picture `scene` big but drawn `w` x `h` (smaller: fewer screen pixels a texel).
fn shot_of(name: &str, time: f32, scene: (u32, u32), w: u32, h: u32) -> Vec<u8> {
    shot_with(name, time, scene, w, h, true)
}

/// The same, with or without the hero (the Walker named hero).
fn shot_with(name: &str, time: f32, scene: (u32, u32), w: u32, h: u32, hero: bool) -> Vec<u8> {
    let (world, mut items, _, pixel) = island();
    if !hero {
        // the camera keeps following where the hero would be: it stays, drawn nowhere
        for i in &mut items {
            if let Item::Part(p) = i {
                if p.0.name() == "hero" {
                    let mut v = i.json();
                    v["sprite"] = "nobody".into();
                    *i = serde_json::from_value(v).unwrap();
                }
            }
        }
    }
    let root = root();
    let w0 = load_world(&root, Path::new("scenes/pixel-island.scene.luau")).unwrap();
    let c = Camera { lens: Vec::new(), ..w0.cameras[name].clone() };
    let mut f = frame(&world, &items, Some(&c), None, &[], time, w as f32 / h as f32);
    f.pixel = pixel_look(&items, Some(&c), pixel, scene.0 as f32, scene.1 as f32);
    render_in(&format!("test/pixel-lens/{name}/{w}x{h}"), &f, w, h).unwrap()
}

/// Lengths of the runs of one colour along each row (or column) of `px`, leaving out each line's first and last run
/// (cut by the picture's edge).
fn runs(px: &[u8], w: usize, h: usize, across: bool) -> Vec<usize> {
    let (lines, len) = if across { (h, w) } else { (w, h) };
    let at = |line: usize, k: usize| {
        let i = if across { line * w + k } else { k * w + line };
        &px[i * 4..i * 4 + 3]
    };
    let mut out = Vec::new();
    for line in 0..lines {
        let mut found = Vec::new();
        let mut start = 0;
        for k in 1..=len {
            if k == len || at(line, k) != at(line, start) {
                found.push(k - start);
                start = k;
            }
        }
        if found.len() > 2 {
            out.extend(&found[1..found.len() - 1]);
        }
    }
    out
}

/// Every art pixel is exactly N x N screen pixels: along every row and column, each run of one colour is a whole
/// number of texels long (never 2 then 3, never a texel cut short), landscape and portrait, still and panning, far
/// and close.
#[test]
fn every_art_pixel_is_n_by_n() {
    for (camera, scale, w, h) in [("pan", 6, 1920, 1080), ("pan", 6, 1080, 1920), ("close", 12, 1920, 1080)] {
        for time in [0.0, 0.4, 1.3] {
            let px = shot(camera, time, w, h);
            for across in [true, false] {
                let bad: Vec<usize> = runs(&px, w as usize, h as usize, across).into_iter().filter(|r| r % scale != 0).collect();
                assert!(bad.is_empty(), "{camera} {w}x{h} at {time} s: runs not a multiple of {scale}: {:?}", &bad[..bad.len().min(12)]);
            }
        }
    }
}

/// A walk the camera follows keeps the character whole: frame by frame (30 a second) the hero stands on the same
/// texels of the picture, and its outline and eyes (its darkest texels) number the same for every frame of the same
/// step, so none drop out or double as it moves.
#[test]
fn the_followed_hero_keeps_every_outline_and_eye_pixel() {
    // native scale: a picture pixel is a texel
    let (w, h) = (160u32, 90u32);
    let mut by_step: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    let mut boxes = std::collections::BTreeSet::new();
    for k in 0..90 {
        // across the square from x = -1 to 4.6, clear of anything in front of or behind it
        let t = 3.2 + k as f32 / 30.0;
        let with = shot_with("close", t, (1920, 1080), w, h, true);
        let without = shot_with("close", t, (1920, 1080), w, h, false);
        // the hero's texels: where the picture differs without it (its shadow on the ground aside: dark ones only
        // count if they are the hero's ink, far darker than any shade)
        let (mut n, mut bx) = (0, (u32::MAX, u32::MAX, 0, 0));
        // the hero's card: 12 x 16 texels, its foot on the picture's middle texel (the camera follows it)
        for y in h / 2 - 16..h / 2 + 1 {
            for x in w / 2 - 6..w / 2 + 6 {
                let i = ((y * w + x) * 4) as usize;
                let p = &with[i..i + 3];
                if !hero_texel(p, &without[i..i + 3]) {
                    continue;
                }
                bx = (bx.0.min(x), bx.1.min(y), bx.2.max(x), bx.3.max(y));
                if (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) < 32.0 {
                    n += 1;
                }
            }
        }
        // the walk's step: 0.25 m a frame of it at 1.875 m/s
        let step = ((t * 1.875 / 0.25) as usize) % 4;
        by_step.entry(step).or_default().push(n);
        boxes.insert((bx.0, bx.1));
    }
    for (step, counts) in &by_step {
        assert!(counts.iter().all(|c| *c == counts[0] && *c > 10), "step {step}: dark texels {counts:?}");
    }
    assert!(boxes.len() <= 2, "the hero's top left moved on the picture: {boxes:?}");
}

/// A pan moves the picture the same number of screen pixels every frame: 0.625 m/s is 10 texels a second, 2 screen
/// pixels a frame at 6 a texel and 30 frames a second; no frame stalls or jumps.
#[test]
fn a_pan_moves_steadily() {
    let (w, h) = (960u32, 540u32);
    // at half size a texel is 3 pixels and a frame's move is 1
    let frames: Vec<Vec<u8>> = (0..16).map(|k| shot("pan", 1.0 + k as f32 / 30.0, w, h)).collect();
    let mut moves = Vec::new();
    for pair in frames.windows(2) {
        let best = (-4..=4).min_by_key(|dx| {
            let (bad, _) = differ(&pair[0], &pair[1], w as usize, h as usize, *dx, 0);
            bad
        });
        moves.push(best.unwrap());
    }
    println!("pan: screen pixels a frame {moves:?}");
    assert!(moves.iter().all(|m| *m == moves[0] && *m != 0), "{moves:?}");
}

fn oklab(p: &[u8]) -> [f32; 3] {
    let c = [0, 1, 2].map(|k| atelico_render_3d::look::to_linear(p[k] as f32 / 255.0));
    let l = (0.4122214708 * c[0] + 0.5363325363 * c[1] + 0.0514459929 * c[2]).cbrt();
    let m = (0.2119034982 * c[0] + 0.6806995451 * c[1] + 0.1073969566 * c[2]).cbrt();
    let s = (0.0883024619 * c[0] + 0.2817188376 * c[1] + 0.6299787005 * c[2]).cbrt();
    [0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s, 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s, 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s]
}

/// The hero stands out: in the wide, dusk and close shots its mean colour is far (in OKLab, as colours look: hue and
/// lightness together) from the mean of the ground round it.
#[test]
fn the_hero_stands_out_from_the_ground() {
    let (w, h) = (1920u32, 1080u32);
    for (camera, time) in [("wide", 6.0), ("dusk", 5.0), ("close", 3.0)] {
        let (with, without) = (shot_with(camera, time, (w, h), w, h, true), shot_with(camera, time, (w, h), w, h, false));
        let (mut hero, mut n, mut bx) = ([0f32; 3], 0f32, (u32::MAX, u32::MAX, 0u32, 0u32));
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if hero_texel(&with[i..i + 3], &without[i..i + 3]) {
                    let c = oklab(&with[i..i + 3]);
                    (0..3).for_each(|k| hero[k] += c[k]);
                    n += 1.0;
                    bx = (bx.0.min(x), bx.1.min(y), bx.2.max(x), bx.3.max(y));
                }
            }
        }
        assert!(n > 0.0, "{camera}: the hero is in the picture");
        let pad = (bx.2 - bx.0 + 1).max(bx.3 - bx.1 + 1);
        let (mut ground, mut m) = ([0f32; 3], 0f32);
        for y in bx.1.saturating_sub(pad)..(bx.3 + pad).min(h - 1) {
            for x in bx.0.saturating_sub(pad)..(bx.2 + pad).min(w - 1) {
                let i = ((y * w + x) * 4) as usize;
                let c = oklab(&without[i..i + 3]);
                (0..3).for_each(|k| ground[k] += c[k]);
                m += 1.0;
            }
        }
        let (hero, ground) = (hero.map(|v| v / n), ground.map(|v| v / m));
        let hue = ((hero[1] - ground[1]).powi(2) + (hero[2] - ground[2]).powi(2)).sqrt();
        let apart = (hue * hue + (hero[0] - ground[0]).powi(2)).sqrt();
        println!("{camera}: hero {hero:?}, ground {ground:?}, hue apart {hue:.3}, lightness apart {:.3}, in all {apart:.3}", (hero[0] - ground[0]).abs());
        assert!(apart > 0.09, "{camera}: the hero is too close to the ground's colour ({apart:.3})");
    }
}

/// Whether a pixel that differs with the hero drawn is the hero itself, not its shadow (the same colour, darker).
fn hero_texel(with: &[u8], without: &[u8]) -> bool {
    if with == without {
        return false;
    }
    let (a, b) = (oklab(with), oklab(without));
    // a shadow is the ground darkened, its hue kept: darker, and its colour the same relative to its lightness (a
    // night shadow on dark earth can be a third of the ground's lightness; the hero's ink is another hue)
    let rel = |c: [f32; 3]| [c[1] / c[0].max(0.02), c[2] / c[0].max(0.02)];
    let (ra, rb) = (rel(a), rel(b));
    let shadow = a[0] < b[0] && a[0] > b[0] * 0.25 && ((ra[0] - rb[0]).powi(2) + (ra[1] - rb[1]).powi(2)).sqrt() < 0.05;
    !shadow
}

