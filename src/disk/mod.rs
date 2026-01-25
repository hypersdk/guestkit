// SPDX-License-Identifier: LGPL-3.0-or-later
//! Pure Rust disk image handling
//!
//! This module provides pure Rust implementations for reading disk images,
//! parsing partition tables, and detecting filesystems.

pub mod filesystem;
pub mod loop_device;
pub mod nbd;
pub mod partition;
pub mod reader;

pub use filesystem::{FileSystem, FileSystemType};
pub use loop_device::LoopDevice;
pub use nbd::NbdDevice;
pub use partition::{Partition, PartitionTable, PartitionType};
pub use reader::DiskReader;
