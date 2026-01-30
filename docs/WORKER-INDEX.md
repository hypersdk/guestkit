# Guestkit Worker System Documentation

Documentation for the **guestkit-worker** distributed job processing system.

**Main Docs**: See [README.md](README.md) for guestctl CLI tool documentation.

---

## 🚀 Quick Start

- **[Worker Quickstart](guides/quickstart.md)** - Get the worker running in 5 minutes
- **[CLI Guide](CLI-GUIDE.md)** - Complete command-line interface reference
- **[Docker Deployment](guides/DOCKER-QUICKSTART.md)** - Run worker in containers
- **[Kubernetes Guide](guides/K8S-DEPLOYMENT.md)** - Deploy at scale
- **[k9s Guide](guides/K9S-GUIDE.md)** - Manage Kubernetes with k9s

---

## 📖 Phase Documentation

Complete implementation history:

### Phase 1: Foundation
**[Phase 1 Complete](phases/phase-1/PHASE-1-COMPLETE.md)**
- Job Protocol v1.0 specification
- Worker daemon implementation
- File-based transport

### Phase 2: Handlers
**[Phase 2 Complete](phases/phase-2/PHASE-2-COMPLETE.md)**
- Echo handler (testing)
- Inspect handler (VM inspection)
- Profile handler (security)

### Phase 3: Integration
**[Phase 3 Complete](phases/phase-3/PHASE-3-COMPLETE.md)**
- Real guestkit library integration
- VM disk inspection
- Package and service detection
- **[Integration Summary](phases/phase-3/PHASE-3-INTEGRATION-SUMMARY.md)**

### Phase 4: Production Features
**[Phase 4 Overview](phases/phase-4/PHASE-4-OVERVIEW.md)** - Security, observability, scalability

#### Phase 4.1: SHA256 Checksum Verification ✅
- **[Feature Guide](phases/phase-4/PHASE-4.1-CHECKSUM-VERIFICATION.md)**
- **[Session Summary](phases/phase-4/PHASE-4.1-SESSION-SUMMARY.md)**
- Cryptographic image verification
- Protection against corruption/tampering
- Audit trail for compliance

#### Phase 4.2: Prometheus Metrics ✅
- **[Feature Guide](phases/phase-4/PHASE-4.2-PROMETHEUS-METRICS.md)**
- **[Session Summary](phases/phase-4/PHASE-4.2-SESSION-SUMMARY.md)**
- 13 comprehensive metrics
- HTTP server (port 9090)
- /metrics and /health endpoints

#### Phase 4.3: REST API ✅
- **[Feature Guide](phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md)**
- **[Session Summary](phases/phase-4/PHASE-4.3-SESSION-SUMMARY.md)**
- 6 REST endpoints
- HTTP job submission
- JSON request/response
- Python/TypeScript clients

---

## 🎯 Features

### Worker System
- **[Worker Implementation](features/worker/WORKER-IMPLEMENTATION-COMPLETE.md)**
- Distributed job processing
- Pluggable transport layer
- Handler registry pattern

### Explore Command
- **[Explore Quickstart](features/explore/EXPLORE-QUICKSTART.md)**
- **[Command Reference](features/explore/EXPLORE-COMMAND.md)**
- **[Development Summary](features/explore/EXPLORE-DEVELOPMENT-SUMMARY.md)**

### TUI File Browser
- **[Files View](features/tui/TUI-FILES-VIEW.md)**
- **[Navigation](features/tui/TUI-FILES-NAVIGATION.md)**
- **[Preview](features/tui/TUI-FILES-PREVIEW-INFO.md)**
- **[Filtering](features/tui/TUI-FILES-FILTER.md)**

---

## 🛠️ Development

### Project Status
- **[Complete System Summary](development/COMPLETE-SYSTEM-SUMMARY.md)** - Overall status
- **[CLI Development Summary](development/CLI-DEVELOPMENT-SUMMARY.md)** - CLI implementation
- **[Session Logs](development/SESSION-CONTINUATION-2026-01-30.md)** - Latest work

### Building & Packaging
- **[RPM Build](development/RPM-BUILD.md)**
- **[Docker Build](development/DOCKER-BUILD-FIX-SUMMARY.md)**

### Contributing
- **[Contributing Guide](development/CONTRIBUTING.md)**
- **[Changelog](development/CHANGELOG.md)**
- **[Commands Summary](development/COMMANDS_SUMMARY.md)**
- **[Future Enhancements](FUTURE-ENHANCEMENTS.md)** — Roadmap and planned features

---

## 📊 API Reference

### REST API (Port 8080)
```bash
POST   /api/v1/jobs              # Submit job
GET    /api/v1/jobs              # List jobs
GET    /api/v1/jobs/:id          # Get status
GET    /api/v1/jobs/:id/result   # Get result
GET    /api/v1/capabilities      # Worker capabilities
GET    /api/v1/health            # Health check
```

**Full API Docs**: [REST API Transport](phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md)

### Metrics API (Port 9090)
```bash
GET /metrics    # Prometheus metrics
GET /health     # Health check
```

**Full Metrics Docs**: [Prometheus Metrics](phases/phase-4/PHASE-4.2-PROMETHEUS-METRICS.md)

---

## 📈 Statistics

### Codebase
- **5,400+ lines** of Rust code
- **39 unit tests** (100% passing)
- **33 source files**
- **2 production crates**

### Features Delivered
- ✅ Job Protocol v1.0
- ✅ Worker daemon
- ✅ 3 operation handlers
- ✅ File & HTTP transports
- ✅ SHA256 verification
- ✅ Prometheus metrics (13 metrics)
- ✅ REST API (6 endpoints)

### Servers
- **Port 8080**: REST API
- **Port 9090**: Prometheus metrics

---

## 🎓 Use Cases

### Submit Jobs via REST API
```bash
curl -X POST http://localhost:8080/api/v1/jobs \
  -H "Content-Type: application/json" \
  -d @job.json
```

### Monitor with Prometheus
```bash
curl http://localhost:9090/metrics | grep guestkit
```

### Verify Image Integrity
```json
{
  "payload": {
    "data": {
      "image": {
        "path": "/vms/prod.qcow2",
        "checksum": "sha256:abc123..."
      }
    }
  }
}
```

---

## 🔗 Quick Links

### Deployment
- [Docker Guide](guides/DOCKER.md)
- [Kubernetes Deployment](guides/K8S-DEPLOYMENT.md)
- [k9s Guide](guides/K9S-GUIDE.md)
- [Quickstart](guides/quickstart.md)

### Development
- [Phase 4 Overview](phases/phase-4/PHASE-4-OVERVIEW.md)
- [System Summary](development/COMPLETE-SYSTEM-SUMMARY.md)
- [Contributing](development/CONTRIBUTING.md)

### API & Integration
- [REST API](phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md)
- [Metrics](phases/phase-4/PHASE-4.2-PROMETHEUS-METRICS.md)
- [Job Protocol](job-protocol-v1.md)

---

## Navigation

- **[← Main Docs](README.md)** - Guestctl CLI documentation
- **[↑ Index](INDEX.md)** - Complete documentation index

---

**Last Updated**: 2026-01-30
**Worker Version**: 0.1.0 (Phase 4.3)
