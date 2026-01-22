// SPDX-License-Identifier: LGPL-3.0-or-later
//! Pure Rust implementation of GuestFS-compatible API
//!
//! This module provides a GuestFS-compatible API implemented entirely in Rust,
//! allowing disk image inspection and manipulation without libguestfs.

pub mod handle;
pub mod inspect;
pub mod mount;
pub mod file_ops;
pub mod device;
pub mod lvm;
pub mod partition;
pub mod command;
pub mod archive;

pub use handle::Guestfs;
pub use inspect::*;
