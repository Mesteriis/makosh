#!/usr/bin/env bash

set -euo pipefail

MODE="old"
SESSION_ID=""
OLDER_THAN_SECONDS="${MAKOSH_TESTCONTAINERS_CLEANUP_MAX_AGE_SECS:-7200}"

usage() {
	cat <<'USAGE'
usage: clean-testcontainers.sh [--all | --session-id <id> | --older-than-seconds <seconds>]

Removes Макошь-owned testcontainers. By default, removes stopped Макошь test
containers and running Макошь test containers older than the age threshold.

Modern containers are matched by com.makosh.testkit=true. Legacy Макошь test
containers are matched by testcontainers' label plus the known pgvector/NATS
images used by the repository testkit.
USAGE
}

while [[ $# -gt 0 ]]; do
	case "$1" in
		--all)
			MODE="all"
			shift
			;;
		--session-id)
			MODE="session"
			SESSION_ID="${2:-}"
			if [[ -z "${SESSION_ID}" ]]; then
				echo "missing value for --session-id" >&2
				exit 2
			fi
			shift 2
			;;
		--older-than-seconds)
			OLDER_THAN_SECONDS="${2:-}"
			if [[ -z "${OLDER_THAN_SECONDS}" ]]; then
				echo "missing value for --older-than-seconds" >&2
				exit 2
			fi
			shift 2
			;;
		-h|--help)
			usage
			exit 0
			;;
		*)
			echo "unknown argument: $1" >&2
			usage >&2
			exit 2
			;;
	esac
done

if ! command -v docker >/dev/null 2>&1; then
	echo "docker is not available; skipping testcontainers cleanup" >&2
	exit 0
fi

if ! docker info >/dev/null 2>&1; then
	echo "docker daemon is not available; skipping testcontainers cleanup" >&2
	exit 0
fi

if ! [[ "${OLDER_THAN_SECONDS}" =~ ^[0-9]+$ ]]; then
	echo "--older-than-seconds must be a non-negative integer" >&2
	exit 2
fi

collect_ids() {
	if [[ "${MODE}" == "session" ]]; then
		docker ps -aq \
			--filter "label=com.makosh.testkit=true" \
			--filter "label=com.makosh.testkit.session=${SESSION_ID}"
		return
	fi

	{
		docker ps -aq --filter "label=com.makosh.testkit=true"
		docker ps -aq \
			--filter "label=org.testcontainers.managed-by=testcontainers" \
			--filter "ancestor=pgvector/pgvector:0.8.2-pg16"
		docker ps -aq \
			--filter "label=org.testcontainers.managed-by=testcontainers" \
			--filter "ancestor=nats:2.11-alpine"
	} | awk 'NF && !seen[$0]++'
}

is_older_than_threshold() {
	local container_id="$1"
	local created
	created="$(docker inspect --format '{{.Created}}' "${container_id}" 2>/dev/null || true)"
	if [[ -z "${created}" ]]; then
		return 1
	fi

	node -e '
const created = Date.parse(process.argv[1]);
const maxAgeMs = Number(process.argv[2]) * 1000;
if (!Number.isFinite(created) || !Number.isFinite(maxAgeMs)) process.exit(1);
process.exit(Date.now() - created >= maxAgeMs ? 0 : 1);
' "${created}" "${OLDER_THAN_SECONDS}"
}

ids="$(collect_ids || true)"
if [[ -z "${ids}" ]]; then
	echo "no Макошь testcontainers found"
	exit 0
fi

to_remove=()
while IFS= read -r container_id; do
	[[ -z "${container_id}" ]] && continue

	running="$(docker inspect --format '{{.State.Running}}' "${container_id}" 2>/dev/null || true)"
	if [[ "${MODE}" == "all" || "${MODE}" == "session" ]]; then
		to_remove+=("${container_id}")
	elif [[ "${running}" != "true" ]]; then
		to_remove+=("${container_id}")
	elif is_older_than_threshold "${container_id}"; then
		to_remove+=("${container_id}")
	fi
done <<< "${ids}"

if [[ "${#to_remove[@]}" -eq 0 ]]; then
	echo "no eligible Макошь testcontainers found"
	exit 0
fi

docker rm -f "${to_remove[@]}" >/dev/null
printf 'removed %s Макошь testcontainer(s)\n' "${#to_remove[@]}"
