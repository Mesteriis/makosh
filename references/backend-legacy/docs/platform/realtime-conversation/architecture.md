# Realtime Conversation Platform Architecture

## Boundary

The Meeting Platform is a provider-neutral kernel for realtime conversations.
It lives conceptually around:

```text
backend/src/platform/realtime_conversation
backend/src/engines/call_intelligence
backend/src/engines/speaker_identity
backend/src/workflows/realtime_conversation_*
frontend/src/integrations/<provider>
frontend/src-tauri/src/<provider>_companion.rs
```

Provider-specific code lives under `integrations/*` or desktop companion
modules. Durable meeting memory belongs to provider-neutral bundles, workflows,
engines and projections.

## Flow

```text
External provider
↓
Provider runtime adapter / visible WebView / local recorder
↓
Call Bundle
↓
Transcription and diarization
↓
Speaker identity merge
↓
Call Intelligence
↓
Radar / Timeline / Knowledge Graph / Tasks / Documents
```

## Ownership

| Layer | Owns | Must not own |
|---|---|---|
| Provider integration | API, runtime auth, provider commands, WebView opening | Calendar, Tasks, Radar, Knowledge |
| Desktop companion | visible WebView, local recording process, speaker hints | business truth |
| Call Bundle | immutable local artifacts and manifest | final decisions |
| Call Intelligence engine | candidates and extracted structure | domain state |
| Workflows | orchestration and projections | raw provider sessions |
| Domains | accepted business state | provider runtime |

## Event language

Provider integrations publish provider facts:

```text
integration.<provider>.conference.created
integration.<provider>.conference.observed
integration.<provider>.speaker_hint.observed
integration.<provider>.local_recording.completed
```

Provider-neutral workflows publish meeting memory facts:

```text
realtime_conversation.bundle.created
realtime_conversation.transcript.completed
realtime_conversation.speaker_identity.candidate_detected
realtime_conversation.decision.candidate_detected
realtime_conversation.action_item.candidate_detected
realtime_conversation.radar_signal.detected
```

Domain owners later accept or reject candidates. AI output remains a candidate
until it is backed by evidence and accepted through the owning review workflow.

## Source policy

Every generated insight must include:

```text
source artifact
source time range
confidence
model/tool version if applicable
evidence reference
```

Examples:

```text
source = audio.mp3#t=00:15:20-00:16:02
source = transcript.json#segment=42
source = speaker-hints.jsonl#line=17
source = screenshots/screen-00042.png
```

## Reprocessing

The bundle is intentionally artifact-first. When diarization, Whisper, OCR or
entity extraction improves, Макошь can rerun the pipeline without asking the
provider for the meeting again. Providers are unreliable memory. Local evidence
is less glamorous, but it survives product redesigns.
