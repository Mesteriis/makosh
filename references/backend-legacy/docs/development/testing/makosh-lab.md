# Макошь Lab

Status: proposed.

Макошь Lab is the system-level E2E harness for following real external signals
through Макошь. Its first reference provider is Zulip.

The goal is not to prove one handler works. The goal is to trace a signal across
the whole system:

```text
provider action
↓
integration observation
↓
canonical event envelope
↓
Communications materialization
↓
Signal Hub / Review / Timeline / Search / Memory
↓
UI/debug surface
```

## Why a lab exists

Provider integrations are easy to lie about. A mocked message can make a unit
test pass while the real provider queue, payload shape, runtime credentials and
projection chain remain unverified. Макошь Lab gives the project one repeatable
place to generate external events and inspect their path.

## First provider: Zulip

Zulip is used because it can be self-hosted and controlled inside a local test
session. Макошь has two Zulip harness layers:

- the Rust `zulip_live` testcontainers fixture, which uses a minimal Compose
  stack aligned with the official Zulip Docker service shape and images;
- the Макошь Lab CLI, which is the broader operator-facing harness and can use
  an official docker-zulip checkout for local lab operations.

Both layers must use real Zulip services, not a fake HTTP server standing in for
the messenger.

## Scenario file shape

Scenario files live under:

```text
testing/makosh-lab/scenarios/<provider>/*.json
```

Minimal shape:

```json
{
  "scenario_id": "zulip_message_to_task_candidate",
  "provider": "zulip",
  "description": "Send a task-like message and trace it through Макошь.",
  "correlation_id_prefix": "lab-zulip-task",
  "actions": [
    {
      "kind": "send_stream_message",
      "stream": "Макошь Lab",
      "topic": "Tasks",
      "content": "Надо проверить backup retention до пятницы."
    },
    {
      "kind": "add_reaction",
      "message_id_ref": "last",
      "emoji_name": "thumbs_up",
      "emoji_code": "1f44d",
      "reaction_type": "unicode_emoji"
    },
    {
      "kind": "update_message",
      "message_id_ref": "last",
      "updated_content": "Надо проверить backup retention и назначить owner до пятницы."
    },
    {
      "kind": "send_stream_message_with_upload",
      "stream": "Макошь Lab",
      "topic": "Attachments",
      "content": "См. вложение для Макошь Lab.",
      "filename": "makosh-lab-evidence.txt",
      "file_content": "Макошь Lab evidence {{correlation_id}}"
    },
    {
      "kind": "download_user_upload",
      "upload_uri_ref": "last"
    },
    {
      "kind": "delete_message",
      "message_id_ref": "last"
    }
  ],
  "expect": [
    "signal.raw.zulip.message.observed",
    "signal.raw.zulip.reaction.observed",
    "signal.raw.zulip.message_update.observed",
    "signal.raw.zulip.message_delete.observed"
  ],
  "backend_expected_stages": [
    "signal.accepted.zulip.message",
    "communication.message.recorded",
    "signal.accepted.zulip.reaction",
    "communication_message_reactions",
    "signal.accepted.zulip.message_update",
    "communication_message_versions",
    "signal.accepted.zulip.message_delete",
    "communication_message_tombstones"
  ]
}
```

## Trace report

Every run writes a report under:

```text
.local/makosh-lab/reports/<provider>/<run_id>.json
```

Required fields:

```text
run_id
scenario_id
provider
lab_correlation_id
started_at
finished_at
status
provider_actions
provider_events
expected_stages
observed_stages
backend_expected_stages
backend_observed_stages
capability_contract
backend_validation
failures
```

The lab runner must not claim backend stages were observed unless it actually
executes the backend path. Dry-run and provider-only reports keep backend stages
as expectations/contracts and point to the separate validation command.

For file actions, reports store only file size and SHA-256. They must not store
uploaded file bytes or provider credentials.

## Local operation

Use the root Makefile as the single entrypoint:

```sh
make makosh-lab ACTION=readiness
make makosh-lab ACTION=prepare
make makosh-lab ACTION=init
make makosh-lab ACTION=up
make makosh-lab ACTION=down
make makosh-lab ACTION=logs
make makosh-lab ACTION=realm-link
make makosh-lab ACTION=scenario
make makosh-lab ACTION=scenario SCENARIO=testing/makosh-lab/scenarios/zulip/direct-message.json
make makosh-lab ACTION=scenario SCENARIO=testing/makosh-lab/scenarios/zulip/attachment-materialization.json
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1
```

For direct-message scenarios against Zulip 12, prefer Zulip user IDs over email
recipients. Set:

```sh
ZULIP_DIRECT_RECIPIENT_USER_IDS='[123]'
```

or a comma-separated value such as `ZULIP_DIRECT_RECIPIENT_USER_IDS=123,456`
when executing `testing/makosh-lab/scenarios/zulip/direct-message.json`.

The Make target delegates to:

```sh
node scripts/makosh-lab.mjs --provider zulip <action>
```

The CLI provides three levels:

1. readiness checks for files/env/tools;
2. Zulip stack helpers around the official docker-zulip repository;
3. scenario runner that talks to Zulip and emits a local trace report.

For hermetic provider-side execution without manually managing credentials, use
the testcontainers-backed local Zulip fixture:

```sh
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 \
  SCENARIO=testing/makosh-lab/scenarios/zulip/direct-message.json
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 \
  SCENARIO=testing/makosh-lab/scenarios/zulip/attachment-materialization.json
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 BACKEND=1 \
  SCENARIO=testing/makosh-lab/scenarios/zulip/direct-message.json
```

This mode starts the minimal local Zulip Compose fixture from
`testing/zulip/compose.testcontainers.yml`, provisions a realm, owner, bot,
human user and stream, executes the scenario with bot credentials, writes the
report under `.local/makosh-lab/reports/zulip`, then runs
`docker compose down -v --remove-orphans` for the temporary project.

The runner prints progress for Compose startup, API readiness, realm
provisioning, each scenario action, event polling and cleanup. Long-running
Compose startup and backend live-evidence subprocesses emit periodic heartbeat
lines with elapsed time. Reports include fixture metadata such as stream/user ids
and provider observations, but not Zulip API keys or uploaded file bytes.

With `BACKEND=1`, the same `scenario` action also runs the Zulip backend live
harness:

```sh
cargo test --manifest-path backend/Cargo.toml --test zulip_live -- --ignored --nocapture
```

The Rust harness writes a sanitized backend evidence report under
`.local/makosh-lab/reports/zulip/backend`, and the Lab runner merges its
`observed_stages` into the scenario report. In this mode,
`backend_expected_stages` are enforced against actual backend evidence instead
of remaining documentation-only expectations.

Макошь assertions are intentionally staged. Provider-only runs prove
send/observe and produce a trace artifact. `BACKEND=1` runs add EventStore,
Signal Hub, Communications, Review, provider command, reconciliation, queue
recovery and attachment-materialization evidence.
