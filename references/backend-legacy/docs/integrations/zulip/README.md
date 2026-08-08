# Zulip Integration

Status: proposed reference provider package.

Zulip is the Макошь reference communication provider for local system-level E2E
sessions. It exists to produce real external communication events that Макошь can
observe, normalize, trace and assert from provider boundary to user-facing
projections.

It is not a new product domain. User-facing message state and durable provider
commands belong to Communications. Triage belongs to Review/Radar. Zulip
integration code owns only provider API calls, event mapping and transport
adapters. Application workers own runtime orchestration.

## Why Zulip

Zulip is useful for Макошь Lab because it provides:

- self-hosted local deployment;
- bot/API credentials for repeatable test actions;
- REST APIs for sending and fetching messages;
- event queues for real-time observation;
- channel/topic semantics that map well to conversations and threads;
- reactions, edits, deletes and users for lifecycle scenarios;
- attachments for media-transfer and scanner-boundary scenarios.

## Package ownership

```text
backend/src/integrations/zulip/
├── mod.rs
├── command_execution.rs
├── client.rs
├── event_mapper.rs
└── models.rs
```

The integration owns:

- Zulip REST client configuration;
- provider event queue registration/fetching;
- provider REST calls for stream/direct send, edit, delete, reactions and file
  upload/download;
- provider command execution adapter for message send, edit, delete and
  reactions;
- provider event to raw communication record mapping;
- provider runtime health/readiness evidence.

Application-level command orchestration lives outside the integration package.
The application worker claims Communications-owned provider commands, resolves
credentials and calls the Zulip transport adapter.

It must not own:

- communication business state;
- task/person/document creation;
- review promotion decisions;
- user-facing Zulip pages.

## Inbound event contract

Canonical event types:

```text
signal.raw.zulip.message.observed
signal.raw.zulip.reaction.observed
signal.raw.zulip.message_update.observed
signal.raw.zulip.message_delete.observed
signal.raw.zulip.unknown.observed
```

The implementation maps raw Zulip events into `NewRawCommunicationRecord`
values and uses the Signal Hub raw dispatcher to append canonical
`signal.raw.zulip.*.observed` events. The Communications accepted-signal
consumer currently materializes:

- `signal.accepted.zulip.message` into `communication_messages` and
  `communication.message.recorded`, preserving stream/topic conversations and
  direct-message conversations separately;
- `signal.accepted.zulip.reaction` into `communication_message_reactions` and
  `communication.message.updated`;
- `signal.accepted.zulip.message_update` into `communication_messages`,
  `communication_message_versions` and `communication.message.updated`;
- `signal.accepted.zulip.message_delete` into
  `communication_message_tombstones` and `communication.message.updated`.

Attachments initially remain raw/accepted evidence. Message attachment metadata
is preserved in raw payloads and projected message metadata with `bytes_state =
not_transferred`, `scan_status = not_scanned` and `materialization_state =
not_materialized`; it is not written to blob-backed
`communication_attachments` until bytes are transferred.

The backend byte-transfer workflow in
`backend/src/workflows/zulip_attachment_storage.rs` materializes a Zulip
attachment only after downloaded bytes cross the storage/scanner boundary. It
writes the bytes to local Communications blob storage, records checksum and blob
metadata, runs the existing attachment safety scanner, upserts the canonical
`communication_attachments` row and updates projected Zulip message metadata
idempotently.

`backend/src/application/zulip_attachment_download.rs` wires that workflow into
live account processing. It scans projected Zulip message metadata for pending
attachments, resolves the account-scoped API key, downloads same-realm
`/user_uploads/...` bytes through the Zulip adapter and delegates persistence to
the storage/scanner workflow. It does not materialize messages from REST
responses and does not persist API keys, upload URLs or bytes in logs, reports
or command payloads.

Task-like Zulip messages can feed the existing provider-neutral Review workflow
after message projection. The current backend assertion creates reviewable
`task_candidates` and `review_items` for deterministic English and Russian
action phrasing and does not auto-create accepted Tasks or Obligations.

## Inbound event runtime

`backend/src/application/zulip_event_ingest.rs` owns account-scoped Zulip event
queue polling. It:

- resolves `account_id + zulip_api_key` through the existing secret binding
  boundary;
- registers or resumes a Zulip event queue using
  `communication_ingestion_checkpoints` with stream id `zulip:event_queue`;
- maps provider events into raw communication records;
- dispatches canonical `signal.raw.zulip.*.observed` events through Signal Hub;
- stores only sanitized runtime errors.

The application bootstrap starts the worker behind the Signal Hub runtime gate
and unlocked HostVault gate. Queue expiration causes checkpoint reset and
re-registration on the next poll.

## Outbound API boundary

`ZulipApiClient` supports the REST calls needed at the provider edge for command
execution:

- send stream message;
- send direct message;
- update message content/topic/stream with propagation mode;
- delete message;
- add/remove emoji reaction;
- upload file bytes;
- download same-realm `/user_uploads/...` bytes.

This is only the external provider API boundary. REST is not the internal Макошь
communication contract: Макошь durable outbox commands, capability gates,
Signal Hub evidence, reconciliation and UI/API command routes remain separate
work and must not be inferred from the client methods alone.

## Durable command foundation

Zulip provider commands use the Communications-owned canonical
`communication_provider_commands` table with `channel_kind = 'zulip'`. The
current store supports:

- idempotent enqueue by account and idempotency key;
- due-command claim for queued/retrying commands;
- retryable failure state;
- completed state with provider result payload;
- manual retry for failed/dead-letter commands.

This is the durable queue foundation. A background worker that resolves Zulip
credentials and executes account-scoped message commands through the Zulip
adapter exists in `backend/src/application/zulip_command_executor.rs`.

The worker currently supports:

- `send_stream_message`;
- `send_direct_message`;
- `upload_file`;
- `send_stream_message_with_upload`;
- `send_direct_message_with_upload`;
- `update_message`;
- `delete_message`;
- `add_reaction`;
- `remove_reaction`.

It resolves credentials through the existing account-scoped secret binding
boundary: `account_id + zulip_api_key -> secret_ref -> SecretResolver`. Provider
API errors are stored as sanitized public messages; API keys and provider error
bodies are not written to command errors or result payloads.

Upload commands reference local Communications attachments by `attachment_id` or
`blob_id`. The worker resolves the local blob, rejects non-local or malicious
imports, reads bytes in memory and calls the Zulip upload API. Durable command
payloads store references and metadata, not attachment bytes.

The backend exposes minimal enqueue endpoints for upload commands:

```text
POST /api/v1/integrations/zulip/accounts/{account_id}/commands/upload
POST /api/v1/integrations/zulip/accounts/{account_id}/commands/stream-upload
POST /api/v1/integrations/zulip/accounts/{account_id}/commands/direct-upload
```

Upload-only requests include either `attachment_id` or `blob_id`.
Stream-upload requests also include `stream`, `topic` and `content`.
Direct-upload requests include `recipients` and `content`. Optional fields are
`idempotency_key`, `command_id`, `filename` and `actor_id`. These endpoints only
enqueue durable provider commands; provider API execution still happens through
the account-scoped worker.

The application bootstrap starts periodic Zulip event ingest, attachment
download and command executor workers behind separate Signal Hub runtime gates
and the unlocked HostVault gate.

Макошь Lab includes a dedicated attachment scenario at
`testing/makosh-lab/scenarios/zulip/attachment-materialization.json`. It uploads
a file, sends a stream message containing the returned Zulip upload URI and can
download the same-realm user-upload bytes for provider-side evidence. The Lab
report records file size and SHA-256, not file bytes.

Макошь Lab also includes
`testing/makosh-lab/scenarios/zulip/direct-message.json` for provider-side direct
message evidence. Direct message backend projection is covered separately so
private/direct messages do not fall back to synthetic stream/topic subjects.

`backend/src/application/zulip_provider_observation_reconciliation.rs`
reconciles outbound command completion with later accepted Zulip observations.
REST success marks message lifecycle commands as `completed` with
`reconciliation_status = 'awaiting_provider'`; a matching
`signal.accepted.zulip.*` event marks the command as
`reconciliation_status = 'observed'` and publishes `zulip.command.reconciled`.
Send-with-attachment commands follow the same reconciliation rule after the
uploaded file URI is included in the sent Zulip message content. Upload-only
commands complete after the provider upload response because they do not create
a provider message observation.

## Lab scenario flow

```text
Макошь Lab scenario
↓
Zulip API action
↓
Zulip event queue
↓
signal.raw.zulip.*.observed
↓
Signal Hub
↓
signal.accepted.zulip.*
↓
Communications projection
↓
Review / Timeline / Search assertions
```

## Runtime configuration

The local lab runner expects credentials through environment variables or a
`.env` file kept outside source control:

```text
ZULIP_BASE_URL=http://localhost:8080
ZULIP_EMAIL=bot@example.test
ZULIP_API_KEY=...
```

Secrets stay outside repository files. The committed files contain only examples
and shape contracts.

For backend-managed account setup, `POST /api/v1/integrations/zulip/accounts`
creates a `zulip_bot` provider account and stores the `api_key` in HostVault.
PostgreSQL stores account metadata, a `secret_references` row with
`secret_kind = api_token`, and the account-to-secret binding for
`zulip_api_key`; it does not store the API key payload.

## Documentation

- [Architecture](architecture.md)
- [Implementation plan](implementation-plan.md)
- [Fixture test matrix](fixture-test-matrix.md)
- [Status](status.md)
