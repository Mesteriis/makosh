# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `031-source-backend-part-011`
- Group / Группа: `backend`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/backend.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/src/app/handlers/persons/memory.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/memory.rs`
- Size bytes / Размер в байтах: `7470`
- Included characters / Включено символов: `7018`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

// ── Person Facts ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonFactsResponse {
    items: Vec<crate::domains::persons::memory::PersonFact>,
}

pub(crate) async fn get_person_facts(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonFactsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonFactStore>(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonFactsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonFactRequest {
    fact_type: String,
    value: String,
    source: Option<String>,
    confidence: Option<f64>,
}

pub(crate) async fn post_person_fact(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonFactRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonFact>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_fact_manual(
                &person_id,
                &req.fact_type,
                &req.value,
                requested_source,
                req.confidence.unwrap_or(1.0),
            )
            .await?,
    ))
}

// ── Person Memory Cards ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonMemoryCardsResponse {
    items: Vec<crate::domains::persons::memory::PersonMemoryCard>,
}

pub(crate) async fn get_person_memory_cards(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonMemoryCardsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonMemoryCardStore>(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonMemoryCardsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonMemoryCardRequest {
    title: String,
    description: String,
    source: Option<String>,
    importance: Option<i16>,
}

pub(crate) async fn post_person_memory_card(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonMemoryCardRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonMemoryCard>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_memory_card_manual(
                &person_id,
                &req.title,
                &req.description,
                requested_source,
                req.importance.unwrap_or(5),
            )
            .await?,
    ))
}

// ── Person Preferences ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPreferencesResponse {
    items: Vec<crate::domains::persons::memory::PersonPreference>,
}

pub(crate) async fn get_person_preferences(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPreferencesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonPreferenceStore>(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPreferencesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonPreferenceRequest {
    preference_type: String,
    value: String,
    source: Option<String>,
}

pub(crate) async fn post_person_preference(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonPreferenceRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonPreference>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_preference_manual(
                &person_id,
                &req.preference_type,
                &req.value,
                requested_source,
            )
            .await?,
    ))
}

// ── Relationship Timeline ───────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct RelationshipTimelineResponse {
    items: Vec<crate::domains::persons::memory::RelationshipEvent>,
}

pub(crate) async fn get_person_timeline(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<RelationshipTimelineResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<RelationshipEventStore>(pool)
        .timeline(&person_id, query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(RelationshipTimelineResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct TimelineQuery {
    limit: Option<i64>,
}

pub(crate) async fn post_relationship_event(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewRelationshipEventRequest>,
) -> Result<Json<crate::domains::persons::memory::RelationshipEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .add_relationship_event_manual(&NewRelationshipEvent {
                person_id,
                event_type: req.event_type,
                title: req.title,
                description: req.description,
                occurred_at: req.occurred_at,
                source: req.source,
                related_entity_id: req.related_entity_id,
                related_entity_kind: req.related_entity_kind,
            })
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct NewRelationshipEventRequest {
    event_type: String,
    title: String,
    description: Option<String>,
    occurred_at: DateTime<Utc>,
    source: String,
    related_entity_id: Option<String>,
    related_entity_kind: Option<String>,
}
```

### `backend/src/app/handlers/persons/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/mod.rs`
- Size bytes / Размер в байтах: `376`
- Included characters / Включено символов: `376`
- Truncated / Обрезано: `no`

```rust
mod compatibility;
mod errors;
mod health;
mod history;
mod identity;
mod intelligence;
mod investigator;
mod memory;
mod profile;
mod support;

pub(crate) use compatibility::*;
pub(crate) use health::*;
pub(crate) use history::*;
pub(crate) use identity::*;
pub(crate) use intelligence::*;
pub(crate) use investigator::*;
pub(crate) use memory::*;
pub(crate) use profile::*;
```

### `backend/src/app/handlers/persons/profile.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/profile.rs`
- Size bytes / Размер в байтах: `387`
- Included characters / Включено символов: `387`
- Truncated / Обрезано: `no`

```rust
mod actions;
mod legacy;
mod models;
mod owner;
mod personas;
mod search;

pub(crate) use actions::{post_person_favorite, post_person_fingerprint, put_person_notes};
pub(crate) use legacy::{get_person, get_persons};
pub(crate) use owner::{get_owner_persona, put_owner_persona};
pub(crate) use personas::{get_persona, get_personas, put_persona};
pub(crate) use search::get_person_search;
```

### `backend/src/app/handlers/persons/profile/actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/profile/actions.rs`
- Size bytes / Размер в байтах: `2350`
- Included characters / Включено символов: `2350`
- Truncated / Обрезано: `no`

```rust
use super::super::support::*;

pub(crate) async fn post_person_fingerprint(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let messages = crate::app::api_support::app_store::<
        crate::domains::communications::messages::MessageProjectionStore,
    >(pool.clone())
    .recent_messages(50)
    .await?;
    let person_messages = messages
        .into_iter()
        .filter(|message| {
            message.message.sender.contains(&person_id)
                || message
                    .message
                    .recipients
                    .iter()
                    .any(|recipient| recipient.contains(&person_id))
        })
        .map(
            |message| crate::domains::persons::intelligence::PersonMessage {
                subject: message.message.subject,
                body_text: message.message.body_text,
                occurred_at: message.message.occurred_at,
            },
        )
        .collect::<Vec<_>>();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .fingerprint_person_manual(&person_id, &person_messages)
            .await?,
    ))
}

pub(crate) async fn post_person_favorite(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let fav = crate::domains::persons::service::PersonCommandService::new(pool)
        .toggle_favorite_manual(&person_id)
        .await?;
    Ok(Json(json!({"is_favorite": fav})))
}

#[derive(Deserialize)]
pub(crate) struct PersonNotesRequest {
    notes: String,
}

pub(crate) async fn put_person_notes(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<PersonNotesRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::persons::service::PersonCommandService::new(pool)
        .set_notes_manual(&person_id, &req.notes)
        .await?;
    Ok(Json(json!({"saved": true})))
}
```

### `backend/src/app/handlers/persons/profile/legacy.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/profile/legacy.rs`
- Size bytes / Размер в байтах: `1413`
- Included characters / Включено символов: `1413`
- Truncated / Обрезано: `no`

```rust
use super::super::support::*;
use super::models::PersonListResponse;

#[derive(Deserialize)]
pub(crate) struct PersonListQuery {
    favorites_only: Option<bool>,
    limit: Option<i64>,
}

pub(crate) async fn get_persons(
    State(state): State<AppState>,
    Query(query): Query<PersonListQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::persons::enrichment::PersonEnrichmentStore,
    >(pool);
    let items = store
        .list_enriched(
            query.favorites_only.unwrap_or(false),
            query.limit.unwrap_or(50),
        )
        .await?;
    Ok(Json(PersonListResponse { items }))
}

pub(crate) async fn get_person(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<crate::domains::persons::enrichment::EnrichedPerson>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::persons::enrichment::PersonEnrichmentStore,
    >(pool);
    match store.get_enriched(&person_id).await? {
        Some(person) => Ok(Json(person)),
        None => Err(ApiError::PersonIdentityNotFound),
    }
}
```

### `backend/src/app/handlers/persons/profile/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/profile/models.rs`
- Size bytes / Размер в байтах: `1745`
- Included characters / Включено символов: `1745`
- Truncated / Обрезано: `no`

```rust
use super::super::support::*;

#[derive(Serialize)]
pub(crate) struct PersonListResponse {
    pub(super) items: Vec<crate::domains::persons::enrichment::EnrichedPerson>,
}

#[derive(Serialize)]
pub(crate) struct PersonaListResponse {
    pub(super) items: Vec<PersonaReadModel>,
}

#[derive(Serialize)]
pub(crate) struct PersonaReadModel {
    persona_id: String,
    persona_type: crate::domains::persons::api::PersonaType,
    is_self: bool,
    identity: PersonaIdentityReadModel,
    communication: PersonaCommunicationReadModel,
    compatibility: PersonaCompatibilityReadModel,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub(crate) struct PersonaIdentityReadModel {
    display_name: String,
    email_address: String,
}

#[derive(Serialize)]
pub(crate) struct PersonaCommunicationReadModel {
    primary_email: String,
}

#[derive(Serialize)]
pub(crate) struct PersonaCompatibilityReadModel {
    legacy_person_id: String,
    legacy_route: &'static str,
}

pub(super) fn persona_read_model(person: Person) -> PersonaReadModel {
    PersonaReadModel {
        persona_id: person.person_id.clone(),
        persona_type: person.persona_type,
        is_self: person.is_self,
        identity: PersonaIdentityReadModel {
            display_name: person.display_name,
            email_address: person.email_address.clone(),
        },
        communication: PersonaCommunicationReadModel {
            primary_email: person.email_address,
        },
        compatibility: PersonaCompatibilityReadModel {
            legacy_person_id: person.person_id,
            legacy_route: "/api/v1/persons",
        },
        created_at: person.created_at,
        updated_at: person.updated_at,
    }
}
```

### `backend/src/app/handlers/persons/profile/owner.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/profile/owner.rs`
- Size bytes / Размер в байтах: `1227`
- Included characters / Включено символов: `1227`
- Truncated / Обрезано: `no`

```rust
use super::super::support::*;
#[derive(Serialize)]
pub(crate) struct OwnerPersonaResponse {
    owner_persona: Option<crate::domains::persons::api::Person>,
}

#[derive(Deserialize)]
pub(crate) struct SetOwnerPersonaRequest {
    person_id: String,
}

pub(crate) async fn get_owner_persona(
    State(state): State<AppState>,
) -> Result<Json<OwnerPersonaResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let owner_persona = crate::app::api_support::app_store::<PersonProjectionStore>(pool)
        .owner_persona()
        .await?;
    Ok(Json(OwnerPersonaResponse { owner_persona }))
}

pub(crate) async fn put_owner_persona(
    State(state): State<AppState>,
    Json(req): Json<SetOwnerPersonaRequest>,
) -> Result<Json<OwnerPersonaResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let owner_persona = crate::domains::persons::service::PersonCommandService::new(pool)
        .set_owner_persona_manual(&req.person_id)
        .await?;
    Ok(Json(OwnerPersonaResponse {
        owner_persona: Some(owner_persona),
    }))
}
```

### `backend/src/app/handlers/persons/profile/personas.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/profile/personas.rs`
- Size bytes / Размер в байтах: `2427`
- Included characters / Включено символов: `2427`
- Truncated / Обрезано: `no`

```rust
use super::super::support::*;
use super::models::{PersonaListResponse, PersonaReadModel, persona_read_model};
#[derive(Deserialize)]
pub(crate) struct PersonaListQuery {
    limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct PersonaUpdateRequest {
    identity: Option<PersonaIdentityUpdateRequest>,
    is_self: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct PersonaIdentityUpdateRequest {
    display_name: Option<String>,
}

pub(crate) async fn get_personas(
    State(state): State<AppState>,
    Query(query): Query<PersonaListQuery>,
) -> Result<Json<PersonaListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonProjectionStore>(pool);
    let items = store
        .list_personas(query.limit.unwrap_or(50))
        .await?
        .into_iter()
        .map(persona_read_model)
        .collect();
    Ok(Json(PersonaListResponse { items }))
}

pub(crate) async fn get_persona(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaReadModel>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonProjectionStore>(pool);
    match store.get_persona(&persona_id).await? {
        Some(persona) => Ok(Json(persona_read_model(persona))),
        None => Err(ApiError::PersonIdentityNotFound),
    }
}

pub(crate) async fn put_persona(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<PersonaUpdateRequest>,
) -> Result<Json<PersonaReadModel>, ApiError> {
    if req.is_self == Some(false) {
        return Err(ApiError::InvalidPersonaQuery(
            "is_self=false is not supported; set another Persona as owner instead",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let display_name = req
        .identity
        .as_ref()
        .and_then(|identity| identity.display_name.as_deref());
    let persona = crate::domains::persons::service::PersonCommandService::new(pool)
        .update_persona_manual(&persona_id, display_name, req.is_self == Some(true))
        .await?;
    Ok(Json(persona_read_model(persona)))
}
```

### `backend/src/app/handlers/persons/profile/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/profile/search.rs`
- Size bytes / Размер в байтах: `866`
- Included characters / Включено символов: `866`
- Truncated / Обрезано: `no`

```rust
use super::super::support::*;
use super::models::PersonListResponse;

#[derive(Deserialize)]
pub(crate) struct PersonSearchQuery {
    q: String,
    limit: Option<i64>,
}

pub(crate) async fn get_person_search(
    State(state): State<AppState>,
    Query(query): Query<PersonSearchQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery("search query required"));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::persons::enrichment::PersonEnrichmentStore,
    >(pool);
    let items = store
        .search_persons(&query.q, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(PersonListResponse { items }))
}
```

### `backend/src/app/handlers/persons/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/support.rs`
- Size bytes / Размер в байтах: `1697`
- Included characters / Включено символов: `1697`
- Truncated / Обрезано: `no`

```rust
pub(super) use axum::Json;
pub(super) use axum::extract::{Path, Query, RawQuery, State};
pub(super) use axum::http::{HeaderMap, HeaderName, HeaderValue, header};
pub(super) use chrono::{DateTime, Utc};
pub(super) use serde::{Deserialize, Serialize};
pub(super) use serde_json::{Value, json};

pub(super) use crate::app::api_support::*;
pub(super) use crate::app::{ApiError, AppState};
pub(super) use crate::domains::persons::analytics::{AnalyticsError, PersonAnalyticsService};
pub(super) use crate::domains::persons::api::{Person, PersonProjectionStore};
pub(super) use crate::domains::persons::core::{
    NewPersonPersona, PersonIdentity, PersonPersona, PersonPersonaStore, PersonRole,
    PersonRoleStore, PersonsIdentityStore,
};
pub(super) use crate::domains::persons::enrichment_engine::{
    EnrichmentEngineError, EnrichmentResultStore,
};
pub(super) use crate::domains::persons::expertise::{PersonExpertiseError, PersonExpertiseStore};
pub(super) use crate::domains::persons::export::{ExportError, ExportFormat, PersonExportService};
pub(super) use crate::domains::persons::health::{PersonHealthError, PersonHealthStore};
pub(super) use crate::domains::persons::identity::PersonIdentityDetail;
pub(super) use crate::domains::persons::investigator::{
    DossierReviewState, DossierSnapshot, InvestigatorError, PersonDossier, PersonInvestigator,
};
pub(super) use crate::domains::persons::memory::{
    NewRelationshipEvent, PersonFactStore, PersonMemoryCardStore, PersonPreferenceStore,
    RelationshipEventStore,
};
pub(super) use crate::domains::persons::trust::{
    PersonPromiseStore, PersonRiskStore, PersonTrustError,
};
pub(super) use crate::platform::audit::NewApiAuditRecord;
```

### `backend/src/app/handlers/projects/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/projects/mod.rs`
- Size bytes / Размер в байтах: `15674`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::io;

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::Html;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;
use url::form_urlencoded;

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiService, AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
};
use crate::domains::persons::analytics::{AnalyticsError, PersonAnalyticsService};
use crate::domains::persons::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::persons::expertise::{PersonExpertiseError, PersonExpertiseStore};
use crate::domains::persons::export::{ExportError, ExportFormat, PersonExportService};
use crate::domains::persons::health::{PersonHealthError, PersonHealthStore};
use crate::domains::persons::investigator::{InvestigatorError, PersonInvestigator};
use crate::engines::automation::{
    AutomationError, AutomationPolicy, AutomationStore, AutomationTemplate, NewAutomationPolicy,
    NewAutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::platform::calls::{
    CallDirection, CallError, CallIntelligenceStore, CallState, CallTranscript,
    FixtureSpeechToTextProvider, NewCallTranscript, NewTelegramCall, SpeechToTextProvider,
    TelegramCall, TranscriptStatus,
};
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};
use crate::platform::config::AppConfig;

use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore, PersonTrustError};

use crate::domains::persons::memory::{
    NewRelationshipEvent, PersonFactStore, PersonMemoryCardStore, PersonMemoryError,
    PersonPreferenceStore, RelationshipEventStore,
};

use crate::domains::persons::core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonPersona, PersonPersonaStore,
    PersonRole, PersonRoleStore, PersonsIdentityStore,
};
use crate::domains::persons::identity::{
    PersonIdentityCandidate, PersonIdentityDetail, PersonIdentityError,
    PersonIdentityReviewCommand, PersonIdentityReviewState, PersonIdentityStore,
};

use crate::application::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};
use crate::domains::calendar::brain::{CalendarBrainError, CalendarBrainService};
use crate::domains::calendar::core::{
    CalendarCoreError, ContextPackInput, EventAgendaStore, EventChecklistStore,
    EventContextPackStore, EventParticipantStore, EventRelationStore,
};
use crate::domains::calendar::events::{
    CalendarAccountStore, CalendarAccountUpdate, CalendarError, CalendarEventListQuery,
    CalendarEventStore, CalendarEventUpdate, CalendarSourceStore, NewCalendarEvent,
};
use crate::domains::calendar::health::{CalendarHealthError, CalendarWatchtowerService};
use crate::domains::calendar::intelligence::CalendarIntelligenceService;
use crate::domains::calendar::meetings::{
    EventRecordingStore, EventTranscriptStore, MeetingNoteStore, MeetingOutcomeStore, MeetingsError,
};
use crate::domains::calendar::reminders::{CalendarReminderStore, ReminderError};
use crate::domains::calendar::rules::{CalendarRuleError, CalendarRuleStore, RuleUpdate};
use crate::domains::calendar::scheduling::{
    DeadlineStore, FocusBlockStore, SchedulingError, SmartSchedulingService,
};
use crate::domains::calendar::sync::{export_event_ics, export_event_md};
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
    WorkflowState,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, StoredCommunicationAttachmentWithBlob,
};
use crate::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use crate::domains::graph::core::{GraphNodeKind, node_id};
use crate::domains::organizations::api::{
    OrganizationError, OrganizationStore, OrganizationUpdate,
};
use crate::domains::projects::core::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::domains::tasks::api::{NewTask, TaskError, TaskListQuery, TaskStore, TaskUpdate};
use crate::domains::tasks::brain::{TaskBrainError, TaskBrainService};
use crate::domains::tasks::candidates::{
    TaskCandidate, TaskCandidateError, TaskCandidateReviewCommand, TaskCandidateReviewState,
    TaskCandidateStore,
};
use crate::domains::tasks::core::{
    ExternalTaskIdentityStore, TaskChecklistStore, TaskContextPackStore, TaskCoreError,
    TaskEvidenceStore, TaskProviderStore, TaskRelationStore, TaskSubtaskStore,
};
use crate::domains::tasks::health::{TaskHealthError, TaskWatchtowerService};
use crate::domains::tasks::intelligence::TaskIntelligenceService;
use crate::domains::tasks::rules::{TaskRuleError, TaskRuleStore, TaskTemplateStore};
use crate::domains::tasks::sync::{export_task_json, export_task_md};
use crate::integrations::mail::accounts::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::platform::secrets::DatabaseEncryptedSecretVault;
use crate::platform::secrets::{SecretKind, SecretReferenceStore};
use crate::platform::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::platform::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};

use crate::app::api_support::*;
use crate::app::{ApiError, AppState};

pub(crate) async fn get_projects(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectListResponse>, ApiError> {
    let query = parse_projects_query(raw_query.as_deref())?;
    let items = project_store(&state)?.list_projects(query.limit).await?;

    Ok(Json(ProjectListResponse { items }))
}

pub(crate) async fn get_project_detail(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<crate::domains::projects::core::ProjectDetail>, ApiError> {
    let Some(project) = project_store(&state)?.project_detail(&project_id).await? else {
        return Err(ApiError::ProjectNotFound);
    };

    Ok(Json(project))
}

pub(crate) async fn get_project_link_candidates(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectLinkCandidateListResponse>, ApiError> {
    let query = parse_project_link_candidates_query(raw_query.as_deref())?;
    let project_id = validate_non_empty_project_link_field("project_id", &project_id)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();

    let project_store = project_store(&state)?;
    let review_store = project_link_review_store(&state)?;
    let mut candidates = Vec::new();

    for message in project_store.matching_project_messages(&project_id).await? {
        let graph_node_id = node_id(GraphNodeKind::Message, &message.message_id);
        let title = text_preview(&message.subject, 120);
        let sender_excerpt = text_preview(&message.sender, 140);
        crate::application::ensure_project_link_candidate_review_item(
            &pool,
            &project_id,
            ProjectLinkTargetKind::Message,
            &message.message_id,
            &title,
            &sender_excerpt,
            1.0,
            &message.observation_id,
            Some(&graph_node_id),
        )
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
        let review_state = review_store
            .explicit_review(
                &project_id,
                ProjectLinkTargetKind::Message,
                &message.message_id,
            )
            .await?
            .map(|review| review.review_state)
            .unwrap_or(ProjectLinkReviewState::Suggested);
        let occurred_at = message.occurred_at.unwrap_or(message.projected_at);

        candidates.push(ProjectLinkCandidate {
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message.as_str().to_owned(),
            target_id: message.message_id,
            graph_node_id,
            title,
            subtitle: message.sender,
            source_label: message.account_id,
            occurred_at,
            review_state: review_state.as_str().to_owned(),
            evidence_excerpt: Some(sender_excerpt),
        });
    }

    for document in project_store
        .matching_project_documents(&project_id)
        .await?
    {
        let graph_node_id = node_id(GraphNodeKind::Document, &document.document_id);
        let title = text_preview(&document.title, 140);
        crate::application::ensure_project_link_candidate_review_item(
            &pool,
            &project_id,
            ProjectLinkTargetKind::Document,
            &document.document_id,
            &title,
            &title,
            1.0,
            &document.observation_id,
            Some(&graph_node_id),
        )
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
        let review_state = review_store
            .explicit_review(
                &project_id,
                ProjectLinkTargetKind::Document,
                &document.document_id,
            )
            .await?
            .map(|review| review.review_state)
            .unwrap_or(ProjectLinkReviewState::Suggested);

        candidates.push(ProjectLinkCandidate {
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Document.as_str().to_owned(),
            target_id: document.document_id,
            graph_node_id,
            title: title.clone(),
            subtitle: document.document_kind,
            source_label: document.source_fingerprint,
            occurred_at: document.imported_at,
            review_state: review_state.as_str().to_owned(),
            evidence_excerpt: Some(title),
        });
    }

    candidates.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at));
    candidates.truncate(query.limit.unwrap_or(25));

    Ok(Json(ProjectLinkCandidateListResponse { items: candidates }))
}

pub(crate) async fn put_project_link_review(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(request): Json<ProjectLinkReviewApiRequest>,
) -> Result<Json<ProjectLinkReviewApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let command = request.into_command(project_id, actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::project_link_review_set(
            &command.actor_id,
            &command.project_id,
            command.target_kind.as_str(),
            &command.target_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/handlers/relationships/handlers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/relationships/handlers.rs`
- Size bytes / Размер в байтах: `4407`
- Included characters / Включено символов: `4407`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde_json::json;

use super::models::{
    RelationshipListQuery, RelationshipListResponse, RelationshipReviewApiRequest,
};
use crate::app::{ApiError, AppState};
use crate::application::RelationshipReviewApplicationService;
use crate::domains::relationships::{
    Relationship, RelationshipEntityKind, RelationshipReviewState, RelationshipStore,
};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};

const RELATIONSHIP_API_ACTOR_ID: &str = "makosh-frontend";
const DEFAULT_RELATIONSHIP_LIMIT: i64 = 50;
const MIN_RELATIONSHIP_LIMIT: i64 = 1;
const MAX_RELATIONSHIP_LIMIT: i64 = 100;

pub(crate) async fn get_v1_relationships(
    State(state): State<AppState>,
    Query(query): Query<RelationshipListQuery>,
) -> Result<Json<RelationshipListResponse>, ApiError> {
    let limit = validate_limit(query.limit)?;
    let store = relationship_store(&state)?;
    let items = match (
        query.review_state.as_deref(),
        query.entity_kind.as_deref(),
        query.entity_id.as_deref(),
    ) {
        (Some(review_state), None, None) => {
            let review_state = parse_review_state(review_state)?;
            store.list_by_review_state(review_state, limit).await?
        }
        (None, Some(entity_kind), Some(entity_id)) => {
            let entity_kind = parse_required_entity_kind(Some(entity_kind))?;
            let entity_id = validate_required_query_value(Some(entity_id))?;
            store
                .list_for_entity(entity_kind, &entity_id, limit)
                .await?
        }
        (Some(_), _, _) => {
            return Err(ApiError::InvalidRelationshipQuery(
                "review_state cannot be combined with entity filters",
            ));
        }
        (None, _, _) => {
            return Err(ApiError::InvalidRelationshipQuery(
                "missing required relationship query field",
            ));
        }
    };

    Ok(Json(RelationshipListResponse { items }))
}

pub(crate) async fn put_v1_relationship_review(
    State(state): State<AppState>,
    Path(relationship_id): Path<String>,
    Json(request): Json<RelationshipReviewApiRequest>,
) -> Result<Json<Relationship>, ApiError> {
    let relationship_id = validate_required_query_value(Some(&relationship_id))?;
    let review_state = parse_review_state(&request.review_state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::relationship_review_set(
            RELATIONSHIP_API_ACTOR_ID,
            &relationship_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let relationship = RelationshipReviewApplicationService::new(pool)
        .review_manual(&relationship_id, review_state)
        .await?;

    Ok(Json(relationship))
}

fn relationship_store(state: &AppState) -> Result<RelationshipStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::app_store::<RelationshipStore>(
        pool.clone(),
    ))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn parse_required_entity_kind(value: Option<&str>) -> Result<RelationshipEntityKind, ApiError> {
    let value = validate_required_query_value(value)?;
    RelationshipEntityKind::parse(&value).map_err(ApiError::from)
}

fn parse_review_state(value: &str) -> Result<RelationshipReviewState, ApiError> {
    RelationshipReviewState::parse(value).map_err(ApiError::from)
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidRelationshipQuery(
            "missing required relationship query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_RELATIONSHIP_LIMIT);
    if !(MIN_RELATIONSHIP_LIMIT..=MAX_RELATIONSHIP_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidRelationshipQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
```

### `backend/src/app/handlers/relationships/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/relationships/mod.rs`
- Size bytes / Размер в байтах: `215`
- Included characters / Включено символов: `215`
- Truncated / Обрезано: `no`

```rust
mod handlers;
mod models;

pub(crate) use handlers::{get_v1_relationships, put_v1_relationship_review};
pub(crate) use models::{
    RelationshipListQuery, RelationshipListResponse, RelationshipReviewApiRequest,
};
```

### `backend/src/app/handlers/relationships/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/relationships/models.rs`
- Size bytes / Размер в байтах: `564`
- Included characters / Включено символов: `564`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use crate::domains::relationships::Relationship;

#[derive(Debug, Deserialize)]
pub(crate) struct RelationshipListQuery {
    pub(crate) entity_kind: Option<String>,
    pub(crate) entity_id: Option<String>,
    pub(crate) review_state: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RelationshipReviewApiRequest {
    pub(crate) review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RelationshipListResponse {
    pub(crate) items: Vec<Relationship>,
}
```

### `backend/src/app/handlers/review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/review.rs`
- Size bytes / Размер в байтах: `8439`
- Included characters / Включено символов: `8439`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::application::review_promotion::ReviewPromotionService;
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxService, ReviewInboxStore, ReviewItem,
    ReviewItemKind, ReviewItemStatus, ReviewPromotionTarget,
};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

const DEFAULT_REVIEW_LIMIT: i64 = 50;
const MIN_REVIEW_LIMIT: i64 = 1;
const MAX_REVIEW_LIMIT: i64 = 100;
const REVIEW_STATUS_ACTIVE: &str = "active";
const REVIEW_STATUS_ALL: &str = "all";

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewItemsQuery {
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReviewItemsResponse {
    items: Vec<ReviewItem>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateReviewItemRequest {
    item_kind: String,
    title: String,
    summary: String,
    confidence: f64,
    metadata: Option<Value>,
    evidence: Vec<CreateReviewItemEvidenceRequest>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateReviewItemEvidenceRequest {
    observation_id: String,
    evidence_role: Option<String>,
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PromoteReviewItemRequest {
    target_domain: String,
    target_entity_kind: String,
    target_entity_id: String,
}

pub(crate) async fn get_v1_review_items(
    State(state): State<AppState>,
    Query(query): Query<ReviewItemsQuery>,
) -> Result<Json<ReviewItemsResponse>, ApiError> {
    let status = parse_status_filter(query.status.as_deref())?;
    let limit = validate_limit(query.limit)?;
    let items = match status {
        ReviewItemsStatusFilter::Single(status) => {
            review_store(&state)?.list_by_status(status, limit).await?
        }
        ReviewItemsStatusFilter::Active => review_store(&state)?.list_open(limit).await?,
        ReviewItemsStatusFilter::All => review_store(&state)?.list_all(limit).await?,
    };
    Ok(Json(ReviewItemsResponse { items }))
}

pub(crate) async fn post_v1_review_items(
    State(state): State<AppState>,
    Json(request): Json<CreateReviewItemRequest>,
) -> Result<Json<ReviewItem>, ApiError> {
    let mut item = NewReviewItem::new(
        parse_item_kind(&request.item_kind)?,
        request.title,
        request.summary,
        request.confidence,
    );
    if let Some(metadata) = request.metadata {
        item = item.metadata(metadata);
    }

    let evidence: Vec<NewReviewItemEvidence> = request
        .evidence
        .into_iter()
        .map(|item| {
            let mut evidence = NewReviewItemEvidence::new(item.observation_id);
            if let Some(role) = item.evidence_role {
                evidence = evidence.role(role);
            }
            if let Some(metadata) = item.metadata {
                evidence = evidence.metadata(metadata);
            }
            evidence
        })
        .collect();

    let item = review_store(&state)?
        .create_with_evidence(&item, &evidence)
        .await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_approve(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Approved).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_dismiss(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Dismissed).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_archive(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Archived).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_take(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::InReview).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_promote(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
    Json(request): Json<PromoteReviewItemRequest>,
) -> Result<Json<ReviewItem>, ApiError> {
    let target = ReviewPromotionTarget::new(
        request.target_domain,
        request.target_entity_kind,
        request.target_entity_id,
    );
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let observation = crate::app::api_support::app_store::<ObservationStore>(pool.clone())
        .capture(
            &NewObservation::new(
                "REVIEW_TRANSITION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "review_item_id": review_item_id,
                    "operation": "review_item_promote",
                    "target_domain": target.target_domain,
                    "target_entity_kind": target.target_entity_kind,
                    "target_entity_id": target.target_entity_id,
                }),
                format!("review-item://{review_item_id}/promote"),
            )
            .provenance(json!({
                "captured_by": "review_api.post_v1_review_item_promote",
                "endpoint": "post_v1_review_item_promote",
            })),
        )
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "review item promote observation capture failed");
            ApiError::InvalidReviewQuery("review item promote observation capture failed")
        })?;

    let item = ReviewPromotionService::new(pool.clone())
        .promote_with_observation(
            &review_item_id,
            target,
            Some(&observation.observation_id),
            Some(json!({
                "captured_by": "review_api.post_v1_review_item_promote",
                "endpoint": "post_v1_review_item_promote",
            })),
        )
        .await?;
    Ok(Json(item))
}

async fn transition_review_item_status(
    state: &AppState,
    review_item_id: &str,
    status: ReviewItemStatus,
) -> Result<ReviewItem, ApiError> {
    let item = review_service(state)?
        .transition_status_from_manual(
            review_item_id,
            status,
            "review_api.transition_review_item_status",
            "transition_review_item_status",
        )
        .await?;
    Ok(item)
}

fn review_service(state: &AppState) -> Result<ReviewInboxService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ReviewInboxService::new(pool.clone()))
}

fn review_store(state: &AppState) -> Result<ReviewInboxStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::app_store::<ReviewInboxStore>(
        pool.clone(),
    ))
}

fn parse_item_kind(value: &str) -> Result<ReviewItemKind, ApiError> {
    ReviewItemKind::parse(value).map_err(ApiError::from)
}

enum ReviewItemsStatusFilter {
    Single(ReviewItemStatus),
    Active,
    All,
}

fn parse_status_filter(value: Option<&str>) -> Result<ReviewItemsStatusFilter, ApiError> {
    match value {
        None => Ok(ReviewItemsStatusFilter::Active),
        Some(value) => match value {
            REVIEW_STATUS_ACTIVE => Ok(ReviewItemsStatusFilter::Active),
            REVIEW_STATUS_ALL => Ok(ReviewItemsStatusFilter::All),
            unknown => ReviewItemStatus::parse(unknown)
                .map(ReviewItemsStatusFilter::Single)
                .map_err(ApiError::from),
        },
    }
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_REVIEW_LIMIT);
    if !(MIN_REVIEW_LIMIT..=MAX_REVIEW_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidReviewQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
```

### `backend/src/app/handlers/settings/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/settings/mod.rs`
- Size bytes / Размер в байтах: `7957`
- Included characters / Включено символов: `7957`
- Truncated / Обрезано: `no`

```rust
use std::io;

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::Html;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;
use url::form_urlencoded;

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiService, AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
};
use crate::domains::persons::analytics::{AnalyticsError, PersonAnalyticsService};
use crate::domains::persons::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::persons::expertise::{PersonExpertiseError, PersonExpertiseStore};
use crate::domains::persons::export::{ExportError, ExportFormat, PersonExportService};
use crate::domains::persons::investigator::{InvestigatorError, PersonInvestigator};
use crate::engines::automation::{
    AutomationError, AutomationPolicy, AutomationStore, AutomationTemplate, NewAutomationPolicy,
    NewAutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::platform::calls::{
    CallDirection, CallError, CallIntelligenceStore, CallState, CallTranscript,
    FixtureSpeechToTextProvider, NewCallTranscript, NewTelegramCall, SpeechToTextProvider,
    TelegramCall, TranscriptStatus,
};
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};
use crate::platform::config::AppConfig;

use crate::domains::persons::health::{PersonHealthError, PersonHealthStore};

use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore, PersonTrustError};

use crate::domains::persons::memory::{
    NewRelationshipEvent, PersonFactStore, PersonMemoryCardStore, PersonMemoryError,
    PersonPreferenceStore, RelationshipEventStore,
};

use crate::domains::persons::core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonPersona, PersonPersonaStore,
    PersonRole, PersonRoleStore, PersonsIdentityStore,
};
use crate::domains::persons::identity::{
    PersonIdentityCandidate, PersonIdentityDetail, PersonIdentityError,
    PersonIdentityReviewCommand, PersonIdentityReviewState, PersonIdentityStore,
};

use crate::application::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};
use crate::domains::calendar::brain::{CalendarBrainError, CalendarBrainService};
use crate::domains::calendar::core::{
    CalendarCoreError, ContextPackInput, EventAgendaStore, EventChecklistStore,
    EventContextPackStore, EventParticipantStore, EventRelationStore,
};
use crate::domains::calendar::events::{
    CalendarAccountStore, CalendarAccountUpdate, CalendarError, CalendarEventListQuery,
    CalendarEventStore, CalendarEventUpdate, CalendarSourceStore, NewCalendarEvent,
};
use crate::domains::calendar::health::{CalendarHealthError, CalendarWatchtowerService};
use crate::domains::calendar::intelligence::CalendarIntelligenceService;
use crate::domains::calendar::meetings::{
    EventRecordingStore, EventTranscriptStore, MeetingNoteStore, MeetingOutcomeStore, MeetingsError,
};
use crate::domains::calendar::reminders::{CalendarReminderStore, ReminderError};
use crate::domains::calendar::rules::{CalendarRuleError, CalendarRuleStore, RuleUpdate};
use crate::domains::calendar::scheduling::{
    DeadlineStore, FocusBlockStore, SchedulingError, SmartSchedulingService,
};
use crate::domains::calendar::sync::{export_event_ics, export_event_md};
use crate::domains::communications::core::CommunicationProviderAccountStore;
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
    WorkflowState,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, StoredCommunicationAttachmentWithBlob,
};
use crate::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use crate::domains::graph::core::{GraphNodeKind, node_id};
use crate::domains::organizations::api::{
    OrganizationError, OrganizationStore, OrganizationUpdate,
};
use crate::domains::projects::core::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::domains::tasks::api::{NewTask, TaskError, TaskListQuery, TaskStore, TaskUpdate};
use crate::domains::tasks::brain::{TaskBrainError, TaskBrainService};
use crate::domains::tasks::candidates::{
    TaskCandidate, TaskCandidateError, TaskCandidateReviewCommand, TaskCandidateReviewState,
    TaskCandidateStore,
};
use crate::domains::tasks::core::{
    ExternalTaskIdentityStore, TaskChecklistStore, TaskContextPackStore, TaskCoreError,
    TaskEvidenceStore, TaskProviderStore, TaskRelationStore, TaskSubtaskStore,
};
use crate::domains::tasks::health::{TaskHealthError, TaskWatchtowerService};
use crate::domains::tasks::intelligence::TaskIntelligenceService;
use crate::domains::tasks::rules::{TaskRuleError, TaskRuleStore, TaskTemplateStore};
use crate::domains::tasks::sync::{export_task_json, export_task_md};
use crate::integrations::mail::accounts::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::platform::secrets::DatabaseEncryptedSecretVault;
use crate::platform::secrets::{SecretKind, SecretReferenceStore};
use crate::platform::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::platform::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};

use crate::app::api_support::*;
use crate::app::{ApiError, AppState};

pub(crate) async fn get_application_settings(
    State(state): State<AppState>,
) -> Result<Json<ApplicationSettingsResponse>, ApiError> {
    let items = settings_store(&state)?.list_settings().await?;

    Ok(Json(ApplicationSettingsResponse { items }))
}

pub(crate) async fn get_application_settings_accounts(
    State(state): State<AppState>,
) -> Result<Json<ApplicationAccountsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<CommunicationProviderAccountStore>(pool)
        .list()
        .await?;

    Ok(Json(ApplicationAccountsResponse { items }))
}

pub(crate) async fn put_application_setting(
    State(state): State<AppState>,
    Path(setting_key): Path<String>,
    Json(request): Json<ApplicationSettingUpdateRequest>,
) -> Result<Json<ApplicationSetting>, ApiError> {
    let actor_id = "makosh-frontend".to_string();

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::application_setting_set(
            &actor_id,
            &setting_key,
        ))
        .await?;
    let setting = settings_store(&state)?
        .update_setting_value(&setting_key, &request.value, &actor_id)
        .await?;

    Ok(Json(setting))
}
```

### `backend/src/app/handlers/signal_hub.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/signal_hub.rs`
- Size bytes / Размер в байтах: `24450`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::app::signal_hub_support::run_signal_hub_health_check;
use crate::app::{ApiError, AppState};
use crate::application::SignalHubReplayService;
use crate::domains::signal_hub::{
    FixtureRestoreReport, SignalCapability, SignalConnection, SignalConnectionCreate,
    SignalConnectionUpdate, SignalFixtureEmission, SignalFixtureEmitRequest, SignalFixtureSource,
    SignalFixtureSourceService, SignalHealth, SignalHealthCheckRequest, SignalHubCapabilityService,
    SignalHubConnectionService, SignalHubControlRequest, SignalHubControlResult,
    SignalHubControlService, SignalHubError, SignalHubHealthService, SignalHubProfileService,
    SignalHubStore, SignalPolicy, SignalPolicyMode, SignalPolicyScope, SignalProfileCreate,
    SignalProfilePolicy, SignalProfileSummary, SignalProfileUpdate, SignalReplayRequest,
    SignalReplayRequestCreate, SignalRuntimeState, SignalRuntimeStateUpdate, SignalSource,
};
use crate::platform::settings::ApplicationSettingsStore;

#[derive(Serialize)]
pub(crate) struct SignalHubSourcesResponse {
    items: Vec<SignalSource>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubPoliciesResponse {
    items: Vec<SignalPolicyDto>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubProfilesResponse {
    items: Vec<SignalProfileSummary>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubConnectionsResponse {
    items: Vec<SignalConnection>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubCapabilitiesResponse {
    items: Vec<SignalCapability>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubConnectionResponse {
    item: SignalConnection,
}

#[derive(Serialize)]
pub(crate) struct SignalHubHealthResponse {
    items: Vec<SignalHealth>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubRuntimeStatesResponse {
    items: Vec<SignalRuntimeState>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubReplayRequestsResponse {
    items: Vec<SignalReplayRequest>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubFixtureEmissionResponse {
    item: SignalFixtureEmission,
}

#[derive(Serialize)]
pub(crate) struct SignalHubFixtureSourcesResponse {
    items: Vec<SignalFixtureSource>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubPolicyRequest {
    scope: String,
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    mode: String,
    reason: String,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubCreatePolicyResponse {
    id: String,
}

#[derive(Serialize)]
pub(crate) struct SignalHubControlResponse {
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    policy_id: Option<String>,
    cleared_count: u64,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubRuntimeStateRequest {
    source_code: String,
    runtime_kind: String,
    state: String,
    #[serde(default = "empty_json_object")]
    metadata: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubHealthCheckBody {
    source_code: String,
    connection_id: Option<String>,
    runtime_kind: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubControlBody {
    scope: String,
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    reason: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubReplayRequestBody {
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    from_position: Option<i64>,
    to_position: Option<i64>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    target_consumer: Option<String>,
    target_projection: Option<String>,
    #[serde(default = "empty_json_object")]
    metadata: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubConnectionCreateRequest {
    source_code: String,
    display_name: String,
    status: String,
    profile: Option<String>,
    #[serde(default = "empty_json_object")]
    settings: serde_json::Value,
    secret_ref: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubConnectionUpdateRequest {
    display_name: Option<String>,
    status: Option<String>,
    profile: Option<String>,
    settings: Option<serde_json::Value>,
    secret_ref: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubProfileCreateRequest {
    code: String,
    display_name: String,
    description: String,
    #[serde(default)]
    source_policies: Vec<SignalProfilePolicy>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubProfileUpdateRequest {
    display_name: Option<String>,
    description: Option<String>,
    source_policies: Option<Vec<SignalProfilePolicy>>,
}

#[derive(Serialize)]
struct SignalPolicyDto {
    scope: String,
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    mode: String,
    reason: String,
    expires_at: Option<DateTime<Utc>>,
}

pub(crate) async fn get_signal_hub_sources(
    State(state): State<AppState>,
) -> Result<Json<SignalHubSourcesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubStore::new(pool).list_sources().await?;

    Ok(Json(SignalHubSourcesResponse { items }))
}

pub(crate) async fn get_signal_hub_source(
    State(state): State<AppState>,
    Path(source_code): Path<String>,
) -> Result<Json<SignalSource>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubStore::new(pool).get_source(&source_code).await?;

    Ok(Json(item))
}

#[derive(Deserialize)]
pub(crate) struct SignalHubCapabilitiesQuery {
    source_code: Option<String>,
    connection_id: Option<String>,
}

pub(crate) async fn get_signal_hub_capabilities(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SignalHubCapabilitiesQuery>,
) -> Result<Json<SignalHubCapabilitiesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubCapabilityService::new(SignalHubStore::new(pool))
        .list_capabilities(query.source_code.as_deref(), query.connection_id.as_deref())
        .await?;

    Ok(Json(SignalHubCapabilitiesResponse { items }))
}

pub(crate) async fn post_signal_hub_restore_system_fixture(
    State(state): State<AppState>,
) -> Result<Json<FixtureRestoreReport>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let report = SignalHubStore::new(pool).restore_system_sources().await?;

    Ok(Json(report))
}

pub(crate) async fn get_signal_hub_fixture_sources(
    State(state): State<AppState>,
) -> Result<Json<SignalHubFixtureSourcesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalFixtureSourceService::new(
        SignalHubStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
    )
    .list_fixture_sources()?;

    Ok(Json(SignalHubFixtureSourcesResponse { items }))
}

pub(crate) async fn get_signal_hub_profiles(
    State(state): State<AppState>,
) -> Result<Json<SignalHubProfilesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
    )
    .list_profiles()
    .await?;

    Ok(Json(SignalHubProfilesResponse { items }))
}

pub(crate) async fn post_signal_hub_profile(
    State(state): State<AppState>,
    Json(body): Json<SignalHubProfileCreateRequest>,
) -> Result<Json<SignalProfileSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
    )
    .create_profile(&SignalProfileCreate {
        code: body.code,
        display_name: body.display_name,
        description: body.description,
        source_policies: body.source_policies,
    })
    .await?;

    Ok(Json(item))
}

pub(crate) async fn post_signal_hub_apply_profile(
    State(state): State<AppState>,
    Path(profile_code): Path<String>,
) -> Result<Json<SignalProfileSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
    )
    .apply_profile(&profile_code)
    .await?;

    Ok(Json(item))
}

pub(crate) async fn patch_signal_hub_profile(
    State(state): State<AppState>,
    Path(profile_code): Path<String>,
    Json(body): Json<SignalHubProfileUpdateRequest>,
) -> Result<Json<SignalProfileSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
    )
    .update_profile(&SignalProfileUpdate {
        code: profile_code,
        display_name: body.display_name,
        description: body.description,
        source_policies: body.source_policies,
    })
    .await?;

    Ok(Json(item))
}

pub(crate) async fn delete_signal_hub_profile(
    State(state): State<AppState>,
    Path(profile_code): Path<String>,
) -> Result<Json<SignalProfileSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
    )
    .remove_profile(&profile_code)
    .await?;

    Ok(Json(item))
}

pub(crate) async fn post_signal_hub_enable_source(
    State(state): State<AppState>,
    Path(source_code): Path<String>,
) -> Result<Json<SignalHubControlResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubControlService::new(
        SignalHubStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
    )
    .enable_source(&source_code, None)
    .await?;

    Ok(Json(control_response(item)))
}

pub(crate) async fn post_signal_hub_disable_source(
    State(state): State<AppState>,
    Path(source_code): Path<String>,
) -> Result<Json<SignalHubControlResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubControlService::new(
        SignalHubStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
    )
    .disable_source(&source_code, None)
    .await?;

    Ok(Json(control_response(item)))
}

pub(crate) async fn get_signal_hub_connections(
    State(state): State<AppState>,
) -> Result<Json<SignalHubConnectionsResponse>, ApiError> {
    let pool = state
        .database
        .pool()

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/handlers/tasks/candidates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/candidates.rs`
- Size bytes / Размер в байтах: `1661`
- Included characters / Включено символов: `1661`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, RawQuery, State};
use serde_json::json;

use crate::app::api_support::{
    TaskCandidateListResponse, TaskCandidateReviewApiRequest, TaskCandidateReviewApiResponse,
    api_audit_log, observation_store, parse_task_candidates_query, task_candidate_store,
};
use crate::app::{ApiError, AppState};
use crate::application::TaskCandidateReviewApplicationService;
use crate::platform::audit::NewApiAuditRecord;
pub(crate) async fn get_task_candidates(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<TaskCandidateListResponse>, ApiError> {
    let query = parse_task_candidates_query(raw_query.as_deref())?;
    let items = task_candidate_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(TaskCandidateListResponse { items }))
}

pub(crate) async fn put_task_candidate_review(
    State(state): State<AppState>,
    Path(task_candidate_id): Path<String>,
    Json(request): Json<TaskCandidateReviewApiRequest>,
) -> Result<Json<TaskCandidateReviewApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let command = request.into_command(task_candidate_id, actor_id)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::task_candidate_review_set(
            &command.actor_id,
            &command.task_candidate_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = TaskCandidateReviewApplicationService::new(pool)
        .review_manual(&command)
        .await?;

    Ok(Json(result.into()))
}
```

### `backend/src/app/handlers/tasks/core_records.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/core_records.rs`
- Size bytes / Размер в байтах: `6585`
- Included characters / Включено символов: `6585`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::tasks::core::{
    ExternalTaskIdentity, ExternalTaskIdentityStore, TaskChecklist, TaskChecklistStore,
    TaskContextPack, TaskContextPackStore, TaskEvidence, TaskEvidenceStore, TaskRelation,
    TaskRelationStore, TaskSubtask, TaskSubtaskStore,
};
use crate::domains::tasks::service::TaskCommandService;

use super::support::database_pool;

pub(crate) async fn get_task_context_pack(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let pack = crate::app::api_support::app_store::<TaskContextPackStore>(pool)
        .get(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct UpsertContextPackRequest {
    summary: Option<String>,
    open_questions: Option<Value>,
    blockers: Option<Value>,
    risks: Option<Value>,
    suggested_next_action: Option<String>,
}

pub(crate) async fn post_task_context_pack(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<UpsertContextPackRequest>,
) -> Result<Json<TaskContextPack>, ApiError> {
    let pool = database_pool(&state)?;
    let pack = crate::app::api_support::app_store::<TaskContextPackStore>(pool)
        .upsert(
            &task_id,
            req.summary.as_deref(),
            req.open_questions.unwrap_or(json!([])),
            req.blockers.unwrap_or(json!([])),
            req.risks.unwrap_or(json!([])),
            req.suggested_next_action.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(pack))
}

#[derive(Serialize)]
pub(crate) struct TaskEvidenceResponse {
    items: Vec<TaskEvidence>,
}

pub(crate) async fn get_task_evidence(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskEvidenceResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<TaskEvidenceStore>(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskEvidenceResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewEvidenceRequest {
    source_type: Option<String>,
    source_id: Option<String>,
    quote: Option<String>,
    confidence: Option<f64>,
}

pub(crate) async fn post_task_evidence(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewEvidenceRequest>,
) -> Result<Json<TaskEvidence>, ApiError> {
    let pool = database_pool(&state)?;
    let evidence = TaskCommandService::new(pool)
        .add_evidence(
            &task_id,
            req.source_type.as_deref(),
            req.source_id.as_deref(),
            req.quote,
            req.confidence,
        )
        .await?;
    Ok(Json(evidence))
}

#[derive(Serialize)]
pub(crate) struct TaskRelationsResponse {
    items: Vec<TaskRelation>,
}

pub(crate) async fn get_task_relations(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskRelationsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<TaskRelationStore>(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskRelationsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRelationReq {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}

pub(crate) async fn post_task_relation(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewRelationReq>,
) -> Result<Json<TaskRelation>, ApiError> {
    let pool = database_pool(&state)?;
    let relation = TaskCommandService::new(pool)
        .add_relation_manual(
            &task_id,
            &req.entity_type,
            &req.entity_id,
            &req.relation_type,
        )
        .await?;
    Ok(Json(relation))
}

pub(crate) async fn get_task_checklist(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let checklist = crate::app::api_support::app_store::<TaskChecklistStore>(pool)
        .get(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&checklist).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetChecklistReq {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_task_checklist(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<SetChecklistReq>,
) -> Result<Json<TaskChecklist>, ApiError> {
    let items = req.items;
    let pool = database_pool(&state)?;
    let checklist = TaskCommandService::new(pool)
        .set_checklist_manual(&task_id, items, req.source.as_deref())
        .await?;
    Ok(Json(checklist))
}

#[derive(Serialize)]
pub(crate) struct TaskSubtasksResponse {
    items: Vec<TaskSubtask>,
}

pub(crate) async fn get_task_subtasks(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskSubtasksResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<TaskSubtaskStore>(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskSubtasksResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewSubtaskReq {
    child_task_id: String,
    sort_order: Option<i32>,
}

pub(crate) async fn post_task_subtask(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<NewSubtaskReq>,
) -> Result<Json<TaskSubtask>, ApiError> {
    let pool = database_pool(&state)?;
    let subtask = TaskCommandService::new(pool)
        .add_subtask_manual(&task_id, &req.child_task_id, req.sort_order.unwrap_or(0))
        .await?;
    Ok(Json(subtask))
}

#[derive(Serialize)]
pub(crate) struct ExtIdentitiesResponse {
    items: Vec<ExternalTaskIdentity>,
}

pub(crate) async fn get_task_external(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<ExtIdentitiesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<ExternalTaskIdentityStore>(pool)
        .list(&task_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ExtIdentitiesResponse { items }))
}
```

### `backend/src/app/handlers/tasks/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/health.rs`
- Size bytes / Размер в байтах: `1658`
- Included characters / Включено символов: `1658`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::tasks::health::TaskWatchtowerService;

use super::support::database_pool;

#[derive(Deserialize)]
pub(crate) struct WatchtowerQuery {
    days: Option<i64>,
}

pub(crate) async fn get_task_watchtower(
    State(state): State<AppState>,
    Query(q): Query<WatchtowerQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let days = q.days.unwrap_or(14);
    let overdue = TaskWatchtowerService::overdue(&pool)
        .await
        .map_err(ApiError::from)?;
    let stale = TaskWatchtowerService::stale_tasks(&pool, days)
        .await
        .map_err(ApiError::from)?;
    let no_ctx = TaskWatchtowerService::without_context(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        json!({"overdue": overdue, "stale": stale, "without_context": no_ctx}),
    ))
}

pub(crate) async fn get_task_health(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let wl = TaskWatchtowerService::workload(&pool)
        .await
        .map_err(ApiError::from)?;
    let ct = TaskWatchtowerService::cycle_time(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"workload": wl, "cycle_time": ct})))
}

pub(crate) async fn get_task_analytics(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let _pool = database_pool(&state)?;
    Ok(Json(
        json!({"analytics": "available via /tasks/health and /tasks/watchtower"}),
    ))
}
```

### `backend/src/app/handlers/tasks/intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/intelligence.rs`
- Size bytes / Размер в байтах: `3026`
- Included characters / Включено символов: `3026`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::tasks::api::{TaskStore, TaskUpdate};
use crate::domains::tasks::brain::TaskBrainService;
use crate::domains::tasks::core::{TaskContextPackStore, TaskRelationStore};
use crate::domains::tasks::service::TaskCommandService;
use crate::domains::tasks::sync::{export_task_json, export_task_md};

use super::support::database_pool;

pub(crate) async fn post_task_analyze(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let analysis = TaskCommandService::new(pool)
        .analyze_runtime(&task_id)
        .await?;
    Ok(Json(json!({
        "priority": analysis.priority,
        "risk": analysis.risk,
        "readiness": analysis.readiness,
        "missing_context": analysis.missing_context,
        "next_action": analysis.next_action
    })))
}

#[derive(Deserialize)]
pub(crate) struct TaskExportQuery {
    format: Option<String>,
}

pub(crate) async fn get_task_export(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(q): Query<TaskExportQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let task = crate::app::api_support::app_store::<TaskStore>(pool)
        .get(&task_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    match q.format.as_deref().unwrap_or("json") {
        "md" => Ok(Json(
            json!({"format":"markdown","content": export_task_md(&task.title, task.description.as_deref(), &task.makosh_status, task.why.as_deref(), task.outcome.as_deref())}),
        )),
        _ => Ok(Json(export_task_json(
            &task.title,
            task.description.as_deref(),
            &task.makosh_status,
            task.priority_score,
            task.due_at.map(|d| d.to_rfc3339()).as_deref(),
        ))),
    }
}

#[derive(Deserialize)]
pub(crate) struct TaskBrainQueryParams {
    q: String,
}

pub(crate) async fn post_task_brain(
    State(state): State<AppState>,
    Json(req): Json<TaskBrainQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let answer = TaskBrainService::explain_task(&pool, &req.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(answer))
}

#[derive(Deserialize)]
pub(crate) struct TaskSearchQueryParams {
    q: String,
}

pub(crate) async fn get_task_search(
    State(state): State<AppState>,
    Query(q): Query<TaskSearchQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let results = TaskBrainService::search_tasks(&pool, &q.q).await?;
    Ok(Json(results))
}

pub(crate) async fn get_task_daily_brief(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let brief = TaskBrainService::daily_brief(&pool).await?;
    Ok(Json(brief))
}
```

### `backend/src/app/handlers/tasks/items.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/items.rs`
- Size bytes / Размер в байтах: `2843`
- Included characters / Включено символов: `2843`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::tasks::api::{NewTask, Task, TaskListQuery, TaskStore, TaskUpdate};
use crate::domains::tasks::service::TaskCommandService;

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct TaskRecordsResponse {
    items: Vec<Task>,
}

#[derive(Deserialize)]
pub(crate) struct TaskListQueryParams {
    status: Option<String>,
    project_id: Option<String>,
    source_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_tasks(
    State(state): State<AppState>,
    Query(q): Query<TaskListQueryParams>,
) -> Result<Json<TaskRecordsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<TaskStore>(pool)
        .list(&TaskListQuery {
            status: q.status,
            project_id: q.project_id,
            source_type: q.source_type,
            limit: q.limit,
        })
        .await?;
    Ok(Json(TaskRecordsResponse { items }))
}

pub(crate) async fn post_task(
    State(state): State<AppState>,
    Json(req): Json<NewTask>,
) -> Result<Json<Task>, ApiError> {
    let pool = database_pool(&state)?;
    let task = TaskCommandService::new(pool)
        .create_task_manual(&req)
        .await?;
    Ok(Json(task))
}

pub(crate) async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    let pool = database_pool(&state)?;
    crate::app::api_support::app_store::<TaskStore>(pool)
        .get(&task_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(update): Json<TaskUpdate>,
) -> Result<Json<Task>, ApiError> {
    let pool = database_pool(&state)?;
    let task = TaskCommandService::new(pool)
        .update_task_manual(&task_id, &update)
        .await?;
    Ok(Json(task))
}

#[derive(Deserialize)]
pub(crate) struct TaskStatusRequest {
    status: String,
}

pub(crate) async fn post_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<TaskStatusRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    TaskCommandService::new(pool)
        .set_status_manual(&task_id, &req.status)
        .await?;
    Ok(Json(json!({"status": req.status})))
}

pub(crate) async fn post_task_archive(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    TaskCommandService::new(pool)
        .archive_manual(&task_id)
        .await?;
    Ok(Json(json!({"archived": true})))
}
```

### `backend/src/app/handlers/tasks/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/mod.rs`
- Size bytes / Размер в байтах: `314`
- Included characters / Включено символов: `314`
- Truncated / Обрезано: `no`

```rust
mod candidates;
mod core_records;
mod health;
mod intelligence;
mod items;
mod providers;
mod rules;
mod support;

pub(crate) use candidates::*;
pub(crate) use core_records::*;
pub(crate) use health::*;
pub(crate) use intelligence::*;
pub(crate) use items::*;
pub(crate) use providers::*;
pub(crate) use rules::*;
```

### `backend/src/app/handlers/tasks/providers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/providers.rs`
- Size bytes / Размер в байтах: `1316`
- Included characters / Включено символов: `1316`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::app::{ApiError, AppState};
use crate::domains::tasks::core::{TaskProviderAccount, TaskProviderStore};

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct TaskProvidersResponse {
    items: Vec<TaskProviderAccount>,
}

pub(crate) async fn get_task_providers(
    State(state): State<AppState>,
) -> Result<Json<TaskProvidersResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items =
        crate::app::api_support::app_store::<crate::domains::tasks::core::TaskProviderStore>(pool)
            .list()
            .await
            .map_err(ApiError::from)?;
    Ok(Json(TaskProvidersResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewTaskProviderReq {
    provider: String,
    account_name: String,
}

pub(crate) async fn post_task_provider(
    State(state): State<AppState>,
    Json(req): Json<NewTaskProviderReq>,
) -> Result<Json<TaskProviderAccount>, ApiError> {
    let pool = database_pool(&state)?;
    let provider =
        crate::app::api_support::app_store::<crate::domains::tasks::core::TaskProviderStore>(pool)
            .create(&req.provider, &req.account_name)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(provider))
}
```

### `backend/src/app/handlers/tasks/rules.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/rules.rs`
- Size bytes / Размер в байтах: `2367`
- Included characters / Включено символов: `2367`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::tasks::rules::{TaskRule, TaskRuleStore, TaskTemplate, TaskTemplateStore};

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct TaskRulesResponse {
    items: Vec<TaskRule>,
}

pub(crate) async fn get_task_rules(
    State(state): State<AppState>,
) -> Result<Json<TaskRulesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<TaskRuleStore>(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskRulesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewTaskRuleReq {
    name: String,
    description: Option<String>,
    dsl: Option<Value>,
    config: Option<Value>,
    rule_type: Option<String>,
    approval_mode: Option<String>,
}

pub(crate) async fn post_task_rule(
    State(state): State<AppState>,
    Json(req): Json<NewTaskRuleReq>,
) -> Result<Json<TaskRule>, ApiError> {
    let pool = database_pool(&state)?;
    let dsl = req.dsl.or(req.config).unwrap_or_else(|| json!({}));
    let description = req.description.or(req.rule_type);
    let rule = crate::app::api_support::app_store::<TaskRuleStore>(pool)
        .create(
            &req.name,
            description.as_deref(),
            dsl,
            req.approval_mode.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rule))
}

pub(crate) async fn delete_task_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    crate::app::api_support::app_store::<TaskRuleStore>(pool)
        .delete(&rule_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": true})))
}

#[derive(Serialize)]
pub(crate) struct TaskTemplatesResponse {
    items: Vec<TaskTemplate>,
}

pub(crate) async fn get_task_templates(
    State(state): State<AppState>,
) -> Result<Json<TaskTemplatesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<TaskTemplateStore>(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(TaskTemplatesResponse { items }))
}
```
