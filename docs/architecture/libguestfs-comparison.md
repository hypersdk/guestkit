# LibGuestFS API Comparison

Complete comparison between libguestfs (733 functions) and guestctl implementation.

## Statistics

| Category | LibGuestFS | GuestCtl | Coverage |
|----------|-----------|----------|----------|
| **Total Core Functions** | 733 | 85 | 11.6% |
| **Fully Working** | 733 | 35 | 4.8% |
| **API Defined** | 733 | 85 | 11.6% |

## What We Have Implemented

### ✅ Core Functions (35 working)

**Lifecycle & Configuration:**
- `create` → `Guestfs::new()`
- `add_drive_ro` → `add_drive_ro()`
- `add_drive_opts` → `add_drive_opts()`
- `launch` → `launch()`
- `shutdown` → `shutdown()`
- `close` → `close()`
- `set_verbose` / `get_verbose`
- `set_trace` / `get_trace`

**Inspection (11 functions):**
- `inspect_os`
- `inspect_get_type`
- `inspect_get_distro`
- `inspect_get_product_name`
- `inspect_get_arch`
- `inspect_get_major_version`
- `inspect_get_minor_version`
- `inspect_get_hostname`
- `inspect_get_package_format`
- `inspect_get_mountpoints`
- `inspect_list_applications`
- `inspect_is_live`

**Device Operations (9 functions):**
- `list_devices`
- `list_partitions`
- `list_filesystems`
- `blockdev_getsize64`
- `blockdev_getsz`
- `canonical_device_name`
- `device_index`
- `is_whole_device`

**Filesystem Properties (3 functions):**
- `vfs_type`
- `vfs_label`
- `vfs_uuid`

**Partition Operations (5 functions):**
- `part_list`
- `part_get_parttype`
- `part_get_bootable`
- `part_get_mbr_id`
- `part_to_dev`
- `part_to_partnum`

### ⚠️ API Defined (50 additional)

**Mount Operations:**
- `mount`, `mount_ro`, `mount_options`, `mount_vfs`
- `umount`, `umount_all`
- `mounts`, `mountpoints`
- `mkmountpoint`, `rmmountpoint`
- `sync`

**File Operations:**
- `is_file`, `is_dir`, `exists`
- `read_file`, `cat`, `read_lines`
- `write`, `mkdir`, `mkdir_p`
- `ls`, `ll`, `stat`, `filesize`
- `rm`, `rmdir`, `touch`
- `chmod`, `chown`, `realpath`

**LVM:**
- `vgscan`, `lvs`, `lvs_full`, `vgs`, `pvs`

## What's Missing (648 functions)

### Critical for hyper2kvm (HIGH PRIORITY)

**Command Execution:**
- ❌ `command` - Execute command in guest
- ❌ `command_lines` - Execute and get output lines
- ❌ `sh` - Execute shell command
- ❌ `sh_lines` - Execute shell and get lines

**Archive Operations:**
- ❌ `tar_in` - Extract tar archive
- ❌ `tar_out` - Create tar archive
- ❌ `tgz_in` / `tgz_out` - Compressed tar
- ❌ `tar_in_opts` - Tar with options
- ❌ `tar_out_opts` - Tar out with options
- ❌ `cpio_out` - Create cpio archive

**Advanced File Operations:**
- ❌ `cp` - Copy file
- ❌ `cp_a` - Copy with attributes
- ❌ `cp_r` - Copy recursively
- ❌ `mv` - Move/rename
- ❌ `download` - Download file from guest
- ❌ `upload` - Upload file to guest
- ❌ `write_append` - Append to file
- ❌ `grep` / `egrep` / `fgrep` - Search in files
- ❌ `find` / `find0` - Find files
- ❌ `du` - Disk usage

**Extended Attributes:**
- ❌ `getxattr` - Get extended attribute
- ❌ `lgetxattr` - Get xattr (no follow symlink)
- ❌ `setxattr` - Set extended attribute
- ❌ `lsetxattr` - Set xattr (no follow symlink)
- ❌ `removexattr` - Remove xattr
- ❌ `lremovexattr` - Remove xattr (no follow symlink)
- ❌ `getxattrs` - List all xattrs
- ❌ `lgetxattrs` - List xattrs (no follow symlink)

**ACL Operations:**
- ❌ `acl_get_file` - Get POSIX ACL
- ❌ `acl_set_file` - Set POSIX ACL
- ❌ `acl_delete_def_file` - Delete default ACL

**Filesystem Creation:**
- ❌ `mkfs` / `mkfs_opts` - Make filesystem
- ❌ `mkfs_b` / `mkfs_btrfs` - Make btrfs
- ❌ `mke2fs` / `mke2fs_J` / `mke2fs_JL` / `mke2fs_JU` - Make ext2/3/4
- ❌ `mkswap` / `mkswap_opts` - Make swap
- ❌ `mktemp` - Make temporary file

**LVM Operations:**
- ❌ `pvcreate` / `pvremove` / `pvresize` - Physical volume ops
- ❌ `vgcreate` / `vgremove` / `vgactivate` - Volume group ops
- ❌ `lvcreate` / `lvremove` / `lvresize` - Logical volume ops
- ❌ `vg_activate_all` / `vg_activate` - Activation
- ❌ `lvm_remove_all` - Remove all LVM

**LUKS/Encryption:**
- ❌ `luks_format` / `luks_format_cipher` - Format LUKS
- ❌ `luks_open` / `luks_open_ro` - Open LUKS
- ❌ `luks_close` - Close LUKS
- ❌ `luks_add_key` / `luks_kill_slot` - Key management
- ❌ `luks_uuid` - Get LUKS UUID

**MD RAID:**
- ❌ `md_create` - Create RAID array
- ❌ `md_stop` - Stop RAID array
- ❌ `md_detail` - Get RAID details
- ❌ `mdadm_create` / `mdadm_stop` - mdadm operations

**Partition Management:**
- ❌ `part_add` - Add partition
- ❌ `part_del` - Delete partition
- ❌ `part_disk` - Initialize whole disk as one partition
- ❌ `part_init` - Initialize partition table
- ❌ `part_resize` - Resize partition
- ❌ `part_set_bootable` - Set bootable flag
- ❌ `part_set_mbr_id` / `part_set_gpt_type` - Set partition type
- ❌ `part_get_gpt_type` / `part_get_gpt_guid` - Get GPT info

**Filesystem Checks/Tuning:**
- ❌ `fsck` - Filesystem check
- ❌ `e2fsck` / `e2fsck_f` - ext2/3/4 check
- ❌ `tune2fs` / `tune2fs_l` - Tune ext2/3/4
- ❌ `resize2fs` / `resize2fs_M` / `resize2fs_size` - Resize ext2/3/4
- ❌ `ntfsresize` / `ntfsresize_opts` / `ntfsresize_size` - Resize NTFS
- ❌ `ntfsfix` / `ntfsclone_out` / `ntfsclone_in` - NTFS operations
- ❌ `xfs_admin` / `xfs_growfs` / `xfs_info` / `xfs_repair` - XFS ops

**Augeas (Config File Editing):**
- ❌ `aug_init` / `aug_close` - Initialize/close Augeas
- ❌ `aug_get` / `aug_set` - Get/set config values
- ❌ `aug_insert` / `aug_rm` / `aug_mv` - Modify config
- ❌ `aug_match` / `aug_ls` - Query config
- ❌ `aug_load` / `aug_save` - Load/save files
- ❌ `aug_defvar` / `aug_defnode` - Define variables

**SELinux:**
- ❌ `selinux_relabel` - Relabel filesystem
- ❌ `setcon` / `getcon` - Set/get SELinux context

**Windows Registry (Hivex):**
- ❌ `hivex_open` / `hivex_close` - Open/close registry hive
- ❌ `hivex_root` / `hivex_node_name` - Navigate registry
- ❌ `hivex_node_values` / `hivex_value_value` - Read values
- ❌ `hivex_node_set_value` / `hivex_node_add_child` - Modify registry

### Medium Priority

**Btrfs Operations (50+ functions):**
- `btrfs_subvolume_create` / `btrfs_subvolume_delete` / `btrfs_subvolume_list`
- `btrfs_filesystem_balance` / `btrfs_filesystem_resize` / `btrfs_filesystem_sync`
- `btrfs_qgroup_*` - Quota group operations
- `btrfs_scrub_*` - Scrub operations
- `btrfs_rescue_*` - Recovery operations

**ZFS Operations:**
- `zfs_create` / `zfs_destroy` / `zfs_list`
- `zpool_create` / `zpool_destroy` / `zpool_list`

**iSCSI Operations:**
- `iscsi_create` / `iscsi_remove`

**NBD Operations:**
- `nbd_server_start` / `nbd_server_stop`
- `nbd_export_name` / `nbd_export_description`

**Compression:**
- `compress_out` / `compress_device_out` - Compress files
- `uncompress` - Decompress files

**Disk Image Operations:**
- `disk_create` - Create disk image
- `disk_format` - Get disk image format
- `disk_has_backing_file` - Check for backing file
- `disk_virtual_size` - Get virtual size

**Performance/Caching:**
- `set_cachedir` / `get_cachedir` - Cache directory
- `set_memsize` / `get_memsize` - Memory size
- `set_smp` / `get_smp` - SMP configuration

### Low Priority (Specialized)

**YARA (Malware Scanning):**
- `yara_load` / `yara_destroy` / `yara_scan`

**TSK (Sleuthkit/File Recovery):**
- `tsk_*` - Various file recovery functions

**inotify:**
- `inotify_init` / `inotify_add_watch` / `inotify_files` / `inotify_close`

**Internal/Debug:**
- `debug` / `debug_drives` / `debug_cmdline` / `debug_upload`

**Checksums:**
- `checksums_out` / `checksum` / `checksum_device`

**RSync:**
- `rsync` / `rsync_in` / `rsync_out`

## Recommendations for Implementation Priority

### Phase 1: Essential Operations (2-3 weeks)
Implement these for 90% hyper2kvm compatibility:

1. **Command execution** (4 functions)
   - `command`, `command_lines`, `sh`, `sh_lines`

2. **Archive operations** (6 functions)
   - `tar_in`, `tar_out`, `tgz_in`, `tgz_out`, `tar_in_opts`, `tar_out_opts`

3. **File operations** (10 functions)
   - `cp`, `cp_a`, `mv`, `download`, `upload`, `write_append`
   - `grep`, `egrep`, `find`, `du`

4. **LUKS** (6 functions)
   - `luks_open`, `luks_open_ro`, `luks_close`, `luks_format`
   - `luks_add_key`, `luks_uuid`

5. **LVM activation** (4 functions)
   - `vg_activate_all`, `vg_activate`, `lvcreate`, `lvremove`

**Total: 30 critical functions**

### Phase 2: Filesystem Operations (2 weeks)

1. **Filesystem creation** (8 functions)
   - `mkfs`, `mkfs_opts`, `mkfs_btrfs`, `mke2fs`

2. **Filesystem repair** (6 functions)
   - `fsck`, `e2fsck`, `ntfsfix`, `xfs_repair`

3. **Extended attributes** (8 functions)
   - `getxattr`, `setxattr`, `getxattrs`, `removexattr`

**Total: 22 functions**

### Phase 3: Advanced (3-4 weeks)

1. **Augeas** (15 functions) - Config file editing
2. **Windows registry** (10 functions) - Hivex operations
3. **Partition management** (10 functions) - Add/delete/resize
4. **SELinux** (2 functions) - Relabeling

**Total: 37 functions**

### Phase 4: Specialized (Optional)

1. **Btrfs** (50+ functions)
2. **ZFS** (10+ functions)
3. **YARA** (4 functions)
4. **TSK** (6 functions)

**Total: 70+ functions**

## Full Coverage Timeline

| Phase | Functions | Time | Cumulative Coverage |
|-------|-----------|------|---------------------|
| Current | 85 | - | 11.6% |
| Phase 1 | +30 | 3 weeks | 15.7% |
| Phase 2 | +22 | 2 weeks | 18.7% |
| Phase 3 | +37 | 4 weeks | 23.7% |
| Phase 4 | +70 | 8 weeks | 33.3% |
| Remaining | +489 | 6+ months | 100% |

**Realistic target for hyper2kvm: Phases 1-3 = 174 functions (23.7% coverage) in 9 weeks**

This covers all critical operations while remaining focused on what hyper2kvm actually uses.

## Implementation Strategy

### For Complete Coverage (All 733 functions):

**Option 1: Pure Rust (6-12 months)**
- Implement each function from scratch
- Requires filesystem parsers, LVM parsers, etc.
- Maximum control, no dependencies
- Extremely time-consuming

**Option 2: Hybrid NBD + Kernel (2-3 months for 90% coverage)**
- Use NBD to export images as block devices
- Leverage kernel drivers for filesystems
- Implement missing operations via system tools
- Best effort/results ratio

**Option 3: Wrapper Around libguestfs (1 week)**
- Link to libguestfs.so dynamically
- Provide Rust-safe wrappers
- Full compatibility immediately
- Defeats purpose of "no C dependencies"

**Recommended: Option 2 (Hybrid)** for production use

## Conclusion

LibGuestFS is a **massive** library with 733 core functions covering:
- Filesystem operations
- Partition management
- LVM/LUKS/RAID
- Archive handling
- Config file editing (Augeas)
- Windows registry (Hivex)
- File recovery (TSK)
- Malware scanning (YARA)
- Btrfs/ZFS advanced features

**GuestCtl current status:**
- ✅ Excellent foundation (85 APIs, 35 working)
- ✅ Perfect for inspection/detection
- ⚠️ Needs Phase 1-3 for full hyper2kvm support
- ⚠️ Full parity would take 6-12 months

**For hyper2kvm integration:**
- Use guestctl for fast inspection (no root needed)
- Implement Phase 1 (30 critical functions) for most operations
- Fall back to libguestfs for specialized needs

---

**Last Updated:** 2026-01-23
**LibGuestFS Version Analyzed:** Latest (from git)
**Total Functions Analyzed:** 733 core + 250 variants/helpers = 983 total
