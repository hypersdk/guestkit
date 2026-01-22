// SPDX-License-Identifier: LGPL-3.0-or-later
//! Guest OS detection using libguestfs

use crate::core::{GuestIdentity, Result};
use std::path::Path;

#[cfg(feature = "ffi-bindings")]
use crate::ffi::Guestfs;

/// Guest OS detector
pub struct GuestDetector {}

impl GuestDetector {
    /// Create a new guest detector
    pub fn new() -> Self {
        Self {}
    }

    /// Detect guest OS from disk image
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestkit::detectors::GuestDetector;
    /// use std::path::Path;
    ///
    /// let detector = GuestDetector::new();
    /// let guest = detector.detect_from_image(Path::new("/path/to/disk.qcow2")).unwrap();
    /// println!("OS: {}", guest.os_name);
    /// ```
    #[cfg(feature = "ffi-bindings")]
    pub fn detect_from_image<P: AsRef<Path>>(&self, path: P) -> Result<GuestIdentity> {
        let g = Guestfs::new()?;
        g.inspect_image(path)
    }

    /// Detect guest OS (stub when FFI bindings not available)
    #[cfg(not(feature = "ffi-bindings"))]
    pub fn detect_from_image<P: AsRef<Path>>(&self, _path: P) -> Result<GuestIdentity> {
        Err(crate::core::Error::Unsupported(
            "Guest detection requires ffi-bindings feature".to_string()
        ))
    }
}

impl Default for GuestDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = GuestDetector::new();
        let _ = detector;
    }
}
