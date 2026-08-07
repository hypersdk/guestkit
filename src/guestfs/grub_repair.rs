// SPDX-License-Identifier: Apache-2.0
//! Offline GRUB repair via chroot `grub-mkconfig` / optional `grub-install`,
//! with a first-boot oneshot fallback when chroot tools are unavailable.

use crate::core::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Guest path for the first-boot GRUB regenerate script.
pub const FIRSTBOOT_SCRIPT: &str = "/usr/lib/guestkit/firstboot-grub.sh";
/// Guest path for the first-boot systemd unit.
pub const FIRSTBOOT_UNIT: &str = "/etc/systemd/system/guestkit-firstboot-grub.service";
/// WantedBy symlink for the first-boot unit.
pub const FIRSTBOOT_WANTS: &str =
    "/etc/systemd/system/multi-user.target.wants/guestkit-firstboot-grub.service";

/// Outcome of an offline GRUB repair attempt.
#[derive(Debug, Clone, Default)]
pub struct GrubRepairReport {
    pub mkconfig_ok: bool,
    pub mkconfig_tool: Option<String>,
    pub install_ok: bool,
    pub install_device: Option<String>,
    pub firstboot_staged: bool,
    pub backups: Vec<String>,
    pub notes: Vec<String>,
}

/// Repair GRUB on a mounted guest root (`root_mount` is the host path of `/`).
///
/// 1. Bind-mount `/proc`, `/sys`, `/dev` into the guest root.
/// 2. Run `grub2-mkconfig` / `grub-mkconfig` / `update-grub` in chroot.
/// 3. Optionally run `grub-install` / `grub2-install` onto `install_device`.
/// 4. If mkconfig fails, stage a first-boot oneshot inside the guest tree.
pub fn repair_grub(
    root_mount: &Path,
    install_device: Option<&Path>,
    verbose: bool,
) -> Result<GrubRepairReport> {
    if !root_mount.is_dir() {
        return Err(Error::InvalidOperation(format!(
            "GRUB repair root is not a directory: {}",
            root_mount.display()
        )));
    }

    let mut report = GrubRepairReport::default();
    backup_grub_cfgs(root_mount, &mut report);

    let chroot_ok = mount_binds(root_mount, verbose);
    if let Err(e) = &chroot_ok {
        report
            .notes
            .push(format!("bind-mount failed ({e}); will stage first-boot fallback"));
    }

    if chroot_ok.is_ok() {
        match run_mkconfig(root_mount, verbose) {
            Ok(tool) => {
                report.mkconfig_ok = true;
                report.mkconfig_tool = Some(tool);
            }
            Err(e) => {
                report
                    .notes
                    .push(format!("chroot grub-mkconfig failed: {e}"));
            }
        }

        if let Some(dev) = install_device {
            match run_grub_install(root_mount, dev, verbose) {
                Ok(tool) => {
                    report.install_ok = true;
                    report.install_device = Some(dev.display().to_string());
                    report.notes.push(format!("grub-install via {tool}"));
                }
                Err(e) => {
                    report
                        .notes
                        .push(format!("grub-install {} failed: {e}", dev.display()));
                }
            }
        }

        let _ = unmount_binds(root_mount);
    }

    if !report.mkconfig_ok {
        stage_firstboot_grub(root_mount)?;
        report.firstboot_staged = true;
        report.notes.push(
            "staged guestkit-firstboot-grub.service (runs update-grub / grub2-mkconfig on boot)"
                .into(),
        );
    }

    if !report.mkconfig_ok && !report.firstboot_staged {
        return Err(Error::CommandFailed(
            "GRUB repair failed (chroot mkconfig and first-boot staging both unavailable)"
                .into(),
        ));
    }

    Ok(report)
}

fn backup_grub_cfgs(root: &Path, report: &mut GrubRepairReport) {
    for rel in ["boot/grub2/grub.cfg", "boot/grub/grub.cfg"] {
        let cfg = root.join(rel);
        if cfg.is_file() {
            let bak = cfg.with_extension("cfg.pre-guestkit");
            if fs::copy(&cfg, &bak).is_ok() {
                report.backups.push(bak.display().to_string());
            }
        }
    }
}

fn mount_binds(root: &Path, verbose: bool) -> Result<()> {
    let root_s = root
        .to_str()
        .ok_or_else(|| Error::InvalidFormat("non-UTF-8 root mount".into()))?;
    for dir in ["proc", "sys", "dev"] {
        let target = format!("{root_s}/{dir}");
        fs::create_dir_all(&target)
            .map_err(|e| Error::CommandFailed(format!("mkdir {target}: {e}")))?;
        let mut cmd = maybe_sudo("mount");
        let output = cmd
            .args(["--bind", &format!("/{dir}"), &target])
            .output()
            .map_err(|e| Error::CommandFailed(format!("bind mount /{dir}: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!(
                "bind mount /{dir} → {target}: {stderr}"
            )));
        }
        if verbose {
            eprintln!("grub_repair: bind-mounted /{dir} → {target}");
        }
    }
    Ok(())
}

fn unmount_binds(root: &Path) -> Result<()> {
    let root_s = root
        .to_str()
        .ok_or_else(|| Error::InvalidFormat("non-UTF-8 root mount".into()))?;
    for dir in ["dev", "sys", "proc"] {
        let target = format!("{root_s}/{dir}");
        let mut cmd = maybe_sudo("umount");
        let _ = cmd.arg(&target).output();
    }
    Ok(())
}

fn run_mkconfig(root: &Path, verbose: bool) -> Result<String> {
    let attempts: &[(&[&str], &str)] = &[
        (&["grub2-mkconfig", "-o", "/boot/grub2/grub.cfg"], "grub2-mkconfig"),
        (&["grub-mkconfig", "-o", "/boot/grub/grub.cfg"], "grub-mkconfig"),
        (&["update-grub"], "update-grub"),
    ];
    let mut last_err = String::new();
    for (argv, name) in attempts {
        if verbose {
            eprintln!("grub_repair: chroot {}", argv.join(" "));
        }
        match chroot_cmd(root, argv) {
            Ok(out) if out.status.success() => return Ok((*name).into()),
            Ok(out) => {
                last_err = format!(
                    "{name}: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
            Err(e) => last_err = format!("{name}: {e}"),
        }
    }
    Err(Error::CommandFailed(last_err))
}

fn run_grub_install(root: &Path, device: &Path, verbose: bool) -> Result<String> {
    let dev = device
        .to_str()
        .ok_or_else(|| Error::InvalidFormat("non-UTF-8 install device".into()))?;
    // Prefer installing to the host device from outside chroot when possible;
    // fall back to chrooted grub-install (needs device visible in guest /dev).
    for tool in ["grub2-install", "grub-install"] {
        if verbose {
            eprintln!("grub_repair: {tool} {dev} (host)");
        }
        let mut cmd = maybe_sudo(tool);
        let output = cmd
            .args(["--root-directory", root.to_str().unwrap_or("/"), dev])
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                return Ok(format!("{tool} (host --root-directory)"));
            }
        }
    }
    for tool in ["grub2-install", "grub-install"] {
        if verbose {
            eprintln!("grub_repair: chroot {tool} {dev}");
        }
        match chroot_cmd(root, &[tool, dev]) {
            Ok(out) if out.status.success() => return Ok(format!("{tool} (chroot)")),
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if verbose {
                    eprintln!("grub_repair: {tool} failed: {}", stderr.trim());
                }
            }
            Err(e) if verbose => eprintln!("grub_repair: {tool}: {e}"),
            Err(_) => {}
        }
    }
    Err(Error::CommandFailed(format!(
        "grub-install failed for {dev}"
    )))
}

/// Script body written for first-boot GRUB regenerate.
pub fn firstboot_grub_script() -> String {
    r#"#!/bin/bash
set -euo pipefail
if command -v update-grub >/dev/null 2>&1; then
  update-grub
elif command -v grub2-mkconfig >/dev/null 2>&1; then
  grub2-mkconfig -o /boot/grub2/grub.cfg
elif command -v grub-mkconfig >/dev/null 2>&1; then
  grub-mkconfig -o /boot/grub/grub.cfg
fi
systemctl disable guestkit-firstboot-grub.service >/dev/null 2>&1 || true
rm -f /etc/systemd/system/multi-user.target.wants/guestkit-firstboot-grub.service
"#
    .to_string()
}

/// Systemd unit body for first-boot GRUB regenerate.
pub fn firstboot_grub_unit() -> String {
    format!(
        r#"[Unit]
Description=GuestKit first-boot GRUB regenerate
After=local-fs.target
ConditionPathExists=/boot

[Service]
Type=oneshot
ExecStart={FIRSTBOOT_SCRIPT}
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
"#
    )
}

fn stage_firstboot_grub(root: &Path) -> Result<()> {
    let script_host = root.join(FIRSTBOOT_SCRIPT.trim_start_matches('/'));
    if let Some(parent) = script_host.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::CommandFailed(format!("mkdir {}: {e}", parent.display())))?;
    }
    fs::write(&script_host, firstboot_grub_script())
        .map_err(|e| Error::CommandFailed(format!("write {}: {e}", script_host.display())))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_host)
            .map_err(|e| Error::CommandFailed(format!("stat script: {e}")))?
            .permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&script_host, perms);
    }

    let unit_host = root.join(FIRSTBOOT_UNIT.trim_start_matches('/'));
    if let Some(parent) = unit_host.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::CommandFailed(format!("mkdir {}: {e}", parent.display())))?;
    }
    fs::write(&unit_host, firstboot_grub_unit())
        .map_err(|e| Error::CommandFailed(format!("write {}: {e}", unit_host.display())))?;

    let wants_dir = root.join("etc/systemd/system/multi-user.target.wants");
    fs::create_dir_all(&wants_dir)
        .map_err(|e| Error::CommandFailed(format!("mkdir wants: {e}")))?;
    let link = root.join(FIRSTBOOT_WANTS.trim_start_matches('/'));
    let _ = fs::remove_file(&link);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("../guestkit-firstboot-grub.service", &link).map_err(|e| {
            Error::CommandFailed(format!("symlink firstboot grub unit: {e}"))
        })?;
    }
    #[cfg(not(unix))]
    {
        let _ = link;
        return Err(Error::InvalidOperation(
            "first-boot GRUB staging requires Unix".into(),
        ));
    }
    Ok(())
}

fn chroot_cmd(root: &Path, argv: &[&str]) -> Result<std::process::Output> {
    let root_s = root
        .to_str()
        .ok_or_else(|| Error::InvalidFormat("non-UTF-8 root".into()))?;
    let mut cmd = maybe_sudo("chroot");
    cmd.arg(root_s);
    for a in argv {
        cmd.arg(a);
    }
    cmd.output()
        .map_err(|e| Error::CommandFailed(format!("chroot {}: {e}", argv.join(" "))))
}

fn maybe_sudo(program: &str) -> Command {
    if nix_is_root() {
        Command::new(program)
    } else {
        let mut c = Command::new("sudo");
        c.arg(program);
        c
    }
}

fn nix_is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Detect a reasonable host block device for `grub-install` from a disk image path.
pub fn infer_install_device(image: &Path) -> Option<PathBuf> {
    // Only use real block devices — never pass a .qcow2 path to grub-install.
    let p = image.to_path_buf();
    if p.starts_with("/dev/") {
        return Some(p);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn infer_skips_image_files() {
        assert!(infer_install_device(Path::new("/tmp/disk.qcow2")).is_none());
        assert_eq!(
            infer_install_device(Path::new("/dev/nbd0")).as_deref(),
            Some(Path::new("/dev/nbd0"))
        );
    }

    #[test]
    fn stage_firstboot_writes_unit() {
        let dir = tempdir().unwrap();
        // Minimal tree
        fs::create_dir_all(dir.path().join("boot")).unwrap();
        stage_firstboot_grub(dir.path()).unwrap();
        assert!(dir
            .path()
            .join("usr/lib/guestkit/firstboot-grub.sh")
            .is_file());
        assert!(dir
            .path()
            .join("etc/systemd/system/guestkit-firstboot-grub.service")
            .is_file());
        assert!(dir
            .path()
            .join("etc/systemd/system/multi-user.target.wants/guestkit-firstboot-grub.service")
            .is_symlink());
    }
}
