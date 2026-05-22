# Crates Published to crates.io

**Publication Date:** 2026-01-31
**Status:** ✅ All Published Successfully

---

## Published Crates

### 1. guestkit-job-spec v0.1.0

**Published:** ✅ Success
**Link:** https://crates.io/crates/guestkit-job-spec
**Description:** VM Operations Job Protocol specification and types

**Features:**
- Job Protocol v1.0 type definitions
- Job validation
- Builder pattern for job creation
- Serialization (JSON, YAML)
- ULID-based job IDs

**Installation:**
```toml
[dependencies]
guestkit-job-spec = "0.1.0"
```

```bash
cargo add guestkit-job-spec
```

---

### 2. guestkit v0.3.2

**Published:** ✅ Success
**Link:** https://crates.io/crates/guestkit
**Description:** Pure Rust VM disk toolkit with beautiful output, Windows registry parsing, and VM migration support

**What's New in 0.3.2:**
- ✅ Fixed library name (was `guestkit`, now `guestkit`)
- ✅ Proper exports for Guestfs type
- ✅ Compatible with guestkit-worker

**Features:**
- VM disk inspection
- Windows registry parsing
- TUI file browser
- Guest OS detection
- Package and service enumeration
- Beautiful CLI output
- Python bindings (optional)

**Installation:**
```toml
[dependencies]
guestkit = { version = "0.3.2", features = ["guest-inspect"] }
```

```bash
cargo add guestkit --features guest-inspect
```

---

### 3. guestkit-worker v0.1.0

**Published:** ✅ Success
**Link:** https://crates.io/crates/guestkit-worker
**Description:** Worker daemon for distributed VM operations

**Features:**
- Job Protocol v1.0 support
- Dual transports (file and HTTP)
- 3 operation handlers (echo, inspect, profile)
- SHA256 checksum verification
- Prometheus metrics (13 metrics)
- REST API (6 endpoints)
- Complete CLI (7 commands)

**Installation:**
```toml
[dependencies]
guestkit-worker = "0.1.0"
```

```bash
cargo add guestkit-worker
```

**Binary Installation:**
```bash
cargo install guestkit-worker
```

---

## Installation Guide

### For Library Users

```toml
[dependencies]
guestkit-job-spec = "0.1.0"  # Job protocol types
guestkit = "0.3.2"           # VM operations library
guestkit-worker = "0.1.0"    # Worker system
```

### For CLI Users

```bash
# Install guestkit-worker CLI
cargo install guestkit-worker

# Verify installation
guestkit-worker --version
# Output: guestkit-worker 0.1.0

# Start daemon
guestkit-worker daemon --transport http

# Submit a job
guestkit-worker submit --operation system.echo --json '{"message":"hello"}'
```

---

## Dependency Graph

```
guestkit-worker v0.1.0
├── guestkit-job-spec v0.1.0
└── guestkit v0.3.2

guestkit-job-spec v0.1.0
└── (no local dependencies)

guestkit v0.3.2
└── (no local dependencies)
```

---

## Version Summary

| Crate | Version | Status | Downloads |
|-------|---------|--------|-----------|
| guestkit | 0.3.2 | ✅ Published | [crates.io](https://crates.io/crates/guestkit) |
| guestkit-job-spec | 0.1.0 | ✅ Published | [crates.io](https://crates.io/crates/guestkit-job-spec) |
| guestkit-worker | 0.1.0 | ✅ Published | [crates.io](https://crates.io/crates/guestkit-worker) |

---

## Publication Notes

### guestkit 0.3.1 → 0.3.2

**Why the version bump?**

The published guestkit 0.3.1 had a library name mismatch:
- Package name: `guestkit`
- Library name: `guestkit` (old name)

This caused import errors when trying to use:
```rust
use guestkit::Guestfs;  // Error: unresolved module
```

Version 0.3.2 fixes this:
- Package name: `guestkit`
- Library name: `guestkit` ✅

Now imports work correctly:
```rust
use guestkit::Guestfs;  // ✅ Works!
```

### Publication Process

1. **Published guestkit-job-spec 0.1.0** first (no dependencies)
2. **Published guestkit 0.3.2** (fixes library name issue)
3. **Updated guestkit-worker** to use published versions
4. **Published guestkit-worker 0.1.0**

### Flags Used

```bash
--allow-dirty    # Allow uncommitted changes
--dry-run        # Test packaging without uploading
```

---

## Usage Examples

### Using guestkit-job-spec

```rust
use guestkit_job_spec::{JobBuilder, operations};

let job = JobBuilder::new()
    .generate_job_id()
    .operation(operations::GUESTKIT_INSPECT)
    .payload(
        operations::GUESTKIT_INSPECT,
        serde_json::json!({
            "image": "/vms/disk.qcow2"
        })
    )
    .build()?;

println!("Job ID: {}", job.job_id);
```

### Using guestkit

```rust
use guestkit::Guestfs;

let mut g = Guestfs::new()?;
g.add_drive_ro("/vms/disk.qcow2")?;
g.launch()?;

let oses = g.inspect()?;
for os in oses {
    println!("Found OS: {:?}", os);
}
```

### Using guestkit-worker (Library)

```rust
use guestkit_worker::{Worker, WorkerConfig, HandlerRegistry};

let config = WorkerConfig {
    worker_id: "worker-1".to_string(),
    max_concurrent_jobs: 4,
    ..Default::default()
};

let registry = HandlerRegistry::new();
let worker = Worker::new(config, registry, transport)?;
worker.run().await?;
```

### Using guestkit-worker (CLI)

```bash
# Start daemon
guestkit-worker daemon --transport http

# Submit job
guestkit-worker submit \
  --operation guestkit.inspect \
  --image /vms/disk.qcow2 \
  --wait

# Check status
guestkit-worker list

# Get result
guestkit-worker result <job-id>
```

---

## Documentation Links

### Crates.io Pages

- [guestkit](https://crates.io/crates/guestkit)
- [guestkit-job-spec](https://crates.io/crates/guestkit-job-spec)
- [guestkit-worker](https://crates.io/crates/guestkit-worker)

### Docs.rs (Auto-generated)

- [guestkit docs](https://docs.rs/guestkit)
- [guestkit-job-spec docs](https://docs.rs/guestkit-job-spec)
- [guestkit-worker docs](https://docs.rs/guestkit-worker)

### GitHub Repository

- [Source Code](https://github.com/ssahani/guestkit)
- [Issues](https://github.com/ssahani/guestkit/issues)
- [Releases](https://github.com/ssahani/guestkit/releases)

---

## What's Next

### For Users

1. **Try it out:**
   ```bash
   cargo install guestkit-worker
   guestkit-worker --help
   ```

2. **Read the docs:**
   - [CLI Guide](docs/CLI-GUIDE.md)
   - [Worker Index](docs/WORKER-INDEX.md)
   - [API Reference](docs/phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md)

3. **Deploy:**
   - [Docker Guide](docs/guides/DOCKER-QUICKSTART.md)
   - [Kubernetes Guide](docs/guides/K8S-DEPLOYMENT.md)

### For Contributors

1. **Check out the code:**
   ```bash
   git clone https://github.com/ssahani/guestkit
   cd guestkit
   ```

2. **Build from source:**
   ```bash
   cargo build --release
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Read contributing guide:**
   - [Contributing](docs/development/CONTRIBUTING.md)

---

## Support & Community

### Getting Help

- **Documentation:** `docs/` directory
- **Issues:** https://github.com/ssahani/guestkit/issues
- **Maintainer:** Susant Sahani <ssahani@redhat.com>

### Reporting Issues

Please report issues at: https://github.com/ssahani/guestkit/issues

Include:
- Crate version
- Rust version (`rustc --version`)
- Operating system
- Error messages
- Minimal reproduction code

---

## License

All crates are licensed under **LGPL-3.0-or-later**

---

## Metrics

### Initial Publication Stats

**Published:** 2026-01-31

| Metric | Value |
|--------|-------|
| Total Crates | 3 |
| Combined Lines of Code | ~15,000+ |
| Total Tests | 39 |
| Documentation Pages | 20+ |
| CLI Commands | 7 |
| REST API Endpoints | 6 |
| Prometheus Metrics | 13 |

---

**Publication Complete!** 🎉

All crates are now available on crates.io and ready for use.

```bash
cargo install guestkit-worker
guestkit-worker --version
```
