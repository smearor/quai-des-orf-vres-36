//! Binary entry point of the qdo36-artwork generator.

use clap::Parser;
use miette::IntoDiagnostic;
use qdo36_artwork::generate;
use std::path::PathBuf;

/// Command-line interface of the artwork variant generator.
#[derive(Debug, Parser)]
#[command(name = "qdo36-artwork", about = "Generate SVG artwork variants from parametrized masters")]
struct Cli {
    /// Path to the variants.toml configuration.
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/variants.toml"))]
    config: PathBuf,
    /// Directory containing the parametrized master SVGs.
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/masters"))]
    masters_dir: PathBuf,
    /// Directory the generated SVG variants are written to.
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../app/resources/art"))]
    out_dir: PathBuf,
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let written = generate(&cli.config, &cli.masters_dir, &cli.out_dir).into_diagnostic()?;
    for path in &written {
        println!("wrote {}", path.display());
    }
    println!("{} variant(s) generated", written.len());
    Ok(())
}
