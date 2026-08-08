# Coverage

Coverage is provided by `cargo-llvm-cov` and runs through `cargo nextest`.

Commands:

- `make coverage` - summary report
- `make coverage-html` - HTML report under `target/coverage/html`
- `make coverage-ci` - LCOV report under `target/coverage/lcov.info`

Important constraint:

Coverage commands also run through the `makosh_test_session` harness so integration tests keep the same testcontainer lifecycle guarantees as normal backend runs.
