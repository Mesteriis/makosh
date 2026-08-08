# Zulip Integration Status

Status date: 2026-06-30.

## Current state

This package is a reference-provider foundation with backend runtime, Макошь Lab
evidence and a Settings integration surface for Zulip bot account setup and
reference upload commands. It has local testcontainers live-realm evidence for
the provider API, event queue and Макошь projection path.

## Done by initial slice

- Documentation package.
- ADR for Zulip as reference provider and Макошь Lab.
- Signal Hub raw event types.
- Planned scenario matrix.
- REST client and DTOs in `backend/src/integrations/zulip`, including
  provider API calls for stream/direct send, edit, delete, reactions and file
  upload.
- Durable canonical provider-command store for Zulip enqueue, claim, retry,
  completion and manual retry.
- Account-scoped Zulip command worker that resolves `zulip_api_key`, executes
  due message/reaction/edit/delete commands through the provider adapter, marks
  completed/retrying/dead-letter states, and stores sanitized provider errors.
- Durable Zulip upload command execution from local Communications blobs:
  `upload_file`, `send_stream_message_with_upload` and
  `send_direct_message_with_upload` prepare bytes from `attachment_id` or
  `blob_id`, reject malicious/non-local imports, call the Zulip upload API and
  store sanitized result payloads without embedding bytes in the durable command
  payload.
- Backend enqueue endpoints for Zulip upload commands:
  `POST /api/v1/integrations/zulip/accounts/{account_id}/commands/upload`,
  `POST /api/v1/integrations/zulip/accounts/{account_id}/commands/stream-upload`
  and
  `POST /api/v1/integrations/zulip/accounts/{account_id}/commands/direct-upload`.
- Always-on application bootstrap wiring for the Zulip command worker, gated by
  Signal Hub runtime state and unlocked HostVault state.
- Provider observation reconciliation for completed Zulip message lifecycle
  commands: accepted Zulip message/reaction/edit/delete signals mark matching
  canonical provider commands as `observed` and publish
  `zulip.command.reconciled`.
- Always-on application bootstrap wiring for the Zulip provider-observation
  reconciliation consumer, gated by Signal Hub runtime state.
- Account-scoped Zulip event ingest worker that resolves `zulip_api_key`,
  registers/resumes the Zulip event queue, records raw communication records,
  dispatches `signal.raw.zulip.*.observed` through Signal Hub, persists
  `zulip:event_queue` checkpoints, and sanitizes provider API failures.
- Regression coverage for expired Zulip event queue checkpoints:
  `BAD_EVENT_QUEUE_ID`/HTTP 400 triggers checkpoint reset, queue
  re-registration, raw signal dispatch and new checkpoint persistence.
- Always-on application bootstrap wiring for the Zulip event ingest worker,
  gated by Signal Hub runtime state and unlocked HostVault state.
- Backend Zulip account setup endpoint that stores `zulip_api_key` in HostVault,
  persists only `secret_references` and account-to-secret bindings in
  PostgreSQL, and synchronizes the Signal Hub account connection.
- Frontend Settings integration panel for Zulip bot account setup. The UI posts
  credentials only to the HostVault-backed account setup endpoint and does not
  store API keys in frontend state beyond the submit form.
- Frontend Settings integration panel for Zulip upload-only,
  stream-send-with-upload and direct-send-with-upload command enqueue. The UI
  accepts `attachment_id`/`blob_id` references and never uploads raw file bytes
  from the browser.
- Raw event mapper for `signal.raw.zulip.*.observed`.
- Signal Hub source and raw dispatcher for Zulip.
- Communications projection for `signal.accepted.zulip.message`.
- Direct Zulip message projection that preserves direct conversation shape
  instead of falling back to stream/topic subject and recipients.
- Communications lifecycle materialization for accepted Zulip reactions, message
  edits and provider deletes.
- Review task-candidate assertion for a Russian task-like Zulip message using
  the existing deterministic candidate refresh and Review inbox mirror. It
  creates `task_candidates` and `review_items` evidence, not accepted Tasks or
  Obligations.
- Provider-neutral deterministic Review/Radar task-candidate extraction now
  covers explicit English, Russian, Spanish, French and German action markers,
  with simple due-text extraction for `by`, `до`, `para`, `avant` and `bis`
  clauses. The Zulip path benefits through Communications evidence rather than
  provider-specific task logic.
- Provider-neutral deterministic Review/Radar task-candidate extraction also
  covers free-form polite task requests in English, Russian, Spanish, French
  and German, such as “Could you check…”, “Можешь проверить…”,
  “¿Puedes preparar…”, “Peux-tu préparer…” and “Kannst du … prüfen…”. This is
  deterministic request recognition, not a claim of open-ended LLM extraction.
- Zulip sender identity traces can be represented in `person_identities` with
  `identity_type = 'zulip'`. Provider-neutral Polygraph/Consistency refresh can
  match Zulip channel messages to active Personas by sender id/email and create
  contradiction observations from structured facts such as `Location: Madrid`
  without overwriting remembered `persona_facts`.
- Zulip message attachment metadata mapping as raw/accepted evidence with
  explicit `bytes_state = not_transferred`, `scan_status = not_scanned` and
  `materialization_state = not_materialized`; no canonical
  `communication_attachments` rows are created until bytes cross the media
  safety boundary.
- Safe same-realm Zulip user-upload download support in the REST adapter.
- Backend attachment byte-transfer/materialization workflow that accepts
  downloaded bytes, writes them to the local Communications blob store, records
  checksum/size/blob metadata, runs the existing attachment scanner boundary,
  upserts a canonical `communication_attachments` row and updates projected
  Zulip message metadata idempotently.
- Account-scoped Zulip attachment download worker that scans projected Zulip
  message metadata, resolves `zulip_api_key`, downloads same-realm
  `/user_uploads/...` bytes through the Zulip adapter, and delegates
  materialization to the storage/scanner workflow.
- Always-on application bootstrap wiring for the Zulip attachment download
  worker, gated by Signal Hub runtime state and unlocked HostVault state.
- Макошь Lab scenario actions for Zulip stream send, direct send, reaction,
  edit, delete, upload, send-with-upload and same-realm user-upload download.
- Single Макошь Lab CLI and scenario fixtures, including a dedicated attachment
  materialization scenario whose reports store file size/hash instead of bytes
  and a direct-message scenario for provider-side direct send evidence.
- Макошь Lab `TESTCONTAINERS=1` scenario execution that starts the local Zulip
  Compose fixture, provisions a real realm/bot/human/stream, executes provider
  actions with visible progress output, writes sanitized local reports and
  cleans the temporary Compose project with volumes.
- Макошь Lab `BACKEND=1` scenario execution that runs the Zulip backend live
  harness, writes a sanitized backend evidence report, merges backend observed
  stages into the Lab scenario report and enforces `backend_expected_stages`
  against actual backend evidence.
- Макошь Lab `ACTION=compliance` Communication Compliance Suite report that
  aggregates Zulip scenario contracts, local Lab reports and backend evidence
  into pass/pending/deferred capability status under
  `.local/makosh-lab/reports/zulip/compliance`.
- Макошь Lab `ACTION=compliance BACKEND=1` backend contract evidence mode that
  runs the targeted Zulip, Polygraph and multilingual task-candidate suites
  through `makosh_test_session`, writes a machine-readable backend contract
  evidence report, and folds those passes into the compliance report.
- Shared Zulip realm provisioning script used by both the Rust live fixture and
  Макошь Lab to avoid divergent test-realm setup.
- Testcontainers-backed local Zulip live fixture that provisions a real realm,
  owner, bot, human user and stream, then verifies real provider events for
  stream messages, direct messages, attachment messages, reaction add/remove,
  message edit and provider delete through Signal Hub and Communications
  projection. The fixture also verifies Review task-candidate evidence without
  auto-creating durable Tasks or Obligations.
- Testcontainers-backed live backend worker round-trip against a real Zulip
  realm: durable `send_stream_message_with_upload` command execution from a
  local Communications blob, real Zulip provider observation through the event
  ingest worker, accepted-signal projection, provider-observed reconciliation
  and attachment byte materialization into `communication_attachments`.
- Testcontainers-backed live event-ingest recovery evidence for bad/expired
  Zulip queue checkpoints: the worker handles the real provider `HTTP 400`
  response, registers a fresh event queue, persists the replacement checkpoint
  and ingests a subsequent real Zulip message through Signal Hub and
  Communications projection.
- Mapper fallback for Zulip message events where the real event queue contains a
  `/user_uploads/...` link in message content but omits `message.attachments`;
  the fallback records attachment metadata as evidence-only with
  `bytes_state = not_transferred`.
- Provider-neutral Event Trace UI fixture for a sanitized Zulip
  message-to-task path from raw signal through accepted signal, Communications
  projection and Review candidate availability. The fixture is credential-free
  and lives under the platform event-tracing surface, not a provider-specific
  business page.

## Latest evidence

Validated on 2026-06-30:

```sh
cargo test --manifest-path backend/Cargo.toml domains::tasks::candidates::extraction::tests -- --nocapture
cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress none --test-threads 1 --test task_candidates task_candidate_refresh_detects_multilingual_message_actions_against_postgres
cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress none --test-threads 1 --test task_candidates task_candidate_refresh_detects_freeform_multilingual_message_requests_against_postgres
cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress none --test-threads 1 --test task_candidates
cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress none --test-threads 1 --test consistency_contradiction contradiction_refresh_detects_zulip_message_claim_against_active_person_fact_without_overwriting_memory
cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress none --test-threads 1 --test consistency_contradiction
cargo fmt --manifest-path backend/Cargo.toml --check
node --check scripts/makosh-lab.mjs
cargo fmt --manifest-path crates/testkit/Cargo.toml --check
cargo test --manifest-path backend/Cargo.toml --test zulip_live --no-run
make makosh-lab ACTION=readiness
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 BACKEND=1
make testcontainers-clean
cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- cargo nextest run --manifest-path backend/Cargo.toml --profile default --show-progress bar --test-threads 1 --test email_account_setup startup_reconciles_icloud_account_from_host_vault_manifest_after_postgres_metadata_wipe
cd frontend && pnpm test:unit -- src/platform/event-tracing/fixtures/zulipMessageToTaskTrace.test.ts src/platform/event-tracing/EventTracePanel.boundary.test.ts src/integrations/zulip/components/ZulipSettingsPanel.boundary.test.ts
cd frontend && pnpm typecheck
cd frontend && pnpm lint:ox
make validate
git diff --check
make makosh-lab ACTION=compliance PROVIDER=zulip REQUIRE_CLOSED=1
```

Result: passed. The targeted extraction unit tests cover Spanish, French and
German action/due markers and free-form polite task requests in English,
Russian, Spanish, French and German. The session-backed targeted free-form
multilingual task-candidate regression passed 1/1 against PostgreSQL/NATS
testcontainers. The session-backed `task_candidates` run passed 14/14 tests
against PostgreSQL/NATS testcontainers, including multilingual candidate
persistence, free-form multilingual request extraction, Russian action
extraction, obligation extraction and review promotion behavior. The targeted
Zulip Polygraph regression passed, and the full session-backed
`consistency_contradiction` run passed 15/15 tests against PostgreSQL/NATS
testcontainers, including Telegram, WhatsApp and Zulip provider message
contradiction detection. The test sessions reported no stale Макошь
testcontainers before startup and no remaining Макошь testcontainers after
cleanup. The live Макошь Lab run started a real local Zulip fixture, provisioned
a realm/bot/human/stream, observed provider raw stages, removed the temporary
Compose project, then ran the Rust `zulip_live` backend evidence harness against
a second real Zulip testcontainers fixture. The backend evidence report was
written to
`.local/makosh-lab/reports/zulip/backend/zulip_backend_live_trace-2026-06-30T00-49-02.805538-00-00.json`;
the scenario report was written to
`.local/makosh-lab/reports/zulip/zulip_message_to_task_candidate-2026-06-30T00-46-55.499Z.json`.
Post-run Docker checks found no Макошь-labelled testcontainers and no
stopped/created/dead containers.
The frontend unit command completed with the full Vitest suite green
(154 test files, 538 tests), including the sanitized Zulip Event Trace UI
fixture and Zulip settings boundary checks. Frontend typecheck and Oxlint also
passed. Full `make validate` passed after fixing the provider-account metadata
wipe race in the vault reconciliation regression test; backend nextest reported
1441/1441 passed with 1 skipped, frontend lint/unit/typecheck/build passed, and
the final Zulip compliance hard-gate reported 30/30 capabilities passing with
zero pending/deferred capabilities.

Earlier Zulip provider evidence:

```sh
MAKOSH_ZULIP_TESTCONTAINERS=1 MAKOSH_ZULIP_START_TIMEOUT_SECS=900 cargo test --manifest-path backend/Cargo.toml --test zulip_live -- --ignored --nocapture
```

Result: passed, 1/1. The observed real Zulip queue events included
`message`, `reaction`, `update_message` and `delete_message`. The same run also
verified the backend worker path for durable send-with-upload, event ingest,
provider-observed reconciliation and attachment materialization against the real
local realm. It also verified live event-ingest recovery after a bad Zulip queue
checkpoint by re-registering the queue and ingesting a subsequent provider
message.

```sh
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 SCENARIO=testing/makosh-lab/scenarios/zulip/attachment-materialization.json
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 SCENARIO=testing/makosh-lab/scenarios/zulip/direct-message.json
```

Result: passed. The three provider-side lab reports were written under
`.local/makosh-lab/reports/zulip` with zero failures, sanitized fixture metadata,
no credential payloads, and observed raw Zulip stages for stream message,
reaction, message update, message delete, attachment message and direct message
events.

```sh
make makosh-lab ACTION=scenario PROVIDER=zulip EXECUTE=1 TESTCONTAINERS=1 BACKEND=1 SCENARIO=testing/makosh-lab/scenarios/zulip/direct-message.json
make makosh-lab ACTION=compliance PROVIDER=zulip
make makosh-lab ACTION=compliance PROVIDER=zulip BACKEND=1
make makosh-lab ACTION=compliance PROVIDER=zulip REQUIRE_CLOSED=1
make makosh-lab ACTION=compliance PROVIDER=zulip BACKEND=1 REQUIRE_CLOSED=1
```

Result: passed. The Lab report status was `provider_and_backend_observed`,
with zero failures, no credential payloads, provider-side direct-message
observation and 32 backend observed stages from
`backend/tests/zulip_live.rs`, including `zulip.direct_conversation`,
`provider_command.completed`, `zulip.command.reconciled`,
`communication_attachments`, `attachment_state.materialized` and
`zulip_event_queue.reregistered`. The compliance report command writes a
machine-readable pass/pending/deferred capability report from local evidence.
With `BACKEND=1 REQUIRE_CLOSED=1`, the compliance action passed targeted Zulip
backend contract (21/21), Zulip Polygraph (1/1), deterministic multilingual
task-candidate (1/1) and deterministic free-form multilingual task-candidate
(1/1) suites, then wrote a hard-gate report with 30/30 capabilities passing and
zero pending/deferred capabilities.

```sh
cd frontend && pnpm test:unit -- src/integrations/zulip/api/zulip.test.ts src/integrations/zulip/components/ZulipSettingsPanel.boundary.test.ts
cd frontend && pnpm typecheck
```

Result: passed. The Zulip frontend API test covers account setup and all three
upload command enqueue routes. The boundary test verifies the Settings panel
uses query mutations and backend attachment/blob references instead of direct
fetches, `FormData` or file inputs.

## Closure gate

Zulip reference-provider closure requires evidence for:

```text
message send via Zulip API
direct message send via Zulip API
direct message communication projection
edit/delete/reaction REST API call support
account-scoped provider command execution with resolved zulip_api_key
HostVault-backed Zulip account setup without PostgreSQL secret payloads
sanitized provider command failure recording
always-on scheduler/bootstrap wiring for queued Zulip commands
provider-observed reconciliation for completed Zulip commands
file upload via Zulip API
send-with-upload via Zulip API
same-realm user-upload download via Zulip API
durable upload/send-with-attachment command execution from local blobs
message observed from Zulip queue
always-on scheduler/bootstrap wiring for Zulip event ingest
Zulip event queue checkpoint persistence, replay idempotency and re-registration
canonical Signal Hub raw and accepted event append
communication message materialization
reaction/edit/delete canonical materialization
attachment metadata retained as evidence without blob-backed materialization
attachment bytes transferred through scanner boundary into communication_attachments
always-on scheduler/bootstrap wiring for Zulip attachment downloads
zulip.command.reconciled event append
trace metadata visible in Макошь
review/radar candidate path for Russian task-like message
review/radar candidate path for Spanish/French/German deterministic task-like messages
person identity trace support for Zulip senders
Polygraph fact contradiction path for Zulip messages without overwriting memory
replay idempotency
architecture/code-boundary checks
```
