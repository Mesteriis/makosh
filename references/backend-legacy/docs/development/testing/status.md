# Test Modernization Status

Status date: `2026-06-23`

This file tracks what is actually completed for the Макошь test modernization plan, what is only partially implemented, and what is still unverified.

## Overall readiness

- Completed: `13 / 16`
- Partial or integrated but not fully proven: `3 / 16`
- Not started: `0 / 16`

This is not a marketing summary. It is the current evidence-based state from the repository worktree and the validation executed so far.

## Acceptance criteria matrix

1. `cargo-nextest` is used by default
   Status: `complete`
   Evidence: `.config/nextest.toml`, `.cargo/config.toml`, `Makefile`, `scripts/test/run-nextest.sh`

2. All tests are split into categories
   Status: `partial`
   Evidence: `scripts/test/backend-test-targets.mjs`, `Makefile` category targets, snapshot target.
   Gap: classification is logical and automated, but backend tests were not physically relocated into `tests/unit`, `tests/integration`, `tests/e2e`, `tests/architecture`, `tests/snapshots`.

3. Coverage works through `cargo-llvm-cov`
   Status: `complete`
   Evidence: `scripts/test/run-llvm-cov.sh`, `Makefile` coverage targets, CI coverage job, real local run that produced `target/coverage/lcov.info`.

4. Snapshot testing works through `insta`
   Status: `complete`
   Evidence: `backend/tests/snapshot_smoke.rs`, committed snapshot file, targeted test pass.

5. Mutation testing works through `cargo-mutants`
   Status: `partial`
   Evidence: `Makefile`, nightly CI workflow, docs, real local mutant enumeration observed `38` mutants, interrupted local execution attempt against `backend/src/app/handlers/communications/remote_images/url_policy.rs`.
   Gap: a full local mutation test pass is still too expensive for the current crate shape; the local run spent almost five minutes in baseline build inside a 2.6 GB scratch copy before being interrupted.

6. `sccache` is integrated
   Status: `complete`
   Evidence: `Makefile` exports `RUSTC_WRAPPER` when `sccache` is present, cache commands exist, and a measured local cold/warm experiment was executed.
   Result: `sccache --show-stats` reported 1300 compile requests, 124 hits, 996 Rust misses, 11.07% overall hit rate, 0.00% Rust hit rate in the cross-target-dir experiment. Integration is proven; optimization remains open.

7. `cargo-watch` is integrated
   Status: `complete`
   Evidence: `Makefile` watch targets, docs.

8. `cargo-audit` is integrated
   Status: `complete`
   Evidence: `Makefile`, CI security lane, docs, real `make security` / `cargo audit` execution.
   Note: `make audit` is green with a documented `RUSTSEC-2023-0071` ignore for the inactive optional `sqlx-mysql -> rsa` lockfile path. `cargo tree -i sqlx-mysql` prints no active backend/`makosh-backend-testkit` path; SQLx is configured for PostgreSQL only.

9. `cargo-deny` is integrated
   Status: `complete`
   Evidence: `Makefile`, `deny.toml`, CI security lane, docs, real `make deny` execution.
   Note: `make deny` is green after updating the testcontainers dependency chain. Duplicate-version warnings remain non-fatal.

10. `cargo-udeps` is integrated
   Status: `complete`
   Evidence: `Makefile`, docs, installed nightly toolchain, real `make udeps` execution.
   Result: current graph is green after removing unused `mockall` and `testcontainers-modules` dependencies.

11. Documentation exists
    Status: `complete`
    Evidence: `docs/development/testing/`

12. Updated `Makefile` exists
    Status: `complete`
    Evidence: root `Makefile`

13. CI integration exists
    Status: `complete`
    Evidence: `.github/workflows/ci.yml`, `.github/workflows/nightly.yml`

14. Acceleration report exists
   Status: `partial`
   Evidence: `reports/test-performance/2026-06-23-baseline.md`, `2026-06-23-testcontainers-audit.md`, `reports/test-performance/backend-full.md`, `reports/test-performance/unit.md`
   Gap: the repository now has measured after-state timings, but it still lacks a normalized before/after comparison from equivalent full-suite runs.

15. Slowest-tests report exists
    Status: `complete`
    Evidence: baseline report, JUnit analyzer, `make test-unit`, `reports/test-performance/unit.md`, `reports/test-performance/unit.json`

16. No degradation of existing functionality
   Status: `complete`
   Evidence: `make validate` passed with architecture/code-boundary/backend/frontend gates green. After the NATS cleanup fix, `make backend-validate` passed again with `1223` backend tests green.

## What was done in this pass

- Added `cargo-nextest` configuration and dedicated profiles.
- Added reusable shell helpers for Rust tooling checks.
- Added nextest/coverage/report scripts.
- Added a first committed backend snapshot test.
- Added CI split for PR, main, and nightly quality lanes.
- Added docs for nextest, coverage, snapshots, mutation testing, security, and CI.
- Added baseline and testcontainers audit reports.
- Added `color-eyre` bootstrap in the backend binary.
- Added explicit nextest progress-bar flags in the local wrappers so future `make backend-test` / coverage runs are not silent.
- Added nextest progress flags to direct Makefile nextest targets and post-run ASCII summary output after JUnit report generation.
- Upgraded the testcontainers chain and `quinn-proto`, removed unused Rust dependencies, and documented the remaining cargo-audit lockfile-only `rsa` exception.

## What is still not fully closed

- A complete local mutation test pass on a real backend module
- A normalized before/after acceleration comparison from equivalent full-suite runs

## Confirmed command results in this pass

- `make test-unit`
  - Result: passed
  - Evidence: `261 tests run: 261 passed`, `target/nextest/default/junit.xml`, `reports/test-performance/unit.{json,md}`
- `make backend-test`
  - Result: passed
  - Evidence: `1223 tests run: 1223 passed`, `reports/test-performance/backend-full.{json,md}`
- `make backend-validate`
  - Result: passed after the NATS session-container cleanup fix
  - Evidence: `1223 tests run: 1223 passed`, final post-run summary in `reports/test-performance/backend-full.{json,md}`
- `make backend-clippy`
  - Result: passed
- `make frontend-validate`
  - Result: passed
  - Evidence: `132` frontend test files passed, `468` frontend tests passed, `vite build` passed
- `make architecture-check`
  - Result: passed after adding an application-layer project review mirror wrapper and explicit transitional projection-bridge exceptions for existing derived-domain stores
- `make code-boundaries-check`
  - Result: passed after excluding generated frontend protobuf and frontend build output from source-boundary scanning
- `make backend-fmt-check`
  - Result: passed after applying `cargo fmt`
- `./scripts/test/run-llvm-cov.sh ci --test snapshot_smoke --lcov --output-path target/coverage/lcov.info`
  - Result: passed
  - Evidence: `target/coverage/lcov.info`
- `make udeps`
  - Result: passed after removing `backend` dev-dependency `mockall` and `crates/testkit` dependency `testcontainers-modules`
- `make security`
  - Result: passed after dependency updates and the documented inactive `sqlx-mysql -> rsa` cargo-audit ignore
- `make deny`
  - Result: passed; duplicate-version warnings remain non-fatal
- `make validate`
  - Result: passed
  - Evidence: architecture/code-boundary/backend/frontend gates completed; frontend reported `132` files and `468` tests passed, and `vite build` completed.

## Mutation testing note

- `cargo mutants --list -f 'backend/src/app/handlers/communications/remote_images/url_policy.rs'`
  - Result: found `38` mutants
- `cargo mutants -f 'backend/src/app/handlers/communications/remote_images/url_policy.rs' --test-tool nextest -j 1`
  - Result: started real execution, but local run was interrupted during the baseline build phase after ~5 minutes
  - Evidence: command output was observed locally; generated `mutants.out*` scratch directories were removed instead of being retained in the worktree.
  - Important nuance: in this repository layout the working file filter must match `backend/src/...`, not only `src/...`, or `cargo-mutants` reports `0 mutants to test`

## Container cleanup note

- Earlier full runs left stale anonymous NATS testcontainers because `NATS_CONTAINER` was stored in a static `OnceCell`; Rust statics are not dropped at test-binary exit, so `ContainerAsync` did not reliably clean up.
- `makosh_test_session` now owns one NATS container and one PostgreSQL container per backend session and passes their ports to test binaries through environment variables.
- Verified with targeted `event_platform` and final `make backend-validate`: temporary testcontainers were removed after the harness exited; only the named development Compose service `makosh-dev-postgres-1` remained.

## Current security posture

From `cargo audit`:

- `RUSTSEC-2023-0071` for `rsa 0.9.10` is ignored by `make audit` because it is pulled into `Cargo.lock` through inactive optional `sqlx-mysql`; active SQLx usage is PostgreSQL only.
- warnings remain for `lru 0.12.5` and `memmap2 0.9.10`, and cargo-audit treats them as allowed warnings.

From `cargo deny`:

- `advisories`, `bans`, `licenses`, and `sources` pass.
- duplicate-version warnings remain, but they are not hard failures under current policy.

The previous `tokio-tar`, `rustls-pemfile`, and `quinn-proto` blockers were removed by updating the testcontainers chain and `quinn-proto`.

## Repository-wide gate status

- `make validate` was executed and passed.
- `make backend-validate` was executed and passed again after the NATS testcontainer cleanup fix.
- `make frontend-validate` was executed and passed.
- `make security` and `make udeps` were executed and passed.
- `make architecture-check`, `make code-boundaries-check`, and `make backend-fmt-check` were executed and passed.
- Docker cleanup was verified after backend validation; no anonymous testcontainers remained.
