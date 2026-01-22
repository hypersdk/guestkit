// SPDX-License-Identifier: LGPL-3.0-or-later
//! Archive operations (tar, tgz, cpio)
//!
//! This implementation uses system tar command with mounted filesystems.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::path::Path;
use std::process::Command;

impl Guestfs {
    /// Extract tar archive into directory
    ///
    /// Compatible with libguestfs g.tar_in()
    ///
    /// # Arguments
    ///
    /// * `tarfile` - Path to tar archive on host
    /// * `directory` - Directory in guest to extract to
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestkit::guestfs::Guestfs;
    ///
    /// let mut g = Guestfs::new().unwrap();
    /// // ... setup and mount ...
    ///
    /// g.tar_in("/path/to/archive.tar", "/target/dir").unwrap();
    /// ```
    pub fn tar_in<P: AsRef<Path>>(&mut self, tarfile: P, directory: &str) -> Result<()> {
        self.ensure_ready()?;

        let tarfile = tarfile.as_ref();

        if self.verbose {
            eprintln!("guestfs: tar_in {} {}", tarfile.display(), directory);
        }

        // Verify tar file exists
        if !tarfile.exists() {
            return Err(Error::NotFound(format!("Tar file not found: {}", tarfile.display())));
        }

        // Get root mount point
        let root_mountpoint = self.mounted.get("/dev/sda1")
            .or_else(|| self.mounted.get("/dev/sda2"))
            .or_else(|| self.mounted.get("/dev/vda1"))
            .or_else(|| self.mounted.values().next())
            .ok_or_else(|| Error::InvalidState(
                "No filesystem mounted. Call mount_ro() first.".to_string()
            ))?;

        // Build target directory path
        let directory_clean = directory.trim_start_matches('/');
        let target_path = std::path::PathBuf::from(root_mountpoint).join(directory_clean);

        // Ensure target directory exists
        std::fs::create_dir_all(&target_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to create target directory: {}", e))
        })?;

        // Extract tar
        let output = Command::new("tar")
            .arg("-xf")
            .arg(tarfile)
            .arg("-C")
            .arg(&target_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run tar: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Tar extraction failed: {}", stderr)));
        }

        Ok(())
    }

    /// Create tar archive from directory
    ///
    /// Compatible with libguestfs g.tar_out()
    pub fn tar_out<P: AsRef<Path>>(&mut self, directory: &str, tarfile: P) -> Result<()> {
        self.ensure_ready()?;

        let tarfile = tarfile.as_ref();

        if self.verbose {
            eprintln!("guestfs: tar_out {} {}", directory, tarfile.display());
        }

        // Get root mount point
        let root_mountpoint = self.mounted.get("/dev/sda1")
            .or_else(|| self.mounted.get("/dev/sda2"))
            .or_else(|| self.mounted.get("/dev/vda1"))
            .or_else(|| self.mounted.values().next())
            .ok_or_else(|| Error::InvalidState(
                "No filesystem mounted. Call mount_ro() first.".to_string()
            ))?;

        // Build source directory path
        let directory_clean = directory.trim_start_matches('/');
        let source_path = std::path::PathBuf::from(root_mountpoint).join(directory_clean);

        // Verify source exists
        if !source_path.exists() {
            return Err(Error::NotFound(format!("Directory not found: {}", directory)));
        }

        // Create tar archive
        let output = Command::new("tar")
            .arg("-cf")
            .arg(tarfile)
            .arg("-C")
            .arg(&source_path)
            .arg(".")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run tar: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Tar creation failed: {}", stderr)));
        }

        Ok(())
    }

    /// Extract compressed tar archive
    ///
    /// Compatible with libguestfs g.tgz_in()
    pub fn tgz_in<P: AsRef<Path>>(&mut self, tarball: P, directory: &str) -> Result<()> {
        self.ensure_ready()?;

        let tarball = tarball.as_ref();

        if self.verbose {
            eprintln!("guestfs: tgz_in {} {}", tarball.display(), directory);
        }

        // Verify tar file exists
        if !tarball.exists() {
            return Err(Error::NotFound(format!("Tar file not found: {}", tarball.display())));
        }

        // Get root mount point
        let root_mountpoint = self.mounted.get("/dev/sda1")
            .or_else(|| self.mounted.get("/dev/sda2"))
            .or_else(|| self.mounted.get("/dev/vda1"))
            .or_else(|| self.mounted.values().next())
            .ok_or_else(|| Error::InvalidState(
                "No filesystem mounted. Call mount_ro() first.".to_string()
            ))?;

        // Build target directory path
        let directory_clean = directory.trim_start_matches('/');
        let target_path = std::path::PathBuf::from(root_mountpoint).join(directory_clean);

        // Ensure target directory exists
        std::fs::create_dir_all(&target_path).map_err(|e| {
            Error::CommandFailed(format!("Failed to create target directory: {}", e))
        })?;

        // Extract compressed tar
        let output = Command::new("tar")
            .arg("-xzf")
            .arg(tarball)
            .arg("-C")
            .arg(&target_path)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run tar: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Tar extraction failed: {}", stderr)));
        }

        Ok(())
    }

    /// Create compressed tar archive
    ///
    /// Compatible with libguestfs g.tgz_out()
    pub fn tgz_out<P: AsRef<Path>>(&mut self, directory: &str, tarball: P) -> Result<()> {
        self.ensure_ready()?;

        let tarball = tarball.as_ref();

        if self.verbose {
            eprintln!("guestfs: tgz_out {} {}", directory, tarball.display());
        }

        // Get root mount point
        let root_mountpoint = self.mounted.get("/dev/sda1")
            .or_else(|| self.mounted.get("/dev/sda2"))
            .or_else(|| self.mounted.get("/dev/vda1"))
            .or_else(|| self.mounted.values().next())
            .ok_or_else(|| Error::InvalidState(
                "No filesystem mounted. Call mount_ro() first.".to_string()
            ))?;

        // Build source directory path
        let directory_clean = directory.trim_start_matches('/');
        let source_path = std::path::PathBuf::from(root_mountpoint).join(directory_clean);

        // Verify source exists
        if !source_path.exists() {
            return Err(Error::NotFound(format!("Directory not found: {}", directory)));
        }

        // Create compressed tar archive
        let output = Command::new("tar")
            .arg("-czf")
            .arg(tarball)
            .arg("-C")
            .arg(&source_path)
            .arg(".")
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run tar: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Tar creation failed: {}", stderr)));
        }

        Ok(())
    }

    /// Extract tar with options
    ///
    /// Compatible with libguestfs g.tar_in_opts()
    pub fn tar_in_opts<P: AsRef<Path>>(&mut self, tarfile: P, directory: &str, compress: Option<&str>, xattrs: bool, selinux: bool, acls: bool) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: tar_in_opts {} {} compress={:?} xattrs={} selinux={} acls={}",
                tarfile.as_ref().display(), directory, compress, xattrs, selinux, acls);
        }

        // TODO: Implement with options
        Err(Error::Unsupported(
            "Archive extraction with options requires implementation".to_string()
        ))
    }

    /// Create tar with options
    ///
    /// Compatible with libguestfs g.tar_out_opts()
    pub fn tar_out_opts<P: AsRef<Path>>(&mut self, directory: &str, tarfile: P, compress: Option<&str>, numericowner: bool, xattrs: bool, selinux: bool, acls: bool) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: tar_out_opts {} {} compress={:?} xattrs={} selinux={} acls={}",
                directory, tarfile.as_ref().display(), compress, xattrs, selinux, acls);
        }

        // TODO: Implement with options
        Err(Error::Unsupported(
            "Archive creation with options requires implementation".to_string()
        ))
    }

    /// Create cpio archive
    ///
    /// Compatible with libguestfs g.cpio_out()
    pub fn cpio_out<P: AsRef<Path>>(&mut self, directory: &str, cpiofile: P, format: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cpio_out {} {} {}", directory, cpiofile.as_ref().display(), format);
        }

        // TODO: Implement cpio creation
        Err(Error::Unsupported(
            "CPIO archive creation requires implementation".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_api_exists() {
        let mut g = Guestfs::new().unwrap();
        let _ = g;
    }
}
