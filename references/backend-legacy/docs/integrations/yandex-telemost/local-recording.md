# Yandex Telemost Local Recording Contract

## Purpose

The local recorder captures the audio that reaches the owner-visible Telemost
WebView and stores it as later transcription evidence. It is a desktop runtime
feature, not a Telemost cloud API feature and not a backend secret boundary.

```text
visible Telemost WebView
↓
owner-configured loopback / virtual audio source
↓
ffmpeg
↓
audio.mp3
↓
transcription + diarization workflow
```

## Consent gate

`yandex_telemost_recording_start` refuses to start unless:

```json
{
  "consent_attested": true
}
```

Макошь does not start hidden captures, does not join a conference in the
background and does not silently install system audio drivers.

## Output layout

Each recording session writes:

```text
app_data_dir/telemost-recordings/{account_id}/{recording_session_id}/
├── audio.mp3
├── speaker-timeline.jsonl
└── speaker-timeline.txt
```

`audio.mp3` is the source artifact for Whisper or another transcription engine.
The timeline files are hints only.

## Speaker timeline hint format

`yandex_telemost_speaker_timeline_append` appends one JSON line per WebView
active-speaker observation:

```json
{
  "observed_at_epoch_ms": 1780000000000,
  "speaker_label": "Alice",
  "confidence": 0.42,
  "source": "webview_dom_multi_selector_heuristic",
  "truth_status": "hint_not_truth"
}
```

The adjacent text file mirrors this as tab-separated rows:

```text
epoch_ms    speaker_label    event          confidence       source
1780000000  Alice            speaker_hint   confidence=0.42  source=webview_dom_multi_selector_heuristic
```

These rows are not authoritative. They only help the downstream diarization
stage estimate speaker count and rough speaker turns before Whisper/audio-based
analysis corrects or rejects the WebView hint.

The desktop companion now scans multiple DOM patterns instead of a single text
match. It prefers:

- explicit speaking/activity attributes such as `data-speaking`,
  `data-speaker-active` and `data-active-speaker`;
- participant-oriented containers or test ids;
- nearby participant-name fields such as `data-participant-name`,
  `data-display-name`, `aria-label` and `title`.

This remains a heuristic path. Макошь still records the result as
`truth_status=hint_not_truth`.
The current companion source label is
`webview_dom_multi_selector_heuristic`.

## Platform strategy

### Linux

`yandex_telemost_prepare_audio_device` attempts to create a PulseAudio/PipeWire
null sink:

```text
makosh_telemost
```

The recorder defaults to:

```text
makosh_telemost.monitor
```

The user still needs to route the Telemost WebView audio output to that sink.

### macOS

macOS requires an explicitly configured loopback device, for example BlackHole
2ch. Макошь reports the requirement and records from the device selected through
`MAKOSH_TELEMOST_FFMPEG_INPUT` or the command argument.

### Windows

Windows uses an explicit WASAPI/virtual-device path selected through
`MAKOSH_TELEMOST_FFMPEG_INPUT` or the command argument.

## Environment variables

```text
MAKOSH_TELEMOST_FFMPEG_PATH   optional ffmpeg binary path
MAKOSH_TELEMOST_FFMPEG_INPUT  optional platform-specific ffmpeg input selector
```

## Retention policy

Local Telemost artifacts now follow owner-visible application settings:

```text
privacy.yandex_telemost_recording_retention_days
privacy.yandex_telemost_speaker_timeline_retention_days
```

These settings control automatic cleanup for:

- `audio.mp3`;
- `speaker-timeline.jsonl`;
- `speaker-timeline.txt`;
- and the provider-neutral copied hint file `speaker-hints.jsonl`.

Макошь snapshots the active retention policy into the Call Bundle manifest and
an hourly backend cleanup pass removes expired files. A manual cleanup route is
also available through
`POST /api/v1/integrations/yandex-telemost/accounts/{account_id}/retention/prune`.

## Event/projection policy

Local recording artifacts should later be imported through a provider-neutral
workflow:

```text
local recording receipt
↓
document/call evidence import
↓
transcription/diarization
↓
transcript with Source, Confidence, Evidence
↓
Calendar / Calls / Radar / Timeline projections
```

The recorder itself does not mutate Calendar, Calls or Radar directly. It owns
only the local runtime process and local artifact manifest.

The current backend contract exposes a transcript completion bridge. The local
runtime can hand the finished STT result back to Макошь, which writes
`transcript.json` and `transcript.md`, emits
`realtime_conversation.transcript.completed`, and lets the provider-neutral
workflow project the transcript into durable meeting memory.
