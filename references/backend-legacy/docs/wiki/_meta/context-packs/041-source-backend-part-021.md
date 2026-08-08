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

- Chunk ID / ID чанка: `041-source-backend-part-021`
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

### `backend/src/domains/communications/messages/store/upsert.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/store/upsert.rs`
- Size bytes / Размер в байтах: `8683`
- Included characters / Включено символов: `8683`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::MessageProjectionStore;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::ids::message_id;
use crate::domains::communications::messages::models::{NewProjectedMessage, ProjectedMessage};
use crate::domains::communications::messages::rows::row_to_projected_message;

impl MessageProjectionStore {
    pub async fn upsert_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        message.validate()?;
        let canonical_message_id = message_id(&message.account_id, &message.provider_record_id);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_messages (
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            )
            SELECT
                $1,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                $5,
                $6,
                $7,
                $8,
                $9,
                'email',
                NULL,
                $6,
                'received',
                '{}'::jsonb
            FROM communication_raw_records
            WHERE raw_record_id = $2
              AND account_id = $3
              AND provider_record_id = $4
              AND record_kind = 'email_message'
            ON CONFLICT (account_id, provider_record_id)
            DO UPDATE SET
                message_id = EXCLUDED.message_id,
                raw_record_id = EXCLUDED.raw_record_id,
                observation_id = EXCLUDED.observation_id,
                subject = EXCLUDED.subject,
                sender = EXCLUDED.sender,
                recipients = EXCLUDED.recipients,
                body_text = EXCLUDED.body_text,
                occurred_at = EXCLUDED.occurred_at,
                channel_kind = EXCLUDED.channel_kind,
                conversation_id = EXCLUDED.conversation_id,
                sender_display_name = EXCLUDED.sender_display_name,
                delivery_state = EXCLUDED.delivery_state,
                message_metadata = EXCLUDED.message_metadata,
                projected_at = now()
            RETURNING
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                workflow_state,
                importance_score,
                ai_category,
                ai_summary,
                ai_summary_generated_at,
                local_state,
                local_state_changed_at,
                local_state_reason
            "#,
        )
        .bind(&canonical_message_id)
        .bind(&message.raw_record_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.subject)
        .bind(&message.sender)
        .bind(json!(message.recipients))
        .bind(&message.body_text)
        .bind(message.occurred_at)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MessageProjectionError::RawRecordTupleMismatch {
                raw_record_id: message.raw_record_id.clone(),
                account_id: message.account_id.clone(),
                provider_record_id: message.provider_record_id.clone(),
            });
        };

        row_to_projected_message(row)
    }

    pub async fn upsert_channel_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.upsert_channel_message_with_body_policy(message, false)
            .await
    }

    pub async fn upsert_channel_message_allowing_empty_body_text(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.upsert_channel_message_with_body_policy(message, true)
            .await
    }

    async fn upsert_channel_message_with_body_policy(
        &self,
        message: &NewProjectedMessage,
        allow_empty_body_text: bool,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        message.validate_with_body_policy(allow_empty_body_text)?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_messages (
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            )
            SELECT
                $1,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                $12,
                $13,
                $14
            FROM communication_raw_records
            WHERE raw_record_id = $2
              AND account_id = $3
              AND provider_record_id = $4
            ON CONFLICT (account_id, provider_record_id)
            DO UPDATE SET
                message_id = EXCLUDED.message_id,
                raw_record_id = EXCLUDED.raw_record_id,
                observation_id = EXCLUDED.observation_id,
                subject = EXCLUDED.subject,
                sender = EXCLUDED.sender,
                recipients = EXCLUDED.recipients,
                body_text = EXCLUDED.body_text,
                occurred_at = EXCLUDED.occurred_at,
                channel_kind = EXCLUDED.channel_kind,
                conversation_id = EXCLUDED.conversation_id,
                sender_display_name = EXCLUDED.sender_display_name,
                delivery_state = EXCLUDED.delivery_state,
                message_metadata = EXCLUDED.message_metadata,
                projected_at = now()
            RETURNING
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                workflow_state,
                importance_score,
                ai_category,
                ai_summary,
                ai_summary_generated_at,
                local_state,
                local_state_changed_at,
                local_state_reason
            "#,
        )
        .bind(&message.message_id)
        .bind(&message.raw_record_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.subject)
        .bind(&message.sender)
        .bind(json!(message.recipients))
        .bind(&message.body_text)
        .bind(message.occurred_at)
        .bind(&message.channel_kind)
        .bind(message.conversation_id.as_deref())
        .bind(message.sender_display_name.as_deref())
        .bind(&message.delivery_state)
        .bind(&message.message_metadata)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MessageProjectionError::RawRecordTupleMismatch {
                raw_record_id: message.raw_record_id.clone(),
                account_id: message.account_id.clone(),
                provider_record_id: message.provider_record_id.clone(),
            });
        };

        row_to_projected_message(row)
    }
}
```

### `backend/src/domains/communications/messages/store/workflow.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/store/workflow.rs`
- Size bytes / Размер в байтах: `3295`
- Included characters / Включено символов: `3295`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use super::MessageProjectionStore;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::rows::row_to_projected_message;
use crate::domains::communications::messages::states::WorkflowState;
use crate::domains::communications::messages::validation::validate_non_empty;

impl MessageProjectionStore {
    pub async fn transition_workflow_state(
        &self,
        message_id: &str,
        new_state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.transition_workflow_state_with_observation(
            message_id,
            new_state,
            None,
            "workflow_state_transition",
            None,
        )
        .await
    }

    pub async fn transition_workflow_state_with_observation(
        &self,
        message_id: &str,
        new_state: WorkflowState,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let message =
            Self::transition_workflow_state_in_transaction(&mut transaction, message_id, new_state)
                .await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                message.message_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "workflow_state": message.workflow_state.as_str(),
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(message)
    }

    pub(crate) async fn transition_workflow_state_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        message_id: &str,
        new_state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        let row = sqlx::query(
            r#"UPDATE communication_messages SET workflow_state = $2, projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, observation_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                local_state, local_state_changed_at, local_state_reason"#,
        )
        .bind(message_id.trim())
        .bind(new_state.as_str())
        .fetch_optional(&mut **transaction)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }
}
```

### `backend/src/domains/communications/messages/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/validation.rs`
- Size bytes / Размер в байтах: `497`
- Included characters / Включено символов: `497`
- Truncated / Обрезано: `no`

```rust
use super::errors::MessageProjectionError;

pub(crate) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), MessageProjectionError> {
    if value.trim().is_empty() {
        return Err(MessageProjectionError::EmptyField(field_name));
    }

    Ok(())
}

pub(crate) fn validate_limit(limit: i64) -> Result<i64, MessageProjectionError> {
    if !(1..=5000).contains(&limit) {
        return Err(MessageProjectionError::InvalidLimit(limit));
    }

    Ok(limit)
}
```

### `backend/src/domains/communications/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/mod.rs`
- Size bytes / Размер в байтах: `835`
- Included characters / Включено символов: `835`
- Truncated / Обрезано: `no`

```rust
pub mod actions;
pub mod ai_reply;
pub mod ai_state;
pub mod analytics;
pub mod archive_inspection;
pub mod attachment_dedup;
pub mod attachment_search;
pub mod blockers;
pub mod bulk_actions;
mod command_service;
pub mod core;
pub mod delivery_notifications;
pub mod drafts;
pub(crate) mod evidence;
pub mod explain;
pub mod export;
pub mod extract;
pub mod finance;
pub mod fixtures;
pub mod flags;
pub mod folders;
pub mod import;
pub mod ingestion;
pub mod legal;
pub mod messages;
pub mod multilingual;
pub mod outbox;
pub mod personas;
pub mod ports;
pub mod read_receipts;
pub mod rich_template;
pub mod rules;
pub mod saved_search_counts;
pub mod saved_searches;
pub mod search;
pub mod service;
pub mod signatures;
pub mod sources;
pub mod spf_dkim;
pub mod storage;
pub mod subscriptions;
pub mod templates;
pub mod threads;
```

### `backend/src/domains/communications/multilingual.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/multilingual.rs`
- Size bytes / Размер в байтах: `5984`
- Included characters / Включено символов: `5964`
- Truncated / Обрезано: `no`

```rust
use crate::platform::ai_runtime::{AiRuntimePortError, SharedAiRuntimePort};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LanguageDetection {
    pub language: String,
    pub confidence: f32,
    pub script: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Translation {
    pub original_language: String,
    pub target_language: String,
    pub translated_text: String,
    pub model: String,
}

#[derive(Clone)]
pub struct MultilingualService {
    runtime: Option<SharedAiRuntimePort>,
}

impl MultilingualService {
    pub fn new(runtime: Option<SharedAiRuntimePort>) -> Self {
        Self { runtime }
    }

    /// Heuristic language detection based on character sets and common words.
    pub fn detect_language(text: &str) -> LanguageDetection {
        let text = text.trim();
        if text.is_empty() {
            return LanguageDetection {
                language: "unknown".into(),
                confidence: 0.0,
                script: None,
            };
        }

        let has_cyrillic = text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
        let has_spanish = text.to_lowercase().contains('ñ');
        let has_latin = text.chars().any(|c| c.is_ascii_alphabetic());
        let has_cjk = text.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));

        let lower = text.to_lowercase();

        if has_cyrillic {
            // Distinguish Russian vs Ukrainian
            if lower.contains('ї') || lower.contains('є') {
                return LanguageDetection {
                    language: "uk".into(),
                    confidence: 0.85,
                    script: Some("cyrillic".into()),
                };
            }
            return LanguageDetection {
                language: "ru".into(),
                confidence: 0.90,
                script: Some("cyrillic".into()),
            };
        }
        if has_spanish {
            return LanguageDetection {
                language: "es".into(),
                confidence: 0.85,
                script: Some("latin".into()),
            };
        }
        if has_cjk {
            return LanguageDetection {
                language: "zh".into(),
                confidence: 0.70,
                script: Some("cjk".into()),
            };
        }

        // Check common words
        let spanish_words = [
            "hola",
            "gracias",
            "para",
            "como",
            "que",
            "por favor",
            "saludos",
            "adjunto",
        ];
        let russian_latin = ["privet", "spasibo", "pozhaluysta"];
        let german_words = [
            "mit", "und", "der", "die", "das", "ist", "von", "für", "danke", "bitte",
        ];

        let spanish_score = spanish_words.iter().filter(|w| lower.contains(*w)).count() as f32
            / spanish_words.len() as f32;
        let russian_latin_score = russian_latin.iter().filter(|w| lower.contains(*w)).count()
            as f32
            / russian_latin.len() as f32;
        let german_score = german_words.iter().filter(|w| lower.contains(*w)).count() as f32
            / german_words.len() as f32;

        if spanish_score > 0.1 {
            return LanguageDetection {
                language: "es".into(),
                confidence: 0.70,
                script: Some("latin".into()),
            };
        }
        if russian_latin_score > 0.1 {
            return LanguageDetection {
                language: "ru".into(),
                confidence: 0.55,
                script: Some("latin".into()),
            };
        }
        if german_score > 0.1 {
            return LanguageDetection {
                language: "de".into(),
                confidence: 0.65,
                script: Some("latin".into()),
            };
        }
        if has_latin {
            return LanguageDetection {
                language: "en".into(),
                confidence: 0.50,
                script: Some("latin".into()),
            };
        }

        LanguageDetection {
            language: "unknown".into(),
            confidence: 0.0,
            script: None,
        }
    }

    /// Translate text using LLM. Falls back to identity if no LLM.
    pub async fn translate(
        &self,
        text: &str,
        target_lang: &str,
    ) -> Result<Option<Translation>, MultilingualError> {
        let Some(ref runtime) = self.runtime else {
            return Ok(None);
        };
        let prompt = format!(
            "Translate the following text to {target_lang}. Return ONLY the translated text, no explanations:\n\n{text}"
        );
        let result = runtime.chat(&prompt).await?;
        Ok(Some(Translation {
            original_language: "detected".into(),
            target_language: target_lang.into(),
            translated_text: result.content,
            model: result.model,
        }))
    }
}

#[derive(Debug, Error)]
pub enum MultilingualError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimePortError),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detect_russian() {
        let d = MultilingualService::detect_language("Привет, как дела?");
        assert_eq!(d.language, "ru");
    }
    #[test]
    fn detect_spanish() {
        let d = MultilingualService::detect_language("Hola, ¿cómo estás?");
        assert_eq!(d.language, "es");
    }
    #[test]
    fn detect_english() {
        let d = MultilingualService::detect_language("Hello, how are you?");
        assert_eq!(d.language, "en");
    }
    #[test]
    fn detect_spanish_words() {
        let d = MultilingualService::detect_language("Gracias por su ayuda, saludos cordiales");
        assert_eq!(d.language, "es");
    }
    #[test]
    fn detect_empty() {
        let d = MultilingualService::detect_language("");
        assert_eq!(d.language, "unknown");
    }
}
```

### `backend/src/domains/communications/outbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/outbox.rs`
- Size bytes / Размер в байтах: `31359`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, PgRow, Postgres};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
pub use crate::platform::communications::SmtpTransport;
use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

mod delivery;
mod delivery_status;
mod provider_send_store;
mod provider_sender;
mod smtp_sender;

pub use delivery::{
    EmailOutboxDeliveryWorker, OutboxDeliveryError, OutboxDeliveryReport, OutboxEmailSender,
    OutboxRetryPolicy, OutboxSendReceipt,
};
pub use delivery_status::{
    NewOutboxDeliveryStatus, OutboxDeliveryStatus, OutboxDeliveryStatusRecord,
};
pub use provider_send_store::{ProviderSendStore, ProviderSendStoreError};
pub use provider_sender::CommunicationOutboxEmailSender;
pub use smtp_sender::{
    SmtpOutboxEmailSender, outgoing_email_from_outbox_item, smtp_config_for_provider_account,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationOutboxStatus {
    Queued,
    Scheduled,
    Sending,
    Sent,
    Failed,
    Canceled,
}

impl CommunicationOutboxStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Scheduled => "scheduled",
            Self::Sending => "sending",
            Self::Sent => "sent",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "queued" => Some(Self::Queued),
            "scheduled" => Some(Self::Scheduled),
            "sending" => Some(Self::Sending),
            "sent" => Some(Self::Sent),
            "failed" => Some(Self::Failed),
            "canceled" => Some(Self::Canceled),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommunicationOutboxItem {
    pub outbox_id: String,
    pub account_id: String,
    pub draft_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub status: CommunicationOutboxStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub undo_deadline_at: Option<DateTime<Utc>>,
    pub send_attempts: i32,
    pub claimed_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub provider_message_id: Option<String>,
    pub last_error: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailOutboxListPage {
    pub items: Vec<CommunicationOutboxItem>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct NewCommunicationOutboxItem {
    pub outbox_id: String,
    pub account_id: String,
    pub draft_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub status: CommunicationOutboxStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub undo_deadline_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewCommunicationOutboxItem {
    fn validate(&self) -> Result<(), CommunicationOutboxError> {
        validate_non_empty("outbox_id", &self.outbox_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        if self
            .to_recipients
            .iter()
            .chain(self.cc_recipients.iter())
            .chain(self.bcc_recipients.iter())
            .all(|recipient| recipient.trim().is_empty())
        {
            return Err(CommunicationOutboxError::Invalid(
                "at least one recipient is required",
            ));
        }
        if !self.metadata.is_object() {
            return Err(CommunicationOutboxError::Invalid(
                "metadata must be a JSON object",
            ));
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct CommunicationOutboxStore {
    pool: PgPool,
}

impl CommunicationOutboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(
        &self,
        outbox_id: &str,
    ) -> Result<Option<CommunicationOutboxItem>, CommunicationOutboxError> {
        let row = sqlx::query(
            r#"
            SELECT
                outbox.outbox_id,
                outbox.account_id,
                outbox.draft_id,
                outbox.to_participants AS to_recipients,
                outbox.cc_participants AS cc_recipients,
                outbox.bcc_participants AS bcc_recipients,
                outbox.subject,
                outbox.body_text,
                outbox.body_html,
                outbox.status,
                outbox.scheduled_send_at,
                outbox.undo_deadline_at,
                outbox.send_attempts,
                outbox.claimed_at,
                outbox.sent_at,
                outbox.provider_message_id,
                outbox.last_error,
                CASE
                    WHEN latest_receipt.latest_read_receipt IS NULL THEN outbox.metadata
                    ELSE jsonb_set(
                        outbox.metadata,
                        '{latest_read_receipt}',
                        latest_receipt.latest_read_receipt,
                        true
                    )
                END AS metadata,
                outbox.created_at,
                outbox.updated_at
            FROM communication_outbox outbox
            LEFT JOIN LATERAL (
                SELECT jsonb_build_object(
                    'receipt_kind', receipt.receipt_kind,
                    'read_at', receipt.read_at,
                    'source_kind', receipt.source_kind
                ) AS latest_read_receipt
                FROM communication_read_receipts receipt
                WHERE receipt.outbox_id = outbox.outbox_id
                ORDER BY receipt.read_at DESC, receipt.receipt_id ASC
                LIMIT 1
            ) latest_receipt ON true
            WHERE outbox.outbox_id = $1
            "#,
        )
        .bind(outbox_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_outbox_item).transpose()
    }

    pub async fn enqueue(
        &self,
        item: &NewCommunicationOutboxItem,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        self.enqueue_with_observation(item, None, "outbox_status_transition", None)
            .await
    }

    pub async fn enqueue_with_observation(
        &self,
        item: &NewCommunicationOutboxItem,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CommunicationOutboxItem, CommunicationOutboxError> {
        item.validate()?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, Some(item.account_id.as_str()))
            .await?;
        let sql = outbox_returning_query(
            r#"
            INSERT INTO communication_outbox (
                outbox_id,
                account_id,
                draft_id,
                to_participants,
                cc_participants,
                bcc_participants,
                subject,
                body_text,
                body_html,
                status,
                scheduled_send_at,
                undo_deadline_at,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            "communication_outbox",
        );
        let row = sqlx::query(&sql)
            .bind(item.outbox_id.trim())
            .bind(item.account_id.trim())
            .bind(item.draft_id.as_deref())
            .bind(serde_json::to_value(&item.to_recipients)?)
            .bind(serde_json::to_value(&item.cc_recipients)?)
            .bind(serde_json::to_value(&item.bcc_recipients)?)
            .bind(&item.subject)
            .bind(&item.body_text)
            .bind(item.body_html.as_deref())
            .bind(item.status.as_str())
            .bind(item.scheduled_send_at)
            .bind(item.undo_deadline_at)
            .bind(&item.metadata)
            .fetch_one(&mut *transaction)
            .await?;

        let outbox_item = row_to_outbox_item(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.trim().is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "status": outbox_item.status.as_str(),
                    "draft_id": outbox_item.draft_id,
                    "scheduled_send_at": outbox_item.scheduled_send_at,
                    "undo_deadline_at": outbox_item.undo_deadline_at,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "outbox_item",
                outbox_item.outbox_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(outbox_item)
    }

    pub async fn list(
        &self,
        account_id: Option<&str>,
        status: Option<CommunicationOutboxStatus>,
        limit: i64,
    ) -> Result<Vec<CommunicationOutboxItem>, CommunicationOutboxError> {
        Ok(self.list_page(account_id, status, None, limit).await?.items)
    }

    pub async fn list_page(
        &self,
        account_id: Option<&str>,
        status: Option<CommunicationOutboxStatus>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<EmailOutboxListPage, CommunicationOutboxError> {
        let limit = validate_limit(limit)?;
        let cursor = cursor.map(decode_outbox_list_cursor).transpose()?;
        let status = status.map(CommunicationOutboxStatus::as_str);
        let rows = sqlx::query(
            r#"
            SELECT
                outbox.outbox_id,
                outbox.account_id,
                outbox.draft_id,
                outbox.to_participants AS to_recipients,
                outbox.cc_participants AS cc_recipients,
                outbox.bcc_participants AS bcc_recipients,
                outbox.subject,
                outbox.body_text,
                outbox.body_html,
                outbox.status,
                outbox.scheduled_send_at,
                outbox.undo_deadline_at,
                outbox.send_attempts,
                outbox.claimed_at,
                outbox.sent_at,
                outbox.provider_message_id,
                outbox.last_error,
                CASE
                    WHEN latest_receipt.latest_read_receipt IS NULL THEN outbox.metadata
                    ELSE jsonb_set(
                        outbox.metadata,
                        '{latest_read_receipt}',
                        latest_receipt.latest_read_receipt,
                        true
                    )
                END AS metadata,
                outbox.created_at,
                outbox.updated_at
            FROM communication_outbox outbox
            LEFT JOIN LATERAL (
                SELECT jsonb_build_object(
                    'receipt_kind', receipt.receipt_kind,
                    'read_at', receipt.read_at,
                    'source_kind', receipt.source_kind
                ) AS latest_read_receipt

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/outbox/delivery.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/outbox/delivery.rs`
- Size bytes / Размер в байтах: `4731`
- Included characters / Включено символов: `4731`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

use super::{CommunicationOutboxError, CommunicationOutboxItem, CommunicationOutboxStore};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxSendReceipt {
    pub provider_message_id: String,
    pub accepted_recipients: Vec<String>,
}

#[derive(Debug, Error)]
pub enum OutboxDeliveryError {
    #[error("{0}")]
    Transport(String),
}

impl OutboxDeliveryError {
    pub fn public_message(&self) -> &str {
        match self {
            Self::Transport(message) => message.as_str(),
        }
    }
}

pub trait OutboxEmailSender: Send + Sync {
    fn send<'a>(
        &'a self,
        item: &'a CommunicationOutboxItem,
    ) -> Pin<Box<dyn Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>> + Send + 'a>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxRetryPolicy {
    max_attempts: i32,
    base_delay: Duration,
    max_delay: Duration,
}

impl OutboxRetryPolicy {
    pub fn new(max_attempts: i32, base_delay: Duration, max_delay: Duration) -> Self {
        let base_delay = duration_with_minimum_seconds(base_delay, 1);
        let max_delay = duration_with_minimum_seconds(max_delay, base_delay.num_seconds());

        Self {
            max_attempts: max_attempts.max(1),
            base_delay,
            max_delay,
        }
    }

    pub fn disabled() -> Self {
        Self::new(1, Duration::seconds(1), Duration::seconds(1))
    }

    fn next_attempt_at(
        &self,
        now: DateTime<Utc>,
        completed_attempts: i32,
    ) -> Option<DateTime<Utc>> {
        if completed_attempts >= self.max_attempts {
            return None;
        }

        let retry_index = completed_attempts.saturating_sub(1).max(0) as u32;
        let factor = 1_i64.checked_shl(retry_index.min(30)).unwrap_or(i64::MAX);
        let delay_seconds = self
            .base_delay
            .num_seconds()
            .saturating_mul(factor)
            .min(self.max_delay.num_seconds());

        Some(now + Duration::seconds(delay_seconds))
    }
}

impl Default for OutboxRetryPolicy {
    fn default() -> Self {
        Self::new(3, Duration::seconds(30), Duration::minutes(15))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxDeliveryReport {
    pub claimed: usize,
    pub sent: usize,
    pub failed: usize,
    pub retried: usize,
}

#[derive(Clone)]
pub struct EmailOutboxDeliveryWorker<S> {
    store: CommunicationOutboxStore,
    sender: S,
    retry_policy: OutboxRetryPolicy,
}

impl<S> EmailOutboxDeliveryWorker<S>
where
    S: OutboxEmailSender,
{
    pub fn new(store: CommunicationOutboxStore, sender: S) -> Self {
        Self::with_retry_policy(store, sender, OutboxRetryPolicy::default())
    }

    pub fn with_retry_policy(
        store: CommunicationOutboxStore,
        sender: S,
        retry_policy: OutboxRetryPolicy,
    ) -> Self {
        Self {
            store,
            sender,
            retry_policy,
        }
    }

    pub async fn deliver_due(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<OutboxDeliveryReport, CommunicationOutboxError> {
        let claimed = self.store.claim_due(now, limit).await?;
        let mut report = OutboxDeliveryReport {
            claimed: claimed.len(),
            sent: 0,
            failed: 0,
            retried: 0,
        };

        for item in claimed {
            match self.sender.send(&item).await {
                Ok(receipt) => {
                    self.store.mark_sent(&item.outbox_id, now, &receipt).await?;
                    report.sent += 1;
                }
                Err(error) => {
                    if let Some(next_attempt_at) =
                        self.retry_policy.next_attempt_at(now, item.send_attempts)
                    {
                        self.store
                            .mark_retry_scheduled(
                                &item.outbox_id,
                                now,
                                next_attempt_at,
                                error.public_message(),
                            )
                            .await?;
                        report.retried += 1;
                    } else {
                        self.store
                            .mark_failed(&item.outbox_id, now, error.public_message())
                            .await?;
                        report.failed += 1;
                    }
                }
            }
        }

        Ok(report)
    }
}

fn duration_with_minimum_seconds(duration: Duration, minimum_seconds: i64) -> Duration {
    Duration::seconds(duration.num_seconds().max(minimum_seconds))
}
```

### `backend/src/domains/communications/outbox/delivery_status.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/outbox/delivery_status.rs`
- Size bytes / Размер в байтах: `8510`
- Included characters / Включено символов: `8510`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

use super::super::evidence::link_mail_entity_in_transaction;
use super::{
    CommunicationOutboxError, CommunicationOutboxStore, generate_outbox_event_id,
    validate_non_empty,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxDeliveryStatus {
    Delivered,
    Delayed,
    Failed,
}

impl OutboxDeliveryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Delayed => "delayed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewOutboxDeliveryStatus {
    pub account_id: String,
    pub provider_message_id: String,
    pub delivery_status: OutboxDeliveryStatus,
    pub smtp_status: Option<String>,
    pub source_kind: String,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutboxDeliveryStatusRecord {
    pub account_id: String,
    pub outbox_id: Option<String>,
    pub provider_message_id: String,
    pub delivery_status: OutboxDeliveryStatus,
    pub smtp_status: Option<String>,
    pub source_kind: String,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

impl CommunicationOutboxStore {
    pub async fn record_delivery_status(
        &self,
        delivery_status: &NewOutboxDeliveryStatus,
    ) -> Result<OutboxDeliveryStatusRecord, CommunicationOutboxError> {
        let account_id = normalize_non_empty("account_id", &delivery_status.account_id)?;
        let provider_message_id =
            normalize_non_empty("provider_message_id", &delivery_status.provider_message_id)?;
        let source_kind = normalize_non_empty("source_kind", &delivery_status.source_kind)?;
        let smtp_status = normalize_optional(delivery_status.smtp_status.as_deref());
        let provider_record_id = normalize_optional(delivery_status.provider_record_id.as_deref());
        let raw_record_id = normalize_optional(delivery_status.raw_record_id.as_deref());
        let metadata = json!({
            "delivery_status": delivery_status.delivery_status.as_str(),
            "smtp_status": smtp_status,
            "source_kind": source_kind,
            "provider_record_id": provider_record_id,
            "recorded_at": delivery_status.recorded_at,
        });
        let terminal_error = match (delivery_status.delivery_status, smtp_status.as_deref()) {
            (OutboxDeliveryStatus::Failed, Some(status)) => {
                Some(format!("delivery failed with SMTP status {status}"))
            }
            (OutboxDeliveryStatus::Failed, None) => Some("delivery failed".to_owned()),
            _ => None,
        };

        let mut transaction = self.pool.begin().await?;
        let outbox_id = sqlx::query_scalar::<_, String>(
            r#"
            UPDATE communication_outbox
            SET metadata = jsonb_set(metadata, '{delivery_status}', $3::jsonb, true),
                last_error = CASE
                    WHEN $4::text IS NULL THEN last_error
                    ELSE $4
                END,
                updated_at = $5
            WHERE account_id = $1
              AND provider_message_id = $2
            RETURNING outbox_id
            "#,
        )
        .bind(&account_id)
        .bind(&provider_message_id)
        .bind(&metadata)
        .bind(terminal_error.as_deref())
        .bind(delivery_status.recorded_at)
        .fetch_optional(&mut *transaction)
        .await?;
        let record = OutboxDeliveryStatusRecord {
            account_id,
            outbox_id,
            provider_message_id,
            delivery_status: delivery_status.delivery_status,
            smtp_status,
            source_kind,
            provider_record_id,
            raw_record_id,
            recorded_at: delivery_status.recorded_at,
        };
        capture_delivery_status_observation(&mut transaction, &record).await?;
        let event = outbox_delivery_status_event(&record)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(record)
    }
}

async fn capture_delivery_status_observation(
    transaction: &mut Transaction<'_, Postgres>,
    record: &OutboxDeliveryStatusRecord,
) -> Result<(), CommunicationOutboxError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "COMMUNICATION_DELIVERY_STATUS",
            ObservationOriginKind::LocalRuntime,
            record.recorded_at,
            json!({
                "account_id": record.account_id,
                "outbox_id": record.outbox_id,
                "provider_message_id": record.provider_message_id,
                "delivery_status": record.delivery_status.as_str(),
                "smtp_status": record.smtp_status,
                "source_kind": record.source_kind,
                "provider_record_id": record.provider_record_id,
                "raw_record_id": record.raw_record_id,
                "operation": "delivery_status_recorded",
            }),
            format!(
                "delivery-status://{}/{}",
                record
                    .outbox_id
                    .as_deref()
                    .unwrap_or(record.provider_message_id.as_str()),
                record.delivery_status.as_str()
            ),
        )
        .provenance(json!({
            "captured_by": "mail.outbox.delivery_status",
            "operation": "delivery_status_recorded",
        })),
    )
    .await?;
    if let Some(outbox_id) = &record.outbox_id {
        link_mail_entity_in_transaction(
            transaction,
            &observation.observation_id,
            "outbox_item",
            outbox_id.clone(),
            "delivery_status_observed",
            json!({
                "delivery_status": record.delivery_status.as_str(),
                "smtp_status": record.smtp_status,
            }),
            None,
        )
        .await?;
    }
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_message",
        record.provider_message_id.clone(),
        "delivery_status_observed",
        json!({
            "delivery_status": record.delivery_status.as_str(),
            "source_kind": record.source_kind,
        }),
        None,
    )
    .await?;
    Ok(())
}

fn normalize_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<String, CommunicationOutboxError> {
    validate_non_empty(field_name, value)?;
    Ok(value.trim().to_owned())
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn outbox_delivery_status_event(
    record: &OutboxDeliveryStatusRecord,
) -> Result<NewEventEnvelope, CommunicationOutboxError> {
    let subject_id = record
        .outbox_id
        .as_deref()
        .unwrap_or(record.provider_message_id.as_str());
    Ok(NewEventEnvelope::builder(
        generate_outbox_event_id("mail.outbox.delivery_status_changed", subject_id),
        "mail.outbox.delivery_status_changed",
        Utc::now(),
        json!({ "kind": "mail_delivery_notification" }),
        json!({
            "kind": "email_outbox_delivery_status",
            "id": subject_id,
            "account_id": record.account_id,
            "outbox_id": record.outbox_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-delivery-notification" }))
    .payload(json!({
        "account_id": record.account_id,
        "outbox_id": record.outbox_id,
        "provider_message_id": record.provider_message_id,
        "delivery_status": record.delivery_status.as_str(),
        "smtp_status": record.smtp_status,
        "source_kind": record.source_kind,
        "recorded_at": record.recorded_at,
    }))
    .provenance(json!({
        "source_kind": record.source_kind,
        "source_id": record.provider_record_id,
    }))
    .correlation_id(subject_id.to_owned())
    .build()?)
}
```

### `backend/src/domains/communications/outbox/provider_send_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/outbox/provider_send_store.rs`
- Size bytes / Размер в байтах: `1908`
- Included characters / Включено символов: `1908`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

use super::super::evidence::link_mail_entity_in_transaction;

#[derive(Clone)]
pub struct ProviderSendStore {
    pool: PgPool,
}

impl ProviderSendStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_sent_with_observation(
        &self,
        observation_id: &str,
        provider_message_id: &str,
        transport: &str,
        metadata: Option<Value>,
    ) -> Result<(), ProviderSendStoreError> {
        if observation_id.trim().is_empty() {
            return Err(ProviderSendStoreError::Invalid(
                "observation_id must not be empty",
            ));
        }
        if provider_message_id.trim().is_empty() {
            return Err(ProviderSendStoreError::Invalid(
                "provider_message_id must not be empty",
            ));
        }
        if transport.trim().is_empty() {
            return Err(ProviderSendStoreError::Invalid(
                "transport must not be empty",
            ));
        }

        let mut transaction = self.pool.begin().await?;
        link_mail_entity_in_transaction(
            &mut transaction,
            observation_id.trim(),
            "provider_send",
            provider_message_id.trim().to_owned(),
            "provider_send",
            json!({
                "transport": transport.trim(),
                "status": "sent",
            }),
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ProviderSendStoreError {
    #[error("invalid provider send payload: {0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
```

### `backend/src/domains/communications/outbox/provider_sender.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/outbox/provider_sender.rs`
- Size bytes / Размер в байтах: `4693`
- Included characters / Включено символов: `4693`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore, EmailProviderKind,
    ProviderAccount, ProviderAccountSecretPurpose,
};
use crate::platform::communications::{
    GmailOutboxSendRequest, GmailOutboxTransport, SmtpTransport,
};
use crate::platform::secrets::SecretResolver;

use super::smtp_sender::SmtpOutboxEmailSender;
use super::{
    CommunicationOutboxItem, OutboxDeliveryError, OutboxEmailSender, OutboxSendReceipt,
    outgoing_email_from_outbox_item,
};

#[derive(Clone)]
pub struct CommunicationOutboxEmailSender<R, T, G> {
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    smtp_sender: SmtpOutboxEmailSender<R, T>,
    gmail_transport: G,
}

impl<R, T, G> CommunicationOutboxEmailSender<R, T, G>
where
    R: SecretResolver,
    T: SmtpTransport,
    G: GmailOutboxTransport,
{
    pub fn new(pool: PgPool, resolver: R, smtp_transport: T, gmail_transport: G) -> Self {
        Self {
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            smtp_sender: SmtpOutboxEmailSender::new(pool, resolver, smtp_transport),
            gmail_transport,
        }
    }
}

impl<R, T, G> OutboxEmailSender for CommunicationOutboxEmailSender<R, T, G>
where
    R: SecretResolver + Send + Sync,
    T: SmtpTransport,
    G: GmailOutboxTransport,
{
    fn send<'a>(
        &'a self,
        item: &'a CommunicationOutboxItem,
    ) -> Pin<Box<dyn Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>> + Send + 'a>>
    {
        Box::pin(async move {
            let account = self
                .provider_account_store
                .get(&item.account_id)
                .await
                .map_err(|error| delivery_error("provider account lookup failed", error))?
                .ok_or_else(|| {
                    OutboxDeliveryError::Transport("provider account was not found".to_owned())
                })?;

            if matches!(account.provider_kind, EmailProviderKind::Gmail)
                && gmail_send_enabled(&account.config)
            {
                return self.send_gmail(item, &account).await;
            }

            self.smtp_sender.send(item).await
        })
    }
}

impl<R, T, G> CommunicationOutboxEmailSender<R, T, G>
where
    R: SecretResolver,
    T: SmtpTransport,
    G: GmailOutboxTransport,
{
    async fn send_gmail(
        &self,
        item: &CommunicationOutboxItem,
        account: &ProviderAccount,
    ) -> Result<OutboxSendReceipt, OutboxDeliveryError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::OauthToken,
            )
            .await
            .map_err(|error| delivery_error("Gmail OAuth credential lookup failed", error))?
            .ok_or_else(|| {
                OutboxDeliveryError::Transport(
                    "Gmail OAuth credential is unavailable for this account".to_owned(),
                )
            })?;
        let email = outgoing_email_from_outbox_item(item, account);
        let result = self
            .gmail_transport
            .send(GmailOutboxSendRequest {
                account_id: &account.account_id,
                oauth_secret_ref: &binding.secret_ref,
                api_base_url: gmail_api_base_url(&account.config),
                email: &email,
            })
            .await
            .map_err(|error| delivery_error("Gmail API send failed", error))?;

        Ok(OutboxSendReceipt {
            provider_message_id: result.message_id,
            accepted_recipients: result.accepted_recipients,
        })
    }
}

fn gmail_send_enabled(config: &Value) -> bool {
    config
        .get("gmail_send_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn gmail_api_base_url(config: &Value) -> &str {
    config
        .get("gmail_api_base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("https://www.googleapis.com")
}

fn delivery_error(
    public_message: &'static str,
    error: impl std::fmt::Display,
) -> OutboxDeliveryError {
    tracing::warn!(error = %error, "provider outbox delivery failed");
    OutboxDeliveryError::Transport(public_message.to_owned())
}
```

### `backend/src/domains/communications/outbox/smtp_sender.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/outbox/smtp_sender.rs`
- Size bytes / Размер в байтах: `6666`
- Included characters / Включено символов: `6666`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use crate::domains::communications::core::{
    EmailProviderKind, ProviderAccount, ProviderAccountSecretPurpose, ProviderCredentialReader,
};
use crate::platform::communications::{OutgoingEmail, SmtpConfig, SmtpTransport};
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::{CommunicationOutboxItem, OutboxDeliveryError, OutboxEmailSender, OutboxSendReceipt};

#[derive(Clone)]
pub struct SmtpOutboxEmailSender<R, T> {
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    resolver: R,
    transport: T,
}

impl<R, T> SmtpOutboxEmailSender<R, T>
where
    R: SecretResolver,
    T: SmtpTransport,
{
    pub fn new(pool: PgPool, resolver: R, transport: T) -> Self {
        Self {
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            secret_store: SecretReferenceStore::new(pool),
            resolver,
            transport,
        }
    }
}

impl<R, T> OutboxEmailSender for SmtpOutboxEmailSender<R, T>
where
    R: SecretResolver + Send + Sync,
    T: SmtpTransport,
{
    fn send<'a>(
        &'a self,
        item: &'a CommunicationOutboxItem,
    ) -> Pin<Box<dyn Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>> + Send + 'a>>
    {
        Box::pin(async move {
            let account = self
                .provider_account_store
                .get(&item.account_id)
                .await
                .map_err(|error| delivery_error("provider account lookup failed", error))?
                .ok_or_else(|| {
                    OutboxDeliveryError::Transport("provider account was not found".to_owned())
                })?;
            let smtp_config = smtp_config_for_provider_account(&account)?;
            let credential_reader = ProviderCredentialReader::new(
                self.provider_secret_binding_store.clone(),
                self.secret_store.clone(),
                &self.resolver,
            );
            let credential = credential_reader
                .read(
                    &account.account_id,
                    ProviderAccountSecretPurpose::SmtpPassword,
                )
                .await
                .map_err(|error| {
                    delivery_error("SMTP credential is unavailable for this account", error)
                })?;
            let email = outgoing_email_from_outbox_item(item, &account);
            let result = self
                .transport
                .send(&smtp_config, &credential.secret, &email)
                .await
                .map_err(|error| delivery_error("SMTP send failed", error))?;

            Ok(OutboxSendReceipt {
                provider_message_id: result.message_id,
                accepted_recipients: result.accepted_recipients,
            })
        })
    }
}

pub fn smtp_config_for_provider_account(
    account: &ProviderAccount,
) -> Result<SmtpConfig, OutboxDeliveryError> {
    match account.provider_kind {
        EmailProviderKind::Icloud | EmailProviderKind::Imap => {}
        EmailProviderKind::Gmail => {
            return Err(OutboxDeliveryError::Transport(
                "Gmail send is unavailable until OAuth send scopes are configured".to_owned(),
            ));
        }
        _ => {
            return Err(OutboxDeliveryError::Transport(
                "provider does not support SMTP send".to_owned(),
            ));
        }
    }

    let config = account.config.as_object().ok_or_else(|| {
        OutboxDeliveryError::Transport("provider account config must be a JSON object".to_owned())
    })?;
    let host = config
        .get("smtp_host")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            OutboxDeliveryError::Transport("SMTP config is unavailable for this account".to_owned())
        })?;
    let port = config
        .get("smtp_port")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0 && *value <= u64::from(u16::MAX))
        .ok_or_else(|| {
            OutboxDeliveryError::Transport("SMTP port is unavailable for this account".to_owned())
        })? as u16;
    let username = config
        .get("smtp_username")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(account.external_account_id.as_str());
    let tls = config
        .get("smtp_tls")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let starttls = config
        .get("smtp_starttls")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(SmtpConfig::new(host, port, tls, username).starttls(starttls))
}

pub fn outgoing_email_from_outbox_item(
    item: &CommunicationOutboxItem,
    account: &ProviderAccount,
) -> OutgoingEmail {
    OutgoingEmail {
        from: account.external_account_id.clone(),
        to: item.to_recipients.clone(),
        cc: item.cc_recipients.clone(),
        bcc: item.bcc_recipients.clone(),
        subject: item.subject.clone(),
        body_text: item.body_text.clone(),
        body_html: item.body_html.clone(),
        in_reply_to: optional_metadata_string(&item.metadata, "in_reply_to"),
        references: metadata_string_array(&item.metadata, "references"),
    }
}

fn optional_metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn metadata_string_array(metadata: &Value, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn delivery_error(
    public_message: &'static str,
    error: impl std::fmt::Display,
) -> OutboxDeliveryError {
    tracing::warn!(error = %error, "outbox SMTP delivery failed");
    OutboxDeliveryError::Transport(public_message.to_owned())
}
```

### `backend/src/domains/communications/personas.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/personas.rs`
- Size bytes / Размер в байтах: `5557`
- Included characters / Включено символов: `5557`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationPersona {
    pub persona_id: String,
    pub name: String,
    pub account_id: String,
    pub display_name: String,
    pub signature: String,
    pub default_language: Option<String>,
    pub default_tone: Option<String>,
    pub is_default: bool,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CommunicationPersonaStore {
    pool: PgPool,
}

impl CommunicationPersonaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        persona: &NewCommunicationPersona,
    ) -> Result<CommunicationPersona, CommunicationPersonaError> {
        persona.validate()?;
        ensure_canonical_account(&self.pool, &persona.account_id).await?;
        let row = sqlx::query(
            r#"INSERT INTO communication_personas (persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (persona_id) DO UPDATE SET
                name = EXCLUDED.name, account_id = EXCLUDED.account_id, display_name = EXCLUDED.display_name,
                signature = EXCLUDED.signature, default_language = EXCLUDED.default_language,
                default_tone = EXCLUDED.default_tone, is_default = EXCLUDED.is_default,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata, created_at, updated_at"#,
        )
        .bind(&persona.persona_id).bind(&persona.name).bind(&persona.account_id)
        .bind(&persona.display_name).bind(&persona.signature)
        .bind(persona.default_language.as_deref()).bind(persona.default_tone.as_deref())
        .bind(persona.is_default).bind(&persona.metadata)
        .fetch_one(&self.pool).await?;
        row_to_persona(row)
    }

    pub async fn list(&self) -> Result<Vec<CommunicationPersona>, CommunicationPersonaError> {
        let rows = sqlx::query(
            r#"SELECT persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata, created_at, updated_at
            FROM communication_personas ORDER BY name"#,
        ).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_persona).collect()
    }

    pub async fn get_default(
        &self,
    ) -> Result<Option<CommunicationPersona>, CommunicationPersonaError> {
        let row = sqlx::query(
            r#"SELECT persona_id, name, account_id, display_name, signature, default_language, default_tone, is_default, metadata, created_at, updated_at
            FROM communication_personas WHERE is_default = true LIMIT 1"#,
        ).fetch_optional(&self.pool).await?;
        row.map(row_to_persona).transpose()
    }
}

async fn ensure_canonical_account(
    pool: &PgPool,
    account_id: &str,
) -> Result<(), CommunicationPersonaError> {
    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(pool)
    .await?;
    Ok(())
}

fn row_to_persona(row: PgRow) -> Result<CommunicationPersona, CommunicationPersonaError> {
    Ok(CommunicationPersona {
        persona_id: row.try_get("persona_id")?,
        name: row.try_get("name")?,
        account_id: row.try_get("account_id")?,
        display_name: row.try_get("display_name")?,
        signature: row.try_get("signature")?,
        default_language: row.try_get("default_language")?,
        default_tone: row.try_get("default_tone")?,
        is_default: row.try_get("is_default")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Clone, Debug)]
pub struct NewCommunicationPersona {
    pub persona_id: String,
    pub name: String,
    pub account_id: String,
    pub display_name: String,
    pub signature: String,
    pub default_language: Option<String>,
    pub default_tone: Option<String>,
    pub is_default: bool,
    pub metadata: Value,
}

impl NewCommunicationPersona {
    fn validate(&self) -> Result<(), CommunicationPersonaError> {
        if self.persona_id.trim().is_empty() {
            return Err(CommunicationPersonaError::Invalid("persona_id empty"));
        }
        if self.name.trim().is_empty() {
            return Err(CommunicationPersonaError::Invalid("name empty"));
        }
        if self.account_id.trim().is_empty() {
            return Err(CommunicationPersonaError::Invalid("account_id empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CommunicationPersonaError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid persona: {0}")]
    Invalid(&'static str),
}
```

### `backend/src/domains/communications/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/ports.rs`
- Size bytes / Размер в байтах: `426`
- Included characters / Включено символов: `426`
- Truncated / Обрезано: `no`

```rust
pub use super::core::CommunicationIngestionStore as CommunicationIngestionPort;
pub use super::core::CommunicationProviderAccountStore as CommunicationProviderAccountPort;
pub use super::messages::MessageProjectionStore as CommunicationMessageProjectionPort;
pub use super::storage::CommunicationStorageStore as CommunicationBlobMetadataPort;
pub use super::storage::LocalCommunicationBlobStore as LocalCommunicationBlobPort;
```

### `backend/src/domains/communications/read_receipts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/read_receipts.rs`
- Size bytes / Размер в байтах: `13913`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

const EVENT_TYPE_RECORDED: &str = "mail.read_receipt.recorded";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationReadReceiptKind {
    Read,
}

impl CommunicationReadReceiptKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
        }
    }
}

impl TryFrom<&str> for CommunicationReadReceiptKind {
    type Error = CommunicationReadReceiptError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "read" => Ok(Self::Read),
            _ => Err(CommunicationReadReceiptError::Invalid("receipt_kind")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationReadReceipt {
    pub receipt_id: String,
    pub account_id: String,
    pub outbox_id: Option<String>,
    pub provider_message_id: String,
    pub recipient: String,
    pub receipt_kind: CommunicationReadReceiptKind,
    pub read_at: DateTime<Utc>,
    pub source_kind: String,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewCommunicationReadReceipt {
    pub receipt_id: Option<String>,
    pub account_id: String,
    pub provider_message_id: String,
    pub recipient: String,
    pub read_at: DateTime<Utc>,
    pub source_kind: Option<String>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Clone)]
pub struct CommunicationReadReceiptStore {
    pool: PgPool,
}

impl CommunicationReadReceiptStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        receipt: NewCommunicationReadReceipt,
    ) -> Result<CommunicationReadReceipt, CommunicationReadReceiptError> {
        let normalized = NormalizedCommunicationReadReceipt::from_new(receipt)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(
            &mut transaction,
            Some(normalized.account_id.as_str()),
        )
        .await?;
        let outbox_id = correlate_outbox(
            &mut transaction,
            &normalized.account_id,
            &normalized.provider_message_id,
        )
        .await?;
        let receipt = insert_receipt(&mut transaction, &normalized, outbox_id.as_deref()).await?;
        capture_read_receipt_observation(&mut transaction, &receipt).await?;
        let event = read_receipt_event(&receipt)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(receipt)
    }
}

async fn capture_read_receipt_observation(
    transaction: &mut Transaction<'_, Postgres>,
    receipt: &CommunicationReadReceipt,
) -> Result<(), CommunicationReadReceiptError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "COMMUNICATION_READ_RECEIPT",
            ObservationOriginKind::LocalRuntime,
            receipt.read_at,
            json!({
                "receipt_id": receipt.receipt_id,
                "account_id": receipt.account_id,
                "outbox_id": receipt.outbox_id,
                "provider_message_id": receipt.provider_message_id,
                "recipient": receipt.recipient,
                "receipt_kind": receipt.receipt_kind.as_str(),
                "read_at": receipt.read_at,
                "source_kind": receipt.source_kind,
                "provider_record_id": receipt.provider_record_id,
                "raw_record_id": receipt.raw_record_id,
                "operation": "read_receipt_recorded",
            }),
            format!("read-receipt://{}", receipt.receipt_id),
        )
        .provenance(json!({
            "captured_by": "mail.read_receipts",
            "operation": "read_receipt_recorded",
        })),
    )
    .await?;
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "read_receipt",
        receipt.receipt_id.clone(),
        "read_receipt_recorded",
        json!({
            "receipt_kind": receipt.receipt_kind.as_str(),
            "source_kind": receipt.source_kind,
        }),
        None,
    )
    .await?;
    if let Some(outbox_id) = &receipt.outbox_id {
        link_mail_entity_in_transaction(
            transaction,
            &observation.observation_id,
            "outbox_item",
            outbox_id.clone(),
            "read_receipt_observed",
            json!({
                "receipt_kind": receipt.receipt_kind.as_str(),
                "source_kind": receipt.source_kind,
            }),
            None,
        )
        .await?;
    }
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_message",
        receipt.provider_message_id.clone(),
        "read_receipt_observed",
        json!({
            "receipt_kind": receipt.receipt_kind.as_str(),
            "source_kind": receipt.source_kind,
        }),
        None,
    )
    .await?;
    Ok(())
}

#[derive(Debug)]
struct NormalizedCommunicationReadReceipt {
    receipt_id: String,
    account_id: String,
    provider_message_id: String,
    recipient: String,
    read_at: DateTime<Utc>,
    source_kind: String,
    provider_record_id: Option<String>,
    raw_record_id: Option<String>,
    metadata: Value,
}

impl NormalizedCommunicationReadReceipt {
    fn from_new(
        receipt: NewCommunicationReadReceipt,
    ) -> Result<Self, CommunicationReadReceiptError> {
        let account_id = normalize_required("account_id", &receipt.account_id)?;
        let provider_message_id =
            normalize_required("provider_message_id", &receipt.provider_message_id)?;
        let provider_record_id = normalize_optional(receipt.provider_record_id)?;
        let receipt_id = match receipt.receipt_id {
            Some(value) => normalize_required("receipt_id", &value)?,
            None => generate_receipt_id(&account_id, provider_record_id.as_deref()),
        };
        let metadata = receipt.metadata.unwrap_or_else(|| json!({}));
        if !metadata.is_object() {
            return Err(CommunicationReadReceiptError::Invalid("metadata"));
        }

        Ok(Self {
            receipt_id,
            account_id,
            provider_message_id,
            recipient: normalize_required("recipient", &receipt.recipient)?,
            read_at: receipt.read_at,
            source_kind: normalize_optional(receipt.source_kind)?
                .unwrap_or_else(|| "mdn".to_owned()),
            provider_record_id,
            raw_record_id: normalize_optional(receipt.raw_record_id)?,
            metadata,
        })
    }
}

async fn correlate_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    provider_message_id: &str,
) -> Result<Option<String>, CommunicationReadReceiptError> {
    Ok(sqlx::query_scalar::<_, String>(
        r#"
        SELECT outbox_id
        FROM communication_outbox
        WHERE account_id = $1
          AND provider_message_id = $2
        ORDER BY sent_at DESC NULLS LAST, created_at DESC, outbox_id ASC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .bind(provider_message_id)
    .fetch_optional(&mut **transaction)
    .await?)
}

async fn insert_receipt(
    transaction: &mut Transaction<'_, Postgres>,
    receipt: &NormalizedCommunicationReadReceipt,
    outbox_id: Option<&str>,
) -> Result<CommunicationReadReceipt, CommunicationReadReceiptError> {
    let row = sqlx::query(
        r#"
        INSERT INTO communication_read_receipts (
            receipt_id,
            account_id,
            outbox_id,
            provider_message_id,
            recipient,
            receipt_kind,
            read_at,
            source_kind,
            provider_record_id,
            raw_record_id,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, 'read', $6, $7, $8, $9, $10)
        RETURNING
            receipt_id,
            account_id,
            outbox_id,
            provider_message_id,
            recipient,
            receipt_kind,
            read_at,
            source_kind,
            provider_record_id,
            raw_record_id,
            metadata,
            created_at
        "#,
    )
    .bind(&receipt.receipt_id)
    .bind(&receipt.account_id)
    .bind(outbox_id)
    .bind(&receipt.provider_message_id)
    .bind(&receipt.recipient)
    .bind(receipt.read_at)
    .bind(&receipt.source_kind)
    .bind(receipt.provider_record_id.as_deref())
    .bind(receipt.raw_record_id.as_deref())
    .bind(&receipt.metadata)
    .fetch_one(&mut **transaction)
    .await?;

    row_to_receipt(row)
}

async fn ensure_canonical_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Option<&str>,
) -> Result<(), CommunicationReadReceiptError> {
    let Some(account_id) = account_id else {
        return Ok(());
    };

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

fn row_to_receipt(row: PgRow) -> Result<CommunicationReadReceipt, CommunicationReadReceiptError> {
    let receipt_kind: String = row.try_get("receipt_kind")?;
    Ok(CommunicationReadReceipt {
        receipt_id: row.try_get("receipt_id")?,
        account_id: row.try_get("account_id")?,
        outbox_id: row.try_get("outbox_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        recipient: row.try_get("recipient")?,
        receipt_kind: CommunicationReadReceiptKind::try_from(receipt_kind.as_str())?,
        read_at: row.try_get("read_at")?,
        source_kind: row.try_get("source_kind")?,
        provider_record_id: row.try_get("provider_record_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}

fn read_receipt_event(
    receipt: &CommunicationReadReceipt,
) -> Result<NewEventEnvelope, CommunicationReadReceiptError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_read_receipt_event:{}:{}",
            receipt.receipt_id,
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        EVENT_TYPE_RECORDED,
        Utc::now(),
        json!({ "kind": "mail_read_receipt_api" }),
        json!({
            "kind": "mail_read_receipt",
            "id": receipt.receipt_id,
            "account_id": receipt.account_id,
            "outbox_id": receipt.outbox_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(json!({
        "receipt_id": receipt.receipt_id,
        "account_id": receipt.account_id,
        "outbox_id": receipt.outbox_id,
        "provider_message_id": receipt.provider_message_id,
        "receipt_ki
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/rich_template.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rich_template.rs`
- Size bytes / Размер в байтах: `6841`
- Included characters / Включено символов: `6840`
- Truncated / Обрезано: `no`

```rust
// §15: Rich templates with conditional blocks, tables, polls, mail merge
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichTemplate {
    pub name: String,
    pub subject: String,
    pub blocks: Vec<TemplateBlock>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TemplateBlock {
    Text {
        content: String,
    },
    Variable {
        key: String,
        default: Option<String>,
    },
    Conditional {
        condition: Condition,
        then_blocks: Vec<TemplateBlock>,
        else_blocks: Option<Vec<TemplateBlock>>,
    },
    Table {
        headers: Vec<String>,
        row_variable: String,
        columns: Vec<String>,
    },
    Button {
        text: String,
        url_template: String,
    },
    Divider,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    pub variable: String,
    pub operator: String,
    pub value: String,
}

pub struct RichTemplateEngine;

impl RichTemplateEngine {
    pub fn render(
        template: &RichTemplate,
        vars: &HashMap<String, String>,
    ) -> Result<(String, String), RenderError> {
        let subject = Self::render_text(&template.subject, vars);
        let body = template
            .blocks
            .iter()
            .map(|b| Self::render_block(b, vars))
            .collect::<Vec<_>>()
            .join("\n");
        Ok((subject, body))
    }

    fn render_text(text: &str, vars: &HashMap<String, String>) -> String {
        let mut result = text.to_owned();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{key}}}}}"), value);
        }
        result
    }

    fn render_block(block: &TemplateBlock, vars: &HashMap<String, String>) -> String {
        match block {
            TemplateBlock::Text { content } => Self::render_text(content, vars),
            TemplateBlock::Variable { key, default } => vars
                .get(key)
                .cloned()
                .or_else(|| default.clone())
                .unwrap_or_default(),
            TemplateBlock::Conditional {
                condition,
                then_blocks,
                else_blocks,
            } => {
                let val = vars.get(&condition.variable).cloned().unwrap_or_default();
                let is_true = match condition.operator.as_str() {
                    "equals" => val == condition.value,
                    "not_empty" => !val.is_empty(),
                    "contains" => val.contains(&condition.value),
                    _ => false,
                };
                if is_true {
                    then_blocks
                        .iter()
                        .map(|b| Self::render_block(b, vars))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    else_blocks
                        .as_ref()
                        .map(|blocks| {
                            blocks
                                .iter()
                                .map(|b| Self::render_block(b, vars))
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_default()
                }
            }
            TemplateBlock::Table {
                headers,
                row_variable: _,
                columns: _,
            } => {
                let header_line = headers.join(" | ");
                let sep = headers
                    .iter()
                    .map(|_| "---")
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("{header_line}\n{sep}")
            }
            TemplateBlock::Button { text, url_template } => {
                let url = Self::render_text(url_template, vars);
                format!("[{text}]({url})")
            }
            TemplateBlock::Divider => "---".into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("template render error")]
pub struct RenderError;

#[cfg(test)]
mod tests {
    use super::*;
    fn vars() -> HashMap<String, String> {
        [
            ("name".into(), "Alice".into()),
            ("project".into(), "Макошь".into()),
        ]
        .into()
    }

    #[test]
    fn render_text_with_vars() {
        let tpl = RichTemplate {
            name: "test".into(),
            subject: "Hello {{name}}".into(),
            blocks: vec![TemplateBlock::Text {
                content: "Hi {{name}}, project {{project}}".into(),
            }],
        };
        let (s, b) = RichTemplateEngine::render(&tpl, &vars()).unwrap();
        assert_eq!(s, "Hello Alice");
        assert_eq!(b, "Hi Alice, project Макошь");
    }

    #[test]
    fn conditional_true() {
        let tpl = RichTemplate {
            name: "t".into(),
            subject: "S".into(),
            blocks: vec![TemplateBlock::Conditional {
                condition: Condition {
                    variable: "name".into(),
                    operator: "not_empty".into(),
                    value: "".into(),
                },
                then_blocks: vec![TemplateBlock::Text {
                    content: "Has name".into(),
                }],
                else_blocks: None,
            }],
        };
        let (_, b) = RichTemplateEngine::render(&tpl, &vars()).unwrap();
        assert_eq!(b, "Has name");
    }

    #[test]
    fn conditional_false() {
        let empty_vars: HashMap<String, String> = HashMap::new();
        let tpl = RichTemplate {
            name: "t".into(),
            subject: "S".into(),
            blocks: vec![TemplateBlock::Conditional {
                condition: Condition {
                    variable: "name".into(),
                    operator: "not_empty".into(),
                    value: "".into(),
                },
                then_blocks: vec![TemplateBlock::Text {
                    content: "Has name".into(),
                }],
                else_blocks: Some(vec![TemplateBlock::Text {
                    content: "No name".into(),
                }]),
            }],
        };
        let (_, b) = RichTemplateEngine::render(&tpl, &empty_vars).unwrap();
        assert_eq!(b, "No name");
    }

    #[test]
    fn button_renders_link() {
        let tpl = RichTemplate {
            name: "t".into(),
            subject: "S".into(),
            blocks: vec![TemplateBlock::Button {
                text: "Click".into(),
                url_template: "https://ex.com/{{name}}".into(),
            }],
        };
        let (_, b) = RichTemplateEngine::render(&tpl, &vars()).unwrap();
        assert_eq!(b, "[Click](https://ex.com/Alice)");
    }
}
```

### `backend/src/domains/communications/rules.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules.rs`
- Size bytes / Размер в байтах: `272`
- Included characters / Включено символов: `272`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod evaluation;
mod mode;
mod models;
mod rows;
mod store;
mod validation;

#[cfg(test)]
mod tests;

pub use errors::EmailRuleError;
pub use mode::RuleMode;
pub use models::{EmailRule, NewEmailRule, RuleAction, RuleMatchResult};
pub use store::EmailRuleStore;
```

### `backend/src/domains/communications/rules/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules/errors.rs`
- Size bytes / Размер в байтах: `198`
- Included characters / Включено символов: `198`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailRuleError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid rule: {0}")]
    InvalidRule(&'static str),
}
```

### `backend/src/domains/communications/rules/evaluation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules/evaluation.rs`
- Size bytes / Размер в байтах: `2351`
- Included characters / Включено символов: `2351`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::domains::communications::messages::ProjectedMessage;

use super::models::RuleAction;

pub(in crate::domains::communications::rules) fn evaluate_conditions(
    conditions: &Value,
    message: &ProjectedMessage,
) -> Vec<String> {
    let mut matched = Vec::new();
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if let Some(arr) = conditions.as_array() {
        for cond in arr {
            let field = cond.get("field").and_then(|v| v.as_str()).unwrap_or("");
            let operator = cond
                .get("operator")
                .and_then(|v| v.as_str())
                .unwrap_or("contains");
            let value = cond.get("value").and_then(|v| v.as_str()).unwrap_or("");

            let is_match = match (field, operator) {
                ("subject", "contains") => subject_lower.contains(&value.to_lowercase()),
                ("subject", "equals") => subject_lower == value.to_lowercase(),
                ("body", "contains") => body_lower.contains(&value.to_lowercase()),
                ("sender", "contains") => message
                    .sender
                    .to_lowercase()
                    .contains(&value.to_lowercase()),
                ("sender", "equals") => message.sender.to_lowercase() == value.to_lowercase(),
                ("has_attachment", "equals") => value == "true",
                _ => false,
            };

            if is_match {
                let label = cond
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("condition matched");
                matched.push(label.to_owned());
            }
        }
    }
    matched
}

pub(in crate::domains::communications::rules) fn parse_actions(actions: &Value) -> Vec<RuleAction> {
    let mut result = Vec::new();
    if let Some(arr) = actions.as_array() {
        for action in arr {
            if let (Some(action_type), Some(params)) = (
                action.get("action_type").and_then(|v| v.as_str()),
                action.get("params"),
            ) {
                result.push(RuleAction {
                    action_type: action_type.to_owned(),
                    params: params.clone(),
                });
            }
        }
    }
    result
}
```

### `backend/src/domains/communications/rules/mode.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules/mode.rs`
- Size bytes / Размер в байтах: `998`
- Included characters / Включено символов: `998`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
    Suggest,
    AskBeforeExecute,
    AutoExecute,
    DryRun,
}

impl RuleMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleMode::Suggest => "suggest",
            RuleMode::AskBeforeExecute => "ask_before_execute",
            RuleMode::AutoExecute => "auto_execute",
            RuleMode::DryRun => "dry_run",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "suggest" => Some(RuleMode::Suggest),
            "ask_before_execute" => Some(RuleMode::AskBeforeExecute),
            "auto_execute" => Some(RuleMode::AutoExecute),
            "dry_run" => Some(RuleMode::DryRun),
            _ => None,
        }
    }
}

pub(in crate::domains::communications::rules) fn format_mode(mode: RuleMode) -> String {
    mode.as_str().to_owned()
}
```

### `backend/src/domains/communications/rules/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules/models.rs`
- Size bytes / Размер в байтах: `1102`
- Included characters / Включено символов: `1102`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::mode::RuleMode;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailRule {
    pub rule_id: String,
    pub name: String,
    pub description_nl: String,
    pub conditions_json: Value,
    pub actions_json: Value,
    pub mode: RuleMode,
    pub enabled: bool,
    pub match_count: i64,
    pub last_matched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMatchResult {
    pub rule_id: String,
    pub matched: bool,
    pub matched_conditions: Vec<String>,
    pub suggested_actions: Vec<RuleAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: String,
    pub params: Value,
}

#[derive(Clone, Debug)]
pub struct NewEmailRule {
    pub rule_id: String,
    pub name: String,
    pub description_nl: String,
    pub conditions_json: Value,
    pub actions_json: Value,
    pub mode: RuleMode,
    pub enabled: bool,
}
```

### `backend/src/domains/communications/rules/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules/rows.rs`
- Size bytes / Размер в байтах: `1119`
- Included characters / Включено символов: `1119`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::EmailRuleError;
use super::mode::RuleMode;
use super::models::EmailRule;

pub(in crate::domains::communications::rules) const EMAIL_RULE_COLUMNS: &str = "rule_id, name, description_nl, conditions_json, actions_json, mode, enabled, match_count, \
     last_matched_at, created_at, updated_at";

pub(in crate::domains::communications::rules) fn row_to_email_rule(
    row: PgRow,
) -> Result<EmailRule, EmailRuleError> {
    let mode_str: String = row.try_get("mode")?;
    Ok(EmailRule {
        rule_id: row.try_get("rule_id")?,
        name: row.try_get("name")?,
        description_nl: row.try_get("description_nl")?,
        conditions_json: row.try_get("conditions_json")?,
        actions_json: row.try_get("actions_json")?,
        mode: RuleMode::parse(&mode_str).unwrap_or(RuleMode::Suggest),
        enabled: row.try_get("enabled")?,
        match_count: row.try_get("match_count")?,
        last_matched_at: row.try_get("last_matched_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/communications/rules/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules/store.rs`
- Size bytes / Размер в байтах: `2797`
- Included characters / Включено символов: `2797`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;

use super::errors::EmailRuleError;
use super::evaluation::{evaluate_conditions, parse_actions};
use super::mode::format_mode;
use super::models::{EmailRule, NewEmailRule, RuleMatchResult};
use super::rows::{EMAIL_RULE_COLUMNS, row_to_email_rule};

#[derive(Clone)]
pub struct EmailRuleStore {
    pool: PgPool,
}

impl EmailRuleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_rule(&self, rule: &NewEmailRule) -> Result<EmailRule, EmailRuleError> {
        rule.validate()?;
        let mode = format_mode(rule.mode);
        let sql = format!(
            "INSERT INTO communication_rules \
             (rule_id, name, description_nl, conditions_json, actions_json, mode, enabled) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (rule_id) DO UPDATE SET \
             name = EXCLUDED.name, \
             description_nl = EXCLUDED.description_nl, \
             conditions_json = EXCLUDED.conditions_json, \
             actions_json = EXCLUDED.actions_json, \
             mode = EXCLUDED.mode, \
             enabled = EXCLUDED.enabled, \
             updated_at = now() \
             RETURNING {EMAIL_RULE_COLUMNS}"
        );
        let row = sqlx::query(&sql)
            .bind(&rule.rule_id)
            .bind(&rule.name)
            .bind(&rule.description_nl)
            .bind(&rule.conditions_json)
            .bind(&rule.actions_json)
            .bind(&mode)
            .bind(rule.enabled)
            .fetch_one(&self.pool)
            .await?;
        row_to_email_rule(row)
    }

    pub async fn list_rules(&self) -> Result<Vec<EmailRule>, EmailRuleError> {
        let sql = format!(
            "SELECT {EMAIL_RULE_COLUMNS} FROM communication_rules ORDER BY created_at DESC"
        );
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_email_rule).collect()
    }

    pub async fn match_rules(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Vec<RuleMatchResult>, EmailRuleError> {
        let rules = self.list_rules().await?;
        let mut results = Vec::new();

        for rule in &rules {
            if !rule.enabled {
                continue;
            }
            let matched_conditions = evaluate_conditions(&rule.conditions_json, message);
            if !matched_conditions.is_empty() {
                results.push(RuleMatchResult {
                    rule_id: rule.rule_id.clone(),
                    matched: true,
                    matched_conditions,
                    suggested_actions: parse_actions(&rule.actions_json),
                });
            }
        }
        Ok(results)
    }
}
```

### `backend/src/domains/communications/rules/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules/tests.rs`
- Size bytes / Размер в байтах: `2614`
- Included characters / Включено символов: `2614`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;

use crate::domains::communications::messages::{
    LocalMessageState, ProjectedMessage, WorkflowState,
};

use super::evaluation::evaluate_conditions;
use super::mode::RuleMode;

fn test_message(subject: &str, sender: &str, body: &str) -> ProjectedMessage {
    ProjectedMessage {
        message_id: "msg:test:1".into(),
        raw_record_id: "raw:1".into(),
        observation_id: "observation:1".into(),
        account_id: "acct:1".into(),
        provider_record_id: "prov:1".into(),
        subject: subject.into(),
        sender: sender.into(),
        recipients: vec!["to@ex.com".into()],
        body_text: body.into(),
        occurred_at: Some(Utc::now()),
        projected_at: Utc::now(),
        channel_kind: "email".into(),
        conversation_id: None,
        sender_display_name: None,
        delivery_state: "received".into(),
        message_metadata: json!({}),
        workflow_state: WorkflowState::New,
        importance_score: None,
        ai_category: None,
        ai_summary: None,
        ai_summary_generated_at: None,
        local_state: LocalMessageState::Active,
        local_state_changed_at: None,
        local_state_reason: None,
    }
}

#[test]
fn evaluate_conditions_matches_subject() {
    let msg = test_message("Urgent: Project Update", "alice@ex.com", "Body text");
    let conditions = json!([
        {"field": "subject", "operator": "contains", "value": "urgent", "label": "urgent subject"}
    ]);
    let matched = evaluate_conditions(&conditions, &msg);
    assert_eq!(matched, vec!["urgent subject"]);
}

#[test]
fn evaluate_conditions_matches_sender() {
    let msg = test_message("Hello", "bob@company.com", "Body");
    let conditions = json!([
        {"field": "sender", "operator": "contains", "value": "company.com", "label": "company sender"}
    ]);
    let matched = evaluate_conditions(&conditions, &msg);
    assert_eq!(matched, vec!["company sender"]);
}

#[test]
fn evaluate_conditions_no_match() {
    let msg = test_message("Regular", "alice@ex.com", "Nothing special");
    let conditions = json!([
        {"field": "subject", "operator": "contains", "value": "urgent", "label": "urgent"}
    ]);
    let matched = evaluate_conditions(&conditions, &msg);
    assert!(matched.is_empty());
}

#[test]
fn rule_mode_parse_all() {
    assert_eq!(RuleMode::parse("suggest"), Some(RuleMode::Suggest));
    assert_eq!(RuleMode::parse("auto_execute"), Some(RuleMode::AutoExecute));
    assert_eq!(RuleMode::parse("dry_run"), Some(RuleMode::DryRun));
    assert_eq!(RuleMode::parse("invalid"), None);
}
```

### `backend/src/domains/communications/rules/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/rules/validation.rs`
- Size bytes / Размер в байтах: `735`
- Included characters / Включено символов: `735`
- Truncated / Обрезано: `no`

```rust
use super::errors::EmailRuleError;
use super::models::NewEmailRule;

impl NewEmailRule {
    pub(in crate::domains::communications::rules) fn validate(&self) -> Result<(), EmailRuleError> {
        if self.rule_id.trim().is_empty() {
            return Err(EmailRuleError::InvalidRule("rule_id is empty"));
        }
        if self.name.trim().is_empty() {
            return Err(EmailRuleError::InvalidRule("name is empty"));
        }
        if !self.conditions_json.is_array() {
            return Err(EmailRuleError::InvalidRule("conditions must be an array"));
        }
        if !self.actions_json.is_array() {
            return Err(EmailRuleError::InvalidRule("actions must be an array"));
        }
        Ok(())
    }
}
```

### `backend/src/domains/communications/saved_search_counts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/saved_search_counts.rs`
- Size bytes / Размер в байтах: `2996`
- Included characters / Включено символов: `2996`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashMap;

use sqlx::postgres::PgPool;
use sqlx::{Postgres, QueryBuilder, Row};

use crate::domains::communications::messages::parse_communication_message_search_query;
use crate::domains::communications::messages::{MessageSearchQuery, append_message_search_filter};
use crate::domains::communications::saved_searches::{
    CommunicationSavedSearchError, SavedSearchRecord,
};

pub(crate) async fn count_messages_for_saved_search<'e, E>(
    executor: E,
    record: &SavedSearchRecord,
) -> Result<i64, CommunicationSavedSearchError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let mut builder = QueryBuilder::<Postgres>::new(
        "SELECT count(*)::BIGINT AS message_count FROM communication_messages m WHERE 1 = 1",
    );
    append_saved_search_filters(&mut builder, record);
    let row = builder.build().fetch_one(executor).await?;
    Ok(row.try_get::<i64, _>("message_count")?)
}

pub(crate) async fn load_message_counts_for_saved_searches(
    pool: &PgPool,
    records: &[SavedSearchRecord],
) -> Result<HashMap<String, i64>, CommunicationSavedSearchError> {
    if records.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<Postgres>::new("");
    for (index, record) in records.iter().enumerate() {
        if index > 0 {
            builder.push(" UNION ALL ");
        }
        builder.push("SELECT ");
        builder.push_bind(record.saved_search_id.clone());
        builder.push(
            " AS saved_search_id, count(*)::BIGINT AS message_count FROM communication_messages m WHERE 1 = 1",
        );
        append_saved_search_filters(&mut builder, record);
    }
    let rows = builder.build().fetch_all(pool).await?;

    let mut counts = HashMap::new();
    for row in rows {
        counts.insert(
            row.try_get::<String, _>("saved_search_id")?,
            row.try_get::<i64, _>("message_count")?,
        );
    }

    Ok(counts)
}

fn append_saved_search_filters<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    record: &SavedSearchRecord,
) {
    if let Some(account_id) = record.account_id.as_deref() {
        builder.push(" AND m.account_id = ");
        builder.push_bind(account_id.to_owned());
    }
    if let Some(workflow_state) = record.workflow_state {
        builder.push(" AND m.workflow_state = ");
        builder.push_bind(workflow_state.as_str().to_owned());
    }
    if let Some(channel_kind) = record.channel_kind.as_deref() {
        builder.push(" AND m.channel_kind = ");
        builder.push_bind(channel_kind.to_owned());
    }
    if let Some(local_state) = record.local_state.persisted_filter() {
        builder.push(" AND m.local_state = ");
        builder.push_bind(local_state.to_owned());
    }
    append_message_search_filter(builder, "m", &parsed_search_query(record));
}

fn parsed_search_query(record: &SavedSearchRecord) -> MessageSearchQuery {
    parse_communication_message_search_query(Some(&record.query))
}
```
