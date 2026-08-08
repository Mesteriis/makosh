# Макошь Platform Layer

Status: documentation package aligned to the current repository structure.

The platform layer mirrors `backend/src/platform`.

Platform modules provide technical primitives used by domains, integrations,
workflows and the app layer. Platform code owns infrastructure contracts, not
business source-of-truth entities.

## Current Documentation Packages

- [Event Tracing](event-tracing/README.md)
- [Application Settings](settings/README.md)
- [Realtime Conversation](realtime-conversation/README.md)

## Current Code Areas

- `platform/events` - event store, event bus, trace context and dispatch.
- `platform/audit` - local audit records.
- `platform/config` - application/runtime configuration parsing.
- `platform/settings` - allowlisted application settings.
- `platform/secrets` - secret reference metadata and resolver contracts.
- `platform/storage` - local storage primitives.
- `platform/calls` - provider-neutral call/transcript evidence primitives.
- `platform/realtime_conversation` - provider-neutral live-conversation
  bundle and provider capability contracts.

## Documentation Rule

Use this folder for reusable technical contracts. If a document starts owning
Tasks, Personas, Communications, provider sessions or product decisions, move
that content to the owning domain, integration or workflow package.
