// SPDX-License-Identifier: LGPL-3.0-or-later
//! NBD (Network Block Device) support using qemu-nbd
//!
//! This module provides NBD device management for mounting disk images
//! as block devices. This allows filesystem access without implementing
//! full filesystem parsers.

use crate::core::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

/// NBD device manager
pub struct NbdDevice {
    /// NBD device path (e.g., /dev/nbd0)
    device_path: PathBuf,
    /// qemu-nbd process handle
    process: Option<Child>,
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
            process: None,
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
                // Check if device is in use
                if let Ok(output) = Command::new("lsblk")
                    .arg(device.to_str().unwrap())
                    .output()
                {
                    // If lsblk shows no partitions, device is free
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if !stdout.contains("part") && !stdout.contains("disk") {
                        return Ok(device);
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
            return Err(Error::InvalidState("NBD device already connected".to_string()));
        }

        let image_path = image_path.as_ref();
        if !image_path.exists() {
            return Err(Error::NotFound(format!(
                "Image file does not exist: {}",
                image_path.display()
            )));
        }

        // Build qemu-nbd command
        let mut cmd = Command::new("qemu-nbd");
        cmd.arg("--connect")
            .arg(&self.device_path)
            .arg(image_path);

        if read_only {
            cmd.arg("--read-only");
        }

        // Enable partition detection
        cmd.arg("--format")
            .arg("auto"); // Auto-detect format

        cmd.stdout(Stdio::null())
            .stderr(Stdio::null());

        // Execute qemu-nbd
        let child = cmd.spawn().map_err(|e| {
            Error::CommandFailed(format!(
                "Failed to start qemu-nbd: {}. Is qemu-nbd installed?",
                e
            ))
        })?;

        self.process = Some(child);
        self.image_path = image_path.to_path_buf();
        self.connected = true;

        // Wait for device to be ready
        self.wait_for_device()?;

        Ok(())
    }

    /// Wait for NBD device to become available
    fn wait_for_device(&self) -> Result<()> {
        for _ in 0..50 {
            // Try to read from device
            if let Ok(_) = std::fs::metadata(&self.device_path) {
                // Check if device is actually readable
                if let Ok(_) = std::fs::File::open(&self.device_path) {
                    return Ok(());
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        Err(Error::CommandFailed(format!(
            "NBD device {} did not become ready in time",
            self.device_path.display()
        )))
    }

    /// Disconnect NBD device
    pub fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        // Disconnect using qemu-nbd
        let output = Command::new("qemu-nbd")
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

        // Kill process if still running
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
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
