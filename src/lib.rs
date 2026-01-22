// SPDX-License-Identifier: LGPL-3.0-or-later
//! # guestkit
//!
//! A Guest VM toolkit for disk inspection and manipulation, inspired by libguestfs.
//!
//! ## Features
//!
//! - **Disk format conversion** - Convert between VMDK, qcow2, RAW, VHD, VDI
//! - **Guest OS detection** - Identify guest operating systems
//! - **Retry logic** - Built-in exponential backoff for reliable operations
//! - **Pipeline orchestration** - Multi-stage migration pipelines
//!
//! ## Quick Start
//!
//! ```no_run
//! use guestkit::converters::DiskConverter;
//! use std::path::Path;
//!
//! let converter = DiskConverter::new();
//! let result = converter.convert(
//!     Path::new("/path/to/source.vmdk"),
//!     Path::new("/path/to/output.qcow2"),
//!     "qcow2",
//!     true,  // compress
//!     true,  // flatten
//! ).unwrap();
//!
//! if result.success {
//!     println!("Conversion successful!");
//!     println!("Output size: {} bytes", result.output_size);
//! }
//! ```
//!
//! ## Architecture
//!
//! guestkit is organized into focused modules:
//!
//! - `core` - Error types, retry logic, common types
//! - `converters` - Disk format conversion
//! - `detectors` - Guest OS detection
//! - `fixers` - Guest OS repair operations
//! - `orchestrator` - Pipeline orchestration
//! - `cli` - Command-line interface
//! - `ffi` - FFI bindings to libguestfs (optional)

pub mod core;
pub mod converters;

// Optional modules
#[cfg(feature = "guest-inspect")]
pub mod detectors;

#[cfg(feature = "disk-ops")]
pub mod fixers;

pub mod orchestrator;

#[cfg(feature = "ffi-bindings")]
pub mod ffi;

#[cfg(feature = "python-bindings")]
pub mod python;

// Re-exports for convenience
pub use core::{Error, Result, RetryConfig};
pub use core::types::*;
pub use converters::DiskConverter;

#[cfg(feature = "guest-inspect")]
pub use detectors::GuestDetector;

#[cfg(feature = "ffi-bindings")]
pub use ffi::Guestfs;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
