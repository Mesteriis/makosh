# ADR-0037 Local Write Capability Token

Status: Superseded by ADR-0038

## Context

ADR-0027 requires capability-based permissions. ADR-0035 introduced a local event API command boundary, but accepting unauthenticated writes to the canonical event log is unsafe even in development because future UI, agent and plugin code may accidentally depend on an open mutation path.

The project does not yet have the full user/session/capability runtime.

## Decision

Use a temporary local write capability token for mutating backend HTTP endpoints.

This decision was superseded by ADR-0038 because read endpoints also expose private event payloads and need the same local API capability guard.

Rules:

- `MAKOSH_LOCAL_WRITE_TOKEN` configures the local write token.
- Empty `MAKOSH_LOCAL_WRITE_TOKEN` is invalid configuration.
- `POST /api/events` requires `Authorization: Bearer <token>`.
- If `MAKOSH_LOCAL_WRITE_TOKEN` is not configured, write commands fail closed with `503 write_token_not_configured`.
- If the bearer token is missing or invalid, write commands return `401 invalid_write_token`.
- `GET /healthz` and `GET /readyz` remain unauthenticated operational probes.

This token is a local-development and local-desktop command guard, not a substitute for the long-term capability runtime.

## Consequences

- Accidental unauthenticated writes to the event log are blocked.
- Local smoke and development commands must provide `MAKOSH_LOCAL_WRITE_TOKEN`.
- `docker/.env.example` may contain only a non-secret placeholder token.
- Before network exposure, multi-user access, plugins or agents can perform writes, this temporary token must be replaced or wrapped by the full capability policy model.
