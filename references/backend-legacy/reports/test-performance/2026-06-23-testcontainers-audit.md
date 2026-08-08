# Testcontainers audit - 2026-06-23

## Verified current behavior

- `crates/testkit/src/context.rs` keeps PostgreSQL and NATS containers in `tokio::sync::OnceCell`.
- `crates/testkit/src/bin/makosh_test_session.rs` creates a single owned PostgreSQL session container for the full backend run and tears it down when the command exits.
- Individual tests create isolated databases inside the shared PostgreSQL container instead of starting a new PostgreSQL container per test.
- NATS is started lazily only for tests that request it through `TestContext::nats_server_url()` / `app_config_with_nats()`.

## Main risk found

Direct full-suite `cargo test` bypasses `makosh_test_session`. That removes the shared-session contract and is the main path that can leave extra Docker garbage behind after interrupts or failures.

## Changes in this modernization pass

1. Full backend test and coverage entry points are now routed through `scripts/test/run-nextest.sh` and `scripts/test/run-llvm-cov.sh`.
2. Repository guidance explicitly keeps `make backend-test` / `make backend-validate` as the safe harness entry points.
3. CI jobs are split so heavy container-backed lanes run separately from unit/snapshot lanes.

## Remaining optimization opportunities

1. Move more backend logic from integration targets into lib/unit tests where Docker is not needed.
2. Identify the slowest container-backed suites and merge repeated bootstraps inside the same target where practical.
3. Add per-target nextest grouping once there is enough measured data to justify serializing specific resource-heavy targets.
4. Keep Redis out of the test stack unless a real Макошь subsystem needs it and the architecture decision is explicit.
