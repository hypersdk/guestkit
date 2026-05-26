# Migration assurance platform

GuestKit treats each disk image as a **digital twin**: an offline `EvidenceSnapshot` plus scoring engines that answer “will this VM boot?” and “what must change before cutover?” — without powering the guest on.

## Architecture

```text
Disk image (QCOW2/VMDK/…)
        │
        ▼
  guestfs mount (read-only)
        │
        ▼
  EvidenceSnapshot          ← fstab, modules, VM tools, Windows signals, …
        │
        ├─► BootabilityReport   (doctor / boot engine)
        ├─► MigrationScoreReport (migrate-plan)
        ├─► Policy validation   (policy check + expression DSL)
        ├─► Fleet clusters      (fleet analyze)
        └─► FixPlan             (repair --fix boot)
```

| Module | Role |
|--------|------|
| `src/evidence/` | Normalized snapshot schema (`EvidenceSnapshot`, v1) |
| `src/boot/` | Weighted bootability checks, blockers, warnings |
| `src/cli/migrate/plan.rs` | Hypervisor-aware migration scoring |
| `src/inference/` | Root-cause chain for `--explain` |
| `src/fleet/` | Cluster identical VMs, snowflakes, blockers |
| `src/cli/plan/` | Fix plans — security profiles **and** boot repair |

Evidence is cached under `~/.cache/guestkit/` when `doctor` runs successfully.

## Commands

### `guestkit doctor` — boot probability

Predicts first-boot success on a target hypervisor before migration.

```bash
guestkit doctor vm.qcow2 --target kvm
guestkit doctor vm.vmdk --target proxmox --explain
guestkit doctor vm.qcow2 --target kvm -o json
```

| Flag | Description |
|------|-------------|
| `--target` | `kvm`, `proxmox`, `qemu`, `hyperv`, `aws`, `azure`, `gcp`, `cloud` |
| `--explain` | Root-cause chain from inference engine |
| `-o json` | Machine-readable `bootability` + optional `root_cause` |

Output includes a **boot probability** message, **blockers** (with remediation hints), **warnings**, and per-check pass/fail lines.

### `guestkit migrate-plan` — hypervisor-aware migration score

Builds on the same evidence + boot report, then applies target-specific rules (VirtIO drivers, cloud-init, VMware Tools removal, BitLocker, SELinux relabel, etc.).

```bash
guestkit migrate-plan vm.vmdk --target proxmox
guestkit migrate-plan vm.qcow2 --target aws --explain -o json
```

JSON fields: `migration_score`, `bootability`, `driver_injections`, `required_changes`, `licensing_warnings`, `estimated_downtime_minutes`.

**Target mapping (examples)**

| `--target` | Boot analysis | Migration rules |
|------------|---------------|-----------------|
| `kvm`, `proxmox`, `qemu` | Proxmox/KVM | VirtIO, virtio-scsi/net, VMware Tools → qemu-ga |
| `aws`, `azure`, `gcp`, `cloud` | Cloud | cloud-init datasource, licensing (BYOL) |
| `hyperv`, `hyper-v` | Hyper-V | Hyper-V-specific checks |

### `guestkit policy check` — policy-as-code

Alias over validation with an **expression DSL** over evidence fields, e.g. `bootability.score >= 80`. Use `--policy policy.yaml` or built-in benchmarks.

```bash
guestkit policy check vm.qcow2 --policy cis.yaml
guestkit policy check vm.qcow2 --benchmark cis -o json
```

### `guestkit fleet analyze` — fleet posture

Scans a directory of disk images, clusters identical OS fingerprints, flags snowflakes and low boot-score blockers.

```bash
guestkit fleet analyze ./vms/ -o json
```

### `guestkit forensic-diff` — security drift

Compares two snapshots (before/after incident, golden vs drifted) for config drift, suspicious persistence, and ransomware indicators.

```bash
guestkit forensic-diff before.qcow2 after.qcow2 -o json
```

### `guestkit repair --fix boot` — transactional boot repair

Converts doctor blockers/warnings into a **fix plan**, applies it with backup semantics, then re-runs doctor to show score delta.

```bash
guestkit repair vm.qcow2 --fix boot --dry-run   # preview operations
guestkit repair vm.qcow2 --fix boot             # apply + re-score
```

Plans are tagged `boot` / `doctor` and generated via `PlanGenerator::from_boot_report`.

### `guestkit inspect --profile windows-migration`

Deep Windows signals for migration: BitLocker, domain join, RDP, hypervisor remnants, driver gaps (SAM/SECURITY hive parsing).

```bash
guestkit inspect win.vmdk --profile windows-migration -o json
```

## Recommended workflow

```bash
# 1. Boot gate
guestkit doctor source.vmdk --target proxmox --explain

# 2. Migration checklist + downtime estimate
guestkit migrate-plan source.vmdk --target proxmox -o json > plan.json

# 3. Windows-specific inventory (if applicable)
guestkit inspect source.vmdk --profile windows-migration -o json

# 4. Policy sign-off
guestkit policy check source.vmdk --policy migration-policy.yaml

# 5. Fleet context (many disks)
guestkit fleet analyze ./exports/

# 6. Fix blockers offline, then re-doctor
guestkit repair source.vmdk --fix boot --dry-run
guestkit repair source.vmdk --fix boot
guestkit doctor source.vmdk --target proxmox

# 7. Hand off to hyper2kvm / hypervisor import
```

## Relationship to fix plans

| Plan source | Profile | Use case |
|-------------|---------|----------|
| Security profile | `security` | Hardening from inspect findings |
| Doctor boot report | `boot-repair` | Boot blockers from `repair --fix boot` |
| Migration profile | `migration` | Manual/runbook plans (see [fix-plans.md](fix-plans.md)) |

`migrate-plan` is **scoring and guidance**; it does not auto-apply disk changes. Use **fix plans** or **repair** for offline mutations.

## Library API (Rust)

```rust
use guestkit::evidence::build_evidence;
use guestkit::boot::{analyze_bootability, BootTarget};
use guestkit::cli::migrate::plan::compute_migration_score;

// After guestfs mount: build_evidence → analyze_bootability → compute_migration_score
```

## See also

- [VM migration guide](../user-guides/vm-migration.md) — fstab, registry, hyper2kvm handoff
- [Fix plans](fix-plans.md) — preview, export, apply
- [Security profiles](../user-guides/profiles.md) — migration and windows-migration profiles
- [Changelog](../development/CHANGELOG.md) — unreleased assurance features
