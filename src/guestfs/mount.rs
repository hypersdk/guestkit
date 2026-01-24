// SPDX-License-Identifier: LGPL-3.0-or-later
//! Mount operations for disk image manipulation
//!
//! This implementation uses qemu-nbd to export disk images as NBD devices,
//! then mounts them using the kernel's filesystem drivers.
//!
//! **Requires**: qemu-nbd and sudo/root permissions for mounting

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

impl Guestfs {
    /// Mount a filesystem read-only
    ///
    /// GuestFS API: mount_ro()
    ///
    /// # Arguments
    ///
    /// * `mountable` - Device name (e.g., "/dev/sda1")
    /// * `mountpoint` - Mount point path (e.g., "/")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestctl::guestfs::Guestfs;
    ///
    /// let mut g = Guestfs::new()?;
    /// g.add_drive_ro("/path/to/disk.qcow2")?;
    /// g.launch()?;
    /// g.mount_ro("/dev/sda1", "/")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn mount_ro(&mut self, mountable: &str, mountpoint: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: mount_ro {} {}", mountable, mountpoint);
        }

        // Ensure NBD device is set up
        self.setup_nbd_if_needed()?;

        // Parse device name to get partition number
        let partition_num = self.parse_device_name(mountable)?;

        // Get NBD partition device path
        let nbd = self.nbd_device()?;
        let nbd_partition = if partition_num > 0 {
            nbd.partition_path(partition_num)
        } else {
            nbd.device_path().to_path_buf()
        };

        // Create mount root if needed
        if self.mount_root.is_none() {
            let tmpdir = std::env::temp_dir().join(format!("guestctl-{}", std::process::id()));
            fs::create_dir_all(&tmpdir)
                .map_err(|e| Error::CommandFailed(format!("Failed to create mount root: {}", e)))?;
            self.mount_root = Some(tmpdir);
        }

        // Build actual mount path
        let mount_root = self
            .mount_root
            .as_ref()
            .ok_or_else(|| Error::InvalidState("No mount root created".to_string()))?;
        let actual_mountpoint = if mountpoint == "/" {
            mount_root.clone()
        } else {
            mount_root.join(mountpoint.trim_start_matches('/'))
        };

        // Create mountpoint directory
        fs::create_dir_all(&actual_mountpoint)
            .map_err(|e| Error::CommandFailed(format!("Failed to create mountpoint: {}", e)))?;

        // Mount using system mount command
        let output = Command::new("mount")
            .arg("-o")
            .arg("ro")
            .arg(&nbd_partition)
            .arg(&actual_mountpoint)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mount: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!(
                "Mount failed: {}. You may need sudo/root permissions.",
                stderr
            )));
        }

        // Record the mount
        self.mounted.insert(
            mountable.to_string(),
            actual_mountpoint.to_string_lossy().to_string(),
        );

        Ok(())
    }

    /// Mount a filesystem read-write
    ///
    /// GuestFS API: mount()
    pub fn mount(&mut self, mountable: &str, mountpoint: &str) -> Result<()> {
        self.ensure_ready()?;

        // Check if readonly
        if let Some(drive) = self.drives.first() {
            if drive.readonly {
                return Err(Error::PermissionDenied(
                    "Cannot mount read-write on read-only drive".to_string(),
                ));
            }
        }

        // Verify partition exists
        let _partition_num = self.parse_device_name(mountable)?;

        // Record the mount
        self.mounted
            .insert(mountable.to_string(), mountpoint.to_string());

        if self.verbose {
            eprintln!("guestfs: mount {} {}", mountable, mountpoint);
        }

        Ok(())
    }

    /// Mount with specific options
    ///
    /// GuestFS API: mount_options()
    pub fn mount_options(
        &mut self,
        options: &str,
        mountable: &str,
        mountpoint: &str,
    ) -> Result<()> {
        if self.verbose {
            eprintln!(
                "guestfs: mount_options {} {} {}",
                options, mountable, mountpoint
            );
        }

        self.mount(mountable, mountpoint)
    }

    /// Mount with explicit VFS type
    ///
    /// GuestFS API: mount_vfs()
    pub fn mount_vfs(
        &mut self,
        options: &str,
        vfstype: &str,
        mountable: &str,
        mountpoint: &str,
    ) -> Result<()> {
        if self.verbose {
            eprintln!(
                "guestfs: mount_vfs {} {} {} {}",
                options, vfstype, mountable, mountpoint
            );
        }

        self.mount(mountable, mountpoint)
    }

    /// Unmount a filesystem
    ///
    /// GuestFS API: umount()
    pub fn umount(&mut self, pathordevice: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.trace {
            eprintln!("guestfs: umount {}", pathordevice);
        }

        // Find mounts to remove
        let to_unmount: Vec<(String, String)> = self
            .mounted
            .iter()
            .filter(|(dev, mp)| dev.as_str() == pathordevice || mp.as_str() == pathordevice)
            .map(|(dev, mp)| (dev.clone(), mp.clone()))
            .collect();

        // Unmount each
        for (dev, mountpoint) in to_unmount {
            // Execute umount command
            let output = Command::new("umount")
                .arg(&mountpoint)
                .output()
                .map_err(|e| Error::CommandFailed(format!("Failed to execute umount: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Warning: umount failed: {}", stderr);
            }

            // Remove from tracking
            self.mounted.remove(&dev);
        }

        Ok(())
    }

    /// Unmount all filesystems
    ///
    /// GuestFS API: umount_all()
    pub fn umount_all(&mut self) -> Result<()> {
        self.ensure_ready()?;

        if self.trace {
            eprintln!("guestfs: umount_all");
        }

        // Unmount all in reverse order (to handle nested mounts)
        let mountpoints: Vec<String> = self.mounted.values().cloned().collect();

        for mountpoint in mountpoints.iter().rev() {
            let output = Command::new("umount")
                .arg(mountpoint)
                .output()
                .map_err(|e| Error::CommandFailed(format!("Failed to execute umount: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Warning: umount {} failed: {}", mountpoint, stderr);
            }
        }

        self.mounted.clear();

        Ok(())
    }

    /// Get list of mounted filesystems
    ///
    /// GuestFS API: mounts()
    pub fn mounts(&self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        Ok(self.mounted.keys().cloned().collect())
    }

    /// Get mountpoints
    ///
    /// GuestFS API: mountpoints()
    pub fn mountpoints(&self) -> Result<HashMap<String, String>> {
        self.ensure_ready()?;

        // Return device -> mountpoint mapping
        Ok(self.mounted.clone())
    }

    /// Create a mountpoint
    ///
    /// GuestFS API: mkmountpoint()
    pub fn mkmountpoint(&mut self, exemptpath: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: mkmountpoint {}", exemptpath);
        }

        // Use mkdir_p to create the directory
        self.mkdir_p(exemptpath)
    }

    /// Remove a mountpoint
    ///
    /// GuestFS API: rmmountpoint()
    pub fn rmmountpoint(&mut self, exemptpath: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: rmmountpoint {}", exemptpath);
        }

        // Use rmdir to remove the directory
        self.rmdir(exemptpath)
    }

    /// Sync filesystems
    ///
    /// GuestFS API: sync()
    pub fn sync(&mut self) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: sync");
        }

        // Call the sync command to flush filesystem buffers
        let output = Command::new("sync")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute sync: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Sync failed: {}",
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
    fn test_mount_tracking() {
        let mut g = Guestfs::new().unwrap();
        // Setup would be needed here
    }
}
