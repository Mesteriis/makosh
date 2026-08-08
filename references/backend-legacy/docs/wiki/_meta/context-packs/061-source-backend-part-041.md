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

- Chunk ID / ID чанка: `061-source-backend-part-041`
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

### `backend/src/integrations/telegram/client/messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages.rs`
- Size bytes / Размер в байтах: `306`
- Included characters / Включено символов: `306`
- Truncated / Обрезано: `no`

```rust
mod account_lookup;
mod attachments;
mod chat_lookup;
mod ingestion;
mod intelligence;
mod manual_send;
mod message_metadata;
mod queries;
mod raw_signals;
pub(in crate::integrations::telegram) mod reaction_metadata;
mod tdlib_ingestion;

pub(crate) use attachments::TelegramAttachmentDownloadStateUpdate;
```

### `backend/src/integrations/telegram/client/messages/account_lookup.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/account_lookup.rs`
- Size bytes / Размер в байтах: `1228`
- Included characters / Включено символов: `1228`
- Truncated / Обрезано: `no`

```rust
use crate::platform::communications::ProviderAccount;

use super::super::errors::TelegramError;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub async fn telegram_account_record(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, TelegramError> {
        self.telegram_provider_account(account_id).await
    }

    pub(in crate::integrations::telegram::client) async fn telegram_provider_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, TelegramError> {
        let provider_account = self
            .provider_account_store()
            .get(account_id)
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{account_id}` is not configured"
                ))
            })?;
        if !provider_account.provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(format!(
                "account `{}` is not a Telegram provider account",
                provider_account.account_id
            )));
        }
        Ok(provider_account)
    }
}
```

### `backend/src/integrations/telegram/client/messages/attachments.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/attachments.rs`
- Size bytes / Размер в байтах: `2788`
- Included characters / Включено символов: `2788`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;

use super::super::errors::TelegramError;
use super::super::models::TelegramAttachmentAnchor;
use super::super::observations::TelegramAttachmentDownloadObservation;
use super::super::store::TelegramStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

pub(crate) struct TelegramAttachmentDownloadStateUpdate<'a> {
    pub(crate) message_id: &'a str,
    pub(crate) provider_attachment_id: &'a str,
    pub(crate) tdlib_file_id: i64,
    pub(crate) download_state: &'a str,
    pub(crate) local_path: Option<&'a str>,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) content_type: &'a str,
    pub(crate) filename: Option<&'a str>,
}

impl TelegramStore {
    pub(crate) async fn attachment_anchor_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<TelegramAttachmentAnchor, TelegramError> {
        let anchor = self
            .provider_channel_message_store()
            .attachment_anchor(
                account_id,
                provider_chat_id,
                provider_message_id,
                TELEGRAM_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram message `{}` is not projected for chat `{}` and account `{}`",
                    provider_message_id.trim(),
                    provider_chat_id.trim(),
                    account_id.trim()
                ))
            })?;

        Ok(TelegramAttachmentAnchor {
            message_id: anchor.message_id,
            raw_record_id: anchor.raw_record_id,
        })
    }

    pub(crate) async fn update_message_attachment_download_state(
        &self,
        update: TelegramAttachmentDownloadStateUpdate<'_>,
    ) -> Result<(), TelegramError> {
        let message = self
            .message_by_id(update.message_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram message `{}` was not found",
                    update.message_id
                ))
            })?;
        self.append_attachment_download_observation(
            &message,
            TelegramAttachmentDownloadObservation {
                provider_attachment_id: update.provider_attachment_id,
                tdlib_file_id: update.tdlib_file_id,
                download_state: update.download_state,
                local_path: update.local_path,
                size_bytes: update.size_bytes,
                content_type: update.content_type,
                filename: update.filename,
                observed_at: Utc::now(),
            },
        )
        .await?;
        Ok(())
    }
}
```

### `backend/src/integrations/telegram/client/messages/chat_lookup.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/chat_lookup.rs`
- Size bytes / Размер в байтах: `1038`
- Included characters / Включено символов: `1038`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::TelegramError;
use super::super::models::TelegramChat;
use super::super::rows::row_to_telegram_chat;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(crate) async fn telegram_chat(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Result<Option<TelegramChat>, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE account_id = $1 AND provider_chat_id = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_chat_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_chat).transpose()
    }
}
```

### `backend/src/integrations/telegram/client/messages/ingestion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/ingestion.rs`
- Size bytes / Размер в байтах: `6698`
- Included characters / Включено символов: `6698`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use crate::platform::communications::NewRawCommunicationRecord;

use super::super::TELEGRAM_MESSAGE_RECORD_KIND;
use super::super::errors::TelegramError;
use super::super::identifiers::{stable_hash, telegram_raw_record_id};
use super::super::models::{
    NewTelegramChat, NewTelegramMessage, TelegramObservedMessage, TelegramSyncState,
};
use super::super::store::TelegramStore;
use super::message_metadata::{
    derive_mention_metadata, derive_tdlib_attachment_metadata, derive_tdlib_media_album_metadata,
    derive_tdlib_structured_evidence, telegram_public_message_link,
};
use super::reaction_metadata::{
    derive_tdlib_chosen_reaction_emojis, derive_tdlib_provider_reactions,
    derive_tdlib_reaction_summary_metadata,
};

impl TelegramStore {
    pub async fn ingest_fixture_message(
        &self,
        message: &NewTelegramMessage,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        self.observe_message_with_runtime(message, "fixture", None)
            .await
    }

    pub(in crate::integrations::telegram::client::messages) async fn ingest_message_with_runtime(
        &self,
        message: &NewTelegramMessage,
        runtime_kind: &str,
        tdlib_raw: Option<Value>,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        self.observe_message_with_runtime(message, runtime_kind, tdlib_raw)
            .await
    }

    pub(in crate::integrations::telegram::client::messages) async fn observe_message_with_runtime(
        &self,
        message: &NewTelegramMessage,
        runtime_kind: &str,
        tdlib_raw: Option<Value>,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        message.validate_for_runtime(runtime_kind)?;
        let provider_account = self.telegram_provider_account(&message.account_id).await?;

        let chat = NewTelegramChat {
            account_id: message.account_id.clone(),
            provider_chat_id: message.provider_chat_id.clone(),
            chat_kind: message.chat_kind,
            title: message.chat_title.clone(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: Some(message.occurred_at),
            metadata: json!({"runtime": runtime_kind}),
        };
        let chat = self.upsert_chat(&chat).await?;

        let mention_metadata = derive_mention_metadata(&message.text, tdlib_raw.as_ref());
        let public_message_link =
            telegram_public_message_link(chat.username.as_deref(), &message.provider_message_id);
        let tdlib_media_album = tdlib_raw
            .as_ref()
            .and_then(|raw| derive_tdlib_media_album_metadata(raw, &message.provider_chat_id));
        let tdlib_attachments = tdlib_raw
            .as_ref()
            .map(derive_tdlib_attachment_metadata)
            .unwrap_or_default();
        let tdlib_structured_evidence = tdlib_raw
            .as_ref()
            .map(derive_tdlib_structured_evidence)
            .unwrap_or_default();
        let tdlib_reaction_summary = tdlib_raw
            .as_ref()
            .and_then(derive_tdlib_reaction_summary_metadata);
        let tdlib_provider_reactions = tdlib_raw
            .as_ref()
            .map(derive_tdlib_provider_reactions)
            .unwrap_or_default();
        let tdlib_chosen_reactions = tdlib_raw
            .as_ref()
            .map(derive_tdlib_chosen_reaction_emojis)
            .unwrap_or_default();
        let mut payload = json!({
            "provider_chat_id": message.provider_chat_id,
            "chat_title": message.chat_title,
            "chat_kind": message.chat_kind.as_str(),
            "sender_id": message.sender_id,
            "sender_display_name": message.sender_display_name,
            "text": message.text,
            "delivery_state": message.delivery_state.as_str(),
            "mention_count": mention_metadata.count,
            "mentions": mention_metadata.mentions,
            "mentions_detected_by": mention_metadata.detected_by,
        });
        if let Some(payload) = payload.as_object_mut() {
            if let Some(link) = public_message_link {
                payload.insert("message_link".to_owned(), Value::String(link));
                payload.insert(
                    "message_link_kind".to_owned(),
                    Value::String("public_t_me".to_owned()),
                );
            }
            if let Some((album_id, album_key)) = tdlib_media_album {
                payload.insert("media_album_id".to_owned(), Value::String(album_id));
                payload.insert("media_album_key".to_owned(), Value::String(album_key));
            }
            if !tdlib_attachments.is_empty() {
                payload.insert("attachments".to_owned(), Value::Array(tdlib_attachments));
            }
            if let Some(reaction_summary) = tdlib_reaction_summary {
                payload.insert("reaction_summary".to_owned(), reaction_summary);
            }
            for (key, value) in tdlib_structured_evidence {
                payload.insert(key, value);
            }
            if let Some(tdlib_raw) = tdlib_raw {
                payload.insert("tdlib_raw".to_owned(), tdlib_raw);
            }
        }
        let raw_record_id = telegram_raw_record_id(
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            payload,
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "telegram",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": runtime_kind,
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        let _ = (
            provider_account.external_account_id,
            tdlib_provider_reactions,
            tdlib_chosen_reactions,
        );

        let message_id = format!(
            "message:v4:telegram:{}",
            stable_hash(
                [
                    message.account_id.as_str(),
                    message.provider_message_id.as_str()
                ]
                .join("\0")
                .as_bytes()
            )
        );

        Ok(TelegramObservedMessage {
            raw_record_id,
            message_id,
            raw,
            telegram_chat_id: chat.telegram_chat_id,
        })
    }
}
```

### `backend/src/integrations/telegram/client/messages/intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/intelligence.rs`
- Size bytes / Размер в байтах: `315`
- Included characters / Включено символов: `315`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::TelegramError;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(in crate::integrations::telegram::client::messages) async fn refresh_message_intelligence_candidates(
        &self,
        _message_id: &str,
    ) -> Result<(), TelegramError> {
        Ok(())
    }
}
```

### `backend/src/integrations/telegram/client/messages/manual_send.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/manual_send.rs`
- Size bytes / Размер в байтах: `2698`
- Included characters / Включено символов: `2698`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;

use super::super::errors::TelegramError;
use super::super::identifiers::{telegram_account_runtime, telegram_text_preview_hash};
use super::super::models::{
    NewTelegramMessage, TelegramChatKind, TelegramDeliveryState, TelegramManualSendRequest,
    TelegramManualSendResponse,
};
use super::super::store::TelegramStore;

impl TelegramStore {
    pub async fn manual_send_message(
        &self,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError> {
        request.validate()?;
        let provider_account = self.telegram_provider_account(&request.account_id).await?;
        let runtime_kind = telegram_account_runtime(&provider_account);
        if runtime_kind != "fixture" {
            return Err(TelegramError::InvalidRequest(
                "manual live Telegram sends require an enabled TDLib actor".to_owned(),
            ));
        }

        let chat = self
            .telegram_chat(&request.account_id, &request.provider_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{}` is not synced for account `{}`",
                    request.provider_chat_id, request.account_id
                ))
            })?;
        let provider_message_id = format!("manual:{}", request.command_id.trim());
        let rendered_preview_hash = telegram_text_preview_hash(&request.text);
        let message = NewTelegramMessage {
            account_id: request.account_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id.trim().to_owned(),
            provider_message_id,
            chat_kind: TelegramChatKind::try_from(chat.chat_kind.as_str())?,
            chat_title: chat.title,
            sender_id: "makosh".to_owned(),
            sender_display_name: "Макошь".to_owned(),
            text: request.text.trim().to_owned(),
            import_batch_id: format!("telegram-manual-send:{}", request.command_id.trim()),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Sent,
        };
        let result = self.ingest_fixture_message(&message).await?;

        Ok(TelegramManualSendResponse {
            raw: Some(result.raw),
            raw_record_id: result.raw_record_id,
            message_id: result.message_id,
            account_id: request.account_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id.trim().to_owned(),
            delivery_state: TelegramDeliveryState::Sent.as_str().to_owned(),
            status: "sent".to_owned(),
            runtime_kind,
            rendered_preview_hash,
        })
    }
}
```

### `backend/src/integrations/telegram/client/messages/message_metadata.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/message_metadata.rs`
- Size bytes / Размер в байтах: `19092`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::{Value, json};

pub(super) struct MentionMetadata {
    pub(super) count: i64,
    pub(super) mentions: Vec<String>,
    pub(super) detected_by: &'static str,
}

pub(super) fn derive_mention_metadata(text: &str, tdlib_raw: Option<&Value>) -> MentionMetadata {
    let text_mentions = extract_text_mentions(text);
    let entity_count = tdlib_raw.map(tdlib_mention_entity_count).unwrap_or(0);

    if entity_count > 0 {
        MentionMetadata {
            count: entity_count,
            mentions: text_mentions,
            detected_by: "tdlib_entities",
        }
    } else {
        MentionMetadata {
            count: i64::try_from(text_mentions.len()).unwrap_or(0),
            mentions: text_mentions,
            detected_by: "text_regex",
        }
    }
}

pub(super) fn telegram_public_message_link(
    username: Option<&str>,
    provider_message_id: &str,
) -> Option<String> {
    let username = username
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| {
            value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })?;
    let message_id = provider_message_id
        .rsplit(':')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| value.chars().all(|ch| ch.is_ascii_digit()))?;

    Some(format!("https://t.me/{username}/{message_id}"))
}

pub(super) fn derive_tdlib_media_album_metadata(
    raw: &Value,
    provider_chat_id: &str,
) -> Option<(String, String)> {
    let album_id = raw
        .get("media_album_id")
        .and_then(|value| {
            value
                .as_i64()
                .map(|number| number.to_string())
                .or_else(|| value.as_str().map(str::to_owned))
        })
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .filter(|value| value != "0")?;
    let chat_id = provider_chat_id.trim();
    let album_key = if chat_id.is_empty() {
        album_id.clone()
    } else {
        format!("{chat_id}:{album_id}")
    };
    Some((album_id, album_key))
}

pub(super) fn derive_tdlib_attachment_metadata(raw: &Value) -> Vec<Value> {
    let Some(content) = raw.get("content") else {
        return Vec::new();
    };
    let Some(content_type) = content.get("@type").and_then(Value::as_str) else {
        return Vec::new();
    };

    match content_type {
        "messageSticker" => tdlib_file_attachment(
            content,
            "sticker",
            &["sticker", "sticker"],
            tdlib_sticker_mime_type(content),
            tdlib_sticker_filename(content),
        ),
        "messageAnimation" => tdlib_file_attachment(
            content,
            "animation",
            &["animation", "animation"],
            tdlib_nested_string(content, &["animation", "mime_type"])
                .unwrap_or_else(|| "video/mp4".to_owned()),
            tdlib_nested_string(content, &["animation", "file_name"])
                .unwrap_or_else(|| "animation.mp4".to_owned()),
        ),
        "messageVideoNote" => tdlib_file_attachment(
            content,
            "video_note",
            &["video_note", "video"],
            "video/mp4".to_owned(),
            "video-note.mp4".to_owned(),
        ),
        _ => None,
    }
    .into_iter()
    .collect()
}

pub(super) fn derive_tdlib_structured_evidence(raw: &Value) -> Vec<(String, Value)> {
    let Some(content) = raw.get("content") else {
        return Vec::new();
    };
    let Some(content_type) = content.get("@type").and_then(Value::as_str) else {
        return Vec::new();
    };

    match content_type {
        "messagePoll" => tdlib_poll_evidence(content)
            .map(|value| vec![("telegram_poll".to_owned(), value)])
            .unwrap_or_default(),
        "messageLocation" => tdlib_location_evidence(content)
            .map(|value| vec![("telegram_location".to_owned(), value)])
            .unwrap_or_default(),
        "messageVenue" => tdlib_venue_evidence(content)
            .map(|value| vec![("telegram_location".to_owned(), value)])
            .unwrap_or_default(),
        "messageContact" => tdlib_contact_evidence(content)
            .map(|value| vec![("telegram_contact_card".to_owned(), value)])
            .unwrap_or_default(),
        "messageChatAddMembers"
        | "messageChatJoinByLink"
        | "messageChatJoinByRequest"
        | "messageChatDeleteMember" => tdlib_join_leave_evidence(content, content_type)
            .map(|value| vec![("telegram_join_leave".to_owned(), value)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn extract_text_mentions(text: &str) -> Vec<String> {
    let mut mentions = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] != '@' {
            index += 1;
            continue;
        }
        let mut end = index + 1;
        while end < chars.len() && is_telegram_mention_char(chars[end]) {
            end += 1;
        }
        if end.saturating_sub(index) >= 3 {
            let mention: String = chars[index..end].iter().collect();
            if !mentions.iter().any(|existing| existing == &mention) {
                mentions.push(mention);
            }
        }
        index = end;
    }
    mentions
}

fn is_telegram_mention_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

fn tdlib_mention_entity_count(raw: &Value) -> i64 {
    tdlib_formatted_text_entities(raw)
        .into_iter()
        .flat_map(|entities| entities.iter())
        .filter(|entity| {
            matches!(
                entity
                    .get("type")
                    .and_then(|value| value.get("@type"))
                    .and_then(Value::as_str),
                Some("textEntityTypeMention" | "textEntityTypeMentionName")
            )
        })
        .count() as i64
}

fn tdlib_formatted_text_entities(raw: &Value) -> Vec<&Vec<Value>> {
    let mut entities = Vec::new();
    if let Some(content) = raw.get("content") {
        for key in ["text", "caption"] {
            if let Some(array) = content
                .get(key)
                .and_then(|value| value.get("entities"))
                .and_then(Value::as_array)
            {
                entities.push(array);
            }
        }
    }
    entities
}

fn tdlib_poll_evidence(content: &Value) -> Option<Value> {
    let poll = content.get("poll")?;
    let question = tdlib_nested_string(poll, &["question", "text"])
        .or_else(|| tdlib_nested_string(poll, &["question"]))?;
    let options = poll
        .get("options")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|option| {
                    tdlib_nested_string(option, &["text", "text"])
                        .or_else(|| tdlib_nested_string(option, &["text"]))
                        .map(|text| {
                            json!({
                                "text": text,
                                "voter_count": option.get("voter_count").and_then(Value::as_i64),
                            })
                        })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(json!({
        "question": question,
        "options": options,
        "total_voter_count": poll.get("total_voter_count").and_then(Value::as_i64),
        "is_closed": poll.get("is_closed").and_then(Value::as_bool).unwrap_or(false),
        "poll_type": poll.get("type").and_then(|value| value.get("@type")).and_then(Value::as_str),
    }))
}

fn tdlib_location_evidence(content: &Value) -> Option<Value> {
    let location = content.get("location")?;
    Some(json!({
        "kind": "location",
        "latitude": location.get("latitude").and_then(Value::as_f64)?,
        "longitude": location.get("longitude").and_then(Value::as_f64)?,
        "horizontal_accuracy": location.get("horizontal_accuracy").and_then(Value::as_f64),
    }))
}

fn tdlib_venue_evidence(content: &Value) -> Option<Value> {
    let venue = content.get("venue")?;
    let location = venue.get("location")?;
    Some(json!({
        "kind": "venue",
        "title": venue.get("title").and_then(Value::as_str),
        "address": venue.get("address").and_then(Value::as_str),
        "latitude": location.get("latitude").and_then(Value::as_f64)?,
        "longitude": location.get("longitude").and_then(Value::as_f64)?,
        "provider": venue.get("provider").and_then(Value::as_str),
        "venue_id": venue.get("id").and_then(Value::as_str),
    }))
}

fn tdlib_contact_evidence(content: &Value) -> Option<Value> {
    let contact = content.get("contact")?;
    Some(json!({
        "phone_number": contact.get("phone_number").and_then(Value::as_str),
        "first_name": contact.get("first_name").and_then(Value::as_str),
        "last_name": contact.get("last_name").and_then(Value::as_str),
        "user_id": contact.get("user_id").and_then(Value::as_i64),
        "vcard": contact.get("vcard").and_then(Value::as_str),
    }))
}

fn tdlib_join_leave_evidence(content: &Value, content_type: &str) -> Option<Value> {
    match content_type {
        "messageChatAddMembers" => Some(json!({
            "action": "join",
            "source": "add_members",
            "user_ids": content.get("member_user_ids").and_then(Value::as_array).cloned().unwrap_or_default(),
        })),
        "messageChatJoinByLink" => Some(json!({
            "action": "join",
            "source": "join_by_link",
        })),
        "messageChatJoinByRequest" => Some(json!({
            "action": "join",
            "source": "join_by_request",
        })),
        "messageChatDeleteMember" => Some(json!({
            "action": "leave",
            "source": "delete_member",
            "user_id": content.get("user_id").and_then(Value::as_i64),
        })),
        _ => None,
    }
}

fn tdlib_file_attachment(
    content: &Value,
    attachment_type: &str,
    file_path: &[&str],
    content_type: String,
    filename: String,
) -> Option<Value> {
    let file = tdlib_nested_value(content, file_path)?;
    let tdlib_file_id = file.get("id").and_then(Value::as_i64)?;
    if tdlib_file_id <= 0 {
        return None;
    }
    let size = file.get("size").and_then(Value::as_i64);
    let provider_attachment_id = format!("tdlib:{attachment_type}:{tdlib_file_id}");
    let mut attachment = json!({
        "id": provider_attachment_id,
        "attachment_id": provider_attachment_id,
        "attachment_type": attachment_type,
        "content_type": content_type,
        "filename": filename,
        "tdlib_file_id": tdlib_file_id,
        "download_state": "remote",
        "metadata": {
            "tdlib_content_type": content.get("@type").and_then(Value::as_str),
        },
    });
    if let (Some(object), Some(size)) = (attachment.as_object_mut(), size) {
        object.insert("size".to_owned(), json!(size));
    }
    if attachment_type == "sticker"
        && let Some(emoji) = tdlib_nested_string(content, &["sticker", "emoji"])
        && let Some(metadata) = attachment
            .get_mut("metadata")
            .and_then(Value::as_object_mut)
    {
        metadata.insert("emoji".to_owned(), Value::String(emoji));
    }
    Some(attachment)
}

fn tdlib_sticker_mime_type(content: &Value) -> String {
    match tdlib_nested_string(content, &["sticker", "format", "@type"]).as_deref() {
        Some("stickerFormatTgs") => "application/x-tgsticker".to_owned(),
        Some("stickerFormatWebm") => "video/webm".to_owned(),
        _ => "image/webp".to_owned(),
    }
}

fn tdlib_sticker_filename(content: &Value) -> String {
    let extension = match tdlib_nested_string(content, &["sticker", "format", "@type"]).as_deref() {
        Some("stickerFormatTgs") => "tgs",
        Some("sti
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/messages/queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/queries.rs`
- Size bytes / Размер в байтах: `2406`
- Included characters / Включено символов: `2406`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::TelegramError;
use super::super::models::TelegramMessage;
use super::super::rows::provider_channel_message_to_telegram_message;
use super::super::store::TelegramStore;
use super::super::validation::validate_message_list_limit;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

impl TelegramStore {
    pub(in crate::integrations::telegram) async fn message_by_provider_message_id(
        &self,
        account_id: &str,
        provider_message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        Ok(self
            .provider_channel_message_store()
            .message_by_provider_record_id(account_id, provider_message_id, TELEGRAM_CHANNEL_KINDS)
            .await?
            .map(provider_channel_message_to_telegram_message))
    }

    pub async fn message_by_id(
        &self,
        message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        Ok(self
            .provider_channel_message_store()
            .message_by_id(message_id, TELEGRAM_CHANNEL_KINDS)
            .await?
            .map(provider_channel_message_to_telegram_message))
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = validate_message_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        Ok(self
            .provider_channel_message_store()
            .recent_messages(account_id, provider_chat_id, TELEGRAM_CHANNEL_KINDS, limit)
            .await?
            .into_iter()
            .map(provider_channel_message_to_telegram_message)
            .collect())
    }

    pub async fn messages_by_ids(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        if message_ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(self
            .provider_channel_message_store()
            .messages_by_ids(message_ids, TELEGRAM_CHANNEL_KINDS)
            .await?
            .into_iter()
            .map(provider_channel_message_to_telegram_message)
            .collect())
    }
}
```

### `backend/src/integrations/telegram/client/messages/raw_signals.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/raw_signals.rs`
- Size bytes / Размер в байтах: `1402`
- Included characters / Включено символов: `1402`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramStore;
use crate::integrations::telegram::client::models::TelegramObservedMessage;
use crate::platform::communications::{
    CommunicationRawRecordCommandPort, CommunicationRawSignalSource,
    ProviderCommunicationMessagePortError, build_communication_raw_signal_event,
};
use crate::platform::events::{EventBus, EventStore};

use super::super::errors::TelegramError;

impl TelegramStore {
    pub(in crate::integrations::telegram) async fn publish_observed_message_raw_signal(
        &self,
        observed: &TelegramObservedMessage,
        event_bus: Option<&EventBus>,
    ) -> Result<(), TelegramError> {
        let stored_raw = self
            .communication_raw_record_store()
            .record_raw_source(&observed.raw)
            .await?;
        let event = build_communication_raw_signal_event(
            CommunicationRawSignalSource::Telegram,
            &stored_raw,
            None,
        )
        .map_err(ProviderCommunicationMessagePortError::from)?;
        let appended = EventStore::new(self.pool().clone())
            .append_for_dispatch_idempotent(&event)
            .await
            .map_err(ProviderCommunicationMessagePortError::from)?;
        if appended.is_some()
            && let Some(event_bus) = event_bus
        {
            let _ = event_bus.broadcast(event);
        }
        Ok(())
    }
}
```

### `backend/src/integrations/telegram/client/messages/reaction_metadata.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/reaction_metadata.rs`
- Size bytes / Размер в байтах: `13293`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::integrations::telegram) struct TdlibProviderReaction {
    pub(in crate::integrations::telegram) sender_id: String,
    pub(in crate::integrations::telegram) reaction_emoji: String,
    pub(in crate::integrations::telegram) is_outgoing: bool,
}

pub(in crate::integrations::telegram) fn derive_tdlib_reaction_summary_metadata(
    raw: &Value,
) -> Option<Value> {
    let reactions = raw
        .get("interaction_info")
        .and_then(|value| value.get("reactions"))
        .and_then(|value| value.get("reactions"))
        .and_then(Value::as_array)?;

    let groups = reactions
        .iter()
        .filter_map(tdlib_reaction_group)
        .collect::<Vec<_>>();
    let custom_groups = reactions
        .iter()
        .filter_map(tdlib_custom_reaction_group)
        .collect::<Vec<_>>();
    if groups.is_empty() && custom_groups.is_empty() {
        return None;
    }

    let total_reactions = groups
        .iter()
        .chain(custom_groups.iter())
        .filter_map(|group| group.get("count").and_then(Value::as_i64))
        .sum::<i64>();

    Some(json!({
        "source": "tdlib_interaction_info",
        "total_reactions": total_reactions,
        "active_reactions": total_reactions,
        "reactions": groups,
        "custom_reactions": custom_groups,
    }))
}

pub(in crate::integrations::telegram) fn derive_tdlib_provider_reactions(
    raw: &Value,
) -> Vec<TdlibProviderReaction> {
    raw.get("interaction_info")
        .and_then(|value| value.get("reactions"))
        .and_then(|value| value.get("recent_reactions"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(tdlib_provider_reaction)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(in crate::integrations::telegram) fn derive_tdlib_chosen_reaction_emojis(
    raw: &Value,
) -> Vec<String> {
    let mut chosen = raw
        .get("interaction_info")
        .and_then(|value| value.get("reactions"))
        .and_then(|value| value.get("reactions"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter(|reaction| {
                    reaction
                        .get("is_chosen")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .filter_map(tdlib_reaction_emoji)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    chosen.sort();
    chosen.dedup();
    chosen
}

fn tdlib_provider_reaction(reaction: &Value) -> Option<TdlibProviderReaction> {
    Some(TdlibProviderReaction {
        sender_id: tdlib_sender_id(reaction.get("sender_id")?)?,
        reaction_emoji: tdlib_reaction_emoji(reaction)?,
        is_outgoing: reaction
            .get("is_outgoing")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn tdlib_reaction_group(reaction: &Value) -> Option<Value> {
    let emoji = tdlib_reaction_emoji(reaction)?;
    let count = reaction
        .get("total_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if count <= 0 {
        return None;
    }

    Some(json!({
        "reaction_emoji": emoji,
        "count": count,
        "senders": [],
        "is_chosen": reaction.get("is_chosen").and_then(Value::as_bool).unwrap_or(false),
        "source": "tdlib_interaction_info",
    }))
}

fn tdlib_custom_reaction_group(reaction: &Value) -> Option<Value> {
    let custom_emoji_id = reaction
        .get("type")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .filter(|reaction_type| *reaction_type == "reactionTypeCustomEmoji")
        .and_then(|_| reaction.get("type"))
        .and_then(|value| {
            value
                .get("custom_emoji_id")
                .and_then(Value::as_i64)
                .map(|number| number.to_string())
                .or_else(|| {
                    value
                        .get("custom_emoji_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
        })
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())?;
    let count = reaction
        .get("total_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if count <= 0 {
        return None;
    }

    Some(json!({
        "custom_emoji_id": custom_emoji_id,
        "count": count,
        "senders": [],
        "is_chosen": reaction.get("is_chosen").and_then(Value::as_bool).unwrap_or(false),
        "source": "tdlib_interaction_info",
    }))
}

fn tdlib_reaction_emoji(reaction: &Value) -> Option<String> {
    reaction
        .get("type")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .filter(|reaction_type| *reaction_type == "reactionTypeEmoji")
        .and_then(|_| reaction.get("type"))
        .and_then(|value| value.get("emoji"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn tdlib_sender_id(sender: &Value) -> Option<String> {
    match sender.get("@type").and_then(Value::as_str)? {
        "messageSenderUser" => tdlib_id(sender, "user_id").map(|id| format!("user:{id}")),
        "messageSenderChat" => tdlib_id(sender, "chat_id").map(|id| format!("chat:{id}")),
        _ => None,
    }
}

fn tdlib_id(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|value| {
            value
                .as_i64()
                .map(|number| number.to_string())
                .or_else(|| value.as_str().map(ToOwned::to_owned))
        })
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .filter(|value| value != "0")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        derive_tdlib_chosen_reaction_emojis, derive_tdlib_provider_reactions,
        derive_tdlib_reaction_summary_metadata,
    };

    #[test]
    fn derives_tdlib_emoji_reaction_summary_from_interaction_info() {
        let summary = derive_tdlib_reaction_summary_metadata(&json!({
            "@type": "message",
            "interaction_info": {
                "@type": "messageInteractionInfo",
                "reactions": {
                    "@type": "messageReactions",
                    "reactions": [
                        {
                            "@type": "messageReaction",
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "total_count": 3,
                            "is_chosen": true
                        },
                        {
                            "@type": "messageReaction",
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "🔥"
                            },
                            "total_count": 2
                        }
                    ]
                }
            }
        }))
        .expect("reaction summary");

        assert_eq!(summary["source"], "tdlib_interaction_info");
        assert_eq!(summary["total_reactions"], 5);
        assert_eq!(summary["active_reactions"], 5);
        assert_eq!(summary["reactions"][0]["reaction_emoji"], "👍");
        assert_eq!(summary["reactions"][0]["count"], 3);
        assert_eq!(summary["reactions"][0]["is_chosen"], true);
        assert_eq!(summary["reactions"][1]["reaction_emoji"], "🔥");
        assert_eq!(summary["reactions"][1]["senders"], json!([]));
        assert_eq!(summary["custom_reactions"], json!([]));
    }

    #[test]
    fn preserves_custom_tdlib_reaction_summary_without_faking_emoji_contract() {
        let summary = derive_tdlib_reaction_summary_metadata(&json!({
            "@type": "message",
            "interaction_info": {
                "reactions": {
                    "reactions": [
                        {
                            "type": {
                                "@type": "reactionTypeCustomEmoji",
                                "custom_emoji_id": 42
                            },
                            "total_count": 10,
                            "is_chosen": true
                        },
                        {
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": " "
                            },
                            "total_count": 1
                        },
                        {
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "total_count": 0
                        }
                    ]
                }
            }
        }));

        let summary = summary.expect("custom reaction summary");
        assert_eq!(summary["total_reactions"], 10);
        assert_eq!(summary["reactions"], json!([]));
        assert_eq!(summary["custom_reactions"][0]["custom_emoji_id"], "42");
        assert_eq!(summary["custom_reactions"][0]["count"], 10);
        assert_eq!(summary["custom_reactions"][0]["is_chosen"], true);
    }

    #[test]
    fn derives_sender_level_tdlib_recent_reactions() {
        let reactions = derive_tdlib_provider_reactions(&json!({
            "@type": "message",
            "interaction_info": {
                "reactions": {
                    "recent_reactions": [
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderUser",
                                "user_id": 777
                            },
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "is_outgoing": true
                        },
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderChat",
                                "chat_id": -10042
                            },
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "🔥"
                            }
                        },
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderUser",
                                "user_id": 999
                            },
                            "type": {
                                "@type": "reactionTypeCustomEmoji",
                                "custom_emoji_id": 42
                            }
                        }
                    ]
                }
            }
        }));

        assert_eq!(reactions.len(), 2);
        assert_eq!(reactions[0].sender_id, "user:777");
        assert_eq!(reactions[0].reaction_emoji, "👍");
        assert!(reactions[0].is_outgoing);
        assert_eq!(reactions[1].sender_id, "chat:-10042");
        assert_eq!(reactions[1].reaction_emoji, "🔥");
        assert!(!reactions[1].is_outgoing);
    }

    #[test]
    fn derives_chosen_emoji_reactions_for_current_actor() {
        let chosen = derive_tdlib_chosen_reaction_emojis(&json!({
            "@type": "message",
            "interaction_info": {
                "reactions": {
                    "r
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/messages/tdlib_ingestion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/messages/tdlib_ingestion.rs`
- Size bytes / Размер в байтах: `1925`
- Included characters / Включено символов: `1925`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::tdjson::TelegramTdlibMessageSnapshot;

use super::super::errors::TelegramError;
use super::super::models::{NewTelegramMessage, TelegramChatKind, TelegramObservedMessage};
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(crate) async fn ingest_tdlib_message_snapshot(
        &self,
        account_id: &str,
        snapshot: &TelegramTdlibMessageSnapshot,
        import_batch_id: &str,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        let provider_account = self.telegram_provider_account(account_id).await?;
        let existing_chat = self
            .telegram_chat(&provider_account.account_id, &snapshot.provider_chat_id)
            .await?;
        let (chat_kind, chat_title) = match existing_chat {
            Some(chat) => (
                TelegramChatKind::try_from(chat.chat_kind.as_str())?,
                chat.title,
            ),
            None => (
                TelegramChatKind::Private,
                format!("Telegram Chat {}", snapshot.provider_chat_id),
            ),
        };
        let provider_message_id = format!(
            "{}:{}",
            snapshot.provider_chat_id, snapshot.provider_message_id
        );
        let message = NewTelegramMessage {
            account_id: provider_account.account_id,
            provider_chat_id: snapshot.provider_chat_id.clone(),
            provider_message_id,
            chat_kind,
            chat_title,
            sender_id: snapshot.sender_id.clone(),
            sender_display_name: snapshot.sender_display_name.clone(),
            text: snapshot.text.clone(),
            import_batch_id: import_batch_id.trim().to_owned(),
            occurred_at: snapshot.occurred_at,
            delivery_state: snapshot.delivery_state,
        };

        self.ingest_message_with_runtime(&message, "tdlib", Some(snapshot.raw.clone()))
            .await
    }
}
```

### `backend/src/integrations/telegram/client/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/mod.rs`
- Size bytes / Размер в байтах: `3018`
- Included characters / Включено символов: `3018`
- Truncated / Обрезано: `no`

```rust
mod accounts;
mod chat_metadata;
mod chat_reconciliation;
mod chat_state;
mod chats;
pub mod commands;
mod errors;
mod evidence;
mod identifiers;
pub mod lifecycle;
mod messages;
pub mod models;
mod observations;
pub mod participants;
mod reactions;
mod references;
pub mod rows;
mod search;
mod store;
#[cfg(test)]
mod tests;
pub mod topics;
mod validation;
mod vault;

const TELEGRAM_MESSAGE_RECORD_KIND: &str = "telegram_message";
const TELEGRAM_CHAT_RECORD_KIND: &str = "telegram_chat";
const TELEGRAM_ACCOUNT_ACTIVE: &str = "active";
const TELEGRAM_ACCOUNT_LOGGED_OUT: &str = "logged_out";
const TELEGRAM_ACCOUNT_REMOVED: &str = "removed";

pub use self::chat_state::{
    TelegramProviderChatPositionUpdate, reconcile_archive_commands_from_provider_state,
    reconcile_folder_add_commands_from_provider_state,
    reconcile_folder_remove_commands_from_provider_state,
    reconcile_mark_read_commands_from_provider_state,
    reconcile_marked_as_unread_commands_from_provider_state,
    reconcile_mute_commands_from_provider_state, reconcile_pin_commands_from_provider_state,
};
pub use self::errors::TelegramError;
pub(crate) use self::messages::TelegramAttachmentDownloadStateUpdate;
pub(in crate::integrations::telegram) use self::messages::reaction_metadata::derive_tdlib_chosen_reaction_emojis;
pub(in crate::integrations::telegram) use self::messages::reaction_metadata::{
    derive_tdlib_provider_reactions, derive_tdlib_reaction_summary_metadata,
};
pub use self::models::messages::TelegramReactionRequest;
pub use self::models::{
    NewTelegramChat, NewTelegramChatParticipant, NewTelegramMessage, NewTelegramTopic,
    TelegramAccount, TelegramAccountLifecycleResponse, TelegramAccountListResponse,
    TelegramAccountSetupRequest, TelegramAccountSetupResponse, TelegramChat,
    TelegramChatGroupFilter, TelegramChatGroupFilterListResponse, TelegramChatKind,
    TelegramChatMember, TelegramCredentialBinding, TelegramDeliveryState, TelegramForwardRequest,
    TelegramLiveAccountSetupRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramMessage, TelegramMessageIngestResult, TelegramQrLoginPasswordRequest,
    TelegramQrLoginStartRequest, TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
    TelegramReplyRequest, TelegramSyncState, TelegramTopic, TelegramTopicCloseRequest,
    TelegramTopicCreateRequest, TelegramTopicLifecycleResponse, TelegramTopicListResponse,
};
pub use self::participants::mark_absent_members_from_exhaustive_roster;
pub(in crate::integrations::telegram) use self::reactions::{
    TelegramReactionMessageRef, sync_provider_reactions,
};
pub use self::reactions::{add_reaction, reconcile_reaction_commands_from_provider_reactions};
pub use self::store::TelegramStore;
pub use self::vault::TelegramSecretVault;
pub type ProviderCommunicationMessage = TelegramMessage;

pub(crate) use self::identifiers::{
    ensure_telegram_account_active, telegram_chat_id, telegram_text_preview_hash,
};
pub(crate) use self::models::TelegramAttachmentAnchor;
```

### `backend/src/integrations/telegram/client/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/models.rs`
- Size bytes / Размер в байтах: `1138`
- Included characters / Включено символов: `1138`
- Truncated / Обрезано: `no`

```rust
mod accounts;
mod chats;
pub mod messages;
mod qr_login;
pub mod topics;

pub use accounts::{
    TelegramAccount, TelegramAccountLifecycleResponse, TelegramAccountListResponse,
    TelegramAccountSetupRequest, TelegramAccountSetupResponse, TelegramCredentialBinding,
    TelegramLiveAccountSetupRequest,
};
pub use chats::{
    NewTelegramChat, NewTelegramChatParticipant, TelegramChat, TelegramChatGroupFilter,
    TelegramChatGroupFilterListResponse, TelegramChatKind, TelegramChatMember, TelegramSyncState,
};
pub use messages::{
    NewTelegramMessage, TelegramDeliveryState, TelegramForwardRequest, TelegramManualSendRequest,
    TelegramManualSendResponse, TelegramMessage, TelegramMessageIngestResult,
    TelegramObservedMessage, TelegramReplyRequest,
};
pub use qr_login::{
    TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest, TelegramQrLoginStatus,
    TelegramQrLoginStatusResponse,
};
pub use topics::{
    NewTelegramTopic, TelegramTopic, TelegramTopicCloseRequest, TelegramTopicCreateRequest,
    TelegramTopicLifecycleResponse, TelegramTopicListResponse,
};

pub(crate) use messages::TelegramAttachmentAnchor;
```

### `backend/src/integrations/telegram/client/models/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/models/accounts.rs`
- Size bytes / Размер в байтах: `5935`
- Included characters / Включено символов: `5935`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::platform::communications::CommunicationProviderKind;
use crate::platform::secrets::{SecretKind, SecretStoreKind};

use super::super::errors::TelegramError;
use super::super::validation::{required_optional_value, validate_non_empty};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramAccountSetupRequest {
    pub(in crate::integrations::telegram::client) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramLiveAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub api_id: Option<i64>,
    pub api_hash: Option<String>,
    pub bot_token: Option<String>,
    pub session_encryption_key: Option<String>,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub qr_authorized: bool,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramLiveAccountSetupRequest {
    pub(crate) fn with_inferred_qr_authorization(mut self) -> Self {
        if self.is_finalized_qr_user_account() {
            self.qr_authorized = true;
        }
        self
    }

    pub(crate) fn with_app_credentials(
        mut self,
        api_id: Option<i64>,
        api_hash: Option<String>,
    ) -> Self {
        if self.is_qr_authorized_user_account() {
            return self;
        }
        if self.api_id.is_none() {
            self.api_id = api_id;
        }
        if self
            .api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            self.api_hash = api_hash;
        }
        self
    }

    pub(in crate::integrations::telegram::client) fn is_qr_authorized_user_account(&self) -> bool {
        self.qr_authorized && self.provider_kind == CommunicationProviderKind::TelegramUser
    }

    fn is_finalized_qr_user_account(&self) -> bool {
        self.provider_kind == CommunicationProviderKind::TelegramUser
            && self
                .external_account_id
                .trim()
                .strip_prefix("telegram:")
                .is_some_and(|provider_user_id| !provider_user_id.trim().is_empty())
            && self
                .tdlib_data_path
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
    }

    pub(in crate::integrations::telegram::client) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        match self.provider_kind {
            CommunicationProviderKind::TelegramUser => {
                if self.is_qr_authorized_user_account() {
                    required_optional_value("tdlib_data_path", self.tdlib_data_path.as_deref())?;
                    return Ok(());
                }
                let api_id = self.api_id.ok_or_else(|| {
                    TelegramError::InvalidRequest("api_id must not be empty".to_owned())
                })?;
                if api_id <= 0 {
                    return Err(TelegramError::InvalidRequest(
                        "api_id must be greater than zero".to_owned(),
                    ));
                }
                required_optional_value("api_hash", self.api_hash.as_deref())?;
            }
            CommunicationProviderKind::TelegramBot => {
                if self.qr_authorized {
                    return Err(TelegramError::InvalidRequest(
                        "qr_authorized is only supported for telegram_user".to_owned(),
                    ));
                }
                required_optional_value("bot_token", self.bot_token.as_deref())?;
            }
            _ => {
                return Err(TelegramError::InvalidRequest(
                    "provider_kind must be telegram_user or telegram_bot".to_owned(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub transcription_enabled: bool,
    pub credential_bindings: Vec<TelegramCredentialBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccount {
    pub account_id: String,
    pub provider_kind: String,
    pub display_name: String,
    pub external_account_id: String,
    pub runtime: String,
    pub lifecycle_state: String,
    pub transcription_enabled: bool,
    pub tdlib_data_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountListResponse {
    pub items: Vec<TelegramAccount>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountLifecycleResponse {
    pub account: TelegramAccount,
    pub stopped_runtime_actor: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramCredentialBinding {
    pub secret_purpose: String,
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
}
```

### `backend/src/integrations/telegram/client/models/chats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/models/chats.rs`
- Size bytes / Размер в байтах: `4109`
- Included characters / Включено символов: `4109`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::errors::TelegramError;
use super::super::validation::{validate_non_empty, validate_object};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChat {
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: String,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: String,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatMember {
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub message_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
    pub source: String,
    pub provider_member_id: String,
    pub username: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub is_admin: bool,
    pub is_owner: bool,
    pub permissions: Value,
    pub observed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTelegramChatParticipant {
    pub participant_id: String,
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_member_id: String,
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub role: String,
    pub status: String,
    pub is_admin: bool,
    pub is_owner: bool,
    pub permissions: Value,
    pub raw_payload: Value,
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatGroupFilter {
    pub id: String,
    pub label: String,
    pub source: String,
    pub count: i64,
    pub icon: String,
    pub provider_folder_id: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatGroupFilterListResponse {
    pub items: Vec<TelegramChatGroupFilter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTelegramChat {
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: TelegramChatKind,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: TelegramSyncState,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewTelegramChat {
    pub(in crate::integrations::telegram::client) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("title", &self.title)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramChatKind {
    Private,
    Group,
    Channel,
    Bot,
}

impl TelegramChatKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Group => "group",
            Self::Channel => "channel",
            Self::Bot => "bot",
        }
    }
}

impl TryFrom<&str> for TelegramChatKind {
    type Error = TelegramError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "private" => Ok(Self::Private),
            "group" => Ok(Self::Group),
            "channel" => Ok(Self::Channel),
            "bot" => Ok(Self::Bot),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram chat_kind `{other}`"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramSyncState {
    Fixture,
    Syncing,
    Synced,
    Degraded,
    Error,
}

impl TelegramSyncState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Syncing => "syncing",
            Self::Synced => "synced",
            Self::Degraded => "degraded",
            Self::Error => "error",
        }
    }
}
```

### `backend/src/integrations/telegram/client/models/messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/models/messages.rs`
- Size bytes / Размер в байтах: `22548`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::errors::TelegramError;
use super::super::validation::validate_non_empty;
use super::chats::TelegramChatKind;
use crate::platform::communications::NewRawCommunicationRecord;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewTelegramMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_kind: TelegramChatKind,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: TelegramDeliveryState,
}

impl NewTelegramMessage {
    fn validate(&self) -> Result<(), TelegramError> {
        self.validate_common()?;
        validate_non_empty("text", &self.text)?;
        Ok(())
    }

    pub(in crate::integrations::telegram::client) fn validate_for_runtime(
        &self,
        runtime_kind: &str,
    ) -> Result<(), TelegramError> {
        if runtime_kind == "tdlib" {
            self.validate_common()
        } else {
            self.validate()
        }
    }

    fn validate_common(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("chat_title", &self.chat_title)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(in crate::integrations::telegram::client) fn source_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.account_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_chat_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_message_id.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramDeliveryState {
    Received,
    Sent,
    SendDryRun,
    SendBlocked,
}

impl TelegramDeliveryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Sent => "sent",
            Self::SendDryRun => "send_dry_run",
            Self::SendBlocked => "send_blocked",
        }
    }

    pub fn as_message_delivery_state(self) -> &'static str {
        self.as_str()
    }
}

impl TryFrom<String> for TelegramDeliveryState {
    type Error = TelegramError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "received" => Ok(Self::Received),
            "sent" => Ok(Self::Sent),
            "send_dry_run" => Ok(Self::SendDryRun),
            "send_blocked" => Ok(Self::SendBlocked),
            _ => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram delivery_state `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessageIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramObservedMessage {
    pub raw_record_id: String,
    pub message_id: String,
    pub raw: NewRawCommunicationRecord,
    pub telegram_chat_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramManualSendRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub text: String,
}

impl TelegramManualSendRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("text", &self.text)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramManualSendResponse {
    #[serde(skip_serializing)]
    pub raw: Option<NewRawCommunicationRecord>,
    pub raw_record_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub delivery_state: String,
    pub status: String,
    pub runtime_kind: String,
    pub rendered_preview_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramReplyRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub reply_to_provider_message_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramForwardRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub from_provider_chat_id: String,
    pub from_provider_message_id: String,
}

impl TelegramForwardRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("from_provider_chat_id", &self.from_provider_chat_id)?;
        validate_non_empty("from_provider_message_id", &self.from_provider_message_id)?;
        Ok(())
    }
}

impl TelegramReplyRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty(
            "reply_to_provider_message_id",
            &self.reply_to_provider_message_id,
        )?;
        validate_non_empty("text", &self.text)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: Option<String>,
    pub chat_title: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub delivery_state: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramAttachmentAnchor {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
}

// ---------------------------------------------------------------------------
// Message lifecycle — ADR-0091: versions, tombstones, provider-write commands
// ---------------------------------------------------------------------------

/// An append-only observed edit version of a Telegram message.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageVersion {
    pub version_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub version_number: i32,
    pub body_text: Option<String>,
    pub edit_timestamp: DateTime<Utc>,
    pub source_event: Option<String>,
    pub raw_diff_payload: serde_json::Value,
    pub provenance: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Reason class for a tombstone per ADR-0091.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TombstoneReasonClass {
    DeletedByOwner,
    DeletedByCounterparty,
    DeletedByProvider,
    ModerationRemoved,
    AccountRemoved,
    RetentionPolicy,
    Unknown,
}

impl TombstoneReasonClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeletedByOwner => "deleted_by_owner",
            Self::DeletedByCounterparty => "deleted_by_counterparty",
            Self::DeletedByProvider => "deleted_by_provider",
            Self::ModerationRemoved => "moderation_removed",
            Self::AccountRemoved => "account_removed",
            Self::RetentionPolicy => "retention_policy",
            Self::Unknown => "unknown",
        }
    }
}

impl TryFrom<&str> for TombstoneReasonClass {
    type Error = TelegramError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "deleted_by_owner" => Ok(Self::DeletedByOwner),
            "deleted_by_counterparty" => Ok(Self::DeletedByCounterparty),
            "deleted_by_provider" => Ok(Self::DeletedByProvider),
            "moderation_removed" => Ok(Self::ModerationRemoved),
            "account_removed" => Ok(Self::AccountRemoved),
            "retention_policy" => Ok(Self::RetentionPolicy),
            "unknown" => Ok(Self::Unknown),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported tombstone reason class `{other}`"
            ))),
        }
    }
}

/// Actor class for tombstone provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TombstoneActorClass {
    Owner,
    Provider,
    Automation,
    System,
    Unknown,
}

impl TombstoneActorClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Provider => "provider",
            Self::Automation => "automation",
            Self::System => "system",
            Self::Unknown => "unknown",
        }
    }
}

/// A local visibility and delete evidence record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageTombstone {
    pub tombstone_id: String,
    pub message_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub reason_class: String,
    pub actor_class: String,
    pub observed_at: DateTime<Utc>,
    pub source_event: Option<String>,
    pub is_provider_delete: bool,
    pub is_local_visible: bool,
    pub metadata: serde_json::Value,
    pub provenance: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Durable provider-write command kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramCommandKind {
    SendText,
    SendMedia,
    Edit,
    Delete,
    RestoreVisibility,
    MarkRead,
    MarkUnread,
    Pin,
    Unpin,
    Archive,
    Unarchive,
    Mute,
    Unmute,
    React,
    Unreact,
    Reply,
    Forward,
    Join,
    Leave,
    FolderAdd,
    FolderRemove,
    TopicCreate,
    TopicClose,
    TopicReopen,
    AdminAction,
}

impl TelegramCommandKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SendText => "send_text",
            Self::SendMedia => "send_media",
            Self::Edit => "edit",
            Self::Delete => "delete",
            Self::RestoreVisibility => "restore_visibility",
            Self::MarkRead => "mark_read",
            Self::MarkUnread => "mark_unread",
            Self::Pin => "pin",
            Self::Unpin => "unpin",
            Self::Archive => "archive",
            Self::Unarchive => "unarchive",
            Self::Mute => "mute",
            Self::Unmute => "unmute",
            Self::React => "react",
            Self::Unreact => "unreact",
            Self::Reply => "reply",
            Self::Forward => "forward",
            Self::Join => "join",
            Self::Leave => "leave",
            Self::FolderAdd => "folder_add",
            Self::FolderRemove => "folder_remove",
            Self::TopicCreate => "topic_create",
            Self::TopicClose => "
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/models/qr_login.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/models/qr_login.rs`
- Size bytes / Размер в байтах: `3071`
- Included characters / Включено символов: `3071`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::super::errors::TelegramError;
use super::super::validation::validate_non_empty;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramQrLoginStartRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub api_id: Option<i64>,
    pub api_hash: Option<String>,
    pub session_encryption_key: Option<String>,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramQrLoginStartRequest {
    pub(crate) fn with_app_credentials(
        mut self,
        api_id: Option<i64>,
        api_hash: Option<String>,
    ) -> Self {
        if self.api_id.is_none() {
            self.api_id = api_id;
        }
        if self
            .api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            self.api_hash = api_hash;
        }
        self
    }

    pub(crate) fn required_api_id(&self) -> Result<i64, TelegramError> {
        let api_id = self
            .api_id
            .ok_or_else(|| TelegramError::InvalidRequest("api_id must not be empty".to_owned()))?;
        if api_id <= 0 {
            return Err(TelegramError::InvalidRequest(
                "api_id must be greater than zero".to_owned(),
            ));
        }
        Ok(api_id)
    }

    pub(crate) fn required_api_hash(&self) -> Result<&str, TelegramError> {
        self.api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| TelegramError::InvalidRequest("api_hash must not be empty".to_owned()))
    }

    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        self.required_api_id()?;
        self.required_api_hash()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramQrLoginPasswordRequest {
    pub password: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramQrLoginStatus {
    WaitingQrScan,
    WaitingPassword,
    Ready,
    Expired,
    Failed,
    RuntimeUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramQrLoginStatusResponse {
    pub setup_id: String,
    pub account_id: String,
    pub status: TelegramQrLoginStatus,
    pub qr_link: Option<String>,
    pub qr_svg: Option<String>,
    pub telegram_user_id: Option<String>,
    pub telegram_username: Option<String>,
    pub suggested_account_id: Option<String>,
    pub suggested_display_name: Option<String>,
    pub suggested_external_account_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub poll_after_ms: u64,
    pub message: Option<String>,
}
```

### `backend/src/integrations/telegram/client/models/topics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/models/topics.rs`
- Size bytes / Размер в байтах: `2610`
- Included characters / Включено символов: `2610`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::errors::TelegramError;
use super::super::validation::validate_non_empty;

#[derive(Clone, Debug, Serialize)]
pub struct TelegramTopic {
    pub topic_id: String,
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_topic_id: i64,
    pub provider_chat_id: String,
    pub title: String,
    pub icon_emoji: Option<String>,
    pub is_pinned: bool,
    pub is_closed: bool,
    pub unread_count: i32,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewTelegramTopic {
    pub topic_id: String,
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_topic_id: i64,
    pub provider_chat_id: String,
    pub title: String,
    pub icon_emoji: Option<String>,
    pub is_pinned: bool,
    pub is_closed: bool,
    pub unread_count: i32,
    pub last_message_at: Option<DateTime<Utc>>,
}

pub struct TelegramTopicListResponse {
    pub telegram_chat_id: String,
    pub items: Vec<TelegramTopic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramTopicCreateRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub title: String,
}

impl TelegramTopicCreateRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("title", &self.title)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramTopicCloseRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub is_closed: bool,
}

impl TelegramTopicCloseRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramTopicLifecycleResponse {
    pub operation: String,
    pub topic_id: Option<String>,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_topic_id: Option<i64>,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub command_id: String,
}
```

### `backend/src/integrations/telegram/client/observations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/observations.rs`
- Size bytes / Размер в байтах: `4274`
- Included characters / Включено символов: `4274`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::errors::TelegramError;
use super::models::TelegramMessage;
use super::store::TelegramStore;
use crate::platform::communications::ProviderMessageObservationEvent;

pub(in crate::integrations::telegram) struct TelegramAttachmentDownloadObservation<'a> {
    pub(in crate::integrations::telegram) provider_attachment_id: &'a str,
    pub(in crate::integrations::telegram) tdlib_file_id: i64,
    pub(in crate::integrations::telegram) download_state: &'a str,
    pub(in crate::integrations::telegram) local_path: Option<&'a str>,
    pub(in crate::integrations::telegram) size_bytes: Option<i64>,
    pub(in crate::integrations::telegram) content_type: &'a str,
    pub(in crate::integrations::telegram) filename: Option<&'a str>,
    pub(in crate::integrations::telegram) observed_at: DateTime<Utc>,
}

impl TelegramStore {
    pub(in crate::integrations::telegram) async fn append_message_metadata_observation(
        &self,
        message: &TelegramMessage,
        metadata: &Value,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "metadata_observed",
            Utc::now(),
            &json!({ "message_metadata": metadata }),
        )
        .await
    }

    pub(in crate::integrations::telegram) async fn append_message_content_observation(
        &self,
        message: &TelegramMessage,
        body_text: &str,
        metadata: &Value,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "content_observed",
            observed_at,
            &json!({
                "body_text": body_text,
                "message_metadata": metadata,
                "observed_at": observed_at,
            }),
        )
        .await
    }

    pub(in crate::integrations::telegram) async fn append_message_pin_observation(
        &self,
        message: &TelegramMessage,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "pinned_state_observed",
            observed_at,
            &json!({
                "is_pinned": is_pinned,
                "observed_at": observed_at,
            }),
        )
        .await
    }

    pub(in crate::integrations::telegram) async fn append_attachment_download_observation(
        &self,
        message: &TelegramMessage,
        observation: TelegramAttachmentDownloadObservation<'_>,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "attachment_download_state_observed",
            observation.observed_at,
            &json!({
                "provider_attachment_id": observation.provider_attachment_id,
                "provider_file_id": observation.tdlib_file_id,
                "download_state": observation.download_state,
                "local_path": observation.local_path,
                "size_bytes": observation.size_bytes,
                "content_type": observation.content_type,
                "filename": observation.filename,
                "observed_at": observation.observed_at,
            }),
        )
        .await
    }

    async fn append_message_observation_event(
        &self,
        message: &TelegramMessage,
        event_kind: &str,
        observed_at: DateTime<Utc>,
        payload: &Value,
    ) -> Result<Option<i64>, TelegramError> {
        self.provider_observation_events()
            .append_provider_message_observation(ProviderMessageObservationEvent {
                provider: "telegram",
                account_id: &message.account_id,
                channel_kind: message.channel_kind.as_str(),
                message_id: &message.message_id,
                external_message_id: &message.provider_message_id,
                event_kind,
                observed_at,
                external_event_id: None,
                payload,
                causation_id: None,
                correlation_id: None,
            })
            .await
            .map_err(Into::into)
    }
}
```

### `backend/src/integrations/telegram/client/participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/participants.rs`
- Size bytes / Размер в байтах: `22725`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgPool, Postgres, Row, Transaction};

use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::lifecycle::mark_command_reconciled;
use super::models::messages::TelegramProviderWriteCommand;
use super::models::{NewTelegramChatParticipant, TelegramChatMember};
use super::rows::row_to_telegram_provider_write_command;
use super::store::TelegramStore;
use super::validation::validate_chat_list_limit;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

fn row_to_provider_member(row: sqlx::postgres::PgRow) -> Result<TelegramChatMember, TelegramError> {
    Ok(TelegramChatMember {
        sender_id: row.try_get("provider_member_id")?,
        sender_display_name: row.try_get("display_name")?,
        message_count: 0,
        last_message_at: None,
        source: row.try_get("source")?,
        provider_member_id: row.try_get("provider_member_id")?,
        username: row.try_get("username")?,
        role: row.try_get("role")?,
        status: row.try_get("status")?,
        is_admin: row.try_get("is_admin")?,
        is_owner: row.try_get("is_owner")?,
        permissions: row.try_get("permissions")?,
        observed_at: row.try_get("observed_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn capture_chat_participant_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    telegram_chat_id: &str,
    account_id: &str,
    provider_chat_id: &str,
    member: &TelegramChatMember,
    raw_payload: &serde_json::Value,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), TelegramError> {
    let entity_id = format!("{telegram_chat_id}:{}", member.provider_member_id);
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_CHAT_PARTICIPANT",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "telegram_chat_id": telegram_chat_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_member_id": member.provider_member_id,
                "display_name": member.sender_display_name,
                "username": member.username,
                "role": member.role,
                "status": member.status,
                "is_admin": member.is_admin,
                "is_owner": member.is_owner,
                "permissions": member.permissions,
                "source": member.source,
                "observed_at": member.observed_at,
                "raw_payload": raw_payload,
                "operation": relationship_kind,
            }),
            format!("telegram-chat-participant://{entity_id}/{relationship_kind}"),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "chat_participant",
        entity_id,
        relationship_kind,
        json!({
            "telegram_chat_id": telegram_chat_id,
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_member_id": member.provider_member_id,
            "role": member.role,
            "status": member.status,
            "source": member.source,
        }),
    )
    .await?;
    Ok(())
}

pub async fn upsert_chat_participant(
    pool: &PgPool,
    participant: &NewTelegramChatParticipant,
) -> Result<TelegramChatMember, TelegramError> {
    let now = Utc::now();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO telegram_chat_participants (
            participant_id, telegram_chat_id, account_id, provider_chat_id, provider_member_id,
            display_name, username, role, status, is_admin, is_owner, permissions, raw_payload,
            source, observed_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $15, $15)
        ON CONFLICT (telegram_chat_id, provider_member_id)
        DO UPDATE SET
            account_id         = EXCLUDED.account_id,
            provider_chat_id   = EXCLUDED.provider_chat_id,
            display_name       = EXCLUDED.display_name,
            username           = EXCLUDED.username,
            role               = EXCLUDED.role,
            status             = EXCLUDED.status,
            is_admin           = EXCLUDED.is_admin,
            is_owner           = EXCLUDED.is_owner,
            permissions        = EXCLUDED.permissions,
            raw_payload        = EXCLUDED.raw_payload,
            source             = EXCLUDED.source,
            observed_at        = EXCLUDED.observed_at,
            updated_at         = EXCLUDED.updated_at
        RETURNING provider_member_id, display_name, username, role, status, is_admin, is_owner,
                  permissions, source, observed_at
        "#,
    )
    .bind(&participant.participant_id)
    .bind(&participant.telegram_chat_id)
    .bind(&participant.account_id)
    .bind(&participant.provider_chat_id)
    .bind(&participant.provider_member_id)
    .bind(&participant.display_name)
    .bind(&participant.username)
    .bind(&participant.role)
    .bind(&participant.status)
    .bind(participant.is_admin)
    .bind(participant.is_owner)
    .bind(&participant.permissions)
    .bind(&participant.raw_payload)
    .bind(&participant.source)
    .bind(now)
    .fetch_one(&mut *transaction)
    .await
    .map_err(TelegramError::from)?;

    let member = row_to_provider_member(row)?;
    capture_chat_participant_observation_in_transaction(
        &mut transaction,
        &participant.telegram_chat_id,
        &participant.account_id,
        &participant.provider_chat_id,
        &member,
        &participant.raw_payload,
        "upsert",
        "telegram.client.participants.upsert_chat_participant",
        member.observed_at.unwrap_or(now),
    )
    .await?;
    transaction.commit().await?;
    Ok(member)
}

pub async fn mark_absent_members_from_exhaustive_roster(
    pool: &PgPool,
    telegram_chat_id: &str,
    observed_member_ids: &[String],
    observed_via: &str,
) -> Result<Vec<TelegramChatMember>, TelegramError> {
    let observed_at = Utc::now();
    let permissions_patch = json!({
        "membership_state": "absent_exhaustive",
        "observed_via": observed_via,
    });
    let raw_payload_patch = json!({
        "membership_state": "absent_exhaustive",
        "observed_via": observed_via,
    });
    let mut transaction = pool.begin().await?;
    let rows = sqlx::query(
        r#"
        UPDATE telegram_chat_participants
        SET status = 'absent_exhaustive',
            permissions = COALESCE(permissions, '{}'::jsonb) || $3::jsonb,
            raw_payload = COALESCE(raw_payload, '{}'::jsonb) || $4::jsonb,
            observed_at = $2,
            updated_at = $2
        WHERE telegram_chat_id = $1
          AND source = 'tdlib'
          AND provider_member_id <> ALL($5)
          AND status IS DISTINCT FROM 'absent_exhaustive'
        RETURNING account_id, provider_chat_id, provider_member_id, display_name, username, role,
                  status, is_admin, is_owner, permissions, raw_payload, source, observed_at
        "#,
    )
    .bind(telegram_chat_id)
    .bind(observed_at)
    .bind(&permissions_patch)
    .bind(&raw_payload_patch)
    .bind(observed_member_ids)
    .fetch_all(&mut *transaction)
    .await
    .map_err(TelegramError::from)?;

    let mut members = Vec::with_capacity(rows.len());
    for row in rows {
        let account_id: String = row.try_get("account_id")?;
        let provider_chat_id: String = row.try_get("provider_chat_id")?;
        let raw_payload: serde_json::Value = row.try_get("raw_payload")?;
        let member = row_to_provider_member(row)?;
        capture_chat_participant_observation_in_transaction(
            &mut transaction,
            telegram_chat_id,
            &account_id,
            &provider_chat_id,
            &member,
            &raw_payload,
            "absent_exhaustive",
            "telegram.client.participants.mark_absent_members_from_exhaustive_roster",
            member.observed_at.unwrap_or(observed_at),
        )
        .await?;
        members.push(member);
    }
    transaction.commit().await?;
    Ok(members)
}

pub fn telegram_self_provider_member_id(external_account_id: &str) -> Option<String> {
    let value = external_account_id.trim();
    if value.is_empty() {
        return None;
    }

    if let Some(user_id) = value.strip_prefix("user:").filter(|id| is_numeric_id(id)) {
        return Some(format!("user:{user_id}"));
    }

    let user_id = value.strip_prefix("telegram:").unwrap_or(value).trim();
    is_numeric_id(user_id).then(|| format!("user:{user_id}"))
}

pub async fn reconcile_join_commands_from_provider_roster(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_member_id: &str,
    observed_at: chrono::DateTime<Utc>,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_join_commands_from_provider_roster_with_source(
        pool,
        account_id,
        provider_chat_id,
        provider_member_id,
        observed_at,
        "tdlib.getSupergroupMembers",
    )
    .await
}

pub async fn reconcile_join_commands_from_provider_roster_with_source(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_member_id: &str,
    observed_at: chrono::DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let provider_state = json!({
        "provider_chat_id": provider_chat_id,
        "provider_member_id": provider_member_id,
        "observed_via": observed_via,
        "membership_state": "present",
    });
    let result_payload = json!({
        "source": observed_via,
        "provider_chat_id": provider_chat_id,
        "provider_member_id": provider_member_id,
        "membership_state": "present",
        "provider_observed_at": observed_at,
    });
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = 'join'
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state.clone(),
                result_payload.clone(),
            )
            .await?,
        );
    }
    Ok(reconciled)
}

#[allow(clippy::too_many_arguments)]
pub async fn reconcile_leave_commands_from_provider_roster(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_member_id: &str,
    membership_state: &str,
    status: Option<&str>,
    role: Option<&str>,
    observed_at: chrono::DateTime<Utc>,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_leave_commands_from_provider_roster_with_source(
        pool,
        account_id,
        provider_chat_id,
        provider_m
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/participants/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/participants/tests.rs`
- Size bytes / Размер в байтах: `6069`
- Included characters / Включено символов: `6069`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::models::{
    NewTelegramChat, NewTelegramChatParticipant, TelegramChatKind, TelegramSyncState,
};
use super::super::store::TelegramStore;
use super::{
    TelegramChatMember, inactive_roster_membership_state,
    mark_absent_members_from_exhaustive_roster, tdlib_self_membership_lifecycle,
    telegram_self_provider_member_id, upsert_chat_participant,
};

#[test]
fn derives_self_provider_member_id_only_from_numeric_telegram_identity() {
    assert_eq!(
        telegram_self_provider_member_id("telegram:12345").as_deref(),
        Some("user:12345")
    );
    assert_eq!(
        telegram_self_provider_member_id("user:456").as_deref(),
        Some("user:456")
    );
    assert_eq!(telegram_self_provider_member_id("fixture-user"), None);
    assert_eq!(
        telegram_self_provider_member_id("telegram:not-numeric"),
        None
    );
}

#[test]
fn parses_self_leave_and_join_membership_evidence_from_tdlib_service_messages() {
    let leave = tdlib_self_membership_lifecycle(
        "telegram:42",
        &json!({
            "content": {
                "@type": "messageChatDeleteMember",
                "user_id": 42
            }
        }),
    )
    .expect("leave lifecycle");
    assert_eq!(leave.command_kind, "leave");
    assert_eq!(leave.provider_member_id, "user:42");
    assert_eq!(leave.observed_via, "tdlib.messageChatDeleteMember");
    assert_eq!(leave.membership_state, "left");

    let join = tdlib_self_membership_lifecycle(
        "telegram:42",
        &json!({
            "content": {
                "@type": "messageChatAddMembers",
                "member_user_ids": [1, 42, 9]
            }
        }),
    )
    .expect("join lifecycle");
    assert_eq!(join.command_kind, "join");
    assert_eq!(join.provider_member_id, "user:42");
    assert_eq!(join.observed_via, "tdlib.messageChatAddMembers");
    assert_eq!(join.membership_state, "present");
}

#[test]
fn ignores_service_messages_for_other_members_or_unsupported_content() {
    assert!(
        tdlib_self_membership_lifecycle(
            "telegram:42",
            &json!({
                "content": {
                    "@type": "messageChatDeleteMember",
                    "user_id": 7
                }
            }),
        )
        .is_none()
    );
    assert!(
        tdlib_self_membership_lifecycle(
            "telegram:42",
            &json!({
                "content": {
                    "@type": "messageChatJoinByLink"
                }
            }),
        )
        .is_none()
    );
}

#[test]
fn derives_inactive_roster_membership_state_from_status_or_role() {
    let member = TelegramChatMember {
        sender_id: "user:42".to_owned(),
        sender_display_name: None,
        message_count: 0,
        last_message_at: None,
        source: "tdlib".to_owned(),
        provider_member_id: "user:42".to_owned(),
        username: None,
        role: Some("member".to_owned()),
        status: Some("left".to_owned()),
        is_admin: false,
        is_owner: false,
        permissions: json!({}),
        observed_at: None,
    };
    assert_eq!(inactive_roster_membership_state(&member), Some("left"));

    let banned = TelegramChatMember {
        status: Some("administrator".to_owned()),
        role: Some("banned".to_owned()),
        ..member
    };
    assert_eq!(inactive_roster_membership_state(&banned), Some("banned"));
}

#[tokio::test]
async fn marks_stale_tdlib_participants_as_absent_from_exhaustive_roster() {
    use crate::platform::storage::Database;
    use testkit::context::TestContext;

    let ctx = TestContext::new().await;
    let database = Database::connect(Some(&ctx.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();

    crate::test_support::upsert_telegram_runtime_account(
        &pool,
        "acct-1",
        "Telegram Test Account",
        "telegram:1",
    )
    .await;

    let store = crate::test_support::telegram_store(&pool);
    let chat = store
        .upsert_chat(&NewTelegramChat {
            account_id: "acct-1".to_owned(),
            provider_chat_id: "provider-chat-1".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Roster Room".to_owned(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: None,
            metadata: json!({}),
        })
        .await
        .expect("insert telegram chat");

    for (participant_id, provider_member_id, display_name) in [
        ("participant-1", "user:1", "User One"),
        ("participant-2", "user:2", "User Two"),
    ] {
        upsert_chat_participant(
            &pool,
            &NewTelegramChatParticipant {
                participant_id: participant_id.to_owned(),
                telegram_chat_id: chat.telegram_chat_id.clone(),
                account_id: "acct-1".to_owned(),
                provider_chat_id: "provider-chat-1".to_owned(),
                provider_member_id: provider_member_id.to_owned(),
                display_name: Some(display_name.to_owned()),
                username: None,
                role: "member".to_owned(),
                status: "member".to_owned(),
                is_admin: false,
                is_owner: false,
                permissions: json!({}),
                raw_payload: json!({}),
                source: "tdlib".to_owned(),
            },
        )
        .await
        .expect("insert participant");
    }

    let updated = mark_absent_members_from_exhaustive_roster(
        &pool,
        &chat.telegram_chat_id,
        &[String::from("user:1")],
        "tdlib.getSupergroupMembers.exhaustive_absence",
    )
    .await
    .expect("mark absent");

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].provider_member_id, "user:2");
    assert_eq!(updated[0].status.as_deref(), Some("absent_exhaustive"));
    assert_eq!(
        updated[0].permissions["membership_state"],
        "absent_exhaustive"
    );
}
```

### `backend/src/integrations/telegram/client/reactions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/reactions.rs`
- Size bytes / Размер в байтах: `22156`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};

use super::commands::insert_command;
use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::lifecycle::{mark_command_mismatch, mark_command_reconciled};
use super::messages::reaction_metadata::TdlibProviderReaction;
use super::models::messages::{
    TelegramCommandKind, TelegramProviderWriteCommand, TelegramReaction, TelegramReactionGroup,
    TelegramReactionRequest, TelegramReactionResponse, TelegramReactionSummary,
};
use super::rows::{row_to_telegram_provider_write_command, row_to_telegram_reaction};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

const REACTION_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different reaction state than requested";

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

fn new_reaction_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgreact_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("react_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

fn provider_reaction_id(message_id: &str, sender_id: &str, reaction_emoji: &str) -> String {
    format!(
        "tmsgreact_provider_{}",
        stable_short_hash(&format!("{message_id}\0{sender_id}\0{reaction_emoji}"))
    )
}

pub(in crate::integrations::telegram) struct TelegramReactionMessageRef<'a> {
    pub(in crate::integrations::telegram) message_id: &'a str,
    pub(in crate::integrations::telegram) account_id: &'a str,
    pub(in crate::integrations::telegram) provider_chat_id: &'a str,
    pub(in crate::integrations::telegram) provider_message_id: &'a str,
}

struct TelegramSelfReactionSync<'a> {
    sender_id: &'a str,
    chosen_reactions: &'a [String],
    observed_at: chrono::DateTime<Utc>,
}

async fn capture_reaction_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    reaction: &TelegramReaction,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_MESSAGE_REACTION",
            ObservationOriginKind::LocalRuntime,
            reaction.updated_at,
            json!({
                "reaction_id": reaction.reaction_id,
                "message_id": reaction.message_id,
                "account_id": reaction.account_id,
                "provider_message_id": reaction.provider_message_id,
                "provider_chat_id": reaction.provider_chat_id,
                "sender_id": reaction.sender_id,
                "sender_display_name": reaction.sender_display_name,
                "reaction_emoji": reaction.reaction_emoji,
                "is_active": reaction.is_active,
                "observed_at": reaction.observed_at,
                "source_event": reaction.source_event,
                "provider_actor_id": reaction.provider_actor_id,
                "metadata": reaction.metadata,
                "provenance": reaction.provenance,
                "operation": relationship_kind,
            }),
            format!(
                "telegram-message-reaction://{}/{}",
                reaction.reaction_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "message_reaction",
        reaction.reaction_id.clone(),
        relationship_kind,
        json!({
            "message_id": reaction.message_id,
            "account_id": reaction.account_id,
            "provider_message_id": reaction.provider_message_id,
            "provider_chat_id": reaction.provider_chat_id,
            "sender_id": reaction.sender_id,
            "reaction_emoji": reaction.reaction_emoji,
            "is_active": reaction.is_active,
        }),
    )
    .await?;
    Ok(())
}

pub(in crate::integrations::telegram) async fn sync_provider_reactions(
    pool: &PgPool,
    message_ref: TelegramReactionMessageRef<'_>,
    reactions: &[TdlibProviderReaction],
    self_sender_id: Option<&str>,
    chosen_reactions: &[String],
) -> Result<(), TelegramError> {
    let now = Utc::now();
    let mut transaction = pool.begin().await?;
    for reaction in reactions {
        let reaction_id = provider_reaction_id(
            message_ref.message_id,
            &reaction.sender_id,
            &reaction.reaction_emoji,
        );
        let row = sqlx::query(
            r#"
            INSERT INTO telegram_message_reactions
                (reaction_id, message_id, account_id, provider_message_id, provider_chat_id,
                 sender_id, sender_display_name, reaction_emoji, is_active, observed_at,
                 source_event, provider_actor_id, metadata, provenance)
            VALUES ($1, $2, $3, $4, $5, $6, NULL, $7, true, $8,
                    'tdlib_interaction_info', $9, $10, $11)
            ON CONFLICT (message_id, sender_id, reaction_emoji)
            DO UPDATE SET
                is_active = true,
                observed_at = EXCLUDED.observed_at,
                source_event = EXCLUDED.source_event,
                provider_actor_id = EXCLUDED.provider_actor_id,
                metadata = EXCLUDED.metadata,
                provenance = EXCLUDED.provenance,
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(&reaction_id)
        .bind(message_ref.message_id)
        .bind(message_ref.account_id)
        .bind(message_ref.provider_message_id)
        .bind(message_ref.provider_chat_id)
        .bind(&reaction.sender_id)
        .bind(&reaction.reaction_emoji)
        .bind(now)
        .bind(&reaction.sender_id)
        .bind(json!({
            "is_outgoing": reaction.is_outgoing,
            "source": "tdlib_interaction_info",
        }))
        .bind(json!({
            "provider": "telegram",
            "runtime": "tdlib",
            "source": "interaction_info.reactions.recent_reactions",
        }))
        .fetch_one(&mut *transaction)
        .await?;
        let stored = row_to_telegram_reaction(row)?;
        capture_reaction_observation_in_transaction(
            &mut transaction,
            &stored,
            "provider_sync_activate",
            "telegram.client.reactions.sync_provider_reactions",
        )
        .await?;
    }
    if let Some(self_sender_id) = self_sender_id {
        sync_self_provider_reactions(
            &mut transaction,
            &message_ref,
            TelegramSelfReactionSync {
                sender_id: self_sender_id,
                chosen_reactions,
                observed_at: now,
            },
        )
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}

async fn sync_self_provider_reactions(
    transaction: &mut Transaction<'_, Postgres>,
    message_ref: &TelegramReactionMessageRef<'_>,
    sync: TelegramSelfReactionSync<'_>,
) -> Result<(), TelegramError> {
    let deactivated_rows = sqlx::query(
        r#"
        UPDATE telegram_message_reactions
        SET is_active = false,
            observed_at = $4,
            source_event = 'tdlib_interaction_info',
            provider_actor_id = $3,
            metadata = jsonb_build_object(
                'source', 'tdlib_interaction_info',
                'is_chosen', false,
                'is_outgoing', true
            ),
            provenance = jsonb_build_object(
                'provider', 'telegram',
                'runtime', 'tdlib',
                'source', 'interaction_info.reactions.reactions'
            ),
            updated_at = now()
        WHERE message_id = $1
          AND sender_id = $2
          AND (
              COALESCE(array_length($5::text[], 1), 0) = 0
              OR reaction_emoji <> ALL($5::text[])
          )
          AND is_active = true
        RETURNING *
        "#,
    )
    .bind(message_ref.message_id)
    .bind(sync.sender_id)
    .bind(sync.sender_id)
    .bind(sync.observed_at)
    .bind(sync.chosen_reactions)
    .fetch_all(&mut **transaction)
    .await?;
    for row in deactivated_rows {
        let stored = row_to_telegram_reaction(row)?;
        capture_reaction_observation_in_transaction(
            transaction,
            &stored,
            "provider_sync_deactivate",
            "telegram.client.reactions.sync_self_provider_reactions",
        )
        .await?;
    }

    for reaction_emoji in sync.chosen_reactions {
        let reaction_id =
            provider_reaction_id(message_ref.message_id, sync.sender_id, reaction_emoji);
        let row = sqlx::query(
            r#"
            INSERT INTO telegram_message_reactions
                (reaction_id, message_id, account_id, provider_message_id, provider_chat_id,
                 sender_id, sender_display_name, reaction_emoji, is_active, observed_at,
                 source_event, provider_actor_id, metadata, provenance)
            VALUES ($1, $2, $3, $4, $5, $6, NULL, $7, true, $8,
                    'tdlib_interaction_info', $6, $9, $10)
            ON CONFLICT (message_id, sender_id, reaction_emoji)
            DO UPDATE SET
                is_active = true,
                observed_at = EXCLUDED.observed_at,
                source_event = EXCLUDED.source_event,
                provider_actor_id = EXCLUDED.provider_actor_id,
                metadata = EXCLUDED.metadata,
                provenance = EXCLUDED.provenance,
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(&reaction_id)
        .bind(message_ref.message_id)
        .bind(message_ref.account_id)
        .bind(message_ref.provider_message_id)
        .bind(message_ref.provider_chat_id)
        .bind(sync.sender_id)
        .bind(reaction_emoji)
        .bind(sync.observed_at)
        .bind(json!({
            "is_outgoing": true,
            "is_chosen": true,
            "source": "tdlib_interaction_info",
        }))
        .bind(json!({
            "provider": "telegram",
            "runtime": "tdlib",
            "source": "interaction_info.reactions.reactions",
        }))
        .fetch_one(&mut **transaction)
        .await?;
        let stored = row_to_telegram_reaction(row)?;
        capture_reaction_observation_in_transaction(
            transaction,
            &stored,
            "provider_sync_activate",
            "telegram.client.reactions.sync_self_provider_reactions",
        )
        .await?;
    }

    Ok(())
}

pub async fn reconcile_reaction_commands_from_provider_reactions(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    chosen_reactions: &[String],
    observed_at: chrono::DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let chosen_reactions = normalized_reaction_emojis(chosen_reactions);
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND provider_message_id = $3
          AND command_kind IN ('react', 'unreact')
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/reactions/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/reactions/tests.rs`
- Size bytes / Размер в байтах: `2803`
- Included characters / Включено символов: `2788`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use testkit::context::TestContext;

use super::*;
use crate::integrations::telegram::client::models::messages::TelegramReactionRequest;

#[tokio::test]
async fn provider_state_sync_deactivates_absent_self_reactions() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "reaction-sync", "telegram:123").await;
    let message_id = "msg_reaction_sync";
    let provider_chat_id = "-100reaction-sync";
    let provider_message_id = "-100reaction-sync:77";
    let self_sender_id = "user:123";

    let add_request = TelegramReactionRequest {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        sender_id: self_sender_id.to_owned(),
        sender_display_name: Some("Owner".to_owned()),
        reaction_emoji: "👍".to_owned(),
        command_id: None,
    };
    add_reaction(&pool, &add_request, message_id)
        .await
        .expect("add chosen reaction");

    let remove_request = TelegramReactionRequest {
        reaction_emoji: "🔥".to_owned(),
        ..add_request.clone()
    };
    add_reaction(&pool, &remove_request, message_id)
        .await
        .expect("add stale reaction");

    sync_provider_reactions(
        &pool,
        TelegramReactionMessageRef {
            message_id,
            account_id: &account_id,
            provider_chat_id,
            provider_message_id,
        },
        &[],
        Some(self_sender_id),
        &["👍".to_owned()],
    )
    .await
    .expect("sync provider reactions");

    let rows = sqlx::query(
        r#"
        SELECT reaction_emoji, is_active
        FROM telegram_message_reactions
        WHERE message_id = $1 AND sender_id = $2
        ORDER BY reaction_emoji ASC
        "#,
    )
    .bind(message_id)
    .bind(self_sender_id)
    .fetch_all(&pool)
    .await
    .expect("reaction rows");

    let states = rows
        .into_iter()
        .map(|row| {
            (
                row.try_get::<String, _>("reaction_emoji")
                    .expect("reaction_emoji"),
                row.try_get::<bool, _>("is_active").expect("is_active"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        states,
        vec![("👍".to_owned(), true), ("🔥".to_owned(), false)]
    );
}

async fn create_telegram_account(
    pool: &sqlx::PgPool,
    suffix: &str,
    external_account_id: &str,
) -> String {
    let account_id = format!("telegram-reactions-{suffix}");
    crate::test_support::upsert_telegram_runtime_account(
        pool,
        &account_id,
        &format!("Telegram Reactions {suffix}"),
        external_account_id,
    )
    .await;
    account_id
}
```
