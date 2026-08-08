# Макошь Realtime Conversation Platform

Status: `TARGET_ARCHITECTURE`, 2026-06-28.

The Realtime Conversation Platform is the provider-neutral layer for live
conversations in Макошь. Zoom, Yandex Telemost, Google Meet, Jitsi, Discord and
future call providers are external systems. They do not own Макошь memory. They
only provide runtime access, links, local capture opportunities and provider
evidence.

Макошь owns the durable memory object:

```text
Live conversation
↓
Call Bundle
↓
Transcription / diarization / speaker identity
↓
Call Intelligence
↓
Timeline / Radar / Documents / Knowledge Graph / Tasks
```

This keeps provider integrations thin and reusable. The durable value is the
evidence-backed memory of what was said, who said it, what was decided, what
was promised and which context it belongs to.

## Provider-neutral invariants

- Provider integrations never become domains.
- A provider conference is evidence, not the source of truth.
- Local recording is explicit, visible and consent-gated.
- WebView speaker state is a hint, not truth.
- AI produces candidates with source, confidence and evidence.
- Domain mutations go through workflows/events, not direct integration calls.
- Reprocessing must be possible when better models appear later.

## Documents

- [Architecture](./architecture.md)
- [Recording bundle](./recording-bundle.md)
- [Call intelligence](../../engines/call-intelligence/README.md)
- [Speaker identity](../../engines/speaker-identity/README.md)
- [Providers](./providers.md)
- [Replay and live notes](./replay-and-live-notes.md)
