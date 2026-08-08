# Zoom Provider Architecture

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

Zoom is a provider runtime integration. It observes provider facts and
translates them into Макошь evidence. It does not own product meaning.

Current repository state: the foundation architecture below is implemented in
this checkout; ADR-0102 is `Accepted` after target backend and frontend zoom
validation.

## Ownership model

| Area | Target owner | Notes |
|---|---|---|
| Provider account metadata | `integrations/zoom` through provider account port | Stores account kind, display name, external account id and config metadata. |
| Runtime lifecycle | `integrations/zoom` | Stores fixture/running/stopped/blocked/removed status in provider account config. |
| Call evidence | `platform/calls` | Zoom meeting observations are upserted as provider-neutral calls. |
| Transcript evidence | `platform/calls` | Zoom transcript observations are upserted as call transcripts. |
| Canonical events | `platform/events` | Zoom emits canonical envelopes and broadcasts realtime events. |
| Business interpretation | Workflows/domains/engines | Radar, Calendar, Communications, Documents, Tasks, Personas and Knowledge consume downstream evidence. |

There is no `domains/zoom`.

## Inbound flow

```text
Zoom observation source
  -> runtime bridge request
  -> integrations/zoom validation
  -> provider account verification
  -> call/transcript evidence persistence when applicable
  -> sanitized zoom.* event append
  -> realtime broadcast
  -> Signal Hub / workflows / projections
```

The first bridge is local/runtime-safe. A protected account-scoped webhook
bridge validates Zoom URL validation requests and signed meeting/recording
webhooks before normalizing them into the same bridge methods. The
`makosh-zoom-edge-proxy` binary provides public/edge forwarding while preserving
the raw body and Zoom headers for protected bridge verification.

## Runtime shapes

```text
fixture
  -> runtime_kind: zoom_fixture_runtime
  -> lifecycle_state: fixture_ready | running | stopped | removed
  -> live_runtime_available: false

live blocked
  -> runtime_kind: zoom_live_blocked_runtime
  -> auth_shape: oauth_user | server_to_server
  -> lifecycle_state: blocked | stopped | removed
  -> runtime_blockers: [zoom_live_authorization_required]

live authorized
  -> runtime_kind: zoom_live_authorized_runtime
  -> auth_shape: oauth_user | server_to_server
  -> lifecycle_state: authorized | running | stopped | removed
  -> live_runtime_available: true
  -> runtime_blockers: []
```

Live account metadata can exist before authorization. Authorization stores
token bundles and client secrets in HostVault and marks the account authorized.
Explicit refresh, maintenance scans and scheduled token maintenance can renew
those token bundles. Authorized accounts can also reconcile managed Zoom app
event subscriptions through provider APIs. Started authorized runtimes can then
participate in the recording-sync worker, including best-effort recording media
downloads after explicit owner-visible opt-in.

## Event contract

Zoom events use canonical event envelopes with:

- stable `event_id` generated from event kind, account id, subject id and
  optional observation id;
- stable `event_type`;
- `source` identifying Zoom provider and account;
- `subject` identifying meeting, recording, transcript or runtime account;
- sanitized `payload`;
- `provenance` describing bridge source and storage effect;
- optional `causation_id`;
- `correlation_id` defaulting to request correlation id, observation id or
  event id.

Supported target event types:

```text
zoom.runtime.status_changed
zoom.meeting.observed
zoom.recording.observed
zoom.transcript.observed
```

## Evidence persistence

### Meetings

`ZoomMeetingObservationRequest` should be converted to a provider-neutral
`NewProviderCall` using a stable call id derived from account id and meeting id.

```text
stable_zoom_call_id(account_id, meeting_id)
  -> zoom_call_<sha256>
```

The call metadata keeps Zoom-specific context such as meeting id, meeting UUID,
topic, host email, join URL, duration, participants, recording references and
transcript reference.

### Recordings

Recording observations are event-sourced, and both the authorized
recording-sync worker and signed `recording.completed` webhook path can
best-effort import non-transcript recording media files after
`privacy.zoom_remote_recording_download_enabled` is enabled. Imported media is
stored through the local communication blob store, recorded as attachment-import
metadata and passed through the heuristic attachment safety scanner.
Retention/audit surfaces remain later phases.

### Transcripts

Transcript observations create or reuse a placeholder provider call and then
persist a call transcript with:

```text
stt_provider: zoom-cloud-transcript
transcript_status: succeeded
source_audio_ref: source_recording_ref
segments: provider/runtime supplied segments
provenance: zoom meeting metadata
```

Already obtained VTT, SRT and plain text transcript file contents can be
imported through `/runtime-bridge/transcript-files`. Provider-side transcript
file download from Zoom recording references is implemented for signed
`recording.completed` webhooks and authorized provider-sync after explicit
owner-visible opt-in.

## Sanitization boundary

Before an event is appended or broadcast, token-like fields are removed
recursively:

```text
access_token
refresh_token
token
client_secret
webhook_secret
download_token
password
```

This is a runtime safety boundary, not a substitute for secret storage policy.
Credentials must be stored through secret references.

## Authorization boundary

OAuth user grants and Server-to-Server account credentials are exchanged inside
the integration boundary. Raw provider credential payloads are written to
HostVault, while PostgreSQL stores `secret_references`,
provider-account-secret bindings and non-secret account metadata only.

The authorization boundary can explicitly refresh OAuth user credentials using
the stored refresh token, renew Server-to-Server credentials using account
credentials, scan authorized accounts for expiring token bundles, reconcile
managed provider webhook subscriptions, and schedule maintenance through
backend bootstrap. It also powers the authorized recording-sync worker that
downloads recording metadata, selected media files and transcript-like text
files through explicit opt-in policy gates. Authorized accounts remain degraded
until broader provider workers and downstream operational surfaces are
implemented and validated.

## Downstream interpretation

Zoom does not create business entities directly. The intended downstream flow
is:

```text
zoom.meeting.observed
  -> Calls projection
  -> Calendar matching workflow
  -> participant identity candidate workflow
  -> Radar signal detection
  -> Review promotion
  -> target domain command
```

Example target outcomes:

- meeting preparation context pack;
- follow-up task candidate;
- Persona identity trace;
- organization relationship evidence;
- document note from transcript;
- knowledge graph edge after review.

## Safety invariants

- No hidden recording.
- No automatic meeting joining without explicit owner-visible setup.
- No raw Zoom credentials in domain models.
- No raw secrets in event payloads.
- No AI extraction as source of truth.
- No provider-specific business UI cache for canonical communication data.
