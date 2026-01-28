// SPDX-License-Identifier: LGPL-3.0-or-later
//! Filesystem detection
//!
//! Pure Rust implementation for detecting filesystem types

use crate::core::{Error, Result};
use crate::disk::partition::Partition;
use crate::disk::reader::DiskReader;

/// Filesystem type
#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemType {
    /// ext2/ext3/ext4
    Ext,
    /// NTFS
    Ntfs,
    /// FAT32
    Fat32,
    /// XFS
    Xfs,
    /// Btrfs
    Btrfs,
    /// ZFS
    Zfs,
    /// UFS (BSD)
    Ufs,
    /// HFS+ (macOS)
    HfsPlus,
    /// APFS (macOS)
    Apfs,
    /// Unknown filesystem
    Unknown,
}

/// Filesystem information
#[derive(Debug, Clone)]
pub struct FileSystem {
    fs_type: FileSystemType,
    label: Option<String>,
    uuid: Option<String>,
}

impl FileSystem {
    /// Detect filesystem from partition
    pub fn detect(reader: &mut DiskReader, partition: &Partition) -> Result<Self> {
        let offset = partition.start_lba * 512;

        // Try different filesystem detection methods
        if let Ok(fs) = Self::detect_ext(reader, offset) {
            return Ok(fs);
        }

        if let Ok(fs) = Self::detect_ntfs(reader, offset) {
            return Ok(fs);
        }

        if let Ok(fs) = Self::detect_fat32(reader, offset) {
            return Ok(fs);
        }

        if let Ok(fs) = Self::detect_xfs(reader, offset) {
            return Ok(fs);
        }

        if let Ok(fs) = Self::detect_btrfs(reader, offset) {
            return Ok(fs);
        }

        if let Ok(fs) = Self::detect_zfs(reader, offset) {
            return Ok(fs);
        }

        if let Ok(fs) = Self::detect_ufs(reader, offset) {
            return Ok(fs);
        }

        if let Ok(fs) = Self::detect_hfsplus(reader, offset) {
            return Ok(fs);
        }

        if let Ok(fs) = Self::detect_apfs(reader, offset) {
            return Ok(fs);
        }

        Ok(Self {
            fs_type: FileSystemType::Unknown,
            label: None,
            uuid: None,
        })
    }

    /// Detect ext2/ext3/ext4 filesystem
    fn detect_ext(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        // ext superblock is at offset 1024 from partition start
        let superblock_offset = partition_offset + 1024;
        let mut superblock = vec![0u8; 264];
        reader.read_exact_at(superblock_offset, &mut superblock)?;

        // Check magic number at offset 56-57 (0xEF53)
        if superblock[56] == 0x53 && superblock[57] == 0xEF {
            // Read volume label at offset 120 (16 bytes)
            let label_bytes = &superblock[120..136];
            let label = String::from_utf8_lossy(label_bytes)
                .trim_end_matches('\0')
                .to_string();

            let label = if label.is_empty() { None } else { Some(label) };

            // Read UUID at offset 104 (16 bytes)
            let uuid_bytes = &superblock[104..120];
            let uuid = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                uuid_bytes[0], uuid_bytes[1], uuid_bytes[2], uuid_bytes[3],
                uuid_bytes[4], uuid_bytes[5],
                uuid_bytes[6], uuid_bytes[7],
                uuid_bytes[8], uuid_bytes[9],
                uuid_bytes[10], uuid_bytes[11], uuid_bytes[12], uuid_bytes[13], uuid_bytes[14], uuid_bytes[15]
            );

            return Ok(Self {
                fs_type: FileSystemType::Ext,
                label,
                uuid: Some(uuid),
            });
        }

        Err(Error::Detection("Not an ext filesystem".to_string()))
    }

    /// Detect NTFS filesystem
    fn detect_ntfs(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        let mut boot_sector = vec![0u8; 512];
        reader.read_exact_at(partition_offset, &mut boot_sector)?;

        // Check NTFS signature "NTFS    " at offset 3
        if &boot_sector[3..11] == b"NTFS    " {
            return Ok(Self {
                fs_type: FileSystemType::Ntfs,
                label: None,
                uuid: None,
            });
        }

        Err(Error::Detection("Not an NTFS filesystem".to_string()))
    }

    /// Detect FAT32 filesystem
    fn detect_fat32(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        let mut boot_sector = vec![0u8; 512];
        reader.read_exact_at(partition_offset, &mut boot_sector)?;

        // Check FAT32 signature "FAT32   " at offset 82
        if boot_sector.len() > 90 && &boot_sector[82..90] == b"FAT32   " {
            return Ok(Self {
                fs_type: FileSystemType::Fat32,
                label: None,
                uuid: None,
            });
        }

        Err(Error::Detection("Not a FAT32 filesystem".to_string()))
    }

    /// Detect XFS filesystem
    fn detect_xfs(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        let mut superblock = vec![0u8; 512];
        reader.read_exact_at(partition_offset, &mut superblock)?;

        // Check XFS magic "XFSB"
        if &superblock[0..4] == b"XFSB" {
            return Ok(Self {
                fs_type: FileSystemType::Xfs,
                label: None,
                uuid: None,
            });
        }

        Err(Error::Detection("Not an XFS filesystem".to_string()))
    }

    /// Detect Btrfs filesystem
    fn detect_btrfs(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        // Btrfs superblock is at offset 65536
        let superblock_offset = partition_offset + 65536;
        let mut superblock = vec![0u8; 512];
        reader.read_exact_at(superblock_offset, &mut superblock)?;

        // Check Btrfs magic "_BHRfS_M"
        if &superblock[64..72] == b"_BHRfS_M" {
            return Ok(Self {
                fs_type: FileSystemType::Btrfs,
                label: None,
                uuid: None,
            });
        }

        Err(Error::Detection("Not a Btrfs filesystem".to_string()))
    }

    /// Detect ZFS filesystem
    fn detect_zfs(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        // ZFS has multiple labels at different offsets (128K, 256K, 512K, 1M)
        // We'll check the first one at 128K
        let label_offset = partition_offset + 131072; // 128KB
        let mut buffer = vec![0u8; 512];
        reader.read_exact_at(label_offset, &mut buffer)?;

        // Check for ZFS magic number (0x00bab10c at the start of the uberblock)
        // ZFS is complex, we'll do a simple check
        if buffer.len() >= 8 && u64::from_le_bytes(buffer[0..8].try_into().unwrap_or([0; 8])) != 0 {
            // Simple heuristic - check for typical ZFS patterns
            let label_offset2 = partition_offset + 262144; // 256KB
            let mut buffer2 = vec![0u8; 512];
            if reader.read_exact_at(label_offset2, &mut buffer2).is_ok() {
                return Ok(Self {
                    fs_type: FileSystemType::Zfs,
                    label: None,
                    uuid: None,
                });
            }
        }

        Err(Error::Detection("Not a ZFS filesystem".to_string()))
    }

    /// Detect UFS (BSD) filesystem
    fn detect_ufs(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        // UFS superblock is at offset 8192 for UFS1, or 65536 for UFS2
        // Try UFS2 first (more modern)
        let superblock_offset = partition_offset + 65536;
        let mut superblock = vec![0u8; 512];
        reader.read_exact_at(superblock_offset, &mut superblock)?;

        // Check UFS2 magic: 0x19540119
        if superblock.len() >= 1376 {
            let magic = u32::from_le_bytes([
                superblock[1372],
                superblock[1373],
                superblock[1374],
                superblock[1375],
            ]);
            if magic == 0x19540119 {
                return Ok(Self {
                    fs_type: FileSystemType::Ufs,
                    label: None,
                    uuid: None,
                });
            }
        }

        // Try UFS1 at offset 8192
        let superblock_offset = partition_offset + 8192;
        reader.read_exact_at(superblock_offset, &mut superblock)?;

        if superblock.len() >= 1376 {
            let magic = u32::from_le_bytes([
                superblock[1372],
                superblock[1373],
                superblock[1374],
                superblock[1375],
            ]);
            if magic == 0x011954 || magic == 0x19540119 {
                return Ok(Self {
                    fs_type: FileSystemType::Ufs,
                    label: None,
                    uuid: None,
                });
            }
        }

        Err(Error::Detection("Not a UFS filesystem".to_string()))
    }

    /// Detect HFS+ filesystem (macOS)
    fn detect_hfsplus(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        // HFS+ volume header is at offset 1024
        let header_offset = partition_offset + 1024;
        let mut header = vec![0u8; 512];
        reader.read_exact_at(header_offset, &mut header)?;

        // Check HFS+ signature "H+" or "HX" at offset 0-1
        if header.len() >= 2 {
            if (header[0] == b'H' && header[1] == b'+') || (header[0] == b'H' && header[1] == b'X') {
                return Ok(Self {
                    fs_type: FileSystemType::HfsPlus,
                    label: None,
                    uuid: None,
                });
            }
        }

        Err(Error::Detection("Not an HFS+ filesystem".to_string()))
    }

    /// Detect APFS filesystem (macOS)
    fn detect_apfs(reader: &mut DiskReader, partition_offset: u64) -> Result<Self> {
        // APFS container superblock is at the start of the partition
        let mut superblock = vec![0u8; 4096];
        reader.read_exact_at(partition_offset, &mut superblock)?;

        // Check APFS magic "NXSB" (container superblock) or "APSB" (volume superblock)
        if superblock.len() >= 36 {
            // Magic is at offset 32-35
            if &superblock[32..36] == b"NXSB" || &superblock[32..36] == b"APSB" {
                return Ok(Self {
                    fs_type: FileSystemType::Apfs,
                    label: None,
                    uuid: None,
                });
            }
        }

        Err(Error::Detection("Not an APFS filesystem".to_string()))
    }

    /// Get filesystem type
    pub fn fs_type(&self) -> &FileSystemType {
        &self.fs_type
    }

    /// Get filesystem label
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Get filesystem UUID
    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    /// Read file from filesystem (basic implementation)
    pub fn read_file(
        &self,
        _reader: &mut DiskReader,
        _partition: &Partition,
        path: &str,
    ) -> Result<Vec<u8>> {
        // This is a simplified implementation
        // A full implementation would need to:
        // 1. Parse the filesystem structure (inodes, directories)
        // 2. Navigate the directory tree
        // 3. Read file blocks
        //
        // For now, we'll focus on detecting OS from common locations
        Err(Error::Detection(format!(
            "File reading not yet implemented for path: {}",
            path
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_types() {
        assert_eq!(FileSystemType::Ext, FileSystemType::Ext);
        assert_eq!(FileSystemType::Ntfs, FileSystemType::Ntfs);
    }
}
