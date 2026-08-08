# Zoom Implementation Status

Status date: 2026-06-28.

Documentation status: `TARGET_DOCUMENTED`.

Implementation status in this checkout:
`FOUNDATION_IMPLEMENTED`.

ADR status: `ADR-0102` is `Accepted` in this checkout.
The foundation implementation artifacts are present and backend targeted
validation has passed in this environment.

Invariant: Zoom is a provider integration. Zoom does not own business domains.

## Current repository evidence

As of this implementation pass, the current checkout contains:

- `backend/src/integrations/zoom`;
- `frontend/src/integrations/zoom`;
- `backend/src/bin/makosh_zoom_edge_proxy.rs`;
- migration `backend/migrations/0160_add_zoom_provider_kind.sql`;
- `/api/v1/integrations/zoom/*` routes;
- Zoom event constants under the platform event bus.
- account-scoped verified webhook bridge for URL validation, signed meeting
  events and signed recording events.
- public/edge webhook proxy that forwards raw Zoom webhook bodies and
  `x-zm-*` headers to the protected runtime bridge.
- live OAuth user and Server-to-Server authorization boundary that exchanges
  provider tokens and stores credential payloads in HostVault.
- token refresh/renew route that updates OAuth user and Server-to-Server token
  bundles in HostVault without exposing raw access tokens.
- token maintenance route that scans authorized accounts and renews expiring
  HostVault token bundles through the same refresh boundary.
- scheduled token maintenance daemon that invokes the same HostVault-backed
  maintenance boundary through the local background-service bootstrap.
- manual provider-sync route for authorized Zoom cloud recordings that records
  meeting/recording evidence and best-effort imports recording media plus
  transcript-like text files through the same bearer-token boundary.
- scheduled recording sync daemon that periodically syncs recent cloud
  recording metadata for started authorized runtimes through the same
  HostVault-backed provider-sync boundary.
- live webhook subscription status/reconcile/remove routes that use app-owned
  access tokens plus HostVault-backed webhook secret bindings without exposing
  raw provider secrets.
- declared privacy setting `privacy.zoom_remote_recording_download_enabled`
  with backend enforcement for provider-side recording media downloads.
- declared privacy setting `privacy.zoom_remote_transcript_download_enabled`
  with backend enforcement for provider-side transcript-like file downloads.
- declared retention settings `privacy.zoom_recording_import_retention_days`
  and `privacy.zoom_transcript_retention_days`, with stamped retention metadata
  on imported recording blobs and transcript evidence provenance.
- transcript file import route for already obtained VTT, SRT and plain text
  transcript files.
- signed recording webhook downloader that imports non-transcript recording
  media files and transcript-like textual recording files from Zoom
  `download_url` plus `download_token`.
- read-only recording import audit route plus settings-panel inspection for
  imported Zoom recording blobs.
- explicit recording import retention/remove route plus settings-panel control
  for deleting imported Zoom recording blobs from local audit/storage state.
- explicit retention cleanup route plus settings-panel control for pruning
  expired imported Zoom recording blobs and expired transcript evidence using
  owner-visible retention settings.
- scheduled retention cleanup daemon that periodically prunes expired imported
  Zoom recording blobs and expired transcript evidence through the same local
  retention boundary.
- read-only Zoom event audit route plus settings-panel inspection for recent
  authorization, refresh/maintenance, runtime and bridge events on the
  selected account.
- downstream Zoom signal-detection workflow that projects
  `zoom.meeting.observed`, `zoom.recording.observed` and
  `zoom.transcript.observed` into Signal Hub `signal.raw.zoom.*` events and
  policy-derived `signal.accepted|muted|paused.zoom.*` detection events.
- provider-neutral Communications `calls` and `meetings` sections that read the
  shared call evidence store and surface projected Zoom meeting/transcript
  evidence, participant snapshots and recording references outside the
  integration settings surface.

Targeted Zoom backend validation passes in this environment:
`CARGO_TARGET_DIR=target/zoom-verify ./scripts/test/run-nextest.sh integration --test zoom_provider_foundation --test zoom_signal_detection --test zoom_calendar_matching --test zoom_participant_identity`
(`27` tests passed, `0` failed).
Frontend targeted checks do pass:
`pnpm exec vitest run src/integrations/zoom/api/zoom.test.ts src/integrations/zoom/queries/zoomQueryKeys.test.ts src/platform/bootstrap/realtimeZoomInvalidation.test.ts src/domains/communications/api/callApi.test.ts`.
Backend/frontend static checks pass:
`make backend-fmt-check`, `make backend-clippy`, `node scripts/check-architecture.mjs`,
`node scripts/check-code-boundaries.mjs`, `pnpm lint`, `pnpm typecheck` and
`git diff --check`.

## Target foundation summary

| Area | Target state | Current repo state |
|---|---|---|
| Integration boundary | Implement under `backend/src/integrations/zoom`, not `domains/zoom`. | Present. |
| Fixture account setup | Deterministic local validation route. | Present and targeted-tested. |
| Live account metadata | Metadata and secret references registered in blocked mode. | Present and targeted-tested. |
| Live authorization | OAuth user and Server-to-Server token exchange with HostVault storage. | Present and targeted-tested. |
| Token refresh | Explicit OAuth refresh-token and Server-to-Server renew route. | Present and targeted-tested. |
| Token maintenance | Scan authorized accounts and refresh expiring token bundles. | Present and targeted-tested. |
| Scheduled token maintenance | Background scheduler invokes token maintenance with Signal Hub and HostVault gates. | Present and targeted-tested. |
| Manual recording provider sync | Authorized accounts can manually fetch Zoom cloud recording metadata and best-effort recording media plus transcript-like text files through the integration boundary. | Present and targeted-tested. |
| Webhook subscription management | Authorized accounts can query, reconcile and remove managed Zoom app event subscriptions through live provider APIs using app-owned access tokens. | Present and targeted-tested. |
| Remote transcript download consent policy | Provider-side transcript-like file downloads require explicit owner-visible opt-in through application settings. | Present and targeted-tested. |
| Runtime status | Query and account-path status routes. | Present and targeted-tested. |
| Runtime start/stop/remove | Lifecycle state updates and runtime status events. | Present and targeted-tested. |
| Meeting bridge | Upsert provider-neutral call evidence and emit event. | Present and targeted-tested. |
| Recording bridge | Emit sanitized recording evidence event. | Present and targeted-tested. |
| Transcript bridge | Upsert call transcript and emit event. | Present and targeted-tested. |
| Transcript file import | Parse VTT/SRT/plain transcript file text into call transcript evidence and auto-import transcript-like text files from signed recording webhooks. | Present and targeted-tested. |
| Recording webhook media import | Best-effort import non-transcript recording media files from signed recording webhooks after explicit owner-visible opt-in, local blob persistence and heuristic safety scan. | Present and targeted-tested. |
| Recording import audit view | Read-only route and settings-panel inspection for imported Zoom recording blobs, including source, scan status and storage metadata. | Present and targeted-tested. |
| Recording import retention control | Explicit per-import remove route and settings-panel control that deletes imported Zoom recording blobs from local audit/storage state and emits a follow-up audit event. | Present and targeted-tested. |
| Owner-visible retention policy | Application settings define retention days for imported recording blobs and transcript evidence; imports stamp retention mode and expiry intent into audit/provenance metadata. | Present and targeted-tested. |
| Retention cleanup control | Explicit account-scoped prune route and settings-panel control remove expired imported recording blobs and expired transcript evidence using stamped retention expiry intent. | Present and targeted-tested. |
| Scheduled retention cleanup | Background scheduler periodically prunes expired imported recording blobs and expired transcript evidence through the same local retention boundary. | Present and targeted-tested. |
| Credential lifecycle audit trail | Account-scoped Zoom event log records authorization completion plus token refresh success, skip and failure events, including maintenance-triggered refresh decisions. | Present and targeted-tested. |
| Runtime and bridge event audit view | Read-only route and settings-panel inspection for recent Zoom runtime and bridge events from the account-scoped event log. | Present and targeted-tested. |
| Event contract | Zoom event types registered and canonical envelopes used. | Present. |
| Payload sanitization | Token-like fields stripped recursively before event append/broadcast. | Present and targeted-tested. |
| Signal Hub source | Zoom provider source fixture registered. | Present and targeted-tested. |
| Signal detection workflow | Downstream Signal Hub detection path, not integration ownership. | Implemented and targeted-tested via `zoom.*` -> `signal.raw.zoom.*` -> policy-derived Signal Hub events. |
| Frontend API client/types | Integration API client, types, runtime query and settings UI controls for fixture/live setup, OAuth/S2S authorization, token maintenance, local runtime-bridge ingestion and read-only observed call evidence inspection, including recording references and transcript provenance. | Present and targeted-tested. |
| Provider-neutral Calls/Meetings view | Communications `calls` and `meetings` sections surface shared call evidence, transcript reads, participant snapshots and recording references, including projected Zoom meetings. | Present and targeted-tested. |
| Live provider runtime | Authorized accounts can be started into a running live runtime, and the background recording-sync worker polls recent Zoom cloud recordings through the existing provider-sync boundary. | Present and targeted-tested. |
| Webhook signature verification | Protected account-scoped runtime bridge. | Present and targeted-tested for URL validation, meeting events, recording events and invalid signatures. |
| Public/edge webhook receiver | Standalone `makosh-zoom-edge-proxy`. | Present and targeted-tested for readiness, raw body forwarding, Zoom header forwarding and account scoping. |
| Cloud recording download worker | Authorized accounts can best-effort import provider-side recording media files through the existing recording-sync boundary and signed recording webhooks after explicit owner-visible opt-in, local blob persistence and heuristic safety scan. | Present and targeted-tested. |
| Calendar matching workflow | Downstream workflow, not integration ownership. | Implemented and targeted-tested via `zoom.meeting.observed` -> Calendar event relation projection. |
| Participant identity resolution | Downstream candidate/review workflow. | Implemented and targeted-tested for conservative `attach_email_address` identity-candidate generation from `zoom.meeting.observed` participants into the existing Review inbox pipeline. |

## Target capability state

```text
available in the foundation implementation:
  accounts.fixture
  auth.oauth_user
  auth.server_to_server
  auth.token_refresh
  auth.token_maintenance
  auth.token_rotation_policy
  token_maintenance.scheduler
  provider_sync.recordings
  provider_sync.recordings.scheduler
  provider_sync.recording_media_downloads
  recording_imports.remove
  retention.cleanup
  retention.cleanup.scheduler
  audit.events
  webhooks.subscription_management
  runtime.status
  bridge.meetings
  bridge.recordings
  bridge.transcripts
  bridge.transcript_files
  webhooks.verified
  webhooks.edge_proxy
  calendar_event_matching
  meeting_participant_identity_resolution

degraded in the foundation implementation:
  accounts.live_blocked

unsupported:
  hidden_recording
  joining_meetings_as_a_bot_without_explicit_setup
  auto_dialing
  training_models_on_zoom_content_by_default
```

## Validation note

The Zoom foundation is implemented in code and verified by frontend checks and
backend Zoom foundation tests in this environment. `ADR-0102` is upgraded to
`Accepted` after that proof.

## Navigation

- [Pass Log](status/pass-log.md)
- [Fixture Test Matrix](fixture-test-matrix.md)
- [Live Smoke Checklist](live-smoke-checklist.md)
