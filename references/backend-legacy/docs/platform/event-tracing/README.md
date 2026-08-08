# Event Tracing

Status: documentation package aligned to the current repository structure.

## Purpose

Event Tracing is the causal observability layer of Макошь.

It does not replace the event backbone. It formalizes how persistent events are
connected into traces.

Макошь does not need a separate telemetry server to answer:

- why does this object exist?
- what caused this event?
- what happened before this projection?
- which provider signal created this communication message?
- which review promoted this task?

## Position

The canonical trace store is PostgreSQL append-only `event_log`.

```text
EventEnvelope = Span
event_id = span_id
correlation_id = trace_id
causation_id = parent_span_id
event_log = trace store
```

OpenTelemetry, logs and metrics may export or diagnose traces, but they do not
own the canonical causal graph.

## Package

- [ADR-0100](../../archive/adr/ADR-0100-trace-first-event-observability.md)
- [Architecture](architecture.md)
- [Data model](data-model.md)
- [API](api.md)
- [Testing](testing.md)
- [Operations](operations.md)
- [Gap analysis](gap-analysis.md)
- [Status](status.md)

## Navigation

- [Architecture](./architecture.md)
- [API Reference](./api.md)
- [Data Model](./data-model.md)
- [Status](./status.md)
- [Gap Analysis](./gap-analysis.md)
- [Operations](./operations.md)
- [Testing](./testing.md)
