# ADR-0040 Local API Actor Identity

Status: Superseded by ADR-0056

## Context

ADR-0038 protects local event API access with a temporary API capability token, and ADR-0039 records authorized event API access in `api_audit_log`.

The token identifies a shared local capability, not the client or tool using that capability. Without a caller identity, audit records can prove that the local API token was used but cannot distinguish the desktop UI, CLI smoke tests, future local agents or other local development tools.

## Decision

Require a temporary local API actor identity header for protected local event API endpoints.

Rules:

- Protected local event API requests must include `Authorization: Bearer <MAKOSH_LOCAL_API_TOKEN>`.
- After the bearer token is valid, protected local event API requests must include `X-Макошь-Actor-Id`.
- `X-Макошь-Actor-Id` is a stable local client identifier, not a secret.
- The accepted actor ID character set is ASCII letters, digits, `.`, `_`, `-`, `:`, `@` and `/`.
- Actor IDs must be non-empty after trimming and at most 128 bytes.
- Missing or invalid actor IDs return `400 invalid_actor_id`.
- `api_audit_log` stores `actor_kind = local_api_token` and the supplied `actor_id`.
- Audit inspection supports filtering by `actor_id`.
- API tokens and secrets must never be stored in audit records.

## Consequences

- Local event API audit records can distinguish authorized local clients.
- Existing clients must send `X-Макошь-Actor-Id` in addition to the bearer token.
- This remains self-asserted identity while the temporary token model exists.
- The full capability runtime must replace this with authenticated capability and actor identifiers before multi-user access, plugins or agents are allowed to perform broad reads or writes.
