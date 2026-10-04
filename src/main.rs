use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;

use plutarch_reader::llm::batch::{ApiBatchBackend, BatchBackend};
use plutarch_reader::llm::{ApiVision, Client, Vision};
use plutarch_reader::{run, secrets, RunArgs};

/// Generate a Loeb-size, Latin-only reader from the two Google Books scans of
/// Plutarch's *Lives* in the Didot bilingual recension: transcribe the Latin
/// column with AI vision, then emit per-volume LaTeX and imposed PDFs for
/// hand-binding.
#[derive(Debug, Parser)]
#[command(name = "plutarch-reader", version, about)]
struct Args {
    /// Path to the first source scan (`Ploutarchou_Bioi_v1.pdf`).
    #[arg(long)]
    v1: PathBuf,

    /// Path to the second source scan (`Ploutarchou_Bioi_v2.pdf`).
    #[arg(long)]
    v2: PathBuf,

    /// Directory to write the per-volume `.tex` files and build artifacts into.
    #[arg(long)]
    out: PathBuf,

    /// Re-call the API to re-tag pages even when a fresh sidecar is present.
    #[arg(long)]
    retag: bool,

    /// Run the full pipeline on only the first N body pages, for cheap testing.
    /// Uses a separate `plutarch.dryrun.tags.json` so it never pollutes the
    /// committed cache.
    #[arg(long)]
    dry_run: Option<u32>,

    /// Restrict the run to a source-qualified page range, e.g. `v1:40-60`. Like
    /// `--dry-run`, this is a restricted run and targets the dryrun sidecar.
    #[arg(long)]
    pages: Option<String>,

    /// Write the `.tex` files but do not invoke lualatex.
    #[arg(long)]
    no_compile: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let run_args = RunArgs {
        v1: args.v1,
        v2: args.v2,
        out: args.out,
        retag: args.retag,
        dry_run: args.dry_run,
        pages: args.pages,
        no_compile: args.no_compile,
    };

    run(&run_args, || {
        let secrets = secrets::load(Path::new("secrets.json"))?;
        let client = Client::new(secrets.anthropic_api_key);
        let vision: Box<dyn Vision> = Box::new(ApiVision::new(client.clone()));
        let backend: Box<dyn BatchBackend> = Box::new(ApiBatchBackend::new(client));
        Ok((vision, backend))
    })
}
