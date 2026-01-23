// SPDX-License-Identifier: LGPL-3.0-or-later
//! Advanced disk operations compatible with libguestfs
//!
//! This implementation provides disk-level operations.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::process::Command;

impl Guestfs {
    /// Create swap partition
    ///
    /// Compatible with libguestfs g.mkswap()
    pub fn mkswap(&mut self, device: &str, label: Option<&str>, uuid: Option<&str>) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: mkswap {}", device);
        }

        // Ensure NBD device is set up
        self.setup_nbd_if_needed()?;

        // Parse device name to get partition number
        let partition_num = self.parse_device_name(device)?;

        // Get NBD partition device path
        let nbd = self.nbd_device.as_ref().unwrap();
        let nbd_partition = if partition_num > 0 {
            nbd.partition_path(partition_num)
        } else {
            nbd.device_path().to_path_buf()
        };

        let mut cmd = Command::new("mkswap");

        if let Some(lbl) = label {
            cmd.arg("-L").arg(lbl);
        }

        if let Some(u) = uuid {
            cmd.arg("-U").arg(u);
        }

        cmd.arg(&nbd_partition);

        let output = cmd.output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mkswap: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Mkswap failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Enable swap
    ///
    /// Compatible with libguestfs g.swapon_device()
    pub fn swapon_device(&mut self, device: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: swapon_device {}", device);
        }

        // Ensure NBD device is set up
        self.setup_nbd_if_needed()?;

        let partition_num = self.parse_device_name(device)?;
        let nbd = self.nbd_device.as_ref().unwrap();
        let nbd_partition = if partition_num > 0 {
            nbd.partition_path(partition_num)
        } else {
            nbd.device_path().to_path_buf()
        };

        let output = Command::new("swapon")
            .arg(&nbd_partition)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute swapon: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Swapon failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Disable swap
    ///
    /// Compatible with libguestfs g.swapoff_device()
    pub fn swapoff_device(&mut self, device: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: swapoff_device {}", device);
        }

        // Ensure NBD device is set up
        self.setup_nbd_if_needed()?;

        let partition_num = self.parse_device_name(device)?;
        let nbd = self.nbd_device.as_ref().unwrap();
        let nbd_partition = if partition_num > 0 {
            nbd.partition_path(partition_num)
        } else {
            nbd.device_path().to_path_buf()
        };

        let output = Command::new("swapoff")
            .arg(&nbd_partition)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute swapoff: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Swapoff failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Get hexdump of file
    ///
    /// Compatible with libguestfs g.hexdump()
    pub fn hexdump(&mut self, path: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hexdump {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let output = Command::new("hexdump")
            .arg("-C")
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute hexdump: {}", e)))?;

        if !output.status.success() {
            return Err(Error::NotFound(format!(
                "Hexdump failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get printable strings from file
    ///
    /// Compatible with libguestfs g.strings()
    pub fn strings(&mut self, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: strings {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let output = Command::new("strings")
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute strings: {}", e)))?;

        if !output.status.success() {
            return Err(Error::NotFound(format!(
                "Strings failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Get printable strings from file (with encoding)
    ///
    /// Compatible with libguestfs g.strings_e()
    pub fn strings_e(&mut self, encoding: &str, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: strings_e {} {}", encoding, path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let mut cmd = Command::new("strings");

        // Map encoding to strings flags
        match encoding {
            "b" | "B" => { cmd.arg("-eb"); } // 16-bit big-endian
            "l" | "L" => { cmd.arg("-el"); } // 16-bit little-endian
            "s" | "S" => { cmd.arg("-e").arg("s"); } // 7-bit
            _ => {} // default
        }

        cmd.arg(&host_path);

        let output = cmd.output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute strings: {}", e)))?;

        if !output.status.success() {
            return Err(Error::NotFound(format!(
                "Strings failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Fill device with pattern
    ///
    /// Compatible with libguestfs g.fill()
    pub fn fill(&mut self, c: i32, len: i32, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: fill {} {} {}", c, len, path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Create pattern and write to file
        let pattern = vec![c as u8; len as usize];
        std::fs::write(&host_path, pattern).map_err(|e| {
            Error::CommandFailed(format!("Failed to fill file: {}", e))
        })
    }

    /// Fill device with pattern from source
    ///
    /// Compatible with libguestfs g.fill_pattern()
    pub fn fill_pattern(&mut self, pattern: &str, len: i32, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: fill_pattern {} {} {}", pattern, len, path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Repeat pattern to fill len bytes
        let pattern_bytes = pattern.as_bytes();
        let mut data = Vec::with_capacity(len as usize);

        while data.len() < len as usize {
            let remaining = len as usize - data.len();
            let to_copy = std::cmp::min(remaining, pattern_bytes.len());
            data.extend_from_slice(&pattern_bytes[..to_copy]);
        }

        std::fs::write(&host_path, data).map_err(|e| {
            Error::CommandFailed(format!("Failed to fill file: {}", e))
        })
    }

    /// Fill directory with empty files
    ///
    /// Compatible with libguestfs g.fill_dir()
    pub fn fill_dir(&mut self, dir: &str, nr: i32) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: fill_dir {} {}", dir, nr);
        }

        // Create nr empty files in directory
        for i in 0..nr {
            let filename = format!("{}/{:08x}", dir, i);
            self.touch(&filename)?;
        }

        Ok(())
    }

    /// Get disk identifier
    ///
    /// Compatible with libguestfs g.disk_identifier()
    pub fn disk_identifier(&mut self, device: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: disk_identifier {}", device);
        }

        // Ensure NBD device is set up
        self.setup_nbd_if_needed()?;

        let partition_num = self.parse_device_name(device)?;
        let nbd = self.nbd_device.as_ref().unwrap();
        let nbd_device = if partition_num > 0 {
            // Get disk, not partition
            nbd.device_path().to_path_buf()
        } else {
            nbd.device_path().to_path_buf()
        };

        // Use blkid to get disk identifier
        let output = Command::new("blkid")
            .arg("-s")
            .arg("PTUUID")
            .arg("-o")
            .arg("value")
            .arg(&nbd_device)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute blkid: {}", e)))?;

        if !output.status.success() {
            return Ok(String::new());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Scrub device
    ///
    /// Compatible with libguestfs g.scrub_device()
    pub fn scrub_device(&mut self, device: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: scrub_device {}", device);
        }

        // Ensure NBD device is set up
        self.setup_nbd_if_needed()?;

        let partition_num = self.parse_device_name(device)?;
        let nbd = self.nbd_device.as_ref().unwrap();
        let nbd_partition = if partition_num > 0 {
            nbd.partition_path(partition_num)
        } else {
            nbd.device_path().to_path_buf()
        };

        // Use shred to securely erase
        let output = Command::new("shred")
            .arg("-n")
            .arg("3")
            .arg("-z")
            .arg(&nbd_partition)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute shred: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Scrub failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Scrub file
    ///
    /// Compatible with libguestfs g.scrub_file()
    pub fn scrub_file(&mut self, file: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: scrub_file {}", file);
        }

        let host_path = self.resolve_guest_path(file)?;

        let output = Command::new("shred")
            .arg("-n")
            .arg("3")
            .arg("-z")
            .arg("-u")
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute shred: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Scrub failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_ops_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
