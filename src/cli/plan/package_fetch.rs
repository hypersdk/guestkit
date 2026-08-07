// SPDX-License-Identifier: Apache-2.0
//! Host-side package download for offline PackageInstall staging.
//!
//! When `GUESTKIT_PACKAGE_FETCH` is enabled and cache files are missing,
//! download `.rpm`/`.deb` onto the host with `dnf download` / `yumdownloader`
//! / `apt-get download`, then stage into the guest as usual.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Guest package format for host download tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind {
    Rpm,
    Deb,
}

/// True when host network fetch is allowed (`GUESTKIT_PACKAGE_FETCH=1|true|yes`).
pub fn fetch_enabled() -> bool {
    match std::env::var("GUESTKIT_PACKAGE_FETCH") {
        Ok(v) => {
            let t = v.trim().to_ascii_lowercase();
            t == "1" || t == "true" || t == "yes" || t == "on"
        }
        Err(_) => false,
    }
}

/// Default download directory (`$XDG_CACHE_HOME/guestkit/packages` or `~/.cache/...`).
pub fn default_fetch_cache() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("guestkit").join("packages");
        }
    }
    dirs_home()
        .map(|h| h.join(".cache/guestkit/packages"))
        .unwrap_or_else(|| PathBuf::from("/tmp/guestkit-packages"))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Infer RPM vs DEB from a mounted guest root.
pub fn detect_package_kind(g: &mut crate::guestfs::Guestfs) -> PackageKind {
    if g.exists("/etc/debian_version").unwrap_or(false)
        || g.exists("/etc/dpkg/dpkg.cfg").unwrap_or(false)
    {
        return PackageKind::Deb;
    }
    if g.exists("/etc/redhat-release").unwrap_or(false)
        || g.exists("/etc/dnf/dnf.conf").unwrap_or(false)
        || g.exists("/usr/bin/dnf").unwrap_or(false)
        || g.exists("/usr/bin/yum").unwrap_or(false)
    {
        return PackageKind::Rpm;
    }
    // Prefer RPM when ambiguous (common for migration targets).
    PackageKind::Rpm
}

/// Download missing packages into `dest`. Returns names successfully fetched.
pub fn fetch_packages(names: &[String], dest: &Path, kind: PackageKind) -> Result<Vec<String>> {
    fs::create_dir_all(dest)
        .with_context(|| format!("create package fetch dir {}", dest.display()))?;

    let mut ok = Vec::new();
    for name in names {
        match fetch_one(name, dest, kind) {
            Ok(()) => {
                eprintln!(
                    "Fetched package '{name}' → {}",
                    dest.display()
                );
                ok.push(name.clone());
            }
            Err(e) => {
                eprintln!("Warning: could not fetch package '{name}': {e}");
            }
        }
    }
    Ok(ok)
}

fn fetch_one(name: &str, dest: &Path, kind: PackageKind) -> Result<()> {
    match kind {
        PackageKind::Rpm => fetch_rpm(name, dest),
        PackageKind::Deb => fetch_deb(name, dest),
    }
}

fn fetch_rpm(name: &str, dest: &Path) -> Result<()> {
    let dest_s = dest
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 dest"))?;

    // dnf download --destdir=DIR NAME
    if which("dnf") {
        let output = Command::new("dnf")
            .args(["download", "--destdir", dest_s, name])
            .output()
            .context("run dnf download")?;
        if output.status.success() {
            return Ok(());
        }
        return Err(anyhow::anyhow!(
            "dnf download: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    if which("yumdownloader") {
        let output = Command::new("yumdownloader")
            .args(["--destdir", dest_s, name])
            .output()
            .context("run yumdownloader")?;
        if output.status.success() {
            return Ok(());
        }
        return Err(anyhow::anyhow!(
            "yumdownloader: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Err(anyhow::anyhow!(
        "no dnf/yumdownloader on host (install dnf-plugins-core or yum-utils)"
    ))
}

fn fetch_deb(name: &str, dest: &Path) -> Result<()> {
    if !which("apt-get") {
        return Err(anyhow::anyhow!(
            "no apt-get on host (Debian/Ubuntu host required for .deb fetch)"
        ));
    }
    let output = Command::new("apt-get")
        .args(["download", name])
        .current_dir(dest)
        .output()
        .context("run apt-get download")?;
    if output.status.success() {
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "apt-get download: {}",
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn which(prog: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {prog} >/dev/null 2>&1")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Ensure `dirs` is non-empty when fetch is enabled (create default cache).
pub fn ensure_fetch_cache_dirs(mut dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    if dirs.is_empty() && fetch_enabled() {
        let d = default_fetch_cache();
        let _ = fs::create_dir_all(&d);
        if d.is_dir() {
            dirs.push(d);
        }
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_enabled_parses_truthy() {
        std::env::remove_var("GUESTKIT_PACKAGE_FETCH");
        assert!(!fetch_enabled());
        std::env::set_var("GUESTKIT_PACKAGE_FETCH", "1");
        assert!(fetch_enabled());
        std::env::set_var("GUESTKIT_PACKAGE_FETCH", "yes");
        assert!(fetch_enabled());
        std::env::set_var("GUESTKIT_PACKAGE_FETCH", "0");
        assert!(!fetch_enabled());
        std::env::remove_var("GUESTKIT_PACKAGE_FETCH");
    }

    #[test]
    fn default_cache_under_home_or_xdg() {
        let p = default_fetch_cache();
        assert!(p.to_string_lossy().contains("guestkit"));
        assert!(p.to_string_lossy().contains("packages"));
    }
}
