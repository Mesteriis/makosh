# Replay and Live Notes

Meeting memory should be useful during the meeting and after it.

## Live notes panel

A meeting WebView can be paired with a Макошь side panel:

```text
Meeting
├── active speaker hints
├── emerging topics
├── candidate action items
├── candidate decisions
├── mentioned people/organizations
├── attached documents
└── Radar signals
```

Live output must be marked as provisional. The post-meeting pipeline may correct
speaker identity, segment boundaries and candidate confidence.

## Replay model

A completed Call Bundle should support synchronized replay:

```text
audio position
transcript segment
speaker identity
topic timeline
chat messages
screenshots/OCR
candidate decisions
action items
Radar signals
```

Selecting `15:23` should jump to:

```text
audio t=15:23
transcript segment covering 15:23
nearest screenshot
current topic
current speaker candidate
related tasks/decisions if accepted
```

## Event track

`event-track.jsonl` records non-transcript events:

```json
{"offset_ms":0,"event":"meeting_opened"}
{"offset_ms":12000,"event":"participant_joined","label":"Ivan"}
{"offset_ms":42000,"event":"screen_share_started"}
{"offset_ms":960000,"event":"decision_candidate_detected"}
{"offset_ms":1420000,"event":"meeting_left"}
```

## Search

Meeting memory should be searchable by:

```text
spoken text
topic
participant
organization
project
decision
action item
screen OCR
attachment name
```

This turns meetings from disposable conversation into searchable evidence.
