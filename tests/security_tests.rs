//! Security test suite for GuestKit
//!
//! Tests for path traversal, command injection, symlink attacks,
//! and resource limit enforcement.

use guestkit::guestfs::security_utils::PathValidator;
use guestkit::guestfs::{Guestfs, ResourceLimits};

#[test]
fn test_path_traversal_detection() {
    // Test that path traversal patterns are rejected
    assert!(PathValidator::validate_fs_path("../etc/passwd").is_err());
    assert!(PathValidator::validate_fs_path("/dir/../../../etc/shadow").is_err());
    assert!(PathValidator::validate_fs_path("./../../root").is_err());
    assert!(PathValidator::validate_fs_path("valid/path/../../../escape").is_err());
}

#[test]
fn test_path_traversal_valid_paths() {
    // Test that valid paths are accepted
    assert!(PathValidator::validate_fs_path("/etc/passwd").is_ok());
    assert!(PathValidator::validate_fs_path("/home/user/file.txt").is_ok());
    assert!(PathValidator::validate_fs_path("relative/path/file").is_ok());
    assert!(PathValidator::validate_fs_path("/").is_ok());
}

#[test]
fn test_device_path_command_injection_patterns() {
    // Test that command injection patterns in device paths are rejected
    assert!(PathValidator::validate_device_path("/dev/sda; rm -rf /").is_err());
    assert!(PathValidator::validate_device_path("/dev/sda && malicious").is_err());
    assert!(PathValidator::validate_device_path("/dev/sda | cat /etc/passwd").is_err());
    assert!(PathValidator::validate_device_path("/dev/sda$(evil)").is_err());
    assert!(PathValidator::validate_device_path("/dev/sda`cmd`").is_err());
    assert!(PathValidator::validate_device_path("/dev/sda>output").is_err());
    assert!(PathValidator::validate_device_path("/dev/sda<input").is_err());
}

#[test]
fn test_device_path_valid() {
    // Test that valid device paths are accepted
    assert!(PathValidator::validate_device_path("/dev/sda").is_ok());
    assert!(PathValidator::validate_device_path("/dev/sda1").is_ok());
    assert!(PathValidator::validate_device_path("/dev/vda2").is_ok());
    assert!(PathValidator::validate_device_path("/dev/nvme0n1").is_ok());
    assert!(PathValidator::validate_device_path("/dev/loop0").is_ok());
}

#[test]
fn test_cpio_format_validation() {
    // Test that only valid cpio formats are accepted
    assert!(PathValidator::validate_cpio_format("newc").is_ok());
    assert!(PathValidator::validate_cpio_format("crc").is_ok());
    assert!(PathValidator::validate_cpio_format("tar").is_ok());
    assert!(PathValidator::validate_cpio_format("ustar").is_ok());

    // Test that invalid/malicious formats are rejected
    assert!(PathValidator::validate_cpio_format("newc; rm -rf /").is_err());
    assert!(PathValidator::validate_cpio_format("invalid").is_err());
    assert!(PathValidator::validate_cpio_format("").is_err());
    assert!(PathValidator::validate_cpio_format("tar && malicious").is_err());
}

#[test]
fn test_null_byte_injection() {
    // Test that null bytes in paths are rejected
    assert!(PathValidator::validate_fs_path("/path\0/file").is_err());
    assert!(PathValidator::validate_fs_path("file\0name").is_err());
}

#[test]
fn test_path_length_limits() {
    // Test that excessively long paths are rejected
    let long_path = "a/".repeat(3000); // 6000 characters
    assert!(PathValidator::validate_fs_path(&long_path).is_err());

    // Test that reasonable paths are accepted
    let normal_path = "a/".repeat(100); // 200 characters
    assert!(PathValidator::validate_fs_path(&normal_path).is_ok());
}

#[test]
fn test_device_path_length_limits() {
    // Test that excessively long device paths are rejected
    let long_device = format!("/dev/{}", "a".repeat(300));
    assert!(PathValidator::validate_device_path(&long_device).is_err());

    // Test that normal device paths are accepted
    assert!(PathValidator::validate_device_path("/dev/sda1").is_ok());
}

#[test]
fn test_resource_limits_file_size() {
    let mut g = Guestfs::new().unwrap();

    // Set strict file size limit
    g.set_resource_limits(ResourceLimits {
        max_file_size: Some(1024), // 1KB limit
        ..Default::default()
    });

    // Test that exceeding limit is detected
    assert!(g.check_file_size_limit(2048).is_err());
    assert!(g.check_file_size_limit(1024).is_ok());
    assert!(g.check_file_size_limit(512).is_ok());
}

#[test]
fn test_resource_limits_path_length() {
    let g = Guestfs::new().unwrap();

    // Default path length limit is 4096
    let long_path = "a".repeat(5000);
    assert!(g.check_path_length_limit(&long_path).is_err());

    let normal_path = "a".repeat(100);
    assert!(g.check_path_length_limit(&normal_path).is_ok());
}

#[test]
fn test_resource_limits_customization() {
    let mut g = Guestfs::new().unwrap();

    // Set custom limits
    g.set_resource_limits(ResourceLimits {
        max_file_size: Some(10 * 1024 * 1024), // 10MB
        operation_timeout: Some(std::time::Duration::from_secs(60)),
        max_path_length: 2048,
    });

    let limits = g.get_resource_limits();
    assert_eq!(limits.max_file_size, Some(10 * 1024 * 1024));
    assert_eq!(limits.max_path_length, 2048);
}

#[test]
fn test_path_component_validation() {
    // Valid components
    assert!(PathValidator::validate_path_component("file.txt").is_ok());
    assert!(PathValidator::validate_path_component("directory").is_ok());
    assert!(PathValidator::validate_path_component("file-name_123").is_ok());

    // Invalid components
    assert!(PathValidator::validate_path_component("..").is_err());
    assert!(PathValidator::validate_path_component(".").is_err());
    assert!(PathValidator::validate_path_component("").is_err());
    assert!(PathValidator::validate_path_component("dir/file").is_err());
    assert!(PathValidator::validate_path_component("file\0name").is_err());
}

#[test]
fn test_device_name_parsing() {
    let g = Guestfs::new().unwrap();

    // Valid device names
    assert_eq!(g.parse_device_name("/dev/sda1").unwrap(), 1);
    assert_eq!(g.parse_device_name("/dev/vda2").unwrap(), 2);
    assert_eq!(g.parse_device_name("/dev/hda3").unwrap(), 3);
    assert_eq!(g.parse_device_name("/dev/xvda1").unwrap(), 1);
    assert_eq!(g.parse_device_name("/dev/nvme0n1p1").unwrap(), 1);

    // Whole device (no partition number)
    assert_eq!(g.parse_device_name("/dev/sda").unwrap(), 0);
    assert_eq!(g.parse_device_name("/dev/vda").unwrap(), 0);

    // Invalid device names
    assert!(g.parse_device_name("/home/user/file").is_err());
    assert!(g.parse_device_name("sda1").is_err());
    assert!(g.parse_device_name("/dev/unknown99").is_err());
}

#[test]
fn test_shell_metacharacters_blocked() {
    // Test that various shell metacharacters are blocked in device paths
    let dangerous_chars = vec![
        "/dev/sda;",
        "/dev/sda&",
        "/dev/sda|",
        "/dev/sda$",
        "/dev/sda`",
        "/dev/sda(",
        "/dev/sda)",
        "/dev/sda{",
        "/dev/sda}",
        "/dev/sda[",
        "/dev/sda]",
        "/dev/sda<",
        "/dev/sda>",
        "/dev/sda\n",
        "/dev/sda\r",
        "/dev/sda\\",
        "/dev/sda'",
        "/dev/sda\"",
        "/dev/sda ",
        "/dev/sda\t",
        "/dev/sda*",
        "/dev/sda?",
    ];

    for device in dangerous_chars {
        assert!(
            PathValidator::validate_device_path(device).is_err(),
            "Should reject device path with dangerous character: {}",
            device
        );
    }
}

#[test]
fn test_empty_path_rejection() {
    // Empty paths should be rejected
    assert!(PathValidator::validate_fs_path("").is_err());
    assert!(PathValidator::validate_device_path("").is_err());
    assert!(PathValidator::validate_cpio_format("").is_err());
}

#[test]
fn test_device_prefix_validation() {
    // Devices must start with /dev/
    assert!(PathValidator::validate_device_path("sda1").is_err());
    assert!(PathValidator::validate_device_path("/home/sda1").is_err());
    assert!(PathValidator::validate_device_path("dev/sda1").is_err());
    assert!(PathValidator::validate_device_path("/dev/sda1").is_ok());
}
