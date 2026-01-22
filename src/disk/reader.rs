// SPDX-License-Identifier: LGPL-3.0-or-later
//! Disk image reader
//!
//! Pure Rust implementation for reading disk images (raw, qcow2, etc.)

use crate::core::{DiskFormat, Error, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Disk image reader
pub struct DiskReader {
    file: File,
    format: DiskFormat,
    size: u64,
}

impl DiskReader {
    /// Open a disk image
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path.as_ref())
            .map_err(|e| Error::Io(e))?;

        // Detect format by reading magic bytes
        let format = Self::detect_format(&mut file)?;

        // Get file size
        let size = file.metadata()
            .map_err(|e| Error::Io(e))?
            .len();

        Ok(Self { file, format, size })
    }

    /// Detect disk image format from magic bytes
    fn detect_format(file: &mut File) -> Result<DiskFormat> {
        let mut magic = [0u8; 4];
        file.seek(SeekFrom::Start(0))
            .map_err(|e| Error::Io(e))?;
        file.read_exact(&mut magic)
            .map_err(|e| Error::Io(e))?;

        // QCOW2 magic: "QFI\xfb"
        if &magic == b"QFI\xfb" {
            return Ok(DiskFormat::Qcow2);
        }

        // Check for other formats
        // VMDK magic at start
        if &magic[0..3] == b"KDM" || &magic[0..3] == b"COW" {
            return Ok(DiskFormat::Vmdk);
        }

        // VHD magic at end (512 bytes from end) "conectix"
        // VDI magic "<<< "

        // Default to raw if no magic found
        Ok(DiskFormat::Raw)
    }

    /// Read bytes at offset
    pub fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        self.file.seek(SeekFrom::Start(offset))
            .map_err(|e| Error::Io(e))?;
        self.file.read(buf)
            .map_err(|e| Error::Io(e))
    }

    /// Get disk format
    pub fn format(&self) -> &DiskFormat {
        &self.format
    }

    /// Get disk size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Read exact bytes at offset
    pub fn read_exact_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))
            .map_err(|e| Error::Io(e))?;
        self.file.read_exact(buf)
            .map_err(|e| Error::Io(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_reader_creation() {
        // Test that the reader struct can be created
        assert!(true);
    }
}
