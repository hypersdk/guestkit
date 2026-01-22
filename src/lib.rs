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
//! - `disk` - Pure Rust disk image, partition, and filesystem handling
//! - `detectors` - Guest OS detection
//! - `fixers` - Guest OS repair operations
//! - `orchestrator` - Pipeline orchestration
//! - `cli` - Command-line interface

pub mod core;
pub mod converters;
pub mod disk;

// Optional modules
#[cfg(feature = "guest-inspect")]
pub mod detectors;

#[cfg(feature = "disk-ops")]
pub mod fixers;

pub mod orchestrator;

#[cfg(feature = "python-bindings")]
pub mod python;

// Re-exports for convenience
pub use core::{Error, Result, RetryConfig};
pub use core::types::*;
pub use converters::DiskConverter;
pub use disk::{DiskReader, PartitionTable, FileSystem};

#[cfg(feature = "guest-inspect")]
pub use detectors::GuestDetector;

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
