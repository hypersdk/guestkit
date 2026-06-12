# AI Guest Agent Roadmap

Evolution of GuestKit's optional AI layer into a **Guest Intelligence Agent** — an offline, evidence-grounded co-pilot for migration, security, and fleet analysis. Core inspection stays deterministic and auditable; AI is additive (`--features ai`).

## Philosophy

- **EvidenceSnapshot is the source of truth** — collectors populate typed structs; AI tools read the snapshot, not raw Guestfs.
- **Graceful degradation** — malformed unit files, missing hives, or partial mounts never abort collection.
- **Grounded output** — every AI finding should cite file paths, unit names, or registry keys from evidence.
- **Air-gap friendly** — OpenAI today; xAI/Anthropic/Ollama via env configuration.

## Phase 0 — Richer evidence

| Component | Status |
|-----------|--------|
| `SystemdInfo`, `SystemdUnit`, problem hints | Shipped |
| `collect_systemd_guest` / `collect_systemd_live` | Shipped |
| Windows services, apps, event log summary | Shipped |
| `SystemdStaticCheck` in doctor | Shipped |
| TUI **Systemd Deep Dive** view | Shipped |
| Windows persistence (Run keys, tasks) | Shipped |

## Phase 1 — Semantic analysis

| Component | Status |
|-----------|--------|
| Dependency graph from unit `After`/`Before`/`Requires`/`Wants` | Shipped (`src/ai/semantic.rs`) |
| Sandboxing score per service | Shipped |
| Windows service risk flags | Shipped |
| Improved AI context injection | Shipped (`build_intelligence`) |
| TUI timers/sockets/problems in Systemd Deep Dive | Shipped |

## Phase 2 — Agentic loop

| Component | Status |
|-----------|--------|
| Tool registry over snapshot | Shipped (`src/ai/tools.rs`) |
| Multi-step agent loop | Shipped (`src/ai/agent.rs`) |
| `doctor --explain --ai`, `migrate-plan --explain --ai` | Shipped |
| Providers: OpenAI, xAI, Anthropic, Ollama | Shipped (`src/ai/providers.rs`) |
| TUI **AI Insights** panel | Shipped |

## Phase 3 — Local AI & what-if

| Component | Status |
|-----------|--------|
| Ollama integration (`OLLAMA_HOST`, `--features local-ai`) | Shipped |
| What-if simulator (disable unit → boot score delta) | Shipped (`src/ai/whatif.rs`) |
| AI narrative sections for reports | Shipped (`src/ai/reports.rs`) |
| Proactive recommendations engine | Shipped (`src/ai/recommendations.rs`) |
| Fleet semantic drift explanations | Shipped (`src/ai/drift.rs`) |

## Phase 4 — Platform integration

| Component | Status |
|-----------|--------|
| Machina dashboard export type | Shipped (`src/ai/platform.rs`) |
| Policy DSL hints from CIS-lite profile | Shipped |
| CIS-style security profiles | Shipped (`src/ai/security_profiles.rs`) |
| Full `.evtx` parsing for forensic profiles | Shipped (`evtx` crate + `WindowsForensicProfile`) |

## Module layout

```text
src/ai/
  mod.rs              — public API
  semantic.rs         — Phase 1 analysis
  tools.rs            — Phase 2 snapshot tool registry (feature ai)
  agent.rs            — Phase 2 agent loop (feature ai)
  prompts.rs          — versioned system prompts
  providers.rs        — OpenAI / xAI / Anthropic / Ollama (feature ai)
  recommendations.rs  — Phase 3 proactive engine
  whatif.rs           — Phase 3 boot score simulator
  drift.rs            — Phase 3 fleet drift
  reports.rs          — Phase 3 report narratives
  security_profiles.rs— Phase 4 CIS-lite
  platform.rs         — Phase 4 Machina export
  intelligence.rs     — bundled output for doctor/TUI
src/evidence/collectors/
  systemd.rs, windows.rs — Phase 0 collectors
```

## CLI usage

```bash
# Deterministic intelligence (no LLM)
guestkit doctor disk.qcow2 --explain
guestkit migrate-plan disk.qcow2 --target kubevirt --explain

# LLM agent (requires --features ai + API key or Ollama)
cargo build --release --features ai
export OPENAI_API_KEY=...
guestkit doctor disk.qcow2 --explain --ai

# Local Ollama
export OLLAMA_HOST=http://127.0.0.1:11434
export GUESTKIT_AI_PROVIDER=ollama
guestkit migrate-plan disk.qcow2 --target kvm --ai
```

## Related docs

- [roadmap.md](roadmap.md) — product-wide roadmap
- [../features/guest-agent.md](../features/guest-agent.md) — live in-guest agent (virtio-serial RPC)
