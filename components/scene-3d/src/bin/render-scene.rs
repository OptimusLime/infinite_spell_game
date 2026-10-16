//! render-scene: a scene file's World rendered without an editor (1280 x 720, or --size), through one of its Cameras (at the
//! Camera's time) or `-` (the World's own view at time 0): a still (.png) or, with --seconds, a clip (.mp4, 30 fps). A
//! scene with a `pixel` is drawn as pixel art.
//! --cameras lists the Cameras as `name<TAB>reference<TAB>target`.
use std::path::PathBuf;

use clap::Parser;
use component_scene_3d::{scene_file, snapshot_as, Item};

#[derive(Parser)]
struct Args {
    /// The scene file.
    scene: PathBuf,
    /// A Camera's name, or - for the World's own view.
    camera: Option<String>,
    /// Where the still (.png) or clip (.mp4) goes.
    out: Option<PathBuf>,
    /// List the scene's Cameras instead.
    #[arg(long)]
    cameras: bool,
    /// A clip this many seconds long from the Camera's time.
    #[arg(long)]
    seconds: Option<f32>,
    /// The Grade's look strength instead of its own.
    #[arg(long)]
    look: Option<f64>,
    /// The hour instead of the World's TimeOfDay (0 to 24; a Camera's own hour still wins).
    #[arg(long)]
    hour: Option<f64>,
    /// The picture's size, WxH.
    #[arg(long, value_parser = size, default_value = "1280x720")]
    size: (u32, u32),
}

fn size(s: &str) -> Result<(u32, u32), String> {
    let (w, h) = s.split_once('x').ok_or("want WxH")?;
    Ok((w.parse().map_err(|_| "want WxH")?, h.parse().map_err(|_| "want WxH")?))
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut w = scene_file::load_world(&root, &args.scene)?;
    if args.cameras {
        for (name, c) in &w.cameras {
            println!("{name}\t{}\t{}", c.reference, c.target);
        }
        return Ok(());
    }
    let (Some(camera), Some(out)) = (&args.camera, &args.out) else { return Err("render-scene <scene> <camera|-> <out.png|out.mp4>".into()) };
    for item in &mut w.items {
        if let (Item::Grade(g), Some(look)) = (&mut *item, args.look) {
            g.look_strength = look;
        }
    }
    if let Some(hour) = args.hour {
        match w.items.iter_mut().find_map(|i| if let Item::TimeOfDay(c) = i { Some(c) } else { None }) {
            Some(clock) => clock.hour = hour,
            None => w.items.push(Item::TimeOfDay(component_scene_3d::nodes::TimeOfDay { hour, ..Default::default() })),
        }
    }
    let cam = match camera.as_str() {
        "-" => None,
        name => Some(w.cameras.get(name).ok_or_else(|| format!("no camera {name}; there are {:?}", w.cameras.keys().collect::<Vec<_>>()))?.clone()),
    };
    let start = cam.as_ref().map_or(0.0, |c| c.time.max(0.0) as f32);
    match args.seconds {
        None => snapshot_as(&w.world, &w.items, cam.as_ref(), args.size.0, args.size.1, start, w.pixel, out)?,
        Some(seconds) => {
            let dir = std::env::temp_dir().join(format!("render-scene-{}", std::process::id()));
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            for i in 0..(seconds * 30.0) as usize {
                snapshot_as(&w.world, &w.items, cam.as_ref(), args.size.0, args.size.1, start + i as f32 / 30.0, w.pixel, &dir.join(format!("{i:05}.png")))?;
            }
            let status = std::process::Command::new("ffmpeg")
                .args(["-loglevel", "error", "-y", "-framerate", "30", "-i"])
                .arg(dir.join("%05d.png"))
                // pixel art keeps its hard edges only near lossless: its texels are an even number of pixels and move
                // by even numbers, so the colour samples stay aligned to them
                .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", if w.pixel > 1.0 { "8" } else { "18" }, "-preset", "slow"])
                .arg(out)
                .status()
                .map_err(|e| format!("ffmpeg: {e}"))?;
            // (the frames stay in the system's temporary folder, which clears itself)
            if !status.success() {
                return Err("ffmpeg failed".into());
            }
        }
    }
    println!("{}", out.display());
    Ok(())
}
