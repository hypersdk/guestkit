// SPDX-License-Identifier: LGPL-3.0-or-later
//! File operations for disk image manipulation
//!
//! This implementation uses mounted filesystems (via NBD) to perform
//! file operations using standard Rust file I/O.

use crate::core::{Error, Result};
use crate::guestfs::security_utils::PathValidator;
use crate::guestfs::Guestfs;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

impl Guestfs {
    /// Find the root mountpoint (internal helper)
    fn find_root_mountpoint(&self) -> Result<&str> {
        self.mounted
            .get("/dev/sda1")
            .or_else(|| self.mounted.get("/dev/sda2"))
            .or_else(|| self.mounted.get("/dev/vda1"))
            .or_else(|| self.mounted.values().next())
            .ok_or_else(|| {
                Error::InvalidState("No filesystem mounted. Call mount_ro() first.".to_string())
            })
            .map(|s| s.as_str())
    }

    /// Resolve guest path to host path (internal helper)
    ///
    /// This function securely resolves guest paths by:
    /// 1. Validating the path doesn't contain dangerous patterns like ".."
    /// 2. Normalizing the path
    /// 3. Canonicalizing it to resolve symlinks
    /// 4. Verifying the canonical path stays within the guest root
    pub(crate) fn resolve_guest_path(&self, guest_path: &str) -> Result<PathBuf> {
        // 1. Validate path to prevent path traversal attacks
        PathValidator::validate_fs_path(guest_path)?;

        // 2. Find root mount
        let root_mountpoint = self.find_root_mountpoint()?;

        // 3. Build candidate path
        let guest_path_clean = guest_path.trim_start_matches('/');
        let candidate_path = PathBuf::from(root_mountpoint).join(guest_path_clean);

        // 4. Canonicalize path to resolve symlinks and get absolute path
        // Note: canonicalize() requires the path to exist
        let canonical = candidate_path.canonicalize().map_err(|e| {
            // If path doesn't exist, return NotFound instead of generic error
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::NotFound(format!("Path does not exist: {}", guest_path))
            } else {
                Error::Io(e)
            }
        })?;

        // 5. Get canonical root for security check
        let root_canonical = PathBuf::from(root_mountpoint).canonicalize().map_err(|e| {
            Error::InvalidState(format!("Failed to canonicalize mount root: {}", e))
        })?;

        // 6. Security check: ensure resolved path is within guest root
        if !canonical.starts_with(&root_canonical) {
            return Err(Error::InvalidOperation(format!(
                "Path '{}' escapes guest root (symlink attack or path traversal)",
                guest_path
            )));
        }

        Ok(canonical)
    }

    /// Check if path is a file
    ///
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
    pub fn is_dir(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.trace {
            eprintln!("guestfs: is_dir {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        Ok(host_path.is_dir())
    }

    /// Check if path exists
    ///
    pub fn exists(&mut self, path: &str) -> Result<bool> {
        self.ensure_ready()?;

        if self.trace {
            eprintln!("guestfs: exists {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        Ok(host_path.exists())
    }

    /// Read file content as bytes
    ///
    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        self.ensure_ready()?;

        // Only show detailed file operations in trace mode (not verbose)
        if self.trace {
            eprintln!("guestfs: read_file {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::read(&host_path).map_err(|e| Error::NotFound(format!("Failed to read {}: {}", path, e)))
    }

    /// Read file as text
    ///
    pub fn cat(&mut self, path: &str) -> Result<String> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).map_err(|e| Error::InvalidFormat(format!("Not UTF-8: {}", e)))
    }

    /// Read file as lines
    ///
    pub fn read_lines(&mut self, path: &str) -> Result<Vec<String>> {
        let content = self.cat(path)?;
        Ok(content.lines().map(|s| s.to_string()).collect())
    }

    /// Write content to file
    ///
    pub fn write(&mut self, path: &str, content: &[u8]) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: write {} ({} bytes)", path, content.len());
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::write(&host_path, content)
            .map_err(|e| Error::CommandFailed(format!("Failed to write {}: {}", path, e)))
    }

    /// Create directory
    ///
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
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }

        names.sort();
        Ok(names)
    }

    /// List directory with long format
    ///
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

    /// Get file size
    ///
    pub fn filesize(&mut self, file: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: filesize {}", file);
        }

        let host_path = self.resolve_guest_path(file)?;
        let metadata = fs::metadata(&host_path)
            .map_err(|e| Error::NotFound(format!("Failed to get size of {}: {}", file, e)))?;

        Ok(metadata.len() as i64)
    }

    /// Remove directory
    ///
    pub fn rmdir(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.trace {
            eprintln!("guestfs: rmdir {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;
        fs::remove_dir(&host_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to remove directory {}: {}", path, e))
        })
    }

    /// Touch a file (create empty or update timestamp)
    ///
    pub fn touch(&mut self, path: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: touch {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Create file if it doesn't exist
        if !host_path.exists() {
            fs::File::create(&host_path)
                .map_err(|e| Error::CommandFailed(format!("Failed to touch {}: {}", path, e)))?;
        } else {
            // Update timestamp - use filetime crate or just write/truncate
            let file = fs::OpenOptions::new()
                .write(true)
                .open(&host_path)
                .map_err(|e| Error::CommandFailed(format!("Failed to touch {}: {}", path, e)))?;
            // Just opening for write updates the timestamp
            drop(file);
        }

        Ok(())
    }

    /// Change file permissions
    ///
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
            fs::set_permissions(&host_path, permissions)
                .map_err(|e| Error::CommandFailed(format!("Failed to chmod {}: {}", path, e)))
        }

        #[cfg(not(unix))]
        {
            Err(Error::Unsupported(
                "Chmod is only supported on Unix systems".to_string(),
            ))
        }
    }

    /// Change file ownership
    ///
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
    pub fn realpath(&mut self, path: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: realpath {}", path);
        }

        let host_path = self.resolve_guest_path(path)?;

        // Canonicalize to resolve symlinks and relative paths
        let canonical = fs::canonicalize(&host_path)
            .map_err(|e| Error::NotFound(format!("Failed to resolve path {}: {}", path, e)))?;

        // Convert back to guest path by stripping the mount root prefix
        let root_mountpoint = self
            .mounted
            .get("/dev/sda1")
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
    pub fn cp(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.trace {
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
                src,
                dest,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Copy recursively
    ///
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
                src,
                dest,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Move/rename file
    ///
    pub fn mv(&mut self, src: &str, dest: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: mv {} {}", src, dest);
        }

        let src_path = self.resolve_guest_path(src)?;
        let dest_path = self.resolve_guest_path(dest)?;

        fs::rename(&src_path, &dest_path)
            .map_err(|e| Error::CommandFailed(format!("Failed to move {} to {}: {}", src, dest, e)))
    }

    /// Download file from guest to host
    ///
    pub fn download(&mut self, remotefilename: &str, filename: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: download {} {}", remotefilename, filename);
        }

        let guest_path = self.resolve_guest_path(remotefilename)?;
        let host_path = Path::new(filename);

        fs::copy(&guest_path, host_path).map_err(|e| {
            Error::CommandFailed(format!(
                "Failed to download {} to {}: {}",
                remotefilename, filename, e
            ))
        })?;

        Ok(())
    }

    /// Upload file from host to guest
    ///
    pub fn upload(&mut self, filename: &str, remotefilename: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: upload {} {}", filename, remotefilename);
        }

        let host_path = Path::new(filename);
        let guest_path = self.resolve_guest_path(remotefilename)?;

        fs::copy(host_path, &guest_path).map_err(|e| {
            Error::CommandFailed(format!(
                "Failed to upload {} to {}: {}",
                filename, remotefilename, e
            ))
        })?;

        Ok(())
    }

    /// Append content to file
    ///
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
            .map_err(|e| {
                Error::CommandFailed(format!("Failed to open {} for append: {}", path, e))
            })?;

        file.write_all(content)
            .map_err(|e| Error::CommandFailed(format!("Failed to append to {}: {}", path, e)))?;

        Ok(())
    }

    /// Search file for pattern
    ///
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
            return Err(Error::InvalidOperation(format!(
                "Cannot rm directory (use rmdir or rm_rf): {}",
                path
            )));
        }

        fs::remove_file(&host_path).map_err(Error::Io)
    }

    /// Remove a file or directory recursively (force)
    ///
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
            fs::remove_dir_all(&host_path).map_err(Error::Io)
        } else {
            fs::remove_file(&host_path).map_err(Error::Io)
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
