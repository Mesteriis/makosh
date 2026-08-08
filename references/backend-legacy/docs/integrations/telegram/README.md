# Макошь Communications - Telegram Channel

Status: `COMPLETED` base channel capability set, 2026-06-18.

Telegram in Макошь is a Communication Channel inside Макошь Communications. It
does not own Memory, Knowledge, Persona, Organization, Project, Obligation or
Decision lifecycle. Telegram supplies source evidence, provider commands,
communication projections, realtime events, identity traces, timeline evidence
and media evidence for other systems.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

```text
Telegram Provider
  -> Source Evidence
  -> Communication Projection
  -> Realtime Events
  -> Timeline Evidence
  -> Shared Engines
```

## Completed Boundary

The base Telegram channel capability set is complete for daily desktop work:

- account setup, QR-authorized TDLib user runtime metadata and runtime health;
- provider-write outbox, command status, retry/dead-letter visibility and
  provider-observed reconciliation;
- dialog pin/archive/mute/read/unread/folder commands and realtime patches;
- message edit/delete/pin/reaction/reply/forward lifecycle evidence;
- edit versions, tombstones, provider edit/delete evidence and diff metadata;
- reply and forward attribution with bounded chain traversal and cycle guards;
- forum topic projection, unread state, realtime topic updates and command
  reconciliation;
- provider-refreshed message/media search with projection-backed UI results;
- media gallery, album metadata, preview/download/upload lifecycle through the
  shared Communication attachment boundary;
- frontend server state through TanStack Query composables and shared realtime
  bootstrap, without component-level fetches.

Provider ACK is not treated as success. Commands complete only from
provider-observed state or returned provider snapshots that have been projected
locally.

## Deferred Initiatives

The following are intentionally outside the base Telegram channel capability set
and are tracked as `planned` capabilities by ADR-0094/ADR-0097:

- Bot Runtime;
- Voice Recording;
- Voice Send;
- Video Recording;
- Live Calls;
- Session Export;
- Session Import;
- MTProxy;
- SOCKS5;
- AI Summary;
- Translation;
- Bilingual Reply;
- AI Review Flows.

Hidden recording, fine-tuning on private Telegram data and untrusted plugin
execution remain unsupported.

## Navigation

- [Architecture](architecture.md)
- [Modules](modules.md)
- [API Reference](api.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
- [Product Research](product-research.md)
