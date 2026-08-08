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

- Chunk ID / ID чанка: `075-source-backend-part-055`
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

### `backend/src/workflows/graph_projection/persons.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/persons.rs`
- Size bytes / Размер в байтах: `2338`
- Included characters / Включено символов: `2338`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, NewGraphEdge, NewGraphEvidence,
    NewGraphNode, RelationshipType,
};

use super::errors::GraphProjectionError;
use super::helpers::normalize_email_address;
use super::models::{GraphProjectionReport, PersonRow};
use super::rows::row_to_person;
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn list_persons(&self) -> Result<Vec<PersonRow>, GraphProjectionError> {
        let rows = sqlx::query(
            "SELECT person_id, display_name, email_address FROM persons ORDER BY person_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_person).collect()
    }

    pub(super) async fn project_person(
        &self,
        person: &PersonRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let normalized_email = normalize_email_address(&person.email_address);
        let person_node = self
            .graph
            .upsert_node(
                &NewGraphNode::new(
                    GraphNodeKind::Person,
                    &person.person_id,
                    &person.display_name,
                )
                .properties(json!({ "email_address": normalized_email.clone() })),
            )
            .await?;
        report.nodes_upserted += 1;

        let email = self
            .graph
            .upsert_node(&NewGraphNode::new(
                GraphNodeKind::EmailAddress,
                &normalized_email,
                &normalized_email,
            ))
            .await?;
        report.nodes_upserted += 1;

        self.graph
            .upsert_edge_with_evidence(
                &NewGraphEdge::new(
                    person_node.node_id,
                    email.node_id,
                    RelationshipType::PersonHasEmailAddress,
                    1.0,
                    GraphReviewState::SystemAccepted,
                ),
                &[NewGraphEvidence::new(
                    GraphEvidenceSourceKind::Person,
                    person.person_id.clone(),
                )],
            )
            .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }
}
```

### `backend/src/workflows/graph_projection/projects.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/projects.rs`
- Size bytes / Размер в байтах: `6787`
- Included characters / Включено символов: `6787`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeSet;

use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::domains::graph::core::{
    GraphNodeKind, GraphProjectionPort, NewGraphEdge, NewGraphNode, RelationshipType, node_id,
};
use crate::domains::projects::core::{
    ProjectMatchedDocument, ProjectMatchedMessage, ProjectProjectionSource,
};

use super::errors::GraphProjectionError;
use super::evidence::{project_document_evidence, project_message_evidence};
use super::helpers::{
    normalize_email_address, project_review_confidence, project_review_graph_state,
};
use super::models::GraphProjectionReport;
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn project_project(
        &self,
        project: &ProjectProjectionSource,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let messages = self
            .projects
            .matching_project_messages(&project.project.project_id)
            .await?;
        let documents = self
            .projects
            .matching_project_documents(&project.project.project_id)
            .await?;

        let mut transaction = self.pool.begin().await?;
        let project_node = GraphProjectionPort::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Project,
                &project.project.project_id,
                &project.project.name,
            )
            .properties(json!({
                "kind": project.project.kind,
                "status": project.project.status,
                "description": project.project.description,
                "owner_display_name": project.project.owner_display_name,
                "progress_percent": project.project.progress_percent,
                "start_date": project.project.start_date,
                "target_date": project.project.target_date,
                "keywords": project.keywords,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_project_edges(&mut transaction, &project_node.node_id)
            .await?;

        for message in &messages {
            self.project_project_message(&mut transaction, &project_node.node_id, message, report)
                .await?;
            self.project_project_people(&mut transaction, &project_node.node_id, message, report)
                .await?;
        }

        for document in &documents {
            self.project_project_document(
                &mut transaction,
                &project_node.node_id,
                document,
                report,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn delete_project_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE source_node_id = $1
              AND relationship_type IN (
                  'project_has_message',
                  'project_has_document',
                  'project_involves_person',
                  'project_involves_email_address'
              )
            "#,
        )
        .bind(project_node_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    async fn project_project_message(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        message: &ProjectMatchedMessage,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                project_node_id.to_owned(),
                node_id(GraphNodeKind::Message, &message.message_id),
                RelationshipType::ProjectHasMessage,
                project_review_confidence(message.review_state),
                project_review_graph_state(message.review_state),
            )
            .properties(json!({ "match_rule": "project_keyword" })),
            &[project_message_evidence(message)],
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn project_project_document(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        document: &ProjectMatchedDocument,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                project_node_id.to_owned(),
                node_id(GraphNodeKind::Document, &document.document_id),
                RelationshipType::ProjectHasDocument,
                project_review_confidence(document.review_state),
                project_review_graph_state(document.review_state),
            )
            .properties(json!({ "match_rule": "project_keyword" })),
            &[project_document_evidence(document)],
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn project_project_people(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        message: &ProjectMatchedMessage,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let mut participant_emails = BTreeSet::new();
        participant_emails.insert(normalize_email_address(&message.sender));
        for recipient in &message.recipients {
            participant_emails.insert(normalize_email_address(recipient));
        }

        for participant_email in participant_emails {
            let endpoint = self
                .resolve_message_endpoint(transaction, &participant_email, report)
                .await?;
            GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
                transaction,
                &NewGraphEdge::new(
                    project_node_id.to_owned(),
                    endpoint.node_id().to_owned(),
                    endpoint.project_relationship_type(),
                    project_review_confidence(message.review_state),
                    project_review_graph_state(message.review_state),
                )
                .properties(json!({ "match_rule": "project_keyword" })),
                &[project_message_evidence(message)],
            )
            .await?;
            report.edges_upserted += 1;
            report.evidence_upserted += 1;
        }

        Ok(())
    }
}
```

### `backend/src/workflows/graph_projection/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/rows.rs`
- Size bytes / Размер в байтах: `1932`
- Included characters / Включено символов: `1932`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::GraphProjectionError;
use super::models::{DocumentRow, MessageRow, PersonRow};

pub(super) fn row_to_person(row: PgRow) -> Result<PersonRow, GraphProjectionError> {
    Ok(PersonRow {
        person_id: row.try_get("person_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
    })
}

pub(super) fn row_to_message(row: PgRow) -> Result<MessageRow, GraphProjectionError> {
    Ok(MessageRow {
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
    })
}

pub(super) fn row_to_document(row: PgRow) -> Result<DocumentRow, GraphProjectionError> {
    Ok(DocumentRow {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        observation_id: row.try_get("observation_id")?,
        imported_at: row.try_get("imported_at")?,
    })
}

fn recipients_from_value(value: Value) -> Result<Vec<String>, GraphProjectionError> {
    let Some(values) = value.as_array() else {
        return Err(GraphProjectionError::InvalidRecipients);
    };

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(GraphProjectionError::InvalidRecipients)
        })
        .collect()
}
```

### `backend/src/workflows/graph_projection/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/graph_projection/service.rs`
- Size bytes / Размер в байтах: `1620`
- Included characters / Включено символов: `1620`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::graph::core::GraphProjectionPort;
use crate::domains::projects::core::ProjectCommandPort;

use super::errors::GraphProjectionError;
use super::models::GraphProjectionReport;

#[derive(Clone)]
pub struct GraphProjectionService {
    pub(super) pool: PgPool,
    pub(super) graph: GraphProjectionPort,
    pub(super) projects: ProjectCommandPort,
}

impl GraphProjectionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            graph: GraphProjectionPort::new(pool.clone()),
            projects: ProjectCommandPort::new(pool.clone()),
            pool,
        }
    }

    pub async fn project_from_v1(&self) -> Result<GraphProjectionReport, GraphProjectionError> {
        let mut report = GraphProjectionReport::default();

        for person in self.list_persons().await? {
            self.project_person(&person, &mut report).await?;
        }
        for message in self.list_messages().await? {
            self.project_message(&message, &mut report).await?;
        }
        for document in self.list_documents().await? {
            self.project_document(&document, &mut report).await?;
        }
        for project in self.projects.graph_projection_projects().await? {
            self.project_project(&project, &mut report).await?;
        }
        for decision in self.list_decisions().await? {
            self.project_decision(&decision, &mut report).await?;
        }
        for obligation in self.list_obligations().await? {
            self.project_obligation(&obligation, &mut report).await?;
        }

        Ok(report)
    }
}
```

### `backend/src/workflows/mail_background_sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync.rs`
- Size bytes / Размер в байтах: `808`
- Included characters / Включено символов: `808`
- Truncated / Обрезано: `no`

```rust
pub const DEFAULT_MAIL_SYNC_BATCH_SIZE: i32 = 100;
pub const DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS: i32 = 300;
pub use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;

const MAX_BATCH_SIZE: i32 = 500;
const MIN_POLL_INTERVAL_SECONDS: i32 = 60;
const MAX_POLL_INTERVAL_SECONDS: i32 = 86_400;
pub const DEFAULT_GMAIL_API_BASE_URL: &str = "https://www.googleapis.com";

mod errors;
mod events;
mod evidence;
mod models;
mod provider;
mod rows;
mod service;
mod store;
mod validation;

pub use self::errors::MailSyncError;
pub use self::models::{
    MailSyncDueAccount, MailSyncFailureReason, MailSyncRun, MailSyncRunResponse, MailSyncSettings,
    MailSyncSettingsUpdate, MailSyncStatus, MailSyncTrigger,
};
pub use self::service::MailBackgroundSyncService;
pub use self::store::MailSyncStore;
```

### `backend/src/workflows/mail_background_sync/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/errors.rs`
- Size bytes / Размер в байтах: `1620`
- Included characters / Включено символов: `1620`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::communications::core::CommunicationIngestionError;
use crate::platform::communications::EmailProviderSyncError;
use crate::platform::events::{EventEnvelopeError, EventLogPortError};
use crate::platform::observations::ObservationPortError;
use crate::workflows::email_sync_pipeline::EmailSyncPipelineError;
use crate::workflows::graph_projection::GraphProjectionError;

#[derive(Debug, Error)]
pub enum MailSyncError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventLogPort(#[from] EventLogPortError),

    #[error(transparent)]
    ObservationPort(#[from] ObservationPortError),

    #[error("mail sync account was not found")]
    AccountNotFound,

    #[error("mail sync run is already active for account")]
    RunAlreadyActive,

    #[error("mail sync run was not found")]
    RunNotFound,

    #[error("invalid mail sync setting {field}: {message}")]
    InvalidSetting {
        field: &'static str,
        message: &'static str,
    },
}

#[derive(Debug, Error)]
pub(super) enum ProviderSyncError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    ProviderSync(#[from] EmailProviderSyncError),

    #[error(transparent)]
    Pipeline(#[from] EmailSyncPipelineError),

    #[error(transparent)]
    Graph(#[from] GraphProjectionError),

    #[error(transparent)]
    SyncState(#[from] MailSyncError),
}
```

### `backend/src/workflows/mail_background_sync/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/events.rs`
- Size bytes / Размер в байтах: `2810`
- Included characters / Включено символов: `2810`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use super::errors::MailSyncError;
use super::models::MailSyncRun;
use crate::platform::events::NewEventEnvelope;

const EVENT_TYPE_STARTED: &str = "mail.sync.started";
const EVENT_TYPE_PROGRESS: &str = "mail.sync.progress";
const EVENT_TYPE_COMPLETED: &str = "mail.sync.completed";
const EVENT_TYPE_FAILED: &str = "mail.sync.failed";
const EVENT_TYPE_SKIPPED: &str = "mail.sync.skipped";

pub(super) fn sync_run_started_event(run: &MailSyncRun) -> Result<NewEventEnvelope, MailSyncError> {
    sync_run_event(EVENT_TYPE_STARTED, run)
}

pub(super) fn sync_run_progress_event(
    run: &MailSyncRun,
) -> Result<NewEventEnvelope, MailSyncError> {
    sync_run_event(EVENT_TYPE_PROGRESS, run)
}

pub(super) fn sync_run_finished_event(
    run: &MailSyncRun,
) -> Result<NewEventEnvelope, MailSyncError> {
    let event_type = match run.status.as_str() {
        "completed" => EVENT_TYPE_COMPLETED,
        "failed" => EVENT_TYPE_FAILED,
        "skipped" => EVENT_TYPE_SKIPPED,
        _ => EVENT_TYPE_PROGRESS,
    };
    sync_run_event(event_type, run)
}

fn sync_run_event(event_type: &str, run: &MailSyncRun) -> Result<NewEventEnvelope, MailSyncError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_sync_event:{event_type}:{}:{:x}",
            run.run_id,
            system_time_nanos()
        ),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_background_sync" }),
        json!({
            "kind": "mail_sync_run",
            "id": run.run_id,
            "run_id": run.run_id,
            "account_id": run.account_id,
        }),
    )
    .payload(json!({
        "run_id": run.run_id,
        "account_id": run.account_id,
        "trigger": run.trigger,
        "status": run.status,
        "phase": run.phase,
        "progress_mode": run.progress_mode,
        "progress_percent": run.progress_percent,
        "processed_messages": run.processed_messages,
        "estimated_total_messages": run.estimated_total_messages,
        "current_batch_size": run.current_batch_size,
        "fetched_messages": run.fetched_messages,
        "projected_messages": run.projected_messages,
        "upserted_persons": run.upserted_persons,
        "upserted_organizations": run.upserted_organizations,
        "checkpoint_saved": run.checkpoint_saved,
        "error_code": run.error_code,
        "next_run_at": run.next_run_at,
    }))
    .provenance(json!({
        "source_kind": "mail_sync_run",
        "source_id": run.run_id,
    }))
    .correlation_id(run.account_id.clone())
    .build()?)
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
```

### `backend/src/workflows/mail_background_sync/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/evidence.rs`
- Size bytes / Размер в байтах: `2683`
- Included characters / Включено символов: `2683`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationPort};

use super::errors::MailSyncError;
use super::models::MailSyncRun;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;

pub(super) async fn capture_mail_sync_run_observation(
    transaction: &mut Transaction<'_, Postgres>,
    run: &MailSyncRun,
    kind_code: &str,
    relationship_kind: &str,
    observed_at: DateTime<Utc>,
    actor: &str,
) -> Result<(), MailSyncError> {
    let observation = ObservationPort::capture_in_transaction(
        transaction,
        &NewObservation::new(
            kind_code,
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "run_id": run.run_id,
                "account_id": run.account_id,
                "trigger": run.trigger,
                "status": run.status,
                "phase": run.phase,
                "progress_mode": run.progress_mode,
                "progress_percent": run.progress_percent,
                "processed_messages": run.processed_messages,
                "estimated_total_messages": run.estimated_total_messages,
                "current_batch_size": run.current_batch_size,
                "fetched_messages": run.fetched_messages,
                "projected_messages": run.projected_messages,
                "upserted_persons": run.upserted_persons,
                "upserted_organizations": run.upserted_organizations,
                "checkpoint_before": run.checkpoint_before,
                "checkpoint_after": run.checkpoint_after,
                "checkpoint_saved": run.checkpoint_saved,
                "error_code": run.error_code,
                "error_message": run.error_message,
                "started_at": run.started_at,
                "completed_at": run.completed_at,
                "next_run_at": run.next_run_at,
                "operation": relationship_kind,
            }),
            format!("mail-sync-run://{}/{}", run.run_id, relationship_kind),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "mail_sync_run",
        run.run_id.clone(),
        relationship_kind,
        json!({
            "account_id": run.account_id,
            "status": run.status,
            "phase": run.phase,
            "progress_mode": run.progress_mode,
        }),
        None,
    )
    .await?;
    Ok(())
}
```

### `backend/src/workflows/mail_background_sync/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/models.rs`
- Size bytes / Размер в байтах: `587`
- Included characters / Включено символов: `587`
- Truncated / Обрезано: `no`

```rust
mod failures;
mod finish;
mod progress;
mod runs;
mod settings;
mod status;

pub use progress::MailSyncTrigger;
pub use runs::{MailSyncFailureReason, MailSyncRun, MailSyncRunResponse};
pub use settings::{MailSyncDueAccount, MailSyncSettings, MailSyncSettingsUpdate};
pub use status::MailSyncStatus;

pub(in crate::workflows::mail_background_sync) use failures::SanitizedSyncFailure;
pub(in crate::workflows::mail_background_sync) use finish::FinishRun;
pub(in crate::workflows::mail_background_sync) use progress::{
    MailSyncPhase, MailSyncRunStatus, ProgressMode, ProgressUpdate,
};
```

### `backend/src/workflows/mail_background_sync/models/failures.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/models/failures.rs`
- Size bytes / Размер в байтах: `4018`
- Included characters / Включено символов: `4018`
- Truncated / Обрезано: `no`

```rust
use crate::platform::communications::{EmailProviderSyncErrorKind, EmailSyncPlanError};
use crate::vault::HostVaultError;

use super::super::errors::ProviderSyncError;

#[derive(Debug)]
pub(in crate::workflows::mail_background_sync) struct SanitizedSyncFailure {
    pub(in crate::workflows::mail_background_sync) code: String,
    pub(in crate::workflows::mail_background_sync) message: String,
}

impl SanitizedSyncFailure {
    pub(in crate::workflows::mail_background_sync) fn from_plan(error: EmailSyncPlanError) -> Self {
        tracing::warn!(error = %error, "mail sync provider configuration is invalid");
        Self {
            code: "provider_config_invalid".to_owned(),
            message: "Mail provider configuration is invalid".to_owned(),
        }
    }

    pub(in crate::workflows::mail_background_sync) fn from_vault(error: HostVaultError) -> Self {
        match error {
            HostVaultError::Locked => Self {
                code: "vault_locked".to_owned(),
                message: "Host vault is locked".to_owned(),
            },
            HostVaultError::Uninitialized => Self {
                code: "vault_uninitialized".to_owned(),
                message: "Host vault is not initialized".to_owned(),
            },
            other => {
                tracing::warn!(error = %other, "mail sync vault check failed");
                Self {
                    code: "vault_unavailable".to_owned(),
                    message: "Host vault is unavailable".to_owned(),
                }
            }
        }
    }
}

impl From<ProviderSyncError> for SanitizedSyncFailure {
    fn from(error: ProviderSyncError) -> Self {
        match error {
            ProviderSyncError::ProviderSync(error) => match error.kind {
                EmailProviderSyncErrorKind::MissingCredential
                | EmailProviderSyncErrorKind::Credential => Self {
                    code: "credential_unavailable".to_owned(),
                    message: "Provider credential is unavailable for this account".to_owned(),
                },
                EmailProviderSyncErrorKind::AccountSetup => Self {
                    code: "oauth_refresh_failed".to_owned(),
                    message: "OAuth access token refresh failed".to_owned(),
                },
                EmailProviderSyncErrorKind::ProviderNetwork => {
                    tracing::warn!(error = %error, "mail provider sync network call failed");
                    Self {
                        code: "provider_network_error".to_owned(),
                        message: "Mail provider network request failed".to_owned(),
                    }
                }
            },
            ProviderSyncError::Pipeline(error) => {
                tracing::error!(error = %error, "mail sync projection pipeline failed");
                Self {
                    code: "projection_failed".to_owned(),
                    message: "Mail sync projection failed".to_owned(),
                }
            }
            ProviderSyncError::Graph(error) => {
                tracing::error!(error = %error, "mail sync graph projection failed");
                Self {
                    code: "graph_projection_failed".to_owned(),
                    message: "Mail graph projection failed".to_owned(),
                }
            }
            ProviderSyncError::Communication(error) => {
                tracing::error!(error = %error, "mail sync communication store failed");
                Self {
                    code: "communication_store_error".to_owned(),
                    message: "Mail sync communication store failed".to_owned(),
                }
            }
            ProviderSyncError::SyncState(error) => {
                tracing::error!(error = %error, "mail sync status store failed");
                Self {
                    code: "sync_store_error".to_owned(),
                    message: "Mail sync status store failed".to_owned(),
                }
            }
        }
    }
}
```

### `backend/src/workflows/mail_background_sync/models/finish.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/models/finish.rs`
- Size bytes / Размер в байтах: `2352`
- Included characters / Включено символов: `2352`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;

use super::failures::SanitizedSyncFailure;
use super::progress::{MailSyncPhase, MailSyncRunStatus, ProgressMode};
use super::settings::MailSyncSettings;
use crate::workflows::mail_background_sync::validation::next_run_at;

pub(in crate::workflows::mail_background_sync) struct FinishRun {
    pub(in crate::workflows::mail_background_sync) status: MailSyncRunStatus,
    pub(in crate::workflows::mail_background_sync) phase: MailSyncPhase,
    pub(in crate::workflows::mail_background_sync) progress_mode: ProgressMode,
    pub(in crate::workflows::mail_background_sync) progress_percent: Option<i32>,
    pub(in crate::workflows::mail_background_sync) processed_messages: i64,
    pub(in crate::workflows::mail_background_sync) estimated_total_messages: Option<i64>,
    pub(in crate::workflows::mail_background_sync) fetched_messages: i64,
    pub(in crate::workflows::mail_background_sync) projected_messages: i64,
    pub(in crate::workflows::mail_background_sync) upserted_persons: i64,
    pub(in crate::workflows::mail_background_sync) upserted_organizations: i64,
    pub(in crate::workflows::mail_background_sync) checkpoint_after: Option<Value>,
    pub(in crate::workflows::mail_background_sync) checkpoint_saved: bool,
    pub(in crate::workflows::mail_background_sync) error_code: Option<String>,
    pub(in crate::workflows::mail_background_sync) error_message: Option<String>,
    pub(in crate::workflows::mail_background_sync) next_run_at: Option<DateTime<Utc>>,
}

impl FinishRun {
    pub(in crate::workflows::mail_background_sync) fn failed(
        phase: MailSyncPhase,
        failure: SanitizedSyncFailure,
        settings: &MailSyncSettings,
    ) -> Self {
        Self {
            status: MailSyncRunStatus::Failed,
            phase,
            progress_mode: ProgressMode::None,
            progress_percent: None,
            processed_messages: 0,
            estimated_total_messages: None,
            fetched_messages: 0,
            projected_messages: 0,
            upserted_persons: 0,
            upserted_organizations: 0,
            checkpoint_after: None,
            checkpoint_saved: false,
            error_code: Some(failure.code),
            error_message: Some(failure.message),
            next_run_at: next_run_at(settings),
        }
    }
}
```

### `backend/src/workflows/mail_background_sync/models/progress.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/models/progress.rs`
- Size bytes / Размер в байтах: `2371`
- Included characters / Включено символов: `2371`
- Truncated / Обрезано: `no`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncTrigger {
    Scheduled,
    Manual,
}

impl MailSyncTrigger {
    pub(in crate::workflows::mail_background_sync) fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
        }
    }
}

pub(in crate::workflows::mail_background_sync) struct ProgressUpdate<'a> {
    pub(in crate::workflows::mail_background_sync) run_id: &'a str,
    pub(in crate::workflows::mail_background_sync) phase: MailSyncPhase,
    pub(in crate::workflows::mail_background_sync) progress_mode: ProgressMode,
    pub(in crate::workflows::mail_background_sync) progress_percent: Option<i32>,
    pub(in crate::workflows::mail_background_sync) processed_messages: i64,
    pub(in crate::workflows::mail_background_sync) estimated_total_messages: Option<i64>,
    pub(in crate::workflows::mail_background_sync) current_batch_size: i32,
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync) enum MailSyncRunStatus {
    Completed,
    Failed,
    Skipped,
}

impl MailSyncRunStatus {
    pub(in crate::workflows::mail_background_sync) fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync) enum MailSyncPhase {
    Listing,
    Fetching,
    Projecting,
    PersonsGraph,
    Completed,
    Failed,
}

impl MailSyncPhase {
    pub(in crate::workflows::mail_background_sync) fn as_str(self) -> &'static str {
        match self {
            Self::Listing => "listing",
            Self::Fetching => "fetching",
            Self::Projecting => "projecting",
            Self::PersonsGraph => "persons_graph",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync) enum ProgressMode {
    None,
    Determinate,
    Indeterminate,
}

impl ProgressMode {
    pub(in crate::workflows::mail_background_sync) fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Determinate => "determinate",
            Self::Indeterminate => "indeterminate",
        }
    }
}
```

### `backend/src/workflows/mail_background_sync/models/runs.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/models/runs.rs`
- Size bytes / Размер в байтах: `3421`
- Included characters / Включено символов: `3421`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncRun {
    pub run_id: String,
    pub account_id: String,
    pub trigger: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub fetched_messages: i64,
    pub projected_messages: i64,
    pub upserted_persons: i64,
    pub upserted_organizations: i64,
    pub checkpoint_before: Option<Value>,
    pub checkpoint_after: Option<Value>,
    pub checkpoint_saved: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncRunResponse {
    pub run_id: String,
    pub account_id: String,
    pub trigger: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub fetched_messages: i64,
    pub projected_messages: i64,
    pub upserted_persons: i64,
    pub upserted_organizations: i64,
    pub checkpoint_before_present: bool,
    pub checkpoint_after_present: bool,
    pub checkpoint_saved: bool,
    pub failure_reason: Option<MailSyncFailureReason>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
}

impl From<MailSyncRun> for MailSyncRunResponse {
    fn from(run: MailSyncRun) -> Self {
        Self {
            run_id: run.run_id,
            account_id: run.account_id,
            trigger: run.trigger,
            status: run.status,
            phase: run.phase,
            progress_mode: run.progress_mode,
            progress_percent: run.progress_percent,
            processed_messages: run.processed_messages,
            estimated_total_messages: run.estimated_total_messages,
            current_batch_size: run.current_batch_size,
            fetched_messages: run.fetched_messages,
            projected_messages: run.projected_messages,
            upserted_persons: run.upserted_persons,
            upserted_organizations: run.upserted_organizations,
            checkpoint_before_present: checkpoint_is_present(run.checkpoint_before.as_ref()),
            checkpoint_after_present: checkpoint_is_present(run.checkpoint_after.as_ref()),
            checkpoint_saved: run.checkpoint_saved,
            failure_reason: run.error_code.map(|code| MailSyncFailureReason {
                code,
                message: run
                    .error_message
                    .unwrap_or_else(|| "Mail sync failed".to_owned()),
            }),
            started_at: run.started_at,
            completed_at: run.completed_at,
            next_run_at: run.next_run_at,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncFailureReason {
    pub code: String,
    pub message: String,
}

fn checkpoint_is_present(checkpoint: Option<&Value>) -> bool {
    checkpoint
        .and_then(Value::as_object)
        .is_some_and(|object| !object.is_empty())
}
```

### `backend/src/workflows/mail_background_sync/models/settings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/models/settings.rs`
- Size bytes / Размер в байтах: `650`
- Included characters / Включено символов: `650`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncSettings {
    pub account_id: String,
    pub sync_enabled: bool,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct MailSyncSettingsUpdate {
    pub sync_enabled: bool,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncDueAccount {
    pub account_id: String,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
}
```

### `backend/src/workflows/mail_background_sync/models/status.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/models/status.rs`
- Size bytes / Размер в байтах: `769`
- Included characters / Включено символов: `769`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncStatus {
    pub account_id: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub last_fetched_messages: i64,
    pub last_projected_messages: i64,
    pub last_upserted_persons: i64,
    pub last_upserted_organizations: i64,
}
```

### `backend/src/workflows/mail_background_sync/provider.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/provider.rs`
- Size bytes / Размер в байтах: `1176`
- Included characters / Включено символов: `1176`
- Truncated / Обрезано: `no`

```rust
mod gmail;
mod imap;
mod projection;
mod summary;
mod types;

use crate::platform::communications::EmailSyncAdapterConfig;

use super::errors::ProviderSyncError;
use super::service::MailBackgroundSyncService;

pub(super) use self::summary::ProviderSyncSummary;
use self::types::ImapAccountConfig;
pub(super) use self::types::ProviderSyncContext;

impl MailBackgroundSyncService {
    pub(super) async fn execute_provider_sync(
        &self,
        adapter: &EmailSyncAdapterConfig,
        context: ProviderSyncContext<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        match adapter {
            EmailSyncAdapterConfig::Gmail { .. } => self.sync_gmail(context).await,
            EmailSyncAdapterConfig::Imap {
                host,
                port,
                tls,
                mailbox,
            } => {
                self.sync_imap(
                    context,
                    ImapAccountConfig {
                        host,
                        port: *port,
                        tls: *tls,
                        mailbox,
                    },
                )
                .await
            }
        }
    }
}
```

### `backend/src/workflows/mail_background_sync/provider/gmail.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/provider/gmail.rs`
- Size bytes / Размер в байтах: `2396`
- Included characters / Включено символов: `2396`
- Truncated / Обрезано: `no`

```rust
mod history;
mod message_list;

use serde_json::Value;

use super::super::errors::ProviderSyncError;
use super::super::service::MailBackgroundSyncService;
use super::{ProviderSyncContext, ProviderSyncSummary};

impl MailBackgroundSyncService {
    pub(in crate::workflows::mail_background_sync::provider) async fn sync_gmail(
        &self,
        context: ProviderSyncContext<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        let mut summary = ProviderSyncSummary::default();
        let checkpoint_next_page_token = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("next_page_token"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let checkpoint_page_kind = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("page_kind"))
            .and_then(Value::as_str);

        if checkpoint_next_page_token.is_some() && checkpoint_page_kind != Some("history") {
            self.sync_gmail_message_list_pages(&context, &mut summary, checkpoint_next_page_token)
                .await?;
            return Ok(summary);
        }

        if let Some(history_id) = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("history_id"))
            .and_then(Value::as_str)
            .map(str::to_owned)
        {
            let start_history_id = context
                .checkpoint_before
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("start_history_id"))
                .and_then(Value::as_str)
                .unwrap_or(&history_id)
                .to_owned();
            let history_page_token = if checkpoint_page_kind == Some("history") {
                checkpoint_next_page_token
            } else {
                None
            };
            let history_expired = self
                .sync_gmail_history_pages(
                    &context,
                    &mut summary,
                    &start_history_id,
                    history_page_token,
                )
                .await?;
            if !history_expired {
                return Ok(summary);
            }
        }

        self.sync_gmail_message_list_pages(&context, &mut summary, None)
            .await?;

        Ok(summary)
    }
}
```

### `backend/src/workflows/mail_background_sync/provider/gmail/history.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/provider/gmail/history.rs`
- Size bytes / Размер в байтах: `2813`
- Included characters / Включено символов: `2813`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::platform::communications::GmailHistoryFetchRequest;

use super::super::super::errors::ProviderSyncError;
use super::super::super::models::{MailSyncPhase, ProgressMode, ProgressUpdate};
use super::super::super::service::MailBackgroundSyncService;
use super::super::{ProviderSyncContext, ProviderSyncSummary};

impl MailBackgroundSyncService {
    pub(in crate::workflows::mail_background_sync::provider::gmail) async fn sync_gmail_history_pages(
        &self,
        context: &ProviderSyncContext<'_>,
        summary: &mut ProviderSyncSummary,
        start_history_id: &str,
        mut page_token: Option<String>,
    ) -> Result<bool, ProviderSyncError> {
        loop {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Listing,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let history_batch = self
                .provider_sync
                .fetch_gmail_history(GmailHistoryFetchRequest {
                    account_id: context.account.account_id.clone(),
                    start_history_id: start_history_id.to_owned(),
                    max_results: context.settings.batch_size as u16,
                    page_token,
                })
                .await;
            let batch = match history_batch {
                Ok(batch) => batch,
                Err(error) if error.history_expired => {
                    context
                        .store
                        .mark_recoverable_full_resync(context.run_id, "gmail_history_expired")
                        .await?;
                    return Ok(true);
                }
                Err(error) => return Err(error.into()),
            };
            page_token = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("next_page_token"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let fetched_count = batch.messages.len();
            self.project_batch(
                context.store,
                context.run_id,
                context.settings,
                summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if page_token.is_none() || fetched_count == 0 {
                break;
            }
        }

        Ok(false)
    }
}
```

### `backend/src/workflows/mail_background_sync/provider/gmail/message_list.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/provider/gmail/message_list.rs`
- Size bytes / Размер в байтах: `2268`
- Included characters / Включено символов: `2268`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::platform::communications::GmailMessageListFetchRequest;

use super::super::super::errors::ProviderSyncError;
use super::super::super::models::{MailSyncPhase, ProgressMode, ProgressUpdate};
use super::super::super::service::MailBackgroundSyncService;
use super::super::{ProviderSyncContext, ProviderSyncSummary};

impl MailBackgroundSyncService {
    pub(in crate::workflows::mail_background_sync::provider::gmail) async fn sync_gmail_message_list_pages(
        &self,
        context: &ProviderSyncContext<'_>,
        summary: &mut ProviderSyncSummary,
        mut page_token: Option<String>,
    ) -> Result<(), ProviderSyncError> {
        loop {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Listing,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let batch = self
                .provider_sync
                .fetch_gmail_message_list(GmailMessageListFetchRequest {
                    account_id: context.account.account_id.clone(),
                    max_results: context.settings.batch_size as u16,
                    page_token,
                })
                .await?;
            page_token = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("next_page_token"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let fetched_count = batch.messages.len();
            self.project_batch(
                context.store,
                context.run_id,
                context.settings,
                summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if page_token.is_none() || fetched_count == 0 {
                break;
            }
        }

        Ok(())
    }
}
```

### `backend/src/workflows/mail_background_sync/provider/imap.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/provider/imap.rs`
- Size bytes / Размер в байтах: `3874`
- Included characters / Включено символов: `3874`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::platform::communications::ImapMessageFetchRequest;

use super::super::errors::ProviderSyncError;
use super::super::models::{MailSyncPhase, ProgressMode, ProgressUpdate};
use super::super::service::MailBackgroundSyncService;
use super::types::ImapAccountConfig;
use super::{ProviderSyncContext, ProviderSyncSummary};

impl MailBackgroundSyncService {
    pub(in crate::workflows::mail_background_sync::provider) async fn sync_imap(
        &self,
        context: ProviderSyncContext<'_>,
        config: ImapAccountConfig<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        let mut summary = ProviderSyncSummary::default();
        let mut last_seen_uid = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("last_seen_uid"))
            .and_then(Value::as_u64)
            .and_then(|uid| u32::try_from(uid).ok());
        let checkpoint_uid_validity = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("uid_validity"))
            .and_then(Value::as_u64)
            .and_then(|uid_validity| u32::try_from(uid_validity).ok());
        let mut retried_after_uid_validity_reset = false;

        loop {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Fetching,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let batch = self
                .provider_sync
                .fetch_imap_messages(ImapMessageFetchRequest {
                    account_id: context.account.account_id.clone(),
                    provider_kind: context.account.provider_kind,
                    host: config.host.to_owned(),
                    port: config.port,
                    tls: config.tls,
                    mailbox: config.mailbox.to_owned(),
                    username: context.account.external_account_id.clone(),
                    max_messages: context.settings.batch_size as usize,
                    last_seen_uid,
                })
                .await?;
            let fetched_count = batch.messages.len();
            let batch_uid_validity = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("uid_validity"))
                .and_then(Value::as_u64)
                .and_then(|uid_validity| u32::try_from(uid_validity).ok());
            if !retried_after_uid_validity_reset
                && checkpoint_uid_validity.is_some()
                && batch_uid_validity.is_some()
                && checkpoint_uid_validity != batch_uid_validity
            {
                retried_after_uid_validity_reset = true;
                last_seen_uid = None;
                continue;
            }

            last_seen_uid = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("last_seen_uid"))
                .and_then(Value::as_u64)
                .and_then(|uid| u32::try_from(uid).ok())
                .or(last_seen_uid);
            self.project_batch(
                context.store,
                context.run_id,
                context.settings,
                &mut summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if fetched_count == 0 {
                break;
            }
        }

        Ok(summary)
    }
}
```

### `backend/src/workflows/mail_background_sync/provider/projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/provider/projection.rs`
- Size bytes / Размер в байтах: `2790`
- Included characters / Включено символов: `2790`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::storage::LocalCommunicationBlobPort;
use crate::platform::communications::EmailSyncBatch;
use crate::workflows::email_sync_pipeline::project_email_sync_batch_with_mail_blobs;
use crate::workflows::graph_projection::GraphProjectionService;

use super::super::errors::ProviderSyncError;
use super::super::models::{MailSyncPhase, MailSyncSettings, ProgressMode, ProgressUpdate};
use super::super::service::MailBackgroundSyncService;
use super::super::store::MailSyncStatePort;
use super::ProviderSyncSummary;

impl MailBackgroundSyncService {
    pub(in crate::workflows::mail_background_sync::provider) async fn project_batch(
        &self,
        store: &MailSyncStatePort,
        run_id: &str,
        settings: &MailSyncSettings,
        summary: &mut ProviderSyncSummary,
        account_id: &str,
        batch: EmailSyncBatch,
    ) -> Result<(), ProviderSyncError> {
        let fetched_count = batch.messages.len() as i64;
        summary.fetched_messages += fetched_count;
        summary.processed_messages += fetched_count;
        summary.current_batch_size = i32::try_from(fetched_count).unwrap_or(i32::MAX);
        summary.checkpoint_after = batch.checkpoint.clone();

        store
            .update_progress(ProgressUpdate {
                run_id,
                phase: MailSyncPhase::Projecting,
                progress_mode: ProgressMode::Indeterminate,
                progress_percent: None,
                processed_messages: summary.processed_messages,
                estimated_total_messages: summary.estimated_total_messages,
                current_batch_size: settings.batch_size,
            })
            .await?;

        let blob_store = LocalCommunicationBlobPort::new(&self.blob_root);
        let report = project_email_sync_batch_with_mail_blobs(
            self.pool.clone(),
            &blob_store,
            account_id,
            &format!("{run_id}:batch:{}", summary.processed_messages),
            &batch,
        )
        .await?;
        summary.apply_pipeline_report(&report);

        store
            .update_progress(ProgressUpdate {
                run_id,
                phase: MailSyncPhase::PersonsGraph,
                progress_mode: ProgressMode::Indeterminate,
                progress_percent: None,
                processed_messages: summary.processed_messages,
                estimated_total_messages: summary.estimated_total_messages,
                current_batch_size: settings.batch_size,
            })
            .await?;

        GraphProjectionService::new(self.pool.clone())
            .project_from_v1()
            .await?;

        if batch.checkpoint.is_some() {
            summary.checkpoint_saved = report.checkpoint_saved;
        }

        Ok(())
    }
}
```

### `backend/src/workflows/mail_background_sync/provider/summary.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/provider/summary.rs`
- Size bytes / Размер в байтах: `1308`
- Included characters / Включено символов: `1308`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::workflows::email_sync_pipeline::EmailSyncPipelineReport;

#[derive(Default)]
pub(in crate::workflows::mail_background_sync) struct ProviderSyncSummary {
    pub(in crate::workflows::mail_background_sync) processed_messages: i64,
    pub(in crate::workflows::mail_background_sync) estimated_total_messages: Option<i64>,
    pub(in crate::workflows::mail_background_sync) current_batch_size: i32,
    pub(in crate::workflows::mail_background_sync) fetched_messages: i64,
    pub(in crate::workflows::mail_background_sync) projected_messages: i64,
    pub(in crate::workflows::mail_background_sync) upserted_persons: i64,
    pub(in crate::workflows::mail_background_sync) upserted_organizations: i64,
    pub(in crate::workflows::mail_background_sync) checkpoint_after: Option<Value>,
    pub(in crate::workflows::mail_background_sync) checkpoint_saved: bool,
}

impl ProviderSyncSummary {
    pub(in crate::workflows::mail_background_sync::provider) fn apply_pipeline_report(
        &mut self,
        report: &EmailSyncPipelineReport,
    ) {
        self.projected_messages += report.projected_messages as i64;
        self.upserted_persons += report.upserted_person_identities as i64;
        self.upserted_organizations += report.upserted_organizations as i64;
    }
}
```

### `backend/src/workflows/mail_background_sync/provider/types.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/provider/types.rs`
- Size bytes / Размер в байтах: `1243`
- Included characters / Включено символов: `1243`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::domains::communications::core::CommunicationIngestionPort;
use crate::platform::communications::ProviderAccount;

use super::super::models::MailSyncSettings;
use super::super::store::MailSyncStatePort;

pub(in crate::workflows::mail_background_sync) struct ProviderSyncContext<'a> {
    pub(in crate::workflows::mail_background_sync) store: &'a MailSyncStatePort,
    pub(in crate::workflows::mail_background_sync) communication_store:
        &'a CommunicationIngestionPort,
    pub(in crate::workflows::mail_background_sync) account: &'a ProviderAccount,
    pub(in crate::workflows::mail_background_sync) run_id: &'a str,
    pub(in crate::workflows::mail_background_sync) settings: &'a MailSyncSettings,
    pub(in crate::workflows::mail_background_sync) checkpoint_before: Option<Value>,
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync::provider) struct ImapAccountConfig<'a> {
    pub(in crate::workflows::mail_background_sync::provider) host: &'a str,
    pub(in crate::workflows::mail_background_sync::provider) port: u16,
    pub(in crate::workflows::mail_background_sync::provider) tls: bool,
    pub(in crate::workflows::mail_background_sync::provider) mailbox: &'a str,
}
```

### `backend/src/workflows/mail_background_sync/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/rows.rs`
- Size bytes / Размер в байтах: `3323`
- Included characters / Включено символов: `3323`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::MailSyncError;
use super::models::{MailSyncDueAccount, MailSyncRun, MailSyncSettings, MailSyncStatus};

pub(super) fn row_to_settings(row: PgRow) -> Result<MailSyncSettings, MailSyncError> {
    Ok(MailSyncSettings {
        account_id: row.try_get("account_id")?,
        sync_enabled: row.try_get("sync_enabled")?,
        batch_size: row.try_get("batch_size")?,
        poll_interval_seconds: row.try_get("poll_interval_seconds")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_status(row: PgRow) -> Result<MailSyncStatus, MailSyncError> {
    Ok(MailSyncStatus {
        account_id: row.try_get("account_id")?,
        status: row.try_get("status")?,
        phase: row.try_get("phase")?,
        progress_mode: row.try_get("progress_mode")?,
        progress_percent: row.try_get("progress_percent")?,
        processed_messages: row.try_get("processed_messages")?,
        estimated_total_messages: row.try_get("estimated_total_messages")?,
        current_batch_size: row.try_get("current_batch_size")?,
        last_started_at: row.try_get("last_started_at")?,
        last_completed_at: row.try_get("last_completed_at")?,
        next_run_at: row.try_get("next_run_at")?,
        last_error_code: row.try_get("last_error_code")?,
        last_error_message: row.try_get("last_error_message")?,
        last_fetched_messages: row.try_get("last_fetched_messages")?,
        last_projected_messages: row.try_get("last_projected_messages")?,
        last_upserted_persons: row.try_get("last_upserted_persons")?,
        last_upserted_organizations: row.try_get("last_upserted_organizations")?,
    })
}

pub(super) fn row_to_due_account(row: PgRow) -> Result<MailSyncDueAccount, MailSyncError> {
    Ok(MailSyncDueAccount {
        account_id: row.try_get("account_id")?,
        batch_size: row.try_get("batch_size")?,
        poll_interval_seconds: row.try_get("poll_interval_seconds")?,
    })
}

pub(super) fn row_to_run(row: PgRow) -> Result<MailSyncRun, MailSyncError> {
    Ok(MailSyncRun {
        run_id: row.try_get("run_id")?,
        account_id: row.try_get("account_id")?,
        trigger: row.try_get("trigger")?,
        status: row.try_get("status")?,
        phase: row.try_get("phase")?,
        progress_mode: row.try_get("progress_mode")?,
        progress_percent: row.try_get("progress_percent")?,
        processed_messages: row.try_get("processed_messages")?,
        estimated_total_messages: row.try_get("estimated_total_messages")?,
        current_batch_size: row.try_get("current_batch_size")?,
        fetched_messages: row.try_get("fetched_messages")?,
        projected_messages: row.try_get("projected_messages")?,
        upserted_persons: row.try_get("upserted_persons")?,
        upserted_organizations: row.try_get("upserted_organizations")?,
        checkpoint_before: row.try_get("checkpoint_before")?,
        checkpoint_after: row.try_get("checkpoint_after")?,
        checkpoint_saved: row.try_get("checkpoint_saved")?,
        error_code: row.try_get("error_code")?,
        error_message: row.try_get("error_message")?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        next_run_at: row.try_get("next_run_at")?,
    })
}
```

### `backend/src/workflows/mail_background_sync/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/service.rs`
- Size bytes / Размер в байтах: `8847`
- Included characters / Включено символов: `8847`
- Truncated / Обрезано: `no`

```rust
use std::path::PathBuf;

use chrono::Utc;
use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::domains::communications::core::CommunicationIngestionPort;
use crate::domains::communications::core::CommunicationProviderAccountPort;
use crate::platform::communications::{SharedEmailProviderSyncPort, plan_email_sync};
use crate::vault::HostVault;

use super::errors::MailSyncError;
use super::models::{
    FinishRun, MailSyncPhase, MailSyncRunResponse, MailSyncRunStatus, MailSyncSettings,
    MailSyncTrigger, ProgressMode, SanitizedSyncFailure,
};
use super::provider::ProviderSyncContext;
use super::store::MailSyncStatePort;
use super::validation::{next_run_at, require_unlocked_vault};

#[derive(Clone)]
pub struct MailBackgroundSyncService {
    pub(super) pool: PgPool,
    pub(super) vault: HostVault,
    pub(super) blob_root: PathBuf,
    pub(super) provider_sync: SharedEmailProviderSyncPort,
}

impl MailBackgroundSyncService {
    pub fn new(
        pool: PgPool,
        vault: HostVault,
        blob_root: impl Into<PathBuf>,
        provider_sync: SharedEmailProviderSyncPort,
    ) -> Self {
        Self {
            pool,
            vault,
            blob_root: blob_root.into(),
            provider_sync,
        }
    }

    pub async fn run_due_accounts(&self) -> Result<Vec<MailSyncRunResponse>, MailSyncError> {
        let store = MailSyncStatePort::new(self.pool.clone());
        let accounts = store.due_accounts(Utc::now(), 20).await?;
        let mut responses = Vec::new();
        for account in accounts {
            responses.push(
                self.run_account(&account.account_id, MailSyncTrigger::Scheduled)
                    .await?,
            );
        }
        Ok(responses)
    }

    pub async fn run_account(
        &self,
        account_id: &str,
        trigger: MailSyncTrigger,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let store = MailSyncStatePort::new(self.pool.clone());
        let communication_store = CommunicationIngestionPort::new(self.pool.clone());
        let account = CommunicationProviderAccountPort::new(self.pool.clone())
            .get(account_id)
            .await?
            .ok_or(MailSyncError::AccountNotFound)?;
        let settings = store.settings_for_account(account_id).await?;

        if !settings.sync_enabled {
            let run = store
                .start_run(account_id, trigger, &settings, None)
                .await
                .map_err(|error| match error {
                    MailSyncError::RunAlreadyActive => MailSyncError::RunAlreadyActive,
                    other => other,
                })?;
            return store
                .finish_run(
                    &run.run_id,
                    FinishRun {
                        status: MailSyncRunStatus::Skipped,
                        phase: MailSyncPhase::Completed,
                        progress_mode: ProgressMode::None,
                        progress_percent: None,
                        processed_messages: run.processed_messages,
                        estimated_total_messages: run.estimated_total_messages,
                        fetched_messages: 0,
                        projected_messages: 0,
                        upserted_persons: 0,
                        upserted_organizations: 0,
                        checkpoint_after: None,
                        checkpoint_saved: false,
                        error_code: Some("sync_disabled".to_owned()),
                        error_message: Some("Mail sync is disabled for this account".to_owned()),
                        next_run_at: next_run_at(&settings),
                    },
                )
                .await
                .map(Into::into);
        }

        let plan = match plan_email_sync(&account) {
            Ok(plan) => plan,
            Err(error) => {
                return self
                    .fail_without_provider_io(
                        account_id,
                        trigger,
                        &settings,
                        None,
                        SanitizedSyncFailure::from_plan(error),
                    )
                    .await;
            }
        };
        let checkpoint_before = communication_store
            .checkpoint(account_id, &plan.stream_id)
            .await?
            .map(|checkpoint| checkpoint.checkpoint);

        let run = match store
            .start_run(account_id, trigger, &settings, checkpoint_before.clone())
            .await
        {
            Ok(run) => run,
            Err(MailSyncError::RunAlreadyActive) => {
                return store.latest_run_response(account_id).await;
            }
            Err(error) => return Err(error),
        };

        if let Err(error) = require_unlocked_vault(&self.vault) {
            return store
                .finish_run(
                    &run.run_id,
                    FinishRun::failed(
                        MailSyncPhase::Failed,
                        SanitizedSyncFailure::from_vault(error),
                        &settings,
                    ),
                )
                .await
                .map(Into::into);
        }

        let result = self
            .execute_provider_sync(
                &plan.adapter_config,
                ProviderSyncContext {
                    store: &store,
                    communication_store: &communication_store,
                    account: &account,
                    run_id: &run.run_id,
                    settings: &settings,
                    checkpoint_before,
                },
            )
            .await;

        match result {
            Ok(summary) => store
                .finish_run(
                    &run.run_id,
                    FinishRun {
                        status: MailSyncRunStatus::Completed,
                        phase: MailSyncPhase::Completed,
                        progress_mode: ProgressMode::Determinate,
                        progress_percent: Some(100),
                        processed_messages: summary.processed_messages,
                        estimated_total_messages: summary.estimated_total_messages,
                        fetched_messages: summary.fetched_messages,
                        projected_messages: summary.projected_messages,
                        upserted_persons: summary.upserted_persons,
                        upserted_organizations: summary.upserted_organizations,
                        checkpoint_after: summary.checkpoint_after,
                        checkpoint_saved: summary.checkpoint_saved,
                        error_code: None,
                        error_message: None,
                        next_run_at: next_run_at(&settings),
                    },
                )
                .await
                .map(Into::into),
            Err(error) => store
                .finish_run(
                    &run.run_id,
                    FinishRun::failed(
                        MailSyncPhase::Failed,
                        SanitizedSyncFailure::from(error),
                        &settings,
                    ),
                )
                .await
                .map(Into::into),
        }
    }

    pub async fn run_account_full_resync(
        &self,
        account_id: &str,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let communication_store = CommunicationIngestionPort::new(self.pool.clone());
        let account = CommunicationProviderAccountPort::new(self.pool.clone())
            .get(account_id)
            .await?
            .ok_or(MailSyncError::AccountNotFound)?;
        if let Ok(plan) = plan_email_sync(&account) {
            communication_store
                .delete_checkpoint(account_id, &plan.stream_id)
                .await?;
        }

        self.run_account(account_id, MailSyncTrigger::Manual).await
    }

    async fn fail_without_provider_io(
        &self,
        account_id: &str,
        trigger: MailSyncTrigger,
        settings: &MailSyncSettings,
        checkpoint_before: Option<Value>,
        failure: SanitizedSyncFailure,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let store = MailSyncStatePort::new(self.pool.clone());
        let run = match store
            .start_run(account_id, trigger, settings, checkpoint_before)
            .await
        {
            Ok(run) => run,
            Err(MailSyncError::RunAlreadyActive) => {
                return store.latest_run_response(account_id).await;
            }
            Err(error) => return Err(error),
        };
        store
            .finish_run(
                &run.run_id,
                FinishRun::failed(MailSyncPhase::Failed, failure, settings),
            )
            .await
            .map(Into::into)
    }
}
```
