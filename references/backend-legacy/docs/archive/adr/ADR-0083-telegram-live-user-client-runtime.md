# ADR-0083 Telegram Live User Client Runtime

Status: Proposed

## Context

ADR-0050 made Telegram a first-class communication channel, but the implemented
V4 surface is still a foundation: fixture accounts, QR authorization setup,
policy dry-runs, call metadata and transcript storage. A usable Telegram client
needs a live `telegram_user` runtime that can keep TDLib state warm, sync chats,
sync selected history, send user-confirmed messages and fetch media on demand.

This must not turn Telegram into the source of truth. Макошь remains
local-first and event-backed: Telegram provider data is preserved as source
evidence, while canonical messages, graph links and task candidates remain local
projections.

## Decision

Implement the first live Telegram user-client slice around an account-scoped
backend TDLib runtime boundary.

- `telegram_user` accounts use a Rust backend runtime manager with one actor per
  account. The actor owns TDLib receive-loop state and serializes commands for
  chat sync, selected-chat history sync, manual sends and media downloads.
- CI and local smoke tests may use a fixture runtime through the same API shape.
  Fixture runtime support must remain available even when native TDLib is not
  installed.
- `telegram_bot` accounts remain setup-compatible, but Bot API live runtime is a
  later slice.
- All chat metadata may be synced for configured user accounts. Message history
  and media are synced deeply only for selected or pinned chats.
- Manual sends are `provider_write` actions. The UI click is explicit user
  confirmation, but backend policy/audit remains authoritative.
- Automated live sends remain blocked. Existing policy dry-runs continue to use
  the ADR-0052 automation policy model.
- Raw provider records are append-only and idempotent. Canonical
  `communication_messages` remain projections from source records.
- TDLib local state and downloaded media bytes stay under ignored local data
  paths. PostgreSQL stores metadata, provenance, checkpoints, attachment records,
  hashes and local blob references only.
- On-demand Telegram media uses the existing communication attachment and safety
  scanner boundary. A provider-neutral facade may wrap the current mail-named
  blob store; table renaming is not part of this slice.
- Live calls, desktop audio capture and real speech-to-text remain blocked
  capabilities until separate runtime, permission and validation work exists.

## Consequences

Positive:

- Telegram can become a usable desktop workbench without making provider state
  canonical.
- CI can validate command, projection, audit and media behavior without live
  Telegram credentials.
- Manual sends get the same backend authority and audit posture as future
  automation and destructive actions.

Negative:

- TDLib update handling becomes long-lived runtime infrastructure instead of a
  short QR-login helper.
- Runtime status, degraded states and account lifecycle must be visible in UI.
- Media sync requires size, storage and scanner discipline from the first slice.

Risk handling:

- Do not report live TDLib send/sync capability as `available` unless the native
  runtime, account authorization, command path and opt-in smoke validation exist.
- Do not store Telegram API hashes, bot tokens, session encryption keys, message
  bodies or media bytes in audit records.
- Do not auto-download all media by default.
- Do not allow AI or automation to choose destinations, accounts, templates or
  live-send authority.

## Non-Goals

- Telegram Bot API live runtime.
- Automated live Telegram sends.
- Message edit, delete, forward, pin or reaction parity.
- Video calls, group calls or screen sharing.
- Hidden recording or cloud transcription by default.
- Mobile UI.
