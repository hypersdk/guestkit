// SPDX-License-Identifier: LGPL-3.0-or-later
//! File metadata operations compatible with libguestfs
//!
//! This implementation provides file metadata and stat operations.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::fs;
use std::os::unix::fs::PermissionsExt;

impl Guestfs {
    /// Get file inode number
    ///
    /// Compatible with libguestfs g.inotify_files()
    pub fn get_inode(&mut self, path: &str) -> Result<u64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_inode {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.ino())
        }

        #[cfg(not(unix))]
        {
            Err(Error::Unsupported("Inode numbers not supported on this platform".to_string()))
        }
    }

    /// Get file access time
    ///
    /// Compatible with libguestfs g.lstatns() - access time
    pub fn get_atime(&mut self, path: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_atime {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.atime())
        }

        #[cfg(not(unix))]
        {
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            if let Ok(accessed) = metadata.accessed() {
                if let Ok(duration) = accessed.duration_since(std::time::UNIX_EPOCH) {
                    return Ok(duration.as_secs() as i64);
                }
            }
            Err(Error::Unsupported("Access time not available".to_string()))
        }
    }

    /// Get file modification time
    ///
    /// Compatible with libguestfs g.lstatns() - modification time
    pub fn get_mtime(&mut self, path: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_mtime {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.mtime())
        }

        #[cfg(not(unix))]
        {
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    return Ok(duration.as_secs() as i64);
                }
            }
            Err(Error::Unsupported("Modification time not available".to_string()))
        }
    }

    /// Get file change time
    ///
    /// Compatible with libguestfs g.lstatns() - change time
    pub fn get_ctime(&mut self, path: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_ctime {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.ctime())
        }

        #[cfg(not(unix))]
        {
            Err(Error::Unsupported("Change time not supported on this platform".to_string()))
        }
    }

    /// Get file UID
    ///
    /// Compatible with libguestfs g.lstatns() - UID
    pub fn get_uid(&mut self, path: &str) -> Result<u32> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_uid {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.uid())
        }

        #[cfg(not(unix))]
        {
            Err(Error::Unsupported("UID not supported on this platform".to_string()))
        }
    }

    /// Get file GID
    ///
    /// Compatible with libguestfs g.lstatns() - GID
    pub fn get_gid(&mut self, path: &str) -> Result<u32> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_gid {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.gid())
        }

        #[cfg(not(unix))]
        {
            Err(Error::Unsupported("GID not supported on this platform".to_string()))
        }
    }

    /// Get file mode (permissions)
    ///
    /// Compatible with libguestfs g.lstatns() - mode
    pub fn get_mode(&mut self, path: &str) -> Result<u32> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_mode {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let metadata = fs::metadata(&host_path)
            .map_err(|e| Error::Io(e))?;

        #[cfg(unix)]
        {
            Ok(metadata.permissions().mode())
        }

        #[cfg(not(unix))]
        {
            // On Windows, just check read-only
            let mode = if metadata.permissions().readonly() {
                0o444
            } else {
                0o644
            };
            Ok(mode)
        }
    }

    /// Get number of hard links
    ///
    /// Compatible with libguestfs g.lstatns() - nlink
    pub fn get_nlink(&mut self, path: &str) -> Result<u64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_nlink {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.nlink())
        }

        #[cfg(not(unix))]
        {
            Ok(1) // Windows doesn't have hard link count in standard metadata
        }
    }

    /// Get device ID
    ///
    /// Compatible with libguestfs g.lstatns() - dev
    pub fn get_dev(&mut self, path: &str) -> Result<u64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_dev {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.dev())
        }

        #[cfg(not(unix))]
        {
            Err(Error::Unsupported("Device ID not supported on this platform".to_string()))
        }
    }

    /// Get device type for special files
    ///
    /// Compatible with libguestfs g.lstatns() - rdev
    pub fn get_rdev(&mut self, path: &str) -> Result<u64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_rdev {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.rdev())
        }

        #[cfg(not(unix))]
        {
            Err(Error::Unsupported("Device type not supported on this platform".to_string()))
        }
    }

    /// Get block count
    ///
    /// Compatible with libguestfs g.lstatns() - blocks
    pub fn get_blocks(&mut self, path: &str) -> Result<u64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_blocks {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.blocks())
        }

        #[cfg(not(unix))]
        {
            // Approximate from file size
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok((metadata.len() + 511) / 512) // Round up to 512-byte blocks
        }
    }

    /// Get block size
    ///
    /// Compatible with libguestfs g.lstatns() - blksize
    pub fn get_blksize(&mut self, path: &str) -> Result<u64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: get_blksize {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(&host_path)
                .map_err(|e| Error::Io(e))?;
            Ok(metadata.blksize())
        }

        #[cfg(not(unix))]
        {
            Ok(4096) // Standard block size
        }
    }

    /// Check if path is socket
    ///
    /// Compatible with libguestfs g.is_socket()
    pub fn is_socket(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: is_socket {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let metadata = fs::metadata(&host_path)
            .map_err(|e| Error::Io(e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            Ok(metadata.file_type().is_socket())
        }

        #[cfg(not(unix))]
        {
            Ok(false)
        }
    }

    /// Check if path is FIFO (named pipe)
    ///
    /// Compatible with libguestfs g.is_fifo()
    pub fn is_fifo(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: is_fifo {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let metadata = fs::metadata(&host_path)
            .map_err(|e| Error::Io(e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            Ok(metadata.file_type().is_fifo())
        }

        #[cfg(not(unix))]
        {
            Ok(false)
        }
    }

    /// Check if path is block device
    ///
    /// Compatible with libguestfs g.is_blockdev()
    pub fn is_blockdev(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: is_blockdev {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let metadata = fs::metadata(&host_path)
            .map_err(|e| Error::Io(e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            Ok(metadata.file_type().is_block_device())
        }

        #[cfg(not(unix))]
        {
            Ok(false)
        }
    }

    /// Check if path is character device
    ///
    /// Compatible with libguestfs g.is_chardev()
    pub fn is_chardev(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: is_chardev {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        let metadata = fs::metadata(&host_path)
            .map_err(|e| Error::Io(e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            Ok(metadata.file_type().is_char_device())
        }

        #[cfg(not(unix))]
        {
            Ok(false)
        }
    }

    /// Check if path is symbolic link
    ///
    /// Compatible with libguestfs g.is_symlink()
    pub fn is_symlink(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: is_symlink {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Use symlink_metadata to not follow symlinks
        let metadata = fs::symlink_metadata(&host_path)
            .map_err(|e| Error::Io(e))?;

        Ok(metadata.file_type().is_symlink())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
