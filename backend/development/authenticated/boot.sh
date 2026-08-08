#!/bin/sh
set -eu

readonly secret_path=/run/secrets/storage_pgbouncer_admin_password
readonly userlist_path=/etc/makosh/auth/users.txt

umask 077
mkdir -p /etc/makosh/auth
chmod 700 /etc/makosh/auth
password=$(cat "$secret_path")
printf '"makosh_pgbouncer_admin" "%s"\n' "$password" > "$userlist_path"
unset password

exec /entrypoint.sh "$@"
