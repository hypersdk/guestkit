// SPDX-License-Identifier: LGPL-3.0-or-later
//! File operations compatible with libguestfs
//!
//! This implementation uses mounted filesystems (via NBD) to perform
//! file operations using standard Rust file I/O.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::fs;
use std::io::Write;
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
    pub(crate) fn resolve_guest_path(&self, guest_path: &str) -> Result<PathBuf> {
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

        let host_path = self.resolve_guest_path(directory)?;

        // Use ls -l command for long listing
        let output = std::process::Command::new("ls")
            .arg("-l")
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute ls: {}", e)))?;

        if !output.status.success() {
            return Err(Error::NotFound(format!(
                "Directory listing failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get file statistics
    ///
    /// Compatible with libguestfs g.stat()
    pub fn stat(&mut self, path: &str) -> Result<Stat> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: stat {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        let metadata = fs::metadata(&host_path).map_err(|e| {
            Error::NotFound(format!("Failed to stat {}: {}", path, e))
        })?;

        // Convert Rust metadata to libguestfs Stat format
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(Stat {
                dev: metadata.dev() as i64,
                ino: metadata.ino() as i64,
                mode: metadata.mode() as i64,
                nlink: metadata.nlink() as i64,
                uid: metadata.uid() as i64,
                gid: metadata.gid() as i64,
                rdev: metadata.rdev() as i64,
                size: metadata.size() as i64,
                blksize: metadata.blksize() as i64,
                blocks: metadata.blocks() as i64,
                atime: metadata.atime(),
                mtime: metadata.mtime(),
                ctime: metadata.ctime(),
            })
        }

        #[cfg(not(unix))]
        {
            Ok(Stat {
                dev: 0,
                ino: 0,
                mode: if metadata.is_dir() { 0o040755 } else { 0o100644 },
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                size: metadata.len() as i64,
                blksize: 4096,
                blocks: (metadata.len() / 512) as i64,
                atime: 0,
                mtime: 0,
                ctime: 0,
            })
        }
    }

    /// Get file size
    ///
    /// Compatible with libguestfs g.filesize()
    pub fn filesize(&mut self, file: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: filesize {}", file);
        }

        let host_path = self.resolve_guest_path(file)?;
        let metadata = fs::metadata(&host_path).map_err(|e| {
            Error::NotFound(format!("Failed to get size of {}: {}", file, e))
        })?;

        Ok(metadata.len() as i64)
    }

    /// Remove file
    ///
    /// Compatible with libguestfs g.rm()
    pub fn rm(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: rm {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::remove_file(&host_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to remove {}: {}", path, e))
        })
    }

    /// Remove directory
    ///
    /// Compatible with libguestfs g.rmdir()
    pub fn rmdir(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: rmdir {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::remove_dir(&host_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to remove directory {}: {}", path, e))
        })
    }

    /// Touch a file (create empty or update timestamp)
    ///
    /// Compatible with libguestfs g.touch()
    pub fn touch(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: touch {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Create file if it doesn't exist
        if !host_path.exists() {
            fs::File::create(&host_path).map_err(|e| {
                Error::CommandFailed(format!("Failed to touch {}: {}", path, e))
            })?;
        } else {
            // Update timestamp - use filetime crate or just write/truncate
            let file = fs::OpenOptions::new()
                .write(true)
                .open(&host_path)
                .map_err(|e| {
                    Error::CommandFailed(format!("Failed to touch {}: {}", path, e))
                })?;
            // Just opening for write updates the timestamp
            drop(file);
        }

        Ok(())
    }

    /// Change file permissions
    ///
    /// Compatible with libguestfs g.chmod()
    pub fn chmod(&mut self, mode: i32, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: chmod {:o} {}", mode, path);
        }

        let host_path = self.resolve_guest_path(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(mode as u32);
            fs::set_permissions(&host_path, permissions).map_err(|e| {
                Error::CommandFailed(format!("Failed to chmod {}: {}", path, e))
            })
        }

        #[cfg(not(unix))]
        {
            Err(Error::Unsupported(
                "Chmod is only supported on Unix systems".to_string()
            ))
        }
    }

    /// Change file ownership
    ///
    /// Compatible with libguestfs g.chown()
    pub fn chown(&mut self, owner: i32, group: i32, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: chown {}:{} {}", owner, group, path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Use chown command to change ownership
        let output = std::process::Command::new("chown")
            .arg(format!("{}:{}", owner, group))
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute chown: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Chown failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Resolve symlink to real path
    ///
    /// Compatible with libguestfs g.realpath()
    pub fn realpath(&mut self, path: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: realpath {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Canonicalize to resolve symlinks and relative paths
        let canonical = fs::canonicalize(&host_path).map_err(|e| {
            Error::NotFound(format!("Failed to resolve path {}: {}", path, e))
        })?;

        // Convert back to guest path by stripping the mount root prefix
        let root_mountpoint = self.mounted.get("/dev/sda1")
            .or_else(|| self.mounted.get("/dev/sda2"))
            .or_else(|| self.mounted.values().next())
            .ok_or_else(|| Error::InvalidState("No filesystem mounted".to_string()))?;

        let canonical_str = canonical.to_string_lossy();
        let guest_path = canonical_str
            .strip_prefix(root_mountpoint)
            .unwrap_or(&canonical_str);

        // Ensure path starts with /
        let result = if guest_path.starts_with('/') {
            guest_path.to_string()
        } else {
            format!("/{}", guest_path)
        };

        Ok(result)
    }

    /// Copy file
    ///
    /// Compatible with libguestfs g.cp()
    pub fn cp(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cp {} {}", src, dest);
        }

        let src_path = self.resolve_guest_path(src)?;
        let dest_path = self.resolve_guest_path(dest)?;

        fs::copy(&src_path, &dest_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to copy {} to {}: {}", src, dest, e))
        })?;

        Ok(())
    }

    /// Copy file preserving attributes
    ///
    /// Compatible with libguestfs g.cp_a()
    pub fn cp_a(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cp_a {} {}", src, dest);
        }

        let src_path = self.resolve_guest_path(src)?;
        let dest_path = self.resolve_guest_path(dest)?;

        // Use cp command to preserve attributes
        let output = std::process::Command::new("cp")
            .arg("-a")
            .arg(&src_path)
            .arg(&dest_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute cp: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Failed to copy {} to {}: {}",
                src, dest,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Copy recursively
    ///
    /// Compatible with libguestfs g.cp_r()
    pub fn cp_r(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cp_r {} {}", src, dest);
        }

        let src_path = self.resolve_guest_path(src)?;
        let dest_path = self.resolve_guest_path(dest)?;

        // Use cp command for recursive copy
        let output = std::process::Command::new("cp")
            .arg("-r")
            .arg(&src_path)
            .arg(&dest_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute cp: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Failed to copy {} to {}: {}",
                src, dest,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Move/rename file
    ///
    /// Compatible with libguestfs g.mv()
    pub fn mv(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: mv {} {}", src, dest);
        }

        let src_path = self.resolve_guest_path(src)?;
        let dest_path = self.resolve_guest_path(dest)?;

        fs::rename(&src_path, &dest_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to move {} to {}: {}", src, dest, e))
        })
    }

    /// Download file from guest to host
    ///
    /// Compatible with libguestfs g.download()
    pub fn download(&mut self, remotefilename: &str, filename: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: download {} {}", remotefilename, filename);
        }

        let guest_path = self.resolve_guest_path(remotefilename)?;
        let host_path = Path::new(filename);

        fs::copy(&guest_path, &host_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to download {} to {}: {}", remotefilename, filename, e))
        })?;

        Ok(())
    }

    /// Upload file from host to guest
    ///
    /// Compatible with libguestfs g.upload()
    pub fn upload(&mut self, filename: &str, remotefilename: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: upload {} {}", filename, remotefilename);
        }

        let host_path = Path::new(filename);
        let guest_path = self.resolve_guest_path(remotefilename)?;

        fs::copy(&host_path, &guest_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to upload {} to {}: {}", filename, remotefilename, e))
        })?;

        Ok(())
    }

    /// Append content to file
    ///
    /// Compatible with libguestfs g.write_append()
    pub fn write_append(&mut self, path: &str, content: &[u8]) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: write_append {} ({} bytes)", path, content.len());
        }

        let host_path = self.resolve_guest_path(path)?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&host_path)
            .map_err(|e| Error::CommandFailed(format!("Failed to open {} for append: {}", path, e)))?;

        file.write_all(content)
            .map_err(|e| Error::CommandFailed(format!("Failed to append to {}: {}", path, e)))?;

        Ok(())
    }

    /// Search file for pattern
    ///
    /// Compatible with libguestfs g.grep()
    pub fn grep(&mut self, regex: &str, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: grep {} {}", regex, path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Use grep command
        let output = std::process::Command::new("grep")
            .arg(regex)
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute grep: {}", e)))?;

        // grep returns exit code 1 if no matches found, which is not an error
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Search file for pattern (extended regex)
    ///
    /// Compatible with libguestfs g.egrep()
    pub fn egrep(&mut self, regex: &str, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: egrep {} {}", regex, path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Use grep with -E flag for extended regex
        let output = std::process::Command::new("grep")
            .arg("-E")
            .arg(regex)
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute egrep: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Search file for fixed strings
    ///
    /// Compatible with libguestfs g.fgrep()
    pub fn fgrep(&mut self, pattern: &str, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: fgrep {} {}", pattern, path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Use grep with -F flag for fixed string matching
        let output = std::process::Command::new("grep")
            .arg("-F")
            .arg(pattern)
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute fgrep: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Find files
    ///
    /// Compatible with libguestfs g.find()
    pub fn find(&mut self, directory: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: find {}", directory);
        }

        let host_path = self.resolve_guest_path(directory)?;

        // Use find command
        let output = std::process::Command::new("find")
            .arg(&host_path)
            .arg("-type")
            .arg("f")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute find: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Find failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Remove the host mount prefix to return guest paths
        let prefix = host_path.to_string_lossy();
        Ok(stdout
            .lines()
            .map(|line| {
                line.strip_prefix(prefix.as_ref())
                    .unwrap_or(line)
                    .to_string()
            })
            .collect())
    }

    /// Find files (NUL-separated)
    ///
    /// Compatible with libguestfs g.find0()
    pub fn find0(&mut self, directory: &str, files: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: find0 {} {}", directory, files);
        }

        let host_path = self.resolve_guest_path(directory)?;

        // Use find command with -print0 to get NUL-separated output
        let output = std::process::Command::new("find")
            .arg(&host_path)
            .arg("-type")
            .arg("f")
            .arg("-print0")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute find: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Find failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Write output to the specified file
        fs::write(files, &output.stdout).map_err(|e| {
            Error::CommandFailed(format!("Failed to write find0 output to {}: {}", files, e))
        })
    }

    /// Calculate disk usage
    ///
    /// Compatible with libguestfs g.du()
    pub fn du(&mut self, path: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: du {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Use du command to get disk usage in bytes
        let output = std::process::Command::new("du")
            .arg("-sb")
            .arg(&host_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute du: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "Du failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse output: "12345\t/path"
        let size_str = stdout
            .split_whitespace()
            .next()
            .ok_or_else(|| Error::InvalidFormat("Invalid du output".to_string()))?;

        size_str
            .parse::<i64>()
            .map_err(|e| Error::InvalidFormat(format!("Failed to parse du output: {}", e)))
    }

    /// Remove a file
    ///
    /// Compatible with libguestfs g.rm()
    pub fn rm(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: rm {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        if !host_path.exists() {
            return Err(Error::NotFound(format!("File not found: {}", path)));
        }

        if host_path.is_dir() {
            return Err(Error::InvalidOperation(format!("Cannot rm directory (use rmdir or rm_rf): {}", path)));
        }

        fs::remove_file(&host_path)
            .map_err(|e| Error::Io(e))
    }

    /// Remove a file or directory recursively (force)
    ///
    /// Compatible with libguestfs g.rm_rf()
    pub fn rm_rf(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: rm_rf {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        if !host_path.exists() {
            // rm_rf doesn't error if path doesn't exist (like shell rm -rf)
            return Ok(());
        }

        if host_path.is_dir() {
            fs::remove_dir_all(&host_path)
                .map_err(|e| Error::Io(e))
        } else {
            fs::remove_file(&host_path)
                .map_err(|e| Error::Io(e))
        }
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
