# Event Tracing Data Model

## Purpose

This document defines the canonical trace graph shape built from `event_log`.

## Position

Trace state is not a separate store. It is a read model reconstructed from
canonical event rows and consumer metadata.

## Canonical Mapping

| Trace concept | Макошь field |
|---|---|
| Trace | `correlation_id` |
| Span | `event_id` |
| Parent span | `causation_id` |
| Trace store | `event_log` |

Root event:

```text
causation_id = null
correlation_id = non-empty
```

Derived event:

```text
causation_id = parent.event_id
correlation_id = parent.correlation_id
```

## Trace Graph Model

```json
{
  "correlation_id": "obs_...",
  "root_event_ids": [],
  "events": [],
  "edges": [],
  "orphan_event_ids": [],
  "missing_parent_ids": [],
  "consumer_annotations": [],
  "dead_letters": []
}
```

`edges` are deterministic parent-child links:

```json
{
  "parent_event_id": "event:v1:observation-captured:obs_123",
  "child_event_id": "signal_raw_telegram_..."
}
```

## Event And Consumer Tables

| Table | Role |
|---|---|
| `event_log` | canonical append-only event and trace store |
| `event_outbox` | pending/delivered event transport dispatch state |
| `event_consumers` | durable consumer cursor and runtime status |
| `event_consumer_processed_events` | processed-event annotations |
| `event_consumer_failures` | retry/failure annotations |
| `event_dead_letters` | DLQ annotations and review state |

Consumer processing metadata is an annotation of trace execution. It must not
be confused with business or domain events.

## Legacy Events

Events created before trace normalization may have null `correlation_id`. When
read by event id, they are shown under a legacy orphan trace whose trace id is
the event id. They should not be silently mutated unless a safe migration plan
exists.
