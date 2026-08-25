# Open source vs Enterprise (Zyvor)

**GuestKit (this repo)** is the full Apache-2.0 **offline disk engine** — CLI, TUI, Python, Passport emit, CI Action, and a self-hosted web/worker stack. It answers *will this disk boot?* and *what must change before cutover?* without powering on the guest.

**GuestKit Enterprise** is Zyvor’s commercial **migration control plane**: Command Center, portfolio, Image Vault (with free UX parity), Migration Factory, Passport Authority, OIDC/RBAC/audit, reports, KubeVirt inventory, and worker fabric. It calls **this** engine through the same `guestkit doctor` / job boundary — it does **not** fork the binary or withhold OSS features.

| | Open source (this repo) | GuestKit Enterprise |
| --- | --- | --- |
| **Buy when…** | You need proven offline assurance & CI gates | You run a **migration program** (waves, SSO, audit, shared Command Center) |
| **Support** | GitHub Issues & Discussions | SLA · [sales@zyvor.dev](mailto:sales@zyvor.dev) · workshops · professional services |
| **Typical scale** | Lab, CI, single-VM / small fleet | hypervisor exit, 50–10,000+ VMs, multi-site ops |
| **CLI / TUI / Python** | ✅ Full | ✅ Same engine + priority fixes |
| **doctor / migrate-plan / repair / Passport emit** | ✅ | ✅ Same evidence via workers / adapter |
| **Free web (`zyvor-ui`) + workers** | ✅ Self-hosted GHCR / Helm | Enterprise **Command Center** (includes vault dock + program screens) |
| **Program ops** | Scripts / fleet helpers | ✅ Portfolio · waves · Passport Authority · Copilot · reports |
| **Auth** | Configure JWT / OIDC yourself | ✅ Reference Keycloak OIDC · RBAC · audit stream |
| **Deploy** | Containers / Helm | ✅ Hardened reference + k3s / air-gap packs |
| **Platform pipeline** | Pair with [hyper2kvm](https://github.com/hypersdk/hyper2kvm) | Full Zyvor pipeline: HyperSDK → hyper2kvm → GuestKit → Zeus OS → PacketWolf |

---

## Why teams upgrade to Enterprise

1. **Spreadsheets stop working** — waves, owners, and Passport status need a shared system of record.  
2. **Security review blocks DIY dashboards** — SSO, RBAC, and audit must be productized.  
3. **Executives need one readiness number** — Command Center KPIs and exportable reports.  
4. **Cutover risk is commercial** — SLA and migration workshops put Zyvor on the critical path with you.  
5. **Engineers keep the tools they trust** — Image Vault still runs inspect / doctor / repair / launch YAML / agent; Enterprise adds program screens around them.

Buyer brief: why upgrade — [zyvor.dev/docs/guestkit#community-vs-enterprise](https://zyvor.dev/docs/guestkit#community-vs-enterprise) · Product: [zyvor.dev/guestkit](https://zyvor.dev/guestkit?utm_source=github&utm_medium=guestkit)

---

## Capability depth

### Included in open source (you already have this)

- Pure-Rust offline inspection (no libguestfs appliance)  
- Boot assurance `doctor` with weighted blockers and `--explain`  
- Migration plans, repair, harden, fleet analyze / wave-plan helpers  
- Cutover Passport emit for CI (`hypersdk/guestkit@v1`)  
- `guestctl` TUI, Python bindings, in-guest agent  
- Self-hosted `zyvor-ui` / API / worker images  

### Added by GuestKit Enterprise (what you buy)

| Area | Enterprise value |
| --- | --- |
| **Command Center** | Estate readiness, blockers, velocity for sponsors |
| **Image Vault** | Free-dock parity under Enterprise login (sources, batch doctor, evidence pane, launch YAML, online agent) |
| **Migration Factory** | Named waves, risk, windows, owners |
| **Passport Authority** | In-product issue / certify + JSON evidence download |
| **Reports** | Control-plane JSON / CSV export |
| **KubeVirt screen** | Target cluster inventory for operators |
| **Identity** | Keycloak OIDC reference, role gates, audit stream |
| **Worker fabric** | Sites / workers posture; control plane stays out of disk I/O |
| **Services** | SLA, air-gapped packages, hypervisor exit playbooks, partner/MSP programs |

---

## When to stay on open source

- Proving `doctor` quality on a handful of disks  
- CI / golden-image gates only  
- Lab and personal tooling  

## When to contact Zyvor

- Hypervisor exit with program governance  
- Regulated environments needing SSO + audit  
- Multi-site cutovers with shared deadlines  
- You want contractual accountability, not only GitHub Issues  

**→ [Book a demo](https://zyvor.dev/contact?utm_source=github&utm_medium=guestkit&intent=demo)** · **[sales@zyvor.dev](mailto:sales@zyvor.dev)** · **[zyvor.dev/pricing](https://zyvor.dev/pricing?utm_source=github&utm_medium=guestkit)**

See also: [zyvor-enterprise.md](zyvor-enterprise.md) · [Production checklist](guides/DOCKER.md#production-checklist)
