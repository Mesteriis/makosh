#!/usr/bin/env bash

set -euo pipefail

# shellcheck source=./common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

prepare_backend_sidecar_macos() {
	local binary_root="$REPO_ROOT/frontend/src-tauri/binaries"
	local backend_manifest="$REPO_ROOT/backend/Cargo.toml"
	local backend_bin="makosh-backend"
	local target_root target_triple source_bin target_bin

	if [ "$(uname -s)" != "Darwin" ]; then
		info "Skipping macOS backend sidecar preparation on non-macOS host"
		return 0
	fi

	case "$(uname -m)" in
		arm64)
			target_triple="${MAKOSH_MACOS_TARGET_TRIPLE:-aarch64-apple-darwin}"
			;;
		x86_64)
			target_triple="${MAKOSH_MACOS_TARGET_TRIPLE:-x86_64-apple-darwin}"
			;;
		*)
			error "Unsupported macOS architecture: $(uname -m)"
			exit 1
			;;
	esac

	target_root="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
	source_bin="$target_root/$target_triple/release/$backend_bin"
	target_bin="$binary_root/$backend_bin-$target_triple"

	cargo build \
		--manifest-path "$backend_manifest" \
		--bin "$backend_bin" \
		--release \
		--target "$target_triple"

	if [ ! -f "$source_bin" ]; then
		error "Backend sidecar build completed, but $source_bin was not found."
		exit 1
	fi

	mkdir -p "$binary_root"
	cp "$source_bin" "$target_bin"
	chmod 0755 "$target_bin"
	success "Prepared bundled backend sidecar: $target_bin"
}

prepare_google_oauth_resource() {
	local resource_root="$REPO_ROOT/frontend/src-tauri/resources/google-oauth"
	local target_json="$resource_root/client_secret.json"
	local source_json
	source_json="${MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE:-${MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH:-}}"

	if [ -z "$source_json" ]; then
		if [ -n "${MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON:-}" ] || [ -n "${MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_ID:-}" ]; then
			info "Skipping bundled Google OAuth resource; build already has bundled Google OAuth config"
			return 0
		fi
		error "Unable to prepare bundled Google OAuth Desktop client resource. Set MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH."
		exit 1
	fi
	if [ ! -f "$source_json" ]; then
		error "Google OAuth client config file was not found: $source_json"
		exit 1
	fi

	node - "$source_json" <<'NODE'
const fs = require('fs');
const sourcePath = process.argv[2];
const raw = fs.readFileSync(sourcePath, 'utf8');
const parsed = JSON.parse(raw);
if (!parsed || typeof parsed !== 'object' || !parsed.installed || typeof parsed.installed !== 'object') {
	throw new Error('Google OAuth bundle config must be a Desktop app JSON with top-level "installed".');
}
for (const field of ['client_id', 'auth_uri', 'token_uri']) {
	if (typeof parsed.installed[field] !== 'string' || parsed.installed[field].trim() === '') {
		throw new Error(`Google OAuth Desktop client JSON is missing required field: installed.${field}`);
	}
}
NODE

	mkdir -p "$resource_root"
	cp "$source_json" "$target_json"
	chmod 0644 "$target_json"
	success "Prepared bundled Google OAuth Desktop client resource: $target_json"
}

prepare_tdlib_macos() {
	local resource_root="$REPO_ROOT/frontend/src-tauri/resources/tdlib"
	local platform_dir target_dir target_lib source_lib

	if [ "$(uname -s)" != "Darwin" ]; then
		info "Skipping macOS TDLib resource preparation on non-macOS host"
		return 0
	fi

	case "$(uname -m)" in
		arm64)
			platform_dir="${MAKOSH_TDLIB_MACOS_PLATFORM_DIR:-macos-arm64}"
			;;
		x86_64)
			platform_dir="${MAKOSH_TDLIB_MACOS_PLATFORM_DIR:-macos-x64}"
			;;
		*)
			error "Unsupported macOS architecture: $(uname -m)"
			exit 1
			;;
	esac

	target_dir="$resource_root/$platform_dir"
	target_lib="$target_dir/libtdjson.dylib"
	source_lib="$(find_tdjson_source_lib || true)"

	if [ -z "$source_lib" ] && [ "${MAKOSH_TDLIB_BUILD_FROM_SOURCE:-0}" = "1" ]; then
		source_lib="$(build_tdlib_from_source || true)"
	fi
	if [ -z "$source_lib" ] || [ ! -f "$source_lib" ]; then
		error "Unable to find libtdjson.dylib. Set MAKOSH_TDJSON_SOURCE/MAKOSH_TDJSON_PATH or install tdlib."
		exit 1
	fi

	mkdir -p "$target_dir"
	cp "$source_lib" "$target_lib"
	chmod 0644 "$target_lib"
	success "Prepared bundled TDLib runtime: $target_lib"
}

find_tdjson_source_lib() {
	if [ -n "${MAKOSH_TDJSON_SOURCE:-}" ]; then
		printf '%s\n' "$MAKOSH_TDJSON_SOURCE"
		return 0
	fi
	if [ -n "${MAKOSH_TDJSON_PATH:-}" ]; then
		printf '%s\n' "$MAKOSH_TDJSON_PATH"
		return 0
	fi
	if command -v brew >/dev/null 2>&1; then
		local brew_prefix
		if brew_prefix="$(brew --prefix tdlib 2>/dev/null)"; then
			printf '%s\n' "$brew_prefix/lib/libtdjson.dylib"
			return 0
		fi
	fi
	if [ -f /opt/homebrew/lib/libtdjson.dylib ]; then
		printf '%s\n' /opt/homebrew/lib/libtdjson.dylib
		return 0
	fi
	if [ -f /usr/local/lib/libtdjson.dylib ]; then
		printf '%s\n' /usr/local/lib/libtdjson.dylib
		return 0
	fi
	return 1
}

build_tdlib_from_source() {
	local build_root source_dir build_dir tdlib_ref built_lib
	build_root="${MAKOSH_TDLIB_BUILD_ROOT:-$REPO_ROOT/.local/tdlib-build}"
	source_dir="$build_root/td"
	build_dir="$source_dir/build"
	tdlib_ref="${MAKOSH_TDLIB_REF:-master}"

	ensure_command git
	ensure_command cmake

	mkdir -p "$build_root"
	if [ ! -d "$source_dir/.git" ]; then
		git clone https://github.com/tdlib/td.git "$source_dir"
	fi

	git -C "$source_dir" fetch --tags origin
	git -C "$source_dir" checkout "$tdlib_ref"
	cmake -S "$source_dir" -B "$build_dir" -DCMAKE_BUILD_TYPE=Release -DTD_ENABLE_JNI=OFF
	cmake --build "$build_dir" --target tdjson --config Release --parallel "$(sysctl -n hw.ncpu)"

	built_lib="$(find "$build_dir" -type f -name libtdjson.dylib -print -quit)"
	if [ -z "$built_lib" ]; then
		error "TDLib source build completed, but libtdjson.dylib was not found under $build_dir."
		exit 1
	fi

	printf '%s\n' "$built_lib"
}
