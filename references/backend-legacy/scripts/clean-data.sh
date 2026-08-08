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
confirm_or_exit "This will delete local PostgreSQL data under $(postgres_data_dir)." "DELETE"
compose_cmd down --remove-orphans >/dev/null 2>&1 || true
rm -rf "$(postgres_data_dir)"
mkdir -p "$(postgres_data_dir)"
success "Deleted PostgreSQL development data"
