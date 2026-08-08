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

- Chunk ID / ID чанка: `074-source-backend-part-054`
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

### `backend/src/workflows/email_intelligence/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_intelligence/tests.rs`
- Size bytes / Размер в байтах: `5736`
- Included characters / Включено символов: `5736`
- Truncated / Обрезано: `no`

```rust
use super::*;
use crate::domains::communications::messages::{
    LocalMessageState, ProjectedMessage, WorkflowState,
};
use chrono::Utc;
use serde_json::json;

fn test_message(subject: &str, body: &str) -> ProjectedMessage {
    ProjectedMessage {
        message_id: "msg:test:1".into(),
        raw_record_id: "raw:1".into(),
        observation_id: "observation:1".into(),
        account_id: "acct:1".into(),
        provider_record_id: "prov:1".into(),
        subject: subject.into(),
        sender: "sender@example.com".into(),
        recipients: vec!["recipient@example.com".into()],
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
fn heuristic_score_urgent_subject() {
    let msg = test_message("URGENT: Action Required", "Please respond ASAP");
    let score = EmailIntelligenceService::heuristic_score(&msg);
    assert!(score >= 35, "got {score}");
}

#[test]
fn heuristic_score_finance_body() {
    let msg = test_message(
        "Update",
        "Please find the invoice attached for payment. Amount due: $500",
    );
    let score = EmailIntelligenceService::heuristic_score(&msg);
    assert!(score >= 50, "got {score}");
}

#[test]
fn heuristic_score_marketing_body() {
    let msg = test_message(
        "Digest",
        "Click here. To unsubscribe, click here. If you no longer wish to receive...",
    );
    let score = EmailIntelligenceService::heuristic_score(&msg);
    assert!(score <= 30, "got {score}");
}

#[test]
fn heuristic_category_finance() {
    let msg = test_message("Invoice #123", "Here is your invoice for services");
    assert_eq!(
        EmailIntelligenceService::heuristic_category(&msg).as_deref(),
        Some("finance")
    );
}

#[test]
fn heuristic_category_legal() {
    let msg = test_message("Contract", "Please review the NDA and agreement");
    assert_eq!(
        EmailIntelligenceService::heuristic_category(&msg).as_deref(),
        Some("legal")
    );
}

#[test]
fn heuristic_category_none() {
    let msg = test_message("Hello", "Just checking in");
    assert!(EmailIntelligenceService::heuristic_category(&msg).is_none());
}

#[test]
fn heuristic_structured_summary_extracts_key_points_actions_risks_and_deadlines() {
    let msg = test_message(
        "Action Required: Contract review deadline",
        "Please review the NDA by Friday. The payment risk remains open. Confirm approval before EOD.",
    );

    let summary = EmailIntelligenceService::heuristic_structured_summary(&msg);

    assert!(
        summary
            .key_points
            .contains(&"Action Required: Contract review deadline".to_owned())
    );
    assert!(
        summary
            .action_items
            .iter()
            .any(|item| item.contains("Please review the NDA"))
    );
    assert!(
        summary
            .risks
            .iter()
            .any(|risk| risk.contains("payment risk"))
    );
    assert!(
        summary
            .deadlines
            .iter()
            .any(|deadline| deadline.contains("Friday"))
    );
}

#[test]
fn heuristic_structured_summary_extracts_mail_knowledge_candidates() {
    let msg = test_message(
        "Contract review with Acme Corp",
        "From: Ada Lovelace <ada@acme.example>\nPlease review the attached MSA and NDA before Friday. Meeting on Monday at 10:00 with Acme Corp.",
    );

    let summary = EmailIntelligenceService::heuristic_structured_summary(&msg);

    assert!(
        summary
            .event_candidates
            .iter()
            .any(|candidate| candidate.title.contains("Meeting on Monday"))
    );
    assert!(
        summary
            .persona_candidates
            .iter()
            .any(|candidate| candidate.title.contains("Ada Lovelace"))
    );
    assert!(
        summary
            .organization_candidates
            .iter()
            .any(|candidate| candidate.title.contains("acme.example"))
    );
    assert!(
        summary
            .document_candidates
            .iter()
            .any(|candidate| candidate.title.contains("MSA"))
    );
    assert!(
        summary
            .agreement_candidates
            .iter()
            .any(|candidate| candidate.title.contains("NDA"))
    );
}

#[test]
fn heuristic_structured_summary_is_bounded_and_deduplicated() {
    let msg = test_message(
        "Deadline reminder",
        "Deadline reminder. Deadline reminder. Please confirm. Please confirm.",
    );

    let summary = EmailIntelligenceService::heuristic_structured_summary(&msg);

    assert_eq!(summary.key_points, vec!["Deadline reminder"]);
    assert_eq!(summary.action_items, vec!["Please confirm"]);
    assert_eq!(summary.deadlines, vec!["Deadline reminder"]);
    assert!(summary.event_candidates.is_empty());
    assert_eq!(summary.persona_candidates.len(), 1);
    assert_eq!(summary.persona_candidates[0].title, "sender@example.com");
}

#[test]
fn email_category_from_str_all_valid() {
    assert_eq!(
        EmailCategory::parse("critical"),
        Some(EmailCategory::Critical)
    );
    assert_eq!(EmailCategory::parse("spam"), Some(EmailCategory::Spam));
    assert_eq!(
        EmailCategory::parse("finance"),
        Some(EmailCategory::Finance)
    );
}
```

### `backend/src/workflows/email_sync_pipeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline.rs`
- Size bytes / Размер в байтах: `451`
- Included characters / Включено символов: `451`
- Truncated / Обрезано: `no`

```rust
mod attachments;
mod candidates;
mod errors;
mod ids;
mod knowledge;
mod organizations;
mod participants;
mod raw_payload;
mod raw_records;
mod recording;
mod relationships;
mod report;
mod service;

pub use errors::{EmailSyncPipelineError, EmailSyncRecordError};
pub use recording::{record_email_sync_batch, record_email_sync_batch_with_mail_blobs};
pub use report::EmailSyncPipelineReport;
pub use service::project_email_sync_batch_with_mail_blobs;
```

### `backend/src/workflows/email_sync_pipeline/attachments.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/attachments.rs`
- Size bytes / Размер в байтах: `3501`
- Included characters / Включено символов: `3501`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::core::StoredRawCommunicationRecord;
use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::communications::storage::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanStatus, AttachmentSafetyScanner,
    CommunicationAttachmentDisposition, CommunicationBlobMetadataPort, LocalCommunicationBlobPort,
    NewCommunicationAttachment, NewCommunicationBlob,
};
use crate::platform::communications::rfc822::{
    ParsedEmailAttachment, ParsedEmailAttachmentDisposition,
};

use super::errors::EmailSyncPipelineError;

#[derive(Default)]
pub(crate) struct AttachmentProjectionReport {
    pub(crate) attachment_blobs_upserted: usize,
    pub(crate) attachments_extracted: usize,
    pub(crate) attachments_not_scanned: usize,
}

pub(crate) async fn project_attachments(
    mail_store: &CommunicationBlobMetadataPort,
    blob_store: &LocalCommunicationBlobPort,
    raw_record: &StoredRawCommunicationRecord,
    message: &ProjectedMessage,
    attachments: &[ParsedEmailAttachment],
    attachment_scanner: &impl AttachmentSafetyScanner,
) -> Result<AttachmentProjectionReport, EmailSyncPipelineError> {
    let mut report = AttachmentProjectionReport::default();

    for parsed_attachment in attachments {
        let local_blob = blob_store.put_blob(&parsed_attachment.body_bytes).await?;
        let blob = mail_store
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob)
                    .content_type(&parsed_attachment.content_type),
            )
            .await?;
        let scan_report = attachment_scanner.scan(&AttachmentSafetyScanRequest {
            provider_attachment_id: &parsed_attachment.provider_attachment_id,
            filename: parsed_attachment.filename.as_deref(),
            content_type: &parsed_attachment.content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &blob.sha256,
            storage_kind: &blob.storage_kind,
            storage_path: &blob.storage_path,
            bytes: &parsed_attachment.body_bytes,
        })?;
        let scan_status = scan_report.status;

        let mut attachment = NewCommunicationAttachment::new(
            &message.message_id,
            &raw_record.raw_record_id,
            &blob.blob_id,
            &parsed_attachment.provider_attachment_id,
            &parsed_attachment.content_type,
            local_blob.size_bytes,
            &blob.sha256,
        )
        .disposition(mail_attachment_disposition(parsed_attachment.disposition))
        .scan_report(scan_report);

        if let Some(filename) = &parsed_attachment.filename {
            attachment = attachment.filename(filename);
        }

        mail_store.upsert_attachment(&attachment).await?;
        report.attachment_blobs_upserted += 1;
        report.attachments_extracted += 1;
        if scan_status == AttachmentSafetyScanStatus::NotScanned {
            report.attachments_not_scanned += 1;
        }
    }

    Ok(report)
}

fn mail_attachment_disposition(
    disposition: ParsedEmailAttachmentDisposition,
) -> CommunicationAttachmentDisposition {
    match disposition {
        ParsedEmailAttachmentDisposition::Attachment => {
            CommunicationAttachmentDisposition::Attachment
        }
        ParsedEmailAttachmentDisposition::Inline => CommunicationAttachmentDisposition::Inline,
        ParsedEmailAttachmentDisposition::Unknown => CommunicationAttachmentDisposition::Unknown,
    }
}
```

### `backend/src/workflows/email_sync_pipeline/candidates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/candidates.rs`
- Size bytes / Размер в байтах: `1387`
- Included characters / Включено символов: `1387`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;
use crate::workflows::review_inbox::{
    refresh_message_decisions_into_review, refresh_message_knowledge_candidates_into_review,
    refresh_message_task_candidates_into_review,
};

use super::errors::EmailSyncPipelineError;

#[derive(Default)]
pub(crate) struct MessageCandidateRefreshReport {
    pub(crate) refreshed_decision_candidates: usize,
    pub(crate) refreshed_knowledge_candidates: usize,
    pub(crate) refreshed_task_candidates: usize,
}

pub(crate) async fn refresh_message_context_candidates(
    pool: &PgPool,
    messages: &[ProjectedMessage],
) -> Result<MessageCandidateRefreshReport, EmailSyncPipelineError> {
    let message_ids = messages
        .iter()
        .map(|message| message.message_id.clone())
        .collect::<Vec<_>>();
    if message_ids.is_empty() {
        return Ok(MessageCandidateRefreshReport::default());
    }

    Ok(MessageCandidateRefreshReport {
        refreshed_decision_candidates: refresh_message_decisions_into_review(pool, &message_ids)
            .await?,
        refreshed_knowledge_candidates: refresh_message_knowledge_candidates_into_review(
            pool, messages,
        )
        .await?,
        refreshed_task_candidates: refresh_message_task_candidates_into_review(pool, &message_ids)
            .await?,
    })
}
```

### `backend/src/workflows/email_sync_pipeline/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/errors.rs`
- Size bytes / Размер в байтах: `2975`
- Included characters / Включено символов: `2975`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::communications::core::CommunicationIngestionError;
use crate::domains::communications::messages::{
    CommunicationSignalProjectionError, MessageProjectionError,
};
use crate::domains::communications::storage::{
    AttachmentSafetyScanError, CommunicationStorageError,
};
use crate::domains::decisions::DecisionReviewPortError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::organizations::core::OrgCoreError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::domains::relationships::RelationshipReviewPortError;
use crate::domains::signal_hub::SignalHubError;
use crate::domains::tasks::candidates::TaskCandidateError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;

#[derive(Debug, Error)]
pub enum EmailSyncRecordError {
    #[error("email sync record field must not be empty: {0}")]
    EmptyField(&'static str),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error("email sync payload must be a JSON object before raw blob projection")]
    InvalidRawPayloadObject,

    #[error("email sync payload missing provider raw field: {field}")]
    MissingRawPayloadField { field: &'static str },

    #[error("email sync payload field {field} is invalid base64: {source}")]
    InvalidRawPayloadBase64 {
        field: &'static str,
        #[source]
        source: base64::DecodeError,
    },

    #[error("email sync does not support provider kind: {0}")]
    UnsupportedProviderKind(String),
}

#[derive(Debug, Error)]
pub enum EmailSyncPipelineError {
    #[error(transparent)]
    Sync(#[from] EmailSyncRecordError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error(transparent)]
    SignalProjection(#[from] CommunicationSignalProjectionError),

    #[error(transparent)]
    Contact(#[from] PersonProjectionError),

    #[error(transparent)]
    PersonMemory(#[from] PersonMemoryError),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error(transparent)]
    AttachmentScan(#[from] AttachmentSafetyScanError),

    #[error(transparent)]
    Decision(#[from] DecisionReviewPortError),

    #[error(transparent)]
    Organization(#[from] OrganizationError),

    #[error(transparent)]
    OrganizationCore(#[from] OrgCoreError),

    #[error(transparent)]
    Relationship(#[from] RelationshipReviewPortError),

    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid email participant address: {0}")]
    InvalidParticipantEmail(String),
}
```

### `backend/src/workflows/email_sync_pipeline/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/ids.rs`
- Size bytes / Размер в байтах: `663`
- Included characters / Включено символов: `663`
- Truncated / Обрезано: `no`

```rust
pub(super) const EMAIL_MESSAGE_RECORD_KIND: &str = "email_message";

pub(super) fn raw_record_id(
    account_id: &str,
    record_kind: &str,
    provider_record_id: &str,
) -> String {
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
```

### `backend/src/workflows/email_sync_pipeline/knowledge.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/knowledge.rs`
- Size bytes / Размер в байтах: `2660`
- Included characters / Включено символов: `2660`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeSet;

use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::persons::api::PersonProjectionPort;

use super::errors::EmailSyncPipelineError;
use super::organizations::project_email_participant_organization;
use super::participants::{parse_email_participant, upsert_message_participant};
use super::relationships::insert_relationship_event;

#[derive(Default)]
pub(crate) struct MessageKnowledgeReport {
    pub(crate) upserted_persons: usize,
    pub(crate) upserted_person_identities: usize,
    pub(crate) upserted_message_participants: usize,
    pub(crate) upserted_relationship_events: usize,
    pub(crate) upserted_organizations: usize,
    pub(crate) upserted_organization_contact_links: usize,
}

pub(crate) async fn project_message_knowledge(
    pool: &PgPool,
    person_store: &PersonProjectionPort,
    messages: &[ProjectedMessage],
) -> Result<MessageKnowledgeReport, EmailSyncPipelineError> {
    let mut report = MessageKnowledgeReport::default();
    let mut seen_persons = BTreeSet::new();

    for message in messages {
        let mut participants = Vec::new();
        participants.push(parse_email_participant(&message.sender, "sender")?);
        for recipient in &message.recipients {
            participants.push(parse_email_participant(recipient, "recipient")?);
        }

        for participant in participants {
            let person = person_store
                .upsert_email_person_with_observation(
                    &participant.email_address,
                    &message.observation_id,
                )
                .await?;
            if seen_persons.insert(person.person_id.clone()) {
                report.upserted_persons += 1;
                report.upserted_person_identities += 1;
            }
            if upsert_message_participant(pool, message, &person.person_id, &participant).await? {
                report.upserted_message_participants += 1;
            }
            if insert_relationship_event(pool, message, &person.person_id, &participant).await? {
                report.upserted_relationship_events += 1;
            }

            let organization_report = project_email_participant_organization(
                pool,
                &person.person_id,
                message,
                &participant,
            )
            .await?;
            report.upserted_organizations += organization_report.upserted_organizations;
            report.upserted_organization_contact_links +=
                organization_report.upserted_organization_contact_links;
        }
    }

    Ok(report)
}
```

### `backend/src/workflows/email_sync_pipeline/organizations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/organizations.rs`
- Size bytes / Размер в байтах: `5504`
- Included characters / Включено символов: `5504`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::organizations::api::OrganizationCommandPort;
use crate::domains::organizations::core::{OrgContactLink, OrganizationContactLinkPort};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewPort, RelationshipReviewState,
};

use super::errors::EmailSyncPipelineError;
use super::participants::EmailParticipant;

#[derive(Default)]
pub(crate) struct OrganizationProjectionReport {
    pub(crate) upserted_organizations: usize,
    pub(crate) upserted_organization_contact_links: usize,
}

pub(crate) async fn project_email_participant_organization(
    pool: &PgPool,
    person_id: &str,
    message: &ProjectedMessage,
    participant: &EmailParticipant,
) -> Result<OrganizationProjectionReport, EmailSyncPipelineError> {
    let Some(domain) = organization_domain_for_email(&participant.email_address) else {
        return Ok(OrganizationProjectionReport::default());
    };

    let organization_id =
        upsert_email_domain_organization(pool, &domain, &message.observation_id).await?;
    let organization_inserted = organization_id.is_some();
    let organization_id = organization_id.unwrap_or_else(|| organization_id_for_domain(&domain));
    let contact_link_inserted =
        upsert_organization_contact_link(pool, &organization_id, person_id, message, participant)
            .await?;

    Ok(OrganizationProjectionReport {
        upserted_organizations: usize::from(organization_inserted),
        upserted_organization_contact_links: usize::from(contact_link_inserted),
    })
}

fn organization_domain_for_email(email_address: &str) -> Option<String> {
    let domain = email_address.split('@').nth(1)?.trim().to_ascii_lowercase();
    if domain.is_empty() || is_public_mail_domain(&domain) {
        None
    } else {
        Some(domain)
    }
}

fn is_public_mail_domain(domain: &str) -> bool {
    matches!(
        domain,
        "gmail.com"
            | "googlemail.com"
            | "icloud.com"
            | "me.com"
            | "mac.com"
            | "outlook.com"
            | "hotmail.com"
            | "live.com"
            | "yahoo.com"
            | "proton.me"
            | "protonmail.com"
            | "mail.ru"
            | "yandex.ru"
    )
}

fn organization_id_for_domain(domain: &str) -> String {
    format!("org:v1:email-domain:{}:{domain}", domain.len())
}

async fn upsert_email_domain_organization(
    pool: &PgPool,
    domain: &str,
    observation_id: &str,
) -> Result<Option<String>, EmailSyncPipelineError> {
    let (_, inserted) = OrganizationCommandPort::new(pool.clone())
        .upsert_email_domain_organization_with_observation(domain, observation_id)
        .await?;
    Ok(inserted.then(|| organization_id_for_domain(domain)))
}

async fn upsert_organization_contact_link(
    pool: &PgPool,
    organization_id: &str,
    person_id: &str,
    message: &ProjectedMessage,
    _participant: &EmailParticipant,
) -> Result<bool, EmailSyncPipelineError> {
    let (link, inserted) = OrganizationContactLinkPort::new(pool.clone())
        .link_email_participant_with_observation(
            organization_id,
            person_id,
            &message.message_id,
            &message.observation_id,
        )
        .await?;
    materialize_email_participant_member_relationship(
        pool,
        &link,
        &message.message_id,
        &message.observation_id,
    )
    .await?;
    Ok(inserted)
}

async fn materialize_email_participant_member_relationship(
    pool: &PgPool,
    link: &OrgContactLink,
    message_id: &str,
    observation_id: &str,
) -> Result<(), EmailSyncPipelineError> {
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: link.person_id.clone(),
        target_entity_kind: RelationshipEntityKind::Organization,
        target_entity_id: link.organization_id.clone(),
        relationship_type: "member_of".to_owned(),
        trust_score: 0.5,
        strength_score: 0.5,
        confidence: link.confidence,
        review_state: RelationshipReviewState::SystemAccepted,
        valid_from: link.valid_from,
        valid_to: link.valid_to,
        metadata: serde_json::json!({
            "compatibility_table": "organization_contact_links",
            "compatibility_record_id": link.id,
            "organization_id": link.organization_id,
            "person_id": link.person_id,
            "role": link.role,
            "department": link.department,
            "source": link.source,
        }),
    };
    let evidence = NewRelationshipEvidence {
        source_kind: RelationshipEvidenceSourceKind::Communication,
        source_id: message_id.to_owned(),
        observation_id: Some(observation_id.to_owned()),
        excerpt: Some(
            "Persona is linked to organization through compatibility organization contact data."
                .to_owned(),
        ),
        metadata: serde_json::json!({
            "compatibility_table": "organization_contact_links",
            "compatibility_record_id": link.id,
            "organization_id": link.organization_id,
            "person_id": link.person_id,
        }),
    };
    let _ = RelationshipReviewPort::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    Ok(())
}
```

### `backend/src/workflows/email_sync_pipeline/participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/participants.rs`
- Size bytes / Размер в байтах: `1909`
- Included characters / Включено символов: `1909`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::communications::messages::{
    CommunicationMessageProjectionPort, ProjectedMessage,
};

use super::errors::EmailSyncPipelineError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EmailParticipant {
    pub(crate) email_address: String,
    pub(crate) display_name: Option<String>,
    pub(crate) role: &'static str,
}

pub(crate) fn parse_email_participant(
    raw: &str,
    role: &'static str,
) -> Result<EmailParticipant, EmailSyncPipelineError> {
    let trimmed = raw.trim();
    let (display_name, email) = if let Some((name, tail)) = trimmed.rsplit_once('<') {
        if let Some((addr, _)) = tail.split_once('>') {
            (clean_display_name(name), addr.trim())
        } else {
            (None, trimmed)
        }
    } else {
        (None, trimmed)
    };
    let email_address = email.trim_matches('"').trim().to_ascii_lowercase();
    if email_address.is_empty() || !email_address.contains('@') {
        return Err(EmailSyncPipelineError::InvalidParticipantEmail(
            raw.to_owned(),
        ));
    }
    Ok(EmailParticipant {
        email_address,
        display_name,
        role,
    })
}

pub(crate) async fn upsert_message_participant(
    pool: &PgPool,
    message: &ProjectedMessage,
    person_id: &str,
    participant: &EmailParticipant,
) -> Result<bool, EmailSyncPipelineError> {
    let inserted = CommunicationMessageProjectionPort::new(pool.clone())
        .upsert_email_participant(
            message,
            person_id,
            &participant.email_address,
            participant.display_name.as_deref(),
            participant.role,
        )
        .await?;
    Ok(inserted)
}

fn clean_display_name(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"').trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
```

### `backend/src/workflows/email_sync_pipeline/raw_payload.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/raw_payload.rs`
- Size bytes / Размер в байтах: `2757`
- Included characters / Включено символов: `2757`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use serde_json::{Value, json};

use crate::domains::communications::storage::StoredCommunicationBlob;
use crate::platform::communications::EmailProviderKind;

use super::errors::EmailSyncRecordError;

pub(super) fn raw_message_bytes(
    provider_kind: EmailProviderKind,
    payload: &Value,
) -> Result<Vec<u8>, EmailSyncRecordError> {
    match provider_kind {
        EmailProviderKind::Gmail => {
            let raw = required_payload_string(payload, "raw_base64url")?;
            URL_SAFE_NO_PAD
                .decode(raw)
                .or_else(|_| URL_SAFE.decode(raw))
                .map_err(|source| EmailSyncRecordError::InvalidRawPayloadBase64 {
                    field: "raw_base64url",
                    source,
                })
        }
        EmailProviderKind::Icloud | EmailProviderKind::Imap => {
            let raw = required_payload_string(payload, "raw_rfc822_base64")?;
            BASE64_STANDARD.decode(raw).map_err(|source| {
                EmailSyncRecordError::InvalidRawPayloadBase64 {
                    field: "raw_rfc822_base64",
                    source,
                }
            })
        }
        EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb
        | EmailProviderKind::WhatsappBusinessCloud
        | EmailProviderKind::ZoomUser
        | EmailProviderKind::ZoomServerToServer
        | EmailProviderKind::YandexTelemostUser => Err(
            EmailSyncRecordError::UnsupportedProviderKind(provider_kind.as_str().to_owned()),
        ),
    }
}

pub(super) fn payload_with_raw_blob_reference(
    payload: &Value,
    blob: &StoredCommunicationBlob,
) -> Result<Value, EmailSyncRecordError> {
    let Some(object) = payload.as_object() else {
        return Err(EmailSyncRecordError::InvalidRawPayloadObject);
    };
    let mut object = object.clone();
    object.remove("raw_base64url");
    object.remove("raw_rfc822_base64");
    object.insert("raw_blob_id".to_owned(), json!(blob.blob_id));
    object.insert("raw_blob_sha256".to_owned(), json!(blob.sha256));
    object.insert("raw_blob_storage_kind".to_owned(), json!(blob.storage_kind));
    object.insert("raw_blob_storage_path".to_owned(), json!(blob.storage_path));
    object.insert("raw_blob_size_bytes".to_owned(), json!(blob.size_bytes));

    Ok(Value::Object(object))
}

fn required_payload_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, EmailSyncRecordError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .ok_or(EmailSyncRecordError::MissingRawPayloadField { field })
}
```

### `backend/src/workflows/email_sync_pipeline/raw_records.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/raw_records.rs`
- Size bytes / Размер в байтах: `2498`
- Included characters / Включено символов: `2498`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::communications::core::StoredRawCommunicationRecord;
use crate::domains::communications::ingestion::analyze_ingested_message;
use crate::domains::communications::messages::{
    CommunicationMessageProjectionPort, ProjectedMessage, parse_raw_email_message_from_blob,
    project_accepted_signal_if_runtime_allows,
};
use crate::domains::communications::storage::{
    AttachmentSafetyScanner, CommunicationBlobMetadataPort, LocalCommunicationBlobPort,
};
use crate::domains::signal_hub::dispatch_mail_raw_signal;

use super::attachments::project_attachments;
use super::errors::EmailSyncPipelineError;

#[derive(Default)]
pub(crate) struct RawRecordProjectionReport {
    pub(crate) projected_messages: Vec<ProjectedMessage>,
    pub(crate) attachment_blobs_upserted: usize,
    pub(crate) attachments_extracted: usize,
    pub(crate) attachments_not_scanned: usize,
}

pub(crate) async fn project_raw_records(
    pool: &PgPool,
    mail_store: &CommunicationBlobMetadataPort,
    blob_store: &LocalCommunicationBlobPort,
    raw_records: &[StoredRawCommunicationRecord],
    attachment_scanner: &impl AttachmentSafetyScanner,
) -> Result<RawRecordProjectionReport, EmailSyncPipelineError> {
    let mut report = RawRecordProjectionReport::default();
    let message_store = CommunicationMessageProjectionPort::new(pool.clone());
    for raw_record in raw_records {
        let Some(accepted_event) =
            dispatch_mail_raw_signal(pool.clone(), raw_record, Some(blob_store.root())).await?
        else {
            continue;
        };
        let Some(message) =
            project_accepted_signal_if_runtime_allows(pool.clone(), &accepted_event).await?
        else {
            continue;
        };
        let parsed = parse_raw_email_message_from_blob(blob_store, raw_record).await?;
        let _analysis = analyze_ingested_message(&message_store, &message).await?;
        let attachment_report = project_attachments(
            mail_store,
            blob_store,
            raw_record,
            &message,
            &parsed.attachments,
            attachment_scanner,
        )
        .await?;

        report.attachment_blobs_upserted += attachment_report.attachment_blobs_upserted;
        report.attachments_extracted += attachment_report.attachments_extracted;
        report.attachments_not_scanned += attachment_report.attachments_not_scanned;
        report.projected_messages.push(message);
    }
    Ok(report)
}
```

### `backend/src/workflows/email_sync_pipeline/recording.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/recording.rs`
- Size bytes / Размер в байтах: `5172`
- Included characters / Включено символов: `5172`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::domains::communications::core::{CommunicationIngestionPort, NewIngestionCheckpoint};
use crate::domains::communications::storage::{
    CommunicationBlobMetadataPort, LocalCommunicationBlobPort, NewCommunicationBlob,
};
use crate::platform::communications::{
    EmailSyncBatch, EmailSyncBlobImportReport, EmailSyncImportReport, NewRawCommunicationRecord,
};

use super::errors::EmailSyncRecordError;
use super::ids::{EMAIL_MESSAGE_RECORD_KIND, raw_record_id};
use super::raw_payload::{payload_with_raw_blob_reference, raw_message_bytes};

pub async fn record_email_sync_batch(
    store: &CommunicationIngestionPort,
    account_id: &str,
    import_batch_id: &str,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncImportReport, EmailSyncRecordError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let import_batch_id = validate_non_empty("import_batch_id", import_batch_id)?;
    validate_non_empty("stream_id", &batch.stream_id)?;

    let mut inserted_or_existing_records = 0;
    for message in &batch.messages {
        let mut raw_record = NewRawCommunicationRecord::new(
            raw_record_id(
                &account_id,
                EMAIL_MESSAGE_RECORD_KIND,
                &message.provider_record_id,
            ),
            &account_id,
            EMAIL_MESSAGE_RECORD_KIND,
            &message.provider_record_id,
            &message.source_fingerprint,
            &import_batch_id,
            message.payload.clone(),
        )
        .provenance(json!({
            "source": "email_provider_sync",
            "provider": batch.provider_kind.as_str(),
            "stream_id": batch.stream_id
        }));

        if let Some(occurred_at) = message.occurred_at {
            raw_record = raw_record.occurred_at(occurred_at);
        }

        store.record_raw_source(&raw_record).await?;
        inserted_or_existing_records += 1;
    }

    let checkpoint_saved = save_checkpoint_if_present(store, &account_id, batch).await?;

    Ok(EmailSyncImportReport {
        inserted_or_existing_records,
        checkpoint_saved,
    })
}

pub async fn record_email_sync_batch_with_mail_blobs(
    store: &CommunicationIngestionPort,
    mail_store: &CommunicationBlobMetadataPort,
    blob_store: &LocalCommunicationBlobPort,
    account_id: &str,
    import_batch_id: &str,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncBlobImportReport, EmailSyncRecordError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let import_batch_id = validate_non_empty("import_batch_id", import_batch_id)?;
    validate_non_empty("stream_id", &batch.stream_id)?;

    let mut inserted_or_existing_records = 0;
    let mut blobs_upserted = 0;
    let mut raw_records = Vec::new();
    for message in &batch.messages {
        let raw_bytes = raw_message_bytes(batch.provider_kind, &message.payload)?;
        let local_blob = blob_store.put_blob(&raw_bytes).await?;
        let stored_blob = mail_store
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob).content_type("message/rfc822"),
            )
            .await?;
        let payload = payload_with_raw_blob_reference(&message.payload, &stored_blob)?;

        let mut raw_record = NewRawCommunicationRecord::new(
            raw_record_id(
                &account_id,
                EMAIL_MESSAGE_RECORD_KIND,
                &message.provider_record_id,
            ),
            &account_id,
            EMAIL_MESSAGE_RECORD_KIND,
            &message.provider_record_id,
            &message.source_fingerprint,
            &import_batch_id,
            payload,
        )
        .provenance(json!({
            "source": "email_provider_sync",
            "provider": batch.provider_kind.as_str(),
            "stream_id": batch.stream_id,
            "raw_storage": stored_blob.storage_kind
        }));

        if let Some(occurred_at) = message.occurred_at {
            raw_record = raw_record.occurred_at(occurred_at);
        }

        raw_records.push(store.record_raw_source(&raw_record).await?);
        inserted_or_existing_records += 1;
        blobs_upserted += 1;
    }

    let checkpoint_saved = save_checkpoint_if_present(store, &account_id, batch).await?;

    Ok(EmailSyncBlobImportReport {
        inserted_or_existing_records,
        checkpoint_saved,
        blobs_upserted,
        raw_records,
    })
}

async fn save_checkpoint_if_present(
    store: &CommunicationIngestionPort,
    account_id: &str,
    batch: &EmailSyncBatch,
) -> Result<bool, EmailSyncRecordError> {
    if let Some(checkpoint) = &batch.checkpoint {
        store
            .save_checkpoint(&NewIngestionCheckpoint::new(
                account_id,
                &batch.stream_id,
                checkpoint.clone(),
            ))
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, EmailSyncRecordError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EmailSyncRecordError::EmptyField(field));
    }
    Ok(value.to_owned())
}
```

### `backend/src/workflows/email_sync_pipeline/relationships.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/relationships.rs`
- Size bytes / Размер в байтах: `1087`
- Included characters / Включено символов: `1087`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::persons::memory::RelationshipEventPort;

use super::errors::EmailSyncPipelineError;
use super::participants::EmailParticipant;

pub(crate) async fn insert_relationship_event(
    pool: &PgPool,
    message: &ProjectedMessage,
    person_id: &str,
    participant: &EmailParticipant,
) -> Result<bool, EmailSyncPipelineError> {
    let event_type = if participant.role == "sender" {
        "email_sent"
    } else {
        "email_received"
    };
    let title = if participant.role == "sender" {
        "Sent email"
    } else {
        "Received email"
    };
    let inserted = RelationshipEventPort::new(pool.clone())
        .upsert_email_message_event(
            &message.observation_id,
            &message.message_id,
            message.occurred_at.unwrap_or(message.projected_at),
            person_id,
            event_type,
            title,
            Some(&format!("Email subject: {}", message.subject)),
        )
        .await?;
    Ok(inserted)
}
```

### `backend/src/workflows/email_sync_pipeline/report.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/report.rs`
- Size bytes / Размер в байтах: `760`
- Included characters / Включено символов: `760`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailSyncPipelineReport {
    pub imported_records: usize,
    pub raw_blobs_upserted: usize,
    pub projected_messages: usize,
    pub attachment_blobs_upserted: usize,
    pub attachments_extracted: usize,
    pub attachments_not_scanned: usize,
    pub upserted_persons: usize,
    pub upserted_person_identities: usize,
    pub upserted_message_participants: usize,
    pub upserted_relationship_events: usize,
    pub upserted_organizations: usize,
    pub upserted_organization_contact_links: usize,
    pub refreshed_decision_candidates: usize,
    pub refreshed_knowledge_candidates: usize,
    pub refreshed_task_candidates: usize,
    pub checkpoint_saved: bool,
}
```

### `backend/src/workflows/email_sync_pipeline/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/email_sync_pipeline/service.rs`
- Size bytes / Размер в байтах: `3155`
- Included characters / Включено символов: `3155`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::communications::core::CommunicationIngestionPort;
use crate::domains::communications::storage::{
    CommunicationBlobMetadataPort, HeuristicAttachmentSafetyScanner, LocalCommunicationBlobPort,
};
use crate::domains::persons::api::PersonProjectionPort;
use crate::platform::communications::EmailSyncBatch;

use super::candidates::refresh_message_context_candidates;
use super::errors::EmailSyncPipelineError;
use super::knowledge::project_message_knowledge;
use super::raw_records::project_raw_records;
use super::recording::record_email_sync_batch_with_mail_blobs;
use super::report::EmailSyncPipelineReport;

pub async fn project_email_sync_batch_with_mail_blobs(
    pool: PgPool,
    blob_store: &LocalCommunicationBlobPort,
    account_id: &str,
    import_batch_id: impl AsRef<str>,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncPipelineReport, EmailSyncPipelineError> {
    let communication_store = CommunicationIngestionPort::new(pool.clone());
    let mail_store = CommunicationBlobMetadataPort::new(pool.clone());
    let person_store = PersonProjectionPort::new(pool.clone());
    let attachment_scanner = HeuristicAttachmentSafetyScanner;
    let import_report = record_email_sync_batch_with_mail_blobs(
        &communication_store,
        &mail_store,
        blob_store,
        account_id,
        import_batch_id.as_ref(),
        batch,
    )
    .await?;

    let projection_report = project_raw_records(
        &pool,
        &mail_store,
        blob_store,
        &import_report.raw_records,
        &attachment_scanner,
    )
    .await?;
    let knowledge_report =
        project_message_knowledge(&pool, &person_store, &projection_report.projected_messages)
            .await?;
    let candidate_report =
        refresh_message_context_candidates(&pool, &projection_report.projected_messages).await?;

    Ok(EmailSyncPipelineReport {
        imported_records: import_report.inserted_or_existing_records,
        raw_blobs_upserted: import_report.blobs_upserted,
        projected_messages: projection_report.projected_messages.len(),
        attachment_blobs_upserted: projection_report.attachment_blobs_upserted,
        attachments_extracted: projection_report.attachments_extracted,
        attachments_not_scanned: projection_report.attachments_not_scanned,
        upserted_persons: knowledge_report.upserted_persons,
        upserted_person_identities: knowledge_report.upserted_person_identities,
        upserted_message_participants: knowledge_report.upserted_message_participants,
        upserted_relationship_events: knowledge_report.upserted_relationship_events,
        upserted_organizations: knowledge_report.upserted_organizations,
        upserted_organization_contact_links: knowledge_report.upserted_organization_contact_links,
        refreshed_decision_candidates: candidate_report.refreshed_decision_candidates,
        refreshed_knowledge_candidates: candidate_report.refreshed_knowledge_candidates,
        refreshed_task_candidates: candidate_report.refreshed_task_candidates,
        checkpoint_saved: import_report.checkpoint_saved,
    })
}
```

### `backend/src/workflows/graph_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection.rs`
- Size bytes / Размер в байтах: `296`
- Included characters / Включено символов: `296`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod decisions;
mod documents;
mod errors;
mod evidence;
mod helpers;
mod messages;
mod models;
mod obligations;
mod persons;
mod projects;
mod rows;
mod service;

pub use errors::GraphProjectionError;
pub use models::GraphProjectionReport;
pub use service::GraphProjectionService;
```

### `backend/src/workflows/graph_projection/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/constants.rs`
- Size bytes / Размер в байтах: `57`
- Included characters / Включено символов: `57`
- Truncated / Обрезано: `no`

```rust
pub(super) const PROJECT_KEYWORD_CONFIDENCE: f64 = 0.75;
```

### `backend/src/workflows/graph_projection/decisions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/decisions.rs`
- Size bytes / Размер в байтах: `8974`
- Included characters / Включено символов: `8974`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphProjectionPort, GraphReviewState, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType,
};

use super::errors::GraphProjectionError;
use super::models::GraphProjectionReport;
use super::service::GraphProjectionService;

pub(super) struct DecisionProjectionRow {
    decision_id: String,
    title: String,
    status: String,
    review_state: String,
    confidence: f64,
}

struct DecisionImpactedEntityRow {
    entity_kind: String,
    entity_id: String,
    impact_type: String,
}

struct DecisionEvidenceRow {
    source_kind: String,
    source_id: String,
    observation_id: Option<String>,
    quote: Option<String>,
}

impl GraphProjectionService {
    pub(super) async fn list_decisions(
        &self,
    ) -> Result<Vec<DecisionProjectionRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                decision_id,
                title,
                status,
                review_state,
                confidence::float8 AS confidence
            FROM decisions
            ORDER BY decision_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(DecisionProjectionRow {
                    decision_id: row.try_get("decision_id")?,
                    title: row.try_get("title")?,
                    status: row.try_get("status")?,
                    review_state: row.try_get("review_state")?,
                    confidence: row.try_get("confidence")?,
                })
            })
            .collect()
    }

    pub(super) async fn project_decision(
        &self,
        decision: &DecisionProjectionRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let impacted_entities = self
            .list_decision_impacted_entities(&decision.decision_id)
            .await?;
        let evidence = self.decision_evidence(&decision.decision_id).await?;

        let mut transaction = self.pool.begin().await?;
        let decision_node = GraphProjectionPort::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Decision,
                &decision.decision_id,
                &decision.title,
            )
            .properties(json!({
                "domain": "decision",
                "decision_id": decision.decision_id,
                "status": decision.status,
                "review_state": decision.review_state,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_decision_edges(&mut transaction, &decision.decision_id)
            .await?;

        let graph_review_state = decision_review_state(&decision.review_state)?;
        let graph_evidence = decision_graph_evidence(decision, evidence.as_ref());

        for entity in impacted_entities {
            let target_node = GraphProjectionPort::upsert_node_in_transaction(
                &mut transaction,
                &NewGraphNode::new(
                    entity_graph_node_kind("decision", &entity.entity_kind)?,
                    &entity.entity_id,
                    &entity.entity_id,
                )
                .properties(json!({
                    "domain": "decision",
                    "entity_kind": entity.entity_kind,
                    "entity_id": entity.entity_id,
                })),
            )
            .await?;
            report.nodes_upserted += 1;

            GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
                &mut transaction,
                &NewGraphEdge::new(
                    decision_node.node_id.clone(),
                    target_node.node_id,
                    RelationshipType::EntityRelationship,
                    decision.confidence,
                    graph_review_state,
                )
                .properties(json!({
                    "domain": "decision",
                    "decision_id": decision.decision_id,
                    "impact_type": entity.impact_type,
                })),
                std::slice::from_ref(&graph_evidence),
            )
            .await?;
            report.edges_upserted += 1;
            report.evidence_upserted += 1;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn list_decision_impacted_entities(
        &self,
        decision_id: &str,
    ) -> Result<Vec<DecisionImpactedEntityRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT entity_kind, entity_id, impact_type
            FROM decision_impacted_entities
            WHERE decision_id = $1
            ORDER BY entity_kind, entity_id
            "#,
        )
        .bind(decision_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(DecisionImpactedEntityRow {
                    entity_kind: row.try_get("entity_kind")?,
                    entity_id: row.try_get("entity_id")?,
                    impact_type: row.try_get("impact_type")?,
                })
            })
            .collect()
    }

    async fn decision_evidence(
        &self,
        decision_id: &str,
    ) -> Result<Option<DecisionEvidenceRow>, GraphProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT source_kind, source_id, observation_id, quote
            FROM decision_evidence
            WHERE decision_id = $1
            ORDER BY created_at, evidence_id
            LIMIT 1
            "#,
        )
        .bind(decision_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(DecisionEvidenceRow {
                source_kind: row.try_get("source_kind")?,
                source_id: row.try_get("source_id")?,
                observation_id: row.try_get("observation_id")?,
                quote: row.try_get("quote")?,
            })
        })
        .transpose()
    }

    async fn delete_decision_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        decision_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE edge_id IN (
                SELECT edge.edge_id
                FROM graph_edges edge
                JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
                WHERE evidence.source_kind = 'decision'
                  AND evidence.source_id = $1
            )
            "#,
        )
        .bind(decision_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

fn decision_graph_evidence(
    decision: &DecisionProjectionRow,
    evidence: Option<&DecisionEvidenceRow>,
) -> NewGraphEvidence {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Decision,
        decision.decision_id.clone(),
    )
    .metadata(json!({ "domain": "decision" }));

    if let Some(evidence) = evidence {
        graph_evidence = graph_evidence.metadata(json!({
            "domain": "decision",
            "source_kind": evidence.source_kind,
            "source_id": evidence.source_id,
        }));
        if let Some(quote) = &evidence.quote {
            graph_evidence = graph_evidence.excerpt(quote.clone());
        }
        if let Some(observation_id) = &evidence.observation_id {
            graph_evidence = graph_evidence.observation_id(observation_id.clone());
        }
    }

    graph_evidence
}

fn decision_review_state(value: &str) -> Result<GraphReviewState, GraphProjectionError> {
    match value {
        "suggested" => Ok(GraphReviewState::Suggested),
        "user_confirmed" => Ok(GraphReviewState::UserConfirmed),
        "user_rejected" => Ok(GraphReviewState::UserRejected),
        _ => Err(GraphProjectionError::InvalidReviewState {
            domain: "decision",
            value: value.to_owned(),
        }),
    }
}

pub(super) fn entity_graph_node_kind(
    domain: &'static str,
    value: &str,
) -> Result<GraphNodeKind, GraphProjectionError> {
    match value {
        "persona" => Ok(GraphNodeKind::Person),
        "organization" => Ok(GraphNodeKind::Organization),
        "project" => Ok(GraphNodeKind::Project),
        "communication" => Ok(GraphNodeKind::Message),
        "document" => Ok(GraphNodeKind::Document),
        "task" => Ok(GraphNodeKind::Task),
        "event" => Ok(GraphNodeKind::Event),
        "decision" => Ok(GraphNodeKind::Decision),
        "obligation" => Ok(GraphNodeKind::Obligation),
        "knowledge" => Ok(GraphNodeKind::Knowledge),
        _ => Err(GraphProjectionError::InvalidEntityKind {
            domain,
            value: value.to_owned(),
        }),
    }
}
```

### `backend/src/workflows/graph_projection/documents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/documents.rs`
- Size bytes / Размер в байтах: `1482`
- Included characters / Включено символов: `1482`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::domains::graph::core::{GraphNodeKind, NewGraphNode};

use super::errors::GraphProjectionError;
use super::models::{DocumentRow, GraphProjectionReport};
use super::rows::row_to_document;
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn list_documents(&self) -> Result<Vec<DocumentRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT document_id, document_kind, title, source_fingerprint, observation_id, imported_at
            FROM documents
            ORDER BY document_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_document).collect()
    }

    pub(super) async fn project_document(
        &self,
        document: &DocumentRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        self.graph
            .upsert_node(
                &NewGraphNode::new(
                    GraphNodeKind::Document,
                    &document.document_id,
                    &document.title,
                )
                .properties(json!({
                    "document_kind": document.document_kind,
                    "source_fingerprint": document.source_fingerprint,
                    "imported_at": document.imported_at,
                })),
            )
            .await?;
        report.nodes_upserted += 1;

        Ok(())
    }
}
```

### `backend/src/workflows/graph_projection/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/errors.rs`
- Size bytes / Размер в байтах: `775`
- Included characters / Включено символов: `775`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::graph::core::GraphProjectionPortError;
use crate::domains::projects::core::ProjectCommandPortError;

#[derive(Debug, Error)]
pub enum GraphProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] GraphProjectionPortError),

    #[error(transparent)]
    Project(#[from] ProjectCommandPortError),

    #[error("message recipients must be a JSON array of strings")]
    InvalidRecipients,

    #[error("{domain} graph projection has invalid entity kind: {value}")]
    InvalidEntityKind { domain: &'static str, value: String },

    #[error("{domain} graph projection has invalid review state: {value}")]
    InvalidReviewState { domain: &'static str, value: String },
}
```

### `backend/src/workflows/graph_projection/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/evidence.rs`
- Size bytes / Размер в байтах: `1841`
- Included characters / Включено символов: `1841`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::domains::graph::core::{GraphEvidenceSourceKind, NewGraphEvidence};
use crate::domains::projects::core::{ProjectMatchedDocument, ProjectMatchedMessage};

use super::models::MessageRow;

pub(super) fn message_evidence(message: &MessageRow) -> NewGraphEvidence {
    NewGraphEvidence::new(GraphEvidenceSourceKind::Message, message.message_id.clone())
        .observation_id(message.observation_id.clone())
        .excerpt(message.subject.clone())
        .metadata(json!({
            "raw_record_id": message.raw_record_id,
            "observation_id": message.observation_id,
            "provider_record_id": message.provider_record_id,
        }))
}

pub(super) fn project_message_evidence(message: &ProjectMatchedMessage) -> NewGraphEvidence {
    NewGraphEvidence::new(GraphEvidenceSourceKind::Message, message.message_id.clone())
        .observation_id(message.observation_id.clone())
        .excerpt(message.subject.clone())
        .metadata(json!({
            "raw_record_id": message.raw_record_id,
            "observation_id": message.observation_id,
            "account_id": message.account_id,
            "provider_record_id": message.provider_record_id,
            "occurred_at": message.occurred_at,
            "projected_at": message.projected_at,
            "match_rule": "project_keyword",
        }))
}

pub(super) fn project_document_evidence(document: &ProjectMatchedDocument) -> NewGraphEvidence {
    NewGraphEvidence::new(
        GraphEvidenceSourceKind::Document,
        document.document_id.clone(),
    )
    .excerpt(document.title.clone())
    .metadata(json!({
        "document_kind": document.document_kind,
        "source_fingerprint": document.source_fingerprint,
        "imported_at": document.imported_at,
        "match_rule": "project_keyword",
    }))
}
```

### `backend/src/workflows/graph_projection/helpers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/helpers.rs`
- Size bytes / Размер в байтах: `962`
- Included characters / Включено символов: `962`
- Truncated / Обрезано: `no`

```rust
use crate::domains::graph::core::GraphReviewState;
use crate::domains::projects::link_reviews::ProjectLinkReviewState;

use super::constants::PROJECT_KEYWORD_CONFIDENCE;

pub(super) fn normalize_email_address(email_address: &str) -> String {
    email_address.trim().to_ascii_lowercase()
}

pub(super) fn project_review_graph_state(review_state: ProjectLinkReviewState) -> GraphReviewState {
    match review_state {
        ProjectLinkReviewState::Suggested => GraphReviewState::Suggested,
        ProjectLinkReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        ProjectLinkReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

pub(super) fn project_review_confidence(review_state: ProjectLinkReviewState) -> f64 {
    match review_state {
        ProjectLinkReviewState::Suggested => PROJECT_KEYWORD_CONFIDENCE,
        ProjectLinkReviewState::UserConfirmed => 1.0,
        ProjectLinkReviewState::UserRejected => 0.0,
    }
}
```

### `backend/src/workflows/graph_projection/messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/messages.rs`
- Size bytes / Размер в байтах: `6197`
- Included characters / Включено символов: `6197`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::domains::graph::core::{
    GraphNodeKind, GraphProjectionPort, GraphReviewState, NewGraphEdge, NewGraphNode, node_id,
};

use super::errors::GraphProjectionError;
use super::evidence::message_evidence;
use super::helpers::normalize_email_address;
use super::models::{
    GraphProjectionReport, MessageEndpoint, MessageRow, PersonRow, RelationshipDirection,
};
use super::rows::{row_to_message, row_to_person};
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn list_messages(&self) -> Result<Vec<MessageRow>, GraphProjectionError> {
        let rows = sqlx::query(
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
                occurred_at
            FROM communication_messages
            ORDER BY message_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_message).collect()
    }

    pub(super) async fn project_message(
        &self,
        message: &MessageRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let message_node = GraphProjectionPort::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Message,
                &message.message_id,
                &message.subject,
            )
            .properties(serde_json::json!({
                "account_id": message.account_id,
                "provider_record_id": message.provider_record_id,
                "raw_record_id": message.raw_record_id,
                "observation_id": message.observation_id,
                "occurred_at": message.occurred_at,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_message_edges(&mut transaction, &message.message_id)
            .await?;

        let sender = self
            .resolve_message_endpoint(&mut transaction, &message.sender, report)
            .await?;
        self.project_message_endpoint(
            &mut transaction,
            sender,
            &message_node.node_id,
            message,
            RelationshipDirection::Sent,
            report,
        )
        .await?;

        for recipient in &message.recipients {
            let recipient = self
                .resolve_message_endpoint(&mut transaction, recipient, report)
                .await?;
            self.project_message_endpoint(
                &mut transaction,
                recipient,
                &message_node.node_id,
                message,
                RelationshipDirection::Received,
                report,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn delete_message_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE edge_id IN (
                SELECT edge.edge_id
                FROM graph_edges edge
                JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
                WHERE evidence.source_kind = 'message'
                  AND evidence.source_id = $1
            )
            "#,
        )
        .bind(message_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub(super) async fn resolve_message_endpoint(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        email_address: &str,
        report: &mut GraphProjectionReport,
    ) -> Result<MessageEndpoint, GraphProjectionError> {
        let normalized_email = normalize_email_address(email_address);
        let person = self
            .person_by_normalized_email(transaction, &normalized_email)
            .await?;

        if let Some(person) = person {
            return Ok(MessageEndpoint::Person {
                node_id: node_id(GraphNodeKind::Person, &person.person_id),
            });
        }

        let email = GraphProjectionPort::upsert_node_in_transaction(
            transaction,
            &NewGraphNode::new(
                GraphNodeKind::EmailAddress,
                &normalized_email,
                &normalized_email,
            ),
        )
        .await?;
        report.nodes_upserted += 1;

        Ok(MessageEndpoint::EmailAddress {
            node_id: email.node_id,
        })
    }

    async fn person_by_normalized_email(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        normalized_email: &str,
    ) -> Result<Option<PersonRow>, GraphProjectionError> {
        let row = sqlx::query(
            "SELECT person_id, display_name, email_address FROM persons WHERE email_address = $1",
        )
        .bind(normalized_email)
        .fetch_optional(&mut **transaction)
        .await?;

        row.map(row_to_person).transpose()
    }

    async fn project_message_endpoint(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        endpoint: MessageEndpoint,
        message_node_id: &str,
        message: &MessageRow,
        direction: RelationshipDirection,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let relationship_type = endpoint.relationship_type(direction);
        GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                endpoint.node_id().to_owned(),
                message_node_id.to_owned(),
                relationship_type,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &[message_evidence(message)],
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }
}
```

### `backend/src/workflows/graph_projection/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/models.rs`
- Size bytes / Размер в байтах: `2591`
- Included characters / Включено символов: `2591`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domains::graph::core::RelationshipType;

/// Counts deterministic projection operations attempted during a V1 graph projection run.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct GraphProjectionReport {
    pub nodes_upserted: usize,
    pub edges_upserted: usize,
    pub evidence_upserted: usize,
}

pub(super) struct PersonRow {
    pub(super) person_id: String,
    pub(super) display_name: String,
    pub(super) email_address: String,
}

pub(super) struct MessageRow {
    pub(super) message_id: String,
    pub(super) raw_record_id: String,
    pub(super) observation_id: String,
    pub(super) account_id: String,
    pub(super) provider_record_id: String,
    pub(super) subject: String,
    pub(super) sender: String,
    pub(super) recipients: Vec<String>,
    pub(super) body_text: String,
    pub(super) occurred_at: Option<DateTime<Utc>>,
}

pub(super) struct DocumentRow {
    pub(super) document_id: String,
    pub(super) document_kind: String,
    pub(super) title: String,
    pub(super) source_fingerprint: String,
    pub(super) observation_id: String,
    pub(super) imported_at: DateTime<Utc>,
}

pub(super) enum MessageEndpoint {
    Person { node_id: String },
    EmailAddress { node_id: String },
}

impl MessageEndpoint {
    pub(super) fn node_id(&self) -> &str {
        match self {
            Self::Person { node_id } | Self::EmailAddress { node_id } => node_id,
        }
    }

    pub(super) fn relationship_type(&self, direction: RelationshipDirection) -> RelationshipType {
        match (self, direction) {
            (Self::Person { .. }, RelationshipDirection::Sent) => {
                RelationshipType::PersonSentMessage
            }
            (Self::Person { .. }, RelationshipDirection::Received) => {
                RelationshipType::PersonReceivedMessage
            }
            (Self::EmailAddress { .. }, RelationshipDirection::Sent) => {
                RelationshipType::EmailAddressSentMessage
            }
            (Self::EmailAddress { .. }, RelationshipDirection::Received) => {
                RelationshipType::EmailAddressReceivedMessage
            }
        }
    }

    pub(super) fn project_relationship_type(&self) -> RelationshipType {
        match self {
            Self::Person { .. } => RelationshipType::ProjectInvolvesPerson,
            Self::EmailAddress { .. } => RelationshipType::ProjectInvolvesEmailAddress,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum RelationshipDirection {
    Sent,
    Received,
}
```

### `backend/src/workflows/graph_projection/obligations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/obligations.rs`
- Size bytes / Размер в байтах: `9113`
- Included characters / Включено символов: `9113`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphProjectionPort, GraphReviewState, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType,
};

use super::decisions::entity_graph_node_kind;
use super::errors::GraphProjectionError;
use super::models::GraphProjectionReport;
use super::service::GraphProjectionService;

pub(super) struct ObligationProjectionRow {
    obligation_id: String,
    obligated_entity_kind: String,
    obligated_entity_id: String,
    beneficiary_entity_kind: Option<String>,
    beneficiary_entity_id: Option<String>,
    statement: String,
    status: String,
    review_state: String,
    confidence: f64,
}

struct ObligationEvidenceRow {
    source_kind: String,
    source_id: String,
    observation_id: Option<String>,
    quote: Option<String>,
}

impl GraphProjectionService {
    pub(super) async fn list_obligations(
        &self,
    ) -> Result<Vec<ObligationProjectionRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                confidence::float8 AS confidence
            FROM obligations
            ORDER BY obligation_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ObligationProjectionRow {
                    obligation_id: row.try_get("obligation_id")?,
                    obligated_entity_kind: row.try_get("obligated_entity_kind")?,
                    obligated_entity_id: row.try_get("obligated_entity_id")?,
                    beneficiary_entity_kind: row.try_get("beneficiary_entity_kind")?,
                    beneficiary_entity_id: row.try_get("beneficiary_entity_id")?,
                    statement: row.try_get("statement")?,
                    status: row.try_get("status")?,
                    review_state: row.try_get("review_state")?,
                    confidence: row.try_get("confidence")?,
                })
            })
            .collect()
    }

    pub(super) async fn project_obligation(
        &self,
        obligation: &ObligationProjectionRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let evidence = self.obligation_evidence(&obligation.obligation_id).await?;

        let mut transaction = self.pool.begin().await?;
        let obligation_node = GraphProjectionPort::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Obligation,
                &obligation.obligation_id,
                &obligation.statement,
            )
            .properties(json!({
                "domain": "obligation",
                "obligation_id": obligation.obligation_id,
                "status": obligation.status,
                "review_state": obligation.review_state,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_obligation_edges(&mut transaction, &obligation.obligation_id)
            .await?;

        let graph_review_state = obligation_review_state(&obligation.review_state)?;
        let graph_evidence = obligation_graph_evidence(obligation, evidence.as_ref());

        self.project_obligation_entity_edge(
            &mut transaction,
            &obligation_node.node_id,
            &obligation.obligated_entity_kind,
            &obligation.obligated_entity_id,
            "obligated_entity",
            obligation.confidence,
            graph_review_state,
            &graph_evidence,
            report,
        )
        .await?;

        if let (Some(beneficiary_entity_kind), Some(beneficiary_entity_id)) = (
            obligation.beneficiary_entity_kind.as_deref(),
            obligation.beneficiary_entity_id.as_deref(),
        ) {
            self.project_obligation_entity_edge(
                &mut transaction,
                &obligation_node.node_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                "beneficiary_entity",
                obligation.confidence,
                graph_review_state,
                &graph_evidence,
                report,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn project_obligation_entity_edge(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        obligation_node_id: &str,
        entity_kind: &str,
        entity_id: &str,
        link_role: &str,
        confidence: f64,
        review_state: GraphReviewState,
        graph_evidence: &NewGraphEvidence,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let target_node = GraphProjectionPort::upsert_node_in_transaction(
            transaction,
            &NewGraphNode::new(
                entity_graph_node_kind("obligation", entity_kind)?,
                entity_id,
                entity_id,
            )
            .properties(json!({
                "domain": "obligation",
                "entity_kind": entity_kind,
                "entity_id": entity_id,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                obligation_node_id.to_owned(),
                target_node.node_id,
                RelationshipType::EntityRelationship,
                confidence,
                review_state,
            )
            .properties(json!({
                "domain": "obligation",
                "link_role": link_role,
            })),
            std::slice::from_ref(graph_evidence),
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn obligation_evidence(
        &self,
        obligation_id: &str,
    ) -> Result<Option<ObligationEvidenceRow>, GraphProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT source_kind, source_id, observation_id, quote
            FROM obligation_evidence
            WHERE obligation_id = $1
            ORDER BY created_at, evidence_id
            LIMIT 1
            "#,
        )
        .bind(obligation_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(ObligationEvidenceRow {
                source_kind: row.try_get("source_kind")?,
                source_id: row.try_get("source_id")?,
                observation_id: row.try_get("observation_id")?,
                quote: row.try_get("quote")?,
            })
        })
        .transpose()
    }

    async fn delete_obligation_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        obligation_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE edge_id IN (
                SELECT edge.edge_id
                FROM graph_edges edge
                JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
                WHERE evidence.source_kind = 'obligation'
                  AND evidence.source_id = $1
            )
            "#,
        )
        .bind(obligation_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

fn obligation_graph_evidence(
    obligation: &ObligationProjectionRow,
    evidence: Option<&ObligationEvidenceRow>,
) -> NewGraphEvidence {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Obligation,
        obligation.obligation_id.clone(),
    )
    .metadata(json!({ "domain": "obligation" }));

    if let Some(evidence) = evidence {
        graph_evidence = graph_evidence.metadata(json!({
            "domain": "obligation",
            "source_kind": evidence.source_kind,
            "source_id": evidence.source_id,
        }));
        if let Some(quote) = &evidence.quote {
            graph_evidence = graph_evidence.excerpt(quote.clone());
        }
        if let Some(observation_id) = &evidence.observation_id {
            graph_evidence = graph_evidence.observation_id(observation_id.clone());
        }
    }

    graph_evidence
}

fn obligation_review_state(value: &str) -> Result<GraphReviewState, GraphProjectionError> {
    match value {
        "suggested" => Ok(GraphReviewState::Suggested),
        "user_confirmed" => Ok(GraphReviewState::UserConfirmed),
        "user_rejected" => Ok(GraphReviewState::UserRejected),
        _ => Err(GraphProjectionError::InvalidReviewState {
            domain: "obligation",
            value: value.to_owned(),
        }),
    }
}
```
