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

- Chunk ID / ID чанка: `064-source-backend-part-044`
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

### `backend/src/integrations/telegram/runtime/manager/message_events/projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/message_events/projection.rs`
- Size bytes / Размер в байтах: `3718`
- Included characters / Включено символов: `3718`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::integrations::telegram::client::{
    TelegramError, TelegramMessage, TelegramStore, derive_tdlib_reaction_summary_metadata,
};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibMessageContentSnapshot, TelegramTdlibMessageEditedSnapshot,
};

pub(super) async fn update_message_reaction_summary(
    store: &TelegramStore,
    message: &TelegramMessage,
    raw: &Value,
) -> Result<Option<TelegramMessage>, TelegramError> {
    let mut metadata = message.metadata.clone();
    let Some(metadata_map) = metadata.as_object_mut() else {
        return Err(TelegramError::InvalidRequest(
            "telegram message metadata must be a JSON object".to_owned(),
        ));
    };

    if let Some(summary) = derive_tdlib_reaction_summary_metadata(raw) {
        metadata_map.insert("reaction_summary".to_owned(), summary);
    } else {
        metadata_map.remove("reaction_summary");
    }

    store
        .append_message_metadata_observation(message, &metadata)
        .await?;

    let mut observed = message.clone();
    observed.metadata = metadata;
    Ok(Some(observed))
}

pub(super) async fn project_provider_message_content_observation(
    store: &TelegramStore,
    message: &TelegramMessage,
    snapshot: &TelegramTdlibMessageContentSnapshot,
    observed_at: DateTime<Utc>,
) -> Result<Option<TelegramMessage>, TelegramError> {
    let mut metadata = message.metadata.clone();
    let Some(metadata_map) = metadata.as_object_mut() else {
        return Err(TelegramError::InvalidRequest(
            "telegram message metadata must be a JSON object".to_owned(),
        ));
    };
    metadata_map.insert("text".to_owned(), Value::String(snapshot.text.clone()));
    metadata_map.insert("tdlib_content".to_owned(), snapshot.new_content.clone());
    metadata_map.insert(
        "last_provider_content_update_source".to_owned(),
        Value::String(snapshot.source_event.clone()),
    );

    store
        .append_message_content_observation(message, &snapshot.text, &metadata, observed_at)
        .await?;

    let mut observed = message.clone();
    observed.text = snapshot.text.clone();
    observed.metadata = metadata;
    Ok(Some(observed))
}

pub(super) async fn project_provider_message_edit_observation(
    store: &TelegramStore,
    message: &TelegramMessage,
    snapshot: &TelegramTdlibMessageEditedSnapshot,
) -> Result<Option<TelegramMessage>, TelegramError> {
    let mut metadata = message.metadata.clone();
    let Some(metadata_map) = metadata.as_object_mut() else {
        return Err(TelegramError::InvalidRequest(
            "telegram message metadata must be a JSON object".to_owned(),
        ));
    };
    metadata_map.insert(
        "provider_edit_timestamp".to_owned(),
        Value::String(snapshot.edit_timestamp.to_rfc3339()),
    );
    metadata_map.insert(
        "last_provider_edit_source".to_owned(),
        Value::String(snapshot.source_event.clone()),
    );
    if let Some(reply_markup) = &snapshot.reply_markup {
        metadata_map.insert("tdlib_reply_markup".to_owned(), reply_markup.clone());
    }

    store
        .append_message_metadata_observation(message, &metadata)
        .await?;

    let mut observed = message.clone();
    observed.metadata = metadata;
    Ok(Some(observed))
}

pub(super) fn observed_edit_timestamp(
    message: &TelegramMessage,
    fallback: DateTime<Utc>,
) -> DateTime<Utc> {
    message
        .metadata
        .get("provider_edit_timestamp")
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or(fallback)
}
```

### `backend/src/integrations/telegram/runtime/manager/message_events/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/message_events/tests.rs`
- Size bytes / Размер в байтах: `9921`
- Included characters / Включено символов: `9915`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use testkit::context::TestContext;

use super::*;
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramChatKind, TelegramDeliveryState,
};

async fn seed_runtime_account(pool: &sqlx::PgPool, account_id: &str, external: &str) {
    crate::test_support::upsert_telegram_runtime_account(
        pool,
        account_id,
        "Telegram Runtime Account",
        external,
    )
    .await;
}

#[tokio::test]
async fn publish_message_content_updated_event_skips_without_projected_message() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-content-runtime";
    let provider_chat_id = "-100content-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = EventBus::new();
    let mut events = event_bus.subscribe();

    seed_runtime_account(&pool, account_id, "telegram-ext-content").await;

    let store = crate::test_support::telegram_store(&pool);
    let _observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Content Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: "telegram-runtime-content".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageContentSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        text: "after".to_owned(),
        new_content: json!({
            "@type": "messageText",
            "text": {"@type": "formattedText", "text": "after"}
        }),
        source_event: "updateMessageContent".to_owned(),
        raw: json!({"@type": "message"}),
    };

    publish_message_content_updated_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    assert!(events.try_recv().is_err());
}

#[tokio::test]
async fn publish_message_created_event_publishes_signal_hub_raw_signal_instead_of_legacy_event() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-created-runtime";
    let provider_chat_id = "-100created-runtime";
    let event_bus = EventBus::new();
    let mut events = event_bus.subscribe();

    seed_runtime_account(&pool, account_id, "telegram-ext-created").await;

    let snapshot = TelegramTdlibMessageSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: "42".to_owned(),
        sender_id: "user:777".to_owned(),
        sender_display_name: "Alice".to_owned(),
        text: "hello from runtime".to_owned(),
        occurred_at: Utc::now(),
        delivery_state: TelegramDeliveryState::Received,
        raw: json!({
            "@type": "message",
            "chat_id": provider_chat_id,
            "id": 42,
            "content": {
                "@type": "messageText",
                "text": {
                    "@type": "formattedText",
                    "text": "hello from runtime"
                }
            }
        }),
    };

    publish_message_created_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    let event = events.try_recv().expect("raw signal broadcast");
    assert_eq!(event.event_type, "signal.raw.telegram.message.observed");

    let legacy_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'telegram.message.created'",
    )
    .fetch_one(&pool)
    .await
    .expect("legacy event count");
    assert_eq!(legacy_count, 0);

    let raw_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.telegram.message.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("raw signal count");
    assert_eq!(raw_signal_count, 1);

    let raw_record_id = event.source["source_id"]
        .as_str()
        .expect("raw signal source_id");
    let raw_record = crate::test_support::load_communication_raw_record(&pool, raw_record_id).await;
    assert_eq!(raw_record.account_id, account_id);
    assert_eq!(
        raw_record.provider_record_id,
        format!("{provider_chat_id}:42")
    );
}

#[tokio::test]
async fn publish_message_edited_event_skips_without_projected_message() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-edited-runtime";
    let provider_chat_id = "-100edited-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = EventBus::new();
    let mut events = event_bus.subscribe();
    let edit_timestamp = Utc::now();

    seed_runtime_account(&pool, account_id, "telegram-ext-edited").await;

    let store = crate::test_support::telegram_store(&pool);
    let _observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Edited Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "hello".to_owned(),
            import_batch_id: "telegram-runtime-edited".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageEditedSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        edit_timestamp,
        reply_markup: Some(json!({
            "@type": "replyMarkupInlineKeyboard",
            "rows": []
        })),
        source_event: "updateMessageEdited".to_owned(),
        raw: json!({"@type": "message"}),
    };

    publish_message_edited_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    assert!(events.try_recv().is_err());
}

#[tokio::test]
async fn publish_reaction_changed_event_skips_without_projected_message() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-reaction-runtime";
    let provider_chat_id = "-100reaction-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = EventBus::new();
    let mut events = event_bus.subscribe();

    seed_runtime_account(&pool, account_id, "telegram-ext-reaction").await;

    let store = crate::test_support::telegram_store(&pool);
    let _observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Reaction Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "hello".to_owned(),
            import_batch_id: "telegram-runtime-reactions".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageInteractionInfoSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        source_event: "updateMessageInteractionInfo".to_owned(),
        raw: json!({
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
                            "total_count": 1,
                            "is_chosen": true
                        }
                    ],
                    "recent_reactions": [
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderUser",
                                "user_id": 888
                            },
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "is_outgoing": false
                        }
                    ]
                }
            }
        }),
    };

    publish_reaction_changed_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    assert!(events.try_recv().is_err());
}
```

### `backend/src/integrations/telegram/runtime/manager/participant_events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/participant_events.rs`
- Size bytes / Размер в байтах: `6115`
- Included characters / Включено символов: `6115`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::integrations::telegram::client::TelegramChatMember;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventStore, NewEventEnvelope};

use super::realtime_events::TelegramRuntimeEventBridgeContext;

pub(super) async fn publish_participant_updated_event(
    context: Option<&TelegramRuntimeEventBridgeContext>,
    account_id: &str,
    telegram_chat_id: &str,
    provider_chat_id: &str,
    participant: &TelegramChatMember,
    source: &str,
) {
    let Some(context) = context else {
        return;
    };

    let occurred_at = Utc::now();
    let Ok(event) = participant_updated_event(
        account_id,
        telegram_chat_id,
        provider_chat_id,
        participant,
        source,
        occurred_at,
    ) else {
        return;
    };

    if let Some(store) = &context.telegram_store {
        let pool = store.pool();
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(
                error = %error,
                "Telegram runtime event bridge: failed to append participant update event"
            );
        }
    }

    let _ = context.event_bus.broadcast(event);
}

fn participant_updated_event(
    account_id: &str,
    telegram_chat_id: &str,
    provider_chat_id: &str,
    participant: &TelegramChatMember,
    source: &str,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    let tdlib_event = participant_event_source(source);
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_participant_{}_{}_{}",
            telegram_chat_id,
            participant.provider_member_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::PARTICIPANT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat_participant",
            "telegram_chat_id": telegram_chat_id,
            "provider_chat_id": provider_chat_id,
            "provider_member_id": participant.provider_member_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": telegram_chat_id,
        "provider_chat_id": provider_chat_id,
        "participant": participant,
        "source": source
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": tdlib_event
    }))
    .build()
}

fn participant_event_source(source: &str) -> &str {
    source.strip_prefix("tdlib.").unwrap_or(source)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use serde_json::json;

    use super::*;

    #[test]
    fn participant_updated_event_contains_projection_member_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let participant = TelegramChatMember {
            sender_id: "user:42".to_owned(),
            sender_display_name: Some("Owner User".to_owned()),
            message_count: 0,
            last_message_at: None,
            source: "tdlib".to_owned(),
            provider_member_id: "user:42".to_owned(),
            username: Some("owner".to_owned()),
            role: Some("owner".to_owned()),
            status: Some("creator".to_owned()),
            is_admin: true,
            is_owner: true,
            permissions: json!({"can_invite_users": true}),
            observed_at: Some(occurred_at),
        };

        let event = participant_updated_event(
            "acct-1",
            "telegram_chat:v1:test",
            "-100123",
            &participant,
            "tdlib.getSupergroupMembers",
            occurred_at,
        )
        .expect("event");

        assert_eq!(event.event_type, telegram_event_types::PARTICIPANT_UPDATED);
        assert_eq!(event.subject["kind"], "telegram_chat_participant");
        assert_eq!(event.subject["provider_member_id"], "user:42");
        assert_eq!(event.payload["telegram_chat_id"], "telegram_chat:v1:test");
        assert_eq!(event.payload["participant"]["role"], "owner");
        assert_eq!(
            event.payload["participant"]["permissions"]["can_invite_users"],
            true
        );
        assert_eq!(event.payload["source"], "tdlib.getSupergroupMembers");
        assert_eq!(event.provenance["tdlib_event"], "getSupergroupMembers");
    }

    #[test]
    fn participant_updated_event_uses_runtime_source_as_provenance() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let participant = TelegramChatMember {
            sender_id: "user:42".to_owned(),
            sender_display_name: Some("Owner User".to_owned()),
            message_count: 0,
            last_message_at: None,
            source: "tdlib".to_owned(),
            provider_member_id: "user:42".to_owned(),
            username: Some("owner".to_owned()),
            role: Some("member".to_owned()),
            status: Some("absent_exhaustive".to_owned()),
            is_admin: false,
            is_owner: false,
            permissions: json!({
                "membership_state": "absent_exhaustive"
            }),
            observed_at: Some(occurred_at),
        };

        let event = participant_updated_event(
            "acct-1",
            "telegram_chat:v1:test",
            "-100123",
            &participant,
            "tdlib.getSupergroupMembers.exhaustive_absence",
            occurred_at,
        )
        .expect("event");

        assert_eq!(
            event.provenance["tdlib_event"],
            "getSupergroupMembers.exhaustive_absence"
        );
        assert_eq!(
            event.payload["source"],
            "tdlib.getSupergroupMembers.exhaustive_absence"
        );
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/participants.rs`
- Size bytes / Размер в байтах: `21653`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::Utc;
use serde_json::{Value, json};

use crate::integrations::telegram::client::participants::{
    inactive_roster_membership_state, reconcile_join_commands_from_provider_roster_with_source,
    reconcile_leave_commands_from_exhaustive_absence,
    reconcile_leave_commands_from_provider_roster_with_source, telegram_self_provider_member_id,
};
use crate::integrations::telegram::client::{
    NewTelegramChatParticipant, TelegramChat, TelegramChatMember, TelegramError,
    mark_absent_members_from_exhaustive_roster,
};

use super::super::participant_commands::{
    request_actor_get_basic_group_members, request_actor_get_supergroup_administrators,
    request_actor_get_supergroup_members,
};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::participant_events::publish_participant_updated_event;
use super::realtime_events::publish_command_reconciled_events;
use super::{TelegramMemberSyncContext, TelegramRuntimeManager};

const TELEGRAM_MEMBER_SYNC_TARGET_LIMIT: i32 = 500;

impl TelegramRuntimeManager {
    pub(crate) async fn sync_chat_members<S>(
        &self,
        context: TelegramMemberSyncContext<'_, S>,
        telegram_chat_id: &str,
    ) -> Result<Vec<TelegramChatMember>, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        let chat = context
            .telegram_store
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or(TelegramError::InvalidRequest(format!(
                "chat {telegram_chat_id} not found"
            )))?;

        let account = load_active_account(context.provider_account_store, &chat.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        if runtime_kind != "tdlib_qr_authorized" {
            return Ok(Vec::new());
        }

        if let Some(private_items) = sync_private_chat_members(
            context.telegram_store.pool(),
            &chat,
            &account.external_account_id,
        )
        .await?
        {
            for item in &private_items {
                publish_participant_updated_event(
                    context.event_bridge.as_ref(),
                    &chat.account_id,
                    &chat.telegram_chat_id,
                    &chat.provider_chat_id,
                    item,
                    "tdlib.chat.metadata",
                )
                .await;
            }
            return Ok(private_items);
        }

        if let Some(basic_group_id) = tdlib_basic_group_id(&chat.metadata) {
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
            let snapshots =
                request_actor_get_basic_group_members(command_tx, basic_group_id).await?;
            return sync_provider_roster_snapshots(
                context,
                &chat,
                &account.external_account_id,
                snapshots,
                "tdlib.getBasicGroupFullInfo",
                true,
            )
            .await;
        }

        let Some(supergroup_id) = tdlib_supergroup_id(&chat.metadata) else {
            return Ok(Vec::new());
        };

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

        let snapshots = request_actor_get_supergroup_members(
            command_tx.clone(),
            supergroup_id,
            TELEGRAM_MEMBER_SYNC_TARGET_LIMIT,
        )
        .await?;
        let roster_is_exhaustive = supergroup_roster_is_exhaustive(&snapshots);
        let admin_snapshots = request_actor_get_supergroup_administrators(
            command_tx,
            supergroup_id,
            TELEGRAM_MEMBER_SYNC_TARGET_LIMIT,
        )
        .await?;
        sync_provider_roster_snapshots(
            context,
            &chat,
            &account.external_account_id,
            merge_supergroup_member_snapshots(snapshots, admin_snapshots),
            "tdlib.getSupergroupMembers",
            roster_is_exhaustive,
        )
        .await
    }
}

fn tdlib_supergroup_id(metadata: &Value) -> Option<i64> {
    metadata
        .get("tdlib_supergroup_id")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
}

fn tdlib_basic_group_id(metadata: &Value) -> Option<i64> {
    metadata
        .get("tdlib_basic_group_id")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
}

async fn sync_provider_roster_snapshots<S>(
    context: TelegramMemberSyncContext<'_, S>,
    chat: &TelegramChat,
    external_account_id: &str,
    snapshots: Vec<crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot>,
    observed_via: &str,
    roster_is_exhaustive: bool,
) -> Result<Vec<TelegramChatMember>, TelegramError>
where
    S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
{
    let mut items = Vec::with_capacity(snapshots.len());
    for snapshot in snapshots {
        let participant = NewTelegramChatParticipant {
            participant_id: telegram_participant_id(
                &chat.telegram_chat_id,
                &snapshot.provider_member_id,
            ),
            telegram_chat_id: chat.telegram_chat_id.clone(),
            account_id: chat.account_id.clone(),
            provider_chat_id: chat.provider_chat_id.clone(),
            provider_member_id: snapshot.provider_member_id,
            display_name: snapshot.display_name,
            username: snapshot.username,
            role: snapshot.role,
            status: snapshot.status,
            is_admin: snapshot.is_admin,
            is_owner: snapshot.is_owner,
            permissions: snapshot.permissions,
            raw_payload: snapshot.raw,
            source: "tdlib".to_owned(),
        };
        let item = crate::integrations::telegram::client::participants::upsert_chat_participant(
            context.telegram_store.pool(),
            &participant,
        )
        .await?;
        publish_participant_updated_event(
            context.event_bridge.as_ref(),
            &chat.account_id,
            &chat.telegram_chat_id,
            &chat.provider_chat_id,
            &item,
            observed_via,
        )
        .await;
        items.push(item);
    }

    if roster_is_exhaustive {
        let observed_member_ids = items
            .iter()
            .map(|item| item.provider_member_id.clone())
            .collect::<Vec<_>>();
        let absent_members = mark_absent_members_from_exhaustive_roster(
            context.telegram_store.pool(),
            &chat.telegram_chat_id,
            &observed_member_ids,
            &format!("{observed_via}.exhaustive_absence"),
        )
        .await?;
        for member in absent_members {
            publish_participant_updated_event(
                context.event_bridge.as_ref(),
                &chat.account_id,
                &chat.telegram_chat_id,
                &chat.provider_chat_id,
                &member,
                &format!("{observed_via}.exhaustive_absence"),
            )
            .await;
        }
    }

    reconcile_self_membership_from_provider_roster(
        context,
        chat,
        external_account_id,
        observed_via,
        &items,
        roster_is_exhaustive,
    )
    .await?;

    Ok(items)
}

async fn sync_private_chat_members(
    pool: &sqlx::PgPool,
    chat: &TelegramChat,
    external_account_id: &str,
) -> Result<Option<Vec<TelegramChatMember>>, TelegramError> {
    let Some(private_user_id) = tdlib_private_user_id(&chat.metadata) else {
        return Ok(None);
    };

    let mut participants = Vec::new();
    if chat
        .metadata
        .get("is_saved_messages")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        if let Some(provider_member_id) = telegram_self_provider_member_id(external_account_id) {
            participants.push(build_private_chat_participant(
                chat,
                provider_member_id,
                Some(chat.title.clone()),
                chat.username.clone(),
            ));
        }
    } else {
        participants.push(build_private_chat_participant(
            chat,
            format!("user:{private_user_id}"),
            Some(chat.title.clone()),
            chat.username.clone(),
        ));
    }

    let mut items = Vec::with_capacity(participants.len());
    for participant in participants {
        items.push(
            crate::integrations::telegram::client::participants::upsert_chat_participant(
                pool,
                &participant,
            )
            .await?,
        );
    }

    Ok(Some(items))
}

fn tdlib_private_user_id(metadata: &Value) -> Option<String> {
    metadata
        .get("tdlib_private_user_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn build_private_chat_participant(
    chat: &TelegramChat,
    provider_member_id: String,
    display_name: Option<String>,
    username: Option<String>,
) -> NewTelegramChatParticipant {
    NewTelegramChatParticipant {
        participant_id: telegram_participant_id(&chat.telegram_chat_id, &provider_member_id),
        telegram_chat_id: chat.telegram_chat_id.clone(),
        account_id: chat.account_id.clone(),
        provider_chat_id: chat.provider_chat_id.clone(),
        provider_member_id,
        display_name,
        username,
        role: "member".to_owned(),
        status: "member".to_owned(),
        is_admin: false,
        is_owner: false,
        permissions: json!({
            "observed_via": "tdlib.chat.metadata",
            "tdlib_chat_type": chat.metadata.get("tdlib_chat_type").cloned().unwrap_or(Value::Null),
            "is_saved_messages": chat.metadata.get("is_saved_messages").and_then(Value::as_bool).unwrap_or(false),
        }),
        raw_payload: json!({
            "observed_via": "tdlib.chat.metadata",
            "tdlib_private_user_id": chat.metadata.get("tdlib_private_user_id").cloned().unwrap_or(Value::Null),
            "tdlib_chat_type": chat.metadata.get("tdlib_chat_type").cloned().unwrap_or(Value::Null),
            "is_saved_messages": chat.metadata.get("is_saved_messages").and_then(Value::as_bool).unwrap_or(false),
        }),
        source: "tdlib".to_owned(),
    }
}

fn telegram_participant_id(telegram_chat_id: &str, provider_member_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(telegram_chat_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_member_id.as_bytes());
    format!("telegram_participant:v1:{:x}", hasher.finalize())
}

fn is_active_provider_member(item: &TelegramChatMember, provider_member_id: &str) -> bool {
    if item.provider_member_id != provider_member_id {
        return false;
    }
    let inactive_status = matches!(item.status.as_deref(), Some("left" | "banned"));
    let inactive_role = matches!(item.role.as_deref(), Some("left" | "banned"));
    !(inactive_status || inactive_role)
}

async fn reconcile_self_membership_from_provider_roster<S>(
    context: TelegramMemberSyncContext<'_, S>,
    chat: &TelegramChat,
    external_account_id: &str,
    observed_via: &str,
    items: &[TelegramChatMember],
    roster_is_exhaustive: bool,
) -> Result<(), TelegramError>
where
    S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/participants_runtime_tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/participants_runtime_tests.rs`
- Size bytes / Размер в байтах: `12251`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::super::{TelegramMemberSyncContext, TelegramRuntimeEventBridgeContext};
use super::sync_provider_roster_snapshots;
use crate::integrations::telegram::client::TelegramChat;
use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::models::{
    NewTelegramChat, TelegramChatKind, TelegramSyncState,
};
use crate::integrations::telegram::client::participants::upsert_chat_participant;
use crate::integrations::telegram::client::{
    NewTelegramChatParticipant, TelegramError, TelegramStore,
};
use crate::platform::config::AppConfig;
use crate::platform::events::EventBus;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::secrets::{InMemorySecretResolver, SecretReferenceStore};
use serde_json::json;
use sqlx::{PgPool, Row};
use testkit::context::TestContext;

async fn seed_chat(
    pool: &PgPool,
    account_id: &str,
    external_account_id: &str,
    provider_chat_id: &str,
) -> Result<TelegramChat, TelegramError> {
    crate::test_support::upsert_telegram_runtime_account(
        pool,
        account_id,
        "Runtime Participant Account",
        external_account_id,
    )
    .await;
    crate::test_support::telegram_store(pool)
        .upsert_chat(&NewTelegramChat {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Runtime Participants".to_owned(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: None,
            metadata: json!({}),
        })
        .await
}

#[tokio::test]
async fn sync_provider_roster_snapshots_appends_join_reconciliation_after_participant_update() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-1";
    let provider_chat_id = "-10042";
    let chat = seed_chat(&pool, account_id, "user:42", provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "join-runtime-reconciled";
    insert_command(
        &pool,
        command_id,
        account_id,
        "join",
        "join:runtime:test",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({}),
        json!({"telegram_chat_id": chat.telegram_chat_id, "provider_chat_id": provider_chat_id}),
        json!({"source": "test"}),
    )
    .await
    .expect("seed join command");

    let provider_account_store = crate::test_support::communication_provider_account_store(&pool);
    let provider_secret_binding_store =
        crate::test_support::communication_provider_secret_binding_store(&pool);
    let telegram_store = crate::test_support::telegram_store(&pool);
    let secret_store = SecretReferenceStore::new(pool.clone());
    let secret_resolver = InMemorySecretResolver::new();
    let config = AppConfig::default();
    let event_bridge = Some(TelegramRuntimeEventBridgeContext::new(
        Some(telegram_store.clone()),
        EventBus::new(),
    ));
    let context = TelegramMemberSyncContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &telegram_store,
        secret_store: &secret_store,
        secret_resolver: &secret_resolver,
        config: &config,
        event_bridge,
    };

    sync_provider_roster_snapshots(
        context,
        &chat,
        "user:42",
        vec![
            crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot {
                provider_member_id: "user:42".to_owned(),
                display_name: Some("Owner User".to_owned()),
                username: Some("owner".to_owned()),
                role: "member".to_owned(),
                status: "member".to_owned(),
                is_admin: false,
                is_owner: false,
                permissions: json!({}),
                raw: json!({}),
            },
        ],
        "tdlib.getSupergroupMembers",
        true,
    )
    .await
    .expect("sync members");

    let rows: Vec<(String, serde_json::Value, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, subject, payload
        FROM event_log
        WHERE event_type IN (
            'telegram.participant.updated',
            'telegram.command.status_changed',
            'telegram.command.reconciled'
        )
        ORDER BY position ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("runtime events");

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, telegram_event_types::PARTICIPANT_UPDATED);
    assert_eq!(rows[0].1["provider_member_id"], json!("user:42"));
    assert_eq!(rows[1].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(rows[1].1["id"], json!(command_id));
    assert_eq!(rows[2].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(rows[2].1["id"], json!(command_id));

    let command_status: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_optional(&pool)
    .await
    .expect("command status");
    assert_eq!(
        command_status,
        Some(("completed".to_owned(), "observed".to_owned()))
    );
    let participant_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'chat_participant'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(format!("{}:user:42", chat.telegram_chat_id))
    .fetch_all(&pool)
    .await
    .expect("participant observations");
    assert!(
        participant_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_CHAT_PARTICIPANT"
                && row.get::<String, _>("relationship_kind") == "upsert"
                && row.get::<serde_json::Value, _>("payload")["provider_member_id"]
                    == json!("user:42")
        }),
        "participant upsert observation must exist"
    );
}

#[tokio::test]
async fn sync_provider_roster_snapshots_appends_leave_reconciliation_after_absence_update() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-2";
    let provider_chat_id = "-10043";
    let chat = seed_chat(&pool, account_id, "user:42", provider_chat_id)
        .await
        .expect("seed chat");
    let _ = upsert_chat_participant(
        &pool,
        &NewTelegramChatParticipant {
            participant_id: "participant-self".to_owned(),
            telegram_chat_id: chat.telegram_chat_id.clone(),
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_member_id: "user:42".to_owned(),
            display_name: Some("Owner User".to_owned()),
            username: Some("owner".to_owned()),
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
    .expect("seed participant");
    let command_id = "leave-runtime-reconciled";
    insert_command(
        &pool,
        command_id,
        account_id,
        "leave",
        "leave:runtime:test",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({}),
        json!({"telegram_chat_id": chat.telegram_chat_id, "provider_chat_id": provider_chat_id}),
        json!({"source": "test"}),
    )
    .await
    .expect("seed leave command");

    let provider_account_store = crate::test_support::communication_provider_account_store(&pool);
    let provider_secret_binding_store =
        crate::test_support::communication_provider_secret_binding_store(&pool);
    let telegram_store = crate::test_support::telegram_store(&pool);
    let secret_store = SecretReferenceStore::new(pool.clone());
    let secret_resolver = InMemorySecretResolver::new();
    let config = AppConfig::default();
    let event_bridge = Some(TelegramRuntimeEventBridgeContext::new(
        Some(telegram_store.clone()),
        EventBus::new(),
    ));
    let context = TelegramMemberSyncContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &telegram_store,
        secret_store: &secret_store,
        secret_resolver: &secret_resolver,
        config: &config,
        event_bridge,
    };

    sync_provider_roster_snapshots(
        context,
        &chat,
        "user:42",
        Vec::new(),
        "tdlib.getSupergroupMembers",
        true,
    )
    .await
    .expect("sync members");

    let rows: Vec<(String, serde_json::Value, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, subject, payload
        FROM event_log
        WHERE event_type IN (
            'telegram.participant.updated',
            'telegram.command.status_changed',
            'telegram.command.reconciled'
        )
        ORDER BY position ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("runtime events");

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, telegram_event_types::PARTICIPANT_UPDATED);
    assert_eq!(rows[0].1["provider_member_id"], json!("user:42"));
    assert_eq!(
        rows[0].2["participant"]["status"],
        json!("absent_exhaustive")
    );
    assert_eq!(rows[1].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(rows[1].1["id"], json!(command_id));
    assert_eq!(rows[2].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(rows[2].1["id"], json!(command_id));

    let command_status: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_optional(&pool)
    .await
    .expect("command status");
    assert_eq!(
        command_status,
        Some(("completed".to_owned(), "observed".to_owned()))
    );
    let participant_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'chat_participant'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(format!("{}:user:42", chat.telegram_chat_id))
    .fetch_all(&pool)
    .await
    .expect("participant observations");
    assert!(
        participant_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_CHAT_PARTICIPANT"
                && row.get::<String, _>("relationship_kind") == "upsert"
        }),
        "seed participant upsert observation must exist"
    );
    assert!(
        participant_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_CHAT_PARTICIPANT"
                && row.get::<String, _>
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/realtime_events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/realtime_events.rs`
- Size bytes / Размер в байтах: `21324`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::integrations::telegram::client::TelegramStore;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::tdjson::TelegramTdlibTypingSnapshot;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

use super::super::state::TelegramRuntimeEvent;
use super::chat_events::{
    publish_chat_folders_event, publish_chat_marked_as_unread_event,
    publish_chat_notification_settings_event, publish_chat_position_event,
    publish_chat_removed_from_list_event, publish_chat_unread_event,
};
use super::message_events::{
    publish_message_content_updated_event, publish_message_created_event,
    publish_message_deleted_event, publish_message_edited_event, publish_message_pinned_event,
    publish_reaction_changed_event,
};
use super::topic_events::publish_topic_event;

const TELEGRAM_RUNTIME_EVENT_BRIDGE_RUNTIME: &str = "telegram_runtime_event_bridge";

#[derive(Clone)]
pub struct TelegramRuntimeEventBridgeContext {
    pub(super) telegram_store: Option<TelegramStore>,
    pub(super) event_bus: EventBus,
}

impl TelegramRuntimeEventBridgeContext {
    pub(crate) fn new(telegram_store: Option<TelegramStore>, event_bus: EventBus) -> Self {
        Self {
            telegram_store,
            event_bus,
        }
    }
}

pub(super) fn spawn_telegram_runtime_event_bridge(
    telegram_store: Option<TelegramStore>,
    event_bus: EventBus,
    account_id: String,
    mut runtime_events: UnboundedReceiver<TelegramRuntimeEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = runtime_events.recv().await {
            if !telegram_runtime_event_bridge_allows_processing(&telegram_store).await {
                continue;
            }
            match event {
                TelegramRuntimeEvent::MessageCreated(snapshot) => {
                    publish_message_created_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageContentUpdated(snapshot) => {
                    publish_message_content_updated_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageEdited(snapshot) => {
                    publish_message_edited_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessagePinnedUpdated(snapshot) => {
                    publish_message_pinned_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageDeleted(snapshot) => {
                    publish_message_deleted_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageInteractionInfoUpdated(snapshot) => {
                    publish_reaction_changed_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::TypingChanged(snapshot) => {
                    publish_typing_event(&telegram_store, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::TopicUpdated(snapshot) => {
                    publish_topic_event(&telegram_store, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::ChatUnreadUpdated(snapshot) => {
                    publish_chat_unread_event(&telegram_store, &event_bus, &account_id, &snapshot)
                        .await;
                }
                TelegramRuntimeEvent::ChatMarkedAsUnreadUpdated(snapshot) => {
                    publish_chat_marked_as_unread_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatNotificationSettingsUpdated(snapshot) => {
                    publish_chat_notification_settings_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatPositionUpdated(snapshot) => {
                    publish_chat_position_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatRemovedFromList(snapshot) => {
                    publish_chat_removed_from_list_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatFoldersUpdated(folders) => {
                    publish_chat_folders_event(&telegram_store, &event_bus, &account_id, &folders)
                        .await;
                }
            }
        }
    });
}

async fn telegram_runtime_event_bridge_allows_processing(
    telegram_store: &Option<TelegramStore>,
) -> bool {
    let Some(store) = telegram_store else {
        return true;
    };

    match crate::platform::events::runtime_allows_processing(
        store.pool(),
        "telegram",
        TELEGRAM_RUNTIME_EVENT_BRIDGE_RUNTIME,
        &json!({
            "label": "Telegram realtime event bridge",
            "scope": "subscription",
            "runtime": "tdlib",
        }),
    )
    .await
    {
        Ok(allowed) => allowed,
        Err(error) => {
            tracing::warn!(
                error = %error,
                runtime_kind = TELEGRAM_RUNTIME_EVENT_BRIDGE_RUNTIME,
                "telegram runtime event bridge gate check failed"
            );
            true
        }
    }
}

pub(super) async fn publish_command_reconciled_events(
    context: Option<&TelegramRuntimeEventBridgeContext>,
    command: &TelegramProviderWriteCommand,
    source: &str,
) {
    let Some(context) = context else {
        return;
    };
    let payload = json!({
        "source": source,
        "reconciliation_status": command.reconciliation_status,
        "provider_observed_at": command.provider_observed_at,
        "reconciled_at": command.reconciled_at,
        "provider_state": command.provider_state,
    });
    publish_command_event(
        context,
        command,
        telegram_event_types::COMMAND_STATUS_CHANGED,
        payload.clone(),
    )
    .await;
    publish_command_event(
        context,
        command,
        telegram_event_types::COMMAND_RECONCILED,
        payload,
    )
    .await;
}

pub(super) fn command_event_payload(
    command: &TelegramProviderWriteCommand,
    status: &str,
    extra_payload: serde_json::Value,
) -> serde_json::Value {
    let mut payload = json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "command_kind": command.command_kind,
        "provider_chat_id": command.provider_chat_id,
        "message_id": command.provider_message_id,
        "status": status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "result_payload": command.result_payload,
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "provider_state": command.provider_state,
        "reconciliation_status": command.reconciliation_status,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    if let Some(payload_obj) = payload.as_object_mut() {
        payload_obj.insert("payload".to_owned(), extra_payload);
    }
    payload
}

async fn publish_command_event(
    context: &TelegramRuntimeEventBridgeContext,
    command: &TelegramProviderWriteCommand,
    event_type: &str,
    extra_payload: serde_json::Value,
) {
    let now = Utc::now();
    let payload = command_event_payload(command, &command.status, extra_payload);

    let event = NewEventEnvelope::builder(
        format!(
            "evt_telegram_command_{}_{}_{}",
            event_type.replace('.', "_"),
            command.command_id,
            now.timestamp_nanos_opt().unwrap_or(0)
        ),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_command"}),
    )
    .payload(payload)
    .build();

    let Ok(event) = event else {
        return;
    };

    if let Some(store) = &context.telegram_store {
        let pool = store.pool();
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append command reconciliation event");
        }
    }

    let _ = context.event_bus.broadcast(event);
}

#[cfg(test)]
mod typing_tests {
    use chrono::Utc;
    use serde_json::json;
    use testkit::context::TestContext;

    use super::command_event_payload;
    use super::{TelegramRuntimeEventBridgeContext, publish_command_reconciled_events};
    use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
    use crate::platform::events::EventBus;

    fn sample_command() -> TelegramProviderWriteCommand {
        TelegramProviderWriteCommand {
            command_id: "cmd-1".to_owned(),
            account_id: "account-1".to_owned(),
            command_kind: "edit".to_owned(),
            idempotency_key: "idem-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            provider_message_id: Some("chat-1:42".to_owned()),
            target_ref: json!({"provider_message_id": "chat-1:42"}),
            payload: json!({"new_text": "after"}),
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "not_required".to_owned(),
            status: "completed".to_owned(),
            retry_count: 2,
            max_retries: 5,
            last_error: Some("temporary failure".to_owned()),
            result_payload: json!({"projection_message_id": "msg-1"}),
            audit_metadata: json!({"source": "test"}),
            actor_id: "makosh-frontend".to_owned(),
            happened_at: Utc::now(),
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/registry.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/registry.rs`
- Size bytes / Размер в байтах: `2414`
- Included characters / Включено символов: `2414`
- Truncated / Обрезано: `no`

```rust
use std::sync::mpsc::Sender;

use crate::integrations::telegram::client::TelegramError;

use super::super::state::{
    TelegramRuntimeActorHandle, TelegramRuntimeActorState, TelegramRuntimeCommand,
};
use super::super::validation::validate_non_empty;
use super::TelegramRuntimeManager;

impl TelegramRuntimeManager {
    pub(in crate::integrations::telegram::runtime::manager) fn actor_state(
        &self,
        account_id: &str,
    ) -> Result<Option<TelegramRuntimeActorState>, TelegramError> {
        let actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors.get(account_id).map(|handle| handle.state.clone()))
    }

    pub fn stop_account(&self, account_id: &str) -> Result<bool, TelegramError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let mut actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors.remove(&account_id).is_some())
    }

    pub(in crate::integrations::telegram::runtime::manager) fn set_actor_handle(
        &self,
        account_id: String,
        actor_handle: TelegramRuntimeActorHandle,
    ) -> Result<(), TelegramError> {
        let mut actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        actors.insert(account_id, actor_handle);
        Ok(())
    }

    pub(in crate::integrations::telegram::runtime::manager) fn actor_command_tx(
        &self,
        account_id: &str,
    ) -> Result<Option<Sender<TelegramRuntimeCommand>>, TelegramError> {
        let actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors
            .get(account_id)
            .and_then(|handle| handle.command_tx.clone()))
    }

    pub(crate) fn active_account_ids(&self) -> Result<Vec<String>, TelegramError> {
        let actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors
            .iter()
            .filter(|(_, handle)| handle.command_tx.is_some())
            .map(|(id, _)| id.clone())
            .collect())
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/search.rs`
- Size bytes / Размер в байтах: `4200`
- Included characters / Включено символов: `4200`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;

use super::super::commands::{request_actor_search_chat_messages, request_actor_search_messages};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

pub struct TelegramProviderSearchRequest {
    pub account_id: String,
    pub provider_chat_id: Option<String>,
    pub query: String,
    pub limit: i32,
}

impl TelegramRuntimeManager {
    /// Calls TDLib `searchMessages` or `searchChatMessages` and ingests results.
    ///
    /// Returns ingested message IDs. Falls back to Ok(vec![]) for fixture mode or when no
    /// active actor is available.
    pub(crate) async fn search_provider_messages<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramProviderSearchRequest,
    ) -> Result<Vec<String>, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        if request.query.trim().is_empty() {
            return Ok(vec![]);
        }

        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);

        if runtime_kind != "tdlib_qr_authorized" {
            return Ok(vec![]);
        }

        let command_tx = match self
            .ensure_tdlib_actor(
                context.provider_secret_binding_store,
                context.secret_store,
                context.secret_resolver,
                context.config,
                &account,
                context.event_bridge.clone(),
            )
            .await
        {
            Ok(tx) => tx,
            Err(error) => {
                tracing::debug!(
                    error = %error,
                    account_id = %request.account_id,
                    "search_provider_messages: TDLib actor unavailable"
                );
                return Ok(vec![]);
            }
        };

        let snapshots = if let Some(provider_chat_id) = &request.provider_chat_id {
            request_actor_search_chat_messages(
                command_tx,
                provider_chat_id.clone(),
                request.query.clone(),
                request.limit,
            )
            .await?
        } else {
            request_actor_search_messages(command_tx, request.query.clone(), request.limit).await?
        };

        let import_batch_id = format!(
            "telegram-search:{}:{}",
            request.account_id,
            &request.query[..request.query.len().min(32)]
        );

        let mut message_ids = Vec::with_capacity(snapshots.len());
        for snapshot in &snapshots {
            match context
                .telegram_store
                .ingest_tdlib_message_snapshot(&request.account_id, snapshot, &import_batch_id)
                .await
            {
                Ok(result) => {
                    if let Err(error) = context
                        .telegram_store
                        .publish_observed_message_raw_signal(
                            &result,
                            context
                                .event_bridge
                                .as_ref()
                                .map(|bridge| &bridge.event_bus),
                        )
                        .await
                    {
                        tracing::warn!(
                            error = %error,
                            provider_message_id = %snapshot.provider_message_id,
                            "search_provider_messages: failed to publish Signal Hub raw signal"
                        );
                    }
                    message_ids.push(result.message_id);
                }
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        provider_message_id = %snapshot.provider_message_id,
                        "search_provider_messages: failed to ingest snapshot"
                    );
                }
            }
        }

        Ok(message_ids)
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/send.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/send.rs`
- Size bytes / Размер в байтах: `8755`
- Included characters / Включено символов: `8755`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::{
    TelegramError, TelegramForwardRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramReplyRequest, telegram_text_preview_hash,
};

use super::super::commands::{request_actor_forward, request_actor_reply, request_actor_send};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

impl TelegramRuntimeManager {
    pub(crate) async fn send_manual_message<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => context.telegram_store.manual_send_message(request).await,
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
                let snapshot = request_actor_send(command_tx, request.clone()).await?;
                let import_batch_id = format!(
                    "telegram-manual-send:{}:{}",
                    account.account_id,
                    request.command_id.trim()
                );
                let result = context
                    .telegram_store
                    .ingest_tdlib_message_snapshot(&account.account_id, &snapshot, &import_batch_id)
                    .await?;
                Ok(TelegramManualSendResponse {
                    raw: Some(result.raw),
                    raw_record_id: result.raw_record_id,
                    message_id: result.message_id,
                    account_id: account.account_id,
                    provider_chat_id: request.provider_chat_id.trim().to_owned(),
                    delivery_state: snapshot.delivery_state.as_str().to_owned(),
                    status: "sent".to_owned(),
                    runtime_kind,
                    rendered_preview_hash: telegram_text_preview_hash(&request.text),
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

    pub(crate) async fn send_reply_message<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramReplyRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => Err(TelegramError::InvalidRequest(
                "reply command is not supported in fixture mode".to_owned(),
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
                let snapshot = request_actor_reply(
                    command_tx,
                    request.provider_chat_id.trim().to_owned(),
                    request.reply_to_provider_message_id.trim().to_owned(),
                    request.text.trim().to_owned(),
                    request.command_id.trim().to_owned(),
                )
                .await?;
                let import_batch_id = format!(
                    "telegram-reply:{}:{}",
                    account.account_id,
                    request.command_id.trim()
                );
                let result = context
                    .telegram_store
                    .ingest_tdlib_message_snapshot(&account.account_id, &snapshot, &import_batch_id)
                    .await?;
                Ok(TelegramManualSendResponse {
                    raw: Some(result.raw),
                    raw_record_id: result.raw_record_id,
                    message_id: result.message_id,
                    account_id: account.account_id,
                    provider_chat_id: request.provider_chat_id.trim().to_owned(),
                    delivery_state: snapshot.delivery_state.as_str().to_owned(),
                    status: "sent".to_owned(),
                    runtime_kind,
                    rendered_preview_hash: telegram_text_preview_hash(&request.text),
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

    pub(crate) async fn send_forward_message<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramForwardRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => Err(TelegramError::InvalidRequest(
                "forward command is not supported in fixture mode".to_owned(),
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
                let snapshot = request_actor_forward(
                    command_tx,
                    request.provider_chat_id.trim().to_owned(),
                    request.from_provider_chat_id.trim().to_owned(),
                    request.from_provider_message_id.trim().to_owned(),
                    request.command_id.trim().to_owned(),
                )
                .await?;
                let import_batch_id = format!(
                    "telegram-forward:{}:{}",
                    account.account_id,
                    request.command_id.trim()
                );
                let result = context
                    .telegram_store
                    .ingest_tdlib_message_snapshot(&account.account_id, &snapshot, &import_batch_id)
                    .await?;
                Ok(TelegramManualSendResponse {
                    raw: Some(result.raw),
                    raw_record_id: result.raw_record_id,
                    message_id: result.message_id,
                    account_id: account.account_id,
                    provider_chat_id: request.provider_chat_id.trim().to_owned(),
                    delivery_state: snapshot.delivery_state.as_str().to_owned(),
                    status: "sent".to_owned(),
                    runtime_kind,
                    rendered_preview_hash: telegram_text_preview_hash(&snapshot.text),
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

### `backend/src/integrations/telegram/runtime/manager/sync_chats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/sync_chats.rs`
- Size bytes / Размер в байтах: `4469`
- Included characters / Включено символов: `4469`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeSet;

use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;

use super::super::commands::{request_actor_chat_folders, request_actor_chats};
use super::super::models::{TelegramChatSyncRequest, TelegramChatSyncResponse};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

impl TelegramRuntimeManager {
    pub(crate) async fn sync_chats<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramChatSyncRequest,
    ) -> Result<TelegramChatSyncResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => {
                let items = context
                    .telegram_store
                    .list_chats(Some(&account.account_id), request.limit.unwrap_or(50))
                    .await?;
                Ok(TelegramChatSyncResponse {
                    account_id: account.account_id,
                    runtime_kind,
                    status: "synced".to_owned(),
                    synced_count: items.len(),
                    items,
                })
            }
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
                let snapshots =
                    request_actor_chats(command_tx.clone(), request.limit.unwrap_or(50) as i32)
                        .await?;
                for snapshot in &snapshots {
                    context
                        .telegram_store
                        .ingest_tdlib_chat_snapshot(&account.account_id, snapshot)
                        .await?;
                }
                let folder_ids = tdlib_folder_ids_from_chat_snapshots(&snapshots);
                if !folder_ids.is_empty() {
                    let folder_snapshots =
                        request_actor_chat_folders(command_tx, folder_ids).await?;
                    context
                        .telegram_store
                        .apply_provider_chat_folders(&account.account_id, &folder_snapshots)
                        .await?;
                }
                let items = context
                    .telegram_store
                    .list_chats(Some(&account.account_id), request.limit.unwrap_or(50))
                    .await?;
                Ok(TelegramChatSyncResponse {
                    account_id: account.account_id,
                    runtime_kind,
                    status: "synced".to_owned(),
                    synced_count: snapshots.len(),
                    items,
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

fn tdlib_folder_ids_from_chat_snapshots(
    snapshots: &[crate::integrations::telegram::tdjson::TelegramTdlibChatSnapshot],
) -> Vec<i64> {
    let mut folder_ids = BTreeSet::new();
    for snapshot in snapshots {
        let Some(lists) = snapshot.raw.get("positions").and_then(Value::as_array) else {
            continue;
        };
        for list in lists {
            let Some(chat_list) = list.get("list").and_then(Value::as_object) else {
                continue;
            };
            if chat_list.get("@type").and_then(Value::as_str) != Some("chatListFolder") {
                continue;
            }
            if let Some(folder_id) = chat_list.get("chat_folder_id").and_then(Value::as_i64) {
                folder_ids.insert(folder_id);
            }
        }
    }
    folder_ids.into_iter().collect()
}
```

### `backend/src/integrations/telegram/runtime/manager/sync_history.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/sync_history.rs`
- Size bytes / Размер в байтах: `3004`
- Included characters / Включено символов: `3004`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::{TelegramError, TelegramStore};

use super::super::actor::oldest_tdlib_message_id;
use super::super::commands::request_actor_history;
use super::super::models::{
    TelegramHistorySyncMode, TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::sync_history_tdlib::TdlibHistorySyncContext;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

impl TelegramRuntimeManager {
    pub(crate) async fn sync_history<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramHistorySyncRequest,
    ) -> Result<TelegramHistorySyncResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => {
                sync_fixture_history(context.telegram_store, &account.account_id, request).await
            }
            "tdlib_qr_authorized" => {
                let context = TdlibHistorySyncContext {
                    provider_account_store: context.provider_account_store,
                    provider_secret_binding_store: context.provider_secret_binding_store,
                    telegram_store: context.telegram_store,
                    secret_store: context.secret_store,
                    secret_resolver: context.secret_resolver,
                    config: context.config,
                    account: &account,
                    runtime_kind,
                    event_bridge: context.event_bridge.clone(),
                };
                self.sync_tdlib_history(context, request).await
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

async fn sync_fixture_history(
    telegram_store: &TelegramStore,
    account_id: &str,
    request: &TelegramHistorySyncRequest,
) -> Result<TelegramHistorySyncResponse, TelegramError> {
    let items = telegram_store
        .recent_messages(
            Some(account_id),
            Some(&request.provider_chat_id),
            request.limit.unwrap_or(50),
        )
        .await?;
    Ok(TelegramHistorySyncResponse {
        account_id: account_id.to_owned(),
        provider_chat_id: request.provider_chat_id.trim().to_owned(),
        runtime_kind: "fixture".to_owned(),
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: false,
        next_from_message_id: None,
        items,
    })
}
```

### `backend/src/integrations/telegram/runtime/manager/sync_history_tdlib.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/sync_history_tdlib.rs`
- Size bytes / Размер в байтах: `7413`
- Included characters / Включено символов: `7413`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::participants::{
    reconcile_participant_commands_from_message_evidence, tdlib_self_membership_lifecycle,
};
use crate::integrations::telegram::client::{
    TelegramError, TelegramStore, derive_tdlib_chosen_reaction_emojis,
    reconcile_reaction_commands_from_provider_reactions,
};
use crate::platform::communications::{
    ProviderAccount, ProviderAccountLookupPort, ProviderSecretBindingLookupPort,
};
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::super::actor::oldest_tdlib_message_id;
use super::super::commands::request_actor_history;
use super::super::models::{
    TelegramHistorySyncMode, TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};
use super::TelegramRuntimeManager;
use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};

pub(in crate::integrations::telegram::runtime::manager) struct TdlibHistorySyncContext<
    'a,
    S: SecretResolver + Sync + ?Sized,
> {
    pub(in crate::integrations::telegram::runtime::manager) provider_account_store:
        &'a dyn ProviderAccountLookupPort,
    pub(in crate::integrations::telegram::runtime::manager) provider_secret_binding_store:
        &'a dyn ProviderSecretBindingLookupPort,
    pub(in crate::integrations::telegram::runtime::manager) telegram_store: &'a TelegramStore,
    pub(in crate::integrations::telegram::runtime::manager) secret_store: &'a SecretReferenceStore,
    pub(in crate::integrations::telegram::runtime::manager) secret_resolver: &'a S,
    pub(in crate::integrations::telegram::runtime::manager) config: &'a AppConfig,
    pub(in crate::integrations::telegram::runtime::manager) account: &'a ProviderAccount,
    pub(in crate::integrations::telegram::runtime::manager) runtime_kind: String,
    pub(in crate::integrations::telegram::runtime::manager) event_bridge:
        Option<TelegramRuntimeEventBridgeContext>,
}

impl TelegramRuntimeManager {
    pub(in crate::integrations::telegram::runtime::manager) async fn sync_tdlib_history<
        S: SecretResolver + Sync + ?Sized,
    >(
        &self,
        context: TdlibHistorySyncContext<'_, S>,
        request: &TelegramHistorySyncRequest,
    ) -> Result<TelegramHistorySyncResponse, TelegramError> {
        let mode = request.mode();
        if mode == TelegramHistorySyncMode::Full {
            ensure_private_chat_for_full_sync(context.telegram_store, context.account, request)
                .await?;
        }
        let command_tx = self
            .ensure_tdlib_actor(
                context.provider_secret_binding_store,
                context.secret_store,
                context.secret_resolver,
                context.config,
                context.account,
                context.event_bridge.clone(),
            )
            .await?;
        let snapshots = request_actor_history(
            command_tx,
            request.provider_chat_id.trim().to_owned(),
            request.from_message_id,
            request.limit.unwrap_or(50) as i32,
            mode,
        )
        .await?;
        let next_from_message_id = oldest_tdlib_message_id(&snapshots);
        let has_more = mode != TelegramHistorySyncMode::Full
            && next_from_message_id.is_some()
            && snapshots.len() >= request.limit.unwrap_or(50) as usize;
        let import_batch_id = format!(
            "telegram-tdlib-history-sync:{}:{}",
            context.account.account_id,
            request.provider_chat_id.trim()
        );
        for snapshot in &snapshots {
            let observed = context
                .telegram_store
                .ingest_tdlib_message_snapshot(
                    &context.account.account_id,
                    snapshot,
                    &import_batch_id,
                )
                .await?;
            context
                .telegram_store
                .publish_observed_message_raw_signal(
                    &observed,
                    context
                        .event_bridge
                        .as_ref()
                        .map(|bridge| &bridge.event_bus),
                )
                .await?;
            if let Some(lifecycle) =
                tdlib_self_membership_lifecycle(&context.account.external_account_id, &snapshot.raw)
            {
                let commands = reconcile_participant_commands_from_message_evidence(
                    context.telegram_store.pool(),
                    &context.account.account_id,
                    &snapshot.provider_chat_id,
                    &snapshot.provider_message_id,
                    snapshot.occurred_at,
                    &lifecycle,
                )
                .await?;
                for command in commands {
                    publish_command_reconciled_events(
                        context.event_bridge.as_ref(),
                        &command,
                        &lifecycle.observed_via,
                    )
                    .await;
                }
            }
            let chosen_reactions = derive_tdlib_chosen_reaction_emojis(&snapshot.raw);
            let commands = reconcile_reaction_commands_from_provider_reactions(
                context.telegram_store.pool(),
                &context.account.account_id,
                &snapshot.provider_chat_id,
                &snapshot.provider_message_id,
                &chosen_reactions,
                snapshot.occurred_at,
                "tdlib.interaction_info.reactions",
            )
            .await?;
            for command in commands {
                publish_command_reconciled_events(
                    context.event_bridge.as_ref(),
                    &command,
                    "tdlib.interaction_info.reactions",
                )
                .await;
            }
        }
        let items = context
            .telegram_store
            .recent_messages(
                Some(&context.account.account_id),
                Some(&request.provider_chat_id),
                request.limit.unwrap_or(50),
            )
            .await?;
        Ok(TelegramHistorySyncResponse {
            account_id: context.account.account_id.clone(),
            provider_chat_id: request.provider_chat_id.trim().to_owned(),
            runtime_kind: context.runtime_kind,
            status: "synced".to_owned(),
            synced_count: snapshots.len(),
            has_more,
            next_from_message_id,
            items,
        })
    }
}

async fn ensure_private_chat_for_full_sync(
    telegram_store: &TelegramStore,
    account: &ProviderAccount,
    request: &TelegramHistorySyncRequest,
) -> Result<(), TelegramError> {
    let chat = telegram_store
        .telegram_chat(&account.account_id, &request.provider_chat_id)
        .await?
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram chat `{}` is not synced for account `{}`",
                request.provider_chat_id.trim(),
                account.account_id
            ))
        })?;
    if chat.chat_kind != "private" {
        return Err(TelegramError::InvalidRequest(
            "full Telegram history sync is only allowed for private chats; group and channel history must be paged with mode=older"
                .to_owned(),
        ));
    }
    Ok(())
}
```

### `backend/src/integrations/telegram/runtime/manager/tdlib_actor.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/tdlib_actor.rs`
- Size bytes / Размер в байтах: `2437`
- Included characters / Включено символов: `2437`
- Truncated / Обрезано: `no`

```rust
use std::sync::mpsc::Sender;

use chrono::Utc;

use crate::integrations::telegram::client::TelegramError;
use crate::platform::communications::{ProviderAccount, ProviderSecretBindingLookupPort};
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::super::actor::{optional_telegram_session_key, spawn_tdlib_actor};
use super::super::state::{TelegramRuntimeActorHandle, TelegramRuntimeCommand};
use super::TelegramRuntimeManager;
use super::actor_states::running_actor_state;
use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, spawn_telegram_runtime_event_bridge,
};

impl TelegramRuntimeManager {
    pub(in crate::integrations::telegram::runtime::manager) async fn ensure_tdlib_actor(
        &self,
        provider_secret_binding_store: &dyn ProviderSecretBindingLookupPort,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        account: &ProviderAccount,
        event_bridge: Option<TelegramRuntimeEventBridgeContext>,
    ) -> Result<Sender<TelegramRuntimeCommand>, TelegramError> {
        if let Some(command_tx) = self.actor_command_tx(&account.account_id)? {
            return Ok(command_tx);
        }

        let session_encryption_key = optional_telegram_session_key(
            provider_secret_binding_store,
            secret_store,
            secret_resolver,
            &account.account_id,
        )
        .await?;
        let (runtime_event_tx, runtime_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let runtime_event_tx = event_bridge.as_ref().map(|_| runtime_event_tx);
        let command_tx = spawn_tdlib_actor(
            config.clone(),
            account.clone(),
            session_encryption_key,
            runtime_event_tx,
        )?;
        if let Some(event_bridge) = event_bridge {
            spawn_telegram_runtime_event_bridge(
                event_bridge.telegram_store,
                event_bridge.event_bus,
                account.account_id.clone(),
                runtime_event_rx,
            );
        }
        self.set_actor_handle(
            account.account_id.clone(),
            TelegramRuntimeActorHandle {
                state: running_actor_state(Utc::now()),
                command_tx: Some(command_tx.clone()),
            },
        )?;
        Ok(command_tx)
    }
}
```

### `backend/src/integrations/telegram/runtime/manager/topic_events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/topic_events.rs`
- Size bytes / Размер в байтах: `14716`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::lifecycle::mark_command_reconciled;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::models::topics::{NewTelegramTopic, TelegramTopic};
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibTopicSnapshot, TelegramTdlibTopicUpdateSnapshot,
};
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};
use super::topics::telegram_topic_id;

pub(super) async fn publish_topic_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibTopicUpdateSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let observed_at = Utc::now();
    let topic = match upsert_topic_snapshot(
        store,
        account_id,
        &snapshot.provider_chat_id,
        &snapshot.topic,
    )
    .await
    {
        Ok(Some(topic)) => topic,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project topic update");
            return;
        }
    };

    let reconciled = match reconcile_topic_commands_from_provider_state(
        pool,
        account_id,
        &snapshot.provider_chat_id,
        snapshot.topic.provider_topic_id,
        snapshot.topic.is_closed,
        observed_at,
        "tdlib.updateForumTopicInfo",
    )
    .await
    {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, topic_id = %topic.topic_id, "Telegram runtime event bridge: failed to reconcile topic commands");
            Vec::new()
        }
    };
    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, "updateForumTopicInfo").await;
    }

    let Ok(event) = topic_updated_event(account_id, &topic, observed_at) else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append topic event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn upsert_topic_snapshot(
    store: &TelegramStore,
    account_id: &str,
    provider_chat_id: &str,
    snapshot: &TelegramTdlibTopicSnapshot,
) -> Result<Option<TelegramTopic>, TelegramError> {
    let Some(chat) = store.telegram_chat(account_id, provider_chat_id).await? else {
        return Ok(None);
    };

    let topic = NewTelegramTopic {
        topic_id: telegram_topic_id(&chat.telegram_chat_id, snapshot.provider_topic_id),
        telegram_chat_id: chat.telegram_chat_id.clone(),
        account_id: chat.account_id,
        provider_topic_id: snapshot.provider_topic_id,
        provider_chat_id: provider_chat_id.to_owned(),
        title: snapshot.title.clone(),
        icon_emoji: snapshot.icon_emoji.clone(),
        is_pinned: snapshot.is_pinned,
        is_closed: snapshot.is_closed,
        unread_count: topic_unread_count(snapshot.unread_count),
        last_message_at: snapshot.last_message_at,
    };

    crate::integrations::telegram::client::topics::upsert_topic(store.pool(), &topic)
        .await
        .map(Some)
}

fn topic_unread_count(unread_count: i64) -> i32 {
    unread_count.clamp(0, i32::MAX as i64) as i32
}

async fn reconcile_topic_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_topic_id: i64,
    is_closed: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let command_kind = if is_closed {
        "topic_close"
    } else {
        "topic_reopen"
    };
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = $3
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND payload->>'provider_topic_id' = $4
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(command_kind)
    .bind(provider_topic_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command =
            crate::integrations::telegram::client::rows::row_to_telegram_provider_write_command(
                row,
            )?;
        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_topic_id": provider_topic_id,
            "is_closed": is_closed,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_topic_id": provider_topic_id,
            "is_closed": is_closed,
            "provider_observed_at": observed_at,
        });
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state,
                result_payload,
            )
            .await?,
        );
    }

    Ok(reconciled)
}

fn topic_updated_event(
    account_id: &str,
    topic: &TelegramTopic,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_topic_{}_{}_{}",
            account_id,
            topic.topic_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::TOPIC_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_topic",
            "id": topic.topic_id,
            "telegram_chat_id": topic.telegram_chat_id,
            "provider_chat_id": topic.provider_chat_id,
            "provider_topic_id": topic.provider_topic_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": topic.telegram_chat_id,
        "provider_chat_id": topic.provider_chat_id,
        "provider_topic_id": topic.provider_topic_id,
        "topic_id": topic.topic_id,
        "topic": topic,
        "source": "tdlib.updateForumTopicInfo"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateForumTopicInfo"
    }))
    .build()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use sqlx::Row;
    use testkit::context::TestContext;

    use super::*;
    use crate::integrations::telegram::client::lifecycle::insert_command;
    use crate::integrations::telegram::client::models::{
        NewTelegramChat, TelegramChatKind, TelegramSyncState,
    };

    #[test]
    fn topic_updated_event_contains_sanitized_projection_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let topic = TelegramTopic {
            topic_id: "telegram_topic:v1:test".to_owned(),
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_topic_id: 42,
            provider_chat_id: "-100123".to_owned(),
            title: "Release notes".to_owned(),
            icon_emoji: Some("123456".to_owned()),
            is_pinned: true,
            is_closed: false,
            unread_count: 0,
            last_message_at: None,
            metadata: json!({}),
            created_at: occurred_at,
            updated_at: occurred_at,
        };

        let event = topic_updated_event("acct-1", &topic, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::TOPIC_UPDATED);
        assert_eq!(event.subject["id"], "telegram_topic:v1:test");
        assert_eq!(event.payload["topic"]["title"], "Release notes");
        assert_eq!(event.payload["topic"]["is_pinned"], true);
    }

    #[tokio::test]
    async fn publish_topic_event_reconciles_topic_close_and_appends_runtime_events() {
        let ctx = TestContext::new().await;
        let pool = ctx.pool().clone();
        let account_id = "acct-1";
        let provider_chat_id = "chat-1";
        let event_bus = EventBus::new();

        crate::test_support::upsert_telegram_runtime_account(
            &pool,
            account_id,
            "Topic Runtime Account",
            "telegram-ext-acct-1",
        )
        .await;

        let chat = crate::test_support::telegram_store(&pool)
            .upsert_chat(&NewTelegramChat {
                account_id: account_id.to_owned(),
                provider_chat_id: provider_chat_id.to_owned(),
                chat_kind: TelegramChatKind::Group,
                title: "Forum Chat".to_owned(),
                username: None,
                sync_state: TelegramSyncState::Synced,
                last_message_at: None,
                metadata: json!({}),
            })
            .await
            .expect("seed chat");

        let command_id = "cmd-topic-close-1";
        insert_command(
            &pool,
            command_id,
            account_id,
            "topic_close",
            "topic_close:42:seed",
            provider_chat_id,
            None,
            "available",
            "provider_write",
            "confirmed",
            "makosh-frontend",
            json!({
                "provider_topic_id": 42,
                "is_closed": true
            }),
            json!({
                "telegram_chat_id": chat.telegram_chat_id,
                "provider_chat_id": provider_chat_id,
                "provider_topic_id": 42
            }),
            json!({"source": "test"}),
        )
        .await
        .expect("seed topic close command");

        let snapshot = TelegramTdlibTopicUpdateSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            topic: TelegramTdlibTopicSnapshot {
                provider_topic_id: 42,
                title: "Release Notes".to_owned(),
                icon_emoji: None,
                is_pinned: false,
                is_closed: true,
                unread_count: 7,
                last_message_at: None,
            },
        };

        publish_topic_event(
            &Some(crate::test_support::telegram_store(&pool)),
            &event_bus,
            account_id,
            &snapshot,
        )
        .await;

        let rows: Vec<(String, serde_json::Value, serde_json::Value)> = sqlx::query_as(
            r#"
            SELECT event_type, subject, payload
            FROM event_log
            WHERE event_type IN (
                'telegram.command.status_changed',
                'telegram.command.reconciled',
                'telegram.topic.updated'
            )
            ORDER BY position ASC
            "#,
        )
        .fetch_all(&pool)
        .await
        .expect("topic runtime events");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
        assert_eq!(rows[0].1["id"], json!(command_id));
        assert_eq!(rows[0].2["status"], json
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/manager/topics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/manager/topics.rs`
- Size bytes / Размер в байтах: `3749`
- Included characters / Включено символов: `3749`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::models::topics::NewTelegramTopic;
use crate::integrations::telegram::client::{TelegramError, TelegramStore};

use super::super::commands::request_actor_get_forum_topics;
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

pub(super) fn telegram_topic_id(telegram_chat_id: &str, provider_topic_id: i64) -> String {
    use sha2::{Digest, Sha256};
    let input = format!("{telegram_chat_id}\0{provider_topic_id}");
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("telegram_topic:v1:{:x}", hasher.finalize())
}

impl TelegramRuntimeManager {
    /// Fetches forum topics from TDLib for the given chat and upserts them into the projection.
    ///
    /// Returns the number of topics upserted. If the account has no active TDLib actor or runs
    /// in fixture mode, returns Ok(0) without error so the API can still serve DB rows.
    pub(crate) async fn sync_forum_topics<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        telegram_chat_id: &str,
    ) -> Result<usize, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        let chat = context
            .telegram_store
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or(TelegramError::InvalidRequest(format!(
                "chat {telegram_chat_id} not found"
            )))?;

        let account = load_active_account(context.provider_account_store, &chat.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);

        if runtime_kind != "tdlib_qr_authorized" {
            return Ok(0);
        }

        let command_tx = match self
            .ensure_tdlib_actor(
                context.provider_secret_binding_store,
                context.secret_store,
                context.secret_resolver,
                context.config,
                &account,
                context.event_bridge.clone(),
            )
            .await
        {
            Ok(tx) => tx,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    telegram_chat_id,
                    "sync_forum_topics: failed to get TDLib actor, serving DB projection"
                );
                return Ok(0);
            }
        };

        let snapshots =
            request_actor_get_forum_topics(command_tx, chat.provider_chat_id.clone(), 100).await?;

        let mut upserted = 0;
        for snapshot in &snapshots {
            let new_topic = NewTelegramTopic {
                topic_id: telegram_topic_id(&chat.telegram_chat_id, snapshot.provider_topic_id),
                telegram_chat_id: chat.telegram_chat_id.clone(),
                account_id: chat.account_id.clone(),
                provider_topic_id: snapshot.provider_topic_id,
                provider_chat_id: chat.provider_chat_id.clone(),
                title: snapshot.title.clone(),
                icon_emoji: snapshot.icon_emoji.clone(),
                is_pinned: snapshot.is_pinned,
                is_closed: snapshot.is_closed,
                unread_count: topic_unread_count(snapshot.unread_count),
                last_message_at: snapshot.last_message_at,
            };
            crate::integrations::telegram::client::topics::upsert_topic(
                context.telegram_store.pool(),
                &new_topic,
            )
            .await?;
            upserted += 1;
        }

        Ok(upserted)
    }
}

fn topic_unread_count(unread_count: i64) -> i32 {
    unread_count.clamp(0, i32::MAX as i64) as i32
}
```

### `backend/src/integrations/telegram/runtime/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/models.rs`
- Size bytes / Размер в байтах: `9189`
- Included characters / Включено символов: `9189`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::integrations::telegram::client::{TelegramChat, TelegramError, TelegramMessage};

use super::validation::{validate_limit, validate_non_empty};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramRuntimeStartRequest {
    pub account_id: String,
}

impl TelegramRuntimeStartRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramRuntimeStopRequest {
    pub account_id: String,
}

impl TelegramRuntimeStopRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramRuntimeRestartRequest {
    pub account_id: String,
}

impl TelegramRuntimeRestartRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramChatSyncRequest {
    pub account_id: String,
    pub limit: Option<i64>,
}

impl TelegramChatSyncRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        if let Some(limit) = self.limit {
            validate_limit(limit)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatSyncResponse {
    pub account_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub synced_count: usize,
    pub items: Vec<TelegramChat>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramHistorySyncRequest {
    pub account_id: String,
    pub provider_chat_id: String,
    pub from_message_id: Option<i64>,
    pub mode: Option<TelegramHistorySyncMode>,
    pub limit: Option<i64>,
}

impl TelegramHistorySyncRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        if let Some(from_message_id) = self.from_message_id
            && from_message_id <= 0
        {
            return Err(TelegramError::InvalidRequest(
                "from_message_id must be a positive TDLib message id".to_owned(),
            ));
        }
        if self.mode() == TelegramHistorySyncMode::Older && self.from_message_id.is_none() {
            return Err(TelegramError::InvalidRequest(
                "from_message_id is required when mode=older".to_owned(),
            ));
        }
        if let Some(limit) = self.limit {
            validate_limit(limit)?;
        }
        Ok(())
    }

    pub(super) fn mode(&self) -> TelegramHistorySyncMode {
        self.mode.unwrap_or(TelegramHistorySyncMode::Latest)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramHistorySyncMode {
    Latest,
    Older,
    Full,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramHistorySyncResponse {
    pub account_id: String,
    pub provider_chat_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub synced_count: usize,
    pub has_more: bool,
    pub next_from_message_id: Option<i64>,
    pub items: Vec<TelegramMessage>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramMediaDownloadRequest {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub tdlib_file_id: i64,
    pub provider_attachment_id: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub priority: Option<i32>,
}

impl TelegramMediaDownloadRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        if self.tdlib_file_id <= 0 {
            return Err(TelegramError::InvalidRequest(
                "tdlib_file_id must be a positive TDLib file id".to_owned(),
            ));
        }
        if let Some(priority) = self.priority
            && !(1..=32).contains(&priority)
        {
            return Err(TelegramError::InvalidRequest(
                "priority must be between 1 and 32".to_owned(),
            ));
        }
        if let Some(value) = &self.provider_attachment_id {
            validate_non_empty("provider_attachment_id", value)?;
        }
        if let Some(value) = &self.filename {
            validate_non_empty("filename", value)?;
        }
        if let Some(value) = &self.content_type {
            validate_non_empty("content_type", value)?;
        }
        Ok(())
    }

    pub(crate) fn provider_attachment_id(&self) -> String {
        self.provider_attachment_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("tdlib-file:{}", self.tdlib_file_id))
    }

    pub(crate) fn content_type(&self) -> String {
        self.content_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "application/octet-stream".to_owned())
    }

    pub(crate) fn filename(&self) -> Option<String> {
        self.filename
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMediaDownloadResponse {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub tdlib_file_id: i64,
    pub local_path: Option<String>,
    pub size_bytes: Option<i64>,
    pub expected_size_bytes: Option<i64>,
    pub downloaded_size_bytes: Option<i64>,
    pub is_downloading_active: bool,
    pub is_downloading_completed: bool,
    pub attachment_id: Option<String>,
    pub blob_id: Option<String>,
    pub scan_status: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramMediaSendRequest {
    pub command_id: String,
    pub provider_chat_id: String,
    pub media_type: TelegramMediaSendType,
    pub local_path: String,
    pub caption: Option<String>,
    pub filename: Option<String>,
}

impl TelegramMediaSendRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("local_path", &self.local_path)?;
        if let Some(caption) = &self.caption {
            validate_non_empty("caption", caption)?;
        }
        if let Some(filename) = &self.filename {
            validate_non_empty("filename", filename)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramMediaSendType {
    Photo,
    Video,
    Document,
    Audio,
    Voice,
    Sticker,
    Animation,
}

impl TelegramMediaSendType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Photo => "photo",
            Self::Video => "video",
            Self::Document => "document",
            Self::Audio => "audio",
            Self::Voice => "voice",
            Self::Sticker => "sticker",
            Self::Animation => "animation",
        }
    }
}

impl TryFrom<&str> for TelegramMediaSendType {
    type Error = TelegramError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "photo" => Ok(Self::Photo),
            "video" => Ok(Self::Video),
            "document" => Ok(Self::Document),
            "audio" => Ok(Self::Audio),
            "voice" | "voice_note" => Ok(Self::Voice),
            "sticker" => Ok(Self::Sticker),
            "animation" | "gif" => Ok(Self::Animation),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram media upload type `{other}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub status: String,
    pub fixture_runtime: bool,
    pub tdjson_path: Option<String>,
    pub tdjson_runtime_available: bool,
    pub tdjson_probe_error: Option<String>,
    pub telegram_api_id_configured: bool,
    pub telegram_api_hash_configured: bool,
    pub telegram_app_credentials_configured: bool,
    pub live_send_available: bool,
    pub runtime_blockers: Vec<String>,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}
```

### `backend/src/integrations/telegram/runtime/participant_commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/participant_commands.rs`
- Size bytes / Размер в байтах: `3266`
- Included characters / Включено символов: `3266`
- Truncated / Обрезано: `no`

```rust
use std::sync::mpsc::{self, Sender};

use tokio::task;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot;

use super::TDJSON_COMMAND_TIMEOUT;
use super::state::TelegramRuntimeCommand;

pub(super) async fn request_actor_get_supergroup_members(
    command_tx: Sender<TelegramRuntimeCommand>,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetSupergroupMembers {
                supergroup_id,
                limit,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting member roster requests".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib member roster timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_get_supergroup_administrators(
    command_tx: Sender<TelegramRuntimeCommand>,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetSupergroupAdministrators {
                supergroup_id,
                limit,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting supergroup administrator sync commands"
                        .to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime(
                "Telegram TDLib supergroup administrator sync timed out".to_owned(),
            )
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_get_basic_group_members(
    command_tx: Sender<TelegramRuntimeCommand>,
    basic_group_id: i64,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetBasicGroupMembers {
                basic_group_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting basic-group roster requests".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib basic-group roster timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}
```
