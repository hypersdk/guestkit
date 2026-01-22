// SPDX-License-Identifier: LGPL-3.0-or-later
//! Main GuestFS handle implementation

use crate::core::{Error, Result};
use crate::disk::{DiskReader, PartitionTable};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// GuestFS handle state
#[derive(Debug, PartialEq)]
pub enum GuestfsState {
    /// Initial state after creation
    Config,
    /// After launch() called
    Ready,
    /// After shutdown() called
    Closed,
}

/// Main GuestFS handle
pub struct Guestfs {
    state: GuestfsState,
    pub(crate) verbose: bool,
    pub(crate) trace: bool,
    pub(crate) drives: Vec<DriveConfig>,
    pub(crate) reader: Option<DiskReader>,
    pub(crate) partition_table: Option<PartitionTable>,
    pub(crate) mounted: HashMap<String, String>, // device -> mountpoint
}

/// Drive configuration
#[derive(Debug, Clone)]
pub struct DriveConfig {
    pub path: PathBuf,
    pub readonly: bool,
    pub format: Option<String>,
}

impl Guestfs {
    /// Create a new GuestFS handle
    ///
    /// # Examples
    ///
    /// ```
    /// use guestkit::guestfs::Guestfs;
    ///
    /// let g = Guestfs::new().unwrap();
    /// ```
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: GuestfsState::Config,
            verbose: false,
            trace: false,
            drives: Vec::new(),
            reader: None,
            partition_table: None,
            mounted: HashMap::new(),
        })
    }

    /// Add a drive in read-only mode
    pub fn add_drive_ro<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.add_drive_opts(path, false, None)
    }

    /// Add a drive with options
    pub fn add_drive_opts<P: AsRef<Path>>(
        &mut self,
        path: P,
        readonly: bool,
        format: Option<&str>,
    ) -> Result<()> {
        if self.state != GuestfsState::Config {
            return Err(Error::InvalidState(
                "Cannot add drives after launch".to_string()
            ));
        }

        self.drives.push(DriveConfig {
            path: path.as_ref().to_path_buf(),
            readonly,
            format: format.map(|s| s.to_string()),
        });

        Ok(())
    }

    /// Launch the guestfs handle (prepare for operations)
    pub fn launch(&mut self) -> Result<()> {
        if self.state != GuestfsState::Config {
            return Err(Error::InvalidState(
                "Already launched or closed".to_string()
            ));
        }

        if self.drives.is_empty() {
            return Err(Error::InvalidState(
                "No drives added".to_string()
            ));
        }

        // Open the first drive (for now, multi-drive support TODO)
        let drive = &self.drives[0];
        let reader = DiskReader::open(&drive.path)?;
        let partition_table = PartitionTable::parse(&mut DiskReader::open(&drive.path)?)?;

        self.reader = Some(reader);
        self.partition_table = Some(partition_table);
        self.state = GuestfsState::Ready;

        if self.verbose {
            eprintln!("guestfs: launched with {} drive(s)", self.drives.len());
        }

        Ok(())
    }

    /// Shutdown the guestfs handle
    pub fn shutdown(&mut self) -> Result<()> {
        if self.state == GuestfsState::Closed {
            return Ok(());
        }

        // Unmount all
        self.mounted.clear();
        self.reader = None;
        self.partition_table = None;
        self.state = GuestfsState::Closed;

        if self.verbose {
            eprintln!("guestfs: shutdown complete");
        }

        Ok(())
    }

    /// Close the handle (same as shutdown)
    pub fn close(&mut self) -> Result<()> {
        self.shutdown()
    }

    /// Set verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Get verbose mode
    pub fn get_verbose(&self) -> bool {
        self.verbose
    }

    /// Set trace mode
    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    /// Get trace mode
    pub fn get_trace(&self) -> bool {
        self.trace
    }

    /// Get current state
    pub fn state(&self) -> &GuestfsState {
        &self.state
    }

    /// Get reader reference (internal)
    pub(crate) fn reader_mut(&mut self) -> Result<&mut DiskReader> {
        self.reader.as_mut()
            .ok_or_else(|| Error::InvalidState("Not launched".to_string()))
    }

    /// Get partition table reference (internal)
    pub(crate) fn partition_table(&self) -> Result<&PartitionTable> {
        self.partition_table.as_ref()
            .ok_or_else(|| Error::InvalidState("Not launched".to_string()))
    }

    /// Check if ready for operations
    pub(crate) fn ensure_ready(&self) -> Result<()> {
        if self.state != GuestfsState::Ready {
            return Err(Error::InvalidState(
                "Handle not ready (call launch first)".to_string()
            ));
        }
        Ok(())
    }

    /// Parse device name to partition number
    pub(crate) fn parse_device_name(&self, device: &str) -> Result<u32> {
        // Parse /dev/sdaX format
        if let Some(num_str) = device.strip_prefix("/dev/sda") {
            num_str.parse::<u32>()
                .map_err(|_| Error::InvalidFormat(format!("Invalid device name: {}", device)))
        } else if let Some(num_str) = device.strip_prefix("/dev/vda") {
            num_str.parse::<u32>()
                .map_err(|_| Error::InvalidFormat(format!("Invalid device name: {}", device)))
        } else {
            Err(Error::InvalidFormat(format!("Invalid device name: {}", device)))
        }
    }
}

impl Drop for Guestfs {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guestfs_creation() {
        let g = Guestfs::new().unwrap();
        assert_eq!(g.state(), &GuestfsState::Config);
    }

    #[test]
    fn test_guestfs_verbose() {
        let mut g = Guestfs::new().unwrap();
        assert_eq!(g.get_verbose(), false);
        g.set_verbose(true);
        assert_eq!(g.get_verbose(), true);
    }

    #[test]
    fn test_guestfs_trace() {
        let mut g = Guestfs::new().unwrap();
        assert_eq!(g.get_trace(), false);
        g.set_trace(true);
        assert_eq!(g.get_trace(), true);
    }
}
