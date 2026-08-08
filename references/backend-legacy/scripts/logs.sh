#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

current_log_dir="$LOG_ROOT/current"
live_log="$current_log_dir/live.log"
follow_logs="${MAKOSH_LOGS_FOLLOW:-1}"

if [ ! -L "$current_log_dir" ] && [ ! -d "$current_log_dir" ]; then
	error "No active dev log session found at $current_log_dir. Run make dev first."
	exit 1
fi

if [ ! -f "$live_log" ]; then
	error "Live log file not found: $live_log"
	exit 1
fi

info "Streaming $live_log"
if [ "$follow_logs" = "0" ]; then
	tail -n 50 "$live_log"
else
	tail -n 50 -f "$live_log"
fi
