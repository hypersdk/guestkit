// SPDX-License-Identifier: LGPL-3.0-or-later
//! Command execution inside guest
//!
//! NOTE: Command execution requires a running guest with shell access.
//! This needs either:
//! 1. Guest agent (like qemu-guest-agent)
//! 2. SSH/network access to guest
//! 3. Mounted filesystem + chroot
//!
//! This provides the API structure for future implementation.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;

impl Guestfs {
    /// Execute a command in the guest
    ///
    /// Compatible with libguestfs g.command()
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

        // TODO: Implement command execution
        // Options:
        // 1. Use guest agent (qemu-guest-agent)
        // 2. Use chroot after NBD mount
        // 3. Use libvirt domExecCommand API

        Err(Error::Unsupported(
            "Command execution requires guest agent or mount implementation".to_string()
        ))
    }

    /// Execute a command and return output as lines
    ///
    /// Compatible with libguestfs g.command_lines()
    pub fn command_lines(&mut self, arguments: &[&str]) -> Result<Vec<String>> {
        let output = self.command(arguments)?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    /// Execute a shell command
    ///
    /// Compatible with libguestfs g.sh()
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
    /// Compatible with libguestfs g.sh_lines()
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
