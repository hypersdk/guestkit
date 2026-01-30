# Guestkit Worker CLI Guide

Complete reference for the `guestkit-worker` command-line interface.

---

## Overview

The `guestkit-worker` CLI provides commands for:

1. **Running the worker daemon** - Process jobs from file or HTTP transport
2. **Submitting jobs** - Send jobs to the worker via REST API
3. **Monitoring jobs** - Check status, get results, list all jobs
4. **Worker management** - Check capabilities and health

---

## Installation

```bash
# Build from source
cd crates/guestkit-worker
cargo build --release

# Binary location
target/release/guestkit-worker
```

---

## Commands

### `daemon` - Start Worker Daemon

Start the guestkit-worker daemon to process jobs.

```bash
guestkit-worker daemon [OPTIONS]
```

**Options:**

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--worker-id <ID>` | `-w` | Generated ULID | Unique worker identifier |
| `--pool <POOL>` | `-p` | `default` | Worker pool name |
| `--jobs-dir <DIR>` | `-j` | `./jobs` | Directory to watch for job files (file transport) |
| `--work-dir <DIR>` | | `/tmp/guestkit-worker` | Working directory for job execution |
| `--results-dir <DIR>` | `-r` | `./results` | Directory for result output files |
| `--max-concurrent <N>` | `-m` | `4` | Maximum concurrent jobs |
| `--log-level <LEVEL>` | | `info` | Log level (trace, debug, info, warn, error) |
| `--metrics-enabled` | | `true` | Enable Prometheus metrics server |
| `--metrics-addr <ADDR>` | | `0.0.0.0:9090` | Metrics server bind address |
| `--api-enabled` | | `true` | Enable REST API server (HTTP transport only) |
| `--api-addr <ADDR>` | | `0.0.0.0:8080` | API server bind address |
| `--transport <MODE>` | | `file` | Transport mode: `file` or `http` |

**Examples:**

```bash
# Start with file transport (default)
guestkit-worker daemon

# Start with HTTP transport and REST API
guestkit-worker daemon --transport http --api-enabled

# Custom configuration
guestkit-worker daemon \
  --worker-id my-worker-1 \
  --pool production \
  --max-concurrent 8 \
  --log-level debug

# Disable metrics
guestkit-worker daemon --metrics-enabled false
```

---

### `submit` - Submit a Job

Submit a job to the worker via REST API.

```bash
guestkit-worker submit [OPTIONS]
```

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--file <PATH>` | `-f` | Job file path (JSON or YAML) |
| `--json <JSON>` | `-j` | Job JSON (inline) |
| `--operation <OP>` | `-o` | Operation to perform (quick job) |
| `--image <PATH>` | `-i` | Image path (for quick jobs) |
| `--api-url <URL>` | | API server URL (default: `http://localhost:8080`) |
| `--wait` | `-w` | Wait for job to complete |
| `--output <FORMAT>` | | Output format: `json`, `yaml`, or `table` (default: `table`) |

**Examples:**

```bash
# Submit from file
guestkit-worker submit --file job.json

# Submit inline JSON
guestkit-worker submit --json '{
  "operation": "guestkit.inspect",
  "payload": {
    "type": "guestkit.inspect",
    "data": {"image": "/vms/disk.qcow2"}
  }
}'

# Quick submit with operation and image
guestkit-worker submit \
  --operation guestkit.inspect \
  --image /vms/disk.qcow2

# Submit and wait for completion
guestkit-worker submit --file job.json --wait

# Submit to remote worker
guestkit-worker submit \
  --file job.json \
  --api-url http://worker-1.example.com:8080
```

**Job File Format (JSON):**

```json
{
  "job_id": "job-01HX...",
  "operation": "guestkit.inspect",
  "payload": {
    "type": "guestkit.inspect",
    "data": {
      "image": "/vms/disk.qcow2"
    }
  }
}
```

**Job File Format (YAML):**

```yaml
job_id: job-01HX...
operation: guestkit.inspect
payload:
  type: guestkit.inspect
  data:
    image: /vms/disk.qcow2
```

---

### `status` - Get Job Status

Get the status of a specific job.

```bash
guestkit-worker status [OPTIONS] <JOB_ID>
```

**Arguments:**

- `<JOB_ID>` - Job ID to check

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--api-url <URL>` | | API server URL (default: `http://localhost:8080`) |
| `--output <FORMAT>` | | Output format: `json`, `yaml`, or `table` (default: `table`) |
| `--watch` | `-w` | Watch mode (continuously update until terminal state) |

**Examples:**

```bash
# Check job status
guestkit-worker status job-01HX1234...

# Watch job until completion
guestkit-worker status job-01HX1234... --watch

# JSON output
guestkit-worker status job-01HX1234... --output json
```

**Output (Table):**

```
+----------------+---------------------------+
| Field          | Value                     |
+----------------+---------------------------+
| Job ID         | job-01HX1234...           |
| Status         | completed                 |
| Operation      | guestkit.inspect          |
| Submitted At   | 2026-01-30T10:00:00Z      |
| Started At     | 2026-01-30T10:00:01Z      |
| Completed At   | 2026-01-30T10:00:05Z      |
| Worker ID      | worker-abc123             |
+----------------+---------------------------+
```

---

### `result` - Get Job Result

Get the result of a completed job.

```bash
guestkit-worker result [OPTIONS] <JOB_ID>
```

**Arguments:**

- `<JOB_ID>` - Job ID to get result for

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--api-url <URL>` | | API server URL (default: `http://localhost:8080`) |
| `--output <FORMAT>` | | Output format: `json` or `yaml` (default: `json`) |
| `--save <PATH>` | `-s` | Save result to file |

**Examples:**

```bash
# Get result (stdout)
guestkit-worker result job-01HX1234...

# Save to file
guestkit-worker result job-01HX1234... --save result.json

# YAML format
guestkit-worker result job-01HX1234... --output yaml
```

---

### `list` - List All Jobs

List all jobs known to the worker.

```bash
guestkit-worker list [OPTIONS]
```

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--api-url <URL>` | | API server URL (default: `http://localhost:8080`) |
| `--output <FORMAT>` | | Output format: `json`, `yaml`, or `table` (default: `table`) |
| `--status <STATUS>` | `-s` | Filter by status |
| `--operation <OP>` | `-o` | Filter by operation |

**Examples:**

```bash
# List all jobs
guestkit-worker list

# Filter by status
guestkit-worker list --status completed
guestkit-worker list --status failed

# Filter by operation
guestkit-worker list --operation guestkit.inspect

# JSON output
guestkit-worker list --output json
```

**Output (Table):**

```
+------------------+------------+------------------+---------------+----------------------+
| Job ID           | Status     | Operation        | Worker ID     | Submitted At         |
+------------------+------------+------------------+---------------+----------------------+
| job-01HX1234...  | completed  | guestkit.inspect | worker-abc123 | 2026-01-30T10:00:00Z |
| job-01HX5678...  | running    | guestkit.profile | worker-abc123 | 2026-01-30T10:05:00Z |
| job-01HX9012...  | pending    | guestkit.inspect | -             | 2026-01-30T10:10:00Z |
+------------------+------------+------------------+---------------+----------------------+

Total: 3 jobs
```

---

### `capabilities` - Get Worker Capabilities

Get the capabilities of the worker (supported operations, features, disk formats).

```bash
guestkit-worker capabilities [OPTIONS]
```

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--api-url <URL>` | | API server URL (default: `http://localhost:8080`) |
| `--output <FORMAT>` | | Output format: `json`, `yaml`, or `table` (default: `table`) |

**Examples:**

```bash
# Get capabilities
guestkit-worker capabilities

# JSON output
guestkit-worker capabilities --output json
```

**Output (Table):**

```
+------------+---------------+
| Field      | Value         |
+------------+---------------+
| Worker ID  | worker-abc123 |
| Pool       | production    |
+------------+---------------+

Operations (4):
  • system.echo
  • test.echo
  • guestkit.inspect
  • guestkit.profile

Features (3):
  • rust
  • lvm
  • nbd

Disk Formats (5):
  • qcow2
  • vmdk
  • vdi
  • vhdx
  • raw
```

---

### `health` - Check Worker Health

Check the health status of the worker.

```bash
guestkit-worker health [OPTIONS]
```

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--api-url <URL>` | | API server URL (default: `http://localhost:8080`) |
| `--output <FORMAT>` | | Output format: `json`, `yaml`, or `table` (default: `table`) |

**Examples:**

```bash
# Check health
guestkit-worker health

# JSON output
guestkit-worker health --output json
```

**Output (Table):**

```
+-------------------+-------+
| Field             | Value |
+-------------------+-------+
| Status            | healthy |
| Uptime (seconds)  | 3600  |
+-------------------+-------+

Uptime: 1h 0m 0s
✓ Worker is healthy
```

---

## Common Workflows

### Development Workflow

```bash
# Terminal 1: Start worker with HTTP transport
guestkit-worker daemon --transport http

# Terminal 2: Submit jobs
guestkit-worker submit --operation guestkit.inspect --image /vms/test.qcow2

# Check status
guestkit-worker list

# Get result
guestkit-worker result <job-id>
```

### Production Workflow

```bash
# Start worker with custom configuration
guestkit-worker daemon \
  --worker-id prod-worker-1 \
  --pool production \
  --max-concurrent 16 \
  --transport http \
  --api-addr 0.0.0.0:8080 \
  --metrics-addr 0.0.0.0:9090 \
  --log-level info

# Submit job from file
guestkit-worker submit --file inspect-job.json --wait

# Monitor
guestkit-worker list --status running
guestkit-worker health
```

### Remote Worker Management

```bash
# Submit to remote worker
guestkit-worker submit \
  --file job.json \
  --api-url http://worker-1.prod.example.com:8080

# Check remote worker status
guestkit-worker status <job-id> \
  --api-url http://worker-1.prod.example.com:8080

# Check remote worker health
guestkit-worker health \
  --api-url http://worker-1.prod.example.com:8080
```

---

## Environment Variables

The CLI respects these environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level override | `info` |
| `GUESTKIT_WORKER_API_URL` | Default API URL | `http://localhost:8080` |

**Example:**

```bash
export RUST_LOG=debug
export GUESTKIT_WORKER_API_URL=http://worker-1.prod.example.com:8080

guestkit-worker list
```

---

## Exit Codes

| Code | Description |
|------|-------------|
| `0` | Success |
| `1` | General error |
| `2` | API error (connection failed, 4xx/5xx response) |

---

## Tips & Tricks

### Quick Job Inspection

```bash
# Quick one-liner to inspect a VM
guestkit-worker submit -o guestkit.inspect -i /vms/disk.qcow2 --wait
```

### Batch Job Submission

```bash
# Submit multiple jobs
for vm in /vms/*.qcow2; do
  guestkit-worker submit -o guestkit.inspect -i "$vm"
done

# Monitor all
guestkit-worker list
```

### Watch Job Progress

```bash
# Submit and watch in real-time
guestkit-worker submit --file job.json &
JOB_ID=$(guestkit-worker list | tail -1 | awk '{print $1}')
guestkit-worker status $JOB_ID --watch
```

### JSON Output Parsing

```bash
# Use jq for JSON processing
guestkit-worker list --output json | jq '.[] | select(.status=="failed")'

# Get all job IDs
guestkit-worker list --output json | jq -r '.[].job_id'
```

---

## Troubleshooting

### Connection Refused

```
Error: Failed to send request: connection refused
```

**Solution:** Ensure the worker daemon is running with HTTP transport:

```bash
guestkit-worker daemon --transport http
```

### Job Not Found

```
Error: Job not found
```

**Solution:** The job may not exist or has expired. List all jobs:

```bash
guestkit-worker list
```

### Invalid Job Format

```
Error: Job validation failed
```

**Solution:** Ensure your job document is valid. Check the required fields:

```json
{
  "job_id": "...",
  "operation": "...",
  "payload": {
    "type": "...",
    "data": {}
  }
}
```

---

## See Also

- [Worker Quickstart](guides/quickstart.md) - Get started quickly
- [REST API Reference](phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md) - API documentation
- [Job Protocol](job-protocol-v1.md) - Job specification format
- [Docker Deployment](guides/DOCKER-QUICKSTART.md) - Run in containers

---

**Last Updated:** 2026-01-30
**CLI Version:** 0.1.0
