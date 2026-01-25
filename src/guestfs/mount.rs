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

        // Determine the actual device path to mount
        let device_partition = if mountable.starts_with("/dev/mapper/") {
            // LVM logical volume - use the path directly
            std::path::PathBuf::from(mountable)
        } else {
            // Parse device name to get partition number
            let partition_num = self.parse_device_name(mountable)?;

            // Get the actual device path (loop or NBD)
            if let Some(loop_dev) = &self.loop_device {
                // Using loop device
                if partition_num > 0 {
                    loop_dev.partition_path(partition_num)
                        .ok_or_else(|| Error::InvalidState("Loop device not connected".to_string()))?
                } else {
                    loop_dev.device_path()
                        .ok_or_else(|| Error::InvalidState("Loop device not connected".to_string()))?
                        .to_path_buf()
                }
            } else if let Some(nbd) = &self.nbd_device {
                // Using NBD device
                if partition_num > 0 {
                    nbd.partition_path(partition_num)
                } else {
                    nbd.device_path().to_path_buf()
                }
            } else {
                return Err(Error::InvalidState(
                    "No block device available (neither loop nor NBD)".to_string(),
                ));
            }
        };

        // Create mount root if needed
        if self.mount_root.is_none() {
            // Use /run instead of /tmp for runtime mounts (tmpfs, faster, auto-cleanup)
            let mount_dir = std::path::PathBuf::from("/run")
                .join(format!("guestctl-{}", std::process::id()));
            fs::create_dir_all(&mount_dir)
                .map_err(|e| Error::CommandFailed(format!("Failed to create mount root: {}", e)))?;
            self.mount_root = Some(mount_dir);
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

        // Check if we need sudo for mount
        let need_sudo = unsafe { libc::geteuid() } != 0;

        // Detect filesystem type to use appropriate mount options
        let fs_type = self.vfs_type(&mountable).unwrap_or_else(|_| "ext4".to_string());

        // Build mount command
        let mut cmd = if need_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("mount");
            sudo_cmd
        } else {
            Command::new("mount")
        };

        // Use filesystem-specific mount options
        // For ext* filesystems: use noload to prevent journal updates on read-only mounts
        // For btrfs and others: just use ro
        let mount_opts = if fs_type.starts_with("ext") {
            "ro,noload"
        } else {
            "ro"
        };

        let output = cmd
            .arg("-o")
            .arg(mount_opts)
            .arg(&device_partition)
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

        if to_unmount.is_empty() {
            return Err(Error::NotFound(format!(
                "No filesystem mounted at {}",
                pathordevice
            )));
        }

        // Check if we need sudo
        let need_sudo = unsafe { libc::geteuid() } != 0;

        // Unmount each
        for (dev, mountpoint) in to_unmount {
            if self.trace {
                eprintln!("guestfs: unmounting {} ({})", dev, mountpoint);
            }

            // Build umount command
            let mut cmd = if need_sudo {
                let mut sudo_cmd = Command::new("sudo");
                sudo_cmd.arg("umount");
                sudo_cmd
            } else {
                Command::new("umount")
            };

            let output = cmd
                .arg(&mountpoint)
                .output()
                .map_err(|e| Error::CommandFailed(format!("Failed to execute umount: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::CommandFailed(format!("umount failed: {}", stderr)));
            }

            if self.trace {
                eprintln!("guestfs: successfully unmounted {}", mountpoint);
            }

            // Remove from tracking
            self.mounted.remove(&dev);
        }

        Ok(())
    }

    /// Unmount all filesystems
    ///
    pub fn umount_all(&mut self) -> Result<()> {
        // Don't check ensure_ready() - we need to unmount even during shutdown
        if self.trace {
            eprintln!("guestfs: umount_all");
        }

        // If no mounts, nothing to do
        if self.mounted.is_empty() {
            return Ok(());
        }

        // Check if we need sudo
        let need_sudo = unsafe { libc::geteuid() } != 0;

        // Unmount all in reverse order (to handle nested mounts)
        let mountpoints: Vec<String> = self.mounted.values().cloned().collect();

        for mountpoint in mountpoints.iter().rev() {
            if self.trace {
                eprintln!("guestfs: unmounting {}", mountpoint);
            }

            // Always check what's using the mountpoint before unmounting
            // This helps diagnose unmount failures
            let lsof_output = Command::new("lsof")
                .arg(mountpoint)
                .output();
            let mut has_users = false;
            if let Ok(out) = lsof_output {
                if !out.stdout.is_empty() {
                    has_users = true;
                    if self.debug {
                        eprintln!("guestfs: processes using {}:\n{}", mountpoint, String::from_utf8_lossy(&out.stdout));
                    }
                }
            }

            // Try recursive unmount first to handle stacked mounts from previous lazy unmounts
            let mut cmd = if need_sudo {
                let mut sudo_cmd = Command::new("sudo");
                sudo_cmd.arg("umount");
                sudo_cmd
            } else {
                Command::new("umount")
            };

            let output = cmd
                .arg("-R")  // Recursive unmount to handle stacked mounts from previous runs
                .arg(mountpoint)
                .output()
                .map_err(|e| Error::CommandFailed(format!("Failed to execute umount: {}", e)))?;

            if self.debug {
                eprintln!("[DEBUG] umount {} exited with status: {}, stderr: {}",
                    mountpoint,
                    output.status,
                    String::from_utf8_lossy(&output.stderr));
            }

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Warning: umount {} failed: {}", mountpoint, stderr);

                // If we detected processes using the mount, show a helpful message
                if has_users {
                    eprintln!("Note: The mount point has active users. Use 'lsof {}' to see what's using it.", mountpoint);
                }

                // Try force unmount
                if self.trace {
                    eprintln!("guestfs: trying force unmount for {}", mountpoint);
                }

                let mut force_cmd = if need_sudo {
                    let mut sudo_cmd = Command::new("sudo");
                    sudo_cmd.arg("umount");
                    sudo_cmd
                } else {
                    Command::new("umount")
                };

                let force_output = force_cmd
                    .arg("-f")
                    .arg(mountpoint)
                    .output();

                match force_output {
                    Ok(out) if out.status.success() => {
                        if self.trace {
                            eprintln!("guestfs: force unmount succeeded for {}", mountpoint);
                        }
                    }
                    Ok(out) => {
                        eprintln!("Warning: force umount also failed: {}", String::from_utf8_lossy(&out.stderr));

                        // Last resort: lazy unmount
                        if self.trace {
                            eprintln!("guestfs: trying lazy unmount for {}", mountpoint);
                        }

                        let mut lazy_cmd = if need_sudo {
                            let mut sudo_cmd = Command::new("sudo");
                            sudo_cmd.arg("umount");
                            sudo_cmd
                        } else {
                            Command::new("umount")
                        };

                        if let Ok(lazy_out) = lazy_cmd.arg("-l").arg(mountpoint).output() {
                            if lazy_out.status.success() {
                                eprintln!("Note: Used lazy unmount for {}. Filesystem is detached but may still be active in kernel.", mountpoint);
                                if self.trace {
                                    eprintln!("guestfs: lazy unmount succeeded for {}", mountpoint);
                                }
                                // Mark that we used lazy unmount - directory cleanup should be skipped
                                self.lazy_unmount_used = true;
                            } else {
                                eprintln!("Warning: lazy unmount also failed for {}: {}",
                                    mountpoint, String::from_utf8_lossy(&lazy_out.stderr));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: failed to execute force umount: {}", e);
                    }
                }
            } else if self.trace {
                eprintln!("guestfs: successfully unmounted {}", mountpoint);
            }
        }

        self.mounted.clear();

        // Sync filesystem to ensure all unmounts are complete
        if let Err(e) = std::process::Command::new("sync").output() {
            eprintln!("Warning: sync command failed: {}", e);
        }

        // Brief wait to ensure all filesystem operations are complete
        std::thread::sleep(std::time::Duration::from_millis(200));

        Ok(())
    }

    /// Get list of mounted filesystems
    ///
    pub fn mounts(&self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        Ok(self.mounted.keys().cloned().collect())
    }

    /// Get mountpoints
    ///
    pub fn mountpoints(&self) -> Result<HashMap<String, String>> {
        self.ensure_ready()?;

        // Return device -> mountpoint mapping
        Ok(self.mounted.clone())
    }

    /// Create a mountpoint
    ///
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
