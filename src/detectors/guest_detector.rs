// SPDX-License-Identifier: LGPL-3.0-or-later
//! Guest OS detection using pure Rust
//!
//! This module implements guest OS detection without external dependencies

use crate::core::{Firmware, GuestIdentity, GuestType, Result};
use crate::disk::{DiskReader, FileSystem, PartitionTable};
use std::path::Path;

/// Guest OS detector
pub struct GuestDetector {}

impl GuestDetector {
    /// Create a new guest detector
    pub fn new() -> Self {
        Self {}
    }

    /// Detect guest OS from disk image
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestctl::detectors::GuestDetector;
    /// use std::path::Path;
    ///
    /// let detector = GuestDetector::new();
    /// let guest = detector.detect_from_image(Path::new("/path/to/disk.qcow2")).unwrap();
    /// println!("OS: {}", guest.os_name);
    /// ```
    pub fn detect_from_image<P: AsRef<Path>>(&self, path: P) -> Result<GuestIdentity> {
        // Open disk image
        let mut reader = DiskReader::open(path.as_ref())?;

        // Parse partition table
        let partition_table = PartitionTable::parse(&mut reader)?;

        // Analyze partitions to detect OS
        let mut os_type = GuestType::Unknown;
        let mut os_name = String::from("Unknown");
        let mut os_version = String::from("Unknown");
        let architecture = String::from("x86_64");
        let mut firmware = Firmware::Bios;
        let mut distro = None;

        // Check for GPT (indicates UEFI)
        if matches!(
            partition_table.table_type(),
            crate::disk::PartitionType::GPT
        ) {
            firmware = Firmware::Uefi;
        }

        // Examine each partition
        for partition in partition_table.partitions() {
            if let Ok(fs) = FileSystem::detect(&mut reader, partition) {
                // Detect OS based on filesystem and partition analysis
                match fs.fs_type() {
                    crate::disk::FileSystemType::Ntfs => {
                        // Likely Windows
                        os_type = GuestType::Windows;
                        os_name = "Windows".to_string();
                        os_version = "Unknown".to_string();
                    }
                    crate::disk::FileSystemType::Ext
                    | crate::disk::FileSystemType::Xfs
                    | crate::disk::FileSystemType::Btrfs => {
                        // Likely Linux
                        os_type = GuestType::Linux;

                        // Try to detect specific distribution
                        if let Some(label) = fs.label() {
                            // Use filesystem label for hints
                            if label.contains("fedora") || label.contains("Fedora") {
                                os_name = "Fedora Linux".to_string();
                                distro = Some("fedora".to_string());
                            } else if label.contains("ubuntu") || label.contains("Ubuntu") {
                                os_name = "Ubuntu".to_string();
                                distro = Some("ubuntu".to_string());
                            } else if label.contains("debian") || label.contains("Debian") {
                                os_name = "Debian".to_string();
                                distro = Some("debian".to_string());
                            } else if label.contains("rhel") || label.contains("RHEL") {
                                os_name = "Red Hat Enterprise Linux".to_string();
                                distro = Some("rhel".to_string());
                            } else {
                                os_name = "Linux".to_string();
                            }
                        } else {
                            os_name = "Linux".to_string();
                        }

                        os_version = "Unknown".to_string();
                    }
                    crate::disk::FileSystemType::Fat32 => {
                        // Could be EFI partition or DOS
                        if partition.start_lba < 2048 && partition.size_sectors < 1024 * 1024 {
                            // Likely EFI system partition
                            firmware = Firmware::Uefi;
                        }
                    }
                    _ => {}
                }

                // If we found an OS, break
                if os_type != GuestType::Unknown {
                    break;
                }
            }
        }

        Ok(GuestIdentity {
            os_type,
            os_name,
            os_version,
            architecture,
            firmware,
            init_system: None,
            distro,
        })
    }
}

impl Default for GuestDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = GuestDetector::new();
        let _ = detector;
    }
}
