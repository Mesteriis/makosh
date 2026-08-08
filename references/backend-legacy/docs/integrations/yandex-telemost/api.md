# Yandex Telemost API Surface

## Backend routes

```text
GET  /api/v1/integrations/yandex-telemost/capabilities
GET  /api/v1/integrations/yandex-telemost/accounts
POST /api/v1/integrations/yandex-telemost/accounts
GET  /api/v1/integrations/yandex-telemost/runtime/status?account_id={account_id}
POST /api/v1/integrations/yandex-telemost/accounts/{account_id}/retention/prune
POST /api/v1/integrations/yandex-telemost/conferences
GET  /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}
PATCH /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}
GET  /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}/cohosts
POST /api/v1/integrations/yandex-telemost/webview/manifest
POST /api/v1/integrations/yandex-telemost/recording/intent
POST /api/v1/integrations/yandex-telemost/runtime-bridge/recordings
POST /api/v1/integrations/yandex-telemost/runtime-bridge/transcripts
```

## Account setup

```json
{
  "account_id": "telemost-main",
  "display_name": "Yandex Telemost",
  "external_account_id": "user@yandex.ru",
  "oauth_token": "<redacted>",
  "oauth_token_ref": null,
  "api_base_url": "https://cloud-api.yandex.net/v1/telemost-api",
  "metadata": { "source": "settings_panel" }
}
```

Either `oauth_token` or `oauth_token_ref` must be supplied. If `oauth_token` is
present, it is stored in HostVault and bound under
`yandex_telemost_oauth_token`.

## Create conference

```json
{
  "account_id": "telemost-main",
  "body": {
    "waiting_room_level": "PUBLIC",
    "cohosts": [{ "email": "cohost@yandex.ru" }],
    "is_auto_summarization_enabled": true,
    "metadata": { "source": "calendar_workflow" }
  }
}
```

The provider payload drops `metadata` before sending to Yandex. Metadata remains
local provenance.

## WebView manifest

```json
{
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "join_url": "https://telemost.yandex.ru/j/abcdef",
  "display_name": "Макошь Owner"
}
```

The backend returns the expected visible WebView contract. The actual window is
opened by the Tauri command `open_yandex_telemost_companion`.

## Tauri commands

```text
open_yandex_telemost_companion
yandex_telemost_companion_manifest
yandex_telemost_prepare_audio_device
yandex_telemost_recording_start
yandex_telemost_recording_stop
yandex_telemost_speaker_timeline_append
```

`yandex_telemost_recording_start` requires:

```json
{
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "join_url": "https://telemost.yandex.ru/j/abcdef",
  "consent_attested": true
}
```

## Recording completion bridge

After local `ffmpeg` capture stops, the desktop client posts the completed
recording manifest back to Макошь:

```json
{
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "join_url": "https://telemost.yandex.ru/j/abcdef",
  "recording_session_id": "session-123",
  "output_dir": "/.../telemost-recordings/telemost-main/session-123",
  "audio_path": "/.../audio.mp3",
  "speaker_jsonl_path": "/.../speaker-timeline.jsonl",
  "speaker_txt_path": "/.../speaker-timeline.txt",
  "started_at_epoch_ms": 1719550000000,
  "stopped_at_epoch_ms": 1719550300000,
  "consent_attested": true
}
```

Макошь validates that all paths stay under `output_dir`, materializes the
provider-neutral Call Bundle files (`manifest.json`, `meeting.json`,
`provider.json`, `participants.json`, `event-track.jsonl`,
`speaker-hints.jsonl`), publishes
`integration.yandex_telemost.local_recording.completed`, and queues the
provider-neutral realtime conversation pipeline bootstrap events.

Each Radar candidate is also captured as a provider-neutral
`REALTIME_CONVERSATION_RADAR_SIGNAL` observation and mirrored into the existing
Review Inbox with the closest existing owner-domain review kind:

- `unknown_cohosts` -> `potential_relationship`;
- `unmatched_meeting_link` -> `potential_project`;
- remaining Telemost radar artifacts -> `knowledge_candidate`.

The Call Bundle manifest also snapshots the active owner-visible retention
policy for local Telemost artifacts under `provenance.retention_policy`.

## Transcript completion bridge

After local STT finishes, the desktop/runtime side posts the transcript result
back to Макошь:

```json
{
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "bundle_id": "session-123",
  "bundle_root": "/.../telemost-recordings/telemost-main/session-123",
  "transcript_text": "Owner: ship the Telemost runtime.",
  "segments": [
    {
      "speaker": "Owner",
      "start_ms": 0,
      "end_ms": 1200,
      "text": "ship the Telemost runtime"
    }
  ],
  "language_code": "en",
  "stt_provider": "whisper-local",
  "summary": "Decision to ship the Telemost runtime.",
  "confidence": 0.91,
  "metadata": { "engine_version": "local-dev" }
}
```

Макошь validates the bundle root and manifest, writes `transcript.json`,
`transcript.md` and optional `summary.md`, updates `manifest.json`, publishes
`realtime_conversation.transcript.completed`, and projects the transcript into
the `documents` domain through the provider-neutral transcript workflow.

When the Call Bundle already carries `calendar_event_id`, the same workflow also
projects the transcript into calendar meeting state by creating
`event_transcripts` evidence and attaching the transcript to the matching
`event_recordings` row.

## Automatic local transcription execution

The `realtime_conversation.transcript.requested` payload now includes the local
runtime paths required for STT execution:

```json
{
  "bundle_id": "session-123",
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "provider_kind": "yandex_telemost",
  "bundle_root": "/.../telemost-recordings/telemost-main/session-123",
  "manifest_path": "/.../telemost-recordings/telemost-main/session-123/manifest.json",
  "audio_path": "/.../telemost-recordings/telemost-main/session-123/audio.mp3"
}
```

If `MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER` is present, Макошь runs that
local executable and passes bundle metadata through environment variables:

```text
MAKOSH_TRANSCRIPT_BUNDLE_ID
MAKOSH_TRANSCRIPT_ACCOUNT_ID
MAKOSH_TRANSCRIPT_CONFERENCE_ID
MAKOSH_TRANSCRIPT_PROVIDER_KIND
MAKOSH_TRANSCRIPT_BUNDLE_ROOT
MAKOSH_TRANSCRIPT_MANIFEST_PATH
MAKOSH_TRANSCRIPT_AUDIO_PATH
MAKOSH_TRANSCRIPT_MANIFEST_JSON
```

Optional settings:

```text
MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_ARGS_JSON='["--flag","value"]'
MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_TIMEOUT_SECONDS=900
```

The executable must emit JSON on stdout with:
`transcript_text`, `segments`, `stt_provider`, and optional
`language_code`, `summary`, `confidence`, `metadata`.

## Local artifact retention cleanup

Макошь now supports owner-visible retention cleanup for local Telemost files:

```json
{
  "remove_audio": true,
  "remove_speaker_hints": true,
  "limit": 100
}
```

The cleanup route removes expired local files according to the application
settings:

```text
privacy.yandex_telemost_recording_retention_days
privacy.yandex_telemost_speaker_timeline_retention_days
```

`0` disables automatic cleanup for that artifact class.

When a bundle is cleaned, Макошь removes:

- `audio.mp3` when the recording retention policy has expired;
- `speaker-timeline.jsonl`, `speaker-timeline.txt`, and
  `speaker-hints.jsonl` when the speaker-hint retention policy has expired;
- and records the cleanup result back into `manifest.json`.

## Provider API dependency

The Yandex provider calls use:

```text
Authorization: OAuth <token>
POST  /v1/telemost-api/conferences
GET   /v1/telemost-api/conferences/{id}
PATCH /v1/telemost-api/conferences/{id}
GET   /v1/telemost-api/conferences/{id}/cohosts?offset={offset}&limit={limit}
```
