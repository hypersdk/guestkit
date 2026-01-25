# Performance Tuning Guide

This guide helps you optimize guestctl for maximum performance in your use case.

## Table of Contents

- [Quick Wins](#quick-wins)
- [Benchmarking](#benchmarking)
- [Disk Image Optimization](#disk-image-optimization)
- [Operation-Specific Optimization](#operation-specific-optimization)
- [System-Level Tuning](#system-level-tuning)
- [Scaling and Concurrency](#scaling-and-concurrency)
- [Memory Optimization](#memory-optimization)
- [I/O Optimization](#io-optimization)
- [Best Practices](#best-practices)

---

## Quick Wins

Start here for immediate performance improvements:

### 1. Use Read-Only Mode When Possible

```rust
// 20-30% faster for read-only operations
g.add_drive_ro(path)?;  // ✓ Fast

// vs
g.add_drive_opts(path, None, false)?;  // ✗ Slower
```

### 2. Disable Verbose Logging in Production

```rust
// Production
g.set_verbose(false);  // ✓ Default, fastest

// Development only
g.set_verbose(true);   // ✗ Adds overhead
```

### 3. Use Compressed Formats for Storage

```bash
# qcow2 is faster than raw for large sparse images
guestctl convert disk.raw --output disk.qcow2 --format qcow2 --compress
```

### 4. Batch Operations Instead of Loops

```rust
// ✓ Fast - single tar operation
g.tar_out("/data", "/backup.tar.gz")?;

// ✗ Slow - many individual operations
for file in files {
    g.download(&file, &dest)?;
}
```

### 5. Minimize Mount/Unmount Cycles

```rust
// ✓ Fast - mount once
g.mount("/dev/sda1", "/")?;
process_many_files()?;
g.umount("/")?;

// ✗ Slow - mount for each file
for file in files {
    g.mount(...)?;
    process_file()?;
    g.umount(...)?;
}
```

---

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench disk_creation

# Save baseline for comparison
cargo bench --bench benchmarks -- --save-baseline main

# Compare against baseline
cargo bench --bench benchmarks -- --baseline main
```

### Interpreting Results

```
disk_creation/100MB    time:   [245.32 ms 248.91 ms 252.84 ms]
                       ↑ lower  ↑ mean    ↑ upper
                       bound               bound
```

**Target performance:**
- Disk creation (100MB): < 250ms
- Launch/shutdown: < 500ms
- Mount operation: < 100ms
- File read (1MB): < 50ms
- Checksum (1MB): < 100ms

### Creating Custom Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guestctl::Guestfs;

fn bench_my_operation(c: &mut Criterion) {
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            let mut g = Guestfs::new().unwrap();
            g.my_operation(black_box("param")).unwrap();
        });
    });
}

criterion_group!(benches, bench_my_operation);
criterion_main!(benches);
```

---

## Disk Image Optimization

### Format Selection

**Performance Ranking (fastest to slowest):**

1. **raw** - Fastest I/O, largest size
   - Use for: Temporary images, maximum I/O performance
   - Size: 100% of virtual size

2. **qcow2** - Good balance, compressed
   - Use for: Production, storage efficiency
   - Size: ~30-60% of virtual size

3. **vmdk** - Compatibility, moderate performance
   - Use for: VMware compatibility
   - Size: Variable

**Recommendation:** Use qcow2 with compression for most cases.

### Optimizing Existing Images

```bash
# Convert to qcow2 with compression
guestctl convert input.vmdk \
  --output output.qcow2 \
  --format qcow2 \
  --compress

# Flatten snapshot chains (improves read performance)
guestctl convert input.qcow2 \
  --output output.qcow2 \
  --format qcow2 \
  --flatten

# Combine both
guestctl convert input.vmdk \
  --output output.qcow2 \
  --format qcow2 \
  --compress \
  --flatten
```

### Sparse Files

For raw images, use sparse files to save space:

```bash
# Create sparse image
truncate -s 10G disk.img

# Copy with sparse support
cp --sparse=always source.img dest.img

# Check actual size vs. apparent size
du -h disk.img     # Actual disk usage
ls -lh disk.img    # Apparent size
```

---

## Operation-Specific Optimization

### File Operations

#### Reading Files

```rust
// ✓ Fast - direct read
let content = g.cat("/small-file.txt")?;

// ✓ Faster for large files - streaming download
g.download("/large-file.dat", "/tmp/output")?;

// ✗ Slow - reading in chunks manually
let mut content = Vec::new();
for chunk in 0..100 {
    content.extend(g.pread(...)?);
}
```

#### Writing Files

```rust
// ✓ Fast - single write
g.write("/file.txt", &content)?;

// ✗ Slow - multiple writes
for line in lines {
    g.write_append("/file.txt", line)?;  // Many I/O operations
}
```

#### Listing Files

```rust
// ✓ Fast - use ls()
let files = g.ls("/path")?;

// ✓ Faster for large directories - use find()
let files = g.find("/path")?;

// ✗ Slow - stat each file
for file in files {
    let stat = g.stat(&file)?;  // Many operations
}
```

### Archive Operations

```rust
// ✓ Fastest - compressed tar in one operation
g.tar_out_opts("/data", "/backup.tar.gz", Some("gzip"))?;

// ✗ Slower - uncompressed
g.tar_out("/data", "/backup.tar")?;

// ✗ Slowest - individual files
for file in files {
    g.cp(&file, &backup)?;
}
```

**Compression Trade-offs:**

| Format | Speed | Size | CPU Usage |
|--------|-------|------|-----------|
| none   | Fast  | Large| Low       |
| gzip   | Medium| Medium| Medium    |
| bzip2  | Slow  | Small| High      |
| xz     | Slowest| Smallest| Highest |

**Recommendation:** Use gzip for balanced performance.

### Checksum Operations

```rust
// Performance by algorithm (fastest to slowest):
// md5 (fastest)
let checksum = g.checksum("md5", "/file")?;

// sha1
let checksum = g.checksum("sha1", "/file")?;

// sha256 (recommended balance)
let checksum = g.checksum("sha256", "/file")?;

// sha512 (slowest, most secure)
let checksum = g.checksum("sha512", "/file")?;
```

**Recommendation:** Use SHA256 for security, MD5 only for speed in non-security contexts.

### Filesystem Operations

```rust
// ✓ Fast formats
g.mkfs("ext4", "/dev/sda1")?;  // ~2-3 seconds for 1GB

// ✓ Moderate
g.mkfs("xfs", "/dev/sda1")?;   // ~3-4 seconds for 1GB

// ✗ Slower
g.mkfs("btrfs", "/dev/sda1")?; // ~5-7 seconds for 1GB
```

---

## System-Level Tuning

### NBD Configuration

Optimize NBD performance:

```bash
# Load NBD with more devices and connections
sudo modprobe nbd max_part=16 nbds_max=32

# Increase NBD timeout
echo 30 | sudo tee /sys/block/nbd0/timeout
```

### Kernel Parameters

```bash
# Increase file descriptor limit
ulimit -n 65536

# Increase inotify limits (for monitoring)
sudo sysctl fs.inotify.max_user_watches=524288
sudo sysctl fs.inotify.max_user_instances=512

# Improve I/O performance
sudo sysctl vm.dirty_ratio=15
sudo sysctl vm.dirty_background_ratio=5
```

### Disk Scheduler

```bash
# Check current scheduler
cat /sys/block/sda/queue/scheduler

# Set to none for SSDs (best performance)
echo none | sudo tee /sys/block/sda/queue/scheduler

# Set to mq-deadline for HDDs
echo mq-deadline | sudo tee /sys/block/sda/queue/scheduler
```

---

## Scaling and Concurrency

### Processing Multiple Images

#### Sequential Processing (Simple)

```rust
for disk in disks {
    process_disk(&disk)?;
}
```

**Throughput:** 1 disk at a time

#### Parallel Processing (Better)

```rust
use rayon::prelude::*;

disks.par_iter()
    .try_for_each(|disk| process_disk(disk))?;
```

**Throughput:** N disks concurrently (N = CPU cores)

#### Batch Processing (Best for I/O bound)

```rust
use std::thread;

const BATCH_SIZE: usize = 4;

for batch in disks.chunks(BATCH_SIZE) {
    let handles: Vec<_> = batch.iter()
        .map(|disk| {
            let disk = disk.clone();
            thread::spawn(move || process_disk(&disk))
        })
        .collect();

    for handle in handles {
        handle.join().unwrap()?;
    }
}
```

**Throughput:** Controlled concurrency, prevents resource exhaustion

### Resource Management

```rust
// ✓ Good - clean up between operations
for disk in disks {
    let mut g = Guestfs::new()?;
    g.add_drive_ro(disk)?;
    g.launch()?;

    process(&g)?;

    g.shutdown()?;  // Release resources
}

// ✗ Bad - resource leak
let mut g = Guestfs::new()?;
for disk in disks {
    g.add_drive_ro(disk)?;  // Accumulates!
}
```

---

## Memory Optimization

### Minimize Memory Footprint

```rust
// ✓ Stream large files instead of loading into memory
g.download("/large-file", "/tmp/output")?;

// ✗ Loads entire file into memory
let content = g.cat("/large-file")?;  // May OOM on large files
```

### Monitor Memory Usage

```bash
# Watch memory usage during operations
watch -n 1 'ps aux | grep guestctl | grep -v grep'

# Profile memory with valgrind
valgrind --tool=massif target/release/guestctl inspect disk.img
```

### Memory Limits

```rust
// Set resource limits in code
use std::os::unix::process::CommandExt;

std::process::Command::new("guestctl")
    .pre_exec(|| {
        // Limit memory to 512MB
        unsafe {
            libc::setrlimit(
                libc::RLIMIT_AS,
                &libc::rlimit {
                    rlim_cur: 512 * 1024 * 1024,
                    rlim_max: 512 * 1024 * 1024,
                },
            );
        }
        Ok(())
    })
    .spawn()?;
```

---

## I/O Optimization

### Reduce Syscalls

```rust
// ✓ Bulk operation - fewer syscalls
g.tar_out("/data", "/backup.tar")?;

// ✗ Many syscalls
for file in files {
    g.cp(&file, &dest)?;  // Each is a syscall
}
```

### Use Appropriate Buffer Sizes

```rust
// For custom I/O, use larger buffers
const BUFFER_SIZE: usize = 1024 * 1024;  // 1MB buffer

// Read/write in large chunks
let mut buffer = vec![0u8; BUFFER_SIZE];
```

### Caching

```rust
// Cache filesystem metadata
struct DiskCache {
    filesystems: HashMap<String, String>,
    partitions: Vec<String>,
}

impl DiskCache {
    fn get_filesystems(&mut self, g: &mut Guestfs) -> Result<&HashMap<String, String>> {
        if self.filesystems.is_empty() {
            self.filesystems = g.list_filesystems()?
                .into_iter()
                .collect();
        }
        Ok(&self.filesystems)
    }
}
```

---

## Best Practices

### DO ✓

1. **Use read-only mode** for inspection tasks
2. **Batch operations** instead of loops
3. **Reuse Guestfs handles** when safe
4. **Use compressed formats** for storage
5. **Monitor performance** with benchmarks
6. **Profile** before optimizing
7. **Cache** expensive lookups
8. **Use bulk operations** (tar, rsync)
9. **Clean up resources** after use
10. **Test at scale** before production

### DON'T ✗

1. **Don't enable verbose logging** in production
2. **Don't process all files individually** when bulk operations exist
3. **Don't mount/unmount** in tight loops
4. **Don't load large files** into memory
5. **Don't ignore resource limits**
6. **Don't over-parallelize** (diminishing returns)
7. **Don't skip cleanup** (causes resource leaks)
8. **Don't use raw format** unless needed
9. **Don't optimize prematurely** (profile first)
10. **Don't forget to benchmark** your changes

---

## Performance Checklist

Before deploying to production:

- [ ] Benchmarked common operations
- [ ] Tested with production-size disk images
- [ ] Profiled memory usage
- [ ] Verified resource cleanup
- [ ] Tested concurrent operations
- [ ] Monitored I/O patterns
- [ ] Optimized hot paths
- [ ] Set appropriate timeouts
- [ ] Configured system limits
- [ ] Documented performance characteristics

---

## Monitoring in Production

### Metrics to Track

```rust
use std::time::Instant;

let start = Instant::now();
g.launch()?;
let launch_time = start.elapsed();

println!("Launch time: {:?}", launch_time);

// Track operation times
let start = Instant::now();
let files = g.find("/")?;
println!("Find operation: {:?}, {} files", start.elapsed(), files.len());
```

### Logging Performance Data

```rust
use log::info;

info!("operation=launch duration_ms={} disk_size_gb={}",
      launch_time.as_millis(),
      disk_size_gb);
```

### Alerting Thresholds

Set up alerts for:
- Launch time > 5 seconds
- File operation > 1 second
- Memory usage > 2GB per handle
- Concurrent handles > 50

---

## Example: Optimized Batch Processing

```rust
use guestctl::Guestfs;
use std::time::Instant;
use rayon::prelude::*;

fn process_disk_optimized(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let mut g = Guestfs::new()?;
    g.set_verbose(false);  // ✓ Disable verbose
    g.add_drive_ro(path)?;  // ✓ Read-only
    g.launch()?;

    // Auto-mount
    let roots = g.inspect_os()?;
    if !roots.is_empty() {
        let mounts = g.inspect_get_mountpoints(&roots[0])?;
        for (mp, dev) in mounts {
            g.mount(&dev, &mp)?;
        }

        // ✓ Bulk operation
        g.tar_out_opts("/", "/tmp/backup.tar.gz", Some("gzip"))?;

        g.umount_all()?;
    }

    g.shutdown()?;  // ✓ Clean up

    println!("Processed {} in {:?}", path, start.elapsed());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let disks = vec!["disk1.img", "disk2.img", "disk3.img"];

    // ✓ Parallel processing with controlled concurrency
    disks.par_iter()
        .try_for_each(|disk| process_disk_optimized(disk))?;

    Ok(())
}
```

---

## Further Reading

- [Benchmarking Guide](https://github.com/ssahani/guestkit/tree/main/benches)
- [Troubleshooting Guide](TROUBLESHOOTING.md)
- [API Reference](../API_REFERENCE.md)
- [Examples](../examples/)
