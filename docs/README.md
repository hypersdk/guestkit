# guestkit Documentation

Offline VM intelligence and migration assurance

## Start Here

| Goal | Document |
|------|----------|
| Getting started | [getting-started.md](user-guides/getting-started.md) |
| Run from GHCR (Docker/Helm) | [guides/DOCKER.md](guides/DOCKER.md#published-images-ghcr) |
| CLI guide | [cli-guide.md](user-guides/cli-guide.md) |
| Migration assurance | [migration-assurance.md](features/migration-assurance.md) |
| Architecture | [overview.md](architecture/overview.md) |
| Full index | [INDEX.md](INDEX.md) |
| **User journeys & acceptance criteria** | [User Stories](USER_STORIES.md) |

## User Stories

Persona-based journeys with acceptance criteria: **[USER_STORIES.md](USER_STORIES.md)**

| Persona | Focus |
|---------|-------|
| Alex (Migration Engineer) | Pre-flight VM inspection before cutover |
| Morgan (SRE) | Fleet drift analysis and forensic diff |
| Jordan (Platform Architect) | Boot probability scoring and fix plans |

## Ecosystem

Part of the [Zyvor / HyperSDK platform stack](https://zyvor.dev):

| Product | Role |
|---------|------|
| **hypercluster** | Kubernetes bootstrap |
| **machina** | Bare-metal hypervisor OS |
| **zeus-os (v9s)** | Cloud / KubeVirt control plane |
| **forge** | AI infrastructure on K8s |
| **hypersdk / hyper2kvm** | VM migration |
| **guestkit** | Offline VM assurance |
| **packetwolf** | Network intelligence |
| **Aether** | Runtime portability |
| **hermes** | Application layer for K8s |

See also: [../README.md](../README.md)
