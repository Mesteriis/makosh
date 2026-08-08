# Security and Dependency Hygiene

Commands:

- `make audit` - RustSec vulnerability scan via `cargo-audit`
- `make deny` - advisory/license/source/multi-version checks via `cargo-deny`
- `make security` - combined audit + deny
- `make udeps` - unused dependency scan via `cargo-udeps` on nightly

Files:

- `deny.toml`
- `Makefile`

Notes:

- `cargo-udeps` requires nightly Rust to execute.
- `cargo-deny` is broader than `cargo-audit`; it also checks sources, versions, and licenses.
- `make audit` intentionally passes `--ignore RUSTSEC-2023-0071`. The affected `rsa` crate is pulled into `Cargo.lock` through `sqlx-mysql`, while the active backend/`makosh-backend-testkit` SQLx graph uses PostgreSQL only and `cargo tree -i sqlx-mysql` prints no active path. The upstream advisory has no fixed version, so this is a documented lockfile-only exception rather than a hidden greenwash.

Current repository state as of `2026-06-23`:

- tooling is wired and executable locally;
- `make security` is green after updating `testcontainers`, updating `quinn-proto`, and documenting the inactive `sqlx-mysql` / `rsa` cargo-audit exception;
- `make audit` still reports allowed warnings for `lru` and `memmap2`;
- `cargo-deny` still reports duplicate-version warnings, but the deny gate exits successfully;
- `make udeps` is green after removing unused `mockall` and `testcontainers-modules` dependencies.
