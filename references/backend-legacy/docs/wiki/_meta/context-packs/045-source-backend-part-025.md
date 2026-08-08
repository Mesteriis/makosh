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

- Chunk ID / ID чанка: `045-source-backend-part-025`
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

### `backend/src/domains/documents/processing/jobs.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/jobs.rs`
- Size bytes / Размер в байтах: `10545`
- Included characters / Включено символов: `10545`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::constants::DEFAULT_MAX_ATTEMPTS;
use super::errors::DocumentProcessingError;
use super::evidence::link_document_processing_entity_in_transaction;
use super::ids::job_id;
use super::models::{DocumentProcessingJob, DocumentProcessingStatus, DocumentProcessingStep};
use super::rows::try_row_to_job;
use super::store::DocumentProcessingStore;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

async fn capture_job_observation(
    tx: &mut Transaction<'_, Postgres>,
    job: &DocumentProcessingJob,
    kind_code: &str,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), DocumentProcessingError> {
    let observed_at = job.updated_at;
    let observation = ObservationStore::capture_in_transaction(
        tx,
        &NewObservation::new(
            kind_code,
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "job_id": job.job_id,
                "document_id": job.document_id,
                "step": job.step,
                "status": job.status,
                "attempts": job.attempts,
                "max_attempts": job.max_attempts,
                "last_error_summary": job.last_error_summary,
                "queued_at": job.queued_at,
                "started_at": job.started_at,
                "finished_at": job.finished_at,
                "operation": relationship_kind,
            }),
            format!(
                "document-processing-job://{}/{}",
                job.job_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_document_processing_entity_in_transaction(
        tx,
        &observation.observation_id,
        "document_processing_job",
        job.job_id.clone(),
        relationship_kind,
        json!({
            "document_id": job.document_id,
            "step": job.step,
            "status": job.status,
        }),
    )
    .await?;
    Ok(())
}

impl DocumentProcessingStore {
    pub(super) async fn upsert_job(
        &self,
        document_id: &str,
        step: DocumentProcessingStep,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let job_id = job_id(document_id, step);
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO document_processing_jobs (
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                updated_at
            )
            VALUES ($1, $2, $3, 'queued', 0, $4, now())
            ON CONFLICT (document_id, step)
            DO UPDATE
                SET status = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.status
                    ELSE 'queued'
                END,
                attempts = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.attempts
                    ELSE 0
                END,
                last_error_summary = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.last_error_summary
                    ELSE NULL
                END,
                started_at = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.started_at
                    ELSE NULL
                END,
                finished_at = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.finished_at
                    ELSE NULL
                END,
                updated_at = now()
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(&job_id)
        .bind(document_id)
        .bind(step.as_str())
        .bind(DEFAULT_MAX_ATTEMPTS)
        .fetch_one(&mut *tx)
        .await?;
        let job = try_row_to_job(row)?;
        capture_job_observation(
            &mut tx,
            &job,
            "DOCUMENT_PROCESSING_JOB",
            "queued",
            "documents.processing.upsert_job",
        )
        .await?;
        tx.commit().await?;
        Ok(job)
    }

    pub(super) async fn next_jobs(
        &self,
        limit: i64,
    ) -> Result<Vec<QueuedJob>, DocumentProcessingError> {
        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE status = 'queued'
              AND attempts < max_attempts
            ORDER BY queued_at ASC, job_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(QueuedJob {
                    base: try_row_to_job(row)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub(super) async fn mark_running(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: &QueuedJob,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = 'running',
                attempts = attempts + 1,
                started_at = now(),
                updated_at = now()
            WHERE job_id = $1
              AND status = 'queued'
              AND attempts < max_attempts
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(&job.base.job_id)
        .fetch_optional(&mut **tx)
        .await?;

        let job = row
            .map(try_row_to_job)
            .ok_or(DocumentProcessingError::JobNotFound)??;
        capture_job_observation(
            tx,
            &job,
            "DOCUMENT_PROCESSING_JOB_STATUS",
            "running",
            "documents.processing.mark_running",
        )
        .await?;
        Ok(job)
    }

    pub(super) async fn job_for_update(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job_id: &str,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE job_id = $1
            FOR UPDATE
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await?;

        row.map(try_row_to_job)
            .ok_or(DocumentProcessingError::JobNotFound)?
    }

    pub(super) async fn requeue_failed_job(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job_id: &str,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = 'queued',
                attempts = 0,
                last_error_summary = NULL,
                started_at = NULL,
                finished_at = NULL,
                updated_at = now()
            WHERE job_id = $1
              AND status = 'failed'
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await?;

        let job = row
            .map(try_row_to_job)
            .ok_or(DocumentProcessingError::RetryRequiresFailedJob)??;
        capture_job_observation(
            tx,
            &job,
            "DOCUMENT_PROCESSING_JOB_STATUS",
            "requeued",
            "documents.processing.requeue_failed_job",
        )
        .await?;
        Ok(job)
    }

    pub(super) async fn finish_job(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: &DocumentProcessingJob,
        status: DocumentProcessingStatus,
        last_error_summary: Option<String>,
    ) -> Result<(), DocumentProcessingError> {
        sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = $2,
                last_error_summary = $3,
                finished_at = now(),
                updated_at = now()
            WHERE job_id = $1
            "#,
        )
        .bind(&job.job_id)
        .bind(status.as_str())
        .bind(last_error_summary)
        .execute(&mut **tx)
        .await?;
        let job = self.job_for_update(tx, &job.job_id).await?;
        capture_job_observation(
            tx,
            &job,
            "DOCUMENT_PROCESSING_JOB_STATUS",
            status.as_str(),
            "documents.processing.finish_job",
        )
        .await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) struct QueuedJob {
    pub(super) base: DocumentProcessingJob,
}
```

### `backend/src/domains/documents/processing/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/models.rs`
- Size bytes / Размер в байтах: `4065`
- Included characters / Включено символов: `4065`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::errors::DocumentProcessingError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentProcessingStep {
    ExtractText,
    Ocr,
}

impl DocumentProcessingStep {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ExtractText => "extract_text",
            Self::Ocr => "ocr",
        }
    }

    pub(super) fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "extract_text" => Ok(Self::ExtractText),
            "ocr" => Ok(Self::Ocr),
            _ => Err(DocumentProcessingError::InvalidStep(raw.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentProcessingStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

impl DocumentProcessingStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub(super) fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            _ => Err(DocumentProcessingError::InvalidStatus(raw.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentArtifactKind {
    ExtractedText,
    OcrText,
}

impl DocumentArtifactKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ExtractedText => "extracted_text",
            Self::OcrText => "ocr_text",
        }
    }

    pub(super) fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "extracted_text" => Ok(Self::ExtractedText),
            "ocr_text" => Ok(Self::OcrText),
            _ => Err(DocumentProcessingError::InvalidArtifactKind(raw.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingJob {
    pub job_id: String,
    pub document_id: String,
    pub step: DocumentProcessingStep,
    pub status: DocumentProcessingStatus,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_error_summary: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingArtifact {
    pub artifact_id: String,
    pub document_id: String,
    pub job_id: String,
    pub artifact_kind: DocumentArtifactKind,
    pub content_sha256: String,
    pub text_content: Option<String>,
    pub storage_kind: Option<String>,
    pub storage_path: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentProcessingRecord {
    pub document_id: String,
    pub jobs: Vec<DocumentProcessingJob>,
    pub artifacts: Vec<DocumentProcessingArtifact>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct DocumentProcessingRunReport {
    pub jobs_seen: i64,
    pub jobs_queued: i64,
    pub jobs_succeeded: i64,
    pub jobs_failed: i64,
    pub jobs_skipped: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentProcessingRetryCommand {
    pub command_id: String,
    pub job_id: String,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingRetryCommandResult {
    pub job_id: String,
    pub status: DocumentProcessingStatus,
    pub event_id: String,
}
```

### `backend/src/domains/documents/processing/retry.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/retry.rs`
- Size bytes / Размер в байтах: `6315`
- Included characters / Включено символов: `6315`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use crate::domains::documents::processing::evidence::link_document_processing_entity_in_transaction;
use crate::platform::events::{EventStore, NewEventEnvelope};

use super::constants::{
    RETRY_EVENT_ID_PREFIX, RETRY_EVENT_TYPE, RETRY_SOURCE_KIND, RETRY_SOURCE_PROVIDER,
};
use super::errors::DocumentProcessingError;
use super::models::{
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
};
use super::store::DocumentProcessingStore;
use super::validation::validate_non_empty;

impl DocumentProcessingStore {
    pub async fn retry_failed_job(
        &self,
        command: &DocumentProcessingRetryCommand,
    ) -> Result<DocumentProcessingRetryCommandResult, DocumentProcessingError> {
        self.retry_failed_job_with_observation(command, None).await
    }

    pub async fn retry_failed_job_with_observation(
        &self,
        command: &DocumentProcessingRetryCommand,
        observation_id: Option<&str>,
    ) -> Result<DocumentProcessingRetryCommandResult, DocumentProcessingError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let job_id = validate_non_empty("job_id", &command.job_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;
        let event_id = format!("{RETRY_EVENT_ID_PREFIX}{command_id}");

        if let Some(result) = self
            .retry_result_for_existing_event(&event_id, &job_id)
            .await?
        {
            self.link_retry_observation_if_present(observation_id, &job_id, &event_id)
                .await?;
            return Ok(result);
        }

        let mut transaction = self.pool.begin().await?;
        let current_job = self.job_for_update(&mut transaction, &job_id).await?;
        if current_job.status != DocumentProcessingStatus::Failed {
            if let Some(result) = self
                .retry_result_for_existing_event(&event_id, &job_id)
                .await?
            {
                self.link_retry_observation_if_present(observation_id, &job_id, &event_id)
                    .await?;
                return Ok(result);
            }
            return Err(DocumentProcessingError::RetryRequiresFailedJob);
        }

        let event = RetryCommandEvent {
            command_id,
            job_id: job_id.clone(),
            actor_id,
            event_id: event_id.clone(),
            occurred_at: Utc::now(),
        }
        .into_event()?;

        if let Err(error) = EventStore::append_in_transaction(&mut transaction, &event).await {
            if error.is_unique_violation() {
                transaction.rollback().await?;
                return self
                    .retry_result_for_existing_event(&event_id, &job_id)
                    .await?
                    .ok_or(DocumentProcessingError::RetryCommandConflict);
            }

            return Err(DocumentProcessingError::EventStore(error));
        }
        let retried_job = self.requeue_failed_job(&mut transaction, &job_id).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_document_processing_entity_in_transaction(
                &mut transaction,
                observation_id,
                "document_processing_job",
                retried_job.job_id.clone(),
                "retry_command",
                json!({
                    "event_id": event_id,
                }),
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(DocumentProcessingRetryCommandResult {
            job_id: retried_job.job_id,
            status: retried_job.status,
            event_id,
        })
    }

    async fn link_retry_observation_if_present(
        &self,
        observation_id: Option<&str>,
        job_id: &str,
        event_id: &str,
    ) -> Result<(), DocumentProcessingError> {
        let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) else {
            return Ok(());
        };

        let mut transaction = self.pool.begin().await?;
        link_document_processing_entity_in_transaction(
            &mut transaction,
            observation_id,
            "document_processing_job",
            job_id.to_owned(),
            "retry_command",
            json!({
                "event_id": event_id,
            }),
        )
        .await?;
        transaction.commit().await?;

        Ok(())
    }

    async fn retry_result_for_existing_event(
        &self,
        event_id: &str,
        job_id: &str,
    ) -> Result<Option<DocumentProcessingRetryCommandResult>, DocumentProcessingError> {
        let Some(event) = EventStore::new(self.pool.clone())
            .get_by_id(event_id)
            .await?
        else {
            return Ok(None);
        };

        let Some(event_job_id) = event.payload.get("job_id").and_then(Value::as_str) else {
            return Err(DocumentProcessingError::RetryCommandConflict);
        };

        if event.event_type != RETRY_EVENT_TYPE || event_job_id != job_id {
            return Err(DocumentProcessingError::RetryCommandConflict);
        }

        Ok(Some(DocumentProcessingRetryCommandResult {
            job_id: job_id.to_owned(),
            status: DocumentProcessingStatus::Queued,
            event_id: event_id.to_owned(),
        }))
    }
}

#[derive(Debug)]
struct RetryCommandEvent {
    command_id: String,
    job_id: String,
    actor_id: String,
    event_id: String,
    occurred_at: DateTime<Utc>,
}

impl RetryCommandEvent {
    fn into_event(self) -> Result<NewEventEnvelope, DocumentProcessingError> {
        let job_id = self.job_id;
        Ok(NewEventEnvelope::builder(
            self.event_id,
            RETRY_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": RETRY_SOURCE_KIND,
                "provider": RETRY_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "document_processing_job",
                "job_id": job_id.clone(),
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(json!({ "job_id": job_id }))
        .build()?)
    }
}
```

### `backend/src/domains/documents/processing/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/rows.rs`
- Size bytes / Размер в байтах: `1840`
- Included characters / Включено символов: `1840`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::DocumentProcessingError;
use super::models::{
    DocumentArtifactKind, DocumentProcessingArtifact, DocumentProcessingJob,
    DocumentProcessingStatus, DocumentProcessingStep,
};

pub(super) fn try_row_to_job(row: PgRow) -> Result<DocumentProcessingJob, DocumentProcessingError> {
    let step = DocumentProcessingStep::parse(row.try_get::<String, _>("step")?.as_str())?;
    let status = DocumentProcessingStatus::parse(row.try_get::<String, _>("status")?.as_str())?;

    Ok(DocumentProcessingJob {
        job_id: row.try_get("job_id")?,
        document_id: row.try_get("document_id")?,
        step,
        status,
        attempts: row.try_get("attempts")?,
        max_attempts: row.try_get("max_attempts")?,
        last_error_summary: row.try_get("last_error_summary")?,
        queued_at: row.try_get("queued_at")?,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn try_row_to_artifact(
    row: PgRow,
) -> Result<DocumentProcessingArtifact, DocumentProcessingError> {
    let artifact_kind =
        DocumentArtifactKind::parse(row.try_get::<String, _>("artifact_kind")?.as_str())?;

    Ok(DocumentProcessingArtifact {
        artifact_id: row.try_get("artifact_id")?,
        document_id: row.try_get("document_id")?,
        job_id: row.try_get("job_id")?,
        artifact_kind,
        content_sha256: row.try_get("content_sha256")?,
        text_content: row.try_get("text_content")?,
        storage_kind: row.try_get("storage_kind")?,
        storage_path: row.try_get("storage_path")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}
```

### `backend/src/domains/documents/processing/runner.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/runner.rs`
- Size bytes / Размер в байтах: `5563`
- Included characters / Включено символов: `5563`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use super::errors::DocumentProcessingError;
use super::jobs::QueuedJob;
use super::models::{
    DocumentArtifactKind, DocumentProcessingJob, DocumentProcessingRunReport,
    DocumentProcessingStatus, DocumentProcessingStep,
};
use super::store::DocumentProcessingStore;
use super::validation::validate_limit;

impl DocumentProcessingStore {
    pub async fn run_queued_jobs(
        &self,
        limit: i64,
    ) -> Result<DocumentProcessingRunReport, DocumentProcessingError> {
        let limit = validate_limit(limit)?;

        let candidate_jobs = self.next_jobs(limit).await?;

        let mut report = DocumentProcessingRunReport {
            jobs_seen: 0,
            jobs_queued: 0,
            jobs_succeeded: 0,
            jobs_failed: 0,
            jobs_skipped: 0,
        };

        for candidate in candidate_jobs {
            report.jobs_seen += 1;
            match self.run_single_job(candidate).await {
                Ok(DocumentProcessingRunStepResult::Succeeded) => {
                    report.jobs_succeeded += 1;
                    report.jobs_queued += 1;
                }
                Ok(DocumentProcessingRunStepResult::Skipped(_)) => {
                    report.jobs_skipped += 1;
                    report.jobs_queued += 1;
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }

        Ok(report)
    }

    async fn run_single_job(
        &self,
        job: QueuedJob,
    ) -> Result<DocumentProcessingRunStepResult, DocumentProcessingError> {
        let mut transaction = self.pool.begin().await?;
        let running_job = self.mark_running(&mut transaction, &job).await?;

        let result = match running_job.step {
            DocumentProcessingStep::ExtractText => {
                self.run_extract_text_step(&mut transaction, &running_job)
                    .await
            }
            DocumentProcessingStep::Ocr => self.run_ocr_step(&mut transaction, &running_job).await,
        };

        match result {
            Ok(DocumentProcessingRunStepResult::Succeeded) => {
                self.finish_job(
                    &mut transaction,
                    &running_job,
                    DocumentProcessingStatus::Succeeded,
                    None,
                )
                .await?;
                transaction.commit().await?;
                Ok(DocumentProcessingRunStepResult::Succeeded)
            }
            Ok(DocumentProcessingRunStepResult::Skipped(summary)) => {
                self.finish_job(
                    &mut transaction,
                    &running_job,
                    DocumentProcessingStatus::Skipped,
                    Some(summary.clone()),
                )
                .await?;
                transaction.commit().await?;
                Ok(DocumentProcessingRunStepResult::Skipped(summary))
            }
            Err(error) => {
                let summary = safe_summary(&error.to_string());
                self.finish_job(
                    &mut transaction,
                    &running_job,
                    DocumentProcessingStatus::Failed,
                    Some(summary),
                )
                .await?;
                transaction.commit().await?;
                Err(error)
            }
        }
    }

    async fn run_extract_text_step(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        job: &DocumentProcessingJob,
    ) -> Result<DocumentProcessingRunStepResult, DocumentProcessingError> {
        let document = self
            .document_for_id(transaction, &job.document_id)
            .await?
            .ok_or(DocumentProcessingError::DocumentNotFound)?;

        if document.kind == "markdown" {
            if document.extracted_text.trim().is_empty() {
                return Err(DocumentProcessingError::MissingSourceText);
            }

            self.upsert_artifact(
                transaction,
                job,
                DocumentArtifactKind::ExtractedText,
                Some(document.extracted_text),
            )
            .await?;
            return Ok(DocumentProcessingRunStepResult::Succeeded);
        }

        Ok(DocumentProcessingRunStepResult::Skipped(format!(
            "extract text is not supported for document kind {}",
            document.kind
        )))
    }

    async fn run_ocr_step(
        &self,
        _transaction: &mut Transaction<'_, Postgres>,
        _job: &DocumentProcessingJob,
    ) -> Result<DocumentProcessingRunStepResult, DocumentProcessingError> {
        Ok(DocumentProcessingRunStepResult::Skipped(
            "ocr backend is not configured".to_owned(),
        ))
    }
}

#[derive(Debug)]
enum DocumentProcessingRunStepResult {
    Succeeded,
    Skipped(String),
}

fn safe_summary(value: &str) -> String {
    let sanitized = value
        .chars()
        .filter(|character| !character.is_control() || *character == '\n')
        .collect::<String>();
    sanitized
        .chars()
        .take(240)
        .collect::<String>()
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::safe_summary;

    #[test]
    fn safe_summary_truncates_to_240_and_removes_control_chars() {
        let long_text = "a\n".repeat(200) + &"b".repeat(80);
        let summary = safe_summary(&long_text);

        assert!(summary.chars().count() <= 240);
        assert!(!summary.contains('\u{0007}'));
        assert!(summary.contains('\n'));
    }
}
```

### `backend/src/domains/documents/processing/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/service.rs`
- Size bytes / Размер в байтах: `2245`
- Included characters / Включено символов: `2245`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{
    DocumentProcessingError, DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult,
    DocumentProcessingStore,
};

#[derive(Clone)]
pub struct DocumentProcessingCommandService {
    pool: PgPool,
}

impl DocumentProcessingCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn retry_failed_job_manual(
        &self,
        command: &DocumentProcessingRetryCommand,
    ) -> Result<DocumentProcessingRetryCommandResult, DocumentProcessingCommandServiceError> {
        let result = DocumentProcessingStore::new(self.pool.clone())
            .retry_failed_job(command)
            .await?;

        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "DOCUMENT_PROCESSING_JOB_STATUS",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "job_id": result.job_id,
                        "status": serde_json::to_value(result.status).unwrap_or(Value::Null),
                        "event_id": result.event_id,
                        "operation": "document_processing_retry",
                    }),
                    format!("document-processing://jobs/{}/retry", result.job_id),
                )
                .provenance(json!({
                    "captured_by": "documents.processing_service.retry_failed_job_manual",
                    "operation": "retry_failed_job_manual",
                })),
            )
            .await?;

        DocumentProcessingStore::new(self.pool.clone())
            .retry_failed_job_with_observation(command, Some(&observation.observation_id))
            .await?;

        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum DocumentProcessingCommandServiceError {
    #[error(transparent)]
    DocumentProcessing(#[from] DocumentProcessingError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
```

### `backend/src/domains/documents/processing/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/store.rs`
- Size bytes / Размер в байтах: `4546`
- Included characters / Включено символов: `4546`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::errors::DocumentProcessingError;
use super::models::{
    DocumentArtifactKind, DocumentProcessingArtifact, DocumentProcessingJob,
    DocumentProcessingRecord, DocumentProcessingStep,
};
use super::rows::{try_row_to_artifact, try_row_to_job};
use super::validation::{validate_non_empty, validate_optional_limit};

#[derive(Clone)]
pub struct DocumentProcessingStore {
    pub(super) pool: PgPool,
}

impl DocumentProcessingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;
        self.ensure_document_exists(&document_id).await?;
        let extract_text_job = self
            .upsert_job(&document_id, DocumentProcessingStep::ExtractText)
            .await?;
        let ocr_job = self
            .upsert_job(&document_id, DocumentProcessingStep::Ocr)
            .await?;

        Ok(vec![extract_text_job, ocr_job])
    }

    pub async fn list_jobs(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let limit = validate_optional_limit(limit)?;

        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            ORDER BY queued_at DESC, job_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_job)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn list_jobs_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE document_id = $1
            ORDER BY queued_at
            "#,
        )
        .bind(&document_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_job)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn list_artifacts_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingArtifact>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                artifact_id,
                document_id,
                job_id,
                artifact_kind,
                content_sha256,
                text_content,
                storage_kind,
                storage_path,
                metadata,
                created_at
            FROM document_artifacts
            WHERE document_id = $1
            ORDER BY artifact_kind
            "#,
        )
        .bind(&document_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_artifact)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn document_processing(
        &self,
        document_id: &str,
    ) -> Result<DocumentProcessingRecord, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;
        let Some(_) = self.document_record_by_id(&document_id).await? else {
            return Err(DocumentProcessingError::DocumentNotFound);
        };

        let jobs = self.list_jobs_for_document(&document_id).await?;
        let artifacts = self.list_artifacts_for_document(&document_id).await?;

        Ok(DocumentProcessingRecord {
            document_id,
            jobs,
            artifacts,
        })
    }
}
```

### `backend/src/domains/documents/processing/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/validation.rs`
- Size bytes / Размер в байтах: `783`
- Included characters / Включено символов: `783`
- Truncated / Обрезано: `no`

```rust
use super::constants::{DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT, MIN_LIST_LIMIT};
use super::errors::DocumentProcessingError;

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, DocumentProcessingError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(DocumentProcessingError::EmptyField(field));
    }

    Ok(value.to_owned())
}

pub(super) fn validate_limit(limit: i64) -> Result<i64, DocumentProcessingError> {
    if !(MIN_LIST_LIMIT..=MAX_LIST_LIMIT).contains(&limit) {
        return Err(DocumentProcessingError::InvalidLimit);
    }
    Ok(limit)
}

pub(super) fn validate_optional_limit(limit: Option<i64>) -> Result<i64, DocumentProcessingError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIST_LIMIT))
}
```

### `backend/src/domains/graph/core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core.rs`
- Size bytes / Размер в байтах: `654`
- Included characters / Включено символов: `654`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod errors;
mod ids;
mod models;
mod queries;
mod row_mapping;
mod store;
mod validation;

pub use constants::{GRAPH_NEIGHBORHOOD_EDGE_LIMIT, GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT};
pub use errors::GraphStoreError;
pub use errors::GraphStoreError as GraphProjectionPortError;
pub use ids::{edge_id, evidence_id, node_id};
pub use models::{
    GraphCount, GraphEdge, GraphEvidenceSourceKind, GraphEvidenceSummary, GraphNeighborhood,
    GraphNode, GraphNodeKind, GraphReviewState, GraphSummary, NewGraphEdge, NewGraphEvidence,
    NewGraphNode, RelationshipType,
};
pub use store::GraphStore;
pub use store::GraphStore as GraphProjectionPort;
```

### `backend/src/domains/graph/core/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core/constants.rs`
- Size bytes / Размер в байтах: `108`
- Included characters / Включено символов: `108`
- Truncated / Обрезано: `no`

```rust
pub const GRAPH_NEIGHBORHOOD_EDGE_LIMIT: i64 = 100;
pub const GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT: i64 = 100;
```

### `backend/src/domains/graph/core/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core/errors.rs`
- Size bytes / Размер в байтах: `1288`
- Included characters / Включено символов: `1288`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("graph evidence source kind `{source_kind}` requires observation_id")]
    MissingObservationEvidence { source_kind: &'static str },

    #[error("observation graph evidence must use the same source_id and observation_id")]
    ObservationSourceMismatch,

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("graph edge confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(f64),

    #[error("graph edges require evidence in the first graph slice")]
    SystemEdgeRequiresEvidence,

    #[error("closed temporal graph edges are unsupported in the first graph slice")]
    TemporalEdgesUnsupported,

    #[error("unknown graph node kind stored in database: {0}")]
    UnknownNodeKind(String),

    #[error("unknown graph relationship type stored in database: {0}")]
    UnknownRelationshipType(String),

    #[error("unknown graph review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("unknown graph evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),
}
```

### `backend/src/domains/graph/core/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core/ids.rs`
- Size bytes / Размер в байтах: `798`
- Included characters / Включено символов: `798`
- Truncated / Обрезано: `no`

```rust
pub use crate::platform::graph::node_id;

use super::models::{GraphEvidenceSourceKind, RelationshipType};

pub fn edge_id(
    source_node_id: &str,
    relationship_type: RelationshipType,
    target_node_id: &str,
) -> String {
    format!(
        "graph:edge:v1:{}:{}:{}:{}:{}:{}",
        source_node_id.len(),
        source_node_id,
        relationship_type.as_str().len(),
        relationship_type.as_str(),
        target_node_id.len(),
        target_node_id
    )
}

pub fn evidence_id(edge_id: &str, source_kind: GraphEvidenceSourceKind, source_id: &str) -> String {
    format!(
        "graph:evidence:v1:{}:{}:{}:{}:{}:{}",
        edge_id.len(),
        edge_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}
```

### `backend/src/domains/graph/core/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core/models.rs`
- Size bytes / Размер в байтах: `8968`
- Included characters / Включено символов: `8968`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use crate::platform::graph::GraphNodeKind;

use super::errors::GraphStoreError;
use super::validation::{validate_json_object, validate_non_empty};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    PersonHasEmailAddress,
    PersonSentMessage,
    PersonReceivedMessage,
    EmailAddressSentMessage,
    EmailAddressReceivedMessage,
    ProjectHasMessage,
    ProjectHasDocument,
    ProjectInvolvesPerson,
    ProjectInvolvesEmailAddress,
    EntityRelationship,
}

impl RelationshipType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersonHasEmailAddress => "person_has_email_address",
            Self::PersonSentMessage => "person_sent_message",
            Self::PersonReceivedMessage => "person_received_message",
            Self::EmailAddressSentMessage => "email_address_sent_message",
            Self::EmailAddressReceivedMessage => "email_address_received_message",
            Self::ProjectHasMessage => "project_has_message",
            Self::ProjectHasDocument => "project_has_document",
            Self::ProjectInvolvesPerson => "project_involves_person",
            Self::ProjectInvolvesEmailAddress => "project_involves_email_address",
            Self::EntityRelationship => "entity_relationship",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphReviewState {
    SystemAccepted,
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl GraphReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SystemAccepted => "system_accepted",
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEvidenceSourceKind {
    Person,
    Message,
    Document,
    Relationship,
    Decision,
    Obligation,
    Observation,
}

impl GraphEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Person => "contact",
            Self::Message => "message",
            Self::Document => "document",
            Self::Relationship => "relationship",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Observation => "observation",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewGraphNode {
    pub node_kind: GraphNodeKind,
    pub stable_key: String,
    pub label: String,
    pub properties: Value,
}

impl NewGraphNode {
    pub fn new(
        node_kind: GraphNodeKind,
        stable_key: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            node_kind,
            stable_key: stable_key.into(),
            label: label.into(),
            properties: json!({}),
        }
    }

    pub fn properties(mut self, properties: Value) -> Self {
        self.properties = properties;
        self
    }

    pub(super) fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("stable_key", &self.stable_key)?;
        validate_non_empty("label", &self.label)?;
        validate_json_object("node properties", &self.properties)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewGraphEdge {
    pub source_node_id: String,
    pub target_node_id: String,
    pub relationship_type: RelationshipType,
    pub confidence: f64,
    pub review_state: GraphReviewState,
    pub properties: Value,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}

impl NewGraphEdge {
    pub fn new(
        source_node_id: String,
        target_node_id: String,
        relationship_type: RelationshipType,
        confidence: f64,
        review_state: GraphReviewState,
    ) -> Self {
        Self {
            source_node_id,
            target_node_id,
            relationship_type,
            confidence,
            review_state,
            properties: json!({}),
            valid_from: None,
            valid_to: None,
        }
    }

    pub fn properties(mut self, properties: Value) -> Self {
        self.properties = properties;
        self
    }

    pub(super) fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("source_node_id", &self.source_node_id)?;
        validate_non_empty("target_node_id", &self.target_node_id)?;
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(GraphStoreError::InvalidConfidence(self.confidence));
        }
        if self.valid_to.is_some() {
            return Err(GraphStoreError::TemporalEdgesUnsupported);
        }
        validate_json_object("edge properties", &self.properties)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewGraphEvidence {
    pub source_kind: GraphEvidenceSourceKind,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

impl NewGraphEvidence {
    pub fn new(source_kind: GraphEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            observation_id: None,
            excerpt: None,
            metadata: json!({}),
        }
    }

    pub fn observation(observation_id: impl Into<String>) -> Self {
        let observation_id = observation_id.into();
        Self {
            source_kind: GraphEvidenceSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id),
            excerpt: None,
            metadata: json!({}),
        }
    }

    pub fn observation_id(mut self, observation_id: impl Into<String>) -> Self {
        self.observation_id = Some(observation_id.into());
        self
    }

    pub fn excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(super) fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        if let Some(observation_id) = &self.observation_id {
            validate_non_empty("observation_id", observation_id)?;
        }
        if self.source_kind == GraphEvidenceSourceKind::Message && self.observation_id.is_none() {
            return Err(GraphStoreError::MissingObservationEvidence {
                source_kind: self.source_kind.as_str(),
            });
        }
        if self.source_kind == GraphEvidenceSourceKind::Observation
            && self.observation_id.as_deref() != Some(self.source_id.as_str())
        {
            return Err(GraphStoreError::ObservationSourceMismatch);
        }
        validate_json_object("evidence metadata", &self.metadata)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphNode {
    pub node_id: String,
    pub node_kind: GraphNodeKind,
    pub stable_key: String,
    pub label: String,
    pub properties: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEdge {
    pub edge_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub relationship_type: RelationshipType,
    pub confidence: f64,
    pub review_state: GraphReviewState,
    pub properties: Value,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphCount {
    pub key: String,
    pub count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphSummary {
    pub node_counts: Vec<GraphCount>,
    pub edge_counts: Vec<GraphCount>,
    pub evidence_count: i64,
    pub latest_projection_at: Option<DateTime<Utc>>,
    pub is_empty: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEvidenceSummary {
    pub edge_id: String,
    pub source_kind: GraphEvidenceSourceKind,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphNeighborhood {
    pub selected_node: GraphNode,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub evidence: Vec<GraphEvidenceSummary>,
    pub edge_limit: i64,
    pub truncated: bool,
    pub evidence_limit: i64,
    pub evidence_truncated: bool,
}
```

### `backend/src/domains/graph/core/queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core/queries.rs`
- Size bytes / Размер в байтах: `8871`
- Included characters / Включено символов: `8871`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use super::constants::{GRAPH_NEIGHBORHOOD_EDGE_LIMIT, GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT};
use super::errors::GraphStoreError;
use super::models::{GraphNeighborhood, GraphNode, GraphSummary};
use super::row_mapping::{row_to_count, row_to_edge, row_to_evidence_summary, row_to_node};
use super::store::GraphStore;

impl GraphStore {
    pub async fn summary(&self) -> Result<GraphSummary, GraphStoreError> {
        let node_count_rows = sqlx::query(
            r#"
            SELECT node_kind AS key, count(*) AS count
            FROM graph_nodes
            GROUP BY node_kind
            ORDER BY node_kind
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let node_counts = node_count_rows
            .into_iter()
            .map(row_to_count)
            .collect::<Result<Vec<_>, _>>()?;

        let edge_count_rows = sqlx::query(
            r#"
            SELECT relationship_type AS key, count(*) AS count
            FROM graph_edges
            GROUP BY relationship_type
            ORDER BY relationship_type
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let edge_counts = edge_count_rows
            .into_iter()
            .map(row_to_count)
            .collect::<Result<Vec<_>, _>>()?;

        let evidence_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_evidence")
            .fetch_one(&self.pool)
            .await?;
        let latest_projection_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT max(updated_at)
            FROM (
                SELECT updated_at FROM graph_nodes
                UNION ALL
                SELECT updated_at FROM graph_edges
            ) graph_updates
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        let total_nodes = node_counts.iter().map(|count| count.count).sum::<i64>();

        Ok(GraphSummary {
            node_counts,
            edge_counts,
            evidence_count,
            latest_projection_at,
            is_empty: total_nodes == 0,
        })
    }

    pub async fn search_nodes(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<GraphNode>, GraphStoreError> {
        let search_pattern = format!("%{query}%");
        let rows = sqlx::query(
            r#"
            SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
            FROM graph_nodes
            WHERE label ILIKE $1 OR stable_key ILIKE $1
            ORDER BY node_kind, label
            LIMIT $2
            "#,
        )
        .bind(search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_node).collect()
    }

    pub async fn list_nodes_for_picker(
        &self,
        limit: i64,
    ) -> Result<Vec<GraphNode>, GraphStoreError> {
        let rows = sqlx::query(
            r#"
            WITH node_degree AS (
                SELECT node_id, count(*) AS edge_count
                FROM (
                    SELECT source_node_id AS node_id
                    FROM graph_edges
                    WHERE valid_to IS NULL
                    UNION ALL
                    SELECT target_node_id AS node_id
                    FROM graph_edges
                    WHERE valid_to IS NULL
                ) edge_endpoints
                GROUP BY node_id
            )
            SELECT
                graph_nodes.node_id,
                graph_nodes.node_kind,
                graph_nodes.stable_key,
                graph_nodes.label,
                graph_nodes.properties,
                graph_nodes.created_at,
                graph_nodes.updated_at
            FROM graph_nodes
            LEFT JOIN node_degree ON node_degree.node_id = graph_nodes.node_id
            ORDER BY
                coalesce(node_degree.edge_count, 0) DESC,
                graph_nodes.updated_at DESC,
                graph_nodes.label,
                graph_nodes.node_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_node).collect()
    }

    pub async fn neighborhood(
        &self,
        node_id: &str,
    ) -> Result<Option<GraphNeighborhood>, GraphStoreError> {
        let mut transaction = self.pool.begin().await?;
        // Neighborhood assembles several query results; one read-only snapshot keeps edges,
        // neighbor nodes, and evidence mutually consistent while projections commit.
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *transaction)
            .await?;

        let Some(selected_row) = sqlx::query(
            r#"
            SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
            FROM graph_nodes
            WHERE node_id = $1
            "#,
        )
        .bind(node_id)
        .fetch_optional(&mut *transaction)
        .await?
        else {
            transaction.commit().await?;
            return Ok(None);
        };
        let selected_node = row_to_node(selected_row)?;

        let edge_rows = sqlx::query(
            r#"
            SELECT
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence::float8 AS confidence,
                review_state,
                properties,
                valid_from,
                valid_to,
                created_at,
                updated_at
            FROM graph_edges
            WHERE valid_to IS NULL
              AND (source_node_id = $1 OR target_node_id = $1)
            ORDER BY relationship_type, source_node_id, target_node_id
            LIMIT $2
            "#,
        )
        .bind(&selected_node.node_id)
        .bind(GRAPH_NEIGHBORHOOD_EDGE_LIMIT + 1)
        .fetch_all(&mut *transaction)
        .await?;
        let mut edges = edge_rows
            .into_iter()
            .map(row_to_edge)
            .collect::<Result<Vec<_>, _>>()?;
        let truncated = edges.len() > GRAPH_NEIGHBORHOOD_EDGE_LIMIT as usize;
        edges.truncate(GRAPH_NEIGHBORHOOD_EDGE_LIMIT as usize);

        let mut node_ids = BTreeSet::new();
        for edge in &edges {
            if edge.source_node_id != selected_node.node_id {
                node_ids.insert(edge.source_node_id.clone());
            }
            if edge.target_node_id != selected_node.node_id {
                node_ids.insert(edge.target_node_id.clone());
            }
        }
        let node_ids = node_ids.into_iter().collect::<Vec<_>>();
        let nodes = if node_ids.is_empty() {
            Vec::new()
        } else {
            let node_rows = sqlx::query(
                r#"
                SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
                FROM graph_nodes
                WHERE node_id = ANY($1)
                ORDER BY node_kind, label, node_id
                "#,
            )
            .bind(&node_ids)
            .fetch_all(&mut *transaction)
            .await?;

            node_rows
                .into_iter()
                .map(row_to_node)
                .collect::<Result<Vec<_>, _>>()?
        };

        let edge_ids = edges
            .iter()
            .map(|edge| edge.edge_id.clone())
            .collect::<Vec<_>>();
        let (evidence, evidence_truncated) = if edge_ids.is_empty() {
            (Vec::new(), false)
        } else {
            let evidence_rows = sqlx::query(
                r#"
                SELECT edge_id, source_kind, source_id, observation_id, excerpt, metadata
                FROM graph_evidence
                WHERE edge_id = ANY($1)
                ORDER BY edge_id, source_kind, source_id
                LIMIT $2
                "#,
            )
            .bind(&edge_ids)
            .bind(GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT + 1)
            .fetch_all(&mut *transaction)
            .await?;

            let mut evidence = evidence_rows
                .into_iter()
                .map(row_to_evidence_summary)
                .collect::<Result<Vec<_>, _>>()?;
            let evidence_truncated = evidence.len() > GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT as usize;
            evidence.truncate(GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT as usize);
            (evidence, evidence_truncated)
        };

        transaction.commit().await?;

        Ok(Some(GraphNeighborhood {
            selected_node,
            nodes,
            edges,
            evidence,
            edge_limit: GRAPH_NEIGHBORHOOD_EDGE_LIMIT,
            truncated,
            evidence_limit: GRAPH_NEIGHBORHOOD_EVIDENCE_LIMIT,
            evidence_truncated,
        }))
    }
}
```

### `backend/src/domains/graph/core/row_mapping.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core/row_mapping.rs`
- Size bytes / Размер в байтах: `4862`
- Included characters / Включено символов: `4862`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::GraphStoreError;
use super::models::{
    GraphCount, GraphEdge, GraphEvidenceSourceKind, GraphEvidenceSummary, GraphNode, GraphNodeKind,
    GraphReviewState, RelationshipType,
};

pub(super) fn row_to_node(row: PgRow) -> Result<GraphNode, GraphStoreError> {
    Ok(GraphNode {
        node_id: row.try_get("node_id")?,
        node_kind: parse_node_kind(row.try_get("node_kind")?)?,
        stable_key: row.try_get("stable_key")?,
        label: row.try_get("label")?,
        properties: row.try_get("properties")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_edge(row: PgRow) -> Result<GraphEdge, GraphStoreError> {
    Ok(GraphEdge {
        edge_id: row.try_get("edge_id")?,
        source_node_id: row.try_get("source_node_id")?,
        target_node_id: row.try_get("target_node_id")?,
        relationship_type: parse_relationship_type(row.try_get("relationship_type")?)?,
        confidence: row.try_get("confidence")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        properties: row.try_get("properties")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_count(row: PgRow) -> Result<GraphCount, GraphStoreError> {
    Ok(GraphCount {
        key: row.try_get("key")?,
        count: row.try_get("count")?,
    })
}

pub(super) fn row_to_evidence_summary(row: PgRow) -> Result<GraphEvidenceSummary, GraphStoreError> {
    Ok(GraphEvidenceSummary {
        edge_id: row.try_get("edge_id")?,
        source_kind: parse_evidence_source_kind(row.try_get("source_kind")?)?,
        source_id: row.try_get("source_id")?,
        observation_id: row.try_get("observation_id")?,
        excerpt: row.try_get("excerpt")?,
        metadata: row.try_get("metadata")?,
    })
}

fn parse_node_kind(value: String) -> Result<GraphNodeKind, GraphStoreError> {
    match value.as_str() {
        "person" => Ok(GraphNodeKind::Person),
        "email_address" => Ok(GraphNodeKind::EmailAddress),
        "message" => Ok(GraphNodeKind::Message),
        "document" => Ok(GraphNodeKind::Document),
        "project" => Ok(GraphNodeKind::Project),
        "organization" => Ok(GraphNodeKind::Organization),
        "task" => Ok(GraphNodeKind::Task),
        "event" => Ok(GraphNodeKind::Event),
        "decision" => Ok(GraphNodeKind::Decision),
        "obligation" => Ok(GraphNodeKind::Obligation),
        "knowledge" => Ok(GraphNodeKind::Knowledge),
        _ => Err(GraphStoreError::UnknownNodeKind(value)),
    }
}

fn parse_relationship_type(value: String) -> Result<RelationshipType, GraphStoreError> {
    match value.as_str() {
        "person_has_email_address" => Ok(RelationshipType::PersonHasEmailAddress),
        "person_sent_message" => Ok(RelationshipType::PersonSentMessage),
        "person_received_message" => Ok(RelationshipType::PersonReceivedMessage),
        "email_address_sent_message" => Ok(RelationshipType::EmailAddressSentMessage),
        "email_address_received_message" => Ok(RelationshipType::EmailAddressReceivedMessage),
        "project_has_message" => Ok(RelationshipType::ProjectHasMessage),
        "project_has_document" => Ok(RelationshipType::ProjectHasDocument),
        "project_involves_person" => Ok(RelationshipType::ProjectInvolvesPerson),
        "project_involves_email_address" => Ok(RelationshipType::ProjectInvolvesEmailAddress),
        "entity_relationship" => Ok(RelationshipType::EntityRelationship),
        _ => Err(GraphStoreError::UnknownRelationshipType(value)),
    }
}

fn parse_review_state(value: String) -> Result<GraphReviewState, GraphStoreError> {
    match value.as_str() {
        "system_accepted" => Ok(GraphReviewState::SystemAccepted),
        "suggested" => Ok(GraphReviewState::Suggested),
        "user_confirmed" => Ok(GraphReviewState::UserConfirmed),
        "user_rejected" => Ok(GraphReviewState::UserRejected),
        _ => Err(GraphStoreError::UnknownReviewState(value)),
    }
}

fn parse_evidence_source_kind(value: String) -> Result<GraphEvidenceSourceKind, GraphStoreError> {
    match value.as_str() {
        "contact" | "person" => Ok(GraphEvidenceSourceKind::Person),
        "message" => Ok(GraphEvidenceSourceKind::Message),
        "document" => Ok(GraphEvidenceSourceKind::Document),
        "relationship" => Ok(GraphEvidenceSourceKind::Relationship),
        "decision" => Ok(GraphEvidenceSourceKind::Decision),
        "obligation" => Ok(GraphEvidenceSourceKind::Obligation),
        "observation" => Ok(GraphEvidenceSourceKind::Observation),
        _ => Err(GraphStoreError::UnknownEvidenceSourceKind(value)),
    }
}
```

### `backend/src/domains/graph/core/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core/store.rs`
- Size bytes / Размер в байтах: `6151`
- Included characters / Включено символов: `6151`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::GraphStoreError;
use super::ids::{edge_id, evidence_id, node_id};
use super::models::{GraphEdge, GraphNode, NewGraphEdge, NewGraphEvidence, NewGraphNode};
use super::row_mapping::{row_to_edge, row_to_node};
use super::validation::validate_edge_with_evidence;

#[derive(Clone)]
pub struct GraphStore {
    pub(super) pool: PgPool,
}

impl GraphStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_node(&self, node: &NewGraphNode) -> Result<GraphNode, GraphStoreError> {
        node.validate()?;
        let node_id = node_id(node.node_kind, &node.stable_key);
        let row = sqlx::query(
            r#"
            INSERT INTO graph_nodes (node_id, node_kind, stable_key, label, properties)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = EXCLUDED.properties,
                updated_at = now()
            RETURNING node_id, node_kind, stable_key, label, properties, created_at, updated_at
            "#,
        )
        .bind(&node_id)
        .bind(node.node_kind.as_str())
        .bind(&node.stable_key)
        .bind(&node.label)
        .bind(&node.properties)
        .fetch_one(&self.pool)
        .await?;

        row_to_node(row)
    }

    pub async fn upsert_edge_with_evidence(
        &self,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphStoreError> {
        validate_edge_with_evidence(edge, evidence)?;
        let mut transaction = self.pool.begin().await?;
        let stored_edge =
            Self::upsert_edge_with_evidence_in_transaction(&mut transaction, edge, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored_edge)
    }

    pub(crate) async fn upsert_node_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        node: &NewGraphNode,
    ) -> Result<GraphNode, GraphStoreError> {
        node.validate()?;
        let node_id = node_id(node.node_kind, &node.stable_key);
        let row = sqlx::query(
            r#"
            INSERT INTO graph_nodes (node_id, node_kind, stable_key, label, properties)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = EXCLUDED.properties,
                updated_at = now()
            RETURNING node_id, node_kind, stable_key, label, properties, created_at, updated_at
            "#,
        )
        .bind(&node_id)
        .bind(node.node_kind.as_str())
        .bind(&node.stable_key)
        .bind(&node.label)
        .bind(&node.properties)
        .fetch_one(&mut **transaction)
        .await?;

        row_to_node(row)
    }

    pub(crate) async fn upsert_edge_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphStoreError> {
        validate_edge_with_evidence(edge, evidence)?;

        let edge_id = edge_id(
            &edge.source_node_id,
            edge.relationship_type,
            &edge.target_node_id,
        );
        let row = sqlx::query(
            r#"
            INSERT INTO graph_edges (
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence,
                review_state,
                properties,
                valid_from,
                valid_to
            )
            VALUES ($1, $2, $3, $4, CAST($5 AS NUMERIC(5,4)), $6, $7, $8, $9)
            ON CONFLICT (source_node_id, target_node_id, relationship_type) WHERE valid_to IS NULL
            DO UPDATE SET
                confidence = EXCLUDED.confidence,
                review_state = EXCLUDED.review_state,
                properties = EXCLUDED.properties,
                valid_from = EXCLUDED.valid_from,
                valid_to = EXCLUDED.valid_to,
                updated_at = now()
            RETURNING
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence::float8 AS confidence,
                review_state,
                properties,
                valid_from,
                valid_to,
                created_at,
                updated_at
            "#,
        )
        .bind(&edge_id)
        .bind(&edge.source_node_id)
        .bind(&edge.target_node_id)
        .bind(edge.relationship_type.as_str())
        .bind(edge.confidence)
        .bind(edge.review_state.as_str())
        .bind(&edge.properties)
        .bind(edge.valid_from)
        .bind(edge.valid_to)
        .fetch_one(&mut **transaction)
        .await?;

        for item in evidence {
            let evidence_id = evidence_id(&edge_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO graph_evidence (
                    evidence_id,
                    edge_id,
                    source_kind,
                    source_id,
                    observation_id,
                    excerpt,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (edge_id, source_kind, source_id)
                DO UPDATE SET
                    observation_id = COALESCE(EXCLUDED.observation_id, graph_evidence.observation_id),
                    excerpt = EXCLUDED.excerpt,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&edge_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(item.observation_id.as_deref())
            .bind(&item.excerpt)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        row_to_edge(row)
    }
}
```

### `backend/src/domains/graph/core/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/core/validation.rs`
- Size bytes / Размер в байтах: `915`
- Included characters / Включено символов: `915`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::GraphStoreError;
use super::models::{NewGraphEdge, NewGraphEvidence};

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), GraphStoreError> {
    if value.trim().is_empty() {
        return Err(GraphStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_edge_with_evidence(
    edge: &NewGraphEdge,
    evidence: &[NewGraphEvidence],
) -> Result<(), GraphStoreError> {
    edge.validate()?;
    if evidence.is_empty() {
        return Err(GraphStoreError::SystemEdgeRequiresEvidence);
    }
    for item in evidence {
        item.validate()?;
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), GraphStoreError> {
    if !value.is_object() {
        return Err(GraphStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
```

### `backend/src/domains/graph/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/mod.rs`
- Size bytes / Размер в байтах: `29`
- Included characters / Включено символов: `29`
- Truncated / Обрезано: `no`

```rust
pub mod core;
pub mod ports;
```

### `backend/src/domains/graph/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/graph/ports.rs`
- Size bytes / Размер в байтах: `56`
- Included characters / Включено символов: `56`
- Truncated / Обрезано: `no`

```rust
pub use super::core::GraphStore as GraphProjectionPort;
```

### `backend/src/domains/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/mod.rs`
- Size bytes / Размер в байтах: `266`
- Included characters / Включено символов: `266`
- Truncated / Обрезано: `no`

```rust
pub mod calendar;
pub mod communications;
pub mod decisions;
pub mod documents;
pub mod graph;
pub mod obligations;
pub mod organizations;
pub mod persons;
pub mod projects;
pub mod relationships;
pub mod review;
pub mod settings;
pub mod signal_hub;
pub mod tasks;
```

### `backend/src/domains/obligations/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/errors.rs`
- Size bytes / Размер в байтах: `1504`
- Included characters / Включено символов: `1504`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum ObligationStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("obligation evidence is required")]
    MissingEvidence,

    #[error("observation obligation evidence must use the same source_id and observation_id")]
    InvalidObservationEvidenceSource,

    #[error("obligation evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("obligation was not found")]
    ObligationNotFound,

    #[error("beneficiary entity kind and id must be provided together")]
    PartialBeneficiary,

    #[error("unknown obligation entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown obligation evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown obligation status stored in database: {0}")]
    UnknownStatus(String),

    #[error("unknown obligation review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("unknown obligation risk state stored in database: {0}")]
    UnknownRiskState(String),
}
```

### `backend/src/domains/obligations/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/evidence.rs`
- Size bytes / Размер в байтах: `1338`
- Included characters / Включено символов: `1338`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::{
    ObservationStoreError, link_domain_entity_in_transaction,
    materialize_review_transition_link_in_transaction,
};

use super::models::ObligationReviewState;

pub(crate) async fn link_obligation_support_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    obligation_id: impl Into<String>,
    confidence: f64,
    metadata: Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "obligations",
        "obligation",
        obligation_id.into(),
        Some("supports"),
        Some(confidence),
        Some(metadata),
    )
    .await
}

pub(crate) async fn link_obligation_review_transition_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    obligation_id: &str,
    review_state: ObligationReviewState,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    materialize_review_transition_link_in_transaction(
        transaction,
        observation_id,
        "obligations",
        "obligation",
        obligation_id,
        "review_state",
        review_state.as_str(),
        metadata,
    )
    .await
}
```

### `backend/src/domains/obligations/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/ids.rs`
- Size bytes / Размер в байтах: `1393`
- Included characters / Включено символов: `1393`
- Truncated / Обрезано: `no`

```rust
use super::models::{NewObligation, ObligationEntityKind, ObligationEvidenceSourceKind};

pub fn obligation_id(obligation: &NewObligation) -> String {
    let beneficiary_kind = obligation
        .beneficiary_entity_kind
        .map(ObligationEntityKind::as_str)
        .unwrap_or("");
    let beneficiary_id = obligation.beneficiary_entity_id.as_deref().unwrap_or("");
    let statement = normalize_statement(&obligation.statement);

    format!(
        "obligation:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        obligation.obligated_entity_kind.as_str().len(),
        obligation.obligated_entity_kind.as_str(),
        obligation.obligated_entity_id.len(),
        obligation.obligated_entity_id,
        beneficiary_kind.len(),
        beneficiary_kind,
        beneficiary_id.len(),
        beneficiary_id,
        statement.len(),
        statement
    )
}

pub fn evidence_id(
    obligation_id: &str,
    source_kind: ObligationEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "obligation:evidence:v1:{}:{}:{}:{}:{}:{}",
        obligation_id.len(),
        obligation_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}

fn normalize_statement(statement: &str) -> String {
    statement
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
```

### `backend/src/domains/obligations/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/mod.rs`
- Size bytes / Размер в байтах: `625`
- Included characters / Включено символов: `625`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod evidence;
mod ids;
mod models;
pub mod ports;
mod row_mapping;
mod service;
mod store;
mod validation;

pub use errors::ObligationStoreError;
pub use errors::ObligationStoreError as ObligationReviewPortError;
pub use ids::{evidence_id, obligation_id};
pub use models::{
    NewObligation, NewObligationEvidence, Obligation, ObligationEntityKind,
    ObligationEvidenceSourceKind, ObligationReviewState, ObligationRiskState, ObligationStatus,
};
pub use service::{ObligationCommandService, ObligationCommandServiceError};
pub use store::ObligationStore;
pub use store::ObligationStore as ObligationReviewPort;
```

### `backend/src/domains/obligations/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/models.rs`
- Size bytes / Размер в байтах: `375`
- Included characters / Включено символов: `375`
- Truncated / Обрезано: `no`

```rust
mod entity_kind;
mod evidence;
mod obligation;
mod read_model;
mod source_kind;
mod states;

pub use entity_kind::ObligationEntityKind;
pub use evidence::NewObligationEvidence;
pub use obligation::NewObligation;
pub use read_model::Obligation;
pub use source_kind::ObligationEvidenceSourceKind;
pub use states::{ObligationReviewState, ObligationRiskState, ObligationStatus};
```
