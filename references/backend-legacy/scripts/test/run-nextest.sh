#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

source "${REPO_ROOT}/scripts/lib/rust-tooling.sh"

PROFILE="${1:-default}"
shift || true

require_cargo_subcommand "nextest" "cargo install --locked cargo-nextest"

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target/validate-test}"
NEXTEST_SHOW_PROGRESS="${NEXTEST_SHOW_PROGRESS:-bar}"
export CARGO_TARGET_DIR
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-1}"

# sccache rejects incremental compilation. Local nextest loops intentionally
# prefer incremental builds, while CI sets CARGO_INCREMENTAL=0 explicitly and
# can keep the compiler cache enabled.
disable_sccache_for_incremental

cd "${REPO_ROOT}"

cargo run --manifest-path crates/test-session/Cargo.toml --bin makosh-test-session -- \
	cargo nextest run \
		--manifest-path backend/Cargo.toml \
		--profile "${PROFILE}" \
		--show-progress "${NEXTEST_SHOW_PROGRESS}" \
		--test-threads "${MAKOSH_NEXTEST_JOBS:-4}" \
		"$@"
