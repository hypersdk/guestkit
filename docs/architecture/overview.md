# GuestKit architecture (v0.3.14)

Offline VM intelligence and migration assurance — **Rust control plane** with host block-device access for mount-heavy operations.

## Positioning (accurate)

| Claim | Reality |
|-------|---------|
| **No libguestfs appliance** | ✅ No libguestfs daemon or guestfish workflow |
| **Pure Rust parsing** | ✅ Partition tables, FS signatures, evidence schema, boot engine, assurance APIs |
| **In-process QCOW2 file read** | Partial — format detection + selective reads; full cluster walk defers to **qemu-nbd** |
| **File access inside guests** | Via **loop devices / qemu-nbd** + host mount (`src/guestfs/`), not in-process ext4/NTFS parsers |
| **Web UI** | ✅ Shipped — `deploy/ui/`, GHCR `zyvor-ui` |

**Host dependencies (Linux):** `losetup`, `qemu-nbd` (for QCOW2/VMDK), kernel `nbd`/`loop` modules, optional `qemu-img` for format conversion.

## System layers

```text
┌─────────────────────────────────────────────────────────────────┐
│  Surfaces: guestkit CLI · guestctl TUI · Python · zyvor-ui      │
├─────────────────────────────────────────────────────────────────┤
│  Assurance: evidence → boot score → migrate-plan → fleet/policy │
├─────────────────────────────────────────────────────────────────┤
│  AI (optional): deterministic intel + `--features ai` LLM agent │
├─────────────────────────────────────────────────────────────────┤
│  guestfs façade: Rust orchestration + NBD/loop + mount        │
├─────────────────────────────────────────────────────────────────┤
│  Disk parsers: MBR/GPT, FS magic, registry/hive (Rust)          │
└─────────────────────────────────────────────────────────────────┘

Parallel platform runtime (same repo):
  zyvor-api (Axum) → Redis job queue → guestkit-worker (privileged)
  PostgreSQL · KubeVirt client · guest-agent mTLS · PacketWolf hooks
```

## Repository layout

| Path | Role |
|------|------|
| `src/` | Main `guestkit` crate — CLI, guestfs, boot, evidence, assurance, fleet, TUI |
| `src/ai/` | Deterministic intelligence; LLM agent behind `ai` feature |
| `crates/guestkit-job-spec` | Worker job schema |
| `crates/guestkit-worker` | Redis-queue disk inspection daemon |
| `crates/guestkit-agent-protocol` | Guest agent JSON-RPC framing |
| `crates/zyvor-api` | Web API, auth, KubeVirt, guest agent CA |
| `crates/zyvor-guest-agent` | In-guest agent binary |
| `deploy/` | Docker Compose, Helm, UI static assets |
| `k8s/` | KubeVirt-oriented manifests |

## Core data flow

```text
Disk image → guestfs mount (ro) → EvidenceSnapshot
                                        │
                    ├─► BootabilityReport (doctor)
                    ├─► MigrationScoreReport (migrate-plan)
                    ├─► Policy evaluation (policy check)
                    ├─► Fleet clusters (fleet analyze)
                    └─► FixPlan (repair / plan apply)
```

Evidence is cached under `~/.cache/guestkit/` after successful `doctor` runs.

## Crates and workspace

Root `Cargo.toml` workspace includes the main package and `guestkit-job-spec`. `zyvor-api` and `guestkit-worker` build as sibling crates with path dependencies (see each crate `Cargo.toml`).

**Version:** 0.3.14 (package) · **License:** Apache-2.0 · **Owner:** ZyvorAI Labs Private Limited

## Security model (web stack)

- **Eval default:** `AUTH_ENABLED=false` in `deploy/docker-compose.ghcr.yml` — localhost demos only
- **Production:** auth on, `JWT_SECRET`, `AGENT_BOOTSTRAP_TOKEN`, Redis password — see [DOCKER.md](../guides/DOCKER.md#production-checklist)
- **OIDC:** ID tokens verified via JWKS (signature, issuer, audience, expiry)

## Further reading

- [Migration assurance](../features/migration-assurance.md)
- [KubeVirt integration](../features/kubevirt-integration.md)
- [CE vs Enterprise](../ce-vs-enterprise.md)
- [User stories](../USER_STORIES.md)
