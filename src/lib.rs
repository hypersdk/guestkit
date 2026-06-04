// SPDX-License-Identifier: LGPL-3.0-or-later
//! # guestkit
//!
//! A Guest VM toolkit for disk inspection and manipulation.
//!
//! ## Features
//!
//! - **Disk format conversion** - Convert between VMDK, qcow2, RAW, VHD, VDI
//! - **Guest OS detection** - Identify guest operating systems
//! - **Retry logic** - Built-in exponential backoff for reliable operations
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
//! - `export` - Report generation in various formats (HTML with Chart.js, PDF, Markdown)
//! - `guestfs` - GuestFS-compatible API for disk inspection and manipulation
//! - `detectors` - Guest OS detection
//! - `fixers` - Guest OS repair operations
//! - `cli` - Command-line interface

pub mod cli;
pub mod boot;
pub mod converters;
pub mod core;
pub mod disk;
pub mod evidence;
pub mod export;
pub mod fleet;
pub mod guestfs;
pub mod inference;
pub mod storage;

// Optional modules
#[cfg(feature = "guest-inspect")]
pub mod detectors;

#[cfg(feature = "python-bindings")]
pub mod python;

#[cfg(feature = "agent")]
pub mod agent;

// Re-exports for convenience
pub use converters::DiskConverter;
pub use core::types::*;
pub use core::{Error, Result, RetryConfig};
pub use disk::{DiskReader, FileSystem, PartitionTable};
pub use export::{
    create_variable_map, HtmlExporter, HtmlExportOptions, PaperSize, PdfExporter,
    PdfExportOptions, TemplateEngine, TemplateFormat, TemplateLevel,
};
pub use guestfs::Guestfs;
pub use evidence::EvidenceSnapshot;
pub use boot::BootabilityReport;

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
