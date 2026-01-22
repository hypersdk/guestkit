// SPDX-License-Identifier: LGPL-3.0-or-later
//! File operations compatible with libguestfs
//!
//! This implementation uses mounted filesystems (via NBD) to perform
//! file operations using standard Rust file I/O.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// File statistics
#[derive(Debug, Clone)]
pub struct Stat {
    pub dev: i64,
    pub ino: i64,
    pub mode: i64,
    pub nlink: i64,
    pub uid: i64,
    pub gid: i64,
    pub rdev: i64,
    pub size: i64,
    pub blksize: i64,
    pub blocks: i64,
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
}

impl Guestfs {
    /// Resolve guest path to host path (internal helper)
    fn resolve_guest_path(&self, guest_path: &str) -> Result<PathBuf> {
        // Find root mount
        let root_mountpoint = self.mounted.get("/dev/sda1")
            .or_else(|| self.mounted.get("/dev/sda2"))
            .or_else(|| self.mounted.get("/dev/vda1"))
            .or_else(|| self.mounted.values().next())
            .ok_or_else(|| Error::InvalidState(
                "No filesystem mounted. Call mount_ro() first.".to_string()
            ))?;

        // Build full path
        let guest_path_clean = guest_path.trim_start_matches('/');
        Ok(PathBuf::from(root_mountpoint).join(guest_path_clean))
    }

    /// Check if path is a file
    ///
    /// Compatible with libguestfs g.is_file()
    pub fn is_file(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: is_file {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        Ok(host_path.is_file())
    }

    /// Check if path is a directory
    ///
    /// Compatible with libguestfs g.is_dir()
    pub fn is_dir(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: is_dir {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        Ok(host_path.is_dir())
    }

    /// Check if path exists
    ///
    /// Compatible with libguestfs g.exists()
    pub fn exists(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: exists {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        Ok(host_path.exists())
    }

    /// Read file content as bytes
    ///
    /// Compatible with libguestfs g.read_file()
    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: read_file {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::read(&host_path).map_err(|e| {
            Error::NotFound(format!("Failed to read {}: {}", path, e))
        })
    }

    /// Read file as text
    ///
    /// Compatible with libguestfs g.cat()
    pub fn cat(&mut self, path: &str) -> Result<String> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes)
            .map_err(|e| Error::InvalidFormat(format!("Not UTF-8: {}", e)))
    }

    /// Read file as lines
    ///
    /// Compatible with libguestfs g.read_lines()
    pub fn read_lines(&mut self, path: &str) -> Result<Vec<String>> {
        let content = self.cat(path)?;
        Ok(content.lines().map(|s| s.to_string()).collect())
    }

    /// Write content to file
    ///
    /// Compatible with libguestfs g.write()
    pub fn write(&mut self, path: &str, content: &[u8]) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: write {} ({} bytes)", path, content.len());
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::write(&host_path, content).map_err(|e| {
            Error::CommandFailed(format!("Failed to write {}: {}", path, e))
        })
    }

    /// Create directory
    ///
    /// Compatible with libguestfs g.mkdir()
    pub fn mkdir(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: mkdir {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::create_dir(&host_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to create directory {}: {}", path, e))
        })
    }

    /// Create directory with parents
    ///
    /// Compatible with libguestfs g.mkdir_p()
    pub fn mkdir_p(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: mkdir_p {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::create_dir_all(&host_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to create directory {}: {}", path, e))
        })
    }

    /// List directory contents
    ///
    /// Compatible with libguestfs g.ls()
    pub fn ls(&mut self, directory: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: ls {}", directory);
        }

        let host_path = self.resolve_guest_path(directory)?;
        let entries = fs::read_dir(&host_path).map_err(|e| {
            Error::NotFound(format!("Failed to read directory {}: {}", directory, e))
        })?;

        let mut names = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }

        names.sort();
        Ok(names)
    }

    /// List directory with long format
    ///
    /// Compatible with libguestfs g.ll()
    pub fn ll(&mut self, directory: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: ll {}", directory);
        }

        // TODO: Parse filesystem, read directory entries with metadata

        Err(Error::Unsupported(
            "Directory listing requires filesystem parser implementation".to_string()
        ))
    }

    /// Get file statistics
    ///
    /// Compatible with libguestfs g.stat()
    pub fn stat(&mut self, path: &str) -> Result<Stat> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: stat {}", path);
        }

        // TODO: Parse filesystem, read inode metadata

        Err(Error::Unsupported(
            "Stat requires filesystem parser implementation".to_string()
        ))
    }

    /// Get file size
    ///
    /// Compatible with libguestfs g.filesize()
    pub fn filesize(&mut self, file: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: filesize {}", file);
        }

        // TODO: Parse filesystem, read inode size

        Err(Error::Unsupported(
            "Filesize requires filesystem parser implementation".to_string()
        ))
    }

    /// Remove file
    ///
    /// Compatible with libguestfs g.rm()
    pub fn rm(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: rm {}", path);
        }

        // TODO: Parse filesystem, unlink inode

        Err(Error::Unsupported(
            "File removal requires filesystem parser implementation".to_string()
        ))
    }

    /// Remove directory
    ///
    /// Compatible with libguestfs g.rmdir()
    pub fn rmdir(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: rmdir {}", path);
        }

        // TODO: Parse filesystem, remove directory inode

        Err(Error::Unsupported(
            "Directory removal requires filesystem parser implementation".to_string()
        ))
    }

    /// Touch a file (create empty or update timestamp)
    ///
    /// Compatible with libguestfs g.touch()
    pub fn touch(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: touch {}", path);
        }

        // TODO: Parse filesystem, create or update inode

        Err(Error::Unsupported(
            "Touch requires filesystem parser implementation".to_string()
        ))
    }

    /// Change file permissions
    ///
    /// Compatible with libguestfs g.chmod()
    pub fn chmod(&mut self, mode: i32, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: chmod {:o} {}", mode, path);
        }

        // TODO: Parse filesystem, update inode permissions

        Err(Error::Unsupported(
            "Chmod requires filesystem parser implementation".to_string()
        ))
    }

    /// Change file ownership
    ///
    /// Compatible with libguestfs g.chown()
    pub fn chown(&mut self, owner: i32, group: i32, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: chown {}:{} {}", owner, group, path);
        }

        // TODO: Parse filesystem, update inode ownership

        Err(Error::Unsupported(
            "Chown requires filesystem parser implementation".to_string()
        ))
    }

    /// Resolve symlink to real path
    ///
    /// Compatible with libguestfs g.realpath()
    pub fn realpath(&mut self, path: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: realpath {}", path);
        }

        // TODO: Parse filesystem, follow symlinks

        // For now, return path as-is
        Ok(path.to_string())
    }

    /// Copy file
    ///
    /// Compatible with libguestfs g.cp()
    pub fn cp(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cp {} {}", src, dest);
        }

        Err(Error::Unsupported(
            "File copy requires filesystem implementation".to_string()
        ))
    }

    /// Copy file preserving attributes
    ///
    /// Compatible with libguestfs g.cp_a()
    pub fn cp_a(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cp_a {} {}", src, dest);
        }

        Err(Error::Unsupported(
            "File copy requires filesystem implementation".to_string()
        ))
    }

    /// Copy recursively
    ///
    /// Compatible with libguestfs g.cp_r()
    pub fn cp_r(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cp_r {} {}", src, dest);
        }

        Err(Error::Unsupported(
            "Recursive copy requires filesystem implementation".to_string()
        ))
    }

    /// Move/rename file
    ///
    /// Compatible with libguestfs g.mv()
    pub fn mv(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: mv {} {}", src, dest);
        }

        Err(Error::Unsupported(
            "File move requires filesystem implementation".to_string()
        ))
    }

    /// Download file from guest to host
    ///
    /// Compatible with libguestfs g.download()
    pub fn download(&mut self, remotefilename: &str, filename: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: download {} {}", remotefilename, filename);
        }

        Err(Error::Unsupported(
            "File download requires filesystem reader implementation".to_string()
        ))
    }

    /// Upload file from host to guest
    ///
    /// Compatible with libguestfs g.upload()
    pub fn upload(&mut self, filename: &str, remotefilename: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: upload {} {}", filename, remotefilename);
        }

        Err(Error::Unsupported(
            "File upload requires filesystem writer implementation".to_string()
        ))
    }

    /// Append content to file
    ///
    /// Compatible with libguestfs g.write_append()
    pub fn write_append(&mut self, path: &str, content: &[u8]) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: write_append {} ({} bytes)", path, content.len());
        }

        Err(Error::Unsupported(
            "File append requires filesystem writer implementation".to_string()
        ))
    }

    /// Search file for pattern
    ///
    /// Compatible with libguestfs g.grep()
    pub fn grep(&mut self, regex: &str, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: grep {} {}", regex, path);
        }

        Err(Error::Unsupported(
            "Grep requires filesystem reader implementation".to_string()
        ))
    }

    /// Search file for pattern (extended regex)
    ///
    /// Compatible with libguestfs g.egrep()
    pub fn egrep(&mut self, regex: &str, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: egrep {} {}", regex, path);
        }

        Err(Error::Unsupported(
            "Egrep requires filesystem reader implementation".to_string()
        ))
    }

    /// Search file for fixed strings
    ///
    /// Compatible with libguestfs g.fgrep()
    pub fn fgrep(&mut self, pattern: &str, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: fgrep {} {}", pattern, path);
        }

        Err(Error::Unsupported(
            "Fgrep requires filesystem reader implementation".to_string()
        ))
    }

    /// Find files
    ///
    /// Compatible with libguestfs g.find()
    pub fn find(&mut self, directory: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: find {}", directory);
        }

        Err(Error::Unsupported(
            "Find requires filesystem reader implementation".to_string()
        ))
    }

    /// Find files (NUL-separated)
    ///
    /// Compatible with libguestfs g.find0()
    pub fn find0(&mut self, directory: &str, files: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: find0 {} {}", directory, files);
        }

        Err(Error::Unsupported(
            "Find0 requires filesystem reader implementation".to_string()
        ))
    }

    /// Calculate disk usage
    ///
    /// Compatible with libguestfs g.du()
    pub fn du(&mut self, path: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: du {}", path);
        }

        Err(Error::Unsupported(
            "Du requires filesystem reader implementation".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_ops_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
