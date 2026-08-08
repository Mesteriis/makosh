# Speaker Diarization and Identity

Speaker identity is a merge problem, not a single-source lookup.

## Inputs

```text
audio diarization speakers: Speaker A, Speaker B, Speaker C
WebView speaker hints: visible participant labels and time ranges
calendar attendees
provider cohosts/participants if available
known contacts/person identities
historical voice embeddings if explicitly enabled
manual user confirmations
```

## WebView hints

WebView active speaker state is useful, but not authoritative.

```json
{
  "observed_at_epoch_ms": 1780000000000,
  "speaker_label": "Ivan",
  "confidence": 0.35,
  "source": "webview_dom_heuristic",
  "truth_status": "hint_not_truth"
}
```

It can help answer:

```text
How many people likely spoke?
Which visible label was active around this time?
Which diarized speaker might map to which contact?
```

It cannot answer with certainty:

```text
Who truly spoke this segment?
Whether someone was speaking off-camera or from another audio source?
Whether a DOM label belongs to the audio speaker?
```

## Merge output

```json
{
  "diarized_speaker_id": "spk_01",
  "person_id": "person_...",
  "display_name": "Ivan Petrov",
  "confidence": 0.91,
  "evidence": {
    "audio_similarity": 0.88,
    "webview_hint_overlap": 0.74,
    "calendar_attendee_match": true,
    "manual_confirmation": false
  }
}
```

## Confidence policy

| Confidence | Meaning |
|---|---|
| `>= 0.90` | strong identity match |
| `0.70 - 0.89` | likely match, reviewable |
| `0.40 - 0.69` | weak candidate |
| `< 0.40` | keep unknown speaker |

Макошь should keep `Unknown Speaker #n` when confidence is low. Speaker labels
must remain reviewable when evidence is weak or contradictory.

## Manual confirmation

The UI should allow the owner to confirm speaker identity. Once confirmed, the
mapping becomes stronger evidence for future meetings, subject to privacy rules
and explicit local storage policy.
