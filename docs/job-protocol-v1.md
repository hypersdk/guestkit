# VM Operations Job Protocol v1.0

**Status:** Draft for Review
**Date:** 2026-01-30
**Authors:** guestkit team

---

## 🎯 Design Principles

1. **Generic control plane** - Stable envelope for all operations
2. **Typed data plane** - Operation-specific payloads
3. **Forward compatible** - Unknown fields allowed everywhere
4. **Namespace isolation** - Operations namespaced to prevent collisions
5. **Independent versioning** - Envelope, operations, and payloads version separately
6. **Transport agnostic** - Works with files, REST, queues, gRPC
7. **Idempotent by design** - Safe to retry, safe to replay

---

## 📦 Layer 1: Job Envelope (Stable Forever)

This is the **frozen contract**. Breaking changes here require major version bump.

### Complete Job Schema

```json
{
  "$schema": "https://guestkit.dev/schemas/job-v1.json",
  "version": "1.0",
  "job_id": "job-550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2026-01-30T10:00:00Z",

  "kind": "VMOperation",
  "operation": "guestkit.inspect",

  "metadata": {
    "name": "inspect-production-vm-01",
    "namespace": "production",
    "labels": {
      "environment": "prod",
      "team": "platform",
      "cost-center": "engineering"
    },
    "annotations": {
      "ticket": "INC-12345",
      "requested_by": "alice@company.com",
      "description": "Weekly security scan"
    }
  },

  "execution": {
    "idempotency_key": "scan-2026-01-30-weekly",
    "attempt": 1,
    "max_attempts": 3,
    "timeout_seconds": 7200,
    "deadline": "2026-01-30T14:00:00Z",
    "priority": 5,
    "cancellable": true
  },

  "constraints": {
    "required_capabilities": [
      "guestkit.inspect",
      "disk.qcow2",
      "fs.lvm"
    ],
    "required_features": [
      "lvm",
      "selinux"
    ],
    "minimum_worker_version": "0.4.0",
    "maximum_disk_size_gb": 1000,
    "require_privileged": true,
    "allowed_worker_pools": ["pool-secure", "pool-prod"]
  },

  "routing": {
    "worker_id": null,
    "worker_pool": "pool-prod",
    "affinity": {
      "disk_locality": "/mnt/vms"
    },
    "anti_affinity": {
      "concurrent_jobs": ["job-abc123"]
    }
  },

  "payload": {
    "type": "guestkit.inspect.v1",
    "data": {
      "image": {
        "path": "/mnt/vms/prod-web-01.qcow2",
        "format": "qcow2",
        "checksum": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "size_bytes": 42949672960
      },
      "options": {
        "deep_scan": true,
        "include_packages": true,
        "include_services": true,
        "include_security": true
      },
      "output": {
        "format": "json",
        "destination": "/mnt/output/inspect-results.json"
      }
    }
  },

  "observability": {
    "trace_id": "550e8400-e29b-41d4-a716-446655440001",
    "span_id": "446655440001",
    "parent_span_id": null,
    "correlation_id": "weekly-scan-batch-2026-w04"
  },

  "audit": {
    "submitted_by": "api-service-v1.2.3",
    "submitted_from": "10.0.1.50",
    "authorization": {
      "method": "bearer_token",
      "subject": "service-account-scanner"
    }
  }
}
```

---

## 📋 Field Specifications

### Core Fields (REQUIRED)

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `version` | string | Protocol version | `"1.0"` |
| `job_id` | string (UUID) | Unique job identifier | `"job-uuid"` |
| `created_at` | string (ISO8601) | Job creation timestamp | `"2026-01-30T10:00:00Z"` |
| `kind` | string | Job kind (always "VMOperation" for v1) | `"VMOperation"` |
| `operation` | string | Namespaced operation type | `"guestkit.inspect"` |
| `payload` | object | Operation-specific data | See Layer 2 |

### Metadata (OPTIONAL)

| Field | Type | Description |
|-------|------|-------------|
| `metadata.name` | string | Human-readable job name |
| `metadata.namespace` | string | Logical namespace for isolation |
| `metadata.labels` | map[string]string | Key-value labels for filtering |
| `metadata.annotations` | map[string]string | Arbitrary metadata |

### Execution Policy (OPTIONAL)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `execution.idempotency_key` | string | null | Unique key for idempotent execution |
| `execution.attempt` | integer | 1 | Current attempt number |
| `execution.max_attempts` | integer | 1 | Maximum retry attempts |
| `execution.timeout_seconds` | integer | 3600 | Job timeout in seconds |
| `execution.deadline` | string (ISO8601) | null | Hard deadline for completion |
| `execution.priority` | integer [1-10] | 5 | Job priority (higher = more urgent) |
| `execution.cancellable` | boolean | true | Whether job can be cancelled |

### Constraints (OPTIONAL)

Defines worker capabilities required to execute this job.

| Field | Type | Description |
|-------|------|-------------|
| `constraints.required_capabilities` | array[string] | Capabilities worker must have |
| `constraints.required_features` | array[string] | System features required |
| `constraints.minimum_worker_version` | string (semver) | Minimum worker version |
| `constraints.maximum_disk_size_gb` | integer | Max disk size worker can handle |
| `constraints.require_privileged` | boolean | Requires privileged execution |
| `constraints.allowed_worker_pools` | array[string] | Allowed worker pool names |

### Routing (OPTIONAL)

Controls job placement and scheduling.

| Field | Type | Description |
|-------|------|-------------|
| `routing.worker_id` | string | Pin to specific worker (use sparingly) |
| `routing.worker_pool` | string | Target worker pool |
| `routing.affinity` | object | Scheduling preferences |
| `routing.anti_affinity` | object | Scheduling anti-preferences |

### Observability (OPTIONAL but RECOMMENDED)

| Field | Type | Description |
|-------|------|-------------|
| `observability.trace_id` | string (UUID) | Distributed trace ID |
| `observability.span_id` | string | Span ID within trace |
| `observability.parent_span_id` | string | Parent span ID |
| `observability.correlation_id` | string | Correlation ID for related jobs |

### Audit (OPTIONAL but RECOMMENDED)

| Field | Type | Description |
|-------|------|-------------|
| `audit.submitted_by` | string | Submitter identity |
| `audit.submitted_from` | string | Submitter IP/hostname |
| `audit.authorization` | object | Authorization details |

---

## 🔧 Layer 2: Operation Namespace

Operations are namespaced to prevent collisions and enable independent evolution.

### Namespace Format

```
<tool>.<operation>[.<variant>]
```

### Reserved Namespaces

| Namespace | Owner | Purpose |
|-----------|-------|---------|
| `guestkit.*` | guestkit | VM disk operations |
| `hyper2kvm.*` | hyper2kvm | VM migration operations |
| `system.*` | core | System-level operations |

### Guestkit Operations (v1)

| Operation | Description | Payload Version |
|-----------|-------------|-----------------|
| `guestkit.inspect` | VM disk inspection | v1 |
| `guestkit.profile` | Security/compliance profiling | v1 |
| `guestkit.fix` | Offline repair operations | v1 |
| `guestkit.convert` | Disk format conversion | v1 |
| `guestkit.compare` | VM comparison | v1 |

### Future Operations (Examples)

```
hyper2kvm.convert
hyper2kvm.validate
system.health-check
system.capability-probe
```

---

## 📦 Layer 3: Payload Specifications

Each operation defines its own payload schema with independent versioning.

### Payload Structure

```json
{
  "type": "namespace.operation.version",
  "data": {
    // Operation-specific fields
  }
}
```

### guestkit.inspect.v1

```json
{
  "type": "guestkit.inspect.v1",
  "data": {
    "image": {
      "path": "/path/to/vm.qcow2",
      "format": "qcow2",
      "checksum": "sha256:...",
      "size_bytes": 42949672960,
      "read_only": true
    },
    "options": {
      "deep_scan": false,
      "include_packages": true,
      "include_services": true,
      "include_users": true,
      "include_network": true,
      "include_security": true,
      "include_storage": true,
      "include_databases": true
    },
    "output": {
      "format": "json",
      "destination": "/path/to/output.json",
      "compression": null
    }
  }
}
```

### guestkit.profile.v1

```json
{
  "type": "guestkit.profile.v1",
  "data": {
    "image": {
      "path": "/path/to/vm.qcow2",
      "format": "qcow2",
      "checksum": "sha256:...",
      "size_bytes": 42949672960
    },
    "profiles": ["security", "compliance", "hardening", "performance"],
    "options": {
      "severity_threshold": "medium",
      "fail_on_critical": true
    },
    "output": {
      "format": "json",
      "destination": "/path/to/profile-results.json",
      "include_remediation": true
    }
  }
}
```

### guestkit.fix.v1

```json
{
  "type": "guestkit.fix.v1",
  "data": {
    "image": {
      "path": "/path/to/vm.qcow2",
      "format": "qcow2",
      "checksum": "sha256:...",
      "size_bytes": 42949672960,
      "create_backup": true,
      "backup_path": "/path/to/vm-backup.qcow2"
    },
    "operations": [
      {
        "type": "fsck",
        "options": {
          "auto_repair": true,
          "filesystem": "ext4"
        }
      },
      {
        "type": "selinux_relabel",
        "options": {
          "force": false
        }
      },
      {
        "type": "initramfs_regenerate",
        "options": {
          "kernel_version": "auto"
        }
      }
    ],
    "execution_policy": {
      "stop_on_error": true,
      "validate_before_commit": true
    },
    "output": {
      "fixed_image": "/path/to/vm-fixed.qcow2",
      "log_path": "/path/to/fix.log",
      "report_format": "json"
    }
  }
}
```

### guestkit.convert.v1

```json
{
  "type": "guestkit.convert.v1",
  "data": {
    "source": {
      "path": "/path/to/source.vmdk",
      "format": "vmdk",
      "checksum": "sha256:..."
    },
    "target": {
      "path": "/path/to/target.qcow2",
      "format": "qcow2",
      "compression": true,
      "compression_type": "zstd"
    },
    "options": {
      "verify_after_convert": true,
      "preserve_sparse": true
    }
  }
}
```

### guestkit.compare.v1

```json
{
  "type": "guestkit.compare.v1",
  "data": {
    "baseline": {
      "path": "/path/to/baseline.qcow2",
      "format": "qcow2",
      "checksum": "sha256:..."
    },
    "target": {
      "path": "/path/to/target.qcow2",
      "format": "qcow2",
      "checksum": "sha256:..."
    },
    "options": {
      "compare_packages": true,
      "compare_files": false,
      "compare_config": true,
      "ignore_timestamps": true
    },
    "output": {
      "format": "json",
      "destination": "/path/to/diff.json",
      "include_recommendations": true
    }
  }
}
```

---

## 📊 Result Schema

Workers return results in a standardized envelope.

### Success Result

```json
{
  "job_id": "job-550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "completed_at": "2026-01-30T11:30:00Z",
  "worker_id": "worker-01",

  "execution_summary": {
    "started_at": "2026-01-30T10:00:00Z",
    "duration_seconds": 5400,
    "attempt": 1,
    "idempotency_key": "scan-2026-01-30-weekly"
  },

  "outputs": {
    "primary": "/mnt/output/inspect-results.json",
    "artifacts": [
      "/mnt/output/logs/job.log",
      "/mnt/output/metrics.json"
    ]
  },

  "metrics": {
    "disk_read_bytes": 42949672960,
    "disk_write_bytes": 1048576,
    "peak_memory_mb": 2048,
    "cpu_seconds": 3600
  },

  "observability": {
    "trace_id": "550e8400-e29b-41d4-a716-446655440001",
    "span_id": "446655440002"
  }
}
```

### Failure Result

```json
{
  "job_id": "job-550e8400-e29b-41d4-a716-446655440000",
  "status": "failed",
  "failed_at": "2026-01-30T10:15:00Z",
  "worker_id": "worker-01",

  "execution_summary": {
    "started_at": "2026-01-30T10:00:00Z",
    "duration_seconds": 900,
    "attempt": 2
  },

  "error": {
    "code": "IMAGE_CHECKSUM_MISMATCH",
    "message": "Image checksum verification failed",
    "phase": "validation",
    "details": {
      "expected": "sha256:e3b0c44...",
      "actual": "sha256:abc123..."
    },
    "recoverable": false,
    "retry_recommended": false
  },

  "outputs": {
    "artifacts": [
      "/mnt/output/logs/error.log"
    ]
  }
}
```

### Result Status Values

| Status | Description | Terminal | Retryable |
|--------|-------------|----------|-----------|
| `pending` | Waiting to start | No | N/A |
| `assigned` | Assigned to worker | No | N/A |
| `running` | Currently executing | No | N/A |
| `completed` | Successfully finished | Yes | No |
| `failed` | Failed with error | Yes | Maybe |
| `cancelled` | User cancelled | Yes | No |
| `timeout` | Exceeded deadline | Yes | Maybe |

---

## 🔄 Progress Events

Workers emit progress events during execution for observability.

### Progress Event Schema

```json
{
  "job_id": "job-550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-01-30T10:05:00Z",
  "sequence": 42,

  "phase": "disk_inspection",
  "progress_percent": 35,
  "message": "Inspecting partition 2 of 4",

  "details": {
    "current_partition": "/dev/sda2",
    "filesystem": "ext4",
    "size_bytes": 10737418240
  },

  "observability": {
    "trace_id": "550e8400-e29b-41d4-a716-446655440001",
    "span_id": "446655440003"
  }
}
```

### Standard Phases (Guestkit)

| Phase | Description |
|-------|-------------|
| `validation` | Pre-execution validation |
| `disk_attach` | NBD/loop device attachment |
| `disk_inspection` | Disk structure analysis |
| `os_detection` | Operating system detection |
| `package_scan` | Package inventory |
| `service_scan` | Service enumeration |
| `security_scan` | Security analysis |
| `result_generation` | Creating output artifacts |
| `cleanup` | Resource cleanup |

---

## 🛡️ Worker Capability Advertisement

Workers advertise capabilities to scheduler for matching.

### Worker Registration Schema

```json
{
  "worker_id": "worker-01",
  "version": "0.4.0",
  "hostname": "worker-node-01.example.com",
  "registered_at": "2026-01-30T09:00:00Z",

  "capabilities": {
    "operations": [
      "guestkit.inspect",
      "guestkit.profile",
      "guestkit.fix",
      "guestkit.convert",
      "guestkit.compare"
    ],
    "features": [
      "lvm",
      "nbd",
      "loop",
      "selinux",
      "windows-registry"
    ],
    "disk_formats": [
      "qcow2",
      "vmdk",
      "vdi",
      "vhdx",
      "raw"
    ]
  },

  "resources": {
    "max_concurrent_jobs": 4,
    "max_disk_size_gb": 2000,
    "available_disk_gb": 5000,
    "cpu_cores": 16,
    "memory_gb": 64
  },

  "configuration": {
    "privileged": true,
    "worker_pool": "pool-prod",
    "data_locality": ["/mnt/vms", "/mnt/images"]
  },

  "status": {
    "state": "ready",
    "current_jobs": 2,
    "last_heartbeat": "2026-01-30T10:30:00Z"
  }
}
```

---

## ✅ Validation Rules

### Job Validation (Pre-execution)

Workers MUST validate before execution:

1. **Schema version supported**
2. **Operation type supported** by worker
3. **Required capabilities** available
4. **Image checksum** matches (if provided)
5. **Disk exists** and is accessible
6. **Timeout** is reasonable (warn if > 24h)
7. **Idempotency key** unique (or job already completed)

### Idempotency Guarantees

If `execution.idempotency_key` is provided:

1. Worker MUST check if job with same key already completed
2. If already completed, return previous result (do NOT re-execute)
3. If in progress, return error indicating duplicate
4. Workers MUST store idempotency key → result mapping

---

## 🔐 Security Considerations

### Job Signing (Future)

```json
{
  "signature": {
    "algorithm": "ed25519",
    "public_key_id": "key-abc123",
    "value": "base64-encoded-signature"
  }
}
```

### Path Validation

Workers MUST validate:
- All paths are within allowed directories
- No path traversal attacks (`../`)
- No symlink escapes

### Resource Limits

Workers MUST enforce:
- Job timeout
- Disk size constraints
- Memory limits
- Concurrent job limits

---

## 📚 Versioning Strategy

### Protocol Versioning

`version` field uses semantic versioning:
- Major: Breaking changes to envelope
- Minor: Backward-compatible additions
- Patch: Bug fixes

### Operation Versioning

Operations version independently:
- `guestkit.inspect.v1` → `guestkit.inspect.v2`
- Workers can support multiple versions simultaneously
- Clients specify required version in payload type

### Compatibility Matrix

| Worker Version | Supports Protocol | Supports Operations |
|----------------|-------------------|---------------------|
| 0.4.x | 1.0 | guestkit.*.v1 |
| 0.5.x | 1.0, 1.1 | guestkit.*.v1, guestkit.*.v2 |

---

## 🚀 Extension Points

### Custom Metadata

Jobs can include custom metadata in annotations:

```json
"metadata": {
  "annotations": {
    "custom.company.com/team": "platform",
    "custom.company.com/cost-code": "12345"
  }
}
```

### Custom Constraints

```json
"constraints": {
  "custom.company.com/gpu-required": "true",
  "custom.company.com/max-cost-dollars": "10"
}
```

### Unknown Fields

Workers MUST ignore unknown fields gracefully (forward compatibility).

---

## 📖 Examples

See [examples/](../examples/jobs/) directory for complete examples:
- `inspect-basic.json` - Minimal inspection job
- `inspect-full.json` - Full-featured inspection
- `profile-security.json` - Security profiling
- `fix-offline.json` - Offline repair operations
- `batch-convert.json` - Batch conversion job

---

## 🔗 References

- JSON Schema: `schemas/job-v1.schema.json`
- OpenAPI Spec: `api/openapi-v1.yaml`
- Rust Types: `crates/guestkit-job-spec/src/v1.rs`

---

**End of Protocol Specification v1.0**
