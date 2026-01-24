// SPDX-License-Identifier: LGPL-3.0-or-later
//! NBD (Network Block Device) support using qemu-nbd
//!
//! This module provides NBD device management for mounting disk images
//! as block devices. This allows filesystem access without implementing
//! full filesystem parsers.

use crate::core::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

/// NBD device manager
pub struct NbdDevice {
    /// NBD device path (e.g., /dev/nbd0)
    device_path: PathBuf,
    /// Image path being exported
    image_path: PathBuf,
    /// Whether device is connected
    connected: bool,
}

impl NbdDevice {
    /// Create a new NBD device manager
    ///
    /// This finds an available /dev/nbd* device
    pub fn new() -> Result<Self> {
        let device_path = Self::find_available_device()?;

        Ok(NbdDevice {
            device_path,
            image_path: PathBuf::new(),
            connected: false,
        })
    }

    /// Find an available NBD device
    fn find_available_device() -> Result<PathBuf> {
        // Try /dev/nbd0 through /dev/nbd15
        for i in 0..16 {
            let device = PathBuf::from(format!("/dev/nbd{}", i));
            if device.exists() {
                // Check if device is in use by checking its size
                let device_str = device.to_str().ok_or_else(|| {
                    Error::InvalidFormat(format!(
                        "Device path contains invalid Unicode: {:?}",
                        device
                    ))
                })?;

                if let Ok(output) = Command::new("lsblk")
                    .arg("-b") // Show sizes in bytes
                    .arg("-n") // No headings
                    .arg("-o")
                    .arg("SIZE")
                    .arg(device_str)
                    .output()
                {
                    // If size is 0, device is not connected
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Ok(size) = stdout.trim().parse::<u64>() {
                        if size == 0 {
                            return Ok(device);
                        }
                    }
                }
            }
        }

        Err(Error::NotFound(
            "No available NBD devices found. Load nbd kernel module with: sudo modprobe nbd max_part=8".to_string()
        ))
    }

    /// Connect disk image to NBD device
    ///
    /// # Arguments
    ///
    /// * `image_path` - Path to disk image (qcow2, raw, vmdk, etc.)
    /// * `read_only` - Whether to connect in read-only mode
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestkit::disk::nbd::NbdDevice;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut nbd = NbdDevice::new()?;
    /// nbd.connect("/path/to/disk.qcow2", true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect<P: AsRef<Path>>(&mut self, image_path: P, read_only: bool) -> Result<()> {
        if self.connected {
            return Err(Error::InvalidState(
                "NBD device already connected".to_string(),
            ));
        }

        let image_path = image_path.as_ref();
        if !image_path.exists() {
            return Err(Error::NotFound(format!(
                "Image file does not exist: {}",
                image_path.display()
            )));
        }

        // Check if we need to use sudo (qemu-nbd --connect requires root)
        let need_sudo = unsafe { libc::geteuid() } != 0;

        // Build qemu-nbd command
        let mut cmd = if need_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("qemu-nbd");
            sudo_cmd
        } else {
            Command::new("qemu-nbd")
        };

        cmd.arg("--connect").arg(&self.device_path).arg(image_path);

        if read_only {
            cmd.arg("--read-only");
        }

        // Detect image format from extension
        let format = image_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "qcow2" => "qcow2",
                "vmdk" => "vmdk",
                "vdi" => "vdi",
                "vhd" | "vpc" => "vpc",
                _ => "raw",
            })
            .unwrap_or("raw");

        cmd.arg("--format").arg(format);

        // Capture output for debugging
        let output = cmd.output().map_err(|e| {
            Error::CommandFailed(format!(
                "Failed to execute qemu-nbd: {}. Is qemu-nbd installed?",
                e
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(Error::CommandFailed(format!(
                "qemu-nbd failed: stdout='{}', stderr='{}'. You may need sudo privileges.",
                stdout, stderr
            )));
        }

        self.image_path = image_path.to_path_buf();
        self.connected = true;

        // Wait for device to be ready
        self.wait_for_device()?;

        Ok(())
    }

    /// Wait for NBD device to become available
    fn wait_for_device(&self) -> Result<()> {
        for _i in 0..50 {
            // Try to read from device
            if std::fs::metadata(&self.device_path).is_ok() {
                // Check if device is actually readable and has non-zero size
                if let Ok(file) = std::fs::File::open(&self.device_path) {
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
            }
            thread::sleep(Duration::from_millis(200));
        }

        Err(Error::CommandFailed(format!(
            "NBD device {} did not become ready in time (no data or zero size)",
            self.device_path.display()
        )))
    }

    /// Disconnect NBD device
    pub fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        // Check if we need to use sudo
        let need_sudo = unsafe { libc::geteuid() } != 0;

        // Disconnect using qemu-nbd
        let mut cmd = if need_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg("qemu-nbd");
            sudo_cmd
        } else {
            Command::new("qemu-nbd")
        };

        let output = cmd
            .arg("--disconnect")
            .arg(&self.device_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to disconnect NBD: {}", e)))?;

        if !output.status.success() {
            eprintln!(
                "Warning: qemu-nbd disconnect failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        self.connected = false;

        Ok(())
    }

    /// Get NBD device path
    pub fn device_path(&self) -> &Path {
        &self.device_path
    }

    /// Get partition device path
    ///
    /// # Arguments
    ///
    /// * `partition_num` - Partition number (1-based)
    pub fn partition_path(&self, partition_num: u32) -> PathBuf {
        PathBuf::from(format!("{}p{}", self.device_path.display(), partition_num))
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// List partitions on NBD device
    pub fn list_partitions(&self) -> Result<Vec<PathBuf>> {
        if !self.connected {
            return Err(Error::InvalidState("NBD device not connected".to_string()));
        }

        let mut partitions = Vec::new();

        // Check for partitions (p1, p2, etc.)
        for i in 1..=16 {
            let part_path = self.partition_path(i);
            if part_path.exists() {
                partitions.push(part_path);
            } else {
                break;
            }
        }

        // If no partitions, return the main device
        if partitions.is_empty() {
            partitions.push(self.device_path.clone());
        }

        Ok(partitions)
    }
}

impl Drop for NbdDevice {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nbd_device_creation() {
        // Just test that we can create the struct
        // Actual connection requires root and qemu-nbd
        let result = NbdDevice::new();

        // May fail if no NBD devices available, which is fine
        match result {
            Ok(nbd) => {
                assert!(!nbd.is_connected());
            }
            Err(e) => {
                // Expected if NBD module not loaded
                eprintln!("NBD device creation failed (expected): {}", e);
            }
        }
    }
}
