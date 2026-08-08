# ADR-0005 PostgreSQL Primary Store

Status: Proposed

## Context

Макошь needs durable relational state, event storage, graph-like relationships, JSON payloads, migrations and local deployment.

## Decision

Use PostgreSQL as the primary local store.

## Consequences

- Events, entities, relationships, metadata and projection offsets can live in one transactional system.
- JSONB can support versioned payloads while typed tables support queryable state.
- Local PostgreSQL installation and lifecycle must be handled cleanly.
- PostgreSQL is not the only storage component; search and object storage remain separate.
