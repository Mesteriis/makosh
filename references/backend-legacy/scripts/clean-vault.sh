#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"

load_makosh_env
confirm_or_exit "This will delete local vault data under $MAKOSH_HOST_VAULT_HOME." "DELETE"
rm -rf "$MAKOSH_HOST_VAULT_HOME"
success "Deleted local vault data at $MAKOSH_HOST_VAULT_HOME"
