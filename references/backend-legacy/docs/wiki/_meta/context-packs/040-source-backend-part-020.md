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

- Chunk ID / ID чанка: `040-source-backend-part-020`
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

### `backend/src/domains/communications/folders/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/folders/events.rs`
- Size bytes / Размер в байтах: `2728`
- Included characters / Включено символов: `2728`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use super::{
    CommunicationFolder, CommunicationFolderError, FolderMessageActionResponse,
    FolderMessageOperation,
};
use crate::platform::events::NewEventEnvelope;

pub(super) const EVENT_TYPE_FOLDER_CREATED: &str = "mail.folder.created";
pub(super) const EVENT_TYPE_FOLDER_UPDATED: &str = "mail.folder.updated";
pub(super) const EVENT_TYPE_FOLDER_DELETED: &str = "mail.folder.deleted";

const EVENT_TYPE_MESSAGE_COPIED: &str = "mail.folder_message.copied";
const EVENT_TYPE_MESSAGE_MOVED: &str = "mail.folder_message.moved";

pub(super) fn folder_event(
    event_type: &str,
    folder: &CommunicationFolder,
) -> Result<NewEventEnvelope, CommunicationFolderError> {
    Ok(NewEventEnvelope::builder(
        generate_folder_event_id(event_type, &folder.folder_id),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_folder_api" }),
        json!({
            "kind": "mail_folder",
            "id": folder.folder_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(serde_json::to_value(folder)?)
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": folder.folder_id,
    }))
    .correlation_id(folder.folder_id.clone())
    .build()?)
}

pub(super) fn folder_message_event(
    response: &FolderMessageActionResponse,
) -> Result<NewEventEnvelope, CommunicationFolderError> {
    let event_type = match response.operation {
        FolderMessageOperation::Copy => EVENT_TYPE_MESSAGE_COPIED,
        FolderMessageOperation::Move => EVENT_TYPE_MESSAGE_MOVED,
    };
    let subject_id = format!("{}:{}", response.folder_id, response.message_id);
    Ok(NewEventEnvelope::builder(
        generate_folder_event_id(event_type, &subject_id),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_folder_api" }),
        json!({
            "kind": "mail_folder_message",
            "id": subject_id,
            "folder_id": response.folder_id,
            "message_id": response.message_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(serde_json::to_value(response)?)
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": response.folder_id,
    }))
    .correlation_id(response.message_id.clone())
    .build()?)
}

fn generate_folder_event_id(event_type: &str, subject_id: &str) -> String {
    format!(
        "mail_folder_event:{event_type}:{subject_id}:{:x}",
        system_time_nanos()
    )
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
```

### `backend/src/domains/communications/import.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/import.rs`
- Size bytes / Размер в байтах: `3821`
- Included characters / Включено символов: `3821`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use thiserror::Error;

use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionStore, NewRawCommunicationRecord,
    StoredRawCommunicationRecord,
};
use crate::domains::communications::sources::{
    FixtureEmailSourceError, parse_fixture_email_messages,
};

const EMAIL_MESSAGE_RECORD_KIND: &str = "email_message";

pub struct FixtureEmailImportRequest {
    pub account_id: String,
    pub import_batch_id: String,
    pub fixture_json: String,
}

impl FixtureEmailImportRequest {
    pub fn new(
        account_id: impl Into<String>,
        import_batch_id: impl Into<String>,
        fixture_json: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            import_batch_id: import_batch_id.into(),
            fixture_json: fixture_json.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureEmailImportReport {
    pub inserted_or_existing_records: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureEmailImportWithRecordsReport {
    pub inserted_or_existing_records: usize,
    pub raw_records: Vec<StoredRawCommunicationRecord>,
}

pub async fn import_fixture_email_messages(
    store: &CommunicationIngestionStore,
    request: &FixtureEmailImportRequest,
) -> Result<FixtureEmailImportReport, FixtureEmailImportError> {
    let report = import_fixture_email_messages_with_records(store, request).await?;

    Ok(FixtureEmailImportReport {
        inserted_or_existing_records: report.inserted_or_existing_records,
    })
}

pub async fn import_fixture_email_messages_with_records(
    store: &CommunicationIngestionStore,
    request: &FixtureEmailImportRequest,
) -> Result<FixtureEmailImportWithRecordsReport, FixtureEmailImportError> {
    let messages = parse_fixture_email_messages(&request.fixture_json)?;
    let mut inserted_or_existing_records = 0;
    let mut raw_records = Vec::new();

    for message in messages {
        let mut raw_record = NewRawCommunicationRecord::new(
            raw_record_id(
                &request.account_id,
                EMAIL_MESSAGE_RECORD_KIND,
                &message.provider_record_id,
            ),
            &request.account_id,
            EMAIL_MESSAGE_RECORD_KIND,
            &message.provider_record_id,
            &message.source_fingerprint,
            &request.import_batch_id,
            json!({
                "subject": message.subject,
                "from": message.from,
                "to": message.to,
                "body_text": message.body_text
            }),
        )
        .provenance(json!({"source": "fixture_email"}));

        if let Some(sent_at) = message.sent_at {
            raw_record = raw_record.occurred_at(sent_at);
        }

        raw_records.push(store.record_raw_source(&raw_record).await?);
        inserted_or_existing_records += 1;
    }

    Ok(FixtureEmailImportWithRecordsReport {
        inserted_or_existing_records,
        raw_records,
    })
}

fn raw_record_id(account_id: &str, record_kind: &str, provider_record_id: &str) -> String {
    let mut encoded = String::from("raw:v1:");
    append_raw_record_id_component(&mut encoded, account_id);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, record_kind);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, provider_record_id);
    encoded
}

fn append_raw_record_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}

#[derive(Debug, Error)]
pub enum FixtureEmailImportError {
    #[error(transparent)]
    Source(#[from] FixtureEmailSourceError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),
}
```

### `backend/src/domains/communications/ingestion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/ingestion.rs`
- Size bytes / Размер в байтах: `7001`
- Included characters / Включено символов: `6999`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, WorkflowState,
};

const URGENT_WORDS: &[&str] = &[
    "urgent",
    "asap",
    "deadline",
    "immediately",
    "critical",
    "action required",
];
const FINANCE_WORDS: &[&str] = &[
    "invoice",
    "payment",
    "factura",
    "bill",
    "amount due",
    "receipt",
    "tax",
];
const LEGAL_WORDS: &[&str] = &[
    "contract",
    "agreement",
    "nda",
    "legal",
    "liability",
    "confidential",
    "attorney",
];
const ATTACHMENT_WORDS: &[&str] = &["attached", "attachment", "see attached", "please find"];
const JUNK_WORDS: &[&str] = &[
    "unsubscribe",
    "opt out",
    "this email was sent",
    "if you no longer wish",
];

/// Result of Макошь auto-analysis on an ingested message.
#[derive(Debug)]
pub struct IngestionAnalysis {
    pub category: Option<String>,
    pub importance_score: i16,
    pub is_spam: bool,
    pub is_phishing: bool,
    pub auto_workflow_state: WorkflowState,
}

/// Analyze an incoming message and persist results.
/// This is the mandatory analysis step for every ingested email —
/// provider spam classification is completely ignored.
pub async fn analyze_ingested_message(
    store: &MessageProjectionStore,
    message: &ProjectedMessage,
) -> Result<IngestionAnalysis, MessageProjectionError> {
    let score = heuristic_score(message);
    let category = heuristic_category(message);

    let body_lower = message.body_text.to_lowercase();

    let is_spam = body_lower.contains("unsubscribe")
        && (body_lower.contains("buy now")
            || body_lower.contains("limited offer")
            || body_lower.contains("click here"));
    let is_phishing = (body_lower.contains("verify your account")
        || body_lower.contains("confirm your password")
        || body_lower.contains("urgent action required"))
        && !message.sender.contains(&message.account_id);

    let auto_state = if is_phishing || (is_spam && score < 20) {
        WorkflowState::Spam
    } else if score >= 75 {
        WorkflowState::NeedsAction
    } else {
        WorkflowState::New
    };

    store
        .set_ai_analysis(&message.message_id, category.as_deref(), None, Some(score))
        .await?;

    if auto_state != WorkflowState::New {
        store
            .transition_workflow_state(&message.message_id, auto_state)
            .await?;
    }

    Ok(IngestionAnalysis {
        category,
        importance_score: score,
        is_spam,
        is_phishing,
        auto_workflow_state: auto_state,
    })
}

fn heuristic_score(message: &ProjectedMessage) -> i16 {
    let mut score: i16 = 30;
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if contains_any(&subject_lower, URGENT_WORDS) {
        score = score.saturating_add(15);
    }
    if contains_any(&body_lower, FINANCE_WORDS) || contains_any(&subject_lower, FINANCE_WORDS) {
        score = score.saturating_add(20);
    }
    if contains_any(&body_lower, LEGAL_WORDS) || contains_any(&subject_lower, LEGAL_WORDS) {
        score = score.saturating_add(25);
    }

    if body_lower.contains('?') {
        score = score.saturating_add(10);
    }
    if contains_any(&body_lower, ATTACHMENT_WORDS) {
        score = score.saturating_add(10);
    }
    if contains_any(&body_lower, JUNK_WORDS) {
        score = score.saturating_sub(20);
    }
    if message.body_text.len() < 50 {
        score = score.saturating_sub(10);
    }

    score.clamp(0, 100)
}

fn heuristic_category(message: &ProjectedMessage) -> Option<String> {
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if body_lower.contains("invoice")
        || body_lower.contains("factura")
        || body_lower.contains("payment")
    {
        return Some("finance".to_owned());
    }
    if body_lower.contains("contract")
        || body_lower.contains("nda")
        || body_lower.contains("agreement")
    {
        return Some("legal".to_owned());
    }
    if body_lower.contains("unsubscribe") || body_lower.contains("newsletter") {
        return Some("marketing".to_owned());
    }
    if subject_lower.contains("notification") || body_lower.contains("notification") {
        return Some("notification".to_owned());
    }
    if body_lower.contains("click here")
        && (body_lower.contains("account") || body_lower.contains("verify"))
    {
        return Some("suspicious".to_owned());
    }

    None
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::communications::messages::LocalMessageState;
    use chrono::Utc;
    use serde_json::json;

    fn test_message(subject: &str, sender: &str, body: &str) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "m:1".into(),
            raw_record_id: "r:1".into(),
            observation_id: "observation:1".into(),
            account_id: "personal@ex.com".into(),
            provider_record_id: "p:1".into(),
            subject: subject.into(),
            sender: sender.into(),
            recipients: vec!["me@ex.com".into()],
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
    fn phishing_detection_flags_spam() {
        let msg = test_message(
            "Urgent",
            "hacker@evil.com",
            "Please verify your account immediately by clicking here",
        );
        let analysis = heuristic_score(&msg);
        assert!(analysis > 0);
    }

    #[test]
    fn newsletter_detected_as_low_score() {
        let msg = test_message(
            "Weekly Digest",
            "news@company.com",
            "Click here to read. To unsubscribe, click here.",
        );
        let score = heuristic_score(&msg);
        assert!(score <= 30, "newsletters should score low, got {score}");
    }

    #[test]
    fn invoice_gets_high_score() {
        let msg = test_message(
            "Invoice #456",
            "billing@vendor.com",
            "Please find your invoice attached. Amount due: $500. Payment required by June 15.",
        );
        let score = heuristic_score(&msg);
        assert!(score >= 50, "invoices should score high, got {score}");
    }
}
```

### `backend/src/domains/communications/legal.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/legal.rs`
- Size bytes / Размер в байтах: `8368`
- Included characters / Включено символов: `8368`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegalDocument {
    pub document_id: String,
    pub message_id: Option<String>,
    pub document_type: LegalDocType,
    pub title: String,
    pub parties: Vec<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub status: LegalDocStatus,
    pub linked_project_id: Option<String>,
    pub risks: Vec<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalDocType {
    Contract,
    Nda,
    Msa,
    Dpa,
    Agreement,
    LegalNotice,
    Claim,
    CourtDocument,
    TaxNotice,
    GovernmentDoc,
    Other,
}

impl LegalDocType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LegalDocType::Contract => "contract",
            LegalDocType::Nda => "nda",
            LegalDocType::Msa => "msa",
            LegalDocType::Dpa => "dpa",
            LegalDocType::Agreement => "agreement",
            LegalDocType::LegalNotice => "legal_notice",
            LegalDocType::Claim => "claim",
            LegalDocType::CourtDocument => "court_document",
            LegalDocType::TaxNotice => "tax_notice",
            LegalDocType::GovernmentDoc => "government_doc",
            LegalDocType::Other => "other",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "contract" => Some(LegalDocType::Contract),
            "nda" => Some(LegalDocType::Nda),
            "msa" => Some(LegalDocType::Msa),
            "dpa" => Some(LegalDocType::Dpa),
            "agreement" => Some(LegalDocType::Agreement),
            "legal_notice" => Some(LegalDocType::LegalNotice),
            "claim" => Some(LegalDocType::Claim),
            "court_document" => Some(LegalDocType::CourtDocument),
            "tax_notice" => Some(LegalDocType::TaxNotice),
            "government_doc" => Some(LegalDocType::GovernmentDoc),
            "other" => Some(LegalDocType::Other),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalDocStatus {
    Active,
    Expired,
    PendingReview,
    Signed,
    Terminated,
    Draft,
}

impl LegalDocStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LegalDocStatus::Active => "active",
            LegalDocStatus::Expired => "expired",
            LegalDocStatus::PendingReview => "pending_review",
            LegalDocStatus::Signed => "signed",
            LegalDocStatus::Terminated => "terminated",
            LegalDocStatus::Draft => "draft",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "active" => Some(LegalDocStatus::Active),
            "expired" => Some(LegalDocStatus::Expired),
            "pending_review" => Some(LegalDocStatus::PendingReview),
            "signed" => Some(LegalDocStatus::Signed),
            "terminated" => Some(LegalDocStatus::Terminated),
            "draft" => Some(LegalDocStatus::Draft),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct LegalDocumentStore {
    pool: PgPool,
}

impl LegalDocumentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        doc: &NewLegalDocument,
    ) -> Result<LegalDocument, LegalDocumentError> {
        doc.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO communication_legal_documents (document_id, message_id, document_type, title, parties, effective_date, expiry_date, amount, currency, status, linked_project_id, risks, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (document_id) DO UPDATE SET
                message_id = EXCLUDED.message_id, document_type = EXCLUDED.document_type, title = EXCLUDED.title,
                parties = EXCLUDED.parties, effective_date = EXCLUDED.effective_date, expiry_date = EXCLUDED.expiry_date,
                amount = EXCLUDED.amount, currency = EXCLUDED.currency, status = EXCLUDED.status,
                linked_project_id = EXCLUDED.linked_project_id, risks = EXCLUDED.risks,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING document_id, message_id, document_type, title, parties, effective_date, expiry_date, amount, currency, status, linked_project_id, risks, metadata, created_at, updated_at"#,
        )
        .bind(&doc.document_id).bind(doc.message_id.as_deref()).bind(doc.document_type.as_str())
        .bind(&doc.title).bind(serde_json::to_value(&doc.parties).unwrap_or_default())
        .bind(doc.effective_date).bind(doc.expiry_date).bind(doc.amount).bind(doc.currency.as_deref())
        .bind(doc.status.as_str()).bind(doc.linked_project_id.as_deref())
        .bind(serde_json::to_value(&doc.risks).unwrap_or_default()).bind(&doc.metadata)
        .fetch_one(&self.pool).await?;
        row_to_legal_doc(row)
    }

    pub async fn list(
        &self,
        doc_type: Option<LegalDocType>,
        status: Option<LegalDocStatus>,
    ) -> Result<Vec<LegalDocument>, LegalDocumentError> {
        let dt = doc_type.map(|t| t.as_str().to_owned());
        let st = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT document_id, message_id, document_type, title, parties, effective_date, expiry_date, amount, currency, status, linked_project_id, risks, metadata, created_at, updated_at
            FROM communication_legal_documents WHERE ($1::text IS NULL OR document_type = $1) AND ($2::text IS NULL OR status = $2) ORDER BY COALESCE(effective_date, created_at) DESC"#,
        ).bind(dt.as_deref()).bind(st.as_deref()).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_legal_doc).collect()
    }
}

fn row_to_legal_doc(row: PgRow) -> Result<LegalDocument, LegalDocumentError> {
    let dt_str: String = row.try_get("document_type")?;
    let st_str: String = row.try_get("status")?;
    Ok(LegalDocument {
        document_id: row.try_get("document_id")?,
        message_id: row.try_get("message_id")?,
        document_type: LegalDocType::parse(&dt_str).unwrap_or(LegalDocType::Other),
        title: row.try_get("title")?,
        parties: serde_json::from_value(row.try_get("parties")?).unwrap_or_default(),
        effective_date: row.try_get("effective_date")?,
        expiry_date: row.try_get("expiry_date")?,
        amount: row.try_get("amount")?,
        currency: row.try_get("currency")?,
        status: LegalDocStatus::parse(&st_str).unwrap_or(LegalDocStatus::Draft),
        linked_project_id: row.try_get("linked_project_id")?,
        risks: serde_json::from_value(row.try_get("risks")?).unwrap_or_default(),
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Clone, Debug)]
pub struct NewLegalDocument {
    pub document_id: String,
    pub message_id: Option<String>,
    pub document_type: LegalDocType,
    pub title: String,
    pub parties: Vec<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub status: LegalDocStatus,
    pub linked_project_id: Option<String>,
    pub risks: Vec<String>,
    pub metadata: Value,
}

impl NewLegalDocument {
    fn validate(&self) -> Result<(), LegalDocumentError> {
        if self.document_id.trim().is_empty() {
            return Err(LegalDocumentError::Invalid("document_id empty"));
        }
        if self.title.trim().is_empty() {
            return Err(LegalDocumentError::Invalid("title empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum LegalDocumentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid document: {0}")]
    Invalid(&'static str),
}
```

### `backend/src/domains/communications/messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages.rs`
- Size bytes / Размер в байтах: `1770`
- Included characters / Включено символов: `1770`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod ids;
mod models;
mod payload;
mod projection;
mod provider_channel_store;
mod provider_observation_projection;
mod query_parser;
mod rows;
mod search;
mod states;
mod store;
mod validation;

pub use crate::platform::communications::{
    ProviderChannelMessage, ProviderCommunicationMessagePortError, ProviderHeuristicMember,
    ProviderMessageAttachmentAnchor, ProviderMessageProjectionObservationContext,
    ProviderMessageReferenceSummary,
};
pub use errors::MessageProjectionError;
pub use models::{
    MessageSearchMatchMode, MessageSearchQuery, NewProjectedMessage, ProjectedMessage,
    ProjectedMessagePage, ProjectedMessagePageQuery, ProjectedMessageSummary, WorkflowStateCount,
};
pub use projection::{
    parse_raw_email_message_from_blob, project_parsed_raw_email_message, project_raw_email_message,
    project_raw_email_message_from_blob,
};
pub use provider_channel_store::ProviderChannelMessageStore;
pub use provider_observation_projection::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, CommunicationSignalProjectionError,
    consume_accepted_signal_event, project_accepted_signal_if_runtime_allows,
    project_provider_observation_event, project_whatsapp_content_observed,
    project_whatsapp_delivery_state_observed, replay_accepted_signal_event,
    supports_communication_projection_signal_event,
};
pub(crate) use query_parser::parse_communication_message_search_query;
pub use search::{
    MessageSearchBoolean, MessageSearchExpression, MessageSearchField, MessageSearchPredicate,
    MessageSearchPredicateOperator, append_message_search_filter,
};
pub use states::{LocalMessageState, WorkflowState};
pub use store::MessageProjectionStore;
pub use store::MessageProjectionStore as CommunicationMessageProjectionPort;
```

### `backend/src/domains/communications/messages/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/errors.rs`
- Size bytes / Размер в байтах: `1986`
- Included characters / Включено символов: `1986`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::communications::storage::CommunicationStorageError;
use crate::platform::communications::rfc822::EmailRfc822ParseError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum MessageProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error(transparent)]
    Rfc822(#[from] EmailRfc822ParseError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error("raw email payload missing required field or wrong type: {0}")]
    MissingPayloadField(&'static str),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error(
        "raw communication record does not match projected message tuple: raw_record_id={raw_record_id}, account_id={account_id}, provider_record_id={provider_record_id}"
    )]
    RawRecordTupleMismatch {
        raw_record_id: String,
        account_id: String,
        provider_record_id: String,
    },

    #[error("raw communication record was not found: {0}")]
    RawRecordNotFound(String),

    #[error("stored communication message recipients must be a JSON array of strings")]
    InvalidStoredRecipients,

    #[error("communication message metadata must be a JSON object")]
    InvalidMessageMetadata,

    #[error("unsupported raw blob storage kind: {0}")]
    UnsupportedRawBlobStorageKind(String),

    #[error("message query limit must be between 1 and 5000: {0}")]
    InvalidLimit(i64),

    #[error("invalid communication message cursor")]
    InvalidCursor,

    #[error("communication message was not found")]
    MessageNotFound,

    #[error("invalid workflow state: {0}")]
    InvalidWorkflowState(String),

    #[error("invalid local message state: {0}")]
    InvalidLocalState(String),

    #[error("invalid importance score: {0}, must be 0-100")]
    InvalidImportanceScore(i16),
}
```

### `backend/src/domains/communications/messages/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/ids.rs`
- Size bytes / Размер в байтах: `462`
- Included characters / Включено символов: `462`
- Truncated / Обрезано: `no`

```rust
pub(crate) fn message_id(account_id: &str, provider_record_id: &str) -> String {
    let mut encoded = String::from("msg:v1:");
    append_message_id_component(&mut encoded, account_id);
    encoded.push(':');
    append_message_id_component(&mut encoded, provider_record_id);
    encoded
}

fn append_message_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}
```

### `backend/src/domains/communications/messages/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/models.rs`
- Size bytes / Размер в байтах: `5679`
- Included characters / Включено символов: `5679`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::errors::MessageProjectionError;
use super::search::MessageSearchExpression;
use super::states::{LocalMessageState, WorkflowState};
use super::validation::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProjectedMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub channel_kind: String,
    pub conversation_id: Option<String>,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub message_metadata: Value,
}

impl NewProjectedMessage {
    pub(crate) fn validate(&self) -> Result<(), MessageProjectionError> {
        self.validate_with_body_policy(false)
    }

    pub(crate) fn validate_with_body_policy(
        &self,
        allow_empty_body_text: bool,
    ) -> Result<(), MessageProjectionError> {
        validate_non_empty("message_id", &self.message_id)?;
        validate_non_empty("raw_record_id", &self.raw_record_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_record_id", &self.provider_record_id)?;
        validate_non_empty("subject", &self.subject)?;
        validate_non_empty("sender", &self.sender)?;
        if !allow_empty_body_text {
            validate_non_empty("body_text", &self.body_text)?;
        }
        validate_non_empty("channel_kind", &self.channel_kind)?;
        validate_non_empty("delivery_state", &self.delivery_state)?;
        if !self.message_metadata.is_object() {
            return Err(MessageProjectionError::InvalidMessageMetadata);
        }
        for recipient in &self.recipients {
            validate_non_empty("to", recipient)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub observation_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub conversation_id: Option<String>,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub message_metadata: Value,
    pub workflow_state: WorkflowState,
    pub importance_score: Option<i16>,
    pub ai_category: Option<String>,
    pub ai_summary: Option<String>,
    pub ai_summary_generated_at: Option<DateTime<Utc>>,
    pub local_state: LocalMessageState,
    pub local_state_changed_at: Option<DateTime<Utc>>,
    pub local_state_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessageSummary {
    pub message: ProjectedMessage,
    pub attachment_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessagePage {
    pub items: Vec<ProjectedMessageSummary>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MessageSearchMatchMode {
    #[default]
    All,
    Any,
}

impl MessageSearchMatchMode {
    pub const fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    pub const fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MessageSearchQuery {
    pub plain_terms: Vec<String>,
    pub subject_contains: Vec<String>,
    pub subject_equals: Vec<String>,
    pub body_contains: Vec<String>,
    pub body_equals: Vec<String>,
    pub sender_contains: Vec<String>,
    pub sender_equals: Vec<String>,
    pub all_contains: Vec<String>,
    pub all_equals: Vec<String>,
    pub match_mode: MessageSearchMatchMode,
    pub expression: Option<MessageSearchExpression>,
}

impl MessageSearchQuery {
    pub fn is_empty(&self) -> bool {
        self.expression.is_none()
            && self.plain_terms.is_empty()
            && self.subject_contains.is_empty()
            && self.subject_equals.is_empty()
            && self.body_contains.is_empty()
            && self.body_equals.is_empty()
            && self.sender_contains.is_empty()
            && self.sender_equals.is_empty()
            && self.all_contains.is_empty()
            && self.all_equals.is_empty()
    }

    pub fn term_count(&self) -> usize {
        self.expression
            .as_ref()
            .map(MessageSearchExpression::term_count)
            .unwrap_or(0)
            + self.plain_terms.len()
            + self.subject_contains.len()
            + self.subject_equals.len()
            + self.body_contains.len()
            + self.body_equals.len()
            + self.sender_contains.len()
            + self.sender_equals.len()
            + self.all_contains.len()
            + self.all_equals.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessagePageQuery<'a> {
    pub account_id: Option<&'a str>,
    pub workflow_state: Option<WorkflowState>,
    pub channel_kind: Option<&'a str>,
    pub conversation_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub match_mode: MessageSearchMatchMode,
    pub search: MessageSearchQuery,
    pub local_state: LocalMessageState,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkflowStateCount {
    pub state: WorkflowState,
    pub count: i64,
}
```

### `backend/src/domains/communications/messages/payload.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/payload.rs`
- Size bytes / Размер в байтах: `1396`
- Included characters / Включено символов: `1396`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::MessageProjectionError;

pub(crate) fn required_payload_string(
    payload: &Value,
    field_name: &'static str,
) -> Result<String, MessageProjectionError> {
    payload
        .get(field_name)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(MessageProjectionError::MissingPayloadField(field_name))
}

pub(crate) fn required_payload_string_array(
    payload: &Value,
    field_name: &'static str,
) -> Result<Vec<String>, MessageProjectionError> {
    let values = payload
        .get(field_name)
        .and_then(Value::as_array)
        .ok_or(MessageProjectionError::MissingPayloadField(field_name))?;

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(MessageProjectionError::MissingPayloadField(field_name))
        })
        .collect()
}

pub(crate) fn recipients_from_value(value: Value) -> Result<Vec<String>, MessageProjectionError> {
    let Some(values) = value.as_array() else {
        return Err(MessageProjectionError::InvalidStoredRecipients);
    };

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(MessageProjectionError::InvalidStoredRecipients)
        })
        .collect()
}
```

### `backend/src/domains/communications/messages/projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/projection.rs`
- Size bytes / Размер в байтах: `3604`
- Included characters / Включено символов: `3604`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::domains::communications::core::StoredRawCommunicationRecord;
use crate::domains::communications::storage::LocalCommunicationBlobStore;
use crate::platform::communications::rfc822::{
    ParsedCommunicationSourceMessage, parse_rfc822_message,
};

use super::errors::MessageProjectionError;
use super::ids::message_id;
use super::models::{NewProjectedMessage, ProjectedMessage};
use super::payload::{required_payload_string, required_payload_string_array};
use super::store::MessageProjectionStore;

pub async fn project_raw_email_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let subject = required_payload_string(&raw.payload, "subject")?;
    let sender = required_payload_string(&raw.payload, "from")?;
    let recipients = required_payload_string_array(&raw.payload, "to")?;
    let body_text = required_payload_string(&raw.payload, "body_text")?;
    let message = NewProjectedMessage {
        message_id: message_id(&raw.account_id, &raw.provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id: raw.provider_record_id.clone(),
        subject,
        sender: sender.clone(),
        recipients,
        body_text,
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some(sender.clone()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    store.upsert_message(&message).await
}

pub async fn project_raw_email_message_from_blob(
    store: &MessageProjectionStore,
    blob_store: &LocalCommunicationBlobStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let parsed = parse_raw_email_message_from_blob(blob_store, raw).await?;
    project_parsed_raw_email_message(store, raw, &parsed).await
}

pub async fn parse_raw_email_message_from_blob(
    blob_store: &LocalCommunicationBlobStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ParsedCommunicationSourceMessage, MessageProjectionError> {
    let storage_kind = required_payload_string(&raw.payload, "raw_blob_storage_kind")?;
    if storage_kind != "local_fs" {
        return Err(MessageProjectionError::UnsupportedRawBlobStorageKind(
            storage_kind,
        ));
    }
    let storage_path = required_payload_string(&raw.payload, "raw_blob_storage_path")?;
    let bytes = blob_store.read_blob(&storage_path).await?;
    Ok(parse_rfc822_message(&bytes)?)
}

pub async fn project_parsed_raw_email_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
    parsed: &ParsedCommunicationSourceMessage,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let message = NewProjectedMessage {
        message_id: message_id(&raw.account_id, &raw.provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id: raw.provider_record_id.clone(),
        subject: parsed.subject.clone(),
        sender: parsed.from.clone(),
        recipients: parsed.to.clone(),
        body_text: parsed.body_text.clone(),
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some(parsed.from.clone()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    store.upsert_message(&message).await
}
```

### `backend/src/domains/communications/messages/provider_channel_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/provider_channel_store.rs`
- Size bytes / Размер в байтах: `48274`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};

use crate::platform::communications::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage,
    ProviderChannelMessageCommandPort, ProviderChannelMessageLookupPort,
    ProviderCommunicationMessagePortError, ProviderHeuristicMember,
    ProviderMessageAttachmentAnchor, ProviderMessageProjectionObservationContext,
    ProviderMessageReferenceSummary,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
    link_domain_entity_in_transaction,
};

#[derive(Clone)]
pub struct ProviderChannelMessageStore {
    pool: PgPool,
}

impl ProviderChannelMessageStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) fn clone_pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn message_by_id(
        &self,
        message_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let row = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE message_id = $1
              AND channel_kind = ANY($2)
            "#,
        )
        .bind(message_id.trim())
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_channel_message).transpose()
    }

    pub async fn message_by_provider_record_id(
        &self,
        account_id: &str,
        provider_record_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let row = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE account_id = $1
              AND provider_record_id = $2
              AND channel_kind = ANY($3)
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_record_id.trim())
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_channel_message).transpose()
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        conversation_id: Option<&str>,
        channel_kinds: &[&str],
        limit: i64,
    ) -> Result<Vec<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let conversation_id = conversation_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages message
            WHERE message.channel_kind = ANY($1)
              AND ($2::text IS NULL OR message.account_id = $2)
              AND (
                  $3::text IS NULL
                  OR message.conversation_id = $3
                  OR EXISTS (
                      SELECT 1
                      FROM communication_conversations conversation
                      WHERE conversation.conversation_id = message.conversation_id
                        AND conversation.account_id = message.account_id
                        AND conversation.provider_conversation_id = $3
                        AND conversation.channel_kind = ANY($1)
                  )
              )
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT $4
            "#,
        )
        .bind(channel_kinds)
        .bind(account_id)
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_channel_message)
            .collect()
    }

    pub async fn messages_by_ids(
        &self,
        message_ids: &[String],
        channel_kinds: &[&str],
    ) -> Result<Vec<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        if message_ids.is_empty() {
            return Ok(vec![]);
        }
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE message_id = ANY($1)
              AND channel_kind = ANY($2)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            "#,
        )
        .bind(message_ids)
        .bind(channel_kinds)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_channel_message)
            .collect()
    }

    pub async fn search_messages(
        &self,
        account_id: Option<&str>,
        conversation_id: Option<&str>,
        query: &str,
        channel_kinds: &[&str],
        limit: i64,
    ) -> Result<Vec<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let like_pattern = format!("%{query}%");
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let conversation_id = conversation_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE channel_kind = ANY($1)
              AND body_text ILIKE $2
              AND ($3::text IS NULL OR account_id = $3)
              AND ($4::text IS NULL OR conversation_id = $4)
            ORDER BY COALESCE(occurred_at, projected_at) DESC
            LIMIT $5
            "#,
        )
        .bind(channel_kinds)
        .bind(&like_pattern)
        .bind(account_id)
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_channel_message)
            .collect()
    }

    pub async fn pinned_messages(
        &self,
        account_id: &str,
        conversation_id: &str,
        channel_kinds: &[&str],
        limit: i64,
    ) -> Result<Vec<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE channel_kind = ANY($1)
              AND account_id = $2
              AND conversation_id = $3
              AND (
                COALESCE(message_metadata->>'is_pinned', 'false') = 'true'
                OR COALESCE(message_metadata->>'pinned', 'false') = 'true'
              )
            ORDER BY COALESCE(occurred_at, projected_at) DESC
            LIMIT $4
            "#,
        )
        .bind(channel_kinds)
        .bind(account_id)
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_channel_message)
            .collect()
    }

    pub async fn body_text(
        &self,
        message_id: &str,
    ) -> Result<Option<String>, ProviderCommunicationMessagePortError> {
        Ok(sqlx::query_scalar::<_, Option<String>>(
            "SELECT body_text FROM communication_messages WHERE message_id = $1",
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?
        .flatten())
    }

    pub async fn message_ids_by_metadata_string(
        &self,
        metadata_key: &str,
        metadata_value: &str,
        channel_kinds: &[&str],
        limit: i64,
    ) -> Result<Vec<String>, ProviderCommunicationMessagePortError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT message_id
            FROM communication_messages
            WHERE message_metadata ->> $1 = $2
              AND channel_kind = ANY($3)
            ORDER BY COALESCE(occurred_at, projected_at) DESC NULLS LAST, message_id ASC
            LIMIT $4
            "#,
        )
        .bind(metadata_key)
        .bind(metadata_value)
        .bind(channel_kinds)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    pub async fn message_id_by_provider_record_id(
        &self,
        account_id: &str,
        provider_record_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<String>, ProviderCommunicationMessagePortError> {
        sqlx::query_scalar(
            r#"
            SELECT message_id
            FROM communication_messages
            WHERE account_id = $1
              AND provider_record_id = $2
              AND channel_kind = ANY($3)
            LIMIT 1
            "#,
        )
        .bind(account_id)
        .bind(provider_record_id)
        .bind(channel_kinds)
        .fetch_optional(&self.pool)
        .await
        .map_err(ProviderCommunicationMessagePortError::from)
    }

    pub async fn reference_summaries(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<ProviderMessageReferenceSummary>, ProviderCommunicationMessagePortError> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                Option<String>,
                String,
                Option<DateTime<Utc>>,
            ),
        >(
            r#"
            SELECT

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/messages/provider_observation_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/provider_observation_projection.rs`
- Size bytes / Размер в байтах: `26384`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::{
    CommunicationMessageProjectionPort, MessageProjectionError, NewProjectedMessage,
    ProjectedMessage, ProviderChannelMessageStore, project_raw_email_message,
    project_raw_email_message_from_blob,
};
use crate::domains::communications::core::CommunicationIngestionPort;
use crate::domains::communications::delivery_notifications::consume_accepted_mail_delivery_signal;
use crate::domains::communications::storage::LocalCommunicationBlobStore;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::communications::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage,
    ProviderCommunicationMessagePortError, ProviderMessageProjectionObservationContext,
};
use crate::platform::events::{
    EventEnvelope, EventStore, EventStoreError, NewEventEnvelope, StoredEventEnvelope,
};

pub const COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER: &str =
    "communication_provider_observation_projection";

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];
const WHATSAPP_CHANNEL_KINDS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];

pub async fn project_provider_observation_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    if is_supported_mail_delivery_signal_event(&event.event.event_type) {
        consume_accepted_mail_delivery_signal(pool.clone(), &event.event)
            .await
            .map(|_| ())
            .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
        return Ok(());
    }

    if is_base_accepted_signal_event(&event.event.event_type) {
        consume_accepted_signal_event(pool.clone(), &event.event)
            .await
            .map(|_| ())
            .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
        return Ok(());
    }

    if !is_supported_provider_observation_event(&event.event.event_type) {
        return Ok(());
    }

    let updated = project_telegram_observation(pool.clone(), &event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
    if let Some(message) = updated {
        append_communication_message_updated_event(pool, &event, &message).await?;
    }

    Ok(())
}

pub async fn replay_accepted_signal_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_provider_observation_event(pool, event).await
}

fn is_base_accepted_signal_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.mail.message"
            | "signal.accepted.telegram.message"
            | "signal.accepted.whatsapp.message"
    )
}

fn is_supported_mail_delivery_signal_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.mail.delivery_status" | "signal.accepted.mail.read_receipt"
    )
}

fn is_supported_provider_observation_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.telegram.message.content"
            | "signal.accepted.telegram.message.metadata"
            | "signal.accepted.telegram.message.delivery_state"
            | "signal.accepted.telegram.message.pinned_state"
            | "signal.accepted.telegram.attachment.download_state"
    )
}

pub fn supports_communication_projection_signal_event(event_type: &str) -> bool {
    is_base_accepted_signal_event(event_type) || is_supported_provider_observation_event(event_type)
}

pub async fn project_accepted_signal_if_runtime_allows(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    if !accepted_signal_projection_runtime_allows(&pool).await? {
        return Ok(None);
    }

    consume_accepted_signal_event(pool, event).await
}

pub async fn consume_accepted_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    let Some(projection) = project_accepted_signal_event(pool.clone(), event).await? else {
        return Ok(None);
    };

    append_communication_message_projected_event(
        pool,
        event,
        &projection.message,
        projection.message_existed,
    )
    .await?;

    Ok(Some(projection.message))
}

struct AcceptedSignalProjection {
    message: ProjectedMessage,
    message_existed: bool,
}

async fn project_accepted_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type == "signal.accepted.mail.message" {
        return project_mail_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.telegram.message" {
        return project_telegram_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.whatsapp.message" {
        return project_whatsapp_signal_event(pool, event).await;
    }

    Ok(None)
}

async fn accepted_signal_projection_runtime_allows(
    pool: &PgPool,
) -> Result<bool, CommunicationSignalProjectionError> {
    Ok(crate::platform::events::runtime_allows_processing(
        pool,
        "system",
        COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        &json!({
            "label": "Communications accepted-signal consumer",
            "scope": "consumer",
        }),
    )
    .await?)
}

async fn project_mail_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.mail.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionPort::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;
    let message_store = CommunicationMessageProjectionPort::new(pool);

    let message = if raw_record.payload.get("raw_blob_storage_path").is_some() {
        let blob_store = LocalCommunicationBlobStore::new(mail_blob_root_from_event(event));
        project_raw_email_message_from_blob(&message_store, &blob_store, &raw_record).await?
    } else {
        project_raw_email_message(&message_store, &raw_record).await?
    };

    Ok(Some(AcceptedSignalProjection {
        message,
        message_existed,
    }))
}

async fn project_whatsapp_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.whatsapp.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionPort::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let provider_chat_id = required_payload_str(&raw_record.payload, "provider_chat_id")?;
    let chat_title = required_payload_str(&raw_record.payload, "chat_title")?;
    let sender_display_name = required_payload_str(&raw_record.payload, "sender_display_name")?;
    let body_text = required_payload_str(&raw_record.payload, "text")?;
    let delivery_state = required_payload_str(&raw_record.payload, "delivery_state")?;
    let channel_kind = raw_record
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| matches!(*value, "whatsapp_web" | "whatsapp_business_cloud"))
        .unwrap_or("whatsapp_web")
        .to_owned();
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;

    let message = CommunicationMessageProjectionPort::new(pool)
        .upsert_channel_message(&NewProjectedMessage {
            message_id: whatsapp_web_message_id(
                &raw_record.account_id,
                &raw_record.provider_record_id,
            ),
            raw_record_id: raw_record.raw_record_id.clone(),
            account_id: raw_record.account_id.clone(),
            provider_record_id: raw_record.provider_record_id.clone(),
            subject: chat_title,
            sender: sender_display_name.clone(),
            recipients: vec![provider_chat_id.clone()],
            body_text,
            occurred_at: raw_record.occurred_at,
            channel_kind,
            conversation_id: Some(provider_chat_id),
            sender_display_name: Some(sender_display_name),
            delivery_state,
            message_metadata: raw_record.payload,
        })
        .await?;

    Ok(Some(AcceptedSignalProjection {
        message,
        message_existed,
    }))
}

async fn project_telegram_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.telegram.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionPort::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let provider_chat_id = required_payload_str(&raw_record.payload, "provider_chat_id")?;
    let chat_title = required_payload_str(&raw_record.payload, "chat_title")?;
    let sender_display_name = required_payload_str(&raw_record.payload, "sender_display_name")?;
    let body_text = optional_payload_str(&raw_record.payload, "text").unwrap_or_default();
    let delivery_state = required_payload_str(&raw_record.payload, "delivery_state")?;
    let channel_kind = raw_record
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .unwrap_or("telegram_user")
        .trim()
        .to_owned();
    let allow_empty_body_text = body_text.is_empty()
        && raw_record
            .provenance
            .get("runtime")
            .and_then(Value::as_str)
            .map(str::trim)
            == Some("tdlib")
        && raw_record.payload.get("tdlib_raw").is_some();
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;

    let message = NewProjectedMessage {
        message_id: telegram_message_id(&raw_record.account_id, &raw_record.provider_record_id),
        raw_record_id: raw_record.raw_record_id.clone(),
        account_id: raw_record.account_id.clone(),
        provider_record_id: raw_record.provider_record_id.clone(),
        subject: chat_title,
        sender: sender_display_name.clone(),
        recipients: vec![provider_chat_id.clone()],
        body_text,
        occurred_at: raw_record.occurred_at,
        channel_kind,
        conversation_id: Some(provider_chat_id),
        sender_display_name: Some(sender_display_name),
        delivery_state,
        message_metadata: raw_record.payload,
    };

    let projected = if allow_empty_body_text {
        CommunicationMessageProjectionPort::new(pool)
            .upsert_channel_message_allowing_empty_body_text(&message)
            .await?
    } else {
        CommunicationMessageProjectionPort::new(pool)
            .upsert_channel_message(&message)
            .await?
    };

    Ok(Some(AcceptedSignalProjection {
        message: projected
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/messages/query_parser.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/query_parser.rs`
- Size bytes / Размер в байтах: `15687`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::models::{MessageSearchMatchMode, MessageSearchQuery};
use super::search::{
    MessageSearchBoolean, MessageSearchExpression, MessageSearchField, MessageSearchPredicate,
    MessageSearchPredicateOperator,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct MessageSearchRule {
    field: MessageSearchField,
    operator: MessageSearchPredicateOperator,
    value: String,
}

pub(crate) fn parse_communication_message_search_query(
    raw_query: Option<&str>,
) -> MessageSearchQuery {
    let Some(query) = raw_query else {
        return MessageSearchQuery {
            match_mode: MessageSearchMatchMode::All,
            ..MessageSearchQuery::default()
        };
    };
    let query = query.trim();
    if query.is_empty() {
        return MessageSearchQuery {
            match_mode: MessageSearchMatchMode::All,
            ..MessageSearchQuery::default()
        };
    }
    if let Some(expression) = parse_explicit_search_expression(query) {
        return MessageSearchQuery {
            match_mode: MessageSearchMatchMode::All,
            expression: Some(expression),
            ..MessageSearchQuery::default()
        };
    }

    let mut parsed = MessageSearchQuery {
        match_mode: MessageSearchMatchMode::All,
        ..MessageSearchQuery::default()
    };
    let mut explicit_match_mode_seen = false;

    for token in tokenize_query_terms(query) {
        if let Some(match_mode) = parse_match_mode_token(&token) {
            if !explicit_match_mode_seen {
                parsed.match_mode = match_mode;
                explicit_match_mode_seen = true;
            }
            continue;
        }

        if let Some(rule) = parse_query_rule(&token) {
            add_rule_to_query(&mut parsed, rule);
            continue;
        }

        let normalized = strip_outer_quotes(&token);
        if !normalized.is_empty() {
            parsed.plain_terms.push(normalized);
        }
    }

    parsed
}

fn add_rule_to_query(parsed: &mut MessageSearchQuery, rule: MessageSearchRule) {
    if rule.value.trim().is_empty() {
        return;
    }

    match (rule.field, rule.operator) {
        (MessageSearchField::Subject, MessageSearchPredicateOperator::Contains) => {
            parsed.subject_contains.push(rule.value)
        }
        (MessageSearchField::Subject, MessageSearchPredicateOperator::Equals) => {
            parsed.subject_equals.push(rule.value)
        }
        (MessageSearchField::Body, MessageSearchPredicateOperator::Contains) => {
            parsed.body_contains.push(rule.value)
        }
        (MessageSearchField::Body, MessageSearchPredicateOperator::Equals) => {
            parsed.body_equals.push(rule.value)
        }
        (MessageSearchField::Sender, MessageSearchPredicateOperator::Contains) => {
            parsed.sender_contains.push(rule.value)
        }
        (MessageSearchField::Sender, MessageSearchPredicateOperator::Equals) => {
            parsed.sender_equals.push(rule.value)
        }
        (MessageSearchField::All, MessageSearchPredicateOperator::Contains) => {
            parsed.all_contains.push(rule.value)
        }
        (MessageSearchField::All, MessageSearchPredicateOperator::Equals) => {
            parsed.all_equals.push(rule.value)
        }
    }
}

fn parse_query_rule(token: &str) -> Option<MessageSearchRule> {
    if parse_match_mode_token(token).is_some() {
        return None;
    }

    let (field_name, operator, raw_value) = parse_rule_expression(token)?;
    let field = parse_search_field(field_name)?;
    let operator = parse_search_operator(operator)?;
    let value = strip_outer_quotes(raw_value);
    if value.is_empty() {
        return None;
    }

    Some(MessageSearchRule {
        field,
        operator,
        value,
    })
}

fn parse_match_mode_token(token: &str) -> Option<MessageSearchMatchMode> {
    let normalized = token.trim();
    if normalized.is_empty() {
        return None;
    }

    let mode_separator = normalized.find(':')?;
    if mode_separator == 0 {
        return None;
    }

    let field = normalized[..mode_separator].trim().to_lowercase();
    if field != "mode" {
        return None;
    }

    let value = normalized[(mode_separator + 1)..].trim().to_lowercase();
    match value.as_str() {
        "any" => Some(MessageSearchMatchMode::Any),
        "all" => Some(MessageSearchMatchMode::All),
        _ => None,
    }
}

fn parse_rule_expression(token: &str) -> Option<(&str, &str, &str)> {
    if let Some(index) = token.find("==")
        && index > 0
    {
        return Some((&token[0..index], "==", &token[index + 2..]));
    }
    if let Some(index) = token.find('=')
        && index > 0
    {
        return Some((&token[0..index], "=", &token[index + 1..]));
    }
    if let Some(index) = token.find(':')
        && index > 0
    {
        return Some((&token[0..index], ":", &token[index + 1..]));
    }

    None
}

fn parse_search_field(input: &str) -> Option<MessageSearchField> {
    match input.trim().to_lowercase().as_str() {
        "subject" => Some(MessageSearchField::Subject),
        "body" => Some(MessageSearchField::Body),
        "sender" | "from" => Some(MessageSearchField::Sender),
        "all" => Some(MessageSearchField::All),
        _ => None,
    }
}

fn parse_search_operator(value: &str) -> Option<MessageSearchPredicateOperator> {
    match value {
        ":" => Some(MessageSearchPredicateOperator::Contains),
        "=" | "==" => Some(MessageSearchPredicateOperator::Equals),
        _ => None,
    }
}

fn parse_explicit_search_expression(query: &str) -> Option<MessageSearchExpression> {
    let tokens = tokenize_search_expression(query);
    if !tokens.iter().any(|token| {
        matches!(
            token,
            SearchToken::OpenParen | SearchToken::CloseParen | SearchToken::And | SearchToken::Or
        )
    }) {
        return None;
    }

    let mut parser = ExplicitSearchParser::new(tokens);
    let expression = parser.parse_expression()?;
    if parser.has_remaining_tokens() {
        return None;
    }

    Some(expression)
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SearchToken {
    OpenParen,
    CloseParen,
    And,
    Or,
    Term(String),
}

fn tokenize_search_expression(query: &str) -> Vec<SearchToken> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote: Option<char> = None;

    let push_current = |tokens: &mut Vec<SearchToken>, current: &mut String| {
        let trimmed = current.trim();
        if trimmed.is_empty() {
            current.clear();
            return;
        }
        tokens.push(match trimmed {
            "AND" => SearchToken::And,
            "OR" => SearchToken::Or,
            _ => SearchToken::Term(trimmed.to_owned()),
        });
        current.clear();
    };

    for value in query.chars() {
        if matches!(value, '"' | '\'') {
            if !in_quotes {
                in_quotes = true;
                quote = Some(value);
                current.push(value);
                continue;
            }

            if Some(value) == quote {
                in_quotes = false;
                quote = None;
                current.push(value);
                continue;
            }
        }

        if !in_quotes && matches!(value, '(' | ')') {
            push_current(&mut tokens, &mut current);
            tokens.push(if value == '(' {
                SearchToken::OpenParen
            } else {
                SearchToken::CloseParen
            });
            continue;
        }

        if value.is_whitespace() && !in_quotes {
            push_current(&mut tokens, &mut current);
            continue;
        }

        current.push(value);
    }

    push_current(&mut tokens, &mut current);
    tokens
}

struct ExplicitSearchParser {
    tokens: Vec<SearchToken>,
    index: usize,
}

impl ExplicitSearchParser {
    fn new(tokens: Vec<SearchToken>) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse_expression(&mut self) -> Option<MessageSearchExpression> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Option<MessageSearchExpression> {
        let mut children = vec![self.parse_and_expression()?];
        while matches!(self.peek(), Some(SearchToken::Or)) {
            self.index += 1;
            children.push(self.parse_and_expression()?);
        }
        Some(collapse_expression_group(
            MessageSearchBoolean::Or,
            children,
        ))
    }

    fn parse_and_expression(&mut self) -> Option<MessageSearchExpression> {
        let mut children = vec![self.parse_primary()?];
        while matches!(self.peek(), Some(SearchToken::And)) {
            self.index += 1;
            children.push(self.parse_primary()?);
        }
        Some(collapse_expression_group(
            MessageSearchBoolean::And,
            children,
        ))
    }

    fn parse_primary(&mut self) -> Option<MessageSearchExpression> {
        match self.peek()? {
            SearchToken::OpenParen => {
                self.index += 1;
                let expression = self.parse_expression()?;
                if !matches!(self.peek(), Some(SearchToken::CloseParen)) {
                    return None;
                }
                self.index += 1;
                Some(expression)
            }
            SearchToken::Term(_) => self.parse_term_predicate(),
            SearchToken::CloseParen | SearchToken::And | SearchToken::Or => None,
        }
    }

    fn parse_term_predicate(&mut self) -> Option<MessageSearchExpression> {
        let SearchToken::Term(raw_term) = self.tokens.get(self.index)?.clone() else {
            return None;
        };
        self.index += 1;

        if let Some(rule) = parse_query_rule(&raw_term) {
            return Some(MessageSearchExpression::Predicate(
                MessageSearchPredicate::Rule {
                    field: rule.field,
                    operator: rule.operator,
                    value: rule.value,
                },
            ));
        }

        let normalized = strip_outer_quotes(&raw_term);
        if normalized.is_empty() {
            return None;
        }
        Some(MessageSearchExpression::Predicate(
            MessageSearchPredicate::PlainTerm(normalized),
        ))
    }

    fn peek(&self) -> Option<&SearchToken> {
        self.tokens.get(self.index)
    }

    fn has_remaining_tokens(&self) -> bool {
        self.index < self.tokens.len()
    }
}

fn collapse_expression_group(
    boolean: MessageSearchBoolean,
    mut children: Vec<MessageSearchExpression>,
) -> MessageSearchExpression {
    if children.len() == 1 {
        return children.remove(0);
    }

    MessageSearchExpression::Group { boolean, children }
}

fn tokenize_query_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote: Option<char> = None;

    for value in query.chars() {
        if matches!(value, '"' | '\'') {
            if !in_quotes {
                in_quotes = true;
                quote = Some(value);
                current.push(value);
                continue;
            }

            if Some(value) == quote {
                in_quotes = false;
                quote = None;
                current.push(value);
                continue;
            }
        }

        if value.is_whitespace() && !in_quotes {
            if !current.trim().is_empty() {
                terms.push(current.trim().to_owned());
            }
            current.clear();
            continue;
        }

        current.push(value);
    }

    if !current.trim().is_empty() {
        terms.push(current.trim().to_owned());
    }

    terms
}

fn strip_outer_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let is_double = trimmed.starts_with('"') &&
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/messages/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/rows.rs`
- Size bytes / Размер в байтах: `2305`
- Included characters / Включено символов: `2305`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::MessageProjectionError;
use super::models::{ProjectedMessage, ProjectedMessageSummary};
use super::payload::recipients_from_value;
use super::states::{LocalMessageState, WorkflowState};

pub(crate) fn row_to_projected_message_summary(
    row: PgRow,
) -> Result<ProjectedMessageSummary, MessageProjectionError> {
    let attachment_count = row.try_get("attachment_count")?;
    Ok(ProjectedMessageSummary {
        message: row_to_projected_message(row)?,
        attachment_count,
    })
}

pub(crate) fn row_to_projected_message(
    row: PgRow,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let workflow_state: String = row.try_get("workflow_state")?;
    let local_state: String = row.try_get("local_state")?;
    Ok(ProjectedMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        observation_id: row.try_get("observation_id")?,
        account_id: row.try_get("account_id")?,
        provider_record_id: row.try_get("provider_record_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        recipients: recipients_from_value(row.try_get("recipients")?)?,
        body_text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        conversation_id: row.try_get("conversation_id")?,
        sender_display_name: row.try_get("sender_display_name")?,
        delivery_state: row.try_get("delivery_state")?,
        message_metadata: row.try_get("message_metadata")?,
        workflow_state: workflow_state
            .parse::<WorkflowState>()
            .unwrap_or(WorkflowState::New),
        importance_score: row.try_get("importance_score")?,
        ai_category: row.try_get("ai_category")?,
        ai_summary: row.try_get("ai_summary")?,
        ai_summary_generated_at: row.try_get("ai_summary_generated_at")?,
        local_state: local_state
            .parse::<LocalMessageState>()
            .unwrap_or(LocalMessageState::Active),
        local_state_changed_at: row.try_get("local_state_changed_at")?,
        local_state_reason: row.try_get("local_state_reason")?,
    })
}
```

### `backend/src/domains/communications/messages/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/search.rs`
- Size bytes / Размер в байтах: `9588`
- Included characters / Включено символов: `9588`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, QueryBuilder};

use super::models::{MessageSearchMatchMode, MessageSearchQuery};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageSearchField {
    Subject,
    Body,
    Sender,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageSearchPredicateOperator {
    Contains,
    Equals,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageSearchPredicate {
    PlainTerm(String),
    Rule {
        field: MessageSearchField,
        operator: MessageSearchPredicateOperator,
        value: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageSearchBoolean {
    And,
    Or,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageSearchExpression {
    Predicate(MessageSearchPredicate),
    Group {
        boolean: MessageSearchBoolean,
        children: Vec<MessageSearchExpression>,
    },
}

impl MessageSearchExpression {
    pub fn term_count(&self) -> usize {
        match self {
            Self::Predicate(_) => 1,
            Self::Group { children, .. } => children.iter().map(Self::term_count).sum(),
        }
    }
}

pub fn append_message_search_filter<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    alias: &str,
    search: &MessageSearchQuery,
) {
    let Some(expression) = effective_message_search_expression(search) else {
        return;
    };

    builder.push(" AND ");
    push_expression(builder, alias, &expression);
}

pub fn effective_message_search_expression(
    search: &MessageSearchQuery,
) -> Option<MessageSearchExpression> {
    if let Some(expression) = search.expression.clone() {
        return Some(expression);
    }

    let boolean = if search.match_mode.is_any() {
        MessageSearchBoolean::Or
    } else {
        MessageSearchBoolean::And
    };
    let mut children = Vec::new();
    children.extend(search.plain_terms.iter().map(|value| {
        MessageSearchExpression::Predicate(MessageSearchPredicate::PlainTerm(value.clone()))
    }));
    extend_rule_terms(
        &mut children,
        MessageSearchField::Subject,
        MessageSearchPredicateOperator::Contains,
        &search.subject_contains,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Subject,
        MessageSearchPredicateOperator::Equals,
        &search.subject_equals,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Body,
        MessageSearchPredicateOperator::Contains,
        &search.body_contains,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Body,
        MessageSearchPredicateOperator::Equals,
        &search.body_equals,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Sender,
        MessageSearchPredicateOperator::Contains,
        &search.sender_contains,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Sender,
        MessageSearchPredicateOperator::Equals,
        &search.sender_equals,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::All,
        MessageSearchPredicateOperator::Contains,
        &search.all_contains,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::All,
        MessageSearchPredicateOperator::Equals,
        &search.all_equals,
    );

    match children.len() {
        0 => None,
        1 => children.into_iter().next(),
        _ => Some(MessageSearchExpression::Group { boolean, children }),
    }
}

fn extend_rule_terms(
    children: &mut Vec<MessageSearchExpression>,
    field: MessageSearchField,
    operator: MessageSearchPredicateOperator,
    values: &[String],
) {
    children.extend(values.iter().map(|value| {
        MessageSearchExpression::Predicate(MessageSearchPredicate::Rule {
            field,
            operator,
            value: value.clone(),
        })
    }));
}

fn push_expression<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    alias: &str,
    expression: &MessageSearchExpression,
) {
    match expression {
        MessageSearchExpression::Predicate(predicate) => push_predicate(builder, alias, predicate),
        MessageSearchExpression::Group { boolean, children } => {
            builder.push("(");
            for (index, child) in children.iter().enumerate() {
                if index > 0 {
                    builder.push(match boolean {
                        MessageSearchBoolean::And => " AND ",
                        MessageSearchBoolean::Or => " OR ",
                    });
                }
                push_expression(builder, alias, child);
            }
            builder.push(")");
        }
    }
}

fn push_predicate<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    alias: &str,
    predicate: &MessageSearchPredicate,
) {
    match predicate {
        MessageSearchPredicate::PlainTerm(value) => {
            push_contains_predicate(builder, &combined_search_target(alias), value)
        }
        MessageSearchPredicate::Rule {
            field,
            operator,
            value,
        } => match (field, operator) {
            (MessageSearchField::Subject, MessageSearchPredicateOperator::Contains) => {
                push_contains_predicate(builder, &coalesce_column(alias, "subject"), value)
            }
            (MessageSearchField::Subject, MessageSearchPredicateOperator::Equals) => {
                push_equals_predicate(builder, &coalesce_column(alias, "subject"), value)
            }
            (MessageSearchField::Body, MessageSearchPredicateOperator::Contains) => {
                push_contains_predicate(builder, &coalesce_column(alias, "body_text"), value)
            }
            (MessageSearchField::Body, MessageSearchPredicateOperator::Equals) => {
                push_equals_predicate(builder, &coalesce_column(alias, "body_text"), value)
            }
            (MessageSearchField::Sender, MessageSearchPredicateOperator::Contains) => {
                push_contains_predicate(builder, &coalesce_column(alias, "sender"), value)
            }
            (MessageSearchField::Sender, MessageSearchPredicateOperator::Equals) => {
                push_equals_predicate(builder, &coalesce_column(alias, "sender"), value)
            }
            (MessageSearchField::All, MessageSearchPredicateOperator::Contains) => {
                push_contains_predicate(builder, &combined_search_target(alias), value)
            }
            (MessageSearchField::All, MessageSearchPredicateOperator::Equals) => {
                builder.push("(");
                push_equals_predicate(builder, &coalesce_column(alias, "subject"), value);
                builder.push(" OR ");
                push_equals_predicate(builder, &coalesce_column(alias, "sender"), value);
                builder.push(" OR ");
                push_equals_predicate(builder, &coalesce_column(alias, "body_text"), value);
                builder.push(" OR ");
                push_equals_predicate(
                    builder,
                    &coalesce_column(alias, "provider_record_id"),
                    value,
                );
                builder.push(" OR ");
                push_equals_predicate(
                    builder,
                    &coalesce_column(alias, "sender_display_name"),
                    value,
                );
                builder.push(")");
            }
        },
    }
}

fn push_contains_predicate<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    target_sql: &str,
    value: &str,
) {
    builder.push("lower(");
    builder.push(target_sql);
    builder.push(") LIKE '%' || lower(");
    builder.push_bind(value.to_owned());
    builder.push(") || '%'");
}

fn push_equals_predicate<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    target_sql: &str,
    value: &str,
) {
    builder.push("lower(");
    builder.push(target_sql);
    builder.push(") = lower(");
    builder.push_bind(value.to_owned());
    builder.push(")");
}

fn coalesce_column(alias: &str, column: &str) -> String {
    format!("coalesce({alias}.{column}, '')")
}

fn combined_search_target(alias: &str) -> String {
    format!(
        "concat_ws(' ', {}, {}, {}, {}, {})",
        coalesce_column(alias, "subject"),
        coalesce_column(alias, "sender"),
        coalesce_column(alias, "body_text"),
        coalesce_column(alias, "provider_record_id"),
        coalesce_column(alias, "sender_display_name")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_expression_builds_legacy_any_group() {
        let query = MessageSearchQuery {
            plain_terms: vec!["invoice".to_owned()],
            sender_contains: vec!["alex".to_owned()],
            match_mode: MessageSearchMatchMode::Any,
            ..MessageSearchQuery::default()
        };

        assert_eq!(
            effective_message_search_expression(&query),
            Some(MessageSearchExpression::Group {
                boolean: MessageSearchBoolean::Or,
                children: vec![
                    MessageSearchExpression::Predicate(MessageSearchPredicate::PlainTerm(
                        "invoice".to_owned()
                    )),
                    MessageSearchExpression::Predicate(MessageSearchPredicate::Rule {
                        field: MessageSearchField::Sender,
                        operator: MessageSearchPredicateOperator::Contains,
                        value: "alex".to_owned(),
                    }),
                ],
            })
        );
    }
}
```

### `backend/src/domains/communications/messages/states.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/states.rs`
- Size bytes / Размер в байтах: `4486`
- Included characters / Включено символов: `4486`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use super::errors::MessageProjectionError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    New,
    Reviewed,
    NeedsAction,
    Waiting,
    Done,
    Archived,
    Muted,
    Spam,
}

impl WorkflowState {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowState::New => "new",
            WorkflowState::Reviewed => "reviewed",
            WorkflowState::NeedsAction => "needs_action",
            WorkflowState::Waiting => "waiting",
            WorkflowState::Done => "done",
            WorkflowState::Archived => "archived",
            WorkflowState::Muted => "muted",
            WorkflowState::Spam => "spam",
        }
    }

    pub fn valid_transitions(&self) -> &[WorkflowState] {
        match self {
            WorkflowState::New => &[
                WorkflowState::Reviewed,
                WorkflowState::NeedsAction,
                WorkflowState::Archived,
                WorkflowState::Muted,
                WorkflowState::Spam,
            ],
            WorkflowState::Reviewed => &[
                WorkflowState::New,
                WorkflowState::NeedsAction,
                WorkflowState::Waiting,
                WorkflowState::Done,
                WorkflowState::Archived,
                WorkflowState::Muted,
                WorkflowState::Spam,
            ],
            WorkflowState::NeedsAction => &[
                WorkflowState::Waiting,
                WorkflowState::Done,
                WorkflowState::Archived,
                WorkflowState::Reviewed,
            ],
            WorkflowState::Waiting => &[
                WorkflowState::NeedsAction,
                WorkflowState::Done,
                WorkflowState::Archived,
                WorkflowState::Reviewed,
            ],
            WorkflowState::Done => &[
                WorkflowState::Archived,
                WorkflowState::Reviewed,
                WorkflowState::NeedsAction,
            ],
            WorkflowState::Archived => &[
                WorkflowState::Reviewed,
                WorkflowState::NeedsAction,
                WorkflowState::Done,
            ],
            WorkflowState::Muted => &[WorkflowState::Reviewed, WorkflowState::Archived],
            WorkflowState::Spam => &[
                WorkflowState::Reviewed,
                WorkflowState::Archived,
                WorkflowState::New,
            ],
        }
    }

    pub fn is_valid_transition(from: &Self, to: &Self) -> bool {
        from.valid_transitions().contains(to)
    }
}

impl std::str::FromStr for WorkflowState {
    type Err = MessageProjectionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "new" => Ok(WorkflowState::New),
            "reviewed" => Ok(WorkflowState::Reviewed),
            "needs_action" => Ok(WorkflowState::NeedsAction),
            "waiting" => Ok(WorkflowState::Waiting),
            "done" => Ok(WorkflowState::Done),
            "archived" => Ok(WorkflowState::Archived),
            "muted" => Ok(WorkflowState::Muted),
            "spam" => Ok(WorkflowState::Spam),
            _ => Err(MessageProjectionError::InvalidWorkflowState(
                value.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalMessageState {
    Active,
    Trash,
    All,
}

impl LocalMessageState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LocalMessageState::Active => "active",
            LocalMessageState::Trash => "trash",
            LocalMessageState::All => "all",
        }
    }

    pub(crate) fn persisted_filter(&self) -> Option<&'static str> {
        match self {
            LocalMessageState::Active => Some("active"),
            LocalMessageState::Trash => Some("trash"),
            LocalMessageState::All => None,
        }
    }
}

impl std::str::FromStr for LocalMessageState {
    type Err = MessageProjectionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "active" => Ok(LocalMessageState::Active),
            "trash" => Ok(LocalMessageState::Trash),
            "all" => Ok(LocalMessageState::All),
            _ => Err(MessageProjectionError::InvalidLocalState(value.to_owned())),
        }
    }
}
```

### `backend/src/domains/communications/messages/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/store.rs`
- Size bytes / Размер в байтах: `291`
- Included characters / Включено символов: `291`
- Truncated / Обрезано: `no`

```rust
mod local_state;
mod metadata;
mod participants;
mod queries;
mod upsert;
mod workflow;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct MessageProjectionStore {
    pool: PgPool,
}

impl MessageProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### `backend/src/domains/communications/messages/store/local_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/store/local_state.rs`
- Size bytes / Размер в байтах: `5314`
- Included characters / Включено символов: `5314`
- Truncated / Обрезано: `no`

```rust
use super::MessageProjectionStore;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::rows::row_to_projected_message;
use crate::domains::communications::messages::validation::validate_non_empty;

impl MessageProjectionStore {
    pub async fn move_to_local_trash(
        &self,
        message_id: &str,
        reason: &str,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.move_to_local_trash_with_observation(
            message_id,
            reason,
            None,
            "local_state_transition",
            None,
        )
        .await
    }

    pub async fn move_to_local_trash_with_observation(
        &self,
        message_id: &str,
        reason: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        validate_non_empty("local_state_reason", reason)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"UPDATE communication_messages
            SET local_state = 'trash',
                local_state_changed_at = now(),
                local_state_reason = $2,
                projected_at = now()
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
        .bind(reason.trim())
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        let message = row_to_projected_message(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                message.message_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "local_state": message.local_state.as_str(),
                    "source": reason,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(message)
    }

    pub async fn restore_from_local_trash(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.restore_from_local_trash_with_observation(
            message_id,
            None,
            "local_state_transition",
            None,
        )
        .await
    }

    pub async fn restore_from_local_trash_with_observation(
        &self,
        message_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"UPDATE communication_messages
            SET local_state = 'active',
                local_state_changed_at = now(),
                local_state_reason = NULL,
                projected_at = now()
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
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        let message = row_to_projected_message(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                message.message_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "local_state": message.local_state.as_str(),
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(message)
    }
}
```

### `backend/src/domains/communications/messages/store/metadata.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/store/metadata.rs`
- Size bytes / Размер в байтах: `4575`
- Included characters / Включено символов: `4575`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::MessageProjectionStore;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::rows::row_to_projected_message;
use crate::domains::communications::messages::validation::validate_non_empty;

impl MessageProjectionStore {
    pub async fn set_ai_analysis(
        &self,
        message_id: &str,
        category: Option<&str>,
        summary: Option<&str>,
        importance_score: Option<i16>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        if let Some(score) = importance_score
            && !(0..=100).contains(&score)
        {
            return Err(MessageProjectionError::InvalidImportanceScore(score));
        }
        let row = sqlx::query(
            r#"UPDATE communication_messages SET
                ai_category = COALESCE($2, ai_category),
                ai_summary = COALESCE($3, ai_summary),
                ai_summary_generated_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE ai_summary_generated_at END,
                importance_score = COALESCE($4, importance_score),
                projected_at = now()
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
        .bind(category)
        .bind(summary)
        .bind(importance_score)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }

    pub async fn set_message_metadata(
        &self,
        message_id: &str,
        metadata: &Value,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.set_message_metadata_with_observation(
            message_id,
            metadata,
            None,
            "message_flag_update",
            None,
        )
        .await
    }

    pub async fn set_message_metadata_with_observation(
        &self,
        message_id: &str,
        metadata: &Value,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        if !metadata.is_object() {
            return Err(MessageProjectionError::InvalidMessageMetadata);
        }
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"UPDATE communication_messages SET message_metadata = $2, projected_at = now()
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
        .bind(metadata)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        let message = row_to_projected_message(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                message.message_id.clone(),
                relationship_kind,
                serde_json::json!({}),
                link_metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(message)
    }
}
```

### `backend/src/domains/communications/messages/store/participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/store/participants.rs`
- Size bytes / Размер в байтах: `2154`
- Included characters / Включено символов: `2154`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::Row;

use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage,
};

impl MessageProjectionStore {
    pub async fn upsert_email_participant(
        &self,
        message: &ProjectedMessage,
        person_id: &str,
        email_address: &str,
        display_name: Option<&str>,
        role: &str,
    ) -> Result<bool, MessageProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_message_participants (
                message_id, person_id, email_address, display_name, role, source, confidence
            )
            VALUES ($1, $2, $3, $4, $5, 'email_sync', 1.0)
            ON CONFLICT (message_id, person_id, role, email_address)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                updated_at = now()
            RETURNING id::text AS participant_id, (xmax = 0) AS inserted
            "#,
        )
        .bind(&message.message_id)
        .bind(person_id)
        .bind(email_address)
        .bind(display_name)
        .bind(role)
        .fetch_one(&mut *transaction)
        .await?;
        let participant_id: String = row.try_get("participant_id")?;
        let inserted: bool = row.try_get("inserted")?;

        link_mail_entity_in_transaction(
            &mut transaction,
            &message.observation_id,
            "message_participant",
            participant_id,
            "email_sync_participant",
            json!({
                "message_id": message.message_id,
                "person_id": person_id,
                "email_address": email_address,
                "display_name": display_name,
                "role": role,
                "source": "email_sync",
            }),
            None,
        )
        .await?;

        transaction.commit().await?;
        Ok(inserted)
    }
}
```

### `backend/src/domains/communications/messages/store/queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/messages/store/queries.rs`
- Size bytes / Размер в байтах: `14586`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder, Row};

use super::MessageProjectionStore;
use crate::domains::communications::messages::append_message_search_filter;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::{
    MessageSearchMatchMode, MessageSearchQuery, ProjectedMessage, ProjectedMessagePage,
    ProjectedMessagePageQuery, ProjectedMessageSummary, WorkflowStateCount,
};
use crate::domains::communications::messages::rows::{
    row_to_projected_message, row_to_projected_message_summary,
};
use crate::domains::communications::messages::states::{LocalMessageState, WorkflowState};
use crate::domains::communications::messages::validation::{validate_limit, validate_non_empty};

const TELEGRAM_CHANNEL_KIND_ALIAS: &[&str] = &["telegram_user", "telegram_bot"];
const WHATSAPP_CHANNEL_KIND_ALIAS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];
const MAIL_CHANNEL_KIND_ALIAS: &[&str] = &["email"];

impl MessageProjectionStore {
    pub async fn recent_messages(
        &self,
        limit: i64,
    ) -> Result<Vec<ProjectedMessageSummary>, MessageProjectionError> {
        let limit = validate_limit(limit)?;
        let rows = sqlx::query(
            r#"
            SELECT
                m.message_id,
                m.raw_record_id,
                m.observation_id,
                m.account_id,
                m.provider_record_id,
                m.subject,
                m.sender,
                m.recipients,
                m.body_text,
                m.occurred_at,
                m.projected_at,
                m.channel_kind,
                m.conversation_id,
                m.sender_display_name,
                m.delivery_state,
                m.message_metadata,
                m.workflow_state,
                m.importance_score,
                m.ai_category,
                m.ai_summary,
                m.ai_summary_generated_at,
                m.local_state,
                m.local_state_changed_at,
                m.local_state_reason,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM communication_messages m
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            WHERE m.local_state = 'active'
            GROUP BY
                m.message_id,
                m.raw_record_id,
                m.observation_id,
                m.account_id,
                m.provider_record_id,
                m.subject,
                m.sender,
                m.recipients,
                m.body_text,
                m.occurred_at,
                m.projected_at,
                m.channel_kind,
                m.conversation_id,
                m.sender_display_name,
                m.delivery_state,
                m.message_metadata,
                m.workflow_state,
                m.importance_score,
                m.ai_category,
                m.ai_summary,
                m.ai_summary_generated_at,
                m.local_state,
                m.local_state_changed_at,
                m.local_state_reason
            ORDER BY
                COALESCE(m.occurred_at, m.projected_at) DESC,
                m.projected_at DESC,
                m.message_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_projected_message_summary)
            .collect()
    }

    pub async fn message(
        &self,
        message_id: &str,
    ) -> Result<Option<ProjectedMessage>, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;

        let row = sqlx::query(
            r#"
            SELECT
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
            FROM communication_messages
            WHERE message_id = $1
            "#,
        )
        .bind(message_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_projected_message).transpose()
    }

    pub async fn list_messages(
        &self,
        account_id: Option<&str>,
        workflow_state: Option<WorkflowState>,
        channel_kind: Option<&str>,
        query: Option<&str>,
        local_state: LocalMessageState,
        limit: i64,
    ) -> Result<Vec<ProjectedMessageSummary>, MessageProjectionError> {
        Ok(self
            .list_messages_page(ProjectedMessagePageQuery {
                account_id,
                workflow_state,
                channel_kind,
                conversation_id: None,
                query,
                match_mode: MessageSearchMatchMode::All,
                search: MessageSearchQuery::default(),
                local_state,
                cursor: None,
                limit,
            })
            .await?
            .items)
    }

    pub async fn list_messages_page(
        &self,
        request: ProjectedMessagePageQuery<'_>,
    ) -> Result<ProjectedMessagePage, MessageProjectionError> {
        let limit = validate_limit(request.limit)?;
        let workflow_state_str = request.workflow_state.map(|s| s.as_str().to_owned());
        let local_state_filter = request.local_state.persisted_filter();
        let query = request
            .query
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        let search = if request.search.is_empty() {
            fallback_message_search(query.as_deref(), request.match_mode)
        } else {
            request.search.clone()
        };
        let cursor = request
            .cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_message_list_cursor)
            .transpose()?;
        let cursor_sort_at = cursor.as_ref().map(|cursor| cursor.sort_at);
        let cursor_projected_at = cursor.as_ref().map(|cursor| cursor.projected_at);
        let cursor_message_id = cursor.as_ref().map(|cursor| cursor.message_id.as_str());
        let fetch_limit = limit + 1;
        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                m.message_id, m.raw_record_id, m.observation_id, m.account_id, m.provider_record_id,
                m.subject, m.sender, m.recipients, m.body_text,
                m.occurred_at, m.projected_at, m.channel_kind, m.conversation_id,
                m.sender_display_name, m.delivery_state, m.message_metadata,
                m.workflow_state, m.importance_score, m.ai_category,
                m.ai_summary, m.ai_summary_generated_at,
                m.local_state, m.local_state_changed_at, m.local_state_reason,
                count(a.attachment_id)::BIGINT AS attachment_count
            FROM communication_messages m
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            WHERE 1 = 1
            "#,
        );
        if let Some(account_id) = request.account_id {
            builder.push(" AND m.account_id = ");
            builder.push_bind(account_id);
        }
        if let Some(workflow_state) = workflow_state_str.as_deref() {
            builder.push(" AND m.workflow_state = ");
            builder.push_bind(workflow_state);
        }
        if let Some(channel_kind) = request.channel_kind {
            append_channel_kind_filter(&mut builder, channel_kind);
        }
        if let Some(conversation_id) = request.conversation_id {
            builder.push(" AND m.conversation_id = ");
            builder.push_bind(conversation_id);
        }
        if let Some(local_state) = local_state_filter {
            builder.push(" AND m.local_state = ");
            builder.push_bind(local_state);
        }
        append_message_search_filter(&mut builder, "m", &search);
        if let Some(sort_at) = cursor_sort_at {
            builder.push(" AND (COALESCE(m.occurred_at, m.projected_at) < ");
            builder.push_bind(sort_at);
            builder.push(" OR (COALESCE(m.occurred_at, m.projected_at) = ");
            builder.push_bind(sort_at);
            builder.push(" AND m.projected_at < ");
            builder.push_bind(cursor_projected_at.expect("cursor projected_at"));
            builder.push(") OR (COALESCE(m.occurred_at, m.projected_at) = ");
            builder.push_bind(sort_at);
            builder.push(" AND m.projected_at = ");
            builder.push_bind(cursor_projected_at.expect("cursor projected_at"));
            builder.push(" AND m.message_id > ");
            builder.push_bind(cursor_message_id.expect("cursor message_id"));
            builder.push("))");
        }
        builder.push(
            r#"
            GROUP BY
                m.message_id, m.raw_record_id, m.observation_id, m.account_id, m.provider_record_id,
                m.subject, m.sender, m.recipients, m.body_text,
                m.occurred_at, m.projected_at, m.channel_kind, m.conversation_id,
                m.sender_display_name, m.delivery_state, m.message_metadata,
                m.workflow_state, m.importance_score, m.ai_category,
                m.ai_summary, m.ai_summary_generated_at,
                m.local_state, m.local_state_changed_at, m.local_state_reason
            ORDER BY COALESCE(m.occurred_at, m.projected_at) DESC, m.projected_at DESC, m.message_id ASC
            LIMIT
            "#,
        );
        builder.push_bind(fetch_limit);
        let rows = builder.build().fetch_all(&self.pool).await?;
        let has_more = rows.len() > limit as usize;
        let summaries = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_projected_message_summary)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            summaries
                .last()
                .map(|summary| encode_message_list_cursor(&summary.message))
                .transpose()?
        } else {
            None
        };

        Ok(ProjectedMessagePage {
            items: summaries,
            next_cursor,
            has_more,
        })
    }

    pub async fn count_messages_by_state(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<WorkflowStateCount>, MessageProjectionError> {
        self.count_messages_by_state_with_local_state(account_id, LocalMessageState::Active)
            .await
    }

    pub async fn count_messages_by_state_with_local_state(
        &self,
        account_id: Option<&str>,
        local_state: LocalMessageState,
    ) -> Result<Vec<WorkflowStateCount>, MessageProjectionError> {
        let local_state_filter = local_state.persisted_filter();
        let rows = sqlx::query(
            r#"SELECT m.workflow_state, count(*)::BIGINT AS msg_count
            FROM communication_messages m
            WHERE ($1::text IS NULL OR m.account_id = $1)
              AND ($2::text IS NULL OR m.local_state = $2)
            GROUP BY m.workflow_state ORDER BY m.workflow_state"#,
        )
        .bind(account_id)
        .bind(local_state_filter)
        .fetch_all(&self.pool)
        .await?;
        let mut counts = Vec::new();
        for ro
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
