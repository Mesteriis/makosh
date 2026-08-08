# Yandex Telemost Live Smoke Checklist

## Preconditions

```text
MAKOSH_SECRET_VAULT_KEY configured
HostVault unlocked
ffmpeg installed and available on PATH or MAKOSH_TELEMOST_FFMPEG_PATH set
Yandex OAuth token has Telemost scopes
```

## Backend smoke

1. `GET /api/v1/integrations/yandex-telemost/capabilities`
2. `POST /api/v1/integrations/yandex-telemost/accounts`
3. `GET /api/v1/integrations/yandex-telemost/runtime/status?account_id={account_id}`
4. `POST /api/v1/integrations/yandex-telemost/conferences`
5. `GET /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}`
6. `PATCH /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}`
7. `GET /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}/cohosts`

## Desktop smoke

1. Open `open_yandex_telemost_companion` with a valid join URL.
2. Confirm a visible WebView appears.
3. Start recording with `consent_attested=true`.
4. Speak in the meeting.
5. Confirm files are created:

```text
audio.mp3
speaker-timeline.jsonl
speaker-timeline.txt
```

6. Stop recording.
7. Import `audio.mp3` into the future transcription workflow.
8. Use timeline hints as diarization hints only.

## Platform audio notes

Linux:

```text
yandex_telemost_prepare_audio_device
route WebView output to makosh_telemost
record makosh_telemost.monitor
```

macOS:

```text
install/configure BlackHole 2ch or equivalent manually
set MAKOSH_TELEMOST_FFMPEG_INPUT if ffmpeg needs a device index
```

Windows:

```text
configure WASAPI loopback or virtual audio cable
set MAKOSH_TELEMOST_FFMPEG_INPUT when needed
```
