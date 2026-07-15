# Open source vs Enterprise (Zyvor)

This repository ships a **full open-source stack** — CLI, TUI, Python bindings, and a self-hosted web platform (GHCR images, Helm chart). Enterprise is about **support, scale programs, and hardened deployments**, not withholding core features from the repo.

| | Open source (this repo) | Enterprise ([zyvor.dev](https://zyvor.dev/?utm_source=github&utm_medium=guestkit)) |
|---|------------------------|-------------------------------------------------------------------------------------|
| **Support** | GitHub Issues & Discussions | SLA, [sales@zyvor.dev](mailto:sales@zyvor.dev), migration workshops, professional services |
| **Typical use** | Lab, CI gates, single-VM / small-fleet assurance | VMware exit programs, 100+ VM migrations, regulated / air-gapped rollouts |
| **CLI / TUI / Python** | ✅ `guestkit`, `guestctl`, PyPI bindings | Same codebase + priority fixes |
| **Assurance** | ✅ `doctor`, `migrate-plan`, `fleet`, `policy`, repair | Same + guided playbooks |
| **Web console** | ✅ `zyvor-ui` + `zyvor-api` + `guestkit-worker` (self-hosted) | Hardened reference architectures, multi-tenant ops support |
| **Auth** | ✅ JWT, local login, OIDC/SAML hooks (configure + secure yourself) | SSO hardening reviews, air-gap identity, audited deployments |
| **KubeVirt / Zeus** | ✅ API routes, guest agent, VM tools hooks | Fleet-scale Zeus OS programs, PacketWolf correlation at scale |
| **Platform pipeline** | Use alongside [hyper2kvm](https://github.com/hypersdk/hyper2kvm) | Full managed pipeline: HyperSDK → hyper2kvm → GuestKit → v9s → PacketWolf |

**What Enterprise adds (not “missing from OSS”):**

- Contractual SLA and escalation
- Air-gapped / disconnected deployment packages
- Carbon-aware scheduling and fleet automation at program scale
- Partner / MSP programs and architecture reviews

**Approach Zyvor for production programs:** [zyvor.dev/contact](https://zyvor.dev/contact?utm_source=github&utm_medium=guestkit) · [sales@zyvor.dev](mailto:sales@zyvor.dev)

See also: [zyvor-enterprise.md](zyvor-enterprise.md) · [Production checklist](guides/DOCKER.md#production-checklist)
