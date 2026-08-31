# h2kvm integration

GuestKit provides **offline disk intelligence**; [h2kvm](https://github.com/zyvorai/h2kvm) provides **hypervisor-to-KVM conversion, deployment, and day-2 orchestration**. Together they form the inspect → plan → fix → convert → deploy pipeline.

## Recommended stack (v1.1.0+)

| Layer | Component | Role |
|-------|-----------|------|
| Assurance | GuestKit CLI or Python | Doctor, migrate-plan, migrate-repair |
| Conversion | h2kvm (`h2kvmctl`) | VMDK/OVA → qcow2, flatten, libvirt/KubeVirt deploy |
| Python binding | `hypersdk-guestkit` | Native `run_*` functions — **no subprocess wrapper required** |
| Day-2 | Zeus OS / Axiom | Post-cutover operations |

## Python integration (preferred)

Since **v1.1.0**, h2kvm delegates offline repair to GuestKit via PyO3 bindings:

```python
import guestkit

# Dry-run — see what would change
plan = guestkit.run_migrate_repair(
    "/var/lib/h2kvm/input/ubuntu-test.vmdk",
    target="kvm",
    apply=False,
    verbose=True,
)
print(plan["fix_plan"], plan["assessment_score"])

# Apply fstab / GRUB / initramfs fixes offline
result = guestkit.run_migrate_repair(
    "/var/lib/h2kvm/demo/ubuntu-test/ubuntu-test.qcow2",
    target="kvm",
    apply=True,
)
print(result["message"], result["applied"])
```

h2kvm wraps the same calls in `h2kvm.core.guestkit_client`:

```python
from h2kvm.core import guestkit_client
guestkit_client.migrate_repair(path, target="kvm", apply=True)
```

### Install on migration hosts

```bash
pip install 'hypersdk-guestkit>=1.1.0'
pip install 'h2kvm[full]'
```

If 1.1.0 is not on PyPI yet, build from source:

```bash
cd guestkit
pip install maturin
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin build \
  --release --features python-bindings --out dist
pip install dist/hypersdk_guestkit-*.whl
```

## CLI integration (CI gates, passports)

For CI pipelines and Passport artifacts, use the **GuestKit CLI** directly:

```bash
guestkit doctor disk.qcow2 --target kvm --explain
guestkit migrate-plan disk.qcow2 --target kvm
guestkit migrate-repair disk.qcow2 --target kvm --apply
guestkit passport emit disk.qcow2 --out passport.json
```

Then hand off to h2kvm for conversion:

```bash
h2kvmctl local --vmdk source.vmdk --to-output out.qcow2 \
  --backend guestkit --libvirt-import
```

## Deploy both projects to a lab host

```bash
# GuestKit CLI (Rust binary)
GUESTKIT_ZYVOR_ACCEPT=1 ./scripts/deploy-remote.sh 175.110.122.71 sus --quick --key

# h2kvm (Python + h2kweb + libvirt stack)
cd /path/to/h2kvm
./scripts/deploy-remote.sh 175.110.122.71 sus --keep-sources
```

See [DEPLOY-REMOTE.md](../guides/DEPLOY-REMOTE.md) (GuestKit) and [h2kvm deploy-remote](https://github.com/zyvorai/h2kvm/blob/main/docs/deployment/deploy-remote.md).

## Assurance API reference

| Python | Returns (dict keys) |
|--------|---------------------|
| `run_doctor(image, target="kvm")` | `bootability`, `target`, optional `root_cause`, `copilot` |
| `run_boot_inspect(image, target="kvm")` | `os_release`, `fstab_valid`, `bootloader`, `message` |
| `run_migrate_plan(image, target="kvm")` | `migration_score`, `bootability`, `fix_plan` |
| `run_repair_plan(image, dry_run=True)` | `before_score`, `after_score`, `fix_plan`, `applied` |
| `run_migrate_repair(image, apply=False)` | `dry_run`, `applied`, `assessment_score`, `fix_plan`, `notes` |

`bootability` includes `score`, `confidence`, `blockers[]`, `warnings[]`, `checks[]`.

## Legacy subprocess wrapper

The `integration/python/guestkit_wrapper.py` subprocess wrapper remains for older integrations. **New code should use `pip install hypersdk-guestkit` and import `guestkit` directly.**

## Validated lab workflow (Ubuntu 24.04)

1. Download osboxes.org VMDK (SourceForge 7z archive)
2. `demo-libvirt.sh ubuntu2404.vmdk ubuntu-test` on h2kvm host
3. GuestKit `run_migrate_repair` applies 4+ operations during offline fix
4. Output qcow2 → libvirt domain `ubuntu-test` (credentials: osboxes / osboxes.org)

## See also

- [migration-assurance.md](migration-assurance.md)
- [python-bindings.md](../user-guides/python-bindings.md)
- [h2kvm GUESTKIT.md](https://github.com/zyvorai/h2kvm/blob/main/docs/architecture/GUESTKIT.md)
