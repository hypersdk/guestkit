// SPDX-License-Identifier: Apache-2.0
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

/// Check if we need sudo (i.e., not running as root).
///
/// # Safety
/// `geteuid()` is a read-only syscall with no side effects; always safe to call.
pub(crate) fn need_sudo() -> bool {
    unsafe { libc::geteuid() != 0 }
}

impl Guestfs {
    /// Resolve the device path, create mount root and mountpoint directory.
    /// Shared setup for mount_ro(), mount(), and mount_options().
    fn prepare_mount(
        &mut self,
        mountable: &str,
        mountpoint: &str,
    ) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
        self.ensure_ready()?;

        // Check if this device is already mounted - prevent duplicate mounts
        if self.mounted.contains_key(mountable) {
            return Err(Error::InvalidState("already_mounted".to_string()));
        }

        // Determine the actual device path to mount
        let device_partition = if mountable.starts_with("/dev/mapper/")
            || (mountable.starts_with("/dev/") && mountable.matches('/').count() >= 3)
        {
            // LVM logical volume - use the path directly
            std::path::PathBuf::from(mountable)
        } else {
            let partition_num = self.parse_device_name(mountable)?;

            if let Some(loop_dev) = &self.loop_device {
                if partition_num > 0 {
                    loop_dev.partition_path(partition_num).ok_or_else(|| {
                        Error::InvalidState("Loop device not connected".to_string())
                    })?
                } else {
                    loop_dev
                        .device_path()
                        .ok_or_else(|| {
                            Error::InvalidState("Loop device not connected".to_string())
                        })?
                        .to_path_buf()
                }
            } else if let Some(nbd) = &self.nbd_device {
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
            let mount_dir =
                std::path::PathBuf::from("/run").join(format!("guestkit-{}", std::process::id()));
            fs::create_dir_all(&mount_dir)
                .map_err(|e| Error::CommandFailed(format!("Failed to create mount root: {}", e)))?;
            self.mount_root.get_or_insert(mount_dir);
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

        Ok((device_partition, actual_mountpoint))
    }

    /// Record a successful mount in the mounted map.
    fn record_mount(&mut self, mountable: &str, actual_mountpoint: &std::path::Path) {
        self.mounted.insert(
            mountable.to_string(),
            actual_mountpoint.to_string_lossy().to_string(),
        );
    }

    /// Mount a filesystem read-only
    ///
    /// # Arguments
    ///
    /// * `mountable` - Device name (e.g., "/dev/sda1")
    /// * `mountpoint` - Mount point path (e.g., "/")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestkit::guestfs::Guestfs;
    ///
    /// let mut g = Guestfs::new()?;
    /// g.add_drive_ro("/path/to/disk.qcow2")?;
    /// g.launch()?;
    /// g.mount_ro("/dev/sda1", "/")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn mount_ro(&mut self, mountable: &str, mountpoint: &str) -> Result<()> {
        if self.verbose {
            eprintln!("guestfs: mount_ro {} {}", mountable, mountpoint);
        }

        let (device_partition, actual_mountpoint) = match self.prepare_mount(mountable, mountpoint)
        {
            Ok(v) => v,
            Err(e) if e.to_string().contains("already_mounted") => return Ok(()),
            Err(e) => return Err(e),
        };

        let need_sudo = need_sudo();

        // Detect filesystem type to use appropriate mount options
        // Use the original mountable parameter, as device_partition might not exist yet (LVM)
        let fs_type = self.vfs_type(mountable).unwrap_or_else(|e| {
            log::debug!(
                "Could not detect filesystem type for {}: {}. Falling back to 'auto'.",
                mountable,
                e
            );
            "auto".to_string()
        });

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
        // For XFS: use norecovery to skip log replay (which requires write access)
        // For btrfs and others: just use ro
        let mount_opts = if fs_type.starts_with("ext") {
            "ro,noload"
        } else if fs_type == "xfs" {
            // nouuid prevents failure when duplicate UUIDs exist (common with clones/LVM snapshots)
            "ro,norecovery,nouuid"
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
            // Retry with fallback options (inspired by vmcraft mount.py)
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            // For XFS: try with just ro,norecovery (without nouuid)
            // For ext: try ro without noload
            // For unknown fs_type: try ro,norecovery then ro,norecovery,nouuid
            let fallback_opts: &[&str] = if fs_type == "xfs" {
                &["ro,norecovery"]
            } else if fs_type.starts_with("ext") {
                &["ro"]
            } else {
                &["ro,norecovery", "ro,norecovery,nouuid", "ro,noload"]
            };

            let mut mounted = false;
            for opts in fallback_opts {
                let mut retry_cmd = if need_sudo {
                    let mut c = Command::new("sudo");
                    c.arg("mount");
                    c
                } else {
                    Command::new("mount")
                };

                let retry_out = retry_cmd
                    .arg("-o")
                    .arg(opts)
                    .arg(&device_partition)
                    .arg(&actual_mountpoint)
                    .output();

                if let Ok(out) = retry_out {
                    if out.status.success() {
                        if self.verbose {
                            eprintln!(
                                "guestfs: mount_ro {} succeeded with fallback options: {}",
                                mountable, opts
                            );
                        }
                        mounted = true;
                        break;
                    }
                }
            }

            if !mounted {
                return Err(Error::CommandFailed(format!(
                    "Mount failed: {}. You may need sudo/root permissions.",
                    stderr
                )));
            }
        }

        self.record_mount(mountable, &actual_mountpoint);

        Ok(())
    }

    /// Mount a filesystem read-write
    ///
    pub fn mount(&mut self, mountable: &str, mountpoint: &str) -> Result<()> {
        if self.verbose {
            eprintln!("guestfs: mount {} {}", mountable, mountpoint);
        }

        // Check if readonly
        if let Some(drive) = self.drives.first() {
            if drive.readonly {
                return Err(Error::PermissionDenied(
                    "Cannot mount read-write on read-only drive".to_string(),
                ));
            }
        }

        let (device_partition, actual_mountpoint) = match self.prepare_mount(mountable, mountpoint)
        {
            Ok(v) => v,
            Err(e) if e.to_string().contains("already_mounted") => return Ok(()),
            Err(e) => return Err(e),
        };

        let need_sudo = need_sudo();

        // Build mount command (read-write, no "ro" flag)
        let mut cmd = if need_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("mount");
            sudo_cmd
        } else {
            Command::new("mount")
        };

        let output = cmd
            .arg(&device_partition)
            .arg(&actual_mountpoint)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mount: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::CommandFailed(format!(
                "Mount failed: {}. You may need sudo/root permissions.",
                stderr
            )));
        }

        self.record_mount(mountable, &actual_mountpoint);

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

        // Validate mount options: reject dangerous options
        {
            let options_lower = options.to_lowercase();
            let dangerous = ["suid", "dev", "exec"];
            for opt in options_lower.split(',') {
                let opt = opt.trim();
                for d in &dangerous {
                    if opt == *d {
                        return Err(Error::SecurityViolation(format!(
                            "Mount option '{}' is not allowed for security reasons",
                            opt
                        )));
                    }
                }
            }
        }

        // Check if readonly drive and options don't include "ro"
        if let Some(drive) = self.drives.first() {
            if drive.readonly && !options.contains("ro") {
                return Err(Error::PermissionDenied(
                    "Cannot mount read-write on read-only drive".to_string(),
                ));
            }
        }

        let (device_partition, actual_mountpoint) = match self.prepare_mount(mountable, mountpoint)
        {
            Ok(v) => v,
            Err(e) if e.to_string().contains("already_mounted") => return Ok(()),
            Err(e) => return Err(e),
        };

        let need_sudo = need_sudo();

        let mut cmd = if need_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("mount");
            sudo_cmd
        } else {
            Command::new("mount")
        };

        if !options.is_empty() {
            cmd.arg("-o").arg(options);
        }

        let output = cmd
            .arg(&device_partition)
            .arg(&actual_mountpoint)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mount: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::CommandFailed(format!(
                "Mount failed: {}. You may need sudo/root permissions.",
                stderr
            )));
        }

        self.record_mount(mountable, &actual_mountpoint);

        Ok(())
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
        self.ensure_ready()?;

        if self.verbose {
            eprintln!(
                "guestfs: mount_vfs {} {} {} {}",
                options, vfstype, mountable, mountpoint
            );
        }

        // Validate VFS type against whitelist of known filesystem types
        {
            const ALLOWED_VFS_TYPES: &[&str] = &[
                "proc",
                "sysfs",
                "devtmpfs",
                "tmpfs",
                "devpts",
                "cgroup",
                "cgroup2",
                "securityfs",
                "debugfs",
                "tracefs",
                "configfs",
                "fusectl",
                "pstore",
                "efivarfs",
                "bpf",
                "hugetlbfs",
                "mqueue",
                "overlay",
            ];
            if !vfstype.is_empty() && !ALLOWED_VFS_TYPES.contains(&vfstype) {
                return Err(Error::SecurityViolation(format!(
                    "VFS type '{}' is not in the allowed list: {}",
                    vfstype,
                    ALLOWED_VFS_TYPES.join(", ")
                )));
            }
        }

        // Check if readonly drive and options don't include "ro"
        if let Some(drive) = self.drives.first() {
            if drive.readonly && !options.contains("ro") {
                return Err(Error::PermissionDenied(
                    "Cannot mount read-write on read-only drive".to_string(),
                ));
            }
        }

        // Check if this device is already mounted
        if self.mounted.contains_key(mountable) {
            return Ok(());
        }

        // Determine the actual device path to mount
        let device_partition = if mountable.starts_with("/dev/mapper/")
            || (mountable.starts_with("/dev/") && mountable.matches('/').count() >= 3)
        {
            std::path::PathBuf::from(mountable)
        } else {
            let partition_num = self.parse_device_name(mountable)?;

            if let Some(loop_dev) = &self.loop_device {
                if partition_num > 0 {
                    loop_dev.partition_path(partition_num).ok_or_else(|| {
                        Error::InvalidState("Loop device not connected".to_string())
                    })?
                } else {
                    loop_dev
                        .device_path()
                        .ok_or_else(|| {
                            Error::InvalidState("Loop device not connected".to_string())
                        })?
                        .to_path_buf()
                }
            } else if let Some(nbd) = &self.nbd_device {
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
            let mount_dir =
                std::path::PathBuf::from("/run").join(format!("guestkit-{}", std::process::id()));
            fs::create_dir_all(&mount_dir)
                .map_err(|e| Error::CommandFailed(format!("Failed to create mount root: {}", e)))?;
            self.mount_root.get_or_insert(mount_dir);
        }

        let mount_root = self
            .mount_root
            .as_ref()
            .ok_or_else(|| Error::InvalidState("No mount root created".to_string()))?;
        let actual_mountpoint = if mountpoint == "/" {
            mount_root.clone()
        } else {
            mount_root.join(mountpoint.trim_start_matches('/'))
        };

        fs::create_dir_all(&actual_mountpoint)
            .map_err(|e| Error::CommandFailed(format!("Failed to create mountpoint: {}", e)))?;

        let need_sudo = need_sudo();

        let mut cmd = if need_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("mount");
            sudo_cmd
        } else {
            Command::new("mount")
        };

        if !options.is_empty() {
            cmd.arg("-o").arg(options);
        }
        if !vfstype.is_empty() {
            cmd.arg("-t").arg(vfstype);
        }

        let output = cmd
            .arg(&device_partition)
            .arg(&actual_mountpoint)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute mount: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::CommandFailed(format!(
                "Mount failed: {}. You may need sudo/root permissions.",
                stderr
            )));
        }

        self.record_mount(mountable, &actual_mountpoint);

        Ok(())
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
        let need_sudo = need_sudo();

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
        let need_sudo = need_sudo();

        // Unmount deepest mounts first (sorted by path component count, descending)
        let mut mountpoints: Vec<String> = self.mounted.values().cloned().collect();
        mountpoints.sort_by(|a, b| {
            let depth_a = a.matches('/').count();
            let depth_b = b.matches('/').count();
            depth_b.cmp(&depth_a)
        });

        for mountpoint in &mountpoints {
            if self.trace {
                eprintln!("guestfs: unmounting {}", mountpoint);
            }

            // Always check what's using the mountpoint before unmounting
            // This helps diagnose unmount failures
            let lsof_output = Command::new("lsof").arg(mountpoint).output();
            let mut has_users = false;
            if let Ok(out) = lsof_output {
                if !out.stdout.is_empty() {
                    has_users = true;
                    if self.debug {
                        eprintln!(
                            "guestfs: processes using {}:\n{}",
                            mountpoint,
                            String::from_utf8_lossy(&out.stdout)
                        );
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
                .arg("-R") // Recursive unmount to handle stacked mounts from previous runs
                .arg(mountpoint)
                .output()
                .map_err(|e| Error::CommandFailed(format!("Failed to execute umount: {}", e)))?;

            if self.debug {
                eprintln!(
                    "[DEBUG] umount {} exited with status: {}, stderr: {}",
                    mountpoint,
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
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

                let force_output = force_cmd.arg("-f").arg(mountpoint).output();

                match force_output {
                    Ok(out) if out.status.success() => {
                        if self.trace {
                            eprintln!("guestfs: force unmount succeeded for {}", mountpoint);
                        }
                    }
                    Ok(out) => {
                        eprintln!(
                            "Warning: force umount also failed: {}",
                            String::from_utf8_lossy(&out.stderr)
                        );

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
                                eprintln!(
                                    "Warning: lazy unmount also failed for {}: {}",
                                    mountpoint,
                                    String::from_utf8_lossy(&lazy_out.stderr)
                                );
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
    pub fn mountpoints(&self) -> Result<&HashMap<String, String>> {
        self.ensure_ready()?;

        // Return device -> mountpoint mapping
        Ok(&self.mounted)
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
        let _g = Guestfs::new().unwrap();
        // Setup would be needed here
    }
}
