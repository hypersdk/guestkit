# Admin Basics (GuestKit)

## Ports / access

| Port | Service |
|------|--------|
| **8088** | Optional zyvor-api |
| **8765** | Guest agent (when used) |
| CLI/TUI | `guestkit` / `guestctl` |

## Auth

SAML/OIDC on zyvor-api when enabled; local CLI needs disk/image access.

## Install sketch

Follow the product README and deploy/Helm docs in the repository. Verify health endpoints or CLI status before opening the UI.

## Related

- [Getting Started](getting-started.md)
