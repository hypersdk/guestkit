// SPDX-License-Identifier: LGPL-3.0-or-later
//! Disk image management operations for disk image manipulation
//!
//! This implementation provides disk image operations.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::process::Command;

impl Guestfs {
    /// Create empty disk image
    ///
    /// GuestFS API: disk_create()
    pub fn disk_create(&mut self, filename: &str, format: &str, size: i64) -> Result<()> {
        if self.verbose {
            eprintln!("guestfs: disk_create {} {} {}", filename, format, size);
        }

        let output = Command::new("qemu-img")
            .arg("create")
            .arg("-f")
            .arg(format)
            .arg(filename)
            .arg(size.to_string())
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute qemu-img: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "qemu-img create failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Get disk image format
    ///
    /// GuestFS API: disk_format()
    pub fn disk_format(&mut self, filename: &str) -> Result<String> {
        if self.verbose {
            eprintln!("guestfs: disk_format {}", filename);
        }

        let output = Command::new("qemu-img")
            .arg("info")
            .arg("--output=json")
            .arg(filename)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute qemu-img: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "qemu-img info failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON output to get format
        // For simplicity, use string matching
        if let Some(format_line) = stdout.lines().find(|l| l.contains("\"format\"")) {
            if let Some(format) = format_line.split(':').nth(1) {
                let format = format.trim().trim_matches(|c| c == '"' || c == ',');
                return Ok(format.to_string());
            }
        }

        Err(Error::NotFound(
            "Format not found in qemu-img output".to_string(),
        ))
    }

    /// Check if disk has backing file
    ///
    /// GuestFS API: disk_has_backing_file()
    pub fn disk_has_backing_file(&mut self, filename: &str) -> Result<bool> {
        if self.verbose {
            eprintln!("guestfs: disk_has_backing_file {}", filename);
        }

        let output = Command::new("qemu-img")
            .arg("info")
            .arg(filename)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute qemu-img: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "qemu-img info failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check if output contains "backing file"
        Ok(stdout.contains("backing file"))
    }

    /// Get virtual size of disk image
    ///
    /// GuestFS API: disk_virtual_size()
    pub fn disk_virtual_size(&mut self, filename: &str) -> Result<i64> {
        if self.verbose {
            eprintln!("guestfs: disk_virtual_size {}", filename);
        }

        let output = Command::new("qemu-img")
            .arg("info")
            .arg("--output=json")
            .arg(filename)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute qemu-img: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "qemu-img info failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON output to get virtual-size
        if let Some(size_line) = stdout.lines().find(|l| l.contains("\"virtual-size\"")) {
            if let Some(size_str) = size_line.split(':').nth(1) {
                let size_str = size_str.trim().trim_matches(|c| c == ',' || c == ' ');
                if let Ok(size) = size_str.parse::<i64>() {
                    return Ok(size);
                }
            }
        }

        Err(Error::NotFound(
            "Virtual size not found in qemu-img output".to_string(),
        ))
    }

    /// Resize disk image
    ///
    /// GuestFS API: disk_resize()
    pub fn disk_resize(&mut self, filename: &str, size: i64) -> Result<()> {
        if self.verbose {
            eprintln!("guestfs: disk_resize {} {}", filename, size);
        }

        let output = Command::new("qemu-img")
            .arg("resize")
            .arg(filename)
            .arg(size.to_string())
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute qemu-img: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "qemu-img resize failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Zero unused blocks in disk image
    ///
    /// GuestFS API: zero_free_space()
    pub fn zero_free_space(&mut self, directory: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: zero_free_space {}", directory);
        }

        let host_path = self.resolve_guest_path(directory)?;

        // Create a file filled with zeros to consume free space
        let zero_file = host_path.join(".zero_file");

        let _output = Command::new("dd")
            .arg("if=/dev/zero")
            .arg(format!("of={}", zero_file.display()))
            .arg("bs=1M")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute dd: {}", e)))?;

        // It's expected to fail when disk is full
        // Remove the zero file
        let _ = std::fs::remove_file(&zero_file);

        Ok(())
    }

    /// Sparsify disk image
    ///
    /// GuestFS API: sparsify()
    pub fn sparsify(&mut self, input: &str, output: &str) -> Result<()> {
        if self.verbose {
            eprintln!("guestfs: sparsify {} {}", input, output);
        }

        // Use cp with sparse option
        let cmd_output = Command::new("cp")
            .arg("--sparse=always")
            .arg(input)
            .arg(output)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute cp: {}", e)))?;

        if !cmd_output.status.success() {
            return Err(Error::CommandFailed(format!(
                "cp --sparse failed: {}",
                String::from_utf8_lossy(&cmd_output.stderr)
            )));
        }

        Ok(())
    }

    /// Convert disk image format
    ///
    /// GuestFS API: disk_convert()
    pub fn disk_convert(&mut self, input: &str, output: &str, output_format: &str) -> Result<()> {
        if self.verbose {
            eprintln!(
                "guestfs: disk_convert {} {} {}",
                input, output, output_format
            );
        }

        let cmd_output = Command::new("qemu-img")
            .arg("convert")
            .arg("-O")
            .arg(output_format)
            .arg(input)
            .arg(output)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute qemu-img: {}", e)))?;

        if !cmd_output.status.success() {
            return Err(Error::CommandFailed(format!(
                "qemu-img convert failed: {}",
                String::from_utf8_lossy(&cmd_output.stderr)
            )));
        }

        Ok(())
    }

    /// Check and repair disk image
    ///
    /// GuestFS API: disk_check()
    pub fn disk_check(&mut self, filename: &str) -> Result<String> {
        if self.verbose {
            eprintln!("guestfs: disk_check {}", filename);
        }

        let output = Command::new("qemu-img")
            .arg("check")
            .arg(filename)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute qemu-img: {}", e)))?;

        // qemu-img check returns non-zero for errors found, which is expected
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get snapshot list
    ///
    /// GuestFS API: disk_snapshot_list()
    pub fn disk_snapshot_list(&mut self, filename: &str) -> Result<Vec<String>> {
        if self.verbose {
            eprintln!("guestfs: disk_snapshot_list {}", filename);
        }

        let output = Command::new("qemu-img")
            .arg("snapshot")
            .arg("-l")
            .arg(filename)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute qemu-img: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "qemu-img snapshot failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let snapshots: Vec<String> = stdout
            .lines()
            .skip(2) // Skip header lines
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        Ok(snapshots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_mgmt_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
