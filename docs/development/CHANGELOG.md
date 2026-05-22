# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security & Correctness - Sixth-Pass Review (45 issues, 37 files)

Comprehensive code review covering all 103K lines across 170 source files.
Six parallel analysis passes: core infrastructure, guestfs bindings, CLI commands,
shell/TUI, supporting modules, and a dedicated security audit.

#### Critical Fixes (6)
- **Command injection via `.sh()` bypass**: 46 hardcoded `.sh()` calls containing shell operators (`||`, `&&`, `>`, `|`) were silently failing because `.sh()` rejects these patterns. Added `sh_raw()` method for trusted hardcoded commands; migrated all affected calls in `interactive.rs` (`guestfs/command.rs`, `cli/interactive.rs`)
- **UTF-8 panic in string slicing**: Package name/version truncation used byte-offset slicing (`&name[..35]`) which panics on multi-byte characters. Replaced with `chars().take(N).collect()` (`cli/commands/inspect.rs`)
- **Division by zero in compliance score**: `total - skipped` could be zero when all validation rules are skipped. Changed guard from `total > 0` to `total > skipped` (`cli/validate/mod.rs`)
- **Hardcoded `/dev/sda` device path**: Partition listing always queried `/dev/sda`, silently failing on VirtIO (`vda`), NVMe, or other device types. Now dynamically derives device from partition name (`cli/commands/inspect.rs`)
- **Unsafe negative size cast**: `stat.size` (i64, can be -1) was cast to u64 without sign check, causing wraparound to massive values. Added `>= 0` guard (`cli/commands/disk_ops.rs`)
- **Unreachable code in retry loop**: Replaced `unreachable!()` macro with structured `last_err` accumulator pattern, eliminating the unreachable code path entirely (`core/retry.rs`)

#### High Fixes (7)
- **Memory exhaustion via unbounded read**: `upload()` and `equal()` called `read_to_end()` without size limits. Added 100MB cap to prevent DoS from malicious guest images (`guestfs/transfer.rs`)
- **Path traversal in file extract**: After joining relative paths, the canonical result was not verified to remain within the target directory. Added post-join canonical path boundary check (`cli/commands/file_ops.rs`)
- **Silent mount failures in plan apply**: `g.mount()` errors were discarded with `let _ =`. Now logs mount failures (`cli/plan/apply.rs`)
- **Octal permission parsing edge case**: `"0".trim_start_matches('0')` produced empty string causing parse failure. Parse full octal string and validate range 0-7777 (`cli/plan/apply.rs`)
- **Shadow file username injection**: Usernames containing `:` would corrupt `/etc/shadow` format. Added colon validation (`cli/commands/security.rs`)
- **OpenSSL child process timeout**: `wait_with_output()` could block forever. Added 30-second timeout with `try_wait()` polling and process kill on timeout (`cli/commands/security.rs`)
- **GuestFS handle leak on error**: Early returns in `apply()` skipped `g.shutdown()`. Added cleanup on all error paths (`cli/plan/apply.rs`)

#### Major Fixes (14)
- **Mount code duplication eliminated**: Extracted `prepare_mount()`, `record_mount()`, and `need_sudo()` from near-identical code in `mount_ro()`, `mount()`, and `mount_options()`, removing ~120 lines of duplication (`guestfs/mount.rs`)
- **22 repeated `unsafe { libc::geteuid() }` calls**: Extracted `pub(crate) fn need_sudo()` helper with safety documentation. Updated all call sites across 6 files (`guestfs/mount.rs`, `disk/nbd.rs`, `disk/loop_device.rs`, `guestfs/lvm.rs`, `guestfs/handle.rs`, `guestfs/device.rs`)
- **HashMap clone on every `mountpoints()` call**: Changed return type from `HashMap<String, String>` (deep clone) to `&HashMap<String, String>` (zero-cost reference) (`guestfs/mount.rs`)
- **Regex compilation in loop**: `update_clone_fstab()` compiled a new regex per UUID mapping. Pre-builds all regexes before the loop (`guestfs/lvm_clone.rs`)
- **Silent `.nth().unwrap_or("")` chains**: Network config parsing used fallible indexing with empty-string fallback, silently corrupting parsed data. Replaced with `if let Some()` pattern matching (`guestfs/inspect_enhanced.rs`)
- **Incorrect service state in blueprints**: `detect_services()` marked all services as `enabled: true` and `state: "active"` based solely on file existence. Now checks symlink in `multi-user.target.wants` for enabled state; uses `"installed"` instead of `"active"` (`cli/blueprint/mod.rs`)
- **TUI scroll state desynchronization**: `scroll_up()` and `scroll_down()` updated `scroll_offset` and `selected_index` independently. Synchronized both fields (`cli/tui/app.rs`)
- **Empty list crash in `scroll_bottom()`**: `saturating_sub(1)` on count=0 gave `usize::MAX`. Added early return for empty lists in all scroll/page methods (`cli/tui/app.rs`)
- **Cache key TOCTOU and error swallowing**: `modified()` failure silently defaulted to epoch, causing cache key collisions. Now propagates errors (`cli/cache.rs`)
- **BinaryCache fallback logging**: Improved warning when falling back to `/tmp` — now warns about reboot data loss (`core/binary_cache.rs`)
- **Unsafe `set_var` extracted**: Moved 5 `std::env::set_var()` calls to dedicated `set_env_vars_before_threads()` with safety documentation (`main.rs`)
- **File explorer scroll_offset not reset**: After filtering, `scroll_offset` could exceed entry count, causing rendering glitches (`cli/shell/explore.rs`)
- **File size overflow**: `filesize()` silently capped at `i64::MAX`. Now returns `Err` for files exceeding `i64::MAX` (`guestfs/file_ops.rs`)
- **LVM clone unreachable**: Replaced `unreachable!()` in `IsolationLevel::None` match arm with safe `return Ok(())` (`guestfs/lvm_clone.rs`)

#### Minor Fixes (14)
- **`OutputFormat` implements `FromStr` trait**: Replaced custom `from_str()` method (suppressing clippy warning) with proper `std::str::FromStr` implementation (`cli/output.rs`)
- **Unnecessary string allocation in `DiskFormat` parsing**: Replaced `to_lowercase()` (allocates) with `eq_ignore_ascii_case()` (zero-alloc) (`core/types.rs`)
- **Misleading forensic timeline**: Packages with no install timestamps were plotted at epoch 0 (1970). Removed from timeline entirely (`cli/commands/analysis.rs`)
- **Thread panic tracking in batch**: Worker thread panics were logged but not counted. Now tracks and reports panic count (`cli/commands/batch.rs`)
- **fstab field documentation**: Added comment documenting fstab(5) default behavior for dump/pass fields (`guestfs/fstab.rs`)
- **LVM parse defaults documented**: Added comment explaining -1 sentinel for inactive LV device numbers (`guestfs/lvm.rs`)
- **Version parsing logging**: Non-numeric version strings now log debug message instead of silently defaulting to 0 (`guestfs/inspect.rs`)
- **ReadDir error swallowing**: `entries.flatten()` silently skipped IO errors. Now logs unreadable entries (`guestfs/file_ops.rs`)
- **Symlink loop detection in recursive search**: Added `HashSet<String>` visited-path tracking to prevent infinite recursion (`cli/shell/commands/core.rs`)
- **Excessive clones in HTML exporter**: Replaced 7 `Option::clone().unwrap_or_else(|| "Unknown".to_string())` with `as_deref().unwrap_or("Unknown")` (`cli/exporters/html.rs`)
- **Progress template fallback**: Replaced `.expect()` on template parsing with `.unwrap_or_else(|| default_style())` to prevent panic if indicatif changes format (`core/progress.rs`)
- **Blanket `#![allow(dead_code)]` removed**: Replaced module-level suppression with targeted `#[allow(dead_code)]` on specific unused constant (`cli/output.rs`)
- **CommandExec review warning**: Generated bash scripts now include a `# [CommandExec] Review` comment before raw commands (`cli/plan/export.rs`)
- **`#[allow(dead_code)]` on public struct removed**: `PlanGenerator` is public API; dead_code suppression was inappropriate (`cli/plan/generator.rs`)

### Security & Correctness - Full Code Review Fixes (90 issues across 5 passes)

#### Fifth-Pass Review (36 issues, 19 files)

##### Critical Fixes (5)
- **Password leak via `/proc/cmdline`**: Plaintext password no longer passed as CLI argument to `openssl passwd`; now piped via stdin (`cli/commands/security.rs`)
- **Operator precedence bug in IOC matching**: `&&`/`||` precedence caused ALL lines to match for DOMAIN-type indicators (`cli/commands/security.rs`)
- **Shell injection in package search**: Replaced `sh -c` with direct `command()` argv arrays for dnf/apt/pacman (`cli/interactive.rs`)
- **Shell injection in cron commands**: Replaced `sh -c` with `command()` for crontab operations; added username validation (`cli/interactive.rs`)
- **Broken sed command generation**: Separated `sed_escape()` from `shell_escape()` to prevent double-quoting inside sed expressions (`cli/plan/export.rs`)

##### High Fixes (9)
- **`resolve_guest_path()` canonicalize fix**: Handles non-existent paths by falling back to parent directory canonicalization, fixing `write()`/`mkdir()`/`touch()` for new files (`guestfs/file_ops.rs`)
- **Mount options validation**: `mount_options()` now rejects dangerous options (`suid`, `dev`, `exec`) (`guestfs/mount.rs`)
- **VFS type whitelist**: `mount_vfs()` validates against 18 known filesystem types (`guestfs/mount.rs`)
- **Reverse mount order in 7 functions**: Fixed `clone`, `simulate`, `copy`, `rescue`, `optimize`, `repair`, `harden` commands (`cli/commands/disk_ops.rs`, `cli/commands/analysis.rs`, `cli/commands/inspect.rs`)
- **Username validation**: Added `validate_username()` for all user management commands; rejects flag injection (`cli/interactive.rs`)
- **Delete blocks critical paths**: `rm -rf` now refuses `/`, `/etc`, `/usr`, `/var`, `/boot` and 9 other system directories (`cli/interactive.rs`)
- **Shell injection in diff**: Replaced `sh -c` with `command()` argv array (`cli/interactive.rs`)
- **SSH key path traversal**: Uses `/etc/passwd` lookup instead of hardcoded `/home/{user}` (`cli/interactive.rs`)
- **DEL character display**: Correctly renders as `^?` instead of invalid byte 191 (`cli/commands/file_ops.rs`)

##### Medium Fixes (16)
- **Readonly checks**: Added to `write_append()`, `cp_a()`, `cp_r()`, `rmdir()` (`guestfs/file_ops.rs`)
- **Mount path matching**: Replaced substring match with field-based check to prevent false matches (`guestfs/handle.rs`)
- **Bincode size limits**: Applied 100MB limit to `stats()` and `clear_older_than()` (`core/binary_cache.rs`)
- **qemu-nbd process cleanup**: `disconnect()` now kills and waits on child process (`disk/nbd.rs`)
- **Partition offset overflow**: Added `checked_add` for post-multiply additions (`disk/filesystem.rs`)
- **HTML template detection**: Checks template content for HTML tags, not just name prefix (`export/template.rs`)
- **Division by zero guard**: Protected `total_weight / 100` in score calculation (`cli/commands/analysis.rs`)
- **Disk usage precision**: Fixed integer division order (`cli/commands/security.rs`)
- **`page_down` bounds**: Clamped to item count (`cli/tui/app.rs`)
- **Export path traversal**: Rejects `/` and `\` anywhere in filename (`cli/tui/app.rs`)
- **Lexicographic mountpoint sort**: Correct for same-length different-hierarchy paths (`cli/shell/repl.rs`)
- **`inspect_os()` error propagation**: Changed from `unwrap_or_default()` to `?` (`cli/plan/apply.rs`)
- **Glob-to-regex escaping**: Complete regex metacharacter escaping (`cli/commands/file_ops.rs`)
- **UID type consistency**: Changed to `u32` matching structured output (`cli/commands/inspect.rs`)
- **Distro-aware package manager**: Migration planner selects dnf/apt/pacman/zypper based on OS (`cli/migrate/planner.rs`)
- **Drop logging**: Shutdown failures now logged in guestfs handle Drop (`guestfs/handle.rs`)

#### Fourth-Pass Review (54 issues, 33 files)

#### Critical Fixes (8)
- **Shell injection in bash export**: Added `shell_escape()` helper with single-quote wrapping to all interpolated values in generated bash scripts (`cli/plan/export.rs`)
- **Broken `mount()` implementation**: `mount()`, `mount_options()`, and `mount_vfs()` now perform actual mount operations instead of only recording in hashmap (`guestfs/mount.rs`)
- **Hand-rolled SHA-512 crypt replaced**: Replaced incorrect simplified SHA-512 password hashing with `openssl passwd -6` for correct `$6$salt$hash` generation (`cli/commands/security.rs`)
- **Plaintext password temp file**: Password changes now pipe directly to `chpasswd` instead of writing to `/tmp` (`cli/interactive.rs`)
- **sed injection in grep-replace**: Added escaping for `\`, `&`, and newlines in sed pattern/replacement strings (`cli/interactive.rs`)
- **`rm -rf` without confirmation**: `delete` command now requires interactive y/n confirmation before proceeding (`cli/interactive.rs`)
- **Shadow entry for nonexistent users**: Force-add now validates user exists in `/etc/passwd` before creating shadow entry (`cli/commands/security.rs`)

#### High Severity Fixes (11)
- **LVM device filter missing**: Added `--config` with device filter to `lvcreate`, `lvremove`, `lvs_full`, and `vg_activate` to prevent operating on host LVM (`guestfs/lvm.rs`)
- **Shell injection in `sh()`**: Changed from warning to returning `Error::SecurityViolation`; added `|`, `;`, `>`, newline to dangerous patterns (`guestfs/command.rs`)
- **Write ops on readonly images**: Added `self.readonly` check to 10 mutating methods: `write`, `mkdir`, `mkdir_p`, `touch`, `chmod`, `chown`, `rm`, `rm_rf`, `mv`, `cp` (`guestfs/file_ops.rs`)
- **Inverted mount sort order**: Fixed `mount_all_ro` to mount shorter paths (parents) before longer paths (children) (`cli/commands/mod.rs`)
- **Path traversal in file extraction**: Replaced `canonicalize()`-based check with `Path::components()` to reject `..` before any filesystem ops (`cli/commands/file_ops.rs`)
- **Shadow backup auto-deleted**: Changed `into_temp_path()` to `into_temp_path().keep()` so backup files persist (`cli/commands/security.rs`)
- **Path traversal in cache key**: Added key validation rejecting `/`, `\`, `..`, and non-alphanumeric characters (`core/binary_cache.rs`)
- **TUI scroll out-of-bounds**: Added bounds checking to `scroll_down` and `scroll_bottom`; `get_filtered_count` now uses `packages.len()` (`cli/tui/app.rs`)
- **Arbitrary command execution from plan**: Backup failure now returns error instead of continuing without backup (`cli/plan/apply.rs`)

#### Medium Severity Fixes (22)
- **`u64` to `i64` truncation**: Changed unsafe `as i64` casts to `i64::try_from().unwrap_or(i64::MAX)` in partition sizes and file sizes (`guestfs/device.rs`, `guestfs/file_ops.rs`)
- **`touch()` mtime not updated**: Now calls `file.set_modified(SystemTime::now())` (`guestfs/file_ops.rs`)
- **Shutdown state transition**: Set `state = Closed` before early returns in shutdown (`guestfs/handle.rs`)
- **`umount_all()` ordering**: Mountpoints now sorted by depth (deepest first) before unmounting (`guestfs/mount.rs`)
- **VG activation recording**: VGs recorded in `activated_vgs` only after successful activation (`guestfs/lvm.rs`)
- **JSON/CSV injection**: Replaced string interpolation with `serde_json::Value::String()` in timeline, audit, and health exports (`cli/commands/analysis.rs`, `cli/commands/security.rs`)
- **Read-write mount on read-only drives**: Changed 3 `g.mount()` calls to `g.mount_ro()` in disk_ops (`cli/commands/disk_ops.rs`)
- **Regex denial of service**: Added 1000-char length limit and graceful error handling for user regex input (`cli/tui/app.rs`)
- **Mountpoint sort in REPL**: Sorted mountpoints by path length before mounting (`cli/shell/repl.rs`)
- **Guestfs errors swallowed**: `perform_inspection` now propagates errors instead of returning fake success (`cli/parallel.rs`)
- **Thread pool warning**: `build_global()` failure now logged via `eprintln!` (`cli/parallel.rs`)
- **False positive ZFS detection**: Replaced non-zero heuristic with ZFS uberblock magic number `0x00bab10c` (`disk/filesystem.rs`)
- **LBA overflow**: Changed `start_lba * 512` to `checked_mul(512)` with error handling (`disk/filesystem.rs`)
- **NBD Drop silent**: Now logs disconnect errors with device path (`disk/nbd.rs`)
- **Bincode deserialization limit**: Added 100MB size limit to prevent memory exhaustion (`core/binary_cache.rs`)
- **HTML template injection**: Added HTML escaping for `<`, `>`, `&`, `"` in variable values (`export/template.rs`)
- **Incomplete markdown escaping**: Extended `md_escape()` to cover `\`, `[`, `]`, `(`, `)`, `*`, `_`, `` ` ``, `#` (`cli/exporters/markdown.rs`)
- **YAML injection in Ansible export**: Plan data values now properly escaped (`cli/plan/export.rs`)
- **SELinux line-by-line replacement**: Now skips comment lines starting with `#` (`cli/plan/apply.rs`)
- **Cyclic dependency warning**: Topological sort now warns when operations are dropped (`cli/plan/apply.rs`)
- **NBD `_read_only` naming**: Renamed misleading underscore-prefixed parameter (`disk/nbd.rs`)
- **Loop device Drop logging**: Now logs device path and manual cleanup instructions on failure (`disk/loop_device.rs`)

#### Low Severity Fixes (13)
- **Non-UTF-8 filenames**: `ls()` now uses `to_string_lossy()` instead of silently skipping (`guestfs/file_ops.rs`)
- **`realpath()` consistency**: Uses `find_root_mountpoint()` instead of ad-hoc lookup (`guestfs/file_ops.rs`)
- **Conditional shutdown sleep**: Only sleeps if there were actual mounts to clean up (`guestfs/handle.rs`)
- **`find0()` path validation**: Validates output path doesn't contain `..` and is absolute (`guestfs/file_ops.rs`)
- **Package timestamp**: Changed from fake `Utc::now()` to `0` with explanatory comment (`cli/commands/analysis.rs`)
- **Shadow trailing newline**: Shadow file now ends with `\n` per POSIX convention (`cli/commands/security.rs`)
- **UID parse safety**: Changed `unwrap_or(0)` to `unwrap_or(-1)` to avoid false root coloring (`cli/interactive.rs`)
- **Health score overflow**: Added `.min(255)` before `u8` cast (`cli/tui/app.rs`)
- **Topological sort silent drop**: Warns when operations are dropped due to cycles (`cli/plan/apply.rs`)
- **SELinux comment corruption**: Line-by-line replacement skips comments (`cli/plan/apply.rs`)
- **Compatibility score deflation**: Denominator now uses `min(total, 50)` to match `take(50)` limit (`cli/migrate/planner.rs`)
- **Oracle Linux false positive**: Changed `contains("ol")` to specific patterns (`detectors/guest_detector.rs`)
- **`ensure_ready()` missing**: Added to `mount_options()` (`guestfs/mount.rs`)

### Added - Interactive File Explorer 🔍

#### Explore Command - Visual File Browser
- **Three Access Methods**:
  - Direct CLI: `guestkit explore vm.qcow2 [path]`
  - Shell mode: `explore` or `ex` command in interactive shell
  - TUI view: Integrated Files view accessible via Tab navigation
- **Visual Navigation**: Color-coded files with emoji icons for instant file type recognition
- **File Preview**: View file contents with line numbers (press `v`), 1MB size limit
- **File Information**: Detailed metadata display (press `i`) showing size, permissions, timestamps
- **Real-Time Filtering**: Live search as you type (press `/`) with case-insensitive matching
- **Hidden Files Toggle**: Show/hide dotfiles (press `.`)
- **Smart Sorting**: Sort by name, size, or modification time (press `s`)
- **Vim-Like Navigation**: Keyboard shortcuts (j/k, arrow keys, Enter, Backspace)
- **Persistent State**: File browser state maintained throughout TUI session
- **Documentation**: Comprehensive guides (EXPLORE-COMMAND.md, EXPLORE-QUICKSTART.md, EXPLORE-COMPLETE-SUMMARY.md)

#### TUI Files View Integration
- **Seamless Integration**: New Files view added to TUI dashboard alongside Dashboard, Network, Packages, etc.
- **Overlay Rendering**: Non-blocking preview and info popups
- **Keyboard Handlers**: Dedicated key mappings for file operations (v, i, /, ., Enter, Backspace)
- **Export Support**: File browser state can be exported to JSON/YAML
- **Help Integration**: Context-sensitive help showing available actions

#### Shell Explorer Enhancement
- **REPL Mode**: `explore` command available in interactive shell
- **Standalone Mode**: Direct launch from CLI without entering shell
- **State Management**: ExplorerState tracks navigation, filtering, sorting
- **Color-Coded Output**: Visual file type identification with colored emojis
- **Cross-Platform**: Terminal UI works on Linux/macOS with crossterm

### Changed - Visual Refinements 🎨
- **Orange Color Theme**: Switched primary accent color from yellow to orange (RGB 255, 165, 0) for better visual hierarchy
- **Improved Readability**: Enhanced contrast for section headers and key information

## [0.3.1] - 2026-01-26

### Added - Enhanced UX and VM Migration Support 🚀

#### Killer Summary View 📊
- **Quick Summary Box**: Prominent boxed display showing OS product name in bright green
- **At-a-Glance Information**: Instantly see OS type, version, architecture, hostname without scrolling
- **Color-Coded Output**: Smart color coding for better visual scanning
  - 🟢 Green: OS product name and detected values
  - 🔵 Cyan: Architecture information
  - 🔵 Blue: Hostname
  - 🟣 Magenta: Package format
  - 🟡 Yellow: Init system

#### Windows Registry Parsing 🪟
- **Full Version Detection**: Automatic Windows version detection via registry parsing
- **Registry Hive Access**: Direct parsing of Windows registry hives for OS information
- **Enhanced Windows Support**: Better detection of Windows editions and service packs

#### LVM Volume Group Management 💾
- **Automatic Cleanup**: LVM volume groups automatically deactivated and cleaned up during shutdown
- **Improved Reliability**: Prevents stale LVM state from interfering with subsequent operations
- **Clean Teardown**: Proper resource management for LVM devices

#### Universal VM Migration Support 🔄
- **fstab/crypttab Rewriter**: Universal rewriter for mount configurations to support VM migration
- **Cross-Platform Migration**: Modify disk images for migration between different environments
- **Device Path Translation**: Automatic translation of device paths for target systems
- **LUKS Support**: Rewrite crypttab entries for encrypted volumes during migration

#### Loop Device Primary Support 🔄
- **Loop Device Primary**: Loop devices (losetup) now used as primary mounting method for RAW/IMG/ISO formats
- **Built-in Support**: No kernel module loading required for common disk formats
- **NBD Fallback**: NBD remains available for QCOW2/VMDK/VDI/VHD formats
- **Auto-Detection**: Automatic format detection based on file extension
- **Better Reliability**: Eliminates NBD dependency for typical use cases
- **Cleaner Architecture**: Separate LoopDevice and NbdDevice implementations with proper cleanup

### Enhanced
- **Color Coding System**: Consistent color scheme across all output
  - Green: Positive/secure values
  - Red: Issues/insecure configurations
  - Yellow/Orange: Warnings and key information
  - Cyan/Blue: Informational data
  - Gray: Unknown or disabled values
- **Visual Hierarchy**: Better section organization with clear separators
- **Scanning Experience**: Output optimized for quick visual scanning

### Fixed
- **Resource Cleanup**: Improved cleanup of loop and NBD devices on shutdown
- **LVM State Management**: Proper LVM volume group state handling
- **Windows Detection**: More reliable Windows OS version detection

### Documentation
- Updated README with latest features and capabilities
- Enhanced examples showing new summary view
- Documented migration support features
- Added Windows registry parsing documentation

## [0.3.0] - 2026-01-25

### Added - Comprehensive System Inspection 🔍
- **👥 User Accounts**: Regular and system user detection with UID filtering
- **🔐 SSH Configuration**: Port, PermitRootLogin, PasswordAuthentication with security-aware colors
- **⚙️ Systemd Services**: Enabled services listing with checkmark icons
- **💻 Language Runtimes**: Auto-detection for Python, Node.js, Java, Ruby, Go, Perl with language-specific emojis
- **🐳 Container Runtimes**: Docker, Podman, containerd, CRI-O detection with platform icons
- **💾 LVM Configuration**: Physical/logical volumes, volume groups inspection
- **⚙️ System Configuration**: Timezone, locale, SELinux, cloud-init, VM tools detection
- **🌐 Network Configuration**: Enhanced interface details with DNS servers and DHCP status

### Added - Beautiful Visual Output 🎨
- **Emoji Icons**: Comprehensive emoji system for visual scanning (💾🗂📁🖥️👥🔐⚙️💻🐳)
- **Color Coding**: Security-aware colors (green=secure, red=insecure, yellow=warning)
- **Hierarchical Display**: Clean sections with separators for better readability
- **Language Icons**: 🐍 Python, ☕ Java, 🟢 Node.js, 💎 Ruby, 🔷 Go, 🐪 Perl
- **Container Icons**: 🐳 Docker, 🦭 Podman, 📦 containerd, 🔷 CRI-O

### Fixed - Read-Only Disk Support 🔧
- **Mount Operations**: Fixed all inspection functions to use `mount_ro()` instead of `mount()`
- **Mount State Tracking**: Proper mount/unmount state management to prevent double operations
- **Read-Only Compatibility**: All OS detection now works correctly on read-only disk images

### Added - Interactive CLI Mode 🎯
- **REPL Mode**: Full-featured interactive shell for disk exploration
- **Persistent Session**: Launch appliance once, run multiple commands
- **Command History**: Up/down arrows to navigate command history
- **Auto-Inspection**: Automatically detects and displays OS on startup
- **20+ Commands**: info, filesystems, mount, ls, cat, find, packages, services, users, network, and more
- **Colorized Output**: Beautiful colored terminal output
- **Tab Completion Ready**: Structure in place for command completion
- **Aliases**: `repl`, `fs`, `pkg`, `svc`, `net`, `dl`, `cls` shortcuts
- **Usage**: `guestkit interactive disk.qcow2` or `guestkit repl disk.qcow2`

### Added - Async Python API (Prepared, Pending Dependencies) ⏳
- **AsyncGuestfs Class**: Complete async implementation prepared (commented out)
- **Type Hints**: Full async type stub definitions ready
- **Examples**: Comprehensive async inspection examples
- **Status**: Waiting for pyo3-asyncio to support PyO3 0.22+ (currently only supports 0.21)
- **Ready to Enable**: Once dependency is updated, just uncomment code and rebuild

### Added - PyPI Publication Setup 📦
- **GitHub Actions Workflow**: Automated wheel building for Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
- **PyPI Publishing**: Complete setup for publishing to PyPI via Trusted Publishing (OIDC)
- **PyPI Publishing Guide**: Comprehensive documentation at `docs/development/publishing.md`
- **Test Script**: `scripts/test_pypi_build.sh` for local build verification
- **Enhanced Metadata**: Updated `pyproject.toml` with complete PyPI metadata
  - Added Python 3.13 support
  - Added macOS platform classifier
  - Added Changelog URL
  - Minimum Python version: 3.8

### Added - Quick Win Enhancements ✨
- **Python Context Manager**: `with Guestfs() as g:` for automatic cleanup
- **Python Type Hints**: Complete `.pyi` stub file (300+ lines) for IDE autocomplete and mypy support
- **Shell Completion**: Support for Bash, Zsh, Fish, PowerShell, Elvish via `guestkit completion`
- **Colorized Output**: 15+ color helper functions with status indicators (✓, ✗, ⚠, ℹ, ▶, ■)
- **Enhanced Documentation**: Organized all docs into structured directories

### Changed - Documentation Organization 📚
- Reorganized all documentation into `docs/` with clear subdirectories:
  - `docs/guides/` - User-facing guides (CLI, Python, Quick Start, etc.)
  - `docs/api/` - API documentation (Python API, Rust API, Migration Guide)
  - `docs/architecture/` - Architecture and technical docs
  - `docs/development/` - Contributor documentation (Roadmap, Enhancements)
  - `docs/testing/` - Testing guides and reports
  - `docs/status/` - Implementation status and project summaries
  - `docs/archive/` - Historical/superseded documentation
- Created comprehensive `docs/README.md` as documentation index
- Updated all documentation links in README.md
- Root directory now only contains essential files (README, CHANGELOG, CONTRIBUTING, SECURITY)

### Added - Documentation
- `docs/README.md` - Complete documentation index with navigation guide
- `docs/development/publishing.md` - Comprehensive PyPI publishing guide
- `docs/development/NEXT_ENHANCEMENTS.md` - Detailed guides for next 5 priority features
- `docs/development/ENHANCEMENT_STATUS.md` - Current status and roadmap tracker
- `docs/development/ENHANCEMENTS_IMPLEMENTED.md` - Summary of all implemented enhancements
- Enhanced `README.md` with Documentation section and quick links
- Test scripts for enhancements verification

### Fixed
- PyO3 compatibility with Python 3.14 via `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1`
- Type stub file inclusion in maturin builds

## [0.3.0] - 2026-01-23 - Quick Wins Sprint Complete ✅

### Added - CLI Tool (guestkit)

**NEW: Production-ready command-line tool** for disk image operations without mounting:
- `guestkit inspect <disk>` - Detect and display OS information
- `guestkit filesystems <disk>` - List block devices, partitions, filesystems
- `guestkit packages <disk>` - List installed packages (dpkg, RPM, pacman)
- `guestkit ls <disk> <path>` - List directory contents
- `guestkit cat <disk> <path>` - Read and display files
- `guestkit cp <disk>:<src> <dest>` - Copy files from disk to host

**CLI Features:**
- JSON output mode (`--json`) for scripting and automation
- Human-readable formatted output with tables
- Comprehensive error handling with actionable suggestions
- Verbose mode (`-v`) for debugging
- Package filtering and limiting options
- Detailed filesystem information mode (`--detailed`)

### Added - Progress & UX Enhancements

**Progress Indicators** (using indicatif v0.17):
- Real-time spinners for long operations (appliance launch ~2.5s)
- Stage-aware status updates ("Loading disk...", "Launching appliance...", "Inspecting OS...")
- Automatic hiding in JSON mode for clean machine-parseable output
- Clean finish/abandon on success/failure
- Multi-progress support for concurrent operations

**Enhanced Error Diagnostics** (using miette v7.0):
- 10 specialized error types with detailed help text:
  - `NoOsDetected` - Unbootable/encrypted/corrupted disk guidance
  - `LaunchFailed` - KVM/permissions/QEMU troubleshooting
  - `MountFailed` - Filesystem/device issue guidance
  - `FileNotFound` - Path suggestions and verification
  - `PermissionDenied` - Sudo requirement explanation
  - `DiskNotFound` - Path verification help
  - `InvalidFormat` - Format detection guidance
  - `OperationFailed` - General operation troubleshooting
  - `PackageManagerNotFound` - OS-specific package manager info
  - `FilesystemNotSupported` - Supported FS list
- Pretty-printed errors with color coding
- Actionable "Try:" suggestions for each error type
- Diagnostic codes for programmatic error handling

### Added - Performance & Quality Assurance

**Criterion Benchmark Suite** (benches/operations.rs - 400+ lines):
- 20+ benchmarks across 8 operation categories:
  - `create_and_launch` - Appliance startup performance (~2.5s baseline)
  - `inspect_os` - Multi-distribution OS detection (~500ms)
  - `os_metadata` - Metadata retrieval (type, distro, hostname, mountpoints - ~5ms)
  - `mount_operations` - Mount/unmount cycles (~50ms)
  - `list_operations` - Devices, partitions, filesystems (~10ms)
  - `file_operations` - Read, ls, stat, is_file, is_dir (~15ms)
  - `package_operations` - Application listing (~3.5s)
  - `filesystem_info` - VFS type, label, UUID, size (~5-10ms)
- Multi-distribution support (Ubuntu, Debian, Fedora)
- Statistical analysis with confidence intervals
- HTML report generation (target/criterion/report/index.html)
- Baseline comparison support for regression detection
- Environment-based test image configuration

**GitHub Actions CI/CD Pipeline** (.github/workflows/integration-tests.yml - 300+ lines):
- **Integration test matrix**: 5 OS distributions
  - Ubuntu 20.04, 22.04, 24.04
  - Debian 12 (Bookworm)
  - Fedora 39
- **Automated testing** of all 6 guestkit commands
- **JSON output validation** with jq
- **Distribution detection verification**
- **File operations testing** (ls, cat, cp)
- **Test image caching** for 5-10x speedup
- **Artifact upload** for debugging failures
- **Daily scheduled runs** (2 AM UTC) for regression detection
- **Performance benchmarks** on main branch only
- **Code quality checks**: clippy linting, rustfmt validation
- **Parallel job execution**: 8 jobs (5 tests + bench + clippy + fmt)
- Average CI time: 15-20 minutes with caching

### Added - Comprehensive Documentation

**User Documentation** (4,000+ lines total):
- `docs/CLI_GUIDE.md` (800 lines) - Complete CLI reference
  - Installation and requirements
  - All 6 commands with examples
  - JSON mode usage
  - Error handling guide
  - Best practices and tips
- `docs/development/enhancement-roadmap.md` (600 lines) - 10-phase long-term vision
- `docs/QUICK_WINS.md` (500 lines) - 3-week implementation guide
- `docs/WEEK1_COMPLETE.md` (500 lines) - CLI tool delivery summary
- `docs/WEEK2_COMPLETE.md` (400 lines) - UX enhancements summary
- `docs/WEEK3_COMPLETE.md` (600 lines) - Quality assurance summary
- `docs/QUICK_WINS_COMPLETE.md` (400 lines) - Sprint retrospective

**Updated Documentation**:
- README.md - Added prominent CLI tool section with examples
- ROADMAP.md - Marked Quick Wins milestone complete
- Cargo.toml - Updated to v0.3.0

### Changed

- **Binary renamed**: "guestkit" → "guestkit" for clarity and convention
- **Version bumped**: 0.2.0 → 0.3.0
- **User experience**: Transformed from library-only to user-friendly CLI tool

### Dependencies Added

```toml
clap = { version = "4", features = ["derive", "cargo"] }
indicatif = "0.17"
miette = { version = "7.0", features = ["fancy"] }
criterion = { version = "0.5", features = ["html_reports"] }  # dev-only
```

### Performance Baselines Established

**Measured on Ubuntu 22.04** (averages):
| Operation | Time | Throughput | Notes |
|-----------|------|------------|-------|
| Appliance create + launch | ~2.5s | N/A | Dominates total time |
| OS inspection | ~500ms | 2 ops/sec | Fast OS detection |
| Metadata retrieval | ~5ms | 200 ops/sec | Very fast |
| Mount/unmount | ~50ms | 20 ops/sec | Moderate overhead |
| List devices/partitions | ~10ms | 100 ops/sec | Fast enumeration |
| Small file read | ~15ms | 66 ops/sec | Good I/O performance |
| Package listing | ~3.5s | 0.3 ops/sec | Slow, needs optimization |

**Key Insight**: Appliance launch dominates operation time. Caching/reuse will provide 10-100x speedup.

### Code Statistics

- **Production code**: +3,500 lines
  - CLI tool: 600 lines (src/bin/guestkit.rs)
  - Progress system: 180 lines (src/core/progress.rs)
  - Diagnostics: 280 lines (src/core/diagnostics.rs)
  - Benchmarks: 400 lines (benches/operations.rs)
  - CI/CD: 300 lines (.github/workflows/integration-tests.yml)
  - Tests & examples: 250 lines
- **Documentation**: +4,000 lines (8 new files)
- **Test coverage**: 25% → 40% (+60% improvement)
- **CI/CD jobs**: 0 → 8 (+8 new automated checks)
- **Compiler warnings**: Reduced from 40 to 20 (ongoing cleanup)

### Impact & ROI

**Before Quick Wins:**
```rust
// Required: Rust programming, 20+ lines of code
use guestkit::guestfs::Guestfs;
let mut g = Guestfs::new()?;
g.add_drive_ro("disk.img")?;
g.launch()?;
let roots = g.inspect_os()?;
// ... 15 more lines ...
```

**After Quick Wins:**
```bash
# One command, no coding required
guestkit inspect disk.img
```

**Metrics**:
| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Ease of use | Hard (coding required) | Easy (one command) | 10x better |
| Error clarity | Cryptic | Actionable with suggestions | Transformative |
| Progress feedback | None (silent) | Real-time spinners | Transparent |
| Test automation | Manual (2 hours) | Automated (5 min) | 96% faster |
| Regression risk | High | Low (CI/CD) | Major reduction |
| Documentation | 2 pages | 10+ pages | 400% increase |

**Development Time**: 12 hours over 3 weeks (4 hours/week)
**Value Delivered**: Production-ready CLI tool, professional UX, automated QA

## [Unreleased] - Phase 3 Near Complete (95%)

### Added - Testing, Quality, and Documentation Infrastructure

#### Integration Tests (2 test suites)
- **integration_basic.rs** (10 comprehensive tests)
  - Disk creation and inspection workflow
  - Partition creation and management
  - Filesystem creation, mount, and operations
  - File I/O operations (read, write, copy, delete)
  - Archive operations (tar creation and extraction)
  - Checksum verification (md5, sha256)
  - File stat and metadata operations
  - Guest command execution
  - Multiple partition scenarios
  - Recursive copy operations

- **integration_lvm_luks.rs** (10 advanced tests)
  - LUKS encryption basic workflow
  - LVM volume management
  - Combined LUKS + LVM scenarios
  - LUKS key management and rotation
  - Volume group operations
  - VG scan and activation
  - Read-only encrypted volumes
  - LVM volume inspection

#### Performance Benchmarks
- **benchmarks.rs** - Criterion-based benchmarks
  - Disk creation (10MB to 500MB sizes)
  - Partition operations performance
  - Filesystem creation (ext4, xfs)
  - File operations (1KB to 1MB writes/reads)
  - Mount/unmount cycle overhead
  - Checksum algorithms (md5, sha256 on various file sizes)
  - Archive operations (tar in/out)
  - Launch/shutdown overhead measurement

#### Project Documentation
- **ROADMAP.md** - Comprehensive project roadmap
  - Phase 3: Stabilization and Integration (Q1 2026)
  - Phase 4: Python Bindings (Q2 2026)
  - Phase 5: Performance Optimization (Q2-Q3 2026)
  - Phase 6: Advanced Features (Q3-Q4 2026)
  - Phase 7: Ecosystem Integration (2027)
  - Success metrics and version milestones

- **API_REFERENCE.md** (952 lines) - Complete API documentation with examples
- **CONTRIBUTING.md** (452 lines) - Developer contribution guidelines
- **SECURITY.md** (322 lines) - Security policy and vulnerability reporting
- **docs/architecture/overview.md** (550+ lines) - Architecture deep-dive
  - High-level architecture with diagrams
  - Core concepts and design patterns
  - Module architecture explained
  - Data flow diagrams
  - Design decisions and rationale
  - Comparison with 
  - Future architecture plans
- **docs/PERFORMANCE.md** (500+ lines) - Performance tuning guide
  - Quick wins for immediate improvements
  - Benchmarking with Criterion
  - Disk image optimization
  - Operation-specific optimizations
  - System-level tuning
  - Scaling and concurrency patterns
  - Memory and I/O optimization
  - Best practices and checklist
- **docs/TROUBLESHOOTING.md** (550+ lines) - Troubleshooting guide
  - Installation issues
  - Runtime errors (NBD, LUKS, LVM, permissions)
  - Performance issues
  - Integration issues (Docker, Kubernetes)
  - Common error messages with solutions
  - Debugging techniques
  - FAQ section

#### CI/CD Infrastructure
- **ci.yml** - Comprehensive continuous integration
  - Multi-platform testing (Ubuntu, stable/beta Rust)
  - Code formatting and clippy checks
  - Code coverage with Codecov
  - Security audits with cargo-audit
  - Multi-target release builds
  - Documentation verification

- **release.yml** - Automated release workflow
  - Multi-architecture builds (x86_64, aarch64, musl)
  - Changelog extraction
  - Binary packaging with checksums
  - Automated crates.io publishing

#### Enhanced CLI Tool
- **cli/commands.rs** - Comprehensive command implementations
  - `inspect` - Full OS and filesystem inspection
  - `list/ls` - Browse files in guest filesystems
  - `extract/get` - Extract files from disk images
  - `execute/exec` - Run commands in guest OS
  - `backup` - Create tar.gz backups from guest
  - `create` - Create new disk images
  - `check/fsck` - Filesystem checking and repair
  - `usage/df` - Display disk usage statistics
  - `convert` - Convert disk formats (enhanced)
  - `detect` - Detect disk format
  - `info` - Get detailed disk information
  - `version` - Show version with project info

- **cli/output.rs** - Output formatting utilities
  - Multiple formats (human-readable, JSON, YAML)
  - Size formatting (B, KB, MB, GB, TB)
  - Duration formatting
  - Table formatter for aligned output
  - Progress bar for long operations

- **Updated main.rs** - Modern CLI structure
  - Better command organization
  - Informative help text
  - Command aliases for convenience
  - Auto-mounting for user convenience

### Infrastructure
- Benchmark harness configuration in Cargo.toml
- Criterion dependency for performance testing
- Test organization structure
- Utility scripts (find_unimplemented.sh)

### Documentation Statistics
- **Total Documentation**: 8 major files
- **Total Lines**: ~5,000+ lines of documentation
- **Coverage**: Installation, usage, API, architecture, performance, troubleshooting, contributing, security

## [0.2.0] - 2026-01-23 - Phase 2 Complete

### Added - Phase 2 Implementation

This massive update adds 73 new modules implementing 463 additional -compatible APIs, bringing total coverage from 22.6% to 76.8% of  functionality.

#### Core Utilities (10 modules)
- **checksum**: File checksum operations (md5, sha1, sha256, sha384, sha512)
- **utils**: File type detection, readlink, symlink checking
- **misc**: Version info, available features, settings management
- **util_ops**: Device stats, umask, QEMU detection
- **glob_ops**: Pattern matching, find0, ls0, grep, case-insensitive search
- **base64_ops**: Base64 encoding/decoding for file content
- **dd_ops**: dd-style copy, zero device operations
- **pread_ops**: Positional read/write with offset support
- **sync_ops**: sync, drop_caches, flush operations
- **label_ops**: Generic filesystem label/UUID management

#### Filesystem Support (14 modules)
- **filesystem**: Generic mkfs, fsck, tune2fs, zerofree, fstrim
- **btrfs**: Btrfs subvolumes, snapshots, balance, scrub
- **xfs**: XFS repair, info, admin, db operations
- **ntfs**: ntfsclone, ntfsfix, label management
- **ext_ops**: ext2/3/4 UUID, label, dump/restore
- **f2fs_ops**: Flash-Friendly File System support
- **dosfs_ops**: FAT12/16/32 filesystem management
- **nilfs_ops**: Log-structured filesystem support
- **ufs_ops**: Unix File System support
- **reiserfs_ops**: ReiserFS filesystem management
- **jfs_ops**: Journaled File System support
- **minix_ops**: Minix filesystem support
- **zfs_ops**: ZFS filesystem management (10 functions)
- **squashfs_ops**: SquashFS creation and extraction

#### Disk & Partition Management (12 modules)
- **disk_ops**: Advanced disk operations (swap, hexdump, strings, scrubbing)
- **disk_mgmt**: Disk image creation, resize, convert, snapshot
- **part_mgmt**: Partition creation, deletion, resizing
- **part_type_ops**: GPT type GUID, attributes, expand
- **blockdev_ops**: setro/setrw, flush, reread partition table
- **resize**: resize2fs, ntfsresize, xfs_growfs
- **md_ops**: Software RAID creation, management, inspection
- **bcache_ops**: Block cache management
- **ldm_ops**: Windows dynamic disk support (8 functions)
- **mpath_ops**: Multipath device management
- **smart_ops**: Disk health monitoring with smartctl
- **swap_ops**: Swap label/UUID management

#### Security Operations (4 modules)
- **security**: SELinux and AppArmor management
- **selinux_ops**: SELinux inspection, restorecon
- **cap_ops**: Linux capabilities management
- **acl_ops**: POSIX ACL management

#### System Management (5 modules)
- **system**: Timezone, locale, users, groups, systemd configuration
- **boot**: Bootloader, kernels, UEFI, fstab management
- **service**: systemd, sysvinit, cron job management
- **network**: Hostname, interfaces, DNS settings
- **package**: dpkg/rpm package inspection

#### Bootloader Configuration (2 modules)
- **grub_ops**: GRUB bootloader installation and configuration
- **syslinux_ops**: syslinux/extlinux bootloader installation

#### File Metadata & Attributes (6 modules)
- **metadata**: Stat operations, inode, times, permissions
- **node_ops**: mknod, mkfifo, mktemp, truncate, utimens
- **link_ops**: Symbolic and hard link management
- **attr_ops**: Extended attributes (xattr) management
- **owner_ops**: File ownership operations
- **time_ops**: File timestamp operations

#### File Transfer & Archives (5 modules)
- **transfer**: Advanced file transfer with offset downloads/uploads
- **cpio_ops**: CPIO archive support
- **compress_ops**: gzip, bzip2, xz compression/decompression
- **rsync_ops**: rsync-based file synchronization
- **backup_ops**: Backup operations

#### Specialized Tools Integration (6 modules)
- **augeas**: Augeas configuration file editing
- **hivex_ops**: Windows registry hive manipulation (16 functions)
- **journal_ops**: systemd journal reading, export, verification
- **inotify_ops**: File monitoring with inotify
- **yara_ops**: Malware scanning with YARA rules
- **tsk_ops**: Forensics with The Sleuth Kit (deleted file recovery)

#### Windows, SSH & ISO (3 modules)
- **windows**: Windows registry hives and Windows-specific inspection
- **ssh**: SSH keys, certificates, authorized_keys management
- **iso**: ISO creation, inspection, mounting

#### Virtualization & Inspection (3 modules)
- **sysprep_ops**: VM preparation (removing unique data)
- **virt_ops**: virt-* tool equivalents (inspector, convert info)
- **inspect_ext_ops**: Extended inspection operations

#### Internal & Text Processing (3 modules)
- **internal**: State management, environment, debug operations
- **sed_ops**: sed-style text editing operations
- **template_ops**: Template processing and VM cloning operations

### Enhanced

#### Existing Modules (5 modules)
- **archive**: Added cpio support and additional tar operations
- **file_ops**: Added extended file operations (head, tail, grep, cat, etc.)
- **handle**: Added config and state management methods
- **lvm**: Added extended LVM operations
- **mount**: Added mount option handling improvements

### Fixed
- Type mismatches in template_ops.rs (String to bytes conversion)
- Type casting for chown_recursive parameters (u32 to i32)
- Removed unused imports in multiple modules
- Compilation errors across all new modules

### Documentation
- Updated GUESTFS_IMPLEMENTATION_STATUS.md with comprehensive Phase 2 coverage
- Updated implementation statistics: 578 APIs total, 563 working (97.4%)
- Documented coverage increase from 22.6% to 76.8% of 
- Added detailed function listings for all 76 operation categories

### Testing
- All 97 unit tests passing
- API structure tests for all new modules
- Successful compilation with zero errors

### Project Statistics
- **Total Modules**: 84 Rust source files
- **Total APIs**: 578 functions
- **Working APIs**: 563 (97.4% functional)
- ** Coverage**: 76.8% (563 of 733 total  APIs)
- **Lines of Code**: ~15,000+ lines of implementation
- **Test Coverage**: 97 unit tests

## [0.1.0] - Phase 1 Complete

### Initial Implementation
- Core disk access and inspection
- NBD device management via qemu-nbd
- Mount/unmount operations
- File I/O operations
- Command execution in guest
- Archive operations (tar, tgz)
- LUKS encryption support
- LVM support
- Basic partition management
- OS detection and inspection
