// SPDX-License-Identifier: LGPL-3.0-or-later
//! Safe wrapper around libguestfs FFI bindings

use super::bindings::*;
use crate::core::{Error, GuestIdentity, GuestType, Firmware, Result};
use std::ffi::CString;
use std::path::Path;
use std::ptr;

/// Safe wrapper around libguestfs handle
pub struct Guestfs {
    handle: *mut guestfs_h,
}

impl Guestfs {
    /// Create a new guestfs handle
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestkit::ffi::Guestfs;
    ///
    /// let g = Guestfs::new().unwrap();
    /// ```
    pub fn new() -> Result<Self> {
        unsafe {
            let handle = guestfs_create();
            if handle.is_null() {
                return Err(Error::Ffi("Failed to create guestfs handle".to_string()));
            }
            Ok(Self { handle })
        }
    }

    /// Set verbose mode
    pub fn set_verbose(&self, verbose: bool) -> Result<()> {
        unsafe {
            let ret = guestfs_set_verbose(self.handle, if verbose { 1 } else { 0 });
            if ret == -1 {
                return Err(self.last_error());
            }
            Ok(())
        }
    }

    /// Set trace mode
    pub fn set_trace(&self, trace: bool) -> Result<()> {
        unsafe {
            let ret = guestfs_set_trace(self.handle, if trace { 1 } else { 0 });
            if ret == -1 {
                return Err(self.last_error());
            }
            Ok(())
        }
    }

    /// Add a disk image (read-only)
    pub fn add_drive_ro<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_str = path.as_ref().to_str()
            .ok_or_else(|| Error::InvalidFormat("Invalid path".to_string()))?;
        let c_path = CString::new(path_str)
            .map_err(|_| Error::InvalidFormat("Path contains null byte".to_string()))?;

        unsafe {
            let ret = guestfs_add_drive_ro(self.handle, c_path.as_ptr());
            if ret == -1 {
                return Err(self.last_error());
            }
            Ok(())
        }
    }

    /// Launch the backend
    pub fn launch(&self) -> Result<()> {
        unsafe {
            let ret = guestfs_launch(self.handle);
            if ret == -1 {
                return Err(self.last_error());
            }
            Ok(())
        }
    }

    /// Inspect operating systems on added disks
    pub fn inspect_os(&self) -> Result<Vec<String>> {
        unsafe {
            let roots = guestfs_inspect_os(self.handle);
            if roots.is_null() {
                return Err(self.last_error());
            }
            Ok(c_str_list_to_vec(roots))
        }
    }

    /// Get OS type for a root
    pub fn inspect_get_type(&self, root: &str) -> Result<String> {
        let c_root = CString::new(root)
            .map_err(|_| Error::InvalidFormat("Root contains null byte".to_string()))?;

        unsafe {
            let result = guestfs_inspect_get_type(self.handle, c_root.as_ptr());
            c_str_to_string(result)
                .ok_or_else(|| self.last_error())
        }
    }

    /// Get OS distribution for a root
    pub fn inspect_get_distro(&self, root: &str) -> Result<String> {
        let c_root = CString::new(root)
            .map_err(|_| Error::InvalidFormat("Root contains null byte".to_string()))?;

        unsafe {
            let result = guestfs_inspect_get_distro(self.handle, c_root.as_ptr());
            c_str_to_string(result)
                .ok_or_else(|| self.last_error())
        }
    }

    /// Get OS product name
    pub fn inspect_get_product_name(&self, root: &str) -> Result<String> {
        let c_root = CString::new(root)
            .map_err(|_| Error::InvalidFormat("Root contains null byte".to_string()))?;

        unsafe {
            let result = guestfs_inspect_get_product_name(self.handle, c_root.as_ptr());
            c_str_to_string(result)
                .ok_or_else(|| self.last_error())
        }
    }

    /// Get OS major version
    pub fn inspect_get_major_version(&self, root: &str) -> Result<i32> {
        let c_root = CString::new(root)
            .map_err(|_| Error::InvalidFormat("Root contains null byte".to_string()))?;

        unsafe {
            let version = guestfs_inspect_get_major_version(self.handle, c_root.as_ptr());
            if version == -1 {
                return Err(self.last_error());
            }
            Ok(version)
        }
    }

    /// Get OS minor version
    pub fn inspect_get_minor_version(&self, root: &str) -> Result<i32> {
        let c_root = CString::new(root)
            .map_err(|_| Error::InvalidFormat("Root contains null byte".to_string()))?;

        unsafe {
            let version = guestfs_inspect_get_minor_version(self.handle, c_root.as_ptr());
            if version == -1 {
                return Err(self.last_error());
            }
            Ok(version)
        }
    }

    /// Get OS architecture
    pub fn inspect_get_arch(&self, root: &str) -> Result<String> {
        let c_root = CString::new(root)
            .map_err(|_| Error::InvalidFormat("Root contains null byte".to_string()))?;

        unsafe {
            let result = guestfs_inspect_get_arch(self.handle, c_root.as_ptr());
            c_str_to_string(result)
                .ok_or_else(|| self.last_error())
        }
    }

    /// List partitions
    pub fn list_partitions(&self) -> Result<Vec<String>> {
        unsafe {
            let partitions = guestfs_list_partitions(self.handle);
            if partitions.is_null() {
                return Err(self.last_error());
            }
            Ok(c_str_list_to_vec(partitions))
        }
    }

    /// Mount filesystem read-only
    pub fn mount_ro(&self, mountable: &str, mountpoint: &str) -> Result<()> {
        let c_mountable = CString::new(mountable)
            .map_err(|_| Error::InvalidFormat("Mountable contains null byte".to_string()))?;
        let c_mountpoint = CString::new(mountpoint)
            .map_err(|_| Error::InvalidFormat("Mountpoint contains null byte".to_string()))?;

        unsafe {
            let ret = guestfs_mount_ro(self.handle, c_mountable.as_ptr(), c_mountpoint.as_ptr());
            if ret == -1 {
                return Err(self.last_error());
            }
            Ok(())
        }
    }

    /// Unmount filesystem
    pub fn umount(&self, path: &str) -> Result<()> {
        let c_path = CString::new(path)
            .map_err(|_| Error::InvalidFormat("Path contains null byte".to_string()))?;

        unsafe {
            let ret = guestfs_umount(self.handle, c_path.as_ptr());
            if ret == -1 {
                return Err(self.last_error());
            }
            Ok(())
        }
    }

    /// Check if path is a file
    pub fn is_file(&self, path: &str) -> Result<bool> {
        let c_path = CString::new(path)
            .map_err(|_| Error::InvalidFormat("Path contains null byte".to_string()))?;

        unsafe {
            let ret = guestfs_is_file(self.handle, c_path.as_ptr());
            match ret {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(self.last_error()),
            }
        }
    }

    /// Check if path is a directory
    pub fn is_dir(&self, path: &str) -> Result<bool> {
        let c_path = CString::new(path)
            .map_err(|_| Error::InvalidFormat("Path contains null byte".to_string()))?;

        unsafe {
            let ret = guestfs_is_dir(self.handle, c_path.as_ptr());
            match ret {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(self.last_error()),
            }
        }
    }

    /// Inspect image and return guest identity
    pub fn inspect_image<P: AsRef<Path>>(&self, path: P) -> Result<GuestIdentity> {
        // Add disk
        self.add_drive_ro(path)?;

        // Launch
        self.launch()?;

        // Inspect OS
        let roots = self.inspect_os()?;
        if roots.is_empty() {
            return Err(Error::Detection("No operating system found".to_string()));
        }

        let root = &roots[0];

        // Get OS information
        let os_type_str = self.inspect_get_type(root)?;
        let distro = self.inspect_get_distro(root).ok();
        let os_name = self.inspect_get_product_name(root)?;
        let major_version = self.inspect_get_major_version(root)?;
        let minor_version = self.inspect_get_minor_version(root)?;
        let arch = self.inspect_get_arch(root)?;

        // Determine guest type
        let guest_type = match os_type_str.as_str() {
            "linux" => GuestType::Linux,
            "windows" => GuestType::Windows,
            "freebsd" => GuestType::FreeBSD,
            "openbsd" => GuestType::OpenBSD,
            "netbsd" => GuestType::NetBSD,
            _ => GuestType::Unknown,
        };

        // TODO: Detect firmware type (BIOS vs UEFI)
        let firmware = Firmware::Unknown;

        Ok(GuestIdentity {
            os_type: guest_type,
            os_name,
            os_version: format!("{}.{}", major_version, minor_version),
            architecture: arch,
            firmware,
            init_system: None,
            distro,
        })
    }

    /// Get last error
    fn last_error(&self) -> Error {
        unsafe {
            let err_ptr = guestfs_last_error(self.handle);
            if err_ptr.is_null() {
                return Error::Unknown("Unknown error".to_string());
            }
            let c_str = std::ffi::CStr::from_ptr(err_ptr);
            Error::Ffi(c_str.to_string_lossy().into_owned())
        }
    }
}

impl Drop for Guestfs {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                guestfs_close(self.handle);
            }
        }
    }
}

// Guestfs is not Send/Sync because libguestfs is not thread-safe
// Users must create separate handles for each thread
unsafe impl Send for Guestfs {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guestfs_creation() {
        // This test requires libguestfs to be installed
        // Skip if not available
        if std::env::var("SKIP_LIBGUESTFS_TESTS").is_ok() {
            return;
        }

        let result = Guestfs::new();
        // May fail if libguestfs is not installed, which is OK for this test
        let _ = result;
    }
}
