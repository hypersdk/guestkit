# 01 — Passport CI gate

**Goal:** No disk converts until `guestkit passport verify --fail-below N` succeeds. Attach the Passport JSON to the change ticket.

---

## 0. Preconditions

| Check | Why |
|-------|-----|
| Linux worker with `qemu-img`, `losetup`, `qemu-nbd` | Mount/repair paths |
| Disk image reachable (NFS, object store pull, or artifact) | Gate input |
| Target hypervisor known (`kvm`, `proxmox`, `qemu`, cloud, …) | Doctor/plan are target-aware |
| Floor agreed (`80` typical) | Change-control number |

Local smoke:

```bash
guestkit doctor vm.qcow2 --target kvm --explain
guestkit passport emit vm.qcow2 --target kvm -o passport.json --bundle
guestkit passport verify passport.json --fail-below 80
echo $?   # must be 0
```

---

## 1. GitHub Actions — `hypersdk/guestkit` composite Action

The real implementation of this runbook lives at [`action.yml`](../../action.yml)
in this repo, and is dogfooded on every change by
[`.github/workflows/passport-gate-demo.yml`](../../.github/workflows/passport-gate-demo.yml) —
open that workflow for a working, copy-pasteable example.

```yaml
jobs:
  assure:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: hypersdk/guestkit@v1
        id: gate
        with:
          disk: vm.qcow2
          target: kvm
          fail-below: '80'
      - run: echo "score=${{ steps.gate.outputs.score }}"

  convert:
    needs: [assure]
    runs-on: ubuntu-latest
    steps:
      - run: echo "hyper2kvm convert …"   # only runs if assure passed
```

| Input | Default | Notes |
|-------|---------|-------|
| `disk` | *(required)* | Path to the disk image |
| `target` | `kvm` | `kvm`, `proxmox`, `qemu`, `aws`, `azure`, `gcp`, `cloud`, `hyperv` |
| `fail-below` | `80` | Gate floor |
| `version` | `latest` | Pin a specific `guestkit` release (e.g. `0.3.21`) |
| `sign-key` / `issuer` / `require-signature` / `trust-keys` | — | Signed-Passport path, see §2 |

Outputs: `score`, `passport-path`, `passed`. The action always uploads
`doctor.txt` / `plan.yaml` / `passport.json` as a workflow artifact (even on
failure) — attach that artifact to the change ticket per §4.

Installs a prebuilt `guestkit-<version>-linux-amd64.tar.gz` from GitHub
Releases (checksum-verified) — no build step. Not on GitHub Actions, or need
another runner/orchestrator? Use the shell recipe below instead.

---

## 2. Minimal CI job (container or bare, non-GitHub-Actions)

```bash
#!/usr/bin/env bash
set -euo pipefail
IMAGE="${1:?disk image path}"
TARGET="${TARGET:-kvm}"
FLOOR="${FLOOR:-80}"

guestkit doctor "$IMAGE" --target "$TARGET" --explain | tee doctor.txt
guestkit migrate-plan "$IMAGE" --target "$TARGET" --export plan.yaml
# Optional: apply offline repairs here, then re-doctor

guestkit passport emit "$IMAGE" --target "$TARGET" -o passport.json --bundle
guestkit passport verify passport.json --fail-below "$FLOOR"

# Artifact for change ticket / downstream convert job
cp passport.json doctor.txt plan.yaml "$CI_ARTIFACT_DIR"/
```

Gate convert:

```yaml
# pseudocode — your orchestrator
jobs:
  assure:
    steps: [doctor, plan, passport emit, passport verify]
  convert:
    needs: [assure]
    when: assure.success
    steps: [hyper2kvm convert …]
```

---

## 3. Signed Passports (regulated)

Build with agent/signing features available; then:

```bash
guestkit passport keygen --seed ./ed25519.seed --public ./ed25519.pub
# store seed in vault; publish .pub to trust store

guestkit passport emit vm.qcow2 --target kvm -o p.json \
  --sign-key "$PASSPORT_SEED" --issuer "ci/mig-prod" --expires-hours 72

guestkit passport verify p.json --fail-below 80 --require-signature \
  --trust-keys ./trusted-pubs.txt --max-age-hours 168
```

| Control | Setting |
|---------|---------|
| Signature required | `--require-signature` |
| Trust anchors | `--trust-keys` |
| Freshness | `--max-age-hours` / emit `--expires-hours` |

Rotate seeds like any code-signing key. Issuer string should identify pipeline + env.

---

## 4. What verify fails on (expected)

- Score &lt; `--fail-below`  
- BitLocker / undecryptable volume (hard block offline)  
- Missing/invalid signature when required  
- Passport older than `--max-age-hours`  

These are **stops**, not warnings. Fix offline (runbook 02/03/06) and re-emit.

---

## 5. Handoff to hyper2kvm

Passport does **not** convert. After verify:

1. Store `passport.json` (+ bundle FixPlan) next to the disk artifact.  
2. Start convert job with the **same** disk revision that was scored.  
3. Post-convert: optional ZyAIQAAgent smoke / GuestKit live doctor via agent.

See also: [Passport → hyper2kvm blog](https://zyvor.dev/blog/guestkit-hyper2kvm-passport-handoff).

---

## 6. Ownership

| Item | Owner |
|------|--------|
| Workflow + floor | Migration DevOps |
| Signing keys | Security + DevOps |
| Repair when red | Migration eng / OS specialists |
| Convert job | Platform / hyper2kvm owners |
