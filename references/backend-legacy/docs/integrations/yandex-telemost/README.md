# Макошь Communications - Yandex Telemost Provider Stage

Status: `FOUNDATION_PATCH_APPLIED`, 2026-06-28.

Yandex Telemost is an external communication provider adapter. It is not a
Макошь domain, not a calendar source of truth and not a meeting CRM. Telemost
can provide conference metadata, join links, cohost metadata, local desktop
recording artifacts, WebView speaker-timeline hints and provider runtime
signals.

Invariant: A provider is never a domain. A conference is provider evidence. The
business object belongs to Calendar, Communications, Calls, Radar, Timeline,
Documents or another owner domain/workflow.

```text
Yandex Telemost Provider
  -> Provider Runtime API
  -> Visible Desktop WebView
  -> Local MP3 Recorder
  -> Speaker Timeline Hint Files
  -> Canonical Integration Events
  -> Shared Workflows and Engines
```

## Foundation scope

The Yandex Telemost foundation provides:

- provider kind `yandex_telemost_user`;
- secret purpose `yandex_telemost_oauth_token`;
- HostVault-backed OAuth token storage and provider-account secret binding;
- provider API client for conference create/read/update;
- provider API client for cohost list reads;
- matched Telemost cohost observations projected into Calendar `event_participants`;
- sanitized integration events with causation/correlation-ready envelopes;
- runtime status and capability surface;
- backend routes under `/api/v1/integrations/yandex-telemost/*`;
- frontend integration API, query keys and settings panel;
- desktop Tauri command for opening a conference in a visible Макошь WebView;
- local desktop recorder command that writes `audio.mp3` through `ffmpeg`;
- local speaker timeline hint files: `speaker-timeline.jsonl` and
  `speaker-timeline.txt`;
- owner-visible retention policy for local MP3 and speaker hint artifacts;
- explicit consent gate before any local recording starts.

## Current scope

```text
target available:
  account setup through token or token secret ref
  runtime status
  conference create/read/update
  cohost list read
  visible WebView open
  local MP3 recording start/stop
  WebView-derived speaker timeline hints
  sanitized integration events

unsupported until later:
  hidden capture
  automatic meeting join
  provider webhooks
  provider-side recording download
  provider-side transcript download
  treating WebView speaker hints as truth
  installing macOS/windows kernel audio drivers silently
```

## Provider kind

```text
yandex_telemost_user
```

## Secret purpose

```text
yandex_telemost_oauth_token
```

Domains store only references and lifecycle state. Raw OAuth tokens stay in
HostVault and never appear in event payloads, settings JSON or frontend state.

## Local recording artifacts

A recording session writes files under:

```text
app_data_dir/telemost-recordings/{account_id}/{recording_session_id}/
├── audio.mp3
├── speaker-timeline.jsonl
└── speaker-timeline.txt
```

`audio.mp3` is the later transcription source. The speaker timeline files are
only diarization hints for Whisper-side processing. They can help estimate the
number of speakers and rough speaking intervals, but they are not a source of
truth.

## Navigation

- [Architecture](./architecture.md)
- [API](./api.md)
- [Modules](./modules.md)
- [Local recording](./local-recording.md)
- [Implementation plan](./implementation-plan.md)
- [Live smoke checklist](./live-smoke-checklist.md)
- [Status](./status.md)
- [Realtime Conversation Platform](../../platform/realtime-conversation/README.md)
- [Call Intelligence Engine](../../engines/call-intelligence/README.md)
- [Speaker Identity Engine](../../engines/speaker-identity/README.md)
