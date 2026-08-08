# Zulip Integration Architecture

Status: proposed.

## Boundary

Zulip transport code lives in `integrations/zulip`. It talks to the Zulip
REST/event APIs and maps provider observations. It does not import `domains/*`
and does not write Communications tables directly. Application workers own
runtime orchestration.

Backend account setup lives at the app boundary. It may create provider account
metadata, HostVault secret entries, `secret_references`, account secret bindings
and Signal Hub account connections. It must not put Zulip API keys in account
config, logs, reports or database secret-payload tables.

```text
Zulip REST/event API
↓
ZulipApiClient
↓
application Zulip event ingest worker
↓
ZulipEventMapper
↓
NewRawCommunicationRecord
↓
communication_raw_records
↓
signal.raw.zulip.*.observed
↓
Signal Hub
↓
signal.accepted.zulip.*
↓
Communications accepted-signal consumer / workflows / projections
```

## Event mapping

| Zulip event | Макошь event | Subject |
|---|---|---|
| `message` | `signal.raw.zulip.message.observed` | provider message ID |
| `reaction` | `signal.raw.zulip.reaction.observed` | provider message ID + emoji/user evidence |
| `update_message` | `signal.raw.zulip.message_update.observed` | provider message ID |
| `delete_message` | `signal.raw.zulip.message_delete.observed` | provider message ID |
| other | `signal.raw.zulip.unknown.observed` | provider event ID |

The mapper preserves raw provider data only as sanitized JSON payload. It should
not store credentials, cookies, private keys or unrelated realm state in the
payload.

## Correlation

Lab scenarios create a stable lab correlation ID before generating a provider
action. It is retained in report/provenance metadata. Canonical EventStore
`correlation_id` follows the observation trace produced by raw communication
record capture.

Example:

```text
scenario: message_to_task_candidate
lab_correlation_id: lab-zulip-20260629-001
provider_event_id: 42
raw_record_id: raw_zulip_<sha256>
```

## Idempotency

A provider event must produce the same raw record identity when replayed:

```text
account_id + record_kind + provider_record_id
```

This lets the raw communication store, Signal Hub and EventStore de-duplicate
naturally when a queue is re-registered or a scenario is replayed.

The inbound runtime stores queue state in `communication_ingestion_checkpoints`
with stream id `zulip:event_queue`:

```json
{
  "queue_id": "zulip queue id",
  "last_event_id": 42
}
```

## Projection expectations

The integration is only the first stage. Full success for a message scenario is
not just "Zulip returned an event". The current inbound message trace contains:

```text
signal.raw.zulip.message.observed
signal.accepted.zulip.message
communication.message.recorded
```

Accepted lifecycle events are materialized into canonical Communications state:

| Accepted event | Canonical state | Projection event |
|---|---|---|
| `signal.accepted.zulip.reaction` | `communication_message_reactions` | `communication.message.updated` |
| `signal.accepted.zulip.message_update` | `communication_messages`, `communication_message_versions` | `communication.message.updated` |
| `signal.accepted.zulip.message_delete` | `communication_message_tombstones` | `communication.message.updated` |

Zulip message attachments are initially retained only as provider metadata in
the raw record payload and projected `message_metadata`. The metadata explicitly
marks `bytes_state = not_transferred`, `scan_status = not_scanned` and
`materialization_state = not_materialized`.

Blob-backed materialization is a separate workflow boundary. The Zulip adapter
may download only same-realm `/user_uploads/...` URLs. The workflow in
`backend/src/workflows/zulip_attachment_storage.rs` accepts transferred bytes,
writes them to local Communications blob storage, records checksum/size/blob
metadata, runs the existing attachment safety scanner, upserts
`communication_attachments`, and updates projected message metadata. It must not
create `communication_attachments` from metadata alone.

`backend/src/application/zulip_attachment_download.rs` orchestrates this
boundary for live accounts. It scans projected Zulip message metadata for
pending attachment evidence, resolves `account_id + zulip_api_key`, downloads
only same-realm user-upload bytes through the Zulip adapter, and then delegates
all durable materialization to the storage/scanner workflow. The downloader does
not create messages from REST responses and does not log upload URLs, bytes or
API keys.

The later full lab trace should also include conversation recording, search
indexing, Review/Radar candidate evidence and a frontend trace surface.

The current backend Review assertion uses the existing provider-neutral
`refresh_message_task_candidates_into_review` workflow after Zulip message
projection. This keeps candidate detection outside the Zulip integration
package: Zulip provides communication evidence, while the Tasks/Review workflow
creates reviewable `task_candidates` and `review_items` without auto-creating
accepted Tasks or Obligations.

## Outbound reconciliation

Outbound message lifecycle commands are durable Communications provider commands.
Zulip REST success records provider dispatch success and the provider message ID,
but the command remains `reconciliation_status = 'awaiting_provider'`.

The reconciliation consumer observes accepted Zulip signals and marks matching
commands `observed` only when Signal Hub accepted evidence exists:

| Accepted event | Command kinds |
|---|---|
| `signal.accepted.zulip.message` | `send_stream_message`, `send_direct_message`, `send_stream_message_with_upload`, `send_direct_message_with_upload` |
| `signal.accepted.zulip.reaction` | `add_reaction`, `remove_reaction` |
| `signal.accepted.zulip.message_update` | `update_message` |
| `signal.accepted.zulip.message_delete` | `delete_message` |

The consumer appends `zulip.command.reconciled` as a child of the accepted signal
event, preserving the provider-observation trace.

Upload-only commands do not produce a provider message and complete after the
Zulip upload response. Send-with-upload commands first upload bytes from a local
Communications blob, include the returned Zulip upload URI in message content,
and then wait for the normal accepted message observation.

## Failure handling

- `BAD_EVENT_QUEUE_ID` or equivalent queue expiration should trigger queue
  re-registration.
- HTTP errors are runtime failures and should be recorded as integration health
  evidence.
- Mapping errors are data-shape failures and should be visible in the lab report.
- Unknown events are not dropped silently; they become
  `signal.raw.zulip.unknown.observed` for inspection.
