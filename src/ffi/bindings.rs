// SPDX-License-Identifier: LGPL-3.0-or-later
//! Auto-generated FFI bindings to libguestfs
//!
//! These bindings are automatically generated from the libguestfs C headers
//! using bindgen at build time.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(deref_nullptr)]

// Include the auto-generated bindings if available
#[cfg(all(feature = "ffi-bindings", not(doc)))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Fallback manual bindings for documentation or when libguestfs is not available
#[cfg(any(not(feature = "ffi-bindings"), doc))]
pub mod fallback {
    use libc::{c_char, c_int, c_void};

    /// Opaque guestfs handle (fallback)
    #[repr(C)]
    pub struct guestfs_h {
        _private: [u8; 0],
    }

    extern "C" {
        // Core functions
        pub fn guestfs_create() -> *mut guestfs_h;
        pub fn guestfs_create_flags(flags: c_int, ...) -> *mut guestfs_h;
        pub fn guestfs_close(g: *mut guestfs_h);

        // Configuration
        pub fn guestfs_set_verbose(g: *mut guestfs_h, verbose: c_int) -> c_int;
        pub fn guestfs_get_verbose(g: *mut guestfs_h) -> c_int;
        pub fn guestfs_set_trace(g: *mut guestfs_h, trace: c_int) -> c_int;
        pub fn guestfs_get_trace(g: *mut guestfs_h) -> c_int;

        // Disk operations
        pub fn guestfs_add_drive_ro(g: *mut guestfs_h, filename: *const c_char) -> c_int;
        pub fn guestfs_launch(g: *mut guestfs_h) -> c_int;
        pub fn guestfs_shutdown(g: *mut guestfs_h) -> c_int;

        // Inspection
        pub fn guestfs_inspect_os(g: *mut guestfs_h) -> *mut *mut c_char;
        pub fn guestfs_inspect_get_type(g: *mut guestfs_h, root: *const c_char) -> *mut c_char;
        pub fn guestfs_inspect_get_distro(g: *mut guestfs_h, root: *const c_char) -> *mut c_char;
        pub fn guestfs_inspect_get_product_name(g: *mut guestfs_h, root: *const c_char) -> *mut c_char;
        pub fn guestfs_inspect_get_major_version(g: *mut guestfs_h, root: *const c_char) -> c_int;
        pub fn guestfs_inspect_get_minor_version(g: *mut guestfs_h, root: *const c_char) -> c_int;
        pub fn guestfs_inspect_get_arch(g: *mut guestfs_h, root: *const c_char) -> *mut c_char;

        // Filesystem operations
        pub fn guestfs_mount_ro(g: *mut guestfs_h, mountable: *const c_char, mountpoint: *const c_char) -> c_int;
        pub fn guestfs_umount(g: *mut guestfs_h, pathordevice: *const c_char) -> c_int;
        pub fn guestfs_is_file(g: *mut guestfs_h, path: *const c_char) -> c_int;
        pub fn guestfs_is_dir(g: *mut guestfs_h, path: *const c_char) -> c_int;
        pub fn guestfs_list_partitions(g: *mut guestfs_h) -> *mut *mut c_char;

        // Utility
        pub fn guestfs_free_string_list(argv: *mut *mut c_char);
        pub fn guestfs_last_error(g: *mut guestfs_h) -> *const c_char;
    }
}

// Re-export from the appropriate module
#[cfg(all(feature = "ffi-bindings", not(doc)))]
pub use self::*;

#[cfg(any(not(feature = "ffi-bindings"), doc))]
pub use fallback::*;

/// Helper to convert C string to Rust String
pub unsafe fn c_str_to_string(ptr: *mut c_char) -> Option<String> {
    use libc::c_void;
    use std::ffi::CStr;

    if ptr.is_null() {
        None
    } else {
        let c_str = CStr::from_ptr(ptr);
        let result = c_str.to_string_lossy().into_owned();
        libc::free(ptr as *mut c_void);
        Some(result)
    }
}

/// Helper to convert C string list to Rust Vec<String>
pub unsafe fn c_str_list_to_vec(ptr: *mut *mut c_char) -> Vec<String> {
    if ptr.is_null() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut i = 0;
    loop {
        let elem_ptr = *ptr.offset(i);
        if elem_ptr.is_null() {
            break;
        }
        let c_str = std::ffi::CStr::from_ptr(elem_ptr);
        result.push(c_str.to_string_lossy().into_owned());
        i += 1;
    }

    guestfs_free_string_list(ptr);
    result
}

use libc::{c_char, c_int};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bindings_module_exists() {
        // This test just ensures the module compiles
        assert!(true);
    }
}
