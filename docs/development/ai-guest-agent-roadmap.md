# AI Guest Agent Roadmap

Evolution of GuestKit's optional AI layer into a **Guest Intelligence Agent** — an offline, evidence-grounded co-pilot for migration, security, and fleet analysis. Core inspection stays deterministic and auditable; AI is additive (`--features ai`).

## Philosophy

- **EvidenceSnapshot is the source of truth** — collectors populate typed structs; AI tools read the snapshot, not raw Guestfs.
- **Graceful degradation** — malformed unit files, missing hives, or partial mounts never abort collection.
- **Grounded output** — every AI finding should cite file paths, unit names, or registry keys from evidence.
- **Air-gap friendly** — OpenAI today; xAI/Anthropic/local LLM in later phases.

## Phase 0 — Richer evidence (shipped in v0.3.7)

**Goal:** Expand `EvidenceSnapshot` schema v2 with deeper systemd and Windows data.

| Component | Status |
|-----------|--------|
| `SystemdInfo`, `SystemdUnit`, problem hints | Done |
| `collect_systemd_guest` / `collect_systemd_live` | Done |
| Windows services, apps, event log summary | Done |
| `SystemdStaticCheck` in doctor | Done |
| TUI "Systemd Deep Dive" view | Planned |
| Windows persistence (Run keys, tasks) | Stub |

### Schema highlights

```text
EvidenceSnapshot (schema_version = 2)
├── systemd: Option<SystemdInfo>
│   ├── units[] — parsed .service/.timer/.socket/…
│   └── problem_hints[] — static boot/migration hints
└── windows: Option<WindowsEvidence>
    ├── services[] — start type, auto-start flags
    ├── installed_apps[] — sample from Uninstall keys
    └── event_logs — .evtx count and total size
```

## Phase 1 — Semantic analysis

- Dependency graph from unit `After`/`Before`/`Requires`/`Wants`
- Sandboxing score per service (`Protect*`, `Private*`, `NoNewPrivileges`)
- Windows service risk flags (LocalSystem + broad deps, disabled critical services)
- Improved AI context injection from new fields
- TUI views: Timers & Sockets, Failed/Problem Units, Windows Services

## Phase 2 — Agentic loop

- Tool registry over snapshot: `list_systemd_units`, `get_unit_details`, `get_boot_blockers`, etc.
- Multi-step reasoning in REPL `ai`, `doctor --explain`, `migrate-plan --ai`
- xAI (Grok) provider; grounded citations with severity/confidence
- TUI "AI Insights" panel

## Phase 3 — Local AI & what-if

- Ollama / llama.cpp integration (`--features local-ai`)
- What-if simulator (disable unit X → projected boot score)
- AI narrative sections in HTML/PDF reports
- Proactive recommendations engine (Critical / Security / Migration / Performance)
- Fleet semantic drift explanations

## Phase 4 — Platform integration

- Machina dashboard consuming evidence + AI summaries
- Policy DSL extensions with AI-assisted rules
- Advanced security profiles (CIS-style) using sandboxing and Windows service data
- Full `.evtx` parsing for forensic profiles

## Module layout (target)

```text
src/evidence/collectors/   — offline/live collection (Phase 0)
src/ai/ or src/guest_agent/
  agent.rs                 — tool loop + registry (Phase 2)
  prompts.rs               — versioned system prompts
  providers.rs             — OpenAI / xAI / Anthropic / local
```

## Related docs

- [roadmap.md](roadmap.md) — product-wide roadmap
- [../features/guest-agent.md](../features/guest-agent.md) — live in-guest agent (virtio-serial RPC)
- [CHANGELOG.md](CHANGELOG.md)
