# GuestKit VM lifecycle (`guestkit vm`)

GuestKit-native local QEMU lifecycle — a deliberately small virsh/libvirt
replacement for **host-local** VMs. GuestKit owns inspection and boot
assurance; QEMU owns execution; QMP owns day-2 power/pause ops.

There is no libvirt XML and no virsh dependency on this path.

KubeVirt / OpenShift domain objects still stay with `virtctl` / Machina.
Use `guestkit vm` when you are launching a disk on the host with QEMU
directly (lab, CI, convert-and-boot).

## Commands

```bash
export GUESTKIT_VM_DIR=/var/lib/guestkit/vms
export GUESTKIT_VM_RUN_DIR=/run/guestkit

guestkit vm define demo /path/to/demo.qcow2 --memory-mb 4096 --vcpus 2
guestkit vm plan demo
guestkit vm list
guestkit vm start demo
guestkit vm status demo
guestkit vm shutdown demo
guestkit vm reboot demo
guestkit vm pause demo
guestkit vm resume demo
guestkit vm destroy demo
guestkit vm undefine demo
```

## Assurance gate

`define` / `plan` / `start` re-run GuestKit doctor against the image.
`start` refuses by default when the boot score is below `--min-boot-score`
(or when blockers are present). Pass `--force` to override.

## Networking

Default is user-mode networking. Optional `--ssh-port` forwards
`127.0.0.1:PORT → guest:22` only (loopback). `--tap IFACE` uses a
pre-created host TAP; GuestKit does not create bridges.

## UEFI

Pass host firmware explicitly:

```bash
guestkit vm define uefi-demo disk.qcow2 \
  --uefi-code /usr/share/OVMF/OVMF_CODE.fd \
  --uefi-vars /var/lib/guestkit/vms/uefi-demo_VARS.fd
```

## Relation to `guestkit-qemu`

| Tool | Role |
|------|------|
| `guestkit-qemu plan\|run` | One-shot argv from an image path (no persisted definition) |
| `guestkit vm` | Named definitions under `$GUESTKIT_VM_DIR` + QMP lifecycle |

Prefer `guestkit vm` for repeated start/stop of the same guest; prefer
`guestkit-qemu` for a disposable assurance → argv plan.

## See also

- [QEMU / VirtIO runtime](qemu-runtime.md)
- [Dump virsh](../user-guides/virsh-to-guestkit.md)
- [Handoff / quarantine](../user-guides/handoff-quarantine.md)
