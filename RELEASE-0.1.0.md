# Guestkit Worker Release 0.1.0

**Release Date:** 2026-01-31
**Status:** ✅ Production Ready

---

## Release Artifacts

### Binary

**Location:** `target/release/guestkit-worker`

**Details:**
- **Size:** 9.4 MB (stripped)
- **Type:** ELF 64-bit LSB pie executable
- **Architecture:** x86-64
- **Platform:** Linux
- **Version:** 0.1.0

### Installation

```bash
# Copy binary to system path
sudo cp target/release/guestkit-worker /usr/local/bin/

# Verify installation
guestkit-worker --version
```

---

## What's Included

### Worker Daemon

Complete distributed job processing system:

- **Job Protocol v1.0** - Standardized job specification
- **Pluggable Transports** - File-based and HTTP transports
- **Handler Registry** - 3 operation handlers (echo, inspect, profile)
- **Metrics System** - 13 Prometheus metrics (port 9090)
- **REST API** - 6 endpoints for job management (port 8080)
- **Security** - SHA256 checksum verification

### CLI Commands

7 comprehensive commands:

1. **`daemon`** - Start worker daemon
2. **`submit`** - Submit jobs
3. **`status`** - Check job status
4. **`result`** - Get job results
5. **`list`** - List all jobs
6. **`capabilities`** - Query capabilities
7. **`health`** - Health check

### Features

- ✅ **Phase 4.1** - SHA256 checksum verification
- ✅ **Phase 4.2** - Prometheus metrics (13 metrics)
- ✅ **Phase 4.3** - REST API transport (6 endpoints)
- ✅ **CLI** - Complete command-line interface

---

## Quick Start

### Start Worker Daemon

```bash
# File transport (default)
guestkit-worker daemon

# HTTP transport with REST API
guestkit-worker daemon --transport http
```

### Submit Jobs

```bash
# Quick submit
guestkit-worker submit \
  --operation guestkit.inspect \
  --image /vms/disk.qcow2 \
  --wait

# From file
guestkit-worker submit --file job.json
```

### Monitor

```bash
# List all jobs
guestkit-worker list

# Check status
guestkit-worker status <job-id>

# Check health
guestkit-worker health
```

---

## Supported Operations

### Guestkit Operations

1. **`guestkit.inspect`** - VM disk inspection
   - Detect OS, packages, services
   - SHA256 checksum verification
   - File system analysis

2. **`guestkit.profile`** - Security profiling
   - Security posture assessment
   - Compliance checking
   - Risk analysis

3. **`system.echo`** / **`test.echo`** - Testing
   - Echo handler for testing
   - Job pipeline validation

---

## API Endpoints

### REST API (Port 8080)

```
POST   /api/v1/jobs              # Submit job
GET    /api/v1/jobs              # List jobs
GET    /api/v1/jobs/:id          # Get status
GET    /api/v1/jobs/:id/result   # Get result
GET    /api/v1/capabilities      # Worker capabilities
GET    /api/v1/health            # Health check
```

### Metrics API (Port 9090)

```
GET    /metrics                  # Prometheus metrics
GET    /health                   # Health check
```

---

## Metrics Exposed

### Job Metrics

- `guestkit_jobs_total` - Total jobs processed (by operation, status)
- `guestkit_jobs_duration_seconds` - Job execution time
- `guestkit_active_jobs` - Currently active jobs

### Handler Metrics

- `guestkit_handler_executions_total` - Handler executions
- `guestkit_handler_duration_seconds` - Handler execution time
- `guestkit_handler_errors_total` - Handler errors

### Checksum Metrics

- `guestkit_checksum_verifications_total` - Checksum verifications
- `guestkit_checksum_verification_duration_seconds` - Verification time

### Worker Metrics

- `guestkit_worker_start_time_seconds` - Worker start timestamp
- `guestkit_worker_uptime_seconds` - Worker uptime

### Resource Metrics

- `guestkit_disk_read_bytes_total` - Disk read operations
- `guestkit_disk_write_bytes_total` - Disk write operations
- `guestkit_memory_usage_bytes` - Memory usage

---

## Configuration

### Daemon Options

```bash
guestkit-worker daemon \
  --worker-id <ID>              # Worker identifier
  --pool <POOL>                 # Worker pool name
  --jobs-dir <DIR>              # Job directory (file transport)
  --work-dir <DIR>              # Working directory
  --results-dir <DIR>           # Results directory
  --max-concurrent <N>          # Max concurrent jobs
  --transport <MODE>            # file or http
  --api-addr <ADDR>             # API bind address
  --metrics-addr <ADDR>         # Metrics bind address
  --log-level <LEVEL>           # Log level
```

### Environment Variables

```bash
RUST_LOG=debug                  # Enable debug logging
GUESTKIT_WORKER_API_URL=...     # Default API URL
```

---

## Documentation

### User Guides

- **[CLI Guide](docs/CLI-GUIDE.md)** - Complete CLI reference
- **[Worker Quickstart](docs/guides/quickstart.md)** - Get started in 5 minutes
- **[Docker Quickstart](docs/guides/DOCKER-QUICKSTART.md)** - Run in containers
- **[Kubernetes Guide](docs/guides/K8S-DEPLOYMENT.md)** - Deploy at scale

### API Documentation

- **[REST API Reference](docs/phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md)** - Complete API docs
- **[Prometheus Metrics](docs/phases/phase-4/PHASE-4.2-PROMETHEUS-METRICS.md)** - Metrics guide
- **[Checksum Verification](docs/phases/phase-4/PHASE-4.1-CHECKSUM-VERIFICATION.md)** - Security guide

### Development

- **[Worker Index](docs/WORKER-INDEX.md)** - Worker system overview
- **[Complete System Summary](docs/development/COMPLETE-SYSTEM-SUMMARY.md)** - Full implementation
- **[CLI Development](docs/development/CLI-DEVELOPMENT-SUMMARY.md)** - CLI implementation
- **[Contributing Guide](docs/development/CONTRIBUTING.md)** - How to contribute

---

## Architecture

### Components

```
guestkit-worker
├── Worker Daemon
│   ├── Job Executor
│   ├── Handler Registry
│   ├── Transport Layer (File/HTTP)
│   ├── Metrics Registry
│   └── State Machine
├── REST API Server (port 8080)
│   ├── Job Management
│   ├── Status Lookup
│   └── Health Check
├── Metrics Server (port 9090)
│   ├── Prometheus Metrics
│   └── Health Endpoint
└── CLI
    ├── daemon
    ├── submit
    ├── status
    ├── result
    ├── list
    ├── capabilities
    └── health
```

### Supported Disk Formats

- QCOW2
- VMDK
- VDI
- VHDX
- RAW

### Supported Features

- LVM (Logical Volume Manager)
- NBD (Network Block Device)
- Rust-based implementation

---

## System Requirements

### Runtime

- **OS:** Linux (kernel 3.2.0+)
- **Architecture:** x86_64
- **Memory:** 512 MB minimum, 2 GB recommended
- **Disk:** 100 MB for binary, variable for working directory

### Dependencies

- **glibc:** 2.17+
- **libguestfs:** For VM inspection (guestkit operations)
- **NBD kernel module:** For disk mounting

### Optional

- **Prometheus:** For metrics collection
- **Docker:** For containerized deployment
- **Kubernetes:** For orchestrated deployment

---

## Build Information

### Build Environment

- **Rust Version:** 1.85+ (2021 edition)
- **Build Type:** Release (optimized)
- **Build Date:** 2026-01-31
- **Build Target:** x86_64-unknown-linux-gnu

### Compilation Flags

```bash
cargo build --release
```

**Optimizations:**
- Level: 3 (maximum optimization)
- LTO: Disabled (for faster builds)
- Debug symbols: Stripped

### Crates

**Dependencies:**
- `tokio` - Async runtime
- `axum` - HTTP server
- `reqwest` - HTTP client
- `clap` - CLI parsing
- `prometheus-client` - Metrics
- `serde` / `serde_json` - Serialization
- `guestkit` - VM operations library
- Plus 50+ transitive dependencies

---

## Testing

### Test Results

```
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured
```

**Test Coverage:**
- ✅ Unit tests (39 tests)
- ✅ Integration tests
- ✅ API tests
- ✅ Transport tests
- ✅ Handler tests
- ✅ Metrics tests

### Validation

```bash
# Run tests
cargo test

# Check binary
./target/release/guestkit-worker --version

# Verify help
./target/release/guestkit-worker --help
```

---

## Known Issues

### Warnings

Minor compiler warnings present (safe to ignore):

- Unused imports (7 warnings)
- Unused variables (2 warnings)
- Dead code (2 warnings - unused fields)

**Status:** Non-critical, does not affect functionality

### Limitations

1. **File transport:** Requires shared filesystem
2. **HTTP transport:** In-memory queue (not persistent)
3. **Results:** Stored as files (no database yet)

### Future Work

- Persistent job queue (database)
- Multi-worker coordination
- Job scheduling/priority
- Result streaming
- More operation handlers

---

## Changelog

### Version 0.1.0 (2026-01-31)

**Phase 4 Complete:**

#### Phase 4.1: SHA256 Checksum Verification
- ✅ Cryptographic image verification
- ✅ Checksum validation in inspect handler
- ✅ Support for sha256, sha1, md5 algorithms
- ✅ Metrics for checksum operations

#### Phase 4.2: Prometheus Metrics
- ✅ 13 comprehensive metrics
- ✅ HTTP metrics server (port 9090)
- ✅ /metrics and /health endpoints
- ✅ Integration with worker, executor, handlers

#### Phase 4.3: REST API Transport
- ✅ 6 REST endpoints
- ✅ HTTP job submission
- ✅ JSON request/response
- ✅ In-memory job queue
- ✅ Status tracking

#### CLI Development
- ✅ 7 CLI commands
- ✅ HTTP client for API
- ✅ Multiple output formats (table, JSON, YAML)
- ✅ Interactive modes (watch, wait)
- ✅ Remote worker support

**Previous Phases:**
- Phase 1: Job Protocol v1.0, Worker daemon, File transport
- Phase 2: Echo, Inspect, Profile handlers
- Phase 3: Guestkit library integration

---

## License

**LGPL-3.0-or-later**

This software is licensed under the GNU Lesser General Public License v3.0 or later.

---

## Support

### Documentation

- GitHub: https://github.com/ssahani/guestkit
- Docs: `docs/` directory

### Issues

Report issues at: https://github.com/ssahani/guestkit/issues

### Community

- Maintainer: Susant Sahani <ssahani@redhat.com>

---

## Deployment Examples

### Systemd Service

```ini
[Unit]
Description=Guestkit Worker
After=network.target

[Service]
Type=simple
User=guestkit
ExecStart=/usr/local/bin/guestkit-worker daemon \
  --transport http \
  --worker-id %H \
  --pool production
Restart=always

[Install]
WantedBy=multi-user.target
```

### Docker

```dockerfile
FROM fedora:39
COPY target/release/guestkit-worker /usr/local/bin/
ENTRYPOINT ["guestkit-worker"]
CMD ["daemon", "--transport", "http"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: guestkit-worker
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: worker
        image: guestkit-worker:0.1.0
        args: ["daemon", "--transport", "http"]
        ports:
        - containerPort: 8080
        - containerPort: 9090
```

---

## Next Steps

### After Installation

1. **Start daemon:** `guestkit-worker daemon --transport http`
2. **Submit test job:** `guestkit-worker submit -o system.echo -j '{"message":"hello"}'`
3. **Check status:** `guestkit-worker list`
4. **View metrics:** `curl http://localhost:9090/metrics`

### Production Deployment

1. Create systemd service
2. Configure worker pool
3. Set up Prometheus scraping
4. Deploy multiple workers
5. Configure load balancing

---

**Release:** 0.1.0
**Date:** 2026-01-31
**Status:** ✅ Production Ready
