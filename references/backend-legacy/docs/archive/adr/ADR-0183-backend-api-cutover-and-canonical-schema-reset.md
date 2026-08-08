# ADR-0183: Backend API Cutover and Canonical Schema Reset

Status: Superseded by ADR-0184
Date: 2026-07-15

Supersedes: The transitional API-compatibility portions of ADR-0181

Clarifies:

- ADR-0181 Backend Workspace Modularity and Provider Runtime Topology
- ADR-0182 WhatsApp Hidden WebView Runtime Only
- ADR Architecture Communication Contract

## Context

The first workspace slices deliberately kept the `makosh-backend` package,
legacy HTTP routes and frontend adapters while ownership moved between crates.
That was useful during discovery, but retaining two contracts indefinitely would
leave the old composition root and its stores as permanent dependencies. Макошь
has one frontend and one Tauri desktop consumer, so the final migration can be
performed as an atomic vertical cutover instead of preserving a second public
API.

The current PostgreSQL data is local development state. The refactor therefore
does not need a row-by-row legacy data migration. A new canonical schema may be
bootstrapped from the final owners; vault credentials and provider session state
remain untouched and are not deleted by schema reset.

## Decision

### One breaking API cutover

ConnectRPC is the only product command/query transport. Each durable product
context owns a versioned `makosh.<context>.v2` service and uses shared
`makosh.common.v2` identifiers, pagination, timestamps, field violations and
typed errors. Generated TypeScript messages are the frontend wire model; a
mapping is allowed only where the UI model is intentionally different.

HTTP remains only for health/readiness, OAuth callbacks, blob transfer,
canonical SSE realtime and the protected private WhatsApp WebView bridge.
Browser realtime is one authenticated `/api/realtime/v2/events` SSE stream with
the canonical event envelope, replay cursor, reconnect and lag reporting.
WebSocket, long-poll fallback and `/api/v1/events` are removed in the same
cutover.

Every service slice must include the protobuf contract, Rust implementation,
generated frontend code, Vue Query consumers, Tauri consumers when applicable,
fixtures/tests, deletion of the old route and a negative search proving that the
old client/import/path is gone. Dual APIs, compatibility aliases, wildcard
facades and compatibility re-exports are not permitted.

### Composition and ownership

`makosh-desktop-runtime` is the sole composition root. `makosh-api` contains
HTTP/ConnectRPC composition over narrow application ports, and
`makosh-worker-runtime` owns supervised worker scheduling. Neither package may
construct concrete stores, provider implementations, SQL adapters or vault
implementations. Provider implementations and persistence adapters are wired
only by desktop composition.

The runtime policy from ADR-0181 remains: in-process is the default provider
topology; connector processes are explicit, fenced and never selected by silent
fallback. WhatsApp remains hidden WebView only under ADR-0182.

### Canonical schema reset

After all durable owners have moved to their `*-api` and `*-postgres` crates,
the workspace keeps one canonical baseline schema owned by `makosh-schema`.
Legacy migrations and obsolete provider/runtime shapes are removed; no
destructive down migration is created. Development and test PostgreSQL are
recreated from that baseline, projections and indexes are rebuilt, and
credentials/session state in the vault is preserved.

## Consequences

- Frontend and Tauri changes are part of the same commit as each backend API
  cut; a route is not kept solely for an old consumer.
- Rollback before schema reset is a code revert. After reset, rollback means
  restoring the corresponding code/schema commit and recreating the database.
- Existing persisted provider strings are intentionally removed only in the
  canonical reset slice after all consumers have switched.
- Fresh fixture/testcontainer validation, architecture guards and packaging
  smoke are required before removing the legacy binary name.

## Non-goals

This ADR does not redesign product UI, introduce live provider accounts, delete
vault data, or authorize remote provider actions. It also does not use file size
as a decomposition rule: modules are split when ownership or reasons to change
diverge; line counts are only a diagnostic guard.
