# ADR-0095 Event-Driven Domain Communication and DLQ

Status: Accepted

## Context

Макошь is moving to a layered architecture where integrations, domains, engines
and UI modules do not call each other's handlers or domain services directly.
Communication channels such as Email, Telegram and WhatsApp should publish
source evidence and provider state. Domain owners should react through durable
events and explicit promotion/review flows.

The repository already has an append-only `event_log`, canonical event
envelopes and projection cursors. That is enough for replayable projections, but
not enough for reliable cross-domain communication. Inter-domain consumers need
their own checkpoints, retry state and dead-letter handling so one poisoned event
does not silently skip, loop forever or block unrelated consumers.

## Decision

All cross-domain communication must use the event system.

Rules:

- API handlers call only the command/application service of their own bounded
  context.
- A domain must not import another domain's handlers, stores or services for
  synchronous business behavior.
- Integrations must not call Personas, Tasks, Documents, Projects,
  Organizations, Knowledge or other business domains directly.
- Cross-domain intent is expressed as a versioned event.
- The owning domain consumes that event and decides whether to create or update
  its own durable state.
- Event consumers are at-least-once and must be idempotent.
- Consumer cursors move only after successful handling or after the event is
  durably moved to the consumer's dead-letter queue.
- Retry and DLQ state are per consumer. One consumer failing must not prevent
  other consumers from processing the same event.

The event platform adds:

```text
event_consumers
event_consumer_failures
event_consumer_processed_events
event_dead_letters
```

`event_consumers` stores each durable consumer position. `event_consumer_failures`
stores retry state and backoff. `event_consumer_processed_events` records the
per-consumer event positions already applied so duplicate delivery and cursor
rewind do not invoke the handler again for the same event. `event_dead_letters`
stores poison events for owner review and manual replay.

Canonical event families include:

```text
communication.*
integration.email.*
integration.telegram.*
integration.whatsapp.*
radar.*
persona.*
task.*
document.*
knowledge.*
relationship.*
```

The first target flow is:

```text
integration.telegram.message.observed
  -> communication.message.recorded
  -> radar.signal.detected
  -> radar.promotion.requested
  -> task.created / persona.identity_trace.recorded / document.import.requested
```

## Consequences

Positive:

- Domains become isolated bounded contexts.
- Integrations stop owning business meaning.
- Retry, DLQ and replay semantics are centralized.
- Cross-domain behavior becomes auditable and replayable.
- Architecture linting can enforce boundaries in CI.

Negative:

- Existing synchronous cross-domain imports become legacy debt until refactored.
- Event contracts require version discipline.
- Consumers must be explicitly idempotent.
- Some workflows become eventually consistent instead of synchronous.

## Implementation Notes

Superseded by `ADR-architecture-communication-contract`.

The event/DLQ rules in this ADR remain accepted, but the temporary boundary
baseline is abolished. Existing violations must be removed or moved behind the
communication contract. `scripts/architecture-boundary-baseline.json` must not
exist.
