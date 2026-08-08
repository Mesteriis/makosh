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

- Chunk ID / ID чанка: `028-source-backend-part-008`
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

### `backend/src/app/handlers/communications/communication_queries/imports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/imports.rs`
- Size bytes / Размер в байтах: `2582`
- Included characters / Включено символов: `2582`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use crate::domains::communications::service::{
    CommunicationAttachmentImportCommand, CommunicationCommandService,
};
use crate::domains::communications::storage::ImportedCommunicationAttachment;

#[derive(Deserialize)]
pub(crate) struct CommunicationAttachmentImportRequest {
    pub(crate) account_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) content_base64: String,
    pub(crate) source_kind: Option<String>,
    pub(crate) metadata: Option<Value>,
}

#[derive(Serialize)]
pub(crate) struct CommunicationAttachmentImportResponse {
    pub(crate) attachment_id: String,
    pub(crate) account_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) blob_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) scan_status: String,
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
}

pub(crate) async fn post_v1_attachment_import(
    State(state): State<AppState>,
    Json(request): Json<CommunicationAttachmentImportRequest>,
) -> Result<Json<CommunicationAttachmentImportResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let imported = CommunicationCommandService::new(pool)
        .import_attachment(CommunicationAttachmentImportCommand {
            account_id: request.account_id,
            channel_kind: request.channel_kind,
            filename: request.filename,
            content_type: request.content_type,
            content_base64: request.content_base64,
            source_kind: request.source_kind,
            metadata: request.metadata,
        })
        .await?;
    Ok(Json(import_response(imported)))
}

fn import_response(
    imported: ImportedCommunicationAttachment,
) -> CommunicationAttachmentImportResponse {
    CommunicationAttachmentImportResponse {
        attachment_id: imported.attachment_id,
        account_id: imported.account_id,
        channel_kind: imported.channel_kind,
        blob_id: imported.blob_id,
        filename: imported.filename,
        content_type: imported.content_type,
        size_bytes: imported.size_bytes,
        sha256: imported.sha256,
        scan_status: imported.scan_status.as_str().to_owned(),
        storage_kind: imported.storage_kind,
        storage_path: imported.storage_path,
    }
}
```

### `backend/src/app/handlers/communications/communication_queries/outbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/outbox.rs`
- Size bytes / Размер в байтах: `2173`
- Included characters / Включено символов: `2173`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use crate::domains::communications::service::CommunicationCommandService;

#[derive(Deserialize)]
pub(crate) struct OutboxListQuery {
    account_id: Option<String>,
    status: Option<String>,
    cursor: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct OutboxListResponse {
    items: Vec<crate::domains::communications::outbox::CommunicationOutboxItem>,
    next_cursor: Option<String>,
    has_more: bool,
}

pub(crate) async fn get_v1_outbox(
    State(state): State<AppState>,
    Query(query): Query<OutboxListQuery>,
) -> Result<Json<OutboxListResponse>, ApiError> {
    let status = match query.status.as_deref() {
        Some(value) => Some(
            crate::domains::communications::outbox::CommunicationOutboxStatus::parse(value).ok_or(
                ApiError::InvalidCommunicationQuery("invalid outbox status value"),
            )?,
        ),
        None => None,
    };
    let page = outbox_store(&state)?
        .list_page(
            query.account_id.as_deref(),
            status,
            query.cursor.as_deref(),
            query.limit.unwrap_or(100),
        )
        .await?;

    Ok(Json(OutboxListResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn post_v1_outbox_undo(
    State(state): State<AppState>,
    Path(outbox_id): Path<String>,
) -> Result<Json<crate::domains::communications::outbox::CommunicationOutboxItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = CommunicationCommandService::new(pool)
        .undo_outbox(&outbox_id)
        .await?;

    Ok(Json(item))
}

pub(super) fn outbox_store(
    state: &AppState,
) -> Result<crate::domains::communications::outbox::CommunicationOutboxStore, ApiError> {
    Ok(crate::app::api_support::app_store::<
        crate::domains::communications::outbox::CommunicationOutboxStore,
    >(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    ))
}
```

### `backend/src/app/handlers/communications/communication_queries/personas.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/personas.rs`
- Size bytes / Размер в байтах: `2235`
- Included characters / Включено символов: `2235`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Serialize)]
pub(crate) struct PersonaListResponse {
    pub(super) items: Vec<crate::domains::communications::personas::CommunicationPersona>,
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaRequest {
    pub(super) persona_id: String,
    pub(super) name: String,
    pub(super) account_id: String,
    pub(super) display_name: String,
    pub(super) signature: Option<String>,
    pub(super) default_language: Option<String>,
    pub(super) default_tone: Option<String>,
    pub(super) is_default: Option<bool>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn get_v1_personas(
    State(state): State<AppState>,
) -> Result<Json<PersonaListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::personas::CommunicationPersonaStore,
    >(pool);
    let items = store.list().await?;
    Ok(Json(PersonaListResponse { items }))
}

pub(crate) async fn post_v1_persona(
    State(state): State<AppState>,
    Json(request): Json<NewPersonaRequest>,
) -> Result<Json<crate::domains::communications::personas::CommunicationPersona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::personas::CommunicationPersonaStore,
    >(pool);
    let persona = store
        .upsert(
            &crate::domains::communications::personas::NewCommunicationPersona {
                persona_id: request.persona_id,
                name: request.name,
                account_id: request.account_id,
                display_name: request.display_name,
                signature: request.signature.unwrap_or_default(),
                default_language: request.default_language,
                default_tone: request.default_tone,
                is_default: request.is_default.unwrap_or(false),
                metadata: request.metadata.unwrap_or(serde_json::json!({})),
            },
        )
        .await?;
    Ok(Json(persona))
}
```

### `backend/src/app/handlers/communications/communication_queries/read_receipts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/read_receipts.rs`
- Size bytes / Размер в байтах: `5035`
- Included characters / Включено символов: `5035`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use crate::domains::communications::delivery_notifications::{
    CommunicationDeliveryNotificationError, CommunicationDeliveryNotificationRecord,
    NewCommunicationDeliveryNotification, NewProviderDeliveryEvent,
    project_accepted_mail_delivery_signal_if_runtime_allows,
    provider_event_from_delivery_notification,
};
use crate::domains::communications::read_receipts::{
    CommunicationReadReceipt, CommunicationReadReceiptStore, NewCommunicationReadReceipt,
};
use crate::domains::signal_hub::{MailDeliverySignalRequest, dispatch_mail_delivery_event_signal};

pub(crate) async fn post_v1_read_receipt(
    State(state): State<AppState>,
    Json(request): Json<NewCommunicationReadReceipt>,
) -> Result<Json<CommunicationReadReceipt>, ApiError> {
    Ok(Json(read_receipt_store(&state)?.record(request).await?))
}

pub(crate) async fn post_v1_delivery_notification(
    State(state): State<AppState>,
    Json(request): Json<NewCommunicationDeliveryNotification>,
) -> Result<Json<CommunicationDeliveryNotificationRecord>, ApiError> {
    let provider_event = provider_event_from_delivery_notification(&request)?;
    Ok(Json(
        dispatch_provider_delivery_event(state, provider_event)
            .await?
            .ok_or_else(|| {
                CommunicationDeliveryNotificationError::SignalControlBlocked(
                    "mail delivery notification was accepted by Signal Hub but deferred by runtime control"
                        .to_owned(),
                )
            })?,
    ))
}

pub(crate) async fn post_v1_provider_delivery_event(
    State(state): State<AppState>,
    Json(request): Json<NewProviderDeliveryEvent>,
) -> Result<Json<CommunicationDeliveryNotificationRecord>, ApiError> {
    Ok(Json(
        dispatch_provider_delivery_event(state, request)
            .await?
            .ok_or_else(|| {
                CommunicationDeliveryNotificationError::SignalControlBlocked(
                    "mail provider delivery event was accepted by Signal Hub but deferred by runtime control"
                        .to_owned(),
                )
            })?,
    ))
}

fn read_receipt_store(state: &AppState) -> Result<CommunicationReadReceiptStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::app::api_support::app_store::<
        CommunicationReadReceiptStore,
    >(pool))
}

async fn dispatch_provider_delivery_event(
    state: AppState,
    request: NewProviderDeliveryEvent,
) -> Result<Option<CommunicationDeliveryNotificationRecord>, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let occurred_at = request.occurred_at.unwrap_or_else(chrono::Utc::now);
    let account_id = request.account_id.clone();
    let provider_message_id = request.provider_message_id.clone();
    let source_kind = request
        .source_kind
        .clone()
        .unwrap_or_else(|| "provider_event".to_owned());
    let provider_record_id = request.provider_record_id.clone();
    let raw_record_id = request.raw_record_id.clone();
    let payload = serde_json::json!({
        "account_id": request.account_id,
        "provider_message_id": request.provider_message_id,
        "event_kind": request.event_kind.as_str(),
        "occurred_at": occurred_at,
        "recipient": request.recipient,
        "source_kind": request.source_kind,
        "smtp_status": request.smtp_status,
        "provider_record_id": provider_record_id,
        "raw_record_id": raw_record_id,
        "reporting_ua": request.metadata.as_ref().and_then(|m| m.get("reporting_ua")).cloned(),
    });
    let event_kind = match request.event_kind {
        crate::domains::communications::delivery_notifications::ProviderDeliveryEventKind::Read => {
            "read_receipt"
        }
        crate::domains::communications::delivery_notifications::ProviderDeliveryEventKind::Delivered
        | crate::domains::communications::delivery_notifications::ProviderDeliveryEventKind::Delayed
        | crate::domains::communications::delivery_notifications::ProviderDeliveryEventKind::Failed => {
            "delivery_status"
        }
    };
    let accepted = dispatch_mail_delivery_event_signal(
        pool.clone(),
        MailDeliverySignalRequest {
            occurred_at,
            account_id: &account_id,
            provider_message_id: &provider_message_id,
            event_kind,
            payload,
            source_kind: &source_kind,
            provider_record_id: provider_record_id.as_deref(),
            raw_record_id: raw_record_id.as_deref(),
            correlation_id: provider_record_id.as_deref().or(raw_record_id.as_deref()),
        },
    )
    .await?;

    let Some(accepted_event) = accepted else {
        return Ok(None);
    };

    project_accepted_mail_delivery_signal_if_runtime_allows(pool, &accepted_event)
        .await
        .map_err(ApiError::from)
}
```

### `backend/src/app/handlers/communications/communication_queries/saved_searches.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/saved_searches.rs`
- Size bytes / Размер в байтах: `3298`
- Included characters / Включено символов: `3298`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use crate::domains::communications::saved_searches::{
    CommunicationSavedSearch, CommunicationSavedSearchListQuery, CommunicationSavedSearchStore,
    NewCommunicationSavedSearch, UpdateCommunicationSavedSearch,
};
use crate::domains::communications::service::CommunicationCommandService;

#[derive(Deserialize)]
pub(crate) struct SavedSearchesQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) smart_folder: Option<bool>,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct SavedSearchListResponse {
    pub(crate) items: Vec<CommunicationSavedSearch>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) has_more: bool,
}

#[derive(Serialize)]
pub(crate) struct SavedSearchDeleteResponse {
    pub(crate) deleted: bool,
}

pub(crate) async fn get_v1_saved_searches(
    State(state): State<AppState>,
    Query(query): Query<SavedSearchesQuery>,
) -> Result<Json<SavedSearchListResponse>, ApiError> {
    let page = saved_search_store(&state)?
        .list(CommunicationSavedSearchListQuery {
            account_id: query.account_id.as_deref(),
            is_smart_folder: query.smart_folder,
            cursor: query.cursor.as_deref(),
            limit: query.limit.unwrap_or(500),
        })
        .await?;
    Ok(Json(SavedSearchListResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn post_v1_saved_search(
    State(state): State<AppState>,
    Json(request): Json<NewCommunicationSavedSearch>,
) -> Result<Json<CommunicationSavedSearch>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let saved_search = CommunicationCommandService::new(pool)
        .create_saved_search(request)
        .await?;
    Ok(Json(saved_search))
}

pub(crate) async fn put_v1_saved_search(
    State(state): State<AppState>,
    Path(saved_search_id): Path<String>,
    Json(request): Json<UpdateCommunicationSavedSearch>,
) -> Result<Json<CommunicationSavedSearch>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(saved_search) = CommunicationCommandService::new(pool)
        .update_saved_search(&saved_search_id, request)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(saved_search))
}

pub(crate) async fn delete_v1_saved_search(
    State(state): State<AppState>,
    Path(saved_search_id): Path<String>,
) -> Result<Json<SavedSearchDeleteResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = CommunicationCommandService::new(pool)
        .delete_saved_search(&saved_search_id)
        .await?;
    Ok(Json(SavedSearchDeleteResponse { deleted }))
}

fn saved_search_store(state: &AppState) -> Result<CommunicationSavedSearchStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::app::api_support::app_store::<
        CommunicationSavedSearchStore,
    >(pool))
}
```

### `backend/src/app/handlers/communications/communication_queries/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/search.rs`
- Size bytes / Размер в байтах: `1946`
- Included characters / Включено символов: `1946`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Deserialize)]
pub(crate) struct EmailSearchQuery {
    pub(super) q: String,
    pub(super) limit: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct CommunicationSearchResponse {
    pub(super) results: Vec<SearchResultResponse>,
}

#[derive(Serialize)]
pub(crate) struct SearchResultResponse {
    pub(super) object_id: String,
    pub(super) object_kind: String,
    pub(super) title: String,
}

pub(crate) async fn get_v1_email_search(
    State(state): State<AppState>,
    Query(query): Query<EmailSearchQuery>,
) -> Result<Json<CommunicationSearchResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "search query is required",
        ));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<MessageProjectionStore>(pool.clone());

    let search_path: Option<String> = std::env::var("MAKOSH_SEARCH_INDEX_PATH").ok();
    if let Some(path) = search_path {
        let index =
            crate::engines::search::SearchIndex::open_or_create(std::path::Path::new(&path))?;
        let _ = crate::domains::communications::search::index_messages(&index, &store, 100).await;
        let results = crate::domains::communications::search::search_emails(
            &index,
            &query.q,
            query.limit.unwrap_or(20),
        )?;
        let items: Vec<SearchResultResponse> = results
            .into_iter()
            .map(|result| SearchResultResponse {
                object_id: result.object_id,
                object_kind: result.object_kind,
                title: result.title,
            })
            .collect();
        return Ok(Json(CommunicationSearchResponse { results: items }));
    }

    Ok(Json(CommunicationSearchResponse {
        results: Vec::new(),
    }))
}
```

### `backend/src/app/handlers/communications/communication_queries/threads.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/threads.rs`
- Size bytes / Размер в байтах: `1699`
- Included characters / Включено символов: `1699`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(crate) async fn get_v1_threads(
    State(state): State<AppState>,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<ThreadListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::threads::CommunicationThreadStore,
    >(pool);
    let page = store
        .list_threads_page(
            query.account_id.as_deref(),
            query.cursor.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(ThreadListResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn get_v1_thread_messages(
    State(state): State<AppState>,
    Query(query): Query<ThreadMessagesQuery>,
) -> Result<Json<ThreadMessagesResponse>, ApiError> {
    let account_id = query
        .account_id
        .as_deref()
        .ok_or(ApiError::InvalidCommunicationQuery(
            "account_id is required",
        ))?;
    let subject = query
        .subject
        .as_deref()
        .ok_or(ApiError::InvalidCommunicationQuery("subject is required"))?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::threads::CommunicationThreadStore,
    >(pool);
    let items = store
        .thread_messages(account_id, subject, query.limit.unwrap_or(50))
        .await?;

    Ok(Json(ThreadMessagesResponse { items }))
}
```

### `backend/src/app/handlers/communications/finance_analytics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/finance_analytics.rs`
- Size bytes / Размер в байтах: `165`
- Included characters / Включено символов: `165`
- Truncated / Обрезано: `no`

```rust
mod analytics;
mod explain;
mod invoices;
mod models;

pub(crate) use analytics::*;
pub(crate) use explain::*;
pub(crate) use invoices::*;
pub(crate) use models::*;
```

### `backend/src/app/handlers/communications/finance_analytics/analytics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/finance_analytics/analytics.rs`
- Size bytes / Размер в байтах: `1848`
- Included characters / Включено символов: `1848`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Deserialize)]
pub(crate) struct AnalyticsQuery {
    pub(super) account_id: Option<String>,
}

pub(crate) async fn get_v1_analytics_health(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<crate::domains::communications::analytics::MailboxHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::analytics::EmailAnalyticsStore,
    >(pool);
    let health = store.mailbox_health(query.account_id.as_deref()).await?;
    Ok(Json(health))
}

#[derive(Deserialize)]
pub(crate) struct SendersQuery {
    pub(super) account_id: Option<String>,
    pub(super) limit: Option<i64>,
    pub(super) cursor: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct SendersResponse {
    pub(super) items: Vec<crate::domains::communications::analytics::SenderStats>,
    pub(super) next_cursor: Option<String>,
    pub(super) has_more: bool,
}

pub(crate) async fn get_v1_analytics_senders(
    State(state): State<AppState>,
    Query(query): Query<SendersQuery>,
) -> Result<Json<SendersResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::analytics::EmailAnalyticsStore,
    >(pool);
    let page = store
        .top_senders_page(
            query.account_id.as_deref(),
            query.limit.unwrap_or(20),
            query.cursor.as_deref(),
        )
        .await?;
    Ok(Json(SendersResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}
```

### `backend/src/app/handlers/communications/finance_analytics/explain.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/finance_analytics/explain.rs`
- Size bytes / Размер в байтах: `1199`
- Included characters / Включено символов: `1199`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Serialize)]
pub(crate) struct MessageExplainResponse {
    pub(super) reasons: Vec<String>,
}

pub(crate) async fn get_v1_message_explain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<MessageExplainResponse>, ApiError> {
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let ctx = crate::domains::communications::explain::explain_importance(&message);
    Ok(Json(MessageExplainResponse {
        reasons: ctx.reasons,
    }))
}

#[derive(Serialize)]
pub(crate) struct SmartCcResponse {
    pub(super) suggestions: Vec<String>,
}

pub(crate) async fn get_v1_message_smart_cc(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<SmartCcResponse>, ApiError> {
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let suggestions = crate::domains::communications::explain::smart_cc_suggestions(&message);
    Ok(Json(SmartCcResponse { suggestions }))
}
```

### `backend/src/app/handlers/communications/finance_analytics/invoices.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/finance_analytics/invoices.rs`
- Size bytes / Размер в байтах: `2965`
- Included characters / Включено символов: `2965`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Deserialize)]
pub(crate) struct InvoiceListQuery {
    pub(super) status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct InvoiceListResponse {
    pub(super) items: Vec<crate::domains::communications::finance::InvoiceRecord>,
}

#[derive(Deserialize)]
pub(crate) struct NewInvoiceRequest {
    pub(super) invoice_id: String,
    pub(super) message_id: Option<String>,
    pub(super) amount: Option<f64>,
    pub(super) currency: Option<String>,
    pub(super) invoice_number: Option<String>,
    pub(super) issue_date: Option<DateTime<Utc>>,
    pub(super) due_date: Option<DateTime<Utc>>,
    pub(super) counterparty: Option<String>,
    pub(super) tax_id: Option<String>,
    pub(super) status: Option<String>,
    pub(super) linked_project_id: Option<String>,
    pub(super) linked_person_id: Option<String>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn get_v1_invoices(
    State(state): State<AppState>,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<InvoiceListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::finance::CommunicationFinanceStore,
    >(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::domains::communications::finance::InvoiceStatus::parse);
    let items = store.list(status).await?;
    Ok(Json(InvoiceListResponse { items }))
}

pub(crate) async fn post_v1_invoice(
    State(state): State<AppState>,
    Json(req): Json<NewInvoiceRequest>,
) -> Result<Json<crate::domains::communications::finance::InvoiceRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::finance::CommunicationFinanceStore,
    >(pool);
    let invoice = store
        .upsert_invoice(&crate::domains::communications::finance::NewInvoiceRecord {
            invoice_id: req.invoice_id,
            message_id: req.message_id,
            amount: req.amount,
            currency: req.currency,
            invoice_number: req.invoice_number,
            issue_date: req.issue_date,
            due_date: req.due_date,
            counterparty: req.counterparty,
            tax_id: req.tax_id,
            status: req
                .status
                .as_deref()
                .and_then(crate::domains::communications::finance::InvoiceStatus::parse)
                .unwrap_or(crate::domains::communications::finance::InvoiceStatus::Received),
            linked_project_id: req.linked_project_id,
            linked_person_id: req.linked_person_id,
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(invoice))
}
```

### `backend/src/app/handlers/communications/finance_analytics/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/finance_analytics/models.rs`
- Size bytes / Размер в байтах: `418`
- Included characters / Включено символов: `418`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Serialize)]
pub(crate) struct PinToggleResponse {
    pub(in crate::app::handlers::communications) message_id: String,
    pub(in crate::app::handlers::communications) pinned: bool,
}

#[derive(Serialize)]
pub(crate) struct ImportantToggleResponse {
    pub(in crate::app::handlers::communications) message_id: String,
    pub(in crate::app::handlers::communications) important: bool,
}
```

### `backend/src/app/handlers/communications/legal_export.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/legal_export.rs`
- Size bytes / Размер в байтах: `5381`
- Included characters / Включено символов: `5381`
- Truncated / Обрезано: `no`

```rust
use super::*;

pub(crate) async fn get_v1_legal_docs(
    State(state): State<AppState>,
    Query(query): Query<LegalDocQuery>,
) -> Result<Json<LegalDocListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::legal::LegalDocumentStore,
    >(pool);
    let dt = query
        .document_type
        .as_deref()
        .and_then(crate::domains::communications::legal::LegalDocType::parse);
    let st = query
        .status
        .as_deref()
        .and_then(crate::domains::communications::legal::LegalDocStatus::parse);
    let items = store.list(dt, st).await?;
    Ok(Json(LegalDocListResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewLegalDocRequest {
    pub(super) document_id: String,
    pub(super) message_id: Option<String>,
    pub(super) document_type: String,
    pub(super) title: String,
    pub(super) parties: Option<Vec<String>>,
    pub(super) effective_date: Option<DateTime<Utc>>,
    pub(super) expiry_date: Option<DateTime<Utc>>,
    pub(super) amount: Option<f64>,
    pub(super) currency: Option<String>,
    pub(super) status: Option<String>,
    pub(super) linked_project_id: Option<String>,
    pub(super) risks: Option<Vec<String>>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn post_v1_legal_doc(
    State(state): State<AppState>,
    Json(req): Json<NewLegalDocRequest>,
) -> Result<Json<crate::domains::communications::legal::LegalDocument>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::legal::LegalDocumentStore,
    >(pool);
    let doc = store
        .upsert(&crate::domains::communications::legal::NewLegalDocument {
            document_id: req.document_id,
            message_id: req.message_id,
            document_type: crate::domains::communications::legal::LegalDocType::parse(
                &req.document_type,
            )
            .unwrap_or(crate::domains::communications::legal::LegalDocType::Other),
            title: req.title,
            parties: req.parties.unwrap_or_default(),
            effective_date: req.effective_date,
            expiry_date: req.expiry_date,
            amount: req.amount,
            currency: req.currency,
            status: req
                .status
                .as_deref()
                .and_then(crate::domains::communications::legal::LegalDocStatus::parse)
                .unwrap_or(crate::domains::communications::legal::LegalDocStatus::Draft),
            linked_project_id: req.linked_project_id,
            risks: req.risks.unwrap_or_default(),
            metadata: req.metadata.unwrap_or(serde_json::json!({})),
        })
        .await?;
    Ok(Json(doc))
}

#[derive(Serialize)]
pub(crate) struct ExportResponse {
    pub(super) content_type: String,
    pub(super) content: String,
    pub(super) filename: String,
}

#[derive(Deserialize)]
pub(crate) struct MessageExportQuery {
    pub(super) format: Option<String>,
}

pub(crate) async fn get_v1_message_export(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<MessageExportQuery>,
) -> Result<Json<ExportResponse>, ApiError> {
    let msg_store = message_store(&state)?;
    let att_store = communication_storage_store(&state)?;
    let format = match query.format.as_deref().unwrap_or("markdown") {
        "eml" => crate::domains::communications::export::ExportFormat::Eml,
        "json" => crate::domains::communications::export::ExportFormat::Json,
        _ => crate::domains::communications::export::ExportFormat::Markdown,
    };
    let export = crate::domains::communications::export::export_message(
        &msg_store,
        &att_store,
        &message_id,
        format,
    )
    .await?;
    Ok(Json(ExportResponse {
        content_type: export.format.content_type().to_owned(),
        content: export.content,
        filename: format!(
            "message_{}.{}",
            &message_id[..8.min(message_id.len())],
            export.format.extension()
        ),
    }))
}

#[derive(Deserialize)]
pub(crate) struct SendRequest {
    pub(super) account_id: String,
    pub(super) to: Vec<String>,
    pub(super) cc: Option<Vec<String>>,
    pub(super) bcc: Option<Vec<String>>,
    pub(super) subject: String,
    pub(super) body_text: String,
    pub(super) body_html: Option<String>,
    pub(super) in_reply_to: Option<String>,
    pub(super) references: Option<Vec<String>>,
    pub(super) draft_id: Option<String>,
    pub(super) scheduled_send_at: Option<DateTime<Utc>>,
    pub(super) undo_send_seconds: Option<i64>,
    pub(super) confirmed_provider_write: Option<bool>,
}

#[derive(Serialize)]
pub(crate) struct SendResponse {
    pub(super) message_id: String,
    pub(super) outbox_id: Option<String>,
    pub(super) accepted: Vec<String>,
    pub(super) accepted_recipients: Vec<String>,
    pub(super) transport: String,
    pub(super) status: String,
    pub(super) scheduled_send_at: Option<DateTime<Utc>>,
    pub(super) undo_deadline_at: Option<DateTime<Utc>>,
    pub(super) failure_reason: Option<String>,
}
```

### `backend/src/app/handlers/communications/message_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/message_actions.rs`
- Size bytes / Размер в байтах: `8810`
- Included characters / Включено символов: `8810`
- Truncated / Обрезано: `no`

```rust
use super::*;
use crate::domains::communications::service::CommunicationCommandService;

#[derive(Deserialize)]
pub(crate) struct BulkMessageActionRequest {
    action: String,
    message_ids: Vec<String>,
    label: Option<String>,
    snooze_until: Option<String>,
}

pub(crate) async fn post_v1_messages_bulk_action(
    State(state): State<AppState>,
    Json(request): Json<BulkMessageActionRequest>,
) -> Result<Json<crate::domains::communications::bulk_actions::BulkMessageActionOutcome>, ApiError>
{
    let action = parse_bulk_message_action(&request)?;
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::bulk_actions::BulkMessageActionStore,
    >(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    );
    Ok(Json(store.apply(request.message_ids, action).await?))
}

fn parse_bulk_message_action(
    request: &BulkMessageActionRequest,
) -> Result<crate::domains::communications::bulk_actions::BulkMessageAction, ApiError> {
    let action = match request.action.trim() {
        "mark_read" => crate::domains::communications::bulk_actions::BulkMessageAction::MarkRead,
        "mark_unread" => {
            crate::domains::communications::bulk_actions::BulkMessageAction::MarkUnread
        }
        "archive" => crate::domains::communications::bulk_actions::BulkMessageAction::Archive,
        "trash" => crate::domains::communications::bulk_actions::BulkMessageAction::Trash,
        "restore" => crate::domains::communications::bulk_actions::BulkMessageAction::Restore,
        "pin" => crate::domains::communications::bulk_actions::BulkMessageAction::Pin,
        "unpin" => crate::domains::communications::bulk_actions::BulkMessageAction::Unpin,
        "important" => crate::domains::communications::bulk_actions::BulkMessageAction::Important,
        "not_important" => {
            crate::domains::communications::bulk_actions::BulkMessageAction::NotImportant
        }
        "add_label" => {
            let label = request
                .label
                .as_deref()
                .ok_or(ApiError::InvalidCommunicationQuery(
                    "label is required for add_label",
                ))?;
            crate::domains::communications::bulk_actions::BulkMessageAction::AddLabel(
                label.to_owned(),
            )
        }
        "remove_label" => {
            let label = request
                .label
                .as_deref()
                .ok_or(ApiError::InvalidCommunicationQuery(
                    "label is required for remove_label",
                ))?;
            crate::domains::communications::bulk_actions::BulkMessageAction::RemoveLabel(
                label.to_owned(),
            )
        }
        "snooze" => {
            let until = request
                .snooze_until
                .as_deref()
                .ok_or(ApiError::InvalidCommunicationQuery(
                    "snooze_until is required for snooze",
                ))?
                .parse()
                .map_err(|_| ApiError::InvalidCommunicationQuery("invalid snooze_until"))?;
            crate::domains::communications::bulk_actions::BulkMessageAction::Snooze(until)
        }
        _ => {
            return Err(ApiError::InvalidCommunicationQuery(
                "invalid bulk message action",
            ));
        }
    };

    Ok(action)
}

pub(crate) async fn post_v1_message_pin(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pinned = CommunicationCommandService::new(pool)
        .toggle_message_pin(&message_id)
        .await?;
    Ok(Json(PinToggleResponse { message_id, pinned }))
}

pub(crate) async fn post_v1_message_important(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<ImportantToggleResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let important = CommunicationCommandService::new(pool)
        .toggle_message_important(&message_id)
        .await?;
    Ok(Json(ImportantToggleResponse {
        message_id,
        important,
    }))
}

#[derive(Deserialize)]
pub(crate) struct SnoozeRequest {
    pub(super) until: String,
}

pub(crate) async fn post_v1_message_snooze(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<SnoozeRequest>,
) -> Result<Json<Value>, ApiError> {
    let until: DateTime<Utc> = req
        .until
        .parse()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid datetime"))?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationCommandService::new(pool)
        .snooze_message(&message_id, until)
        .await?;
    Ok(Json(serde_json::json!({"snoozed": true})))
}

pub(crate) async fn post_v1_message_mute(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<PinToggleResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let muted = CommunicationCommandService::new(pool)
        .toggle_message_mute(&message_id)
        .await?;
    Ok(Json(PinToggleResponse {
        message_id,
        pinned: muted,
    }))
}

#[derive(Deserialize)]
pub(crate) struct LabelRequest {
    pub(super) label: String,
}

pub(crate) async fn post_v1_message_label(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationCommandService::new(pool)
        .add_message_label(&message_id, &req.label)
        .await?;
    Ok(Json(serde_json::json!({"labeled": true})))
}

pub(crate) async fn delete_v1_message_label(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationCommandService::new(pool)
        .remove_message_label(&message_id, &req.label)
        .await?;
    Ok(Json(serde_json::json!({"removed": true})))
}

#[derive(Deserialize)]
pub(crate) struct SubscriptionsQuery {
    pub(super) account_id: Option<String>,
    pub(super) limit: Option<i64>,
    pub(super) cursor: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct SubscriptionsResponse {
    pub(super) items: Vec<crate::domains::communications::subscriptions::SubscriptionSource>,
    pub(super) next_cursor: Option<String>,
    pub(super) has_more: bool,
}

pub(crate) async fn get_v1_subscriptions(
    State(state): State<AppState>,
    Query(query): Query<SubscriptionsQuery>,
) -> Result<Json<SubscriptionsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::subscriptions::SubscriptionStore,
    >(pool);
    let page = store
        .detect_subscriptions_page(
            query.account_id.as_deref(),
            query.limit.unwrap_or(50),
            query.cursor.as_deref(),
        )
        .await?;
    Ok(Json(SubscriptionsResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

#[derive(Deserialize)]
pub(crate) struct DupQuery {
    pub(super) limit: Option<i64>,
}

pub(crate) async fn get_v1_attachment_duplicates(
    State(state): State<AppState>,
    Query(query): Query<DupQuery>,
) -> Result<Json<Vec<crate::domains::communications::attachment_dedup::DuplicateGroup>>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::attachment_dedup::AttachmentDedupStore,
    >(pool);
    let dups = store.find_duplicates(query.limit.unwrap_or(20)).await?;
    Ok(Json(dups))
}

#[derive(Deserialize)]
pub(crate) struct LegalDocQuery {
    pub(super) document_type: Option<String>,
    pub(super) status: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LegalDocListResponse {
    pub(super) items: Vec<crate::domains::communications::legal::LegalDocument>,
}
```

### `backend/src/app/handlers/communications/message_ai_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/message_ai_state.rs`
- Size bytes / Размер в байтах: `1380`
- Included characters / Включено символов: `1380`
- Truncated / Обрезано: `no`

```rust
use super::*;
use crate::domains::communications::ai_state::{
    CommunicationAiStateRecord, CommunicationAiStateStore, CommunicationAiStateTransitionRequest,
};
use crate::domains::communications::service::CommunicationCommandService;

pub(crate) async fn get_v1_message_ai_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<CommunicationAiStateRecord>, ApiError> {
    let Some(record) = ai_state_store(&state)?.current(&message_id).await? else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(record))
}

pub(crate) async fn put_v1_message_ai_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<CommunicationAiStateTransitionRequest>,
) -> Result<Json<CommunicationAiStateRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let record = CommunicationCommandService::new(pool)
        .transition_message_ai_state(&message_id, request)
        .await?;
    Ok(Json(record))
}

fn ai_state_store(state: &AppState) -> Result<CommunicationAiStateStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::app::api_support::app_store::<
        CommunicationAiStateStore,
    >(pool))
}
```

### `backend/src/app/handlers/communications/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/mod.rs`
- Size bytes / Размер в байтах: `8401`
- Included characters / Включено символов: `8401`
- Truncated / Обрезано: `no`

```rust
// ADR-0073: mail handlers are grouped by bounded context for the first
// handlers.rs extraction; split by communications, accounts and workflow next.
mod account_management;
mod account_setup;
mod account_support;
mod communication_messages;
mod communication_queries;
mod finance_analytics;
mod legal_export;
mod message_actions;
mod message_ai_state;
mod remote_images;
mod sending;
mod templates_status;
mod workflow_actions;
mod workflow_state;
pub(crate) use account_management::*;
pub(crate) use account_setup::*;
use account_support::*;
pub(crate) use communication_messages::*;
pub(crate) use communication_queries::*;
pub(crate) use finance_analytics::*;
pub(crate) use legal_export::*;
pub(crate) use message_actions::*;
pub(crate) use message_ai_state::*;
pub(crate) use remote_images::get_v1_communication_message_remote_image;
pub(crate) use sending::*;
pub(crate) use templates_status::*;
pub(crate) use workflow_actions::{
    WorkflowActionInput, WorkflowActionKind, WorkflowActionProvenance, WorkflowActionRequest,
    WorkflowActionResponse, WorkflowActionSource, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind, execute_workflow_action, post_v1_workflow_action,
};
pub(crate) use workflow_state::*;

use std::collections::HashMap;
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
    ProviderAccountSecretPurpose, ProviderCredentialReader,
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

use crate::application::email_intelligence::{
    EmailIntelligenceError, EmailIntelligenceService, EmailSummaryContract,
};
use crate::application::mail_background_sync::{
    DEFAULT_MAIL_SYNC_BLOB_ROOT, MailBackgroundSyncService, MailSyncError, MailSyncRunResponse,
    MailSyncSettings, MailSyncSettingsUpdate, MailSyncStatus, MailSyncStore, MailSyncTrigger,
};
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
    LocalMessageState, MessageProjectionError, MessageProjectionStore, ProjectedMessage,
    ProjectedMessageSummary, WorkflowState, parse_raw_email_message_from_blob,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, LocalCommunicationBlobStore,
    StoredCommunicationAttachmentWithBlob,
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
use crate::vault::{EntropyEvent, HostVaultError, VaultMode};

use crate::app::api_support::*;
use crate::app::{ApiError, AppState};
```

### `backend/src/app/handlers/communications/remote_images.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/remote_images.rs`
- Size bytes / Размер в байтах: `146`
- Included characters / Включено символов: `146`
- Truncated / Обрезано: `no`

```rust
mod dns;
mod errors;
mod fetcher;
mod handler;
mod reference;
mod url_policy;

pub(crate) use handler::get_v1_communication_message_remote_image;
```

### `backend/src/app/handlers/communications/remote_images/dns.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/remote_images/dns.rs`
- Size bytes / Размер в байтах: `1356`
- Included characters / Включено символов: `1356`
- Truncated / Обрезано: `no`

```rust
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use serde::Deserialize;

use super::errors::RemoteImageFetchError;
use super::url_policy::is_public_ip;

#[derive(Deserialize)]
struct GoogleDnsResponse {
    #[serde(rename = "Answer")]
    answer: Option<Vec<GoogleDnsAnswer>>,
}

#[derive(Deserialize)]
struct GoogleDnsAnswer {
    #[serde(rename = "type")]
    record_type: u16,
    data: String,
}

pub(super) async fn resolve_public_image_addrs(
    host: &str,
    port: u16,
) -> Result<Vec<SocketAddr>, RemoteImageFetchError> {
    let resolver = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let response = resolver
        .get("https://dns.google/resolve")
        .query(&[("name", host), ("type", "A")])
        .send()
        .await?
        .error_for_status()?;
    let dns = response.json::<GoogleDnsResponse>().await?;
    let addrs = dns
        .answer
        .unwrap_or_default()
        .into_iter()
        .filter(|answer| answer.record_type == 1)
        .filter_map(|answer| answer.data.parse::<Ipv4Addr>().ok())
        .map(IpAddr::V4)
        .filter(|ip| is_public_ip(*ip))
        .map(|ip| SocketAddr::new(ip, port))
        .collect::<Vec<_>>();
    if addrs.is_empty() {
        return Err(RemoteImageFetchError::NoPublicAddress);
    }
    Ok(addrs)
}
```

### `backend/src/app/handlers/communications/remote_images/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/remote_images/errors.rs`
- Size bytes / Размер в байтах: `1243`
- Included characters / Включено символов: `1243`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::app::ApiError;

#[derive(Debug, Error)]
pub(super) enum RemoteImageFetchError {
    #[error("remote image host is unavailable")]
    MissingHost,
    #[error("remote image has no public DNS address")]
    NoPublicAddress,
    #[error("remote image client failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("remote image returned non-success status")]
    NonSuccessStatus,
    #[error("remote image content type is not image")]
    NotImage,
    #[error("remote image exceeds size limit")]
    TooLarge,
    #[error("remote image response header is invalid")]
    InvalidHeader,
}

pub(super) fn remote_image_fetch_api_error(error: RemoteImageFetchError) -> ApiError {
    match error {
        RemoteImageFetchError::TooLarge => {
            ApiError::InvalidCommunicationQuery("remote image exceeds size limit")
        }
        RemoteImageFetchError::NotImage => {
            ApiError::InvalidCommunicationQuery("remote asset is not an image")
        }
        RemoteImageFetchError::NoPublicAddress => {
            ApiError::InvalidCommunicationQuery("remote image host has no public address")
        }
        _ => ApiError::InvalidCommunicationQuery("remote image unavailable"),
    }
}
```

### `backend/src/app/handlers/communications/remote_images/fetcher.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/remote_images/fetcher.rs`
- Size bytes / Размер в байтах: `2879`
- Included characters / Включено символов: `2879`
- Truncated / Обрезано: `no`

```rust
use std::net::SocketAddr;
use std::time::Duration;

use axum::http::{HeaderValue, header};
use reqwest::Url;

use super::dns::resolve_public_image_addrs;
use super::errors::RemoteImageFetchError;

const MAX_REMOTE_IMAGE_BYTES: u64 = 12 * 1024 * 1024;
const REMOTE_IMAGE_TIMEOUT: Duration = Duration::from_secs(15);

pub(super) struct RemoteImage {
    pub(super) content_type: HeaderValue,
    pub(super) body: Vec<u8>,
}

pub(super) async fn fetch_remote_image(url: &Url) -> Result<RemoteImage, RemoteImageFetchError> {
    let default_client = remote_image_client(None)?;
    match fetch_remote_image_with_client(&default_client, url).await {
        Ok(image) => Ok(image),
        Err(first_error @ RemoteImageFetchError::Http(_)) => {
            let Some(host) = url.host_str() else {
                return Err(RemoteImageFetchError::MissingHost);
            };
            let port = url.port_or_known_default().unwrap_or(443);
            let public_addrs = resolve_public_image_addrs(host, port).await?;
            if public_addrs.is_empty() {
                return Err(first_error);
            }
            let fallback_client = remote_image_client(Some((host, public_addrs.as_slice())))?;
            fetch_remote_image_with_client(&fallback_client, url).await
        }
        Err(error) => Err(error),
    }
}

fn remote_image_client(
    dns_override: Option<(&str, &[SocketAddr])>,
) -> Result<reqwest::Client, RemoteImageFetchError> {
    let mut builder = reqwest::Client::builder()
        .timeout(REMOTE_IMAGE_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(4))
        .user_agent("МакошьHub-MailImageProxy/0.1");
    if let Some((host, addrs)) = dns_override {
        builder = builder.resolve_to_addrs(host, addrs);
    }
    Ok(builder.build()?)
}

async fn fetch_remote_image_with_client(
    client: &reqwest::Client,
    url: &Url,
) -> Result<RemoteImage, RemoteImageFetchError> {
    let response = client.get(url.clone()).send().await?;
    if !response.status().is_success() {
        return Err(RemoteImageFetchError::NonSuccessStatus);
    }
    if response.content_length().unwrap_or(0) > MAX_REMOTE_IMAGE_BYTES {
        return Err(RemoteImageFetchError::TooLarge);
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .ok_or(RemoteImageFetchError::InvalidHeader)?;
    let content_type_text = content_type
        .to_str()
        .map_err(|_| RemoteImageFetchError::InvalidHeader)?
        .to_ascii_lowercase();
    if !content_type_text.starts_with("image/") {
        return Err(RemoteImageFetchError::NotImage);
    }
    let body = response.bytes().await?;
    if body.len() as u64 > MAX_REMOTE_IMAGE_BYTES {
        return Err(RemoteImageFetchError::TooLarge);
    }
    Ok(RemoteImage {
        content_type,
        body: body.to_vec(),
    })
}
```

### `backend/src/app/handlers/communications/remote_images/handler.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/remote_images/handler.rs`
- Size bytes / Размер в байтах: `1899`
- Included characters / Включено символов: `1899`
- Truncated / Обрезано: `no`

```rust
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use serde::Deserialize;

use super::errors::remote_image_fetch_api_error;
use super::fetcher::fetch_remote_image;
use super::reference::message_html_references_url;
use super::url_policy::parse_remote_image_url;
use crate::app::api_support::message_store;
use crate::app::handlers::communications::communication_messages::rich_body_html_for_message;
use crate::app::{ApiError, AppState};

#[derive(Deserialize)]
pub(crate) struct RemoteImageQuery {
    url: String,
}

pub(crate) async fn get_v1_communication_message_remote_image(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<RemoteImageQuery>,
) -> Result<Response, ApiError> {
    let image_url = parse_remote_image_url(&query.url)?;
    let Some(message) = message_store(&state)?.message(&message_id).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let Some(body_html) = rich_body_html_for_message(&state, &message).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    if !message_html_references_url(&body_html, image_url.as_str()) {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image is not referenced by this message",
        ));
    }

    let image = fetch_remote_image(&image_url)
        .await
        .map_err(remote_image_fetch_api_error)?;

    let mut response = Response::new(Body::from(image.body));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, image.content_type);
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=600"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    Ok(response)
}
```

### `backend/src/app/handlers/communications/remote_images/reference.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/remote_images/reference.rs`
- Size bytes / Размер в байтах: `699`
- Included characters / Включено символов: `699`
- Truncated / Обрезано: `no`

```rust
pub(super) fn message_html_references_url(body_html: &str, image_url: &str) -> bool {
    body_html.contains(image_url)
        || body_html.contains(&image_url.replace('&', "&amp;"))
        || body_html.replace("&amp;", "&").contains(image_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_escaped_message_image_references() {
        let html = r#"<img src="https://img.example.test/a.png?x=1&amp;y=2">"#;
        assert!(message_html_references_url(
            html,
            "https://img.example.test/a.png?x=1&y=2"
        ));
        assert!(!message_html_references_url(
            html,
            "https://img.example.test/other.png"
        ));
    }
}
```

### `backend/src/app/handlers/communications/remote_images/url_policy.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/remote_images/url_policy.rs`
- Size bytes / Размер в байтах: `3022`
- Included characters / Включено символов: `3022`
- Truncated / Обрезано: `no`

```rust
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use reqwest::Url;

use crate::app::ApiError;

const MAX_REMOTE_IMAGE_URL_BYTES: usize = 4096;

pub(super) fn parse_remote_image_url(value: &str) -> Result<Url, ApiError> {
    if value.len() > MAX_REMOTE_IMAGE_URL_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image URL is too long",
        ));
    }
    let url = Url::parse(value)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid remote image URL"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image URL scheme is not allowed",
        ));
    }
    let Some(host) = url.host_str() else {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image URL host is required",
        ));
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image URL host is not allowed",
        ));
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        ensure_public_ip(ip)?;
    }
    Ok(url)
}

fn ensure_public_ip(ip: IpAddr) -> Result<(), ApiError> {
    if is_public_ip(ip) {
        return Ok(());
    }
    Err(ApiError::InvalidCommunicationQuery(
        "remote image URL address is not allowed",
    ))
}

pub(super) fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_ipv4(ip),
        IpAddr::V6(ip) => is_public_ipv6(ip),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    !(ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.octets()[0] == 0
        || ip.octets()[0] >= 224
        || (ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1])))
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    !(ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_unique_local()
        || ((ip.segments()[0] & 0xffc0) == 0xfe80)
        || ((ip.segments()[0] & 0xff00) == 0xff00))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_and_local_image_hosts() {
        assert!(parse_remote_image_url("https://127.0.0.1/a.png").is_err());
        assert!(parse_remote_image_url("https://192.168.1.20/a.png").is_err());
        assert!(parse_remote_image_url("https://localhost/a.png").is_err());
        assert!(parse_remote_image_url("cid:part-1").is_err());
        assert!(parse_remote_image_url("https://image.email.feverup.com/a.png").is_ok());
    }

    #[test]
    fn classifies_public_ips_for_ssrf_guard() {
        assert!(is_public_ip(IpAddr::V4(Ipv4Addr::new(13, 224, 83, 8))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(!is_public_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }
}
```

### `backend/src/app/handlers/communications/sending.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending.rs`
- Size bytes / Размер в байтах: `930`
- Included characters / Включено символов: `930`
- Truncated / Обрезано: `no`

```rust
mod ai_reply;
mod bilingual_reply_flow;
mod certificates;
mod extraction;
mod forwarding;
mod local_state;
mod multilingual;
mod provider_send;

pub(crate) use ai_reply::{post_v1_ai_reply, post_v1_ai_reply_variants};
pub(crate) use bilingual_reply_flow::post_v1_bilingual_reply_flow;
pub(crate) use certificates::{
    get_v1_certs, get_v1_certs_expiring, get_v1_signature_check, get_v1_spf_dkim, post_v1_cert,
};
pub(crate) use extraction::{post_v1_extract_notes, post_v1_extract_tasks};
pub(crate) use forwarding::{
    post_v1_forward, post_v1_forward_eml, post_v1_redirect, post_v1_reply, post_v1_reply_all,
};
pub(crate) use local_state::{
    post_v1_imap_delete, post_v1_imap_mark_read, post_v1_message_restore, post_v1_message_trash,
};
pub(crate) use multilingual::{
    get_v1_detect_language, post_v1_translate, post_v1_translate_attachment,
    post_v1_translate_thread,
};
pub(crate) use provider_send::post_v1_send;
```

### `backend/src/app/handlers/communications/sending/ai_reply.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending/ai_reply.rs`
- Size bytes / Размер в байтах: `3926`
- Included characters / Включено символов: `3926`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Deserialize)]
pub(crate) struct AiReplyRequest {
    pub(super) tone: Option<String>,
    pub(super) language: Option<String>,
    pub(super) context: Option<String>,
}

pub(crate) async fn post_v1_ai_reply(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state).await?;
    let opts = crate::domains::communications::ai_reply::AiReplyOptions {
        tone: req.tone,
        language: req.language,
        context: req.context,
    };
    match service.generate_reply(&msg, &opts).await? {
        Some(draft) => {
            if let Some(pool) = state.database.pool() {
                crate::domains::signal_hub::dispatch_ai_helper_signal(
                    pool.clone(),
                    "reply_drafting",
                    &message_id,
                    serde_json::json!({
                        "kind": "communication_message",
                        "source_code": "ai",
                        "message_id": message_id,
                        "operation": "reply_drafting",
                    }),
                    serde_json::json!({
                        "tone": draft.tone,
                        "language": draft.language,
                    }),
                    serde_json::json!({
                        "source": "communication_message_ai_reply",
                        "message_id": message_id,
                    }),
                    None,
                )
                .await?;
            }

            Ok(Json(
                serde_json::json!({"subject": draft.subject, "body": draft.body, "tone": draft.tone, "language": draft.language}),
            ))
        }
        None => Ok(Json(
            serde_json::json!({"generated": false, "reason": "no LLM configured"}),
        )),
    }
}

#[derive(Deserialize)]
pub(crate) struct AiReplyVariantsRequest {
    pub(super) languages: Option<Vec<String>>,
    pub(super) tones: Option<Vec<String>>,
}

pub(crate) async fn post_v1_ai_reply_variants(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<AiReplyVariantsRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_ai_reply_service(&state).await?;
    let languages = req
        .languages
        .unwrap_or_else(|| vec!["en".into(), "es".into(), "ru".into()]);
    let tones = req
        .tones
        .unwrap_or_else(|| vec!["professional".into(), "friendly".into()]);
    let variants = service
        .generate_reply_variants(&msg, &languages, &tones)
        .await?;
    if !variants.is_empty()
        && let Some(pool) = state.database.pool()
    {
        crate::domains::signal_hub::dispatch_ai_helper_signal(
            pool.clone(),
            "reply_variant_generation",
            &message_id,
            serde_json::json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "reply_variant_generation",
            }),
            serde_json::json!({
                "variant_count": variants.len(),
                "language_count": languages.len(),
                "tone_count": tones.len(),
            }),
            serde_json::json!({
                "source": "communication_message_ai_reply_variants",
                "message_id": message_id,
            }),
            None,
        )
        .await?;
    }
    Ok(Json(serde_json::json!({"variants": variants})))
}
```
