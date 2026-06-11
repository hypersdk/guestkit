// SPDX-License-Identifier: Apache-2.0
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

pub mod assurance;
pub mod boot;
pub mod cli;
pub mod converters;
pub mod core;
pub mod disk;
pub mod evidence;
pub mod export;
pub mod fleet;
pub mod guestfs;
pub mod inference;
pub mod storage;

#[cfg(feature = "agent")]
pub mod metrics;

// Optional modules
#[cfg(feature = "guest-inspect")]
pub mod detectors;

#[cfg(feature = "python-bindings")]
pub mod python;

#[cfg(feature = "agent")]
pub mod agent;

// Re-exports for convenience
pub use assurance::{
    boot_target_from_str, collect_assurance_data, run_boot_inspect, run_doctor, run_migrate_plan,
    run_repair_plan, BootInspectSummary, DoctorResult, MigratePlanOptions, MigrationPlanResult,
    RepairOptions, RepairPlanResult,
};
pub use boot::{BootTarget, BootabilityReport};
pub use converters::DiskConverter;
pub use core::types::*;
pub use core::{Error, Result, RetryConfig};
pub use disk::{DiskReader, FileSystem, PartitionTable};
pub use evidence::EvidenceSnapshot;
pub use export::{
    create_variable_map, HtmlExportOptions, HtmlExporter, PaperSize, PdfExportOptions, PdfExporter,
    TemplateEngine, TemplateFormat, TemplateLevel,
};
pub use guestfs::Guestfs;

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
