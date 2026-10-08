//! make-look <looks/name.look.toml>...: each recipe (render-3d `look::Recipe`: a grade in Oklab over a base look
//! or none) baked into the .cube beside it (looks/name.cube), a 33³ table.
use std::path::PathBuf;

use atelico_render_3d::look::{Lut, Recipe};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Recipes: looks/<name>.look.toml.
    #[arg(required = true)]
    recipes: Vec<PathBuf>,
}

fn main() -> Result<(), String> {
    for path in Args::parse().recipes {
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let recipe: Recipe = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let base = if recipe.base.is_empty() { Lut::identity(33) } else { Lut::read(std::path::Path::new(&recipe.base))? };
        let name = path.file_name().and_then(|n| n.to_str()).and_then(|n| n.strip_suffix(".look.toml")).ok_or(format!("{}: a recipe is <name>.look.toml", path.display()))?;
        let out = path.with_file_name(format!("{name}.cube"));
        base.graded(&recipe).write(&out, name)?;
        println!("{}", out.display());
    }
    Ok(())
}
