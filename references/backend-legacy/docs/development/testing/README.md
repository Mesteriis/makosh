# Testing Infrastructure

Status: documentation package aligned to the current repository structure.

Макошь uses a split test stack:

- Rust backend execution runs through `cargo-nextest`.
- Backend integration and coverage runs go through the `crates/test-session` session harness so PostgreSQL and NATS testcontainers are reused and cleaned correctly. Backend-specific fixtures and explicit test composition remain in `crates/testkit` (`makosh-backend-testkit`).
- Local wrapper scripts and Makefile nextest targets force a visible progress mode via `--show-progress`.
- Post-run JUnit analysis prints a compact completed/passed/failed/flaky summary with an ASCII progress bar, so non-interactive Codex/CI output is not silent after a test set finishes.
- Frontend unit tests stay on Vitest.
- Architecture checks remain first-class and are part of the test taxonomy.
- Макошь Lab is the proposed system-level harness for tracing real provider signals through Communications, Review, Timeline, Search and UI/debug surfaces.

## Command map

- `make test-fast` - local fast loop: backend unit + architecture + snapshots + frontend unit tests
- `make test` - full local validation-oriented test run
- `make test-ci` - backend CI-oriented nextest run plus frontend unit tests
- `make test-unit`
- `make test-integration`
- `make test-e2e`
- `make test-architecture`
- `make test-snapshot`
- `make coverage`
- `make coverage-html`
- `make coverage-ci`
- `make mutants`
- `make audit`
- `make deny`
- `make security`
- `make udeps`
- `make watch-test`
- `make watch-unit`
- `make watch-integration`
- `make cache-stats`
- `make cache-reset`
- `make test-performance-report`
- `make testcontainers-clean`

## Testcontainers cleanup

`make backend-test`, `make test-integration` and related backend Make targets
run through `crates/test-session`'s `makosh-test-session` wrapper. The wrapper starts
session-scoped PostgreSQL/NATS containers, prints progress while long runs are
active, labels Макошь-owned containers and removes the session containers on
exit or shutdown signals.

For manual cleanup of leaked Макошь testcontainers:

```sh
make testcontainers-clean
```

The cleanup command is restricted to Макошь testkit labels and legacy
pgvector/NATS containers created by the repository testkit.

Targeted `cargo test` runs outside `makosh_test_session` now use containers
owned by the individual `TestContext`, so those containers are dropped when the
test context exits.

## Zulip live fixture

`backend/tests/zulip_live.rs` is an ignored, opt-in live contract test for the
Zulip reference provider. It starts a real Zulip Docker Compose stack through
`testcontainers`, provisions a root realm, owner, bot, human user and stream,
then exercises:

- Zulip event queue registration and message observation;
- stream and direct messages;
- file upload/download;
- reactions, edits and deletes;
- Макошь raw signal dispatch, Communications projection and Review task
  candidate creation from a real Zulip message event.

Run it explicitly:

```sh
MAKOSH_ZULIP_TESTCONTAINERS=1 \
MAKOSH_ZULIP_START_TIMEOUT_SECS=900 \
cargo test --manifest-path backend/Cargo.toml --test zulip_live -- --ignored --nocapture
```

The fixture writes progress to stderr so first-boot image pulls and Zulip
readiness are visible. Long-running Zulip Compose startup, realm provisioning
and backend live-evidence commands also emit periodic heartbeat lines with
elapsed time.

## Classification model

Макошь does not yet physically relocate every backend test into `tests/unit`, `tests/integration`, `tests/e2e`, `tests/architecture`, `tests/snapshots`. The repository now uses a stable logical classification generated from the current target naming and a dedicated snapshot target:

- `unit` - Rust library tests under `backend/src`, `crates/test-session/src` and `crates/testkit/src` (`makosh-backend-testkit`)
- `integration` - backend integration targets that are not architecture/e2e/snapshot targets
- `e2e` - high-surface API/runtime targets such as `*_api`, stream/websocket API targets, `communications_connectrpc`, `omniroute`, `hard_v1_routes`
- `architecture` - `*_architecture.rs` targets plus JS architecture guards
- `snapshot` - backend snapshot targets using `insta`

The classifier lives in `scripts/test/backend-test-targets.mjs`.

## Reports

`cargo-nextest` writes JUnit XML into `target/nextest/<profile>/junit.xml`.

Post-run summaries are written into `reports/test-performance/` by `scripts/test/analyze-nextest-junit.mjs`.

Current baseline and optimization notes live in:

- `reports/test-performance/README.md`
- `reports/test-performance/2026-06-23-baseline.md`
- `reports/test-performance/backend-full.md`
- `docs/development/testing/status.md`

## Navigation

- [Status](./status.md)
- [CI](./ci.md)
- [Coverage](./coverage.md)
- [Mutation Testing](./mutation-testing.md)
- [Nextest](./nextest.md)
- [Security](./security.md)
- [Snapshots](./snapshots.md)
- [Макошь Lab](./makosh-lab.md)
- [Communication Compliance Suite](./communication-compliance-suite.md)
