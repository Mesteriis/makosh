# Zulip Integration Implementation Plan

Status: proposed.

## Phase 1: Reference provider skeleton

Goal: add a safe integration boundary and a runnable lab message scenario.

Deliverables:

- `backend/src/integrations/zulip` module;
- REST client for register queue, get events, stream/direct sends, edits,
  deletes, reactions and file upload;
- provider event mapper to raw communication records and Signal Hub raw events;
- single Макошь Lab script for stack management and scenario execution;
- one message scenario with trace metadata;
- readiness script for local prerequisites.

Acceptance:

- No business domain imports from `integrations/zulip`.
- Raw record and Signal Hub event IDs are deterministic.
- Scenario output includes `scenario_id`, `lab_correlation_id`, provider
  response and observed stage names.

## Phase 2: Communications consumer

Goal: convert accepted Zulip message lifecycle signals into provider-neutral
Communications state.

Deliverables:

- accepted-signal consumer using existing EventConsumerRunner conventions;
- idempotent materialization into Communications message state;
- tests asserting message, reaction, edit and delete flow from raw accepted
  signals into canonical Communications tables;
- trace annotations for every stage.
- provider API contract tests for outbound lifecycle and upload calls.

Acceptance:

- Replaying the same Zulip event does not duplicate communication records.
- Provider-specific thread/topic evidence is retained.
- The trace can show provider event -> communication message.
- Reaction, edit and delete accepted-signal replay remains idempotent.

## Phase 2a: Inbound event runtime

Goal: poll Zulip event queues without making the integration package own
Communications state or Signal Hub policy.

Deliverables:

- application worker that resolves `account_id + zulip_api_key`;
- event queue registration/resume through the Zulip adapter;
- `communication_ingestion_checkpoints` state for `zulip:event_queue`;
- raw communication record capture and Signal Hub raw-signal dispatch;
- application bootstrap scheduler behind Signal Hub runtime controls and an
  unlocked HostVault gate;
- tests for first poll, checkpoint resume and replay idempotency.

Acceptance:

- Integration code does not import Communications business domains or mutate
  `communication_*` tables.
- API keys and provider response bodies do not appear in stored errors or logs.
- Replaying a previously observed provider event does not duplicate raw or
  accepted Signal Hub events.
- Queue expiration triggers checkpoint reset and queue re-registration.

Current status:

- Queue registration, checkpoint resume, replay idempotency and expired queue
  re-registration are covered by `backend/tests/zulip.rs`.

## Phase 2b: Attachment metadata evidence

Goal: preserve Zulip attachment metadata without pretending that media bytes,
quarantine or scanner verdicts exist.

Deliverables:

- mapper support for Zulip message `attachments` metadata;
- explicit metadata state: `bytes_state = not_transferred`,
  `scan_status = not_scanned`, `materialization_state = not_materialized`;
- projection retention through accepted Zulip message metadata;
- regression test proving replay idempotency and zero
  `communication_attachments` rows before byte transfer.

Acceptance:

- Attachment metadata is visible as raw/accepted communication evidence.
- No blob-backed Communications attachment is created without a real blob,
  checksum and scanner boundary.
- Future media transfer work can consume the provider metadata without changing
  the Signal Hub event contract.

## Phase 2c: Attachment byte transfer and materialization

Goal: materialize Zulip attachment bytes only after they cross the existing
storage and scanner boundary.

Deliverables:

- safe same-realm `/user_uploads/...` download support in the Zulip REST
  adapter;
- backend workflow that accepts transferred bytes, stores them in local
  Communications blob storage and records blob metadata;
- attachment safety scanner handoff before canonical attachment projection;
- idempotent upsert into `communication_attachments`;
- projected Zulip message metadata update from `not_transferred` to
  `materialized`;
- regression test proving replay/idempotency and zero materialization before
  byte transfer.

Acceptance:

- Cross-realm upload URLs are rejected before any HTTP request.
- Metadata alone never creates `communication_attachments`.
- Re-running materialization for the same provider attachment does not create
  duplicate canonical attachment rows.
- Scanner status is stored with the canonical attachment record and reflected in
  message metadata.

Current backend status:

- The controlled backend workflow and automatic account-scoped download worker
  are implemented and covered by integration regression tests.
- Макошь Lab has a dedicated attachment scenario for upload, stream
  send-with-upload and same-realm download metadata reports.
- Executed Макошь Lab `BACKEND=1` evidence covers automatic backend
  materialization against a real Zulip realm through the Zulip live harness.

## Phase 3: Review/Radar and task debugging

Goal: use Zulip messages to debug candidate extraction and promotion flows.

Deliverables:

- scenario text that should produce a task candidate;
- structured fact text that should produce a Polygraph contradiction candidate;
- deterministic heuristic or AI-disabled fixture mode;
- Review candidate assertion;
- person identity trace support for Zulip senders;
- optional promotion flow assertion into Tasks;
- provider-neutral UI trace page fixture.

Acceptance:

- Supported deterministic English, Russian, Spanish, French or German
  task-like messages can be traced from Zulip/Communications to Review candidate
  evidence.
- A Zulip sender identity can be represented as source evidence and used by the
  provider-neutral Consistency/Polygraph refresh to create a contradiction
  observation without overwriting remembered facts.
- No Task is created without explicit promotion or a documented rule.

Current backend status:

- Supported deterministic Russian task-like Zulip text can be projected into
  `task_candidates` and mirrored to `review_items` through the existing
  provider-neutral Review workflow.
- Provider-neutral deterministic extraction now covers explicit Spanish,
  French and German action markers and simple due clauses in addition to the
  existing English/Russian coverage.
- Provider-neutral deterministic extraction now also covers free-form polite
  task requests in English, Russian, Spanish, French and German without adding
  provider-specific task logic to Zulip.
- Provider-neutral task candidate review tests assert that explicit promotion
  creates durable Tasks while suggested candidates do not.
- `person_identities` accepts Zulip sender identities and provider-neutral
  Consistency/Polygraph refresh can match Zulip messages by sender id/email,
  detect structured fact contradictions and preserve the old memory value.
- Provider-neutral Event Trace UI fixture covers a sanitized Zulip
  message-to-task trace from raw signal through accepted signal, Communications
  projection and Review candidate availability.

## Phase 3a: Outbound command runtime

Goal: execute durable Zulip provider commands without making REST the internal
Макошь contract.

Deliverables:

- provider-neutral command lifecycle in Communications;
- Zulip command adapter that has no business table access;
- application worker that resolves `account_id + zulip_api_key`, claims due
  `channel_kind = 'zulip'` commands, executes them through the adapter and stores
  completed/retrying/dead-letter state;
- provider-observed reconciliation consumer that marks completed commands
  `observed` only after matching `signal.accepted.zulip.*` evidence arrives;
- upload command execution that resolves local `attachment_id` or `blob_id`
  references, uploads bytes through the Zulip adapter, and avoids storing bytes
  in durable command payloads;
- backend enqueue endpoints for upload-only, stream send-with-upload and direct
  send-with-upload commands;
- frontend Settings integration controls for connecting a Zulip bot account and
  queueing upload-only, stream send-with-upload and direct send-with-upload
  commands by `attachment_id` or `blob_id`;
- application bootstrap scheduler that periodically runs the worker behind
  Signal Hub runtime controls and an unlocked HostVault gate;
- tests for successful command execution, sanitized provider failures and
  provider-observed reconciliation.

Acceptance:

- Integration code does not import Communications business domains or mutate
  `communication_*` tables.
- API keys do not appear in `Debug`, stored command errors or result payloads.
- Provider API error bodies are not persisted as command errors.
- REST success alone leaves lifecycle commands awaiting provider observation.
- Matching accepted Zulip observations publish `zulip.command.reconciled`.
- Replaying/claiming completed commands does not execute the provider action
  again.
- Upload commands reject missing, non-local or malicious attachment references.
- Send-with-upload commands remain `awaiting_provider` until a matching accepted
  Zulip message observation arrives.
- The background scheduler does not run without database, runtime permission and
  unlocked HostVault state.

## Phase 3b: Backend account setup

Goal: persist Zulip bot credentials through the current HostVault boundary
without adding production UI in this branch.

Deliverables:

- guarded backend setup endpoint for `zulip_bot` accounts;
- HostVault storage for `zulip_api_key` with `secret_kind = api_token`;
- PostgreSQL `secret_references` and account secret binding only;
- Signal Hub account connection synchronization;
- regression test proving API keys are not stored in provider account config,
  response bodies, secret metadata or legacy database secret-payload tables.

Acceptance:

- Zulip account setup can create an account usable by the command executor and
  event ingest workers.
- API key payloads remain outside PostgreSQL.
- Production UI remains a separate phase.

## Phase 4: Compatibility suite

Goal: measure communication provider coverage.

Deliverables:

- shared scenario categories: receive, send, edit, delete, reaction, attachment,
  identity, thread/topic, search, review candidate;
- provider adapter matrix for Zulip, Telegram, WhatsApp and Mail;
- report format with pass/fail/unsupported/deferred.

Acceptance:

- A provider can publish a percentage and evidence list rather than a vague
  status claim.

Current status:

- Zulip has a `make makosh-lab ACTION=compliance PROVIDER=zulip` report that
  aggregates scenario contracts, local Lab execution reports and backend live
  evidence into per-capability pass/pending/deferred status.
- `BACKEND=1` refreshes machine-readable backend contract evidence for the
  Zulip provider suite, Zulip Polygraph trace, deterministic multilingual
  task-candidate suite and deterministic free-form multilingual task-candidate
  suite before generating the report.
- `REQUIRE_CLOSED=1` turns the report into a hard gate for future closure
  audits.
