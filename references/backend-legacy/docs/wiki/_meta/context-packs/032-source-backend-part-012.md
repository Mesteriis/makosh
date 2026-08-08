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

- Chunk ID / ID чанка: `032-source-backend-part-012`
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

### `backend/src/app/handlers/tasks/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/tasks/support.rs`
- Size bytes / Размер в байтах: `256`
- Included characters / Включено символов: `256`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::app::{ApiError, AppState};

pub(super) fn database_pool(state: &AppState) -> Result<PgPool, ApiError> {
    state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)
        .cloned()
}
```

### `backend/src/app/handlers/telegram.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/telegram.rs`
- Size bytes / Размер в байтах: `67`
- Included characters / Включено символов: `67`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::app::provider_runtime_handlers::telegram::*;
```

### `backend/src/app/handlers/whatsapp.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/whatsapp.rs`
- Size bytes / Размер в байтах: `67`
- Included characters / Включено символов: `67`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::app::provider_runtime_handlers::whatsapp::*;
```

### `backend/src/app/handlers/yandex_telemost.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/yandex_telemost.rs`
- Size bytes / Размер в байтах: `74`
- Included characters / Включено символов: `74`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::app::provider_runtime_handlers::yandex_telemost::*;
```

### `backend/src/app/handlers/zoom.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/zoom.rs`
- Size bytes / Размер в байтах: `63`
- Included characters / Включено символов: `63`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::app::provider_runtime_handlers::zoom::*;
```

### `backend/src/app/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/mod.rs`
- Size bytes / Размер в байтах: `460`
- Included characters / Включено символов: `460`
- Truncated / Обрезано: `no`

```rust
pub(crate) mod api_support;
pub(crate) mod connectrpc;
pub(crate) mod error;
pub(crate) mod guard;
pub(crate) mod handlers;
pub(crate) mod provider_runtime_handlers;
pub(crate) mod router;
pub(crate) mod signal_hub_support;
pub(crate) mod state;
pub(crate) mod vault_reconciliation;

pub(crate) use error::{ApiError, AppError};
pub use router::{build_router, build_router_with_database, init_tracing, run};
pub(crate) use state::{AccountSetupState, AppState};
```

### `backend/src/app/provider_runtime_handlers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers.rs`
- Size bytes / Размер в байтах: `103`
- Included characters / Включено символов: `103`
- Truncated / Обрезано: `no`

```rust
pub(crate) mod telegram;
pub(crate) mod whatsapp;
pub(crate) mod yandex_telemost;
pub(crate) mod zoom;
```

### `backend/src/app/provider_runtime_handlers/telegram.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram.rs`
- Size bytes / Размер в байтах: `2972`
- Included characters / Включено символов: `2972`
- Truncated / Обрезано: `no`

```rust
mod accounts;
mod capabilities;
mod chat_actions;
mod chat_folder_actions;
mod chats;
mod commands;
mod helpers;
mod media;
mod messages;
mod outbox;
mod qr_login;
mod raw;
mod runtime;
mod search;
mod topics;

pub(crate) use accounts::{
    delete_telegram_account, get_telegram_accounts, post_telegram_account,
    post_telegram_account_logout, post_telegram_fixture_account,
};
pub(crate) use capabilities::{get_telegram_account_capabilities, get_telegram_capabilities};
pub(crate) use chat_actions::{
    post_telegram_chat_archive, post_telegram_chat_join, post_telegram_chat_leave,
    post_telegram_chat_mark_read, post_telegram_chat_mark_unread, post_telegram_chat_mute,
    post_telegram_chat_pin, post_telegram_chat_unarchive, post_telegram_chat_unmute,
    post_telegram_chat_unpin,
};
pub(crate) use chat_folder_actions::{
    post_telegram_chat_add_folder, post_telegram_chat_reassign_folders,
    post_telegram_chat_remove_folder,
};
pub(crate) use chats::{
    get_telegram_chat_detail, get_telegram_chat_members, get_telegram_chats, get_telegram_folders,
    post_telegram_chat_members_sync, post_telegram_sync_chats, post_telegram_sync_history,
};
pub(crate) use commands::get_telegram_commands;
pub(crate) use media::{post_telegram_media_download, post_telegram_media_upload};
pub(crate) use messages::{
    delete_telegram_reaction, get_telegram_forward_chain, get_telegram_message_tombstones,
    get_telegram_message_versions, get_telegram_reactions, get_telegram_reply_chain,
    post_communication_conversation_archive, post_communication_conversation_mark_read,
    post_communication_conversation_mark_unread, post_communication_conversation_message,
    post_communication_conversation_mute, post_communication_conversation_pin,
    post_communication_conversation_unarchive, post_communication_conversation_unmute,
    post_communication_conversation_unpin, post_telegram_fixture_message,
    post_telegram_manual_send, post_telegram_message_delete, post_telegram_message_edit,
    post_telegram_message_forward, post_telegram_message_mark_read, post_telegram_message_pin,
    post_telegram_message_reply, post_telegram_message_restore_visibility, post_telegram_reaction,
};
pub(crate) use outbox::post_telegram_command_retry;
pub(crate) use qr_login::{
    delete_telegram_qr_login, get_telegram_qr_login_status, post_telegram_qr_login_password,
    post_telegram_qr_login_start,
};
pub(crate) use raw::get_telegram_message_raw;
pub(crate) use runtime::{
    get_telegram_runtime_status, post_telegram_runtime_restart, post_telegram_runtime_start,
    post_telegram_runtime_stop,
};
pub(crate) use search::{
    get_telegram_pinned_messages, post_telegram_provider_search, search_telegram_chats,
    search_telegram_media, search_telegram_messages,
};
pub(crate) use topics::{
    get_telegram_topic_detail, get_telegram_topic_messages, get_telegram_topics,
    post_telegram_topic_close, post_telegram_topic_create, search_telegram_topics,
};
```

### `backend/src/app/provider_runtime_handlers/telegram/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/accounts.rs`
- Size bytes / Размер в байтах: `4649`
- Included characters / Включено символов: `4649`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use super::helpers::{AUDIT_ACTOR_ID, telegram_api_hash_from_config, telegram_secret_store};
use crate::app::api_support::{
    api_audit_log, ensure_fixture_routes_enabled, telegram_provider_runtime_service,
};
use crate::app::signal_hub_support::{
    provider_account_or_not_found, remove_provider_account_signal_connection,
    sync_provider_account_signal_connection, sync_provider_account_signal_connection_with_status,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramAccountLifecycleResponse, TelegramAccountListResponse, TelegramAccountSetupRequest,
    TelegramAccountSetupResponse, TelegramLiveAccountSetupRequest, TelegramSecretVault,
};
use crate::platform::audit::NewApiAuditRecord;

pub(crate) async fn post_telegram_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<TelegramAccountSetupRequest>,
) -> Result<Json<TelegramAccountSetupResponse>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let response = telegram_provider_runtime_service(&state)?
        .setup_fixture_account(&request)
        .await?;
    let account = provider_account_or_not_found(&state, &response.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, None).await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_account(
    State(state): State<AppState>,
    Json(request): Json<TelegramLiveAccountSetupRequest>,
) -> Result<Json<TelegramAccountSetupResponse>, ApiError> {
    let request = request
        .with_inferred_qr_authorization()
        .with_app_credentials(
            state.config.telegram_api_id(),
            telegram_api_hash_from_config(&state.config),
        );

    let response = telegram_provider_runtime_service(&state)?
        .setup_live_blocked_account(
            &telegram_secret_store(&state)?,
            &TelegramSecretVault::host(state.vault.clone()),
            &request,
        )
        .await?;
    let account = provider_account_or_not_found(&state, &response.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, None).await?;
    Ok(Json(response))
}

#[derive(Deserialize)]
pub(crate) struct TelegramAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

pub(crate) async fn get_telegram_accounts(
    State(state): State<AppState>,
    Query(query): Query<TelegramAccountsQuery>,
) -> Result<Json<TelegramAccountListResponse>, ApiError> {
    let items = telegram_provider_runtime_service(&state)?
        .list_accounts(query.include_removed)
        .await?;

    Ok(Json(TelegramAccountListResponse { items }))
}

pub(crate) async fn post_telegram_account_logout(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<TelegramAccountLifecycleResponse>, ApiError> {
    let account = telegram_provider_runtime_service(&state)?
        .logout_account(&account_id)
        .await?;
    let provider_account = provider_account_or_not_found(&state, &account.account_id).await?;
    sync_provider_account_signal_connection_with_status(
        &state,
        &provider_account,
        "disconnected",
        None,
    )
    .await?;
    let stopped_runtime_actor = state.telegram_runtime.stop_account(&account.account_id)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_account_logout(
            AUDIT_ACTOR_ID,
            &account.account_id,
            &account.provider_kind,
            &account.lifecycle_state,
        ))
        .await?;

    Ok(Json(TelegramAccountLifecycleResponse {
        account,
        stopped_runtime_actor,
    }))
}

pub(crate) async fn delete_telegram_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<TelegramAccountLifecycleResponse>, ApiError> {
    let account = telegram_provider_runtime_service(&state)?
        .remove_account(&account_id)
        .await?;
    let provider_account = provider_account_or_not_found(&state, &account.account_id).await?;
    remove_provider_account_signal_connection(&state, &provider_account).await?;
    let stopped_runtime_actor = state.telegram_runtime.stop_account(&account.account_id)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_account_remove(
            AUDIT_ACTOR_ID,
            &account.account_id,
            &account.provider_kind,
            &account.lifecycle_state,
        ))
        .await?;

    Ok(Json(TelegramAccountLifecycleResponse {
        account,
        stopped_runtime_actor,
    }))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/capabilities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/capabilities.rs`
- Size bytes / Размер в байтах: `822`
- Included characters / Включено символов: `822`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};

use crate::app::api_support::{TelegramCapabilitiesResponse, telegram_provider_runtime_service};
use crate::app::{ApiError, AppState};

pub(crate) async fn get_telegram_capabilities(
    State(state): State<AppState>,
) -> Result<Json<TelegramCapabilitiesResponse>, ApiError> {
    Ok(Json(TelegramCapabilitiesResponse::current(&state.config)))
}

pub(crate) async fn get_telegram_account_capabilities(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<TelegramCapabilitiesResponse>, ApiError> {
    let account = telegram_provider_runtime_service(&state)?
        .telegram_account_record(&account_id)
        .await?;
    Ok(Json(TelegramCapabilitiesResponse::current_for_account(
        &state.config,
        &account,
    )))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/chat_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/chat_actions.rs`
- Size bytes / Размер в байтах: `17948`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use serde_json::json;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
};
use crate::app::api_support::{api_audit_log, telegram_provider_runtime_service};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{TelegramChat, lifecycle};
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

fn build_event(
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": subject_id, "kind": "telegram_sync"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

#[allow(clippy::too_many_arguments)]
fn build_command_event(
    account_id: &str,
    command_id: &str,
    provider_chat_id: &str,
    telegram_chat_id: Option<&str>,
    provider_message_id: Option<&str>,
    action: &str,
    status: &str,
    chat: Option<&TelegramChat>,
) -> NewEventEnvelope {
    build_event(
        telegram_event_types::COMMAND_STATUS_CHANGED,
        account_id,
        command_id,
        json!({
            "command_id": command_id,
            "action": action,
            "provider_chat_id": provider_chat_id,
            "telegram_chat_id": telegram_chat_id,
            "message_id": provider_message_id,
            "status": status,
            "chat": chat,
        }),
    )
}

fn build_chat_flag_event(
    event_type: &str,
    request: &TelegramChatActionRequest,
    telegram_chat_id: &str,
    flag_key: &str,
    flag_value: bool,
    chat: Option<&TelegramChat>,
) -> NewEventEnvelope {
    build_event(
        event_type,
        &request.account_id,
        telegram_chat_id,
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "telegram_chat_id": telegram_chat_id,
            flag_key: flag_value,
            "chat": chat,
        }),
    )
}

fn build_chat_updated_event(
    request: &TelegramChatActionRequest,
    telegram_chat_id: &str,
    action: &str,
    chat: Option<&TelegramChat>,
) -> NewEventEnvelope {
    build_event(
        telegram_event_types::CHAT_UPDATED,
        &request.account_id,
        telegram_chat_id,
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "telegram_chat_id": telegram_chat_id,
            "action": action,
            "chat": chat,
        }),
    )
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatActionRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    #[serde(default)]
    pub(crate) last_read_inbox_provider_message_id: Option<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatActionResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) metadata: serde_json::Value,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatLifecycleCommandResponse {
    pub(crate) telegram_chat_id: Option<String>,
    pub(crate) provider_chat_id: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) command_id: String,
}

async fn record_dialog_command(
    state: &AppState,
    telegram_chat_id: &str,
    request: &TelegramChatActionRequest,
    command_kind: &str,
    action_class: &str,
) -> Result<String, ApiError> {
    record_chat_lifecycle_command(
        state,
        Some(telegram_chat_id),
        request,
        command_kind,
        action_class,
        if command_kind == "mark_read" {
            request.last_read_inbox_provider_message_id.as_deref()
        } else {
            None
        },
    )
    .await
}

async fn record_chat_lifecycle_command(
    state: &AppState,
    telegram_chat_id: Option<&str>,
    request: &TelegramChatActionRequest,
    command_kind: &str,
    action_class: &str,
    provider_message_id: Option<&str>,
) -> Result<String, ApiError> {
    record_chat_lifecycle_command_with_payload(
        state,
        telegram_chat_id,
        request,
        command_kind,
        action_class,
        provider_message_id,
        false,
        json!({
            "source": "telegram_chat_lifecycle",
            "last_read_inbox_provider_message_id": provider_message_id,
        }),
        json!({
            "source": "telegram_chat_lifecycle",
            "last_read_inbox_provider_message_id": provider_message_id,
        }),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn record_chat_lifecycle_command_with_payload(
    state: &AppState,
    telegram_chat_id: Option<&str>,
    request: &TelegramChatActionRequest,
    command_kind: &str,
    action_class: &str,
    provider_message_id: Option<&str>,
    skip_default_audit: bool,
    payload: serde_json::Value,
    audit_metadata: serde_json::Value,
) -> Result<String, ApiError> {
    let service = telegram_provider_runtime_service(state)?;
    let command_id = lifecycle::new_command_id();
    let target_subject = telegram_chat_id.unwrap_or(request.provider_chat_id.trim());
    let target_ref = if let Some(telegram_chat_id) = telegram_chat_id {
        json!({
            "telegram_chat_id": telegram_chat_id,
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": provider_message_id
        })
    } else {
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": provider_message_id
        })
    };

    let _cmd = service
        .insert_command(
            &command_id,
            &request.account_id,
            command_kind,
            &format!(
                "{command_kind}:{}:{}",
                target_subject,
                Utc::now().timestamp_millis()
            ),
            &request.provider_chat_id,
            provider_message_id,
            "available",
            action_class,
            "confirmed",
            AUDIT_ACTOR_ID,
            payload,
            target_ref,
            audit_metadata,
        )
        .await?;

    if !skip_default_audit {
        api_audit_log(state)?
            .record(&NewApiAuditRecord::telegram_chat_action(
                AUDIT_ACTOR_ID,
                telegram_chat_id,
                &request.account_id,
                &request.provider_chat_id,
                provider_message_id,
                command_kind,
            ))
            .await?;
    }

    let chat = if let Some(telegram_chat_id) = telegram_chat_id {
        service.telegram_chat_by_id(telegram_chat_id).await?
    } else {
        None
    };
    let command_event = build_command_event(
        &request.account_id,
        &command_id,
        &request.provider_chat_id,
        telegram_chat_id,
        provider_message_id,
        command_kind,
        "queued",
        chat.as_ref(),
    );
    publish_telegram_event(state, command_event).await?;

    Ok(command_id)
}

pub(crate) async fn post_telegram_chat_join(
    State(state): State<AppState>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatLifecycleCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "participants.join")
        .await?;
    let command_id =
        record_chat_lifecycle_command(&state, None, &request, "join", "provider_write", None)
            .await?;

    Ok(Json(TelegramChatLifecycleCommandResponse {
        telegram_chat_id: None,
        provider_chat_id: request.provider_chat_id,
        action: "join".to_owned(),
        status: "queued".to_owned(),
        command_id,
    }))
}

pub(crate) async fn post_telegram_chat_leave(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatLifecycleCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "participants.leave")
        .await?;
    let command_id = record_chat_lifecycle_command(
        &state,
        Some(&telegram_chat_id),
        &request,
        "leave",
        "provider_write",
        None,
    )
    .await?;

    Ok(Json(TelegramChatLifecycleCommandResponse {
        telegram_chat_id: Some(telegram_chat_id),
        provider_chat_id: request.provider_chat_id,
        action: "leave".to_owned(),
        status: "queued".to_owned(),
        command_id,
    }))
}

async fn publish_chat_flag_event(
    state: &AppState,
    request: &TelegramChatActionRequest,
    telegram_chat_id: &str,
    event_type: &str,
    flag_key: &str,
    flag_value: bool,
) -> Result<(), ApiError> {
    let service = telegram_provider_runtime_service(state)?;
    let chat = service.telegram_chat_by_id(telegram_chat_id).await?;
    let event = build_chat_flag_event(
        event_type,
        request,
        telegram_chat_id,
        flag_key,
        flag_value,
        chat.as_ref(),
    );
    publish_telegram_event(state, event).await
}

async fn publish_chat_updated_event(
    state: &AppState,
    request: &TelegramChatActionRequest,
    telegram_chat_id: &str,
    action: &str,
) -> Result<(), ApiError> {
    let service = telegram_provider_runtime_service(state)?;
    let chat = service.telegram_chat_by_id(telegram_chat_id).await?;
    let event = build_chat_updated_event(request, telegram_chat_id, action, chat.as_ref());
    publish_telegram_event(state, event).await
}

pub(crate) async fn post_telegram_chat_pin(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.pin").await?;
    let metadata = telegram_provider_runtime_service(&state)?
        .set_chat_metadata_bool(&telegram_chat_id, "is_pinned", true)
        .await?;
    let _command_id =
        record_dialog_command(&state, &telegram_chat_id, &request, "pin", "provider_write").await?;
    publish_chat_flag_event(
        &state,
        &request,
        &telegram_chat_id,
        telegram_event_types::CHAT_PINNED,
        "is_pinned",
        true,
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "pin".to_owned(),
        status: "pinned".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_unpin(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.pin").await?;
    let metadata = telegram_provider_runtime_service(&state)?
        .set_chat_metadata_bool(&telegram_chat_id, "is_pinned", false)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "unpin",
        "provider_write",
    )
    .await?;
    publish_chat_flag_event(
        &state,
        &request,
        &telegram_chat_id,
        telegram_event_types::CHAT_PINNED,
        "is_pinned",
        false,
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "unpin".to_owned(),
        status: "unpinned".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_archive(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionRespo
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/provider_runtime_handlers/telegram/chat_folder_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/chat_folder_actions.rs`
- Size bytes / Размер в байтах: `8969`
- Included characters / Включено символов: `8969`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};

use super::chat_actions::{
    TelegramChatActionRequest, TelegramChatLifecycleCommandResponse,
    record_chat_lifecycle_command_with_payload,
};
use super::helpers::{AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed};
use crate::app::api_support::{api_audit_log, telegram_provider_runtime_service};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::TelegramError;
use crate::platform::audit::NewApiAuditRecord;

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatFolderReassignRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) target_provider_folder_ids: Vec<i64>,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatFolderReassignResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) command_ids: Vec<String>,
    pub(crate) added_provider_folder_ids: Vec<i64>,
    pub(crate) removed_provider_folder_ids: Vec<i64>,
}

fn folder_command_payload(source: &str, provider_folder_id: i64) -> Value {
    json!({
        "source": source,
        "provider_folder_id": provider_folder_id,
    })
}

fn chat_folder_ids(metadata: &Value) -> Vec<i64> {
    let Some(metadata_object) = metadata.as_object() else {
        return Vec::new();
    };
    let folder_ids = metadata_object
        .get("tdlib_chat_positions")
        .and_then(Value::as_object)
        .and_then(|positions| positions.get("folder_ids"))
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
        .unwrap_or_default();
    if !folder_ids.is_empty() {
        return folder_ids;
    }
    metadata_object
        .get("provider_folder_id")
        .and_then(Value::as_i64)
        .map(|value| vec![value])
        .unwrap_or_default()
}

fn unique_folder_ids(folder_ids: Vec<i64>) -> Vec<i64> {
    let mut unique = Vec::with_capacity(folder_ids.len());
    for folder_id in folder_ids {
        if !unique.contains(&folder_id) {
            unique.push(folder_id);
        }
    }
    unique
}

pub(crate) async fn post_telegram_chat_add_folder(
    State(state): State<AppState>,
    Path((telegram_chat_id, provider_folder_id)): Path<(String, i64)>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatLifecycleCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.folder_add")
        .await?;
    let payload = folder_command_payload("telegram_chat_lifecycle", provider_folder_id);
    let command_id = record_chat_lifecycle_command_with_payload(
        &state,
        Some(&telegram_chat_id),
        &request,
        "folder_add",
        "provider_write",
        None,
        true,
        payload.clone(),
        payload,
    )
    .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_chat_folder_add(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &request.account_id,
            &request.provider_chat_id,
            provider_folder_id,
        ))
        .await?;

    Ok(Json(TelegramChatLifecycleCommandResponse {
        telegram_chat_id: Some(telegram_chat_id),
        provider_chat_id: request.provider_chat_id,
        action: "folder_add".to_owned(),
        status: "queued".to_owned(),
        command_id,
    }))
}

pub(crate) async fn post_telegram_chat_remove_folder(
    State(state): State<AppState>,
    Path((telegram_chat_id, provider_folder_id)): Path<(String, i64)>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatLifecycleCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.folder_remove")
        .await?;
    let payload = folder_command_payload("telegram_chat_lifecycle", provider_folder_id);
    let command_id = record_chat_lifecycle_command_with_payload(
        &state,
        Some(&telegram_chat_id),
        &request,
        "folder_remove",
        "provider_write",
        None,
        true,
        payload.clone(),
        payload,
    )
    .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_chat_folder_remove(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &request.account_id,
            &request.provider_chat_id,
            provider_folder_id,
        ))
        .await?;

    Ok(Json(TelegramChatLifecycleCommandResponse {
        telegram_chat_id: Some(telegram_chat_id),
        provider_chat_id: request.provider_chat_id,
        action: "folder_remove".to_owned(),
        status: "queued".to_owned(),
        command_id,
    }))
}

pub(crate) async fn post_telegram_chat_reassign_folders(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatFolderReassignRequest>,
) -> Result<Json<TelegramChatFolderReassignResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(
        &state,
        &request.account_id,
        "dialogs.folder_reassign",
    )
    .await?;
    let target_provider_folder_ids = unique_folder_ids(request.target_provider_folder_ids);
    if target_provider_folder_ids.is_empty() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "folder reassignment requires at least one target_provider_folder_id".to_owned(),
        )));
    }

    let chat = telegram_provider_runtime_service(&state)?
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram chat `{telegram_chat_id}` was not found"
            )))
        })?;
    let current_folder_ids = chat_folder_ids(&chat.metadata);
    let added_provider_folder_ids = target_provider_folder_ids
        .iter()
        .copied()
        .filter(|folder_id| !current_folder_ids.contains(folder_id))
        .collect::<Vec<_>>();
    let removed_provider_folder_ids = current_folder_ids
        .iter()
        .copied()
        .filter(|folder_id| !target_provider_folder_ids.contains(folder_id))
        .collect::<Vec<_>>();

    if added_provider_folder_ids.is_empty() && removed_provider_folder_ids.is_empty() {
        return Ok(Json(TelegramChatFolderReassignResponse {
            telegram_chat_id,
            provider_chat_id: request.provider_chat_id,
            action: "folder_reassign".to_owned(),
            status: "noop".to_owned(),
            command_ids: Vec::new(),
            added_provider_folder_ids,
            removed_provider_folder_ids,
        }));
    }

    let lifecycle_request = TelegramChatActionRequest {
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        last_read_inbox_provider_message_id: None,
    };
    let mut command_ids =
        Vec::with_capacity(added_provider_folder_ids.len() + removed_provider_folder_ids.len());

    for provider_folder_id in &added_provider_folder_ids {
        let payload = folder_command_payload("telegram_chat_folder_reassign", *provider_folder_id);
        command_ids.push(
            record_chat_lifecycle_command_with_payload(
                &state,
                Some(&telegram_chat_id),
                &lifecycle_request,
                "folder_add",
                "provider_write",
                None,
                true,
                payload.clone(),
                payload,
            )
            .await?,
        );
    }
    for provider_folder_id in &removed_provider_folder_ids {
        let payload = folder_command_payload("telegram_chat_folder_reassign", *provider_folder_id);
        command_ids.push(
            record_chat_lifecycle_command_with_payload(
                &state,
                Some(&telegram_chat_id),
                &lifecycle_request,
                "folder_remove",
                "provider_write",
                None,
                true,
                payload.clone(),
                payload,
            )
            .await?,
        );
    }

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_chat_folder_reassign(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &request.account_id,
            &request.provider_chat_id,
            &target_provider_folder_ids,
            &added_provider_folder_ids,
            &removed_provider_folder_ids,
        ))
        .await?;

    Ok(Json(TelegramChatFolderReassignResponse {
        telegram_chat_id,
        provider_chat_id: request.provider_chat_id,
        action: "folder_reassign".to_owned(),
        status: "queued".to_owned(),
        command_ids,
        added_provider_folder_ids,
        removed_provider_folder_ids,
    }))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/chats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/chats.rs`
- Size bytes / Размер в байтах: `23985`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde_json::json;
use sqlx::Row;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
};
use crate::app::api_support::{
    TelegramChatListResponse, TelegramListQuery, api_audit_log, telegram_provider_runtime_service,
    telegram_runtime_use_case_context,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramChat, TelegramChatGroupFilterListResponse, TelegramChatMember, TelegramChatSyncRequest,
    TelegramChatSyncResponse, TelegramError, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse,
};
use crate::application::telegram_runtime;
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

const COMMUNICATION_CONVERSATION_CHANNEL_KINDS: &[&str] = &[
    "telegram_user",
    "telegram_bot",
    "whatsapp_web",
    "whatsapp_business_cloud",
];

fn build_event(
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": subject_id, "kind": "telegram_sync"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

pub(crate) async fn get_telegram_chats(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramChatListResponse>, ApiError> {
    let channel_kind = normalized_channel_kind(query.channel_kind.as_deref());
    let limit = query.limit.unwrap_or(50);
    let mut items = if includes_telegram_channel_kind(channel_kind) {
        telegram_provider_runtime_service(&state)?
            .list_chats(query.account_id.as_deref(), limit)
            .await?
    } else {
        Vec::new()
    };
    if includes_whatsapp_channel_kind(channel_kind) {
        items.extend(
            list_canonical_communication_conversations(
                &state,
                query.account_id.as_deref(),
                channel_kind,
                None,
                limit,
            )
            .await?,
        );
    }
    dedupe_and_sort_chats(&mut items, limit);

    Ok(Json(TelegramChatListResponse { items }))
}

pub(crate) async fn get_telegram_folders(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramChatGroupFilterListResponse>, ApiError> {
    let items = telegram_provider_runtime_service(&state)?
        .list_chat_group_filters(query.account_id.as_deref())
        .await?;

    Ok(Json(TelegramChatGroupFilterListResponse { items }))
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatDetailResponse {
    pub(crate) item: TelegramChat,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatMemberListResponse {
    pub(crate) items: Vec<TelegramChatMember>,
    pub(crate) next_cursor: Option<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatMembersQuery {
    pub(crate) query: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) limit: Option<i64>,
    pub(crate) cursor: Option<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatMembersSyncResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) synced_count: usize,
    pub(crate) items: Vec<TelegramChatMember>,
}

pub(crate) async fn get_telegram_chat_detail(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
) -> Result<Json<TelegramChatDetailResponse>, ApiError> {
    let item = if let Some(item) = telegram_provider_runtime_service(&state)?
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
    {
        item
    } else {
        canonical_communication_conversation(&state, &telegram_chat_id)
            .await?
            .ok_or_else(|| {
                ApiError::Telegram(TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                )))
            })?
    };

    Ok(Json(TelegramChatDetailResponse { item }))
}

pub(crate) async fn get_telegram_chat_members(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Query(query): Query<TelegramChatMembersQuery>,
) -> Result<Json<TelegramChatMemberListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    let items = match telegram_provider_runtime_service(&state)?
        .list_chat_members(
            &telegram_chat_id,
            query.query.as_deref(),
            query.role.as_deref(),
            limit,
            query.cursor.as_deref(),
        )
        .await
    {
        Ok(items) => items,
        Err(TelegramError::InvalidRequest(_)) => {
            list_canonical_conversation_members(
                &state,
                &telegram_chat_id,
                query.query.as_deref(),
                query.role.as_deref(),
                limit,
                query.cursor.as_deref(),
            )
            .await?
        }
        Err(error) => return Err(error.into()),
    };
    let next_cursor = if items.len() >= limit as usize {
        let offset = query
            .cursor
            .as_deref()
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0)
            .max(0)
            + limit;
        Some(offset.to_string())
    } else {
        None
    };

    Ok(Json(TelegramChatMemberListResponse { items, next_cursor }))
}

pub(crate) async fn post_telegram_chat_members_sync(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
) -> Result<Json<TelegramChatMembersSyncResponse>, ApiError> {
    let telegram_provider_runtime_service = telegram_provider_runtime_service(&state)?;
    let chat = telegram_provider_runtime_service
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram chat `{telegram_chat_id}` was not found"
            )))
        })?;
    ensure_telegram_account_operation_allowed(&state, &chat.account_id, "participants.sync")
        .await?;
    let started = build_event(
        telegram_event_types::SYNC_STARTED,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
        }),
    );
    publish_telegram_event(&state, started).await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let items = match telegram_runtime::sync_chat_members(&runtime_context, &telegram_chat_id).await
    {
        Ok(items) => items,
        Err(error) => {
            let failed = build_event(
                telegram_event_types::SYNC_FAILED,
                &chat.account_id,
                &telegram_chat_id,
                json!({
                    "scope": "members",
                    "provider_chat_id": &chat.provider_chat_id,
                    "status": "failed",
                }),
            );
            publish_telegram_event(&state, failed).await?;
            return Err(error.into());
        }
    };

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_participants_sync(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &chat.account_id,
            &chat.provider_chat_id,
            items.len() as i64,
        ))
        .await?;

    let progress = build_event(
        telegram_event_types::SYNC_PROGRESS,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
            "synced_count": items.len(),
            "status": "completed",
        }),
    );
    publish_telegram_event(&state, progress).await?;

    let completed = build_event(
        telegram_event_types::SYNC_COMPLETED,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
            "synced_count": items.len(),
            "status": "completed",
        }),
    );
    publish_telegram_event(&state, completed).await?;

    Ok(Json(TelegramChatMembersSyncResponse {
        telegram_chat_id,
        synced_count: items.len(),
        items,
    }))
}

pub(crate) async fn list_canonical_communication_conversations(
    state: &AppState,
    account_id: Option<&str>,
    channel_kind: Option<&str>,
    query: Option<&str>,
    limit: i64,
) -> Result<Vec<TelegramChat>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let limit = limit.clamp(1, 200);
    let like_pattern = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"));
    let canonical_channel_kinds = canonical_conversation_channel_kinds(channel_kind);
    let rows = sqlx::query(
        r#"
        SELECT
            conversation_id,
            account_id,
            channel_kind,
            provider_conversation_id,
            title,
            last_message_at,
            metadata,
            created_at,
            updated_at
        FROM communication_conversations
        WHERE channel_kind = ANY($1)
          AND ($2::text IS NULL OR account_id = $2)
          AND ($3::text IS NULL OR title ILIKE $3)
        ORDER BY COALESCE(last_message_at, updated_at) DESC, conversation_id ASC
        LIMIT $4
        "#,
    )
    .bind(canonical_channel_kinds)
    .bind(account_id.map(str::trim).filter(|value| !value.is_empty()))
    .bind(like_pattern.as_deref())
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(TelegramError::from)?;

    rows.into_iter()
        .filter_map(|row| canonical_row_to_chat(row).transpose())
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub(crate) async fn canonical_communication_conversation(
    state: &AppState,
    conversation_id: &str,
) -> Result<Option<TelegramChat>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let row = sqlx::query(
        r#"
        SELECT
            conversation_id,
            account_id,
            channel_kind,
            provider_conversation_id,
            title,
            last_message_at,
            metadata,
            created_at,
            updated_at
        FROM communication_conversations
        WHERE (conversation_id = $1 OR provider_conversation_id = $1)
          AND channel_kind = ANY($2)
        "#,
    )
    .bind(conversation_id.trim())
    .bind(COMMUNICATION_CONVERSATION_CHANNEL_KINDS)
    .fetch_optional(&pool)
    .await
    .map_err(TelegramError::from)?;

    match row {
        Some(row) => canonical_row_to_chat(row).map_err(Into::into),
        None => canonical_message_row_to_chat(state, conversation_id)
            .await
            .map_err(Into::into),
    }
}

async fn canonical_message_row_to_chat(
    state: &AppState,
    conversation_id: &str,
) -> Result<Option<TelegramChat>, TelegramError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let row = sqlx::query(
        r#"
        SELECT
            conversation_id,
            account_id,
            channel_kind,
            MAX(COALESCE(occurred_at, projected_at)) AS last_message_at,
            MIN(projected_at) AS created_at,
            MAX(projected_at) AS updated_at
        FROM communication_messages
        WHERE conversation_id = $1
          AND channel_kind = ANY($2)
        GROUP BY conversation_i
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/provider_runtime_handlers/telegram/commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/commands.rs`
- Size bytes / Размер в байтах: `1660`
- Included characters / Включено символов: `1660`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::api_support::telegram_provider_runtime_service;
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::TelegramCommandListResponse;
use crate::application::provider_runtime_contracts::TelegramError;

#[derive(Deserialize)]
pub(crate) struct TelegramCommandListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) command_kinds: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) async fn get_telegram_commands(
    State(state): State<AppState>,
    Query(query): Query<TelegramCommandListQuery>,
) -> Result<Json<TelegramCommandListResponse>, ApiError> {
    let account_id = query.account_id.ok_or_else(|| {
        ApiError::Telegram(TelegramError::InvalidRequest(
            "account_id is required".to_owned(),
        ))
    })?;
    let command_kinds = query
        .command_kinds
        .as_deref()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let response = telegram_provider_runtime_service(&state)?
        .list_commands(
            &account_id,
            query.provider_chat_id.as_deref(),
            query.provider_message_id.as_deref(),
            &command_kinds,
            query.limit.unwrap_or(50),
        )
        .await?;
    Ok(Json(response))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/helpers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/helpers.rs`
- Size bytes / Размер в байтах: `2053`
- Included characters / Включено символов: `2053`
- Truncated / Обрезано: `no`

```rust
use crate::app::api_support::{
    TelegramCapabilitiesResponse, event_store, telegram_provider_runtime_service,
    telegram_secret_reference_store,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::TelegramError;
use crate::platform::config::AppConfig;
use crate::platform::events::NewEventEnvelope;

pub(super) const AUDIT_ACTOR_ID: &str = "makosh-frontend";

pub(super) fn telegram_api_hash_from_config(config: &AppConfig) -> Option<String> {
    config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
}

pub(super) fn telegram_secret_store(
    state: &AppState,
) -> Result<crate::platform::secrets::SecretReferenceStore, ApiError> {
    telegram_secret_reference_store(state)
}

pub(super) async fn publish_telegram_event(
    state: &AppState,
    event: NewEventEnvelope,
) -> Result<(), ApiError> {
    if state.database.pool().is_some()
        && let Err(error) = event_store(state)?.append(&event).await
    {
        tracing::warn!(error = %error, "failed to append event to event store");
    }

    let _ = state.event_bus.broadcast(event);
    Ok(())
}

pub(super) async fn ensure_telegram_account_operation_allowed(
    state: &AppState,
    account_id: &str,
    operation: &str,
) -> Result<(), ApiError> {
    let account = telegram_provider_runtime_service(state)?
        .telegram_account_record(account_id)
        .await?;
    let capabilities = TelegramCapabilitiesResponse::current_for_account(&state.config, &account);
    let capability = capabilities
        .capabilities
        .iter()
        .find(|item| item.operation == operation)
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram capability `{operation}` is not defined"
            )))
        })?;

    if matches!(capability.status.as_str(), "available" | "degraded") {
        return Ok(());
    }

    Err(ApiError::Telegram(TelegramError::InvalidRequest(
        capability.reason.clone(),
    )))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/media.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/media.rs`
- Size bytes / Размер в байтах: `22045`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::extract::State;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::helpers::{AUDIT_ACTOR_ID, publish_telegram_event};
use crate::app::api_support::{
    api_audit_log, communication_provider_account_store, communication_storage_store,
    telegram_provider_runtime_service, telegram_runtime_use_case_context,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramAttachmentDownloadStateUpdate, TelegramCommandKind, TelegramError,
    TelegramMediaDownloadRequest, TelegramMediaDownloadResponse, TelegramMediaSendType,
    ensure_telegram_account_active, lifecycle,
};
use crate::application::telegram_runtime;
use crate::domains::communications::core::CommunicationProviderAccountStore;
use crate::domains::communications::storage::AttachmentSafetyScanStatus;
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

fn build_event(
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": subject_id, "kind": "telegram_message"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

fn build_upload_event(
    event_type: &str,
    account_id: &str,
    command_id: &str,
    provider_chat_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    let mut event_payload = json!({
        "command_id": command_id,
        "account_id": account_id,
        "provider_chat_id": provider_chat_id,
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (event_payload.as_object_mut(), payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
        payload_obj.insert("payload".to_owned(), payload);
    }
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": command_id, "kind": "telegram_command"}),
    )
    .payload(event_payload)
    .build()
    .expect("event envelope must be valid")
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TelegramMediaUploadRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) media_type: String,
    pub(crate) caption: Option<String>,
    pub(crate) filename: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TelegramMediaUploadResponse {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: String,
    pub(crate) media_type: String,
    pub(crate) status: String,
    pub(crate) reconciliation_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedMediaUploadRequest {
    command_id: String,
    account_id: String,
    provider_chat_id: String,
    attachment_id: Option<String>,
    blob_id: Option<String>,
    media_type: TelegramMediaSendType,
    caption: Option<String>,
    filename: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UploadAttachmentRef {
    attachment_id: Option<String>,
    blob_id: String,
    content_type: String,
    filename: Option<String>,
    size_bytes: i64,
    sha256: String,
    scan_status: String,
}

pub(crate) async fn post_telegram_media_upload(
    State(state): State<AppState>,
    Json(request): Json<TelegramMediaUploadRequest>,
) -> Result<Json<TelegramMediaUploadResponse>, ApiError> {
    let request = validate_media_upload_request(request)?;
    let provider_account_store = communication_provider_account_store(&state)?;
    let account = provider_account_store
        .get(&request.account_id)
        .await?
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram account `{}` was not found",
                request.account_id
            ))
        })?;
    if !account.provider_kind.is_telegram() {
        return Err(TelegramError::InvalidRequest(format!(
            "account `{}` is not a Telegram provider account",
            account.account_id
        ))
        .into());
    }
    ensure_telegram_account_active(&account)?;
    let runtime_kind = account
        .config
        .get("runtime")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    if runtime_kind != "tdlib_qr_authorized" {
        return Err(TelegramError::InvalidRequest(format!(
            "Telegram media upload requires a tdlib_qr_authorized account; `{}` uses `{runtime_kind}`",
            account.account_id
        ))
        .into());
    }

    let mail_store = communication_storage_store(&state)?;
    let attachment = resolve_upload_attachment(&mail_store, &request).await?;
    let audit_metadata = json!({
        "capability": "telegram.media.upload",
        "action_class": "provider_write",
        "confirmation_decision": "explicit_user_confirmation",
        "attachment_id": &attachment.attachment_id,
        "blob_id": &attachment.blob_id,
        "media_type": request.media_type.as_str(),
        "content_type": &attachment.content_type,
        "size_bytes": attachment.size_bytes,
        "sha256": &attachment.sha256,
        "scan_status": &attachment.scan_status,
    });
    let idempotency_key = media_upload_idempotency_key(&request, &attachment.blob_id);
    let provider_runtime = telegram_provider_runtime_service(&state)?;
    if let Some(existing) = provider_runtime
        .find_command_by_idempotency(&request.account_id, &idempotency_key)
        .await?
    {
        return Ok(Json(media_upload_response(&existing)));
    }
    let command = provider_runtime
        .insert_command(
            &request.command_id,
            &request.account_id,
            TelegramCommandKind::SendMedia.as_str(),
            &idempotency_key,
            &request.provider_chat_id,
            None,
            "available",
            "provider_write",
            "confirmed",
            AUDIT_ACTOR_ID,
            json!({
            "attachment_id": attachment.attachment_id.clone(),
            "blob_id": attachment.blob_id.clone(),
            "media_type": request.media_type.as_str(),
            "caption": request.caption.clone(),
            "filename": request.filename.clone().or(attachment.filename.clone()),
            "content_type": attachment.content_type.clone(),
            "size_bytes": attachment.size_bytes,
            "sha256": attachment.sha256.clone(),
            }),
            json!({
            "provider_chat_id": request.provider_chat_id,
            "attachment_id": request.attachment_id,
            "blob_id": request.blob_id,
            }),
            audit_metadata.clone(),
        )
        .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_media_upload(
            AUDIT_ACTOR_ID,
            &command.command_id,
            &command.account_id,
            &command.provider_chat_id,
            command
                .payload
                .get("attachment_id")
                .and_then(serde_json::Value::as_str),
            command
                .payload
                .get("blob_id")
                .and_then(serde_json::Value::as_str),
            command
                .payload
                .get("media_type")
                .and_then(serde_json::Value::as_str),
        ))
        .await?;

    publish_telegram_event(
        &state,
        build_upload_event(
            telegram_event_types::MEDIA_UPLOAD_STARTED,
            &command.account_id,
            &command.command_id,
            &command.provider_chat_id,
            json!({
                "command_kind": command.command_kind,
                "idempotency_key": command.idempotency_key,
                "payload": command.payload,
                "target_ref": command.target_ref,
                "capability_state": command.capability_state,
                "action_class": command.action_class,
                "confirmation_decision": command.confirmation_decision,
                "status": &command.status,
                "retry_count": command.retry_count,
                "max_retries": command.max_retries,
                "last_error": command.last_error,
                "result_payload": command.result_payload,
                "audit_metadata": command.audit_metadata,
                "actor_id": command.actor_id,
                "happened_at": command.happened_at,
                "next_attempt_at": command.next_attempt_at,
                "last_attempt_at": command.last_attempt_at,
                "provider_observed_at": command.provider_observed_at,
                "provider_state": command.provider_state,
                "reconciliation_status": command.reconciliation_status,
                "reconciled_at": command.reconciled_at,
                "dead_lettered_at": command.dead_lettered_at,
                "completed_at": command.completed_at,
                "created_at": command.created_at,
                "updated_at": command.updated_at,
                "attachment_id": command.payload.get("attachment_id").cloned(),
                "blob_id": command.payload.get("blob_id").cloned(),
                "media_type": command.payload.get("media_type").cloned(),
                "filename": command.payload.get("filename").cloned(),
            }),
        ),
    )
    .await?;
    publish_telegram_event(
        &state,
        build_upload_event(
            telegram_event_types::COMMAND_STATUS_CHANGED,
            &command.account_id,
            &command.command_id,
            &command.provider_chat_id,
            json!({"status": &command.status, "source": "media_upload_api"}),
        ),
    )
    .await?;

    Ok(Json(media_upload_response(&command)))
}

fn media_upload_response(
    command: &crate::application::provider_runtime_contracts::TelegramProviderWriteCommand,
) -> TelegramMediaUploadResponse {
    TelegramMediaUploadResponse {
        command_id: command.command_id.clone(),
        account_id: command.account_id.clone(),
        provider_chat_id: command.provider_chat_id.clone(),
        attachment_id: command
            .payload
            .get("attachment_id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        blob_id: command
            .payload
            .get("blob_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        media_type: command
            .payload
            .get("media_type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        status: command.status.clone(),
        reconciliation_status: command.reconciliation_status.clone(),
    }
}

pub(crate) async fn post_telegram_media_download(
    State(state): State<AppState>,
    Json(request): Json<TelegramMediaDownloadRequest>,
) -> Result<Json<TelegramMediaDownloadResponse>, ApiError> {
    let started = build_event(
        telegram_event_types::MEDIA_DOWNLOAD_STARTED,
        &request.account_id,
        &request.provider_message_id,
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": &requ
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/provider_runtime_handlers/telegram/messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/messages.rs`
- Size bytes / Размер в байтах: `30637`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;

use super::helpers::ensure_telegram_account_operation_allowed;
use crate::app::api_support::{
    communication_provider_account_store, ensure_fixture_routes_enabled,
    telegram_fixture_ingest_service, telegram_message_write_service,
    telegram_runtime_use_case_context,
};
use crate::app::provider_runtime_handlers::telegram::chats::canonical_communication_conversation;
use crate::app::provider_runtime_handlers::whatsapp::{
    WhatsAppConversationCommandApiRequest, post_whatsapp_command_delete,
    post_whatsapp_command_edit, post_whatsapp_command_send_text,
    post_whatsapp_conversation_archive, post_whatsapp_conversation_mark_read,
    post_whatsapp_conversation_mark_unread, post_whatsapp_conversation_mute,
    post_whatsapp_conversation_pin, post_whatsapp_conversation_unarchive,
    post_whatsapp_conversation_unmute, post_whatsapp_conversation_unpin,
};
use crate::app::{ApiError, AppState};
use crate::application::communication_provider_writes::{
    CommunicationConversationMessageRequest, CommunicationProviderMessageCommandResponse,
};
use crate::application::provider_runtime_contracts::NewTelegramMessage;
use crate::application::provider_runtime_contracts::{
    TelegramDeleteRequest, TelegramEditRequest, TelegramForwardRequest, TelegramLifecycleResponse,
    TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessageIngestResult,
    TelegramMessageTombstoneListResponse, TelegramMessageVersionListResponse, TelegramPinRequest,
    TelegramReplyRequest, TelegramRestoreVisibilityRequest, WhatsAppDeleteRequest,
    WhatsAppEditRequest, WhatsAppProviderCommandResponse, WhatsAppTextSendRequest,
};
use crate::application::telegram_runtime;

mod mark_read;
mod reactions;

pub(crate) use mark_read::post_telegram_message_mark_read;
pub(crate) use reactions::{
    delete_telegram_reaction, get_telegram_reactions, post_telegram_reaction,
};

pub(crate) async fn post_telegram_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewTelegramMessage>,
) -> Result<Json<TelegramMessageIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let response = telegram_fixture_ingest_service(&state)?
        .ingest_message(&request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_manual_send(
    State(state): State<AppState>,
    Json(request): Json<TelegramManualSendRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.send_text")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_manual_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_communication_conversation_message(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(mut request): Json<CommunicationConversationMessageRequest>,
) -> Result<Json<CommunicationProviderMessageCommandResponse>, ApiError> {
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(next_whatsapp_command_id);
        let response = post_whatsapp_command_send_text(
            State(state.clone()),
            Json(WhatsAppTextSendRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("send_text", &command_id),
                account_id: request.account_id.clone(),
                provider_chat_id: conversation_id.clone(),
                text: request.text.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_communication_response(
            &command_id,
            &conversation_id,
            None,
            &response,
        )));
    }

    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.send_text")
        .await?;
    let command_id = request
        .command_id
        .clone()
        .unwrap_or_else(crate::application::communication_provider_writes::new_telegram_command_id);
    request.command_id = Some(command_id.clone());
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_conversation_message(&runtime_context, &conversation_id, request)
        .await?;
    Ok(Json(CommunicationProviderMessageCommandResponse::telegram(
        command_id, &response,
    )))
}

pub(crate) async fn post_telegram_message_reply(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<TelegramReplyRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.reply")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_reply_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_forward(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<TelegramForwardRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.forward")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_forward_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// Lifecycle endpoints (ADR-0091)
// ---------------------------------------------------------------------------

pub(crate) async fn post_telegram_message_edit(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    let request: ProviderNeutralEditRequest = serde_json::from_value(payload)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid message edit payload"))?;
    let command_id = request
        .command_id
        .clone()
        .unwrap_or_else(next_whatsapp_command_id);
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let response = post_whatsapp_command_edit(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppEditRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("edit", &command_id),
                account_id: request.account_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                provider_message_id: request.provider_message_id.clone(),
                text: request.new_text.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_lifecycle_response(
            "edit",
            &message_id,
            &response,
        )));
    }

    let request = TelegramEditRequest {
        command_id,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_message_id: request.provider_message_id,
        new_text: request.new_text,
    };
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.edit").await?;
    let response = telegram_message_write_service(&state)?
        .edit_message(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    let request: ProviderNeutralDeleteRequest = serde_json::from_value(payload)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid message delete payload"))?;
    let command_id = request
        .command_id
        .clone()
        .unwrap_or_else(next_whatsapp_command_id);
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let response = post_whatsapp_command_delete(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppDeleteRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("delete", &command_id),
                confirmation_decision: Some("confirmed".to_owned()),
                account_id: request.account_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                provider_message_id: request.provider_message_id.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_lifecycle_response(
            "delete",
            &message_id,
            &response,
        )));
    }

    let request = TelegramDeleteRequest {
        command_id,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_message_id: request.provider_message_id,
        reason_class: request
            .reason_class
            .unwrap_or_else(|| "deleted_by_owner".to_owned()),
        actor_class: request.actor_class.unwrap_or_else(|| "owner".to_owned()),
        is_provider_delete: request.is_provider_delete.unwrap_or(false),
    };
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.delete")
        .await?;
    let response = telegram_message_write_service(&state)?
        .delete_message(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_restore_visibility(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramRestoreVisibilityRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(
        &state,
        &request.account_id,
        "messages.restore_visibility",
    )
    .await?;
    let response = telegram_message_write_service(&state)?
        .restore_message_visibility(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_pin(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramPinRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.pin").await?;
    let response = telegram_message_write_service(&state)?
        .pin_message(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_communication_conversation_pin(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&s
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/provider_runtime_handlers/telegram/messages/mark_read.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/messages/mark_read.rs`
- Size bytes / Размер в байтах: `842`
- Included characters / Включено символов: `842`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};

use crate::app::{ApiError, AppState};
use crate::application::communication_provider_writes::{
    TelegramMessageMarkReadRequest, TelegramMessageMarkReadResponse,
};

use super::super::helpers::ensure_telegram_account_operation_allowed;
use super::telegram_message_write_service;

pub(crate) async fn post_telegram_message_mark_read(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramMessageMarkReadRequest>,
) -> Result<Json<TelegramMessageMarkReadResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.mark_read")
        .await?;
    let response = telegram_message_write_service(&state)?
        .mark_message_read(&message_id, &request)
        .await?;
    Ok(Json(response))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/messages/reactions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/messages/reactions.rs`
- Size bytes / Размер в байтах: `6137`
- Included characters / Включено символов: `6137`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;

use crate::app::api_support::{TelegramReactionDeleteQuery, communication_provider_account_store};
use crate::app::provider_runtime_handlers::whatsapp::{
    delete_whatsapp_command_react, post_whatsapp_command_react,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramReactionListResponse, TelegramReactionRequest, TelegramReactionResponse,
    WhatsAppProviderCommandResponse, WhatsAppReactionRequest,
};

use super::super::helpers::ensure_telegram_account_operation_allowed;
use super::telegram_message_write_service;

/// POST /api/v1/communications/messages/{message_id}/reactions
pub(crate) async fn post_telegram_reaction(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramReactionRequest>,
) -> Result<Json<TelegramReactionResponse>, ApiError> {
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(next_whatsapp_command_id);
        let response = post_whatsapp_command_react(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppReactionRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("react", &command_id),
                account_id: request.account_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                provider_message_id: request.provider_message_id.clone(),
                reaction_emoji: request.reaction_emoji.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_reaction_response(
            &message_id,
            &request,
            true,
            &response,
        )));
    }

    ensure_telegram_account_operation_allowed(&state, &request.account_id, "reactions.add").await?;
    let response = telegram_message_write_service(&state)?
        .add_reaction(&message_id, request)
        .await?;
    Ok(Json(response))
}

/// DELETE /api/v1/communications/messages/{message_id}/reactions
pub(crate) async fn delete_telegram_reaction(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<TelegramReactionDeleteQuery>,
) -> Result<Json<TelegramReactionResponse>, ApiError> {
    let request = TelegramReactionRequest {
        account_id: query.account_id.clone(),
        provider_chat_id: query.provider_chat_id.clone(),
        provider_message_id: query.provider_message_id.clone(),
        reaction_emoji: query.reaction_emoji.clone(),
        sender_id: query.sender_id.clone().unwrap_or_default(),
        sender_display_name: query.sender_display_name.clone(),
        command_id: query.command_id.clone(),
    };
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(next_whatsapp_command_id);
        let response = delete_whatsapp_command_react(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppReactionRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("unreact", &command_id),
                account_id: request.account_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                provider_message_id: request.provider_message_id.clone(),
                reaction_emoji: request.reaction_emoji.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_reaction_response(
            &message_id,
            &request,
            false,
            &response,
        )));
    }

    ensure_telegram_account_operation_allowed(&state, &request.account_id, "reactions.remove")
        .await?;
    let response = telegram_message_write_service(&state)?
        .remove_reaction(&message_id, request)
        .await?;
    Ok(Json(response))
}

/// GET /api/v1/communications/messages/{message_id}/reactions
pub(crate) async fn get_telegram_reactions(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramReactionListResponse>, ApiError> {
    let response = telegram_message_write_service(&state)?
        .reactions(&message_id)
        .await?;
    Ok(Json(response))
}

fn next_whatsapp_command_id() -> String {
    format!(
        "whatsapp-command-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

fn whatsapp_command_idempotency_key(operation: &str, command_id: &str) -> String {
    format!("communications:whatsapp:{operation}:{command_id}")
}

fn whatsapp_command_response_to_reaction_response(
    message_id: &str,
    request: &TelegramReactionRequest,
    is_active: bool,
    response: &WhatsAppProviderCommandResponse,
) -> TelegramReactionResponse {
    TelegramReactionResponse {
        reaction_id: format!(
            "{}:{}:{}",
            request.provider_message_id, request.reaction_emoji, response.command_id
        ),
        message_id: message_id.to_owned(),
        account_id: response.account_id.clone(),
        provider_chat_id: response.provider_chat_id.clone(),
        provider_message_id: response.provider_message_id.clone().unwrap_or_default(),
        reaction_emoji: request.reaction_emoji.clone(),
        is_active,
        status: response.status.clone(),
        timestamp: response.updated_at,
    }
}
```

### `backend/src/app/provider_runtime_handlers/telegram/outbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/outbox.rs`
- Size bytes / Размер в байтах: `2297`
- Included characters / Включено символов: `2297`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use serde_json::json;

use super::helpers::publish_telegram_event;
use crate::app::api_support::telegram_provider_runtime_service;
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::TelegramProviderWriteCommand;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

pub(crate) async fn post_telegram_command_retry(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
) -> Result<Json<TelegramProviderWriteCommand>, ApiError> {
    let now = Utc::now();
    let command = telegram_provider_runtime_service(&state)?
        .manual_retry_command(&command_id, now)
        .await?
        .ok_or(ApiError::NotFound)?;

    let event = NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        telegram_event_types::COMMAND_STATUS_CHANGED.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_command"}),
    )
    .payload(json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "provider_chat_id": command.provider_chat_id,
        "message_id": command.provider_message_id,
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "result_payload": command.result_payload,
        "source": "manual_retry",
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "provider_state": command.provider_state,
        "reconciliation_status": command.reconciliation_status,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
        "payload": {
            "source": "manual_retry",
            "next_attempt_at": command.next_attempt_at,
        },
    }))
    .build()
    .expect("telegram command retry event must be valid");
    publish_telegram_event(&state, event).await?;

    Ok(Json(command))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/qr_login.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/qr_login.rs`
- Size bytes / Размер в байтах: `2247`
- Included characters / Включено символов: `2247`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};

use super::helpers::telegram_api_hash_from_config;
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest,
    TelegramQrLoginStatusResponse, qr_login,
};

pub(crate) async fn post_telegram_qr_login_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramQrLoginStartRequest>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    let request = request.with_app_credentials(
        state.config.telegram_api_id(),
        telegram_api_hash_from_config(&state.config),
    );

    Ok(Json(
        qr_login::start_qr_login(
            state.config.clone(),
            state.account_setup.pending_telegram_qr_login.clone(),
            request,
        )
        .await?,
    ))
}

pub(crate) async fn get_telegram_qr_login_status(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    let pending = state
        .account_setup
        .pending_telegram_qr_login
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    let session = pending
        .get(setup_id.trim())
        .map(|session| session.response.clone())
        .ok_or(ApiError::Telegram(TelegramError::QrLoginNotFound))?;

    Ok(Json(session))
}

pub(crate) async fn delete_telegram_qr_login(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let setup_id = setup_id.trim().to_owned();
    qr_login::cancel_qr_login(
        state.account_setup.pending_telegram_qr_login.clone(),
        &setup_id,
    )?;

    Ok(Json(json!({
        "setup_id": setup_id,
        "cancelled": true
    })))
}

pub(crate) async fn post_telegram_qr_login_password(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
    Json(request): Json<TelegramQrLoginPasswordRequest>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    Ok(Json(qr_login::submit_qr_login_password(
        state.account_setup.pending_telegram_qr_login.clone(),
        &setup_id,
        request,
    )?))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/raw.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/raw.rs`
- Size bytes / Размер в байтах: `4325`
- Included characters / Включено символов: `4325`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::Value;

use crate::app::api_support::communication_ingestion_store;
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::TelegramError;
use crate::domains::communications::core::StoredRawCommunicationRecord;
use crate::domains::communications::messages::ProviderChannelMessageStore;

const COMMUNICATION_RAW_EVIDENCE_CHANNEL_KINDS: &[&str] = &[
    "telegram_user",
    "telegram_bot",
    "whatsapp_web",
    "whatsapp_business_cloud",
];

#[derive(Serialize)]
pub(crate) struct TelegramRawMessageResponse {
    pub(crate) raw_record: TelegramRawMessageRecord,
}

#[derive(Serialize)]
pub(crate) struct TelegramRawMessageRecord {
    pub(crate) raw_record_id: String,
    pub(crate) account_id: String,
    pub(crate) record_kind: String,
    pub(crate) provider_record_id: String,
    pub(crate) source_fingerprint: String,
    pub(crate) import_batch_id: String,
    pub(crate) occurred_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) captured_at: chrono::DateTime<chrono::Utc>,
    pub(crate) payload: Value,
    pub(crate) provenance: Value,
}

impl From<StoredRawCommunicationRecord> for TelegramRawMessageRecord {
    fn from(record: StoredRawCommunicationRecord) -> Self {
        Self {
            raw_record_id: record.raw_record_id,
            account_id: record.account_id,
            record_kind: record.record_kind,
            provider_record_id: record.provider_record_id,
            source_fingerprint: record.source_fingerprint,
            import_batch_id: record.import_batch_id,
            occurred_at: record.occurred_at,
            captured_at: record.captured_at,
            payload: redact_secret_material(record.payload),
            provenance: redact_secret_material(record.provenance),
        }
    }
}

/// GET /api/v1/communications/messages/{message_id}/raw-evidence
pub(crate) async fn get_telegram_message_raw(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramRawMessageResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let Some(message) = ProviderChannelMessageStore::new(pool)
        .message_by_id(&message_id, COMMUNICATION_RAW_EVIDENCE_CHANNEL_KINDS)
        .await
        .map_err(TelegramError::from)?
    else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let Some(raw_record) = communication_ingestion_store(&state)?
        .raw_record(&message.raw_record_id)
        .await?
    else {
        return Err(ApiError::CommunicationMessageNotFound);
    };

    Ok(Json(TelegramRawMessageResponse {
        raw_record: raw_record.into(),
    }))
}

fn redact_secret_material(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    if is_secret_key(&key) {
                        (key, Value::String("[redacted]".to_owned()))
                    } else {
                        (key, redact_secret_material(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_material).collect())
        }
        other => other,
    }
}

fn is_secret_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "access_token"
            | "api_hash"
            | "authorization"
            | "auth"
            | "bot_token"
            | "client_secret"
            | "cookie"
            | "cookies"
            | "credential"
            | "credentials"
            | "password"
            | "private_key"
            | "proxy_password"
            | "refresh_token"
            | "secret"
            | "session"
            | "session_blob"
            | "session_cookie"
            | "session_encryption_key"
            | "session_key"
            | "token"
    ) || normalized.ends_with("_token")
        || normalized.ends_with("_cookie")
        || normalized.ends_with("_credentials")
        || normalized.ends_with("_private_key")
}
```

### `backend/src/app/provider_runtime_handlers/telegram/runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/runtime.rs`
- Size bytes / Размер в байтах: `2625`
- Included characters / Включено символов: `2625`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use super::helpers::AUDIT_ACTOR_ID;
use crate::app::api_support::{api_audit_log, telegram_runtime_use_case_context};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramRuntimeRestartRequest, TelegramRuntimeStartRequest, TelegramRuntimeStatus,
    TelegramRuntimeStopRequest,
};
use crate::application::telegram_runtime;
use crate::platform::audit::NewApiAuditRecord;

#[derive(Deserialize)]
pub(crate) struct TelegramRuntimeStatusQuery {
    pub(crate) account_id: String,
}

pub(crate) async fn get_telegram_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<TelegramRuntimeStatusQuery>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    Ok(Json(
        telegram_runtime::runtime_status(&runtime_context, &query.account_id).await?,
    ))
}

pub(crate) async fn post_telegram_runtime_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStartRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    Ok(Json(
        telegram_runtime::start_runtime(&runtime_context, &request).await?,
    ))
}

pub(crate) async fn post_telegram_runtime_stop(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStopRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let status = telegram_runtime::stop_runtime(&runtime_context, &request).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_runtime_stop(
            AUDIT_ACTOR_ID,
            &status.account_id,
            &status.provider_kind,
            &status.runtime_kind,
            &status.status,
        ))
        .await?;

    Ok(Json(status))
}

pub(crate) async fn post_telegram_runtime_restart(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeRestartRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let status = telegram_runtime::restart_runtime(&runtime_context, &request).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_runtime_restart(
            AUDIT_ACTOR_ID,
            &status.account_id,
            &status.provider_kind,
            &status.runtime_kind,
            &status.status,
        ))
        .await?;

    Ok(Json(status))
}
```

### `backend/src/app/provider_runtime_handlers/telegram/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/search.rs`
- Size bytes / Размер в байтах: `21280`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::chats::{
    dedupe_and_sort_chats, includes_telegram_channel_kind, includes_whatsapp_channel_kind,
    list_canonical_communication_conversations, normalized_channel_kind,
};
use crate::app::api_support::{
    telegram_provider_runtime_service, telegram_runtime_use_case_context,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{TelegramError, models::TelegramChat};
use crate::application::telegram_runtime;
use crate::domains::communications::messages::ProviderChannelMessageStore;
use crate::platform::communications::ProviderChannelMessage;

const COMMUNICATION_SEARCH_CHANNEL_KINDS: &[&str] = &[
    "telegram_user",
    "telegram_bot",
    "whatsapp_web",
    "whatsapp_business_cloud",
];

#[derive(Deserialize)]
pub(crate) struct TelegramMessageSearchQuery {
    pub(crate) q: String,
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramProviderSearchCommand {
    pub(crate) account_id: String,
    pub(crate) q: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramMediaSearchQuery {
    pub(crate) q: Option<String>,
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramChatSearchQuery {
    pub(crate) q: String,
    pub(crate) account_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramPinnedMessagesQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramSearchResponse {
    pub(crate) query: String,
    pub(crate) items: Vec<crate::application::provider_runtime_contracts::TelegramMessage>,
    pub(crate) total: usize,
}

#[derive(Serialize)]
pub(crate) struct TelegramProviderSearchResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) query: String,
    pub(crate) limit: i32,
    pub(crate) status: String,
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TelegramChatSearchResponse {
    pub(crate) query: String,
    pub(crate) items: Vec<TelegramChat>,
    pub(crate) total: usize,
}

#[derive(Serialize)]
pub(crate) struct TelegramMediaItem {
    pub(crate) attachment_id: Option<String>,
    pub(crate) message_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) file_name: String,
    pub(crate) kind: String,
    pub(crate) mime_type: Option<String>,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) occurred_at: Option<String>,
    pub(crate) download_state: String,
    pub(crate) tdlib_file_id: Option<i64>,
    pub(crate) provider_attachment_id: Option<String>,
    pub(crate) local_path: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TelegramMediaSearchResponse {
    pub(crate) query: Option<String>,
    pub(crate) source: String,
    pub(crate) provider_search_attempted: bool,
    pub(crate) provider_search_error: Option<String>,
    pub(crate) items: Vec<TelegramMediaItem>,
}

/// GET /api/v1/communications/search/messages?q=&account_id=&provider_chat_id=&limit=
pub(crate) async fn search_telegram_messages(
    State(state): State<AppState>,
    Query(query): Query<TelegramMessageSearchQuery>,
) -> Result<Json<TelegramSearchResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query.q.trim().to_owned();
    let channel_kinds = search_channel_kinds(query.channel_kind.as_deref());

    if search_q.is_empty() {
        return Err(ApiError::Telegram(
            crate::application::provider_runtime_contracts::TelegramError::InvalidRequest(
                "search query `q` is required".to_owned(),
            ),
        ));
    }

    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let items: Vec<crate::application::provider_runtime_contracts::TelegramMessage> =
        ProviderChannelMessageStore::new(pool)
            .search_messages(
                query.account_id.as_deref(),
                query.provider_chat_id.as_deref(),
                &search_q,
                channel_kinds,
                limit,
            )
            .await
            .map_err(TelegramError::from)?
            .into_iter()
            .map(provider_channel_message_to_search_message)
            .collect();

    Ok(Json(TelegramSearchResponse {
        query: search_q,
        total: items.len(),
        items,
    }))
}

/// POST /api/v1/integrations/telegram/provider-search
pub(crate) async fn post_telegram_provider_search(
    State(state): State<AppState>,
    Json(payload): Json<TelegramProviderSearchCommand>,
) -> Result<Json<TelegramProviderSearchResponse>, ApiError> {
    let limit = payload.limit.unwrap_or(50).clamp(1, 200);
    let search_q = payload.q.trim().to_owned();
    let account_id = payload.account_id.trim();

    if account_id.is_empty() {
        return Err(ApiError::Telegram(
            crate::application::provider_runtime_contracts::TelegramError::InvalidRequest(
                "search payload account_id is required".to_owned(),
            ),
        ));
    }

    if search_q.is_empty() {
        return Err(ApiError::Telegram(
            crate::application::provider_runtime_contracts::TelegramError::InvalidRequest(
                "search query `q` is required".to_owned(),
            ),
        ));
    }

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let result = telegram_runtime::refresh_provider_search(
        &runtime_context,
        account_id.to_owned(),
        payload.provider_chat_id.clone(),
        search_q.clone(),
        limit as i32,
    )
    .await;

    let (status, error) = match result {
        Ok(()) => ("queued".to_owned(), None),
        Err(error) => {
            tracing::debug!(
                error = %error,
                account_id = %account_id,
                "post_telegram_provider_search: TDLib provider search failed"
            );
            ("failed".to_owned(), Some(error.to_string()))
        }
    };

    Ok(Json(TelegramProviderSearchResponse {
        account_id: account_id.to_owned(),
        provider_chat_id: payload.provider_chat_id,
        query: search_q,
        limit: limit as i32,
        status,
        error,
    }))
}

/// GET /api/v1/communications/conversations/search?q=&account_id=&limit=
pub(crate) async fn search_telegram_chats(
    State(state): State<AppState>,
    Query(query): Query<TelegramChatSearchQuery>,
) -> Result<Json<TelegramChatSearchResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query.q.trim().to_owned();
    let channel_kind = normalized_channel_kind(query.channel_kind.as_deref());

    if search_q.is_empty() {
        return Err(ApiError::Telegram(
            crate::application::provider_runtime_contracts::TelegramError::InvalidRequest(
                "search query `q` is required".to_owned(),
            ),
        ));
    }

    let mut items = if includes_telegram_channel_kind(channel_kind) {
        telegram_provider_runtime_service(&state)?
            .search_chats(query.account_id.as_deref(), &search_q, limit)
            .await?
    } else {
        Vec::new()
    };
    if includes_whatsapp_channel_kind(channel_kind) {
        items.extend(
            list_canonical_communication_conversations(
                &state,
                query.account_id.as_deref(),
                channel_kind,
                Some(&search_q),
                limit,
            )
            .await?,
        );
    }
    dedupe_and_sort_chats(&mut items, limit);

    Ok(Json(TelegramChatSearchResponse {
        query: search_q,
        total: items.len(),
        items,
    }))
}

/// GET /api/v1/communications/conversations/{conversation_id}/pinned-messages?limit=
pub(crate) async fn get_telegram_pinned_messages(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<TelegramPinnedMessagesQuery>,
) -> Result<Json<crate::app::api_support::TelegramMessageListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let items = match telegram_provider_runtime_service(&state)?
        .pinned_messages(&conversation_id, limit)
        .await
    {
        Ok(items) => items,
        Err(TelegramError::InvalidRequest(_)) => {
            canonical_pinned_messages(&state, &conversation_id, limit).await?
        }
        Err(error) => return Err(error.into()),
    };

    Ok(Json(crate::app::api_support::TelegramMessageListResponse {
        items,
    }))
}

/// GET /api/v1/communications/search/media?account_id=&provider_chat_id=&kind=&limit=
pub(crate) async fn search_telegram_media(
    State(state): State<AppState>,
    Query(query): Query<TelegramMediaSearchQuery>,
) -> Result<Json<TelegramMediaSearchResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let channel_kinds = search_channel_kinds(query.channel_kind.as_deref());
    let search_q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let messages = ProviderChannelMessageStore::new(pool.clone())
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            channel_kinds,
            limit,
        )
        .await
        .map_err(TelegramError::from)?
        .into_iter()
        .map(provider_channel_message_to_search_message)
        .collect::<Vec<_>>();

    let mut items = Vec::new();
    for msg in &messages {
        if let Some(arr) = msg
            .metadata
            .get("attachments")
            .or(msg.metadata.get("files"))
            .and_then(|v| v.as_array())
        {
            for att in arr {
                let kind = att
                    .get("attachment_type")
                    .or(att.get("kind"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("file");
                if query.kind.as_deref().is_some_and(|fk| kind != fk) {
                    continue;
                }
                let file_name = att
                    .get("filename")
                    .or(att.get("file_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                let mime_type = att
                    .get("content_type")
                    .or(att.get("mime_type"))
                    .and_then(|v| v.as_str())
                    .map(ToOwned::to_owned);
                if let Some(search_q) = search_q {
                    let search_q = search_q.to_lowercase();
                    let mut haystack = vec![
                        file_name.to_lowercase(),
                        kind.to_lowercase(),
                        msg.provider_message_id.to_lowercase(),
                    ];
                    if let Some(mime) = mime_type.as_deref() {
                        haystack.push(mime.to_lowercase());
                    }
                    if !haystack.into_iter().any(|value| value.contains(&search_q)) {
                        continue;
                    }
                }
                items.push(TelegramMediaItem {
                    attachment_i
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/provider_runtime_handlers/telegram/topics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/telegram/topics.rs`
- Size bytes / Размер в байтах: `11824`
- Included characters / Включено символов: `11824`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::app::api_support::{
    TelegramMessageListResponse, api_audit_log, telegram_provider_runtime_service,
    telegram_runtime_use_case_context,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramTopic, TelegramTopicCloseRequest, TelegramTopicCreateRequest,
    TelegramTopicLifecycleResponse, TelegramTopicListResponse,
};
use crate::application::telegram_runtime;
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
};

#[derive(Deserialize)]
pub(crate) struct TelegramTopicsQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramTopicSearchQuery {
    pub(crate) q: String,
    pub(crate) telegram_chat_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramTopicListApiResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) items: Vec<TelegramTopic>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramTopicMessagesQuery {
    pub(crate) limit: Option<i64>,
}

fn build_command_event(
    account_id: &str,
    command_id: &str,
    provider_chat_id: &str,
    action: &str,
    status: &str,
    extra: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    let mut payload = json!({
        "command_id": command_id,
        "account_id": account_id,
        "provider_chat_id": provider_chat_id,
        "action": action,
        "status": status,
    });
    if let (Some(payload_obj), Some(extra_obj)) = (payload.as_object_mut(), extra.as_object()) {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_topic_command_{}",
            now.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::COMMAND_STATUS_CHANGED,
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": command_id, "kind": "telegram_command"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

/// GET /api/v1/communications/conversations/{conversation_id}/topics
///
/// Attempts a live TDLib fetch to refresh the topic projection before serving DB rows.
/// Falls back to the DB projection if TDLib is unavailable or the account is in fixture mode.
pub(crate) async fn get_telegram_topics(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Query(query): Query<TelegramTopicsQuery>,
) -> Result<Json<TelegramTopicListApiResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let chat = store
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(
                crate::application::provider_runtime_contracts::TelegramError::InvalidRequest(
                    format!("telegram chat `{telegram_chat_id}` was not found"),
                ),
            )
        })?;
    ensure_telegram_account_operation_allowed(&state, &chat.account_id, "topics.list").await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    if let Err(error) =
        telegram_runtime::refresh_forum_topics(&runtime_context, &telegram_chat_id).await
    {
        tracing::debug!(
            error = %error,
            telegram_chat_id = %telegram_chat_id,
            "get_telegram_topics: TDLib live sync failed, serving DB projection"
        );
    }

    let items = store.list_topics(&telegram_chat_id, limit).await?.items;

    Ok(Json(TelegramTopicListApiResponse {
        telegram_chat_id,
        items,
    }))
}

/// POST /api/v1/communications/conversations/{conversation_id}/topics
pub(crate) async fn post_telegram_topic_create(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramTopicCreateRequest>,
) -> Result<Json<TelegramTopicLifecycleResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "topics.create").await?;
    let store = telegram_provider_runtime_service(&state)?;
    let chat = store
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(
                crate::application::provider_runtime_contracts::TelegramError::InvalidRequest(
                    format!("telegram chat `{telegram_chat_id}` was not found"),
                ),
            )
        })?;
    let command_id = request.command_id.clone();

    store
        .insert_command(
            &command_id,
            &request.account_id,
            "topic_create",
            &format!(
            "topic_create:{}:{}",
            request.provider_chat_id,
            Utc::now().timestamp_millis()
            ),
            &request.provider_chat_id,
            None,
            "available",
            "provider_write",
            "confirmed",
            AUDIT_ACTOR_ID,
            json!({"title": request.title.trim()}),
            json!({"telegram_chat_id": telegram_chat_id, "provider_chat_id": request.provider_chat_id}),
            json!({"source": "telegram_topic_create"}),
        )
        .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_topic_create(
            AUDIT_ACTOR_ID,
            &chat.telegram_chat_id,
            &request.account_id,
            &request.provider_chat_id,
        ))
        .await?;

    publish_telegram_event(
        &state,
        build_command_event(
            &request.account_id,
            &command_id,
            &request.provider_chat_id,
            "topic_create",
            "queued",
            json!({
                "telegram_chat_id": chat.telegram_chat_id,
                "title": request.title.trim(),
            }),
        ),
    )
    .await?;

    Ok(Json(TelegramTopicLifecycleResponse {
        operation: "topic_create".to_owned(),
        topic_id: None,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_topic_id: None,
        status: "queued".to_owned(),
        timestamp: Utc::now(),
        command_id,
    }))
}

/// GET /api/v1/communications/topics/{topic_id}
pub(crate) async fn get_telegram_topic_detail(
    State(state): State<AppState>,
    Path(topic_id): Path<String>,
) -> Result<Json<TelegramTopic>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let topic = store
        .get_topic(&topic_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(topic))
}

/// POST /api/v1/communications/topics/{topic_id}/close
pub(crate) async fn post_telegram_topic_close(
    State(state): State<AppState>,
    Path(topic_id): Path<String>,
    Json(request): Json<TelegramTopicCloseRequest>,
) -> Result<Json<TelegramTopicLifecycleResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "topics.close").await?;
    let store = telegram_provider_runtime_service(&state)?;
    let topic = store
        .get_topic(&topic_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let command_kind = if request.is_closed {
        "topic_close"
    } else {
        "topic_reopen"
    };
    let command_id = request.command_id.clone();

    store
        .insert_command(
            &command_id,
            &request.account_id,
            command_kind,
            &format!(
                "{command_kind}:{}:{}",
                topic.provider_topic_id,
                Utc::now().timestamp_millis()
            ),
            &request.provider_chat_id,
            None,
            "available",
            "provider_write",
            "confirmed",
            AUDIT_ACTOR_ID,
            json!({
            "provider_topic_id": topic.provider_topic_id,
            "is_closed": request.is_closed,
            }),
            json!({
            "topic_id": topic.topic_id,
            "telegram_chat_id": topic.telegram_chat_id,
            "provider_chat_id": topic.provider_chat_id,
            "provider_topic_id": topic.provider_topic_id,
            }),
            json!({"source": "telegram_topic_close"}),
        )
        .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_topic_close(
            AUDIT_ACTOR_ID,
            &topic.topic_id,
            &request.account_id,
            &request.provider_chat_id,
            request.is_closed,
        ))
        .await?;

    publish_telegram_event(
        &state,
        build_command_event(
            &request.account_id,
            &command_id,
            &request.provider_chat_id,
            command_kind,
            "queued",
            json!({
                "topic_id": topic.topic_id,
                "telegram_chat_id": topic.telegram_chat_id,
                "provider_topic_id": topic.provider_topic_id,
                "is_closed": request.is_closed,
            }),
        ),
    )
    .await?;

    Ok(Json(TelegramTopicLifecycleResponse {
        operation: command_kind.to_owned(),
        topic_id: Some(topic.topic_id),
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_topic_id: Some(topic.provider_topic_id),
        status: "queued".to_owned(),
        timestamp: Utc::now(),
        command_id,
    }))
}

/// GET /api/v1/communications/topics/{topic_id}/messages
/// Returns messages whose metadata.forum_topic_id matches topic_id.
pub(crate) async fn get_telegram_topic_messages(
    State(state): State<AppState>,
    Path(topic_id): Path<String>,
    Query(query): Query<TelegramTopicMessagesQuery>,
) -> Result<Json<TelegramMessageListResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    let message_ids = store.list_topic_message_ids(&topic_id, limit).await?;

    if message_ids.is_empty() {
        return Ok(Json(TelegramMessageListResponse { items: vec![] }));
    }

    let items = store.messages_by_ids(&message_ids).await?;

    Ok(Json(TelegramMessageListResponse { items }))
}

/// GET /api/v1/communications/topics/search?q=&telegram_chat_id=&limit=
pub(crate) async fn search_telegram_topics(
    State(state): State<AppState>,
    Query(query): Query<TelegramTopicSearchQuery>,
) -> Result<Json<TelegramTopicListApiResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query.q.trim().to_owned();
    let telegram_chat_id = query.telegram_chat_id.trim().to_owned();

    if search_q.is_empty() {
        return Err(ApiError::Telegram(
            crate::application::provider_runtime_contracts::TelegramError::InvalidRequest(
                "search query `q` is required".to_owned(),
            ),
        ));
    }

    if telegram_chat_id.is_empty() {
        return Err(ApiError::Telegram(
            crate::application::provider_runtime_contracts::TelegramError::InvalidRequest(
                "search query `telegram_chat_id` is required".to_owned(),
            ),
        ));
    }

    let items = store
        .search_topics(&telegram_chat_id, &search_q, limit)
        .await?;

    Ok(Json(TelegramTopicListApiResponse {
        telegram_chat_id,
        items,
    }))
}
```
