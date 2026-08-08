# cargo-nextest

`cargo-nextest` is the default Rust test runner in Макошь.

Why:

- per-test process isolation
- retries and flaky detection
- slow test detection
- JUnit XML output
- better CI ergonomics than `cargo test`

Repository configuration lives in `.config/nextest.toml`.

Profiles:

- `default` - local broad runs
- `ci` - CI-oriented runs with more retries and separate JUnit output
- `integration` - slower container-backed runs

Important constraint:

For full backend runs, prefer `make backend-test`, `make backend-validate`, `make test`, `make test-ci`, `make test-integration`, or `make test-e2e`.

These routes keep the `makosh_test_session` harness in front of nextest so shared testcontainers are reused and cleaned up.
