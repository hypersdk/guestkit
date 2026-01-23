// SPDX-License-Identifier: LGPL-3.0-or-later
//! XFS filesystem operations compatible with libguestfs
//!
//! This implementation provides XFS-specific functionality.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::process::Command;

impl Guestfs {
    /// Repair XFS filesystem
    ///
    /// Compatible with libguestfs g.xfs_repair()
    pub fn xfs_repair(&mut self, device: &str, forcelogzero: bool, nomodify: bool) -> Result<i32> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: xfs_repair {} {} {}", device, forcelogzero, nomodify);
        }

        self.setup_nbd_if_needed()?;

        let nbd_partition = if let Some(partition_number) = device.chars().last().and_then(|c| c.to_digit(10)) {
            let nbd_device = self.nbd_device.as_ref()
                .ok_or_else(|| Error::InvalidState("NBD device not available".to_string()))?;
            format!("{}p{}", nbd_device.device_path().display(), partition_number)
        } else {
            return Err(Error::InvalidFormat(format!("Invalid device: {}", device)));
        };

        let mut cmd = Command::new("xfs_repair");

        if forcelogzero {
            cmd.arg("-L"); // Force log zeroing
        }

        if nomodify {
            cmd.arg("-n"); // No modify, just check
        }

        cmd.arg(&nbd_partition);

        let output = cmd.output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute xfs_repair: {}", e)))?;

        Ok(output.status.code().unwrap_or(1))
    }

    /// Get XFS filesystem info
    ///
    /// Compatible with libguestfs g.xfs_info()
    pub fn xfs_info(&mut self, pathordevice: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: xfs_info {}", pathordevice);
        }

        // Check if it's a device or a path
        let target = if pathordevice.starts_with("/dev/") {
            // It's a device
            self.setup_nbd_if_needed()?;

            if let Some(partition_number) = pathordevice.chars().last().and_then(|c| c.to_digit(10)) {
                let nbd_device = self.nbd_device.as_ref()
                    .ok_or_else(|| Error::InvalidState("NBD device not available".to_string()))?;
                format!("{}p{}", nbd_device.device_path().display(), partition_number)
            } else {
                return Err(Error::InvalidFormat(format!("Invalid device: {}", pathordevice)));
            }
        } else {
            // It's a path
            let host_path = self.resolve_guest_path(pathordevice)?;
            host_path.to_string_lossy().to_string()
        };

        let output = Command::new("xfs_info")
            .arg(&target)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute xfs_info: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "xfs_info failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Admin XFS filesystem
    ///
    /// Compatible with libguestfs g.xfs_admin()
    pub fn xfs_admin(&mut self, device: &str, extunwritten: bool, imgfile: bool, v2log: bool,
                      projid32bit: bool, lazycounter: bool, label: Option<&str>, uuid: Option<&str>) -> Result<i32> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: xfs_admin {}", device);
        }

        self.setup_nbd_if_needed()?;

        let nbd_partition = if let Some(partition_number) = device.chars().last().and_then(|c| c.to_digit(10)) {
            let nbd_device = self.nbd_device.as_ref()
                .ok_or_else(|| Error::InvalidState("NBD device not available".to_string()))?;
            format!("{}p{}", nbd_device.device_path().display(), partition_number)
        } else {
            return Err(Error::InvalidFormat(format!("Invalid device: {}", device)));
        };

        let mut cmd = Command::new("xfs_admin");

        if extunwritten {
            cmd.arg("-e");
        }
        if imgfile {
            cmd.arg("-i");
        }
        if v2log {
            cmd.arg("-j");
        }
        if projid32bit {
            cmd.arg("-p");
        }
        if lazycounter {
            cmd.arg("-c").arg("1");
        }

        if let Some(l) = label {
            cmd.arg("-L").arg(l);
        }

        if let Some(u) = uuid {
            cmd.arg("-U").arg(u);
        }

        cmd.arg(&nbd_partition);

        let output = cmd.output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute xfs_admin: {}", e)))?;

        Ok(output.status.code().unwrap_or(1))
    }

    /// Get XFS inode count
    ///
    /// Compatible with libguestfs g.xfs_db()
    pub fn xfs_db(&mut self, device: &str, command: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: xfs_db {} {}", device, command);
        }

        self.setup_nbd_if_needed()?;

        let nbd_partition = if let Some(partition_number) = device.chars().last().and_then(|c| c.to_digit(10)) {
            let nbd_device = self.nbd_device.as_ref()
                .ok_or_else(|| Error::InvalidState("NBD device not available".to_string()))?;
            format!("{}p{}", nbd_device.device_path().display(), partition_number)
        } else {
            return Err(Error::InvalidFormat(format!("Invalid device: {}", device)));
        };

        let output = Command::new("xfs_db")
            .arg("-r") // Read-only
            .arg("-c")
            .arg(command)
            .arg(&nbd_partition)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute xfs_db: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "xfs_db failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xfs_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
