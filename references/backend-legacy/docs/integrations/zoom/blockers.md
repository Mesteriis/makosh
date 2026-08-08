# Zoom Blockers

Status date: 2026-06-28.

This document lists blockers that must be resolved before Zoom can move from
foundation authorization and bridge ingestion to live provider runtime workers.

## Foundation blockers

| Blocker | Impact | Required action |
|---|---|---|
| No Zoom integration module | No backend account, runtime or bridge routes can exist. | Add `backend/src/integrations/zoom` behind ADR-0102 boundaries. |
| No provider kind migration | Provider accounts cannot represent Zoom safely. | Add migration for `zoom_user` and `zoom_server_to_server` only after inspecting current constraints. |
| No event constants | Bridge cannot append canonical Zoom events. | Add `zoom.runtime.status_changed`, `zoom.meeting.observed`, `zoom.recording.observed`, `zoom.transcript.observed`. |
| No frontend integration module | Settings/runtime UI cannot call provider setup routes. | Add integration-only API/query/types, not a product Zoom domain. |
| No tests | Boundary and sanitization claims are unverifiable. | Add fixture and negative tests with repository test harness. |

## Live runtime blockers

No current live-runtime blocker remains inside the implemented foundation scope.
The remaining open items are downstream workflow concerns, not missing Zoom
runtime/account plumbing.

## Architecture blockers

### No direct domain mutation

Zoom integration must not call:

```text
domains/calendar
domains/personas
domains/tasks
domains/documents
domains/organizations
domains/communications
```

Cross-domain effects must use events and workflows.

### No provider-specific product domain

Do not add:

```text
backend/src/domains/zoom
frontend/src/domains/zoom
```

Provider setup belongs under integrations. Product views belong under
provider-neutral Calls, Calendar, Communications, Documents, Radar and Knowledge
surfaces.

### No hidden recording

Live meeting capture, local recording, screen capture or transcript generation
requires explicit owner-visible setup and audit. Silent capture is unsupported.

## Security blockers

- Explicit token refresh/renew route. Implemented for OAuth user refresh-token
  and Server-to-Server account-credentials renewal through HostVault-backed
  `zoom_oauth_token` bundles.
- Token maintenance runner. Implemented for authorized account scanning and
  expiring-token renewal.
- Scheduled token maintenance daemon. Implemented through backend bootstrap
  with Signal Hub runtime gating, HostVault unlock gating and
  `MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`.
- Scheduled recording sync daemon. Implemented through backend bootstrap for
  started authorized runtimes, with Signal Hub runtime gating, HostVault unlock
  gating, `MAKOSH_ZOOM_RECORDING_SYNC_SCHEDULER_ENABLED` and the existing
  HostVault-backed provider sync boundary.
- Scheduled retention cleanup daemon. Implemented through backend bootstrap for
  expired imported recording blobs and transcript evidence, with Signal Hub
  runtime gating and `MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED`.
- Token rotation policy. Implemented through explicit refresh thresholds,
  proactive maintenance threshold metadata, expiry handling and
  `zoom_token_rotation_required` failure/expiry blocker exposure in runtime
  status.
- OAuth and Server-to-Server initial token exchange. Implemented through
  HostVault-backed `zoom_oauth_token` secret references.
- Owner-visible privacy opt-in for provider-side transcript-like file
  downloads. Implemented through application setting
  `privacy.zoom_remote_transcript_download_enabled`, enforced for webhook and
  manual provider-sync downloads.
- Webhook secret storage through secret references. Implemented for the
  protected account-scoped runtime bridge; public/edge ingress is implemented
  as `makosh-zoom-edge-proxy`.
- Sanitization test coverage for nested payloads. Implemented for event payload
  and provider-call metadata in the Zoom foundation tests.
- Audit events for authorization completion, token refresh success/skip/failure,
  runtime lifecycle and bridge ingestion. Implemented on the account-scoped
  Zoom audit surface.
- Explicit per-import retention removal for imported recording blobs.
- Owner-visible retention policy settings for imported recordings and
  transcript evidence. Implemented through
  `privacy.zoom_recording_import_retention_days` and
  `privacy.zoom_transcript_retention_days`, with stamped expiry intent in
  audit/provenance metadata.
- Explicit owner-triggered cleanup of expired imported recordings and expired
  transcript evidence. Implemented through the account-scoped retention prune
  route and settings-panel control.

## UI blockers

- Richer Calendar/Communications downstream workflows beyond the current
  provider-neutral Calls/Meetings evidence views.
