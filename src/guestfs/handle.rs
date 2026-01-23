// SPDX-License-Identifier: LGPL-3.0-or-later
//! Main GuestFS handle implementation

use crate::core::{Error, Result};
use crate::disk::{DiskReader, PartitionTable, NbdDevice};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::Duration;

/// GuestFS handle state
#[derive(Debug, PartialEq)]
pub enum GuestfsState {
    /// Initial state after creation
    Config,
    /// Between Config and Ready during launch
    Launching,
    /// After launch() called successfully
    Ready,
    /// Error state with error message
    Error(String),
    /// After shutdown() called
    Closed,
}

/// UTF-8 handling policy
#[derive(Debug, Clone, PartialEq)]
pub enum Utf8Policy {
    /// Return error on invalid UTF-8
    Strict,
    /// Replace invalid UTF-8 with U+FFFD (default for backward compatibility)
    Lossy,
}

/// Resource limits for safe operation
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum file size in bytes (default: 100MB)
    pub max_file_size: Option<u64>,
    /// Operation timeout in seconds (default: 300s)
    pub operation_timeout: Option<Duration>,
    /// Maximum path length (default: 4096)
    pub max_path_length: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            operation_timeout: Some(Duration::from_secs(300)), // 5 minutes
            max_path_length: 4096,
        }
    }
}

/// Main GuestFS handle
pub struct Guestfs {
    pub(crate) state: GuestfsState,
    pub(crate) verbose: bool,
    pub(crate) trace: bool,
    pub(crate) readonly: bool,
    pub(crate) drives: Vec<DriveConfig>,
    pub(crate) reader: Option<DiskReader>,
    pub(crate) partition_table: Option<PartitionTable>,
    pub(crate) nbd_device: Option<NbdDevice>,
    pub(crate) mounted: HashMap<String, String>, // device -> mountpoint
    pub(crate) mount_root: Option<PathBuf>, // Temporary mount directory
    pub(crate) identifier: Option<String>,
    pub(crate) autosync: bool,
    pub(crate) selinux: bool,
    pub(crate) utf8_policy: Utf8Policy,
    pub(crate) resource_limits: ResourceLimits,
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
            readonly: false,
            drives: Vec::new(),
            reader: None,
            partition_table: None,
            nbd_device: None,
            mounted: HashMap::new(),
            mount_root: None,
            identifier: None,
            autosync: true,
            selinux: false,
            utf8_policy: Utf8Policy::Lossy,
            resource_limits: ResourceLimits::default(),
        })
    }

    /// Create a new GuestFS handle (libguestfs compatibility alias)
    ///
    /// Compatible with libguestfs g.create()
    pub fn create() -> Result<Self> {
        Self::new()
    }

    /// Add a drive in read-write mode
    ///
    /// Compatible with libguestfs g.add_drive()
    pub fn add_drive<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.add_drive_opts(path, false, None)
    }

    /// Add a drive in read-only mode
    ///
    /// Compatible with libguestfs g.add_drive_ro()
    pub fn add_drive_ro<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.add_drive_opts(path, true, None)
    }

    /// Add a drive with options
    ///
    /// Compatible with libguestfs g.add_drive_opts()
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
            return Err(Error::InvalidState(format!(
                "Cannot launch from state: {:?}", self.state
            )));
        }

        if self.drives.is_empty() {
            return Err(Error::InvalidState(
                "No drives added".to_string()
            ));
        }

        // Transition to Launching state
        self.state = GuestfsState::Launching;

        // Open the first drive (for now, multi-drive support TODO)
        let drive = &self.drives[0];

        // Attempt to launch - if any error occurs, move to Error state
        let result: Result<()> = (|| {
            let reader = DiskReader::open(&drive.path)?;
            let partition_table = PartitionTable::parse(&mut DiskReader::open(&drive.path)?)?;

            self.reader = Some(reader);
            self.partition_table = Some(partition_table);

            Ok(())
        })();

        match result {
            Ok(_) => {
                self.state = GuestfsState::Ready;

                if self.verbose {
                    eprintln!("guestfs: launched with {} drive(s)", self.drives.len());
                }

                Ok(())
            }
            Err(e) => {
                self.state = GuestfsState::Error(e.to_string());
                Err(e)
            }
        }
    }

    /// Shutdown the guestfs handle
    pub fn shutdown(&mut self) -> Result<()> {
        if self.state == GuestfsState::Closed {
            return Ok(());
        }

        if self.verbose {
            eprintln!("guestfs: shutdown");
        }

        // Unmount all filesystems
        let _ = self.umount_all();

        // Disconnect NBD device
        if let Some(mut nbd) = self.nbd_device.take() {
            let _ = nbd.disconnect();
        }

        // Clean up mount root
        if let Some(mount_root) = self.mount_root.take() {
            let _ = std::fs::remove_dir_all(&mount_root);
        }

        // Clean up resources
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

    /// Get NBD device reference safely (internal)
    pub(crate) fn nbd_device(&self) -> Result<&NbdDevice> {
        self.nbd_device.as_ref()
            .ok_or_else(|| Error::InvalidState(
                "NBD device not initialized. Call setup_nbd_if_needed() first.".to_string()
            ))
    }

    /// Get mutable NBD device reference safely (internal)
    pub(crate) fn nbd_device_mut(&mut self) -> Result<&mut NbdDevice> {
        self.nbd_device.as_mut()
            .ok_or_else(|| Error::InvalidState(
                "NBD device not initialized. Call setup_nbd_if_needed() first.".to_string()
            ))
    }

    /// Convert path to string safely (internal)
    pub(crate) fn path_to_string(path: &Path) -> Result<String> {
        path.to_str()
            .ok_or_else(|| Error::InvalidFormat(
                format!("Path contains invalid Unicode: {:?}", path)
            ))
            .map(|s| s.to_string())
    }

    /// Set UTF-8 handling policy
    pub fn set_utf8_policy(&mut self, policy: Utf8Policy) {
        self.utf8_policy = policy;
    }

    /// Get current UTF-8 handling policy
    pub fn get_utf8_policy(&self) -> &Utf8Policy {
        &self.utf8_policy
    }

    /// Decode bytes to UTF-8 string according to policy (internal)
    pub(crate) fn decode_utf8(&self, bytes: &[u8]) -> Result<String> {
        match self.utf8_policy {
            Utf8Policy::Strict => String::from_utf8(bytes.to_vec())
                .map_err(|e| Error::InvalidFormat(format!("Invalid UTF-8: {}", e))),
            Utf8Policy::Lossy => Ok(String::from_utf8_lossy(bytes).to_string()),
        }
    }

    /// Set resource limits
    pub fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.resource_limits = limits;
    }

    /// Get current resource limits
    pub fn get_resource_limits(&self) -> &ResourceLimits {
        &self.resource_limits
    }

    /// Check if file size is within limits (internal)
    pub(crate) fn check_file_size_limit(&self, size: u64) -> Result<()> {
        if let Some(max) = self.resource_limits.max_file_size {
            if size > max {
                return Err(Error::InvalidOperation(format!(
                    "File size {} exceeds limit {} bytes",
                    size, max
                )));
            }
        }
        Ok(())
    }

    /// Check if path length is within limits (internal)
    pub(crate) fn check_path_length_limit(&self, path: &str) -> Result<()> {
        if path.len() > self.resource_limits.max_path_length {
            return Err(Error::InvalidOperation(format!(
                "Path length {} exceeds limit {}",
                path.len(), self.resource_limits.max_path_length
            )));
        }
        Ok(())
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
    ///
    /// Supports multiple device patterns:
    /// - /dev/sda, /dev/sda1, /dev/sda2, ...
    /// - /dev/vda, /dev/vda1, /dev/vda2, ...
    /// - /dev/hda, /dev/hda1, /dev/hda2, ...
    /// - /dev/xvda, /dev/xvda1, /dev/xvda2, ...
    /// - /dev/nvme0n1p1, /dev/nvme0n1p2, ...
    pub(crate) fn parse_device_name(&self, device: &str) -> Result<u32> {
        // Validate device path length
        if device.len() > 255 || !device.starts_with("/dev/") {
            return Err(Error::InvalidFormat(format!("Invalid device: {}", device)));
        }

        // Support multiple device patterns
        let patterns = [
            "/dev/sda",
            "/dev/vda",
            "/dev/hda",
            "/dev/xvda",
            "/dev/nvme0n1p",
        ];

        for prefix in &patterns {
            if let Some(num_str) = device.strip_prefix(prefix) {
                if num_str.is_empty() {
                    return Ok(0); // Whole device (no partition number)
                }
                return num_str.parse::<u32>()
                    .map_err(|_| Error::InvalidFormat(format!("Invalid device: {}", device)));
            }
        }

        Err(Error::InvalidFormat(format!(
            "Unsupported device pattern: {}. Supported: /dev/{{sd,vd,hd,xvd}}a*, /dev/nvme0n1p*",
            device
        )))
    }

    /// Set up NBD device if not already set up (internal helper)
    pub(crate) fn setup_nbd_if_needed(&mut self) -> Result<()> {
        if self.nbd_device.is_some() {
            return Ok(());
        }

        // Get first drive
        let drive = self.drives.first().ok_or_else(|| {
            Error::InvalidState("No drives added".to_string())
        })?;

        // Create and connect NBD device
        let mut nbd = NbdDevice::new()?;
        nbd.connect(&drive.path, drive.readonly)?;

        self.nbd_device = Some(nbd);

        if self.verbose {
            eprintln!("guestfs: NBD device connected");
        }

        Ok(())
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
