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

- Chunk ID / ID чанка: `063-source-backend-part-043`
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

### `backend/src/integrations/telegram/runtime/actor/topics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/topics.rs`
- Size bytes / Размер в байтах: `2346`
- Included characters / Включено символов: `2346`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibTopicSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};

pub(super) fn actor_get_forum_topics(
    client: &TdJsonClient,
    provider_chat_id: &str,
    limit: i32,
) -> Result<Vec<TelegramTdlibTopicSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-forum-topics-{provider_chat_id}");
    client.send_json(&tdjson::tdlib_get_forum_topics_request(
        chat_id, limit, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_topic_list(&response)
}

pub(super) fn actor_create_forum_topic(
    client: &TdJsonClient,
    provider_chat_id: &str,
    title: &str,
    command_id: &str,
) -> Result<TelegramTdlibTopicSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-forum-topic-create-{command_id}");
    client.send_json(&tdjson::tdlib_create_forum_topic_request(
        chat_id, title, &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_created_forum_topic(&response)
}

pub(super) fn actor_toggle_forum_topic_closed(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_topic_id: i64,
    is_closed: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-forum-topic-closed-{command_id}");
    client.send_json(&tdjson::tdlib_toggle_forum_topic_is_closed_request(
        chat_id,
        provider_topic_id,
        is_closed,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}
```

### `backend/src/integrations/telegram/runtime/commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/commands.rs`
- Size bytes / Размер в байтах: `24660`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::mpsc::{self, Sender};

use tokio::task;

use crate::integrations::telegram::client::{TelegramError, TelegramManualSendRequest};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot,
    TelegramTdlibMessageSnapshot, TelegramTdlibTopicSnapshot,
};

use super::TDJSON_COMMAND_TIMEOUT;
use super::models::{TelegramHistorySyncMode, TelegramMediaSendRequest};
use super::state::TelegramRuntimeCommand;

pub(super) async fn request_actor_chats(
    command_tx: Sender<TelegramRuntimeCommand>,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::LoadChats { limit, reply_tx })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat sync commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat sync timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_chat_folders(
    command_tx: Sender<TelegramRuntimeCommand>,
    folder_ids: Vec<i64>,
) -> Result<Vec<TelegramTdlibChatFolderSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetChatFolders {
                folder_ids,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting folder sync commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib folder sync timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_history(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    from_message_id: Option<i64>,
    limit: i32,
    mode: TelegramHistorySyncMode,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SyncHistory {
                provider_chat_id,
                from_message_id,
                limit,
                mode,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting history sync commands".to_owned(),
                )
            })?;
        let timeout = if mode == TelegramHistorySyncMode::Full {
            TDJSON_COMMAND_TIMEOUT * 10
        } else {
            TDJSON_COMMAND_TIMEOUT
        };
        reply_rx.recv_timeout(timeout).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib history sync timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_send(
    command_tx: Sender<TelegramRuntimeCommand>,
    request: TelegramManualSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SendText { request, reply_tx })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting send commands".to_owned(),
                )
            })?;
        reply_rx
            .recv_timeout(TDJSON_COMMAND_TIMEOUT)
            .map_err(|_| TelegramError::TdlibRuntime("Telegram TDLib send timed out".to_owned()))?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_send_media(
    command_tx: Sender<TelegramRuntimeCommand>,
    request: TelegramMediaSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SendMedia { request, reply_tx })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting media send commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib media send timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_download_file(
    command_tx: Sender<TelegramRuntimeCommand>,
    file_id: i64,
    priority: i32,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::DownloadFile {
                file_id,
                priority,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting media download commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib media download timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_edit_message(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_message_id: String,
    new_text: String,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::EditMessage {
                provider_chat_id,
                provider_message_id,
                new_text,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting edit commands".to_owned(),
                )
            })?;
        reply_rx
            .recv_timeout(TDJSON_COMMAND_TIMEOUT)
            .map_err(|_| TelegramError::TdlibRuntime("Telegram TDLib edit timed out".to_owned()))?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_delete_message(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_message_id: String,
    revoke: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::DeleteMessage {
                provider_chat_id,
                provider_message_id,
                revoke,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting delete commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib delete timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_set_reaction(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_message_id: String,
    reaction_emoji: String,
    is_active: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SetReaction {
                provider_chat_id,
                provider_message_id,
                reaction_emoji,
                is_active,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting reaction commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib reaction timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_reply(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    reply_to_provider_message_id: String,
    text: String,
    command_id: String,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::ReplyMessage {
                provider_chat_id,
                reply_to_provider_message_id,
                text,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting reply commands".to_owned(),
                )
            })?;
        reply_rx
            .recv_timeout(TDJSON_COMMAND_TIMEOUT)
            .map_err(|_| TelegramError::TdlibRuntime("Telegram TDLib reply timed out".to_owned()))?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_forward(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    from_provider_chat_id: String,
    from_provider_message_id: String,
    command_id: String,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::ForwardMessage {
                provider_chat_id,
                from_provider_chat_id,
                from_provider_message_id,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting forward commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib forward timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_pin_message(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_message_id: String,
    pin: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::PinMessage {
                provider_chat_id,
                provider_message_id,
                pin,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager.rs`
- Size bytes / Размер в байтах: `7237`
- Included characters / Включено символов: `7237`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::integrations::telegram::client::TelegramStore;
use crate::platform::communications::{
    DEFAULT_MAIL_SYNC_BLOB_ROOT, ProviderAccountLookupPort, ProviderSecretBindingLookupPort,
};
use crate::platform::config::AppConfig;
use crate::platform::events::EventBus;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};
use sqlx::PgPool;

use super::models::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse,
};
use super::state::TelegramRuntimeActorHandle;
use crate::integrations::telegram::client::{
    TelegramError, TelegramForwardRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramReplyRequest,
};

mod account;
mod actor_states;
mod chat_event_payloads;
mod chat_events;
pub(crate) mod command_executor;
mod command_executor_dispatch;
mod command_executor_media;
mod lifecycle;
mod media_download;
mod message_events;
mod participant_events;
mod participants;
mod realtime_events;
mod registry;
mod search;
mod send;
mod sync_chats;
mod sync_history;
mod sync_history_tdlib;
mod tdlib_actor;
mod topic_events;
mod topics;

pub(crate) use self::realtime_events::TelegramRuntimeEventBridgeContext;
pub(crate) use self::search::TelegramProviderSearchRequest;

#[derive(Clone, Default)]
pub struct TelegramRuntimeManager {
    actors: Arc<Mutex<HashMap<String, TelegramRuntimeActorHandle>>>,
}

pub(crate) struct TelegramMediaDownloadContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramMemberSyncContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramRuntimeOperationContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramRuntimeOperationDeps<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramRuntimeStartContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bus: &'a EventBus,
}

fn telegram_media_blob_root() -> &'static std::path::Path {
    std::path::Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
}

impl TelegramRuntimeManager {
    pub(crate) async fn sync_chats_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramChatSyncRequest,
    ) -> Result<TelegramChatSyncResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.sync_chats(&operation_context(deps), request).await
    }

    pub(crate) async fn sync_history_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramHistorySyncRequest,
    ) -> Result<TelegramHistorySyncResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.sync_history(&operation_context(deps), request).await
    }

    pub(crate) async fn send_manual_message_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.send_manual_message(&operation_context(deps), request)
            .await
    }

    pub(crate) async fn send_reply_message_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramReplyRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.send_reply_message(&operation_context(deps), request)
            .await
    }

    pub(crate) async fn send_forward_message_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramForwardRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.send_forward_message(&operation_context(deps), request)
            .await
    }

    pub(crate) async fn search_provider_messages_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramProviderSearchRequest,
    ) -> Result<Vec<String>, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.search_provider_messages(&operation_context(deps), request)
            .await
    }

    pub(crate) async fn sync_forum_topics_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        telegram_chat_id: &str,
    ) -> Result<usize, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.sync_forum_topics(&operation_context(deps), telegram_chat_id)
            .await
    }
}

fn operation_context<S>(
    deps: TelegramRuntimeOperationDeps<'_, S>,
) -> TelegramRuntimeOperationContext<'_, S>
where
    S: SecretResolver + Sync + ?Sized,
{
    TelegramRuntimeOperationContext {
        provider_account_store: deps.provider_account_store,
        provider_secret_binding_store: deps.provider_secret_binding_store,
        telegram_store: deps.telegram_store,
        secret_store: deps.secret_store,
        secret_resolver: deps.secret_resolver,
        config: deps.config,
        event_bridge: deps.event_bridge,
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/account.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/account.rs`
- Size bytes / Размер в байтах: `589`
- Included characters / Включено символов: `589`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::{TelegramError, ensure_telegram_account_active};
use crate::platform::communications::{ProviderAccount, ProviderAccountLookupPort};

use super::super::status::load_telegram_account;

pub(in crate::integrations::telegram::runtime::manager) async fn load_active_account(
    provider_account_store: &dyn ProviderAccountLookupPort,
    account_id: &str,
) -> Result<ProviderAccount, TelegramError> {
    let account = load_telegram_account(provider_account_store, account_id).await?;
    ensure_telegram_account_active(&account)?;
    Ok(account)
}
```

### `backend/src/integrations/telegram/runtime/manager/actor_states.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/actor_states.rs`
- Size bytes / Размер в байтах: `384`
- Included characters / Включено символов: `384`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::super::state::{TelegramRuntimeActorState, TelegramRuntimeState};

pub(in crate::integrations::telegram::runtime::manager) fn running_actor_state(
    updated_at: DateTime<Utc>,
) -> TelegramRuntimeActorState {
    TelegramRuntimeActorState {
        status: TelegramRuntimeState::Running,
        last_error: None,
        updated_at,
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/chat_event_payloads.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/chat_event_payloads.rs`
- Size bytes / Размер в байтах: `23004`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, TimeZone, Utc};
use serde_json::json;

use crate::integrations::telegram::client::models::TelegramChat;
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatMarkedAsUnreadSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatUnreadSnapshot,
};
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;

pub(super) fn chat_unread_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatUnreadSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_unread_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_unread_update",
        "unread_count": snapshot.unread_count,
        "unread_mention_count": snapshot.unread_mention_count,
        "last_read_inbox_provider_message_id": snapshot.last_read_inbox_message_id,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_marked_as_unread_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatMarkedAsUnreadSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_marked_unread_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_marked_as_unread_update",
        "is_marked_as_unread": snapshot.is_marked_as_unread,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_notification_settings_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    let is_muted = !snapshot.use_default_mute_for && snapshot.mute_for > 0;
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_notification_settings_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_MUTED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "is_muted": is_muted,
        "use_default_mute_for": snapshot.use_default_mute_for,
        "mute_for": snapshot.mute_for,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_notification_settings_chat_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    let is_muted = !snapshot.use_default_mute_for && snapshot.mute_for > 0;
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_updated_notification_settings_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_notification_settings_update",
        "is_muted": is_muted,
        "use_default_mute_for": snapshot.use_default_mute_for,
        "mute_for": snapshot.mute_for,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_archived_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatPositionSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    let is_archived = chat
        .metadata
        .get("is_archived")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_archive_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_ARCHIVED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "is_archived": is_archived,
        "list_kind": snapshot.list_kind,
        "provider_folder_id": snapshot.provider_folder_id,
        "order": snapshot.order,
        "is_pinned": snapshot.is_pinned,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_position_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatPositionSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    let is_archived = chat
        .metadata
        .get("is_archived")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_updated_position_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_chat_position_update",
        "is_archived": is_archived,
        "list_kind": snapshot.list_kind,
        "provider_folder_id": snapshot.provider_folder_id,
        "order": snapshot.order,
        "is_pinned": snapshot.is_pinned,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_folder_labels_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    let folder_labels = chat
        .metadata
        .get("folder_labels")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let provider_folder_id = chat.metadata.get("provider_folder_id").cloned();

    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_folder_labels_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_chat_folder_labels_update",
        "folder_labels": folder_labels,
        "provider_folder_id": provider_folder_id,
        "chat": chat,
        "source": "tdlib.updateChatFolders"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateChatFolders"
    }))
    .build()
}

pub(super) fn chat_pinned_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatPositionSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_pinned_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_PINNED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "accou
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/chat_events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/chat_events.rs`
- Size bytes / Размер в байтах: `19451`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::models::{TelegramChat, TelegramChatGroupFilter};
use crate::integrations::telegram::client::{
    TelegramError, TelegramProviderChatPositionUpdate, TelegramStore,
    reconcile_archive_commands_from_provider_state,
    reconcile_folder_add_commands_from_provider_state,
    reconcile_folder_remove_commands_from_provider_state,
    reconcile_mark_read_commands_from_provider_state,
    reconcile_marked_as_unread_commands_from_provider_state,
    reconcile_mute_commands_from_provider_state, reconcile_pin_commands_from_provider_state,
};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatMarkedAsUnreadSnapshot,
    TelegramTdlibChatNotificationSettingsSnapshot, TelegramTdlibChatPositionSnapshot,
    TelegramTdlibChatRemovedFromListSnapshot, TelegramTdlibChatUnreadSnapshot,
};
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

use super::chat_event_payloads::{
    chat_archived_updated_event, chat_folder_labels_updated_event,
    chat_marked_as_unread_updated_event, chat_notification_settings_chat_updated_event,
    chat_notification_settings_updated_event, chat_pinned_updated_event,
    chat_position_updated_event, chat_unread_updated_event,
};
use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};

pub(super) async fn publish_chat_unread_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatUnreadSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let (chat, reconciled) = match apply_chat_unread_update(store, account_id, snapshot).await {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project unread update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = chat_unread_updated_event(account_id, &chat, snapshot, Utc::now()) else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append unread chat event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_marked_as_unread_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatMarkedAsUnreadSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let (chat, reconciled) = match apply_chat_marked_as_unread_update(store, account_id, snapshot)
        .await
    {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project marked-as-unread update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = chat_marked_as_unread_updated_event(account_id, &chat, snapshot, Utc::now())
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append marked-as-unread chat event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_notification_settings_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let (chat, reconciled) = match apply_chat_notification_settings_update(
        store, account_id, snapshot,
    )
    .await
    {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project notification settings update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(chat_updated_event) =
        chat_notification_settings_chat_updated_event(account_id, &chat, snapshot, Utc::now())
    else {
        return;
    };
    let Ok(event) =
        chat_notification_settings_updated_event(account_id, &chat, snapshot, Utc::now())
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&chat_updated_event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append notification chat-updated event");
    }
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append notification settings event");
    }

    let _ = event_bus.broadcast(chat_updated_event);
    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_position_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatPositionSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let (chat, reconciled) = match apply_chat_position_update(store, account_id, snapshot).await {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project chat position update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let occurred_at = Utc::now();
    let Ok(chat_updated_event) =
        chat_position_updated_event(account_id, &chat, snapshot, occurred_at)
    else {
        return;
    };
    let pin_event = if matches!(snapshot.list_kind.as_str(), "main" | "archive") {
        match chat_pinned_updated_event(account_id, &chat, snapshot, occurred_at) {
            Ok(event) => Some(event),
            Err(_) => return,
        }
    } else {
        None
    };
    let Ok(archive_event) = chat_archived_updated_event(account_id, &chat, snapshot, occurred_at)
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&chat_updated_event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append position chat-updated event");
    }
    if let Some(event) = &pin_event
        && let Err(error) = event_store.append(event).await
    {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append pinned chat event");
    }
    if let Err(error) = event_store.append(&archive_event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append archived chat event");
    }

    let _ = event_bus.broadcast(chat_updated_event);
    if let Some(event) = pin_event {
        let _ = event_bus.broadcast(event);
    }
    let _ = event_bus.broadcast(archive_event);

    if snapshot.list_kind == "folder"
        && let Ok(items) = store.list_chat_group_filters(Some(account_id)).await
    {
        publish_chat_group_filters_event(pool, event_bus, account_id, &items).await;
    }
}

pub(super) async fn publish_chat_removed_from_list_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatRemovedFromListSnapshot,
) {
    let removal_snapshot = TelegramTdlibChatPositionSnapshot {
        provider_chat_id: snapshot.provider_chat_id.clone(),
        list_kind: snapshot.list_kind.clone(),
        provider_folder_id: snapshot.provider_folder_id,
        order: 0,
        is_pinned: false,
        source_event: snapshot.source_event.clone(),
    };

    publish_chat_position_event(telegram_store, event_bus, account_id, &removal_snapshot).await;
}

pub(super) async fn publish_chat_folders_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    folders: &[TelegramTdlibChatFolderSnapshot],
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let update = match apply_chat_folder_update(store, account_id, folders).await {
        Ok(items) => items,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project chat folder update");
            return;
        }
    };

    for chat in &update.chats {
        let Ok(event) = chat_folder_labels_updated_event(account_id, chat, Utc::now()) else {
            continue;
        };
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append folder-label chat event");
        }
        let _ = event_bus.broadcast(event);
    }

    publish_chat_group_filters_event(pool, event_bus, account_id, &update.filters).await;
}

async fn publish_chat_group_filters_event(
    pool: &PgPool,
    event_bus: &EventBus,
    account_id: &str,
    items: &[TelegramChatGroupFilter],
) {
    let now = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_telegram_folders_updated_{}_{}",
            account_id,
            now.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::FOLDERS_UPDATED.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": account_id, "kind": "telegram_account"}),
    )
    .payload(json!({
        "account_id": account_id,
        "items": items,
    }))
    .build();

    let Ok(event) = event else {
        return;
    };
    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append folders event");
    }
    let _ = event_bus.broadcast(event);
}

async fn apply_chat_unread_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatUnreadSnapshot,
) -> Result<Option<(TelegramChat, Vec<TelegramProviderWriteCommand>)>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    store
        .apply_provider_unread_counts(
            &chat.telegram_chat_id,
            snapshot.unread_count,
            snapshot.unread_mention_count,
            snapshot.last_read_inbox_message_id.as_deref(),
            &snapshot.source_event,
        )
        .await?;

    let reconciled =
        if let Some(last_read_inbox_message_id) = snapshot.last_read_inbox_message_id.as_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/chat_events/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/chat_events/tests.rs`
- Size bytes / Размер в байтах: `19443`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;
use sqlx::PgPool;
use testkit::context::TestContext;

use super::{
    publish_chat_folders_event, publish_chat_notification_settings_event,
    publish_chat_position_event, publish_chat_unread_event,
};
use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::models::{
    NewTelegramChat, TelegramChatKind, TelegramSyncState,
};
use crate::integrations::telegram::client::{TelegramChat, TelegramError};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatUnreadSnapshot,
};
use crate::platform::events::EventBus;
use crate::platform::events::bus::telegram_event_types;

#[cfg(test)]
mod archive_reconciliation;
#[cfg(test)]
mod mark_unread_reconciliation;
#[cfg(test)]
mod pin_mute_reconciliation;

async fn seed_chat(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
) -> Result<TelegramChat, TelegramError> {
    crate::test_support::upsert_telegram_runtime_account(
        pool,
        account_id,
        "Runtime Chat Account",
        &format!("telegram-ext-{account_id}"),
    )
    .await;
    crate::test_support::telegram_store(pool)
        .upsert_chat(&NewTelegramChat {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            chat_kind: TelegramChatKind::Private,
            title: "Runtime Chat".to_owned(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: None,
            metadata: json!({}),
        })
        .await
}

#[tokio::test]
async fn publish_chat_notification_settings_event_appends_chat_updated_before_chat_muted() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-1";
    let provider_chat_id = "chat-1";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = EventBus::new();
    let snapshot = TelegramTdlibChatNotificationSettingsSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        use_default_mute_for: false,
        mute_for: 3600,
        source_event: "updateChatNotificationSettings".to_owned(),
    };

    publish_chat_notification_settings_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    let rows: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.chat.updated', 'telegram.chat.muted')
        ORDER BY position ASC
        "#,
    )
    .bind(&chat.telegram_chat_id)
    .fetch_all(&pool)
    .await
    .expect("notification events");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, telegram_event_types::CHAT_UPDATED);
    assert_eq!(
        rows[0].1["action"],
        json!("provider_notification_settings_update")
    );
    assert_eq!(rows[0].1["chat"]["metadata"]["is_muted"], json!(true));
    assert_eq!(rows[1].0, telegram_event_types::CHAT_MUTED);
    assert_eq!(rows[1].1["is_muted"], json!(true));
}

#[tokio::test]
async fn publish_chat_position_event_appends_chat_updated_before_flag_events() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-1";
    let provider_chat_id = "chat-2";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = EventBus::new();
    let snapshot = TelegramTdlibChatPositionSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        list_kind: "archive".to_owned(),
        provider_folder_id: Some(7),
        order: 42,
        is_pinned: true,
        source_event: "updateChatPosition".to_owned(),
    };

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    let rows: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN (
            'telegram.chat.updated',
            'telegram.chat.pinned',
            'telegram.chat.archived'
          )
        ORDER BY position ASC
        "#,
    )
    .bind(&chat.telegram_chat_id)
    .fetch_all(&pool)
    .await
    .expect("position events");

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, telegram_event_types::CHAT_UPDATED);
    assert_eq!(rows[0].1["action"], json!("provider_chat_position_update"));
    assert_eq!(rows[0].1["chat"]["metadata"]["is_archived"], json!(true));
    assert_eq!(rows[0].1["chat"]["metadata"]["is_pinned"], json!(true));
    assert_eq!(rows[1].0, telegram_event_types::CHAT_PINNED);
    assert_eq!(rows[1].1["is_pinned"], json!(true));
    assert_eq!(rows[2].0, telegram_event_types::CHAT_ARCHIVED);
    assert_eq!(rows[2].1["is_archived"], json!(true));
}

#[tokio::test]
async fn publish_chat_position_event_emits_folder_filters_for_folder_membership_changes() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-folder";
    let provider_chat_id = "chat-folder";
    let _chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = EventBus::new();
    let snapshot = TelegramTdlibChatPositionSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        list_kind: "folder".to_owned(),
        provider_folder_id: Some(7),
        order: 42,
        is_pinned: false,
        source_event: "updateChatPosition".to_owned(),
    };

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    let row: (String, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE event_type = 'telegram.folders.updated'
          AND payload->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("folders updated event");

    assert_eq!(row.0, telegram_event_types::FOLDERS_UPDATED);
    let items = row.1["items"].as_array().expect("folder items");
    assert!(items.iter().any(|item| item["id"] == json!("local:all")));
    assert!(items.iter().any(|item| {
        item["id"] == json!("folder:Unknown folder 7")
            && item["provider_folder_id"] == json!(7)
            && item["count"] == json!(1)
    }));
}

#[tokio::test]
async fn publish_chat_folders_event_emits_chat_updated_for_folder_label_projection_changes() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-folder-labels";
    let provider_chat_id = "chat-folder-labels";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = EventBus::new();

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "folder".to_owned(),
            provider_folder_id: Some(7),
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    publish_chat_folders_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &[TelegramTdlibChatFolderSnapshot {
            provider_folder_id: 7,
            title: "Projects".to_owned(),
            icon_name: None,
            color_id: None,
            raw: json!({
                "@type": "chatFolder",
                "id": 7,
                "name": { "@type": "formattedText", "text": "Projects" },
            }),
        }],
    )
    .await;

    let row: (String, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type = 'telegram.chat.updated'
          AND payload->>'action' = 'provider_chat_folder_labels_update'
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&chat.telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("folder label chat update event");

    assert_eq!(row.0, telegram_event_types::CHAT_UPDATED);
    assert_eq!(row.1["provider_folder_id"], json!(7));
    assert_eq!(row.1["folder_labels"], json!(["Projects"]));
    assert_eq!(row.1["chat"]["metadata"]["folder_name"], json!("Projects"));
    assert_eq!(row.1["chat"]["metadata"]["provider_folder_id"], json!(7));
}

#[tokio::test]
async fn publish_chat_folders_event_refreshes_unknown_labels_when_folder_snapshot_disappears() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-folder-missing";
    let provider_chat_id = "chat-folder-missing";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = EventBus::new();

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "folder".to_owned(),
            provider_folder_id: Some(7),
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    publish_chat_folders_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &[TelegramTdlibChatFolderSnapshot {
            provider_folder_id: 7,
            title: "Projects".to_owned(),
            icon_name: None,
            color_id: None,
            raw: json!({
                "@type": "chatFolder",
                "id": 7,
                "name": { "@type": "formattedText", "text": "Projects" },
            }),
        }],
    )
    .await;

    publish_chat_folders_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &[],
    )
    .await;

    let row: (String, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type = 'telegram.chat.updated'
          AND payload->>'action' = 'provider_chat_folder_labels_update'
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&chat.telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("fallback folder label event");

    assert_eq!(row.0, telegram_event_types::CHAT_UPDATED);
    assert_eq!(row.1["folder_labels"], json!(["Unknown folder 7"]));
    assert_eq!(row.1["provider_folder_id"], json!(7));
    assert_eq!(
        row.1["chat"]["metadata"]["folder_name"],
        json!("Unknown folder 7")
    );
    assert_eq!(row.1["chat"]["metadata"]["provider_folder_id"], json!(7));
}

#[tokio::test]
async fn publish_chat_position_event_reconciles_folder_add_and_remove_commands() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-folder-reconcile";
    let provider_chat_id = "chat-folder-reconcile";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = EventBus::new();

    let add_command_id = "cmd-folder-add-1";
    let remove_command_id = "cmd-folder-remove-1";
    insert_command(
        &pool,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/command_executor.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/command_executor.rs`
- Size bytes / Размер в байтах: `17006`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

use super::TelegramRuntimeManager;
use super::command_executor_dispatch::{DispatchOutcome, dispatch_command};
use super::command_executor_media::{emit_media_upload_event, media_upload_progress_payload};
use super::realtime_events::command_event_payload;
use super::topic_events::upsert_topic_snapshot;

const RETRY_BASE_DELAY_SECONDS: i64 = 30;
const RETRY_MAX_DELAY_SECONDS: i64 = 15 * 60;
const STALE_EXECUTION_LOCK_SECONDS: i64 = 120;

/// Processes due provider-write commands for active Telegram account actors.
///
/// Actor dispatch does not mean provider success. Commands that only get an ACK
/// from TDLib stay `executing` with `reconciliation_status=awaiting_provider`;
/// `completed` is reserved for provider-observed state or TDLib calls that
/// return a provider message snapshot.
pub async fn execute_queued_commands(
    telegram_store: &TelegramStore,
    runtime: &TelegramRuntimeManager,
    event_bus: &EventBus,
    per_account_limit: i64,
) {
    let pool = telegram_store.pool();
    let now = Utc::now();
    let stale_before = now - Duration::seconds(STALE_EXECUTION_LOCK_SECONDS);
    match lifecycle::recover_stale_executing_commands(pool, now, stale_before).await {
        Ok(commands) => {
            for command in commands {
                let status = command.status.clone();
                emit_command_event(
                    event_bus,
                    pool,
                    &command,
                    &status,
                    json!({"source": "stale_recovery", "error": command.last_error.clone()}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "command executor: failed to recover stale commands");
        }
    }

    let account_ids = match runtime.active_account_ids() {
        Ok(ids) => ids,
        Err(error) => {
            tracing::warn!(error = %error, "command executor: failed to list active accounts");
            return;
        }
    };

    for account_id in account_ids {
        execute_account_commands(
            telegram_store,
            runtime,
            event_bus,
            &account_id,
            per_account_limit,
        )
        .await;
    }
}

async fn execute_account_commands(
    telegram_store: &TelegramStore,
    runtime: &TelegramRuntimeManager,
    event_bus: &EventBus,
    account_id: &str,
    limit: i64,
) {
    let pool = telegram_store.pool();
    let now = Utc::now();
    let commands =
        match lifecycle::claim_due_commands_for_execution(pool, account_id, now, limit).await {
            Ok(commands) => commands,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    account_id = %account_id,
                    "command executor: failed to claim due commands"
                );
                return;
            }
        };

    if commands.is_empty() {
        return;
    }

    let command_tx = match runtime.actor_command_tx(account_id) {
        Ok(Some(tx)) => tx,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(
                error = %error,
                account_id = %account_id,
                "command executor: failed to get actor command channel"
            );
            return;
        }
    };

    for command in commands {
        emit_command_event(
            event_bus,
            pool,
            &command,
            "executing",
            json!({"source": "command_executor", "phase": "claimed"}),
        )
        .await;
        if command.command_kind == "send_media" {
            emit_media_upload_event(
                event_bus,
                pool,
                &command,
                telegram_event_types::MEDIA_UPLOAD_PROGRESS,
                media_upload_progress_payload(
                    &command,
                    "dispatching_to_provider",
                    "Uploading local media to Telegram",
                ),
            )
            .await;
        }

        let result = dispatch_command(pool, &command, command_tx.clone()).await;
        handle_dispatch_result(telegram_store, event_bus, &command, result).await;
    }
}

async fn handle_dispatch_result(
    telegram_store: &TelegramStore,
    event_bus: &EventBus,
    command: &TelegramProviderWriteCommand,
    result: Result<DispatchOutcome, TelegramError>,
) {
    let pool = telegram_store.pool();
    let now = Utc::now();
    match result {
        Ok(DispatchOutcome::ObservedMessage(snapshot)) => {
            let import_batch_id = format!(
                "telegram-command:{}:{}",
                command.account_id,
                command.command_id.trim()
            );
            let projection = match telegram_store
                .ingest_tdlib_message_snapshot(&command.account_id, &snapshot, &import_batch_id)
                .await
            {
                Ok(result) => result,
                Err(error) => {
                    handle_command_error(pool, event_bus, command, error, now).await;
                    return;
                }
            };
            if let Err(error) = telegram_store
                .publish_observed_message_raw_signal(&projection, Some(event_bus))
                .await
            {
                handle_command_error(pool, event_bus, command, error, now).await;
                return;
            }
            let provider_state = json!({
                "provider_chat_id": snapshot.provider_chat_id,
                "provider_message_id": snapshot.provider_message_id,
                "delivery_state": snapshot.delivery_state.as_str(),
                "observed_via": "tdlib_returned_message",
                "raw_record_id": projection.raw_record_id.clone(),
                "message_id": projection.message_id.clone(),
            });
            let result_payload = json!({
                "provider_chat_id": snapshot.provider_chat_id,
                "provider_message_id": snapshot.provider_message_id,
                "delivery_state": snapshot.delivery_state.as_str(),
                "raw_record_id": projection.raw_record_id.clone(),
                "message_id": projection.message_id.clone(),
            });
            if let Err(error) = lifecycle::mark_command_reconciled(
                pool,
                &command.command_id,
                now,
                provider_state,
                result_payload,
            )
            .await
            {
                tracing::warn!(
                    error = %error,
                    command_id = %command.command_id,
                    "command executor: failed to mark command reconciled"
                );
            }
            let reconciled_event_payload = json!({
                "source": "command_executor",
                "reconciliation_status": "observed",
                "provider_observed_at": now,
                "reconciled_at": now,
            });
            emit_command_event(
                event_bus,
                pool,
                command,
                "completed",
                reconciled_event_payload.clone(),
            )
            .await;
            emit_command_event_type(
                event_bus,
                pool,
                command,
                telegram_event_types::COMMAND_RECONCILED,
                "completed",
                reconciled_event_payload,
            )
            .await;
            if command.command_kind == "send_media" {
                emit_media_upload_event(
                    event_bus,
                    pool,
                    command,
                    telegram_event_types::MEDIA_UPLOAD_COMPLETED,
                    json!({
                        "status": "completed",
                        "provider_message_id": snapshot.provider_message_id,
                        "delivery_state": snapshot.delivery_state.as_str(),
                        "message_id": projection.message_id,
                    }),
                )
                .await;
            }
        }
        Ok(DispatchOutcome::ObservedTopic(snapshot)) => {
            let topic = match upsert_topic_snapshot(
                telegram_store,
                &command.account_id,
                &command.provider_chat_id,
                &snapshot,
            )
            .await
            {
                Ok(Some(topic)) => topic,
                Ok(None) => {
                    handle_command_error(
                        pool,
                        event_bus,
                        command,
                        TelegramError::InvalidRequest(
                            "topic create observed for unknown telegram chat".to_owned(),
                        ),
                        now,
                    )
                    .await;
                    return;
                }
                Err(error) => {
                    handle_command_error(pool, event_bus, command, error, now).await;
                    return;
                }
            };
            let provider_state = json!({
                "provider_chat_id": command.provider_chat_id,
                "provider_topic_id": snapshot.provider_topic_id,
                "topic_id": topic.topic_id,
                "title": topic.title,
                "is_closed": topic.is_closed,
                "observed_via": "tdlib_returned_topic",
            });
            let result_payload = json!({
                "provider_chat_id": command.provider_chat_id,
                "provider_topic_id": snapshot.provider_topic_id,
                "topic_id": topic.topic_id,
                "title": topic.title,
                "is_closed": topic.is_closed,
            });
            if let Err(error) = lifecycle::mark_command_reconciled(
                pool,
                &command.command_id,
                now,
                provider_state,
                result_payload,
            )
            .await
            {
                tracing::warn!(
                    error = %error,
                    command_id = %command.command_id,
                    "command executor: failed to mark topic command reconciled"
                );
            }
            emit_command_event(
                event_bus,
                pool,
                command,
                "completed",
                json!({
                    "source": "command_executor",
                    "reconciliation_status": "observed",
                    "provider_observed_at": now,
                    "reconciled_at": now,
                    "provider_topic_id": snapshot.provider_topic_id,
                    "topic_id": topic.topic_id,
                }),
            )
            .await;
            emit_command_event_type(
                event_bus,
                pool,
                command,
                telegram_event_types::COMMAND_RECONCILED,
                "completed",
                json!({
                    "source": "command_executor",
                    "reconciliation_status": "observed",
                    "provider_observed_at": now,
                    "reconciled_at": now,
                    "provider_topic_id": snapshot.provider_topic_id,
                    "topic_id": topic.topic_id,
                }),
            )
            .await;
        }
        Ok(DispatchOutcome::AwaitingProvider) => {
            if let Err(error) = lifecycle::mark_command_awaiting_provider(
                pool,
                &command.co
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/command_executor_dispatch.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/command_executor_dispatch.rs`
- Size bytes / Размер в байтах: `10318`
- Included characters / Включено символов: `10318`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::PgPool;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::client::TelegramManualSendRequest;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::tdjson::{
    TelegramTdlibMessageSnapshot, TelegramTdlibTopicSnapshot,
};

use super::super::commands::{
    request_actor_add_chat_to_folder, request_actor_create_forum_topic,
    request_actor_delete_message, request_actor_edit_message, request_actor_forward,
    request_actor_join_chat, request_actor_leave_chat, request_actor_pin_message,
    request_actor_remove_chat_from_folder, request_actor_reply, request_actor_send,
    request_actor_send_media, request_actor_set_reaction, request_actor_toggle_chat_archive,
    request_actor_toggle_chat_mute, request_actor_toggle_chat_unread,
    request_actor_toggle_forum_topic_closed,
};
use super::super::models::{TelegramMediaSendRequest, TelegramMediaSendType};
use super::super::state::TelegramRuntimeCommand;

pub(super) enum DispatchOutcome {
    AwaitingProvider,
    ObservedMessage(TelegramTdlibMessageSnapshot),
    ObservedTopic(TelegramTdlibTopicSnapshot),
}

pub(super) async fn dispatch_command(
    _pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    command_tx: std::sync::mpsc::Sender<TelegramRuntimeCommand>,
) -> Result<DispatchOutcome, TelegramError> {
    match command.command_kind.as_str() {
        "send_text" => {
            let snapshot = request_actor_send(
                command_tx,
                TelegramManualSendRequest {
                    command_id: command.command_id.clone(),
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    text: payload_string(command, "text")?,
                },
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "send_media" => {
            let request = TelegramMediaSendRequest {
                command_id: command.command_id.clone(),
                provider_chat_id: command.provider_chat_id.clone(),
                media_type: TelegramMediaSendType::try_from(
                    payload_string(command, "media_type")?.as_str(),
                )?,
                local_path: payload_string(command, "local_path")?,
                caption: payload_optional_string(command, "caption"),
                filename: payload_optional_string(command, "filename"),
            };
            let snapshot = request_actor_send_media(command_tx, request).await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "reply" => {
            let snapshot = request_actor_reply(
                command_tx,
                command.provider_chat_id.clone(),
                payload_string(command, "reply_to_provider_message_id")?,
                payload_string(command, "text")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "forward" => {
            let snapshot = request_actor_forward(
                command_tx,
                command.provider_chat_id.clone(),
                payload_string(command, "from_provider_chat_id")?,
                payload_string(command, "from_provider_message_id")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "edit" => {
            request_actor_edit_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, "edit")?,
                payload_string(command, "new_text")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "delete" => {
            request_actor_delete_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, "delete")?,
                command
                    .payload
                    .get("is_provider_delete")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "react" | "unreact" => {
            let is_active = command.command_kind == "react";
            request_actor_set_reaction(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, &command.command_kind)?,
                payload_string(command, "reaction_emoji")?,
                is_active,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "pin" | "unpin" => {
            let pin = command.command_kind == "pin";
            request_actor_pin_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, &command.command_kind)?,
                pin,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "mark_read" | "mark_unread" => {
            request_actor_toggle_chat_unread(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "mark_unread",
                command.provider_message_id.clone(),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "archive" | "unarchive" => {
            request_actor_toggle_chat_archive(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "archive",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "mute" | "unmute" => {
            request_actor_toggle_chat_mute(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "mute",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "folder_add" => {
            request_actor_add_chat_to_folder(
                command_tx,
                command.provider_chat_id.clone(),
                payload_i64(command, "provider_folder_id")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "folder_remove" => {
            request_actor_remove_chat_from_folder(
                command_tx,
                command.provider_chat_id.clone(),
                payload_i64(command, "provider_folder_id")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "join" => {
            request_actor_join_chat(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "leave" => {
            request_actor_leave_chat(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "topic_create" => {
            let snapshot = request_actor_create_forum_topic(
                command_tx,
                command.provider_chat_id.clone(),
                payload_string(command, "title")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::ObservedTopic(snapshot))
        }
        "topic_close" | "topic_reopen" => {
            request_actor_toggle_forum_topic_closed(
                command_tx,
                command.provider_chat_id.clone(),
                payload_i64(command, "provider_topic_id")?,
                command.command_kind == "topic_close",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        other => Err(TelegramError::InvalidRequest(format!(
            "command executor: unsupported command kind `{other}`"
        ))),
    }
}

fn payload_string(
    command: &TelegramProviderWriteCommand,
    key: &str,
) -> Result<String, TelegramError> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{} command missing `{key}`",
                command.command_kind
            ))
        })
}

fn payload_optional_string(command: &TelegramProviderWriteCommand, key: &str) -> Option<String> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn payload_i64(command: &TelegramProviderWriteCommand, key: &str) -> Result<i64, TelegramError> {
    command
        .payload
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{} command missing numeric `{key}`",
                command.command_kind
            ))
        })
}

fn provider_message_id(
    command: &TelegramProviderWriteCommand,
    operation: &str,
) -> Result<String, TelegramError> {
    command
        .provider_message_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{operation} command missing provider_message_id"
            ))
        })
}
```

### `backend/src/integrations/telegram/runtime/manager/command_executor_media.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/command_executor_media.rs`
- Size bytes / Размер в байтах: `5776`
- Included characters / Включено символов: `5776`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

pub(super) async fn emit_media_upload_event(
    event_bus: &EventBus,
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    event_type: &str,
    extra_payload: Value,
) {
    let now = Utc::now();
    let mut payload = json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "provider_chat_id": command.provider_chat_id,
        "attachment_id": payload_optional_string(command, "attachment_id"),
        "blob_id": payload_optional_string(command, "blob_id"),
        "media_type": payload_optional_string(command, "media_type"),
        "caption_present": payload_optional_string(command, "caption").is_some(),
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    let event = NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_media_upload"}),
    )
    .payload(payload)
    .build();

    let Ok(event) = event else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "command executor: failed to append media upload event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) fn media_upload_progress_payload(
    command: &TelegramProviderWriteCommand,
    phase: &str,
    detail: &str,
) -> Value {
    let mut provider_state = command.provider_state.clone();
    if let Some(provider_state_obj) = provider_state.as_object_mut() {
        provider_state_obj.insert("upload_phase".to_owned(), Value::String(phase.to_owned()));
        provider_state_obj.insert(
            "progress_detail".to_owned(),
            Value::String(detail.to_owned()),
        );
    }
    json!({
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "provider_state": provider_state,
        "reconciliation_status": command.reconciliation_status,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
        "progress_phase": phase,
        "progress_detail": detail,
    })
}

fn payload_optional_string(command: &TelegramProviderWriteCommand, key: &str) -> Option<String> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::media_upload_progress_payload;
    use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;

    fn sample_command() -> TelegramProviderWriteCommand {
        TelegramProviderWriteCommand {
            command_id: "cmd-1".to_owned(),
            account_id: "account-1".to_owned(),
            command_kind: "send_media".to_owned(),
            idempotency_key: "idem-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            provider_message_id: None,
            target_ref: json!({}),
            payload: json!({"attachment_id": "att-1", "blob_id": "blob-1"}),
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "confirmed".to_owned(),
            status: "executing".to_owned(),
            retry_count: 1,
            max_retries: 3,
            last_error: None,
            result_payload: json!({}),
            audit_metadata: json!({}),
            actor_id: "makosh-frontend".to_owned(),
            happened_at: Utc::now(),
            next_attempt_at: None,
            last_attempt_at: None,
            locked_at: None,
            locked_by: None,
            provider_observed_at: None,
            provider_state: json!({"dispatch": "claimed"}),
            reconciliation_status: "not_observed".to_owned(),
            reconciled_at: None,
            dead_lettered_at: None,
            completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn media_upload_progress_payload_carries_phase_detail_in_provider_state() {
        let payload = media_upload_progress_payload(
            &sample_command(),
            "dispatching_to_provider",
            "Uploading local media to Telegram",
        );

        assert_eq!(payload["status"], "executing");
        assert_eq!(payload["progress_phase"], "dispatching_to_provider");
        assert_eq!(
            payload["progress_detail"],
            "Uploading local media to Telegram"
        );
        assert_eq!(payload["provider_state"]["dispatch"], "claimed");
        assert_eq!(
            payload["provider_state"]["upload_phase"],
            "dispatching_to_provider"
        );
        assert_eq!(
            payload["provider_state"]["progress_detail"],
            "Uploading local media to Telegram"
        );
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/lifecycle.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/lifecycle.rs`
- Size bytes / Размер в байтах: `5418`
- Included characters / Включено символов: `5418`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;

use crate::integrations::telegram::client::TelegramError;
use crate::platform::config::AppConfig;

use super::super::actor::{optional_telegram_session_key, spawn_tdlib_actor};
use super::super::models::{
    TelegramRuntimeRestartRequest, TelegramRuntimeStartRequest, TelegramRuntimeStatus,
    TelegramRuntimeStopRequest,
};
use super::super::state::{
    TelegramRuntimeActorHandle, TelegramRuntimeActorState, TelegramRuntimeState,
};
use super::super::status::{account_runtime_kind, load_telegram_account, status_from_account};
use super::account::load_active_account;
use super::actor_states::running_actor_state;
use super::realtime_events::spawn_telegram_runtime_event_bridge;
use super::{TelegramRuntimeManager, TelegramRuntimeStartContext};
use crate::platform::communications::ProviderAccountLookupPort;

impl TelegramRuntimeManager {
    pub async fn status_for_account(
        &self,
        provider_account_store: &dyn ProviderAccountLookupPort,
        config: &AppConfig,
        account_id: &str,
    ) -> Result<TelegramRuntimeStatus, TelegramError> {
        let account = load_telegram_account(provider_account_store, account_id).await?;
        let actor_state = self.actor_state(&account.account_id)?;

        Ok(status_from_account(config, &account, actor_state))
    }

    pub(crate) async fn start_account<S>(
        &self,
        context: &TelegramRuntimeStartContext<'_, S>,
        request: &TelegramRuntimeStartRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let session_encryption_key = optional_telegram_session_key(
            context.provider_secret_binding_store,
            context.secret_store,
            context.secret_resolver,
            &account.account_id,
        )
        .await?;
        let runtime_kind = account_runtime_kind(&account);
        let now = Utc::now();
        let (actor_state, command_tx) = match runtime_kind.as_str() {
            "fixture" => running_actor_state(now).without_command(),
            "tdlib_qr_authorized" => {
                let (runtime_event_tx, runtime_event_rx) = tokio::sync::mpsc::unbounded_channel();
                let result = match spawn_tdlib_actor(
                    context.config.clone(),
                    account.clone(),
                    session_encryption_key,
                    Some(runtime_event_tx),
                ) {
                    Ok(command_tx) => running_actor_state(now).with_command(command_tx),
                    Err(error) => TelegramRuntimeActorState {
                        status: TelegramRuntimeState::Degraded,
                        last_error: Some(error.to_string()),
                        updated_at: now,
                    }
                    .without_command(),
                };
                if result.1.is_some() {
                    spawn_telegram_runtime_event_bridge(
                        Some(context.telegram_store.clone()),
                        context.event_bus.clone(),
                        account.account_id.clone(),
                        runtime_event_rx,
                    );
                }
                result
            }
            "live_blocked" => TelegramRuntimeActorState {
                status: TelegramRuntimeState::Blocked,
                last_error: Some(
                    "account runtime is blocked until live TDLib is enabled".to_owned(),
                ),
                updated_at: now,
            }
            .without_command(),
            other => TelegramRuntimeActorState {
                status: TelegramRuntimeState::Error,
                last_error: Some(format!("unsupported Telegram runtime `{other}`")),
                updated_at: now,
            }
            .without_command(),
        };

        self.set_actor_handle(
            account.account_id.clone(),
            TelegramRuntimeActorHandle {
                state: actor_state.clone(),
                command_tx,
            },
        )?;

        Ok(status_from_account(
            context.config,
            &account,
            Some(actor_state),
        ))
    }

    pub async fn stop_account_runtime(
        &self,
        provider_account_store: &dyn ProviderAccountLookupPort,
        config: &AppConfig,
        request: &TelegramRuntimeStopRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError> {
        request.validate()?;
        let account = load_telegram_account(provider_account_store, &request.account_id).await?;
        self.stop_account(&account.account_id)?;

        Ok(status_from_account(config, &account, None))
    }

    pub(crate) async fn restart_account_runtime<S>(
        &self,
        context: &TelegramRuntimeStartContext<'_, S>,
        request: &TelegramRuntimeRestartRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        self.stop_account(&request.account_id)?;
        self.start_account(
            context,
            &TelegramRuntimeStartRequest {
                account_id: request.account_id.clone(),
            },
        )
        .await
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/media_download.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/media_download.rs`
- Size bytes / Размер в байтах: `3291`
- Included characters / Включено символов: `3291`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;

use super::super::commands::request_actor_download_file;
use super::super::models::{TelegramMediaDownloadRequest, TelegramMediaDownloadResponse};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramMediaDownloadContext, TelegramRuntimeManager};
use crate::platform::secrets::SecretResolver;

impl TelegramRuntimeManager {
    pub(crate) async fn download_media<S: SecretResolver + Sync + ?Sized>(
        &self,
        context: TelegramMediaDownloadContext<'_, S>,
        request: &TelegramMediaDownloadRequest,
    ) -> Result<TelegramMediaDownloadResponse, TelegramError> {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => Err(TelegramError::InvalidRequest(
                "Telegram media downloads require an enabled TDLib actor".to_owned(),
            )),
            "tdlib_qr_authorized" => {
                let command_tx = self
                    .ensure_tdlib_actor(
                        context.provider_secret_binding_store,
                        context.secret_store,
                        context.secret_resolver,
                        context.config,
                        &account,
                        context.event_bridge.clone(),
                    )
                    .await?;
                let file = request_actor_download_file(
                    command_tx,
                    request.tdlib_file_id,
                    request.priority.unwrap_or(16),
                )
                .await?;
                Ok(TelegramMediaDownloadResponse {
                    account_id: request.account_id.clone(),
                    provider_chat_id: request.provider_chat_id.clone(),
                    provider_message_id: request.provider_message_id.clone(),
                    runtime_kind,
                    status: if file.is_downloading_completed {
                        "downloaded".to_owned()
                    } else if file.is_downloading_active {
                        "downloading".to_owned()
                    } else {
                        "remote".to_owned()
                    },
                    tdlib_file_id: file.file_id,
                    local_path: file.local_path,
                    size_bytes: file.size_bytes,
                    expected_size_bytes: file.expected_size_bytes,
                    downloaded_size_bytes: file.downloaded_size_bytes,
                    is_downloading_active: file.is_downloading_active,
                    is_downloading_completed: file.is_downloading_completed,
                    attachment_id: None,
                    blob_id: None,
                    scan_status: None,
                })
            }
            "live_blocked" => Err(TelegramError::InvalidRequest(
                "account runtime is blocked until live TDLib is enabled".to_owned(),
            )),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram runtime `{other}`"
            ))),
        }
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/message_events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/message_events.rs`
- Size bytes / Размер в байтах: `15704`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::Utc;
use serde_json::json;

use crate::integrations::telegram::client::lifecycle::{
    reconcile_delete_commands_from_provider_state, reconcile_edit_commands_from_provider_state,
    reconcile_message_pin_commands_from_provider_state, record_provider_delete_observation,
    record_provider_edit_observation,
};
use crate::integrations::telegram::client::{
    TelegramReactionMessageRef, TelegramStore, derive_tdlib_chosen_reaction_emojis,
    derive_tdlib_provider_reactions, derive_tdlib_reaction_summary_metadata,
    reconcile_reaction_commands_from_provider_reactions, sync_provider_reactions,
};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibMessageContentSnapshot, TelegramTdlibMessageDeleteSnapshot,
    TelegramTdlibMessageEditedSnapshot, TelegramTdlibMessageInteractionInfoSnapshot,
    TelegramTdlibMessagePinnedSnapshot, TelegramTdlibMessageSnapshot,
};
use crate::platform::events::EventBus;

use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};

mod envelopes;
mod projection;
#[cfg(test)]
mod tests;

use envelopes::{
    append_and_broadcast, message_deleted_event, message_updated_event, reaction_changed_event,
};
use projection::{
    observed_edit_timestamp, project_provider_message_content_observation,
    project_provider_message_edit_observation, update_message_reaction_summary,
};

pub(super) async fn publish_message_created_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };

    let import_batch_id = format!(
        "telegram-tdlib-runtime:{}:{}",
        account_id, snapshot.provider_chat_id
    );
    let projection = match store
        .ingest_tdlib_message_snapshot(account_id, snapshot, &import_batch_id)
        .await
    {
        Ok(result) => result,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to ingest message create");
            return;
        }
    };
    if let Err(error) = store
        .publish_observed_message_raw_signal(&projection, Some(event_bus))
        .await
    {
        tracing::warn!(error = %error, account_id, "Telegram runtime event bridge: failed to publish Signal Hub raw create event");
    }
}

pub(super) async fn publish_message_deleted_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageDeleteSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for provider_message_id in &snapshot.provider_message_ids {
        let provider_message_ref = format!("{}:{}", snapshot.provider_chat_id, provider_message_id);
        let message = match store
            .message_by_provider_message_id(account_id, &provider_message_ref)
            .await
        {
            Ok(Some(message)) => message,
            Ok(None) => continue,
            Err(error) => {
                tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load deleted message");
                continue;
            }
        };
        let tombstone = match record_provider_delete_observation(
            pool,
            &message,
            Utc::now(),
            &snapshot.source_event,
            snapshot.is_permanent,
            snapshot.from_cache,
        )
        .await
        {
            Ok(tombstone) => tombstone,
            Err(error) => {
                tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to record provider tombstone");
                continue;
            }
        };
        let reconciled = match reconcile_delete_commands_from_provider_state(
            pool,
            account_id,
            &snapshot.provider_chat_id,
            &provider_message_ref,
            Utc::now(),
            &format!("tdlib.{}", snapshot.source_event),
        )
        .await
        {
            Ok(commands) => commands,
            Err(error) => {
                tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to reconcile delete commands");
                Vec::new()
            }
        };
        for command in reconciled {
            publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event)
                .await;
        }

        let Ok(event) = message_deleted_event(account_id, &message, &tombstone, Utc::now()) else {
            continue;
        };
        append_and_broadcast(Some(pool.clone()), event_bus, event).await;
    }
}

pub(super) async fn publish_message_content_updated_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageContentSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let provider_message_ref = format!(
        "{}:{}",
        snapshot.provider_chat_id, snapshot.provider_message_id
    );
    let Some(message) = (match store
        .message_by_provider_message_id(account_id, &provider_message_ref)
        .await
    {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load edited message");
            return;
        }
    }) else {
        return;
    };

    let previous_text = message.text.clone();
    let observed_at = Utc::now();
    let updated_message = match project_provider_message_content_observation(
        store,
        &message,
        snapshot,
        observed_at,
    )
    .await
    {
        Ok(Some(message)) => message,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to project message content update");
            return;
        }
    };

    if updated_message.text != previous_text
        && let Err(error) = record_provider_edit_observation(
            pool,
            &updated_message,
            &updated_message.text,
            observed_edit_timestamp(&updated_message, observed_at),
            &snapshot.source_event,
            json!({
                "previous_text": previous_text,
                "new_text": updated_message.text,
                "new_content": snapshot.new_content,
            }),
            json!({
                "provider": "telegram",
                "runtime": "tdlib",
                "source": snapshot.source_event,
            }),
        )
        .await
    {
        tracing::warn!(error = %error, message_id = %updated_message.message_id, "Telegram runtime event bridge: failed to record provider edit version");
    }

    let reconciled = match reconcile_edit_commands_from_provider_state(
        pool,
        account_id,
        &snapshot.provider_chat_id,
        &provider_message_ref,
        &updated_message.text,
        observed_at,
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await
    {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %updated_message.message_id, "Telegram runtime event bridge: failed to reconcile edit commands");
            Vec::new()
        }
    };
    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = message_updated_event(
        account_id,
        &updated_message,
        json!({
            "text_changed": updated_message.text != previous_text,
            "provider_edit_timestamp": updated_message.metadata.get("provider_edit_timestamp").cloned(),
            "source": format!("tdlib.{}", snapshot.source_event),
        }),
        observed_at,
    ) else {
        return;
    };
    append_and_broadcast(Some(pool.clone()), event_bus, event).await;
}

pub(super) async fn publish_message_edited_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageEditedSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let provider_message_ref = format!(
        "{}:{}",
        snapshot.provider_chat_id, snapshot.provider_message_id
    );
    let Some(message) = (match store
        .message_by_provider_message_id(account_id, &provider_message_ref)
        .await
    {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load message edit metadata target");
            return;
        }
    }) else {
        return;
    };

    let updated_message = match project_provider_message_edit_observation(store, &message, snapshot)
        .await
    {
        Ok(Some(message)) => message,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to project message edit metadata");
            return;
        }
    };

    let Ok(event) = message_updated_event(
        account_id,
        &updated_message,
        json!({
            "edit_timestamp": snapshot.edit_timestamp,
            "reply_markup_present": snapshot.reply_markup.is_some(),
            "source": format!("tdlib.{}", snapshot.source_event),
        }),
        Utc::now(),
    ) else {
        return;
    };
    append_and_broadcast(Some(pool.clone()), event_bus, event).await;
}

pub(super) async fn publish_message_pinned_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessagePinnedSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let provider_message_ref = format!(
        "{}:{}",
        snapshot.provider_chat_id, snapshot.provider_message_id
    );
    let Some(message) = (match store
        .message_by_provider_message_id(account_id, &provider_message_ref)
        .await
    {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load message pin target");
            return;
        }
    }) else {
        return;
    };

    let observed_at = Utc::now();
    if let Err(error) = store
        .append_message_pin_observation(&message, snapshot.is_pinned, observed_at)
        .await
    {
        tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to append message pin observation");
        return;
    }
    let mut updated_message = message;
    if let Some(metadata) = updated_message.metadata.as_object_mut() {
        metadata.insert(
            "is_pinned".to_owned(),
            serde_json::Value::Bool(snapshot.is_pinned),
        );
    }

    let reconciled = match reconcile_message_pin_commands_from_provider_state(
        pool,
        account_id,
        &snapshot.provider_chat_id,
        &provider_message_ref,
        snapshot.is_pinned,
        observed_at,
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await
    {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %updated_message.message_id, "Telegram runtime event bridge: failed to reconcile message pin
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/message_events/envelopes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/message_events/envelopes.rs`
- Size bytes / Размер в байтах: `10118`
- Included characters / Включено символов: `10118`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::integrations::telegram::client::TelegramMessage;
use crate::integrations::telegram::client::models::messages::TelegramMessageTombstone;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventEnvelopeError, EventStore, NewEventEnvelope};

pub(super) async fn append_and_broadcast(
    pool: Option<PgPool>,
    event_bus: &EventBus,
    event: NewEventEnvelope,
) {
    if let Some(pool) = pool {
        let event_store = EventStore::new(pool);
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append message event");
        }
    }
    let _ = event_bus.broadcast(event);
}

pub(super) fn message_created_event(
    account_id: &str,
    message: &TelegramMessage,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_message_created_{}_{}_{}",
            account_id,
            message.message_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::MESSAGE_CREATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_message",
            "id": message.message_id,
            "provider_chat_id": message.provider_chat_id,
            "provider_message_id": message.provider_message_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "message_id": message.message_id,
        "provider_chat_id": message.provider_chat_id,
        "provider_message_id": message.provider_message_id,
        "message": message,
        "source": "tdlib.updateNewMessage"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateNewMessage"
    }))
    .build()
}

pub(super) fn message_deleted_event(
    account_id: &str,
    message: &TelegramMessage,
    tombstone: &TelegramMessageTombstone,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_message_deleted_{}_{}_{}",
            account_id,
            message.message_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::MESSAGE_DELETED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_message",
            "id": message.message_id,
            "provider_chat_id": message.provider_chat_id,
            "provider_message_id": message.provider_message_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "message_id": message.message_id,
        "provider_chat_id": message.provider_chat_id,
        "provider_message_id": message.provider_message_id,
        "message": message,
        "tombstone": tombstone,
        "source": "tdlib.updateDeleteMessages"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateDeleteMessages"
    }))
    .build()
}

pub(super) fn message_updated_event(
    account_id: &str,
    message: &TelegramMessage,
    extra_payload: Value,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, EventEnvelopeError> {
    let mut payload = json!({
        "account_id": account_id,
        "message_id": message.message_id,
        "provider_chat_id": message.provider_chat_id,
        "provider_message_id": message.provider_message_id,
        "message": message,
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }

    NewEventEnvelope::builder(
        format!(
            "evt_telegram_message_updated_{}_{}_{}",
            account_id,
            message.message_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::MESSAGE_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_message",
            "id": message.message_id,
            "provider_chat_id": message.provider_chat_id,
            "provider_message_id": message.provider_message_id
        }),
    )
    .payload(payload)
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
    }))
    .build()
}

pub(super) fn reaction_changed_event(
    account_id: &str,
    message: &TelegramMessage,
    reaction_summary: Option<Value>,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_reaction_changed_{}_{}_{}",
            account_id,
            message.message_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::REACTION_CHANGED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_message",
            "id": message.message_id,
            "provider_chat_id": message.provider_chat_id,
            "provider_message_id": message.provider_message_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "message_id": message.message_id,
        "provider_chat_id": message.provider_chat_id,
        "provider_message_id": message.provider_message_id,
        "message": message,
        "reaction_summary": reaction_summary,
        "source": "tdlib.updateMessageInteractionInfo"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateMessageInteractionInfo"
    }))
    .build()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sample_message(occurred_at: DateTime<Utc>) -> TelegramMessage {
        TelegramMessage {
            message_id: "message:v4:telegram:test".to_owned(),
            raw_record_id: "raw:v4:telegram:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_message_id: "-100123:42".to_owned(),
            provider_chat_id: Some("-100123".to_owned()),
            chat_title: "Chat".to_owned(),
            sender: "Telegram User 777".to_owned(),
            sender_display_name: Some("Alice".to_owned()),
            text: "hello".to_owned(),
            occurred_at: Some(occurred_at),
            projected_at: occurred_at,
            channel_kind: "telegram_user".to_owned(),
            delivery_state: "received".to_owned(),
            metadata: json!({}),
        }
    }

    #[test]
    fn message_deleted_event_contains_tombstone_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 13, 0, 0)
            .single()
            .expect("valid timestamp");
        let message = sample_message(occurred_at);
        let tombstone = TelegramMessageTombstone {
            tombstone_id: "tomb-1".to_owned(),
            message_id: message.message_id.clone(),
            account_id: "acct-1".to_owned(),
            provider_message_id: "-100123:42".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            reason_class: "deleted_by_provider".to_owned(),
            actor_class: "provider".to_owned(),
            observed_at: occurred_at,
            source_event: Some("updateDeleteMessages".to_owned()),
            is_provider_delete: true,
            is_local_visible: false,
            metadata: json!({"from_cache": false}),
            provenance: json!({"provider": "telegram"}),
            created_at: occurred_at,
        };

        let event = message_deleted_event("acct-1", &message, &tombstone, occurred_at)
            .expect("message deleted event");

        assert_eq!(event.event_type, telegram_event_types::MESSAGE_DELETED);
        assert_eq!(
            event.payload["tombstone"]["reason_class"],
            "deleted_by_provider"
        );
        assert_eq!(event.payload["provider_message_id"], "-100123:42");
    }

    #[test]
    fn reaction_changed_event_contains_summary_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 13, 0, 0)
            .single()
            .expect("valid timestamp");
        let message = sample_message(occurred_at);

        let event = reaction_changed_event(
            "acct-1",
            &message,
            Some(json!({"total_reactions": 2})),
            occurred_at,
        )
        .expect("reaction changed event");

        assert_eq!(event.event_type, telegram_event_types::REACTION_CHANGED);
        assert_eq!(event.payload["reaction_summary"]["total_reactions"], 2);
        assert_eq!(
            event.payload["message"]["message_id"],
            "message:v4:telegram:test"
        );
    }

    #[test]
    fn message_updated_event_contains_extra_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 13, 0, 0)
            .single()
            .expect("valid timestamp");
        let message = sample_message(occurred_at);

        let event = message_updated_event(
            "acct-1",
            &message,
            json!({"text_changed": true, "source": "tdlib.updateMessageContent"}),
            occurred_at,
        )
        .expect("message updated event");

        assert_eq!(event.event_type, telegram_event_types::MESSAGE_UPDATED);
        assert_eq!(event.payload["text_changed"], true);
        assert_eq!(event.payload["source"], "tdlib.updateMessageContent");
    }
}
```
