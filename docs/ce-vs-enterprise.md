# Open source vs Enterprise (Zyvor)

This repository ships a **full open-source GuestKit engine** — CLI, TUI, Python bindings, and a self-hosted web/worker stack (GHCR images, Helm chart). That engine answers *will this disk boot?* and *what must change before cutover?* offline.

**GuestKit Enterprise** is a separate **migration control plane** (Command Center UX, portfolio, Migration Factory, Passport Authority, OIDC/RBAC/audit, site/worker fabric, k3s deploy). It calls this engine through the same `guestkit doctor` / job boundary — it does **not** fork or withhold OSS features from this repo.

| | Open source (this repo) | GuestKit Enterprise ([zyvor.dev/guestkit](https://zyvor.dev/guestkit?utm_source=github&utm_medium=guestkit)) |
|---|------------------------|-------------------------------------------------------------------------------------|
| **Support** | GitHub Issues & Discussions | SLA, [sales@zyvor.dev](mailto:sales@zyvor.dev), migration workshops, professional services |
| **Typical use** | Lab, CI gates, single-VM / small-fleet assurance | VMware exit programs, 100+ VM migrations, multi-site ops |
| **CLI / TUI / Python** | ✅ `guestkit`, `guestctl`, PyPI bindings | Same engine + priority fixes |
| **Assurance** | ✅ `doctor`, `migrate-plan`, `fleet`, `policy`, repair, Passport emit | Same evidence via workers / adapter |
| **Web / workers** | ✅ `zyvor-ui` + `zyvor-api` + `guestkit-worker` (self-hosted) | Enterprise **Command Center** control plane + hardened reference architectures |
| **Program ops** | Scripts / CI / fleet helpers | ✅ Portfolio, Migration Factory waves, Passport Authority UI, Copilot |
| **Auth** | ✅ JWT, local login, OIDC/SAML hooks (configure + secure yourself) | ✅ Reference Keycloak OIDC, RBAC role gates, audit stream |
| **KubeVirt / Zeus** | ✅ API routes, guest agent, VM tools hooks | Fleet-scale Zeus OS programs, PacketWolf correlation at scale |
| **Platform pipeline** | Use alongside [hyper2kvm](https://github.com/hypersdk/hyper2kvm) | Full managed pipeline: HyperSDK → hyper2kvm → GuestKit → v9s → PacketWolf |

**What Enterprise adds (not “missing from OSS”):**

- Estate Command Center and multi-site portfolio control plane
- Migration Factory + Passport Authority product workflows
- Contractual SLA and escalation
- Air-gapped / disconnected deployment packages
- Carbon-aware scheduling and fleet automation at program scale
- Partner / MSP programs and architecture reviews

**Approach Zyvor for production programs:** [zyvor.dev/contact](https://zyvor.dev/contact?utm_source=github&utm_medium=guestkit) · [sales@zyvor.dev](mailto:sales@zyvor.dev)

See also: [zyvor-enterprise.md](zyvor-enterprise.md) · [Production checklist](guides/DOCKER.md#production-checklist)
