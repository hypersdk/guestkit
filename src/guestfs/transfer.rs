// SPDX-License-Identifier: LGPL-3.0-or-later
//! Advanced file transfer operations for disk image manipulation
//!
//! This implementation provides advanced download/upload functionality.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

impl Guestfs {
    /// Download file with offset
    ///
    pub fn download_offset(
        &mut self,
        remote: &str,
        local: &str,
        offset: i64,
        size: i64,
    ) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!(
                "guestfs: download_offset {} {} {} {}",
                remote, local, offset, size
            );
        }

        let host_path = self.resolve_guest_path(remote)?;

        // Read from remote file with offset
        let mut remote_file = File::open(&host_path).map_err(Error::Io)?;

        remote_file
            .seek(SeekFrom::Start(offset as u64))
            .map_err(Error::Io)?;

        let mut buffer = vec![0u8; size as usize];
        remote_file
            .read_exact(&mut buffer)
            .map_err(Error::Io)?;

        // Write to local file
        let mut local_file = File::create(local).map_err(Error::Io)?;

        local_file.write_all(&buffer).map_err(Error::Io)?;

        Ok(())
    }

    /// Upload file with offset
    ///
    pub fn upload_offset(&mut self, local: &str, remote: &str, offset: i64) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: upload_offset {} {} {}", local, remote, offset);
        }

        let host_path = self.resolve_guest_path(remote)?;

        // Read from local file
        let mut local_file = File::open(local).map_err(Error::Io)?;

        let mut buffer = Vec::new();
        local_file
            .read_to_end(&mut buffer)
            .map_err(Error::Io)?;

        // Write to remote file with offset
        let mut remote_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&host_path)
            .map_err(Error::Io)?;

        remote_file
            .seek(SeekFrom::Start(offset as u64))
            .map_err(Error::Io)?;

        remote_file.write_all(&buffer).map_err(Error::Io)?;

        Ok(())
    }

    /// Copy file between locations in guest
    ///
    pub fn copy_file_to_file(
        &mut self,
        src: &str,
        dest: &str,
        srcoffset: i64,
        destoffset: i64,
        size: i64,
    ) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!(
                "guestfs: copy_file_to_file {} {} {} {} {}",
                src, dest, srcoffset, destoffset, size
            );
        }

        let host_src = self.resolve_guest_path(src)?;
        let host_dest = self.resolve_guest_path(dest)?;

        // Read from source with offset
        let mut src_file = File::open(&host_src).map_err(Error::Io)?;

        src_file
            .seek(SeekFrom::Start(srcoffset as u64))
            .map_err(Error::Io)?;

        let mut buffer = vec![0u8; size as usize];
        src_file.read_exact(&mut buffer).map_err(Error::Io)?;

        // Write to destination with offset
        let mut dest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&host_dest)
            .map_err(Error::Io)?;

        dest_file
            .seek(SeekFrom::Start(destoffset as u64))
            .map_err(Error::Io)?;

        dest_file.write_all(&buffer).map_err(Error::Io)?;

        Ok(())
    }

    /// Copy device to device
    ///
    pub fn copy_device_to_device(
        &mut self,
        src: &str,
        dest: &str,
        srcoffset: i64,
        destoffset: i64,
        size: i64,
    ) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!(
                "guestfs: copy_device_to_device {} {} {} {} {}",
                src, dest, srcoffset, destoffset, size
            );
        }

        self.setup_nbd_if_needed()?;

        let nbd_device = self
            .nbd_device
            .as_ref()
            .ok_or_else(|| Error::InvalidState("NBD device not available".to_string()))?;

        // For simplicity, treat as file copy
        // In a full implementation, this would use dd or similar for block devices
        let src_partition = format!(
            "{}p{}",
            nbd_device.device_path().display(),
            src.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1)
        );
        let dest_partition = format!(
            "{}p{}",
            nbd_device.device_path().display(),
            dest.chars()
                .last()
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1)
        );

        // Use dd for device-to-device copy
        use std::process::Command;

        let output = Command::new("dd")
            .arg(format!("if={}", src_partition))
            .arg(format!("of={}", dest_partition))
            .arg("bs=512")
            .arg(format!("skip={}", srcoffset / 512))
            .arg(format!("seek={}", destoffset / 512))
            .arg(format!("count={}", (size + 511) / 512))
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute dd: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "dd failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Copy file to device
    ///
    pub fn copy_file_to_device(
        &mut self,
        src: &str,
        dest: &str,
        srcoffset: i64,
        destoffset: i64,
        size: i64,
    ) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!(
                "guestfs: copy_file_to_device {} {} {} {} {}",
                src, dest, srcoffset, destoffset, size
            );
        }

        let host_src = self.resolve_guest_path(src)?;

        self.setup_nbd_if_needed()?;

        let nbd_device = self
            .nbd_device
            .as_ref()
            .ok_or_else(|| Error::InvalidState("NBD device not available".to_string()))?;

        let dest_partition = format!(
            "{}p{}",
            nbd_device.device_path().display(),
            dest.chars()
                .last()
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1)
        );

        // Read from source file
        let mut src_file = File::open(&host_src).map_err(Error::Io)?;

        src_file
            .seek(SeekFrom::Start(srcoffset as u64))
            .map_err(Error::Io)?;

        let mut buffer = vec![0u8; size as usize];
        src_file.read_exact(&mut buffer).map_err(Error::Io)?;

        // Write to device
        let mut dest_file = OpenOptions::new()
            .write(true)
            .open(&dest_partition)
            .map_err(Error::Io)?;

        dest_file
            .seek(SeekFrom::Start(destoffset as u64))
            .map_err(Error::Io)?;

        dest_file.write_all(&buffer).map_err(Error::Io)?;

        Ok(())
    }

    /// Copy device to file
    ///
    pub fn copy_device_to_file(
        &mut self,
        src: &str,
        dest: &str,
        srcoffset: i64,
        destoffset: i64,
        size: i64,
    ) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!(
                "guestfs: copy_device_to_file {} {} {} {} {}",
                src, dest, srcoffset, destoffset, size
            );
        }

        self.setup_nbd_if_needed()?;

        let nbd_device = self
            .nbd_device
            .as_ref()
            .ok_or_else(|| Error::InvalidState("NBD device not available".to_string()))?;

        let src_partition = format!(
            "{}p{}",
            nbd_device.device_path().display(),
            src.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1)
        );

        // Read from device
        let mut src_file = File::open(&src_partition).map_err(Error::Io)?;

        src_file
            .seek(SeekFrom::Start(srcoffset as u64))
            .map_err(Error::Io)?;

        let mut buffer = vec![0u8; size as usize];
        src_file.read_exact(&mut buffer).map_err(Error::Io)?;

        // Write to destination file
        let host_dest = self.resolve_guest_path(dest)?;

        let mut dest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&host_dest)
            .map_err(Error::Io)?;

        dest_file
            .seek(SeekFrom::Start(destoffset as u64))
            .map_err(Error::Io)?;

        dest_file.write_all(&buffer).map_err(Error::Io)?;

        Ok(())
    }

    /// Compare two files
    ///
    pub fn compare(&mut self, file1: &str, file2: &str) -> Result<i32> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: compare {} {}", file1, file2);
        }

        let host_file1 = self.resolve_guest_path(file1)?;
        let host_file2 = self.resolve_guest_path(file2)?;

        let mut f1 = File::open(&host_file1).map_err(Error::Io)?;
        let mut f2 = File::open(&host_file2).map_err(Error::Io)?;

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();

        f1.read_to_end(&mut buf1).map_err(Error::Io)?;
        f2.read_to_end(&mut buf2).map_err(Error::Io)?;

        if buf1 == buf2 {
            Ok(0) // Files are identical
        } else {
            Ok(1) // Files differ
        }
    }

    /// Get size of file or device
    ///
    pub fn get_size(&mut self, path: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_size {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let metadata = std::fs::metadata(&host_path).map_err(Error::Io)?;

        Ok(metadata.len() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
