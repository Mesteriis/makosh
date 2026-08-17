#!/usr/bin/env bash

set -euo pipefail

backend_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
project_root="$(cd "$backend_root/.." && pwd)"
frontend_root="$project_root/frontend"
compose_file="$backend_root/development/authenticated/compose.yaml"
legacy_compose_file="$backend_root/development/compose.yaml"
authenticated_compose_project_name="${MAKOSH_DEV_AUTHENTICATED_COMPOSE_PROJECT_NAME:-makosh-storage-authenticated-development}"
legacy_compose_project_name="makosh-platform-development"
gateway_address="127.0.0.1:9444"
gateway_target="http://$gateway_address"
owner_vault_host_address="127.0.0.1:9445"
owner_vault_host_target="http://$owner_vault_host_address"
browser_origin="http://127.0.0.1:5173"
browser_url="$browser_origin/"
no_browser="${MAKOSH_DEV_NO_BROWSER:-1}"
data_dir="${MAKOSH_DEV_DATA_DIR:-$project_root/.local/kernel-dev}"
cargo_target_dir="${MAKOSH_DEV_CARGO_TARGET_DIR:-$backend_root/target}"
release_root="${MAKOSH_DEV_RELEASE_ROOT:-$project_root/.local/dev-release}"
distribution_id="makosh-local-development"
generation_metadata_name="development-distribution-generation"
startup_timeout_seconds="${MAKOSH_DEV_STARTUP_TIMEOUT_SECONDS:-120}"
rust_toolchain="${RUST_TOOLCHAIN:-1.97.0}"
legacy_recovery_bundle_root="${MAKOSH_LEGACY_PROVIDER_RECOVERY_BUNDLE_ROOT:-}"
legacy_recovery_frontend_flag=0
telegram_credentials_environment_file="${MAKOSH_DEV_TELEGRAM_CREDENTIALS_FILE:-}"
google_oauth_client_config_file="${MAKOSH_DEV_GOOGLE_OAUTH_CLIENT_CONFIG_PATH:-}"
google_oauth_client_id=""
kernel_pid=""
owner_vault_host_pid=""
frontend_pid=""
temporary_dir=""
proof_file=""

fail() {
	printf 'Макошь development assembly failed: %s\n' "$1" >&2
	exit 1
}

require_command() {
	command -v "$1" >/dev/null 2>&1 || fail "required command '$1' is unavailable"
}

require_absolute_directory_path() {
	case "$2" in
		/*) ;;
		*) fail "$1 must be an absolute path" ;;
	esac
}

require_available_port() {
	if lsof -nP -iTCP:"$1" -sTCP:LISTEN >/dev/null 2>&1; then
		fail "loopback port $1 is already in use"
	fi
}

if test -n "$legacy_recovery_bundle_root"; then
	require_absolute_directory_path \
		"legacy provider recovery bundle root" \
		"$legacy_recovery_bundle_root"
	test -d "$legacy_recovery_bundle_root" && test ! -L "$legacy_recovery_bundle_root" \
		|| fail "legacy provider recovery bundle root is unavailable"
	legacy_recovery_frontend_flag=1
fi

if test -z "$telegram_credentials_environment_file" && test -f "$project_root/.env"; then
	telegram_credentials_environment_file="$project_root/.env"
fi
if test -n "$telegram_credentials_environment_file"; then
	require_absolute_directory_path \
		"development Telegram credentials file" \
		"$telegram_credentials_environment_file"
	test -f "$telegram_credentials_environment_file" \
		&& test ! -L "$telegram_credentials_environment_file" \
		|| fail "development Telegram credentials file is unavailable"
	case "$(stat -f '%Lp' "$telegram_credentials_environment_file")" in
		*00) ;;
		*) fail "development Telegram credentials file permissions must be owner-only" ;;
	esac
fi

run_compose() {
	env \
		MAKOSH_STORAGE_POSTGRES_SECRET_FILE="$data_dir/developer-platform-credentials/postgres-admin-password" \
		MAKOSH_STORAGE_PGBOUNCER_SECRET_FILE="$data_dir/developer-platform-credentials/pgbouncer-admin-password" \
		MAKOSH_STORAGE_PGBOUNCER_DATABASES_DIRECTORY="$runtime_dir/storage/pgbouncer" \
		MAKOSH_STORAGE_PGBOUNCER_AUTH_DIRECTORY="$runtime_dir/storage/pgbouncer/auth" \
		MAKOSH_STORAGE_PGBOUNCER_RUNTIME_UID="$(id -u)" \
		docker compose --project-name "$authenticated_compose_project_name" -f "$compose_file" "$@"
}

cleanup() {
	status=$?
	trap - EXIT INT TERM HUP
	if test -n "$frontend_pid"; then
		kill -TERM "$frontend_pid" 2>/dev/null || true
	fi
	if test -n "$owner_vault_host_pid"; then
		kill -TERM "$owner_vault_host_pid" 2>/dev/null || true
	fi
	if test -n "$kernel_pid"; then
		kill -TERM "$kernel_pid" 2>/dev/null || true
	fi
	attempt=0
	while test "$attempt" -lt 50; do
		frontend_alive=false
		owner_vault_host_alive=false
		kernel_alive=false
		if test -n "$frontend_pid" && kill -0 "$frontend_pid" 2>/dev/null; then
			frontend_alive=true
		fi
		if test -n "$kernel_pid" && kill -0 "$kernel_pid" 2>/dev/null; then
			kernel_alive=true
		fi
		if test -n "$owner_vault_host_pid" && kill -0 "$owner_vault_host_pid" 2>/dev/null; then
			owner_vault_host_alive=true
		fi
		if test "$frontend_alive" = false \
			&& test "$owner_vault_host_alive" = false \
			&& test "$kernel_alive" = false; then
			break
		fi
		attempt=$((attempt + 1))
		sleep 0.1
	done
	if test -n "$frontend_pid" && kill -0 "$frontend_pid" 2>/dev/null; then
		kill -KILL "$frontend_pid" 2>/dev/null || true
	fi
	if test -n "$kernel_pid" && kill -0 "$kernel_pid" 2>/dev/null; then
		kill -KILL "$kernel_pid" 2>/dev/null || true
	fi
	if test -n "$owner_vault_host_pid" && kill -0 "$owner_vault_host_pid" 2>/dev/null; then
		kill -KILL "$owner_vault_host_pid" 2>/dev/null || true
	fi
	if test -n "$frontend_pid"; then
		wait "$frontend_pid" 2>/dev/null || true
	fi
	if test -n "$kernel_pid"; then
		wait "$kernel_pid" 2>/dev/null || true
	fi
	if test -n "$owner_vault_host_pid"; then
		wait "$owner_vault_host_pid" 2>/dev/null || true
	fi
	if test -n "$proof_file"; then
		rm -f -- "$proof_file"
	fi
	if test -n "$temporary_dir"; then
		rmdir -- "$temporary_dir" 2>/dev/null || true
	fi
	exit "$status"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

for command_name in cargo curl docker id lsof make mktemp node pnpm; do
	require_command "$command_name"
done
if test -n "$google_oauth_client_config_file"; then
	require_absolute_directory_path \
		"development Google OAuth client configuration" \
		"$google_oauth_client_config_file"
	google_oauth_client_id="$(
		node "$backend_root/scripts/read-dev-google-oauth-client-id.mjs" \
			--config-file "$google_oauth_client_config_file"
	)" || fail "development Google OAuth client configuration is invalid"
elif test -f "$project_root/.env"; then
	google_oauth_client_id="$(
		node "$backend_root/scripts/read-dev-google-oauth-client-id.mjs" \
			--env-file "$project_root/.env"
	)" || fail "development Google OAuth client configuration is invalid"
fi
if test -n "$google_oauth_client_id"; then
	printf '%s\n' 'Detected the installed Google OAuth client for Gmail development setup.'
fi
case "$no_browser" in
	0) require_command open ;;
	1) ;;
	*) fail "MAKOSH_DEV_NO_BROWSER must be 0 or 1" ;;
esac
require_absolute_directory_path "MAKOSH_DEV_DATA_DIR" "$data_dir"
require_absolute_directory_path "MAKOSH_DEV_CARGO_TARGET_DIR" "$cargo_target_dir"
require_absolute_directory_path "MAKOSH_DEV_RELEASE_ROOT" "$release_root"
case "$startup_timeout_seconds" in
	''|*[!0-9]*) fail "MAKOSH_DEV_STARTUP_TIMEOUT_SECONDS must be a positive integer" ;;
esac
test "$startup_timeout_seconds" -gt 0 || fail "MAKOSH_DEV_STARTUP_TIMEOUT_SECONDS must be positive"

require_available_port 5173
require_available_port 9444
require_available_port 9445

printf '%s\n' 'Materializing the signed clean-room development release...'
MAKOSH_DEV_CARGO_TARGET_DIR="$cargo_target_dir" \
	"$backend_root/scripts/materialize-dev-release.sh"
kernel_bin="$release_root/МакошьDev.app/Contents/MacOS/makosh-kernel"
generation_metadata="$release_root/$generation_metadata_name"
development_assembly_bin="$cargo_target_dir/debug/makosh-development-assembly"
owner_vault_host_bin="$cargo_target_dir/debug/makosh-owner-vault-development-host"
test -x "$kernel_bin" || fail "signed Kernel development binary is unavailable"
test -x "$development_assembly_bin" || fail "development assembly unit is unavailable"
test -f "$generation_metadata" && test ! -L "$generation_metadata" \
	|| fail "development release generation metadata is unavailable"
test "$(stat -f '%Lp' "$generation_metadata")" = "600" \
	|| fail "development release generation metadata permissions must be 0600"
distribution_generation="$(sed -n '1p' "$generation_metadata")"
test "$(wc -l <"$generation_metadata" | tr -d ' ')" = "1" \
	|| fail "development release generation metadata is invalid"
case "$distribution_generation" in
	''|*[!0-9]*) fail "development release generation metadata is invalid" ;;
esac
test "$distribution_generation" -gt 0 \
	|| fail "development release generation metadata is invalid"

printf '%s\n' 'Building the loopback Owner Vault provisioning host...'
cargo +"$rust_toolchain" build \
	--locked \
	--manifest-path "$frontend_root/native/owner-vault-provisioning-host/Cargo.toml" \
	--features development-server \
	--bin makosh-owner-vault-development-host \
	--target-dir "$cargo_target_dir"
test -x "$owner_vault_host_bin" || fail "development Owner Vault host is unavailable"

status_output="$("$kernel_bin" --data-dir "$data_dir" status)"
owner_identity="$(printf '%s\n' "$status_output" | sed -n 's/^owner_identity=//p')"
owner_device_signer="$(printf '%s\n' "$status_output" | sed -n 's/^owner_device_signer=//p')"
case "$owner_identity:$owner_device_signer" in
	missing:missing)
		"$kernel_bin" --data-dir "$data_dir" device-key-generate
		"$kernel_bin" --data-dir "$data_dir" initial-owner-enroll \
			--owner-id development-owner \
			--device-id development-desktop
		;;
	missing:ready)
		"$kernel_bin" --data-dir "$data_dir" initial-owner-enroll \
			--owner-id development-owner \
			--device-id development-desktop
		;;
	enrolled:ready) ;;
	enrolled:missing|enrolled:mismatch|enrolled:unavailable)
		fail "the enrolled development owner signer is unavailable or does not match"
		;;
	*)
		fail "development owner identity state is unavailable"
		;;
esac

status_output="$("$kernel_bin" --data-dir "$data_dir" status)"
printf '%s\n' "$status_output" | grep -qx 'owner_identity=enrolled' \
	|| fail "development owner enrollment did not become ready"
printf '%s\n' "$status_output" | grep -qx 'owner_device_signer=ready' \
	|| fail "development owner signer did not become ready"

"$development_assembly_bin" \
	--data-dir "$data_dir" \
	provision-platform
runtime_dir="$("$development_assembly_bin" --data-dir "$data_dir" runtime-directory)"
require_absolute_directory_path "development runtime directory" "$runtime_dir"

printf '%s\n' 'Starting authenticated PostgreSQL, PgBouncer, NATS and ClamAV infrastructure...'
if test -n "$(docker compose --project-name "$legacy_compose_project_name" -f "$legacy_compose_file" ps --all --quiet 2>/dev/null)"; then
	docker compose --project-name "$legacy_compose_project_name" -f "$legacy_compose_file" down --remove-orphans >/dev/null 2>&1 || true
fi
run_compose up --detach --wait
expected_postgres_hash="$(
	tr -d '\r\n' < "$data_dir/developer-platform-credentials/postgres-admin-password" \
		| sha256sum \
		| awk '{print $1}'
)"
current_postgres_hash="$(
	run_compose exec --no-TTY postgres sh -c "cat /run/secrets/storage_postgres_admin_password | tr -d '\r\n' | sha256sum | awk '{print \$1}'" \
		|| true
)"
if ! test "$expected_postgres_hash" = "$current_postgres_hash"; then
	run_compose up --detach --no-deps --force-recreate --wait postgres
fi

# PostgreSQL applies POSTGRES_PASSWORD_FILE only when its data directory is
# created. Reconcile the fixed development admin role on every restart so a
# preserved volume and the file-backed Vault bootstrap cannot silently drift.
# The SQL reads the Docker secret inside the container; no credential is put in
# host arguments, environment variables or logs.
run_compose exec --no-TTY --user postgres postgres \
	psql --no-psqlrc --set=ON_ERROR_STOP=1 --quiet \
	--username makosh_postgres_admin \
	--dbname makosh_storage_authenticated \
	--file - \
	< "$backend_root/development/authenticated/reconcile-postgres-admin.sql" \
	>/dev/null \
	|| fail "development PostgreSQL admin reconciliation failed"
# A preserved provider database can outlive a development Storage instance
# identity cutover. Advance only the owner-scope role prefix for the exact
# enrolled development owner; provider data and human ownership are unchanged.
run_compose exec --no-TTY --user postgres postgres \
	psql --no-psqlrc --set=ON_ERROR_STOP=1 --quiet \
	--username makosh_postgres_admin \
	--dbname makosh_storage_authenticated \
	--file - \
	< "$backend_root/development/authenticated/reconcile-provider-owner-scopes.sql" \
	>/dev/null \
	|| fail "development provider owner-scope reconciliation failed"
# PgBouncer is stateless and binds the OS-cache runtime directory. Docker
# Desktop can retain a mount to a removed/recreated cache-directory inode while
# keeping the long-lived data services alive. Recreate only a pooler whose
# current mount cannot resolve the expected configuration and runtime files;
# PostgreSQL, NATS and ClamAV state stay untouched. A healthy current mount
# must remain running because its boot script intentionally initializes the
# admin-only auth file.
if ! run_compose exec --no-TTY pgbouncer \
	test -r /etc/pgbouncer/pgbouncer.ini \
		-a -r /etc/makosh/runtime/databases.ini \
		-a -r /etc/makosh/auth/users.txt; then
	run_compose up --detach --no-deps --force-recreate --wait pgbouncer
else
	expected_pgbouncer_hash="$(
		tr -d '\r\n' < "$data_dir/developer-platform-credentials/pgbouncer-admin-password" \
			| sha256sum \
			| awk '{print $1}'
	)"
	current_pgbouncer_hash="$(
		run_compose exec --no-TTY pgbouncer sh -c \
			"sed -n '1p' /etc/makosh/auth/users.txt | sed -E 's/\\\"[^\\\"]*\\\" \\\"([^\\\"]*)\\\"/\\1/' | tr -d '\r\n' | sha256sum | awk '{print \$1}'"
	)"
	if ! test "$expected_pgbouncer_hash" = "$current_pgbouncer_hash"; then
		run_compose up --detach --no-deps --force-recreate --wait pgbouncer
	fi
fi

temporary_dir="$(mktemp -d "${TMPDIR:-/tmp}/makosh-dev-assembly.XXXXXX")"
chmod 700 "$temporary_dir"
proof_file="$temporary_dir/gateway-proof"
node -e \
	'const fs = require("node:fs"); const crypto = require("node:crypto"); fs.writeFileSync(process.argv[1], crypto.randomBytes(32).toString("hex"), { encoding: "utf8", flag: "wx", mode: 0o600 });' \
	"$proof_file"

printf '%s\n' 'Starting the loopback Owner Vault provisioning host...'
owner_vault_host_args=(
	--listen-address "$owner_vault_host_address"
	--proof-file "$proof_file"
	--owner-device-key-file "$data_dir/device-es256.key"
)
if test -n "$telegram_credentials_environment_file"; then
	owner_vault_host_args+=(
		--telegram-credentials-env-file "$telegram_credentials_environment_file"
	)
fi
if test "$legacy_recovery_frontend_flag" = 1; then
	legacy_recovery_receipt_dir="$data_dir/maintenance/legacy-provider-recovery-v1"
	mkdir -p -- "$legacy_recovery_receipt_dir"
	chmod 700 "$data_dir/maintenance" "$legacy_recovery_receipt_dir"
	legacy_recovery_receipt_file="$legacy_recovery_receipt_dir/receipt.v1.json"
	owner_vault_host_args+=(
		--legacy-recovery-bundle-root "$legacy_recovery_bundle_root"
		--legacy-recovery-receipt-file "$legacy_recovery_receipt_file"
	)
fi
"$owner_vault_host_bin" "${owner_vault_host_args[@]}" &
owner_vault_host_pid=$!

deadline=$(( $(date +%s) + startup_timeout_seconds ))
while :; do
	kill -0 "$owner_vault_host_pid" 2>/dev/null \
		|| fail "development Owner Vault host exited before readiness"
	if node "$backend_root/scripts/probe-dev-owner-vault-host.mjs" "$proof_file"; then
		break
	fi
	test "$(date +%s)" -lt "$deadline" \
		|| fail "development Owner Vault host readiness deadline expired"
	sleep 1
done
if test "$legacy_recovery_frontend_flag" = 1; then
	node "$backend_root/scripts/probe-dev-legacy-provider-recovery.mjs" "$proof_file" \
		|| fail "legacy provider recovery host readiness probe failed"
fi

printf '%s\n' 'Starting Макошь Kernel and loopback Core Gateway...'
start_kernel() {
	env MAKOSH_DEVELOPER_VERBOSE=1 "$kernel_bin" \
		--data-dir "$data_dir" \
		serve \
		--browser-gateway-listen-address "$gateway_address" \
		--browser-gateway-origin "$browser_origin" \
		--browser-gateway-rp-id 127.0.0.1 \
		--browser-gateway-development-proxy-proof-file "$proof_file" &
	kernel_pid=$!
}

start_kernel_for_admission() {
	env MAKOSH_DEVELOPER_VERBOSE=1 "$kernel_bin" \
		--data-dir "$data_dir" \
		serve \
		--browser-gateway-listen-address "$gateway_address" \
		--browser-gateway-origin "$browser_origin" \
		--browser-gateway-rp-id 127.0.0.1 \
		--browser-gateway-development-proxy-proof-file "$proof_file" \
		--browser-gateway-development-admission-mode &
	kernel_pid=$!
}

wait_for_gateway() {
	deadline=$(( $(date +%s) + startup_timeout_seconds ))
	while :; do
		kill -0 "$kernel_pid" 2>/dev/null || fail "Kernel exited before readiness"
		if node "$backend_root/scripts/probe-dev-gateway.mjs" "$proof_file"; then
			break
		fi
		test "$(date +%s)" -lt "$deadline" || fail "Kernel readiness deadline expired"
		sleep 1
	done
}

stop_kernel() {
	kill -TERM "$kernel_pid" 2>/dev/null || true
	wait "$kernel_pid" || true
	kernel_pid=""
}

assembly_status="$(
	"$development_assembly_bin" \
		--data-dir "$data_dir" \
		--distribution-id "$distribution_id" \
		--distribution-generation "$distribution_generation" \
		status
)"
case "$assembly_status" in
	development_assembly=missing)
	printf '%s\n' 'Admitting the exact clean-room development module plan...'
	;;
	development_assembly=stale)
	printf '%s\n' 'Refreshing the exact clean-room development module plan...'
	;;
	development_assembly=current) ;;
	*) fail "development assembly state is unavailable" ;;
esac
if test "$assembly_status" != "development_assembly=current"; then
	# Admission needs the authenticated Vault/Storage foundation to fence stale
	# bindings, but it must not run Scheduler against each intermediate catalog.
	# The full Kernel starts once from the completed catalog below.
	start_kernel_for_admission
	wait_for_gateway
	reconcile_output="$(
		"$development_assembly_bin" \
			--data-dir "$data_dir" \
			--distribution-id "$distribution_id" \
			--distribution-generation "$distribution_generation" \
			admit
	)"
	case "$reconcile_output" in
		development_assembly=admitted|development_assembly=updated) ;;
		*) fail "development assembly reconciliation did not complete" ;;
	esac
	stop_kernel
fi

start_kernel
wait_for_gateway

"$development_assembly_bin" \
	--data-dir "$data_dir" \
	--distribution-id "$distribution_id" \
	--distribution-generation "$distribution_generation" \
	start-ensemble

printf '%s\n' 'Starting the Vue/Vite browser client...'
(
	cd "$frontend_root"
	exec env MAKOSH_DEV_GATEWAY_TARGET="$gateway_target" \
		MAKOSH_DEV_GATEWAY_PROOF_FILE="$proof_file" \
		MAKOSH_DEV_OWNER_VAULT_HOST_TARGET="$owner_vault_host_target" \
		VITE_MAKOSH_DEV_OWNER_VAULT_HOST=1 \
		VITE_MAKOSH_DEV_OWNER_DEVICE_PROOF_HOST=1 \
		VITE_MAKOSH_GMAIL_OAUTH_CLIENT_ID="$google_oauth_client_id" \
		VITE_MAKOSH_LEGACY_PROVIDER_RECOVERY="$legacy_recovery_frontend_flag" \
		pnpm exec vite --host 127.0.0.1 --strictPort
) &
frontend_pid=$!

deadline=$(( $(date +%s) + startup_timeout_seconds ))
while :; do
	kill -0 "$kernel_pid" 2>/dev/null || fail "Kernel exited before browser readiness"
	kill -0 "$owner_vault_host_pid" 2>/dev/null \
		|| fail "development Owner Vault host exited before browser readiness"
	kill -0 "$frontend_pid" 2>/dev/null || fail "Vite exited before readiness"
	if curl --fail --silent --show-error --max-time 2 "$browser_origin/readyz" >/dev/null; then
		break
	fi
	test "$(date +%s)" -lt "$deadline" || fail "browser readiness deadline expired"
	sleep 1
done

printf 'Макошь development ensemble is ready at %s\n' "$browser_url"
if test "$no_browser" = 0; then
	open "$browser_url"
	printf '%s\n' 'Browser opened. Press Ctrl-C to stop the full local ensemble.'
else
	printf 'Browser opening skipped. Open %s in an existing browser tab. Press Ctrl-C to stop the full local ensemble.\n' "$browser_url"
fi

while kill -0 "$kernel_pid" 2>/dev/null \
	&& kill -0 "$owner_vault_host_pid" 2>/dev/null \
	&& kill -0 "$frontend_pid" 2>/dev/null; do
	sleep 1
done
if ! kill -0 "$kernel_pid" 2>/dev/null; then
	wait "$kernel_pid" || child_status=$?
	fail "Kernel stopped unexpectedly with status ${child_status:-0}"
fi
if ! kill -0 "$owner_vault_host_pid" 2>/dev/null; then
	wait "$owner_vault_host_pid" || child_status=$?
	fail "development Owner Vault host stopped unexpectedly with status ${child_status:-0}"
fi
wait "$frontend_pid" || child_status=$?
fail "Vite stopped unexpectedly with status ${child_status:-0}"
