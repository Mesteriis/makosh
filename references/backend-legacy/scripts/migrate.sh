#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/postgres.sh
source "$SCRIPT_DIR/lib/postgres.sh"

load_makosh_env
ensure_command cargo
postgres_up

info "Running backend-managed SQLx migrations"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CARGO_DEV_TARGET_DIR}" \
	MAKOSH_LOG_FORMAT=plain \
	cargo run --manifest-path "$REPO_ROOT/backend/Cargo.toml" --bin makosh_migrate
success "Migrations applied successfully"
