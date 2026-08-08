# Backend Architecture Review - 2026-06-06

Status: Historical review.

This review captures backend architecture observations from 2026-06-06. It is
useful for traceability, but it is not the current implementation map. Current
product/domain alignment lives in
`../refactoring/implementation-alignment-plan.md`; current architecture
principles live in `../foundation/` and `../architecture/`.

## Scope

Rust backend (`backend/`) as of current inspected HEAD `a68d908`.

This review covers:

- backend module structure;
- API handler composition;
- coupling between the web layer and domain stores;
- repeated capability checks;
- query parsing;
- error conversion patterns;
- near-term refactoring order.

This review does not cover frontend, Tauri, Docker infrastructure, provider protocol correctness, or database schema design beyond how backend code composes those boundaries.

## Verification Notes

The original draft was directionally useful, but several factual points needed correction.

Verified with:

```sh
git rev-parse --short HEAD
find backend/src -type f -name '*.rs' -print0 | xargs -0 wc -l
find backend/src/bin -type f -name '*.rs' -print0 | xargs -0 wc -l
find backend/tests -type f -name '*.rs' -print0 | xargs -0 wc -l
find backend/migrations -maxdepth 1 -type f -name '*.sql' -print | sort | tail -n 5
rg -n "verify_local_api_capability\(|fn .*_store\(|parse_.*query|impl From<.*> for ApiError" backend/src/lib.rs
```

Important corrections:

- Backend source currently has **24,441 lines** under `backend/src`, including binary targets.
- Top-level backend library modules have **23,623 lines**.
- Tests have **15,369 lines** under `backend/tests`.
- There are **37 top-level library modules** and **5 binary targets**.
- Migrations currently run through **0024**, not 0023.
- `backend/src/lib.rs` is **3,784 lines**.
- Binary targets are **small** at present:
  - `makosh_email_sync_dev.rs` - 302 lines
  - `makosh_email_fixture_export.rs` - 224 lines
  - `makosh_email_fixture_dev.rs` - 148 lines
  - `makosh_graph_project.rs` - 79 lines
  - `makosh_document_process.rs` - 65 lines
- The original concern that `makosh_email_sync_dev` is a 10k-line production-sized binary is not true in the current tree.

## Current State

The backend has a strong domain-module foundation:

- `event_log` remains the append-only event spine.
- projection helpers keep cursor-based replay semantics isolated.
- stores consistently use `XxxStore { pool: PgPool }`.
- secret handling is separated through secret references, resolver boundaries and encrypted vault storage.
- application settings follow ADR-0054: declared non-secret keys, typed JSONB values and startup repair.

The main architectural issue is not the domain modules. The issue is the **web/application composition layer** in `backend/src/lib.rs`.

`lib.rs` currently contains:

- 37 public module declarations;
- all route registration;
- `AppState`;
- all handler functions;
- request and response DTOs;
- store factory helpers;
- local API capability verification;
- query parsing helpers;
- API error response mapping;
- CORS and tracing setup.

This makes `lib.rs` the main merge-conflict and change-amplification point. Every new backend feature tends to touch it in several unrelated places.

## Architecture Assessment

### What Works

#### 1. Domain stores are cohesive

Most domain modules expose a single public store or service entry point. The pattern is predictable:

```rust
pub struct XxxStore {
    pool: PgPool,
}

impl XxxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

That is a good fit for this project. Do not replace it with a broad repository abstraction.

#### 2. Event sourcing remains clean

The event log and projection cursor code preserve the current ADR direction:

- events are canonical facts;
- projections are rebuildable;
- projection cursor updates happen after successful handling;
- search and AI-derived state are downstream, not source of truth.

#### 3. Secret boundaries are explicit

The code keeps provider credentials out of ordinary settings and account config. This matches ADR-0053 and ADR-0054.

#### 4. The backend is already testable

The project has substantial backend tests and API tests using `tower::ServiceExt::oneshot`. That means route-module extraction can be protected by existing tests rather than done blind.

## Problems And Solutions

### P1 - `backend/src/lib.rs` is the composition monolith

Severity: High

Current risk:

- unrelated route changes collide in one file;
- request/response DTOs are far from their handlers;
- adding a new API surface requires touching route registration, handler functions, store factories and error conversion in one place;
- route-level tests cannot easily target a small module boundary.

Solution:

Extract route modules gradually, not as one large rewrite.

Recommended structure:

```text
backend/src/
  routes/
    mod.rs
    health.rs
    settings.rs
    graph.rs
    projects.rs
    tasks.rs
    documents.rs
    communications.rs
    ai.rs
    telegram.rs
    whatsapp.rs
    email_accounts.rs
    events.rs
    audit.rs
```

Each route module should own:

- route registration for its surface;
- request DTOs;
- response DTOs;
- handlers;
- route-local query parsers.

Keep shared cross-cutting pieces in `lib.rs` or a small `api` module at first:

- `AppState`;
- `ApiError`;
- local API capability extractors;
- shared response helpers;
- shared query parsing helpers.

Target shape:

```rust
mod routes;

pub fn build_router_with_database(config: AppConfig, database: Database) -> Router {
    let state = AppState {
        config,
        database,
        account_setup: AccountSetupState::default(),
    };

    Router::new()
        .merge(routes::health::routes())
        .merge(routes::settings::routes())
        .merge(routes::graph::routes())
        .merge(routes::events::routes())
        .with_state(state)
        .layer(local_frontend_cors_layer())
}
```

Do not extract every route module at once. Start with `settings`, because it is cohesive and already has focused tests.

### P1 - Capability verification is repeated in handlers

Severity: High

`verify_local_api_capability(&state.config, &headers)?` is repeated across many handlers. Some handlers need the actor, some only need proof that local API capability was verified.

This is not just duplication. It makes it easy to add a new route and forget the guard.

Solution:

Use Axum extractors:

```rust
#[derive(Clone, Debug)]
struct LocalApiActor {
    actor_id: String,
}

struct LocalApiVerified;
```

Implement extraction against `AppState`:

```rust
impl FromRequestParts<AppState> for LocalApiActor {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        verify_local_api_capability(&state.config, &parts.headers)
    }
}

impl FromRequestParts<AppState> for LocalApiVerified {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        verify_local_api_capability(&state.config, &parts.headers)?;
        Ok(Self)
    }
}
```

Handlers then express authorization in the signature:

```rust
async fn put_application_setting(
    actor: LocalApiActor,
    State(state): State<AppState>,
    Path(setting_key): Path<String>,
    Json(request): Json<ApplicationSettingUpdateRequest>,
) -> Result<Json<ApplicationSetting>, ApiError> {
    let updated = settings_store(&state)?
        .update_setting_value(&setting_key, &request.value, &actor.actor_id)
        .await?;

    Ok(Json(updated))
}
```

This should be implemented before route-module extraction. It makes later route moves safer because missing auth becomes visible in handler signatures.

### P2 - Store factory helpers repeat database-pool extraction

Severity: Medium

There are many helpers with the same shape:

```rust
fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApplicationSettingsStore::new(pool.clone()))
}
```

Solution:

Do the smallest useful refactor first:

```rust
impl AppState {
    fn pool(&self) -> Result<PgPool, ApiError> {
        self.database
            .pool()
            .cloned()
            .ok_or(ApiError::DatabaseNotConfigured)
    }
}
```

Then helpers become:

```rust
fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    Ok(ApplicationSettingsStore::new(state.pool()?))
}
```

Do not introduce a generic `FromPool` trait as the first step. It requires touching every store module and adds indirection before the bigger route split. A simple `AppState::pool()` removes the repeated failure branch while keeping the existing explicit store helpers.

After route modules exist, reconsider whether store helpers should remain functions or become `AppState` methods.

### P2 - Query parsing is duplicated

Severity: Medium

There are nine query parsing helpers in `lib.rs`, several of which parse `limit` with similar logic:

- `parse_communication_messages_query`
- `parse_graph_neighborhood_query`
- `parse_graph_nodes_query`
- `parse_graph_search_query`
- `parse_projects_query`
- `parse_project_link_candidates_query`
- `parse_task_candidates_query`
- `parse_document_processing_jobs_query`
- `parse_persona_identity_candidates_query`

Solution:

Add a small internal helper that parses typed query parameters and lets each route decide its error code.

Example:

```rust
fn query_param<'a>(raw_query: Option<&'a str>, name: &str) -> Option<String> {
    let raw = raw_query?;
    form_urlencoded::parse(raw.as_bytes())
        .find_map(|(key, value)| (key.as_ref() == name).then(|| value.into_owned()))
}

fn parse_limit_param(
    raw_query: Option<&str>,
    min: usize,
    max: usize,
    invalid: &'static str,
) -> Result<Option<usize>, &'static str> {
    let Some(raw_limit) = query_param(raw_query, "limit") else {
        return Ok(None);
    };

    let limit = raw_limit.parse::<usize>().map_err(|_| invalid)?;
    if limit < min || limit > max {
        return Err(invalid);
    }

    Ok(Some(limit))
}
```

Route-specific parsers still return route-specific `ApiError` variants:

```rust
fn parse_projects_query(raw_query: Option<&str>) -> Result<ProjectsQuery, ApiError> {
    Ok(ProjectsQuery {
        limit: parse_limit_param(raw_query, 1, 100, "limit must be between 1 and 100")
            .map_err(ApiError::InvalidProjectQuery)?
            .unwrap_or(25),
    })
}
```

This avoids hiding HTTP error semantics behind a generic parser.

### P2 - `ApiError` is too broad but should not be abstracted too early

Severity: Medium

The original draft proposed an `IntoApiError` trait and blanket `From<T> for ApiError`. That is a possible end state, but it is not the safest first refactor.

Current `ApiError` has domain-specific behavior that matters:

- some errors are logged as server errors;
- some validation errors become 400;
- some missing records become 404;
- event conflicts become 409;
- local API capability errors include `WWW-Authenticate` behavior.

Solution:

Keep `ApiError` centralized during the first route split. Only extract smaller helpers inside `IntoResponse`, for example:

```rust
fn internal_error(code: &'static str, message: &'static str) -> (StatusCode, &'static str, String, bool) {
    (StatusCode::INTERNAL_SERVER_ERROR, code, message.to_owned(), false)
}
```

After route modules are stable, consider one of these:

1. Keep a single `ApiError` enum but move `From` impls to `api/errors.rs`.
2. Add small domain-to-HTTP traits only for domains that have repeated NotFound/Invalid patterns.
3. Avoid blanket `From<T>` if it makes sensitive error mapping less explicit.

### P3 - Binary target size is not a current problem

Severity: None

The original draft stated that `makosh_email_sync_dev` was over 10k lines. Current source inspection shows it is 302 lines. The binary targets are not the present bottleneck.

Recommendation:

No action now. Keep binary targets as thin CLI wrappers. If a binary grows past roughly 500-700 lines or starts duplicating backend runtime logic, extract shared orchestration into a library module.

### P3 - Root Cargo workspace is still missing

Severity: Low

There is no root `Cargo.toml`. That is acceptable today because only `backend/` is a Rust crate, but ADR-0004 expects a Tauri desktop shell later.

Solution:

Add a root workspace when either of these happens:

- Tauri crate is added;
- another Rust crate is added for shared desktop/backend code.

Minimal root file:

```toml
[workspace]
members = ["backend"]
resolver = "2"
```

Do not add this immediately unless there is a current workflow benefit. It is low risk, but it is not on the critical backend architecture path.

## Recommended Implementation Order

### PR 1 - Low-risk cleanup in `lib.rs`

Goal: reduce repetition without moving route modules yet.

Changes:

- add `AppState::pool()`;
- update store helpers to use it;
- add tests only if existing coverage fails to cover disabled-database behavior.

Validation:

```sh
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path backend/Cargo.toml --all
```

### PR 2 - Local API capability extractors

Goal: make authorization explicit and prevent unguarded route additions.

Changes:

- add `LocalApiActor`;
- add `LocalApiVerified`;
- update handlers incrementally;
- preserve existing `verify_local_api_capability` internals initially.

Validation:

- existing API tests;
- targeted tests for missing token, invalid token and missing actor ID.

### PR 3 - Query parser helper

Goal: reduce duplicated query parsing while preserving route-specific error messages.

Changes:

- add `query_param`;
- add `parse_limit_param`;
- update parsers one domain at a time.

Validation:

- existing API query tests;
- add regression tests for invalid and out-of-range `limit` when gaps exist.

### PR 4 - Extract `routes/settings.rs`

Historical note: the route paths and auth extraction language below reflect the
2026-06-06 review state. Current implementation and docs use `/api/v1/settings`
and ADR-0056 router-level `X-Макошь-Secret` auth.

Goal: prove the route-module pattern on a small, cohesive API surface.

Move:

- `/api/v2/settings`;
- `/api/v2/settings/accounts`;
- `/api/v2/settings/{setting_key}`;
- settings request/response DTOs;
- settings handlers.

Keep in shared API layer:

- `AppState`;
- `ApiError`;
- capability extractors;
- common store helpers.

Validation:

- `backend/tests/settings.rs`;
- full backend test gate.

### PR 5 - Extract security-sensitive event/audit routes

Goal: isolate the event API command boundary and audit API without changing behavior.

Move:

- `/api/events`;
- `/api/events/{event_id}`;
- `/api/audit/events`;
- event/audit DTOs and handlers.

This PR should be reviewed carefully against ADR-0038, ADR-0039 and ADR-0040.

### PR 6+ - Extract remaining route groups

Suggested order:

1. `routes/graph.rs`
2. `routes/projects.rs`
3. `routes/tasks.rs`
4. `routes/documents.rs`
5. `routes/communications.rs`
6. `routes/ai.rs`
7. `routes/telegram.rs`
8. `routes/whatsapp.rs`
9. `routes/email_accounts.rs`

Do not combine all of these into one PR. The point is to reduce risk and merge conflicts.

## Acceptance Criteria

The backend route refactor is successful when:

- `backend/src/lib.rs` is mostly composition and shared infrastructure, not the home of all handlers;
- every protected route expresses authorization in the handler signature;
- route modules own their own DTOs and route-local query parsing;
- all existing API paths and response shapes remain unchanged;
- `make backend-test` passes;
- `make backend-validate` or `make validate` passes before broad merge;
- no database migration is required by the refactor;
- no provider adapter or domain behavior changes are mixed into route extraction PRs.

## Risks

### Risk: moving handlers changes route behavior

Mitigation:

- keep route paths exactly the same;
- use `tower::ServiceExt::oneshot` API tests as regression coverage;
- extract one route group per PR.

### Risk: capability extractor changes auth semantics

Mitigation:

- keep `verify_local_api_capability` as the single source of auth truth during the first extractor PR;
- add direct tests for missing token, invalid token and invalid actor ID;
- do not change audit semantics in the same PR.

### Risk: generic error abstraction hides security-sensitive messages

Mitigation:

- do not introduce blanket error conversion until route extraction is stable;
- keep security and secret-related errors explicit.

### Risk: route split blocks feature work

Mitigation:

- start with `AppState::pool()`, extractors and one small route module;
- avoid broad mechanical rewrites;
- do not refactor domain modules while extracting route modules.

## Final Recommendation

The backend architecture is fundamentally sound at the domain and persistence layers. The urgent issue is **application-layer composition**, not the event model, store pattern or database design.

The best solution is a staged extraction:

1. normalize shared `AppState` helpers;
2. move local API capability verification into Axum extractors;
3. deduplicate query parsing;
4. extract one route module at a time, starting with Settings;
5. keep `ApiError` centralized until route boundaries stabilize.

Avoid a single large `lib.rs` rewrite. The current codebase has enough tests to support incremental extraction, and that is the lowest-risk path.

## Validation For This Review Update

This document update was based on repository inspection only. Backend compile/test validation was not run for this documentation change.

Recommended validation before implementing the proposed backend changes:

```sh
make backend-validate
```

For broader changes that touch Docker-backed smoke paths:

```sh
make validate
```
