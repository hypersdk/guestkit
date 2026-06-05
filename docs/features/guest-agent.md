# GuestKit Guest Agent

GuestKit can run **inside** a Linux guest as an in-guest agent (similar to `qemu-guest-agent`), communicating with the host over virtio-serial while reusing the same evidence schema and fix-plan format as offline assurance.

## Architecture

```
Host (libvirt / KubeVirt virt-launcher)
          │  virtio-serial  org.qemu.guest_agent.0
          ▼
Guest VM: guestkit agent --channel virtio
```

Protocol: JSON-RPC 2.0 over 4-byte big-endian length-prefixed frames. QGA `{"execute":...}` commands are also supported on the same channel for KubeVirt compatibility.

## Build

```bash
# Default offline tool (unchanged)
cargo build --release

# In-guest agent artifact (static musl recommended for injection)
cargo build --release --features agent --no-default-features \
  --target x86_64-unknown-linux-musl
```

## Run in guest

```bash
guestkit agent --channel virtio
# Dev/test without VM:
guestkit agent --channel stdio
```

Systemd unit template: [`templates/agent/guestkit-agent.service`](../templates/agent/guestkit-agent.service)

## Host-side proxy

Connect to the libvirt channel unix socket and expose HTTP:

```bash
guestkit agent-proxy \
  --socket /var/lib/libvirt/qemu/channel/target/$VM/org.qemu.guest_agent.0 \
  --listen 127.0.0.1:8765
```

## Host-side one-shot RPC

For automation (e.g. VMRogue pod exec):

```bash
guestkit agent-call \
  --socket /var/run/kubevirt-private/.../org.qemu.guest_agent.0 \
  --method guestkit.getEvidence \
  --params '{}'
```

HTTP endpoints (via `agent-proxy`):

| Method | Path | RPC |
|--------|------|-----|
| GET | `/evidence` | `guestkit.getEvidence` |
| GET | `/doctor` | `guestkit.doctor` |
| GET | `/ping` | `guestkit.ping` |
| POST | `/fix-plan` | `guestkit.runFixPlan` |

## KubeVirt channel declaration

```yaml
spec:
  domain:
    devices:
      channels:
      - name: org.qemu.guest_agent.0
        target:
          type: virtio
          name: org.qemu.guest_agent.0
```

Guest opens: `/dev/virtio-ports/org.qemu.guest_agent.0`

## Offline injection

During migration prep:

```bash
guestkit repair /path/to/disk.qcow2 --fix boot --inject-agent \
  --agent-binary ./target/x86_64-unknown-linux-musl/release/guestkit
```

## QGA compatibility

GuestKit implements common QGA commands including `guest-ping`, `guest-exec`, `guest-fsfreeze-freeze/thaw`, `guest-network-get-interfaces`, and `guest-get-host-name` for KubeVirt `AgentConnected`, snapshot freeze, and guest-exec subresources.

## RPC methods

- `guestkit.ping`
- `guestkit.getVersion`
- `guestkit.getCapabilities`
- `guestkit.getEvidence`
- `guestkit.doctor`
- `guestkit.migrateScore`
- `guestkit.runFixPlan`
- `guestkit.runFixPlanRollback`

## Worker jobs

When `guestkit-worker` runs with a live VM:

- `guestkit.agent.evidence` — fetch via agent-proxy HTTP
- `guestkit.agent.fix` — apply fix plan via agent-proxy

## Security

Same trust model as qemu-guest-agent: only the hypervisor host can connect to the virtio channel. Dangerous operations require `fix_apply` capability.
