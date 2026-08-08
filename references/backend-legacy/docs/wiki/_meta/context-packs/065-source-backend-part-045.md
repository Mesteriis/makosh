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

- Chunk ID / ID чанка: `065-source-backend-part-045`
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

### `backend/src/integrations/telegram/runtime/state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/state.rs`
- Size bytes / Размер в байтах: `8120`
- Included characters / Включено символов: `8120`
- Truncated / Обрезано: `no`

```rust
use std::sync::mpsc::Sender;

use chrono::{DateTime, Utc};

use crate::integrations::telegram::client::{TelegramError, TelegramManualSendRequest};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatMarkedAsUnreadSnapshot,
    TelegramTdlibChatMemberSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatRemovedFromListSnapshot,
    TelegramTdlibChatSnapshot, TelegramTdlibChatUnreadSnapshot, TelegramTdlibFileSnapshot,
    TelegramTdlibMessageContentSnapshot, TelegramTdlibMessageDeleteSnapshot,
    TelegramTdlibMessageEditedSnapshot, TelegramTdlibMessageInteractionInfoSnapshot,
    TelegramTdlibMessagePinnedSnapshot, TelegramTdlibMessageSnapshot, TelegramTdlibTopicSnapshot,
    TelegramTdlibTopicUpdateSnapshot, TelegramTdlibTypingSnapshot,
};

use super::models::{TelegramHistorySyncMode, TelegramMediaSendRequest};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TelegramRuntimeState {
    Stopped,
    Running,
    Blocked,
    Degraded,
    Error,
}

impl TelegramRuntimeState {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Running => "running",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TelegramRuntimeActorState {
    pub(super) status: TelegramRuntimeState,
    pub(super) last_error: Option<String>,
    pub(super) updated_at: DateTime<Utc>,
}

impl TelegramRuntimeActorState {
    pub(super) fn with_command(
        self,
        command_tx: Sender<TelegramRuntimeCommand>,
    ) -> (
        TelegramRuntimeActorState,
        Option<Sender<TelegramRuntimeCommand>>,
    ) {
        (self, Some(command_tx))
    }

    pub(super) fn without_command(
        self,
    ) -> (
        TelegramRuntimeActorState,
        Option<Sender<TelegramRuntimeCommand>>,
    ) {
        (self, None)
    }
}

#[derive(Clone)]
pub(super) struct TelegramRuntimeActorHandle {
    pub(super) state: TelegramRuntimeActorState,
    pub(super) command_tx: Option<Sender<TelegramRuntimeCommand>>,
}

pub(super) enum TelegramRuntimeCommand {
    LoadChats {
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatSnapshot>, TelegramError>>,
    },
    GetChatFolders {
        folder_ids: Vec<i64>,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatFolderSnapshot>, TelegramError>>,
    },
    SyncHistory {
        provider_chat_id: String,
        from_message_id: Option<i64>,
        limit: i32,
        mode: TelegramHistorySyncMode,
        reply_tx: Sender<Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError>>,
    },
    SendText {
        request: TelegramManualSendRequest,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
    SendMedia {
        request: TelegramMediaSendRequest,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
    DownloadFile {
        file_id: i64,
        priority: i32,
        reply_tx: Sender<Result<TelegramTdlibFileSnapshot, TelegramError>>,
    },
    EditMessage {
        provider_chat_id: String,
        provider_message_id: String,
        new_text: String,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    DeleteMessage {
        provider_chat_id: String,
        provider_message_id: String,
        revoke: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    SetReaction {
        provider_chat_id: String,
        provider_message_id: String,
        reaction_emoji: String,
        is_active: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    PinMessage {
        provider_chat_id: String,
        provider_message_id: String,
        pin: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    ToggleChatUnread {
        provider_chat_id: String,
        is_marked_as_unread: bool,
        read_through_provider_message_id: Option<String>,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    ToggleChatArchive {
        provider_chat_id: String,
        archived: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    ToggleChatMute {
        provider_chat_id: String,
        muted: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    AddChatToFolder {
        provider_chat_id: String,
        provider_folder_id: i64,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    RemoveChatFromFolder {
        provider_chat_id: String,
        provider_folder_id: i64,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    JoinChat {
        provider_chat_id: String,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    LeaveChat {
        provider_chat_id: String,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    ReplyMessage {
        provider_chat_id: String,
        reply_to_provider_message_id: String,
        text: String,
        command_id: String,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
    ForwardMessage {
        provider_chat_id: String,
        from_provider_chat_id: String,
        from_provider_message_id: String,
        command_id: String,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
    GetForumTopics {
        provider_chat_id: String,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibTopicSnapshot>, TelegramError>>,
    },
    CreateForumTopic {
        provider_chat_id: String,
        title: String,
        command_id: String,
        reply_tx: Sender<Result<TelegramTdlibTopicSnapshot, TelegramError>>,
    },
    ToggleForumTopicClosed {
        provider_chat_id: String,
        provider_topic_id: i64,
        is_closed: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    GetSupergroupMembers {
        supergroup_id: i64,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError>>,
    },
    GetSupergroupAdministrators {
        supergroup_id: i64,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError>>,
    },
    GetBasicGroupMembers {
        basic_group_id: i64,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError>>,
    },
    SearchMessages {
        query: String,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError>>,
    },
    SearchChatMessages {
        provider_chat_id: String,
        query: String,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum TelegramRuntimeEvent {
    MessageCreated(TelegramTdlibMessageSnapshot),
    MessageContentUpdated(TelegramTdlibMessageContentSnapshot),
    MessageEdited(TelegramTdlibMessageEditedSnapshot),
    MessagePinnedUpdated(TelegramTdlibMessagePinnedSnapshot),
    MessageDeleted(TelegramTdlibMessageDeleteSnapshot),
    MessageInteractionInfoUpdated(TelegramTdlibMessageInteractionInfoSnapshot),
    TypingChanged(TelegramTdlibTypingSnapshot),
    TopicUpdated(TelegramTdlibTopicUpdateSnapshot),
    ChatUnreadUpdated(TelegramTdlibChatUnreadSnapshot),
    ChatMarkedAsUnreadUpdated(TelegramTdlibChatMarkedAsUnreadSnapshot),
    ChatNotificationSettingsUpdated(TelegramTdlibChatNotificationSettingsSnapshot),
    ChatPositionUpdated(TelegramTdlibChatPositionSnapshot),
    ChatRemovedFromList(TelegramTdlibChatRemovedFromListSnapshot),
    ChatFoldersUpdated(Vec<TelegramTdlibChatFolderSnapshot>),
}
```

### `backend/src/integrations/telegram/runtime/status.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/status.rs`
- Size bytes / Размер в байтах: `5441`
- Included characters / Включено символов: `5441`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::TdJsonLibrary;
use crate::platform::communications::{
    CommunicationProviderKind, ProviderAccount, ProviderAccountLookupPort,
};
use crate::platform::config::AppConfig;

use super::models::TelegramRuntimeStatus;
use super::state::{TelegramRuntimeActorState, TelegramRuntimeState};
use super::validation::validate_non_empty;

pub(super) async fn load_telegram_account(
    provider_account_store: &dyn ProviderAccountLookupPort,
    account_id: &str,
) -> Result<ProviderAccount, TelegramError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let account = provider_account_store
        .get(&account_id)
        .await
        .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram account `{account_id}` is not configured"
            ))
        })?;

    if !account.provider_kind.is_telegram() {
        return Err(TelegramError::InvalidRequest(format!(
            "account `{}` is not a Telegram provider account",
            account.account_id
        )));
    }

    Ok(account)
}

pub(super) fn status_from_account(
    config: &AppConfig,
    account: &ProviderAccount,
    actor_state: Option<TelegramRuntimeActorState>,
) -> TelegramRuntimeStatus {
    let runtime_kind = account_runtime_kind(account);
    let default_state = default_state_for_runtime(&runtime_kind);
    let actor_state = actor_state.unwrap_or(default_state);
    let tdjson_path = config.tdjson_path().map(|path| path.display().to_string());
    let (tdjson_runtime_available, tdjson_probe_error) = tdjson_probe(config.tdjson_path());
    let telegram_api_id_configured = config.telegram_api_id().is_some();
    let telegram_api_hash_configured = config.telegram_api_hash().is_some();
    let telegram_app_credentials_configured =
        telegram_api_id_configured && telegram_api_hash_configured;
    let live_send_available = runtime_kind == "tdlib_qr_authorized"
        && actor_state.status == TelegramRuntimeState::Running
        && tdjson_runtime_available
        && telegram_app_credentials_configured;
    let runtime_blockers = runtime_blockers(
        &runtime_kind,
        tdjson_runtime_available,
        telegram_api_id_configured,
        telegram_api_hash_configured,
        actor_state.last_error.as_deref(),
    );

    TelegramRuntimeStatus {
        account_id: account.account_id.clone(),
        provider_kind: account.provider_kind.as_str().to_owned(),
        runtime_kind: runtime_kind.clone(),
        status: actor_state.status.as_str().to_owned(),
        fixture_runtime: runtime_kind == "fixture",
        tdjson_path,
        tdjson_runtime_available,
        tdjson_probe_error,
        telegram_api_id_configured,
        telegram_api_hash_configured,
        telegram_app_credentials_configured,
        live_send_available,
        runtime_blockers,
        last_error: actor_state.last_error,
        updated_at: actor_state.updated_at,
    }
}

fn tdjson_probe(configured_path: Option<&std::path::Path>) -> (bool, Option<String>) {
    match TdJsonLibrary::load(configured_path) {
        Ok(_) => (true, None),
        Err(error) => (false, Some(error.to_string())),
    }
}

fn runtime_blockers(
    runtime_kind: &str,
    tdjson_runtime_available: bool,
    telegram_api_id_configured: bool,
    telegram_api_hash_configured: bool,
    last_error: Option<&str>,
) -> Vec<String> {
    let mut blockers = Vec::new();

    if runtime_kind == "live_blocked" {
        blockers.push("live_tdlib_runtime_blocked".to_owned());
    }
    if runtime_kind == "tdlib_qr_authorized" && !tdjson_runtime_available {
        blockers.push("tdjson_runtime_unavailable".to_owned());
    }
    if runtime_kind == "tdlib_qr_authorized" && !telegram_api_id_configured {
        blockers.push("telegram_api_id_missing".to_owned());
    }
    if runtime_kind == "tdlib_qr_authorized" && !telegram_api_hash_configured {
        blockers.push("telegram_api_hash_missing".to_owned());
    }
    if runtime_kind == "fixture" {
        return blockers;
    }
    if let Some(error) = last_error
        && !error.trim().is_empty()
    {
        blockers.push(error.trim().to_owned());
    }

    blockers
}

fn default_state_for_runtime(runtime_kind: &str) -> TelegramRuntimeActorState {
    let now = Utc::now();
    match runtime_kind {
        "live_blocked" => TelegramRuntimeActorState {
            status: TelegramRuntimeState::Blocked,
            last_error: Some("account runtime is blocked until live TDLib is enabled".to_owned()),
            updated_at: now,
        },
        _ => TelegramRuntimeActorState {
            status: TelegramRuntimeState::Stopped,
            last_error: None,
            updated_at: now,
        },
    }
}

pub(super) fn account_runtime_kind(account: &ProviderAccount) -> String {
    account
        .config
        .get("runtime")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(match account.provider_kind {
            CommunicationProviderKind::TelegramUser | CommunicationProviderKind::TelegramBot => {
                "unknown"
            }
            _ => "unsupported",
        })
        .to_owned()
}
```

### `backend/src/integrations/telegram/runtime/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/tests.rs`
- Size bytes / Размер в байтах: `1187`
- Included characters / Включено символов: `1187`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::*;

#[test]
fn history_sync_request_accepts_older_cursor() {
    let request: TelegramHistorySyncRequest = serde_json::from_value(json!({
        "account_id": "telegram-primary",
        "provider_chat_id": "-100123456789",
        "from_message_id": 987654321,
        "mode": "older",
        "limit": 100
    }))
    .expect("history request");

    request.validate().expect("valid history request");
    assert_eq!(request.mode(), TelegramHistorySyncMode::Older);
    assert_eq!(request.from_message_id, Some(987654321));
}

#[test]
fn history_sync_response_exposes_next_cursor() {
    let response = TelegramHistorySyncResponse {
        account_id: "telegram-primary".to_owned(),
        provider_chat_id: "-100123456789".to_owned(),
        runtime_kind: "tdlib_qr_authorized".to_owned(),
        status: "synced".to_owned(),
        synced_count: 100,
        has_more: true,
        next_from_message_id: Some(12345),
        items: Vec::new(),
    };

    let value = serde_json::to_value(response).expect("serialized response");
    assert_eq!(value["has_more"], json!(true));
    assert_eq!(value["next_from_message_id"], json!(12345));
}
```

### `backend/src/integrations/telegram/runtime/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/validation.rs`
- Size bytes / Размер в байтах: `631`
- Included characters / Включено символов: `631`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, TelegramError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn validate_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=100).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}
```

### `backend/src/integrations/telegram/tdjson.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson.rs`
- Size bytes / Размер в байтах: `3949`
- Included characters / Включено символов: `3949`
- Truncated / Обрезано: `no`

```rust
mod client;
mod folder_requests;
mod identifiers;
mod library_paths;
mod parsing;
mod qr_login;
mod qr_login_support;
mod requests;
mod snapshots;

pub(crate) use self::client::{TdJsonClient, TdJsonLibrary, runtime_available};
pub(crate) use self::folder_requests::tdlib_edit_chat_folder_remove_chat_request;
pub(crate) use self::parsing::{
    TelegramTdlibChatMarkedAsUnreadSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatRemovedFromListSnapshot,
    TelegramTdlibChatUnreadSnapshot, TelegramTdlibTopicUpdateSnapshot, TelegramTdlibTypingSnapshot,
    authorization_state, is_tdlib_database_encryption_key_needed_error,
    is_tdlib_parameters_not_specified_error, parse_tdlib_basic_group_member_list,
    parse_tdlib_chat_folder_snapshot, parse_tdlib_chat_folders_update_snapshot,
    parse_tdlib_chat_ids, parse_tdlib_chat_marked_as_unread_snapshot, parse_tdlib_chat_member_list,
    parse_tdlib_chat_notification_settings_snapshot, parse_tdlib_chat_position_snapshot,
    parse_tdlib_chat_removed_from_list_snapshot, parse_tdlib_chat_snapshot,
    parse_tdlib_chat_unread_snapshot, parse_tdlib_created_forum_topic, parse_tdlib_file_snapshot,
    parse_tdlib_message_content_snapshot, parse_tdlib_message_delete_snapshot,
    parse_tdlib_message_edited_snapshot, parse_tdlib_message_interaction_info_snapshot,
    parse_tdlib_message_list, parse_tdlib_message_pinned_snapshot, parse_tdlib_message_snapshot,
    parse_tdlib_new_message_snapshot, parse_tdlib_topic_list, parse_tdlib_topic_update_snapshot,
    parse_tdlib_typing_snapshot, tdlib_error_message,
};
pub(crate) use self::qr_login::{cancel_qr_login, start_qr_login, submit_qr_login_password};
pub(crate) use self::qr_login_support::{PendingQrLoginMap, TelegramQrLoginSession};
pub(crate) use self::requests::{
    check_database_encryption_key_request, set_tdlib_parameters_request,
    tdlib_add_chat_to_folder_request, tdlib_add_chat_to_list_request,
    tdlib_add_message_reaction_request, tdlib_create_forum_topic_request, tdlib_database_directory,
    tdlib_delete_messages_request, tdlib_download_file_request, tdlib_edit_message_text_request,
    tdlib_get_basic_group_full_info_request, tdlib_get_basic_group_request,
    tdlib_get_chat_folder_request, tdlib_get_chat_history_request, tdlib_get_chat_request,
    tdlib_get_chats_request, tdlib_get_forum_topics_request,
    tdlib_get_supergroup_administrators_request, tdlib_get_supergroup_members_request,
    tdlib_join_chat_request, tdlib_leave_chat_request, tdlib_load_chats_request,
    tdlib_pin_chat_message_request, tdlib_remove_message_reaction_request,
    tdlib_search_chat_messages_request, tdlib_search_messages_request, tdlib_send_forward_request,
    tdlib_send_media_message_request, tdlib_send_reply_request, tdlib_send_text_message_request,
    tdlib_set_chat_mute_request, tdlib_toggle_chat_marked_as_unread_request,
    tdlib_toggle_forum_topic_is_closed_request, tdlib_unpin_chat_message_request,
    tdlib_view_messages_request,
};
pub(crate) use self::snapshots::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatMemberSnapshot, TelegramTdlibChatSnapshot,
    TelegramTdlibFileSnapshot, TelegramTdlibMessageContentSnapshot,
    TelegramTdlibMessageDeleteSnapshot, TelegramTdlibMessageEditedSnapshot,
    TelegramTdlibMessageInteractionInfoSnapshot, TelegramTdlibMessagePinnedSnapshot,
    TelegramTdlibMessageSnapshot, TelegramTdlibTopicSnapshot,
};

#[cfg(test)]
use self::library_paths::{tdjson_library_candidates_with_context, tdjson_platform_dir};
#[cfg(test)]
use self::qr_login::cancel_existing_qr_logins_for_account;
#[cfg(test)]
use self::qr_login_support::{
    TelegramQrLoginCommand, TelegramQrLoginIdentity, mark_worker_complete, new_worker_completion,
    parse_tdlib_user_identity, password_waiting_response, qr_preparing_response, ready_response,
    render_qr_svg, state_allows_qr_request,
};

#[cfg(test)]
mod tests;
```

### `backend/src/integrations/telegram/tdjson/client.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/client.rs`
- Size bytes / Размер в байтах: `7506`
- Included characters / Включено символов: `7506`
- Truncated / Обрезано: `no`

```rust
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::Path;

use libloading::Library;
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;

use super::library_paths::tdjson_library_candidates;

type TdJsonClientCreate = unsafe extern "C" fn() -> *mut c_void;
type TdJsonClientSend = unsafe extern "C" fn(*mut c_void, *const c_char);
type TdJsonClientReceive = unsafe extern "C" fn(*mut c_void, f64) -> *const c_char;
type TdJsonClientExecute = unsafe extern "C" fn(*mut c_void, *const c_char) -> *const c_char;
type TdJsonClientDestroy = unsafe extern "C" fn(*mut c_void);

pub(crate) fn runtime_available(configured_path: Option<&Path>) -> bool {
    TdJsonLibrary::load(configured_path).is_ok()
}

pub(crate) struct TdJsonLibrary {
    create: TdJsonClientCreate,
    send: TdJsonClientSend,
    receive: TdJsonClientReceive,
    execute: TdJsonClientExecute,
    destroy: TdJsonClientDestroy,
    _library: Library,
}

impl TdJsonLibrary {
    pub(crate) fn load(configured_path: Option<&Path>) -> Result<Self, TelegramError> {
        let candidates = tdjson_library_candidates(configured_path);
        let mut load_errors = Vec::new();

        for candidate in candidates {
            let library = {
                // SAFETY: Loading a dynamic library is unsafe because the symbols may not
                // match the expected ABI. Symbols are verified immediately below and kept
                // alive by storing the Library inside TdJsonLibrary.
                unsafe { Library::new(&candidate) }
            };
            match library {
                Ok(library) => return Self::from_library(library, &candidate),
                Err(error) => {
                    load_errors.push(format!("{}: {error}", candidate.display()));
                    if configured_path.is_some() {
                        break;
                    }
                }
            }
        }

        Err(TelegramError::TdlibRuntimeUnavailable(format!(
            "unable to load libtdjson; tried {}",
            load_errors.join("; ")
        )))
    }

    fn from_library(library: Library, candidate: &Path) -> Result<Self, TelegramError> {
        let create = load_symbol(&library, b"td_json_client_create\0", candidate)?;
        let send = load_symbol(&library, b"td_json_client_send\0", candidate)?;
        let receive = load_symbol(&library, b"td_json_client_receive\0", candidate)?;
        let execute = load_symbol(&library, b"td_json_client_execute\0", candidate)?;
        let destroy = load_symbol(&library, b"td_json_client_destroy\0", candidate)?;

        Ok(Self {
            create,
            send,
            receive,
            execute,
            destroy,
            _library: library,
        })
    }

    pub(crate) fn create_client(self) -> Result<TdJsonClient, TelegramError> {
        let client = {
            // SAFETY: The function pointer was loaded from libtdjson with the documented
            // C ABI and returns an opaque TDLib client pointer owned by the caller.
            unsafe { (self.create)() }
        };
        if client.is_null() {
            return Err(TelegramError::TdlibRuntime(
                "td_json_client_create returned null".to_owned(),
            ));
        }

        Ok(TdJsonClient {
            client,
            library: self,
        })
    }
}

pub(crate) struct TdJsonClient {
    client: *mut c_void,
    library: TdJsonLibrary,
}

impl TdJsonClient {
    pub(crate) fn send_json(&self, request: &Value) -> Result<(), TelegramError> {
        let request = CString::new(request.to_string()).map_err(|_| {
            TelegramError::TdlibRuntime("TDLib request contained an interior NUL byte".to_owned())
        })?;
        // SAFETY: The client pointer is created by td_json_client_create and remains
        // valid until Drop calls td_json_client_destroy. CString is NUL-terminated
        // and lives for the duration of the call.
        unsafe {
            (self.library.send)(self.client, request.as_ptr());
        }
        Ok(())
    }

    pub(crate) fn receive_json(
        &self,
        timeout_seconds: f64,
    ) -> Result<Option<Value>, TelegramError> {
        let response = {
            // SAFETY: TDLib owns the returned pointer until the next receive/execute
            // call on this client. The string is copied into an owned Rust String
            // before another TDLib call can invalidate it.
            unsafe { (self.library.receive)(self.client, timeout_seconds) }
        };
        if response.is_null() {
            return Ok(None);
        }

        let response = {
            // SAFETY: td_json_client_receive returns a NUL-terminated UTF-8 JSON string
            // pointer or NULL. NULL was handled above.
            unsafe { CStr::from_ptr(response) }
        }
        .to_str()
        .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON UTF-8: {error}")))?
        .to_owned();

        serde_json::from_str(&response)
            .map(Some)
            .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON: {error}")))
    }

    pub(crate) fn execute_json(&self, request: &Value) -> Result<Option<Value>, TelegramError> {
        let request = CString::new(request.to_string()).map_err(|_| {
            TelegramError::TdlibRuntime("TDLib request contained an interior NUL byte".to_owned())
        })?;
        let response = {
            // SAFETY: The client pointer and request CString satisfy td_json_client_execute
            // requirements. The returned pointer is copied before the next TDLib call.
            unsafe { (self.library.execute)(self.client, request.as_ptr()) }
        };
        if response.is_null() {
            return Ok(None);
        }

        let response = {
            // SAFETY: td_json_client_execute returns a NUL-terminated JSON string pointer
            // or NULL. NULL was handled above.
            unsafe { CStr::from_ptr(response) }
        }
        .to_str()
        .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON UTF-8: {error}")))?
        .to_owned();

        serde_json::from_str(&response)
            .map(Some)
            .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON: {error}")))
    }
}

impl Drop for TdJsonClient {
    fn drop(&mut self) {
        if !self.client.is_null() {
            // SAFETY: The pointer was created by td_json_client_create and is destroyed
            // exactly once here, before the backing library is unloaded.
            unsafe {
                (self.library.destroy)(self.client);
            }
            self.client = std::ptr::null_mut();
        }
    }
}

fn load_symbol<T: Copy>(
    library: &Library,
    name: &'static [u8],
    candidate: &Path,
) -> Result<T, TelegramError> {
    let symbol = {
        // SAFETY: Symbol type T is the exact C ABI function pointer expected for the
        // named TDLib JSON symbol. The Library is retained for at least as long as
        // copied function pointers can be called.
        unsafe { library.get::<T>(name) }
    }
    .map_err(|error| {
        let symbol_name = name.strip_suffix(b"\0").unwrap_or(name);
        TelegramError::TdlibRuntimeUnavailable(format!(
            "libtdjson `{}` is missing symbol `{}`: {error}",
            candidate.display(),
            String::from_utf8_lossy(symbol_name)
        ))
    })?;
    Ok(*symbol)
}
```

### `backend/src/integrations/telegram/tdjson/folder_requests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/folder_requests.rs`
- Size bytes / Размер в байтах: `3752`
- Included characters / Включено символов: `3752`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use crate::integrations::telegram::client::TelegramError;

fn non_empty_trimmed_text(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("")
        .to_owned()
}

fn bool_field(folder: &Value, key: &str) -> bool {
    folder.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn i64_field(folder: &Value, key: &str) -> i64 {
    folder.get(key).and_then(Value::as_i64).unwrap_or_default()
}

fn unique_chat_ids(values: &[i64]) -> Vec<i64> {
    let mut result = Vec::with_capacity(values.len());
    for value in values {
        if !result.contains(value) {
            result.push(*value);
        }
    }
    result
}

fn folder_chat_ids(folder: &Value, key: &str) -> Vec<i64> {
    let ids = folder
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_i64)
        .collect::<Vec<_>>();
    unique_chat_ids(&ids)
}

pub(crate) fn tdlib_edit_chat_folder_remove_chat_request(
    chat_folder_id: i64,
    chat_id: i64,
    folder: &Value,
    extra: &str,
) -> Result<Value, TelegramError> {
    if folder.get("@type").and_then(Value::as_str) != Some("chatFolder") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib getChatFolder response is missing chatFolder payload".to_owned(),
        ));
    }

    let mut pinned_chat_ids = folder_chat_ids(folder, "pinned_chat_ids");
    pinned_chat_ids.retain(|value| *value != chat_id);

    let mut included_chat_ids = folder_chat_ids(folder, "included_chat_ids");
    included_chat_ids.retain(|value| *value != chat_id);

    let mut excluded_chat_ids = folder_chat_ids(folder, "excluded_chat_ids");
    if !excluded_chat_ids.contains(&chat_id) {
        excluded_chat_ids.push(chat_id);
    }

    Ok(json!({
        "@type": "editChatFolder",
        "chat_folder_id": chat_folder_id,
        "folder": {
            "@type": "chatFolder",
            "name": {
                "@type": "chatFolderName",
                "text": non_empty_trimmed_text(
                    folder
                        .get("name")
                        .and_then(|value| value.get("text"))
                        .and_then(Value::as_str)
                ),
                "animate_custom_emoji": folder
                    .get("name")
                    .and_then(|value| value.get("animate_custom_emoji"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            },
            "icon": {
                "@type": "chatFolderIcon",
                "name": non_empty_trimmed_text(
                    folder
                        .get("icon")
                        .and_then(|value| value.get("name"))
                        .and_then(Value::as_str)
                ),
            },
            "color_id": i64_field(folder, "color_id"),
            "is_shareable": bool_field(folder, "is_shareable"),
            "pinned_chat_ids": pinned_chat_ids,
            "included_chat_ids": included_chat_ids,
            "excluded_chat_ids": excluded_chat_ids,
            "exclude_muted": bool_field(folder, "exclude_muted"),
            "exclude_read": bool_field(folder, "exclude_read"),
            "exclude_archived": bool_field(folder, "exclude_archived"),
            "include_contacts": bool_field(folder, "include_contacts"),
            "include_non_contacts": bool_field(folder, "include_non_contacts"),
            "include_bots": bool_field(folder, "include_bots"),
            "include_groups": bool_field(folder, "include_groups"),
            "include_channels": bool_field(folder, "include_channels"),
        },
        "@extra": extra.trim(),
    }))
}
```

### `backend/src/integrations/telegram/tdjson/identifiers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/identifiers.rs`
- Size bytes / Размер в байтах: `467`
- Included characters / Включено символов: `467`
- Truncated / Обрезано: `no`

```rust
pub(super) fn safe_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();

    if sanitized.is_empty() {
        "account".to_owned()
    } else {
        sanitized
    }
}
```

### `backend/src/integrations/telegram/tdjson/library_paths.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/library_paths.rs`
- Size bytes / Размер в байтах: `5319`
- Included characters / Включено символов: `5319`
- Truncated / Обрезано: `no`

```rust
use std::env;
use std::path::{Path, PathBuf};

pub(super) fn tdjson_library_candidates(configured_path: Option<&Path>) -> Vec<PathBuf> {
    let current_exe_dir = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let current_dir = env::current_dir().ok();

    tdjson_library_candidates_with_context(
        configured_path,
        current_exe_dir.as_deref(),
        current_dir.as_deref(),
    )
}

pub(super) fn tdjson_library_candidates_with_context(
    configured_path: Option<&Path>,
    current_exe_dir: Option<&Path>,
    current_dir: Option<&Path>,
) -> Vec<PathBuf> {
    if let Some(path) = configured_path {
        return vec![path.to_path_buf()];
    }

    let mut candidates = Vec::new();
    add_bundled_tdjson_candidates(&mut candidates, current_exe_dir, current_dir);

    #[cfg(target_os = "macos")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/opt/homebrew/opt/tdlib/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/opt/homebrew/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/opt/tdlib/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/lib/libtdjson.dylib"),
        );
    }
    #[cfg(target_os = "linux")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/lib/libtdjson.so"),
        );
        push_unique_candidate(&mut candidates, PathBuf::from("/usr/lib/libtdjson.so"));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/lib/x86_64-linux-gnu/libtdjson.so"),
        );
    }
    #[cfg(target_os = "windows")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
    }

    candidates
}

fn add_bundled_tdjson_candidates(
    candidates: &mut Vec<PathBuf>,
    current_exe_dir: Option<&Path>,
    current_dir: Option<&Path>,
) {
    let library_file_name = tdjson_library_file_name();
    let platform_dir = tdjson_platform_dir();

    if let Some(exe_dir) = current_exe_dir {
        #[cfg(target_os = "macos")]
        if let Some(contents_dir) = exe_dir.parent() {
            add_tdjson_resource_dir_candidates(
                candidates,
                &contents_dir.join("Resources").join("tdlib"),
                platform_dir,
                library_file_name,
            );
        }

        add_tdjson_resource_dir_candidates(
            candidates,
            &exe_dir.join("resources").join("tdlib"),
            platform_dir,
            library_file_name,
        );
        add_tdjson_resource_dir_candidates(
            candidates,
            &exe_dir.join("tdlib"),
            platform_dir,
            library_file_name,
        );
    }

    if let Some(current_dir) = current_dir {
        add_tdjson_resource_dir_candidates(
            candidates,
            &current_dir.join("frontend/src-tauri/resources/tdlib"),
            platform_dir,
            library_file_name,
        );
        add_tdjson_resource_dir_candidates(
            candidates,
            &current_dir.join("resources/tdlib"),
            platform_dir,
            library_file_name,
        );
    }
}

fn add_tdjson_resource_dir_candidates(
    candidates: &mut Vec<PathBuf>,
    tdlib_dir: &Path,
    platform_dir: &str,
    library_file_name: &str,
) {
    push_unique_candidate(
        candidates,
        tdlib_dir.join(platform_dir).join(library_file_name),
    );

    #[cfg(target_os = "macos")]
    push_unique_candidate(
        candidates,
        tdlib_dir.join("macos-universal").join(library_file_name),
    );

    push_unique_candidate(candidates, tdlib_dir.join(library_file_name));
}

fn push_unique_candidate(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

pub(super) fn tdjson_platform_dir() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "macos-arm64";
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "macos-x64";
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return "linux-x64";
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return "linux-arm64";
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return "windows-x64";
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        return "windows-arm64";
    }
    #[allow(unreachable_code)]
    "unknown"
}

fn tdjson_library_file_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        return "libtdjson.dylib";
    }
    #[cfg(target_os = "linux")]
    {
        return "libtdjson.so";
    }
    #[cfg(target_os = "windows")]
    {
        return "tdjson.dll";
    }
    #[allow(unreachable_code)]
    "libtdjson"
}
```

### `backend/src/integrations/telegram/tdjson/parsing.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing.rs`
- Size bytes / Размер в байтах: `1596`
- Included characters / Включено символов: `1596`
- Truncated / Обрезано: `no`

```rust
mod chats;
mod events;
mod files;
mod message_events;
mod message_parts;
mod messages;
mod participants;
mod topics;
mod values;

pub(crate) use chats::{parse_tdlib_chat_ids, parse_tdlib_chat_snapshot};
pub(crate) use events::{
    TelegramTdlibChatMarkedAsUnreadSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatRemovedFromListSnapshot,
    TelegramTdlibChatUnreadSnapshot, TelegramTdlibTopicUpdateSnapshot, TelegramTdlibTypingSnapshot,
    authorization_state, is_tdlib_database_encryption_key_needed_error,
    is_tdlib_parameters_not_specified_error, parse_tdlib_chat_folder_snapshot,
    parse_tdlib_chat_folders_update_snapshot, parse_tdlib_chat_marked_as_unread_snapshot,
    parse_tdlib_chat_notification_settings_snapshot, parse_tdlib_chat_position_snapshot,
    parse_tdlib_chat_removed_from_list_snapshot, parse_tdlib_chat_unread_snapshot,
    parse_tdlib_topic_update_snapshot, parse_tdlib_typing_snapshot, tdlib_error_message,
};
pub(crate) use files::parse_tdlib_file_snapshot;
pub(crate) use message_events::{
    parse_tdlib_message_content_snapshot, parse_tdlib_message_delete_snapshot,
    parse_tdlib_message_edited_snapshot, parse_tdlib_message_interaction_info_snapshot,
    parse_tdlib_message_pinned_snapshot, parse_tdlib_new_message_snapshot,
};
pub(crate) use messages::{parse_tdlib_message_list, parse_tdlib_message_snapshot};
pub(crate) use participants::{parse_tdlib_basic_group_member_list, parse_tdlib_chat_member_list};
pub(crate) use topics::{parse_tdlib_created_forum_topic, parse_tdlib_topic_list};
```

### `backend/src/integrations/telegram/tdjson/parsing/chats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/chats.rs`
- Size bytes / Размер в байтах: `3459`
- Included characters / Включено символов: `3459`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::integrations::telegram::client::{TelegramChatKind, TelegramError};

use super::values::{tdlib_i64_value, tdlib_string_id, tdlib_unix_datetime_value};
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatSnapshot;

pub(crate) fn parse_tdlib_chat_ids(response: &Value) -> Result<Vec<i64>, TelegramError> {
    let chat_ids = response
        .get("chat_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "TDLib getChats response did not include chat_ids".to_owned(),
            )
        })?;

    chat_ids
        .iter()
        .map(tdlib_i64_value)
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) fn parse_tdlib_chat_snapshot(
    chat: &Value,
) -> Result<TelegramTdlibChatSnapshot, TelegramError> {
    if chat.get("@type").and_then(Value::as_str) != Some("chat") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib chat snapshot must have @type=chat".to_owned(),
        ));
    }

    let provider_chat_id = tdlib_string_id(chat, "id")?;
    let chat_kind = tdlib_chat_kind(chat)?;
    let title = chat
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("Telegram Chat {provider_chat_id}"));
    let username = tdlib_username(chat);
    let last_message_at = chat
        .get("last_message")
        .and_then(|message| message.get("date"))
        .map(tdlib_unix_datetime_value)
        .transpose()?;

    Ok(TelegramTdlibChatSnapshot {
        provider_chat_id,
        chat_kind,
        title,
        username,
        last_message_at,
        raw: chat.clone(),
    })
}

fn tdlib_chat_kind(chat: &Value) -> Result<TelegramChatKind, TelegramError> {
    let chat_type = chat
        .get("type")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib chat type is required".to_owned()))?;
    match chat_type {
        "chatTypePrivate" | "chatTypeSecret" => Ok(TelegramChatKind::Private),
        "chatTypeBasicGroup" => Ok(TelegramChatKind::Group),
        "chatTypeSupergroup" => {
            if chat
                .get("type")
                .and_then(|value| value.get("is_channel"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                Ok(TelegramChatKind::Channel)
            } else {
                Ok(TelegramChatKind::Group)
            }
        }
        other => Err(TelegramError::TdlibRuntime(format!(
            "unsupported TDLib chat type `{other}`"
        ))),
    }
}

fn tdlib_username(value: &Value) -> Option<String> {
    value
        .get("username")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            value
                .get("usernames")
                .and_then(|usernames| usernames.get("active_usernames"))
                .and_then(Value::as_array)
                .and_then(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .find(|value| !value.trim().is_empty())
                })
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
}
```

### `backend/src/integrations/telegram/tdjson/parsing/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/events.rs`
- Size bytes / Размер в байтах: `17543`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibTopicSnapshot;

use super::topics::parse_forum_topic_info;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatUnreadSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) unread_count: Option<i64>,
    pub(crate) unread_mention_count: Option<i64>,
    pub(crate) last_read_inbox_message_id: Option<String>,
    pub(crate) source_event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatMarkedAsUnreadSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) is_marked_as_unread: bool,
    pub(crate) source_event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatNotificationSettingsSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) use_default_mute_for: bool,
    pub(crate) mute_for: i64,
    pub(crate) source_event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatPositionSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) list_kind: String,
    pub(crate) provider_folder_id: Option<i64>,
    pub(crate) order: i64,
    pub(crate) is_pinned: bool,
    pub(crate) source_event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatRemovedFromListSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) list_kind: String,
    pub(crate) provider_folder_id: Option<i64>,
    pub(crate) source_event: String,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibTopicUpdateSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) topic: TelegramTdlibTopicSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibTypingSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_thread_id: Option<String>,
    pub(crate) sender_id: String,
    pub(crate) action: String,
    pub(crate) is_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatFoldersUpdateSnapshot {
    pub(crate) folders: Vec<crate::integrations::telegram::tdjson::TelegramTdlibChatFolderSnapshot>,
    pub(crate) source_event: String,
}

pub(crate) fn authorization_state(event: &Value) -> Option<&Value> {
    match event.get("@type").and_then(Value::as_str) {
        Some("updateAuthorizationState") => event.get("authorization_state"),
        Some(value) if value.starts_with("authorizationState") => Some(event),
        _ => None,
    }
}

pub(crate) fn is_tdlib_parameters_not_specified_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event.get("message").and_then(Value::as_str) == Some("Parameters aren't specified")
}

pub(crate) fn is_tdlib_database_encryption_key_needed_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| {
                message.contains("Database encryption key is needed")
                    && message.contains("checkDatabaseEncryptionKey")
            })
}

pub(crate) fn tdlib_error_message(event: &Value) -> Option<String> {
    if event.get("@type").and_then(Value::as_str) != Some("error") {
        return None;
    }

    let code = event
        .get("code")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned());
    let message = event
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("TDLib returned an error");

    Some(format!("TDLib error {code}: {message}"))
}

pub(crate) fn parse_tdlib_typing_snapshot(event: &Value) -> Option<TelegramTdlibTypingSnapshot> {
    if event.get("@type").and_then(Value::as_str) != Some("updateUserChatAction") {
        return None;
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id")?;
    let provider_thread_id = tdlib_event_id(event, "message_thread_id");
    let sender_id = tdlib_sender_id(event.get("sender_id")?)?;
    let action = event
        .get("action")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)?;
    let is_active = action != "chatActionCancel";

    Some(TelegramTdlibTypingSnapshot {
        provider_chat_id,
        provider_thread_id,
        sender_id,
        action,
        is_active,
    })
}

pub(crate) fn parse_tdlib_topic_update_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibTopicUpdateSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateForumTopicInfo") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateForumTopicInfo missing `chat_id`".to_owned())
    })?;
    let info = event.get("info").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateForumTopicInfo missing `info` field".to_owned())
    })?;
    let topic = parse_forum_topic_info(info)?;

    Ok(Some(TelegramTdlibTopicUpdateSnapshot {
        provider_chat_id,
        topic,
    }))
}

pub(crate) fn parse_tdlib_chat_unread_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatUnreadSnapshot>, TelegramError> {
    match event.get("@type").and_then(Value::as_str) {
        Some("updateChatReadInbox") => {
            let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
                TelegramError::TdlibRuntime("updateChatReadInbox missing `chat_id`".to_owned())
            })?;
            let unread_count = event
                .get("unread_count")
                .and_then(Value::as_i64)
                .ok_or_else(|| {
                    TelegramError::TdlibRuntime(
                        "updateChatReadInbox missing `unread_count`".to_owned(),
                    )
                })?;
            Ok(Some(TelegramTdlibChatUnreadSnapshot {
                provider_chat_id,
                unread_count: Some(unread_count),
                unread_mention_count: None,
                last_read_inbox_message_id: tdlib_event_id(event, "last_read_inbox_message_id"),
                source_event: "updateChatReadInbox".to_owned(),
            }))
        }
        Some("updateChatUnreadMentionCount") => {
            let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
                TelegramError::TdlibRuntime(
                    "updateChatUnreadMentionCount missing `chat_id`".to_owned(),
                )
            })?;
            let unread_mention_count = event
                .get("unread_mention_count")
                .and_then(Value::as_i64)
                .ok_or_else(|| {
                    TelegramError::TdlibRuntime(
                        "updateChatUnreadMentionCount missing `unread_mention_count`".to_owned(),
                    )
                })?;
            Ok(Some(TelegramTdlibChatUnreadSnapshot {
                provider_chat_id,
                unread_count: None,
                unread_mention_count: Some(unread_mention_count),
                last_read_inbox_message_id: None,
                source_event: "updateChatUnreadMentionCount".to_owned(),
            }))
        }
        _ => Ok(None),
    }
}

pub(crate) fn parse_tdlib_chat_marked_as_unread_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatMarkedAsUnreadSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateChatIsMarkedAsUnread") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatIsMarkedAsUnread missing `chat_id`".to_owned())
    })?;
    let is_marked_as_unread = event
        .get("is_marked_as_unread")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "updateChatIsMarkedAsUnread missing `is_marked_as_unread`".to_owned(),
            )
        })?;

    Ok(Some(TelegramTdlibChatMarkedAsUnreadSnapshot {
        provider_chat_id,
        is_marked_as_unread,
        source_event: "updateChatIsMarkedAsUnread".to_owned(),
    }))
}

pub(crate) fn parse_tdlib_chat_notification_settings_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatNotificationSettingsSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateChatNotificationSettings") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatNotificationSettings missing `chat_id`".to_owned())
    })?;
    let notification_settings = event.get("notification_settings").ok_or_else(|| {
        TelegramError::TdlibRuntime(
            "updateChatNotificationSettings missing `notification_settings`".to_owned(),
        )
    })?;
    let use_default_mute_for = notification_settings
        .get("use_default_mute_for")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "updateChatNotificationSettings missing `use_default_mute_for`".to_owned(),
            )
        })?;
    let mute_for = notification_settings
        .get("mute_for")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "updateChatNotificationSettings missing `mute_for`".to_owned(),
            )
        })?;

    Ok(Some(TelegramTdlibChatNotificationSettingsSnapshot {
        provider_chat_id,
        use_default_mute_for,
        mute_for,
        source_event: "updateChatNotificationSettings".to_owned(),
    }))
}

pub(crate) fn parse_tdlib_chat_position_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatPositionSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateChatPosition") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatPosition missing `chat_id`".to_owned())
    })?;
    let position = event.get("position").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatPosition missing `position`".to_owned())
    })?;
    let list = position.get("list").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatPosition missing `position.list`".to_owned())
    })?;
    let list_type = list.get("@type").and_then(Value::as_str).ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatPosition missing `position.list.@type`".to_owned())
    })?;
    let (list_kind, provider_folder_id) = match list_type {
        "chatListMain" => ("main".to_owned(), None),
        "chatListArchive" => ("archive".to_owned(), None),
        "chatListFolder" => (
            "folder".to_owned(),
            list.get("chat_folder_id").and_then(Value::as_i64),
        ),
        other => {
            return Err(TelegramError::TdlibRuntime(format!(
                "unsupported updateChatPosition list type `{other}`"
            )));
        }
    };
    let order = position
        .get("order")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("updateChatPosition missing `position.order`".to_owned())
        })?;
    let is_pinned = position
        .get("is_pinned")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "updateChatPosition missing `position.is_pinned`".to_owned(),
            )
        })?;

    Ok(Some(TelegramTdlibChatPositionSnapshot {
        provider_chat_id,
        list_kind,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/tdjson/parsing/files.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/files.rs`
- Size bytes / Размер в байтах: `2343`
- Included characters / Включено символов: `2343`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;

use super::values::tdlib_i64_value;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibFileSnapshot;

pub(crate) fn parse_tdlib_file_snapshot(
    file: &Value,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    if file.get("@type").and_then(Value::as_str) != Some("file") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib file snapshot must have @type=file".to_owned(),
        ));
    }

    let file_id = file
        .get("id")
        .map(tdlib_i64_value)
        .transpose()?
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib file id is required".to_owned()))?;
    let local = file.get("local").and_then(Value::as_object);
    let remote = file.get("remote").and_then(Value::as_object);
    let local_path = local
        .and_then(|value| value.get("path"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let is_downloading_active = local
        .and_then(|value| value.get("is_downloading_active"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let is_downloading_completed = local
        .and_then(|value| value.get("is_downloading_completed"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let downloaded_size_bytes = local
        .and_then(|value| value.get("downloaded_size"))
        .map(tdlib_i64_value)
        .transpose()?;
    let remote_id = remote
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let remote_unique_id = remote
        .and_then(|value| value.get("unique_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Ok(TelegramTdlibFileSnapshot {
        file_id,
        size_bytes: file.get("size").map(tdlib_i64_value).transpose()?,
        expected_size_bytes: file.get("expected_size").map(tdlib_i64_value).transpose()?,
        local_path,
        is_downloading_active,
        is_downloading_completed,
        downloaded_size_bytes,
        remote_id,
        remote_unique_id,
        raw: file.clone(),
    })
}
```

### `backend/src/integrations/telegram/tdjson/parsing/message_events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/message_events.rs`
- Size bytes / Размер в байтах: `10937`
- Included characters / Включено символов: `10937`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::snapshots::{
    TelegramTdlibMessageContentSnapshot, TelegramTdlibMessageDeleteSnapshot,
    TelegramTdlibMessageEditedSnapshot, TelegramTdlibMessageInteractionInfoSnapshot,
    TelegramTdlibMessagePinnedSnapshot, TelegramTdlibMessageSnapshot,
};

use super::message_parts::tdlib_message_text;
use super::messages::parse_tdlib_message_snapshot;
use super::values::{tdlib_string_id, tdlib_unix_datetime_value};

pub(crate) fn parse_tdlib_new_message_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateNewMessage") {
        return Ok(None);
    }

    let message = event.get("message").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateNewMessage missing `message`".to_owned())
    })?;
    parse_tdlib_message_snapshot(message).map(Some)
}

pub(crate) fn parse_tdlib_message_delete_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageDeleteSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateDeleteMessages") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let message_ids = event
        .get("message_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("updateDeleteMessages missing `message_ids`".to_owned())
        })?;
    let provider_message_ids = message_ids
        .iter()
        .map(|value| {
            value
                .as_i64()
                .map(|id| id.to_string())
                .or_else(|| value.as_str().map(ToOwned::to_owned))
                .ok_or_else(|| {
                    TelegramError::TdlibRuntime(
                        "updateDeleteMessages contains a non-numeric `message_ids` value"
                            .to_owned(),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let is_permanent = event
        .get("is_permanent")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("updateDeleteMessages missing `is_permanent`".to_owned())
        })?;
    let from_cache = event
        .get("from_cache")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(Some(TelegramTdlibMessageDeleteSnapshot {
        provider_chat_id,
        provider_message_ids,
        is_permanent,
        from_cache,
        source_event: "updateDeleteMessages".to_owned(),
        raw: event.clone(),
    }))
}

pub(crate) fn parse_tdlib_message_interaction_info_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageInteractionInfoSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateMessageInteractionInfo") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let provider_message_id = tdlib_string_id(event, "message_id")?;
    let interaction_info = event.get("interaction_info").ok_or_else(|| {
        TelegramError::TdlibRuntime(
            "updateMessageInteractionInfo missing `interaction_info`".to_owned(),
        )
    })?;

    Ok(Some(TelegramTdlibMessageInteractionInfoSnapshot {
        provider_chat_id,
        provider_message_id,
        source_event: "updateMessageInteractionInfo".to_owned(),
        raw: json!({
            "interaction_info": interaction_info,
        }),
    }))
}

pub(crate) fn parse_tdlib_message_content_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageContentSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateMessageContent") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let provider_message_id = tdlib_string_id(event, "message_id")?;
    let new_content = event.get("new_content").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateMessageContent missing `new_content`".to_owned())
    })?;
    let text = tdlib_message_text(&json!({
        "content": new_content,
    }))?;

    Ok(Some(TelegramTdlibMessageContentSnapshot {
        provider_chat_id,
        provider_message_id,
        text,
        new_content: new_content.clone(),
        source_event: "updateMessageContent".to_owned(),
        raw: event.clone(),
    }))
}

pub(crate) fn parse_tdlib_message_edited_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageEditedSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateMessageEdited") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let provider_message_id = tdlib_string_id(event, "message_id")?;
    let edit_date = event.get("edit_date").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateMessageEdited missing `edit_date`".to_owned())
    })?;
    let edit_timestamp = tdlib_unix_datetime_value(edit_date)?;
    let reply_markup = event.get("reply_markup").cloned();

    Ok(Some(TelegramTdlibMessageEditedSnapshot {
        provider_chat_id,
        provider_message_id,
        edit_timestamp,
        reply_markup,
        source_event: "updateMessageEdited".to_owned(),
        raw: event.clone(),
    }))
}

pub(crate) fn parse_tdlib_message_pinned_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessagePinnedSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateMessageIsPinned") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let provider_message_id = tdlib_string_id(event, "message_id")?;
    let is_pinned = event
        .get("is_pinned")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("updateMessageIsPinned missing `is_pinned`".to_owned())
        })?;

    Ok(Some(TelegramTdlibMessagePinnedSnapshot {
        provider_chat_id,
        provider_message_id,
        is_pinned,
        source_event: "updateMessageIsPinned".to_owned(),
        raw: event.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_update_new_message_snapshot() {
        let snapshot = parse_tdlib_new_message_snapshot(&json!({
            "@type": "updateNewMessage",
            "message": {
                "@type": "message",
                "chat_id": -100123,
                "id": 42,
                "date": 1_718_618_400,
                "sender_id": {
                    "@type": "messageSenderUser",
                    "user_id": 777
                },
                "content": {
                    "@type": "messageText",
                    "text": {
                        "@type": "formattedText",
                        "text": "hello",
                        "entities": []
                    }
                }
            }
        }))
        .expect("parse updateNewMessage")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert_eq!(snapshot.sender_id, "user:777");
        assert_eq!(snapshot.text, "hello");
    }

    #[test]
    fn parses_update_delete_messages_snapshot() {
        let snapshot = parse_tdlib_message_delete_snapshot(&json!({
            "@type": "updateDeleteMessages",
            "chat_id": -100123,
            "message_ids": [42, 43],
            "is_permanent": true,
            "from_cache": false
        }))
        .expect("parse updateDeleteMessages")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_ids, vec!["42", "43"]);
        assert!(snapshot.is_permanent);
        assert!(!snapshot.from_cache);
    }

    #[test]
    fn parses_update_message_interaction_info_snapshot() {
        let snapshot = parse_tdlib_message_interaction_info_snapshot(&json!({
            "@type": "updateMessageInteractionInfo",
            "chat_id": -100123,
            "message_id": 42,
            "interaction_info": {
                "@type": "messageInteractionInfo",
                "view_count": 0,
                "forward_count": 0,
                "reactions": {
                    "@type": "messageReactions",
                    "reactions": []
                }
            }
        }))
        .expect("parse updateMessageInteractionInfo")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert_eq!(
            snapshot.raw["interaction_info"]["@type"],
            "messageInteractionInfo"
        );
    }

    #[test]
    fn parses_update_message_content_snapshot() {
        let snapshot = parse_tdlib_message_content_snapshot(&json!({
            "@type": "updateMessageContent",
            "chat_id": -100123,
            "message_id": 42,
            "new_content": {
                "@type": "messageText",
                "text": {
                    "@type": "formattedText",
                    "text": "edited",
                    "entities": []
                }
            }
        }))
        .expect("parse updateMessageContent")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert_eq!(snapshot.text, "edited");
        assert_eq!(snapshot.new_content["@type"], "messageText");
    }

    #[test]
    fn parses_update_message_edited_snapshot() {
        let snapshot = parse_tdlib_message_edited_snapshot(&json!({
            "@type": "updateMessageEdited",
            "chat_id": -100123,
            "message_id": 42,
            "edit_date": 1718618400,
            "reply_markup": {
                "@type": "replyMarkupInlineKeyboard",
                "rows": []
            }
        }))
        .expect("parse updateMessageEdited")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert_eq!(
            snapshot.reply_markup.as_ref().expect("reply markup")["@type"],
            "replyMarkupInlineKeyboard"
        );
    }

    #[test]
    fn parses_update_message_is_pinned_snapshot() {
        let snapshot = parse_tdlib_message_pinned_snapshot(&json!({
            "@type": "updateMessageIsPinned",
            "chat_id": -100123,
            "message_id": 42,
            "is_pinned": true
        }))
        .expect("parse updateMessageIsPinned")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert!(snapshot.is_pinned);
    }
}
```

### `backend/src/integrations/telegram/tdjson/parsing/message_parts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/message_parts.rs`
- Size bytes / Размер в байтах: `2435`
- Included characters / Включено символов: `2435`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;

use super::values::tdlib_string_id;

pub(super) fn tdlib_message_sender(message: &Value) -> Result<(String, String), TelegramError> {
    let sender = message
        .get("sender_id")
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib sender_id is required".to_owned()))?;
    match sender.get("@type").and_then(Value::as_str) {
        Some("messageSenderUser") => {
            let user_id = tdlib_string_id(sender, "user_id")?;
            Ok((
                format!("user:{user_id}"),
                format!("Telegram User {user_id}"),
            ))
        }
        Some("messageSenderChat") => {
            let chat_id = tdlib_string_id(sender, "chat_id")?;
            Ok((
                format!("chat:{chat_id}"),
                format!("Telegram Chat {chat_id}"),
            ))
        }
        Some(other) => Err(TelegramError::TdlibRuntime(format!(
            "unsupported TDLib message sender `{other}`"
        ))),
        None => Err(TelegramError::TdlibRuntime(
            "TDLib sender_id @type is required".to_owned(),
        )),
    }
}

pub(super) fn tdlib_message_text(message: &Value) -> Result<String, TelegramError> {
    let content = message.get("content").ok_or_else(|| {
        TelegramError::TdlibRuntime("TDLib message content is required".to_owned())
    })?;
    let content_type = content
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib content @type is required".to_owned()))?;
    let formatted_text = match content_type {
        "messageText" => content.get("text"),
        "messagePhoto" | "messageVideo" | "messageDocument" | "messageAudio"
        | "messageVoiceNote" => content.get("caption"),
        _ => None,
    };

    let text = formatted_text
        .and_then(|value| value.get("text"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    match content_type {
        "messageText" => text.ok_or_else(|| {
            TelegramError::TdlibRuntime("TDLib text message does not contain text".to_owned())
        }),
        "messagePhoto" | "messageVideo" | "messageDocument" | "messageAudio"
        | "messageVoiceNote" | "messageUnsupported" => Ok(text.unwrap_or_default()),
        _ => Ok(text.unwrap_or_default()),
    }
}
```

### `backend/src/integrations/telegram/tdjson/parsing/messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/messages.rs`
- Size bytes / Размер в байтах: `2052`
- Included characters / Включено символов: `2052`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::Value;

use crate::integrations::telegram::client::{TelegramDeliveryState, TelegramError};

use super::message_parts::{tdlib_message_sender, tdlib_message_text};
use super::values::{tdlib_string_id, tdlib_unix_datetime_value};
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibMessageSnapshot;

pub(crate) fn parse_tdlib_message_list(
    response: &Value,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let messages = response
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "TDLib getChatHistory response did not include messages".to_owned(),
            )
        })?;

    messages
        .iter()
        .map(parse_tdlib_message_snapshot)
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) fn parse_tdlib_message_snapshot(
    message: &Value,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    if message.get("@type").and_then(Value::as_str) != Some("message") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib message snapshot must have @type=message".to_owned(),
        ));
    }

    let provider_chat_id = tdlib_string_id(message, "chat_id")?;
    let provider_message_id = tdlib_string_id(message, "id")?;
    let (sender_id, sender_display_name) = tdlib_message_sender(message)?;
    let text = tdlib_message_text(message)?;
    let occurred_at = message
        .get("date")
        .map(tdlib_unix_datetime_value)
        .transpose()?
        .unwrap_or_else(Utc::now);
    let delivery_state = if message
        .get("is_outgoing")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        TelegramDeliveryState::Sent
    } else {
        TelegramDeliveryState::Received
    };

    Ok(TelegramTdlibMessageSnapshot {
        provider_chat_id,
        provider_message_id,
        sender_id,
        sender_display_name,
        text,
        occurred_at,
        delivery_state,
        raw: message.clone(),
    })
}
```

### `backend/src/integrations/telegram/tdjson/parsing/participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/participants.rs`
- Size bytes / Размер в байтах: `4406`
- Included characters / Включено символов: `4406`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Map, Value, json};

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatMemberSnapshot;

pub(crate) fn parse_tdlib_chat_member_list(
    response: &Value,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    parse_member_array(response.get("members"), "TDLib chatMembers response")
}

pub(crate) fn parse_tdlib_basic_group_member_list(
    response: &Value,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    parse_member_array(response.get("members"), "TDLib basicGroupFullInfo response")
}

fn parse_member_array(
    members: Option<&Value>,
    response_kind: &str,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    let members = members.and_then(Value::as_array).ok_or_else(|| {
        TelegramError::TdlibRuntime(format!("{response_kind} missing `members` array"))
    })?;
    members.iter().map(parse_chat_member).collect()
}

fn parse_chat_member(member: &Value) -> Result<TelegramTdlibChatMemberSnapshot, TelegramError> {
    let member_id = member.get("member_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("chatMember missing `member_id` field".to_owned())
    })?;
    let provider_member_id = provider_member_id(member_id)?;
    let status = member.get("status").and_then(Value::as_object);
    let status_kind = status
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .unwrap_or("chatMemberStatusUnknown");
    let role = role_from_status(status_kind).to_owned();
    let permissions = status
        .map(status_permissions)
        .unwrap_or_else(|| json!({ "tdlib_status": status_kind }));

    Ok(TelegramTdlibChatMemberSnapshot {
        provider_member_id,
        display_name: member
            .get("display_name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        username: member
            .get("username")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        role,
        status: status_kind
            .trim_start_matches("chatMemberStatus")
            .to_lowercase(),
        is_admin: matches!(
            status_kind,
            "chatMemberStatusCreator" | "chatMemberStatusAdministrator"
        ),
        is_owner: status_kind == "chatMemberStatusCreator",
        permissions,
        raw: member.clone(),
    })
}

fn provider_member_id(member_id: &Value) -> Result<String, TelegramError> {
    let member_kind = member_id
        .get("@type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match member_kind {
        "messageSenderUser" => member_id
            .get("user_id")
            .and_then(Value::as_i64)
            .map(|id| format!("user:{id}"))
            .ok_or_else(|| {
                TelegramError::TdlibRuntime(
                    "messageSenderUser chatMember missing `user_id`".to_owned(),
                )
            }),
        "messageSenderChat" => member_id
            .get("chat_id")
            .and_then(Value::as_i64)
            .map(|id| format!("chat:{id}"))
            .ok_or_else(|| {
                TelegramError::TdlibRuntime(
                    "messageSenderChat chatMember missing `chat_id`".to_owned(),
                )
            }),
        other => Err(TelegramError::TdlibRuntime(format!(
            "unsupported chat member sender kind `{other}`"
        ))),
    }
}

fn role_from_status(status_kind: &str) -> &'static str {
    match status_kind {
        "chatMemberStatusCreator" => "owner",
        "chatMemberStatusAdministrator" => "admin",
        "chatMemberStatusRestricted" => "restricted",
        "chatMemberStatusBanned" => "banned",
        "chatMemberStatusLeft" => "left",
        "chatMemberStatusMember" => "member",
        _ => "unknown",
    }
}

fn status_permissions(status: &Map<String, Value>) -> Value {
    let mut permissions = Map::new();
    for (key, value) in status {
        if key == "@type" {
            continue;
        }
        if value.is_boolean() || value.is_number() || value.is_string() || value.is_object() {
            permissions.insert(key.clone(), value.clone());
        }
    }
    Value::Object(permissions)
}
```

### `backend/src/integrations/telegram/tdjson/parsing/topics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/topics.rs`
- Size bytes / Размер в байтах: `2667`
- Included characters / Включено символов: `2667`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibTopicSnapshot;

use super::values::{tdlib_i64_value, tdlib_unix_datetime_value};

pub(crate) fn parse_tdlib_topic_list(
    response: &Value,
) -> Result<Vec<TelegramTdlibTopicSnapshot>, TelegramError> {
    let topics = response
        .get("topics")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "TDLib forumTopics response missing `topics` array".to_owned(),
            )
        })?;

    topics.iter().map(parse_forum_topic).collect()
}

pub(crate) fn parse_tdlib_created_forum_topic(
    response: &Value,
) -> Result<TelegramTdlibTopicSnapshot, TelegramError> {
    parse_forum_topic_info(response)
}

fn parse_forum_topic(topic: &Value) -> Result<TelegramTdlibTopicSnapshot, TelegramError> {
    let info = topic
        .get("info")
        .ok_or_else(|| TelegramError::TdlibRuntime("forumTopic missing `info` field".to_owned()))?;
    let mut snapshot = parse_forum_topic_info(info)?;
    snapshot.unread_count = topic
        .get("unread_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    snapshot.last_message_at = topic
        .get("last_message")
        .and_then(|m| m.get("date"))
        .map(tdlib_unix_datetime_value)
        .transpose()?;
    Ok(snapshot)
}

pub(crate) fn parse_forum_topic_info(
    info: &Value,
) -> Result<TelegramTdlibTopicSnapshot, TelegramError> {
    let provider_topic_id = info
        .get("message_thread_id")
        .map(tdlib_i64_value)
        .transpose()?
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("forumTopicInfo missing `message_thread_id`".to_owned())
        })?;

    let title = info
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("Topic {provider_topic_id}"));

    let icon_emoji = info
        .get("icon")
        .and_then(|icon| icon.get("custom_emoji_id"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && *s != "0")
        .map(ToOwned::to_owned);

    let is_pinned = info
        .get("is_pinned")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let is_closed = info
        .get("is_closed")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(TelegramTdlibTopicSnapshot {
        provider_topic_id,
        title,
        icon_emoji,
        is_pinned,
        is_closed,
        unread_count: 0,
        last_message_at: None,
    })
}
```

### `backend/src/integrations/telegram/tdjson/parsing/values.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/parsing/values.rs`
- Size bytes / Размер в байтах: `1112`
- Included characters / Включено символов: `1112`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;

pub(super) fn tdlib_string_id(value: &Value, field: &'static str) -> Result<String, TelegramError> {
    value
        .get(field)
        .map(tdlib_i64_value)
        .transpose()?
        .map(|value| value.to_string())
        .ok_or_else(|| TelegramError::TdlibRuntime(format!("TDLib field `{field}` is required")))
}

pub(super) fn tdlib_i64_value(value: &Value) -> Result<i64, TelegramError> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib id must be an i64".to_owned()))
}

pub(super) fn tdlib_unix_datetime_value(value: &Value) -> Result<DateTime<Utc>, TelegramError> {
    let timestamp = value
        .as_i64()
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib date must be an i64".to_owned()))?;
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .ok_or_else(|| TelegramError::TdlibRuntime(format!("invalid TDLib date `{timestamp}`")))
}
```

### `backend/src/integrations/telegram/tdjson/qr_login.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login.rs`
- Size bytes / Размер в байтах: `269`
- Included characters / Включено символов: `269`
- Truncated / Обрезано: `no`

```rust
mod authorization;
mod commands;
mod driver;
mod tdlib_commands;
mod worker;
mod worker_state;

pub(super) use commands::cancel_existing_qr_logins_for_account;
pub(crate) use commands::{cancel_qr_login, submit_qr_login_password};
pub(crate) use driver::start_qr_login;
```

### `backend/src/integrations/telegram/tdjson/qr_login/authorization.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login/authorization.rs`
- Size bytes / Размер в байтах: `7170`
- Included characters / Включено символов: `7170`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStatus};

use super::super::parsing::{authorization_state, tdlib_error_message};
use super::super::qr_login_support::{
    QR_POLL_AFTER_MS, fetch_authorized_user_identity, mark_pending_ready_status,
    mark_pending_status, password_hint, qr_waiting_response, state_allows_qr_request,
    upsert_pending_response,
};
use super::tdlib_commands::{close_tdlib_session, send_tdlib_parameters};
use super::worker::handle_tdlib_setup_event;
use super::worker_state::{QrLoginEventOutcome, QrLoginRuntimeState, QrLoginWorkerContext};

pub(super) fn handle_qr_login_event(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    event: Value,
) -> Result<QrLoginEventOutcome, TelegramError> {
    if handle_tdlib_setup_event(context, state, &event)? {
        return Ok(QrLoginEventOutcome::Continue);
    }
    if handle_tdlib_error(context, state, &event)? {
        return Ok(QrLoginEventOutcome::Continue);
    }

    let Some(authorization_state) = authorization_state(&event) else {
        return Ok(QrLoginEventOutcome::Continue);
    };
    let Some(state_type) = authorization_state.get("@type").and_then(Value::as_str) else {
        return Ok(QrLoginEventOutcome::Continue);
    };

    handle_authorization_state(context, state, authorization_state, state_type)
}

fn handle_tdlib_error(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    event: &Value,
) -> Result<bool, TelegramError> {
    let Some(message) = tdlib_error_message(event) else {
        return Ok(false);
    };
    if state.password_check_in_flight {
        state.password_check_in_flight = false;
        mark_pending_status(
            context.pending_logins,
            context.setup_id,
            TelegramQrLoginStatus::WaitingPassword,
            "Telegram password was rejected. Try again.",
            QR_POLL_AFTER_MS,
        )?;
        return Ok(true);
    }
    Err(TelegramError::TdlibRuntime(message))
}

fn handle_authorization_state(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    authorization_state: &Value,
    state_type: &str,
) -> Result<QrLoginEventOutcome, TelegramError> {
    match state_type {
        "authorizationStateWaitTdlibParameters" => {
            send_tdlib_parameters(context.client, context.request, context.database_directory)?;
            state.tdlib_parameters_sent = true;
        }
        "authorizationStateWaitEncryptionKey" => {
            context.client.send_json(
                &super::super::requests::check_database_encryption_key_request(context.request),
            )?;
            state.database_encryption_key_checked = true;
        }
        state_name if state_allows_qr_request(state_name) && !state.qr_requested => {
            context.client.send_json(&json!({
                "@type": "requestQrCodeAuthentication",
                "other_user_ids": [],
                "@extra": "makosh-request-qr-code-authentication"
            }))?;
            state.qr_requested = true;
        }
        "authorizationStateWaitOtherDeviceConfirmation" => {
            handle_wait_other_device_confirmation(context, state, authorization_state)?;
        }
        "authorizationStateWaitPassword" => {
            handle_wait_password(context, state, authorization_state)?
        }
        "authorizationStateReady" => return handle_ready(context, state),
        "authorizationStateClosed" => {
            mark_pending_status(
                context.pending_logins,
                context.setup_id,
                TelegramQrLoginStatus::Failed,
                "Telegram TDLib authorization session closed before QR login completed.",
                0,
            )?;
            return Ok(QrLoginEventOutcome::Complete);
        }
        "authorizationStateClosing" | "authorizationStateLoggingOut" => {
            mark_pending_status(
                context.pending_logins,
                context.setup_id,
                TelegramQrLoginStatus::Failed,
                "Telegram TDLib authorization session is closing.",
                0,
            )?;
            return Ok(QrLoginEventOutcome::Complete);
        }
        unsupported if state.qr_requested => {
            mark_pending_status(
                context.pending_logins,
                context.setup_id,
                TelegramQrLoginStatus::Failed,
                &format!(
                    "Telegram QR login requires unsupported authorization state `{unsupported}`."
                ),
                0,
            )?;
            return Ok(QrLoginEventOutcome::Complete);
        }
        _ => {}
    }
    Ok(QrLoginEventOutcome::Continue)
}

fn handle_wait_other_device_confirmation(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    authorization_state: &Value,
) -> Result<(), TelegramError> {
    state.qr_link_issued = true;
    let link = authorization_state
        .get("link")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "TDLib QR authorization state did not include a link".to_owned(),
            )
        })?;
    let response = qr_waiting_response(context.setup_id, &context.request.account_id, link)?;
    upsert_pending_response(
        context.pending_logins,
        response,
        context.command_tx.clone(),
        std::sync::Arc::clone(context.worker_completion),
    )
}

fn handle_wait_password(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    authorization_state: &Value,
) -> Result<(), TelegramError> {
    state.password_check_in_flight = false;
    let password_hint = password_hint(authorization_state);
    let message = password_hint
        .as_deref()
        .map(|hint| format!("Telegram requires your 2-step verification password. Hint: {hint}"))
        .unwrap_or_else(|| "Telegram requires your 2-step verification password.".to_owned());
    mark_pending_status(
        context.pending_logins,
        context.setup_id,
        TelegramQrLoginStatus::WaitingPassword,
        &message,
        QR_POLL_AFTER_MS,
    )
}

fn handle_ready(
    context: &QrLoginWorkerContext<'_>,
    state: &QrLoginRuntimeState,
) -> Result<QrLoginEventOutcome, TelegramError> {
    let identity = match fetch_authorized_user_identity(context.client) {
        Ok(identity) => identity,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "Telegram QR login completed, but TDLib user identity lookup failed"
            );
            None
        }
    };
    let message = if state.qr_requested {
        "Telegram QR login confirmed on the other device."
    } else {
        "Telegram TDLib session is already authorized."
    };
    mark_pending_ready_status(
        context.pending_logins,
        context.setup_id,
        message,
        identity.as_ref(),
    )?;
    close_tdlib_session(context.client);
    Ok(QrLoginEventOutcome::Complete)
}
```

### `backend/src/integrations/telegram/tdjson/qr_login/commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login/commands.rs`
- Size bytes / Размер в байтах: `3961`
- Included characters / Включено символов: `3961`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStatus,
    TelegramQrLoginStatusResponse,
};

use super::super::qr_login_support::{
    PendingQrLoginMap, QR_POLL_AFTER_MS, TelegramQrLoginCommand, wait_for_worker_completion,
};

pub(crate) fn submit_qr_login_password(
    pending_logins: PendingQrLoginMap,
    setup_id: &str,
    request: TelegramQrLoginPasswordRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    let setup_id = setup_id.trim();
    if setup_id.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "setup_id must not be empty".to_owned(),
        ));
    }
    if request.password.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "password must not be empty".to_owned(),
        ));
    }

    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    let session = pending_logins
        .get_mut(setup_id)
        .ok_or(TelegramError::QrLoginNotFound)?;
    if session.response.status != TelegramQrLoginStatus::WaitingPassword {
        return Err(TelegramError::InvalidRequest(
            "Telegram QR login is not waiting for a password".to_owned(),
        ));
    }

    session
        .command_tx
        .send(TelegramQrLoginCommand::CheckPassword(request.password))
        .map_err(|_| {
            TelegramError::TdlibRuntime(
                "Telegram QR login worker is no longer accepting password commands".to_owned(),
            )
        })?;
    session.response.message = Some("Checking Telegram password.".to_owned());
    session.response.poll_after_ms = QR_POLL_AFTER_MS;

    Ok(session.response.clone())
}

pub(crate) fn cancel_qr_login(
    pending_logins: PendingQrLoginMap,
    setup_id: &str,
) -> Result<(), TelegramError> {
    let setup_id = setup_id.trim();
    if setup_id.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "setup_id must not be empty".to_owned(),
        ));
    }

    let session = {
        let mut pending_logins = pending_logins.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
        })?;
        pending_logins
            .remove(setup_id)
            .ok_or(TelegramError::QrLoginNotFound)?
    };
    let _ = session.command_tx.send(TelegramQrLoginCommand::Cancel);
    wait_for_worker_completion(&session.worker_completion)?;
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    pending_logins.remove(setup_id);
    Ok(())
}

pub(in crate::integrations::telegram::tdjson) fn cancel_existing_qr_logins_for_account(
    pending_logins: &PendingQrLoginMap,
    account_id: &str,
) -> Result<(), TelegramError> {
    let sessions = {
        let mut pending = pending_logins.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
        })?;
        let setup_ids = pending
            .iter()
            .filter(|(_, session)| session.response.account_id == account_id)
            .map(|(setup_id, _)| setup_id.clone())
            .collect::<Vec<_>>();
        setup_ids
            .into_iter()
            .filter_map(|setup_id| pending.remove(&setup_id).map(|session| (setup_id, session)))
            .collect::<Vec<_>>()
    };

    for (setup_id, session) in sessions {
        let _ = session.command_tx.send(TelegramQrLoginCommand::Cancel);
        wait_for_worker_completion(&session.worker_completion)?;
        let mut pending = pending_logins.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
        })?;
        pending.remove(&setup_id);
    }

    Ok(())
}
```

### `backend/src/integrations/telegram/tdjson/qr_login/driver.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login/driver.rs`
- Size bytes / Размер в байтах: `3399`
- Included characters / Включено символов: `3399`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use tokio::task;

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginStartRequest, TelegramQrLoginStatusResponse,
};
use crate::platform::config::AppConfig;

use super::super::client::TdJsonLibrary;
use super::super::qr_login_support::{
    PendingQrLoginMap, mark_pending_status, mark_worker_complete, new_setup_id,
    new_worker_completion, qr_preparing_response, short_thread_suffix, upsert_pending_response,
};
use super::commands::cancel_existing_qr_logins_for_account;
use super::worker::drive_qr_login;

pub(crate) async fn start_qr_login(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    request.validate()?;
    task::spawn_blocking(move || start_qr_login_driver(config, pending_logins, request))
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("Telegram QR login worker failed: {error}"))
        })?
}

fn start_qr_login_driver(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    let _runtime_probe = TdJsonLibrary::load(config.tdjson_path())?;
    cancel_existing_qr_logins_for_account(&pending_logins, &request.account_id)?;
    let (command_tx, command_rx) = mpsc::channel();
    let worker_completion = new_worker_completion();
    let setup_id = new_setup_id(&request.account_id);
    let response = qr_preparing_response(&setup_id, &request.account_id);
    let thread_name = format!(
        "telegram-qr-login-{}",
        short_thread_suffix(&request.account_id)
    );

    upsert_pending_response(
        &pending_logins,
        response.clone(),
        command_tx.clone(),
        Arc::clone(&worker_completion),
    )?;

    thread::Builder::new()
        .name(thread_name)
        .spawn({
            let setup_id = setup_id.clone();
            let pending_logins = Arc::clone(&pending_logins);
            let worker_completion = Arc::clone(&worker_completion);
            move || {
                let result = drive_qr_login(
                    config,
                    pending_logins.clone(),
                    request,
                    setup_id.clone(),
                    command_tx,
                    command_rx,
                    Arc::clone(&worker_completion),
                );
                if let Err(error) = result {
                    let _ = mark_pending_status(
                        &pending_logins,
                        &setup_id,
                        crate::integrations::telegram::client::TelegramQrLoginStatus::Failed,
                        "Telegram QR login failed before the QR code was issued.",
                        0,
                    );
                    tracing::warn!(error = %error, "Telegram QR login worker failed");
                }
                mark_worker_complete(&worker_completion);
            }
        })
        .map_err(|error| {
            let _ = pending_logins.lock().map(|mut pending| {
                pending.remove(&setup_id);
            });
            TelegramError::TdlibRuntime(format!(
                "failed to spawn Telegram QR login worker: {error}"
            ))
        })?;

    Ok(response)
}
```

### `backend/src/integrations/telegram/tdjson/qr_login/tdlib_commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login/tdlib_commands.rs`
- Size bytes / Размер в байтах: `1862`
- Included characters / Включено символов: `1862`
- Truncated / Обрезано: `no`

```rust
use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError};

use serde_json::json;

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};

use super::super::client::TdJsonClient;
use super::super::qr_login_support::{DrainedQrLoginCommand, TelegramQrLoginCommand};
use super::super::requests::set_tdlib_parameters_request;

pub(super) fn drain_qr_login_commands(
    client: &TdJsonClient,
    command_rx: &Receiver<TelegramQrLoginCommand>,
) -> Result<DrainedQrLoginCommand, TelegramError> {
    let mut password_submitted = false;
    loop {
        match command_rx.try_recv() {
            Ok(TelegramQrLoginCommand::CheckPassword(password)) => {
                client.send_json(&json!({
                    "@type": "checkAuthenticationPassword",
                    "password": password,
                    "@extra": "makosh-check-authentication-password"
                }))?;
                password_submitted = true;
            }
            Ok(TelegramQrLoginCommand::Cancel) => {
                client.send_json(&json!({ "@type": "close" }))?;
                return Ok(DrainedQrLoginCommand::Cancelled);
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => {
                return Ok(if password_submitted {
                    DrainedQrLoginCommand::PasswordSubmitted
                } else {
                    DrainedQrLoginCommand::None
                });
            }
        }
    }
}

pub(super) fn send_tdlib_parameters(
    client: &TdJsonClient,
    request: &TelegramQrLoginStartRequest,
    database_directory: &Path,
) -> Result<(), TelegramError> {
    client.send_json(&set_tdlib_parameters_request(request, database_directory)?)
}

pub(super) fn close_tdlib_session(client: &TdJsonClient) {
    let _ = client.send_json(&json!({ "@type": "close" }));
}
```

### `backend/src/integrations/telegram/tdjson/qr_login/worker.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login/worker.rs`
- Size bytes / Размер в байтах: `5506`
- Included characters / Включено символов: `5506`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use serde_json::json;

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginStartRequest, TelegramQrLoginStatus,
};
use crate::platform::config::AppConfig;

use super::super::client::TdJsonLibrary;
use super::super::qr_login_support::{
    DrainedQrLoginCommand, PendingQrLoginMap, QR_FIRST_LINK_TIMEOUT, QR_POLL_AFTER_MS,
    QR_SESSION_LIFETIME, QrLoginWorkerCompletion, TelegramQrLoginCommand, mark_pending_status,
};
use super::super::requests::{check_database_encryption_key_request, tdlib_database_directory};
use super::authorization::handle_qr_login_event;
use super::tdlib_commands::{close_tdlib_session, drain_qr_login_commands, send_tdlib_parameters};
use super::worker_state::{QrLoginEventOutcome, QrLoginRuntimeState, QrLoginWorkerContext};

pub(super) fn drive_qr_login(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
    setup_id: String,
    command_tx: Sender<TelegramQrLoginCommand>,
    command_rx: Receiver<TelegramQrLoginCommand>,
    worker_completion: QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let library = TdJsonLibrary::load(config.tdjson_path())?;
    let client = library.create_client()?;
    let database_directory = tdlib_database_directory(&request);
    let files_directory = database_directory.join("files");
    std::fs::create_dir_all(&files_directory).map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to create TDLib data directory `{}`: {error}",
            files_directory.display()
        ))
    })?;

    let _ = client.execute_json(&json!({
        "@type": "setLogVerbosityLevel",
        "new_verbosity_level": 1
    }));
    client.send_json(&json!({
        "@type": "getAuthorizationState",
        "@extra": "makosh-initial-authorization-state"
    }))?;

    let started_at = Instant::now();
    let mut state = QrLoginRuntimeState::default();

    loop {
        if drain_worker_commands(&client, &command_rx, &pending_logins, &setup_id, &mut state)? {
            return Ok(());
        }
        if expire_stale_session(&client, &pending_logins, &setup_id, &state, started_at)? {
            return Ok(());
        }

        let Some(event) = client.receive_json(1.0)? else {
            continue;
        };
        let context = QrLoginWorkerContext {
            client: &client,
            pending_logins: &pending_logins,
            setup_id: &setup_id,
            request: &request,
            command_tx: &command_tx,
            worker_completion: &worker_completion,
            database_directory: &database_directory,
        };
        if handle_qr_login_event(&context, &mut state, event)? == QrLoginEventOutcome::Complete {
            return Ok(());
        }
    }
}

fn drain_worker_commands(
    client: &super::super::client::TdJsonClient,
    command_rx: &Receiver<TelegramQrLoginCommand>,
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    state: &mut QrLoginRuntimeState,
) -> Result<bool, TelegramError> {
    match drain_qr_login_commands(client, command_rx)? {
        DrainedQrLoginCommand::Cancelled => Ok(true),
        DrainedQrLoginCommand::PasswordSubmitted => {
            state.password_check_in_flight = true;
            mark_pending_status(
                pending_logins,
                setup_id,
                TelegramQrLoginStatus::WaitingPassword,
                "Checking Telegram password.",
                QR_POLL_AFTER_MS,
            )?;
            Ok(false)
        }
        DrainedQrLoginCommand::None => Ok(false),
    }
}

fn expire_stale_session(
    client: &super::super::client::TdJsonClient,
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    state: &QrLoginRuntimeState,
    started_at: Instant,
) -> Result<bool, TelegramError> {
    if !state.qr_link_issued && started_at.elapsed() > QR_FIRST_LINK_TIMEOUT {
        mark_pending_status(
            pending_logins,
            setup_id,
            TelegramQrLoginStatus::Failed,
            "Telegram TDLib did not return a QR confirmation link in time.",
            0,
        )?;
        close_tdlib_session(client);
        return Ok(true);
    }
    if started_at.elapsed() > QR_SESSION_LIFETIME {
        mark_pending_status(
            pending_logins,
            setup_id,
            TelegramQrLoginStatus::Expired,
            "Telegram QR login session expired; start a new QR login.",
            0,
        )?;
        close_tdlib_session(client);
        return Ok(true);
    }
    Ok(false)
}

pub(super) fn handle_tdlib_setup_event(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    event: &serde_json::Value,
) -> Result<bool, TelegramError> {
    if super::super::parsing::is_tdlib_parameters_not_specified_error(event) {
        if !state.tdlib_parameters_sent {
            send_tdlib_parameters(context.client, context.request, context.database_directory)?;
            state.tdlib_parameters_sent = true;
        }
        return Ok(true);
    }
    if super::super::parsing::is_tdlib_database_encryption_key_needed_error(event) {
        if !state.database_encryption_key_checked {
            context
                .client
                .send_json(&check_database_encryption_key_request(context.request))?;
            state.database_encryption_key_checked = true;
        }
        return Ok(true);
    }
    Ok(false)
}
```
