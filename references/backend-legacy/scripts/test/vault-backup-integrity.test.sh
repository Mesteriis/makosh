#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
# shellcheck source=../lib/common.sh
source "$REPO_ROOT/scripts/lib/common.sh"

temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/makosh-vault-backup-test.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT

expect_failure() {
	local label="$1"
	shift
	if "$@" >/dev/null 2>&1; then
		error "Expected failure: $label"
		return 1
	fi
}

source_root="$temporary_root/source"
restore_root="$temporary_root/restore"
manifest_path="$temporary_root/mail-blobs.sha256"
mkdir -p "$source_root/nested" "$restore_root"
printf 'mail blob bytes' >"$source_root/nested/blob"

IFS=$'\t' read -r file_count total_bytes < <(
	write_directory_integrity_manifest "$source_root" "$manifest_path"
)
test "$file_count" = "1"
test "$total_bytes" = "15"
verify_directory_integrity_manifest "$source_root" "$manifest_path"

cp -R "$source_root"/. "$restore_root"/
verify_directory_integrity_manifest "$restore_root" "$manifest_path"

printf 'tampered content' >"$restore_root/nested/blob"
expect_failure "changed file" verify_directory_integrity_manifest "$restore_root" "$manifest_path"

rm -rf "$restore_root"
mkdir -p "$restore_root"
cp -R "$source_root"/. "$restore_root"/
rm "$restore_root/nested/blob"
expect_failure "missing file" verify_directory_integrity_manifest "$restore_root" "$manifest_path"

cp -R "$source_root"/. "$restore_root"/
printf 'unexpected' >"$restore_root/unexpected"
expect_failure "added file" verify_directory_integrity_manifest "$restore_root" "$manifest_path"

rm "$restore_root/unexpected"
ln -s "nested/blob" "$restore_root/link"
expect_failure "symlink" verify_directory_integrity_manifest "$restore_root" "$manifest_path"

success "Vault backup mail-blob integrity checks passed"
