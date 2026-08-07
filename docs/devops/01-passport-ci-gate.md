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

## 1. Minimal CI job (container or bare)

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

## 2. Signed Passports (regulated)

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

## 3. What verify fails on (expected)

- Score &lt; `--fail-below`  
- BitLocker / undecryptable volume (hard block offline)  
- Missing/invalid signature when required  
- Passport older than `--max-age-hours`  

These are **stops**, not warnings. Fix offline (runbook 02/03/06) and re-emit.

---

## 4. Handoff to hyper2kvm

Passport does **not** convert. After verify:

1. Store `passport.json` (+ bundle FixPlan) next to the disk artifact.  
2. Start convert job with the **same** disk revision that was scored.  
3. Post-convert: optional ZyAIQAAgent smoke / GuestKit live doctor via agent.

See also: [Passport → hyper2kvm blog](https://zyvor.dev/blog/guestkit-hyper2kvm-passport-handoff).

---

## 5. Ownership

| Item | Owner |
|------|--------|
| Workflow + floor | Migration DevOps |
| Signing keys | Security + DevOps |
| Repair when red | Migration eng / OS specialists |
| Convert job | Platform / hyper2kvm owners |
