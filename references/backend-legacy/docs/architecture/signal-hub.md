# Architecture - Signal Hub

Signal Hub is the platform-level source control plane for Макошь.

It sits before Communications, Calendar, Documents, Tasks, Radar and other
domains. It controls whether external or fixture sources may publish signals
into the event backbone.

```text
External Sources
  -> Signal Hub
  -> PostgreSQL Event Log
  -> NATS JetStream
  -> Domain Consumers
  -> Projections
  -> SSE
  -> UI
```

## Why It Exists

Without Signal Hub, source management spreads across integration modules,
settings, app handlers, provider-specific UI and test fixtures. That makes it
hard to answer basic system questions:

- what sources exist;
- what is connected;
- what is muted;
- what is paused;
- which source is failing;
- which fixture profile is active;
- whether a source is allowed to publish during tests.

Signal Hub centralizes these answers.

## Relation To Existing Architecture

- Complements ADR-0095 event-driven domain communication and DLQ.
- Complements ADR-0097/0098 provider-neutral Communications boundaries.
- Uses the canonical EventEnvelope from platform events.
- Keeps integrations as provider adapters.
- Keeps domains isolated through events.
- Keeps UI on projections and SSE updates.

## Target Technologies

- Rust 2024 backend.
- Tokio runtime.
- Axum HTTP host and SSE.
- SQLx + PostgreSQL for source-of-truth state and event log.
- NATS JetStream for durable event delivery/fan-out.
- Protobuf + ConnectRPC for API contracts.
- Vue 3 + TanStack Query frontend.
- `insta`, `mockall`, testkit and testcontainers-style integration tests.

## Non-Goals

- Redis event bus.
- Kafka.
- RabbitMQ.
- WebSocket hub.
- Multi-tenant source administration.
- Provider-specific product domains.

## Provider Runtime Topology

ADR-0181 keeps Signal Hub as the control plane while allowing provider crates to
run in-process, as a shared connector or as an account-scoped connector. The
selected topology is explicit runtime policy, not a provider-owned business
decision. Connectors publish evidence through the same event backbone and never
own canonical domain state, PostgreSQL credentials or the host vault.
