# Zeus VM Tools

**Product name:** Zeus VM Tools  
**Binary:** `zyvor-guest-agent`  
**Package:** `zyvor-vm-tools` (RPM/DEB/tar.gz/ISO)

Kubernetes-native guest agent, drivers bootstrap, and migration assurance for KubeVirt VMs — the VMware Tools equivalent for Zeus OS.

## Phase 1 capabilities

| Feature | Status |
|---------|--------|
| Linux agent (`zyvor-guest-agent`) | Shipped |
| `guestkit.getStatus` RPC (identity + heartbeat) | Shipped |
| Cloud-init install via zyvor-api | Shipped |
| Windows virtio-win path (Zeus deep link) | Shipped |
| Fleet coverage API | Shipped |
| CRDs (`VMToolsBundle`, `VMGuestAgent`, `VMToolsPolicy`) | Schema shipped |
| Windows MSI | Phase 3 |

## Phase 2 capabilities

| Feature | Status |
|---------|--------|
| Snapshot quiesce / unquiesce (`freeze` / `unfreeze`) | Shipped |
| Guest soft reboot (`softreboot`) | Shipped |
| Graceful shutdown (`stop` with grace period) | Shipped |
| QGA bootstrap install script (live qemu-ga) | Shipped |
| Fleet coverage UI chip | Shipped |
| Guest exec via API | Blocked (KubeVirt has no guest-exec subresource) |

## API routes

| Method | Path |
|--------|------|
| GET | `/api/v1/vmtools/bundle` |
| GET | `/api/v1/vmtools/coverage` |
| GET | `/api/v1/kubevirt/vms/{ns}/{name}/vmtools` |
| POST | `/api/v1/kubevirt/vms/{ns}/{name}/vmtools/install` |
| POST | `/api/v1/kubevirt/vms/{ns}/{name}/vmtools/diagnostics` |
| POST | `/api/v1/kubevirt/vms/{ns}/{name}/vmtools/quiesce` |
| POST | `/api/v1/kubevirt/vms/{ns}/{name}/vmtools/unquiesce` |
| POST | `/api/v1/kubevirt/vms/{ns}/{name}/vmtools/reboot` |
| POST | `/api/v1/kubevirt/vms/{ns}/{name}/vmtools/shutdown` |
| POST | `/api/v1/kubevirt/vms/{ns}/{name}/vmtools/exec` |

## Install methods

1. **Cloud-init** — Zeus/Zyvor UI **Install VM Tools** (Linux, VM stopped or after restart)
2. **QGA bootstrap** — when qemu-ga is connected but Zyvor agent is missing, install returns `bootstrap_script` to run in the guest console
3. **ISO attach** — build with `packaging/vmtools/build-artifacts.sh`
4. **Offline inject** — `guestkit repair --inject-agent` (GuestKit)

## Build packages

```bash
./packaging/vmtools/build-artifacts.sh
```

## VM labels (synced by zyvor-api)

```yaml
metadata:
  labels:
    zeus.zyvor.dev/guest-tools: installed|connected|missing
  annotations:
    zeus.zyvor.dev/tools-version: "0.1.0"
    zeus.zyvor.dev/last-heartbeat: "..."
```

See also [guest-agent.md](guest-agent.md) and [kubevirt-integration.md](kubevirt-integration.md).
