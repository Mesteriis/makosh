# Communication To Knowledge

This workflow explains how a communication becomes evidence-backed knowledge.

## Trigger

The workflow starts when Макошь imports or receives:

- email;
- Telegram message;
- WhatsApp message;
- call record;
- meeting-linked communication;
- future provider message.

## Flow

```text
source communication
  -> preserve source evidence
  -> normalize communication
  -> identify participants
  -> link candidate Personas and Organizations
  -> extract claims and entities
  -> compare against accepted memory
  -> create knowledge candidates
  -> review or policy gate
  -> store accepted facts or observations
```

## Required Outputs

- immutable source evidence;
- Communication record;
- participant records;
- candidate entity links;
- extracted claim candidates;
- contradiction observations when conflicts exist;
- reviewed Knowledge or Memory updates.

## Domain And Engine Boundaries

- Communications owns the source communication.
- Personas and Organizations own accepted identity links.
- Knowledge Graph owns accepted relationships.
- Memory Engine assembles memory views.
- Enrichment Engine proposes candidates.
- Consistency / Contradiction Engine detects conflicts.

## Current Implementation Evidence

Current implementation includes mail ingestion, communication messages,
Telegram, WhatsApp, extraction and email intelligence surfaces. The full
workflow is not yet implemented as one explicit pipeline.

## Migration Plan

1. Keep this workflow as the canonical behavior target.
2. Use current mail/Telegram/WhatsApp ingestion as evidence sources.
3. Add reviewable candidates before automatic knowledge updates.
4. Require source citations for accepted facts.
