# ADR-0102 Zoom Provider Runtime Boundary

Status: Accepted
Date: 2026-06-27

## Context

Макошь integrates communication providers without turning each provider into a
product domain. Zoom contains meetings, recordings, participants, transcripts
and webhook/runtime concerns, but Макошь must treat those as provider
observations that feed memory and context.

Макошь already follows the provider/channel rule for Telegram and WhatsApp:
provider-specific runtime code belongs under integrations, while product
meaning belongs to provider-neutral domains, workflows and engines.

This ADR defines the boundary for the Zoom foundation implementation.
Validation in this checkout is complete:

- Backend targeted suite:
  `CARGO_TARGET_DIR=target/zoom-verify ./scripts/test/run-nextest.sh integration --test zoom_provider_foundation --test zoom_signal_detection --test zoom_calendar_matching --test zoom_participant_identity`
  (27 passed, 0 failed).
- Backend static checks:
  `make backend-fmt-check`, `make backend-clippy`,
  `node scripts/check-architecture.mjs`, `node scripts/check-code-boundaries.mjs`,
  `git diff --check`.
- Frontend targeted checks:
  `cd frontend && pnpm lint`, `cd frontend && pnpm typecheck`,
  `cd frontend && pnpm exec vitest run src/integrations/zoom/api/zoom.test.ts src/integrations/zoom/queries/zoomQueryKeys.test.ts src/platform/bootstrap/realtimeZoomInvalidation.test.ts src/domains/communications/api/callApi.test.ts`.

## Decision

Zoom lives under:

```text
backend/src/integrations/zoom
frontend/src/integrations/zoom
```

Zoom integration may:

- manage provider account metadata;
- expose runtime status and local lifecycle controls;
- register fixture accounts for deterministic validation;
- register live account metadata and secret references in blocked mode;
- exchange OAuth user and Server-to-Server credentials and store credential
  payloads through HostVault-backed secret references;
- explicitly refresh or renew OAuth user and Server-to-Server token bundles
  through HostVault-backed secret references;
- scan authorized accounts and refresh expiring token bundles through the same
  HostVault-backed secret boundary;
- run a configurable local scheduler that invokes the token maintenance scan
  without exposing raw token material;
- expose token rotation policy metadata, refresh due state and
  failure/expiry blockers in runtime status without exposing raw token
  material;
- accept runtime-bridge observations for meetings, recordings and transcripts;
- persist call/transcript evidence through platform call intelligence
  primitives;
- emit `zoom.*` events with canonical envelope metadata;
- sanitize token-like fields before event append/broadcast.

Zoom integration must not:

- become `domains/zoom`;
- create tasks, Personas, notes, organizations, documents or calendar events
  directly;
- call business domains to mutate state;
- store raw secrets in provider account config or event payloads;
- treat AI extraction as source of truth;
- perform hidden recording, hidden transcription or automatic meeting joining
  without explicit owner-visible setup.

## Event contract

Zoom emits:

```text
zoom.authorization.completed
zoom.runtime.status_changed
zoom.token.refreshed
zoom.token.refresh.skipped
zoom.token.refresh.failed
zoom.meeting.observed
zoom.recording.observed
zoom.transcript.observed
zoom.recording.import.removed
zoom.transcript.removed
zoom.retention.cleanup.completed
```

Events must use canonical envelopes and preserve causation/correlation when
supplied.

## Consequences

Zoom meeting evidence can flow into Calls, Calendar preparation, Radar, Review,
Tasks and Knowledge through events/workflows. The provider adapter remains
replaceable: additional provider workers can be added later without changing
domain ownership.

Live provider execution now includes the authorized recording-sync worker,
including privacy-gated recording media download/import and transcript-like file
download/import, explicit owner-visible recording import removal and explicit
retention cleanup for expired recording/transcript evidence, including local
scheduled retention cleanup automation. It remains partial until downstream
workflow boundaries are explicitly implemented.
