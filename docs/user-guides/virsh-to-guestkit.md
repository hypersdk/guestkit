# Dump virsh — use GuestKit

GuestKit is the offline + live *guest* tool. It inspects disks, scores
boot readiness, repairs guests, and speaks the QEMU guest-agent socket.
It is **not** a libvirt replacement. Domain lifecycle (define / start /
destroy / migrate the hypervisor object) stays with the hypervisor
control plane: KubeVirt `virtctl` / `kubectl`, or Machina.

This page is the cut-over map so scripts can delete every `virsh`
invocation.

## Guest / disk operations (use GuestKit)

| Old virsh / libvirt habit | GuestKit |
|---|---|
| `virsh qemu-agent-command $vm '{"execute":"guest-ping"}'` | `guestkit qga --socket $SOCK --execute guest-ping` |
| `virsh qemu-agent-command $vm '{"execute":"guest-exec",...}'` | `guestkit qga --socket $SOCK --execute guest-exec --arguments '{"path":"/bin/true"}'` |
| `virsh qemu-agent-command $vm '{"execute":"guestkit-rpc",...}'` | `guestkit agent-call --socket $SOCK --method guestkit.getVersion` |
| Guess “will it boot?” by `virsh start` + `virsh console` | `guestkit doctor disk.qcow2 --target kvm --explain` |
| Hand-edit fstab after `virt-v2v` | `guestkit migrate-plan disk.qcow2 --target kvm --export plan.yaml` then `guestkit plan apply` |
| `guestfish` / libguestfs appliance | `guestkit inspect disk.qcow2`, `guestkit extract`, `guestkit rescue` |
| `virsh dumpxml` just to find the disk path | `guestkit inspect disk.qcow2 --output json` |
| Serial console “did it boot?” | `guestkit agent-call --method guestkit.getBootAnalysis` |
| Snapshot FS freeze | GuestKit agent implements `guest-fsfreeze-freeze` / `thaw` on the QGA channel |

Typical libvirt QGA socket:

```text
/var/lib/libvirt/qemu/channel/target/<domain>/org.qemu.guest_agent.0
```

KubeVirt virt-launcher (inside the `compute` container):

```text
/var/run/kubevirt-private/libvirt/qemu/channel/target/org.qemu.guest_agent.0
```

`guestkit qga` auto-discovers those paths when `--socket` is omitted.

## Hypervisor object lifecycle (do **not** pretend GuestKit does this)

| Old virsh | Replacement |
|---|---|
| `virsh list --all` | `virtctl get vmi -A` / `kubectl get vmi -A` (KubeVirt) or Machina |
| `virsh start` / `shutdown` / `destroy` | `virtctl start\|stop\|restart` |
| `virsh console` | `virtctl console` or SPICE/VNC via Machina |
| `virsh define` / `undefine` / `dumpxml` | KubeVirt VMI/VM YAML, or Machina domain API |
| `virsh migrate --live` | KubeVirt live migration / Machina |
| `virsh net-*` / `pool-*` | CNI + storage control plane (Atlas), not GuestKit |

## zyvor-api behaviour

`crates/zyvor-api/src/kubevirt_qga.rs` execs a socket client inside
virt-launcher. Order:

1. `guestkit qga --raw …`
2. `python3` / `python` unix-socket client
3. `perl` `IO::Socket::UNIX`
4. `socat` / `nc -U`
5. `virsh qemu-agent-command` **only if** `GUESTKIT_ALLOW_VIRSH=1`

## Script rewrite examples

```bash
# before
virsh qemu-agent-command web01 '{"execute":"guest-ping"}'

# after
guestkit qga --execute guest-ping
# or, pinned:
guestkit qga --socket /var/lib/libvirt/qemu/channel/target/web01/org.qemu.guest_agent.0 \
  --execute guest-ping
```

```bash
# before
virsh start web01 && virsh console web01

# after
guestkit doctor /var/lib/libvirt/images/web01.qcow2 --target kvm --explain
# start the domain with virtctl/Machina, then:
guestkit agent-call --method guestkit.getBootAnalysis
```
