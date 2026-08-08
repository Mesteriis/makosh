# Zoom Provider Runtime Research Notes

Status date: 2026-06-27.

This document records Макошь-side runtime shape decisions. It is not a vendor
API reference. Before implementing live provider calls, verify the current Zoom
documentation and update this file with source links and exact constraints.

## Runtime shapes represented in Макошь

Макошь should model three account/auth shapes:

```text
fixture
  local deterministic validation

oauth_user
  user-authorized account shape, live runtime initially blocked

server_to_server
  account/server authorization shape, live runtime initially blocked
```

Provider kind mapping:

```text
fixture          -> zoom_user
oauth_user       -> zoom_user
server_to_server -> zoom_server_to_server
```

## Target adapters

A future live implementation can add adapters behind the same store methods:

```text
Webhook Adapter
  -> verify signature
  -> normalize payload
  -> call ZoomStore.observe_meeting / observe_recording / observe_transcript

Recording Worker
  -> resolve recording reference through secret-safe provider client
  -> persist local blob/document evidence
  -> emit recording/document events

OAuth Runtime
  -> store/refresh credentials through secret references
  -> expose health and blockers through runtime status
```

## Non-negotiable Макошь rules

- The adapter must never import business domains.
- The adapter must never create tasks, Personas, organizations, documents or
  calendar events directly.
- The adapter must use events/workflows for downstream effects.
- The adapter must preserve causation and correlation ids when bridging provider
  events.
- The adapter must verify webhook provenance before persistence.
- The adapter must never place raw secrets in provider config, metadata or event
  payloads.

## Open research questions

These must be answered against current provider documentation before live
runtime work:

1. What exact authorization flows and scopes are required for meeting,
   recording and transcript metadata?
2. What webhook events are required for meeting lifecycle, recording lifecycle
   and transcript availability?
3. What signature verification mechanism and replay window should be enforced?
4. What are the retention and download constraints for recordings and
   transcripts?
5. What rate limits affect historical sync and recording downloads?
6. What user-visible consent and audit behavior is required for
   recording/transcript processing?
7. How should failed downloads and partial transcript imports be retried and
   DLQ-backed?

## Documentation update requirement

When live runtime is implemented, update:

```text
docs/integrations/zoom/architecture.md
docs/integrations/zoom/api.md
docs/integrations/zoom/status.md
docs/integrations/zoom/gap-analysis.md
docs/integrations/zoom/blockers.md
docs/integrations/zoom/live-smoke-checklist.md
docs/archive/adr/ADR-0102-zoom-provider-runtime-boundary.md or a successor ADR
```
