# Signal Hub Architecture

Status: target architecture, not full implementation.

## Purpose

Signal Hub is the control plane for source activation, source runtime state,
source capabilities, source health, mute/pause policies, replay and deterministic
fixtures.

It exists because Макошь must handle many signal sources without turning every
provider into a separate product domain. Mail, Telegram, WhatsApp, GitHub,
Browser capture, RSS, Calendar, Filesystem and Home Assistant should all enter
Макошь through one governed source boundary.

## High-Level Flow

```text
External Provider / Fixture
  -> Integration Adapter / Fixture Source
  -> observation.captured.v1
  -> signal.raw.<source>.<thing>.observed
  -> signal.accepted.<source>.<thing>
  -> Owning Domain Consumers
  -> Projections
  -> SSE / UI
```

Communication example:

```text
Telegram runtime update
  -> observation.captured.v1
  -> signal.raw.telegram.message.observed
  -> signal.accepted.telegram.message
  -> communication.message.recorded / communication.message.updated
  -> radar.signal.detected
  -> review.item.promoted
  -> task.created / persona.identity_trace.recorded / document.import.requested
```

Calendar example:

```text
Calendar provider update
  -> signal.calendar.event.observed
  -> calendar.event.recorded
  -> timeline.projection.updated
  -> meeting.preparation.requested
```

Filesystem example:

```text
Filesystem watcher event
  -> signal.filesystem.file.observed
  -> document.import.requested
  -> document.processed
  -> knowledge.candidate.detected
```

## Layer Ownership

| Layer | Owns | Must not own |
|---|---|---|
| `domains/signal_hub` | source registry, connections, capabilities, runtime state, health, profiles, mute/pause/replay policy, fixture catalog metadata | provider protocol code, communication messages, tasks, personas, documents |
| `backend/src/integrations/*` | provider protocol, transport, auth/session runtime, provider command execution, raw provider observation capture | Signal Hub policy, Communications state, Radar state, business domain state |
| `platform/events` | event envelope, event store, trace context, trace reconstruction, EventBus abstraction, NATS JetStream transport, consumer cursors, DLQ, replay primitives | business meaning, provider sessions, UI state |
| `domains/communications` | messages, conversations, participants, attachments, drafts, outbox, provider-neutral command state | provider runtime sessions, Signal Hub source policy |
| `domains/radar` | attention signals and review candidates | provider runtime or message storage |
| `workflows/*` | event-driven cross-domain orchestration | direct store mutation outside owner domain |
| `frontend/src/platform` | generated ConnectRPC client setup, SSE bootstrap, shared query plumbing | domain ownership |
| `frontend/src/domains/settings/*` | current user-facing Signal Hub Settings state, queries and composition | provider protocol UI internals or standalone Signal Hub route ownership |
| `frontend/src/integrations/*` | provider setup/runtime panels when needed | business communication workspace |

## Event Backbone

Signal Hub is designed around a versioned event backbone.

Immediate target technologies:

- PostgreSQL append-only `event_log` as audit/recovery source of truth;
- NATS JetStream as production delivery and fan-out transport;
- in-memory EventBus implementation for unit tests;
- Axum SSE for browser realtime updates;
- ConnectRPC + Protobuf for typed command/query API contracts;
- Vue 3 frontend with generated clients and TanStack Query for server state.

The code must depend on traits and contracts, not on transport-specific details:

```text
SignalSource
  -> SignalHubService
  -> EventPublisher trait
  -> EventStore + EventTransport
```

## Event Store And Transport Split

Макошь keeps source-of-truth event history in PostgreSQL and uses NATS JetStream
as the delivery transport.

```text
Domain command / source observation
  -> append EventEnvelope to PostgreSQL event_log
  -> publish same envelope to NATS JetStream subject
  -> durable consumers process through EventConsumerRunner / NATS durable consumer
  -> failures go to DLQ / review state
```

Rationale:

- PostgreSQL is already the primary local store and supports audit/recovery.
- NATS JetStream gives durable live fan-out, replayable delivery and subject
  filtering.
- Consumers must remain idempotent because delivery is at-least-once.
- If NATS publish fails, the event remains in `event_log`; a dispatcher can
  republish from stored positions.

## Canonical Subject Families

Target subject families:

```text
signal.*
communication.*
radar.*
review.*
task.*
persona.*
organization.*
document.*
calendar.*
knowledge.*
projection.*
system.*
```

Provider-specific compatibility families can exist during migration:

```text
integration.telegram.*
integration.mail.*
integration.whatsapp.*
```

New source work should prefer `signal.<source>.*` for source observations and
reserve `integration.<provider>.*` for provider runtime/internal status during
compatibility windows.

## Event Envelope

All cross-boundary events use the canonical envelope:

```text
event_id
event_type
schema_version
occurred_at
recorded_at
source
actor
subject
payload
provenance
causation_id
correlation_id
```

Signal Hub extensions live inside `source`, `subject`, `payload` and
`provenance`; the envelope shape remains stable.

## Trace Contract

Signal Hub participates in the canonical trace graph but does not own product
domain state.

```text
observation.captured.v1
  -> signal.raw.<source>.<thing>.observed
  -> signal.accepted.<source>.<thing>
  -> owning domain event
```

Raw source signals inherit `correlation_id` from the root observation and set
`causation_id` to the observation captured event id. Accepted, rejected, muted
and paused Signal Hub events set `causation_id = raw_event.event_id` and inherit
the raw event correlation id.

## Signal Control Plane

Signal Hub must support these controls from the first implementation slice:

- enable source;
- disable source;
- mute all source events;
- selectively mute by event family/type;
- pause source event publication;
- resume paused source publication;
- replay source events;
- health check;
- fixture mode;
- apply profile.

Control semantics:

| Control | Provider runtime active? | Event captured? | Event published? | Intended use |
|---|---:|---:|---:|---|
| Enabled | yes | yes | yes | normal operation |
| Disabled | no | no | no | source unavailable/off |
| Muted | yes | optional | no | test/debug/maintenance suppression |
| Paused | yes | yes | buffered | maintenance, projection rebuild, deterministic test boundary |
| Replayed | no provider call required | from event store/fixture | yes | recovery and projection rebuild |

## Runtime Model

Signal Hub should stay in the modular monolith initially.

No sidecars are introduced for Mail, Telegram, WhatsApp, Redis or provider
runtimes in the first implementation. Provider runtimes are modules in the same
backend process until a measured reason appears:

- memory isolation;
- crash isolation;
- incompatible native dependencies;
- separate scaling requirements;
- long-running provider runtime that blocks the main process.

The boundary must still make future extraction possible:

```text
InProcessSignalSource
RemoteSignalSource
```

Both implement the same `SignalSource` contract.

## Realtime UI

SSE is the browser realtime delivery path.

```text
Event persisted / projection updated
  -> projection.* event
  -> SSE Gateway
  -> browser EventSource
  -> Vue/TanStack Query cache patch
```

WebSocket hubs are not part of the target Signal Hub architecture.

## ConnectRPC API Boundary

Signal Hub command/query APIs are contract-first.

Canonical API transport:

```text
Protobuf schema
  -> ConnectRPC service
  -> generated Rust / TypeScript clients
```

Axum remains the HTTP host/router. REST compatibility endpoints may exist only
as migration shims and must not define new product contracts.

## Security Boundary

Signal Hub stores only non-secret source and connection metadata.

Allowed:

- provider kind;
- display name;
- capability snapshot;
- health state;
- runtime state;
- settings without secrets;
- `secret_ref`.

Forbidden:

- access tokens;
- refresh tokens;
- cookies;
- TDLib database keys;
- WhatsApp session blobs;
- IMAP passwords;
- OAuth client secrets;
- raw provider payloads containing private message bodies.

Secrets live in the vault/secret resolver boundary. Raw message bodies belong to
provider raw record / Communications evidence storage, not Signal Hub policy
records.
