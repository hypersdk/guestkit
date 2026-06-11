# Zyvor VM Services on Kubernetes

Deploy GuestKit workers, zyvor-api, and supporting services for offline VM intelligence and KubeVirt migration planning.

## Components

| Service | Description |
|---------|-------------|
| `zyvor-api` | REST gateway — import, inspect, doctor, migration-plan, KubeVirt boot inspect, provision |
| `guestkit-worker` | Redis Streams worker running GuestKit operations |
| `postgresql` | VM inventory and job metadata |
| `redis` | Job queue (`zyvor:jobs` stream) |
| `minio` | VM image object storage (optional; PVC path also supported) |
| `zyvor-ui` | Minimal web console |

## Quick start

```bash
# Prerequisites: kind, kubectl, helm, docker
./deploy/scripts/kind-kubevirt-quickstart.sh
```

## Helm install

```bash
helm install zyvor deploy/helm/zyvor -n zyvor --create-namespace
```

## API

See [openapi/zyvor-vm-services.yaml](openapi/zyvor-vm-services.yaml).

Example workflow:

```bash
API=http://api.zyvor.local/api/v1

# Import
curl -F "file=@disk.vmdk" $API/vms/import

# Doctor
curl -X POST "$API/vms/{id}/doctor?target=kubevirt&explain=true"

# Migration plan
curl -X POST "$API/vms/{id}/migration-plan?target=kubevirt"

# Provision KubeVirt YAML
curl -X POST "$API/vms/{id}/provision"

# Offline boot inspect (stopped KubeVirt VM — pure Rust GuestKit, not libguestfs)
curl "$API/kubevirt/vms/default/my-vm/boot-inspect"
curl -X POST "$API/kubevirt/boot-inspect" \
  -H 'Content-Type: application/json' \
  -d '{"namespace":"default","vm":"my-vm","mode":"boot-inspect","source":"zeus-os"}'
```

See [docs/features/kubevirt-integration.md](../docs/features/kubevirt-integration.md) for Zeus OS Guest Intelligence integration.

## KubeVirt

KubeVirt + CDI are **cluster prerequisites**. The Helm chart generates VM/DataVolume YAML; it does not install KubeVirt.

See [examples/kubevirt/migrated-vm.yaml](examples/kubevirt/migrated-vm.yaml).

## Legacy worker DaemonSet

The file-based worker in [`k8s/`](../k8s/) remains available for development. Production deployments should use the Helm chart with Redis transport.
