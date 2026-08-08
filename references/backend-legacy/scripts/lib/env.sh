#!/usr/bin/env bash

set -euo pipefail

# shellcheck source=./common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

DOCKER_ENV_FILE="$REPO_ROOT/docker/.env"
DOCKER_ENV_TEMPLATE="$REPO_ROOT/docker/.env.example"
LOCAL_ENV_FILE="$REPO_ROOT/.env"

provider_runtime_env_names() {
	printf '%s\n' \
		MAKOSH_TDJSON_PATH \
		MAKOSH_TELEGRAM_API_ID \
		MAKOSH_TELEGRAM_API_HASH \
		MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH \
		MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE \
		MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON \
		MAKOSH_GOOGLE_OAUTH_CLIENT_ID \
		MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET
}

ensure_docker_env_file() {
	if [ ! -f "$DOCKER_ENV_FILE" ]; then
		cp "$DOCKER_ENV_TEMPLATE" "$DOCKER_ENV_FILE"
		warn "Created docker/.env from docker/.env.example. Review local secrets before continuing."
	fi
}

source_env_file() {
	local env_file="$1"
	if [ -z "$env_file" ]; then
		return 0
	fi
	if [ ! -f "$env_file" ]; then
		error "Макошь env file was not found: $env_file"
		exit 1
	fi
	set -a
	# shellcheck disable=SC1090
	. "$env_file"
	set +a
}

source_env_file_if_exists() {
	local env_file="$1"
	if [ -f "$env_file" ]; then
		source_env_file "$env_file"
	fi
}

load_makosh_env() {
	prepend_tools_bin_to_path
	ensure_docker_env_file
	source_env_file "$DOCKER_ENV_FILE"
	source_env_file_if_exists "$LOCAL_ENV_FILE"
	import_launchctl_env_name MAKOSH_ENV_FILE
	source_env_file "${MAKOSH_ENV_FILE:-}"

	: "${MAKOSH_POSTGRES_DB:=makosh_hub}"
	: "${MAKOSH_POSTGRES_USER:=makosh}"
	: "${MAKOSH_POSTGRES_PASSWORD:=change-me-local-dev-only}"
	: "${MAKOSH_POSTGRES_BIND:=127.0.0.1}"
	: "${MAKOSH_POSTGRES_PORT:=30432}"
	: "${MAKOSH_NATS_BIND:=127.0.0.1}"
	: "${MAKOSH_NATS_PORT:=34222}"
	: "${MAKOSH_NATS_MONITOR_BIND:=127.0.0.1}"
	: "${MAKOSH_NATS_MONITOR_PORT:=38222}"
	: "${MAKOSH_CLAMAV_PORT:=33310}"
	: "${MAKOSH_CLAMAV_ADDR:=127.0.0.1:$MAKOSH_CLAMAV_PORT}"
	: "${MAKOSH_CLAMAV_TIMEOUT_SECONDS:=30}"
	: "${MAKOSH_ATTACHMENT_EXTRACTOR_ENABLED:=true}"
	: "${MAKOSH_ATTACHMENT_EXTRACTOR_ADDR:=127.0.0.1:8788}"
	: "${MAKOSH_BACKEND_BIND:=127.0.0.1}"
	: "${MAKOSH_BACKEND_PORT:=8080}"
	: "${MAKOSH_BACKEND_STARTUP_ATTEMPTS:=300}"
	: "${MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS:=1}"
	: "${MAKOSH_FRONTEND_BIND:=127.0.0.1}"
	: "${MAKOSH_FRONTEND_PORT:=5174}"
	: "${MAKOSH_FRONTEND_STARTUP_ATTEMPTS:=120}"
	: "${MAKOSH_FRONTEND_STARTUP_SLEEP_SECONDS:=1}"
	: "${MAKOSH_LOCAL_API_SECRET:=change-me-local-api-secret}"
	: "${MAKOSH_DEV_MODE:=true}"
	: "${MAKOSH_HOST_VAULT_HOME:=$HOME/.makosh/vault}"
	: "${MAKOSH_SECRET_VAULT_KEY:=change-me-local-secret-vault-key}"
	: "${MAKOSH_OLLAMA_BASE_URL:=http://127.0.0.1:11434}"
	: "${MAKOSH_OLLAMA_CHAT_MODEL:=qwen3:4b}"
	: "${MAKOSH_OLLAMA_EMBED_MODEL:=qwen3-embedding:4b}"
	: "${MAKOSH_OLLAMA_TIMEOUT_SECONDS:=120}"

	MAKOSH_VAULT_HOME="$MAKOSH_HOST_VAULT_HOME"
	MAKOSH_DEV_KEY_PATH="$MAKOSH_HOST_VAULT_HOME/dev/master.key"
	DATABASE_URL="postgres://${MAKOSH_POSTGRES_USER}:${MAKOSH_POSTGRES_PASSWORD}@127.0.0.1:${MAKOSH_POSTGRES_PORT}/${MAKOSH_POSTGRES_DB}"
	MAKOSH_NATS_SERVER_URL="${MAKOSH_NATS_SERVER_URL:-nats://127.0.0.1:${MAKOSH_NATS_PORT:-34222}}"

	export MAKOSH_VAULT_HOME
	export MAKOSH_DEV_KEY_PATH
	export DATABASE_URL
	export MAKOSH_NATS_BIND
	export MAKOSH_NATS_PORT
	export MAKOSH_NATS_MONITOR_BIND
	export MAKOSH_NATS_MONITOR_PORT
	export MAKOSH_NATS_SERVER_URL
	export MAKOSH_CLAMAV_ADDR
	export MAKOSH_CLAMAV_TIMEOUT_SECONDS
	export MAKOSH_ATTACHMENT_EXTRACTOR_ENABLED
	export MAKOSH_ATTACHMENT_EXTRACTOR_ADDR
	import_launchctl_provider_runtime_env
	export_provider_runtime_env
}

import_launchctl_env_name() {
	local name="$1"
	if [ "$(uname -s 2>/dev/null || true)" != "Darwin" ]; then
		return 0
	fi
	if ! command -v launchctl >/dev/null 2>&1; then
		return 0
	fi
	if [ "${!name+x}" = "x" ]; then
		return 0
	fi

	local value uid
	value="$(launchctl getenv "$name" 2>/dev/null || true)"
	if [ -z "$value" ]; then
		uid="$(id -u)"
		value="$(launchctl asuser "$uid" getenv "$name" 2>/dev/null || true)"
	fi
	if [ -n "$value" ]; then
		export "$name=$value"
	fi
}

import_launchctl_provider_runtime_env() {
	local name
	while IFS= read -r name; do
		import_launchctl_env_name "$name"
	done < <(provider_runtime_env_names)
}

export_provider_runtime_env() {
	local name
	while IFS= read -r name; do
		if [ "${!name+x}" = "x" ]; then
			export "$name"
		fi
	done < <(provider_runtime_env_names)
}

prepare_bundled_provider_runtime_env() {
	if [ -z "${MAKOSH_BUNDLED_TELEGRAM_API_ID:-}" ] && [ -n "${MAKOSH_TELEGRAM_API_ID:-}" ]; then
		MAKOSH_BUNDLED_TELEGRAM_API_ID="$MAKOSH_TELEGRAM_API_ID"
		export MAKOSH_BUNDLED_TELEGRAM_API_ID
	fi
	if [ -z "${MAKOSH_BUNDLED_TELEGRAM_API_HASH:-}" ] && [ -n "${MAKOSH_TELEGRAM_API_HASH:-}" ]; then
		MAKOSH_BUNDLED_TELEGRAM_API_HASH="$MAKOSH_TELEGRAM_API_HASH"
		export MAKOSH_BUNDLED_TELEGRAM_API_HASH
	fi
	if [ -z "${MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON:-}" ]; then
		if [ -n "${MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON:-}" ]; then
			MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON="$MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON"
			export MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON
		else
			local google_oauth_source
			google_oauth_source="${MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE:-${MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH:-}}"
			if [ -n "$google_oauth_source" ] && [ -f "$google_oauth_source" ]; then
				MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON="$(cat "$google_oauth_source")"
				export MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON
			fi
		fi
	fi
	if [ -z "${MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_ID:-}" ] && [ -n "${MAKOSH_GOOGLE_OAUTH_CLIENT_ID:-}" ]; then
		MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_ID="$MAKOSH_GOOGLE_OAUTH_CLIENT_ID"
		export MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_ID
	fi
	if [ -z "${MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET:-}" ] && [ -n "${MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET:-}" ]; then
		MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET="$MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET"
		export MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET
	fi
}

ensure_bacon_available() {
	prepend_tools_bin_to_path
	if command -v bacon >/dev/null 2>&1; then
		return 0
	fi

	ensure_command cargo
	ensure_dir "$TOOLS_ROOT"
	info "Installing local bacon into $TOOLS_ROOT"
	cargo install --locked --root "$TOOLS_ROOT" bacon
	prepend_tools_bin_to_path

	if ! command -v bacon >/dev/null 2>&1; then
		error "bacon installation completed but binary was not found in $TOOLS_BIN"
		exit 1
	fi
}

ensure_frontend_dependencies() {
	ensure_command pnpm
	if [ ! -d "$REPO_ROOT/frontend/node_modules" ] || [ ! -x "$REPO_ROOT/frontend/node_modules/.bin/tauri" ]; then
		info "Installing frontend dependencies"
		(
			cd "$REPO_ROOT/frontend"
			pnpm install --frozen-lockfile
		)
	fi
}
