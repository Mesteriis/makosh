# ADR-0015 Command Query Separation

Status: Proposed

## Context

Макошь has durable state transitions and complex read models. Mixing writes, reads and AI side effects would make behavior hard to test.

## Decision

Separate commands from queries at the application boundary.

## Consequences

- Durable mutations pass through explicit validation.
- Query models can be optimized for UI and AI retrieval.
- Agents can be restricted to read-only or side-effecting tools.
- More boilerplate is required in application services.
