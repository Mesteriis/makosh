# ADR-0033 Backend Managed Local Schema Migrations

Status: Proposed

## Context

Макошь is a local-first desktop product with PostgreSQL as primary store. The user must be able to start the local app without manually applying schema changes, while schema evolution still needs explicit, reviewable migration files.

## Decision

The Rust backend owns local PostgreSQL migrations and applies embedded migrations at startup when `DATABASE_URL` is configured.

Migration files live in `backend/migrations/` and must be append-only once released. Durable schema changes require tests and, when architecturally significant, an ADR update.

`GET /readyz` must verify that the required backend-managed schema is present, not only that PostgreSQL accepts `SELECT 1`.

## Consequences

- Local development and desktop startup can keep schema and backend version aligned.
- Schema changes remain explicit in versioned SQL files.
- Readiness can catch schema drift or missing migrations before API handlers operate on missing tables.
- A bad migration can block startup, so migrations require smoke validation against development PostgreSQL.
- Future production or self-hosted deployment may require a separate migration execution policy.
