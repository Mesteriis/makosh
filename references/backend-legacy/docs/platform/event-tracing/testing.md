# Event Tracing Testing

## Purpose

Trace tests prove that Макошь can reconstruct causal event graphs without
external telemetry infrastructure.

## Unit Tests

Required unit coverage:

- `TraceContext::root`;
- `TraceContext::child_of`;
- event builder correlation normalization;
- trace graph reconstruction from stored events;
- missing parent detection;
- orphan root handling.

## Integration Tests

Required PostgreSQL-backed coverage:

- observation to raw signal to accepted signal to communication event;
- Telegram fixture trace;
- WhatsApp fixture trace;
- Mail fixture trace;
- DLQ annotation on failed consumer;
- realtime payload includes trace fields.

Use existing repository conventions:

- `testcontainers-rs` lifecycle through `crates/test-session`, with backend fixtures in `crates/testkit` (`makosh-backend-testkit`);
- `cargo nextest` through Makefile targets for full backend validation;
- deterministic fixtures instead of live Telegram, WhatsApp, Mail or external
  providers;
- no dependency on a developer's local PostgreSQL instance.

## API Tests

API tests should exercise:

- `GET /api/v1/events/{event_id}/trace`;
- `GET /api/v1/event-traces/{correlation_id}`;
- `GET /api/v1/events/{event_id}/children`;
- payload sanitization on trace and realtime responses;
- missing event response behavior.

## Frontend Tests

Frontend tests cover:

- platform trace API paths;
- provider-neutral query keys;
- shared trace panel ownership boundaries.

## Regression Cases

Every bug that disconnects a trace chain should add a test showing the broken
chain before the fix:

```text
observation.captured.v1
  -> signal.raw.<source>.<thing>.observed
  -> signal.accepted.<source>.<thing>
  -> owning domain event
```
