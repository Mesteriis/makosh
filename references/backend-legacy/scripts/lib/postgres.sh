#!/usr/bin/env bash

set -euo pipefail

# shellcheck source=./common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"
# shellcheck source=./env.sh
source "$COMMON_DIR/env.sh"

compose_cmd() {
	docker compose \
		--env-file "$DOCKER_ENV_FILE" \
		--project-directory "$REPO_ROOT/docker" \
		-f "$REPO_ROOT/docker/docker-compose.yml" \
		"$@"
}

ensure_postgres_runtime_dependencies() {
	ensure_command docker
}

ensure_postgres_client_dependencies() {
	ensure_postgres_runtime_dependencies
	ensure_command psql
}

postgres_up() {
	load_makosh_env
	ensure_postgres_runtime_dependencies
	info "Starting PostgreSQL container"
	compose_cmd up -d --wait postgres
	wait_for_postgres
}

nats_up() {
	load_makosh_env
	ensure_postgres_runtime_dependencies
	info "Starting NATS container"
	compose_cmd up -d --wait nats
}

clamav_up() {
	load_makosh_env
	ensure_postgres_runtime_dependencies
	info "Starting ClamAV container"
	compose_cmd up -d --wait clamav
}

wait_for_postgres() {
	local attempts=60
	local index=1
	while [ "$index" -le "$attempts" ]; do
		if compose_cmd exec -T postgres sh -lc \
			'pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"' >/dev/null 2>&1; then
			return 0
		fi
		sleep 1
		index=$((index + 1))
	done
	error "PostgreSQL did not become ready on 127.0.0.1:$MAKOSH_POSTGRES_PORT"
	exit 1
}

postgres_status() {
	load_makosh_env
	compose_cmd ps postgres
}

nats_status() {
	load_makosh_env
	compose_cmd ps nats
}

clamav_status() {
	load_makosh_env
	compose_cmd ps clamav
}

postgres_stop() {
	load_makosh_env
	compose_cmd stop postgres
}

postgres_data_dir() {
	printf '%s\n' "$REPO_ROOT/docker/data/postgres"
}
