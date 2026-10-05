use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use plutarch_reader::facsimile::{run_facsimile, FacsimileArgs};

/// Build a Loeb-size, Latin-only reader whose body pages are the original 1857
/// Didot Latin column cropped from the source scans and reflowed into the trim
/// text block, under a typeset running head and per-Tomus folio. Only the front
/// matter is re-typeset from vision transcription.
#[derive(Debug, Parser)]
#[command(name = "facsimile", version, about)]
struct Args {
    /// Path to the first source scan (`Ploutarchou_Bioi_v1.pdf`).
    #[arg(long)]
    v1: PathBuf,

    /// Path to the second source scan (`Ploutarchou_Bioi_v2.pdf`).
    #[arg(long)]
    v2: PathBuf,

    /// Directory to write the per-volume `.tex`, body images, and PDFs into.
    #[arg(long)]
    out: PathBuf,

    /// The auto-detected, hand-editable unit-table cache.
    #[arg(long, default_value = "facsimile_units.json")]
    units: PathBuf,

    /// Write the `.tex` files but do not invoke lualatex.
    #[arg(long)]
    no_compile: bool,

    /// Re-transcribe the front matter even when its sidecar is fresh.
    #[arg(long)]
    retag: bool,

    /// Build with cached front-matter prose; never invoke the vision API.
    #[arg(long, conflicts_with_all = ["retag", "review_lines"])]
    offline: bool,

    /// Detect the unit table, write/refresh `facsimile_units.json`, and exit.
    #[arg(long)]
    detect_units: bool,

    /// Re-detect the unit table even when the cache is present (preserves `pinned`
    /// hand edits).
    #[arg(long)]
    redetect: bool,

    /// Restrict the build to a source-qualified page range, e.g. `v1:40-60`.
    #[arg(long)]
    pages: Option<String>,

    /// Build only the named unit, for a cheap single-unit test run.
    #[arg(long)]
    unit: Option<String>,

    /// Run the opt-in Claude Vision line review, writing pinned `skip_top_bands` /
    /// `review_notes` back into the unit table, then exit without building.
    #[arg(long)]
    review_lines: bool,

    /// Delete <out>/render_cache/ before building, forcing a fresh rasterize.
    #[arg(long)]
    clean_cache: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let facsimile_args = FacsimileArgs {
        v1: args.v1,
        v2: args.v2,
        out: args.out,
        units: args.units,
        no_compile: args.no_compile,
        retag: args.retag,
        offline: args.offline,
        detect_units: args.detect_units,
        redetect: args.redetect,
        pages: args.pages,
        unit: args.unit,
        review_lines: args.review_lines,
        clean_cache: args.clean_cache,
    };
    run_facsimile(&facsimile_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_flag_excludes_vision_operations() {
        let base = [
            "facsimile",
            "--v1",
            "v1.pdf",
            "--v2",
            "v2.pdf",
            "--out",
            "o",
            "--offline",
        ];
        assert!(Args::try_parse_from(base).unwrap().offline);
        for flag in ["--retag", "--review-lines"] {
            assert!(Args::try_parse_from(base.into_iter().chain([flag])).is_err());
        }
    }

    #[test]
    fn facsimile_clean_cache_flag_parses() {
        let base = [
            "facsimile",
            "--v1",
            "v1.pdf",
            "--v2",
            "v2.pdf",
            "--out",
            "o",
        ];
        let args = Args::try_parse_from(base).unwrap();
        assert!(!args.clean_cache);
        let args = Args::try_parse_from(base.into_iter().chain(["--clean-cache"])).unwrap();
        assert!(args.clean_cache);
    }

    #[test]
    fn facsimile_run_args_parse() {
        let args = Args::try_parse_from([
            "facsimile",
            "--v1",
            "v1.pdf",
            "--v2",
            "v2.pdf",
            "--out",
            "build_facsimile",
            "--unit",
            "theseus",
            "--no-compile",
        ])
        .unwrap();
        assert_eq!(args.v1, PathBuf::from("v1.pdf"));
        assert_eq!(args.v2, PathBuf::from("v2.pdf"));
        assert_eq!(args.out, PathBuf::from("build_facsimile"));
        assert_eq!(args.units, PathBuf::from("facsimile_units.json"));
        assert_eq!(args.unit.as_deref(), Some("theseus"));
        assert!(args.no_compile);
        assert!(!args.retag);
    }

    #[test]
    fn facsimile_run_args_parse_pages_and_retag() {
        let args = Args::try_parse_from([
            "facsimile",
            "--v1",
            "v1.pdf",
            "--v2",
            "v2.pdf",
            "--out",
            "o",
            "--units",
            "custom.json",
            "--pages",
            "v1:14-45",
            "--retag",
        ])
        .unwrap();
        assert_eq!(args.units, PathBuf::from("custom.json"));
        assert_eq!(args.pages.as_deref(), Some("v1:14-45"));
        assert!(args.retag);
    }

    #[test]
    fn detect_units_cli_flags_parse() {
        let args = Args::try_parse_from([
            "facsimile",
            "--v1",
            "v1.pdf",
            "--v2",
            "v2.pdf",
            "--out",
            "o",
            "--detect-units",
            "--redetect",
        ])
        .unwrap();
        assert!(args.detect_units);
        assert!(args.redetect);
    }
}
