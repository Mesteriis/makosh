# Zoom Runtime Bridge API

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

The runtime bridge routes below are implemented for the Zoom provider
foundation. They accept sanitized local/runtime observations and account-scoped
verified webhook notifications without changing downstream event contracts.

## Meeting observation

```http
POST /api/v1/integrations/zoom/runtime-bridge/meetings
```

Request:

```json
{
  "observation_id": "obs_zoom_meeting_001",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "meeting_uuid": "meeting-uuid",
  "topic": "Макошь Zoom Review",
  "host_email": "owner@example.test",
  "join_url": "https://example.invalid/j/987654321",
  "started_at": "2026-06-27T10:00:00Z",
  "ended_at": "2026-06-27T10:30:00Z",
  "duration_seconds": 1800,
  "participants": [
    {
      "participant_id": "p1",
      "display_name": "Owner",
      "email": "owner@example.test",
      "joined_at": "2026-06-27T10:00:00Z",
      "left_at": "2026-06-27T10:30:00Z",
      "metadata": {}
    }
  ],
  "recording_refs": [],
  "transcript_ref": "tr_zoom_001",
  "metadata": {
    "fixture": true
  },
  "causation_id": null,
  "correlation_id": "corr_zoom_fixture_001"
}
```

Target response:

```json
{
  "call_id": "zoom_call_<stable_hash>",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "event_id": "evt_zoom_meeting_<stable_hash>",
  "status": "recorded"
}
```

Effects:

```text
upsert provider call evidence
append zoom.meeting.observed
broadcast zoom.meeting.observed
```

## Recording observation

```http
POST /api/v1/integrations/zoom/runtime-bridge/recordings
```

Request:

```json
{
  "observation_id": "obs_zoom_recording_001",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "recording": {
    "recording_id": "rec_001",
    "recording_type": "shared_screen_with_speaker_view",
    "download_ref": "blob_or_provider_ref_only",
    "file_extension": "mp4",
    "file_size_bytes": 123456,
    "recorded_at": "2026-06-27T10:31:00Z",
    "metadata": {}
  },
  "metadata": {},
  "correlation_id": "corr_zoom_fixture_001"
}
```

Effects:

```text
append zoom.recording.observed
broadcast zoom.recording.observed
```

The direct `/runtime-bridge/recordings` route records sanitized event evidence
only. Provider-side recording file downloads are handled separately through the
authorized provider-sync worker and signed `recording.completed` webhook path
after explicit privacy opt-in.

## Transcript observation

```http
POST /api/v1/integrations/zoom/runtime-bridge/transcripts
```

Request:

```json
{
  "observation_id": "obs_zoom_transcript_001",
  "transcript_id": "tr_zoom_001",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "meeting_uuid": "meeting-uuid",
  "source_recording_ref": "rec_001",
  "language_code": "en",
  "transcript_text": "Meeting transcript text.",
  "segments": [],
  "metadata": {
    "fixture": true
  },
  "correlation_id": "corr_zoom_fixture_001"
}
```

Target response:

```json
{
  "transcript_id": "tr_zoom_001",
  "call_id": "zoom_call_<stable_hash>",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "event_id": "evt_zoom_transcript_<stable_hash>",
  "status": "recorded"
}
```

Effects:

```text
ensure placeholder provider call if needed
upsert call transcript
append zoom.transcript.observed
broadcast zoom.transcript.observed
```

## Transcript file import

```http
POST /api/v1/integrations/zoom/runtime-bridge/transcript-files
```

This route imports an already obtained Zoom transcript file into the same
`zoom.transcript.observed` contract. It does not download provider files and it
does not bypass account validation.

Supported input formats:

```text
WEBVTT / .vtt
SRT / .srt
plain text
```

Request:

```json
{
  "observation_id": "obs_zoom_transcript_file_001",
  "transcript_id": "tr_zoom_file_001",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "meeting_uuid": "meeting-uuid",
  "source_recording_ref": "rec_001",
  "language_code": "en",
  "file_name": "meeting.vtt",
  "content_type": "text/vtt",
  "file_text": "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nHello",
  "metadata": {
    "fixture": true
  }
}
```

Response:

```json
{
  "transcript_id": "tr_zoom_file_001",
  "call_id": "zoom_call_<stable_hash>",
  "account_id": "zoom_fixture_primary",
  "meeting_id": "987654321",
  "event_id": "evt_zoom_transcript_<stable_hash>",
  "status": "recorded",
  "import_format": "webvtt",
  "parsed_segment_count": 1
}
```

Effects:

```text
parse VTT/SRT/plain transcript file
ensure placeholder provider call if needed
upsert call transcript
append zoom.transcript.observed
broadcast zoom.transcript.observed
```

## Verified webhook bridge

```http
POST /api/v1/integrations/zoom/runtime-bridge/webhooks?account_id=<zoom_account_id>
```

This protected runtime-bridge route handles account-scoped Zoom webhook
notifications. It is not a public internet receiver; the standalone
`makosh-zoom-edge-proxy` may forward raw Zoom webhook requests here after
preserving the raw body and Zoom headers.

The implemented public/edge ingress for that forwarding role is:

```text
makosh-zoom-edge-proxy
PUBLIC:  POST /webhooks/zoom
FORWARDS TO:  POST /api/v1/integrations/zoom/runtime-bridge/webhooks?account_id=...
```

The proxy reads only local env configuration, never stores provider secrets,
and does not parse or rewrite the request body.

Endpoint URL validation:

```json
{
  "event": "endpoint.url_validation",
  "payload": {
    "plainToken": "zoom_plain_token"
  }
}
```

Response:

```json
{
  "plainToken": "zoom_plain_token",
  "encryptedToken": "<hmac_sha256_plain_token>"
}
```

Normal webhook ingestion requires:

```text
x-zm-request-timestamp
x-zm-signature: v0=<hmac_sha256>
```

The HMAC message is:

```text
v0:<x-zm-request-timestamp>:<raw_body>
```

Implemented normalization:

```text
meeting.* webhook -> ZoomMeetingObservationRequest -> zoom.meeting.observed
recording.* webhook -> ZoomRecordingObservationRequest(s) -> zoom.recording.observed
```

Signed `recording.completed` webhook payloads may also trigger best-effort
download/import of non-transcript recording media files and transcript-like
textual recording files when Zoom includes a `download_url` plus
`download_token`. Transcript text is not extracted from webhook metadata alone.
Import of already obtained VTT/SRT/plain transcript text remains available
through `/runtime-bridge/transcript-files`.

## Sanitization

The bridge recursively strips token-like fields from event payloads before
append and broadcast. Provider call/transcript metadata is sanitized with the
same rule before persistence:

```text
access_token
refresh_token
token
client_secret
webhook_secret
download_token
password
```

Do not rely on sanitization as the only defense. Runtime callers should avoid
submitting secret values outside secret-reference fields.
