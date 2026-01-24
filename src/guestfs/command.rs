// SPDX-License-Identifier: LGPL-3.0-or-later
//! Command execution inside guest
//!
//! This implementation uses chroot to execute commands inside the
//! mounted guest filesystem.
//!
//! **Requires**: Mounted filesystem and sudo/root permissions

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::process::Command;

impl Guestfs {
    /// Execute a command in the guest
    ///
    /// GuestFS API: command()
    ///
    /// # Arguments
    ///
    /// * `arguments` - Command and arguments as array
    ///
    /// # Returns
    ///
    /// Command output as string
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestkit::guestfs::Guestfs;
    ///
    /// let mut g = Guestfs::new().unwrap();
    /// g.add_drive_ro("/path/to/disk.qcow2").unwrap();
    /// g.launch().unwrap();
    ///
    /// // Mount root filesystem first
    /// g.mount_ro("/dev/sda1", "/").unwrap();
    ///
    /// // Execute command
    /// let output = g.command(&["/bin/ls", "/etc"]).unwrap();
    /// println!("Output: {}", output);
    /// ```
    pub fn command(&mut self, arguments: &[&str]) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: command {:?}", arguments);
        }

        if arguments.is_empty() {
            return Err(Error::InvalidFormat("No command provided".to_string()));
        }

        // Get root mount point
        let root_mountpoint = self.mounted.get("/dev/sda1")
            .or_else(|| self.mounted.get("/dev/sda2"))
            .or_else(|| self.mounted.get("/dev/vda1"))
            .or_else(|| self.mounted.values().next())
            .ok_or_else(|| Error::InvalidState(
                "No filesystem mounted. Call mount_ro() first.".to_string()
            ))?;

        // Execute command using chroot
        let output = Command::new("chroot")
            .arg(root_mountpoint)
            .args(arguments)
            .output()
            .map_err(|e| Error::CommandFailed(format!(
                "Failed to execute command via chroot: {}. Requires sudo/root.", e
            )))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!(
                "Command failed with exit code {:?}: {}",
                output.status.code(),
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Execute a command and return output as lines
    ///
    /// GuestFS API: command_lines()
    pub fn command_lines(&mut self, arguments: &[&str]) -> Result<Vec<String>> {
        let output = self.command(arguments)?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    /// Execute a shell command
    ///
    /// GuestFS API: sh()
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestkit::guestfs::Guestfs;
    ///
    /// let mut g = Guestfs::new().unwrap();
    /// // ... setup ...
    ///
    /// let output = g.sh("cat /etc/hostname").unwrap();
    /// ```
    pub fn sh(&mut self, command: &str) -> Result<String> {
        self.command(&["/bin/sh", "-c", command])
    }

    /// Execute a shell command and return output as lines
    ///
    /// GuestFS API: sh_lines()
    pub fn sh_lines(&mut self, command: &str) -> Result<Vec<String>> {
        let output = self.sh(command)?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure test - will fail without implementation
        let _ = g;
    }
}
