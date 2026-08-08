# V2 Graph Core Checklist

## Release Goal

The first Version 2 slice is complete when Макошь builds a deterministic,
read-only Knowledge Graph projection from existing Persona-compatible identity
records, communication messages and documents, exposes protected read APIs, and
renders graph-backed desktop dashboard data.

## In Scope

- PostgreSQL graph projection tables.
- Graph node, edge and evidence store.
- Deterministic graph IDs.
- Idempotent projection from Persona-compatible `persons` records, `communication_messages` and `documents`.
- Exact-email identity linking only.
- Read-only graph summary, neighborhood and search APIs.
- Desktop dashboard graph summary and read-only explorer entry point.
- Live PostgreSQL graph smoke validation.

## Out of Scope

- Fuzzy identity merge.
- Persona identity merge/split UI.
- OCR.
- Entity extraction from document text.
- Task candidate extraction.
- AI summaries.
- Graph editing.
- Mobile graph UI.

## Acceptance Gate Status

- [x] `backend/migrations/0010_create_graph_core.sql` creates graph tables and constraints.
- [x] Graph node upserts are idempotent.
- [x] Graph edge upserts are idempotent.
- [x] System-created graph edges require evidence.
- [x] V1 graph projection from Persona-compatible identity records, messages and documents is idempotent.
- [x] Exact email rules do not create fuzzy Persona merges.
- [x] `GET /api/v1/graph/summary` has auth and response coverage.
- [x] `GET /api/v1/graph/neighborhood` has auth, not-found, unsupported-depth and happy-path coverage.
- [x] `GET /api/v1/graph/search` has auth, empty-query and happy-path coverage.
- [x] `make backend-graph-smoke-dev` passes against live PostgreSQL.
- [x] `make validate` includes the graph smoke target.
- [x] `pnpm --dir frontend check` passes after graph UI wiring.
- [x] `pnpm --dir frontend build` passes after graph UI wiring.
