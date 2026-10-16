//! spell-reel: the spells' effects (SpellFx) rendered offscreen from the real town: the town under a scene's moons is
//! driven by Play's keys on a script (select a slot, cast; a second cast inside 1.2 s fuses), and stills are taken at
//! the moments each effect punches, through the scene's camera. The same town state the HUD and the World draw.
//!   spell-reel scenes/town-ink.scene.luau player out-dir
use std::path::PathBuf;

use clap::Parser;
use component_scene_3d::nodes::TimeOfDay;
use component_scene_3d::{scene_file, snapshot_as, Item};

#[derive(Parser)]
struct Args {
    scene: PathBuf,
    camera: String,
    out: PathBuf,
    /// The picture's size, WxH.
    #[arg(long, default_value = "1920x1080")]
    size: String,
    /// The look's name in the file names (ink, pixel).
    #[arg(long, default_value = "ink")]
    look: String,
    /// The hero carries fire caught on an earlier night (a night with no fire falling still has both spells).
    #[arg(long)]
    carry_fire: bool,
    /// Hold the scene's Weather to one kind for these shots (snow, clear: so the spells are the only fire in frame).
    #[arg(long)]
    weather: Option<String>,
    /// Aim the camera at the town's hero (a camera that follows a walking sprite hero is turned into a fixed one
    /// looking where the town's spells start).
    #[arg(long)]
    frame_hero: bool,
    /// Punch the camera in this many times (the eye moves toward what it looks at): spell beats framed close.
    #[arg(long, default_value_t = 1.0)]
    zoom: f64,
    /// Also render beat N (0: wind orb, 1: fire wall, 2: wall of flame, 3: fireball, 4: dud) as a clip, from its first
    /// press for `clip_seconds`, at 30 fps: <out>/<look>-clip-<beat>.mp4.
    #[arg(long)]
    clip: Option<usize>,
    #[arg(long, default_value_t = 3.0)]
    clip_seconds: f64,
}

/// A beat of the script: at scene time `at`, press these slots in turn (each then cast). Its stills are keyed to what
/// the town does: each `(event, after, name)` is taken `after` seconds after the first `event` of the beat appears
/// ("form": the spell's body formed; "knock", "burst", "warm", "hit": the town's moments).
struct Beat {
    at: f64,
    slots: &'static [usize],
    shots: &'static [(&'static str, f64, &'static str)],
}

const SCRIPT: &[Beat] = &[
    Beat { at: 16.0, slots: &[1], shots: &[("form", 0.25, "wind-orb-streaks"), ("knock", 0.06, "wind-orb-knockback")] },
    Beat { at: 21.0, slots: &[2], shots: &[("form", 0.12, "fire-wall-rising"), ("form", 1.4, "fire-wall-burning")] },
    Beat { at: 27.0, slots: &[2, 1], shots: &[("form", 0.7, "wall-of-flame-roof-crest"), ("form", 1.6, "wall-of-flame-leaning")] },
    Beat { at: 33.0, slots: &[1, 2], shots: &[("form", 0.08, "fireball-trail"), ("burst", 0.03, "fireball-impact-flash"), ("burst", 0.14, "fireball-impact-burst"), ("burst", 0.75, "fireball-impact-smoke")] },
    Beat { at: 39.0, slots: &[2, 2], shots: &[("warm", 0.12, "hearthglow-dud-puff"), ("warm", 0.75, "hearthglow-dud-sags")] },
];

fn main() -> Result<(), String> {
    let a = Args::parse();
    let (w, h) = a.size.split_once('x').and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))).ok_or("--size WxH")?;
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut sw = scene_file::load_world(&root, &a.scene)?;
    if let Some(kind) = &a.weather {
        for item in &mut sw.items {
            if let Item::Part(p) = item {
                if p.0.type_name() == "Weather" {
                    let mut props = p.0.props();
                    props["force"] = serde_json::json!(kind);
                    if let Some(Ok(part)) = component_scene_3d::part::parse("Weather", props) {
                        *item = Item::Part(component_scene_3d::part::Part(part));
                    }
                }
            }
        }
    }
    let mut cam = sw.cameras.get(&a.camera).cloned().ok_or_else(|| format!("no camera {}", a.camera))?;
    if a.frame_hero {
        let lib = component_spellcraft::town::Library::load(Some(&root.join("spells")))?;
        // (between the hero and the seed, where the spells play)
        let (hx, hz) = ((lib.town.hero.x + lib.town.seed.x) / 2.0, (lib.town.hero.z + lib.town.seed.z) / 2.0 - 1.0);
        let (dx, dz) = (hx - cam.look_x, hz - cam.look_z);
        cam.x += dx;
        cam.z += dz;
        cam.look_x += dx;
        cam.look_z += dz;
        cam.follow = String::new();
    }
    if a.zoom > 1.0 {
        let k = 1.0 - 1.0 / a.zoom;
        cam.x += (cam.look_x - cam.x) * k;
        cam.y += (cam.look_y - cam.y) * k;
        cam.z += (cam.look_z - cam.z) * k;
    }
    let sky = sw.items.iter().find_map(|i| if let Item::Sky(s) = i { Some(s.clone()) } else { None }).ok_or("the scene has no Sky")?;
    let clock = sw.items.iter().find_map(|i| if let Item::TimeOfDay(c) = i { Some(c.clone()) } else { None }).unwrap_or_else(TimeOfDay::default);
    let key = a.scene.file_stem().and_then(|s| s.to_str()).unwrap_or("town").trim_end_matches(".scene").to_string();
    let weather = component_weather::Config::current();
    let dt = 1.0 / 20.0;
    // Play's keys: (time, slot, cast)
    let mut presses: Vec<(f64, Option<usize>, bool)> = Vec::new();
    for b in SCRIPT {
        let mut t = b.at;
        for s in b.slots {
            presses.push((t, Some(*s), false));
            presses.push((t + 0.1, None, true));
            t += 0.4;
        }
    }
    let end = SCRIPT.last().map_or(0.0, |b| b.at + 5.0);
    let clip = a.clip.and_then(|i| SCRIPT.get(i).map(|b| (i, b.at, b.at + a.clip_seconds)));
    let frames = a.out.join(format!(".frames-{}", a.look));
    let mut frame_no = 0u32;
    let mut next_frame = clip.map_or(f64::MAX, |c| c.1);
    if clip.is_some() {
        std::fs::create_dir_all(&frames).map_err(|e| e.to_string())?;
    }
    // per beat: when each event first appeared, and which shots are taken
    let mut seen: Vec<std::collections::BTreeMap<String, f64>> = SCRIPT.iter().map(|_| Default::default()).collect();
    let mut taken: Vec<Vec<bool>> = SCRIPT.iter().map(|b| vec![false; b.shots.len()]).collect();
    let mut known_bodies: std::collections::BTreeSet<String> = Default::default();
    let mut t = 0.0;
    while t <= end {
        let env = component_scene_3d::sky::environment(&sky, &clock, &[], &[], "", t as f32, None);
        let moons = [Some([env.purple[0], env.purple[1], env.purple[2]]).filter(|_| env.twin), Some([env.orange[0], env.orange[1], env.orange[2]]).filter(|_| env.twin)];
        let mut fire = component_weather::Now::at(sky.moon_seed, env.moon_days, &weather).fire as f64;
        if a.carry_fire && t < dt {
            fire = 1.0;
        }
        let mut select = None;
        let mut cast = false;
        for p in presses.iter().filter(|p| p.0 > t - dt && p.0 <= t) {
            if p.1.is_some() {
                select = p.1;
            }
            cast |= p.2;
        }
        let snap = component_spellcraft::drive(&key, t, moons, fire, select, cast);
        if let Some(bi) = SCRIPT.iter().rposition(|b| b.at <= t) {
            let s = &mut seen[bi];
            for e in snap["events"].as_array().into_iter().flatten() {
                let kind = e["kind"].as_str().unwrap_or("").to_string();
                let age = e["age"].as_f64().unwrap_or(9.0);
                if age < dt * 1.5 && t - age >= SCRIPT[bi].at {
                    s.entry(kind).or_insert(t - age);
                }
            }
            let formed = component_spellcraft::with_town(&key, |town| {
                town.world.bodies.values().filter(|b| b.alive && !b.look.is_empty()).map(|b| b.id.clone()).collect::<Vec<_>>()
            })
            .unwrap_or_default();
            for id in formed {
                if known_bodies.insert(id) && t >= SCRIPT[bi].at {
                    s.entry("form".into()).or_insert(t);
                }
            }
            for (k, (event, after, name)) in SCRIPT[bi].shots.iter().enumerate() {
                if taken[bi][k] {
                    continue;
                }
                if let Some(at) = s.get(*event) {
                    if t >= at + after {
                        let file = a.out.join(format!("{}-{name}.png", a.look));
                        snapshot_as(&sw.world, &sw.items, Some(&cam), w, h, t as f32, sw.pixel, &file)?;
                        println!("{} ({event} +{after})", file.display());
                        taken[bi][k] = true;
                    }
                }
            }
        }
        if let Some((_, _, until)) = clip {
            while t >= next_frame && next_frame <= until {
                snapshot_as(&sw.world, &sw.items, Some(&cam), w, h, t as f32, sw.pixel, &frames.join(format!("{frame_no:05}.png")))?;
                frame_no += 1;
                next_frame += 1.0 / 30.0;
            }
        }
        t += if clip.is_some_and(|c| t >= c.1 - 0.1 && t <= c.2) { 1.0 / 60.0 } else { dt };
    }
    if let Some((i, _, _)) = clip {
        let name = ["wind-orb", "fire-wall", "wall-of-flame", "fireball", "hearthglow-dud"].get(i).copied().unwrap_or("beat");
        let out = a.out.join(format!("{}-{name}-clip.mp4", a.look));
        let status = std::process::Command::new("ffmpeg")
            .args(["-loglevel", "error", "-y", "-framerate", "30", "-i"])
            .arg(frames.join("%05d.png"))
            .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", if sw.pixel > 1.0 { "8" } else { "18" }])
            .arg(&out)
            .status()
            .map_err(|e| format!("ffmpeg: {e}"))?;
        if !status.success() {
            return Err("ffmpeg failed".into());
        }
        println!("{}", out.display());
    }
    for (bi, b) in SCRIPT.iter().enumerate() {
        for (k, (event, _, name)) in b.shots.iter().enumerate() {
            if !taken[bi][k] {
                eprintln!("missed {name}: no {event} in its beat");
            }
        }
    }
    Ok(())
}
