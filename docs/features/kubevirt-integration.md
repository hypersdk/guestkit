# KubeVirt integration (Zyvor API)

GuestKit provides **pure Rust** offline disk intelligence. It does **not** use or require [libguestfs](https://libguestfs.org/). Disk access goes through GuestKit's own `guestfs` module (loop devices, NBD, partition/filesystem parsers) and assurance APIs such as `run_doctor` and `run_boot_inspect`.

When `zyvor-api` runs in-cluster with KubeVirt, it exposes live VM discovery plus **offline boot inspect** for stopped VMs — consumed by Zeus OS Guest Intelligence.

## Boot inspect API

Used when the VM has no running VMI (offline MRI). Resolves the root PVC from the VM spec, locates the disk on the node, then calls `guestkit::run_boot_inspect`.

| Method | Path |
|--------|------|
| GET, POST | `/api/v1/kubevirt/vms/{namespace}/{name}/boot-inspect` |
| GET, POST | `/api/v1/kubevirt/vms/{namespace}/{name}/inspect/boot` |
| POST | `/api/v1/kubevirt/boot-inspect` |

POST body (optional on VM-scoped routes):

```json
{
  "namespace": "default",
  "vm": "my-vm",
  "pvc": "rootdisk-pvc",
  "mode": "boot-inspect",
  "source": "zeus-os"
}
```

Response (`data` object):

| Field | Description |
|-------|-------------|
| `available` | `true` when GuestKit completed offline inspect |
| `source` | `"guestkit"`, `"vm_spec"`, or `"vm_spec_heuristic"` |
| `os_release` | Distribution / version from disk evidence |
| `fstab_valid` | Root mount + fstab UUID check (BOOT-001) |
| `bootloader` | GRUB/systemd-boot/EFI hint from disk |
| `cloud_init_present` | `/etc/cloud` on root filesystem |
| `message` | Doctor summary or fallback explanation |

## Related KubeVirt routes

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/kubevirt/vms` | Fleet VM list |
| GET | `/api/v1/kubevirt/vms/{namespace}/{name}/guest-agent` | Live guest-agent status |
| POST | `/api/v1/kubevirt/vms/{namespace}/{name}/guest-agent/install` | Cloud-init agent bootstrap |

## Disk path resolution

Boot inspect needs a readable path to the root volume. Resolution order:

1. `KUBEVIRT_BOOT_INSPECT_DISK` — explicit path or template with `{namespace}` / `{pvc}`
2. PVC → PV → `hostPath`, CSI `volumeHandle`, or `/dev/longhorn/{handle}`
3. `KUBEVIRT_DISK_ROOT/{namespace}/{pvc}` (and `.qcow2` variants)
4. Longhorn replica scan under `LONGHORN_REPLICAS_ROOT` (default `/var/lib/longhorn/replicas`)

If the VM is **running**, the API returns VM-spec heuristics and advises live guest-agent probes instead of offline disk access.

## Zeus OS Guest Intelligence

Zeus OS proxies these endpoints when `guestkitUrl` is set in migration engines config (`web/src/guestkit_inspect.rs`). Stopped-VM profiles attach `boot_inspect` to the Guest MRI panel.

## Host requirements (zyvor-api)

- Privileged pod or sufficient access to node disk paths / Longhorn devices
- RBAC: `get/list` on `virtualmachines`, `virtualmachineinstances`, `persistentvolumeclaims`, `persistentvolumes`
- **Not required:** libguestfs, guestfish, or virt-inspector

See [deploy/README.md](../../deploy/README.md) and [openapi/zyvor-vm-services.yaml](../../deploy/openapi/zyvor-vm-services.yaml).
