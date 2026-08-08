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

- Chunk ID / ID чанка: `069-source-backend-part-049`
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

### `backend/src/platform/calls/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/calls/rows.rs`
- Size bytes / Размер в байтах: `1609`
- Included characters / Включено символов: `1609`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::{CallError, CallTranscript, TelegramCall};

pub(super) fn row_to_call(row: PgRow) -> Result<TelegramCall, CallError> {
    Ok(TelegramCall {
        call_id: row.try_get("call_id")?,
        account_id: row.try_get("account_id")?,
        provider_call_id: row.try_get("provider_call_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        direction: row.try_get("direction")?,
        call_state: row.try_get("call_state")?,
        started_at: row.try_get("started_at")?,
        ended_at: row.try_get("ended_at")?,
        transcription_policy_id: row.try_get("transcription_policy_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_transcript(row: PgRow) -> Result<CallTranscript, CallError> {
    Ok(CallTranscript {
        transcript_id: row.try_get("transcript_id")?,
        call_id: row.try_get("call_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        transcript_status: row.try_get("transcript_status")?,
        stt_provider: row.try_get("stt_provider")?,
        source_audio_ref: row.try_get("source_audio_ref")?,
        language_code: row.try_get("language_code")?,
        transcript_text: row.try_get("transcript_text")?,
        segments: row.try_get("segments")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/platform/calls/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/calls/store.rs`
- Size bytes / Размер в байтах: `9737`
- Included characters / Включено символов: `9737`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::rows::{row_to_call, row_to_transcript};
use super::validation::{validate_limit, validate_non_empty};
use super::{CallError, CallTranscript, NewCallTranscript, NewTelegramCall, TelegramCall};

#[derive(Clone)]
pub struct CallIntelligenceStore {
    pool: PgPool,
}

impl CallIntelligenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_call(&self, call: &NewTelegramCall) -> Result<TelegramCall, CallError> {
        call.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO telegram_calls (
                call_id,
                account_id,
                provider_call_id,
                provider_chat_id,
                direction,
                call_state,
                started_at,
                ended_at,
                transcription_policy_id,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            ON CONFLICT (account_id, provider_call_id)
            DO UPDATE SET
                provider_chat_id = EXCLUDED.provider_chat_id,
                direction = EXCLUDED.direction,
                call_state = EXCLUDED.call_state,
                started_at = EXCLUDED.started_at,
                ended_at = EXCLUDED.ended_at,
                transcription_policy_id = EXCLUDED.transcription_policy_id,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                call_id,
                account_id,
                provider_call_id,
                provider_chat_id,
                direction,
                call_state,
                started_at,
                ended_at,
                transcription_policy_id,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(call.call_id.trim())
        .bind(call.account_id.trim())
        .bind(call.provider_call_id.trim())
        .bind(call.provider_chat_id.trim())
        .bind(call.direction.as_str())
        .bind(call.call_state.as_str())
        .bind(call.started_at)
        .bind(call.ended_at)
        .bind(call.transcription_policy_id.as_deref())
        .bind(&call.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_call(row)
    }

    pub async fn upsert_transcript(
        &self,
        transcript: &NewCallTranscript,
    ) -> Result<CallTranscript, CallError> {
        transcript.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO call_transcripts (
                transcript_id,
                call_id,
                account_id,
                provider_chat_id,
                transcript_status,
                stt_provider,
                source_audio_ref,
                language_code,
                transcript_text,
                segments,
                provenance,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
            ON CONFLICT (transcript_id)
            DO UPDATE SET
                transcript_status = EXCLUDED.transcript_status,
                stt_provider = EXCLUDED.stt_provider,
                source_audio_ref = EXCLUDED.source_audio_ref,
                language_code = EXCLUDED.language_code,
                transcript_text = EXCLUDED.transcript_text,
                segments = EXCLUDED.segments,
                provenance = EXCLUDED.provenance,
                updated_at = now()
            RETURNING
                transcript_id,
                call_id,
                account_id,
                provider_chat_id,
                transcript_status,
                stt_provider,
                source_audio_ref,
                language_code,
                transcript_text,
                segments,
                provenance,
                created_at,
                updated_at
            "#,
        )
        .bind(transcript.transcript_id.trim())
        .bind(transcript.call_id.trim())
        .bind(transcript.account_id.trim())
        .bind(transcript.provider_chat_id.trim())
        .bind(transcript.transcript_status.as_str())
        .bind(transcript.stt_provider.trim())
        .bind(transcript.source_audio_ref.as_deref())
        .bind(transcript.language_code.as_deref())
        .bind(&transcript.transcript_text)
        .bind(&transcript.segments)
        .bind(&transcript.provenance)
        .fetch_one(&self.pool)
        .await?;

        row_to_transcript(row)
    }

    pub async fn list_calls(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        provider: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramCall>, CallError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let provider = provider.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                call_id,
                account_id,
                provider_call_id,
                provider_chat_id,
                direction,
                call_state,
                started_at,
                ended_at,
                transcription_policy_id,
                metadata,
                created_at,
                updated_at
            FROM telegram_calls
            WHERE ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR provider_chat_id = $2)
              AND ($3::text IS NULL OR metadata ->> 'provider' = $3)
            ORDER BY COALESCE(started_at, created_at) DESC, call_id ASC
            LIMIT $4
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(provider)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_call).collect()
    }

    pub async fn transcript_for_call(
        &self,
        call_id: &str,
    ) -> Result<Option<CallTranscript>, CallError> {
        validate_non_empty("call_id", call_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                transcript_id,
                call_id,
                account_id,
                provider_chat_id,
                transcript_status,
                stt_provider,
                source_audio_ref,
                language_code,
                transcript_text,
                segments,
                provenance,
                created_at,
                updated_at
            FROM call_transcripts
            WHERE call_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(call_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_transcript).transpose()
    }

    pub async fn list_expired_transcripts(
        &self,
        account_id: &str,
        provider: &str,
        limit: i64,
    ) -> Result<Vec<CallTranscript>, CallError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("provider", provider)?;
        let limit = validate_limit(limit.clamp(1, 500))?;
        let rows = sqlx::query(
            r#"
            SELECT
                transcript.transcript_id,
                transcript.call_id,
                transcript.account_id,
                transcript.provider_chat_id,
                transcript.transcript_status,
                transcript.stt_provider,
                transcript.source_audio_ref,
                transcript.language_code,
                transcript.transcript_text,
                transcript.segments,
                transcript.provenance,
                transcript.created_at,
                transcript.updated_at
            FROM call_transcripts transcript
            JOIN telegram_calls call_evidence ON call_evidence.call_id = transcript.call_id
            WHERE transcript.account_id = $1
              AND call_evidence.metadata ->> 'provider' = $2
              AND NULLIF(transcript.provenance -> 'retention_policy' ->> 'expires_at', '') IS NOT NULL
              AND (transcript.provenance -> 'retention_policy' ->> 'expires_at')::timestamptz <= now()
            ORDER BY (transcript.provenance -> 'retention_policy' ->> 'expires_at')::timestamptz ASC,
                     transcript.created_at ASC
            LIMIT $3
            "#,
        )
        .bind(account_id.trim())
        .bind(provider.trim())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_transcript).collect()
    }

    pub async fn remove_transcript(
        &self,
        transcript_id: &str,
    ) -> Result<Option<CallTranscript>, CallError> {
        validate_non_empty("transcript_id", transcript_id)?;
        let row = sqlx::query(
            r#"
            DELETE FROM call_transcripts
            WHERE transcript_id = $1
            RETURNING
                transcript_id,
                call_id,
                account_id,
                provider_chat_id,
                transcript_status,
                stt_provider,
                source_audio_ref,
                language_code,
                transcript_text,
                segments,
                provenance,
                created_at,
                updated_at
            "#,
        )
        .bind(transcript_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_transcript).transpose()
    }
}
```

### `backend/src/platform/calls/stt.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/calls/stt.rs`
- Size bytes / Размер в байтах: `1113`
- Included characters / Включено символов: `1113`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::CallError;
use super::validation::validate_non_empty;

pub trait SpeechToTextProvider {
    fn provider_name(&self) -> &'static str;
    fn transcribe_fixture(&self, audio_ref: &str) -> Result<FixtureTranscript, CallError>;
}

pub struct FixtureSpeechToTextProvider;

impl SpeechToTextProvider for FixtureSpeechToTextProvider {
    fn provider_name(&self) -> &'static str {
        "fixture-stt"
    }

    fn transcribe_fixture(&self, audio_ref: &str) -> Result<FixtureTranscript, CallError> {
        validate_non_empty("audio_ref", audio_ref)?;
        Ok(FixtureTranscript {
            text: format!("Fixture transcript for {audio_ref}: follow up on the Telegram call."),
            segments: json!([
                {
                    "speaker": "local",
                    "start_ms": 0,
                    "end_ms": 2400,
                    "text": "follow up on the Telegram call"
                }
            ]),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureTranscript {
    pub text: String,
    pub segments: Value,
}
```

### `backend/src/platform/calls/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/calls/validation.rs`
- Size bytes / Размер в байтах: `1093`
- Included characters / Включено символов: `1093`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::CallError;

pub(super) fn validate_limit(limit: i64) -> Result<i64, CallError> {
    if !(1..=100).contains(&limit) {
        return Err(CallError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<String, CallError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn validate_object(field: &'static str, value: &Value) -> Result<(), CallError> {
    if !value.is_object() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub(super) fn validate_array(field: &'static str, value: &Value) -> Result<(), CallError> {
    if !value.is_array() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must be a JSON array"
        )));
    }
    Ok(())
}
```

### `backend/src/platform/capabilities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/capabilities.rs`
- Size bytes / Размер в байтах: `3371`
- Included characters / Включено символов: `3371`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CapabilityActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    Export,
    SecretAccess,
    Automation,
}

impl CapabilityActionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::LocalWrite => "local_write",
            Self::ProviderWrite => "provider_write",
            Self::Destructive => "destructive",
            Self::Export => "export",
            Self::SecretAccess => "secret_access",
            Self::Automation => "automation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CapabilityDecisionStatus {
    Allowed,
    Rejected,
}

impl CapabilityDecisionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityDecision {
    action_class: CapabilityActionClass,
    capability: String,
    decision: CapabilityDecisionStatus,
    reason: String,
    confirmation_required: bool,
    scoped_automation_policy: bool,
    automation_policy_id: Option<String>,
}

impl CapabilityDecision {
    pub fn explicit_user_allowed(
        action_class: CapabilityActionClass,
        capability: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            action_class,
            capability: capability.into(),
            decision: CapabilityDecisionStatus::Allowed,
            reason: reason.into(),
            confirmation_required: false,
            scoped_automation_policy: false,
            automation_policy_id: None,
        }
    }

    pub fn scoped_automation_allowed(
        capability: impl Into<String>,
        automation_policy_id: impl Into<String>,
    ) -> Self {
        Self {
            action_class: CapabilityActionClass::Automation,
            capability: capability.into(),
            decision: CapabilityDecisionStatus::Allowed,
            reason: "scoped_automation_policy_authorized".to_owned(),
            confirmation_required: false,
            scoped_automation_policy: true,
            automation_policy_id: Some(automation_policy_id.into()),
        }
    }

    pub fn rejected_high_risk(
        action_class: CapabilityActionClass,
        capability: impl Into<String>,
        reason: impl Into<String>,
        automation_policy_id: Option<String>,
    ) -> Self {
        Self {
            action_class,
            capability: capability.into(),
            decision: CapabilityDecisionStatus::Rejected,
            reason: reason.into(),
            confirmation_required: true,
            scoped_automation_policy: false,
            automation_policy_id,
        }
    }

    pub fn audit_metadata(&self) -> Value {
        json!({
            "action_class": self.action_class.as_str(),
            "capability": self.capability,
            "decision": self.decision.as_str(),
            "reason": self.reason,
            "confirmation_required": self.confirmation_required,
            "scoped_automation_policy": self.scoped_automation_policy,
            "automation_policy_id": self.automation_policy_id,
        })
    }
}
```

### `backend/src/platform/communications.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications.rs`
- Size bytes / Размер в байтах: `41725`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::platform::observations::{ObservationOriginKind, ObservationStoreError};
use crate::platform::secrets::{ResolvedSecret, SecretKind, SecretReference};

mod email_sync;
mod raw_signals;
pub mod rfc822;

pub use email_sync::{EmailSyncPlanError, imap_mailbox_stream_id, plan_email_sync};
pub use raw_signals::{CommunicationRawSignalSource, build_communication_raw_signal_event};

pub const DEFAULT_MAIL_SYNC_BLOB_ROOT: &str = "docker/data/mail";

#[derive(Debug, Error)]
pub enum CommunicationContractError {
    #[error("unsupported communication provider kind: {0}")]
    UnsupportedProviderKind(String),

    #[error("unsupported provider account secret purpose: {0}")]
    UnsupportedSecretPurpose(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationProviderKind {
    Gmail,
    Icloud,
    Imap,
    TelegramUser,
    TelegramBot,
    WhatsappWeb,
    WhatsappBusinessCloud,
    ZoomUser,
    ZoomServerToServer,
    YandexTelemostUser,
}

pub type EmailProviderKind = CommunicationProviderKind;

impl CommunicationProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gmail => "gmail",
            Self::Icloud => "icloud",
            Self::Imap => "imap",
            Self::TelegramUser => "telegram_user",
            Self::TelegramBot => "telegram_bot",
            Self::WhatsappWeb => "whatsapp_web",
            Self::WhatsappBusinessCloud => "whatsapp_business_cloud",
            Self::ZoomUser => "zoom_user",
            Self::ZoomServerToServer => "zoom_server_to_server",
            Self::YandexTelemostUser => "yandex_telemost_user",
        }
    }

    pub fn is_email(self) -> bool {
        matches!(self, Self::Gmail | Self::Icloud | Self::Imap)
    }

    pub fn is_telegram(self) -> bool {
        matches!(self, Self::TelegramUser | Self::TelegramBot)
    }

    pub fn is_whatsapp(self) -> bool {
        matches!(self, Self::WhatsappWeb | Self::WhatsappBusinessCloud)
    }

    pub fn is_zoom(self) -> bool {
        matches!(self, Self::ZoomUser | Self::ZoomServerToServer)
    }

    pub fn is_yandex_telemost(self) -> bool {
        matches!(self, Self::YandexTelemostUser)
    }
}

impl TryFrom<&str> for CommunicationProviderKind {
    type Error = CommunicationContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "gmail" => Ok(Self::Gmail),
            "icloud" => Ok(Self::Icloud),
            "imap" => Ok(Self::Imap),
            "telegram_user" => Ok(Self::TelegramUser),
            "telegram_bot" => Ok(Self::TelegramBot),
            "whatsapp_web" => Ok(Self::WhatsappWeb),
            "whatsapp_business_cloud" => Ok(Self::WhatsappBusinessCloud),
            "zoom_user" => Ok(Self::ZoomUser),
            "zoom_server_to_server" => Ok(Self::ZoomServerToServer),
            "yandex_telemost_user" => Ok(Self::YandexTelemostUser),
            other => Err(CommunicationContractError::UnsupportedProviderKind(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccount {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccountUsage {
    pub raw_record_count: i64,
    pub message_count: i64,
    pub checkpoint_count: i64,
}

impl ProviderAccountUsage {
    pub fn has_retained_evidence(&self) -> bool {
        self.raw_record_count > 0 || self.message_count > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderChannelMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub conversation_id: String,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub message_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMessageAttachmentAnchor {
    pub message_id: String,
    pub raw_record_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMessageReferenceSummary {
    pub message_id: String,
    pub provider_record_id: String,
    pub conversation_id: Option<String>,
    pub subject: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderHeuristicMember {
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub message_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy)]
pub struct ProviderMessageProjectionObservationContext<'a> {
    pub channel_kinds: &'a [&'a str],
    pub relationship_kind: &'a str,
    pub actor: &'a str,
}

pub type ProviderChannelMessagePortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderCommunicationMessagePortError>> + Send + 'a>>;

pub struct ProviderAttachmentDownloadStateUpdate<'a> {
    pub message_id: &'a str,
    pub provider_attachment_id: &'a str,
    pub provider_file_id: i64,
    pub download_state: &'a str,
    pub local_path: Option<&'a str>,
    pub size_bytes: Option<i64>,
    pub content_type: &'a str,
    pub filename: Option<&'a str>,
    pub observed_at: DateTime<Utc>,
    pub context: ProviderMessageProjectionObservationContext<'a>,
}

pub struct ProviderMessageObservationEvent<'a> {
    pub provider: &'a str,
    pub account_id: &'a str,
    pub channel_kind: &'a str,
    pub message_id: &'a str,
    pub external_message_id: &'a str,
    pub event_kind: &'a str,
    pub observed_at: DateTime<Utc>,
    pub external_event_id: Option<&'a str>,
    pub payload: &'a Value,
    pub causation_id: Option<&'a str>,
    pub correlation_id: Option<&'a str>,
}

pub type ProviderMessageObservationEventFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<Option<i64>, ProviderCommunicationMessagePortError>> + Send + 'a,
    >,
>;

pub type CommunicationRawRecordPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderCommunicationMessagePortError>> + Send + 'a>>;

pub trait CommunicationRawRecordCommandPort: Send + Sync {
    fn record_raw_source<'a>(
        &'a self,
        record: &'a NewRawCommunicationRecord,
    ) -> CommunicationRawRecordPortFuture<'a, StoredRawCommunicationRecord>;
}

pub trait ProviderMessageObservationEventPort: Send + Sync {
    fn append_provider_message_observation<'a>(
        &'a self,
        observation: ProviderMessageObservationEvent<'a>,
    ) -> ProviderMessageObservationEventFuture<'a>;
}

#[derive(Clone)]
pub struct EventStoreProviderMessageObservationEventPort {
    event_store: crate::platform::events::EventStore,
}

impl EventStoreProviderMessageObservationEventPort {
    pub fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self {
            event_store: crate::platform::events::EventStore::new(pool),
        }
    }
}

impl ProviderMessageObservationEventPort for EventStoreProviderMessageObservationEventPort {
    fn append_provider_message_observation<'a>(
        &'a self,
        observation: ProviderMessageObservationEvent<'a>,
    ) -> ProviderMessageObservationEventFuture<'a> {
        Box::pin(async move {
            validate_provider_observation_event(&observation).map_err(|error| {
                ProviderCommunicationMessagePortError::InvalidRequest(error.to_string())
            })?;
            let payload_hash = sha256_json(observation.payload)?;
            let idempotency_key = provider_observation_idempotency_key(
                observation.provider,
                observation.account_id,
                observation.event_kind,
                observation.external_message_id,
                observation.external_event_id,
                &payload_hash,
            );
            let event_type =
                provider_observation_event_type(observation.provider, observation.event_kind);
            let builder = crate::platform::events::NewEventEnvelope::builder(
                format!(
                    "evt_provider_observation_{}",
                    stable_event_id_fragment(&idempotency_key)
                ),
                event_type,
                observation.observed_at,
                json!({
                    "kind": "provider_observation",
                    "provider": observation.provider,
                    "account_id": observation.account_id,
                    "source_id": idempotency_key,
                }),
                json!({
                    "kind": "provider_message",
                    "provider": observation.provider,
                    "id": observation.external_message_id,
                    "message_id": observation.message_id,
                }),
            )
            .payload(json!({
                "provider_kind": observation.channel_kind,
                "account_id": observation.account_id,
                "external_event_id": observation.external_event_id,
                "external_message_id": observation.external_message_id,
                "message_id": observation.message_id,
                "event_kind": observation.event_kind,
                "observed_at": observation.observed_at,
                "payload_hash": payload_hash,
                "payload": observation.payload,
            }))
            .provenance(json!({
                "provider": observation.provider,
                "ownership": "provider_observation_fact",
            }));
            let correlation_id = observation
                .correlation_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(&idempotency_key);
            let mut builder = builder.correlation_id(correlation_id);
            if let Some(causation_id) = observation.causation_id {
                builder = builder.causation_id(causation_id);
            }
            let event = builder.build()?;

            self.event_store
                .append_for_dispatch_idempotent(&event)
                .await
                .map_err(Into::into)
        })
    }
}

#[derive(Debug, Error)]
pub enum ProviderCommunicationMessagePortError {
    #[error("invalid provider communication message request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] crate::platform::events::EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeletedProviderAccount {
    pub account: Option<ProviderAccount>,
    pub unbound_secret_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProviderAccount {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
}

impl NewProviderAccount {
    pub fn new(
        account_id: impl Into<String>,
        provider_kind: C
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/platform/communications/email_sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/email_sync.rs`
- Size bytes / Размер в байтах: `7373`
- Included characters / Включено символов: `7373`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use thiserror::Error;

use super::{
    EmailProviderKind, EmailSyncAdapterConfig, EmailSyncPlan, ProviderAccount,
    ProviderAccountSecretPurpose,
};

#[derive(Debug, Error)]
pub enum EmailSyncPlanError {
    #[error("invalid provider config field {field}: {message}")]
    InvalidProviderConfig {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider account config must not contain secret-like key: {key}")]
    SecretLikeConfigKey { key: String },
}

pub fn plan_email_sync(account: &ProviderAccount) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let account_id = validate_non_empty("account_id", &account.account_id)?;
    reject_secret_like_config_keys(&account.config)?;

    match account.provider_kind {
        EmailProviderKind::Gmail => plan_gmail_sync(account, account_id),
        EmailProviderKind::Icloud | EmailProviderKind::Imap => plan_imap_sync(account, account_id),
        EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb
        | EmailProviderKind::WhatsappBusinessCloud
        | EmailProviderKind::ZoomUser
        | EmailProviderKind::ZoomServerToServer
        | EmailProviderKind::YandexTelemostUser => Err(EmailSyncPlanError::InvalidProviderConfig {
            field: "provider_kind",
            message: "email sync supports only gmail, icloud or imap",
        }),
    }
}

pub fn imap_mailbox_stream_id(mailbox: &str) -> String {
    let mut stream_id = String::from("imap:");

    for character in mailbox.chars() {
        match character {
            '%' => stream_id.push_str("%25"),
            ':' => stream_id.push_str("%3A"),
            _ => stream_id.push(character),
        }
    }

    stream_id
}

fn plan_gmail_sync(
    account: &ProviderAccount,
    account_id: String,
) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let history_stream_id = optional_string(&account.config, "history_stream_id")?
        .unwrap_or_else(|| "gmail:history".to_owned());
    validate_non_empty("history_stream_id", &history_stream_id)?;
    validate_no_control_chars("history_stream_id", &history_stream_id)?;

    Ok(EmailSyncPlan {
        account_id,
        provider_kind: account.provider_kind,
        credential_purpose: ProviderAccountSecretPurpose::OauthToken,
        stream_id: history_stream_id.clone(),
        adapter_config: EmailSyncAdapterConfig::Gmail { history_stream_id },
    })
}

fn plan_imap_sync(
    account: &ProviderAccount,
    account_id: String,
) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let host = required_string(&account.config, "host")?;
    let port = required_port(&account.config, "port")?;
    let tls = required_bool(&account.config, "tls")?;
    let mailbox =
        optional_string(&account.config, "mailbox")?.unwrap_or_else(|| "INBOX".to_owned());
    validate_non_empty("mailbox", &mailbox)?;
    validate_no_control_chars("mailbox", &mailbox)?;
    let stream_id = imap_mailbox_stream_id(&mailbox);

    Ok(EmailSyncPlan {
        account_id,
        provider_kind: account.provider_kind,
        credential_purpose: ProviderAccountSecretPurpose::ImapPassword,
        stream_id,
        adapter_config: EmailSyncAdapterConfig::Imap {
            host,
            port,
            tls,
            mailbox,
        },
    })
}

fn required_string(config: &Value, field: &'static str) -> Result<String, EmailSyncPlanError> {
    let Some(value) = optional_string(config, field)? else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "missing string value",
        });
    };
    validate_non_empty(field, &value)
}

fn optional_string(
    config: &Value,
    field: &'static str,
) -> Result<Option<String>, EmailSyncPlanError> {
    let Some(value) = config.get(field) else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected string value",
        });
    };

    Ok(Some(value.trim().to_owned()))
}

fn required_port(config: &Value, field: &'static str) -> Result<u16, EmailSyncPlanError> {
    let Some(value) = config.get(field).and_then(Value::as_u64) else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected integer port",
        });
    };
    let Ok(port) = u16::try_from(value) else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "port must fit u16",
        });
    };
    if port == 0 {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "port must be greater than zero",
        });
    }

    Ok(port)
}

fn required_bool(config: &Value, field: &'static str) -> Result<bool, EmailSyncPlanError> {
    config
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected boolean value",
        })
}

fn reject_secret_like_config_keys(config: &Value) -> Result<(), EmailSyncPlanError> {
    let Some(object) = config.as_object() else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field: "config",
            message: "expected object",
        });
    };

    for (key, value) in object {
        let key_path = key.clone();
        reject_secret_like_config_key(key, &key_path)?;
        reject_secret_like_config_value(value, &key_path)?;
    }

    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, EmailSyncPlanError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "must not be empty",
        });
    }

    Ok(value.to_owned())
}

fn validate_no_control_chars(field: &'static str, value: &str) -> Result<(), EmailSyncPlanError> {
    if value.chars().any(char::is_control) {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "must not contain control characters",
        });
    }

    Ok(())
}

fn reject_secret_like_config_value(value: &Value, path: &str) -> Result<(), EmailSyncPlanError> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let key_path = format!("{path}.{key}");
                reject_secret_like_config_key(key, &key_path)?;
                reject_secret_like_config_value(value, &key_path)?;
            }
            Ok(())
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                reject_secret_like_config_value(value, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_secret_like_config_key(key: &str, key_path: &str) -> Result<(), EmailSyncPlanError> {
    let normalized = key.to_ascii_lowercase();
    if normalized.contains("password")
        || normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("credential")
    {
        return Err(EmailSyncPlanError::SecretLikeConfigKey {
            key: key_path.to_owned(),
        });
    }

    Ok(())
}
```

### `backend/src/platform/communications/raw_signals.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/raw_signals.rs`
- Size bytes / Размер в байтах: `3164`
- Included characters / Включено символов: `3164`
- Truncated / Обрезано: `no`

```rust
use std::path::Path;

use serde_json::json;
use sha2::{Digest, Sha256};

use super::StoredRawCommunicationRecord;
use crate::platform::events::NewEventEnvelope;
use crate::platform::observations::observation_captured_event_id;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRawSignalSource {
    Mail,
    Telegram,
    Whatsapp,
}

impl CommunicationRawSignalSource {
    fn source_code(self) -> &'static str {
        match self {
            Self::Mail => "mail",
            Self::Telegram => "telegram",
            Self::Whatsapp => "whatsapp",
        }
    }

    fn event_type(self) -> &'static str {
        match self {
            Self::Mail => "signal.raw.mail.message.observed",
            Self::Telegram => "signal.raw.telegram.message.observed",
            Self::Whatsapp => "signal.raw.whatsapp.message.observed",
        }
    }

    fn event_id_prefix(self) -> &'static str {
        match self {
            Self::Mail => "mail",
            Self::Telegram => "telegram",
            Self::Whatsapp => "whatsapp",
        }
    }
}

pub fn build_communication_raw_signal_event(
    source: CommunicationRawSignalSource,
    raw_record: &StoredRawCommunicationRecord,
    raw_blob_root: Option<&Path>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    let occurred_at = raw_record.occurred_at.unwrap_or(raw_record.captured_at);
    let source_code = source.source_code();
    let mut provenance = json!({
        "source": "communications_raw_record",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
        "import_batch_id": raw_record.import_batch_id,
        "raw_record_provenance": raw_record.provenance,
    });
    if let Some(root) = raw_blob_root.and_then(Path::to_str) {
        provenance["blob_root"] = json!(root);
    }

    NewEventEnvelope::builder(
        raw_signal_event_id(source, &raw_record.raw_record_id),
        source.event_type(),
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": source_code,
            "source_id": raw_record.raw_record_id,
            "account_id": raw_record.account_id,
        }),
        json!({
            "kind": "communication_raw_record",
            "source_code": source_code,
            "raw_record_id": raw_record.raw_record_id,
            "account_id": raw_record.account_id,
            "provider_record_id": raw_record.provider_record_id,
            "record_kind": raw_record.record_kind,
        }),
    )
    .payload(raw_record.payload.clone())
    .provenance(provenance)
    .causation_id(observation_captured_event_id(&raw_record.observation_id))
    .correlation_id(raw_record.observation_id.clone())
    .build()
}

fn raw_signal_event_id(source: CommunicationRawSignalSource, raw_record_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_record_id.as_bytes());
    format!(
        "evt_signal_raw_{}_{:x}",
        source.event_id_prefix(),
        hasher.finalize()
    )
}
```

### `backend/src/platform/communications/rfc822.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822.rs`
- Size bytes / Размер в байтах: `302`
- Included characters / Включено символов: `302`
- Truncated / Обрезано: `no`

```rust
mod body;
mod decoding;
mod errors;
mod headers;
mod models;
mod multipart;
mod parser;
mod util;
mod wire;

pub use errors::EmailRfc822ParseError;
pub use models::{
    ParsedCommunicationSourceMessage, ParsedEmailAttachment, ParsedEmailAttachmentDisposition,
};
pub use parser::parse_rfc822_message;
```

### `backend/src/platform/communications/rfc822/body.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/body.rs`
- Size bytes / Размер в байтах: `5558`
- Included characters / Включено символов: `5558`
- Truncated / Обрезано: `no`

```rust
use super::decoding::{decode_transfer_body, decode_transfer_bytes};
use super::headers::{header_media_type, header_parameter, header_value};
use super::models::{ParsedEmailAttachment, ParsedEmailAttachmentDisposition};
use super::multipart::multipart_parts;
use super::util::non_empty_or_default;

#[derive(Default)]
pub(crate) struct ParsedEmailBodyContent {
    pub(crate) body_text: Option<String>,
    pub(crate) body_html: Option<String>,
    pub(crate) attachments: Vec<ParsedEmailAttachment>,
    next_attachment_index: usize,
}

pub(crate) fn body_content_from_part(
    headers: &[(String, String)],
    body: &[u8],
) -> ParsedEmailBodyContent {
    let mut content = ParsedEmailBodyContent::default();
    collect_part_content(headers, body, &mut content);
    content
}

fn collect_part_content(
    headers: &[(String, String)],
    body: &[u8],
    content: &mut ParsedEmailBodyContent,
) {
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    let content_type_media_type = header_media_type(&content_type);

    if content_type_media_type.starts_with("multipart/") {
        if let Some(boundary) = header_parameter(&content_type, "boundary") {
            for (part_headers, part_body) in multipart_parts(&boundary, body) {
                collect_part_content(&part_headers, part_body, content);
            }
        }
        return;
    }

    if is_attachment_like_part(headers, &content_type) {
        content.next_attachment_index += 1;
        let provider_attachment_id = format!("part-{}", content.next_attachment_index);
        content.attachments.push(parsed_attachment_from_part(
            headers,
            body,
            &content_type,
            provider_attachment_id,
        ));
        return;
    }

    let charset = header_parameter(&content_type, "charset");
    let decoded = decode_transfer_body(
        body,
        header_value(headers, "content-transfer-encoding")
            .unwrap_or_default()
            .as_str(),
        charset.as_deref(),
    );
    if content_type_media_type == "text/html" {
        if content.body_html.is_none() {
            content.body_html = non_empty_html_body(&decoded);
        }
        if content.body_text.is_none() {
            content.body_text = Some(strip_html_tags(&decoded));
        }
        return;
    }
    if content.body_text.is_none()
        && (content_type_media_type == "text/plain" || content_type_media_type.is_empty())
    {
        content.body_text = Some(normalize_body_text(&decoded));
    }
}

fn parsed_attachment_from_part(
    headers: &[(String, String)],
    body: &[u8],
    content_type: &str,
    provider_attachment_id: String,
) -> ParsedEmailAttachment {
    let transfer_encoding = header_value(headers, "content-transfer-encoding").unwrap_or_default();
    ParsedEmailAttachment {
        provider_attachment_id,
        filename: attachment_filename(headers, content_type),
        content_type: non_empty_or_default(
            header_media_type(content_type),
            "application/octet-stream",
        ),
        disposition: parsed_attachment_disposition(headers),
        body_bytes: decode_transfer_bytes(body, &transfer_encoding),
    }
}

fn is_attachment_like_part(headers: &[(String, String)], content_type: &str) -> bool {
    match parsed_attachment_disposition(headers) {
        ParsedEmailAttachmentDisposition::Attachment => true,
        ParsedEmailAttachmentDisposition::Inline => {
            attachment_filename(headers, content_type).is_some()
        }
        ParsedEmailAttachmentDisposition::Unknown => {
            attachment_filename(headers, content_type).is_some()
        }
    }
}

fn parsed_attachment_disposition(headers: &[(String, String)]) -> ParsedEmailAttachmentDisposition {
    let content_disposition = header_value(headers, "content-disposition").unwrap_or_default();
    match content_disposition
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "attachment" => ParsedEmailAttachmentDisposition::Attachment,
        "inline" => ParsedEmailAttachmentDisposition::Inline,
        _ => ParsedEmailAttachmentDisposition::Unknown,
    }
}

fn attachment_filename(headers: &[(String, String)], content_type: &str) -> Option<String> {
    header_value(headers, "content-disposition")
        .and_then(|value| header_parameter(&value, "filename"))
        .or_else(|| header_parameter(content_type, "name"))
        .map(|value| super::decoding::decode_rfc2047_words(value.trim()))
        .and_then(|value| {
            let value = value.trim().to_owned();
            if value.is_empty() { None } else { Some(value) }
        })
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut inside_tag = false;
    for character in input.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                output.push(' ');
            }
            _ if !inside_tag => output.push(character),
            _ => {}
        }
    }

    normalize_body_text(&output)
}

fn normalize_body_text(input: &str) -> String {
    input
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

fn non_empty_html_body(input: &str) -> Option<String> {
    let value = input.trim().to_owned();
    if value.is_empty() { None } else { Some(value) }
}
```

### `backend/src/platform/communications/rfc822/decoding.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/decoding.rs`
- Size bytes / Размер в байтах: `7266`
- Included characters / Включено символов: `7266`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use encoding_rs::{Encoding, UTF_8};

pub(crate) fn decode_header_value_bytes(value: &[u8]) -> String {
    decode_rfc2047_words(decode_text_bytes(value, None).trim())
}

pub(crate) fn decode_transfer_body(
    body: &[u8],
    transfer_encoding: &str,
    charset: Option<&str>,
) -> String {
    decode_text_bytes(&decode_transfer_bytes(body, transfer_encoding), charset)
}

pub(crate) fn decode_transfer_bytes(body: &[u8], transfer_encoding: &str) -> Vec<u8> {
    match transfer_encoding.trim().to_ascii_lowercase().as_str() {
        "base64" => {
            let compact = body
                .iter()
                .copied()
                .filter(|byte| !byte.is_ascii_whitespace())
                .collect::<Vec<_>>();
            BASE64_STANDARD
                .decode(compact)
                .unwrap_or_else(|_| body.to_vec())
        }
        "quoted-printable" => decode_quoted_printable_bytes(body),
        _ => body.to_vec(),
    }
}

fn decode_quoted_printable(input: &str, charset: Option<&str>) -> String {
    decode_text_bytes(&decode_quoted_printable_bytes(input.as_bytes()), charset)
}

fn decode_quoted_printable_bytes(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;

    while index < input.len() {
        if input[index] == b'=' {
            if input.get(index + 1) == Some(&b'\r') && input.get(index + 2) == Some(&b'\n') {
                index += 3;
                continue;
            }
            if input.get(index + 1) == Some(&b'\n') {
                index += 2;
                continue;
            }
            if let (Some(high), Some(low)) = (input.get(index + 1), input.get(index + 2))
                && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        output.push(input[index]);
        index += 1;
    }

    output
}

pub(crate) fn decode_text_bytes(bytes: &[u8], charset: Option<&str>) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let primary_encoding = charset
        .and_then(|label| Encoding::for_label(label.trim().as_bytes()))
        .unwrap_or(UTF_8);
    let primary = decode_with_encoding(bytes, primary_encoding);
    if charset.is_some() && !primary.had_errors {
        return primary.text;
    }
    if charset.is_none() && !primary.had_errors {
        return primary.text;
    }

    legacy_text_candidates(bytes, primary)
        .into_iter()
        .max_by_key(score_decoded_text)
        .map(|candidate| candidate.text)
        .unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedTextCandidate {
    text: String,
    had_errors: bool,
    fallback_rank: i64,
}

fn decode_with_encoding(bytes: &[u8], encoding: &'static Encoding) -> DecodedTextCandidate {
    let (text, _, had_errors) = encoding.decode(bytes);
    DecodedTextCandidate {
        text: text.into_owned(),
        had_errors,
        fallback_rank: 0,
    }
}

fn legacy_text_candidates(
    bytes: &[u8],
    primary: DecodedTextCandidate,
) -> Vec<DecodedTextCandidate> {
    let mut candidates = vec![primary];
    for (fallback_rank, label) in [
        "windows-1251",
        "koi8-r",
        "iso-8859-5",
        "windows-1252",
        "iso-8859-1",
    ]
    .iter()
    .enumerate()
    {
        let Some(encoding) = Encoding::for_label(label.as_bytes()) else {
            continue;
        };
        let mut candidate = decode_with_encoding(bytes, encoding);
        candidate.fallback_rank = fallback_rank as i64 + 1;
        candidates.push(candidate);
    }
    candidates
}

fn score_decoded_text(candidate: &DecodedTextCandidate) -> i64 {
    let mut replacement_count = 0;
    let mut disallowed_control_count = 0;
    let mut cyrillic_count = 0;
    let mut printable_count = 0;

    for character in candidate.text.chars() {
        if character == '\u{fffd}' {
            replacement_count += 1;
        } else if character.is_control()
            && character != '\n'
            && character != '\r'
            && character != '\t'
        {
            disallowed_control_count += 1;
        } else if ('\u{0400}'..='\u{04ff}').contains(&character) {
            cyrillic_count += 1;
            printable_count += 1;
        } else if !character.is_control() {
            printable_count += 1;
        }
    }

    printable_count + (cyrillic_count * 8)
        - (replacement_count * 1_000)
        - (disallowed_control_count * 100)
        - candidate.fallback_rank
}

pub(crate) fn percent_decode_bytes(value: &str) -> Vec<u8> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2))
            && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
        {
            output.push((high << 4) | low);
            index += 3;
            continue;
        }
        output.push(bytes[index]);
        index += 1;
    }

    output
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn decode_rfc2047_words(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find("=?") {
        output.push_str(&rest[..start]);
        let candidate = &rest[start + 2..];
        let Some(charset_end) = candidate.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let charset = &candidate[..charset_end];
        let candidate = &candidate[charset_end + 1..];
        let Some(encoding_end) = candidate.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let encoding = &candidate[..encoding_end];
        let candidate = &candidate[encoding_end + 1..];
        let Some(encoded_end) = candidate.find("?=") else {
            output.push_str(&rest[start..]);
            return output;
        };
        let encoded = &candidate[..encoded_end];
        let decoded = match encoding.to_ascii_lowercase().as_str() {
            "b" => BASE64_STANDARD
                .decode(encoded)
                .map(|bytes| decode_text_bytes(&bytes, Some(charset)))
                .ok(),
            "q" => Some(decode_quoted_printable(
                &encoded.replace('_', " "),
                Some(charset),
            )),
            _ => None,
        };

        if let Some(decoded) = decoded {
            output.push_str(&decoded);
        } else {
            output.push_str(
                &rest[start..start + 2 + charset_end + 1 + encoding_end + 1 + encoded_end + 2],
            );
        }
        rest = &candidate[encoded_end + 2..];
    }

    output.push_str(rest);
    output
}
```

### `backend/src/platform/communications/rfc822/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/errors.rs`
- Size bytes / Размер в байтах: `164`
- Included characters / Включено символов: `164`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailRfc822ParseError {
    #[error("RFC822 message must contain headers and body")]
    MalformedRfc822,
}
```

### `backend/src/platform/communications/rfc822/headers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/headers.rs`
- Size bytes / Размер в байтах: `6037`
- Included characters / Включено символов: `6037`
- Truncated / Обрезано: `no`

```rust
use super::decoding::{
    decode_header_value_bytes, decode_rfc2047_words, decode_text_bytes, percent_decode_bytes,
};
use super::wire::{strip_trailing_cr, trim_ascii_whitespace};

pub(crate) fn parse_headers(header_block: &[u8]) -> Vec<(String, String)> {
    let mut raw_headers: Vec<(String, Vec<u8>)> = Vec::new();

    for line in header_block.split(|byte| *byte == b'\n') {
        let line = strip_trailing_cr(line);
        if line.starts_with(b" ") || line.starts_with(b"\t") {
            if let Some((_, value)) = raw_headers.last_mut() {
                value.push(b' ');
                value.extend_from_slice(trim_ascii_whitespace(line));
            }
            continue;
        }

        if let Some(separator_index) = line.iter().position(|byte| *byte == b':') {
            let name = decode_ascii_header_name(trim_ascii_whitespace(&line[..separator_index]));
            let value = trim_ascii_whitespace(&line[separator_index + 1..]).to_vec();
            headers_push_if_valid(&mut raw_headers, name, value);
        }
    }

    raw_headers
        .into_iter()
        .map(|(name, value)| (name, decode_header_value_bytes(&value)))
        .collect()
}

fn headers_push_if_valid(headers: &mut Vec<(String, Vec<u8>)>, name: String, value: Vec<u8>) {
    if !name.is_empty() {
        headers.push((name, value));
    }
}

fn decode_ascii_header_name(value: &[u8]) -> String {
    String::from_utf8_lossy(value).trim().to_owned()
}

pub(crate) fn header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| decode_rfc2047_words(value.trim()))
}

pub(crate) fn header_media_type(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

pub(crate) fn header_parameter(value: &str, parameter: &str) -> Option<String> {
    let mut continuation_segments = Vec::new();
    let mut plain_parameter = None;
    let mut encoded_parameter = None;

    for part in value.split(';').skip(1) {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if name.eq_ignore_ascii_case(parameter) {
            plain_parameter = Some(unquote_header_parameter_value(value));
            continue;
        }
        if let Some(segment) = rfc2231_continuation_segment(name, parameter, value) {
            continuation_segments.push(segment);
            continue;
        }
        if let Some(base_name) = name.strip_suffix('*')
            && base_name.eq_ignore_ascii_case(parameter)
        {
            encoded_parameter = Some(decode_rfc2231_parameter_value(value));
        }
    }

    encoded_parameter
        .or_else(|| decode_rfc2231_continuation_segments(continuation_segments))
        .or(plain_parameter)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Rfc2231ContinuationSegment {
    index: usize,
    encoded: bool,
    value: String,
}

fn rfc2231_continuation_segment(
    name: &str,
    parameter: &str,
    value: &str,
) -> Option<Rfc2231ContinuationSegment> {
    if name.len() <= parameter.len() || !name[..parameter.len()].eq_ignore_ascii_case(parameter) {
        return None;
    }
    let rest = &name[parameter.len()..];
    let rest = rest.strip_prefix('*')?;
    let encoded = rest.ends_with('*');
    let index = rest.trim_end_matches('*').parse::<usize>().ok()?;

    Some(Rfc2231ContinuationSegment {
        index,
        encoded,
        value: unquote_header_parameter_value(value),
    })
}

fn decode_rfc2231_continuation_segments(
    mut segments: Vec<Rfc2231ContinuationSegment>,
) -> Option<String> {
    if segments.is_empty() {
        return None;
    }

    segments.sort_by_key(|segment| segment.index);
    if segments.first().map(|segment| segment.index) != Some(0) {
        return None;
    }

    let mut output = Vec::new();
    let mut charset = None;
    for (expected_index, segment) in segments.into_iter().enumerate() {
        if segment.index != expected_index {
            return None;
        }

        let value = if expected_index == 0 {
            let (segment_charset, payload) = rfc2231_charset_and_payload(&segment.value);
            charset = segment_charset.map(str::to_owned);
            payload
        } else {
            segment.value.as_str()
        };
        if segment.encoded {
            output.extend(percent_decode_bytes(value));
        } else {
            output.extend_from_slice(value.as_bytes());
        }
    }

    Some(decode_text_bytes(&output, charset.as_deref()))
}

fn rfc2231_charset_and_payload(value: &str) -> (Option<&str>, &str) {
    let Some((charset, rest)) = value.split_once('\'') else {
        return (None, value);
    };
    let Some((_, encoded)) = rest.split_once('\'') else {
        return (None, value);
    };
    let charset = charset.trim();
    let charset = if charset.is_empty() {
        None
    } else {
        Some(charset)
    };
    (charset, encoded)
}

fn unquote_header_parameter_value(value: &str) -> String {
    let value = value.trim();
    let value = value
        .strip_prefix('"')
        .and_then(|stripped| stripped.strip_suffix('"'))
        .unwrap_or(value);
    let mut output = String::with_capacity(value.len());
    let mut escaped = false;

    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        output.push(character);
    }

    if escaped {
        output.push('\\');
    }

    output
}

fn decode_rfc2231_parameter_value(value: &str) -> String {
    let value = unquote_header_parameter_value(value);
    let (charset, encoded_value) = rfc2231_charset_and_payload(&value);

    decode_text_bytes(&percent_decode_bytes(encoded_value), charset)
}
```

### `backend/src/platform/communications/rfc822/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/models.rs`
- Size bytes / Размер в байтах: `708`
- Included characters / Включено символов: `708`
- Truncated / Обрезано: `no`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCommunicationSourceMessage {
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub headers: Vec<(String, String)>,
    pub body_text: String,
    pub body_html: Option<String>,
    pub attachments: Vec<ParsedEmailAttachment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedEmailAttachment {
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub disposition: ParsedEmailAttachmentDisposition,
    pub body_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParsedEmailAttachmentDisposition {
    Attachment,
    Inline,
    Unknown,
}
```

### `backend/src/platform/communications/rfc822/multipart.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/multipart.rs`
- Size bytes / Размер в байтах: `1635`
- Included characters / Включено символов: `1635`
- Truncated / Обрезано: `no`

```rust
use super::headers::parse_headers;
use super::wire::{find_subslice, next_line_start, split_headers_and_body};

type MimeHeaders = Vec<(String, String)>;
type MimePart<'a> = (MimeHeaders, &'a [u8]);

pub(crate) fn multipart_parts<'a>(boundary: &str, body: &'a [u8]) -> Vec<MimePart<'a>> {
    let mut parts = Vec::new();
    let delimiter = format!("--{boundary}").into_bytes();
    let mut cursor = 0;
    let mut current_part_start = None;

    while let Some(relative_start) = find_subslice(&body[cursor..], &delimiter) {
        let boundary_start = cursor + relative_start;
        if boundary_start > 0 && body[boundary_start - 1] != b'\n' {
            cursor = boundary_start + delimiter.len();
            continue;
        }

        if let Some(part_start) = current_part_start {
            let raw_part = trim_multipart_part_body(&body[part_start..boundary_start]);
            if let Ok((headers, nested_body)) = split_headers_and_body(raw_part) {
                let headers = parse_headers(headers);
                parts.push((headers, nested_body));
            }
        }

        let after_delimiter = boundary_start + delimiter.len();
        if body.get(after_delimiter..after_delimiter + 2) == Some(b"--") {
            break;
        }

        let Some(next_line_start) = next_line_start(body, after_delimiter) else {
            break;
        };
        current_part_start = Some(next_line_start);
        cursor = next_line_start;
    }

    parts
}

fn trim_multipart_part_body(body: &[u8]) -> &[u8] {
    body.strip_suffix(b"\r\n")
        .or_else(|| body.strip_suffix(b"\n"))
        .unwrap_or(body)
}
```

### `backend/src/platform/communications/rfc822/parser.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/parser.rs`
- Size bytes / Размер в байтах: `1398`
- Included characters / Включено символов: `1398`
- Truncated / Обрезано: `no`

```rust
use super::body::body_content_from_part;
use super::errors::EmailRfc822ParseError;
use super::headers::{header_value, parse_headers};
use super::models::ParsedCommunicationSourceMessage;
use super::util::{non_empty_or_default, non_empty_recipients, split_address_list};
use super::wire::split_headers_and_body;

pub fn parse_rfc822_message(
    raw: &[u8],
) -> Result<ParsedCommunicationSourceMessage, EmailRfc822ParseError> {
    let (header_block, body) = split_headers_and_body(raw)?;
    let headers = parse_headers(header_block);

    let subject = header_value(&headers, "subject").unwrap_or_else(|| "(no subject)".to_owned());
    let from =
        header_value(&headers, "from").unwrap_or_else(|| "unknown@example.invalid".to_owned());
    let to = split_address_list(&header_value(&headers, "to").unwrap_or_default());
    let body_content = body_content_from_part(&headers, body);
    let body_text = body_content
        .body_text
        .unwrap_or_else(|| "(empty body)".to_owned());

    Ok(ParsedCommunicationSourceMessage {
        subject: non_empty_or_default(subject, "(no subject)"),
        from: non_empty_or_default(from, "unknown@example.invalid"),
        to: non_empty_recipients(to),
        headers,
        body_text: non_empty_or_default(body_text, "(empty body)"),
        body_html: body_content.body_html,
        attachments: body_content.attachments,
    })
}
```

### `backend/src/platform/communications/rfc822/util.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/util.rs`
- Size bytes / Размер в байтах: `610`
- Included characters / Включено символов: `610`
- Truncated / Обрезано: `no`

```rust
pub(crate) fn split_address_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

pub(crate) fn non_empty_or_default(value: String, default: &str) -> String {
    let value = value.trim().to_owned();
    if value.is_empty() {
        default.to_owned()
    } else {
        value
    }
}

pub(crate) fn non_empty_recipients(recipients: Vec<String>) -> Vec<String> {
    if recipients.is_empty() {
        vec!["unknown@example.invalid".to_owned()]
    } else {
        recipients
    }
}
```

### `backend/src/platform/communications/rfc822/wire.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/communications/rfc822/wire.rs`
- Size bytes / Размер в байтах: `1485`
- Included characters / Включено символов: `1485`
- Truncated / Обрезано: `no`

```rust
use super::errors::EmailRfc822ParseError;

pub(crate) fn split_headers_and_body(raw: &[u8]) -> Result<(&[u8], &[u8]), EmailRfc822ParseError> {
    if let Some(separator_start) = find_subslice(raw, b"\r\n\r\n") {
        return Ok((
            &raw[..separator_start],
            &raw[separator_start + b"\r\n\r\n".len()..],
        ));
    }
    if let Some(separator_start) = find_subslice(raw, b"\n\n") {
        return Ok((
            &raw[..separator_start],
            &raw[separator_start + b"\n\n".len()..],
        ));
    }

    Err(EmailRfc822ParseError::MalformedRfc822)
}

pub(crate) fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub(crate) fn next_line_start(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|line_end| start + line_end + 1)
}

pub(crate) fn strip_trailing_cr(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
}

pub(crate) fn trim_ascii_whitespace(value: &[u8]) -> &[u8] {
    let start = value
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|index| index + 1)
        .unwrap_or(start);
    &value[start..end]
}
```

### `backend/src/platform/config.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config.rs`
- Size bytes / Размер в байтах: `234`
- Included characters / Включено символов: `234`
- Truncated / Обрезано: `no`

```rust
mod ai;
mod app_config;
mod constants;
mod errors;
mod google;
mod parsing;

pub use ai::AiRuntimeProvider;
pub use app_config::AppConfig;
pub use errors::ConfigError;
pub use google::{GoogleOAuthClientConfig, GoogleOAuthClientType};
```

### `backend/src/platform/config/ai.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/ai.rs`
- Size bytes / Размер в байтах: `762`
- Included characters / Включено символов: `762`
- Truncated / Обрезано: `no`

```rust
use super::errors::ConfigError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiRuntimeProvider {
    Ollama,
    OmniRoute,
}

impl AiRuntimeProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OmniRoute => "omniroute",
        }
    }
}

impl TryFrom<&str> for AiRuntimeProvider {
    type Error = ConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "omniroute" | "omni_route" | "omni-route" => Ok(Self::OmniRoute),
            _ => Err(ConfigError::InvalidAiProvider {
                value: value.to_owned(),
            }),
        }
    }
}
```

### `backend/src/platform/config/app_config.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/app_config.rs`
- Size bytes / Размер в байтах: `1477`
- Included characters / Включено символов: `1477`
- Truncated / Обрезано: `no`

```rust
mod accessors;
mod ai_env;
mod core_env;
mod defaults;
mod env;
mod provider_env;
#[cfg(any(test, feature = "test-support"))]
mod test_support;

use std::net::SocketAddr;
use std::path::PathBuf;

use crate::platform::secrets::ResolvedSecret;

use super::ai::AiRuntimeProvider;
use super::google::GoogleOAuthClientConfig;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppConfig {
    service_name: String,
    http_addr: SocketAddr,
    database_url: Option<String>,
    local_api_secret: Option<String>,
    nats_server_url: Option<String>,
    secret_vault_path: Option<PathBuf>,
    secret_vault_key: Option<ResolvedSecret>,
    vault_home: PathBuf,
    dev_mode: bool,
    dev_key_path: PathBuf,
    tdjson_path: Option<PathBuf>,
    telegram_api_id: Option<i64>,
    telegram_api_hash: Option<ResolvedSecret>,
    google_oauth_client: Option<GoogleOAuthClientConfig>,
    google_oauth_client_id: Option<String>,
    google_oauth_client_secret: Option<ResolvedSecret>,
    zoom_token_maintenance_scheduler_enabled: bool,
    zoom_recording_sync_scheduler_enabled: bool,
    zoom_retention_cleanup_scheduler_enabled: bool,
    ai_provider: AiRuntimeProvider,
    ollama_base_url: String,
    ollama_chat_model: String,
    ollama_embed_model: String,
    ollama_timeout_seconds: u64,
    omniroute_base_url: String,
    omniroute_chat_model: String,
    omniroute_embed_model: String,
    omniroute_timeout_seconds: u64,
    omniroute_api_key: Option<ResolvedSecret>,
}
```

### `backend/src/platform/config/app_config/accessors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/app_config/accessors.rs`
- Size bytes / Размер в байтах: `3351`
- Included characters / Включено символов: `3351`
- Truncated / Обрезано: `no`

```rust
use std::net::SocketAddr;
use std::path::Path;

use crate::platform::secrets::ResolvedSecret;

use super::super::ai::AiRuntimeProvider;
use super::super::google::GoogleOAuthClientConfig;
use super::AppConfig;

impl AppConfig {
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr
    }

    pub fn database_url(&self) -> Option<&str> {
        self.database_url.as_deref()
    }

    pub fn local_api_secret(&self) -> Option<&str> {
        self.local_api_secret.as_deref()
    }

    pub fn nats_server_url(&self) -> Option<&str> {
        self.nats_server_url.as_deref()
    }

    pub fn secret_vault_path(&self) -> Option<&Path> {
        self.secret_vault_path.as_deref()
    }

    pub fn secret_vault_key(&self) -> Option<&ResolvedSecret> {
        self.secret_vault_key.as_ref()
    }

    pub fn vault_home(&self) -> &Path {
        &self.vault_home
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode
    }

    pub fn dev_key_path(&self) -> &Path {
        &self.dev_key_path
    }

    pub fn tdjson_path(&self) -> Option<&Path> {
        self.tdjson_path.as_deref()
    }

    pub fn telegram_api_id(&self) -> Option<i64> {
        self.telegram_api_id
    }

    pub fn telegram_api_hash(&self) -> Option<&ResolvedSecret> {
        self.telegram_api_hash.as_ref()
    }

    pub fn google_oauth_client_id(&self) -> Option<&str> {
        self.google_oauth_client_id.as_deref().or_else(|| {
            self.google_oauth_client
                .as_ref()
                .map(GoogleOAuthClientConfig::client_id)
        })
    }

    pub fn google_oauth_client_secret(&self) -> Option<&ResolvedSecret> {
        self.google_oauth_client_secret.as_ref().or_else(|| {
            self.google_oauth_client
                .as_ref()
                .and_then(GoogleOAuthClientConfig::client_secret)
        })
    }

    pub fn google_oauth_client(&self) -> Option<&GoogleOAuthClientConfig> {
        self.google_oauth_client.as_ref()
    }

    pub fn zoom_token_maintenance_scheduler_enabled(&self) -> bool {
        self.zoom_token_maintenance_scheduler_enabled
    }

    pub fn zoom_recording_sync_scheduler_enabled(&self) -> bool {
        self.zoom_recording_sync_scheduler_enabled
    }

    pub fn zoom_retention_cleanup_scheduler_enabled(&self) -> bool {
        self.zoom_retention_cleanup_scheduler_enabled
    }

    pub fn ai_provider(&self) -> AiRuntimeProvider {
        self.ai_provider
    }

    pub fn ollama_base_url(&self) -> &str {
        &self.ollama_base_url
    }

    pub fn ollama_chat_model(&self) -> &str {
        &self.ollama_chat_model
    }

    pub fn ollama_embed_model(&self) -> &str {
        &self.ollama_embed_model
    }

    pub fn ollama_timeout_seconds(&self) -> u64 {
        self.ollama_timeout_seconds
    }

    pub fn omniroute_base_url(&self) -> &str {
        &self.omniroute_base_url
    }

    pub fn omniroute_chat_model(&self) -> &str {
        &self.omniroute_chat_model
    }

    pub fn omniroute_embed_model(&self) -> &str {
        &self.omniroute_embed_model
    }

    pub fn omniroute_timeout_seconds(&self) -> u64 {
        self.omniroute_timeout_seconds
    }

    pub fn omniroute_api_key(&self) -> Option<&ResolvedSecret> {
        self.omniroute_api_key.as_ref()
    }
}
```

### `backend/src/platform/config/app_config/ai_env.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/app_config/ai_env.rs`
- Size bytes / Размер в байтах: `3442`
- Included characters / Включено символов: `3442`
- Truncated / Обрезано: `no`

```rust
use crate::platform::secrets::ResolvedSecret;

use super::super::ai::AiRuntimeProvider;
use super::super::errors::ConfigError;
use super::AppConfig;

pub(super) fn apply_ai_env(
    config: &mut AppConfig,
    key: &str,
    value: &str,
) -> Result<bool, ConfigError> {
    match key {
        "MAKOSH_AI_PROVIDER" => {
            config.ai_provider = AiRuntimeProvider::try_from(value)?;
        }
        "MAKOSH_OLLAMA_BASE_URL" => {
            config.ollama_base_url = non_empty(value, ConfigError::EmptyOllamaBaseUrl)?
                .trim_end_matches('/')
                .to_owned();
        }
        "MAKOSH_OLLAMA_CHAT_MODEL" => {
            config.ollama_chat_model =
                non_empty(value, ConfigError::EmptyOllamaChatModel)?.to_owned();
        }
        "MAKOSH_OLLAMA_EMBED_MODEL" => {
            config.ollama_embed_model =
                non_empty(value, ConfigError::EmptyOllamaEmbedModel)?.to_owned();
        }
        "MAKOSH_OLLAMA_TIMEOUT_SECONDS" => {
            config.ollama_timeout_seconds = parse_positive_timeout(value, TimeoutTarget::Ollama)?;
        }
        "MAKOSH_OMNIROUTE_BASE_URL" => {
            config.omniroute_base_url = non_empty(value, ConfigError::EmptyOmniRouteBaseUrl)?
                .trim_end_matches('/')
                .to_owned();
        }
        "MAKOSH_OMNIROUTE_CHAT_MODEL" => {
            config.omniroute_chat_model =
                non_empty(value, ConfigError::EmptyOmniRouteChatModel)?.to_owned();
        }
        "MAKOSH_OMNIROUTE_EMBED_MODEL" => {
            config.omniroute_embed_model =
                non_empty(value, ConfigError::EmptyOmniRouteEmbedModel)?.to_owned();
        }
        "MAKOSH_OMNIROUTE_TIMEOUT_SECONDS" => {
            config.omniroute_timeout_seconds =
                parse_positive_timeout(value, TimeoutTarget::OmniRoute)?;
        }
        "MAKOSH_OMNIROUTE_API_KEY" => {
            config.omniroute_api_key = Some(ResolvedSecret::new(non_empty(
                value,
                ConfigError::EmptyOmniRouteApiKey,
            )?)?);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn non_empty(value: &str, error: ConfigError) -> Result<&str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(error)
    } else {
        Ok(trimmed)
    }
}

#[derive(Clone, Copy)]
enum TimeoutTarget {
    Ollama,
    OmniRoute,
}

fn parse_positive_timeout(value: &str, target: TimeoutTarget) -> Result<u64, ConfigError> {
    let raw_timeout = value.trim();
    let timeout = raw_timeout.parse::<u64>().map_err(|source| {
        timeout_error(
            target,
            raw_timeout,
            "must be a positive integer",
            Some(source),
        )
    })?;
    if timeout == 0 {
        return Err(timeout_error(
            target,
            raw_timeout,
            "must be greater than zero",
            None,
        ));
    }
    Ok(timeout)
}

fn timeout_error(
    target: TimeoutTarget,
    value: &str,
    reason: &'static str,
    source: Option<std::num::ParseIntError>,
) -> ConfigError {
    match target {
        TimeoutTarget::Ollama => ConfigError::InvalidOllamaTimeout {
            value: value.to_owned(),
            reason,
            source,
        },
        TimeoutTarget::OmniRoute => ConfigError::InvalidOmniRouteTimeout {
            value: value.to_owned(),
            reason,
            source,
        },
    }
}
```

### `backend/src/platform/config/app_config/core_env.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/app_config/core_env.rs`
- Size bytes / Размер в байтах: `2187`
- Included characters / Включено символов: `2187`
- Truncated / Обрезано: `no`

```rust
use std::path::PathBuf;

use crate::platform::secrets::ResolvedSecret;

use super::super::errors::ConfigError;
use super::super::parsing::parse_bool_env;
use super::AppConfig;

pub(super) fn apply_core_env(
    config: &mut AppConfig,
    key: &str,
    value: &str,
) -> Result<bool, ConfigError> {
    match key {
        "MAKOSH_HTTP_ADDR" => {
            let raw_addr = value.trim();
            config.http_addr = raw_addr
                .parse()
                .map_err(|source| ConfigError::InvalidHttpAddr {
                    value: raw_addr.to_owned(),
                    source,
                })?;
        }
        "DATABASE_URL" => {
            config.database_url = Some(non_empty(value, ConfigError::EmptyDatabaseUrl)?.to_owned());
        }
        "MAKOSH_LOCAL_API_SECRET" => {
            config.local_api_secret =
                Some(non_empty(value, ConfigError::EmptyLocalApiSecret)?.to_owned());
        }
        "MAKOSH_NATS_SERVER_URL" => {
            config.nats_server_url =
                Some(non_empty(value, ConfigError::EmptyNatsServerUrl)?.to_owned());
        }
        "MAKOSH_SECRET_VAULT_PATH" => {
            config.secret_vault_path = Some(PathBuf::from(non_empty(
                value,
                ConfigError::EmptySecretVaultPath,
            )?));
        }
        "MAKOSH_SECRET_VAULT_KEY" => {
            config.secret_vault_key = Some(ResolvedSecret::new(non_empty(
                value,
                ConfigError::EmptySecretVaultKey,
            )?)?);
        }
        "MAKOSH_VAULT_HOME" => {
            config.vault_home = PathBuf::from(non_empty(value, ConfigError::EmptyVaultHome)?);
        }
        "MAKOSH_DEV_MODE" => {
            config.dev_mode = parse_bool_env("MAKOSH_DEV_MODE", value.trim())?;
        }
        "MAKOSH_DEV_KEY_PATH" => {
            config.dev_key_path = PathBuf::from(non_empty(value, ConfigError::EmptyDevKeyPath)?);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn non_empty(value: &str, error: ConfigError) -> Result<&str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(error)
    } else {
        Ok(trimmed)
    }
}
```

### `backend/src/platform/config/app_config/defaults.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/app_config/defaults.rs`
- Size bytes / Размер в байтах: `2287`
- Included characters / Включено символов: `2287`
- Truncated / Обрезано: `no`

```rust
use std::env;
use std::path::PathBuf;

use crate::vault::{default_dev_key_path, default_vault_home};

use super::super::ai::AiRuntimeProvider;
use super::super::constants::{
    DEFAULT_HTTP_ADDR, DEFAULT_OLLAMA_BASE_URL, DEFAULT_OLLAMA_CHAT_MODEL,
    DEFAULT_OLLAMA_EMBED_MODEL, DEFAULT_OLLAMA_TIMEOUT_SECONDS, DEFAULT_OMNIROUTE_BASE_URL,
    DEFAULT_OMNIROUTE_CHAT_MODEL, DEFAULT_OMNIROUTE_EMBED_MODEL, DEFAULT_OMNIROUTE_TIMEOUT_SECONDS,
    DEFAULT_SERVICE_NAME,
};
use super::AppConfig;

impl Default for AppConfig {
    fn default() -> Self {
        let home_dir = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            service_name: DEFAULT_SERVICE_NAME.to_owned(),
            http_addr: DEFAULT_HTTP_ADDR
                .parse()
                .expect("default HTTP bind address must be valid"),
            database_url: None,
            local_api_secret: None,
            nats_server_url: None,
            secret_vault_path: None,
            secret_vault_key: None,
            vault_home: default_vault_home(&home_dir),
            dev_mode: false,
            dev_key_path: default_dev_key_path(&home_dir),
            tdjson_path: None,
            telegram_api_id: None,
            telegram_api_hash: None,
            google_oauth_client: None,
            google_oauth_client_id: None,
            google_oauth_client_secret: None,
            zoom_token_maintenance_scheduler_enabled: true,
            zoom_recording_sync_scheduler_enabled: true,
            zoom_retention_cleanup_scheduler_enabled: true,
            ai_provider: AiRuntimeProvider::Ollama,
            ollama_base_url: DEFAULT_OLLAMA_BASE_URL.to_owned(),
            ollama_chat_model: DEFAULT_OLLAMA_CHAT_MODEL.to_owned(),
            ollama_embed_model: DEFAULT_OLLAMA_EMBED_MODEL.to_owned(),
            ollama_timeout_seconds: DEFAULT_OLLAMA_TIMEOUT_SECONDS,
            omniroute_base_url: DEFAULT_OMNIROUTE_BASE_URL.to_owned(),
            omniroute_chat_model: DEFAULT_OMNIROUTE_CHAT_MODEL.to_owned(),
            omniroute_embed_model: DEFAULT_OMNIROUTE_EMBED_MODEL.to_owned(),
            omniroute_timeout_seconds: DEFAULT_OMNIROUTE_TIMEOUT_SECONDS,
            omniroute_api_key: None,
        }
    }
}
```
