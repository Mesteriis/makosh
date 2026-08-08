# ADR-0001 Event Sourcing as System Spine

Status: Proposed

## Context

Макошь must preserve years of communication, document, task and relationship history. Current state alone cannot explain why a conclusion exists or when a commitment emerged.

## Decision

Represent meaningful changes as canonical events and use those events to build projections, graph links, indexes and timelines.

## Consequences

- Historical reconstruction becomes possible.
- Projection bugs can be fixed by replay.
- Schema evolution requires versioned event payloads.
- Implementation must handle idempotency and replay from the beginning.
