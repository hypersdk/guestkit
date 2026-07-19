# GuestKit Agent — Protocol 1.3 (Phase 1+2)

Protocol 1.3 turns the in-guest agent into the enterprise VM intelligence
surface described in the product spec: rich heartbeat, rolling performance
telemetry, a hardened security choke point, and live migration assurance
(assess → plan → repair → cutover → validate).

## Naming and packaging

| Component | Canonical name | Compatibility |
|---|---|---|
| Linux daemon | `guestkitd` | `zyvor-guest-agent` symlink |
| Privileged helper | `guestkitd-exec` | `zyvor-guest-agent-exec` symlink |
| Local CLI | `guestkitctl` | — |
| systemd unit | `guestkit-agent.service` (hardened) | `Alias=zyvor-guest-agent.service` |
| Windows service | `GuestKitAgent` ("Zyvor GuestKit Agent") | MSI/install.ps1 removes old `ZyvorGuestAgent` |
| Policy file | `/etc/guestkit/agent-policy.yaml` | `/etc/zyvor/agent-policy.yaml` fallback |
| Local socket | `/run/guestkit/agent.sock` | `/var/run/zyvor/guest-agent.sock` also bound |

## Channels

| Channel | Purpose | Push allowed |
|---|---|---|
| `org.qemu.guest_agent.0` | QGA compatibility + GuestKit RPC (KubeVirt default) | **never** — a plain-QGA host would choke on unsolicited frames |
| `org.zyvor.guestkit.0` | Dedicated GuestKit channel (plain libvirt/QEMU) | after `guestkit.subscribeEvents` |
| `com.zyvor.guestkit.0` | Legacy, deprecated | after subscribe |
| vsock / local socket | High-throughput / local CLI | after subscribe (vsock) |

KubeVirt VMIs cannot declare arbitrary virtio-serial channels, so on
KubeVirt the agent stays on the QGA channel (pull-only). For plain libvirt,
add the dedicated channel:

```xml
<channel type="unix">
  <target type="virtio" name="org.zyvor.guestkit.0"/>
</channel>
```

The daemon serves every available channel concurrently and skips missing
secondary channels.

## Request envelope (1.3)

Optional fields on every JSON-RPC request; absent fields = legacy client:

```json
{
  "jsonrpc": "2.0", "method": "guestkit.reboot", "id": 7,
  "ts": "2026-07-19T12:00:00Z", "ttl_ms": 30000,
  "nonce": "n-8fe2...", "idempotency_key": "k-restart-1"
}
```

- `ts` + `ttl_ms`: expiry window; expired mutating requests are rejected
  with `-32003 request_expired`. `security.require_request_expiry: true`
  makes the envelope mandatory.
- `nonce`: single-use; replays rejected with `-32004 replay_detected`.
- `idempotency_key`: retried mutating requests return the cached response.
- Policy denials use `-32005 policy_denied`.

Dotted spec-style aliases are accepted (`service.restart`, `agent.health`,
`migration.assess`, …) alongside the canonical `guestkit.*` names.

## New methods

| Group | Methods |
|---|---|
| Health/events | `getAgentHealth`, `subscribeEvents`, `unsubscribeEvents` (+ pushed `guestkit.event.heartbeat`) |
| Telemetry | `getCpuStats`, `getMemoryStats`, `getPerformanceSummary`, `getPerformanceHistory` (1s×15min / 10s×6h / 1min×7d rings) |
| Services/process | `startUnit`, `stopUnit`, `getProcess` |
| Network/storage | `networkTest`, `storageRescan`, `storageTrim`, `storageExpand` (policy-gated, dry-run default) |
| Files | `fileRead/Write/Stat/List/Checksum` (disabled by default; path allowlist, 1 MiB cap, no symlink escape) |
| Time/power | `timeSyncNow`, `setTime`, `reboot`, `shutdown` (approval-gated) |
| Migration | `migration.assess/plan/repair/preCheck/cutoverEnter/cutoverExit/validate`, `baseline.capture/diff` |

## Heartbeat

`getAgentHealth` (and the pushed event) return:

```json
{
  "seq": 42, "agent_state": "healthy", "boot_id": "8b51...",
  "os_uptime_secs": 86400, "cpu_usage_percent": 11.0,
  "memory_usage_percent": 22.5, "root_disk_usage_percent": 41,
  "pressure": {"cpu": 0.11, "memory": 0.02, "io": 0.07},
  "pending_reboot": false, "critical_services_failed": [],
  "migration_ready": true, "fs_frozen": false
}
```

States: `starting`, `connected`, `healthy`, `degraded`, `updating`,
`quiesced`, `recovery_mode`.

## Migration assurance flow

```
guestkit.migration.assess     → score + 6 sub-scores + blockers (MIG-G/L/W checks)
guestkit.migration.plan       → auditable FixPlan + planner notes (destructive ops excluded by default)
guestkit.migration.repair     → dry-run by default; {"dry_run":false,"confirm":true} + policy to apply
guestkit.migration.preCheck   → readiness token (HMAC, 1h) + pre-migration baseline
guestkit.migration.cutoverEnter → stop services, freeze, watchdog auto-thaw (≤600 s)
guestkit.migration.cutoverExit  → thaw + restart services
guestkit.migration.validate   → boot/network validation + before/after drift report
```

Safety invariants:

- Repair plans refuse High/Critical-risk operations without undo info.
- Destructive repairs (VMware Tools uninstall, ghost-NIC removal) need the
  `actions.migration.repair_destructive` policy AND `include_destructive`.
- Every applied operation records before-state, planned change, backup
  path, validation outcome, and rollback procedure (`outcomes[]`).
- A crashed agent that left filesystems frozen thaws itself at startup
  (cutover marker recovery); the freeze watchdog thaws at its deadline.

The same engine runs offline: `guestkit migrate-assess <image> --target kvm
[--fail-below 80]` and `guestkit migrate-repair <image> --target kvm
[--apply]`.

## Policy (excerpt)

See `templates/agent/agent-policy.yaml` for the full annotated file:

```yaml
capabilities:
  file_ops: { enabled: false, allowed_paths: [], max_bytes: 1048576 }
  storage_ops: { rescan: true, trim: true, expand: false }
methods:
  deny: ["guestkit.exec"]
actions:
  migration: { assess: true, repair: false, repair_destructive: false }
security:
  require_request_expiry: false
  max_ttl_ms: 300000
```

`getCapabilities` advertises the enabled categories so hosts can adapt.

## Known limitations

- Windows cutover freeze is **crash-consistent only** until the VSS
  requestor lands (snapshot phase); capability negotiation reflects this.
- Telemetry history resets on agent restart (persistence deferred).
- `guestkitctl` requires the Unix local socket; the Windows named-pipe
  transport is planned.
- Windows e2e coverage is manual/nightly; registry-fixture tests cover the
  offline analysis path.

## KubeVirt status enrichment

The controller merges assessments into the `VMGuestAgent` CR:

```yaml
status:
  migrationReadiness:
    score: 87
    readiness: ready_with_warnings
    subScores: { boot: 95, storage: 90, network: 80, driver: 85, application: 90, security: 75 }
    blockers: []
    target: kubevirt
```
