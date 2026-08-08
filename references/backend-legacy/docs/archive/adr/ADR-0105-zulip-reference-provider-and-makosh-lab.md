# ADR-0105: Zulip Reference Provider and Макошь Lab

Status: Proposed
Date: 2026-06-29

## Context

Макошь already treats Telegram, WhatsApp, Mail, Zoom and other external systems
as integrations rather than product domains. Product truth flows through
provider-neutral Communications, Review, Signal Hub, Timeline, Search, Memory
and target domains. That architecture is correct, but the test loop is still too
fragmented: many checks validate isolated stores, API handlers or fixtures, while
real provider traffic is validated manually or through provider-specific smoke
procedures.

A Personal Operating System needs stronger guarantees than "the endpoint returned
200". For every important external observation Макошь must be able to answer:

```text
Where did the signal originate?
Which integration observed it?
Which canonical event was appended?
Which communication record was materialized?
Which review/radar/task/person/document candidate appeared?
Which projection indexed it?
Which UI surface can explain it?
```

Zulip is a good reference provider because it can be self-hosted for local test
sessions, exposes a documented REST API at the provider edge, provides a
real-time event queue, and has communication primitives that map well to Макошь
scenarios: messages, channels, topics, reactions, edits, deletes, users, bots
and attachments.

## Decision

Макошь will add Zulip as a **Reference Communication Provider** and introduce
**Макошь Lab** as the system-level E2E harness for communication flows.

Zulip is not introduced because the product needs another chat client. It is
introduced because Макошь needs a reproducible external system that can generate
real communication events and let the test runner trace their full path through
the system.

## Scope

### In scope

- A Zulip integration package under `backend/src/integrations/zulip`.
- A provider event mapper that preserves Zulip events as raw communication
  records and feeds Signal Hub raw events using `signal.raw.zulip.*.observed`.
- A local Макошь Lab runner that can prepare/start a Zulip Docker environment via
  the official docker-zulip repository.
- Scenario files that describe provider actions and expected Макошь trace stages.
- A communication compliance suite shared by Zulip and future providers.
- Documentation for operating the lab and interpreting trace output.

### Out of scope

- Making Zulip a user-facing Макошь product domain.
- Persisting Zulip-specific business truth outside integration runtime state.
- Direct `integrations/zulip -> domains/communications` imports.
- Direct creation of Tasks, Personas, Documents or Organizations from Zulip events.
- Replacing existing provider-specific live smoke checks for Telegram, WhatsApp or
  Mail.

## Architectural rules

Inbound flow:

```text
Zulip server
↓
backend/src/integrations/zulip
↓
signal.raw.zulip.<thing>.observed
↓
Signal Hub
↓
signal.accepted.zulip.<thing>
↓
Communications accepted-signal consumer
↓
communication.<thing>.recorded
↓
Review / Timeline / Search / Memory / target workflows
```

Outbound flow:

```text
Макошь UI/App
↓
Communications command
↓
communication.outbox.queued
↓
communication.provider_command.requested
↓
application Zulip command worker
↓
integrations/zulip transport adapter
↓
Zulip REST API
↓
zulip.command.completed/failed
↓
communication.provider_command.completed/failed
```

REST is deliberately limited to the external provider adapter. It must not
become the internal Макошь communication contract, bypass Signal Hub evidence, or
replace durable Communications provider commands.

System trace flow:

```text
Макошь Lab scenario
↓
provider action
↓
provider observation
↓
raw communication record and Signal Hub event
↓
Макошь projections/workflows
↓
trace assertions
```

Every observed event must carry:

- stable `event_type`;
- `schema_version`;
- `source` with provider and account/runtime references;
- `subject` with provider object identity;
- sanitized `payload`;
- `provenance` with provider event ID and lab scenario ID when available;
- `causation_id` and `correlation_id` according to the canonical observation
  trace. Lab run IDs remain provenance/report metadata and must not replace the
  observation trace.

## Compatibility levels

Zulip becomes the first provider for the Communication Compliance Suite.
Providers are evaluated by observable capability coverage and retained evidence.

| Capability | Required for reference provider | Notes |
|---|---:|---|
| Receive message | Yes | Real-time event queue or equivalent polling. |
| Send message | Yes | Through provider command flow. |
| Thread/topic mapping | Yes | Zulip topic is canonical thread evidence. |
| Edit observation | Yes | Must produce an observed event, even if domain support is partial. |
| Delete/tombstone observation | Yes | Must preserve audit/evidence. |
| Reaction observation | Yes | Must map to communication reaction evidence. |
| Attachment observation | Phase 2 | Needs safe media transfer and quarantine policy. |
| Identity trace | Phase 2 | Provider account to Persona candidate. |
| Review candidate detection | Phase 2 | AI/rule output remains candidate only. |
| Full UI trace visualization | Phase 3 | Consumes existing Trace-First observability. |

## Consequences

Positive:

- New communication providers can be checked against the same behavior suite.
- The full path of a message can be debugged from provider action to UI
  projection.
- Trace-first observability becomes product infrastructure, not just a log viewer.
- Provider integration maturity becomes measurable.

Negative:

- The lab adds operational moving parts: Docker, a Zulip realm, bot credentials and
  environment setup.
- The first implementation will be more skeleton than finished product.
- Scenario DSL and trace assertions must be kept stable, or the suite becomes yet
  another unmaintained test artifact.

## Acceptance criteria

- `backend/src/integrations/zulip` exists and does not import business domains.
- Zulip inbound events map to canonical `signal.raw.zulip.*.observed` envelopes.
- Макошь Lab has a documented local setup and at least one runnable scenario file.
- The first scenario can send a Zulip message and produce a local trace report
  for later Макошь assertions.
- The code patch does not create a new user-facing Zulip product domain.
- Architecture and code-boundary checks remain the gate.

## Follow-up decisions

- Whether Макошь Lab scenarios become JSON, YAML or Rust fixtures long term.
- Whether provider command workers stay as per-provider application workers or
  move behind a shared runtime command dispatcher.
- Whether trace assertions are evaluated against EventStore, Signal Hub, a
  dedicated trace API, or all three.
