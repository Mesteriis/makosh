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
ensure_postgres_client_dependencies
ensure_command dropdb
ensure_command createdb
postgres_up

if [ ! -d "$BACKUPS_ROOT" ]; then
	error "No backups directory found at $BACKUPS_ROOT"
	exit 1
fi

backup_dirs=()
while IFS= read -r backup_dir; do
	backup_dirs+=("$backup_dir")
done <<EOF
$(find "$BACKUPS_ROOT" -mindepth 2 -maxdepth 2 -type d | sort)
EOF

if [ "${#backup_dirs[@]}" -eq 0 ]; then
	error "No backups available under $BACKUPS_ROOT"
	exit 1
fi

printf '%s\n' "Available backups:"
select selected_backup in "${backup_dirs[@]}"; do
	if [ -n "${selected_backup:-}" ]; then
		break
	fi
	warn "Invalid selection."
done

postgres_dump="$selected_backup/postgres.sql"
vault_source="$selected_backup/vault"
mail_blobs_source="$selected_backup/mail-blobs"
mail_blobs_integrity_path="$selected_backup/mail-blobs.sha256"
manifest_path="$selected_backup/manifest.json"

if [ ! -f "$postgres_dump" ] || [ ! -f "$manifest_path" ] || [ ! -d "$vault_source" ] \
	|| [ ! -d "$mail_blobs_source" ] || [ ! -f "$mail_blobs_integrity_path" ]; then
	error "Backup is incomplete: required files are missing in $selected_backup"
	exit 1
fi

verify_directory_integrity_manifest "$mail_blobs_source" "$mail_blobs_integrity_path"

confirm_or_exit "Restore will replace database $MAKOSH_POSTGRES_DB, vault path $MAKOSH_HOST_VAULT_HOME, and mail blob path $MAIL_BLOB_ROOT." "RESTORE"

info "Recreating PostgreSQL database $MAKOSH_POSTGRES_DB"
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" psql \
	-h 127.0.0.1 \
	-p "$MAKOSH_POSTGRES_PORT" \
	-U "$MAKOSH_POSTGRES_USER" \
	-d postgres \
	-v ON_ERROR_STOP=1 \
	-c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$MAKOSH_POSTGRES_DB' AND pid <> pg_backend_pid();" >/dev/null
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" dropdb \
	--if-exists \
	-h 127.0.0.1 \
	-p "$MAKOSH_POSTGRES_PORT" \
	-U "$MAKOSH_POSTGRES_USER" \
	"$MAKOSH_POSTGRES_DB"
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" createdb \
	-h 127.0.0.1 \
	-p "$MAKOSH_POSTGRES_PORT" \
	-U "$MAKOSH_POSTGRES_USER" \
	"$MAKOSH_POSTGRES_DB"

info "Restoring PostgreSQL dump"
PGPASSWORD="$MAKOSH_POSTGRES_PASSWORD" psql \
	-h 127.0.0.1 \
	-p "$MAKOSH_POSTGRES_PORT" \
	-U "$MAKOSH_POSTGRES_USER" \
	-d "$MAKOSH_POSTGRES_DB" \
	-v ON_ERROR_STOP=1 \
	-f "$postgres_dump" >/dev/null

info "Restoring vault data"
rm -rf "$MAKOSH_HOST_VAULT_HOME"
mkdir -p "$MAKOSH_HOST_VAULT_HOME"
cp -R "$vault_source"/. "$MAKOSH_HOST_VAULT_HOME"/

# Materialize and verify the replacement before moving it into the live root.
# The previous root remains available until the staging tree is complete.
mail_blobs_staging="${MAIL_BLOB_ROOT}.restore.$$"
mail_blobs_previous="${MAIL_BLOB_ROOT}.previous.$$"
rm -rf "$mail_blobs_staging" "$mail_blobs_previous"
mkdir -p "$mail_blobs_staging"
cp -R "$mail_blobs_source"/. "$mail_blobs_staging"/
verify_directory_integrity_manifest "$mail_blobs_staging" "$mail_blobs_integrity_path"

info "Restoring mail blob storage"
mkdir -p "$(dirname "$MAIL_BLOB_ROOT")"
if [ -e "$MAIL_BLOB_ROOT" ]; then
	mv "$MAIL_BLOB_ROOT" "$mail_blobs_previous"
fi
if ! mv "$mail_blobs_staging" "$MAIL_BLOB_ROOT"; then
	if [ -e "$mail_blobs_previous" ]; then
		mv "$mail_blobs_previous" "$MAIL_BLOB_ROOT"
	fi
	error "Unable to replace mail blob storage; previous storage was restored."
	exit 1
fi
rm -rf "$mail_blobs_previous"

success "Restore completed from $selected_backup"
