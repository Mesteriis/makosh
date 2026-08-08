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

- Chunk ID / ID чанка: `062-source-backend-part-042`
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

### `backend/src/integrations/telegram/client/references.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/references.rs`
- Size bytes / Размер в байтах: `12169`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::{HashMap, HashSet, VecDeque};

use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use super::errors::TelegramError;
use super::models::messages::{
    TelegramForwardChainResponse, TelegramForwardRef, TelegramMessageReferenceSummary,
    TelegramReplyChainResponse, TelegramReplyRef,
};
use super::rows::{row_to_telegram_forward_ref, row_to_telegram_reply_ref};
use super::store::TelegramStore;

const MAX_REFERENCE_CHAIN_DEPTH: usize = 16;
const MAX_REFERENCE_CHAIN_EDGES: usize = 128;
const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

fn new_reply_ref_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgreply_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("reply_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

fn new_forward_ref_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgfwd_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("fwd_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_reply_ref(
    pool: &PgPool,
    source_message_id: &str,
    target_message_id: &str,
    account_id: &str,
    provider_chat_id: &str,
    source_provider_id: &str,
    target_provider_id: &str,
    is_topic_reply: bool,
) -> Result<TelegramReplyRef, TelegramError> {
    let reply_ref_id = new_reply_ref_id();
    let row = sqlx::query(
        r#"
        WITH inserted AS (
        INSERT INTO telegram_message_reply_refs
            (reply_ref_id, source_message_id, target_message_id, account_id,
             provider_chat_id, source_provider_id, target_provider_id,
             reply_depth, is_topic_reply)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8)
        ON CONFLICT (source_message_id, target_message_id) DO NOTHING
        RETURNING *
        )
        SELECT * FROM inserted
        UNION ALL
        SELECT * FROM telegram_message_reply_refs
        WHERE source_message_id = $2
          AND target_message_id = $3
          AND NOT EXISTS (SELECT 1 FROM inserted)
        LIMIT 1
        "#,
    )
    .bind(&reply_ref_id)
    .bind(source_message_id)
    .bind(target_message_id)
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(source_provider_id)
    .bind(target_provider_id)
    .bind(is_topic_reply)
    .fetch_one(pool)
    .await?;

    row_to_telegram_reply_ref(row)
}

pub async fn reply_chain(
    store: &TelegramStore,
    message_id: &str,
) -> Result<TelegramReplyChainResponse, TelegramError> {
    let pool = store.pool();
    let mut replies = collect_reply_descendants(pool, message_id).await?;
    let mut reply_to = collect_reply_ancestors(pool, message_id).await?;

    let mut summary_ids = Vec::new();
    for item in replies.iter().chain(reply_to.iter()) {
        summary_ids.push(item.source_message_id.as_str());
        summary_ids.push(item.target_message_id.as_str());
    }
    let summaries = reference_message_summaries(store, summary_ids).await?;
    for item in &mut replies {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
        item.target_message_summary = summaries.get(&item.target_message_id).cloned();
    }
    for item in &mut reply_to {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
        item.target_message_summary = summaries.get(&item.target_message_id).cloned();
    }

    Ok(TelegramReplyChainResponse {
        message_id: message_id.to_owned(),
        replies,
        reply_to,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_forward_ref(
    pool: &PgPool,
    source_message_id: &str,
    account_id: &str,
    provider_chat_id: &str,
    source_provider_id: &str,
    origin_chat_id: Option<&str>,
    origin_message_id: Option<&str>,
    origin_sender_id: Option<&str>,
    origin_sender_name: Option<&str>,
    forward_date: Option<chrono::DateTime<Utc>>,
) -> Result<TelegramForwardRef, TelegramError> {
    let forward_ref_id = new_forward_ref_id();
    let row = sqlx::query(
        r#"
        WITH inserted AS (
        INSERT INTO telegram_message_forward_refs
            (forward_ref_id, source_message_id, account_id, provider_chat_id,
             source_provider_id, forward_origin_chat_id, forward_origin_message_id,
             forward_origin_sender_id, forward_origin_sender_name, forward_date, forward_depth)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 1)
        ON CONFLICT (source_message_id, account_id) DO NOTHING
        RETURNING *
        )
        SELECT * FROM inserted
        UNION ALL
        SELECT * FROM telegram_message_forward_refs
        WHERE source_message_id = $2
          AND account_id = $3
          AND NOT EXISTS (SELECT 1 FROM inserted)
        LIMIT 1
        "#,
    )
    .bind(&forward_ref_id)
    .bind(source_message_id)
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(source_provider_id)
    .bind(origin_chat_id)
    .bind(origin_message_id)
    .bind(origin_sender_id)
    .bind(origin_sender_name)
    .bind(forward_date)
    .fetch_one(pool)
    .await?;

    row_to_telegram_forward_ref(row)
}

pub async fn forward_chain(
    store: &TelegramStore,
    message_id: &str,
) -> Result<TelegramForwardChainResponse, TelegramError> {
    let mut forwards = collect_forward_ancestors(store, message_id).await?;

    let summaries = reference_message_summaries(
        store,
        forwards
            .iter()
            .map(|item| item.source_message_id.as_str())
            .collect(),
    )
    .await?;
    for item in &mut forwards {
        item.source_message_summary = summaries.get(&item.source_message_id).cloned();
    }

    Ok(TelegramForwardChainResponse {
        message_id: message_id.to_owned(),
        forwards,
    })
}

async fn collect_reply_descendants(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    let mut refs = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(message_id.to_owned());
    queue.push_back((message_id.to_owned(), 0usize));

    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= MAX_REFERENCE_CHAIN_DEPTH {
            continue;
        }
        for mut item in reply_refs_by_target(pool, &current_id).await? {
            let next_id = item.source_message_id.clone();
            if !visited.insert(next_id.clone()) {
                continue;
            }
            item.reply_depth = (depth + 1) as i32;
            refs.push(item);
            if refs.len() >= MAX_REFERENCE_CHAIN_EDGES {
                return Ok(refs);
            }
            queue.push_back((next_id, depth + 1));
        }
    }

    Ok(refs)
}

async fn collect_reply_ancestors(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    let mut refs = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(message_id.to_owned());
    queue.push_back((message_id.to_owned(), 0usize));

    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= MAX_REFERENCE_CHAIN_DEPTH {
            continue;
        }
        for mut item in reply_refs_by_source(pool, &current_id).await? {
            let next_id = item.target_message_id.clone();
            if !visited.insert(next_id.clone()) {
                continue;
            }
            item.reply_depth = (depth + 1) as i32;
            refs.push(item);
            if refs.len() >= MAX_REFERENCE_CHAIN_EDGES {
                return Ok(refs);
            }
            queue.push_back((next_id, depth + 1));
        }
    }

    Ok(refs)
}

async fn reply_refs_by_target(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    sqlx::query(
        "SELECT * FROM telegram_message_reply_refs WHERE target_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_reply_ref)
    .collect::<Result<_, _>>()
}

async fn reply_refs_by_source(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    sqlx::query(
        "SELECT * FROM telegram_message_reply_refs WHERE source_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_reply_ref)
    .collect::<Result<_, _>>()
}

async fn collect_forward_ancestors(
    store: &TelegramStore,
    message_id: &str,
) -> Result<Vec<TelegramForwardRef>, TelegramError> {
    let pool = store.pool();
    let mut refs = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(message_id.to_owned());
    queue.push_back((message_id.to_owned(), 0usize));

    while let Some((current_id, depth)) = queue.pop_front() {
        if depth >= MAX_REFERENCE_CHAIN_DEPTH {
            continue;
        }
        for mut item in forward_refs_by_source(pool, &current_id).await? {
            let origin_message_id = local_forward_origin_message_id(store, &item).await?;
            if let Some(next_id) = origin_message_id {
                if !visited.insert(next_id.clone()) {
                    continue;
                }
                item.forward_depth = (depth + 1) as i32;
                refs.push(item);
                if refs.len() >= MAX_REFERENCE_CHAIN_EDGES {
                    return Ok(refs);
                }
                queue.push_back((next_id, depth + 1));
            } else {
                item.forward_depth = (depth + 1) as i32;
                refs.push(item);
                if refs.len() >= MAX_REFERENCE_CHAIN_EDGES {
                    return Ok(refs);
                }
            }
        }
    }

    Ok(refs)
}

async fn forward_refs_by_source(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramForwardRef>, TelegramError> {
    sqlx::query(
        "SELECT * FROM telegram_message_forward_refs WHERE source_message_id = $1 ORDER BY created_at DESC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(row_to_telegram_forward_ref)
    .collect::<Result<_, _>>()
}

async fn local_forward_origin_message_id(
    store: &TelegramStore,
    item: &TelegramForwardRef,
) -> Result<Option<String>, TelegramError> {
    let Some(origin_provider_message_id) = item.forward_origin_message_id.as_deref() else {
        return Ok(None);
    };
    Ok(store
        .provider_channel_message_store()
        .message_id_by_provider_record_id(
            &item.account_id,
            origin_provider_message_id,
            TELEGRAM_CHANNEL_KINDS,
        )
        .await?)
}

async fn reference_message_summaries(
    store: &TelegramStore,
    message_ids: Vec<&str>,
) -> Result<HashMap<String, TelegramMessageReferenceSummary>, TelegramError> {
    if message_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let message_ids: Vec<String> = message_ids.into_iter().map(ToOwned::to_owned).collect();
    let summaries = store
        .provider_channel_message_store()
        .reference_summaries(&message_ids)
        .await?;

    Ok(summaries
        .into_iter()
        .map(|summary| {
            (
                summary.message_id.clone(),
                TelegramMessageReferenceSummary {
                    message_id: summary.message_id,
                    provider_message_id: summary.provider_record_id,
                    provider_chat_id: summary.conversation_id,
                    chat_title: summary.subject,
                    sender: summary.sender,
                    sender_display_name: summary.sender_display_na
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/rows.rs`
- Size bytes / Размер в байтах: `9242`
- Included characters / Включено символов: `9242`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use crate::platform::communications::ProviderChannelMessage;

use super::errors::TelegramError;
use super::models::messages::{
    TelegramMessageTombstone, TelegramMessageVersion, TelegramProviderWriteCommand,
};
use super::models::{TelegramChat, TelegramMessage};

pub(super) fn row_to_telegram_chat(row: PgRow) -> Result<TelegramChat, TelegramError> {
    Ok(TelegramChat {
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        chat_kind: row.try_get("chat_kind")?,
        title: row.try_get("title")?,
        username: row.try_get("username")?,
        sync_state: row.try_get("sync_state")?,
        last_message_at: row.try_get("last_message_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_telegram_message(row: PgRow) -> Result<TelegramMessage, TelegramError> {
    Ok(TelegramMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_record_id")?,
        provider_chat_id: row.try_get("conversation_id")?,
        chat_title: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        sender_display_name: row.try_get("sender_display_name")?,
        text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        delivery_state: row.try_get("delivery_state")?,
        metadata: row.try_get("message_metadata")?,
    })
}

pub(in crate::integrations::telegram) fn provider_channel_message_to_telegram_message(
    message: ProviderChannelMessage,
) -> TelegramMessage {
    TelegramMessage {
        message_id: message.message_id,
        raw_record_id: message.raw_record_id,
        account_id: message.account_id,
        provider_message_id: message.provider_record_id,
        provider_chat_id: Some(message.conversation_id),
        chat_title: message.subject,
        sender: message.sender,
        sender_display_name: message.sender_display_name,
        text: message.body_text,
        occurred_at: message.occurred_at,
        projected_at: message.projected_at,
        channel_kind: message.channel_kind,
        delivery_state: message.delivery_state,
        metadata: message.message_metadata,
    }
}

pub(super) fn row_to_telegram_message_version(
    row: PgRow,
) -> Result<TelegramMessageVersion, TelegramError> {
    Ok(TelegramMessageVersion {
        version_id: row.try_get("version_id")?,
        message_id: row.try_get("message_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        version_number: row.try_get("version_number")?,
        body_text: row.try_get("body_text")?,
        edit_timestamp: row.try_get("edit_timestamp")?,
        source_event: row.try_get("source_event")?,
        raw_diff_payload: row.try_get("raw_diff_payload")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(super) fn row_to_telegram_message_tombstone(
    row: PgRow,
) -> Result<TelegramMessageTombstone, TelegramError> {
    Ok(TelegramMessageTombstone {
        tombstone_id: row.try_get("tombstone_id")?,
        message_id: row.try_get("message_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        reason_class: row.try_get("reason_class")?,
        actor_class: row.try_get("actor_class")?,
        observed_at: row.try_get("observed_at")?,
        source_event: row.try_get("source_event")?,
        is_provider_delete: row.try_get("is_provider_delete")?,
        is_local_visible: row.try_get("is_local_visible")?,
        metadata: row.try_get("metadata")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(crate) fn row_to_telegram_provider_write_command(
    row: PgRow,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    Ok(TelegramProviderWriteCommand {
        command_id: row.try_get("command_id")?,
        account_id: row.try_get("account_id")?,
        command_kind: row.try_get("command_kind")?,
        idempotency_key: row.try_get("idempotency_key")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        target_ref: row.try_get("target_ref")?,
        payload: row.try_get("payload")?,
        capability_state: row.try_get("capability_state")?,
        action_class: row.try_get("action_class")?,
        confirmation_decision: row.try_get("confirmation_decision")?,
        status: row.try_get("status")?,
        retry_count: row.try_get("retry_count")?,
        max_retries: row.try_get("max_retries")?,
        last_error: row.try_get("last_error")?,
        result_payload: row.try_get("result_payload")?,
        audit_metadata: row.try_get("audit_metadata")?,
        actor_id: row.try_get("actor_id")?,
        happened_at: row.try_get("happened_at")?,
        next_attempt_at: row.try_get("next_attempt_at")?,
        last_attempt_at: row.try_get("last_attempt_at")?,
        locked_at: row.try_get("locked_at")?,
        locked_by: row.try_get("locked_by")?,
        provider_observed_at: row.try_get("provider_observed_at")?,
        provider_state: row.try_get("provider_state")?,
        reconciliation_status: row.try_get("reconciliation_status")?,
        reconciled_at: row.try_get("reconciled_at")?,
        dead_lettered_at: row.try_get("dead_lettered_at")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

// --- Reaction rows ---

use super::models::messages::TelegramReaction;

pub(super) fn row_to_telegram_reaction(row: PgRow) -> Result<TelegramReaction, TelegramError> {
    Ok(TelegramReaction {
        reaction_id: row.try_get("reaction_id")?,
        message_id: row.try_get("message_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        sender_id: row.try_get("sender_id")?,
        sender_display_name: row.try_get("sender_display_name")?,
        reaction_emoji: row.try_get("reaction_emoji")?,
        is_active: row.try_get("is_active")?,
        observed_at: row.try_get("observed_at")?,
        source_event: row.try_get("source_event")?,
        provider_actor_id: row.try_get("provider_actor_id")?,
        metadata: row.try_get("metadata")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

// --- Reply/Forward rows ---

use super::models::messages::{TelegramForwardRef, TelegramReplyRef};

pub(super) fn row_to_telegram_reply_ref(row: PgRow) -> Result<TelegramReplyRef, TelegramError> {
    Ok(TelegramReplyRef {
        reply_ref_id: row.try_get("reply_ref_id")?,
        source_message_id: row.try_get("source_message_id")?,
        target_message_id: row.try_get("target_message_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        source_provider_id: row.try_get("source_provider_id")?,
        target_provider_id: row.try_get("target_provider_id")?,
        reply_depth: row.try_get("reply_depth")?,
        is_topic_reply: row.try_get("is_topic_reply")?,
        topic_id: row.try_get("topic_id")?,
        source_message_summary: None,
        target_message_summary: None,
        metadata: row.try_get("metadata")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(super) fn row_to_telegram_forward_ref(row: PgRow) -> Result<TelegramForwardRef, TelegramError> {
    Ok(TelegramForwardRef {
        forward_ref_id: row.try_get("forward_ref_id")?,
        source_message_id: row.try_get("source_message_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        source_provider_id: row.try_get("source_provider_id")?,
        forward_origin_chat_id: row.try_get("forward_origin_chat_id")?,
        forward_origin_message_id: row.try_get("forward_origin_message_id")?,
        forward_origin_sender_id: row.try_get("forward_origin_sender_id")?,
        forward_origin_sender_name: row.try_get("forward_origin_sender_name")?,
        forward_date: row.try_get("forward_date")?,
        forward_depth: row.try_get("forward_depth")?,
        source_message_summary: None,
        metadata: row.try_get("metadata")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
    })
}
```

### `backend/src/integrations/telegram/client/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/search.rs`
- Size bytes / Размер в байтах: `2846`
- Included characters / Включено символов: `2846`
- Truncated / Обрезано: `no`

```rust
use sqlx::PgPool;

use super::errors::TelegramError;
use super::models::{TelegramChat, TelegramMessage};
use super::rows::{provider_channel_message_to_telegram_message, row_to_telegram_chat};
use super::store::TelegramStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

impl TelegramStore {
    pub async fn pinned_messages(
        &self,
        telegram_chat_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = super::validation::validate_message_list_limit(limit)?;
        let chat = self
            .telegram_chat_by_id(telegram_chat_id.trim())
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?;

        Ok(self
            .provider_channel_message_store()
            .pinned_messages(
                &chat.account_id,
                &chat.provider_chat_id,
                TELEGRAM_CHANNEL_KINDS,
                limit,
            )
            .await?
            .into_iter()
            .map(provider_channel_message_to_telegram_message)
            .collect())
    }

    pub async fn search_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = super::validation::validate_message_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|v| !v.is_empty());
        let provider_chat_id = provider_chat_id.map(str::trim).filter(|v| !v.is_empty());

        Ok(self
            .provider_channel_message_store()
            .search_messages(
                account_id,
                provider_chat_id,
                query,
                TELEGRAM_CHANNEL_KINDS,
                limit,
            )
            .await?
            .into_iter()
            .map(provider_channel_message_to_telegram_message)
            .collect())
    }

    pub async fn search_chats(
        &self,
        account_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let like_pattern = format!("%{query}%");
        let account_id = account_id.map(str::trim).filter(|v| !v.is_empty());

        let rows = sqlx::query(
            r#"
            SELECT * FROM telegram_chats
            WHERE title ILIKE $1
              AND ($2::text IS NULL OR account_id = $2)
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
        )
        .bind(&like_pattern)
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_chat).collect()
    }
}
```

### `backend/src/integrations/telegram/client/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/store.rs`
- Size bytes / Размер в байтах: `2380`
- Included characters / Включено символов: `2380`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use sqlx::postgres::PgPool;

use crate::platform::communications::{
    CommunicationRawRecordCommandPort, ProviderAccountCommandPort,
    ProviderChannelMessageLookupPort, ProviderMessageObservationEventPort,
    ProviderSecretBindingCommandPort,
};

#[derive(Clone)]
pub struct TelegramStore {
    pub(super) pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    communication_raw_record_store: Arc<dyn CommunicationRawRecordCommandPort>,
    provider_observation_events: Arc<dyn ProviderMessageObservationEventPort>,
}

impl TelegramStore {
    pub fn new(
        pool: PgPool,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
        communication_raw_record_store: Arc<dyn CommunicationRawRecordCommandPort>,
        provider_observation_events: Arc<dyn ProviderMessageObservationEventPort>,
    ) -> Self {
        Self {
            pool,
            provider_account_store,
            provider_secret_binding_store,
            provider_channel_message_store,
            communication_raw_record_store,
            provider_observation_events,
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(super) fn provider_account_store(&self) -> &dyn ProviderAccountCommandPort {
        self.provider_account_store.as_ref()
    }

    pub(super) fn provider_secret_binding_store(&self) -> &dyn ProviderSecretBindingCommandPort {
        self.provider_secret_binding_store.as_ref()
    }

    pub(super) fn provider_channel_message_store(&self) -> &dyn ProviderChannelMessageLookupPort {
        self.provider_channel_message_store.as_ref()
    }

    pub(in crate::integrations::telegram) fn communication_raw_record_store(
        &self,
    ) -> &dyn CommunicationRawRecordCommandPort {
        self.communication_raw_record_store.as_ref()
    }

    pub(in crate::integrations::telegram) fn provider_observation_events(
        &self,
    ) -> &dyn ProviderMessageObservationEventPort {
        self.provider_observation_events.as_ref()
    }
}
```

### `backend/src/integrations/telegram/client/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/tests.rs`
- Size bytes / Размер в байтах: `1866`
- Included characters / Включено символов: `1866`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;

use super::validation::{validate_chat_list_limit, validate_message_list_limit};
use super::*;

fn valid_message(text: &str) -> NewTelegramMessage {
    NewTelegramMessage {
        account_id: "telegram-account".to_owned(),
        provider_chat_id: "12345".to_owned(),
        provider_message_id: "12345:67890".to_owned(),
        chat_kind: TelegramChatKind::Private,
        chat_title: "Private Chat".to_owned(),
        sender_id: "user:12345".to_owned(),
        sender_display_name: "Telegram User".to_owned(),
        text: text.to_owned(),
        import_batch_id: "telegram-tdlib-history:telegram-account:12345".to_owned(),
        occurred_at: Utc::now(),
        delivery_state: TelegramDeliveryState::Received,
    }
}

#[test]
fn fixture_message_validation_rejects_empty_text() {
    let message = valid_message("   ");

    let error = message
        .validate_for_runtime("fixture")
        .expect_err("fixture text validation should reject empty body");

    assert!(matches!(error, TelegramError::InvalidRequest(_)));
    assert!(error.to_string().contains("text must not be empty"));
}

#[test]
fn tdlib_message_validation_allows_empty_text_for_media_snapshots() {
    let message = valid_message("");

    message
        .validate_for_runtime("tdlib")
        .expect("TDLib media snapshots may not have text");
}

#[test]
fn message_list_limit_allows_full_selected_chat_window() {
    assert_eq!(validate_message_list_limit(5000).expect("limit"), 5000);
    assert!(matches!(
        validate_message_list_limit(5001),
        Err(TelegramError::InvalidRequest(_))
    ));
}

#[test]
fn chat_list_limit_allows_full_metadata_window() {
    assert_eq!(validate_chat_list_limit(5000).expect("limit"), 5000);
    assert!(matches!(
        validate_chat_list_limit(5001),
        Err(TelegramError::InvalidRequest(_))
    ));
}
```

### `backend/src/integrations/telegram/client/topics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/topics.rs`
- Size bytes / Размер в байтах: `6814`
- Included characters / Включено символов: `6814`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};

use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::models::topics::{NewTelegramTopic, TelegramTopic};
use super::store::TelegramStore;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

fn row_to_telegram_topic(row: sqlx::postgres::PgRow) -> Result<TelegramTopic, TelegramError> {
    use sqlx::Row;
    Ok(TelegramTopic {
        topic_id: row.try_get("topic_id")?,
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        account_id: row.try_get("account_id")?,
        provider_topic_id: row.try_get("provider_topic_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        title: row.try_get("title")?,
        icon_emoji: row.try_get("icon_emoji")?,
        is_pinned: row.try_get("is_pinned")?,
        is_closed: row.try_get("is_closed")?,
        unread_count: row.try_get("unread_count")?,
        last_message_at: row.try_get("last_message_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

async fn capture_topic_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    topic: &TelegramTopic,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_TOPIC",
            ObservationOriginKind::LocalRuntime,
            topic.updated_at,
            json!({
                "topic_id": topic.topic_id,
                "telegram_chat_id": topic.telegram_chat_id,
                "account_id": topic.account_id,
                "provider_topic_id": topic.provider_topic_id,
                "provider_chat_id": topic.provider_chat_id,
                "title": topic.title,
                "icon_emoji": topic.icon_emoji,
                "is_pinned": topic.is_pinned,
                "is_closed": topic.is_closed,
                "unread_count": topic.unread_count,
                "last_message_at": topic.last_message_at,
                "metadata": topic.metadata,
                "operation": relationship_kind,
            }),
            format!("telegram-topic://{}/{}", topic.topic_id, relationship_kind),
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
        "topic",
        topic.topic_id.clone(),
        relationship_kind,
        json!({
            "telegram_chat_id": topic.telegram_chat_id,
            "account_id": topic.account_id,
            "provider_topic_id": topic.provider_topic_id,
            "provider_chat_id": topic.provider_chat_id,
            "is_closed": topic.is_closed,
            "is_pinned": topic.is_pinned,
        }),
    )
    .await?;
    Ok(())
}

pub async fn upsert_topic(
    pool: &PgPool,
    topic: &NewTelegramTopic,
) -> Result<TelegramTopic, TelegramError> {
    let now = Utc::now();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r"
        INSERT INTO telegram_topics (
            topic_id, telegram_chat_id, account_id, provider_topic_id, provider_chat_id,
            title, icon_emoji, is_pinned, is_closed, unread_count, last_message_at,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)
        ON CONFLICT (telegram_chat_id, provider_topic_id)
        DO UPDATE SET
            title        = EXCLUDED.title,
            icon_emoji   = EXCLUDED.icon_emoji,
            is_pinned    = EXCLUDED.is_pinned,
            is_closed    = EXCLUDED.is_closed,
            unread_count = EXCLUDED.unread_count,
            last_message_at = EXCLUDED.last_message_at,
            updated_at   = EXCLUDED.updated_at
        RETURNING *
        ",
    )
    .bind(&topic.topic_id)
    .bind(&topic.telegram_chat_id)
    .bind(&topic.account_id)
    .bind(topic.provider_topic_id)
    .bind(&topic.provider_chat_id)
    .bind(&topic.title)
    .bind(&topic.icon_emoji)
    .bind(topic.is_pinned)
    .bind(topic.is_closed)
    .bind(topic.unread_count)
    .bind(topic.last_message_at)
    .bind(now)
    .fetch_one(&mut *transaction)
    .await
    .map_err(TelegramError::from)?;

    let stored = row_to_telegram_topic(row)?;
    capture_topic_observation_in_transaction(
        &mut transaction,
        &stored,
        "upsert",
        "telegram.client.topics.upsert_topic",
    )
    .await?;
    transaction.commit().await?;
    Ok(stored)
}

pub async fn list_topics(
    pool: &PgPool,
    telegram_chat_id: &str,
    limit: i64,
) -> Result<Vec<TelegramTopic>, TelegramError> {
    let rows = sqlx::query(
        r"
        SELECT * FROM telegram_topics
        WHERE telegram_chat_id = $1
        ORDER BY is_pinned DESC, last_message_at DESC NULLS LAST, updated_at DESC
        LIMIT $2
        ",
    )
    .bind(telegram_chat_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    rows.into_iter().map(row_to_telegram_topic).collect()
}

pub async fn get_topic(
    pool: &PgPool,
    topic_id: &str,
) -> Result<Option<TelegramTopic>, TelegramError> {
    let row = sqlx::query("SELECT * FROM telegram_topics WHERE topic_id = $1")
        .bind(topic_id)
        .fetch_optional(pool)
        .await
        .map_err(TelegramError::from)?;

    row.map(row_to_telegram_topic).transpose()
}

pub async fn search_topics(
    pool: &PgPool,
    telegram_chat_id: &str,
    query: &str,
    limit: i64,
) -> Result<Vec<TelegramTopic>, TelegramError> {
    let pattern = format!("%{}%", query.trim().to_lowercase());
    let rows = sqlx::query(
        r"
        SELECT * FROM telegram_topics
        WHERE telegram_chat_id = $1
          AND lower(title) LIKE $2
        ORDER BY is_pinned DESC, last_message_at DESC NULLS LAST, updated_at DESC
        LIMIT $3
        ",
    )
    .bind(telegram_chat_id)
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    rows.into_iter().map(row_to_telegram_topic).collect()
}

pub async fn list_topic_message_ids(
    store: &TelegramStore,
    topic_id: &str,
    limit: i64,
) -> Result<Vec<String>, TelegramError> {
    Ok(store
        .provider_channel_message_store()
        .message_ids_by_metadata_string("forum_topic_id", topic_id, TELEGRAM_CHANNEL_KINDS, limit)
        .await?)
}
```

### `backend/src/integrations/telegram/client/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/validation.rs`
- Size bytes / Размер в байтах: `1514`
- Included characters / Включено символов: `1514`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::TelegramError;

pub(super) fn validate_message_list_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=5000).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "message list limit must be between 1 and 5000".to_owned(),
        ));
    }
    Ok(limit)
}

pub(super) fn validate_chat_list_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=5000).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "chat list limit must be between 1 and 5000".to_owned(),
        ));
    }
    Ok(limit)
}

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

pub(super) fn required_optional_value(
    field: &'static str,
    value: Option<&str>,
) -> Result<String, TelegramError> {
    let Some(value) = value else {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    };

    validate_non_empty(field, value)
}

pub(super) fn validate_object(field: &'static str, value: &Value) -> Result<(), TelegramError> {
    if !value.is_object() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}
```

### `backend/src/integrations/telegram/client/vault.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/vault.rs`
- Size bytes / Размер в байтах: `2197`
- Included characters / Включено символов: `2197`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::platform::communications::{CommunicationProviderKind, ProviderAccountSecretPurpose};
use crate::platform::secrets::{DatabaseEncryptedSecretVault, SecretKind, SecretStoreKind};
use crate::vault::{HostVault, SecretEntryContext};

use super::errors::TelegramError;

pub(super) struct TelegramCredentialWrite<'a> {
    pub(super) account_id: &'a str,
    pub(super) provider_kind: CommunicationProviderKind,
    pub(super) secret_purpose: ProviderAccountSecretPurpose,
    pub(super) secret_kind: SecretKind,
    pub(super) label: &'a str,
    pub(super) value: String,
}

pub enum TelegramSecretVault {
    Database(DatabaseEncryptedSecretVault),
    Host(HostVault),
}

impl TelegramSecretVault {
    pub fn database(vault: DatabaseEncryptedSecretVault) -> Self {
        Self::Database(vault)
    }

    pub fn host(vault: HostVault) -> Self {
        Self::Host(vault)
    }

    pub(super) fn store_kind(&self) -> SecretStoreKind {
        match self {
            Self::Database(_) => SecretStoreKind::DatabaseEncryptedVault,
            Self::Host(_) => SecretStoreKind::HostVault,
        }
    }

    pub(super) async fn store_secret(
        &self,
        secret_ref: &str,
        credential: &TelegramCredentialWrite<'_>,
    ) -> Result<(), TelegramError> {
        match self {
            Self::Database(vault) => vault.store_secret(secret_ref, &credential.value).await?,
            Self::Host(vault) => vault.store_secret(
                secret_ref,
                &credential.value,
                SecretEntryContext {
                    entry_kind: "provider_credential",
                    account_id: credential.account_id,
                    purpose: credential.secret_purpose.as_str(),
                    secret_kind: credential.secret_kind.as_str(),
                    label: credential.label,
                    metadata: &json!({
                        "provider": credential.provider_kind.as_str(),
                        "account_id": credential.account_id,
                        "secret_purpose": credential.secret_purpose.as_str()
                    }),
                },
            )?,
        }
        Ok(())
    }
}
```

### `backend/src/integrations/telegram/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/mod.rs`
- Size bytes / Размер в байтах: `49`
- Included characters / Включено символов: `49`
- Truncated / Обрезано: `no`

```rust
pub mod client;
pub mod runtime;
pub mod tdjson;
```

### `backend/src/integrations/telegram/runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime.rs`
- Size bytes / Размер в байтах: `1174`
- Included characters / Включено символов: `1174`
- Truncated / Обрезано: `no`

```rust
mod actor;
mod commands;
mod manager;
mod models;
mod participant_commands;
mod state;
mod status;
#[cfg(test)]
mod tests;
mod validation;

const TDJSON_BOOTSTRAP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const TDJSON_COMMAND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const TDJSON_RECEIVE_POLL_SECONDS: f64 = 1.0;

pub(crate) use self::manager::TelegramProviderSearchRequest;
pub use self::manager::TelegramRuntimeManager;
pub(crate) use self::manager::command_executor::execute_queued_commands;
pub(crate) use self::manager::{
    TelegramMediaDownloadContext, TelegramMemberSyncContext, TelegramRuntimeEventBridgeContext,
    TelegramRuntimeOperationContext, TelegramRuntimeOperationDeps, TelegramRuntimeStartContext,
};
pub use self::models::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncMode,
    TelegramHistorySyncRequest, TelegramHistorySyncResponse, TelegramMediaDownloadRequest,
    TelegramMediaDownloadResponse, TelegramMediaSendRequest, TelegramMediaSendType,
    TelegramRuntimeRestartRequest, TelegramRuntimeStartRequest, TelegramRuntimeStatus,
    TelegramRuntimeStopRequest,
};
```

### `backend/src/integrations/telegram/runtime/actor.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor.rs`
- Size bytes / Размер в байтах: `366`
- Included characters / Включено символов: `366`
- Truncated / Обрезано: `no`

```rust
mod authorization;
mod chats;
mod download;
mod driver;
mod edit;
mod history;
mod participants;
mod responses;
mod search;
mod send;
mod session;
mod spawn;
mod start_request;
mod support;
mod topics;

pub(super) use self::session::optional_telegram_session_key;
pub(super) use self::spawn::spawn_tdlib_actor;
pub(super) use self::support::oldest_tdlib_message_id;
```

### `backend/src/integrations/telegram/runtime/actor/authorization.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/authorization.rs`
- Size bytes / Размер в байтах: `3786`
- Included characters / Включено символов: `3786`
- Truncated / Обрезано: `no`

```rust
use std::time::Instant;

use serde_json::{Value, json};

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};
use crate::integrations::telegram::tdjson::{self, TdJsonClient};

use super::super::{TDJSON_BOOTSTRAP_TIMEOUT, TDJSON_RECEIVE_POLL_SECONDS};

pub(super) fn prepare_tdlib_client(
    client: &TdJsonClient,
    start_request: &TelegramQrLoginStartRequest,
) -> Result<(), TelegramError> {
    let database_directory = tdjson::tdlib_database_directory(start_request);
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
        "@extra": "makosh-runtime-initial-authorization-state"
    }))?;
    Ok(())
}

pub(super) fn wait_for_tdlib_ready(
    client: &TdJsonClient,
    start_request: &TelegramQrLoginStartRequest,
) -> Result<(), TelegramError> {
    let database_directory = tdjson::tdlib_database_directory(start_request);
    let started_at = Instant::now();
    let mut tdlib_parameters_sent = false;

    while started_at.elapsed() < TDJSON_BOOTSTRAP_TIMEOUT {
        let Some(event) = client.receive_json(TDJSON_RECEIVE_POLL_SECONDS)? else {
            continue;
        };

        if tdjson::is_tdlib_parameters_not_specified_error(&event) && !tdlib_parameters_sent {
            client.send_json(&tdjson::set_tdlib_parameters_request(
                start_request,
                &database_directory,
            )?)?;
            tdlib_parameters_sent = true;
            continue;
        }
        if tdjson::is_tdlib_database_encryption_key_needed_error(&event) {
            client.send_json(&tdjson::check_database_encryption_key_request(
                start_request,
            ))?;
            continue;
        }
        if let Some(message) = tdjson::tdlib_error_message(&event) {
            return Err(TelegramError::TdlibRuntime(message));
        }

        let Some(authorization_state) = tdjson::authorization_state(&event) else {
            continue;
        };
        match authorization_state.get("@type").and_then(Value::as_str) {
            Some("authorizationStateWaitTdlibParameters") => {
                client.send_json(&tdjson::set_tdlib_parameters_request(
                    start_request,
                    &database_directory,
                )?)?;
                tdlib_parameters_sent = true;
            }
            Some("authorizationStateWaitEncryptionKey") => {
                client.send_json(&tdjson::check_database_encryption_key_request(
                    start_request,
                ))?;
            }
            Some("authorizationStateReady") => return Ok(()),
            Some("authorizationStateClosed")
            | Some("authorizationStateClosing")
            | Some("authorizationStateLoggingOut") => {
                return Err(TelegramError::TdlibRuntime(
                    "Telegram TDLib authorization session is closed".to_owned(),
                ));
            }
            Some(wait_state) if wait_state.starts_with("authorizationStateWait") => {
                return Err(TelegramError::TdlibRuntime(format!(
                    "Telegram TDLib account is not authorized; current state is `{wait_state}`"
                )));
            }
            _ => {}
        }
    }

    Err(TelegramError::TdlibRuntime(
        "Telegram TDLib authorization did not become ready in time".to_owned(),
    ))
}
```

### `backend/src/integrations/telegram/runtime/actor/chats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/chats.rs`
- Size bytes / Размер в байтах: `2939`
- Included characters / Включено символов: `2939`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{
    self, TdJsonClient, TelegramTdlibChatFolderSnapshot, TelegramTdlibChatSnapshot,
};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::receive_tdlib_extra;

pub(super) fn actor_load_chats(
    client: &TdJsonClient,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatSnapshot>, TelegramError> {
    let load_extra = "makosh-runtime-load-chats";
    client.send_json(&tdjson::tdlib_load_chats_request(limit, load_extra))?;
    let load_response = receive_tdlib_extra(client, load_extra, TDJSON_COMMAND_TIMEOUT)?;
    if tdjson::tdlib_error_message(&load_response).is_some() && !is_tdlib_not_found(&load_response)
    {
        return Err(TelegramError::TdlibRuntime(
            tdjson::tdlib_error_message(&load_response)
                .unwrap_or_else(|| "TDLib loadChats failed".to_owned()),
        ));
    }

    let chats_extra = "makosh-runtime-get-chats";
    client.send_json(&tdjson::tdlib_get_chats_request(limit, chats_extra))?;
    let chats_response = receive_tdlib_extra(client, chats_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&chats_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    let chat_ids = tdjson::parse_tdlib_chat_ids(&chats_response)?;
    let mut snapshots = Vec::with_capacity(chat_ids.len());
    for chat_id in chat_ids {
        let extra = format!("makosh-runtime-get-chat-{chat_id}");
        client.send_json(&tdjson::tdlib_get_chat_request(chat_id, &extra))?;
        let chat_response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&chat_response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        snapshots.push(tdjson::parse_tdlib_chat_snapshot(&chat_response)?);
    }
    Ok(snapshots)
}

pub(super) fn actor_get_chat_folders(
    client: &TdJsonClient,
    folder_ids: &[i64],
) -> Result<Vec<TelegramTdlibChatFolderSnapshot>, TelegramError> {
    let mut snapshots = Vec::with_capacity(folder_ids.len());
    for folder_id in folder_ids {
        let extra = format!("makosh-runtime-get-chat-folder-{folder_id}");
        client.send_json(&tdjson::tdlib_get_chat_folder_request(*folder_id, &extra))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_chat_folder_snapshot(&response)? {
            snapshots.push(snapshot);
        }
    }
    Ok(snapshots)
}

fn is_tdlib_not_found(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(404)
}
```

### `backend/src/integrations/telegram/runtime/actor/download.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/download.rs`
- Size bytes / Размер в байтах: `826`
- Included characters / Включено символов: `826`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibFileSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::receive_tdlib_extra;

pub(super) fn actor_download_file(
    client: &TdJsonClient,
    file_id: i64,
    priority: i32,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    let extra = format!("makosh-runtime-download-file-{file_id}");
    client.send_json(&tdjson::tdlib_download_file_request(
        file_id, priority, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_file_snapshot(&response)
}
```

### `backend/src/integrations/telegram/runtime/actor/driver.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/driver.rs`
- Size bytes / Размер в байтах: `15460`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};
use crate::integrations::telegram::tdjson::{self, TdJsonClient};
use crate::platform::config::AppConfig;

use super::super::state::{TelegramRuntimeCommand, TelegramRuntimeEvent};
use super::authorization::{prepare_tdlib_client, wait_for_tdlib_ready};
use super::chats::{actor_get_chat_folders, actor_load_chats};
use super::download::actor_download_file;
use super::edit::{
    actor_add_chat_to_folder, actor_delete_message, actor_edit_message, actor_join_chat,
    actor_leave_chat, actor_pin_message, actor_remove_chat_from_folder, actor_set_reaction,
    actor_toggle_chat_archive, actor_toggle_chat_mute, actor_toggle_chat_unread,
};
use super::history::actor_sync_history;
use super::participants::{
    actor_get_basic_group_members, actor_get_supergroup_administrators,
    actor_get_supergroup_members,
};
use super::search::{actor_search_chat_messages, actor_search_messages};
use super::send::{actor_send_forward, actor_send_media, actor_send_reply, actor_send_text};
use super::topics::{
    actor_create_forum_topic, actor_get_forum_topics, actor_toggle_forum_topic_closed,
};

pub(super) fn drive_tdlib_actor(
    config: AppConfig,
    start_request: TelegramQrLoginStartRequest,
    command_rx: mpsc::Receiver<TelegramRuntimeCommand>,
    runtime_event_tx: Option<UnboundedSender<TelegramRuntimeEvent>>,
) -> Result<(), TelegramError> {
    let library = tdjson::TdJsonLibrary::load(config.tdjson_path())?;
    let client = library.create_client()?;
    prepare_tdlib_client(&client, &start_request)?;
    wait_for_tdlib_ready(&client, &start_request)?;

    loop {
        let command = match command_rx.recv_timeout(Duration::from_millis(250)) {
            Ok(command) => command,
            Err(RecvTimeoutError::Timeout) => {
                drain_unsolicited_tdlib_events(&client, runtime_event_tx.as_ref())?;
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };

        match command {
            TelegramRuntimeCommand::LoadChats { limit, reply_tx } => {
                let _ = reply_tx.send(actor_load_chats(&client, limit));
            }
            TelegramRuntimeCommand::GetChatFolders {
                folder_ids,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_chat_folders(&client, &folder_ids));
            }
            TelegramRuntimeCommand::SyncHistory {
                provider_chat_id,
                from_message_id,
                limit,
                mode,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_sync_history(
                    &client,
                    &provider_chat_id,
                    from_message_id,
                    limit,
                    mode,
                ));
            }
            TelegramRuntimeCommand::SendText { request, reply_tx } => {
                let _ = reply_tx.send(actor_send_text(&client, &request));
            }
            TelegramRuntimeCommand::SendMedia { request, reply_tx } => {
                let _ = reply_tx.send(actor_send_media(&client, &request));
            }
            TelegramRuntimeCommand::DownloadFile {
                file_id,
                priority,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_download_file(&client, file_id, priority));
            }
            TelegramRuntimeCommand::EditMessage {
                provider_chat_id,
                provider_message_id,
                new_text,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_edit_message(
                    &client,
                    &provider_chat_id,
                    &provider_message_id,
                    &new_text,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::DeleteMessage {
                provider_chat_id,
                provider_message_id,
                revoke,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_delete_message(
                    &client,
                    &provider_chat_id,
                    &provider_message_id,
                    revoke,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::SetReaction {
                provider_chat_id,
                provider_message_id,
                reaction_emoji,
                is_active,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_set_reaction(
                    &client,
                    &provider_chat_id,
                    &provider_message_id,
                    &reaction_emoji,
                    is_active,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::PinMessage {
                provider_chat_id,
                provider_message_id,
                pin,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_pin_message(
                    &client,
                    &provider_chat_id,
                    &provider_message_id,
                    pin,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ToggleChatUnread {
                provider_chat_id,
                is_marked_as_unread,
                read_through_provider_message_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_toggle_chat_unread(
                    &client,
                    &provider_chat_id,
                    is_marked_as_unread,
                    read_through_provider_message_id.as_deref(),
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ToggleChatArchive {
                provider_chat_id,
                archived,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_toggle_chat_archive(
                    &client,
                    &provider_chat_id,
                    archived,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ToggleChatMute {
                provider_chat_id,
                muted,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_toggle_chat_mute(
                    &client,
                    &provider_chat_id,
                    muted,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::AddChatToFolder {
                provider_chat_id,
                provider_folder_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_add_chat_to_folder(
                    &client,
                    &provider_chat_id,
                    provider_folder_id,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::RemoveChatFromFolder {
                provider_chat_id,
                provider_folder_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_remove_chat_from_folder(
                    &client,
                    &provider_chat_id,
                    provider_folder_id,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::JoinChat {
                provider_chat_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_join_chat(&client, &provider_chat_id, &command_id));
            }
            TelegramRuntimeCommand::LeaveChat {
                provider_chat_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_leave_chat(&client, &provider_chat_id, &command_id));
            }
            TelegramRuntimeCommand::ReplyMessage {
                provider_chat_id,
                reply_to_provider_message_id,
                text,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_send_reply(
                    &client,
                    &provider_chat_id,
                    &reply_to_provider_message_id,
                    &text,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ForwardMessage {
                provider_chat_id,
                from_provider_chat_id,
                from_provider_message_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_send_forward(
                    &client,
                    &provider_chat_id,
                    &from_provider_chat_id,
                    &from_provider_message_id,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::GetForumTopics {
                provider_chat_id,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_forum_topics(&client, &provider_chat_id, limit));
            }
            TelegramRuntimeCommand::CreateForumTopic {
                provider_chat_id,
                title,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_create_forum_topic(
                    &client,
                    &provider_chat_id,
                    &title,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ToggleForumTopicClosed {
                provider_chat_id,
                provider_topic_id,
                is_closed,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_toggle_forum_topic_closed(
                    &client,
                    &provider_chat_id,
                    provider_topic_id,
                    is_closed,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::GetSupergroupMembers {
                supergroup_id,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_supergroup_members(&client, supergroup_id, limit));
            }
            TelegramRuntimeCommand::GetSupergroupAdministrators {
                supergroup_id,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_supergroup_administrators(
                    &client,
                    supergroup_id,
                    limit,
                ));
            }
            TelegramRuntimeCommand::GetBasicGroupMembers {
                basic_group_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_basic_group_members(&client, basic_group_id));
            }
            TelegramRuntimeCommand::SearchMessages {
                query,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_search_messages(&client, &query, limit));
            }
            TelegramRuntimeCommand::SearchChatMessages {
                provider_chat_id,
                query,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_search_chat_messages(
                    &c
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/runtime/actor/edit.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/edit.rs`
- Size bytes / Размер в байтах: `9590`
- Included characters / Включено символов: `9590`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id, tdlib_provider_message_id};

pub(super) fn actor_edit_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    new_text: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("makosh-edit-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_edit_message_text_request(
        chat_id, message_id, new_text, &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_delete_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    revoke: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("makosh-delete-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_delete_messages_request(
        chat_id,
        &[message_id],
        revoke,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_set_reaction(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    reaction_emoji: &str,
    is_active: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("makosh-reaction-{}", command_id.trim());
    let request = if is_active {
        tdjson::tdlib_add_message_reaction_request(chat_id, message_id, reaction_emoji, &extra)
    } else {
        tdjson::tdlib_remove_message_reaction_request(chat_id, message_id, reaction_emoji, &extra)
    };
    client.send_json(&request)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_pin_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    pin: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("makosh-pin-{}", command_id.trim());
    let request = if pin {
        tdjson::tdlib_pin_chat_message_request(chat_id, message_id, false, &extra)
    } else {
        tdjson::tdlib_unpin_chat_message_request(chat_id, message_id, &extra)
    };
    client.send_json(&request)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_toggle_chat_unread(
    client: &TdJsonClient,
    provider_chat_id: &str,
    is_marked_as_unread: bool,
    read_through_provider_message_id: Option<&str>,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    if is_marked_as_unread {
        let extra = format!("makosh-chat-unread-{}", command_id.trim());
        client.send_json(&tdjson::tdlib_toggle_chat_marked_as_unread_request(
            chat_id, true, &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        return Ok(());
    }

    if let Some(provider_message_id) = read_through_provider_message_id {
        let message_id = tdlib_provider_message_id(provider_message_id)?;
        let extra = format!("makosh-chat-read-{}", command_id.trim());
        client.send_json(&tdjson::tdlib_view_messages_request(
            chat_id,
            &[message_id],
            true,
            &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        return Ok(());
    }

    let extra = format!("makosh-chat-read-toggle-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_toggle_chat_marked_as_unread_request(
        chat_id, false, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_toggle_chat_archive(
    client: &TdJsonClient,
    provider_chat_id: &str,
    archived: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-archive-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_add_chat_to_list_request(
        chat_id, archived, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_toggle_chat_mute(
    client: &TdJsonClient,
    provider_chat_id: &str,
    muted: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-mute-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_set_chat_mute_request(chat_id, muted, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_add_chat_to_folder(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_folder_id: i64,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-folder-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_add_chat_to_folder_request(
        chat_id,
        provider_folder_id,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_remove_chat_from_folder(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_folder_id: i64,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let get_extra = format!("makosh-chat-folder-remove-get-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_get_chat_folder_request(
        provider_folder_id,
        &get_extra,
    ))?;
    let folder_response = receive_tdlib_extra(client, &get_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&folder_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }

    let edit_extra = format!("makosh-chat-folder-remove-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_edit_chat_folder_remove_chat_request(
        provider_folder_id,
        chat_id,
        &folder_response,
        &edit_extra,
    )?)?;
    let response = receive_tdlib_extra(client, &edit_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_join_chat(
    client: &TdJsonClient,
    provider_chat_id: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-join-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_join_chat_request(chat_id, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_leave_chat(
    client: &TdJsonClient,
    provider_chat_id: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-leave-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_leave_chat_request(chat_id, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}
```

### `backend/src/integrations/telegram/runtime/actor/history.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/history.rs`
- Size bytes / Размер в байтах: `1851`
- Included characters / Включено символов: `1851`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibMessageSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::super::models::TelegramHistorySyncMode;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};
use super::support::oldest_tdlib_message_id;

pub(super) fn actor_sync_history(
    client: &TdJsonClient,
    provider_chat_id: &str,
    from_message_id: Option<i64>,
    limit: i32,
    mode: TelegramHistorySyncMode,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let page_limit = limit.clamp(1, 100);
    let mut cursor = from_message_id;
    let mut snapshots = Vec::new();
    let mut page_index = 0;

    loop {
        let extra = format!(
            "makosh-runtime-history-{chat_id}-{}-{page_index}",
            cursor.unwrap_or(0)
        );
        client.send_json(&tdjson::tdlib_get_chat_history_request(
            chat_id, cursor, page_limit, false, &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        let page = tdjson::parse_tdlib_message_list(&response)?;
        if page.is_empty() {
            break;
        }

        let page_len = page.len();
        let next_cursor = oldest_tdlib_message_id(&page);
        snapshots.extend(page);
        if mode != TelegramHistorySyncMode::Full || page_len < page_limit as usize {
            break;
        }
        if next_cursor.is_none() || next_cursor == cursor {
            break;
        }
        cursor = next_cursor;
        page_index += 1;
    }

    Ok(snapshots)
}
```

### `backend/src/integrations/telegram/runtime/actor/participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/participants.rs`
- Size bytes / Размер в байтах: `3665`
- Included characters / Включено символов: `3665`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibChatMemberSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::receive_tdlib_extra;

const TDLIB_SUPERGROUP_MEMBER_PAGE_LIMIT: i32 = 100;

pub(super) fn actor_get_supergroup_members(
    client: &TdJsonClient,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    actor_get_supergroup_members_with_filter(
        client,
        supergroup_id,
        limit,
        "recent",
        tdjson::tdlib_get_supergroup_members_request,
    )
}

pub(super) fn actor_get_supergroup_administrators(
    client: &TdJsonClient,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    actor_get_supergroup_members_with_filter(
        client,
        supergroup_id,
        limit,
        "administrators",
        tdjson::tdlib_get_supergroup_administrators_request,
    )
}

fn actor_get_supergroup_members_with_filter(
    client: &TdJsonClient,
    supergroup_id: i64,
    limit: i32,
    filter_name: &str,
    request_builder: fn(i64, i32, i32, &str) -> serde_json::Value,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    let target_limit = limit.clamp(1, 1000);
    let mut offset = 0_i32;
    let mut items = Vec::new();
    let mut seen_member_ids = std::collections::HashSet::new();

    while items.len() < target_limit as usize {
        let remaining = target_limit - items.len() as i32;
        let page_limit = remaining.clamp(1, TDLIB_SUPERGROUP_MEMBER_PAGE_LIMIT);
        let extra = format!("makosh-supergroup-members-{filter_name}-{supergroup_id}-{offset}");
        client.send_json(&request_builder(supergroup_id, offset, page_limit, &extra))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        let page_items = tdjson::parse_tdlib_chat_member_list(&response)?;
        if page_items.is_empty() {
            break;
        }
        let page_len = page_items.len();
        for item in page_items {
            if seen_member_ids.insert(item.provider_member_id.clone()) {
                items.push(item);
            }
        }
        if page_len < page_limit as usize {
            break;
        }
        offset += page_len as i32;
    }

    Ok(items)
}

pub(super) fn actor_get_basic_group_members(
    client: &TdJsonClient,
    basic_group_id: i64,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    let group_extra = format!("makosh-basic-group-{basic_group_id}");
    client.send_json(&tdjson::tdlib_get_basic_group_request(
        basic_group_id,
        &group_extra,
    ))?;
    let group_response = receive_tdlib_extra(client, &group_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&group_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }

    let full_info_extra = format!("makosh-basic-group-full-info-{basic_group_id}");
    client.send_json(&tdjson::tdlib_get_basic_group_full_info_request(
        basic_group_id,
        &full_info_extra,
    ))?;
    let full_info_response = receive_tdlib_extra(client, &full_info_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&full_info_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }

    tdjson::parse_tdlib_basic_group_member_list(&full_info_response)
}
```

### `backend/src/integrations/telegram/runtime/actor/responses.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/responses.rs`
- Size bytes / Размер в байтах: `1809`
- Included characters / Включено символов: `1809`
- Truncated / Обрезано: `no`

```rust
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient};

use super::super::TDJSON_RECEIVE_POLL_SECONDS;

pub(super) fn receive_tdlib_extra(
    client: &TdJsonClient,
    expected_extra: &str,
    timeout: Duration,
) -> Result<Value, TelegramError> {
    let started_at = Instant::now();
    while started_at.elapsed() < timeout {
        let Some(event) = client.receive_json(TDJSON_RECEIVE_POLL_SECONDS)? else {
            continue;
        };
        if event.get("@extra").and_then(Value::as_str) == Some(expected_extra) {
            return Ok(event);
        }
        if let Some(message) = tdjson::tdlib_error_message(&event) {
            tracing::debug!(error = %message, "ignored unrelated TDLib error while waiting for correlated response");
        }
    }
    Err(TelegramError::TdlibRuntime(format!(
        "TDLib request `{expected_extra}` timed out"
    )))
}

pub(super) fn tdlib_provider_chat_id(provider_chat_id: &str) -> Result<i64, TelegramError> {
    provider_chat_id.trim().parse::<i64>().map_err(|_| {
        TelegramError::InvalidRequest(format!(
            "TDLib provider_chat_id `{}` must be a Telegram numeric chat id",
            provider_chat_id.trim()
        ))
    })
}

pub(super) fn tdlib_provider_message_id(provider_message_id: &str) -> Result<i64, TelegramError> {
    provider_message_id
        .trim()
        .rsplit(':')
        .next()
        .unwrap_or_default()
        .parse::<i64>()
        .map_err(|_| {
            TelegramError::InvalidRequest(format!(
                "TDLib provider_message_id `{}` must end with a Telegram numeric message id",
                provider_message_id.trim()
            ))
        })
}
```

### `backend/src/integrations/telegram/runtime/actor/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/search.rs`
- Size bytes / Размер в байтах: `1839`
- Included characters / Включено символов: `1839`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibMessageSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};

pub(super) fn actor_search_messages(
    client: &TdJsonClient,
    query: &str,
    limit: i32,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let extra = format!("makosh-search-{}", uuid_extra(query));
    client.send_json(&tdjson::tdlib_search_messages_request(query, limit, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_list(&response)
}

pub(super) fn actor_search_chat_messages(
    client: &TdJsonClient,
    provider_chat_id: &str,
    query: &str,
    limit: i32,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-search-chat-{chat_id}-{}", uuid_extra(query));
    client.send_json(&tdjson::tdlib_search_chat_messages_request(
        chat_id, query, limit, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_list(&response)
}

fn uuid_extra(query: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    let hash = hasher.finalize();
    format!(
        "{:016x}",
        u64::from_be_bytes(hash[..8].try_into().unwrap_or([0u8; 8]))
    )
}
```

### `backend/src/integrations/telegram/runtime/actor/send.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/send.rs`
- Size bytes / Размер в байтах: `3695`
- Included characters / Включено символов: `3695`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::{TelegramError, TelegramManualSendRequest};
use crate::integrations::telegram::runtime::TelegramMediaSendRequest;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibMessageSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id, tdlib_provider_message_id};

pub(super) fn actor_send_text(
    client: &TdJsonClient,
    request: &TelegramManualSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(&request.provider_chat_id)?;
    let extra = format!("makosh-runtime-send-{}", request.command_id.trim());
    client.send_json(&tdjson::tdlib_send_text_message_request(
        chat_id,
        &request.text,
        &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_snapshot(&response)
}

pub(super) fn actor_send_media(
    client: &TdJsonClient,
    request: &TelegramMediaSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    request.validate()?;
    let chat_id = tdlib_provider_chat_id(&request.provider_chat_id)?;
    let extra = format!("makosh-media-send-{}", request.command_id.trim());
    client.send_json(&tdjson::tdlib_send_media_message_request(
        chat_id,
        request.media_type,
        &request.local_path,
        request.caption.as_deref(),
        request.filename.as_deref(),
        &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_snapshot(&response)
}

pub(super) fn actor_send_reply(
    client: &TdJsonClient,
    provider_chat_id: &str,
    reply_to_provider_message_id: &str,
    text: &str,
    command_id: &str,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let reply_to_message_id = tdlib_provider_message_id(reply_to_provider_message_id)?;
    let extra = format!("makosh-reply-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_send_reply_request(
        chat_id,
        reply_to_message_id,
        text,
        &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_snapshot(&response)
}

pub(super) fn actor_send_forward(
    client: &TdJsonClient,
    provider_chat_id: &str,
    from_provider_chat_id: &str,
    from_provider_message_id: &str,
    command_id: &str,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let from_chat_id = tdlib_provider_chat_id(from_provider_chat_id)?;
    let message_id = tdlib_provider_message_id(from_provider_message_id)?;
    let extra = format!("makosh-forward-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_send_forward_request(
        chat_id,
        from_chat_id,
        message_id,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_snapshot(&response)
}
```

### `backend/src/integrations/telegram/runtime/actor/session.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/session.rs`
- Size bytes / Размер в байтах: `2034`
- Included characters / Включено символов: `2034`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::TelegramError;
use crate::platform::communications::{
    ProviderAccountSecretPurpose, ProviderSecretBindingLookupPort,
};
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

pub(in crate::integrations::telegram::runtime) async fn optional_telegram_session_key(
    binding_store: &dyn ProviderSecretBindingLookupPort,
    secret_store: &SecretReferenceStore,
    secret_resolver: &(impl SecretResolver + Sync + ?Sized),
    account_id: &str,
) -> Result<Option<String>, TelegramError> {
    let binding = binding_store
        .get_for_account(account_id, ProviderAccountSecretPurpose::TelegramSessionKey)
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!(
                "failed to resolve Telegram session encryption key: {error}"
            ))
        })?;
    let Some(binding) = binding else {
        return Ok(None);
    };
    let reference = secret_store
        .secret_reference(&binding.secret_ref)
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!(
                "failed to resolve Telegram session encryption key: {error}"
            ))
        })?
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(format!(
                "failed to resolve Telegram session encryption key: secret reference metadata not found: {}",
                binding.secret_ref
            ))
        })?;
    if !binding
        .secret_purpose
        .accepts_secret_kind(reference.secret_kind)
    {
        return Err(TelegramError::TdlibRuntime(format!(
            "failed to resolve Telegram session encryption key: incompatible secret kind for {}",
            reference.secret_ref
        )));
    }
    let secret = secret_resolver.resolve(&reference).await.map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to resolve Telegram session encryption key: {error}"
        ))
    })?;

    Ok(Some(secret.expose_for_runtime().to_owned()))
}
```

### `backend/src/integrations/telegram/runtime/actor/spawn.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/spawn.rs`
- Size bytes / Размер в байтах: `1769`
- Included characters / Включено символов: `1769`
- Truncated / Обрезано: `no`

```rust
use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson;
use crate::platform::communications::ProviderAccount;
use crate::platform::config::AppConfig;
use tokio::sync::mpsc::UnboundedSender;

use super::super::state::{TelegramRuntimeCommand, TelegramRuntimeEvent};
use super::driver::drive_tdlib_actor;
use super::start_request::tdlib_start_request_from_account;
use super::support::short_thread_suffix;

pub(in crate::integrations::telegram::runtime) fn spawn_tdlib_actor(
    config: AppConfig,
    account: ProviderAccount,
    session_encryption_key: Option<String>,
    runtime_event_tx: Option<UnboundedSender<TelegramRuntimeEvent>>,
) -> Result<Sender<TelegramRuntimeCommand>, TelegramError> {
    if !tdjson::runtime_available(config.tdjson_path()) {
        return Err(TelegramError::TdlibRuntimeUnavailable(
            "libtdjson is not available for Telegram live runtime".to_owned(),
        ));
    }
    let start_request =
        tdlib_start_request_from_account(&config, &account, session_encryption_key)?;
    let (command_tx, command_rx) = mpsc::channel();
    let thread_name = format!(
        "telegram-tdlib-{}",
        short_thread_suffix(&account.account_id)
    );
    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            if let Err(error) =
                drive_tdlib_actor(config, start_request, command_rx, runtime_event_tx)
            {
                tracing::warn!(error = %error, "Telegram TDLib actor stopped");
            }
        })
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("failed to spawn Telegram TDLib actor: {error}"))
        })?;

    Ok(command_tx)
}
```

### `backend/src/integrations/telegram/runtime/actor/start_request.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/start_request.rs`
- Size bytes / Размер в байтах: `1712`
- Included characters / Включено символов: `1712`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};
use crate::platform::communications::ProviderAccount;
use crate::platform::config::AppConfig;

pub(super) fn tdlib_start_request_from_account(
    config: &AppConfig,
    account: &ProviderAccount,
    session_encryption_key: Option<String>,
) -> Result<TelegramQrLoginStartRequest, TelegramError> {
    let api_id = config.telegram_api_id().ok_or_else(|| {
        TelegramError::InvalidRequest(
            "MAKOSH_TELEGRAM_API_ID is required for Telegram TDLib runtime".to_owned(),
        )
    })?;
    let api_hash = config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
        .ok_or_else(|| {
            TelegramError::InvalidRequest(
                "MAKOSH_TELEGRAM_API_HASH is required for Telegram TDLib runtime".to_owned(),
            )
        })?;
    let tdlib_data_path = account
        .config
        .get("tdlib_data_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(
                "tdlib_data_path is required for Telegram TDLib runtime".to_owned(),
            )
        })?;

    Ok(TelegramQrLoginStartRequest {
        account_id: account.account_id.clone(),
        display_name: account.display_name.clone(),
        external_account_id: account.external_account_id.clone(),
        api_id: Some(api_id),
        api_hash: Some(api_hash),
        session_encryption_key,
        tdlib_data_path: Some(tdlib_data_path),
        transcription_enabled: false,
    })
}
```

### `backend/src/integrations/telegram/runtime/actor/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/runtime/actor/support.rs`
- Size bytes / Размер в байтах: `857`
- Included characters / Включено символов: `857`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::tdjson::TelegramTdlibMessageSnapshot;

pub(in crate::integrations::telegram::runtime) fn oldest_tdlib_message_id(
    snapshots: &[TelegramTdlibMessageSnapshot],
) -> Option<i64> {
    snapshots
        .iter()
        .filter_map(|snapshot| snapshot.provider_message_id.trim().parse::<i64>().ok())
        .min()
}

pub(super) fn short_thread_suffix(account_id: &str) -> String {
    let sanitized = account_id
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
        sanitized.chars().take(32).collect()
    }
}
```
