# Zoom Runtime API

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

The runtime routes below are implemented for the Zoom provider foundation.
Fixture accounts can transition through metadata lifecycle states. Live
accounts are blocked until authorization. Authorized live runtimes can then be
started, expose `running` runtime metadata and participate in the scheduled
recording-sync worker.

## Webhook subscription status

```http
GET /api/v1/integrations/zoom/webhook-subscriptions/status?account_id=zoom_live_primary
```

Behavior:

```text
- requires HostVault to be unlocked;
- requires an authorized OAuth user or Server-to-Server account;
- requires a bound HostVault-backed zoom_webhook_secret reference;
- exchanges an app-owned access token for provider-side webhook subscription management;
- returns the currently managed subscription id plus the remote subscription list.
```

## Webhook subscription reconcile

```http
POST /api/v1/integrations/zoom/webhook-subscriptions/reconcile
```

Request:

```json
{
  "account_id": "zoom_live_primary",
  "endpoint_url": "https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks",
  "subscription_name": "Макошь Zoom Runtime",
  "event_types": [
    "meeting.started",
    "meeting.ended",
    "meeting.participant_joined",
    "meeting.participant_left",
    "recording.completed"
  ]
}
```

Behavior:

```text
- OAuth user accounts exchange client_credentials for app-owned management tokens;
- Server-to-Server accounts exchange account_credentials for app-owned management tokens;
- when the managed subscription already matches endpoint + event types, response status is unchanged;
- when the managed subscription is missing or stale, Макошь deletes the stale managed subscription and recreates it.
```

## Webhook subscription remove

```http
POST /api/v1/integrations/zoom/webhook-subscriptions/remove
```

Request:

```json
{
  "account_id": "zoom_live_primary"
}
```

## Manual provider sync for cloud recordings

```http
POST /api/v1/integrations/zoom/provider-sync/recordings
```

Request:

```json
{
  "account_id": "zoom_live_primary",
  "from": "2026-06-01",
  "to": "2026-06-30",
  "page_size": 30,
  "max_meetings": 100,
  "user_id": "me"
}
```

Behavior:

```text
- requires HostVault to be unlocked;
- requires an authorized OAuth user or Server-to-Server account;
- calls Zoom cloud recording APIs with the stored bearer token;
- upserts meeting call evidence through the existing provider-neutral call store;
- emits recording events through the existing Zoom event contract;
- best-effort imports non-transcript recording media files exposed as recording
  files only when application setting
  privacy.zoom_remote_recording_download_enabled is true;
- best-effort imports transcript-like text files exposed as recording files only
  when application setting privacy.zoom_remote_transcript_download_enabled is true.
```

Target response:

```json
{
  "account_id": "zoom_live_primary",
  "user_id": "me",
  "from": "2026-06-01",
  "to": "2026-06-30",
  "meetings_seen": 4,
  "meetings_recorded": 4,
  "recordings_recorded": 7,
  "media_downloads_recorded": 1,
  "transcripts_recorded": 2,
  "failures": []
}
```

## Status by query

```http
GET /api/v1/integrations/zoom/runtime/status?account_id=zoom_fixture_primary
```

## Status by account path

```http
GET /api/v1/integrations/zoom/accounts/zoom_fixture_primary/runtime/status
```

Both routes return the same `ZoomRuntimeStatus` shape.

## Start runtime

```http
POST /api/v1/integrations/zoom/runtime/start
```

Request:

```json
{
  "account_id": "zoom_fixture_primary",
  "force": false
}
```

Fixture behavior:

```text
lifecycle_state -> running
runtime_kind    -> zoom_fixture_runtime
healthy         -> true
```

Live blocked behavior:

```text
lifecycle_state -> blocked
runtime_kind    -> zoom_live_blocked_runtime
healthy         -> false
runtime_blockers includes zoom_live_authorization_required
```

Authorized live behavior:

```text
lifecycle_state -> running
runtime_kind    -> zoom_live_authorized_runtime
healthy         -> true
live_runtime_available -> true
runtime_blockers empty unless token rotation or another runtime error is active
```

Emits:

```text
zoom.runtime.status_changed
```

Provenance action:

```text
zoom.runtime.start_requested
```

## Stop runtime

```http
POST /api/v1/integrations/zoom/runtime/stop
```

Request:

```json
{
  "account_id": "zoom_fixture_primary",
  "reason": "manual_validation_complete"
}
```

If account lifecycle is not `removed`, stop updates lifecycle to `stopped` and
emits `zoom.runtime.status_changed`.

## Remove runtime

```http
POST /api/v1/integrations/zoom/runtime/remove
```

Request:

```json
{
  "account_id": "zoom_fixture_primary",
  "reason": "fixture_cleanup"
}
```

Target response:

```json
{
  "account_id": "zoom_fixture_primary",
  "provider_kind": "zoom_user",
  "removed": true,
  "removed_at": "2026-06-27T00:00:00Z"
}
```

Removal is a lifecycle mark in provider account config. It is not a destructive
delete.

## Runtime status fields

| Field | Meaning |
|---|---|
| `account_id` | Макошь provider account id. |
| `provider_kind` | `zoom_user` or `zoom_server_to_server`. |
| `runtime_kind` | `zoom_fixture_runtime`, `zoom_live_blocked_runtime` or `zoom_live_authorized_runtime`. |
| `status` | Derived lifecycle status. |
| `healthy` | True for non-removed fixture runtime and for started authorized live runtimes without blockers. |
| `auth_shape` | `fixture`, `oauth_user`, `server_to_server`. |
| `live_runtime_available` | True only after live authorization is completed. |
| `recording_ingest_available` | False only when removed. |
| `transcript_ingest_available` | False only when removed. |
| `runtime_blockers` | Blocking reasons for live execution such as authorization or token-rotation failures. |
| `last_error` | Last recorded runtime error, if any. |
| `metadata` | Provider account config payload plus runtime-derived policy metadata, including the last recording-sync outcome when provider sync has run. |

`metadata.token_rotation_policy` is present on runtime status responses. It
contains the explicit refresh threshold, maintenance refresh threshold, maximum
accepted threshold, provider expiry safety margin, refresh due state, expiry
state, last refresh status and the `zoom_token_rotation_required` blocker name.
The blocker is exposed when a live authorized account has an expired token,
missing token binding or failed last refresh.
