// SPDX-License-Identifier: LGPL-3.0-or-later
//! Optional on-disk cache for inspect metadata (speeds repeat TUI opens).

use anyhow::{Context, Result};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Cache directory: `~/.cache/guestkit/<hash>/`
pub fn cache_dir_for_image(image: &Path) -> Result<Option<PathBuf>> {
    let meta = match fs::metadata(image) {
        Ok(m) => m,
        Err(_) => return Ok(None),
    };
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    image.display().to_string().hash(&mut hasher);
    mtime.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());

    let base = dirs::cache_dir().context("cache dir")?;
    Ok(Some(base.join("guestkit").join(hash)))
}

pub fn read_cached_flag(image: &Path) -> bool {
    if let Ok(Some(dir)) = cache_dir_for_image(image) {
        dir.join("inspect.ok").exists()
    } else {
        false
    }
}

pub fn write_cached_flag(image: &Path) -> Result<()> {
    if let Some(dir) = cache_dir_for_image(image)? {
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("inspect.ok"), b"ok")?;
    }
    Ok(())
}
