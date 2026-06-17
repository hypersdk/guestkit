# guestkit User Stories

**Product:** Offline VM intelligence and migration assurance

Cross-reference: [Documentation index](README.md) · [Main README](../README.md)

## Personas

| Persona | Name | Focus |
|---------|------|-------|
| Migration Engineer | Alex | Pre-flight VM inspection before cutover |
| SRE | Morgan | Fleet drift analysis and forensic diff |
| Platform Architect | Jordan | Boot probability scoring and fix plans |

---

### Story 1 — Score boot probability offline

**As Alex** (Migration Engineer), I want inspect qcow2/vmdk without powering on and get boot probability score, **so that** I deliver reliable outcomes.

| Criterion | Notes |
|-----------|-------|
| Core capability | guestkit doctor, --explain, blockers |

---

### Story 2 — Export migration fix plan

**As Alex** (Migration Engineer), I want generate hypervisor-aware fix plan yaml before cutover, **so that** I deliver reliable outcomes.

| Criterion | Notes |
|-----------|-------|
| Core capability | guestkit migrate-plan, --export |

---

### Story 3 — Explore disk in TUI

**As Morgan** (SRE), I want browse partitions, files, and assurance views in carbon tui, **so that** I deliver reliable outcomes.

| Criterion | Notes |
|-----------|-------|
| Core capability | guestctl tui, Assurance tab |

---

### Story 4 — Fleet analyze for drift

**As Jordan** (Platform Architect), I want compare fleet images for configuration drift, **so that** I deliver reliable outcomes.

| Criterion | Notes |
|-----------|-------|
| Core capability | fleet analyze, forensic-diff |

---

### Story 5 — CI gate on doctor score

**As Alex** (Migration Engineer), I want fail pipeline if boot probability below threshold, **so that** I deliver reliable outcomes.

| Criterion | Notes |
|-----------|-------|
| Core capability | JSON/YAML export, --json |

---

## Validation

Map each story to smoke tests, CI jobs, or manual lab steps before marking production-ready.
