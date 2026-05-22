# Publishing GuestKit

## PyPI (`hypersdk-guestkit`)

The Python distribution is published as **`hypersdk-guestkit`** (PyPI project owned by the `hypersdk` account).

```bash
pip install hypersdk-guestkit
```

CI workflow: **Build and Publish Python Wheels** — requires GitHub secret `PYPI_API_TOKEN`.

Manual publish after building wheels locally:

```bash
maturin build --release --features python-bindings --out dist
twine upload dist/*
```

## crates.io (`guestkit`)

Workflow: **Release** → job `publish-crate`.

Add repository secret:

| Secret | Purpose |
|--------|---------|
| `CARGO_TOKEN` | crates.io API token with publish scope |

Create token at https://crates.io/settings/tokens

Re-run the Release workflow on tag `v*` after adding the secret.

## GitHub release assets

Tag push `v*` builds Linux x86_64 binaries and creates a GitHub Release from `docs/development/CHANGELOG.md`.
