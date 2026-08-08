#!/usr/bin/env bash

set -euo pipefail

COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_DIR="$(cd "$COMMON_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SCRIPTS_DIR/.." && pwd)"
LOG_ROOT="$REPO_ROOT/.local/dev-logs"
BACKUPS_ROOT="$REPO_ROOT/backups"
TOOLS_ROOT="$REPO_ROOT/.local/tools"
TOOLS_BIN="$TOOLS_ROOT/bin"
MAIL_BLOB_ROOT="$REPO_ROOT/docker/data/mail"
CARGO_TARGET_ROOT="${CARGO_TARGET_ROOT:-$REPO_ROOT/target}"
CARGO_DEV_TARGET_DIR="${CARGO_DEV_TARGET_DIR:-$CARGO_TARGET_ROOT/dev}"
CARGO_VALIDATE_TARGET_DIR="${CARGO_VALIDATE_TARGET_DIR:-$CARGO_TARGET_ROOT/validate}"
CARGO_BUILD_TARGET_DIR="${CARGO_BUILD_TARGET_DIR:-$CARGO_TARGET_ROOT/build}"

color_reset=$'\033[0m'
color_blue=$'\033[34m'
color_green=$'\033[32m'
color_yellow=$'\033[33m'
color_red=$'\033[31m'
color_cyan=$'\033[36m'
color_dim=$'\033[2m'

now_utc() {
	date -u +"%Y-%m-%dT%H:%M:%SZ"
}

today_utc() {
	date -u +"%Y-%m-%d"
}

timestamp_compact_utc() {
	date -u +"%Y%m%dT%H%M%SZ"
}

status_line() {
	local color="$1"
	local label="$2"
	local message="$3"
	printf '%b[%s]%b %s\n' "$color" "$label" "$color_reset" "$message"
}

info() {
	status_line "$color_blue" "info" "$1"
}

success() {
	status_line "$color_green" "ok" "$1"
}

warn() {
	status_line "$color_yellow" "warn" "$1"
}

error() {
	status_line "$color_red" "error" "$1" >&2
}

dim() {
	printf '%b%s%b\n' "$color_dim" "$1" "$color_reset"
}

ensure_dir() {
	mkdir -p "$1"
}

prepend_tools_bin_to_path() {
	case ":$PATH:" in
		*":$TOOLS_BIN:"*) ;;
		*) export PATH="$TOOLS_BIN:$PATH" ;;
	esac
}

ensure_command() {
	local command_name="$1"
	if ! command -v "$command_name" >/dev/null 2>&1; then
		error "Required command not found: $command_name"
		exit 1
	fi
}

ensure_one_of() {
	local first="$1"
	local second="$2"
	if command -v "$first" >/dev/null 2>&1; then
		printf '%s\n' "$first"
		return 0
	fi
	if command -v "$second" >/dev/null 2>&1; then
		printf '%s\n' "$second"
		return 0
	fi
	error "Required command not found: need $first or $second"
	exit 1
}

# Frees a dev port held by a stale process from a previous `make dev` session.
# Only processes that look like Макошь dev tooling are terminated; anything
# else still aborts so we never silently kill an unrelated application.
reclaim_dev_port() {
	local port="$1"
	local label="$2"
	if ! command -v lsof >/dev/null 2>&1; then
		return 0
	fi

	local pids
	pids="$(lsof -nP -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null | sort -u || true)"
	if [ -z "$pids" ]; then
		return 0
	fi

	local pid command_path command_name
	for pid in $pids; do
		command_path="$(ps -p "$pid" -o comm= 2>/dev/null || true)"
		command_name="$(basename "${command_path:-unknown}")"
		case "$command_name" in
			makosh-backend | cargo | bacon | node | vite | esbuild | pnpm)
				warn "$label port $port is held by stale dev process '$command_name' (pid $pid); stopping it"
				kill "$pid" 2>/dev/null || true
				;;
			*)
				error "$label port $port is in use by '$command_name' (pid $pid), which does not look like a Макошь dev process. Stop it manually."
				exit 1
				;;
		esac
	done

	local attempt=1
	while [ "$attempt" -le 20 ]; do
		if ! lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
			success "$label port $port reclaimed"
			return 0
		fi
		sleep 0.5
		attempt=$((attempt + 1))
	done

	pids="$(lsof -nP -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null | sort -u || true)"
	for pid in $pids; do
		warn "$label port $port is still busy; force killing pid $pid"
		kill -9 "$pid" 2>/dev/null || true
	done
	sleep 1

	if lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
		error "$label port $port is still in use after attempting to reclaim it."
		exit 1
	fi
	success "$label port $port reclaimed"
}

confirm_or_exit() {
	local prompt="$1"
	local expected="${2:-DELETE}"
	local answer
	printf '%s Type %s to continue: ' "$prompt" "$expected"
	read -r answer
	if [ "$answer" != "$expected" ]; then
		error "Confirmation did not match. Aborting."
		exit 1
	fi
}

json_escape() {
	local value="$1"
	value=${value//\\/\\\\}
	value=${value//\"/\\\"}
	value=${value//$'\t'/\\t}
	value=${value//$'\r'/\\r}
	printf '%s' "$value"
}

file_size_bytes() {
	local file_path="$1"
	case "$(uname -s)" in
		Darwin) stat -f '%z' "$file_path" ;;
		*) stat -c '%s' "$file_path" ;;
	esac
}

sha256_file() {
	local file_path="$1"
	if command -v shasum >/dev/null 2>&1; then
		shasum -a 256 "$file_path" | awk '{print $1}'
		return 0
	fi
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$file_path" | awk '{print $1}'
		return 0
	fi
	error "Required command not found: need shasum or sha256sum"
	return 1
}

is_safe_backup_relative_path() {
	local relative_path="$1"
	if [ -z "$relative_path" ] || [[ "$relative_path" == /* ]] || [[ "$relative_path" == *\\* ]]; then
		return 1
	fi
	if [[ "$relative_path" == *$'\n'* ]] || [[ "$relative_path" == *$'\r'* ]] || [[ "$relative_path" == *$'\t'* ]]; then
		return 1
	fi
	case "/$relative_path/" in
		*/../*|*/./*) return 1 ;;
	esac
	return 0
}

ensure_regular_file_tree() {
	local directory="$1"
	local label="$2"
	local invalid_path
	invalid_path="$(find "$directory" -mindepth 1 \( -type l -o \( ! -type d -a ! -type f \) \) -print -quit)"
	if [ -n "$invalid_path" ]; then
		error "$label must contain only directories and regular files."
		return 1
	fi
}

# Writes a tab-separated, content-free inventory: relative path, SHA-256 and byte size.
# The caller owns the destination directory; symlinks and special files are rejected.
write_directory_integrity_manifest() {
	local source_directory="$1"
	local manifest_path="$2"
	local temporary_path="${manifest_path}.tmp.$$"
	local file_path relative_path file_count=0 total_bytes=0 file_bytes file_sha256

	ensure_regular_file_tree "$source_directory" "Backup source $source_directory" || return 1
	if ! : >"$temporary_path"; then
		error "Unable to create integrity manifest: $manifest_path"
		return 1
	fi
	while IFS= read -r -d '' file_path; do
		relative_path="${file_path#"$source_directory"/}"
		if ! is_safe_backup_relative_path "$relative_path"; then
			rm -f "$temporary_path"
			error "Backup source contains an unsafe relative path."
			return 1
		fi
		if ! file_bytes="$(file_size_bytes "$file_path")" \
			|| ! file_sha256="$(sha256_file "$file_path")"; then
			rm -f "$temporary_path"
			error "Unable to inspect a mail blob for backup integrity."
			return 1
		fi
		if ! printf '%s\t%s\t%s\n' "$relative_path" "$file_sha256" "$file_bytes" >>"$temporary_path"; then
			rm -f "$temporary_path"
			error "Unable to write mail blob integrity metadata."
			return 1
		fi
		file_count=$((file_count + 1))
		total_bytes=$((total_bytes + file_bytes))
	done < <(find "$source_directory" -type f -print0)
	if ! mv "$temporary_path" "$manifest_path"; then
		rm -f "$temporary_path"
		error "Unable to finalize integrity manifest: $manifest_path"
		return 1
	fi
	printf '%s\t%s\n' "$file_count" "$total_bytes"
}

# Validates the backup inventory before it is copied into the live blob root.
# It also rejects added, omitted, symlinked, or special files.
verify_directory_integrity_manifest() {
	local source_directory="$1"
	local manifest_path="$2"
	local expected_paths actual_paths expected_sorted actual_sorted
	local relative_path expected_sha expected_size extra_field actual_path actual_size actual_sha

	if [ ! -f "$manifest_path" ]; then
		error "Integrity manifest is missing: $manifest_path"
		return 1
	fi
	ensure_regular_file_tree "$source_directory" "Backup source $source_directory" || return 1
	expected_paths="$(mktemp "${TMPDIR:-/tmp}/makosh-backup-expected.XXXXXX")" || return 1
	actual_paths="$(mktemp "${TMPDIR:-/tmp}/makosh-backup-actual.XXXXXX")" || {
		rm -f "$expected_paths"
		return 1
	}
	expected_sorted="$(mktemp "${TMPDIR:-/tmp}/makosh-backup-expected-sorted.XXXXXX")" || {
		rm -f "$expected_paths" "$actual_paths"
		return 1
	}
	actual_sorted="$(mktemp "${TMPDIR:-/tmp}/makosh-backup-actual-sorted.XXXXXX")" || {
		rm -f "$expected_paths" "$actual_paths" "$expected_sorted"
		return 1
	}

	while IFS=$'\t' read -r relative_path expected_sha expected_size extra_field || [ -n "${relative_path:-}" ]; do
		if ! is_safe_backup_relative_path "${relative_path:-}" \
			|| ! [[ "${expected_sha:-}" =~ ^[0-9a-f]{64}$ ]] \
			|| ! [[ "${expected_size:-}" =~ ^[0-9]+$ ]] \
			|| [ -n "${extra_field:-}" ]; then
			rm -f "$expected_paths" "$actual_paths" "$expected_sorted" "$actual_sorted"
			error "Integrity manifest contains an invalid record."
			return 1
		fi
		actual_path="$source_directory/$relative_path"
		if [ ! -f "$actual_path" ]; then
			rm -f "$expected_paths" "$actual_paths" "$expected_sorted" "$actual_sorted"
			error "Mail blob backup integrity verification failed."
			return 1
		fi
		if ! actual_size="$(file_size_bytes "$actual_path")" \
			|| ! actual_sha="$(sha256_file "$actual_path")" \
			|| [ "$actual_size" != "$expected_size" ] \
			|| [ "$actual_sha" != "$expected_sha" ]; then
			rm -f "$expected_paths" "$actual_paths" "$expected_sorted" "$actual_sorted"
			error "Mail blob backup integrity verification failed."
			return 1
		fi
		printf '%s\n' "$relative_path" >>"$expected_paths"
	done <"$manifest_path"

	while IFS= read -r -d '' actual_path; do
		relative_path="${actual_path#"$source_directory"/}"
		if ! is_safe_backup_relative_path "$relative_path"; then
			rm -f "$expected_paths" "$actual_paths" "$expected_sorted" "$actual_sorted"
			error "Backup source contains an unsafe relative path."
			return 1
		fi
		printf '%s\n' "$relative_path" >>"$actual_paths"
	done < <(find "$source_directory" -type f -print0)

	if ! env LC_ALL=C sort "$expected_paths" >"$expected_sorted" \
		|| ! env LC_ALL=C sort "$actual_paths" >"$actual_sorted"; then
		rm -f "$expected_paths" "$actual_paths" "$expected_sorted" "$actual_sorted"
		error "Unable to sort mail blob integrity metadata."
		return 1
	fi
	if ! cmp -s "$expected_sorted" "$actual_sorted"; then
		rm -f "$expected_paths" "$actual_paths" "$expected_sorted" "$actual_sorted"
		error "Mail blob backup inventory does not match its files."
		return 1
	fi
	rm -f "$expected_paths" "$actual_paths" "$expected_sorted" "$actual_sorted"
}

emit_json_log() {
	local file_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local message="$6"
	printf '{"timestamp":"%s","service":"%s","pid":%s,"level":"%s","flow_id":"%s","message":"%s"}\n' \
		"$(now_utc)" \
		"$(json_escape "$service")" \
		"$pid" \
		"$(json_escape "$level")" \
		"$(json_escape "$flow_id")" \
		"$(json_escape "$message")" >>"$file_path"
}

emit_live_log() {
	local file_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local message="$6"
	printf '[%s] service=%s pid=%s level=%s flow_id=%s %s\n' \
		"$(now_utc)" \
		"$service" \
		"$pid" \
		"$level" \
		"$flow_id" \
		"$message" >>"$file_path"
}

stream_service_pipe() {
	local pipe_path="$1"
	local service="$2"
	local pid="$3"
	local level="$4"
	local flow_id="$5"
	local color="$6"
	local log_file="$7"
	local live_log_file="$8"
	local line
	while IFS= read -r line || [ -n "$line" ]; do
		emit_json_log "$log_file" "$service" "$pid" "$level" "$flow_id" "$line"
		emit_live_log "$live_log_file" "$service" "$pid" "$level" "$flow_id" "$line"
		printf '%b[%s]%b %s\n' "$color" "$service" "$color_reset" "$line"
	done <"$pipe_path"
}

wait_for_http() {
	local url="$1"
	local label="$2"
	local attempts="${3:-60}"
	local sleep_seconds="${4:-1}"
	local index=1
	while [ "$index" -le "$attempts" ]; do
		if curl --silent --show-error --fail "$url" >/dev/null 2>&1; then
			return 0
		fi
		sleep "$sleep_seconds"
		index=$((index + 1))
	done
	error "$label did not become ready: $url"
	return 1
}

wait_for_service_http() {
	local pid="$1"
	local url="$2"
	local label="$3"
	local attempts="${4:-60}"
	local sleep_seconds="${5:-1}"
	local index=1
	while [ "$index" -le "$attempts" ]; do
		if ! kill -0 "$pid" >/dev/null 2>&1; then
			error "$label failed because the service process exited before readiness: pid=$pid"
			return 1
		fi
		if curl --silent --show-error --fail "$url" >/dev/null 2>&1; then
			return 0
		fi
		sleep "$sleep_seconds"
		index=$((index + 1))
	done
	error "$label did not become ready: $url"
	return 1
}
