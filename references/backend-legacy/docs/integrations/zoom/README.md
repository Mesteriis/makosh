# Макошь Communications - Zoom Provider Stage

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

Implementation evidence in this checkout: foundation code is present under
`backend/src/integrations/zoom` and `frontend/src/integrations/zoom`, with a
Zoom migration, backend routes, targeted backend tests and targeted frontend
tests. ADR-0102 is `Accepted` after target backend and frontend zoom validation.

Zoom in Макошь is an external communication provider adapter. It is not a
Макошь domain, not a meeting CRM and not a calendar source of truth. Zoom can
provide meeting evidence, recording evidence, transcript evidence, provider
account metadata and runtime lifecycle signals.

Invariant: A provider is never a domain. A meeting observation is evidence. The
business object belongs to Calls, Communications, Calendar, Radar, Timeline,
Documents or another owner domain/workflow.

```text
Zoom Provider
  -> Runtime Bridge
  -> Source Evidence
  -> Provider Call Projection
  -> Canonical Events
  -> Shared Workflows and Engines
```

## Foundation scope

The Zoom foundation provides:

- fixture Zoom account setup for deterministic local validation;
- live account metadata setup with `oauth_user` and `server_to_server` auth
  shapes;
- OAuth user and Server-to-Server authorization token exchange with HostVault
  credential storage and PostgreSQL secret-reference bindings only;
- explicit OAuth/S2S token refresh and renewal route that updates HostVault
  token bundles without returning raw access tokens;
- token maintenance route for scanning authorized accounts and refreshing
  expiring HostVault token bundles;
- scheduled token maintenance daemon with Signal Hub runtime gating,
  HostVault unlock gating and `MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`
  operational toggle;
- managed webhook subscription status/reconcile/remove routes for authorized
  accounts, using app-owned access tokens and HostVault-backed webhook secret
  bindings;
- manual provider-sync route for authorized Zoom cloud recordings, including
  provider-neutral meeting/recording evidence ingestion plus best-effort
  recording media and transcript-like text file import;
- owner-visible privacy policy setting
  `privacy.zoom_remote_recording_download_enabled`, which must be enabled
  before Макошь fetches recording media files directly from Zoom;
- owner-visible privacy policy setting
  `privacy.zoom_remote_transcript_download_enabled`, which must be enabled
  before Макошь fetches transcript-like text files directly from Zoom;
- owner-visible retention policy settings
  `privacy.zoom_recording_import_retention_days` and
  `privacy.zoom_transcript_retention_days`, which stamp retention metadata and
  expiry intent onto imported recording blobs and transcript evidence;
- runtime status/start/stop/remove lifecycle controls;
- meeting observation ingestion into provider-neutral call evidence;
- recording observation ingestion as sanitized event evidence;
- transcript observation ingestion into call transcript persistence;
- VTT, SRT and plain text transcript file import into call transcript
  persistence after the file content has already been obtained;
- automatic transcript-like file download/import from signed
  `recording.completed` webhook payloads when Zoom includes textual recording
  files with `download_url`;
- automatic recording media download/import from signed `recording.completed`
  webhook payloads when Zoom includes non-transcript recording files with
  `download_url`;
- protected account-scoped webhook URL validation and signed
  meeting/recording webhook normalization;
- `makosh-zoom-edge-proxy` public/edge ingress that preserves raw Zoom webhook
  bodies and `x-zm-*` headers before forwarding to the protected bridge;
- canonical Zoom events with causation and correlation support;
- provider account listing with optional removed-account visibility;
- frontend API client, query keys and runtime query hook;
- read-only recording import audit route and settings-panel inspection for
  imported Zoom recording blobs;
- explicit recording import remove control for deleting imported Zoom
  recording blobs from local audit/storage state;
- explicit retention cleanup control for pruning expired imported recording
  blobs and expired transcript evidence using stamped retention expiry intent;
- scheduled retention cleanup daemon that periodically prunes expired imported
  recording blobs and transcript evidence through the same local retention
  boundary, gated by `MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED`;
- read-only account-scoped Zoom event audit route and settings-panel inspection
  for recent authorization, refresh/maintenance, runtime and bridge events;
- provider-neutral Communications `calls` and `meetings` sections that read the
  shared call evidence store and surface projected Zoom meeting/transcript
  evidence outside integration settings;
- realtime invalidation coverage for Zoom runtime and observation events;
- Signal Hub source registration for Zoom provider signals;
- downstream Signal Hub detection workflow that turns
  `zoom.meeting.observed`, `zoom.recording.observed` and
  `zoom.transcript.observed` into policy-aware `signal.raw.zoom.*` and derived
  Signal Hub detection events;
- downstream calendar-event matching and participant-identity workflows exposed
  as available capabilities rather than remaining planned-only items;
- ADR coverage for provider runtime boundary.

## Current scope

The stage is intentionally conservative:

```text
target available:
  fixture accounts
  runtime lifecycle metadata
  authorized live recording-sync worker
  recording media download/import through provider sync
  recording media download/import through signed recording webhooks
  recording import audit inspection
  recording import retention/remove control
  runtime, bridge and credential lifecycle audit inspection
  provider-neutral communications calls/meetings evidence view
  webhook subscription management
  meeting bridge ingestion
  recording bridge ingestion
  transcript bridge ingestion
  transcript file import
  protected verified webhook bridge
  public/edge webhook proxy
  OAuth/S2S authorization boundary
  explicit OAuth/S2S token refresh
  token maintenance runner
  scheduled token maintenance daemon
  retention cleanup scheduler
  token rotation policy
  owner-visible opt-in policy for remote transcript downloads
  sanitized event publication

unsupported:
  hidden recording
  automatic meeting joining without explicit setup
  auto-dialing
  model training on Zoom content by default
```

## Provider kinds

```text
zoom_user
zoom_server_to_server
```

## Secret purposes

```text
zoom_oauth_token
zoom_client_secret
zoom_webhook_secret
```

Domains store only references and lifecycle state. Raw credentials stay outside
business models and event payloads.

## Navigation

- [Architecture](architecture.md)
- [API Reference](api.md)
- [Modules](modules.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
- [Implementation Plan](implementation-plan.md)
- [Fixture Test Matrix](fixture-test-matrix.md)
- [Live Smoke Checklist](live-smoke-checklist.md)
- [Provider Runtime Research](provider-runtime-research.md)
- [Runtime Boundary ADR](../../archive/adr/ADR-0102-zoom-provider-runtime-boundary.md)
