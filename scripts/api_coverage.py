#!/usr/bin/env python3
"""
Analyze guestkit API coverage compared to libguestfs

This script helps identify which APIs are implemented and which are missing.
"""

import os
import re
from collections import defaultdict

# Common libguestfs APIs we should have
LIBGUESTFS_CORE_APIS = {
    # Handle management
    "create", "close", "shutdown", "set_verbose", "get_verbose",
    "set_trace", "get_trace", "add_drive", "add_drive_opts", "add_drive_ro",
    "launch",

    # Device operations
    "list_devices", "list_partitions", "list_filesystems",
    "blockdev_getsize64", "blockdev_getsz", "blockdev_getss",
    "canonical_device_name",

    # Partition operations
    "part_init", "part_add", "part_del", "part_list",
    "part_get_parttype", "part_set_parttype",
    "part_get_bootable", "part_set_bootable",
    "part_get_name", "part_set_name",
    "part_get_mbr_id", "part_set_mbr_id",
    "part_to_dev", "part_to_partnum",

    # Filesystem operations
    "mkfs", "mkfs_opts", "fsck", "tune2fs",
    "vfs_type", "vfs_label", "vfs_uuid",
    "set_label", "set_uuid",
    "df", "df_h", "statvfs",

    # Mount operations
    "mount", "mount_ro", "mount_options", "mount_vfs",
    "umount", "umount_all", "mounts", "mountpoints",
    "mkmountpoint", "rmmountpoint",

    # File operations
    "cat", "read_file", "read_lines", "write", "write_append",
    "is_file", "is_dir", "exists", "stat", "lstat",
    "filesize", "touch", "chmod", "chown",
    "mkdir", "mkdir_p", "rmdir", "rm", "rm_rf",
    "ls", "ll", "find", "find0",
    "cp", "cp_a", "cp_r", "mv",
    "download", "upload",
    "grep", "egrep", "fgrep", "zgrep",
    "head", "head_n", "tail", "tail_n",
    "realpath", "readlink", "ln", "ln_s", "ln_f", "ln_sf",

    # Archive operations
    "tar_in", "tar_out", "tgz_in", "tgz_out",
    "tar_in_opts", "tar_out_opts",
    "cpio_in", "cpio_out",

    # Inspection
    "inspect_os", "inspect_get_type", "inspect_get_distro",
    "inspect_get_arch", "inspect_get_product_name",
    "inspect_get_major_version", "inspect_get_minor_version",
    "inspect_get_hostname", "inspect_get_package_format",
    "inspect_get_mountpoints", "inspect_list_applications",

    # Command execution
    "command", "command_lines", "sh", "sh_lines",

    # LUKS
    "luks_open", "luks_open_ro", "luks_close",
    "luks_format", "luks_add_key", "luks_uuid",

    # LVM
    "vgscan", "vg_activate", "vg_activate_all",
    "lvcreate", "lvremove", "lvs", "lvs_full",
    "vgs", "pvs",

    # Additional operations
    "checksum", "checksum_device",
    "du", "sync",
    "file", "file_architecture",
    "getxattr", "lgetxattrs", "setxattr", "removexattr",
}

def find_implemented_functions(src_dir):
    """Scan source files for implemented public functions"""
    implemented = set()

    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if file.endswith('.rs'):
                filepath = os.path.join(root, file)
                try:
                    with open(filepath, 'r') as f:
                        content = f.read()
                        # Find public function definitions
                        matches = re.findall(r'pub\s+fn\s+(\w+)\s*\(', content)
                        implemented.update(matches)
                except Exception as e:
                    print(f"Error reading {filepath}: {e}")

    return implemented

def categorize_apis(implemented):
    """Categorize APIs by functionality"""
    categories = {
        "Handle Management": [],
        "Device Operations": [],
        "Partition Operations": [],
        "Filesystem Operations": [],
        "Mount Operations": [],
        "File Operations": [],
        "Archive Operations": [],
        "Inspection": [],
        "Command Execution": [],
        "LUKS": [],
        "LVM": [],
        "Other": [],
    }

    for func in sorted(implemented):
        if func in ["new", "close", "shutdown", "set_verbose", "get_verbose", "set_trace", "get_trace", "launch"]:
            categories["Handle Management"].append(func)
        elif "device" in func or func.startswith("blockdev_") or func.startswith("canonical_"):
            categories["Device Operations"].append(func)
        elif func.startswith("part_"):
            categories["Partition Operations"].append(func)
        elif func.startswith("mkfs") or func.startswith("vfs_") or func in ["fsck", "tune2fs", "set_label", "set_uuid", "df", "statvfs"]:
            categories["Filesystem Operations"].append(func)
        elif "mount" in func and not func.startswith("umount"):
            categories["Mount Operations"].append(func)
        elif func.startswith("umount"):
            categories["Mount Operations"].append(func)
        elif func in ["cat", "read_file", "write", "ls", "mkdir", "rm", "cp", "mv", "touch", "chmod", "chown", "find", "grep", "head", "tail", "ln", "realpath", "readlink"]:
            categories["File Operations"].append(func)
        elif "tar" in func or "cpio" in func or "tgz" in func:
            categories["Archive Operations"].append(func)
        elif func.startswith("inspect_"):
            categories["Inspection"].append(func)
        elif func in ["command", "command_lines", "sh", "sh_lines"]:
            categories["Command Execution"].append(func)
        elif func.startswith("luks_"):
            categories["LUKS"].append(func)
        elif func.startswith("vg") or func.startswith("lv") or func.startswith("pv"):
            categories["LVM"].append(func)
        else:
            categories["Other"].append(func)

    return categories

def main():
    src_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), "src", "guestfs")

    if not os.path.exists(src_dir):
        print(f"Error: Directory not found: {src_dir}")
        return

    print("=" * 70)
    print("GuestKit API Coverage Analysis")
    print("=" * 70)
    print()

    # Find implemented functions
    implemented = find_implemented_functions(src_dir)

    print(f"Total public functions found: {len(implemented)}")
    print()

    # Categorize
    categories = categorize_apis(implemented)

    print("=" * 70)
    print("Functions by Category")
    print("=" * 70)
    print()

    total = 0
    for category, funcs in categories.items():
        if funcs:
            print(f"{category}: {len(funcs)}")
            total += len(funcs)

    print()
    print(f"Total categorized: {total}")
    print()

    # Check core API coverage
    print("=" * 70)
    print("Core libguestfs API Coverage")
    print("=" * 70)
    print()

    implemented_core = LIBGUESTFS_CORE_APIS & implemented
    missing_core = LIBGUESTFS_CORE_APIS - implemented

    coverage = len(implemented_core) / len(LIBGUESTFS_CORE_APIS) * 100

    print(f"Core APIs implemented: {len(implemented_core)}/{len(LIBGUESTFS_CORE_APIS)} ({coverage:.1f}%)")
    print()

    if missing_core:
        print(f"Missing core APIs ({len(missing_core)}):")
        for func in sorted(missing_core):
            print(f"  - {func}")
    else:
        print("✓ All core APIs implemented!")

    print()
    print("=" * 70)

if __name__ == "__main__":
    main()
