# Event Model

## Purpose

The event model is the system spine. It records facts that happened in or
around Макошь and lets projections build current state, graph links,
indexes and user-facing timelines.

Communications are the primary ingestion spine for external interaction
evidence. The event model is the canonical internal spine that preserves what
happened after evidence enters Макошь.

## Event Categories

- communication events
- document events
- task events
- calendar events
- persona events
- organization events
- project events
- decision events
- obligation events
- relationship events
- contradiction review events
- agent events
- system events
- security and permission events

## Canonical Event Envelope

```json
{
  "event_id": "01MAKOSH...",
  "event_type": "message_received",
  "schema_version": 1,
  "occurred_at": "2026-06-04T12:00:00Z",
  "recorded_at": "2026-06-04T12:00:05Z",
  "source": {
    "kind": "email",
    "provider": "imap",
    "source_id": "provider-message-id",
    "import_batch_id": "01BATCH..."
  },
  "actor": {
    "kind": "persona",
    "entity_id": "persona_..."
  },
  "subject": {
    "kind": "message",
    "entity_id": "message_..."
  },
  "payload": {},
  "provenance": {
    "raw_record_id": "raw_...",
    "confidence": 1.0
  },
  "causation_id": null,
  "correlation_id": "01FLOW..."
}
```

## Required Properties

- Append-only by default.
- Idempotent ingestion using provider source IDs and import batch IDs.
- Explicit schema version.
- Separate occurred_at and recorded_at.
- Traceable causation and correlation IDs.
- Provenance for imported and AI-derived facts.
- Projection rebuild support.

## Trace Semantics

Every canonical event is also a span.

```text
event_id = span id
correlation_id = trace id
causation_id = parent span id
event_log = trace store
```

Every event written through the canonical event builder must have a non-empty
`correlation_id`. Root events may have no `causation_id`. Derived events must
set `causation_id = parent.event_id` and inherit `correlation_id` from the
parent event.

Trace reconstruction belongs to `platform/events` and reads from append-only
`event_log`. Timeline Engine remains a chronological projection engine; it may
display trace links but must not become the trace source of truth.

## Event Examples

- `message_received`
- `message_sent`
- `message_classified`
- `communication_linked_to_persona`
- `document_uploaded`
- `document_version_created`
- `document_ocr_completed`
- `entity_extracted`
- `relationship_created`
- `task_created`
- `task_status_changed`
- `meeting_completed`
- `persona_created`
- `organization_created`
- `decision_recorded`
- `obligation_accepted`
- `contradiction_observed`
- `contradiction_reviewed`
- `payment_received`
- `agent_tool_invoked`
- `permission_granted`

## Projections

Events feed projections for:

- message threads
- unified timeline
- Persona dossiers
- Organization context
- project timelines
- task views
- graph edges
- full text index
- semantic index
- contradiction review queues
- agent memory traces

Projection failures must be observable and replayable. A broken projection must not corrupt the canonical event log.
