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

- Chunk ID / ID чанка: `039-source-backend-part-019`
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

### `backend/src/domains/communications/core/secrets.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/secrets.rs`
- Size bytes / Размер в байтах: `3445`
- Included characters / Включено символов: `3445`
- Truncated / Обрезано: `no`

```rust
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::errors::{CommunicationIngestionError, ProviderCredentialError};
use super::models::{
    NewProviderAccountSecretBinding, ProviderAccountSecretBinding, ProviderAccountSecretPurpose,
    ProviderCredential,
};
use super::provider_store::CommunicationProviderSecretBindingStore;
use super::store::CommunicationIngestionStore;
use super::validation::validate_non_empty;

impl CommunicationIngestionStore {
    pub async fn bind_provider_account_secret(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .bind(binding)
            .await
    }

    pub async fn provider_account_secret_bindings(
        &self,
        account_id: &str,
    ) -> Result<Vec<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .list_for_account(account_id)
            .await
    }

    pub async fn provider_account_secret_binding(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<Option<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .get_for_account(account_id, secret_purpose)
            .await
    }
}

pub struct ProviderCredentialReader<'a, R: SecretResolver + ?Sized> {
    secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    resolver: &'a R,
}

impl<'a, R: SecretResolver + ?Sized> ProviderCredentialReader<'a, R> {
    pub fn new(
        secret_binding_store: CommunicationProviderSecretBindingStore,
        secret_store: SecretReferenceStore,
        resolver: &'a R,
    ) -> Self {
        Self {
            secret_binding_store,
            secret_store,
            resolver,
        }
    }

    pub async fn read(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<ProviderCredential, ProviderCredentialError> {
        validate_non_empty("account_id", account_id)?;

        let binding = self
            .secret_binding_store
            .get_for_account(account_id, secret_purpose)
            .await?
            .ok_or_else(|| ProviderCredentialError::MissingBinding {
                account_id: account_id.trim().to_owned(),
                secret_purpose,
            })?;
        let reference = self
            .secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| ProviderCredentialError::MissingSecretReference {
                secret_ref: binding.secret_ref.clone(),
            })?;
        if !binding
            .secret_purpose
            .accepts_secret_kind(reference.secret_kind)
        {
            return Err(ProviderCredentialError::IncompatibleSecretKind {
                secret_ref: reference.secret_ref.clone(),
                secret_purpose: binding.secret_purpose,
                secret_kind: reference.secret_kind,
            });
        }

        let secret = self.resolver.resolve(&reference).await?;

        Ok(ProviderCredential {
            binding,
            reference,
            secret,
        })
    }
}
```

### `backend/src/domains/communications/core/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/store.rs`
- Size bytes / Размер в байтах: `298`
- Included characters / Включено символов: `298`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct CommunicationIngestionStore {
    pub(super) pool: PgPool,
}

impl CommunicationIngestionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> PgPool {
        self.pool.clone()
    }
}
```

### `backend/src/domains/communications/core/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/validation.rs`
- Size bytes / Размер в байтах: `579`
- Included characters / Включено символов: `579`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::CommunicationIngestionError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), CommunicationIngestionError> {
    if value.trim().is_empty() {
        return Err(CommunicationIngestionError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), CommunicationIngestionError> {
    if !value.is_object() {
        return Err(CommunicationIngestionError::NonObjectJson(field_name));
    }

    Ok(())
}
```

### `backend/src/domains/communications/delivery_notifications.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/delivery_notifications.rs`
- Size bytes / Размер в байтах: `22509`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::messages::COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER;
use super::outbox::{
    CommunicationOutboxStore, NewOutboxDeliveryStatus, OutboxDeliveryStatus,
    OutboxDeliveryStatusRecord,
};
use super::read_receipts::{
    CommunicationReadReceipt, CommunicationReadReceiptStore, NewCommunicationReadReceipt,
};

const MAX_NOTIFICATION_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Deserialize)]
pub struct NewCommunicationDeliveryNotification {
    pub account_id: String,
    pub raw_message: String,
    pub received_at: Option<DateTime<Utc>>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderDeliveryEventKind {
    Delivered,
    Delayed,
    Failed,
    Read,
}

impl ProviderDeliveryEventKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Delayed => "delayed",
            Self::Failed => "failed",
            Self::Read => "read",
        }
    }

    fn delivery_status(self) -> Option<OutboxDeliveryStatus> {
        match self {
            Self::Delivered => Some(OutboxDeliveryStatus::Delivered),
            Self::Delayed => Some(OutboxDeliveryStatus::Delayed),
            Self::Failed => Some(OutboxDeliveryStatus::Failed),
            Self::Read => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewProviderDeliveryEvent {
    pub account_id: String,
    pub provider_message_id: String,
    pub event_kind: ProviderDeliveryEventKind,
    pub recipient: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub source_kind: Option<String>,
    pub smtp_status: Option<String>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationDeliveryNotificationKind {
    DeliveryStatus,
    ReadReceipt,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationDeliveryNotificationRecord {
    pub notification_kind: CommunicationDeliveryNotificationKind,
    pub account_id: String,
    pub outbox_id: Option<String>,
    pub provider_message_id: String,
    pub delivery_status: Option<OutboxDeliveryStatus>,
    pub smtp_status: Option<String>,
    pub source_kind: String,
    pub read_receipt: Option<CommunicationReadReceipt>,
}

#[derive(Clone)]
pub struct CommunicationDeliveryNotificationStore {
    pool: PgPool,
}

impl CommunicationDeliveryNotificationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        notification: NewCommunicationDeliveryNotification,
    ) -> Result<CommunicationDeliveryNotificationRecord, CommunicationDeliveryNotificationError>
    {
        let account_id = normalize_required("account_id", &notification.account_id)?;
        let received_at = notification.received_at.unwrap_or_else(Utc::now);
        let provider_record_id = normalize_optional(notification.provider_record_id);
        let raw_record_id = normalize_optional(notification.raw_record_id);
        let parsed = parse_delivery_notification(&notification.raw_message)?;

        match parsed {
            ParsedDeliveryNotification::DeliveryStatus {
                provider_message_id,
                delivery_status,
                smtp_status,
            } => {
                let record = CommunicationOutboxStore::new(self.pool.clone())
                    .record_delivery_status(&NewOutboxDeliveryStatus {
                        account_id,
                        provider_message_id,
                        delivery_status,
                        smtp_status,
                        source_kind: "dsn".to_owned(),
                        provider_record_id,
                        raw_record_id,
                        recorded_at: received_at,
                    })
                    .await?;
                Ok(delivery_status_response(record))
            }
            ParsedDeliveryNotification::ReadReceipt {
                provider_message_id,
                recipient,
                reporting_ua,
            } => {
                let receipt = CommunicationReadReceiptStore::new(self.pool.clone())
                    .record(NewCommunicationReadReceipt {
                        receipt_id: None,
                        account_id,
                        provider_message_id,
                        recipient,
                        read_at: received_at,
                        source_kind: Some("mdn".to_owned()),
                        provider_record_id,
                        raw_record_id,
                        metadata: Some(json!({ "reporting_ua": reporting_ua })),
                    })
                    .await?;
                Ok(read_receipt_response(receipt))
            }
        }
    }

    pub async fn record_provider_event(
        &self,
        event: NewProviderDeliveryEvent,
    ) -> Result<CommunicationDeliveryNotificationRecord, CommunicationDeliveryNotificationError>
    {
        let account_id = normalize_required("account_id", &event.account_id)?;
        let provider_message_id =
            normalize_required("provider_message_id", &event.provider_message_id)?;
        let occurred_at = event.occurred_at.unwrap_or_else(Utc::now);
        let source_kind =
            normalize_optional(event.source_kind).unwrap_or_else(|| "provider_event".to_owned());
        let provider_record_id = normalize_optional(event.provider_record_id);
        let raw_record_id = normalize_optional(event.raw_record_id);

        if let Some(delivery_status) = event.event_kind.delivery_status() {
            let record = CommunicationOutboxStore::new(self.pool.clone())
                .record_delivery_status(&NewOutboxDeliveryStatus {
                    account_id,
                    provider_message_id,
                    delivery_status,
                    smtp_status: normalize_optional(event.smtp_status),
                    source_kind,
                    provider_record_id,
                    raw_record_id,
                    recorded_at: occurred_at,
                })
                .await?;
            return Ok(delivery_status_response(record));
        }

        let recipient = normalize_optional(event.recipient)
            .ok_or(CommunicationDeliveryNotificationError::Invalid("recipient"))?;
        let mut metadata = json!({ "provider_event_kind": event.event_kind.as_str() });
        if let Some(extra) = event.metadata
            && let (Some(target), Some(extra_map)) = (metadata.as_object_mut(), extra.as_object())
        {
            for (key, value) in extra_map {
                target.insert(key.clone(), value.clone());
            }
        }
        let receipt = CommunicationReadReceiptStore::new(self.pool.clone())
            .record(NewCommunicationReadReceipt {
                receipt_id: None,
                account_id,
                provider_message_id,
                recipient,
                read_at: occurred_at,
                source_kind: Some(source_kind),
                provider_record_id,
                raw_record_id,
                metadata: Some(metadata),
            })
            .await?;

        Ok(read_receipt_response(receipt))
    }
}

pub fn provider_event_from_delivery_notification(
    notification: &NewCommunicationDeliveryNotification,
) -> Result<NewProviderDeliveryEvent, CommunicationDeliveryNotificationError> {
    let account_id = normalize_required("account_id", &notification.account_id)?;
    let occurred_at = notification.received_at.unwrap_or_else(Utc::now);
    let provider_record_id = normalize_optional(notification.provider_record_id.clone());
    let raw_record_id = normalize_optional(notification.raw_record_id.clone());
    let parsed = parse_delivery_notification(&notification.raw_message)?;

    match parsed {
        ParsedDeliveryNotification::DeliveryStatus {
            provider_message_id,
            delivery_status,
            smtp_status,
        } => Ok(NewProviderDeliveryEvent {
            account_id,
            provider_message_id,
            event_kind: delivery_status_provider_event_kind(delivery_status),
            recipient: None,
            occurred_at: Some(occurred_at),
            source_kind: Some("dsn".to_owned()),
            smtp_status,
            provider_record_id,
            raw_record_id,
            metadata: None,
        }),
        ParsedDeliveryNotification::ReadReceipt {
            provider_message_id,
            recipient,
            reporting_ua,
        } => Ok(NewProviderDeliveryEvent {
            account_id,
            provider_message_id,
            event_kind: ProviderDeliveryEventKind::Read,
            recipient: Some(recipient),
            occurred_at: Some(occurred_at),
            source_kind: Some("mdn".to_owned()),
            smtp_status: None,
            provider_record_id,
            raw_record_id,
            metadata: Some(json!({ "reporting_ua": reporting_ua })),
        }),
    }
}

pub async fn project_accepted_mail_delivery_signal_if_runtime_allows(
    pool: PgPool,
    event: &crate::platform::events::EventEnvelope,
) -> Result<Option<CommunicationDeliveryNotificationRecord>, CommunicationDeliveryNotificationError>
{
    if !crate::platform::events::runtime_allows_processing(
        &pool,
        "system",
        COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        &json!({
            "label": "Communications accepted-signal consumer",
            "scope": "consumer",
        }),
    )
    .await?
    {
        return Ok(None);
    }

    consume_accepted_mail_delivery_signal(pool, event).await
}

pub async fn consume_accepted_mail_delivery_signal(
    pool: PgPool,
    event: &crate::platform::events::EventEnvelope,
) -> Result<Option<CommunicationDeliveryNotificationRecord>, CommunicationDeliveryNotificationError>
{
    let Some(provider_event) = provider_event_from_accepted_signal(event)? else {
        return Ok(None);
    };

    CommunicationDeliveryNotificationStore::new(pool)
        .record_provider_event(provider_event)
        .await
        .map(Some)
}

fn delivery_status_response(
    record: OutboxDeliveryStatusRecord,
) -> CommunicationDeliveryNotificationRecord {
    CommunicationDeliveryNotificationRecord {
        notification_kind: CommunicationDeliveryNotificationKind::DeliveryStatus,
        account_id: record.account_id,
        outbox_id: record.outbox_id,
        provider_message_id: record.provider_message_id,
        delivery_status: Some(record.delivery_status),
        smtp_status: record.smtp_status,
        source_kind: record.source_kind,
        read_receipt: None,
    }
}

fn read_receipt_response(
    receipt: CommunicationReadReceipt,
) -> CommunicationDeliveryNotificationRecord {
    CommunicationDeliveryNotificationRecord {
        notification_kind: CommunicationDeliveryNotificationKind::ReadReceipt,
        account_id: receipt.account_id.clone(),
        outbox_id: receipt.outbox_id.clone(),
        provider_message_id: receipt.provider_message_id.clone(),
        delivery_status: None,
        smtp_status: None,
        source_kind: receipt.source_kind.clone(),
        read_receipt: Some(receipt),
    }
}

fn provider_event_from_accepted_signal(
    event: &crate::platform::events::EventEnvelope,
) -> Result<Option<NewProviderDeliveryEvent>, CommunicationDeliveryNotificationError> {
    match event.event_type.as_str() {
        "signal.accepted.mail.delivery_status" => Ok(Some(NewProviderDeliveryEvent {
            account_id: required_payload_str(&event.payload, "account_id")?,
            provide
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/drafts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/drafts.rs`
- Size bytes / Размер в байтах: `21808`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::ObservationStoreError;

const EVENT_TYPE_DRAFT_CREATED: &str = "mail.draft.created";
const EVENT_TYPE_DRAFT_UPDATED: &str = "mail.draft.updated";
const EVENT_TYPE_DRAFT_DELETED: &str = "mail.draft.deleted";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationDraft {
    pub draft_id: String,
    pub account_id: String,
    pub persona_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub status: DraftStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub send_attempts: i32,
    pub last_error: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationDraftListPage {
    pub items: Vec<CommunicationDraft>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Failed,
}

impl DraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DraftStatus::Draft => "draft",
            DraftStatus::Scheduled => "scheduled",
            DraftStatus::Sending => "sending",
            DraftStatus::Sent => "sent",
            DraftStatus::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "draft" => Some(DraftStatus::Draft),
            "scheduled" => Some(DraftStatus::Scheduled),
            "sending" => Some(DraftStatus::Sending),
            "sent" => Some(DraftStatus::Sent),
            "failed" => Some(DraftStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewCommunicationDraft {
    pub draft_id: String,
    pub account_id: String,
    pub persona_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub status: DraftStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewCommunicationDraft {
    fn validate(&self) -> Result<(), CommunicationDraftError> {
        if self.draft_id.trim().is_empty() {
            return Err(CommunicationDraftError::Invalid("draft_id empty"));
        }
        if self.account_id.trim().is_empty() {
            return Err(CommunicationDraftError::Invalid("account_id empty"));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct CommunicationDraftStore {
    pool: PgPool,
}

impl CommunicationDraftStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        draft: &NewCommunicationDraft,
    ) -> Result<CommunicationDraft, CommunicationDraftError> {
        self.upsert_with_observation(draft, None, "draft_upsert", None)
            .await
    }

    pub async fn upsert_with_observation(
        &self,
        draft: &NewCommunicationDraft,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CommunicationDraft, CommunicationDraftError> {
        draft.validate()?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, Some(draft.account_id.as_str()))
            .await?;
        let existed = draft_exists(&mut transaction, &draft.draft_id).await?;
        let row = sqlx::query(
            r#"INSERT INTO communication_drafts (draft_id, account_id, identity_id, to_participants, cc_participants, bcc_participants, subject, body_text, body_html, in_reply_to, message_refs, status, scheduled_send_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (draft_id) DO UPDATE SET
                account_id = EXCLUDED.account_id, identity_id = EXCLUDED.identity_id,
                to_participants = EXCLUDED.to_participants, cc_participants = EXCLUDED.cc_participants,
                bcc_participants = EXCLUDED.bcc_participants, subject = EXCLUDED.subject,
                body_text = EXCLUDED.body_text, body_html = EXCLUDED.body_html,
                in_reply_to = EXCLUDED.in_reply_to, message_refs = EXCLUDED.message_refs,
                status = EXCLUDED.status, scheduled_send_at = EXCLUDED.scheduled_send_at,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                status,
                scheduled_send_at,
                send_attempts,
                last_error,
                metadata,
                created_at,
                updated_at"#,
        )
        .bind(&draft.draft_id).bind(&draft.account_id).bind(draft.persona_id.as_deref())
        .bind(serde_json::to_value(&draft.to_recipients).unwrap_or_default())
        .bind(serde_json::to_value(&draft.cc_recipients).unwrap_or_default())
        .bind(serde_json::to_value(&draft.bcc_recipients).unwrap_or_default())
        .bind(&draft.subject).bind(&draft.body_text).bind(draft.body_html.as_deref())
        .bind(draft.in_reply_to.as_deref())
        .bind(serde_json::to_value(&draft.references).unwrap_or_default())
        .bind(draft.status.as_str()).bind(draft.scheduled_send_at).bind(&draft.metadata)
        .fetch_one(&mut *transaction).await?;
        let draft = row_to_draft(row)?;
        let event_type = if existed {
            EVENT_TYPE_DRAFT_UPDATED
        } else {
            EVENT_TYPE_DRAFT_CREATED
        };
        let event = draft_event(event_type, &draft)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "status": draft.status.as_str(),
                    "operation": if existed { "draft_update" } else { "draft_create" },
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "draft",
                draft.draft_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(draft)
    }

    pub async fn list(
        &self,
        account_id: Option<&str>,
        status: Option<DraftStatus>,
    ) -> Result<Vec<CommunicationDraft>, CommunicationDraftError> {
        let status_str = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                status,
                scheduled_send_at,
                send_attempts,
                last_error,
                metadata,
                created_at,
                updated_at
            FROM communication_drafts
            WHERE ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR status = $2)
            ORDER BY updated_at DESC, draft_id ASC"#,
        )
        .bind(account_id)
        .bind(status_str.as_deref())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_draft).collect()
    }

    pub async fn list_page(
        &self,
        account_id: Option<&str>,
        status: Option<DraftStatus>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<CommunicationDraftListPage, CommunicationDraftError> {
        let limit = validate_limit(limit)?;
        let cursor = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_draft_list_cursor)
            .transpose()?;
        let status_str = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                status,
                scheduled_send_at,
                send_attempts,
                last_error,
                metadata,
                created_at,
                updated_at
            FROM communication_drafts
            WHERE ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR status = $2)
              AND (
                $3::timestamptz IS NULL
                OR updated_at < $3
                OR (updated_at = $3 AND draft_id > $4)
              )
            ORDER BY updated_at DESC, draft_id ASC
            LIMIT $5"#,
        )
        .bind(account_id)
        .bind(status_str.as_deref())
        .bind(cursor.as_ref().map(|value| value.updated_at))
        .bind(cursor.as_ref().map(|value| value.draft_id.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;
        let mut items = rows
            .into_iter()
            .map(row_to_draft)
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > limit as usize;
        if has_more {
            items.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            items.last().map(encode_draft_list_cursor).transpose()?
        } else {
            None
        };
        Ok(CommunicationDraftListPage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn get(
        &self,
        draft_id: &str,
    ) -> Result<Option<CommunicationDraft>, CommunicationDraftError> {
        let row = sqlx::query(
            r#"SELECT
                draft_id,
                account_id,
                identity_id AS persona_id,
                to_participants AS to_recipients,
                cc_participants AS cc_recipients,
                bcc_participants AS bcc_recipients,
                subject,
                body_text,
                body_html,
                in_reply_to,
                message_refs AS message_references,
                status,
                scheduled_send_at,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/evidence.rs`
- Size bytes / Размер в байтах: `1370`
- Included characters / Включено символов: `1370`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::{ObservationStoreError, link_domain_entity_in_transaction};

pub(crate) async fn link_mail_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    base_metadata: Value,
    extra_metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let metadata = merge_metadata(base_metadata, extra_metadata);
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "communications",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}

pub(crate) fn merge_metadata(base_metadata: Value, extra_metadata: Option<Value>) -> Value {
    match extra_metadata {
        Some(extra) if base_metadata.is_object() && extra.is_object() => {
            let mut merged = base_metadata;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => base_metadata,
    }
}
```

### `backend/src/domains/communications/explain.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/explain.rs`
- Size bytes / Размер в байтах: `5449`
- Included characters / Включено символов: `5449`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::messages::ProjectedMessage;

pub struct WhyImportantContext {
    pub reasons: Vec<String>,
}

pub fn explain_importance(message: &ProjectedMessage) -> WhyImportantContext {
    let mut reasons = Vec::new();
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if let Some(score) = message.importance_score {
        reasons.push(format!("importance score is {score}/100"));
    }

    if subject_lower.contains("urgent") || subject_lower.contains("asap") {
        reasons.push("subject contains urgency markers".into());
    }

    let finance_words = ["invoice", "payment", "factura", "amount", "tax", "bill"];
    for w in &finance_words {
        if body_lower.contains(w) || subject_lower.contains(w) {
            reasons.push("contains financial information".into());
            break;
        }
    }

    let legal_words = ["contract", "nda", "agreement", "legal", "liability"];
    for w in &legal_words {
        if body_lower.contains(w) || subject_lower.contains(w) {
            reasons.push("contains legal or contractual content".into());
            break;
        }
    }

    if body_lower.contains('?') {
        reasons.push("contains a question (likely requires reply)".into());
    }

    if body_lower.contains("deadline")
        || body_lower.contains("due date")
        || body_lower.contains("due by")
    {
        reasons.push("mentions a deadline".into());
    }

    let attach_hints = ["attached", "attachment", "see attached", "please find"];
    for hint in &attach_hints {
        if body_lower.contains(hint) {
            reasons.push("references an attachment".into());
            break;
        }
    }

    let junk_hints = ["unsubscribe", "newsletter", "marketing", "promotion"];
    for hint in &junk_hints {
        if body_lower.contains(hint) || subject_lower.contains(hint) {
            reasons.push("appears to be a newsletter or marketing email".into());
            break;
        }
    }

    if reasons.is_empty() {
        reasons.push("no specific importance signals detected".into());
    }

    WhyImportantContext { reasons }
}

pub fn smart_cc_suggestions(message: &ProjectedMessage) -> Vec<String> {
    let mut suggestions = Vec::new();
    let body_lower = message.body_text.to_lowercase();

    if body_lower.contains("invoice")
        || body_lower.contains("factura")
        || body_lower.contains("payment")
    {
        suggestions.push("Consider adding your accountant/bookkeeper to CC".into());
    }
    if body_lower.contains("contract") || body_lower.contains("legal") || body_lower.contains("nda")
    {
        suggestions.push("Consider adding legal counsel to CC".into());
    }
    if body_lower.contains("project")
        && (body_lower.contains("update") || body_lower.contains("status"))
    {
        suggestions.push("Consider adding project stakeholders to CC".into());
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::communications::messages::{LocalMessageState, WorkflowState};
    use chrono::Utc;

    fn test_message(subject: &str, body: &str, score: Option<i16>) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "msg:1".into(),
            raw_record_id: "raw:1".into(),
            observation_id: "observation:1".into(),
            account_id: "a:1".into(),
            provider_record_id: "p:1".into(),
            subject: subject.into(),
            sender: "s@e.com".into(),
            recipients: vec!["r@e.com".into()],
            body_text: body.into(),
            occurred_at: Some(Utc::now()),
            projected_at: Utc::now(),
            channel_kind: "email".into(),
            conversation_id: None,
            sender_display_name: None,
            delivery_state: "received".into(),
            message_metadata: serde_json::json!({}),
            workflow_state: WorkflowState::New,
            importance_score: score,
            ai_category: None,
            ai_summary: None,
            ai_summary_generated_at: None,
            local_state: LocalMessageState::Active,
            local_state_changed_at: None,
            local_state_reason: None,
        }
    }

    #[test]
    fn explain_importance_urgent_email() {
        let msg = test_message("URGENT: Need response", "Please reply ASAP", Some(80));
        let ctx = explain_importance(&msg);
        assert!(ctx.reasons.iter().any(|r| r.contains("urgency")));
        assert!(ctx.reasons.iter().any(|r| r.contains("80")));
    }

    #[test]
    fn explain_importance_finance_email() {
        let msg = test_message(
            "Invoice attached",
            "Here is the invoice for payment",
            Some(70),
        );
        let ctx = explain_importance(&msg);
        assert!(ctx.reasons.iter().any(|r| r.contains("financial")));
    }

    #[test]
    fn smart_cc_for_invoice() {
        let msg = test_message("Invoice", "Please process this invoice for payment", None);
        let suggestions = smart_cc_suggestions(&msg);
        assert!(suggestions.iter().any(|s| s.contains("accountant")));
    }

    #[test]
    fn smart_cc_for_legal() {
        let msg = test_message("Contract", "Please review the NDA and legal terms", None);
        let suggestions = smart_cc_suggestions(&msg);
        assert!(suggestions.iter().any(|s| s.contains("legal counsel")));
    }
}
```

### `backend/src/domains/communications/export.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/export.rs`
- Size bytes / Размер в байтах: `3112`
- Included characters / Включено символов: `3112`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::messages::{MessageProjectionError, MessageProjectionStore};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore,
};

#[derive(Debug, Clone)]
pub struct CommunicationExport {
    pub format: ExportFormat,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Eml,
    Markdown,
    Json,
}

impl ExportFormat {
    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Eml => "message/rfc822",
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Json => "application/json",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Eml => "eml",
            ExportFormat::Markdown => "md",
            ExportFormat::Json => "json",
        }
    }
}

pub async fn export_message(
    message_store: &MessageProjectionStore,
    attachment_store: &CommunicationStorageStore,
    message_id: &str,
    format: ExportFormat,
) -> Result<CommunicationExport, CommunicationExportError> {
    let msg = message_store
        .message(message_id)
        .await?
        .ok_or(CommunicationExportError::NotFound)?;
    let attachments = attachment_store.attachments_for_message(message_id).await?;

    let content = match format {
        ExportFormat::Markdown => format!(
            "# {}\n\n**From:** {}\n**To:** {}\n**Date:** {}\n**State:** {}\n\n{}\n\n---\n*{} attachment(s)*",
            msg.subject,
            msg.sender,
            msg.recipients.join(", "),
            msg.occurred_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            msg.workflow_state.as_str(),
            msg.body_text,
            attachments.len(),
        ),
        ExportFormat::Eml => format!(
            "From: {}\r\nTo: {}\r\nSubject: {}\r\nDate: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
            msg.sender,
            msg.recipients.join(", "),
            msg.subject,
            msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
            msg.body_text,
        ),
        ExportFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "message_id": msg.message_id,
            "subject": msg.subject,
            "sender": msg.sender,
            "recipients": msg.recipients,
            "body_text": msg.body_text,
            "occurred_at": msg.occurred_at,
            "workflow_state": msg.workflow_state.as_str(),
            "importance_score": msg.importance_score,
            "ai_category": msg.ai_category,
            "ai_summary": msg.ai_summary,
            "attachment_count": attachments.len(),
        }))
        .unwrap_or_default(),
    };

    Ok(CommunicationExport { format, content })
}

#[derive(Debug, thiserror::Error)]
pub enum CommunicationExportError {
    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),
    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),
    #[error("message not found")]
    NotFound,
}
```

### `backend/src/domains/communications/extract.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/extract.rs`
- Size bytes / Размер в байтах: `5798`
- Included characters / Включено символов: `5797`
- Truncated / Обрезано: `no`

```rust
// §20-21: Task + Note extraction from email via LLM + heuristics
use crate::domains::communications::messages::ProjectedMessage;
use crate::platform::ai_runtime::{AiRuntimePortError, SharedAiRuntimePort};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedTask {
    pub title: String,
    pub due_date: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedNote {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub source: String,
}

#[derive(Clone)]
pub struct EmailExtractService {
    runtime: Option<SharedAiRuntimePort>,
}

impl EmailExtractService {
    pub fn new(runtime: Option<SharedAiRuntimePort>) -> Self {
        Self { runtime }
    }

    pub async fn extract_tasks(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Vec<ExtractedTask>, ExtractError> {
        let mut tasks = Vec::new();
        let body = &message.body_text;

        // Heuristic: look for task-like patterns
        for line in body.lines() {
            let ll = line.trim().to_lowercase();
            let is_task = ll.starts_with("todo:")
                || ll.starts_with("task:")
                || ll.starts_with("- [ ]")
                || ll.contains("action item")
                || ll.contains("please ") && ll.contains(" by ");
            if is_task && ll.len() > 10 {
                let due = extract_due_date(line);
                tasks.push(ExtractedTask {
                    title: line
                        .trim()
                        .trim_start_matches(['-', '[', ']', ' '])
                        .to_owned(),
                    due_date: due,
                    assignee: None,
                    priority: if ll.contains("urgent") {
                        Some("high".into())
                    } else {
                        Some("normal".into())
                    },
                    source: "heuristic".into(),
                });
            }
        }

        // LLM extraction if available
        if let Some(ref runtime) = self.runtime {
            let prompt = format!(
                "Extract tasks from this email. Return a JSON array of objects with fields: title, due_date (ISO date or null), assignee (or null), priority (high/medium/low).\n\nEmail:\nSubject: {}\nBody:\n{}",
                message.subject,
                truncate(body, 3000)
            );
            if let Ok(result) = runtime.chat(&prompt).await
                && let Ok(mut llm_tasks) =
                    serde_json::from_str::<Vec<ExtractedTask>>(result.content.trim())
            {
                for t in &mut llm_tasks {
                    t.source = "llm".into();
                }
                tasks.extend(llm_tasks);
            }
        }
        Ok(tasks)
    }

    pub async fn extract_notes(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Vec<ExtractedNote>, ExtractError> {
        let mut notes = Vec::new();
        let body = &message.body_text;
        let lower = body.to_lowercase();

        // Heuristic: important information patterns
        let has_finance =
            lower.contains("invoice") || lower.contains("payment") || lower.contains("amount");
        let has_legal =
            lower.contains("contract") || lower.contains("agreement") || lower.contains("nda");
        let has_decision =
            lower.contains("decided") || lower.contains("approved") || lower.contains("confirmed");
        let has_deadline =
            lower.contains("deadline") || lower.contains("due date") || lower.contains("by ");

        if has_finance || has_legal || has_decision || has_deadline {
            let mut tags = Vec::new();
            if has_finance {
                tags.push("finance".into());
            }
            if has_legal {
                tags.push("legal".into());
            }
            if has_decision {
                tags.push("decision".into());
            }

            let preview = body.lines().take(5).collect::<Vec<_>>().join("\n");
            notes.push(ExtractedNote {
                title: message.subject.clone(),
                content: preview,
                tags,
                source: "heuristic".into(),
            });
        }
        Ok(notes)
    }
}

fn extract_due_date(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for prefix in &["by ", "due ", "deadline", "before "] {
        if let Some(pos) = lower.find(prefix) {
            let after = &text[pos + prefix.len()..];
            let rest = after.trim_start_matches([':', ' ']);
            return Some(
                rest.split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimePortError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extract_due_date_by() {
        assert_eq!(
            extract_due_date("Please submit by Friday 5pm"),
            Some("Friday 5pm".into())
        );
    }
    #[test]
    fn extract_due_date_deadline() {
        assert_eq!(
            extract_due_date("Deadline: 2026-06-15 for submission"),
            Some("2026-06-15 for submission".into())
        );
    }
    #[test]
    fn extract_due_date_none() {
        assert_eq!(extract_due_date("Hello, how are you?"), None);
    }
}
```

### `backend/src/domains/communications/finance.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/finance.rs`
- Size bytes / Размер в байтах: `6560`
- Included characters / Включено символов: `6560`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceRecord {
    pub invoice_id: String,
    pub message_id: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub invoice_number: Option<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub counterparty: Option<String>,
    pub tax_id: Option<String>,
    pub status: InvoiceStatus,
    pub linked_project_id: Option<String>,
    pub linked_person_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Received,
    Recognized,
    NeedsReview,
    Approved,
    Paid,
    Closed,
    Rejected,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvoiceStatus::Received => "received",
            InvoiceStatus::Recognized => "recognized",
            InvoiceStatus::NeedsReview => "needs_review",
            InvoiceStatus::Approved => "approved",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Closed => "closed",
            InvoiceStatus::Rejected => "rejected",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "received" => Some(InvoiceStatus::Received),
            "recognized" => Some(InvoiceStatus::Recognized),
            "needs_review" => Some(InvoiceStatus::NeedsReview),
            "approved" => Some(InvoiceStatus::Approved),
            "paid" => Some(InvoiceStatus::Paid),
            "closed" => Some(InvoiceStatus::Closed),
            "rejected" => Some(InvoiceStatus::Rejected),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct CommunicationFinanceStore {
    pool: PgPool,
}

impl CommunicationFinanceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_invoice(
        &self,
        invoice: &NewInvoiceRecord,
    ) -> Result<InvoiceRecord, CommunicationFinanceError> {
        invoice.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO communication_invoices (invoice_id, message_id, amount, currency, invoice_number, issue_date, due_date, counterparty, tax_id, status, linked_project_id, linked_person_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (invoice_id) DO UPDATE SET
                message_id = EXCLUDED.message_id, amount = EXCLUDED.amount, currency = EXCLUDED.currency,
                invoice_number = EXCLUDED.invoice_number, issue_date = EXCLUDED.issue_date,
                due_date = EXCLUDED.due_date, counterparty = EXCLUDED.counterparty,
                tax_id = EXCLUDED.tax_id, status = EXCLUDED.status,
                linked_project_id = EXCLUDED.linked_project_id, linked_person_id = EXCLUDED.linked_person_id,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING invoice_id, message_id, amount, currency, invoice_number, issue_date, due_date, counterparty, tax_id, status, linked_project_id, linked_person_id, metadata, created_at, updated_at"#,
        )
        .bind(&invoice.invoice_id).bind(invoice.message_id.as_deref()).bind(invoice.amount)
        .bind(invoice.currency.as_deref()).bind(invoice.invoice_number.as_deref())
        .bind(invoice.issue_date).bind(invoice.due_date).bind(invoice.counterparty.as_deref())
        .bind(invoice.tax_id.as_deref()).bind(invoice.status.as_str())
        .bind(invoice.linked_project_id.as_deref()).bind(invoice.linked_person_id.as_deref())
        .bind(&invoice.metadata).fetch_one(&self.pool).await?;
        row_to_invoice(row)
    }

    pub async fn list(
        &self,
        status: Option<InvoiceStatus>,
    ) -> Result<Vec<InvoiceRecord>, CommunicationFinanceError> {
        let status_str = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT invoice_id, message_id, amount, currency, invoice_number, issue_date, due_date, counterparty, tax_id, status, linked_project_id, linked_person_id, metadata, created_at, updated_at
            FROM communication_invoices WHERE ($1::text IS NULL OR status = $1) ORDER BY COALESCE(due_date, created_at) DESC"#,
        ).bind(status_str.as_deref()).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_invoice).collect()
    }
}

fn row_to_invoice(row: PgRow) -> Result<InvoiceRecord, CommunicationFinanceError> {
    let status_str: String = row.try_get("status")?;
    Ok(InvoiceRecord {
        invoice_id: row.try_get("invoice_id")?,
        message_id: row.try_get("message_id")?,
        amount: row.try_get("amount")?,
        currency: row.try_get("currency")?,
        invoice_number: row.try_get("invoice_number")?,
        issue_date: row.try_get("issue_date")?,
        due_date: row.try_get("due_date")?,
        counterparty: row.try_get("counterparty")?,
        tax_id: row.try_get("tax_id")?,
        status: InvoiceStatus::parse(&status_str).unwrap_or(InvoiceStatus::Received),
        linked_project_id: row.try_get("linked_project_id")?,
        linked_person_id: row.try_get("linked_person_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Clone, Debug)]
pub struct NewInvoiceRecord {
    pub invoice_id: String,
    pub message_id: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub invoice_number: Option<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub counterparty: Option<String>,
    pub tax_id: Option<String>,
    pub status: InvoiceStatus,
    pub linked_project_id: Option<String>,
    pub linked_person_id: Option<String>,
    pub metadata: Value,
}

impl NewInvoiceRecord {
    fn validate(&self) -> Result<(), CommunicationFinanceError> {
        if self.invoice_id.trim().is_empty() {
            return Err(CommunicationFinanceError::Invalid("invoice_id empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CommunicationFinanceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid invoice: {0}")]
    Invalid(&'static str),
}
```

### `backend/src/domains/communications/fixtures/export.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export.rs`
- Size bytes / Размер в байтах: `1653`
- Included characters / Включено символов: `1653`
- Truncated / Обрезано: `no`

```rust
mod body;
mod encoded_words;
mod encoding;
mod errors;
mod headers;
mod models;
mod raw_payload;
mod redaction;
mod rfc822;
mod text;

use crate::domains::communications::sources::FixtureCommunicationSourceMessage;
use crate::platform::communications::EmailSyncBatch;

pub use self::errors::EmailFixtureExportError;
pub use self::models::{EmailFixtureExportOptions, EmailFixturePrivacyMode};
use self::raw_payload::raw_rfc822_bytes;
use self::redaction::redact_message;
use self::rfc822::parse_rfc822_message;

pub fn export_fixture_messages_from_sync_batch(
    batch: &EmailSyncBatch,
    options: EmailFixtureExportOptions,
) -> Result<Vec<FixtureCommunicationSourceMessage>, EmailFixtureExportError> {
    batch
        .messages
        .iter()
        .map(|message| {
            let raw = raw_rfc822_bytes(&message.payload)?;
            let parsed = parse_rfc822_message(&raw)?;
            let parsed = match options.privacy_mode {
                EmailFixturePrivacyMode::Redacted => redact_message(
                    &message.provider_record_id,
                    &message.source_fingerprint,
                    message.occurred_at,
                    parsed,
                ),
            };

            Ok(FixtureCommunicationSourceMessage {
                provider_record_id: message.provider_record_id.clone(),
                subject: parsed.subject,
                from: parsed.from,
                to: parsed.to,
                sent_at: message.occurred_at,
                body_text: parsed.body_text,
                source_fingerprint: message.source_fingerprint.clone(),
            })
        })
        .collect()
}
```

### `backend/src/domains/communications/fixtures/export/body.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/body.rs`
- Size bytes / Размер в байтах: `2753`
- Included characters / Включено символов: `2753`
- Truncated / Обрезано: `no`

```rust
use super::encoding::decode_transfer_body;
use super::headers::{content_type_parameter, header_value, parse_headers};
use super::rfc822::split_headers_and_body;
use super::text::normalize_body_text;

pub(super) fn body_text_from_part(headers: &[(String, String)], body: &str) -> String {
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    if content_type.to_ascii_lowercase().starts_with("multipart/")
        && let Some(boundary) = content_type_parameter(&content_type, "boundary")
        && let Some(text) = first_text_plain_multipart_part(&boundary, body)
    {
        return text;
    }

    let decoded = decode_transfer_body(
        body,
        header_value(headers, "content-transfer-encoding")
            .unwrap_or_default()
            .as_str(),
    );
    if content_type.to_ascii_lowercase().starts_with("text/html") {
        return strip_html_tags(&decoded);
    }

    normalize_body_text(&decoded)
}

fn first_text_plain_multipart_part(boundary: &str, body: &str) -> Option<String> {
    let delimiter = format!("--{boundary}");
    for raw_part in body.split(&delimiter).skip(1) {
        let part = raw_part.trim_start_matches("\r\n").trim_start_matches('\n');
        if part.starts_with("--") {
            break;
        }
        let Ok((headers, nested_body)) = split_headers_and_body(part) else {
            continue;
        };
        let headers = parse_headers(headers);
        if is_text_plain_non_attachment(&headers) {
            return Some(normalize_body_text(&decode_transfer_body(
                nested_body,
                header_value(&headers, "content-transfer-encoding")
                    .unwrap_or_default()
                    .as_str(),
            )));
        }
    }

    None
}

fn is_text_plain_non_attachment(headers: &[(String, String)]) -> bool {
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    let content_disposition = header_value(headers, "content-disposition").unwrap_or_default();
    let normalized_content_type = content_type.to_ascii_lowercase();
    let normalized_disposition = content_disposition.to_ascii_lowercase();
    normalized_content_type.starts_with("text/plain")
        && !normalized_disposition.contains("attachment")
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
```

### `backend/src/domains/communications/fixtures/export/encoded_words.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/encoded_words.rs`
- Size bytes / Размер в байтах: `2035`
- Included characters / Включено символов: `2035`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use super::encoding::decode_quoted_printable;

pub(super) fn decode_rfc2047_words(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find("=?") {
        output.push_str(&rest[..start]);
        let candidate = &rest[start + 2..];
        let Some(charset_end) = candidate.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
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
        append_decoded_word(
            &mut output,
            rest,
            charset_end,
            encoding_end,
            encoded_end,
            encoding,
        );
        rest = &candidate[encoded_end + 2..];
    }

    output.push_str(rest);
    output
}

fn append_decoded_word(
    output: &mut String,
    rest: &str,
    charset_end: usize,
    encoding_end: usize,
    encoded_end: usize,
    encoding: &str,
) {
    let candidate = &rest[2 + charset_end + 1 + encoding_end + 1..];
    let encoded = &candidate[..encoded_end];
    let decoded = match encoding.to_ascii_lowercase().as_str() {
        "b" => BASE64_STANDARD
            .decode(encoded)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .ok(),
        "q" => Some(decode_quoted_printable(&encoded.replace('_', " "))),
        _ => None,
    };

    if let Some(decoded) = decoded {
        output.push_str(&decoded);
    } else {
        output.push_str(&rest[..2 + charset_end + 1 + encoding_end + 1 + encoded_end + 2]);
    }
}
```

### `backend/src/domains/communications/fixtures/export/encoding.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/encoding.rs`
- Size bytes / Размер в байтах: `1895`
- Included characters / Включено символов: `1895`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

pub(super) fn decode_transfer_body(body: &str, transfer_encoding: &str) -> String {
    match transfer_encoding.trim().to_ascii_lowercase().as_str() {
        "base64" => decode_base64_body(body),
        "quoted-printable" => decode_quoted_printable(body),
        _ => body.to_owned(),
    }
}

fn decode_base64_body(body: &str) -> String {
    let compact = body
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    BASE64_STANDARD
        .decode(compact)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_else(|_| body.to_owned())
}

pub(super) fn decode_quoted_printable(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'=' {
            if bytes.get(index + 1) == Some(&b'\r') && bytes.get(index + 2) == Some(&b'\n') {
                index += 3;
                continue;
            }
            if bytes.get(index + 1) == Some(&b'\n') {
                index += 2;
                continue;
            }
            if let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2))
                && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
```

### `backend/src/domains/communications/fixtures/export/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/errors.rs`
- Size bytes / Размер в байтах: `385`
- Included characters / Включено символов: `385`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailFixtureExportError {
    #[error("email sync payload missing raw_rfc822_base64")]
    MissingRawRfc822,

    #[error("email sync payload raw_rfc822_base64 is invalid base64: {0}")]
    InvalidRawBase64(base64::DecodeError),

    #[error("raw RFC822 message does not contain a header/body separator")]
    MalformedRfc822,
}
```

### `backend/src/domains/communications/fixtures/export/headers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/headers.rs`
- Size bytes / Размер в байтах: `1496`
- Included characters / Включено символов: `1496`
- Truncated / Обрезано: `no`

```rust
use super::encoded_words::decode_rfc2047_words;

pub(super) fn parse_headers(header_block: &str) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = Vec::new();

    for line in header_block.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some((_, value)) = headers.last_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
            continue;
        }

        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }

    headers
}

pub(super) fn header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| decode_rfc2047_words(value.trim()))
}

pub(super) fn split_address_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

pub(super) fn content_type_parameter(content_type: &str, parameter: &str) -> Option<String> {
    for part in content_type.split(';').skip(1) {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case(parameter) {
            return Some(value.trim().trim_matches('"').to_owned());
        }
    }

    None
}
```

### `backend/src/domains/communications/fixtures/export/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/models.rs`
- Size bytes / Размер в байтах: `606`
- Included characters / Включено символов: `606`
- Truncated / Обрезано: `no`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailFixturePrivacyMode {
    Redacted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmailFixtureExportOptions {
    pub privacy_mode: EmailFixturePrivacyMode,
}

impl Default for EmailFixtureExportOptions {
    fn default() -> Self {
        Self {
            privacy_mode: EmailFixturePrivacyMode::Redacted,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ParsedRfc822Message {
    pub(super) subject: String,
    pub(super) from: String,
    pub(super) to: Vec<String>,
    pub(super) body_text: String,
}
```

### `backend/src/domains/communications/fixtures/export/raw_payload.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/raw_payload.rs`
- Size bytes / Размер в байтах: `506`
- Included characters / Включено символов: `506`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde_json::Value;

use super::errors::EmailFixtureExportError;

pub(super) fn raw_rfc822_bytes(payload: &Value) -> Result<Vec<u8>, EmailFixtureExportError> {
    let raw = payload
        .get("raw_rfc822_base64")
        .and_then(Value::as_str)
        .ok_or(EmailFixtureExportError::MissingRawRfc822)?;
    BASE64_STANDARD
        .decode(raw)
        .map_err(EmailFixtureExportError::InvalidRawBase64)
}
```

### `backend/src/domains/communications/fixtures/export/redaction.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/redaction.rs`
- Size bytes / Размер в байтах: `2015`
- Included characters / Включено символов: `2015`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use super::models::ParsedRfc822Message;
use super::text::non_empty_recipients;

pub(super) fn redact_message(
    provider_record_id: &str,
    source_fingerprint: &str,
    occurred_at: Option<DateTime<Utc>>,
    parsed: ParsedRfc822Message,
) -> ParsedRfc822Message {
    let mut recipients = BTreeSet::new();
    for recipient in parsed.to {
        recipients.insert(redacted_email("recipient", &recipient));
    }

    ParsedRfc822Message {
        subject: format!("Redacted subject {}", short_hash(&parsed.subject)),
        from: redacted_email("sender", &parsed.from),
        to: non_empty_recipients(recipients.into_iter().collect()),
        body_text: redacted_body_text(
            provider_record_id,
            source_fingerprint,
            occurred_at,
            &parsed.body_text,
        ),
    }
}

fn redacted_email(prefix: &str, input: &str) -> String {
    format!("{prefix}-{}@example.invalid", short_hash(input))
}

fn redacted_body_text(
    provider_record_id: &str,
    source_fingerprint: &str,
    occurred_at: Option<DateTime<Utc>>,
    body_text: &str,
) -> String {
    let occurred_at = occurred_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_owned());
    format!(
        "Redacted body fixture for provider_record_id_hash={}, source_fingerprint={}, occurred_at={}, original_chars={}.",
        short_hash(provider_record_id),
        source_fingerprint,
        occurred_at,
        body_text.chars().count()
    )
}

fn short_hash(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    hex_lower(&digest[..6])
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
```

### `backend/src/domains/communications/fixtures/export/rfc822.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/rfc822.rs`
- Size bytes / Размер в байтах: `1512`
- Included characters / Включено символов: `1512`
- Truncated / Обрезано: `no`

```rust
use super::body::body_text_from_part;
use super::errors::EmailFixtureExportError;
use super::headers::{header_value, parse_headers, split_address_list};
use super::models::ParsedRfc822Message;
use super::text::{non_empty_or_default, non_empty_recipients};

pub(super) fn parse_rfc822_message(
    raw: &[u8],
) -> Result<ParsedRfc822Message, EmailFixtureExportError> {
    let raw = String::from_utf8_lossy(raw);
    let (header_block, body) = split_headers_and_body(&raw)?;
    let headers = parse_headers(header_block);

    let subject = header_value(&headers, "subject").unwrap_or_else(|| "(no subject)".to_owned());
    let from =
        header_value(&headers, "from").unwrap_or_else(|| "unknown@example.invalid".to_owned());
    let to = split_address_list(&header_value(&headers, "to").unwrap_or_default());
    let body_text = body_text_from_part(&headers, body);

    Ok(ParsedRfc822Message {
        subject: non_empty_or_default(subject, "(no subject)"),
        from: non_empty_or_default(from, "unknown@example.invalid"),
        to: non_empty_recipients(to),
        body_text: non_empty_or_default(body_text, "(empty body)"),
    })
}

pub(super) fn split_headers_and_body(raw: &str) -> Result<(&str, &str), EmailFixtureExportError> {
    if let Some((headers, body)) = raw.split_once("\r\n\r\n") {
        return Ok((headers, body));
    }
    if let Some((headers, body)) = raw.split_once("\n\n") {
        return Ok((headers, body));
    }

    Err(EmailFixtureExportError::MalformedRfc822)
}
```

### `backend/src/domains/communications/fixtures/export/text.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/export/text.rs`
- Size bytes / Размер в байтах: `808`
- Included characters / Включено символов: `808`
- Truncated / Обрезано: `no`

```rust
pub(super) fn normalize_body_text(input: &str) -> String {
    input
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

pub(super) fn non_empty_or_default(value: String, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

pub(super) fn non_empty_recipients(recipients: Vec<String>) -> Vec<String> {
    let recipients = recipients
        .into_iter()
        .map(|recipient| recipient.trim().to_owned())
        .filter(|recipient| !recipient.is_empty())
        .collect::<Vec<_>>();
    if recipients.is_empty() {
        vec!["recipient-unknown@example.invalid".to_owned()]
    } else {
        recipients
    }
}
```

### `backend/src/domains/communications/fixtures/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/fixtures/mod.rs`
- Size bytes / Размер в байтах: `16`
- Included characters / Включено символов: `16`
- Truncated / Обрезано: `no`

```rust
pub mod export;
```

### `backend/src/domains/communications/flags.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/flags.rs`
- Size bytes / Размер в байтах: `11610`
- Included characters / Включено символов: `11610`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage,
};

/// Pin/snooze/label operations on messages stored in message_metadata JSONB.
pub struct MessageFlags;

impl MessageFlags {
    const PINNED_KEY: &'static str = "pinned";
    const IMPORTANT_KEY: &'static str = "important";
    const SNOOZE_UNTIL_KEY: &'static str = "snooze_until";
    const LABELS_KEY: &'static str = "labels";
    const IS_MUTED_KEY: &'static str = "muted";

    pub fn is_pinned(message: &ProjectedMessage) -> bool {
        message
            .message_metadata
            .get(Self::PINNED_KEY)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn is_important(message: &ProjectedMessage) -> bool {
        message
            .message_metadata
            .get(Self::IMPORTANT_KEY)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn snooze_until(message: &ProjectedMessage) -> Option<DateTime<Utc>> {
        message
            .message_metadata
            .get(Self::SNOOZE_UNTIL_KEY)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
    }

    pub fn labels(message: &ProjectedMessage) -> Vec<String> {
        message
            .message_metadata
            .get(Self::LABELS_KEY)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn is_muted(message: &ProjectedMessage) -> bool {
        message
            .message_metadata
            .get(Self::IS_MUTED_KEY)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub async fn toggle_pin(
        store: &MessageProjectionStore,
        message_id: &str,
    ) -> Result<bool, MessageFlagsError> {
        Self::toggle_pin_with_observation(store, message_id, None, "message_flag_update", None)
            .await
    }

    pub async fn toggle_pin_with_observation(
        store: &MessageProjectionStore,
        message_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<bool, MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let currently = Self::is_pinned(&msg);
        let mut meta = msg.message_metadata.clone();
        meta[Self::PINNED_KEY] = serde_json::Value::Bool(!currently);
        store
            .set_message_metadata_with_observation(
                message_id,
                &meta,
                observation_id,
                relationship_kind,
                link_metadata,
            )
            .await?;
        Ok(!currently)
    }

    pub async fn toggle_important(
        store: &MessageProjectionStore,
        message_id: &str,
    ) -> Result<bool, MessageFlagsError> {
        Self::toggle_important_with_observation(
            store,
            message_id,
            None,
            "message_flag_update",
            None,
        )
        .await
    }

    pub async fn toggle_important_with_observation(
        store: &MessageProjectionStore,
        message_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<bool, MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let currently = Self::is_important(&msg);
        let mut meta = msg.message_metadata.clone();
        meta[Self::IMPORTANT_KEY] = serde_json::Value::Bool(!currently);
        store
            .set_message_metadata_with_observation(
                message_id,
                &meta,
                observation_id,
                relationship_kind,
                link_metadata,
            )
            .await?;
        Ok(!currently)
    }

    pub async fn snooze(
        store: &MessageProjectionStore,
        message_id: &str,
        until: DateTime<Utc>,
    ) -> Result<(), MessageFlagsError> {
        Self::snooze_with_observation(store, message_id, until, None, "message_flag_update", None)
            .await
    }

    pub async fn snooze_with_observation(
        store: &MessageProjectionStore,
        message_id: &str,
        until: DateTime<Utc>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<(), MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let mut meta = msg.message_metadata.clone();
        meta[Self::SNOOZE_UNTIL_KEY] = serde_json::Value::String(until.to_rfc3339());
        store
            .set_message_metadata_with_observation(
                message_id,
                &meta,
                observation_id,
                relationship_kind,
                link_metadata,
            )
            .await?;
        Ok(())
    }

    pub async fn add_label(
        store: &MessageProjectionStore,
        message_id: &str,
        label: &str,
    ) -> Result<(), MessageFlagsError> {
        Self::add_label_with_observation(
            store,
            message_id,
            label,
            None,
            "message_flag_update",
            None,
        )
        .await
    }

    pub async fn add_label_with_observation(
        store: &MessageProjectionStore,
        message_id: &str,
        label: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<(), MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let mut labels = Self::labels(&msg);
        if !labels.contains(&label.to_owned()) {
            labels.push(label.to_owned());
        }
        let mut meta = msg.message_metadata.clone();
        meta[Self::LABELS_KEY] = serde_json::to_value(&labels).unwrap_or_default();
        store
            .set_message_metadata_with_observation(
                message_id,
                &meta,
                observation_id,
                relationship_kind,
                link_metadata,
            )
            .await?;
        Ok(())
    }

    pub async fn remove_label(
        store: &MessageProjectionStore,
        message_id: &str,
        label: &str,
    ) -> Result<(), MessageFlagsError> {
        Self::remove_label_with_observation(
            store,
            message_id,
            label,
            None,
            "message_flag_update",
            None,
        )
        .await
    }

    pub async fn remove_label_with_observation(
        store: &MessageProjectionStore,
        message_id: &str,
        label: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<(), MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let mut labels = Self::labels(&msg);
        labels.retain(|l| l != label);
        let mut meta = msg.message_metadata.clone();
        meta[Self::LABELS_KEY] = serde_json::to_value(&labels).unwrap_or_default();
        store
            .set_message_metadata_with_observation(
                message_id,
                &meta,
                observation_id,
                relationship_kind,
                link_metadata,
            )
            .await?;
        Ok(())
    }

    pub async fn toggle_mute(
        store: &MessageProjectionStore,
        message_id: &str,
    ) -> Result<bool, MessageFlagsError> {
        Self::toggle_mute_with_observation(store, message_id, None, "message_flag_update", None)
            .await
    }

    pub async fn toggle_mute_with_observation(
        store: &MessageProjectionStore,
        message_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<bool, MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let currently = Self::is_muted(&msg);
        let mut meta = msg.message_metadata.clone();
        meta[Self::IS_MUTED_KEY] = serde_json::Value::Bool(!currently);
        store
            .set_message_metadata_with_observation(
                message_id,
                &meta,
                observation_id,
                relationship_kind,
                link_metadata,
            )
            .await?;
        Ok(!currently)
    }
}

#[derive(Debug, Error)]
pub enum MessageFlagsError {
    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),
    #[error("message not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::domains::communications::messages::{LocalMessageState, WorkflowState};
    use chrono::Utc;
    use serde_json::json;

    fn test_message(meta: Value) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "m:1".into(),
            raw_record_id: "r:1".into(),
            observation_id: "observation:1".into(),
            account_id: "a:1".into(),
            provider_record_id: "p:1".into(),
            subject: "S".into(),
            sender: "s@e.com".into(),
            recipients: vec!["r@e.com".into()],
            body_text: "B".into(),
            occurred_at: Some(Utc::now()),
            projected_at: Utc::now(),
            channel_kind: "email".into(),
            conversation_id: None,
            sender_display_name: None,
            delivery_state: "received".into(),
            message_metadata: meta,
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
    fn is_pinned_detects_flag() {
        let msg = test_message(serde_json::json!({"pinned": true}));
        assert!(MessageFlags::is_pinned(&msg));
        let msg2 = test_message(serde_json::json!({}));
        assert!(!MessageFlags::is_pinned(&msg2));
    }

    #[test]
    fn is_important_detects_flag() {
        let msg = test_message(serde_json::json!({"important": true}));
        assert!(MessageFlags::is_important(&msg));
        let msg2 = test_message(serde_json::json!({}));
        assert!(!MessageFlags::is_important(&msg2));
    }

    #[test]
    fn labels_extracts_array() {
        let msg = test_message(serde_json::json!({"labels": ["finance", "urgent"]}));
        assert_eq!(MessageFlags::labels(&msg), vec!["finance", "urgent"]);
    }

    #[test]
    fn is_muted_detects_flag() {
        let msg = test_message(serde_json::json!({"muted": true}));
        assert!(MessageFlags::is_muted(&msg));
    }

    #[test]
    fn snooze_until_parses_datetime() {
        let msg = test_message(serde_json::json!({"snooze_until": "2026-06-08T10:00:00+00:00"}));
        assert!(MessageFlags::snooze_until(&msg).is_some());
    }
}
```

### `backend/src/domains/communications/folders.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/folders.rs`
- Size bytes / Размер в байтах: `30486`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
use crate::domains::communications::messages::{LocalMessageState, WorkflowState};
use crate::platform::events::EventStore;
use crate::platform::observations::ObservationStoreError;

mod cursors;
mod events;

use cursors::{
    decode_folder_list_cursor, decode_folder_message_cursor, encode_folder_list_cursor,
    encode_folder_message_cursor,
};
use events::{
    EVENT_TYPE_FOLDER_CREATED, EVENT_TYPE_FOLDER_DELETED, EVENT_TYPE_FOLDER_UPDATED, folder_event,
    folder_message_event,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommunicationFolder {
    pub folder_id: String,
    pub account_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub message_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewCommunicationFolder {
    pub folder_id: Option<String>,
    pub account_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateCommunicationFolder {
    pub account_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationFolderListPage {
    pub items: Vec<CommunicationFolder>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct CommunicationFolderListQuery<'a> {
    pub account_id: Option<&'a str>,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FolderMessageOperation {
    Copy,
    Move,
}

impl FolderMessageOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Move => "move",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FolderMessageActionResponse {
    pub operation: FolderMessageOperation,
    pub folder_id: String,
    pub message_id: String,
    pub message: FolderMessage,
}

#[derive(Clone, Debug, Serialize)]
pub struct FolderMessage {
    pub folder_id: String,
    pub message_id: String,
    pub account_id: String,
    pub subject: String,
    pub sender: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub workflow_state: WorkflowState,
    pub local_state: LocalMessageState,
    pub added_at: DateTime<Utc>,
    pub attachment_count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct FolderMessagePage {
    pub items: Vec<FolderMessage>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct FolderMessageListQuery<'a> {
    pub folder_id: &'a str,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone)]
pub struct CommunicationFolderStore {
    pool: PgPool,
}

impl CommunicationFolderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        query: CommunicationFolderListQuery<'_>,
    ) -> Result<CommunicationFolderListPage, CommunicationFolderError> {
        let limit = validate_limit(query.limit);
        let account_id = normalize_optional(query.account_id.map(str::to_owned))?;
        let cursor = query
            .cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_folder_list_cursor)
            .transpose()?;
        let fetch_limit = limit + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                f.folder_id,
                f.account_id,
                f.name,
                f.description,
                f.color,
                f.sort_order,
                count(fm.message_id)::BIGINT AS message_count,
                f.created_at,
                f.updated_at
            FROM communication_folders f
            LEFT JOIN communication_folder_messages fm ON fm.folder_id = f.folder_id
            WHERE ($1::text IS NULL OR f.account_id = $1)
              AND (
                $2::integer IS NULL
                OR f.sort_order > $2
                OR (f.sort_order = $2 AND lower(f.name) > $3)
                OR (f.sort_order = $2 AND lower(f.name) = $3 AND f.folder_id > $4)
              )
            GROUP BY f.folder_id, f.account_id, f.name, f.description, f.color, f.sort_order, f.created_at, f.updated_at
            ORDER BY f.sort_order ASC, lower(f.name) ASC, f.folder_id ASC
            LIMIT $5
            "#,
        )
        .bind(account_id.as_deref())
        .bind(cursor.as_ref().map(|value| value.sort_order))
        .bind(cursor.as_ref().map(|value| value.name_lower.as_str()))
        .bind(cursor.as_ref().map(|value| value.folder_id.as_str()))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;
        let has_more = rows.len() > limit as usize;
        let items = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_folder)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            items.last().map(encode_folder_list_cursor).transpose()?
        } else {
            None
        };

        Ok(CommunicationFolderListPage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn create(
        &self,
        input: NewCommunicationFolder,
    ) -> Result<CommunicationFolder, CommunicationFolderError> {
        self.create_with_observation(input, None, "folder_upsert", None)
            .await
    }

    pub async fn create_with_observation(
        &self,
        input: NewCommunicationFolder,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommunicationFolder, CommunicationFolderError> {
        let normalized = NormalizedCommunicationFolderInput::from_new(input)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, normalized.account_id.as_deref())
            .await?;
        let folder = insert_folder(&mut transaction, &normalized).await?;
        let event = folder_event(EVENT_TYPE_FOLDER_CREATED, &folder)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                serde_json::json!({
                    "operation": "folder_create",
                    "name": folder.name,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "mail_folder",
                folder.folder_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(folder)
    }

    pub async fn update(
        &self,
        folder_id: &str,
        update: UpdateCommunicationFolder,
    ) -> Result<Option<CommunicationFolder>, CommunicationFolderError> {
        self.update_with_observation(folder_id, update, None, "folder_upsert", None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        folder_id: &str,
        update: UpdateCommunicationFolder,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<CommunicationFolder>, CommunicationFolderError> {
        let folder_id = normalize_required("folder_id", folder_id)?;
        let normalized = NormalizedCommunicationFolderUpdate::from_update(update)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, normalized.account_id.as_deref())
            .await?;
        let Some(folder) = update_folder(&mut transaction, &folder_id, &normalized).await? else {
            transaction.rollback().await?;
            return Ok(None);
        };
        let event = folder_event(EVENT_TYPE_FOLDER_UPDATED, &folder)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                serde_json::json!({
                    "operation": "folder_update",
                    "name": folder.name,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "mail_folder",
                folder.folder_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(Some(folder))
    }

    pub async fn delete(&self, folder_id: &str) -> Result<bool, CommunicationFolderError> {
        self.delete_with_observation(folder_id, None, "folder_delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        folder_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<bool, CommunicationFolderError> {
        let folder_id = normalize_required("folder_id", folder_id)?;
        let mut transaction = self.pool.begin().await?;
        let Some(folder) = delete_folder(&mut transaction, &folder_id).await? else {
            transaction.rollback().await?;
            return Ok(false);
        };
        let event = folder_event(EVENT_TYPE_FOLDER_DELETED, &folder)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                serde_json::json!({
                    "operation": "folder_delete",
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "mail_folder",
                folder_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(true)
    }

    pub async fn list_messages(
        &self,
        query: FolderMessageListQuery<'_>,
    ) -> Result<FolderMessagePage, CommunicationFolderError> {
        let folder_id = normalize_required("folder_id", query.folder_id)?;
        let limit = validate_limit(query.limit);
        let cursor = query
            .cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_folder_message_cursor)
            .transpose()?;
        let fetch_limit = limit + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                fm.folder_id,
                fm.message_id,
                fm.added_at,
                m.account_id,
                m.subject,
                m.sender,
                m.occurred_at,
                m.projected_at,
                m.workflow_state,
                m.local_state,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/folders/cursors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/folders/cursors.rs`
- Size bytes / Размер в байтах: `2434`
- Included characters / Включено символов: `2434`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{CommunicationFolder, CommunicationFolderError, FolderMessage};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct FolderListCursor {
    pub(super) sort_order: i32,
    pub(super) name_lower: String,
    pub(super) folder_id: String,
}

pub(super) fn encode_folder_list_cursor(
    folder: &CommunicationFolder,
) -> Result<String, CommunicationFolderError> {
    let cursor = FolderListCursor {
        sort_order: folder.sort_order,
        name_lower: folder.name.to_lowercase(),
        folder_id: folder.folder_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| CommunicationFolderError::InvalidCursor)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub(super) fn decode_folder_list_cursor(
    cursor: &str,
) -> Result<FolderListCursor, CommunicationFolderError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CommunicationFolderError::InvalidCursor)?;
    let cursor: FolderListCursor =
        serde_json::from_slice(&bytes).map_err(|_| CommunicationFolderError::InvalidCursor)?;
    if cursor.name_lower.trim().is_empty() || cursor.folder_id.trim().is_empty() {
        return Err(CommunicationFolderError::InvalidCursor);
    }

    Ok(cursor)
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct FolderMessageCursor {
    pub(super) added_at: DateTime<Utc>,
    pub(super) message_id: String,
}

pub(super) fn encode_folder_message_cursor(
    message: &FolderMessage,
) -> Result<String, CommunicationFolderError> {
    let cursor = FolderMessageCursor {
        added_at: message.added_at,
        message_id: message.message_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| CommunicationFolderError::InvalidCursor)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub(super) fn decode_folder_message_cursor(
    cursor: &str,
) -> Result<FolderMessageCursor, CommunicationFolderError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CommunicationFolderError::InvalidCursor)?;
    let cursor: FolderMessageCursor =
        serde_json::from_slice(&bytes).map_err(|_| CommunicationFolderError::InvalidCursor)?;
    if cursor.message_id.trim().is_empty() {
        return Err(CommunicationFolderError::InvalidCursor);
    }

    Ok(cursor)
}
```
