// SPDX-License-Identifier: LGPL-3.0-or-later
//! LVM (Logical Volume Manager) operations for disk image manipulation
//!
//! This implementation uses LVM command-line tools (lvm2 package).
//!
//! **Requires**: lvm2 package and sudo/root permissions

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::process::Command;

/// Logical volume information
#[derive(Debug, Clone)]
pub struct LV {
    pub lv_name: String,
    pub lv_uuid: String,
    pub lv_attr: String,
    pub lv_major: i64,
    pub lv_minor: i64,
    pub lv_kernel_major: i64,
    pub lv_kernel_minor: i64,
    pub lv_size: i64,
    pub seg_count: i64,
    pub origin: String,
    pub snap_percent: f32,
    pub copy_percent: f32,
    pub move_pv: String,
    pub lv_tags: String,
    pub mirror_log: String,
    pub modules: String,
}

impl Guestfs {
    /// Scan for LVM volume groups
    ///
    /// GuestFS API: vgscan()
    pub fn vgscan(&mut self) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: vgscan");
        }

        // Ensure NBD device is set up for LVM to detect
        self.setup_nbd_if_needed()?;

        // Run vgscan to detect volume groups
        let output = Command::new("vgscan")
            .arg("--mknodes")
            .output()
            .map_err(|e| {
                Error::CommandFailed(format!(
                    "Failed to run vgscan: {}. Is lvm2 installed? Requires sudo/root.",
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("vgscan failed: {}", stderr)));
        }

        if self.verbose {
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("vgscan output: {}", stdout);
        }

        Ok(())
    }

    /// Activate all LVM volume groups
    ///
    /// GuestFS API: vg_activate_all()
    pub fn vg_activate_all(&mut self, activate: bool) -> Result<()> {
        self.ensure_ready()?;

        let action = if activate { "y" } else { "n" };

        if self.verbose {
            eprintln!("guestfs: vg_activate_all {}", activate);
        }

        // Run vgchange to activate/deactivate all VGs
        let output = Command::new("vgchange")
            .arg("-a")
            .arg(action)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run vgchange: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("vgchange failed: {}", stderr)));
        }

        if self.verbose {
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("vgchange output: {}", stdout);
        }

        Ok(())
    }

    /// Activate a specific volume group
    ///
    /// GuestFS API: vg_activate()
    pub fn vg_activate(&mut self, activate: bool, volgroups: &[&str]) -> Result<()> {
        self.ensure_ready()?;

        let action = if activate { "y" } else { "n" };

        if self.verbose {
            eprintln!("guestfs: vg_activate {} {:?}", activate, volgroups);
        }

        for vg in volgroups {
            let output = Command::new("vgchange")
                .arg("-a")
                .arg(action)
                .arg(vg)
                .output()
                .map_err(|e| Error::CommandFailed(format!("Failed to run vgchange: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::CommandFailed(format!(
                    "vgchange {} failed: {}",
                    vg, stderr
                )));
            }

            if self.verbose {
                let stdout = String::from_utf8_lossy(&output.stdout);
                eprintln!("vgchange {} output: {}", vg, stdout);
            }
        }

        Ok(())
    }

    /// Create a logical volume
    ///
    /// GuestFS API: lvcreate()
    ///
    /// # Arguments
    ///
    /// * `logvol` - Name of logical volume to create
    /// * `volgroup` - Name of volume group
    /// * `mbytes` - Size in megabytes
    pub fn lvcreate(&mut self, logvol: &str, volgroup: &str, mbytes: i32) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: lvcreate {} {} {}M", logvol, volgroup, mbytes);
        }

        // Check if drive is readonly
        if let Some(drive) = self.drives.first() {
            if drive.readonly {
                return Err(Error::PermissionDenied(
                    "Cannot create LV on read-only drive".to_string(),
                ));
            }
        }

        // Create logical volume
        let output = Command::new("lvcreate")
            .arg("-L")
            .arg(format!("{}M", mbytes))
            .arg("-n")
            .arg(logvol)
            .arg(volgroup)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run lvcreate: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("lvcreate failed: {}", stderr)));
        }

        Ok(())
    }

    /// Remove a logical volume
    ///
    /// GuestFS API: lvremove()
    ///
    /// # Arguments
    ///
    /// * `device` - Logical volume device path (e.g., "/dev/vg/lv")
    pub fn lvremove(&mut self, device: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: lvremove {}", device);
        }

        // Remove logical volume
        let output = Command::new("lvremove")
            .arg("-f") // Force, don't ask for confirmation
            .arg(device)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run lvremove: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("lvremove failed: {}", stderr)));
        }

        Ok(())
    }

    /// List logical volumes with full details
    ///
    /// GuestFS API: lvs_full()
    pub fn lvs_full(&self) -> Result<Vec<LV>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: lvs_full");
        }

        // Use lvs with specific output fields
        let output = Command::new("lvs")
            .arg("--noheadings")
            .arg("--separator")
            .arg("|")
            .arg("-o")
            .arg("lv_name,lv_uuid,lv_attr,lv_major,lv_minor,lv_kernel_major,lv_kernel_minor,lv_size,seg_count,origin,snap_percent,copy_percent,move_pv,lv_tags,mirror_log,modules")
            .arg("--units")
            .arg("b")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute lvs: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!(
                "LVS command failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lvs = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 16 {
                continue;
            }

            // Parse size (remove 'B' suffix)
            let size_str = parts[7].trim().trim_end_matches('B');
            let lv_size = size_str.parse::<i64>().unwrap_or(0);

            lvs.push(LV {
                lv_name: parts[0].trim().to_string(),
                lv_uuid: parts[1].trim().to_string(),
                lv_attr: parts[2].trim().to_string(),
                lv_major: parts[3].trim().parse::<i64>().unwrap_or(-1),
                lv_minor: parts[4].trim().parse::<i64>().unwrap_or(-1),
                lv_kernel_major: parts[5].trim().parse::<i64>().unwrap_or(-1),
                lv_kernel_minor: parts[6].trim().parse::<i64>().unwrap_or(-1),
                lv_size,
                seg_count: parts[8].trim().parse::<i64>().unwrap_or(0),
                origin: parts[9].trim().to_string(),
                snap_percent: parts[10].trim().parse::<f32>().unwrap_or(0.0),
                copy_percent: parts[11].trim().parse::<f32>().unwrap_or(0.0),
                move_pv: parts[12].trim().to_string(),
                lv_tags: parts[13].trim().to_string(),
                mirror_log: parts[14].trim().to_string(),
                modules: parts[15].trim().to_string(),
            });
        }

        Ok(lvs)
    }

    /// List logical volumes (simple)
    ///
    /// GuestFS API: lvs()
    pub fn lvs(&self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: lvs");
        }

        // List logical volumes
        let output = Command::new("lvs")
            .arg("--noheadings")
            .arg("-o")
            .arg("lv_path")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run lvs: {}", e)))?;

        if !output.status.success() {
            // Not an error if no LVs found
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lvs: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(lvs)
    }

    /// List volume groups
    ///
    /// GuestFS API: vgs()
    pub fn vgs(&self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: vgs");
        }

        // List volume groups
        let output = Command::new("vgs")
            .arg("--noheadings")
            .arg("-o")
            .arg("vg_name")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run vgs: {}", e)))?;

        if !output.status.success() {
            // Not an error if no VGs found
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let vgs: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(vgs)
    }

    /// List physical volumes
    ///
    /// GuestFS API: pvs()
    pub fn pvs(&self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: pvs");
        }

        // List physical volumes
        let output = Command::new("pvs")
            .arg("--noheadings")
            .arg("-o")
            .arg("pv_name")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run pvs: {}", e)))?;

        if !output.status.success() {
            // Not an error if no PVs found
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pvs: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(pvs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lvm_api_exists() {
        let g = Guestfs::new().unwrap();
        // API structure tests
    }
}
