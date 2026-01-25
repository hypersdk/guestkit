// SPDX-License-Identifier: LGPL-3.0-or-later
//! Device and filesystem operations for disk image manipulation

use crate::core::{Error, Result};
use crate::disk::FileSystem;
use crate::guestfs::Guestfs;
use std::collections::HashMap;

impl Guestfs {
    /// List all block devices
    ///
    pub fn list_devices(&self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        // Return list of drives added
        let mut devices = Vec::new();
        for (i, _) in self.drives.iter().enumerate() {
            devices.push(format!("/dev/sd{}", (b'a' + i as u8) as char));
        }

        Ok(devices)
    }

    /// List all partitions
    ///
    pub fn list_partitions(&self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        let partition_table = self.partition_table()?;
        let mut partitions = Vec::new();

        for partition in partition_table.partitions() {
            partitions.push(format!("/dev/sda{}", partition.number));
        }

        Ok(partitions)
    }

    /// List all filesystems detected
    ///
    pub fn list_filesystems(&mut self) -> Result<HashMap<String, String>> {
        self.ensure_ready()?;

        let mut filesystems = HashMap::new();

        // Clone partition data to avoid borrow checker issues
        let partitions: Vec<_> = {
            let partition_table = self.partition_table()?;
            partition_table.partitions().to_vec()
        };

        for partition in &partitions {
            let device_name = format!("/dev/sda{}", partition.number);

            let reader = self
                .reader
                .as_mut()
                .ok_or_else(|| Error::InvalidState("Reader not initialized".to_string()))?;
            if let Ok(fs) = FileSystem::detect(reader, partition) {
                let fs_type = match fs.fs_type() {
                    crate::disk::FileSystemType::Ext => "ext4",
                    crate::disk::FileSystemType::Ntfs => "ntfs",
                    crate::disk::FileSystemType::Fat32 => "vfat",
                    crate::disk::FileSystemType::Xfs => "xfs",
                    crate::disk::FileSystemType::Btrfs => "btrfs",
                    crate::disk::FileSystemType::Unknown => "unknown",
                };

                filesystems.insert(device_name, fs_type.to_string());
            }
        }

        Ok(filesystems)
    }

    /// Get filesystem type
    ///
    pub fn vfs_type(&mut self, device: &str) -> Result<String> {
        self.ensure_ready()?;

        let partition_num = self.parse_device_name(device)?;

        // Clone partition to avoid borrow checker issues
        let partition = {
            let partition_table = self.partition_table()?;
            partition_table
                .partitions()
                .iter()
                .find(|p| p.number == partition_num)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Partition {} not found", partition_num)))?
        };

        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| Error::InvalidState("Reader not initialized".to_string()))?;
        let fs = FileSystem::detect(reader, &partition)?;

        let fs_type = match fs.fs_type() {
            crate::disk::FileSystemType::Ext => "ext4",
            crate::disk::FileSystemType::Ntfs => "ntfs",
            crate::disk::FileSystemType::Fat32 => "vfat",
            crate::disk::FileSystemType::Xfs => "xfs",
            crate::disk::FileSystemType::Btrfs => "btrfs",
            crate::disk::FileSystemType::Unknown => "unknown",
        };

        Ok(fs_type.to_string())
    }

    /// Get filesystem label
    ///
    pub fn vfs_label(&mut self, device: &str) -> Result<String> {
        self.ensure_ready()?;

        let partition_num = self.parse_device_name(device)?;

        // Clone partition to avoid borrow checker issues
        let partition = {
            let partition_table = self.partition_table()?;
            partition_table
                .partitions()
                .iter()
                .find(|p| p.number == partition_num)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Partition {} not found", partition_num)))?
        };

        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| Error::InvalidState("Reader not initialized".to_string()))?;
        let fs = FileSystem::detect(reader, &partition)?;

        fs.label()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::NotFound("No label".to_string()))
    }

    /// Get filesystem UUID
    ///
    pub fn vfs_uuid(&mut self, device: &str) -> Result<String> {
        self.ensure_ready()?;

        let partition_num = self.parse_device_name(device)?;

        // Clone partition to avoid borrow checker issues
        let partition = {
            let partition_table = self.partition_table()?;
            partition_table
                .partitions()
                .iter()
                .find(|p| p.number == partition_num)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Partition {} not found", partition_num)))?
        };

        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| Error::InvalidState("Reader not initialized".to_string()))?;
        let fs = FileSystem::detect(reader, &partition)?;

        fs.uuid()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::NotFound("No UUID".to_string()))
    }

    /// Get block device size in bytes
    ///
    pub fn blockdev_getsize64(&self, device: &str) -> Result<i64> {
        self.ensure_ready()?;

        let partition_num = self.parse_device_name(device)?;

        if partition_num == 0 {
            // Whole device
            let reader = self
                .reader
                .as_ref()
                .ok_or_else(|| Error::InvalidState("Not launched".to_string()))?;
            Ok(reader.size() as i64)
        } else {
            // Partition - calculate from partition table
            let partition_table = self.partition_table()?;

            let partition = partition_table
                .partitions()
                .iter()
                .find(|p| p.number == partition_num)
                .ok_or_else(|| Error::NotFound(format!("Partition {} not found", partition_num)))?;

            Ok((partition.size_sectors * 512) as i64)
        }
    }

    /// Get block device size in 512-byte sectors
    ///
    pub fn blockdev_getsz(&self, device: &str) -> Result<i64> {
        Ok(self.blockdev_getsize64(device)? / 512)
    }

    /// Get canonical device name
    ///
    pub fn canonical_device_name(&self, device: &str) -> Result<String> {
        // Normalize device names
        let device = device.trim_start_matches("/dev/");

        // Convert variations to canonical form
        if device.starts_with("hd") {
            Ok(format!("/dev/sd{}", &device[2..]))
        } else if device.starts_with("vd") {
            Ok(format!("/dev/sd{}", &device[2..]))
        } else {
            Ok(format!("/dev/{}", device))
        }
    }

    /// Get device index
    ///
    pub fn device_index(&self, device: &str) -> Result<i32> {
        let canonical = self.canonical_device_name(device)?;

        // Extract the drive letter
        if let Some(rest) = canonical.strip_prefix("/dev/sd") {
            if let Some(letter) = rest.chars().next() {
                return Ok((letter as u8 - b'a') as i32);
            }
        }

        Err(Error::InvalidFormat(format!(
            "Cannot parse device: {}",
            device
        )))
    }

    /// Check if device name refers to whole device (not partition)
    ///
    pub fn is_whole_device(&self, device: &str) -> Result<bool> {
        // Whole devices end with just a letter (e.g., /dev/sda)
        // Partitions have numbers (e.g., /dev/sda1)
        let canonical = self.canonical_device_name(device)?;

        Ok(!canonical
            .chars()
            .last()
            .map(|c| c.is_numeric())
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_device_name() {
        let g = Guestfs::new().unwrap();

        assert_eq!(g.canonical_device_name("/dev/sda").unwrap(), "/dev/sda");
        assert_eq!(g.canonical_device_name("/dev/hda").unwrap(), "/dev/sda");
        assert_eq!(g.canonical_device_name("/dev/vda").unwrap(), "/dev/sda");
        assert_eq!(g.canonical_device_name("sda").unwrap(), "/dev/sda");
    }

    #[test]
    fn test_is_whole_device() {
        let g = Guestfs::new().unwrap();

        assert_eq!(g.is_whole_device("/dev/sda").unwrap(), true);
        assert_eq!(g.is_whole_device("/dev/sda1").unwrap(), false);
        assert_eq!(g.is_whole_device("/dev/sda10").unwrap(), false);
    }

    #[test]
    fn test_device_index() {
        let g = Guestfs::new().unwrap();

        assert_eq!(g.device_index("/dev/sda").unwrap(), 0);
        assert_eq!(g.device_index("/dev/sdb").unwrap(), 1);
        assert_eq!(g.device_index("/dev/vda").unwrap(), 0);
    }
}
