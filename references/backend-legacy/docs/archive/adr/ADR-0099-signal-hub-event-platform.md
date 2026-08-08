# ADR-0099 Signal Hub Event Platform

Status: Superseded by ADR-0181
Date: 2026-06-22

Superseded by: ADR-0181 Backend Workspace Modularity and Provider Runtime
Topology and ADR-0183 Backend API Cutover and Canonical Schema Reset. ADR-0181
retains the Signal Hub and event-platform decisions from this record, while
replacing its initial same-process runtime restriction. ADR-0183 changes only
the API compatibility and schema-reset policy for the final refactor.

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0014 Canonical Event Envelope
- ADR-0018 Provider Adapter Boundary
- ADR-0034 Event Replay and Projection Cursors
- ADR-0095 Event-Driven Domain Communication and DLQ
- ADR-0097 Communications Channel Domains To Integrations
- ADR-0098 Provider-Neutral Communications API And Strict Boundaries

## Context

Макошь is growing from a Communications-centered local system into a Personal
Operating System for memory, context and decisions. Email, Telegram and WhatsApp
are only the first external sources. Future sources include GitHub, Browser
capture, RSS, Calendar providers, Filesystem, Home Assistant, voice input and
fixture sources.

The system needs a single place to answer:

```text
What sources exist?
What is connected?
What is enabled?
What is muted?
What is paused?
What is unhealthy?
What can be replayed?
What fixture mode is active?
```

Putting this state inside provider integrations would duplicate policy across
Mail, Telegram, WhatsApp and every future source. Putting it inside
Communications would incorrectly make all signals communication-shaped.

## Decision

Макошь introduces Signal Hub as a first-class system domain.

Signal Hub owns:

- source registry;
- source connections;
- source capabilities;
- source runtime state;
- source health;
- signal policies;
- source profiles;
- replay requests;
- system recovery fixtures;
- fixture source catalog metadata.

Signal Hub does not own:

- provider protocol code;
- provider secrets;
- raw private message bodies;
- Communications state;
- Radar state;
- Tasks, Personas, Documents, Calendar, Knowledge or Graph state.

All new external and synthetic signal sources enter Макошь through Signal Hub
control state and the Event Backbone.

## Event Platform Decision

Макошь designs the event platform from the start for:

- PostgreSQL append-only `event_log` as audit/recovery source of truth;
- NATS JetStream as durable production delivery and fan-out transport;
- in-memory EventBus for deterministic unit tests;
- Axum SSE for browser realtime updates;
- Protobuf + ConnectRPC for typed command/query API contracts.

Redis, Kafka, RabbitMQ and provider-runtime sidecars are not part of this
initial target.

## Canonical Flow

```text
External / Fixture Source
  -> Signal Hub
  -> EventEnvelope
  -> PostgreSQL event_log
  -> NATS JetStream subject
  -> Domain consumer
  -> Projection
  -> SSE UI update
```

Communication example:

```text
signal.telegram.message.observed
  -> communication.message.recorded
  -> radar.signal.detected
  -> review.item.promoted
  -> task.created / persona.identity_trace.recorded / document.import.requested
```

## Signal Controls

Signal Hub must support:

- enable;
- disable;
- global mute;
- selective mute;
- pause;
- resume;
- replay;
- health check;
- fixture mode;
- profile application.

These controls are not test hacks. They are product-level operations for a local
memory system that must be debuggable, recoverable and safe.

## Fixture And Recovery Decision

Signal Hub must provide a schema-agnostic system recovery fixture.

The fixture may contain:

- canonical source codes;
- capability codes;
- profile codes;
- category strings;
- non-secret defaults.

The fixture must not contain:

- UUID values;
- FK values;
- database row IDs;
- secret references;
- provider account IDs;
- graph IDs;
- communication IDs;
- task/document/person IDs.

The loader maps canonical fixture values into the current database schema and is
idempotent. It restores missing system records but does not overwrite user-owned
connections, secrets or runtime sessions.

## Testing Decision

Every real source must have a deterministic fixture source.

Domain and workflow tests must be able to run without live Telegram, WhatsApp,
Mail, GitHub, browser extension, Home Assistant or calendar provider access.

Core testing modes:

```text
Unit: InMemoryEventBus
Domain integration: PostgreSQL + fixture sources
Event transport: PostgreSQL + NATS JetStream test environment
E2E local: Signal Hub UI + SSE + fixtures
```

## Consequences

Positive:

- source control is centralized;
- provider integrations remain adapters;
- non-communication sources are not forced into Communications;
- testing can mute, pause, replay and fixture sources deterministically;
- event delivery is designed for NATS JetStream from the start;
- ConnectRPC contracts prevent REST DTO sprawl.

Negative:

- first implementation is larger than a provider-specific settings page;
- policies and profiles require careful UX;
- event naming must migrate from some provider-specific `integration.*` families
  to canonical `signal.*` families;
- NATS JetStream and ConnectRPC add implementation work immediately.

## Validation

The repository should eventually enforce:

- `domains/signal_hub` exists and owns source control state;
- provider integrations do not own source policy;
- Signal Hub does not mutate Communications or other domain tables;
- source controls emit audit events;
- recovery fixtures contain no IDs or references;
- fixture sources can drive Communications/Radar workflows;
- NATS JetStream transport exists behind EventBus/EventTransport abstractions;
- ConnectRPC contracts exist for Signal Hub command/query APIs;
- SSE updates Signal Hub projections;
- Redis is not introduced as an event substrate.
