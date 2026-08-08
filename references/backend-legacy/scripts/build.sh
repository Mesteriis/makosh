#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/resources.sh
source "$SCRIPT_DIR/lib/resources.sh"

load_makosh_env
ensure_frontend_dependencies
ensure_command cargo
ensure_command node
ensure_command pnpm

backend_target_dir="${CARGO_TARGET_DIR:-$CARGO_BUILD_TARGET_DIR}"

# Generate a per-build random local API secret unless the caller provided one.
# It is baked into both the frontend dist (VITE_MAKOSH_LOCAL_API_SECRET) and
# the Tauri shell (MAKOSH_BUNDLED_LOCAL_API_SECRET via option_env!), so the
# packaged app never ships with the well-known development fallback.
if [ -z "${MAKOSH_BUNDLED_LOCAL_API_SECRET:-}" ]; then
	ensure_command openssl
	MAKOSH_BUNDLED_LOCAL_API_SECRET="$(openssl rand -hex 32)"
	info "Generated random bundled local API secret for this build"
fi
export MAKOSH_BUNDLED_LOCAL_API_SECRET
export VITE_MAKOSH_LOCAL_API_SECRET="$MAKOSH_BUNDLED_LOCAL_API_SECRET"
prepare_bundled_provider_runtime_env

info "Building backend release binary"
CARGO_TARGET_DIR="$backend_target_dir" \
	cargo build --manifest-path "$REPO_ROOT/backend/Cargo.toml" --bin makosh-backend --release

info "Building frontend release assets"
(
	cd "$REPO_ROOT/frontend"
	pnpm build
)

info "Preparing bundled desktop resources"
prepare_google_oauth_resource
prepare_tdlib_macos
CARGO_TARGET_DIR="$backend_target_dir" prepare_backend_sidecar_macos

info "Building Tauri release artifacts"
(
	cd "$REPO_ROOT/frontend"
	pnpm tauri build
)

success "Release build completed"
