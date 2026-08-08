# Meeting Providers

A meeting provider is an adapter. It opens or observes a live conversation and
exposes capabilities. It does not own meeting memory.

## Provider capability surface

```text
conferences.create
conferences.read
conferences.update
webview.open
audio.local_capture
speaker_hints.webview
chat.capture
participants.observe
screenshare.detect
screenshots.capture
recording.local_mp3
transcript.provider_read
recording.provider_read
live_stream.create
```

## Provider types

| Provider | Role |
|---|---|
| Yandex Telemost | API-created conference + visible WebView + local capture |
| Zoom | provider API + possible meeting evidence + future shared call bundle |
| Jitsi | link/WebView-first provider |
| Google Meet | WebView/browser-first provider |
| Discord/Signal calls | realtime conversation source, API surface varies |

## Integration shape

```text
backend/src/integrations/<provider>
frontend/src/integrations/<provider>
frontend/src-tauri/src/<provider>_companion.rs
```

Provider code must expose runtime/account/capability APIs and emit integration
events. It must not write to Calendar, Tasks, Radar, Documents or Knowledge.

## Provider command flow

```text
App/Calendar intent
↓
workflow/provider command
↓
integration provider command handler
↓
external provider API/WebView action
↓
integration.<provider>.command.completed/failed
↓
provider-neutral projection
```

## Provider evidence flow

```text
visible session / provider API / local recorder
↓
integration.<provider>.*.observed
↓
Call Bundle
↓
Call Intelligence
↓
Radar/Timeline/Knowledge/Tasks candidates
```

The goal is not to make Макошь dependent on any vendor's meeting product. The
goal is to use providers as temporary doors into conversations, then store the
useful evidence in Макошь-owned form.
