# GuestKit — Feature Guide

> **Offline VM intelligence and migration assurance.**

GuestKit is a pure-Rust control plane that reads a virtual machine's disk image offline, builds a normalized evidence snapshot, and answers the two questions that decide every migration: will it boot, and what must change before cutover. It ships as a scriptable CLI (guestkit), a carbon-themed TUI (guestctl), Python bindings, and a self-hosted web platform with KubeVirt integration - all sharing one engine, with no libguestfs appliance to install.

**70+** CLI subcommands · **6** disk formats read · **0** libguestfs appliances needed · **8** migration targets scored · **0-100** boot assurance score · **5** inspection profiles

This is the customer-facing feature reference. A print-ready PDF of the same content sits alongside this file. Generated from the product's actual capabilities.

## Contents

1. [Offline Disk Inspection](#1-offline-disk-inspection)
2. [Migration Assurance](#2-migration-assurance)
3. [Fix Plans & Repair](#3-fix-plans-repair)
4. [Security & Threat Analysis](#4-security-threat-analysis)
5. [Inventory & Reporting](#5-inventory-reporting)
6. [Planning & Optimization](#6-planning-optimization)
7. [Interactive Workspaces](#7-interactive-workspaces)
8. [Guest Agent & Live Control](#8-guest-agent-live-control)
9. [KubeVirt & Platform](#9-kubevirt-platform)
10. [Deployment & Editions](#10-deployment-editions)

## 1. Offline Disk Inspection

_Read the full guest OS from a cold disk image - no boot, no agent, no appliance._

- **One-command inspect** — guestkit inspect surfaces OS, distro, version, hostname, architecture, and init system from any supported disk image in a single pass. — _Know exactly what a VM is before you touch it._
- **Six disk formats, auto-detected** — Reads QCOW2, VMDK, VHD, VHDX, VDI, and RAW/IMG images, choosing loop devices or qemu-nbd automatically. — _Point it at whatever your hypervisor exported - it just opens._
- **Guest OS detection** — Fingerprints Linux distributions (Fedora, Ubuntu, Debian, RHEL, CentOS, SUSE and more) plus Windows from on-disk signals. — _Accurate identity without a running kernel._
- **Deep system enumeration** — Extracts packages, kernels, users, SSH config, services, timers, network, DNS, LVM, fstab, runtimes, containers, certificates, and cloud-init state. — _A complete inventory of the machine from bytes on disk._
- **Windows signal parsing** — Reads SAM/SECURITY registry hives to detect BitLocker, domain join, RDP, and driver gaps for Windows guests. — _See the Windows-specific blockers Linux tools miss._
- **Pure-Rust engine, no libguestfs** — Partition tables, filesystem signatures, and evidence schema are parsed in Rust; only host NBD/loop is used for mount. — _No guestfish appliance, no fragile daemon - fewer moving parts._

> Inspection is always read-only: your source image is never modified.

## 2. Migration Assurance

_Score boot readiness and generate hypervisor-aware fix plans before you cut over._

| Target | Boot analysis | Migration rules applied |
|---|---|---|
| kvm / proxmox / qemu | Proxmox/KVM | VirtIO, virtio-scsi/net, VMware Tools to qemu-ga |
| aws / azure / gcp / cloud | Cloud | cloud-init datasource, BYOL licensing |
| hyperv | Hyper-V | Hyper-V-specific boot checks |

- **Boot assurance score** — guestkit doctor predicts first-boot success on a target hypervisor with a 0-100 score, ranked blockers, and warnings. — _Answer 'will it boot?' before the weekend, not during it._
- **Root-cause --explain** — An inference engine traces each blocker back through a causal chain so you see why a VM would fail to boot. — _Fix the cause, not the symptom._
- **Migration score + checklist** — guestkit migrate-plan applies target-specific rules for VirtIO drivers, cloud-init, VMware Tools removal, BitLocker, and SELinux relabel across eight targets. — _A tailored cutover checklist per destination platform._
- **CI boot gate** — --fail-below sets an exit-code threshold so pipelines block any image that scores under your bar, JSON still emitted. — _Golden images that regress never reach production._
- **Policy-as-code** — guestkit policy check evaluates an expression DSL over evidence fields (e.g. bootability.score >= 80) or built-in CIS benchmarks. — _Codify sign-off criteria your whole team can trust._
- **Forensic diff** — guestkit forensic-diff compares two snapshots for config drift, suspicious persistence, and ransomware indicators. — _Prove what changed between golden and drifted._

## 3. Fix Plans & Repair

_Turn findings into reviewable, reversible, executable remediation - not blind edits._

- **Reviewable fix plans** — Findings become a structured plan of operations (file edits, package installs, service ops, SELinux, registry edits) that you preview before anything runs. — _See every change before it happens._
- **Transactional boot repair** — guestkit repair --fix boot converts doctor blockers into a plan, applies it with backups, then re-scores to show the delta. — _Fix boot blockers offline and prove the score improved._
- **Export to bash & Ansible** — Plans export as executable shell scripts, Ansible playbooks, JSON, or YAML for change control and runbooks. — _Hand ops a runbook your CAB can approve._
- **Backup & rollback** — guestkit plan apply creates timestamped backups; plan rollback restores prior state, with dependency ordering and dry-run. — _Every change has an undo button._
- **Automated hardening** — guestkit harden generates security-profile fixes for SSH, firewall, SELinux/AppArmor, and account posture. — _Ship hardened images without hand-editing configs._
- **Offline agent injection** — repair --inject-agent writes a guest agent binary into the disk during migration prep, no boot required. — _The VM comes up already instrumented._

> Security teams generate plans; ops teams apply them - full separation of duties, in version control.

## 4. Security & Threat Analysis

_Audit posture, hunt for compromise, and prove compliance - all from the offline disk._

- **Security profile audit** — guestkit inspect --profile security scores SSH exposure, UID-0 users, firewall, SELinux/AppArmor, and kernel into a risk level. — _A ranked risk verdict per VM in seconds._
- **Secret & credential scan** — guestkit secrets sweeps the disk for exposed credentials and keys. — _Catch leaked secrets before an image ships._
- **Malware & rootkit detection** — guestkit malware scans for rootkits and known-bad artifacts offline, where in-guest malware can't hide from the scanner. — _Inspect a suspect image without executing it._
- **CVE & patch analysis** — guestkit cve maps installed packages to known vulnerabilities and missing security patches. — _See the VM's exposure without a live agent._
- **Compliance checking** — guestkit compliance and audit evaluate images against security standards with detailed reporting. — _Turn every VM into an audit artifact._
- **Threat hunting & IOC** — guestkit threat-intel, hunt, and anomaly correlate indicators, detect anomalies, and surface suspicious persistence offline. — _Forensic triage on a dead disk, safely._
- **Forensic timeline & reconstruction** — guestkit timeline and reconstruct build an incident timeline from multiple on-disk sources and visualize the attack path. — _Rebuild what happened without booting the evidence._

## 5. Inventory & Reporting

_Produce SBOMs, license reports, and shareable documents from any image._

- **SBOM generation** — guestkit sbom emits a software bill of materials in SPDX or CycloneDX from the guest package set. — _Supply-chain inventory for every VM you run._
- **License compliance** — guestkit licenses inventories package licenses across the disk for compliance review. — _Know your license exposure before an audit asks._
- **Self-contained HTML reports** — --export html builds an interactive, collapsible, print-friendly report with all CSS and JS embedded. — _Email a single file to any stakeholder._
- **Git-friendly Markdown** — --export markdown produces version-controllable inventory documents for VM-configuration history. — _Track infrastructure drift in your docs repo._
- **Machine-readable output** — Most commands accept -o json or -o yaml for automation, monitoring, and jq/yq pipelines. — _Wire GuestKit straight into your tooling._
- **Fleet posture analysis** — guestkit fleet analyze scans a directory of images, clusters identical OS fingerprints, and flags snowflakes and low-score blockers. — _See fleet-wide drift at a glance._

> --output (json/yaml/text) and --export (html/markdown) are mutually exclusive - run twice for both. PDF is produced via external tools (wkhtmltopdf, headless browser).

## 6. Planning & Optimization

_Reverse-engineer infrastructure-as-code, model cloud cost, and map dependencies from a disk._

- **Infrastructure-as-code blueprints** — guestkit blueprint generates Terraform, Ansible, Kubernetes, or Docker Compose definitions from what it finds on the image. — _Recreate a legacy VM as code you can redeploy._
- **Cloud cost analysis** — guestkit cost profiles the workload and estimates run cost plus savings opportunities across AWS, Azure, and GCP. — _Price the migration before you commit to a cloud._
- **Dependency graph** — guestkit dependencies builds a package dependency graph with conflict, circular-dependency, and impact analysis. — _Understand blast radius before you change anything._
- **Disk format conversion** — guestkit convert transcodes images between the six supported formats using qemu-img. — _Reformat once, migrate anywhere._
- **Smart recommendations** — guestkit recommend and predict surface tuning and remediation guidance grounded in the evidence snapshot. — _Actionable next steps, not just raw data._
- **Performance profile** — --profile performance flags swappiness, I/O scheduler, mount options, and network tuning opportunities. — _Baseline and tune before cutover._

## 7. Interactive Workspaces

_A carbon TUI, a file explorer, and a shell for hands-on offline investigation._

- **Carbon-themed TUI** — guestctl tui opens a k9s-style dashboard with grouped views, vim keys, a command palette, and glass/transparency themes. — _Explore a VM visually without leaving the terminal._
- **Assurance view parity** — The TUI Assurance tab runs doctor, cycles targets (kvm/proxmox/aws), previews fix plans, and exports YAML - reusing the CLI engine on one mount. — _Full assurance workflow, keyboard-driven._
- **Interactive file explorer** — guestkit explore browses partitions and files in place with view, info, filter, sort, and hidden-file toggles. — _Grep-free spelunking through a cold disk._
- **Guest shell & REPL** — guestkit shell and interactive give ls/cat/grep/find over the mounted image plus a scriptable session. — _Familiar Unix muscle memory on any VM._
- **Fleet & compare modes** — --fleet browses a directory of images with a sidebar; --compare diffs two VMs side by side in the dashboard. — _Spot the odd VM out across a set._
- **Global search & jump** — Cross-view search finds packages, boot blockers, and migration items; a grouped jump menu navigates every view. — _Find any signal without knowing which tab holds it._
- **AI copilot Q&A** — guestkit ai answers natural-language questions grounded in the evidence snapshot, with pluggable LLM backends (OpenAI, Anthropic, xAI, or local Ollama). — _Ask a VM what's wrong and get an evidence-backed answer._

## 8. Guest Agent & Live Control

_Run inside the guest - or reach it host-mediated - even when there's no guest network._

- **In-guest agent** — guestkit agent runs like qemu-guest-agent over virtio-serial, reusing the same evidence and fix-plan schema as offline mode. — _One model for cold-disk and live guests._
- **Transport ladder** — The Guest Control Fabric auto-selects the best path per VM - virtio-serial, QGA exec, QGA builtin, push cache, offline disk, or console. — _Guest control that never depends on guest networking._
- **Snapshot quiesce** — Freeze and thaw guest filesystems (fsfreeze) for application-consistent snapshots, plus soft reboot and graceful shutdown. — _Clean snapshots without crash-consistency risk._
- **Live remediation with approval** — Restart failed units, collect support bundles, and run fix plans - policy-gated with JIT approval workflows. — _Safe, audited guest actions at fleet scale._
- **mTLS & signed updates** — Agents bootstrap client certs, push heartbeats over mTLS, and self-update from Ed25519-signed, SHA256-verified bundles. — _A hardened, tamper-evident guest agent._
- **Deep Linux health** — Component scores for boot, systemd, network, DNS, storage, and security via systemd D-Bus, journald, /proc, and PSI pressure. — _Root-cause the failed unit from journal correlation._

> Deep guest intelligence targets Linux; Windows uses virtio-win and a scheduled-task updater today, with a native agent MSI scaffolded.

## 9. KubeVirt & Platform

_Boot-inspect stopped VMs in-cluster and drive it all from a self-hosted web console._

- **Offline boot-inspect for stopped VMs** — zyvor-api resolves a stopped VM's root PVC and runs guestkit boot-inspect, returning fstab validity, bootloader, and cloud-init state. — _Assurance for halted VMs without booting them._
- **Zeus VM Tools** — A Kubernetes-native guest agent with cloud-init, QGA, ISO, and airgap install paths plus VMToolsPolicy auto-install/upgrade reconciliation. — _The VMware Tools equivalent for KubeVirt._
- **Web console** — Self-hosted zyvor-ui + zyvor-api + guestkit-worker ship as public GHCR images and a Helm chart, backed by a Redis job queue. — _A team-facing UI over the same engine._
- **Python bindings** — hypersdk-guestkit on PyPI exposes a libguestfs-style Guestfs API (100+ methods) for programmatic inspection. — _Automate disk inspection from Python._
- **hyper2kvm pipeline** — Pairs with hyper2kvm for VMware-to-KVM conversion, sitting in the wider HyperSDK to GuestKit to v9s to PacketWolf flow. — _One assurance gate inside a full migration pipeline._
- **Pluggable auth** — The web stack supports JWT, local login, and OIDC/SAML hooks with JWKS-verified ID tokens. — _Wire the console into your existing identity._

> In-cluster boot-inspect needs a privileged pod or node disk access plus get/list RBAC on VMs, VMIs, PVCs, and PVs. A running VM returns VM-spec heuristics - offline disk access is for stopped VMs.

## 10. Deployment & Editions

_Install in one command; run the full open-source stack; scale with Enterprise support._

- **cargo install** — cargo install guestkit installs both the guestkit CLI and guestctl TUI binaries. — _From zero to inspecting in one line._
- **Run from GHCR** — Prebuilt public images (zyvor-ui, zyvor-api, guestkit-worker) come up via docker compose with no docker login. — _Stand up the whole console in minutes._
- **Helm & remote deploy** — A Helm chart for clusters plus scripted remote deploy for Docker hosts. — _Ship it where your fleet already lives._
- **Full open-source stack** — CLI, TUI, Python bindings, assurance APIs, web console, and KubeVirt hooks are all Apache-2.0 in the repo - Enterprise adds support, not features. — _Nothing core is withheld from the open source._
- **Enterprise programs** — SLA, air-gapped deployment packages, guided playbooks, and fleet automation for 100+ VM and regulated migrations. — _Backed help for VMware-exit programs at scale._

> The default eval compose stack runs without authentication - do not expose it beyond localhost. Use the production template and turn auth on before any network exposure.

## Getting started

1. **Install** — Run cargo install guestkit to get the guestkit CLI and guestctl TUI, or pull the web stack from ghcr.io/hypersdk.
2. **Score boot readiness** — guestkit doctor vm.qcow2 --target proxmox --explain returns a 0-100 boot assurance score with ranked blockers and root-cause chains.
3. **Export a fix plan** — guestkit migrate-plan vm.vmdk --target proxmox --export plan.yaml writes an executable, reviewable migration fix plan.
4. **Explore interactively** — guestctl tui vm.qcow2 opens the carbon TUI with the Assurance workspace and fix-plan preview.
5. **Gate your pipeline** — guestkit doctor img.qcow2 --target proxmox -o json --fail-below 80 fails CI on any image that regresses below your bar.

> **Good to know:** GuestKit runs on Linux hosts and needs host tooling (losetup, qemu-nbd, and qemu-img for conversion); mount and in-cluster boot-inspect operations require root or a privileged pod. Offline disk analysis targets stopped VMs - running VMs return VM-spec heuristics or need the live guest agent. Deep guest intelligence is Linux-first; the native Windows agent MSI is scaffolded, and Windows relies on virtio-win today. LLM-assisted features require building with --features ai; deterministic intelligence works without it. Reporting notes: --output and --export are mutually exclusive, HTML reports truncate to 100 packages, and PDF is produced via external tools.

---
_GuestKit is developed by ZyvorAI Labs. Contact **info@zyvor.dev** · Proprietary & Confidential._
