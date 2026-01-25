// SPDX-License-Identifier: LGPL-3.0-or-later
//! Loop device support for mounting disk images
//!
//! This module provides loop device management for mounting disk images
//! as block devices. Loop devices are built into the Linux kernel and
//! don't require external modules like NBD.

use crate::core::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Loop device manager
pub struct LoopDevice {
    /// Loop device path (e.g., /dev/loop0)
    device_path: Option<PathBuf>,
    /// Image path being mounted
    image_path: PathBuf,
    /// Whether device is connected
    connected: bool,
}

impl LoopDevice {
    /// Create a new loop device manager
    pub fn new() -> Result<Self> {
        Ok(LoopDevice {
            device_path: None,
            image_path: PathBuf::new(),
            connected: false,
        })
    }

    /// Connect disk image to loop device
    ///
    /// # Arguments
    ///
    /// * `image_path` - Path to disk image (raw format only)
    /// * `read_only` - Whether to connect in read-only mode
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestctl::disk::loop_device::LoopDevice;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut loop_dev = LoopDevice::new()?;
    /// loop_dev.connect("/path/to/disk.raw", true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect<P: AsRef<Path>>(&mut self, image_path: P, read_only: bool) -> Result<()> {
        if self.connected {
            return Err(Error::InvalidState(
                "Loop device already connected".to_string(),
            ));
        }

        let image_path = image_path.as_ref();
        if !image_path.exists() {
            return Err(Error::NotFound(format!(
                "Image file does not exist: {}",
                image_path.display()
            )));
        }

        // Check if we need to use sudo
        let need_sudo = unsafe { libc::geteuid() } != 0;

        // Build losetup command
        let mut cmd = if need_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("losetup");
            sudo_cmd
        } else {
            Command::new("losetup")
        };

        // Find free loop device and set it up with --partscan
        cmd.arg("-f") // Find first unused loop device
            .arg("--show") // Show device name
            .arg("--partscan"); // Scan for partitions

        if read_only {
            cmd.arg("--read-only");
        }

        cmd.arg(image_path);

        // Execute and capture the loop device path
        let output = cmd.output().map_err(|e| {
            Error::CommandFailed(format!(
                "Failed to execute losetup: {}. Is util-linux installed?",
                e
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!(
                "losetup failed: {}. You may need sudo privileges.",
                stderr
            )));
        }

        // Parse the device path from stdout
        let device_str = String::from_utf8_lossy(&output.stdout);
        let device_path = PathBuf::from(device_str.trim());

        if !device_path.exists() {
            return Err(Error::CommandFailed(format!(
                "Loop device was not created: {}",
                device_path.display()
            )));
        }

        self.device_path = Some(device_path);
        self.image_path = image_path.to_path_buf();
        self.connected = true;

        // Wait for device to be ready
        self.wait_for_device()?;

        Ok(())
    }

    /// Wait for loop device to become available
    fn wait_for_device(&self) -> Result<()> {
        let device_path = self.device_path.as_ref().ok_or_else(|| {
            Error::InvalidState("No device path set".to_string())
        })?;

        for _i in 0..50 {
            // Check if device is readable and has non-zero size
            if let Ok(file) = std::fs::File::open(device_path) {
                use std::os::unix::io::AsRawFd;
                const BLKGETSIZE64: libc::c_ulong = 0x80081272;

                let mut size_bytes: u64 = 0;
                let result = unsafe {
                    libc::ioctl(
                        file.as_raw_fd(),
                        BLKGETSIZE64 as _,
                        &mut size_bytes as *mut u64,
                    )
                };

                if result == 0 && size_bytes > 0 {
                    return Ok(());
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        Err(Error::CommandFailed(format!(
            "Loop device {} did not become ready in time",
            device_path.display()
        )))
    }

    /// Disconnect loop device
    pub fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        let device_path = self.device_path.as_ref().ok_or_else(|| {
            Error::InvalidState("No device path to disconnect".to_string())
        })?;

        // Check if we need to use sudo
        let need_sudo = unsafe { libc::geteuid() } != 0;

        // Disconnect using losetup -d
        let mut cmd = if need_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("losetup");
            sudo_cmd
        } else {
            Command::new("losetup")
        };

        let output = cmd
            .arg("-d")
            .arg(device_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to disconnect loop device: {}", e)))?;

        if !output.status.success() {
            eprintln!(
                "Warning: losetup disconnect failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        self.connected = false;
        self.device_path = None;

        Ok(())
    }

    /// Get loop device path
    pub fn device_path(&self) -> Option<&Path> {
        self.device_path.as_deref()
    }

    /// Get partition device path
    ///
    /// # Arguments
    ///
    /// * `partition_num` - Partition number (1-based)
    pub fn partition_path(&self, partition_num: u32) -> Option<PathBuf> {
        self.device_path.as_ref().map(|dev| {
            PathBuf::from(format!("{}p{}", dev.display(), partition_num))
        })
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// List partitions on loop device
    pub fn list_partitions(&self) -> Result<Vec<PathBuf>> {
        if !self.connected {
            return Err(Error::InvalidState("Loop device not connected".to_string()));
        }

        let device_path = self.device_path.as_ref().ok_or_else(|| {
            Error::InvalidState("No device path set".to_string())
        })?;

        let mut partitions = Vec::new();

        // Check for partitions (p1, p2, etc.)
        for i in 1..=16 {
            let part_path = PathBuf::from(format!("{}p{}", device_path.display(), i));
            if part_path.exists() {
                partitions.push(part_path);
            } else {
                break;
            }
        }

        // If no partitions found, return the main device
        if partitions.is_empty() {
            partitions.push(device_path.clone());
        }

        Ok(partitions)
    }

    /// Check if image format is supported by loop devices
    ///
    /// Loop devices work best with RAW images. Other formats need conversion.
    pub fn is_format_supported(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(), "raw" | "img" | "iso")
        } else {
            // No extension, assume raw
            true
        }
    }
}

impl Drop for LoopDevice {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_device_creation() {
        let result = LoopDevice::new();
        assert!(result.is_ok());

        let loop_dev = result.unwrap();
        assert!(!loop_dev.is_connected());
    }

    #[test]
    fn test_format_detection() {
        assert!(LoopDevice::is_format_supported(Path::new("disk.raw")));
        assert!(LoopDevice::is_format_supported(Path::new("disk.img")));
        assert!(LoopDevice::is_format_supported(Path::new("disk.iso")));
        assert!(!LoopDevice::is_format_supported(Path::new("disk.qcow2")));
        assert!(!LoopDevice::is_format_supported(Path::new("disk.vmdk")));
    }
}
