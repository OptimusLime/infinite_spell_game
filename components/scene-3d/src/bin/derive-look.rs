//! derive-look <ours.png> <reference.png> <out.cube>: a colour look (a 33³ LUT) that makes our render take on the
//! reference's colours, matched per channel (histograms) and in chroma (Oklab). See `look::derive`.
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [ours, reference, out] = &args[..] else { return Err("derive-look <ours.png> <reference.png> <out.cube>".into()) };
    let (a, b) = (component_scene_3d::look::pixels(ours, 3)?, component_scene_3d::look::pixels(reference, 3)?);
    let lut = component_scene_3d::look::derive(&a, &b, 33);
    let title = out.file_stem().and_then(|s| s.to_str()).unwrap_or("look");
    lut.write(out, title)?;
    println!("{}", out.display());
    Ok(())
}
