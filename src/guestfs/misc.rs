// SPDX-License-Identifier: LGPL-3.0-or-later
//! Miscellaneous utility operations for disk image manipulation
//!
//! This implementation provides various utility functions.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::process::Command;

impl Guestfs {
    /// Get available memory in guest
    ///
    /// GuestFS API: available_all_groups()
    pub fn get_meminfo(&mut self) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_meminfo");
        }

        // Read /proc/meminfo if available
        if self.exists("/proc/meminfo")? {
            self.cat("/proc/meminfo")
        } else {
            Err(Error::NotFound("/proc/meminfo not found".to_string()))
        }
    }

    /// Get library version
    ///
    /// GuestFS API: version()
    pub fn version(&self) -> Result<(i64, i64, i64)> {
        // Return guestkit version
        Ok((0, 1, 0))
    }

    /// Get filesystem disk usage
    ///
    /// GuestFS API: disk_usage()
    pub fn disk_usage(&mut self, path: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: disk_usage {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let output = Command::new("du")
            .arg("-sb") // Sum in bytes
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute du: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "du failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(size_str) = stdout.split_whitespace().next() {
            if let Ok(size) = size_str.parse::<i64>() {
                return Ok(size);
            }
        }

        Err(Error::CommandFailed(
            "Could not parse du output".to_string(),
        ))
    }

    /// Get file type (using libmagic-style output)
    ///
    /// GuestFS API: file_type()
    pub fn file_type(&mut self, path: &str) -> Result<String> {
        // Wrapper around file() to get just the type
        self.file(path)
    }

    /// Get available features
    ///
    /// GuestFS API: available()
    pub fn available(&mut self, groups: &[&str]) -> Result<bool> {
        if self.verbose {
            eprintln!("guestfs: available {:?}", groups);
        }

        // Check if requested features are available
        for group in groups {
            match *group {
                "luks" => {
                    // Check if cryptsetup is available
                    if Command::new("which").arg("cryptsetup").output().is_err() {
                        return Ok(false);
                    }
                }
                "lvm2" => {
                    // Check if lvm is available
                    if Command::new("which").arg("lvm").output().is_err() {
                        return Ok(false);
                    }
                }
                "btrfs" => {
                    // Check if btrfs is available
                    if Command::new("which").arg("btrfs").output().is_err() {
                        return Ok(false);
                    }
                }
                "xfs" => {
                    // Check if xfs tools are available
                    if Command::new("which").arg("xfs_repair").output().is_err() {
                        return Ok(false);
                    }
                }
                _ => {
                    // Unknown group - assume not available
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// List all available feature groups
    ///
    /// GuestFS API: available_all_groups()
    pub fn available_all_groups(&mut self) -> Result<Vec<String>> {
        if self.verbose {
            eprintln!("guestfs: available_all_groups");
        }

        let mut groups = vec!["disk".to_string()];

        // Check for optional features
        if Command::new("which").arg("cryptsetup").output().is_ok() {
            groups.push("luks".to_string());
        }
        if Command::new("which").arg("lvm").output().is_ok() {
            groups.push("lvm2".to_string());
        }
        if Command::new("which").arg("btrfs").output().is_ok() {
            groups.push("btrfs".to_string());
        }
        if Command::new("which").arg("xfs_repair").output().is_ok() {
            groups.push("xfs".to_string());
        }
        if Command::new("which").arg("mkfs.ext4").output().is_ok() {
            groups.push("ext2".to_string());
        }
        if Command::new("which").arg("ntfsresize").output().is_ok() {
            groups.push("ntfs3g".to_string());
        }

        Ok(groups)
    }

    /// Get path to daemon socket
    ///
    /// GuestFS API: get_sockdir()
    pub fn get_sockdir(&self) -> Result<String> {
        // For our implementation, we don't use a socket
        Ok("/tmp".to_string())
    }

    /// Get cache directory
    ///
    /// GuestFS API: get_cachedir()
    pub fn get_cachedir(&self) -> Result<String> {
        Ok("/var/cache/guestkit".to_string())
    }

    /// Get temporary directory
    ///
    /// GuestFS API: get_tmpdir()
    pub fn get_tmpdir(&self) -> Result<String> {
        Ok("/tmp".to_string())
    }

    /// Get identifier
    ///
    /// GuestFS API: get_identifier()
    pub fn get_identifier(&self) -> Result<String> {
        if let Some(ref id) = self.identifier {
            Ok(id.clone())
        } else {
            Ok(String::new())
        }
    }

    /// Set identifier
    ///
    /// GuestFS API: set_identifier()
    pub fn set_identifier(&mut self, id: &str) -> Result<()> {
        self.identifier = Some(id.to_string());
        Ok(())
    }

    /// Get program name
    ///
    /// GuestFS API: get_program()
    pub fn get_program(&self) -> Result<String> {
        Ok("guestkit".to_string())
    }

    /// Get library path
    ///
    /// GuestFS API: get_path()
    pub fn get_path(&self) -> Result<String> {
        Ok("/usr/lib/guestkit".to_string())
    }

    /// Get HV (hypervisor) type
    ///
    /// GuestFS API: get_hv()
    pub fn get_hv(&self) -> Result<String> {
        // We use NBD, not a full hypervisor
        Ok("nbd".to_string())
    }

    /// Get autosync setting
    ///
    /// GuestFS API: get_autosync()
    pub fn get_autosync(&self) -> Result<bool> {
        Ok(self.autosync)
    }

    /// Set autosync
    ///
    /// GuestFS API: set_autosync()
    pub fn set_autosync(&mut self, autosync: bool) -> Result<()> {
        self.autosync = autosync;
        Ok(())
    }

    /// Get SELinux context
    ///
    /// GuestFS API: get_selinux()
    pub fn get_selinux(&self) -> Result<bool> {
        Ok(self.selinux)
    }

    /// Set SELinux context
    ///
    /// GuestFS API: set_selinux()
    pub fn set_selinux(&mut self, selinux: bool) -> Result<()> {
        self.selinux = selinux;
        Ok(())
    }

    /// Check if read-only
    ///
    /// GuestFS API: get_readonly()
    pub fn get_readonly(&self) -> Result<bool> {
        Ok(self.readonly)
    }

    /// Get attach method
    ///
    /// GuestFS API: get_attach_method()
    pub fn get_attach_method(&self) -> Result<String> {
        Ok("nbd".to_string())
    }

    /// Get backend
    ///
    /// GuestFS API: get_backend()
    pub fn get_backend(&self) -> Result<String> {
        Ok("direct".to_string())
    }

    /// Internal test command
    ///
    /// GuestFS API: debug()
    pub fn debug(&mut self, subcmd: &str, extraargs: &[&str]) -> Result<String> {
        if self.verbose {
            eprintln!("guestfs: debug {} {:?}", subcmd, extraargs);
        }

        match subcmd {
            "ls" => {
                // Debug ls command
                if extraargs.is_empty() {
                    return Err(Error::InvalidFormat(
                        "debug ls requires path argument".to_string(),
                    ));
                }
                self.ls(extraargs[0]).map(|v| v.join("\n"))
            }
            "help" => Ok("Available debug commands: ls".to_string()),
            _ => Err(Error::Unsupported(format!(
                "Unknown debug command: {}",
                subcmd
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_misc_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
