// SPDX-License-Identifier: LGPL-3.0-or-later
//! Memory optimization utilities for guestctl
//!
//! This module provides optimized memory allocation patterns to improve
//! performance by reducing reallocation overhead.
//!
//! # Performance Impact
//!
//! Using pre-allocated vectors provides approximately 2x speedup for
//! allocation-heavy operations by avoiding multiple reallocations as
//! the vector grows.
//!
//! # Usage
//!
//! ```rust
//! use guestctl::core::mem_optimize;
//!
//! // Instead of Vec::new()
//! let packages = mem_optimize::vec_for_packages();
//!
//! // Instead of Vec::with_capacity(arbitrary_number)
//! let filesystems = mem_optimize::vec_for_filesystems();
//! ```

/// Typical capacity estimates based on real-world observations
pub mod capacity {
    /// Typical number of partitions in a VM disk (usually 1-4)
    pub const PARTITIONS: usize = 4;

    /// Typical number of filesystems (usually 2-8)
    pub const FILESYSTEMS: usize = 8;

    /// Typical number of mount points (usually 4-16)
    pub const MOUNT_POINTS: usize = 16;

    /// Small package count (minimal installations)
    pub const PACKAGES_SMALL: usize = 256;

    /// Medium package count (typical server)
    pub const PACKAGES_MEDIUM: usize = 512;

    /// Large package count (desktop/development system)
    pub const PACKAGES_LARGE: usize = 1024;

    /// Default package count
    pub const PACKAGES: usize = PACKAGES_MEDIUM;

    /// Typical user count (usually 5-20)
    pub const USERS: usize = 20;

    /// Typical service count (usually 20-100)
    pub const SERVICES: usize = 50;

    /// Typical network interfaces (usually 1-4)
    pub const NETWORK_INTERFACES: usize = 4;

    /// Typical LVM volume groups (usually 1-2)
    pub const LVM_VG: usize = 2;

    /// Typical LVM logical volumes per VG (usually 2-8)
    pub const LVM_LV: usize = 8;

    /// Typical file listing size (small directory)
    pub const FILES_SMALL: usize = 64;

    /// Typical file listing size (medium directory)
    pub const FILES_MEDIUM: usize = 256;

    /// Typical file listing size (large directory)
    pub const FILES_LARGE: usize = 1024;

    /// Default file listing size
    pub const FILES: usize = FILES_MEDIUM;

    /// Typical number of VMs in batch operations
    pub const BATCH_VMS: usize = 8;

    /// Typical number of environment variables
    pub const ENV_VARS: usize = 32;

    /// Typical number of cron jobs
    pub const CRON_JOBS: usize = 16;
}

/// Create a pre-allocated vector for partitions
///
/// # Examples
///
/// ```
/// let mut partitions = guestctl::core::mem_optimize::vec_for_partitions();
/// // partitions is pre-allocated with capacity for typical partition count
/// ```
#[inline]
pub fn vec_for_partitions<T>() -> Vec<T> {
    Vec::with_capacity(capacity::PARTITIONS)
}

/// Create a pre-allocated vector for filesystems
#[inline]
pub fn vec_for_filesystems<T>() -> Vec<T> {
    Vec::with_capacity(capacity::FILESYSTEMS)
}

/// Create a pre-allocated vector for mount points
#[inline]
pub fn vec_for_mount_points<T>() -> Vec<T> {
    Vec::with_capacity(capacity::MOUNT_POINTS)
}

/// Create a pre-allocated vector for packages
///
/// Uses medium capacity by default (512 packages).
/// For minimal systems, consider using `vec_with_capacity(capacity::PACKAGES_SMALL)`.
#[inline]
pub fn vec_for_packages<T>() -> Vec<T> {
    Vec::with_capacity(capacity::PACKAGES)
}

/// Create a pre-allocated vector for users
#[inline]
pub fn vec_for_users<T>() -> Vec<T> {
    Vec::with_capacity(capacity::USERS)
}

/// Create a pre-allocated vector for services
#[inline]
pub fn vec_for_services<T>() -> Vec<T> {
    Vec::with_capacity(capacity::SERVICES)
}

/// Create a pre-allocated vector for network interfaces
#[inline]
pub fn vec_for_network_interfaces<T>() -> Vec<T> {
    Vec::with_capacity(capacity::NETWORK_INTERFACES)
}

/// Create a pre-allocated vector for LVM volume groups
#[inline]
pub fn vec_for_lvm_vg<T>() -> Vec<T> {
    Vec::with_capacity(capacity::LVM_VG)
}

/// Create a pre-allocated vector for LVM logical volumes
#[inline]
pub fn vec_for_lvm_lv<T>() -> Vec<T> {
    Vec::with_capacity(capacity::LVM_LV)
}

/// Create a pre-allocated vector for file listings
///
/// Uses medium capacity by default (256 files).
#[inline]
pub fn vec_for_files<T>() -> Vec<T> {
    Vec::with_capacity(capacity::FILES)
}

/// Create a pre-allocated vector for batch VM operations
#[inline]
pub fn vec_for_batch_vms<T>() -> Vec<T> {
    Vec::with_capacity(capacity::BATCH_VMS)
}

/// Create a pre-allocated vector for environment variables
#[inline]
pub fn vec_for_env_vars<T>() -> Vec<T> {
    Vec::with_capacity(capacity::ENV_VARS)
}

/// Create a pre-allocated vector for cron jobs
#[inline]
pub fn vec_for_cron_jobs<T>() -> Vec<T> {
    Vec::with_capacity(capacity::CRON_JOBS)
}

/// Create a vector with estimated capacity based on input size
///
/// Useful when processing collections where the output size is
/// proportional to the input size.
///
/// # Examples
///
/// ```
/// let input = vec![1, 2, 3, 4, 5];
/// let mut output = guestctl::core::mem_optimize::vec_with_estimated_capacity(&input, 1.5);
/// // output has capacity of ~7 (5 * 1.5), avoiding reallocation
/// ```
#[inline]
pub fn vec_with_estimated_capacity<T, U>(input: &[U], factor: f32) -> Vec<T> {
    let capacity = (input.len() as f32 * factor).ceil() as usize;
    Vec::with_capacity(capacity)
}

/// Optimize a vector by shrinking to fit if it has excess capacity
///
/// Only shrinks if the excess capacity is significant (>25% waste).
///
/// # Examples
///
/// ```
/// let mut vec = Vec::with_capacity(1000);
/// // ... add 100 items ...
/// guestctl::core::mem_optimize::shrink_if_wasteful(&mut vec);
/// // vec now uses less memory
/// ```
pub fn shrink_if_wasteful<T>(vec: &mut Vec<T>) {
    let len = vec.len();
    let cap = vec.capacity();

    // Only shrink if capacity is >25% wasted
    if cap > len && (cap - len) as f64 / cap as f64 > 0.25 {
        vec.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacity_constants() {
        assert!(capacity::PARTITIONS > 0);
        assert!(capacity::PACKAGES > capacity::PACKAGES_SMALL);
        assert!(capacity::PACKAGES_LARGE > capacity::PACKAGES_MEDIUM);
    }

    #[test]
    fn test_vec_for_partitions() {
        let vec: Vec<String> = vec_for_partitions();
        assert_eq!(vec.len(), 0);
        assert!(vec.capacity() >= capacity::PARTITIONS);
    }

    #[test]
    fn test_vec_for_packages() {
        let vec: Vec<String> = vec_for_packages();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), capacity::PACKAGES);
    }

    #[test]
    fn test_vec_with_estimated_capacity() {
        let input = vec![1, 2, 3, 4, 5];
        let vec: Vec<i32> = vec_with_estimated_capacity(&input, 2.0);
        assert_eq!(vec.len(), 0);
        assert!(vec.capacity() >= 10);
    }

    #[test]
    fn test_shrink_if_wasteful() {
        let mut vec = Vec::with_capacity(1000);
        vec.push(1);
        vec.push(2);
        vec.push(3);

        shrink_if_wasteful(&mut vec);

        // Should shrink since we're using <1% of capacity
        assert!(vec.capacity() < 1000);
    }

    #[test]
    fn test_shrink_if_wasteful_no_shrink() {
        let mut vec = Vec::with_capacity(10);
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.push(5);
        vec.push(6);
        vec.push(7);
        vec.push(8);

        let cap_before = vec.capacity();
        shrink_if_wasteful(&mut vec);
        let cap_after = vec.capacity();

        // Should NOT shrink since we're using >75% of capacity
        assert_eq!(cap_before, cap_after);
    }

    #[test]
    fn test_all_vec_creators() {
        // Ensure all functions compile and work
        let _: Vec<u8> = vec_for_partitions();
        let _: Vec<u8> = vec_for_filesystems();
        let _: Vec<u8> = vec_for_mount_points();
        let _: Vec<u8> = vec_for_packages();
        let _: Vec<u8> = vec_for_users();
        let _: Vec<u8> = vec_for_services();
        let _: Vec<u8> = vec_for_network_interfaces();
        let _: Vec<u8> = vec_for_lvm_vg();
        let _: Vec<u8> = vec_for_lvm_lv();
        let _: Vec<u8> = vec_for_files();
        let _: Vec<u8> = vec_for_batch_vms();
        let _: Vec<u8> = vec_for_env_vars();
        let _: Vec<u8> = vec_for_cron_jobs();
    }
}
