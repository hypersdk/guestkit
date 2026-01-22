// SPDX-License-Identifier: LGPL-3.0-or-later
//! Pure Rust disk image handling
//!
//! This module provides pure Rust implementations for reading disk images,
//! parsing partition tables, and detecting filesystems.

pub mod reader;
pub mod partition;
pub mod filesystem;

pub use reader::DiskReader;
pub use partition::{Partition, PartitionTable, PartitionType};
pub use filesystem::{FileSystem, FileSystemType};
