//! Reading building files off disk.
//!
//! Used by native tooling (`xtask`, tests). The browser app cannot enumerate a
//! directory, so at runtime it consumes data embedded at build time instead —
//! see the "Data loading" section of `docs/ARCHITECTURE.md`.

use std::io;
use std::path::{Path, PathBuf};

use crate::building::Building;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("could not read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{path} is not valid building TOML: {source}")]
    Parse {
        path: PathBuf,
        // Boxed: toml's error type is large, and an unboxed variant makes every
        // Result in this module heavy for the common success path.
        #[source]
        source: Box<toml::de::Error>,
    },
    #[error("{path} has no usable filename")]
    BadFileName { path: PathBuf },
}

/// Load every `*.toml` file in `dir` as a building, paired with its file stem.
///
/// Returns an empty vec when the directory is empty, and an error only when a
/// file is unreadable or malformed. A missing directory is an error — a silent
/// empty result would hide a mistyped path.
pub fn load_dir(dir: &Path) -> Result<Vec<(String, Building)>, LoadError> {
    let entries = std::fs::read_dir(dir).map_err(|source| LoadError::Io {
        path: dir.to_path_buf(),
        source,
    })?;

    let mut buildings = Vec::new();

    for entry in entries {
        let path = entry
            .map_err(|source| LoadError::Io {
                path: dir.to_path_buf(),
                source,
            })?
            .path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| LoadError::BadFileName { path: path.clone() })?
            .to_string();

        let contents = std::fs::read_to_string(&path).map_err(|source| LoadError::Io {
            path: path.clone(),
            source,
        })?;

        let building = Building::from_toml(&contents).map_err(|source| LoadError::Parse {
            path: path.clone(),
            source: Box::new(source),
        })?;

        buildings.push((stem, building));
    }

    // Directory order is filesystem-dependent; sort so output and validation
    // findings are stable across machines.
    buildings.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(buildings)
}
