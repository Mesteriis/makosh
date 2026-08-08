#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

source "${REPO_ROOT}/scripts/lib/rust-tooling.sh"

PROFILE="${1:-ci}"
shift || true

require_cargo_subcommand "llvm-cov" "cargo install --locked cargo-llvm-cov"
require_cargo_subcommand "nextest" "cargo install --locked cargo-nextest"

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target/coverage-build}"
NEXTEST_SHOW_PROGRESS="${NEXTEST_SHOW_PROGRESS:-bar}"
export CARGO_TARGET_DIR
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"

cd "${REPO_ROOT}"

cargo llvm-cov clean --workspace
cargo run --manifest-path crates/test-session/Cargo.toml --bin makosh-test-session -- \
	cargo llvm-cov nextest \
		--manifest-path backend/Cargo.toml \
		--profile "${PROFILE}" \
		--show-progress "${NEXTEST_SHOW_PROGRESS}" \
		--test-threads "${MAKOSH_NEXTEST_JOBS:-4}" \
		"$@"
