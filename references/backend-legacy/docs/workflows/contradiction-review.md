# Contradiction Review

This workflow explains how Макошь handles conflicts between new evidence and
accepted memory.

User-facing alias: Polygraph review.

## Trigger

The workflow starts when the Consistency / Contradiction Engine finds a conflict
between new evidence and accepted memory, knowledge, obligation state or
decision state.

## Flow

```text
new evidence
  -> compare with accepted memory
  -> create contradiction observation
  -> collect old and new source references
  -> classify conflict type
  -> present review item
  -> owner or policy resolves
  -> update memory, mark disputed, create task or keep existing state
```

## Review Outcomes

- accept new claim;
- keep existing memory;
- mark both claims disputed;
- split entities;
- update relationship confidence;
- create verification task;
- defer until more evidence exists.

## Required Outputs

- contradiction observation;
- old source reference;
- new source reference;
- affected entities;
- confidence and severity;
- review outcome.

## Domain And Engine Boundaries

- Consistency / Contradiction Engine creates observations.
- Domains own accepted state updates.
- Memory Engine updates memory views after accepted changes.
- Trust Engine can use reviewed contradictions as signals.
- Risk Engine can create attention items for unresolved conflicts.

## Current Implementation Evidence

No dedicated implementation exists yet. This workflow is target documentation
approved during product refactoring.

## Migration Plan

1. Start with review items, not automatic overwrites.
2. Require source citations before creating a contradiction observation.
3. Add ADR before persistence or route implementation.
4. Use communications and documents as initial evidence sources.
