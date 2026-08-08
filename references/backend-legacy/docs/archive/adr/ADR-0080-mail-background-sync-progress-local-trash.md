# ADR-0080 Mail Background Sync, Progress and Local Trash

Status: Accepted
Date: 2026-06-10
Deciders: Alex (makosh maintainer)

## Context

Макошь mail already has provider account setup, raw/blob preservation,
message projection and provider SMTP send for iCloud/generic IMAP. The mail
workbench now needs continuous account-scoped ingestion instead of only manual or
fixture-driven imports.

The owner also clarified deletion semantics: deleting mail in the Макошь UI must
not delete, move, trash or expunge the original provider message. Макошь
should hide the item from active workbench views while preserving local raw/blob
content and metadata for replay, AI analysis and analytics.

Mail sync also feeds the knowledge system. Sender and recipient identities should
be projected to Personas, organizations and relationship events as mail arrives.

## Decision

Add account-scoped mail sync settings and durable run history.

- Every `gmail`, `icloud` and `imap` provider account has effective defaults:
  `sync_enabled = true`, `batch_size = 5`, `poll_interval_seconds = 300`.
- Background scheduling is per account and prevents overlapping active runs for
  the same account.
- Manual "check now" uses the same account-scoped service and records a durable
  run.
- Run status records only sanitized metadata: account id, trigger, phase,
  progress, counts, checkpoint presence and sanitized error code/message.
- Status and audit records must never contain mail bodies, raw MIME, attachment
  bytes, tokens, passwords or plaintext secret references.

Provider read behavior:

- IMAP/iCloud full backfill starts without `last_seen_uid`, fetches batches in
  ascending UID order, persists checkpoint data, and loops until an empty batch.
- IMAP/iCloud incremental sync starts after stored `last_seen_uid`.
- If IMAP UID validity changes, UID progress is reset and the mailbox is read
  again from the start.
- Gmail full backfill pages through `users.messages.list` and raw message reads.
- Gmail incremental sync uses Gmail history when a `history_id` checkpoint is
  available. If history is expired, the run records a recoverable full-resync
  condition and restarts full listing.

Local deletion behavior:

- `communication_messages.local_state` is separate from workflow state.
- Active UI lists, counts, threads, search indexing and resource summaries use
  `local_state = active` by default.
- UI delete sets `local_state = trash`, `local_state_changed_at` and
  `local_state_reason`.
- Restore sets `local_state = active`.
- Local trash is not auto-emptied.
- Provider delete operations are not used for UI delete. The legacy
  `/imap-delete` route maps to local trash and does not send IMAP `STORE`,
  `EXPUNGE`, provider move or provider trash commands.
- Reprojection preserves an existing `trash` state.

Knowledge projection behavior:

- Mail projection creates or updates Personas and active email identities from
  normalized addr-spec values.
- Mail projection creates durable `communication_message_participants` links.
- Mail projection creates idempotent relationship events for sender/recipient
  interactions.
- Mail projection creates non-public-domain organizations, organization domains
  and organization-Persona links with mail sync/message provenance.
- Graph projection is refreshed after successful mail batches. Task/note
  extraction remains candidate or user-controlled and does not auto-activate
  tasks.

Frontend behavior:

- The mail workbench exposes a desktop-only account selector with per-account
  sync state and thin progress indicators.
- The selected account exposes sync settings and manual check-now controls.
- All Accounts shows aggregate sync progress.
- The workbench has a Trash filter/folder. Normal inbox/thread/search/resource
  views exclude trash by default.
- Message detail exposes Delete for active messages and Restore for trash
  messages.
- All new visible strings use i18n keys; Russian translations are maintained.
- Validation remains desktop-only while ADR-0031 is active.

## Consequences

- Mail sync is durable, inspectable and configurable per account.
- Provider credentials remain account scoped and resolved only at runtime.
- UI delete is safe for provider mailboxes and reversible locally.
- Local trash keeps data available for replay, AI and analytics.
- Sync can be extended later with provider-specific adapters, outbox queues,
  attachment scanning or mobile behavior without changing the local-state
  contract.

## References

- ADR-0031 — Temporary Desktop Only UI Scope
- ADR-0041 — Email Provider Ingestion Foundation
- ADR-0046 — Persistent Dev Mail Cache and Blob Storage
- ADR-0052 — Capability-Based Provider Writes
- ADR-0055 — Full Email Provider Networking
- ADR-0060 — Person Timeline and Graph Integration
- ADR-0061 — Organization as First-Class Entity
- ADR-0062 — Organization Identity and Resolution
- ADR-0066 — Organization Graph Integration
- ADR-0070 — Tasks First-Class Domain
- ADR-0071 — Task Context Evidence Provenance
- ADR-0074 — Persona Multi-Channel Identity Compatibility
- ADR-0076 — Host Vault on macOS
- ADR-0077 — i18n Russian and English Interface
- ADR-0078 — Frontend Component Decomposition
