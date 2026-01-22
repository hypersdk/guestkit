// SPDX-License-Identifier: LGPL-3.0-or-later
//! FFI bindings to libguestfs
//!
//! This module provides Rust bindings to the libguestfs C library for
//! guest filesystem inspection and modification.

pub mod bindings;
pub mod guestfs;

pub use guestfs::Guestfs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_module_exists() {
        // Basic test to ensure module compiles
        assert!(true);
    }
}
