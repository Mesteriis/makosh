# Zoom Integration

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-27.

Zoom is modeled as an external communication provider adapter, not as a Макошь
domain. Business memory remains in canonical domains and engines:
Communications, Calls, Calendar, Radar, Timeline, Documents and AI extraction.

Current repository state: the foundation integration is implemented in this
checkout, including account metadata, runtime lifecycle state, runtime bridge
ingestion, live authorization token exchange, protected webhook verification
and event publication.
The integration settings UI also exposes a local runtime bridge lab for
meeting, recording, transcript and transcript-file ingestion against the
selected Zoom account.
The same integration surface now exposes read-only observed call evidence and
projected transcript inspection for the selected Zoom account through the
provider-neutral call APIs.

## Boundary

```text
External Zoom
  -> integrations/zoom
  -> zoom.*.observed events
  -> Calls / Communications / Calendar workflows
  -> Radar / Review / target domains
```

The integration owns provider account metadata, runtime lifecycle state,
sanitized runtime bridge ingestion and event publication. It must not own tasks,
Personas, organizations, notes, documents or calendar truth.

## Runtime shape

The implemented foundation is intentionally conservative:

- fixture Zoom accounts can be registered for local validation;
- live OAuth and server-to-server account metadata can be registered;
- OAuth user and Server-to-Server credentials can be exchanged and stored
  through HostVault-backed secret references;
- OAuth user and Server-to-Server token bundles can be explicitly refreshed or
  renewed in HostVault;
- authorized accounts can be scanned by token maintenance for expiring bundle
  renewal;
- started authorized live accounts can participate in the scheduled
  recording-sync worker;
- meeting observations are projected into provider call evidence;
- recording observations are event-sourced and sanitized;
- transcript observations are stored in call transcripts and linked to the
  meeting evidence;
- already obtained VTT, SRT and plain text transcript file contents can be
  imported into the same call transcript evidence path;
- signed `recording.completed` payloads can auto-download non-transcript
  recording media files through `download_url` plus `download_token` and store
  them through the local communication blob pipeline after explicit privacy
  opt-in;
- signed `recording.completed` payloads can auto-download transcript-like text
  files through `download_url` plus `download_token` and import them into the
  same transcript evidence path;
- account-scoped webhook URL validation and signed meeting/recording webhook
  normalization are supported on the protected runtime bridge;
- all cross-boundary output uses canonical event envelopes with
  causation/correlation support.

## Target backend routes

```text
GET  /api/v1/integrations/zoom/capabilities
GET  /api/v1/integrations/zoom/accounts
POST /api/v1/integrations/zoom/accounts
POST /api/v1/integrations/zoom/fixtures/accounts
POST /api/v1/integrations/zoom/oauth/start
POST /api/v1/integrations/zoom/oauth/complete
POST /api/v1/integrations/zoom/oauth/server-to-server/authorize
POST /api/v1/integrations/zoom/oauth/refresh
POST /api/v1/integrations/zoom/oauth/maintenance
POST /api/v1/integrations/zoom/provider-sync/recordings
GET  /api/v1/integrations/zoom/webhook-subscriptions/status?account_id=...
POST /api/v1/integrations/zoom/webhook-subscriptions/reconcile
POST /api/v1/integrations/zoom/webhook-subscriptions/remove
GET  /api/v1/integrations/zoom/runtime/status?account_id=...
GET  /api/v1/integrations/zoom/accounts/{account_id}/runtime/status
GET  /api/v1/integrations/zoom/accounts/{account_id}/recording-imports?limit=...
POST /api/v1/integrations/zoom/accounts/{account_id}/recording-imports/{attachment_id}/remove
GET  /api/v1/integrations/zoom/accounts/{account_id}/audit-events?limit=...
POST /api/v1/integrations/zoom/runtime/start
POST /api/v1/integrations/zoom/runtime/stop
POST /api/v1/integrations/zoom/runtime/remove
POST /api/v1/integrations/zoom/runtime-bridge/meetings
POST /api/v1/integrations/zoom/runtime-bridge/recordings
POST /api/v1/integrations/zoom/runtime-bridge/transcripts
POST /api/v1/integrations/zoom/runtime-bridge/transcript-files
POST /api/v1/integrations/zoom/runtime-bridge/webhooks?account_id=...
```

## Target events

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
```

Secrets and token-like fields must be stripped from event payloads before
append/broadcast.

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

These are references only. Макошь domains store references and lifecycle state,
never raw Zoom credentials.
