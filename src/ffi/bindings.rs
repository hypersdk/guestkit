// SPDX-License-Identifier: LGPL-3.0-or-later
//! Raw FFI bindings to libguestfs C API
//!
//! These are low-level bindings. Use the `guestfs` module for safe wrappers.

use libc::{c_char, c_int, c_void};
use std::ptr;

/// Opaque guestfs handle
#[repr(C)]
pub struct guestfs_h {
    _private: [u8; 0],
}

/// Link to libguestfs C library
#[link(name = "guestfs")]
extern "C" {
    // Handle creation and destruction
    pub fn guestfs_create() -> *mut guestfs_h;
    pub fn guestfs_create_flags(flags: c_int) -> *mut guestfs_h;
    pub fn guestfs_close(g: *mut guestfs_h);

    // Configuration
    pub fn guestfs_set_verbose(g: *mut guestfs_h, verbose: c_int) -> c_int;
    pub fn guestfs_get_verbose(g: *mut guestfs_h) -> c_int;
    pub fn guestfs_set_trace(g: *mut guestfs_h, trace: c_int) -> c_int;
    pub fn guestfs_get_trace(g: *mut guestfs_h) -> c_int;

    // Disk operations
    pub fn guestfs_add_drive_opts(
        g: *mut guestfs_h,
        filename: *const c_char,
        ...
    ) -> c_int;
    pub fn guestfs_add_drive_ro(g: *mut guestfs_h, filename: *const c_char) -> c_int;

    // Launch
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
    pub fn guestfs_mount(g: *mut guestfs_h, mountable: *const c_char, mountpoint: *const c_char) -> c_int;
    pub fn guestfs_umount(g: *mut guestfs_h, pathordevice: *const c_char) -> c_int;
    pub fn guestfs_umount_all(g: *mut guestfs_h) -> c_int;

    // File operations
    pub fn guestfs_is_file(g: *mut guestfs_h, path: *const c_char) -> c_int;
    pub fn guestfs_is_dir(g: *mut guestfs_h, path: *const c_char) -> c_int;
    pub fn guestfs_cat(g: *mut guestfs_h, path: *const c_char) -> *mut c_char;
    pub fn guestfs_read_file(g: *mut guestfs_h, path: *const c_char, size_r: *mut usize) -> *mut c_char;
    pub fn guestfs_write(g: *mut guestfs_h, path: *const c_char, content: *const c_char, content_size: usize) -> c_int;

    // List operations
    pub fn guestfs_list_partitions(g: *mut guestfs_h) -> *mut *mut c_char;
    pub fn guestfs_list_filesystems(g: *mut guestfs_h) -> *mut *mut c_char;

    // Utility
    pub fn guestfs_free_string_list(argv: *mut *mut c_char);
    pub fn guestfs_last_error(g: *mut guestfs_h) -> *const c_char;
}

/// Helper to convert C string to Rust String
pub unsafe fn c_str_to_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        let c_str = std::ffi::CStr::from_ptr(ptr);
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
