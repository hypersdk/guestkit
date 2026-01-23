# GuestKit Quick Wins - High Impact, Low Effort

This document outlines the highest-value enhancements that can be implemented quickly (1-2 weeks each) to significantly improve GuestKit.

## Priority 1: CLI Tool (⚡ Highest Impact)

**Why:** Immediately usable by non-programmers, great for demos, enables scripting.

**Effort:** 1 week
**Impact:** ⭐⭐⭐⭐⭐

### Implementation

Create `src/bin/guestkit.rs`:

```rust
use clap::{Parser, Subcommand};
use guestkit::guestfs::Guestfs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "guestkit", version, about = "Inspect and manipulate disk images")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect a disk image
    Inspect {
        /// Path to disk image
        disk: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List filesystems
    Filesystems {
        disk: PathBuf,
    },

    /// List installed packages
    Packages {
        disk: PathBuf,
        /// Filter by name
        #[arg(long)]
        filter: Option<String>,
    },

    /// Copy file from disk image
    Cp {
        /// Source (disk.img:/path/to/file)
        source: String,
        /// Destination path
        dest: PathBuf,
    },

    /// Mount disk and start interactive shell
    Shell {
        disk: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { disk, json } => cmd_inspect(disk, json)?,
        Commands::Filesystems { disk } => cmd_filesystems(disk)?,
        Commands::Packages { disk, filter } => cmd_packages(disk, filter)?,
        Commands::Cp { source, dest } => cmd_cp(source, dest)?,
        Commands::Shell { disk } => cmd_shell(disk)?,
    }

    Ok(())
}

fn cmd_inspect(disk: PathBuf, json: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro(&disk)?;
    g.launch()?;

    let roots = g.inspect_os()?;

    if json {
        // JSON output
        println!("{}", serde_json::json!({
            "roots": roots,
            "os_info": roots.iter().map(|root| {
                serde_json::json!({
                    "type": g.inspect_get_type(root).ok(),
                    "distro": g.inspect_get_distro(root).ok(),
                    "version": format!("{}.{}",
                        g.inspect_get_major_version(root).unwrap_or(0),
                        g.inspect_get_minor_version(root).unwrap_or(0)),
                    "hostname": g.inspect_get_hostname(root).ok(),
                })
            }).collect::<Vec<_>>(),
        }));
    } else {
        // Human-readable output
        println!("=== Disk Image: {} ===\n", disk.display());
        println!("Found {} operating system(s):\n", roots.len());

        for (i, root) in roots.iter().enumerate() {
            println!("OS #{}", i + 1);
            println!("  Root: {}", root);

            if let Ok(os_type) = g.inspect_get_type(root) {
                println!("  Type: {}", os_type);
            }

            if let Ok(distro) = g.inspect_get_distro(root) {
                println!("  Distribution: {}", distro);
            }

            if let Ok(major) = g.inspect_get_major_version(root) {
                let minor = g.inspect_get_minor_version(root).unwrap_or(0);
                println!("  Version: {}.{}", major, minor);
            }

            if let Ok(hostname) = g.inspect_get_hostname(root) {
                println!("  Hostname: {}", hostname);
            }

            println!();
        }
    }

    g.shutdown()?;
    Ok(())
}

fn cmd_filesystems(disk: PathBuf) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro(&disk)?;
    g.launch()?;

    println!("=== Devices ===");
    for device in g.list_devices()? {
        println!("{}", device);
        if let Ok(size) = g.blockdev_getsize64(&device) {
            println!("  Size: {} GB", size / 1_000_000_000);
        }
    }

    println!("\n=== Partitions ===");
    for partition in g.list_partitions()? {
        println!("{}", partition);
        if let Ok(fstype) = g.vfs_type(&partition) {
            println!("  Type: {}", fstype);
        }
        if let Ok(label) = g.vfs_label(&partition) {
            if !label.is_empty() {
                println!("  Label: {}", label);
            }
        }
    }

    g.shutdown()?;
    Ok(())
}

fn cmd_packages(disk: PathBuf, filter: Option<String>) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro(&disk)?;
    g.launch()?;

    let roots = g.inspect_os()?;
    if roots.is_empty() {
        println!("No OS detected");
        return Ok(());
    }

    let root = &roots[0];

    // Mount filesystems
    let mountpoints = g.inspect_get_mountpoints(root)?;
    for (mount, device) in mountpoints.iter().rev() {
        g.mount_ro(device, mount).ok();
    }

    // List packages
    let apps = g.inspect_list_applications(root)?;

    println!("Found {} packages\n", apps.len());

    for app in apps {
        let name = app.app_name;

        // Apply filter
        if let Some(ref f) = filter {
            if !name.contains(f) {
                continue;
            }
        }

        println!("{} {}", name, app.app_version);
    }

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

fn cmd_cp(source: String, dest: PathBuf) -> Result<()> {
    // Parse "disk.img:/path/to/file" format
    let parts: Vec<&str> = source.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Source must be in format: disk.img:/path/to/file");
    }

    let disk = PathBuf::from(parts[0]);
    let src_path = parts[1];

    let mut g = Guestfs::new()?;
    g.add_drive_ro(&disk)?;
    g.launch()?;

    // Try to mount automatically
    let roots = g.inspect_os()?;
    if !roots.is_empty() {
        let root = &roots[0];
        let mountpoints = g.inspect_get_mountpoints(root)?;
        for (mount, device) in mountpoints.iter().rev() {
            g.mount_ro(device, mount).ok();
        }
    }

    // Copy file
    g.download(src_path, dest.to_str().unwrap())?;

    println!("Copied {} -> {}", source, dest.display());

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

fn cmd_shell(disk: PathBuf) -> Result<()> {
    println!("Interactive shell not yet implemented");
    println!("Use: guestkit inspect {} | guestkit cp {} | etc.", disk.display(), disk.display());
    Ok(())
}
```

Update `Cargo.toml`:

```toml
[[bin]]
name = "guestkit"
path = "src/bin/guestkit.rs"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

### Usage Examples

```bash
# Inspect disk
guestkit inspect ubuntu.qcow2

# JSON output for scripting
guestkit inspect ubuntu.qcow2 --json | jq '.os_info[0].distro'

# List filesystems
guestkit filesystems ubuntu.qcow2

# List all packages
guestkit packages ubuntu.qcow2

# Find specific package
guestkit packages ubuntu.qcow2 --filter nginx

# Copy file from disk
guestkit cp ubuntu.qcow2:/etc/passwd ./passwd
```

---

## Priority 2: Progress Reporting (⚡ High Value)

**Why:** Users think operations are frozen. Progress bars dramatically improve UX.

**Effort:** 3 days
**Impact:** ⭐⭐⭐⭐

### Implementation

Add to `Cargo.toml`:
```toml
indicatif = "0.17"
```

Create `src/guestfs/progress.rs`:

```rust
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

pub struct ProgressReporter {
    bar: Arc<ProgressBar>,
}

impl ProgressReporter {
    pub fn new(total: u64, message: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message(message.to_string());

        Self { bar: Arc::new(bar) }
    }

    pub fn update(&self, current: u64) {
        self.bar.set_position(current);
    }

    pub fn finish(&self) {
        self.bar.finish_with_message("Complete");
    }
}

// Callback type for progress
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;
```

Add to operations:

```rust
impl Guestfs {
    pub fn download_with_progress<F>(
        &self,
        remote: &str,
        local: &str,
        progress: F,
    ) -> Result<()>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        // Get file size
        let size = self.filesize(remote)?;

        // Download in chunks with progress
        let mut downloaded = 0u64;
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB

        while downloaded < size as u64 {
            // Read chunk
            let chunk = self.pread(remote, CHUNK_SIZE, downloaded as i64)?;

            // Write to local file
            // ... write logic ...

            downloaded += chunk.len() as u64;
            progress(downloaded, size as u64);
        }

        Ok(())
    }
}
```

Usage:

```rust
use indicatif::ProgressBar;

let pb = ProgressBar::new(file_size);
pb.set_style(
    ProgressStyle::default_bar()
        .template("[{bar:40}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
);

g.download_with_progress("/var/log/huge.log", "huge.log", |current, total| {
    pb.set_position(current);
})?;

pb.finish_with_message("Download complete");
```

---

## Priority 3: Better Error Messages (⚡ Quality of Life)

**Why:** Cryptic errors frustrate users. Good errors save hours of debugging.

**Effort:** 2 days
**Impact:** ⭐⭐⭐⭐

### Implementation

Add to `Cargo.toml`:
```toml
miette = { version = "5.0", features = ["fancy"] }
thiserror = "1.0"
```

Update `src/core/error.rs`:

```rust
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum GuestkitError {
    #[error("Failed to mount {device} at {mountpoint}")]
    #[diagnostic(
        code(guestkit::mount::failed),
        help("Try checking the filesystem type with vfs_type(\"{device}\")")
    )]
    MountFailed {
        device: String,
        mountpoint: String,
        #[source]
        source: std::io::Error,
    },

    #[error("No operating systems detected in {disk}")]
    #[diagnostic(
        code(guestkit::inspect::no_os),
        help("The disk might be:\n  1. Not bootable\n  2. Encrypted (check with is_luks())\n  3. Using an unsupported OS\n  4. Corrupted")
    )]
    NoOsDetected {
        disk: String,
    },

    #[error("Appliance failed to launch")]
    #[diagnostic(
        code(guestkit::launch::failed),
        help("Common causes:\n  1. No disk images added (use add_drive())\n  2. KVM not available (check /dev/kvm permissions)\n  3. Insufficient memory\n\nEnable verbose mode to see detailed errors:\n  g.set_verbose(true)")
    )]
    LaunchFailed {
        #[source]
        source: Box<dyn std::error::Error>,
    },

    #[error("File not found: {path}")]
    #[diagnostic(
        code(guestkit::file::not_found),
        help("Make sure:\n  1. The filesystem is mounted\n  2. The path is correct (case-sensitive)\n  3. Check with is_file(\"{path}\") or is_dir(\"{path}\")")
    )]
    FileNotFound {
        path: String,
    },
}
```

Update operations to use better errors:

```rust
impl Guestfs {
    pub fn mount(&self, device: &str, mountpoint: &str) -> Result<()> {
        // Try mount
        let result = unsafe {
            guestfs_mount(self.handle, device.as_ptr(), mountpoint.as_ptr())
        };

        if result != 0 {
            return Err(GuestkitError::MountFailed {
                device: device.to_string(),
                mountpoint: mountpoint.to_string(),
                source: std::io::Error::last_os_error(),
            }.into());
        }

        Ok(())
    }

    pub fn inspect_os(&self) -> Result<Vec<String>> {
        let roots = self.inspect_os_internal()?;

        if roots.is_empty() {
            return Err(GuestkitError::NoOsDetected {
                disk: self.current_disk.clone().unwrap_or_default(),
            }.into());
        }

        Ok(roots)
    }
}
```

Error output example:

```
Error: Failed to mount /dev/sda1 at /

Caused by:
    Invalid argument

Help: Try checking the filesystem type with vfs_type("/dev/sda1")

Note: Common issues:
  1. Wrong filesystem type specified
  2. Device is encrypted (LUKS)
  3. Filesystem is corrupted (try fsck)
```

---

## Priority 4: Benchmark Suite (📊 Performance Baseline)

**Why:** Can't optimize what you don't measure. Prevents regressions.

**Effort:** 2 days
**Impact:** ⭐⭐⭐

### Implementation

Add to `Cargo.toml`:
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "operations"
harness = false
```

Create `benches/operations.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use guestkit::guestfs::Guestfs;
use std::path::PathBuf;

fn bench_inspect_os(c: &mut Criterion) {
    let test_images = vec![
        "test-images/ubuntu-22.04.qcow2",
        "test-images/fedora-38.qcow2",
        "test-images/debian-12.img",
    ];

    let mut group = c.benchmark_group("inspect_os");

    for image in test_images {
        group.bench_with_input(
            BenchmarkId::from_parameter(image),
            &image,
            |b, &image| {
                b.iter(|| {
                    let mut g = Guestfs::new().unwrap();
                    g.add_drive_ro(image).unwrap();
                    g.launch().unwrap();
                    black_box(g.inspect_os().unwrap());
                    g.shutdown().unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_mount_operations(c: &mut Criterion) {
    c.bench_function("mount_and_umount", |b| {
        let mut g = Guestfs::new().unwrap();
        g.add_drive_ro("test-images/ubuntu-22.04.qcow2").unwrap();
        g.launch().unwrap();

        b.iter(|| {
            g.mount_ro("/dev/sda2", "/").unwrap();
            g.umount("/").unwrap();
        });

        g.shutdown().unwrap();
    });
}

fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_ops");

    // Setup
    let mut g = Guestfs::new().unwrap();
    g.add_drive_ro("test-images/ubuntu-22.04.qcow2").unwrap();
    g.launch().unwrap();
    g.mount_ro("/dev/sda2", "/").unwrap();

    group.bench_function("read_small_file", |b| {
        b.iter(|| {
            black_box(g.read_file("/etc/hostname").unwrap());
        });
    });

    group.bench_function("ls_directory", |b| {
        b.iter(|| {
            black_box(g.ls("/etc").unwrap());
        });
    });

    group.finish();
    g.shutdown().unwrap();
}

criterion_group!(benches, bench_inspect_os, bench_mount_operations, bench_file_operations);
criterion_main!(benches);
```

Run benchmarks:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench -- inspect_os

# Generate HTML report
cargo bench
open target/criterion/report/index.html
```

---

## Priority 5: Integration Test Matrix (🧪 Quality)

**Why:** Catch bugs before users do. Test against real OSes.

**Effort:** 3 days
**Impact:** ⭐⭐⭐⭐

### Implementation

Create `.github/workflows/integration-tests.yml`:

```yaml
name: Integration Tests

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu
            version: "22.04"
            url: "https://cloud-images.ubuntu.com/releases/22.04/release/ubuntu-22.04-server-cloudimg-amd64.img"

          - os: ubuntu
            version: "24.04"
            url: "https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-amd64.img"

          - os: debian
            version: "12"
            url: "https://cloud.debian.org/images/cloud/bookworm/latest/debian-12-generic-amd64.qcow2"

          - os: fedora
            version: "38"
            url: "https://download.fedoraproject.org/pub/fedora/linux/releases/38/Cloud/x86_64/images/Fedora-Cloud-Base-38-1.6.x86_64.qcow2"

    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libguestfs-tools qemu-kvm

      - name: Cache test images
        uses: actions/cache@v3
        with:
          path: test-images/
          key: test-image-${{ matrix.os }}-${{ matrix.version }}

      - name: Download test image
        run: |
          mkdir -p test-images
          cd test-images
          if [ ! -f "${{ matrix.os }}-${{ matrix.version }}.img" ]; then
            wget -O "${{ matrix.os }}-${{ matrix.version }}.img" "${{ matrix.url }}"
          fi

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable

      - name: Run integration tests
        run: |
          cargo test --test integration_${{ matrix.os }} -- \
            --test-image test-images/${{ matrix.os }}-${{ matrix.version }}.img
        env:
          RUST_BACKTRACE: 1
```

Create `tests/integration_ubuntu.rs`:

```rust
use guestkit::guestfs::Guestfs;
use std::env;

fn get_test_image() -> String {
    env::var("TEST_IMAGE").unwrap_or_else(|_| "test-images/ubuntu-22.04.img".to_string())
}

#[test]
fn test_ubuntu_detection() {
    let image = get_test_image();

    let mut g = Guestfs::new().unwrap();
    g.add_drive_ro(&image).unwrap();
    g.launch().unwrap();

    let roots = g.inspect_os().unwrap();
    assert!(!roots.is_empty(), "Should detect Ubuntu OS");

    let root = &roots[0];
    let os_type = g.inspect_get_type(root).unwrap();
    assert_eq!(os_type, "linux");

    let distro = g.inspect_get_distro(root).unwrap();
    assert_eq!(distro, "ubuntu");

    g.shutdown().unwrap();
}

#[test]
fn test_ubuntu_packages() {
    let image = get_test_image();

    let mut g = Guestfs::new().unwrap();
    g.add_drive_ro(&image).unwrap();
    g.launch().unwrap();

    let roots = g.inspect_os().unwrap();
    let root = &roots[0];

    // Mount filesystem
    let mountpoints = g.inspect_get_mountpoints(root).unwrap();
    for (mount, device) in mountpoints.iter().rev() {
        g.mount_ro(device, mount).ok();
    }

    // Check package manager
    let pkg_fmt = g.inspect_get_package_format(root).unwrap();
    assert_eq!(pkg_fmt, "deb");

    // List packages
    let apps = g.inspect_list_applications(root).unwrap();
    assert!(apps.len() > 100, "Ubuntu should have many packages");

    // Check for common packages
    let has_systemd = apps.iter().any(|app| app.app_name == "systemd");
    assert!(has_systemd, "Ubuntu should have systemd");

    g.umount_all().unwrap();
    g.shutdown().unwrap();
}
```

Run locally:

```bash
# Download test image
wget -O test-images/ubuntu-22.04.img \
  https://cloud-images.ubuntu.com/releases/22.04/release/ubuntu-22.04-server-cloudimg-amd64.img

# Run integration tests
TEST_IMAGE=test-images/ubuntu-22.04.img cargo test --test integration_ubuntu
```

---

## Summary: Implementation Order

### Week 1: CLI Tool
- ✅ Immediate value for users
- ✅ Great for demos and documentation
- ✅ Enables shell scripting

### Week 2: Progress Bars + Better Errors
- ✅ Dramatically improves UX
- ✅ Users know operations aren't frozen
- ✅ Easier debugging

### Week 3: Benchmarks + Integration Tests
- ✅ Performance baseline established
- ✅ Catch regressions automatically
- ✅ Test against real OSes

**Total Time:** 3 weeks
**Impact:** Transforms project usability and quality

## Next Steps

1. **Create GitHub issues** for each priority
2. **Tag as "good first issue"** for contributors
3. **Set up project board** to track progress
4. **Start with CLI tool** - highest immediate value
5. **Iterate based on user feedback**

Once these are done, move to Phase 2 (async/caching) from the full roadmap!
