//! Development tooling for VandyMap.
//!
//! Usage:
//!   cargo run -p xtask -- validate-data
//!
//! `import-osm` lands in Phase 1 (see `ROADMAP.md`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use campus_data::load::load_dir;
use campus_data::validate_all;
use chrono::Utc;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);

    match command.as_deref() {
        Some("validate-data") => validate_data(),
        Some(other) => {
            eprintln!("unknown command {other:?}");
            usage();
            ExitCode::FAILURE
        }
        None => {
            usage();
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    eprintln!("usage: cargo run -p xtask -- validate-data");
}

/// Repository root, derived from this crate's location rather than the current
/// working directory, so the command behaves the same wherever it is run from.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask always lives one level below the repo root")
        .to_path_buf()
}

fn validate_data() -> ExitCode {
    let dir = repo_root().join("data").join("buildings");

    let buildings = match load_dir(&dir) {
        Ok(buildings) => buildings,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let findings = validate_all(&buildings, Utc::now().date_naive());

    if findings.is_empty() {
        // Phase 1 has not populated data/buildings yet; say so plainly rather
        // than reporting a misleading "all good" over an empty directory.
        if buildings.is_empty() {
            println!(
                "no building files in {} yet — nothing to check",
                dir.display()
            );
        } else {
            println!("{} building(s) checked, no problems found", buildings.len());
        }
        return ExitCode::SUCCESS;
    }

    eprintln!(
        "{} problem(s) found across {} building file(s):",
        findings.len(),
        buildings.len()
    );
    for finding in &findings {
        eprintln!("  {finding}");
    }
    ExitCode::FAILURE
}
