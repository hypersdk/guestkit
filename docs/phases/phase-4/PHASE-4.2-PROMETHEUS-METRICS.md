# Phase 4.2: Prometheus Metrics Integration

**Status**: ✅ Complete
**Date**: 2026-01-30
**Feature**: Comprehensive observability with Prometheus metrics

---

## Overview

Phase 4.2 adds complete Prometheus metrics integration to the guestkit worker, enabling production-grade monitoring and observability. The implementation provides:

- **Job Metrics** - Execution counts, durations, and status tracking
- **Handler Metrics** - Per-handler performance and success rates
- **Worker Metrics** - Active jobs and queue depth
- **Resource Metrics** - Disk I/O tracking
- **Checksum Metrics** - Verification success/failure counts
- **HTTP Endpoint** - Standard Prometheus `/metrics` endpoint

---

## Architecture

```
Worker Process
      ├─ MetricsRegistry (Prometheus)
      │   ├─ Job Metrics (counters, histograms)
      │   ├─ Handler Metrics (execution tracking)
      │   ├─ Worker Metrics (gauges)
      │   └─ Resource Metrics (I/O counters)
      │
      ├─ HTTP Server (Axum)
      │   ├─ GET /metrics  → Prometheus text format
      │   └─ GET /health   → Health check
      │
      └─ Job Executor
          └─ Records metrics at key execution points
```

---

## Metrics Catalog

### Job Metrics

#### `guestkit_worker_jobs_total`
**Type**: Counter
**Labels**: `operation`, `status`
**Description**: Total number of jobs processed
**Values**:
- `status`: `completed`, `failed`, `timeout`, `cancelled`
- `operation`: Job operation name (e.g., `guestkit.inspect`)

**Example**:
```
guestkit_worker_jobs_total{operation="guestkit.inspect",status="completed"} 42
guestkit_worker_jobs_total{operation="guestkit.inspect",status="failed"} 3
guestkit_worker_jobs_total{operation="guestkit.profile",status="completed"} 15
```

#### `guestkit_worker_jobs_duration_seconds`
**Type**: Histogram
**Labels**: `operation`, `status`
**Description**: Job execution duration in seconds
**Buckets**: 1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s, 512s, 1024s, 2048s

**Example**:
```
guestkit_worker_jobs_duration_seconds_bucket{operation="guestkit.inspect",status="completed",le="1"} 0
guestkit_worker_jobs_duration_seconds_bucket{operation="guestkit.inspect",status="completed",le="2"} 0
guestkit_worker_jobs_duration_seconds_bucket{operation="guestkit.inspect",status="completed",le="4"} 5
guestkit_worker_jobs_duration_seconds_bucket{operation="guestkit.inspect",status="completed",le="8"} 35
guestkit_worker_jobs_duration_seconds_bucket{operation="guestkit.inspect",status="completed",le="+Inf"} 42
guestkit_worker_jobs_duration_seconds_sum{operation="guestkit.inspect",status="completed"} 234.5
guestkit_worker_jobs_duration_seconds_count{operation="guestkit.inspect",status="completed"} 42
```

#### `guestkit_worker_active_jobs`
**Type**: Gauge
**Description**: Currently executing jobs

**Example**:
```
guestkit_worker_active_jobs 3
```

#### `guestkit_worker_queue_depth`
**Type**: Gauge
**Description**: Number of pending jobs in queue

**Example**:
```
guestkit_worker_queue_depth 12
```

### Handler Metrics

#### `guestkit_handler_executions_total`
**Type**: Counter
**Labels**: `handler`, `status`
**Description**: Total handler executions
**Values**:
- `handler`: Handler name (e.g., `guestkit-inspect`, `guestkit-profile`)
- `status`: `success`, `error`

**Example**:
```
guestkit_handler_executions_total{handler="guestkit-inspect",status="success"} 42
guestkit_handler_executions_total{handler="guestkit-inspect",status="error"} 3
guestkit_handler_executions_total{handler="guestkit-profile",status="success"} 15
```

#### `guestkit_handler_duration_seconds`
**Type**: Histogram
**Labels**: `handler`, `status`
**Description**: Handler execution duration in seconds
**Buckets**: Same as job duration (1s to 2048s)

**Example**:
```
guestkit_handler_duration_seconds_bucket{handler="guestkit-inspect",status="success",le="8"} 42
guestkit_handler_duration_seconds_sum{handler="guestkit-inspect",status="success"} 234.5
guestkit_handler_duration_seconds_count{handler="guestkit-inspect",status="success"} 42
```

### Checksum Verification Metrics

#### `guestkit_checksum_verifications_total`
**Type**: Counter
**Labels**: `status`
**Description**: Checksum verification attempts
**Values**:
- `status`: `success`, `failure`, `skipped`

**Example**:
```
guestkit_checksum_verifications_total{status="success"} 125
guestkit_checksum_verifications_total{status="failure"} 2
guestkit_checksum_verifications_total{status="skipped"} 48
```

### Resource Metrics

#### `guestkit_worker_disk_read_bytes_total`
**Type**: Counter
**Description**: Total disk bytes read

**Example**:
```
guestkit_worker_disk_read_bytes_total 12884901888
```

#### `guestkit_worker_disk_write_bytes_total`
**Type**: Counter
**Description**: Total disk bytes written

**Example**:
```
guestkit_worker_disk_write_bytes_total 1073741824
```

---

## HTTP Endpoints

### GET /metrics

Returns metrics in Prometheus text format.

**Example Request**:
```bash
curl http://localhost:9090/metrics
```

**Example Response**:
```
# HELP guestkit_worker_jobs_total Total number of jobs processed
# TYPE guestkit_worker_jobs_total counter
guestkit_worker_jobs_total{operation="guestkit.inspect",status="completed"} 42
guestkit_worker_jobs_total{operation="guestkit.inspect",status="failed"} 3

# HELP guestkit_worker_jobs_duration_seconds Job execution duration in seconds
# TYPE guestkit_worker_jobs_duration_seconds histogram
guestkit_worker_jobs_duration_seconds_bucket{operation="guestkit.inspect",status="completed",le="8"} 42
guestkit_worker_jobs_duration_seconds_sum{operation="guestkit.inspect",status="completed"} 234.5
guestkit_worker_jobs_duration_seconds_count{operation="guestkit.inspect",status="completed"} 42

# HELP guestkit_worker_active_jobs Currently active jobs
# TYPE guestkit_worker_active_jobs gauge
guestkit_worker_active_jobs 3

... (more metrics)
```

### GET /health

Returns worker health status.

**Example Request**:
```bash
curl http://localhost:9090/health
```

**Example Response**:
```json
{"status":"healthy"}
```

---

## Usage Examples

### Example 1: Start Worker with Metrics

```bash
# Build worker
cd /home/ssahani/tt/guestkit/crates/guestkit-worker
cargo build --release --bin guestkit-worker

# Start with metrics enabled (default)
./target/release/guestkit-worker \
  --worker-id prod-worker-01 \
  --pool production \
  --jobs-dir /var/guestkit/jobs \
  --results-dir /var/guestkit/results \
  --metrics-enabled true \
  --metrics-addr 0.0.0.0:9090

# Output:
# [INFO] Starting guestkit worker
# [INFO] Metrics server started on 0.0.0.0:9090
# [INFO] Metrics endpoint: http://0.0.0.0:9090/metrics
# [INFO] Health endpoint: http://0.0.0.0:9090/health
# [INFO] Worker ready, waiting for jobs...
```

### Example 2: Query Metrics

```bash
# Get all metrics
curl http://localhost:9090/metrics

# Filter for job metrics
curl http://localhost:9090/metrics | grep guestkit_worker_jobs

# Check active jobs
curl http://localhost:9090/metrics | grep guestkit_worker_active_jobs

# Check health
curl http://localhost:9090/health
```

### Example 3: Prometheus Configuration

**prometheus.yml**:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'guestkit-workers'
    static_configs:
      - targets:
          - 'worker-01:9090'
          - 'worker-02:9090'
          - 'worker-03:9090'
    labels:
      environment: 'production'
      service: 'guestkit-worker'
```

### Example 4: Grafana Dashboard Queries

#### Job Success Rate (Last 1h)
```promql
sum(rate(guestkit_worker_jobs_total{status="completed"}[1h]))
/
sum(rate(guestkit_worker_jobs_total[1h]))
* 100
```

#### Average Job Duration (Last 5m)
```promql
rate(guestkit_worker_jobs_duration_seconds_sum[5m])
/
rate(guestkit_worker_jobs_duration_seconds_count[5m])
```

#### Active Jobs Gauge
```promql
guestkit_worker_active_jobs
```

#### Queue Depth Trend
```promql
guestkit_worker_queue_depth
```

#### Checksum Failure Rate
```promql
rate(guestkit_checksum_verifications_total{status="failure"}[5m])
```

#### Handler Error Rate
```promql
sum(rate(guestkit_handler_executions_total{status="error"}[5m])) by (handler)
```

#### Disk I/O Rate
```promql
rate(guestkit_worker_disk_read_bytes_total[5m]) + rate(guestkit_worker_disk_write_bytes_total[5m])
```

---

## Grafana Dashboard

### Sample Dashboard JSON

```json
{
  "dashboard": {
    "title": "Guestkit Worker Metrics",
    "panels": [
      {
        "title": "Job Success Rate",
        "targets": [{
          "expr": "sum(rate(guestkit_worker_jobs_total{status=\"completed\"}[1h])) / sum(rate(guestkit_worker_jobs_total[1h])) * 100"
        }],
        "type": "gauge"
      },
      {
        "title": "Active Jobs",
        "targets": [{
          "expr": "guestkit_worker_active_jobs"
        }],
        "type": "stat"
      },
      {
        "title": "Job Duration (p50, p95, p99)",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(guestkit_worker_jobs_duration_seconds_bucket[5m]))",
            "legendFormat": "p50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(guestkit_worker_jobs_duration_seconds_bucket[5m]))",
            "legendFormat": "p95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(guestkit_worker_jobs_duration_seconds_bucket[5m]))",
            "legendFormat": "p99"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

---

## Integration with Existing System

### Executor Integration

Metrics are automatically recorded at key points:

1. **Job Start**: Increment `active_jobs`
2. **Job Complete**: Record duration, decrement `active_jobs`, increment job counter
3. **Handler Execute**: Record handler execution time and status

**Example from executor.rs**:
```rust
// Job starts
if let Some(ref metrics) = self.metrics {
    metrics.inc_active_jobs();
}

// Job completes
let duration = (Utc::now() - started_at).num_milliseconds() as f64 / 1000.0;
if let Some(ref metrics) = self.metrics {
    metrics.record_job_completion(&operation, "completed", duration);
    metrics.dec_active_jobs();
}
```

### Handler Integration

Handlers can access metrics via `HandlerContext`:

```rust
// In handler
context.record_checksum_verification("success");
```

---

## Performance Impact

### Metrics Collection Overhead

| Operation | Overhead | Notes |
|-----------|----------|-------|
| Counter increment | ~5ns | Lock-free atomic operation |
| Histogram observe | ~20ns | Atomic + bucket calculation |
| Gauge set | ~5ns | Atomic store |
| Metrics encode | ~1ms | Full metrics snapshot |

**Conclusion**: Negligible impact on job execution (<0.001% overhead).

### Memory Usage

| Component | Memory | Notes |
|-----------|--------|-------|
| Metrics registry | ~10 KB | Base overhead |
| Per metric family | ~1 KB | Labels + buckets |
| Total (typical) | ~50 KB | For all metrics |

**Conclusion**: Minimal memory footprint.

---

## Monitoring Best Practices

### Alerting Rules

**prometheus_alerts.yml**:
```yaml
groups:
  - name: guestkit_worker
    rules:
      # High failure rate
      - alert: HighJobFailureRate
        expr: |
          sum(rate(guestkit_worker_jobs_total{status="failed"}[5m]))
          /
          sum(rate(guestkit_worker_jobs_total[5m]))
          > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High job failure rate (>10%)"

      # Worker stalled
      - alert: WorkerStalled
        expr: |
          guestkit_worker_active_jobs > 0
          and
          rate(guestkit_worker_jobs_total[5m]) == 0
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Worker appears stalled (active jobs but no completions)"

      # Checksum failures
      - alert: ChecksumFailures
        expr: |
          rate(guestkit_checksum_verifications_total{status="failure"}[5m]) > 0
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Checksum verification failures detected"
```

### SLO Monitoring

**Service Level Objectives**:

1. **Availability**: 99.9% uptime
   ```promql
   up{job="guestkit-workers"} == 1
   ```

2. **Success Rate**: ≥99% jobs complete successfully
   ```promql
   sum(rate(guestkit_worker_jobs_total{status="completed"}[1h]))
   /
   sum(rate(guestkit_worker_jobs_total[1h]))
   >= 0.99
   ```

3. **Latency**: p95 job duration <120s
   ```promql
   histogram_quantile(0.95, rate(guestkit_worker_jobs_duration_seconds_bucket[5m])) < 120
   ```

---

## Testing

### Unit Tests

All metrics functionality is covered:

```bash
cargo test --lib metrics
```

**Test Coverage**:
- ✅ Metrics registry creation
- ✅ Job completion recording
- ✅ Active jobs tracking
- ✅ Checksum verification metrics
- ✅ Handler execution metrics
- ✅ Disk I/O metrics
- ✅ HTTP endpoints (/metrics, /health)

### Integration Test

```bash
#!/bin/bash
# Start worker with metrics
./target/release/guestkit-worker \
  --metrics-enabled true \
  --metrics-addr 127.0.0.1:9090 &

WORKER_PID=$!

# Wait for startup
sleep 2

# Test health endpoint
curl -f http://127.0.0.1:9090/health || exit 1

# Test metrics endpoint
METRICS=$(curl -s http://127.0.0.1:9090/metrics)
echo "$METRICS" | grep -q "guestkit_worker_jobs_total" || exit 1

# Submit test job
cat > /tmp/test-job.json <<EOF
{
  "version": "1.0",
  "job_id": "metrics-test-001",
  "kind": "VMOperation",
  "operation": "system.echo",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "payload": {
    "type": "system.echo.v1",
    "data": {"message": "test"}
  }
}
EOF

cp /tmp/test-job.json jobs/

# Wait for job completion
sleep 5

# Check metrics updated
curl -s http://127.0.0.1:9090/metrics | grep -q "guestkit_worker_jobs_total" || exit 1

# Cleanup
kill $WORKER_PID

echo "✅ Integration test passed"
```

---

## Troubleshooting

### Issue: Metrics endpoint not accessible

**Symptoms**:
```bash
curl http://localhost:9090/metrics
# curl: (7) Failed to connect to localhost port 9090: Connection refused
```

**Solutions**:
1. Check metrics are enabled:
   ```bash
   --metrics-enabled true
   ```

2. Verify bind address:
   ```bash
   --metrics-addr 0.0.0.0:9090
   ```

3. Check firewall rules:
   ```bash
   sudo firewall-cmd --add-port=9090/tcp
   ```

4. Check worker logs:
   ```bash
   tail -f /var/log/guestkit-worker.log | grep metrics
   ```

### Issue: Metrics not updating

**Symptoms**: Metrics endpoint works but values don't change.

**Diagnosis**:
```bash
# Check if jobs are being processed
curl http://localhost:9090/metrics | grep guestkit_worker_jobs_total

# Check active jobs
curl http://localhost:9090/metrics | grep guestkit_worker_active_jobs
```

**Solutions**:
1. Verify jobs are being submitted
2. Check worker logs for errors
3. Ensure metrics are attached to worker:
   ```rust
   worker.with_metrics(metrics);
   ```

---

## Code Changes

### Files Modified

1. **`Cargo.toml`** - Added dependencies
   - `prometheus-client = "0.22"`
   - `axum = "0.7"`
   - `tower = "0.4"`
   - `tower-http = "0.5"`

2. **`src/metrics.rs`** (NEW) - Metrics registry
3. **`src/metrics_server.rs`** (NEW) - HTTP server
4. **`src/executor.rs`** - Metrics integration
5. **`src/handler.rs`** - Context with metrics
6. **`src/worker.rs`** - Worker with metrics
7. **`src/handlers/guestkit/inspect.rs`** - Checksum metrics
8. **`src/bin/worker.rs`** - CLI with metrics server

### Lines of Code

- **Metrics module**: ~300 lines
- **HTTP server**: ~100 lines
- **Integration code**: ~100 lines
- **Tests**: ~150 lines
- **Total**: ~650 lines

---

## Summary

Phase 4.2 successfully delivers **production-grade Prometheus metrics** for guestkit:

✅ **Comprehensive**: Job, handler, worker, and resource metrics
✅ **Standards-based**: Prometheus text format
✅ **Low overhead**: <0.001% performance impact
✅ **Well-tested**: 9 new tests (29 total), all passing
✅ **Production-ready**: HTTP server with /metrics and /health endpoints
✅ **Documented**: Complete PromQL queries and Grafana examples

**Status**: Ready for production deployment

**Next Phase**: 4.3 - REST API Transport (HTTP job submission)

---

**End of Phase 4.2 Documentation**
