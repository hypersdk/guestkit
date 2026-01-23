// SPDX-License-Identifier: LGPL-3.0-or-later
//! CPIO archive operations compatible with libguestfs
//!
//! This implementation provides CPIO archive creation and extraction.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;
use std::process::Command;

impl Guestfs {
    /// Extract CPIO archive to directory
    ///
    /// Additional functionality for CPIO support
    pub fn cpio_extract(&mut self, archive: &str, directory: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cpio_extract {} {}", archive, directory);
        }

        let host_dir = self.resolve_guest_path(directory)?;

        let archive_data = std::fs::read(archive)
            .map_err(|e| Error::Io(e))?;

        let mut cmd = Command::new("cpio");
        cmd.arg("-idm")
           .arg("-D")
           .arg(&host_dir)
           .stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute cpio: {}", e)))?;

        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&archive_data)
                .map_err(|e| Error::Io(e))?;
        }

        let output = child.wait_with_output()
            .map_err(|e| Error::CommandFailed(format!("Failed to wait for cpio: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "cpio extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Create CPIO archive from directory
    ///
    /// Additional functionality for CPIO support
    pub fn cpio_create(&mut self, directory: &str, archive: &str, format: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cpio_create {} {} {}", directory, archive, format);
        }

        let host_dir = self.resolve_guest_path(directory)?;

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
           .arg(format!(
               "cd {} && find . -print | cpio -o -H {} > {}",
               host_dir.display(),
               format,
               archive
           ));

        let output = cmd.output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute cpio: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "cpio creation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// List contents of CPIO archive
    ///
    /// Additional functionality for CPIO support
    pub fn cpio_list(&mut self, archive: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: cpio_list {}", archive);
        }

        let archive_data = std::fs::read(archive)
            .map_err(|e| Error::Io(e))?;

        let mut cmd = Command::new("cpio");
        cmd.arg("-t")
           .stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute cpio: {}", e)))?;

        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&archive_data)
                .map_err(|e| Error::Io(e))?;
        }

        let output = child.wait_with_output()
            .map_err(|e| Error::CommandFailed(format!("Failed to wait for cpio: {}", e)))?;

        if !output.status.success() {
            return Err(Error::CommandFailed(format!(
                "cpio list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = output_str
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpio_ops_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
