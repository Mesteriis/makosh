# ADR-0036 Projection Runner Checkpoint Semantics

Status: Proposed

## Context

Макошь projections must be rebuildable and resumable. `event_log.position` and `projection_cursors` define durable replay state, but workers also need consistent checkpoint semantics to avoid skipping failed events.

## Decision

Projection runners process events in ascending `event_log.position` order and save the projection cursor only after the event handler succeeds.

If a handler fails, the batch fails and the cursor remains at the last successfully processed position. The failed event remains eligible for retry.

## Consequences

- Projection workers are at-least-once by default.
- Projection handlers must be idempotent or tolerate retries.
- Failed events are not skipped accidentally.
- Future worker leasing/concurrency requires a separate ADR.
