# ADR-0038 Local Event API Capability Token

Status: Temporary

## Context

ADR-0027 requires capability-based permissions. ADR-0035 introduced local event API command/query endpoints. ADR-0037 protected writes, but `GET /api/events/{event_id}` can expose private event payloads and provenance, so protecting only mutating endpoints leaves a local data disclosure path.

The project does not yet have the full user/session/capability runtime.

## Decision

Use a temporary local API capability token for local event API endpoints.

Rules:

- `MAKOSH_LOCAL_API_TOKEN` configures the local API token.
- Empty `MAKOSH_LOCAL_API_TOKEN` is invalid configuration.
- `MAKOSH_LOCAL_WRITE_TOKEN` remains a legacy fallback during the transition from ADR-0037.
- `POST /api/events` requires `Authorization: Bearer <token>`.
- `GET /api/events/{event_id}` requires `Authorization: Bearer <token>`.
- If no local API token is configured, API calls fail closed with `503 api_token_not_configured`.
- If the bearer token is missing or invalid, API calls return `401 invalid_api_token`.
- `GET /healthz` and `GET /readyz` remain unauthenticated operational probes.

This token is a local-development and local-desktop API guard, not a substitute for the long-term capability runtime.

## Consequences

- Event reads and writes both require local API authorization.
- Local smoke and development commands must provide `MAKOSH_LOCAL_API_TOKEN`.
- Existing `MAKOSH_LOCAL_WRITE_TOKEN` development setups continue to work as a fallback.
- `docker/.env.example` may contain only a non-secret placeholder token.
- Before network exposure, multi-user access, plugins or agents can perform reads or writes, this temporary token must be replaced or wrapped by the full capability policy model.
