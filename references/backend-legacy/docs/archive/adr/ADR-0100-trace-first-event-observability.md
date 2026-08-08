# ADR-0100 Trace-First Event Observability

Status: Accepted
Date: 2026-06-24

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0012 OpenTelemetry Observability
- ADR-0014 Canonical Event Envelope
- ADR-0018 Provider Adapter Boundary
- ADR-0034 Event Replay and Projection Cursors
- ADR-0095 Event-Driven Domain Communication and DLQ
- ADR-0097 Communications Channel Domains To Integrations
- ADR-0098 Provider-Neutral Communications API And Strict Boundaries
- ADR-0099 Signal Hub Event Platform

## Context

Макошь already uses an event-driven architecture with an append-only
`event_log`, `EventEnvelope`, outbox, event consumers, DLQ, Signal Hub and
provider integrations. However, events are not yet consistently usable as one
causal trace graph:

- some events have nullable or empty `correlation_id`;
- some derived events do not inherit trace context;
- observation events are not always treated as root trace events;
- Signal Hub, provider and Communications chains are not always connected;
- API and realtime surfaces do not consistently expose full trace context;
- Timeline projection and causal trace reconstruction can be confused.

Макошь needs to explain why a domain object exists without requiring Jaeger,
Tempo, Loki, Grafana or another telemetry server.

## Decision

Макошь treats canonical events as spans.

`event_id` is the span identifier.
`correlation_id` is the trace identifier.
`causation_id` is the parent span identifier.

The append-only `event_log` is the canonical trace store.

Every event written through the canonical event builder must have a non-empty
`correlation_id`.

Every event created as a consequence of another event must set
`causation_id = parent.event_id` and inherit `correlation_id` from the parent.

No separate telemetry server is required to explain why a domain object exists.

Root events may have no `causation_id`, but they must still have
`correlation_id`. Derived events must have `causation_id`. Trace reconstruction
uses only deterministic links stored in `event_log`; AI may summarize a trace
after reconstruction, but must not infer missing links.

OpenTelemetry may be used as an export or diagnostic layer. OpenTelemetry is
not the canonical trace store. The canonical trace store is `event_log`.

## Trace Semantics

| Concept | Макошь field |
|---|---|
| Trace | `correlation_id` |
| Span | `event_id` |
| Parent span | `causation_id` |
| Trace store | PostgreSQL append-only `event_log` |

Rules:

- root event: `causation_id = null`, `correlation_id` is non-empty;
- derived event: `causation_id = parent.event_id`;
- derived event: `correlation_id = parent.correlation_id`;
- legacy events with null `correlation_id` are displayed as legacy orphan
  traces unless migrated;
- consumer status, retries and DLQ records are trace annotations, not domain
  facts by themselves.

## Layer Boundary

Trace reconstruction belongs to `platform/events`.

Timeline projection belongs to `engines/timeline`.

Timeline may display trace links, but it must not be the source of truth for
trace reconstruction. A trace graph is a causal and provenance graph. A
timeline is a user or domain chronological projection.

Provider integrations remain source boundaries. Telegram, WhatsApp and Mail do
not own product-domain trace models. Communications owns canonical
communication state and emits canonical communication events with inherited
trace context.

## Consequences

Positive:

- every persistent event can be inspected as part of a causal graph;
- provider, Signal Hub, Communications and workflow chains become explainable;
- trace APIs can be implemented directly from PostgreSQL;
- Timeline Engine remains focused on chronological user/domain projection;
- OpenTelemetry can export traces without becoming source of truth.

Negative:

- builder normalization changes expectations for events that previously stored
  null `correlation_id`;
- derived event writers must pass parent context explicitly;
- legacy rows may appear as orphan traces until migrated or backfilled;
- realtime/API DTOs need trace fields even for events that older UI code did
  not display.

## Validation

The repository should enforce:

- canonical builder never produces an empty `correlation_id`;
- `TraceContext::root` and `TraceContext::child_of` preserve the rules above;
- observation capture creates a root trace event;
- raw provider/source signals are children of observation events;
- Signal Hub accepted/rejected/muted/paused events are children of raw signals;
- Communications message events inherit trace context from accepted signals;
- EventStore can return trace by event id, trace by correlation id and children
  by causation id;
- trace reconstruction does not depend on Timeline Engine;
- trace API responses include events, edges, roots, orphans, missing parents,
  consumer annotations and DLQ annotations;
- realtime event payloads expose trace fields without leaking private content.
