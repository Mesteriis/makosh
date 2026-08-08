# Zoom Module Map

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-27.

Current repository state: the modules below are present in this checkout.

## Target backend modules

```text
backend/src/integrations/zoom/
|-- client.rs
|-- runtime.rs
`-- client/
    |-- errors.rs
    |-- models.rs
    |-- store.rs
    `-- validation.rs
```

| Module | Responsibility |
|---|---|
| `client.rs` | Public integration client module surface and re-exports. |
| `runtime.rs` | Runtime module placeholder/surface. |
| `client/models.rs` | DTOs, provider constants, runtime models, OAuth/S2S authorization, token-refresh and token-maintenance DTOs, meeting/recording/transcript request models, transcript file parser and shared sanitization helper. |
| `client/store.rs` | Account setup, OAuth/S2S token exchange, refresh and maintenance, HostVault-backed secret-reference storage, runtime lifecycle, bridge ingestion, transcript file import and event append/broadcast. |
| `client/errors.rs` | Zoom-specific error type and conversions. |
| `client/validation.rs` | Request validation helpers for non-empty ids and JSON shape checks. |

## Target backend app surface

```text
backend/src/app/provider_runtime_handlers/zoom.rs
backend/src/app/handlers/zoom.rs
backend/src/app/error/response/integrations/zoom.rs
backend/src/app/router/routes/messaging.rs
backend/src/bin/makosh_zoom_edge_proxy.rs
```

| Module | Responsibility |
|---|---|
| `provider_runtime_handlers/zoom.rs` | Axum route handlers for capabilities, accounts, OAuth/S2S authorization, refresh and maintenance, runtime lifecycle, bridge ingestion and protected webhook verification. |
| `handlers/zoom.rs` | Route handler re-export surface. |
| `error/response/integrations/zoom.rs` | Integration error response mapping. |
| `router/routes/messaging.rs` | Route registration under `/api/v1/integrations/zoom`. |
| `bin/makosh_zoom_edge_proxy.rs` | Public/edge webhook proxy that preserves raw Zoom bodies and `x-zm-*` headers while adding local Макошь auth. |

## Shared platform dependencies

| Platform module | Usage |
|---|---|
| `platform/communications` | Provider account command port and provider kind metadata. |
| `platform/calls` | Provider call evidence and transcript persistence. |
| `platform/events` | Canonical event envelope append and realtime broadcast. |
| `platform/events/bus.rs` | Zoom event type constants. |
| `platform/secrets` and `vault` | Secret reference metadata and HostVault credential payload storage. |
| `fixtures/signal_hub/system_sources.toml` | Registers Zoom as provider source in Signal Hub fixtures. |

## Migration

```text
backend/migrations/0160_add_zoom_provider_kind.sql
```

The migration adds Zoom provider kind support and secret purpose support
required by provider account and secret-reference flows.

## Target frontend modules

```text
frontend/src/integrations/zoom/
|-- api/zoom.ts
|-- queries/zoomQueryKeys.ts
|-- queries/useZoomRuntimeQuery.ts
`-- types/zoom.ts
```

| Module | Responsibility |
|---|---|
| `api/zoom.ts` | API client functions for Zoom integration and authorization routes. |
| `queries/zoomQueryKeys.ts` | TanStack Query key factory for Zoom provider runtime state. |
| `queries/useZoomRuntimeQuery.ts` | Runtime status query hook and integration-only runtime/authorization mutations. |
| `types/zoom.ts` | TypeScript DTOs aligned with backend models. |

## Boundary rules

Allowed:

```text
app -> integrations/zoom runtime API
integrations/zoom -> platform/communications provider account port
integrations/zoom -> platform/calls
integrations/zoom -> platform/events
frontend integrations/zoom -> backend integration API
```

Forbidden:

```text
integrations/zoom -> domains/*
domains/* -> integrations/zoom
frontend domains/* -> frontend integrations/zoom for product communication state
```

Provider-specific caches may be used for runtime/setup state only. Canonical
communication/call/business views should use provider-neutral domains and
projections.
