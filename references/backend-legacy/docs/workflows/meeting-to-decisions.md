# Meeting To Decisions

This workflow explains how meetings and calls become decision memory.

## Trigger

The workflow starts when Макошь has evidence from:

- calendar meeting;
- call;
- meeting notes;
- communication thread around a meeting;
- attached agenda or minutes.

## Flow

```text
event evidence
  -> identify attendees
  -> collect linked communications and documents
  -> extract decision candidates
  -> extract alternatives and rationale
  -> link affected Projects, Personas and Organizations
  -> review or policy gate
  -> store accepted Decisions
  -> generate related obligations or tasks when needed
```

## Required Outputs

- event source reference;
- candidate decisions;
- rationale and alternatives where available;
- affected entities;
- accepted Decisions after review;
- linked obligations, tasks or follow-ups.

## Domain And Engine Boundaries

- Calendar/Events owns event records.
- Communications owns message evidence.
- Documents owns meeting notes or attachments.
- Decisions owns accepted decision records.
- Timeline Engine builds the chronological view.
- Memory Engine assembles meeting memory.

## Current Implementation Evidence

Calendar, calls, documents and communications exist. The accepted Decision
persistence baseline exists in `backend/src/domains/decisions/mod.rs`, and
guarded backend routes can list accepted Decisions and update accepted Decision
review state.

This is still not the full meeting-to-decision workflow. Meeting-to-decision
extraction, candidate-to-Decision review, desktop UI and adapters from
`meeting_outcomes` are not implemented yet.

## Migration Plan

1. Keep decision capture as candidate-first.
2. Require evidence citations for every accepted decision.
3. Feed reviewed candidates into the ADR-0089 Decisions domain model.
4. Link decisions to Projects before deriving project state from them.
