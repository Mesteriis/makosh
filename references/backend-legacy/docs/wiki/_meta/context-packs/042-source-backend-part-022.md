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

- Chunk ID / ID чанка: `042-source-backend-part-022`
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

### `backend/src/domains/communications/saved_searches.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/saved_searches.rs`
- Size bytes / Размер в байтах: `27739`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::domains::communications::evidence::{link_mail_entity_in_transaction, merge_metadata};
use crate::domains::communications::messages::{LocalMessageState, WorkflowState};
use crate::domains::communications::saved_search_counts::{
    count_messages_for_saved_search, load_message_counts_for_saved_searches,
};
use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::ObservationStoreError;

const EVENT_TYPE_CREATED: &str = "mail.saved_search.created";
const EVENT_TYPE_UPDATED: &str = "mail.saved_search.updated";
const EVENT_TYPE_DELETED: &str = "mail.saved_search.deleted";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommunicationSavedSearch {
    pub saved_search_id: String,
    pub name: String,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub query: String,
    pub workflow_state: Option<WorkflowState>,
    pub local_state: LocalMessageState,
    pub channel_kind: Option<String>,
    pub is_smart_folder: bool,
    pub sort_order: i32,
    pub message_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewCommunicationSavedSearch {
    pub saved_search_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub query: Option<String>,
    pub workflow_state: Option<WorkflowState>,
    pub local_state: Option<LocalMessageState>,
    pub channel_kind: Option<String>,
    pub is_smart_folder: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateCommunicationSavedSearch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub query: Option<String>,
    pub workflow_state: Option<WorkflowState>,
    pub local_state: Option<LocalMessageState>,
    pub channel_kind: Option<String>,
    pub is_smart_folder: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationSavedSearchListPage {
    pub items: Vec<CommunicationSavedSearch>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug)]
pub struct CommunicationSavedSearchListQuery<'a> {
    pub account_id: Option<&'a str>,
    pub is_smart_folder: Option<bool>,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone)]
pub struct CommunicationSavedSearchStore {
    pool: PgPool,
}

#[derive(Clone, Debug)]
pub(crate) struct SavedSearchRecord {
    pub(crate) saved_search_id: String,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) account_id: Option<String>,
    pub(crate) query: String,
    pub(crate) workflow_state: Option<WorkflowState>,
    pub(crate) local_state: LocalMessageState,
    pub(crate) channel_kind: Option<String>,
    pub(crate) is_smart_folder: bool,
    pub(crate) sort_order: i32,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

impl CommunicationSavedSearchStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        query: CommunicationSavedSearchListQuery<'_>,
    ) -> Result<CommunicationSavedSearchListPage, CommunicationSavedSearchError> {
        let limit = query.limit.clamp(1, 1000);
        let account_id = normalize_optional(query.account_id.map(str::to_owned))?;
        let cursor = query
            .cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_saved_search_list_cursor)
            .transpose()?;
        let fetch_limit = limit + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                s.saved_search_id,
                s.name,
                s.description,
                s.account_id,
                s.query_text,
                s.workflow_state,
                s.local_state,
                s.channel_kind,
                s.is_smart_folder,
                s.sort_order,
                s.created_at,
                s.updated_at
            FROM communication_saved_searches s
            WHERE ($1::text IS NULL OR s.account_id = $1)
              AND ($2::boolean IS NULL OR s.is_smart_folder = $2)
              AND (
                $3::boolean IS NULL
                OR ($3 = TRUE AND s.is_smart_folder = FALSE)
                OR (
                  s.is_smart_folder = $3
                  AND (
                    s.sort_order > $4
                    OR (s.sort_order = $4 AND lower(s.name) > $5)
                    OR (s.sort_order = $4 AND lower(s.name) = $5 AND s.updated_at < $6)
                    OR (s.sort_order = $4 AND lower(s.name) = $5 AND s.updated_at = $6 AND s.saved_search_id > $7)
                  )
                )
              )
            ORDER BY s.is_smart_folder DESC, s.sort_order ASC, lower(s.name) ASC, s.updated_at DESC, s.saved_search_id ASC
            LIMIT $8
            "#,
        )
        .bind(account_id.as_deref())
        .bind(query.is_smart_folder)
        .bind(cursor.as_ref().map(|value| value.is_smart_folder))
        .bind(cursor.as_ref().map(|value| value.sort_order))
        .bind(cursor.as_ref().map(|value| value.name_lower.as_str()))
        .bind(cursor.as_ref().map(|value| value.updated_at))
        .bind(cursor.as_ref().map(|value| value.saved_search_id.as_str()))
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;

        let has_more = rows.len() > limit as usize;
        let records = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_saved_search_record)
            .collect::<Result<Vec<_>, _>>()?;
        let counts = load_message_counts_for_saved_searches(&self.pool, &records).await?;
        let items = records
            .into_iter()
            .map(|record| {
                let count = *counts.get(record.saved_search_id.as_str()).unwrap_or(&0);
                Ok(saved_search_from_record(record, count))
            })
            .collect::<Result<Vec<_>, CommunicationSavedSearchError>>()?;
        let next_cursor = if has_more {
            items
                .last()
                .map(encode_saved_search_list_cursor)
                .transpose()?
        } else {
            None
        };

        Ok(CommunicationSavedSearchListPage {
            items,
            next_cursor,
            has_more,
        })
    }

    pub async fn create(
        &self,
        input: NewCommunicationSavedSearch,
    ) -> Result<CommunicationSavedSearch, CommunicationSavedSearchError> {
        self.create_with_observation(input, None, "saved_search_upsert", None)
            .await
    }

    pub async fn create_with_observation(
        &self,
        input: NewCommunicationSavedSearch,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommunicationSavedSearch, CommunicationSavedSearchError> {
        let normalized = NormalizedMailSavedSearchInput::from_new(input)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, normalized.account_id.as_deref())
            .await?;
        let saved_search = insert_saved_search(&mut transaction, &normalized).await?;
        let event = saved_search_event(EVENT_TYPE_CREATED, &saved_search)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "operation": "saved_search_create",
                    "name": saved_search.name,
                    "is_smart_folder": saved_search.is_smart_folder,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "saved_search",
                saved_search.saved_search_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(saved_search)
    }

    pub async fn update(
        &self,
        saved_search_id: &str,
        update: UpdateCommunicationSavedSearch,
    ) -> Result<Option<CommunicationSavedSearch>, CommunicationSavedSearchError> {
        self.update_with_observation(saved_search_id, update, None, "saved_search_upsert", None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        saved_search_id: &str,
        update: UpdateCommunicationSavedSearch,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<CommunicationSavedSearch>, CommunicationSavedSearchError> {
        let saved_search_id = normalize_required("saved_search_id", saved_search_id)?;
        let normalized = NormalizedMailSavedSearchUpdate::from_update(update)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(&mut transaction, normalized.account_id.as_deref())
            .await?;
        let Some(saved_search) =
            update_saved_search(&mut transaction, &saved_search_id, &normalized).await?
        else {
            transaction.rollback().await?;
            return Ok(None);
        };
        let event = saved_search_event(EVENT_TYPE_UPDATED, &saved_search)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "operation": "saved_search_update",
                    "name": saved_search.name,
                    "is_smart_folder": saved_search.is_smart_folder,
                }),
                metadata,
            );
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "saved_search",
                saved_search.saved_search_id.clone(),
                relationship_kind,
                link_metadata,
                None,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(Some(saved_search))
    }

    pub async fn delete(
        &self,
        saved_search_id: &str,
    ) -> Result<bool, CommunicationSavedSearchError> {
        self.delete_with_observation(saved_search_id, None, "saved_search_delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        saved_search_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<bool, CommunicationSavedSearchError> {
        let saved_search_id = normalize_required("saved_search_id", saved_search_id)?;
        let mut transaction = self.pool.begin().await?;
        let Some(saved_search) = delete_saved_search(&mut transaction, &saved_search_id).await?
        else {
            transaction.rollback().await?;
            return Ok(false);
        };
        let event = saved_search_event(EVENT_TYPE_DELETED, &saved_search)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            let link_metadata = merge_metadata(
                json!({
                    "operation": "
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/search.rs`
- Size bytes / Размер в байтах: `1454`
- Included characters / Включено символов: `1454`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::messages::{MessageProjectionStore, ProjectedMessage};
use crate::engines::search::{SearchDocument, SearchIndex, SearchResult};

pub fn project_message_to_search_document(message: &ProjectedMessage) -> SearchDocument {
    SearchDocument {
        object_id: message.message_id.clone(),
        object_kind: "communication_message".to_owned(),
        title: format!("[{}] {}", message.sender, message.subject),
        body: message.body_text.clone(),
    }
}

pub async fn index_messages(
    index: &SearchIndex,
    store: &MessageProjectionStore,
    limit: i64,
) -> Result<usize, IndexEmailError> {
    let messages = store.recent_messages(limit).await?;
    let count = messages.len();
    for summary in &messages {
        let doc = project_message_to_search_document(&summary.message);
        index.upsert_document(&doc)?;
    }
    Ok(count)
}

pub fn search_emails(
    index: &SearchIndex,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, IndexEmailError> {
    let results = index.search(query, limit)?;
    Ok(results
        .into_iter()
        .filter(|r| r.object_kind == "communication_message")
        .collect())
}

#[derive(Debug, thiserror::Error)]
pub enum IndexEmailError {
    #[error(transparent)]
    Search(#[from] crate::engines::search::SearchError),
    #[error(transparent)]
    Messages(#[from] crate::domains::communications::messages::MessageProjectionError),
}
```

### `backend/src/domains/communications/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/service.rs`
- Size bytes / Размер в байтах: `35`
- Included characters / Включено символов: `35`
- Truncated / Обрезано: `no`

```rust
pub use super::command_service::*;
```

### `backend/src/domains/communications/signatures.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures.rs`
- Size bytes / Размер в байтах: `484`
- Included characters / Включено символов: `484`
- Truncated / Обрезано: `no`

```rust
mod certificate_type;
mod detector;
mod errors;
mod models;
mod provider;
mod rows;
mod storage_kind;
mod store;
#[cfg(test)]
mod tests;
mod trust;

pub use certificate_type::CertificateType;
pub use detector::{SignatureDetection, SignatureDetector};
pub use errors::CertificateError;
pub use models::{CertificateRecord, NewCertificate};
pub use provider::CertificateProvider;
pub use storage_kind::CertificateStorageKind;
pub use store::CertificateStore;
pub use trust::TrustStatus;
```

### `backend/src/domains/communications/signatures/certificate_type.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/certificate_type.rs`
- Size bytes / Размер в байтах: `994`
- Included characters / Включено символов: `994`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateType {
    Smime,
    Pgp,
    PdfSign,
    Cades,
    Xades,
    GostSign,
    Unknown,
}

impl CertificateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Smime => "smime",
            Self::Pgp => "pgp",
            Self::PdfSign => "pdf_sign",
            Self::Cades => "cades",
            Self::Xades => "xades",
            Self::GostSign => "gost_sign",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "smime" => Some(Self::Smime),
            "pgp" => Some(Self::Pgp),
            "pdf_sign" => Some(Self::PdfSign),
            "cades" => Some(Self::Cades),
            "xades" => Some(Self::Xades),
            "gost_sign" => Some(Self::GostSign),
            _ => None,
        }
    }
}
```

### `backend/src/domains/communications/signatures/detector.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/detector.rs`
- Size bytes / Размер в байтах: `1885`
- Included characters / Включено символов: `1885`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde::Serialize;

use super::{CertificateRecord, CertificateType};

#[derive(Clone, Debug, Serialize)]
pub struct SignatureDetection {
    pub has_signature: bool,
    pub signature_type: Option<CertificateType>,
    pub signer_info: Option<String>,
    pub is_valid: Option<bool>,
    pub cert_expiry_warning: Option<String>,
}

pub struct SignatureDetector;

impl SignatureDetector {
    pub fn detect_in_message(body_text: &str, headers: &str) -> SignatureDetection {
        let has_smime = headers.contains("Content-Type: application/pkcs7-mime")
            || headers.contains("Content-Type: application/x-pkcs7-signature");
        let has_pgp = body_text.contains("-----BEGIN PGP SIGNATURE-----")
            || body_text.contains("-----BEGIN PGP MESSAGE-----");

        if has_smime {
            signature_detected(CertificateType::Smime)
        } else if has_pgp {
            signature_detected(CertificateType::Pgp)
        } else {
            SignatureDetection {
                has_signature: false,
                signature_type: None,
                signer_info: None,
                is_valid: None,
                cert_expiry_warning: None,
            }
        }
    }

    pub fn check_expiry_warning(cert: &CertificateRecord) -> Option<String> {
        let until = cert.valid_until?;
        let days = (until - Utc::now()).num_days();
        if days <= 0 {
            Some("Certificate has expired".into())
        } else if days <= 90 {
            Some(format!("Certificate expires in {days} days"))
        } else {
            None
        }
    }
}

fn signature_detected(signature_type: CertificateType) -> SignatureDetection {
    SignatureDetection {
        has_signature: true,
        signature_type: Some(signature_type),
        signer_info: None,
        is_valid: None,
        cert_expiry_warning: None,
    }
}
```

### `backend/src/domains/communications/signatures/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/errors.rs`
- Size bytes / Размер в байтах: `195`
- Included characters / Включено символов: `195`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CertificateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid cert: {0}")]
    Invalid(&'static str),
}
```

### `backend/src/domains/communications/signatures/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/models.rs`
- Size bytes / Размер в байтах: `1783`
- Included characters / Включено символов: `1783`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    CertificateError, CertificateProvider, CertificateStorageKind, CertificateType, TrustStatus,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateRecord {
    pub cert_id: String,
    pub owner_name: String,
    pub issuer: String,
    pub serial_number: Option<String>,
    pub fingerprint_sha256: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub cert_type: CertificateType,
    pub provider: CertificateProvider,
    pub storage_kind: CertificateStorageKind,
    pub storage_ref: Option<String>,
    pub trust_status: TrustStatus,
    pub is_revoked: bool,
    pub usage: Vec<String>,
    pub linked_message_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewCertificate {
    pub cert_id: String,
    pub owner_name: String,
    pub issuer: String,
    pub serial_number: Option<String>,
    pub fingerprint_sha256: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub cert_type: CertificateType,
    pub provider: CertificateProvider,
    pub storage_kind: CertificateStorageKind,
    pub storage_ref: Option<String>,
    pub trust_status: TrustStatus,
    pub is_revoked: bool,
    pub usage: Vec<String>,
    pub linked_message_id: Option<String>,
    pub metadata: Value,
}

impl NewCertificate {
    pub(super) fn validate(&self) -> Result<(), CertificateError> {
        if self.cert_id.trim().is_empty() {
            Err(CertificateError::Invalid("cert_id empty"))
        } else {
            Ok(())
        }
    }
}
```

### `backend/src/domains/communications/signatures/provider.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/provider.rs`
- Size bytes / Размер в байтах: `1276`
- Included characters / Включено символов: `1276`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateProvider {
    Fnmt,
    Dnie,
    CryptoPro,
    Gost,
    AppleKeychain,
    Pkcs12,
    Yubikey,
    UsbToken,
    Other,
}

impl CertificateProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fnmt => "fnmt",
            Self::Dnie => "dnie",
            Self::CryptoPro => "cryptopro",
            Self::Gost => "gost",
            Self::AppleKeychain => "apple_keychain",
            Self::Pkcs12 => "pkcs12",
            Self::Yubikey => "yubikey",
            Self::UsbToken => "usb_token",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "fnmt" => Some(Self::Fnmt),
            "dnie" => Some(Self::Dnie),
            "cryptopro" => Some(Self::CryptoPro),
            "gost" => Some(Self::Gost),
            "apple_keychain" => Some(Self::AppleKeychain),
            "pkcs12" => Some(Self::Pkcs12),
            "yubikey" => Some(Self::Yubikey),
            "usb_token" => Some(Self::UsbToken),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}
```

### `backend/src/domains/communications/signatures/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/rows.rs`
- Size bytes / Размер в байтах: `1848`
- Included characters / Включено символов: `1848`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::{
    CertificateError, CertificateProvider, CertificateRecord, CertificateStorageKind,
    CertificateType, TrustStatus,
};

pub(super) const CERTIFICATE_COLUMNS: &str = "cert_id,owner_name,issuer,serial_number,fingerprint_sha256,valid_from,valid_until,cert_type,provider,storage_kind,storage_ref,trust_status,is_revoked,usage,linked_message_id,metadata,created_at,updated_at";

pub(super) fn row_to_cert(row: PgRow) -> Result<CertificateRecord, CertificateError> {
    Ok(CertificateRecord {
        cert_id: row.try_get("cert_id")?,
        owner_name: row.try_get("owner_name")?,
        issuer: row.try_get("issuer")?,
        serial_number: row.try_get("serial_number")?,
        fingerprint_sha256: row.try_get("fingerprint_sha256")?,
        valid_from: row.try_get("valid_from")?,
        valid_until: row.try_get("valid_until")?,
        cert_type: CertificateType::parse(&row.try_get::<String, _>("cert_type")?)
            .unwrap_or(CertificateType::Unknown),
        provider: CertificateProvider::parse(&row.try_get::<String, _>("provider")?)
            .unwrap_or(CertificateProvider::Other),
        storage_kind: CertificateStorageKind::parse(&row.try_get::<String, _>("storage_kind")?)
            .unwrap_or(CertificateStorageKind::EncryptedVault),
        storage_ref: row.try_get("storage_ref")?,
        trust_status: TrustStatus::parse(&row.try_get::<String, _>("trust_status")?)
            .unwrap_or(TrustStatus::Untrusted),
        is_revoked: row.try_get("is_revoked")?,
        usage: serde_json::from_value(row.try_get("usage")?).unwrap_or_default(),
        linked_message_id: row.try_get("linked_message_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/communications/signatures/storage_kind.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/storage_kind.rs`
- Size bytes / Размер в байтах: `1219`
- Included characters / Включено символов: `1219`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateStorageKind {
    OsKeychain,
    EncryptedVault,
    Pkcs12File,
    PfxFile,
    SmartCard,
    UsbToken,
    ExternalVault,
}

impl CertificateStorageKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OsKeychain => "os_keychain",
            Self::EncryptedVault => "encrypted_vault",
            Self::Pkcs12File => "pkcs12_file",
            Self::PfxFile => "pfx_file",
            Self::SmartCard => "smart_card",
            Self::UsbToken => "usb_token",
            Self::ExternalVault => "external_vault",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "os_keychain" => Some(Self::OsKeychain),
            "encrypted_vault" => Some(Self::EncryptedVault),
            "pkcs12_file" => Some(Self::Pkcs12File),
            "pfx_file" => Some(Self::PfxFile),
            "smart_card" => Some(Self::SmartCard),
            "usb_token" => Some(Self::UsbToken),
            "external_vault" => Some(Self::ExternalVault),
            _ => None,
        }
    }
}
```

### `backend/src/domains/communications/signatures/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/store.rs`
- Size bytes / Размер в байтах: `2978`
- Included characters / Включено символов: `2978`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::rows::{CERTIFICATE_COLUMNS, row_to_cert};
use super::{CertificateError, CertificateRecord, NewCertificate};

#[derive(Clone)]
pub struct CertificateStore {
    pool: PgPool,
}

impl CertificateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        cert: &NewCertificate,
    ) -> Result<CertificateRecord, CertificateError> {
        cert.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO communication_certificates (cert_id, owner_name, issuer, serial_number, fingerprint_sha256, valid_from, valid_until, cert_type, provider, storage_kind, storage_ref, trust_status, is_revoked, usage, linked_message_id, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            ON CONFLICT (cert_id) DO UPDATE SET owner_name=EXCLUDED.owner_name, issuer=EXCLUDED.issuer, valid_from=EXCLUDED.valid_from, valid_until=EXCLUDED.valid_until, trust_status=EXCLUDED.trust_status, is_revoked=EXCLUDED.is_revoked, usage=EXCLUDED.usage, metadata=EXCLUDED.metadata, updated_at=now()
            RETURNING cert_id,owner_name,issuer,serial_number,fingerprint_sha256,valid_from,valid_until,cert_type,provider,storage_kind,storage_ref,trust_status,is_revoked,usage,linked_message_id,metadata,created_at,updated_at"#,
        )
        .bind(&cert.cert_id)
        .bind(&cert.owner_name)
        .bind(&cert.issuer)
        .bind(cert.serial_number.as_deref())
        .bind(cert.fingerprint_sha256.as_deref())
        .bind(cert.valid_from)
        .bind(cert.valid_until)
        .bind(cert.cert_type.as_str())
        .bind(cert.provider.as_str())
        .bind(cert.storage_kind.as_str())
        .bind(cert.storage_ref.as_deref())
        .bind(cert.trust_status.as_str())
        .bind(cert.is_revoked)
        .bind(serde_json::to_value(&cert.usage).unwrap_or_default())
        .bind(cert.linked_message_id.as_deref())
        .bind(&cert.metadata)
        .fetch_one(&self.pool)
        .await?;
        row_to_cert(row)
    }

    pub async fn list(&self) -> Result<Vec<CertificateRecord>, CertificateError> {
        let query = format!(
            "SELECT {CERTIFICATE_COLUMNS} FROM communication_certificates ORDER BY COALESCE(valid_until, created_at) DESC"
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_cert).collect()
    }

    pub async fn expiring_soon(
        &self,
        days: i64,
    ) -> Result<Vec<CertificateRecord>, CertificateError> {
        let query = format!(
            "SELECT {CERTIFICATE_COLUMNS} FROM communication_certificates WHERE valid_until IS NOT NULL AND valid_until BETWEEN now() AND now() + ($1 || ' days')::interval AND is_revoked = false ORDER BY valid_until ASC"
        );
        let rows = sqlx::query(&query).bind(days).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_cert).collect()
    }
}
```

### `backend/src/domains/communications/signatures/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/tests.rs`
- Size bytes / Размер в байтах: `903`
- Included characters / Включено символов: `903`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[test]
fn detect_smime() {
    let result =
        SignatureDetector::detect_in_message("body", "Content-Type: application/pkcs7-mime\n");

    assert!(result.has_signature);
    assert_eq!(result.signature_type, Some(CertificateType::Smime));
}

#[test]
fn detect_pgp() {
    let result = SignatureDetector::detect_in_message(
        "-----BEGIN PGP SIGNATURE-----\nxyz\n-----END PGP SIGNATURE-----",
        "",
    );

    assert!(result.has_signature);
}

#[test]
fn detect_none() {
    let result = SignatureDetector::detect_in_message("plain text", "");

    assert!(!result.has_signature);
}

#[test]
fn cert_types_roundtrip() {
    for cert_type in [
        CertificateType::Smime,
        CertificateType::Cades,
        CertificateType::GostSign,
        CertificateType::Pgp,
    ] {
        assert_eq!(CertificateType::parse(cert_type.as_str()), Some(cert_type));
    }
}
```

### `backend/src/domains/communications/signatures/trust.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/signatures/trust.rs`
- Size bytes / Размер в байтах: `1063`
- Included characters / Включено символов: `1063`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustStatus {
    Trusted,
    Untrusted,
    Expired,
    Revoked,
    PendingVerification,
    SelfSigned,
}

impl TrustStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
            Self::PendingVerification => "pending_verification",
            Self::SelfSigned => "self_signed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "trusted" => Some(Self::Trusted),
            "untrusted" => Some(Self::Untrusted),
            "expired" => Some(Self::Expired),
            "revoked" => Some(Self::Revoked),
            "pending_verification" => Some(Self::PendingVerification),
            "self_signed" => Some(Self::SelfSigned),
            _ => None,
        }
    }
}
```

### `backend/src/domains/communications/sources.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/sources.rs`
- Size bytes / Размер в байтах: `2472`
- Included characters / Включено символов: `2472`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FixtureCommunicationSourceMessage {
    pub provider_record_id: String,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub body_text: String,
    pub source_fingerprint: String,
}

#[derive(Debug, Deserialize)]
struct RawFixtureCommunicationSourceMessage {
    provider_record_id: String,
    subject: String,
    from: String,
    to: Vec<String>,
    sent_at: Option<DateTime<Utc>>,
    body_text: String,
    source_fingerprint: String,
}

pub fn parse_fixture_email_messages(
    input: &str,
) -> Result<Vec<FixtureCommunicationSourceMessage>, FixtureEmailSourceError> {
    let raw_messages: Vec<RawFixtureCommunicationSourceMessage> = serde_json::from_str(input)?;
    raw_messages
        .into_iter()
        .map(validate_fixture_message)
        .collect()
}

fn validate_fixture_message(
    message: RawFixtureCommunicationSourceMessage,
) -> Result<FixtureCommunicationSourceMessage, FixtureEmailSourceError> {
    validate_non_empty("provider_record_id", &message.provider_record_id)?;
    validate_non_empty("subject", &message.subject)?;
    validate_non_empty("from", &message.from)?;
    validate_non_empty("body_text", &message.body_text)?;
    validate_non_empty("source_fingerprint", &message.source_fingerprint)?;
    if message.to.is_empty() {
        return Err(FixtureEmailSourceError::EmptyRecipients);
    }
    for recipient in &message.to {
        validate_non_empty("to", recipient)?;
    }

    Ok(FixtureCommunicationSourceMessage {
        provider_record_id: message.provider_record_id,
        subject: message.subject,
        from: message.from,
        to: message.to,
        sent_at: message.sent_at,
        body_text: message.body_text,
        source_fingerprint: message.source_fingerprint,
    })
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), FixtureEmailSourceError> {
    if value.trim().is_empty() {
        return Err(FixtureEmailSourceError::EmptyField(field_name));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum FixtureEmailSourceError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("to must contain at least one recipient")]
    EmptyRecipients,
}
```

### `backend/src/domains/communications/spf_dkim.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/spf_dkim.rs`
- Size bytes / Размер в байтах: `6325`
- Included characters / Включено символов: `6285`
- Truncated / Обрезано: `no`

```rust
// §7: SPF/DKIM/DMARC header parsing — технические проверки без внешних DNS-запросов
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct AuthResults {
    pub spf: Option<SpfResult>,
    pub dkim: Option<DkimResult>,
    pub dmarc: Option<DmarcResult>,
    pub raw_headers: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpfResult {
    pub result: String,
    pub domain: Option<String>,
    pub ip: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DkimResult {
    pub result: String,
    pub domain: Option<String>,
    pub selector: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DmarcResult {
    pub result: String,
    pub domain: Option<String>,
    pub policy: Option<String>,
}

/// Parse Authentication-Results and Received-SPF headers from raw email headers.
pub fn parse_auth_headers(raw_headers: &str) -> AuthResults {
    let mut spf = None;
    let mut dkim = None;
    let mut dmarc = None;
    let mut raw = Vec::new();

    for line in raw_headers.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("authentication-results:") || lower.starts_with("received-spf:") {
            raw.push(line.to_owned());

            if lower.contains("spf=") {
                let result = extract_value(line, "spf=");
                let domain = extract_value(line, "smtp.mailfrom=")
                    .or_else(|| extract_value(line, "envelope-from="));
                if let Some(res) = result {
                    spf = Some(SpfResult {
                        result: res,
                        domain,
                        ip: None,
                    });
                }
            }
            if lower.contains("dkim=") {
                let result = extract_value(line, "dkim=");
                let domain = extract_value(line, "d=");
                let selector = extract_value(line, "s=");
                if let Some(res) = result {
                    dkim = Some(DkimResult {
                        result: res,
                        domain,
                        selector,
                    });
                }
            }
            if lower.contains("dmarc=") {
                let result = extract_value(line, "dmarc=");
                let domain = extract_value(line, "header.from=");
                let policy = extract_value(line, "p=");
                if let Some(res) = result {
                    dmarc = Some(DmarcResult {
                        result: res,
                        domain,
                        policy,
                    });
                }
            }
        }
    }

    AuthResults {
        spf,
        dkim,
        dmarc,
        raw_headers: raw,
    }
}

fn extract_value(line: &str, prefix: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let pos = lower.find(prefix)?;
    let start = pos + prefix.len();
    let rest = &line[start..];
    let end = rest.find([';', ' ', '\r', '\n']).unwrap_or(rest.len());
    let val = rest[..end].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_owned())
    }
}

#[derive(Debug, Serialize)]
pub struct SpfDkimReport {
    pub has_spf: bool,
    pub spf_pass: bool,
    pub has_dkim: bool,
    pub dkim_pass: bool,
    pub has_dmarc: bool,
    pub dmarc_pass: bool,
    pub is_spoofed: bool,
    pub risk_summary: String,
}

pub fn assess_auth_risk(auth: &AuthResults) -> SpfDkimReport {
    let spf_pass = auth
        .spf
        .as_ref()
        .map(|s| s.result == "pass")
        .unwrap_or(false);
    let dkim_pass = auth
        .dkim
        .as_ref()
        .map(|d| d.result == "pass")
        .unwrap_or(false);
    let dmarc_pass = auth
        .dmarc
        .as_ref()
        .map(|d| d.result == "pass")
        .unwrap_or(false);
    let has_spf = auth.spf.is_some();
    let has_dkim = auth.dkim.is_some();
    let has_dmarc = auth.dmarc.is_some();
    let is_spoofed =
        (has_spf && !spf_pass) || (has_dkim && !dkim_pass) || (has_dmarc && !dmarc_pass);
    let summary = if is_spoofed {
        "Authentication failed: possible spoofing".into()
    } else if has_spf || has_dkim || has_dmarc {
        "Authentication checks passed".into()
    } else {
        "No authentication headers present".into()
    };
    SpfDkimReport {
        has_spf,
        spf_pass,
        has_dkim,
        dkim_pass,
        has_dmarc,
        dmarc_pass,
        is_spoofed,
        risk_summary: summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_spf_pass() {
        let r = parse_auth_headers(
            "Authentication-Results: mx.google.com; spf=pass smtp.mailfrom=alice@example.com",
        );
        assert_eq!(r.spf.as_ref().unwrap().result, "pass");
        assert_eq!(
            r.spf.as_ref().unwrap().domain.as_deref(),
            Some("alice@example.com")
        );
    }
    #[test]
    fn parse_dkim_fail() {
        let r = parse_auth_headers("Authentication-Results: dkim=fail d=evil.com s=default");
        assert_eq!(r.dkim.as_ref().unwrap().result, "fail");
    }
    #[test]
    fn parse_dmarc() {
        let r = parse_auth_headers(
            "Authentication-Results: dmarc=pass header.from=example.com p=reject",
        );
        assert!(r.dmarc.as_ref().unwrap().result == "pass");
    }
    #[test]
    fn spoofed_email_flagged() {
        let auth = AuthResults {
            spf: Some(SpfResult {
                result: "fail".into(),
                domain: None,
                ip: None,
            }),
            dkim: None,
            dmarc: None,
            raw_headers: vec![],
        };
        let risk = assess_auth_risk(&auth);
        assert!(risk.is_spoofed);
    }
    #[test]
    fn clean_email_passes() {
        let auth = AuthResults {
            spf: Some(SpfResult {
                result: "pass".into(),
                domain: None,
                ip: None,
            }),
            dkim: Some(DkimResult {
                result: "pass".into(),
                domain: None,
                selector: None,
            }),
            dmarc: None,
            raw_headers: vec![],
        };
        let risk = assess_auth_risk(&auth);
        assert!(!risk.is_spoofed);
    }
}
```

### `backend/src/domains/communications/storage.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage.rs`
- Size bytes / Размер в байтах: `1012`
- Included characters / Включено символов: `1012`
- Truncated / Обрезано: `no`

```rust
mod blob_store;
mod constants;
mod errors;
mod ids;
mod imports;
mod models;
mod rows;
mod scanner;
mod store;
mod validation;

pub use blob_store::LocalCommunicationBlobStore as LocalCommunicationBlobPort;
pub use blob_store::{LocalCommunicationBlob, LocalCommunicationBlobStore};
pub use errors::{AttachmentSafetyScanError, CommunicationStorageError};
pub use imports::new_communication_attachment_import_id;
pub use models::{
    CommunicationAttachmentDisposition, ImportedCommunicationAttachment,
    NewCommunicationAttachment, NewCommunicationAttachmentImport, NewCommunicationBlob,
    StoredCommunicationAttachment, StoredCommunicationAttachmentWithBlob, StoredCommunicationBlob,
};
pub use scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanRequest, AttachmentSafetyScanStatus,
    AttachmentSafetyScanner, HeuristicAttachmentSafetyScanner, NoopAttachmentSafetyScanner,
};
pub use store::CommunicationStorageStore;
pub use store::CommunicationStorageStore as CommunicationBlobMetadataPort;
```

### `backend/src/domains/communications/storage/blob_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/blob_store.rs`
- Size bytes / Размер в байтах: `4593`
- Included characters / Включено символов: `4593`
- Truncated / Обрезано: `no`

```rust
use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};

use super::constants::{LOCAL_FS_STORAGE_KIND, SHA256_PREFIX};
use super::errors::CommunicationStorageError;
use super::validation::validate_storage_path;

#[derive(Clone, Debug)]
pub struct LocalCommunicationBlobStore {
    root: PathBuf,
}

impl LocalCommunicationBlobStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub async fn put_blob(
        &self,
        bytes: &[u8],
    ) -> Result<LocalCommunicationBlob, CommunicationStorageError> {
        let size_bytes =
            i64::try_from(bytes.len()).map_err(|_| CommunicationStorageError::BlobTooLarge)?;
        let digest_hex = sha256_hex(bytes);
        let storage_path = relative_blob_path(&digest_hex);
        let absolute_path = self.root.join(&storage_path);

        if let Some(parent) = absolute_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        if !path_exists(&absolute_path).await? {
            let temp_path = absolute_path.with_extension(format!(
                "tmp-{}-{}",
                std::process::id(),
                Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ));
            tokio::fs::write(&temp_path, bytes).await?;
            tokio::fs::rename(&temp_path, &absolute_path).await?;
        }

        let metadata = tokio::fs::metadata(&absolute_path).await?;
        let actual_size =
            i64::try_from(metadata.len()).map_err(|_| CommunicationStorageError::BlobTooLarge)?;
        if actual_size != size_bytes {
            return Err(CommunicationStorageError::BlobSizeMismatch {
                path: absolute_path,
                expected: size_bytes,
                actual: actual_size,
            });
        }

        Ok(LocalCommunicationBlob {
            storage_kind: LOCAL_FS_STORAGE_KIND.to_owned(),
            storage_path,
            sha256: format!("{SHA256_PREFIX}{digest_hex}"),
            size_bytes,
        })
    }

    pub async fn read_blob(
        &self,
        storage_path: &str,
    ) -> Result<Vec<u8>, CommunicationStorageError> {
        let storage_path = validate_storage_path(storage_path)?;
        Ok(tokio::fs::read(self.root.join(storage_path)).await?)
    }

    pub async fn delete_blob(&self, storage_path: &str) -> Result<bool, CommunicationStorageError> {
        let storage_path = validate_storage_path(storage_path)?;
        let absolute_path = self.root.join(&storage_path);
        match tokio::fs::remove_file(&absolute_path).await {
            Ok(()) => {
                prune_empty_parent_dirs(&self.root, &absolute_path).await?;
                Ok(true)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalCommunicationBlob {
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
}

async fn path_exists(path: &Path) -> Result<bool, std::io::Error> {
    match tokio::fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn relative_blob_path(digest_hex: &str) -> String {
    format!("sha256/{}/{}.blob", &digest_hex[..2], digest_hex)
}

async fn prune_empty_parent_dirs(root: &Path, path: &Path) -> Result<(), std::io::Error> {
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir == root {
            break;
        }
        match tokio::fs::remove_dir(dir).await {
            Ok(()) => current = dir.parent(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(hex_char(byte >> 4));
        encoded.push(hex_char(byte & 0x0f));
    }
    encoded
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex nibble must fit in 0..=15"),
    }
}
```

### `backend/src/domains/communications/storage/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/constants.rs`
- Size bytes / Размер в байтах: `109`
- Included characters / Включено символов: `109`
- Truncated / Обрезано: `no`

```rust
pub(crate) const LOCAL_FS_STORAGE_KIND: &str = "local_fs";
pub(crate) const SHA256_PREFIX: &str = "sha256:";
```

### `backend/src/domains/communications/storage/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/errors.rs`
- Size bytes / Размер в байтах: `1598`
- Included characters / Включено символов: `1598`
- Truncated / Обрезано: `no`

```rust
use std::path::PathBuf;

use crate::platform::observations::ObservationStoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommunicationStorageError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("storage_kind must be local_fs: {0}")]
    InvalidStorageKind(String),

    #[error("storage_path must be relative and stay inside mail blob root: {0}")]
    UnsafeStoragePath(String),

    #[error("sha256 must use sha256:<64 lowercase hex chars>: {0}")]
    InvalidSha256(String),

    #[error("size_bytes must not be negative: {0}")]
    NegativeSizeBytes(i64),

    #[error("blob content is too large to represent as i64 size_bytes")]
    BlobTooLarge,

    #[error("blob size mismatch for {path}: expected {expected}, actual {actual}")]
    BlobSizeMismatch {
        path: PathBuf,
        expected: i64,
        actual: i64,
    },

    #[error("invalid attachment disposition: {0}")]
    InvalidDisposition(String),

    #[error("invalid attachment scan status: {0}")]
    InvalidScanStatus(String),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),

    #[error("not_scanned attachment scan reports must not include engine, checked_at or summary")]
    InvalidNotScannedReport,
}

#[derive(Debug, Error)]
pub enum AttachmentSafetyScanError {
    #[error("attachment safety scanner failed: {0}")]
    Scanner(String),
}
```

### `backend/src/domains/communications/storage/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/ids.rs`
- Size bytes / Размер в байтах: `729`
- Included characters / Включено символов: `729`
- Truncated / Обрезано: `no`

```rust
pub(crate) fn mail_blob_id(sha256: &str) -> String {
    format!("blob:v1:{sha256}")
}

pub(crate) fn mail_attachment_id(message_id: &str, provider_attachment_id: &str) -> String {
    let mut encoded = String::from("att:v1:");
    append_id_component(&mut encoded, message_id);
    encoded.push(':');
    append_id_component(&mut encoded, provider_attachment_id);
    encoded
}

pub(crate) fn communication_attachment_import_id(seed: &str) -> String {
    let mut encoded = String::from("att-import:v1:");
    append_id_component(&mut encoded, seed);
    encoded
}

fn append_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}
```

### `backend/src/domains/communications/storage/imports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/imports.rs`
- Size bytes / Размер в байтах: `21240`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::errors::CommunicationStorageError;
use super::ids::communication_attachment_import_id;
use super::models::{
    ImportedCommunicationAttachment, ImportedCommunicationAttachmentRemovalResult,
    NewCommunicationAttachmentImport,
};
use super::rows::{row_to_imported_attachment, row_to_mail_blob};
use super::store::CommunicationStorageStore;
use super::validation::validate_non_empty;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::platform::storage::{
    ImportedAttachmentRecord, ImportedAttachmentRemovalResult, ImportedAttachmentStoragePort,
    ImportedAttachmentUpsert, LocalBlobRecord, SafetyScanReport, SafetyScanStatus, StorageError,
    StoredBlobRecord,
};

impl CommunicationStorageStore {
    pub async fn upsert_imported_attachment(
        &self,
        import: &NewCommunicationAttachmentImport,
    ) -> Result<ImportedCommunicationAttachment, CommunicationStorageError> {
        self.upsert_imported_attachment_with_observation(import, None, "attachment_import", None)
            .await
    }

    pub async fn upsert_imported_attachment_with_observation(
        &self,
        import: &NewCommunicationAttachmentImport,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ImportedCommunicationAttachment, CommunicationStorageError> {
        let import = import.validate()?;
        let mut transaction = self.pool.begin().await?;
        sqlx::query(imported_attachment_upsert_sql())
            .bind(&import.attachment_id)
            .bind(&import.account_id)
            .bind(&import.channel_kind)
            .bind(&import.blob_id)
            .bind(&import.filename)
            .bind(&import.content_type)
            .bind(import.size_bytes)
            .bind(&import.sha256)
            .bind(&import.source_kind)
            .bind(&import.imported_by)
            .bind(import.scan_report.status.as_str())
            .bind(&import.scan_report.engine)
            .bind(import.scan_report.checked_at)
            .bind(&import.scan_report.summary)
            .bind(&import.scan_report.metadata)
            .bind(&import.metadata)
            .execute(&mut *transaction)
            .await?;

        let sql = imported_attachment_select_sql("i.attachment_id = $1");
        let row = sqlx::query(&sql)
            .bind(&import.attachment_id)
            .fetch_optional(&mut *transaction)
            .await?;
        let imported = row
            .map(row_to_imported_attachment)
            .transpose()?
            .ok_or_else(|| CommunicationStorageError::Sqlx(sqlx::Error::RowNotFound))?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "attachment_import",
                imported.attachment_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "blob_id": imported.blob_id,
                    "scan_status": imported.scan_status.as_str(),
                    "content_type": imported.content_type,
                    "sha256": imported.sha256,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(imported)
    }

    pub async fn imported_attachment_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<Option<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let sql = imported_attachment_select_sql("i.attachment_id = $1");
        let row = sqlx::query(&sql)
            .bind(attachment_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_imported_attachment).transpose()
    }

    pub async fn imported_attachment_by_blob_id(
        &self,
        blob_id: &str,
    ) -> Result<Option<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let blob_id = validate_non_empty("blob_id", blob_id)?;
        let sql = imported_attachment_select_sql("i.blob_id = $1");
        let row = sqlx::query(&sql)
            .bind(blob_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_imported_attachment).transpose()
    }

    pub async fn list_imported_attachments(
        &self,
        account_id: &str,
        source_kind: &str,
        limit: i64,
    ) -> Result<Vec<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let source_kind = validate_non_empty("source_kind", source_kind)?;
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query(
            r#"
            SELECT
                i.attachment_id,
                i.account_id,
                i.channel_kind,
                i.blob_id,
                i.filename,
                i.content_type,
                i.size_bytes,
                i.sha256,
                i.source_kind,
                i.imported_by,
                i.scan_status,
                i.scan_engine,
                i.scan_checked_at,
                i.scan_summary,
                i.scan_metadata,
                i.metadata,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path,
                i.created_at,
                i.updated_at
            FROM communication_attachment_imports i
            JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
            WHERE i.account_id = $1
              AND i.source_kind = $2
            ORDER BY i.created_at DESC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(source_kind)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_imported_attachment).collect()
    }

    pub async fn list_expired_imported_attachments(
        &self,
        account_id: &str,
        source_kind: &str,
        limit: i64,
    ) -> Result<Vec<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let source_kind = validate_non_empty("source_kind", source_kind)?;
        let limit = limit.clamp(1, 500);
        let rows = sqlx::query(
            r#"
            SELECT
                i.attachment_id,
                i.account_id,
                i.channel_kind,
                i.blob_id,
                i.filename,
                i.content_type,
                i.size_bytes,
                i.sha256,
                i.source_kind,
                i.imported_by,
                i.scan_status,
                i.scan_engine,
                i.scan_checked_at,
                i.scan_summary,
                i.scan_metadata,
                i.metadata,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path,
                i.created_at,
                i.updated_at
            FROM communication_attachment_imports i
            JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
            WHERE i.account_id = $1
              AND i.source_kind = $2
              AND NULLIF(i.metadata -> 'retention_policy' ->> 'expires_at', '') IS NOT NULL
              AND (i.metadata -> 'retention_policy' ->> 'expires_at')::timestamptz <= now()
            ORDER BY (i.metadata -> 'retention_policy' ->> 'expires_at')::timestamptz ASC,
                     i.created_at ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(source_kind)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_imported_attachment).collect()
    }

    pub async fn blob_by_id(
        &self,
        blob_id: &str,
    ) -> Result<Option<super::models::StoredCommunicationBlob>, CommunicationStorageError> {
        let blob_id = validate_non_empty("blob_id", blob_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type,
                created_at
            FROM communication_mail_blobs
            WHERE blob_id = $1
            "#,
        )
        .bind(blob_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_mail_blob).transpose()
    }

    pub async fn remove_imported_attachment(
        &self,
        attachment_id: &str,
        account_id: &str,
        source_kind: &str,
    ) -> Result<Option<ImportedCommunicationAttachmentRemovalResult>, CommunicationStorageError>
    {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let account_id = validate_non_empty("account_id", account_id)?;
        let source_kind = validate_non_empty("source_kind", source_kind)?;
        let mut transaction = self.pool.begin().await?;
        let sql = imported_attachment_select_sql(
            "i.attachment_id = $1 AND i.account_id = $2 AND i.source_kind = $3",
        );
        let imported = sqlx::query(&sql)
            .bind(&attachment_id)
            .bind(&account_id)
            .bind(&source_kind)
            .fetch_optional(&mut *transaction)
            .await?
            .map(row_to_imported_attachment)
            .transpose()?;
        let Some(imported_attachment) = imported else {
            transaction.commit().await?;
            return Ok(None);
        };

        sqlx::query(
            r#"
            DELETE FROM communication_attachment_imports
            WHERE attachment_id = $1
              AND account_id = $2
              AND source_kind = $3
            "#,
        )
        .bind(&attachment_id)
        .bind(&account_id)
        .bind(&source_kind)
        .execute(&mut *transaction)
        .await?;

        let blob_still_referenced = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM communication_attachment_imports
                WHERE blob_id = $1
            ) OR EXISTS(
                SELECT 1
                FROM communication_attachments
                WHERE blob_id = $1
            )
            "#,
        )
        .bind(&imported_attachment.blob_id)
        .fetch_one(&mut *transaction)
        .await?;

        let blob_metadata_removed = if blob_still_referenced {
            false
        } else {
            sqlx::query(
                r#"
                DELETE FROM communication_mail_blobs
                WHERE blob_id = $1
                "#,
            )
            .bind(&imported_attachment.blob_id)
            .execute(&mut *transaction)
            .await?;
            true
        };

        transaction.commit().await?;
        Ok(Some(ImportedCommunicationAttachmentRemovalResult {
            imported_attachment,
            blob_metadata_removed,
        }))
    }
}

pub fn new_communication_attachment_import_id(seed: &str) -> String {
    communication_attachment_import_id(seed)
}

fn imported_attachment_upsert_sql() -> &'static str {
    r#"
    INSERT INTO communication_attachment_imports (
        attachment_id,
        account_id,
        channel_kind,
        blob_id,
        filename,
        content_type,
        size_bytes,
        sha256,
        source_kind,
        imported_by,
        scan_status,
        scan_engine,
        scan_checked_at,
        scan_summary,
        scan_metadata,
        metadata,
        updated_at
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, now())
    ON CONFLICT (attachment_id)
    DO UPDATE SET
        account_id = EXCLUDED.account_id,
        channel_kind = EXCLUDED.channel_kind,
        blob_id = EXCLUDED.blob_id
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/storage/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/models.rs`
- Size bytes / Размер в байтах: `12538`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;

use super::blob_store::LocalCommunicationBlob;
use super::errors::CommunicationStorageError;
use super::scanner::{AttachmentSafetyScanReport, AttachmentSafetyScanStatus};
use super::validation::{
    validate_non_empty, validate_sha256, validate_size_bytes, validate_storage_kind,
    validate_storage_path,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewCommunicationBlob {
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
}

impl NewCommunicationBlob {
    pub fn new(
        storage_kind: impl Into<String>,
        storage_path: impl Into<String>,
        sha256: impl Into<String>,
        size_bytes: i64,
    ) -> Self {
        Self {
            storage_kind: storage_kind.into(),
            storage_path: storage_path.into(),
            sha256: sha256.into(),
            size_bytes,
            content_type: None,
        }
    }

    pub fn from_local_blob(blob: &LocalCommunicationBlob) -> Self {
        Self::new(
            &blob.storage_kind,
            &blob.storage_path,
            &blob.sha256,
            blob.size_bytes,
        )
    }

    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<ValidatedCommunicationBlob, CommunicationStorageError> {
        let storage_kind = validate_storage_kind(&self.storage_kind)?;
        let storage_path = validate_storage_path(&self.storage_path)?;
        let sha256 = validate_sha256(&self.sha256)?;
        let size_bytes = validate_size_bytes(self.size_bytes)?;
        let content_type = self
            .content_type
            .as_deref()
            .map(|value| validate_non_empty("content_type", value))
            .transpose()?;

        Ok(ValidatedCommunicationBlob {
            storage_kind,
            storage_path,
            sha256,
            size_bytes,
            content_type,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedCommunicationBlob {
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
    pub(crate) content_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredCommunicationBlob {
    pub blob_id: String,
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCommunicationAttachmentImport {
    pub attachment_id: String,
    pub account_id: Option<String>,
    pub channel_kind: Option<String>,
    pub blob_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub source_kind: String,
    pub imported_by: String,
    pub scan_report: AttachmentSafetyScanReport,
    pub metadata: Value,
}

impl NewCommunicationAttachmentImport {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attachment_id: impl Into<String>,
        blob_id: impl Into<String>,
        content_type: impl Into<String>,
        size_bytes: i64,
        sha256: impl Into<String>,
        imported_by: impl Into<String>,
    ) -> Self {
        Self {
            attachment_id: attachment_id.into(),
            account_id: None,
            channel_kind: None,
            blob_id: blob_id.into(),
            filename: None,
            content_type: content_type.into(),
            size_bytes,
            sha256: sha256.into(),
            source_kind: "local_import".to_owned(),
            imported_by: imported_by.into(),
            scan_report: AttachmentSafetyScanReport::not_scanned(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn account_id(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn channel_kind(mut self, channel_kind: impl Into<String>) -> Self {
        self.channel_kind = Some(channel_kind.into());
        self
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn source_kind(mut self, source_kind: impl Into<String>) -> Self {
        self.source_kind = source_kind.into();
        self
    }

    pub fn scan_report(mut self, scan_report: AttachmentSafetyScanReport) -> Self {
        self.scan_report = scan_report;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(crate) fn validate(&self) -> Result<Self, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", &self.attachment_id)?;
        let account_id = self
            .account_id
            .as_deref()
            .map(|value| validate_non_empty("account_id", value))
            .transpose()?;
        let channel_kind = self
            .channel_kind
            .as_deref()
            .map(|value| validate_non_empty("channel_kind", value))
            .transpose()?;
        let blob_id = validate_non_empty("blob_id", &self.blob_id)?;
        let filename = self
            .filename
            .as_deref()
            .map(|value| validate_non_empty("filename", value))
            .transpose()?;
        let content_type = validate_non_empty("content_type", &self.content_type)?;
        let size_bytes = validate_size_bytes(self.size_bytes)?;
        let sha256 = validate_sha256(&self.sha256)?;
        let source_kind = validate_non_empty("source_kind", &self.source_kind)?;
        let imported_by = validate_non_empty("imported_by", &self.imported_by)?;
        let scan_report = self.scan_report.validate()?;
        if !self.metadata.is_object() {
            return Err(CommunicationStorageError::NonObjectJson("metadata"));
        }

        Ok(Self {
            attachment_id,
            account_id,
            channel_kind,
            blob_id,
            filename,
            content_type,
            size_bytes,
            sha256,
            source_kind,
            imported_by,
            scan_report,
            metadata: self.metadata.clone(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedCommunicationAttachment {
    pub attachment_id: String,
    pub account_id: Option<String>,
    pub channel_kind: Option<String>,
    pub blob_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub source_kind: String,
    pub imported_by: String,
    pub scan_status: AttachmentSafetyScanStatus,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
    pub scan_metadata: Value,
    pub metadata: Value,
    pub storage_kind: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedCommunicationAttachmentRemovalResult {
    pub imported_attachment: ImportedCommunicationAttachment,
    pub blob_metadata_removed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCommunicationAttachment {
    pub message_id: String,
    pub raw_record_id: String,
    pub blob_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub disposition: CommunicationAttachmentDisposition,
    pub scan_report: AttachmentSafetyScanReport,
}

impl NewCommunicationAttachment {
    pub fn new(
        message_id: impl Into<String>,
        raw_record_id: impl Into<String>,
        blob_id: impl Into<String>,
        provider_attachment_id: impl Into<String>,
        content_type: impl Into<String>,
        size_bytes: i64,
        sha256: impl Into<String>,
    ) -> Self {
        Self {
            message_id: message_id.into(),
            raw_record_id: raw_record_id.into(),
            blob_id: blob_id.into(),
            provider_attachment_id: provider_attachment_id.into(),
            filename: None,
            content_type: content_type.into(),
            size_bytes,
            sha256: sha256.into(),
            disposition: CommunicationAttachmentDisposition::Unknown,
            scan_report: AttachmentSafetyScanReport::not_scanned(),
        }
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn disposition(mut self, disposition: CommunicationAttachmentDisposition) -> Self {
        self.disposition = disposition;
        self
    }

    pub fn scan_report(mut self, scan_report: AttachmentSafetyScanReport) -> Self {
        self.scan_report = scan_report;
        self
    }

    pub(crate) fn validate(
        &self,
    ) -> Result<ValidatedCommunicationAttachment, CommunicationStorageError> {
        let message_id = validate_non_empty("message_id", &self.message_id)?;
        let raw_record_id = validate_non_empty("raw_record_id", &self.raw_record_id)?;
        let blob_id = validate_non_empty("blob_id", &self.blob_id)?;
        let provider_attachment_id =
            validate_non_empty("provider_attachment_id", &self.provider_attachment_id)?;
        let filename = self
            .filename
            .as_deref()
            .map(|value| validate_non_empty("filename", value))
            .transpose()?;
        let content_type = validate_non_empty("content_type", &self.content_type)?;
        let size_bytes = validate_size_bytes(self.size_bytes)?;
        let sha256 = validate_sha256(&self.sha256)?;
        let scan_report = self.scan_report.validate()?;

        Ok(ValidatedCommunicationAttachment {
            message_id,
            raw_record_id,
            blob_id,
            provider_attachment_id,
            filename,
            content_type,
            size_bytes,
            sha256,
            disposition: self.disposition,
            scan_report,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ValidatedCommunicationAttachment {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) blob_id: String,
    pub(crate) provider_attachment_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) disposition: CommunicationAttachmentDisposition,
    pub(crate) scan_report: AttachmentSafetyScanReport,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredCommunicationAttachment {
    pub attachment_id: String,
    pub message_id: String,
    pub raw_record_id: String,
    pub blob_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub disposition: CommunicationAttachmentDisposition,
    pub scan_status: AttachmentSafetyScanStatus,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
    pub scan_metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredCommunicationAttachmentWithBlob {
    pub attachment: StoredCommunicationAttachment,
    pub storage_kind: String,
    pub storage_path: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationAttachmentDisposition {
    Attachment,
    Inline,
    Unknown,
}

impl CommunicationAttachmentDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attachment => "attachment",
            Self::Inli
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/storage/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/rows.rs`
- Size bytes / Размер в байтах: `3753`
- Included characters / Включено символов: `3753`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::CommunicationStorageError;
use super::models::{
    CommunicationAttachmentDisposition, ImportedCommunicationAttachment,
    StoredCommunicationAttachment, StoredCommunicationAttachmentWithBlob, StoredCommunicationBlob,
};
use super::scanner::AttachmentSafetyScanStatus;

pub(crate) fn row_to_mail_blob(
    row: PgRow,
) -> Result<StoredCommunicationBlob, CommunicationStorageError> {
    Ok(StoredCommunicationBlob {
        blob_id: row.try_get("blob_id")?,
        storage_kind: row.try_get("storage_kind")?,
        storage_path: row.try_get("storage_path")?,
        sha256: row.try_get("sha256")?,
        size_bytes: row.try_get("size_bytes")?,
        content_type: row.try_get("content_type")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(crate) fn row_to_mail_attachment(
    row: PgRow,
) -> Result<StoredCommunicationAttachment, CommunicationStorageError> {
    let disposition: String = row.try_get("disposition")?;
    let scan_status: String = row.try_get("scan_status")?;

    Ok(StoredCommunicationAttachment {
        attachment_id: row.try_get("attachment_id")?,
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        blob_id: row.try_get("blob_id")?,
        provider_attachment_id: row.try_get("provider_attachment_id")?,
        filename: row.try_get("filename")?,
        content_type: row.try_get("content_type")?,
        size_bytes: row.try_get("size_bytes")?,
        sha256: row.try_get("sha256")?,
        disposition: CommunicationAttachmentDisposition::try_from(disposition.as_str())?,
        scan_status: AttachmentSafetyScanStatus::try_from(scan_status.as_str())?,
        scan_engine: row.try_get("scan_engine")?,
        scan_checked_at: row.try_get("scan_checked_at")?,
        scan_summary: row.try_get("scan_summary")?,
        scan_metadata: row.try_get("scan_metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(crate) fn row_to_mail_attachment_with_blob(
    row: PgRow,
) -> Result<StoredCommunicationAttachmentWithBlob, CommunicationStorageError> {
    let storage_kind: String = row.try_get("blob_storage_kind")?;
    let storage_path: String = row.try_get("blob_storage_path")?;
    Ok(StoredCommunicationAttachmentWithBlob {
        attachment: row_to_mail_attachment(row)?,
        storage_kind,
        storage_path,
    })
}

pub(crate) fn row_to_imported_attachment(
    row: PgRow,
) -> Result<ImportedCommunicationAttachment, CommunicationStorageError> {
    let scan_status: String = row.try_get("scan_status")?;
    Ok(ImportedCommunicationAttachment {
        attachment_id: row.try_get("attachment_id")?,
        account_id: row.try_get("account_id")?,
        channel_kind: row.try_get("channel_kind")?,
        blob_id: row.try_get("blob_id")?,
        filename: row.try_get("filename")?,
        content_type: row.try_get("content_type")?,
        size_bytes: row.try_get("size_bytes")?,
        sha256: row.try_get("sha256")?,
        source_kind: row.try_get("source_kind")?,
        imported_by: row.try_get("imported_by")?,
        scan_status: AttachmentSafetyScanStatus::try_from(scan_status.as_str())?,
        scan_engine: row.try_get("scan_engine")?,
        scan_checked_at: row.try_get("scan_checked_at")?,
        scan_summary: row.try_get("scan_summary")?,
        scan_metadata: row.try_get("scan_metadata")?,
        metadata: row.try_get("metadata")?,
        storage_kind: row.try_get("blob_storage_kind")?,
        storage_path: row.try_get("blob_storage_path")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/communications/storage/scanner.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/scanner.rs`
- Size bytes / Размер в байтах: `11170`
- Included characters / Включено символов: `11170`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::errors::{AttachmentSafetyScanError, CommunicationStorageError};
use super::validation::validate_non_empty;

#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentSafetyScanReport {
    pub status: AttachmentSafetyScanStatus,
    pub engine: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub summary: Option<String>,
    pub metadata: Value,
}

impl AttachmentSafetyScanReport {
    pub fn not_scanned() -> Self {
        Self {
            status: AttachmentSafetyScanStatus::NotScanned,
            engine: None,
            checked_at: None,
            summary: None,
            metadata: json!({}),
        }
    }

    pub(crate) fn validate(&self) -> Result<Self, CommunicationStorageError> {
        let engine = self
            .engine
            .as_deref()
            .map(|value| validate_non_empty("scan_engine", value))
            .transpose()?;
        let summary = self
            .summary
            .as_deref()
            .map(|value| validate_non_empty("scan_summary", value))
            .transpose()?;
        if !self.metadata.is_object() {
            return Err(CommunicationStorageError::NonObjectJson("scan_metadata"));
        }

        if self.status == AttachmentSafetyScanStatus::NotScanned
            && (engine.is_some() || self.checked_at.is_some() || summary.is_some())
        {
            return Err(CommunicationStorageError::InvalidNotScannedReport);
        }

        Ok(Self {
            status: self.status,
            engine,
            checked_at: self.checked_at,
            summary,
            metadata: self.metadata.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSafetyScanStatus {
    NotScanned,
    Clean,
    Suspicious,
    Malicious,
    Failed,
}

impl AttachmentSafetyScanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotScanned => "not_scanned",
            Self::Clean => "clean",
            Self::Suspicious => "suspicious",
            Self::Malicious => "malicious",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for AttachmentSafetyScanStatus {
    type Error = CommunicationStorageError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not_scanned" => Ok(Self::NotScanned),
            "clean" => Ok(Self::Clean),
            "suspicious" => Ok(Self::Suspicious),
            "malicious" => Ok(Self::Malicious),
            "failed" => Ok(Self::Failed),
            other => Err(CommunicationStorageError::InvalidScanStatus(
                other.to_owned(),
            )),
        }
    }
}

pub struct AttachmentSafetyScanRequest<'a> {
    pub provider_attachment_id: &'a str,
    pub filename: Option<&'a str>,
    pub content_type: &'a str,
    pub size_bytes: i64,
    pub sha256: &'a str,
    pub storage_kind: &'a str,
    pub storage_path: &'a str,
    pub bytes: &'a [u8],
}

pub trait AttachmentSafetyScanner {
    fn scan(
        &self,
        request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicAttachmentSafetyScanner;

impl AttachmentSafetyScanner for HeuristicAttachmentSafetyScanner {
    fn scan(
        &self,
        request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
        let extension = normalized_extension(request.filename);
        let content_type = normalized_content_type(request.content_type);
        let mut reasons = Vec::new();
        let mut status = AttachmentSafetyScanStatus::NotScanned;

        if has_executable_magic(request.bytes) {
            status = AttachmentSafetyScanStatus::Malicious;
            reasons.push("executable_magic");
        }

        if let Some(extension) = extension.as_deref() {
            if is_active_content_extension(extension) {
                status = AttachmentSafetyScanStatus::Malicious;
                reasons.push("active_content_extension");
            } else if is_macro_document_extension(extension) {
                status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
                reasons.push("macro_enabled_document_extension");
            }
        }

        if let Some(extension) = extension.as_deref()
            && is_mime_extension_mismatch(&content_type, extension)
        {
            status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
            reasons.push("mime_extension_mismatch");
        }

        if status == AttachmentSafetyScanStatus::NotScanned {
            return Ok(AttachmentSafetyScanReport::not_scanned());
        }

        Ok(AttachmentSafetyScanReport {
            status,
            engine: Some("makosh_heuristic_v1".to_owned()),
            checked_at: Some(Utc::now()),
            summary: Some(scan_summary(status).to_owned()),
            metadata: json!({
                "reasons": reasons,
                "content_type": content_type,
                "filename_extension": extension,
                "size_bytes": request.size_bytes,
            }),
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoopAttachmentSafetyScanner;

impl AttachmentSafetyScanner for NoopAttachmentSafetyScanner {
    fn scan(
        &self,
        _request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
        Ok(AttachmentSafetyScanReport::not_scanned())
    }
}

fn normalized_extension(filename: Option<&str>) -> Option<String> {
    let filename = filename?.trim();
    let basename = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .trim();
    let (_, extension) = basename.rsplit_once('.')?;
    let extension = extension.trim().to_ascii_lowercase();
    (!extension.is_empty()).then_some(extension)
}

fn normalized_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase()
}

fn has_executable_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(b"MZ") || bytes.starts_with(b"\x7fELF")
}

fn is_active_content_extension(extension: &str) -> bool {
    matches!(
        extension,
        "app"
            | "bat"
            | "cmd"
            | "com"
            | "dll"
            | "dmg"
            | "exe"
            | "hta"
            | "jar"
            | "jse"
            | "js"
            | "msi"
            | "ps1"
            | "scr"
            | "vbe"
            | "vbs"
            | "wsf"
    )
}

fn is_macro_document_extension(extension: &str) -> bool {
    matches!(
        extension,
        "docm" | "dotm" | "xlsm" | "xltm" | "pptm" | "potm"
    )
}

fn is_mime_extension_mismatch(content_type: &str, extension: &str) -> bool {
    let expected = expected_extensions_for_content_type(content_type);
    !expected.is_empty() && !expected.contains(&extension)
}

fn expected_extensions_for_content_type(content_type: &str) -> &'static [&'static str] {
    match content_type {
        "application/pdf" => &["pdf"],
        "application/zip" => &["zip"],
        "image/jpeg" => &["jpg", "jpeg"],
        "image/png" => &["png"],
        "text/csv" => &["csv"],
        "text/plain" => &["txt", "text", "log", "csv"],
        _ => &[],
    }
}

fn max_scan_status(
    current: AttachmentSafetyScanStatus,
    candidate: AttachmentSafetyScanStatus,
) -> AttachmentSafetyScanStatus {
    if scan_status_rank(candidate) > scan_status_rank(current) {
        candidate
    } else {
        current
    }
}

fn scan_status_rank(status: AttachmentSafetyScanStatus) -> u8 {
    match status {
        AttachmentSafetyScanStatus::NotScanned => 0,
        AttachmentSafetyScanStatus::Clean => 1,
        AttachmentSafetyScanStatus::Suspicious => 2,
        AttachmentSafetyScanStatus::Failed => 3,
        AttachmentSafetyScanStatus::Malicious => 4,
    }
}

fn scan_summary(status: AttachmentSafetyScanStatus) -> &'static str {
    match status {
        AttachmentSafetyScanStatus::Malicious => "Executable payload detected",
        AttachmentSafetyScanStatus::Suspicious => "Attachment metadata requires safety review",
        AttachmentSafetyScanStatus::Failed => "Attachment safety scan failed",
        AttachmentSafetyScanStatus::Clean | AttachmentSafetyScanStatus::NotScanned => {
            "Attachment was not scanned by a safety backend"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(
        filename: Option<&'a str>,
        content_type: &'a str,
        bytes: &'a [u8],
    ) -> AttachmentSafetyScanRequest<'a> {
        AttachmentSafetyScanRequest {
            provider_attachment_id: "part-1",
            filename,
            content_type,
            size_bytes: bytes.len() as i64,
            sha256: "sha256:fixture",
            storage_kind: "local_fs",
            storage_path: "aa/bb/blob",
            bytes,
        }
    }

    #[test]
    fn heuristic_scanner_leaves_unmatched_attachments_not_scanned() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(Some("invoice.txt"), "text/plain", b"hello"))
            .expect("scan report");

        assert_eq!(report, AttachmentSafetyScanReport::not_scanned());
    }

    #[test]
    fn heuristic_scanner_marks_executable_payloads_malicious() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(
                Some("invoice.pdf"),
                "application/pdf",
                b"MZ\x90\x00fake portable executable",
            ))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Malicious);
        assert_eq!(report.engine.as_deref(), Some("makosh_heuristic_v1"));
        assert!(report.checked_at.is_some());
        assert_eq!(
            report.summary.as_deref(),
            Some("Executable payload detected")
        );
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["executable_magic"])
        );
    }

    #[test]
    fn heuristic_scanner_marks_mime_filename_mismatch_suspicious() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(
                Some("invoice.pdf"),
                "application/zip",
                b"PK\x03\x04fake zip",
            ))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(report.engine.as_deref(), Some("makosh_heuristic_v1"));
        assert!(report.checked_at.is_some());
        assert_eq!(
            report.summary.as_deref(),
            Some("Attachment metadata requires safety review")
        );
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["mime_extension_mismatch"])
        );
    }
}
```
