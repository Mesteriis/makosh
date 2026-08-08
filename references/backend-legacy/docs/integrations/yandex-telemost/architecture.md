# Yandex Telemost Architecture

## Boundary

Yandex Telemost lives in:

```text
backend/src/integrations/yandex_telemost
frontend/src/integrations/yandexTelemost
frontend/src-tauri/src/yandex_telemost_companion.rs
```

It must not create `domains/yandex_telemost` and must not write directly to
Calendar, Communications, Calls, Radar or Documents.

## Inbound provider flow

```text
Yandex Telemost API / WebView / local recorder
↓
integrations/yandex_telemost or Tauri companion
↓
integration.yandex_telemost.* event or local artifact manifest
↓
workflow/projection
↓
Calendar / Calls / Radar / Documents
```

Matched conference evidence can enrich Calendar in two stages:

- conference events create the provider-neutral Calendar relation;
- cohost observations can populate `event_participants` for the matched
  Calendar event without making Telemost the owner of attendee truth.

## Outbound provider flow

```text
Calendar/App intent
↓
workflow or provider runtime route
↓
Yandex Telemost client command
↓
provider API result
↓
integration.yandex_telemost.conference.created/updated/observed
```

## WebView model

The desktop command opens a visible owner-controlled WebView. Hidden mode is
forbidden. The WebView may observe active-speaker-like DOM signals and forward
speaker hints to the local recorder. This signal is explicitly marked:

```text
truth_status = hint_not_truth
source = webview_dom_multi_selector_heuristic
confidence ~= 0.42
```

The hint is useful only as a warm start for diarization. Whisper or another
transcription/diarization stage must remain the evidence-producing stage.

## Local recorder model

The recorder is a Tauri-local runtime, not a backend provider API feature.

```text
visible WebView audio output
↓
local loopback/virtual audio device
↓
ffmpeg
↓
audio.mp3
```

Platform strategy:

```text
Linux:
  try to create PulseAudio/PipeWire null sink `makosh_telemost`
  record `makosh_telemost.monitor`

macOS:
  require explicit external loopback device such as BlackHole 2ch
  Макошь does not silently install system audio drivers

Windows:
  use WASAPI loopback or an explicit virtual audio cable/input
```

The recorder refuses to start unless `consent_attested=true`. This preserves
the owner-visible runtime model and prevents hidden capture from becoming part
of the provider integration.

## Events

```text
integration.yandex_telemost.authorization.completed
integration.yandex_telemost.runtime.status_changed
integration.yandex_telemost.conference.created
integration.yandex_telemost.conference.observed
integration.yandex_telemost.conference.updated
integration.yandex_telemost.cohosts.observed
integration.yandex_telemost.webview.open_requested
integration.yandex_telemost.local_recording.requested
integration.yandex_telemost.local_recording.completed
```

## Secret policy

```text
OAuth token value: HostVault only
Provider account config: secret_ref only
Event payloads: sanitized, no token/cookie/audio bytes
Frontend: no raw token after setup response
```

## Provider-neutral meeting memory

Yandex Telemost should project into the shared Meeting Platform instead of
creating Telemost-owned meeting memory. The provider opens or creates the
conference, the desktop companion captures local evidence, and the
provider-neutral Call Bundle becomes the durable object consumed by
transcription, diarization, Call Intelligence, Radar and Timeline workflows.

```text
Yandex Telemost conference
↓
visible Макошь WebView + local capture
↓
Call Bundle
↓
Call Intelligence / Speaker Identity
↓
Radar / Timeline / Knowledge Graph / Tasks candidates
```
