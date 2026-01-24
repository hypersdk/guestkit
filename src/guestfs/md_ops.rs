// SPDX-License-Identifier: LGPL-3.0-or-later
//! MD/RAID operations for disk image manipulation
//!
//! This implementation provides software RAID management functionality.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::process::Command;

impl Guestfs {
    /// Create RAID array
    ///
    /// GuestFS API: md_create()
    pub fn md_create(
        &mut self,
        name: &str,
        devices: &[&str],
        _missingbitmap: i64,
        nrdevices: i32,
        spare: i32,
        chunk: i64,
        level: &str,
    ) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: md_create {} {:?}", name, devices);
        }

        self.setup_nbd_if_needed()?;

        let nbd_device = self
            .nbd_device
            .as_ref()
            .ok_or_else(|| Error::InvalidState("NBD device not available".to_string()))?;

        // Convert device names to NBD partitions
        let mut nbd_partitions = Vec::new();
        for device in devices {
            if let Some(partition_number) = device.chars().last().and_then(|c| c.to_digit(10)) {
                nbd_partitions.push(format!(
                    "{}p{}",
                    nbd_device.device_path().display(),
                    partition_number
                ));
            }
        }

        let mut cmd = Command::new("mdadm");
        cmd.arg("--create")
            .arg(format!("/dev/md/{}", name))
            .arg("--level")
            .arg(level)
            .arg("--raid-devices")
            .arg(nrdevices.to_string());

        if spare > 0 {
            cmd.arg("--spare-devices").arg(spare.to_string());
        }

        if chunk > 0 {
            cmd.arg("--chunk").arg(chunk.to_string());
        }

        for partition in &nbd_partitions {
            cmd.arg(partition);
        }

        let output = cmd
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mdadm: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "mdadm create failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Stop RAID array
    ///
    /// GuestFS API: md_stop()
    pub fn md_stop(&mut self, md: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: md_stop {}", md);
        }

        let output = Command::new("mdadm")
            .arg("--stop")
            .arg(format!("/dev/md/{}", md))
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mdadm: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "mdadm stop failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Get RAID array details
    ///
    /// GuestFS API: md_detail()
    pub fn md_detail(&mut self, md: &str) -> Result<Vec<(String, String)>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: md_detail {}", md);
        }

        let output = Command::new("mdadm")
            .arg("--detail")
            .arg(format!("/dev/md/{}", md))
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mdadm: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "mdadm detail failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut details = Vec::new();

        for line in output_str.lines() {
            if let Some((key, value)) = line.split_once(':') {
                details.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        Ok(details)
    }

    /// List MD devices
    ///
    /// GuestFS API: list_md_devices()
    pub fn list_md_devices(&mut self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: list_md_devices");
        }

        let output = Command::new("mdadm")
            .arg("--detail")
            .arg("--scan")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mdadm: {}", e)))?;

        if !output.status.success() {
            // No MD devices is not an error
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<String> = output_str
            .lines()
            .filter_map(|line| line.split_whitespace().nth(1).map(|s| s.to_string()))
            .collect();

        Ok(devices)
    }

    /// Get MD array stat
    ///
    /// GuestFS API: md_stat()
    pub fn md_stat(&mut self, md: &str) -> Result<Vec<(String, i64)>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: md_stat {}", md);
        }

        let detail = self.md_detail(md)?;
        let mut stats = Vec::new();

        for (key, value) in detail {
            // Try to parse numeric values
            if let Ok(num) = value.parse::<i64>() {
                stats.push((key, num));
            }
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md_ops_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
