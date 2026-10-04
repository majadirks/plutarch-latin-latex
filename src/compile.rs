//! Drives the two-stage `lualatex` compile: the single-column inner pages (one
//! pass, since there is no reledpar to balance), then the imposition wrapper (one
//! pass) once the inner PDF's real page count is known.

use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

/// The inner document compiles in a single pass: it is single-column, so there is
/// no facing-page balancing to iterate.
pub const INNER_PASSES: usize = 1;

/// The `lualatex` binary, overridable with `PLUTARCH_LUALATEX`.
fn lualatex_bin() -> String {
    std::env::var("PLUTARCH_LUALATEX").unwrap_or_else(|_| "lualatex".to_string())
}

/// Whether `lualatex` is callable.
pub fn lualatex_available() -> bool {
    Command::new(lualatex_bin())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run one `lualatex` pass on `tex_file` (a bare filename) inside `dir`, so its
/// relative `\includegraphics` paths resolve. Fails with the tail of the engine
/// output on a non-zero exit.
pub fn run_lualatex(dir: &Path, tex_file: &str) -> Result<()> {
    let output = Command::new(lualatex_bin())
        .current_dir(dir)
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg(tex_file)
        .output()
        .with_context(|| format!("running lualatex on {tex_file}"))?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let tail: Vec<&str> = stdout.lines().rev().take(25).collect();
        let tail = tail.into_iter().rev().collect::<Vec<_>>().join("\n");
        bail!("lualatex failed on {tex_file}:\n{tail}");
    }
    Ok(())
}

/// Compile a `.tex` in `dir` for `passes` passes.
pub fn compile(dir: &Path, tex_file: &str, passes: usize) -> Result<()> {
    for _ in 0..passes {
        run_lualatex(dir, tex_file)?;
    }
    Ok(())
}

/// The page count reported in a lualatex `.log`, parsed from the
/// `Output written on <file> (N pages…` line.
pub fn parse_page_count(log: &str) -> Option<u32> {
    let start = log.find("Output written on")?;
    let open = log[start..].find('(')? + start + 1;
    let digits: String = log[open..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Read the page count from a compiled inner PDF's sibling `.log`.
pub fn inner_page_count(dir: &Path, inner_stem: &str) -> Result<u32> {
    let log_path = dir.join(format!("{inner_stem}.log"));
    let log = std::fs::read_to_string(&log_path)
        .with_context(|| format!("reading {}", log_path.display()))?;
    parse_page_count(&log)
        .ok_or_else(|| anyhow!("could not find a page count in {}", log_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_compiles_in_one_pass() {
        assert_eq!(INNER_PASSES, 1);
    }

    #[test]
    fn parse_page_count_reads_output_line() {
        let log = "\
This is LuaHBTeX...\n\
[1{...}] [2] [3]\n\
Output written on tomus_1_inner.pdf (128 pages, 456789 bytes).\n\
Transcript written on tomus_1_inner.log.\n";
        assert_eq!(parse_page_count(log), Some(128));
    }

    #[test]
    fn parse_page_count_handles_single_page() {
        let log = "Output written on x.pdf (1 page, 100 bytes).";
        assert_eq!(parse_page_count(log), Some(1));
    }

    #[test]
    fn parse_page_count_missing_returns_none() {
        assert_eq!(parse_page_count("no output line here"), None);
    }
}
