# ADR-0056 Local API — Simplified Auth

Status: Accepted

Supersedes: ADR-0037, ADR-0038, ADR-0040

## Context

The backend serves a **single local user** via a desktop app (Tauri shell).
There is no multi-tenancy, no external network exposure (binds `127.0.0.1`),
and no user-facing authentication.

Previous ADRs mandated per-request `MAKOSH_LOCAL_API_TOKEN` verification
and `x-makosh-actor-id` extraction in every handler. This added boilerplate
to 200+ handlers with zero security benefit for a single-user local app.

## Decision

### 1. Router-level secret check

A single `tower::layer` on the router verifies a shared secret header.
If the header is missing or wrong → 403. No per-handler auth code.

```rust
Router::new()
    .layer(require_secret_layer("X-Макошь-Secret", &secret))
    .route(...)
```

### 2. Actor identity is a constant

All audit records use `"makosh-frontend"` as the actor.
No `x-makosh-actor-id` header extraction.

```rust
NewApiAuditRecord::setting_set("makosh-frontend", "theme")
```

### 3. Handlers are plain

```rust
pub async fn list(State(state): State<AppState>) -> Result<Json<T>, ApiError> {
    let store = XStore::new(state.db.pool()?.clone());
    Ok(Json(store.list().await?))
}
```

No `verify_local_api_capability`, no `AuthActor`, no `require_auth`.

## Consequences

### Removed
- `MAKOSH_LOCAL_API_TOKEN` configuration
- `x-makosh-actor-id` header requirement
- `verify_local_api_capability()` function
- `local_api_actor()` function
- `LocalApiActor` struct
- `ApiError::ApiTokenNotConfigured`, `ApiError::InvalidApiToken`, `ApiError::InvalidActorId`

### Added
- `tower::layer` with shared secret check (one place)
- `audit_actor` constant or helper

### Migration
1. Remove token config from `AppConfig`, `docker/.env`
2. Remove token/actor verification from all handlers
3. Replace `actor.actor_id` with `"makosh-frontend"` in audit calls
4. Add router-level secret layer
5. Delete `verify_local_api_capability`, `local_api_actor`, `is_valid_actor_id_byte`, `LocalApiActor`

### Risk
- A malicious process on the same machine could call the API if it knows the secret.
  Mitigation: the secret is in an env var, Tauri IPC provides additional isolation.
  This is acceptable for a single-user desktop app.
