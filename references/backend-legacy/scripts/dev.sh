#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/env.sh
source "$SCRIPT_DIR/lib/env.sh"
# shellcheck source=./lib/postgres.sh
source "$SCRIPT_DIR/lib/postgres.sh"

DESKTOP_MODE=0
for arg in "$@"; do
	case "$arg" in
		--desktop)
			DESKTOP_MODE=1
			;;
		*)
			error "Unknown argument: $arg (supported: --desktop)"
			exit 1
			;;
	esac
done

load_makosh_env
ensure_frontend_dependencies
ensure_bacon_available
ensure_command cargo
ensure_command curl
postgres_up
nats_up
clamav_up

reclaim_dev_port "$MAKOSH_BACKEND_PORT" "Backend"
reclaim_dev_port "$MAKOSH_FRONTEND_PORT" "Frontend"

ensure_dir "$LOG_ROOT"
flow_id="dev-$(timestamp_compact_utc)-$$"
session_log="$LOG_ROOT/$flow_id"
ensure_dir "$session_log"
live_log="$session_log/live.log"
current_log_link="$LOG_ROOT/current"
rm -f "$current_log_link"
ln -s "$session_log" "$current_log_link"

child_pids=()
pipe_paths=()
logger_pids=()
RUN_SERVICE_PID=""

cleanup() {
	local status="$?"
	local pid
	trap - EXIT INT TERM
	for pid in "${child_pids[@]:-}"; do
		kill "$pid" 2>/dev/null || true
	done
	for pid in "${child_pids[@]:-}"; do
		wait "$pid" 2>/dev/null || true
	done
	for pid in "${logger_pids[@]:-}"; do
		kill "$pid" 2>/dev/null || true
	done
	for pid in "${logger_pids[@]:-}"; do
		wait "$pid" 2>/dev/null || true
	done
	local pipe_path
	for pipe_path in "${pipe_paths[@]:-}"; do
		rm -f "$pipe_path"
	done
	exit "$status"
}
trap cleanup EXIT INT TERM

run_service() {
	local service="$1"
	local color="$2"
	shift 2
	local stdout_pipe="$session_log/$service.stdout.pipe"
	local stderr_pipe="$session_log/$service.stderr.pipe"
	local log_file="$session_log/$service.jsonl"
	mkfifo "$stdout_pipe" "$stderr_pipe"
	pipe_paths+=("$stdout_pipe" "$stderr_pipe")

	"$@" >"$stdout_pipe" 2>"$stderr_pipe" &
	local service_pid="$!"
	child_pids+=("$service_pid")

	stream_service_pipe "$stdout_pipe" "$service" "$service_pid" "info" "$flow_id" "$color" "$log_file" "$live_log" &
	logger_pids+=("$!")
	stream_service_pipe "$stderr_pipe" "$service" "$service_pid" "warn" "$flow_id" "$color" "$log_file" "$live_log" &
	logger_pids+=("$!")

	RUN_SERVICE_PID="$service_pid"
}

export DATABASE_URL
export MAKOSH_LOCAL_API_SECRET
export MAKOSH_DEV_MODE
export MAKOSH_VAULT_HOME
export MAKOSH_DEV_KEY_PATH
export MAKOSH_SECRET_VAULT_KEY
export MAKOSH_HTTP_ADDR="$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT"
export MAKOSH_FLOW_ID="$flow_id"
export MAKOSH_LOG_FORMAT="json"
export RUST_LOG="${RUST_LOG:-info}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CARGO_DEV_TARGET_DIR}"
export VITE_MAKOSH_API_BASE_URL="http://$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT"
export VITE_MAKOSH_LOCAL_API_SECRET="$MAKOSH_LOCAL_API_SECRET"

run_service backend "$color_cyan" bash -lc "cd '$REPO_ROOT' && exec bacon --headless backend-dev"
backend_pid="$RUN_SERVICE_PID"
info "Waiting for backend health check"
wait_for_service_http "$backend_pid" "http://$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT/healthz" "Backend healthz" "$MAKOSH_BACKEND_STARTUP_ATTEMPTS" "$MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS"
info "Waiting for backend readiness check"
wait_for_service_http "$backend_pid" "http://$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT/readyz" "Backend readyz" "$MAKOSH_BACKEND_STARTUP_ATTEMPTS" "$MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS"

run_service frontend "$color_green" bash -lc "cd '$REPO_ROOT/frontend' && exec pnpm dev --host '$MAKOSH_FRONTEND_BIND' --port '$MAKOSH_FRONTEND_PORT' --strictPort"
frontend_pid="$RUN_SERVICE_PID"
info "Waiting for frontend dev server"
wait_for_service_http "$frontend_pid" "http://$MAKOSH_FRONTEND_BIND:$MAKOSH_FRONTEND_PORT" "Frontend Vite" "$MAKOSH_FRONTEND_STARTUP_ATTEMPTS" "$MAKOSH_FRONTEND_STARTUP_SLEEP_SECONDS"

tauri_pid=""
if [ "$DESKTOP_MODE" = "1" ]; then
	# Point the Tauri dev window at the Vite server this script started and
	# keep the sidecar disabled; the backend already runs via bacon above.
	tauri_dev_config="{\"build\":{\"devUrl\":\"http://$MAKOSH_FRONTEND_BIND:$MAKOSH_FRONTEND_PORT\"}}"
	export MAKOSH_DISABLE_BACKEND_SIDECAR=1
	run_service tauri "$color_yellow" bash -lc "cd '$REPO_ROOT/frontend' && exec pnpm tauri dev --config '$tauri_dev_config'"
	tauri_pid="$RUN_SERVICE_PID"
	info "Tauri desktop shell is compiling; the app window opens when the build finishes"
fi

info "Flow ID: $flow_id"
info "Logs: $session_log"
info "Live log: $current_log_link/live.log"
printf '%s\n' "PostgreSQL:"
postgres_status
printf '%s\n' "NATS:"
nats_status
printf '%s\n' "ClamAV:"
clamav_status
printf '%s\n' "Backend:  http://$MAKOSH_BACKEND_BIND:$MAKOSH_BACKEND_PORT (pid $backend_pid)"
printf '%s\n' "Frontend: http://$MAKOSH_FRONTEND_BIND:$MAKOSH_FRONTEND_PORT (pid $frontend_pid)"
if [ -n "$tauri_pid" ]; then
	printf '%s\n' "Tauri:    desktop shell (pid $tauri_pid)"
fi

if [ -n "$tauri_pid" ]; then
	wait "$backend_pid" "$frontend_pid" "$tauri_pid"
else
	wait "$backend_pid" "$frontend_pid"
fi
