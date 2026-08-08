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

- Chunk ID / ID чанка: `051-source-backend-part-031`
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

### `backend/src/domains/projects/core/read_model/stats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model/stats.rs`
- Size bytes / Размер в байтах: `3640`
- Included characters / Включено символов: `3640`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::super::errors::ProjectStoreError;
use super::super::ids::project_graph_node_id;
use super::super::models::ProjectStats;
use super::super::projection::reviewed_target_ids;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_stats(
        &self,
        project_id: &str,
    ) -> Result<ProjectStats, ProjectStoreError> {
        let message_targets = self.active_project_messages(project_id).await?;
        let message_ids = reviewed_target_ids(&message_targets);
        let document_targets = self.active_project_documents(project_id).await?;
        let document_ids = reviewed_target_ids(&document_targets);

        let message_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM communication_messages message
            WHERE message_id = ANY($1)
            "#,
        )
        .bind(&message_ids)
        .fetch_one(&self.pool)
        .await?;

        let document_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM documents document
            WHERE document_id = ANY($1)
            "#,
        )
        .bind(&document_ids)
        .fetch_one(&self.pool)
        .await?;

        let people_count = sqlx::query_scalar::<_, i64>(
            r#"
            WITH project_messages AS (
                SELECT sender, recipients
                FROM communication_messages message
                WHERE message_id = ANY($1)
            ),
            participants AS (
                SELECT lower(trim(sender)) AS email_address
                FROM project_messages
                UNION ALL
                SELECT lower(trim(recipient.value)) AS email_address
                FROM project_messages message,
                     jsonb_array_elements_text(message.recipients) AS recipient(value)
            )
            SELECT count(DISTINCT email_address)
            FROM participants
            WHERE email_address <> ''
            "#,
        )
        .bind(&message_ids)
        .fetch_one(&self.pool)
        .await?;

        let graph_node_id = project_graph_node_id(project_id);
        let graph_connection_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM graph_edges
            WHERE valid_to IS NULL
              AND (source_node_id = $1 OR target_node_id = $1)
            "#,
        )
        .bind(&graph_node_id)
        .fetch_one(&self.pool)
        .await?;

        let latest_activity_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            WITH project_message_activity AS (
                SELECT COALESCE(occurred_at, projected_at) AS occurred_at
                FROM communication_messages message
                WHERE message_id = ANY($1)
            ),
            project_document_activity AS (
                SELECT imported_at AS occurred_at
                FROM documents document
                WHERE document_id = ANY($2)
            )
            SELECT max(occurred_at)
            FROM (
                SELECT occurred_at FROM project_message_activity
                UNION ALL
                SELECT occurred_at FROM project_document_activity
            ) activity
            "#,
        )
        .bind(&message_ids)
        .bind(&document_ids)
        .fetch_one(&self.pool)
        .await?;

        Ok(ProjectStats {
            message_count,
            document_count,
            people_count,
            graph_connection_count,
            latest_activity_at,
        })
    }
}
```

### `backend/src/domains/projects/core/read_model/timeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model/timeline.rs`
- Size bytes / Размер в байтах: `2106`
- Included characters / Включено символов: `2106`
- Truncated / Обрезано: `no`

```rust
use crate::engines::timeline::TimelineEngine;

use super::super::errors::ProjectStoreError;
use super::super::models::ProjectTimelineItem;
use super::super::projection::reviewed_target_ids;
use super::super::rows::row_to_timeline_item;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_timeline(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectTimelineItem>, ProjectStoreError> {
        let limit = TimelineEngine::bounded_entity_limit(limit);
        let message_ids = reviewed_target_ids(&self.active_project_messages(project_id).await?);
        let document_ids = reviewed_target_ids(&self.active_project_documents(project_id).await?);

        let rows = sqlx::query(
            r#"
            WITH project_messages AS (
                SELECT
                    'message' AS item_kind,
                    message_id AS item_id,
                    subject AS title,
                    sender AS subtitle,
                    COALESCE(occurred_at, projected_at) AS occurred_at
                FROM communication_messages message
                WHERE message_id = ANY($1)
            ),
            project_documents AS (
                SELECT
                    'document' AS item_kind,
                    document_id AS item_id,
                    title,
                    document_kind AS subtitle,
                    imported_at AS occurred_at
                FROM documents document
                WHERE document_id = ANY($2)
            )
            SELECT item_kind, item_id, title, subtitle, occurred_at
            FROM (
                SELECT * FROM project_messages
                UNION ALL
                SELECT * FROM project_documents
            ) timeline
            ORDER BY occurred_at DESC, item_kind, item_id
            LIMIT $3
            "#,
        )
        .bind(&message_ids)
        .bind(&document_ids)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_timeline_item).collect()
    }
}
```

### `backend/src/domains/projects/core/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/rows.rs`
- Size bytes / Размер в байтах: `4123`
- Included characters / Включено символов: `4123`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use crate::domains::projects::link_reviews::ProjectLinkReviewState;

use super::errors::ProjectStoreError;
use super::models::{
    Project, ProjectDocumentSummary, ProjectMatchedDocument, ProjectMatchedMessage,
    ProjectMessageSummary, ProjectPersonSummary, ProjectTimelineItem,
};

pub(super) fn row_to_project(row: PgRow) -> Result<Project, ProjectStoreError> {
    Ok(Project {
        project_id: row.try_get("project_id")?,
        name: row.try_get("name")?,
        kind: row.try_get("kind")?,
        status: row.try_get("status")?,
        description: row.try_get("description")?,
        owner_display_name: row.try_get("owner_display_name")?,
        progress_percent: row.try_get("progress_percent")?,
        start_date: row.try_get("start_date")?,
        target_date: row.try_get("target_date")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_project_message(
    row: PgRow,
) -> Result<ProjectMessageSummary, ProjectStoreError> {
    Ok(ProjectMessageSummary {
        message_id: row.try_get("message_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

pub(super) fn row_to_project_document(
    row: PgRow,
) -> Result<ProjectDocumentSummary, ProjectStoreError> {
    Ok(ProjectDocumentSummary {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        observation_id: row.try_get("observation_id")?,
        imported_at: row.try_get("imported_at")?,
    })
}

pub(super) fn row_to_project_person(row: PgRow) -> Result<ProjectPersonSummary, ProjectStoreError> {
    Ok(ProjectPersonSummary {
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        interaction_count: row.try_get("interaction_count")?,
        last_interaction_at: row.try_get("last_interaction_at")?,
    })
}

pub(super) fn row_to_timeline_item(row: PgRow) -> Result<ProjectTimelineItem, ProjectStoreError> {
    Ok(ProjectTimelineItem {
        item_kind: row.try_get("item_kind")?,
        item_id: row.try_get("item_id")?,
        title: row.try_get("title")?,
        subtitle: row.try_get("subtitle")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

pub(super) fn row_to_matched_message(
    row: PgRow,
) -> Result<ProjectMatchedMessage, ProjectStoreError> {
    Ok(ProjectMatchedMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        observation_id: row.try_get("observation_id")?,
        account_id: row.try_get("account_id")?,
        provider_record_id: row.try_get("provider_record_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        recipients: recipients_from_value(row.try_get("recipients")?)?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        review_state: ProjectLinkReviewState::Suggested,
    })
}

pub(super) fn row_to_matched_document(
    row: PgRow,
) -> Result<ProjectMatchedDocument, ProjectStoreError> {
    Ok(ProjectMatchedDocument {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        observation_id: row.try_get("observation_id")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        imported_at: row.try_get("imported_at")?,
        review_state: ProjectLinkReviewState::Suggested,
    })
}

fn recipients_from_value(value: serde_json::Value) -> Result<Vec<String>, ProjectStoreError> {
    let Some(values) = value.as_array() else {
        return Err(ProjectStoreError::InvalidRecipients);
    };

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(ProjectStoreError::InvalidRecipients)
        })
        .collect()
}
```

### `backend/src/domains/projects/core/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/store.rs`
- Size bytes / Размер в байтах: `5363`
- Included characters / Включено символов: `5363`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::constants::{DEFAULT_PROJECT_LIMIT, PROJECT_DETAIL_ITEM_LIMIT};
use super::errors::ProjectStoreError;
use super::ids::project_graph_node_id;
use super::models::{NewProject, Project, ProjectDetail, ProjectSummary};
use super::rows::row_to_project;
use super::validation::{validate_limit, validate_non_empty};

#[derive(Clone)]
pub struct ProjectStore {
    pub(super) pool: PgPool,
}

impl ProjectStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_project(&self, project: &NewProject) -> Result<Project, ProjectStoreError> {
        let project = project.validate()?;
        let mut transaction = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO projects (
                project_id,
                name,
                kind,
                status,
                description,
                owner_display_name,
                progress_percent,
                start_date,
                target_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (project_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                kind = EXCLUDED.kind,
                status = EXCLUDED.status,
                description = EXCLUDED.description,
                owner_display_name = EXCLUDED.owner_display_name,
                progress_percent = EXCLUDED.progress_percent,
                start_date = EXCLUDED.start_date,
                target_date = EXCLUDED.target_date,
                updated_at = now()
            RETURNING
                project_id,
                name,
                kind,
                status,
                description,
                owner_display_name,
                progress_percent,
                start_date,
                target_date,
                created_at,
                updated_at
            "#,
        )
        .bind(&project.project_id)
        .bind(&project.name)
        .bind(&project.kind)
        .bind(&project.status)
        .bind(&project.description)
        .bind(&project.owner_display_name)
        .bind(project.progress_percent)
        .bind(project.start_date)
        .bind(project.target_date)
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query("DELETE FROM project_keywords WHERE project_id = $1")
            .bind(&project.project_id)
            .execute(&mut *transaction)
            .await?;

        for keyword in &project.keywords {
            sqlx::query(
                r#"
                INSERT INTO project_keywords (project_id, keyword)
                VALUES ($1, $2)
                ON CONFLICT (project_id, keyword) DO NOTHING
                "#,
            )
            .bind(&project.project_id)
            .bind(keyword)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        row_to_project(row)
    }

    pub async fn list_projects(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<ProjectSummary>, ProjectStoreError> {
        let limit = validate_limit(limit.unwrap_or(DEFAULT_PROJECT_LIMIT))?;
        let rows = sqlx::query(
            r#"
            SELECT
                project_id,
                name,
                kind,
                status,
                description,
                owner_display_name,
                progress_percent,
                start_date,
                target_date,
                created_at,
                updated_at
            FROM projects
            ORDER BY updated_at DESC, name, project_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let projects = rows
            .into_iter()
            .map(row_to_project)
            .collect::<Result<Vec<_>, _>>()?;
        let mut summaries = Vec::with_capacity(projects.len());
        for project in projects {
            summaries.push(ProjectSummary {
                graph_node_id: project_graph_node_id(&project.project_id),
                stats: self.project_stats(&project.project_id).await?,
                project,
            });
        }

        Ok(summaries)
    }

    pub async fn project_detail(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectDetail>, ProjectStoreError> {
        let project_id = validate_non_empty("project_id", project_id)?;
        let Some(project) = self.project_by_id(&project_id).await? else {
            return Ok(None);
        };

        Ok(Some(ProjectDetail {
            graph_node_id: project_graph_node_id(&project.project_id),
            stats: self.project_stats(&project.project_id).await?,
            timeline: self
                .project_timeline(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            key_people: self
                .project_people(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            recent_messages: self
                .project_messages(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            documents: self
                .project_documents(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            project,
        }))
    }
}
```

### `backend/src/domains/projects/core/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/validation.rs`
- Size bytes / Размер в байтах: `558`
- Included characters / Включено символов: `558`
- Truncated / Обрезано: `no`

```rust
use super::constants::MAX_PROJECT_LIMIT;
use super::errors::ProjectStoreError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<String, ProjectStoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProjectStoreError::EmptyField(field_name));
    }

    Ok(trimmed.to_owned())
}

pub(super) fn validate_limit(limit: i64) -> Result<i64, ProjectStoreError> {
    if limit <= 0 {
        return Err(ProjectStoreError::InvalidLimit);
    }

    Ok(limit.min(MAX_PROJECT_LIMIT))
}
```

### `backend/src/domains/projects/link_reviews.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews.rs`
- Size bytes / Размер в байтах: `530`
- Included characters / Включено символов: `530`
- Truncated / Обрезано: `no`

```rust
mod adapters;
mod constants;
mod errors;
mod events;
mod models;
mod rows;
mod service;
mod store;
mod target_checks;
mod validation;

pub use errors::ProjectLinkReviewError;
pub use models::{
    ProjectLinkReview, ProjectLinkReviewCommand, ProjectLinkReviewCommandResult,
    ProjectLinkReviewState, ProjectLinkTargetKind, ProjectReviewedTarget,
};
pub use service::{ProjectLinkReviewService, ProjectLinkReviewServiceError};
pub use store::ProjectLinkReviewStore;
pub use store::ProjectLinkReviewStore as ProjectLinkReviewPort;
```

### `backend/src/domains/projects/link_reviews/adapters.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/adapters.rs`
- Size bytes / Размер в байтах: `2445`
- Included characters / Включено символов: `2445`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use super::errors::ProjectLinkReviewError;
use super::models::{ProjectLinkReviewState, ReviewEventApplication};
use super::store::ProjectLinkReviewStore;

impl ProjectLinkReviewStore {
    pub(crate) async fn apply_review_event_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        application: ReviewEventApplication<'_>,
    ) -> Result<(), ProjectLinkReviewError> {
        match application.review_state {
            ProjectLinkReviewState::Suggested => {
                sqlx::query(
                    r#"
                    DELETE FROM project_link_reviews
                    WHERE project_id = $1
                      AND target_kind = $2
                      AND target_id = $3
                    "#,
                )
                .bind(application.project_id)
                .bind(application.target_kind.as_str())
                .bind(application.target_id)
                .execute(&mut **transaction)
                .await?;
            }
            ProjectLinkReviewState::UserConfirmed | ProjectLinkReviewState::UserRejected => {
                sqlx::query(
                    r#"
                    INSERT INTO project_link_reviews (
                        project_id,
                        target_kind,
                        target_id,
                        review_state,
                        event_id,
                        actor_id,
                        reviewed_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (project_id, target_kind, target_id)
                    DO UPDATE SET
                        review_state = EXCLUDED.review_state,
                        event_id = EXCLUDED.event_id,
                        actor_id = EXCLUDED.actor_id,
                        reviewed_at = EXCLUDED.reviewed_at,
                        updated_at = now()
                    "#,
                )
                .bind(application.project_id)
                .bind(application.target_kind.as_str())
                .bind(application.target_id)
                .bind(application.review_state.as_str())
                .bind(application.event_id)
                .bind(application.actor_id)
                .bind(application.reviewed_at)
                .execute(&mut **transaction)
                .await?;
            }
        }

        Ok(())
    }
}
```

### `backend/src/domains/projects/link_reviews/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/constants.rs`
- Size bytes / Размер в байтах: `247`
- Included characters / Включено символов: `247`
- Truncated / Обрезано: `no`

```rust
pub(crate) const PROJECT_LINK_REVIEW_EVENT_TYPE: &str = "project.link_review_state_changed";
pub(crate) const PROJECT_LINK_REVIEW_SOURCE_KIND: &str = "project_link_review";
pub(crate) const PROJECT_LINK_REVIEW_SOURCE_PROVIDER: &str = "local_api";
```

### `backend/src/domains/projects/link_reviews/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/errors.rs`
- Size bytes / Размер в байтах: `1187`
- Included characters / Включено символов: `1187`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum ProjectLinkReviewError {
    #[error("project_id does not exist")]
    ProjectNotFound,

    #[error("project link target does not exist")]
    TargetNotFound,

    #[error("target_kind must be one of message or document")]
    InvalidTargetKind(String),

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("field missing from payload: {0}")]
    MissingPayloadField(String),

    #[error("field must be a string: {0}")]
    InvalidPayload(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
```

### `backend/src/domains/projects/link_reviews/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/events.rs`
- Size bytes / Размер в байтах: `2975`
- Included characters / Включено символов: `2975`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};

use crate::platform::events::NewEventEnvelope;

use super::constants::{
    PROJECT_LINK_REVIEW_EVENT_TYPE, PROJECT_LINK_REVIEW_SOURCE_KIND,
    PROJECT_LINK_REVIEW_SOURCE_PROVIDER,
};
use super::errors::ProjectLinkReviewError;
use super::models::{ProjectLinkReviewCommand, ProjectLinkReviewState, ProjectLinkTargetKind};
use super::validation::validate_non_empty;

impl ProjectLinkReviewCommand {
    pub(crate) fn to_review_event(
        &self,
        event_id: &str,
    ) -> Result<NewEventEnvelope, ProjectLinkReviewError> {
        Ok(NewEventEnvelope::builder(
            event_id,
            PROJECT_LINK_REVIEW_EVENT_TYPE,
            Utc::now(),
            json!({
                "kind": PROJECT_LINK_REVIEW_SOURCE_KIND,
                "provider": PROJECT_LINK_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "project_link_review",
                "project_id": self.project_id,
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "project_id": self.project_id,
            "target_kind": self.target_kind.as_str(),
            "target_id": self.target_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewEvent {
    pub(crate) project_id: String,
    pub(crate) target_kind: ProjectLinkTargetKind,
    pub(crate) target_id: String,
    pub(crate) review_state: ProjectLinkReviewState,
}

impl ReviewEvent {
    pub(crate) fn from_payload(payload: &Value) -> Result<Self, ProjectLinkReviewError> {
        let payload = as_object(payload)?;
        Ok(Self {
            project_id: required_payload_string(payload, "project_id")?,
            target_kind: ProjectLinkTargetKind::parse(required_payload_string(
                payload,
                "target_kind",
            )?)?,
            target_id: required_payload_string(payload, "target_id")?,
            review_state: ProjectLinkReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, ProjectLinkReviewError> {
    value
        .as_object()
        .ok_or_else(|| ProjectLinkReviewError::InvalidPayload("payload".to_owned()))
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, ProjectLinkReviewError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| ProjectLinkReviewError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| ProjectLinkReviewError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}
```

### `backend/src/domains/projects/link_reviews/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/models.rs`
- Size bytes / Размер в байтах: `2941`
- Included characters / Включено символов: `2941`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::errors::ProjectLinkReviewError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkTargetKind {
    Message,
    Document,
}

impl ProjectLinkTargetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Document => "document",
        }
    }

    pub(crate) fn parse(value: impl AsRef<str>) -> Result<Self, ProjectLinkReviewError> {
        match value.as_ref() {
            "message" => Ok(Self::Message),
            "document" => Ok(Self::Document),
            _ => Err(ProjectLinkReviewError::InvalidTargetKind(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLinkReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ProjectLinkReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub(crate) fn parse(value: impl AsRef<str>) -> Result<Self, ProjectLinkReviewError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ProjectLinkReviewError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReviewCommand {
    pub command_id: String,
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReviewCommandResult {
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLinkReview {
    pub project_id: String,
    pub target_kind: ProjectLinkTargetKind,
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
    pub event_id: String,
    pub actor_id: String,
    pub reviewed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectReviewedTarget {
    pub target_id: String,
    pub review_state: ProjectLinkReviewState,
}

pub(crate) struct ReviewEventApplication<'a> {
    pub(crate) target_kind: ProjectLinkTargetKind,
    pub(crate) project_id: &'a str,
    pub(crate) target_id: &'a str,
    pub(crate) review_state: ProjectLinkReviewState,
    pub(crate) event_id: &'a str,
    pub(crate) actor_id: &'a str,
    pub(crate) reviewed_at: DateTime<Utc>,
}
```

### `backend/src/domains/projects/link_reviews/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/rows.rs`
- Size bytes / Размер в байтах: `1250`
- Included characters / Включено символов: `1250`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::ProjectLinkReviewError;
use super::models::{
    ProjectLinkReview, ProjectLinkReviewState, ProjectLinkTargetKind, ProjectReviewedTarget,
};

pub(crate) fn row_to_project_link_review(
    row: PgRow,
) -> Result<ProjectLinkReview, ProjectLinkReviewError> {
    let target_kind = ProjectLinkTargetKind::parse(row.try_get::<String, _>("target_kind")?)?;
    let review_state = ProjectLinkReviewState::parse(row.try_get::<String, _>("review_state")?)?;
    Ok(ProjectLinkReview {
        project_id: row.try_get("project_id")?,
        target_kind,
        target_id: row.try_get("target_id")?,
        review_state,
        event_id: row.try_get("event_id")?,
        actor_id: row.try_get("actor_id")?,
        reviewed_at: row.try_get("reviewed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(crate) fn row_to_project_reviewed_target(
    row: PgRow,
) -> Result<ProjectReviewedTarget, ProjectLinkReviewError> {
    let review_state = ProjectLinkReviewState::parse(row.try_get::<String, _>("review_state")?)?;

    Ok(ProjectReviewedTarget {
        target_id: row.try_get("target_id")?,
        review_state,
    })
}
```

### `backend/src/domains/projects/link_reviews/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/service.rs`
- Size bytes / Размер в байтах: `2523`
- Included characters / Включено символов: `2523`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{
    ProjectLinkReviewCommand, ProjectLinkReviewCommandResult, ProjectLinkReviewError,
    ProjectLinkReviewStore,
};

#[derive(Clone)]
pub struct ProjectLinkReviewService {
    pool: PgPool,
}

impl ProjectLinkReviewService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        command: &ProjectLinkReviewCommand,
    ) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "project_id": command.project_id,
                        "target_kind": command.target_kind.as_str(),
                        "target_id": command.target_id,
                        "review_state": command.review_state.as_str(),
                        "event_id": format!("project_link_review:{}", command.command_id),
                        "operation": "project_link_review",
                    }),
                    format!(
                        "project://{}/link-review/{}/{}",
                        command.project_id,
                        command.target_kind.as_str(),
                        command.target_id
                    ),
                )
                .provenance(json!({
                    "captured_by": "projects.link_review_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(ProjectLinkReviewStore::new(self.pool.clone())
            .set_review_state_with_observation(
                command,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "projects.link_review_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum ProjectLinkReviewServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    ProjectLinkReview(#[from] ProjectLinkReviewError),
}
```

### `backend/src/domains/projects/link_reviews/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/store.rs`
- Size bytes / Размер в байтах: `10449`
- Included characters / Включено символов: `10449`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use crate::platform::events::{EventEnvelope, EventStore};
use crate::platform::observations::materialize_review_transition_link_in_transaction;

use super::constants::PROJECT_LINK_REVIEW_EVENT_TYPE;
use super::errors::ProjectLinkReviewError;
use super::events::ReviewEvent;
use super::models::{
    ProjectLinkReview, ProjectLinkReviewCommand, ProjectLinkReviewCommandResult,
    ProjectLinkReviewState, ProjectLinkTargetKind, ProjectReviewedTarget, ReviewEventApplication,
};
use super::rows::{row_to_project_link_review, row_to_project_reviewed_target};
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct ProjectLinkReviewStore {
    pool: PgPool,
}

impl ProjectLinkReviewStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn set_review_state(
        &self,
        command: &ProjectLinkReviewCommand,
    ) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewError> {
        self.set_review_state_with_observation(command, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        command: &ProjectLinkReviewCommand,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let project_id = validate_non_empty("project_id", &command.project_id)?;
        let target_id = validate_non_empty("target_id", &command.target_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

        let mut transaction = self.pool.begin().await?;

        self.ensure_project_exists(&mut transaction, &project_id)
            .await?;
        self.ensure_target_exists(&mut transaction, command.target_kind, &target_id)
            .await?;

        let event_id = format!("project_link_review:{command_id}");
        let event = ProjectLinkReviewCommand {
            command_id,
            project_id: project_id.clone(),
            target_kind: command.target_kind,
            target_id: target_id.clone(),
            review_state: command.review_state,
            actor_id: actor_id.clone(),
        }
        .to_review_event(&event_id)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        self.apply_review_event_in_transaction(
            &mut transaction,
            ReviewEventApplication {
                target_kind: command.target_kind,
                project_id: &project_id,
                target_id: &target_id,
                review_state: command.review_state,
                event_id: &event.event_id,
                actor_id: &actor_id,
                reviewed_at: event.occurred_at,
            },
        )
        .await?;
        let metadata = match metadata {
            Some(extra) => Some(json!({
                "event_id": event_id,
                "context": extra,
            })),
            None => Some(json!({
                "event_id": event_id,
            })),
        };
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "projects",
            "project_link_review",
            &event_id,
            "review_state",
            command.review_state.as_str(),
            metadata,
        )
        .await?;

        transaction.commit().await?;

        Ok(ProjectLinkReviewCommandResult {
            project_id,
            target_kind: command.target_kind,
            target_id,
            review_state: command.review_state,
            event_id,
        })
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), ProjectLinkReviewError> {
        let parsed = ReviewEvent::from_payload(&event.payload)?;
        if event.event_type != PROJECT_LINK_REVIEW_EVENT_TYPE {
            return Err(ProjectLinkReviewError::InvalidEventType);
        }

        let actor_id = event
            .actor
            .as_ref()
            .and_then(|value| value.get("actor_id"))
            .and_then(Value::as_str)
            .ok_or(ProjectLinkReviewError::MissingActorId)?;
        let actor_id = validate_non_empty("actor_id", actor_id)?;
        let mut transaction = self.pool.begin().await?;

        self.ensure_project_exists(&mut transaction, &parsed.project_id)
            .await?;
        self.ensure_target_exists(&mut transaction, parsed.target_kind, &parsed.target_id)
            .await?;
        self.apply_review_event_in_transaction(
            &mut transaction,
            ReviewEventApplication {
                target_kind: parsed.target_kind,
                project_id: &parsed.project_id,
                target_id: &parsed.target_id,
                review_state: parsed.review_state,
                event_id: &event.event_id,
                actor_id: &actor_id,
                reviewed_at: event.occurred_at,
            },
        )
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn explicit_review(
        &self,
        project_id: &str,
        target_kind: ProjectLinkTargetKind,
        target_id: &str,
    ) -> Result<Option<ProjectLinkReview>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;
        let target_id = validate_non_empty("target_id", target_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                project_id,
                target_kind,
                target_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
                created_at,
                updated_at
            FROM project_link_reviews
            WHERE project_id = $1 AND target_kind = $2 AND target_id = $3
            "#,
        )
        .bind(&project_id)
        .bind(target_kind.as_str())
        .bind(&target_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_project_link_review).transpose()
    }

    pub async fn active_message_ids_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;

        let rows = sqlx::query(
            r#"
            WITH keyword_matches AS (
                SELECT message_id AS target_id
                FROM communication_messages message
                WHERE EXISTS (
                    SELECT 1
                    FROM project_keywords keyword
                    WHERE keyword.project_id = $1
                      AND (
                          position(lower(keyword.keyword) in lower(message.subject)) > 0
                          OR position(lower(keyword.keyword) in lower(message.body_text)) > 0
                      )
                )
            ),
            confirmed AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'message'
                  AND review_state = 'user_confirmed'
            ),
            rejected AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'message'
                  AND review_state = 'user_rejected'
            ),
            active AS (
                SELECT target_id, 'suggested' AS review_state FROM keyword_matches
                UNION ALL
                SELECT target_id, 'user_confirmed' AS review_state FROM confirmed
            )
            SELECT active.target_id, max(active.review_state) AS review_state
            FROM active
            WHERE NOT EXISTS (
                SELECT 1
                FROM rejected
                WHERE rejected.target_id = active.target_id
            )
            GROUP BY active.target_id
            ORDER BY active.target_id
            "#,
        )
        .bind(&project_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_project_reviewed_target)
            .collect()
    }

    pub async fn active_document_ids_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;

        let rows = sqlx::query(
            r#"
            WITH keyword_matches AS (
                SELECT document_id AS target_id
                FROM documents document
                WHERE EXISTS (
                    SELECT 1
                    FROM project_keywords keyword
                    WHERE keyword.project_id = $1
                      AND (
                          position(lower(keyword.keyword) in lower(document.title)) > 0
                          OR position(lower(keyword.keyword) in lower(document.extracted_text)) > 0
                      )
                )
            ),
            confirmed AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'document'
                  AND review_state = 'user_confirmed'
            ),
            rejected AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'document'
                  AND review_state = 'user_rejected'
            ),
            active AS (
                SELECT target_id, 'suggested' AS review_state FROM keyword_matches
                UNION ALL
                SELECT target_id, 'user_confirmed' AS review_state FROM confirmed
            )
            SELECT active.target_id, max(active.review_state) AS review_state
            FROM active
            WHERE NOT EXISTS (
                SELECT 1
                FROM rejected
                WHERE rejected.target_id = active.target_id
            )
            GROUP BY active.target_id
            ORDER BY active.target_id
            "#,
        )
        .bind(&project_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_project_reviewed_target)
            .collect()
    }
}
```

### `backend/src/domains/projects/link_reviews/target_checks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/target_checks.rs`
- Size bytes / Размер в байтах: `1790`
- Included characters / Включено символов: `1790`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use super::errors::ProjectLinkReviewError;
use super::models::ProjectLinkTargetKind;
use super::store::ProjectLinkReviewStore;

impl ProjectLinkReviewStore {
    pub(crate) async fn ensure_project_exists(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_id: &str,
    ) -> Result<(), ProjectLinkReviewError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM projects WHERE project_id = $1)",
        )
        .bind(project_id)
        .fetch_one(&mut **transaction)
        .await?;

        if !exists {
            return Err(ProjectLinkReviewError::ProjectNotFound);
        }

        Ok(())
    }

    pub(crate) async fn ensure_target_exists(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        target_kind: ProjectLinkTargetKind,
        target_id: &str,
    ) -> Result<(), ProjectLinkReviewError> {
        let exists =
            match target_kind {
                ProjectLinkTargetKind::Message => sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS (SELECT 1 FROM communication_messages WHERE message_id = $1)",
                )
                .bind(target_id)
                .fetch_one(&mut **transaction)
                .await?,
                ProjectLinkTargetKind::Document => {
                    sqlx::query_scalar::<_, bool>(
                        "SELECT EXISTS (SELECT 1 FROM documents WHERE document_id = $1)",
                    )
                    .bind(target_id)
                    .fetch_one(&mut **transaction)
                    .await?
                }
            };

        if !exists {
            return Err(ProjectLinkReviewError::TargetNotFound);
        }

        Ok(())
    }
}
```

### `backend/src/domains/projects/link_reviews/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/link_reviews/validation.rs`
- Size bytes / Размер в байтах: `337`
- Included characters / Включено символов: `337`
- Truncated / Обрезано: `no`

```rust
use super::errors::ProjectLinkReviewError;

pub(crate) fn validate_non_empty(
    field: &str,
    value: &str,
) -> Result<String, ProjectLinkReviewError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ProjectLinkReviewError::EmptyField(field.to_owned()));
    }

    Ok(normalized.to_owned())
}
```

### `backend/src/domains/projects/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/mod.rs`
- Size bytes / Размер в байтах: `51`
- Included characters / Включено символов: `51`
- Truncated / Обрезано: `no`

```rust
pub mod core;
pub mod link_reviews;
pub mod ports;
```

### `backend/src/domains/projects/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/ports.rs`
- Size bytes / Размер в байтах: `57`
- Included characters / Включено символов: `57`
- Truncated / Обрезано: `no`

```rust
pub use super::core::ProjectStore as ProjectCommandPort;
```

### `backend/src/domains/relationships/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/errors.rs`
- Size bytes / Размер в байтах: `1524`
- Included characters / Включено символов: `1524`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::graph::core::GraphStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum RelationshipStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Graph(#[from] GraphStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("relationship evidence is required")]
    MissingEvidence,

    #[error("observation relationship evidence must use the same source_id and observation_id")]
    InvalidObservationEvidenceSource,

    #[error("relationship evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("relationship was not found")]
    RelationshipNotFound,

    #[error("relationship endpoints must be distinct")]
    IdenticalEndpoints,

    #[error("relationship valid_to must not be earlier than valid_from")]
    InvalidTemporalRange,

    #[error("unknown relationship entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown relationship evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown relationship review state stored in database: {0}")]
    UnknownReviewState(String),
}
```

### `backend/src/domains/relationships/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/evidence.rs`
- Size bytes / Размер в байтах: `742`
- Included characters / Включено символов: `742`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::{ObservationStoreError, link_domain_entity_in_transaction};

pub(crate) async fn link_relationship_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "relationships",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        confidence,
        metadata,
    )
    .await
}
```

### `backend/src/domains/relationships/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/ids.rs`
- Size bytes / Размер в байтах: `1082`
- Included characters / Включено символов: `1082`
- Truncated / Обрезано: `no`

```rust
use super::models::{RelationshipEntityKind, RelationshipEvidenceSourceKind};

pub fn relationship_id(
    source_entity_kind: RelationshipEntityKind,
    source_entity_id: &str,
    relationship_type: &str,
    target_entity_kind: RelationshipEntityKind,
    target_entity_id: &str,
) -> String {
    format!(
        "relationship:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        source_entity_kind.as_str().len(),
        source_entity_kind.as_str(),
        source_entity_id.len(),
        source_entity_id,
        relationship_type.len(),
        relationship_type,
        target_entity_kind.as_str().len(),
        target_entity_kind.as_str(),
        target_entity_id.len(),
        target_entity_id
    )
}

pub fn evidence_id(
    relationship_id: &str,
    source_kind: RelationshipEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "relationship:evidence:v1:{}:{}:{}:{}:{}:{}",
        relationship_id.len(),
        relationship_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}
```

### `backend/src/domains/relationships/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/mod.rs`
- Size bytes / Размер в байтах: `616`
- Included characters / Включено символов: `616`
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

pub use errors::RelationshipStoreError;
pub use errors::RelationshipStoreError as RelationshipReviewPortError;
pub use ids::{evidence_id, relationship_id};
pub use models::{
    NewRelationship, NewRelationshipEvidence, Relationship, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState,
};
pub use service::{RelationshipCommandService, RelationshipCommandServiceError};
pub use store::RelationshipStore;
pub use store::RelationshipStore as RelationshipReviewPort;
```

### `backend/src/domains/relationships/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/models.rs`
- Size bytes / Размер в байтах: `6946`
- Included characters / Включено символов: `6946`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::RelationshipStoreError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipEntityKind {
    Persona,
    Organization,
    Project,
    Communication,
    Document,
    Task,
    Event,
    Decision,
    Obligation,
    Knowledge,
}

impl RelationshipEntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Persona => "persona",
            Self::Organization => "organization",
            Self::Project => "project",
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Task => "task",
            Self::Event => "event",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Knowledge => "knowledge",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, RelationshipStoreError> {
        let value = value.as_ref().trim();
        match value {
            "persona" => Ok(Self::Persona),
            "organization" => Ok(Self::Organization),
            "project" => Ok(Self::Project),
            "communication" => Ok(Self::Communication),
            "document" => Ok(Self::Document),
            "task" => Ok(Self::Task),
            "event" => Ok(Self::Event),
            "decision" => Ok(Self::Decision),
            "obligation" => Ok(Self::Obligation),
            "knowledge" => Ok(Self::Knowledge),
            _ => Err(RelationshipStoreError::UnknownEntityKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipEvidenceSourceKind {
    Observation,
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Project,
    Organization,
    Persona,
}

impl RelationshipEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Event => "event",
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Task => "task",
            Self::Project => "project",
            Self::Organization => "organization",
            Self::Persona => "persona",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipReviewState {
    Suggested,
    SystemAccepted,
    UserConfirmed,
    UserRejected,
}

impl RelationshipReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::SystemAccepted => "system_accepted",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, RelationshipStoreError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "system_accepted" => Ok(Self::SystemAccepted),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(RelationshipStoreError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewRelationship {
    pub source_entity_kind: RelationshipEntityKind,
    pub source_entity_id: String,
    pub target_entity_kind: RelationshipEntityKind,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub trust_score: f64,
    pub strength_score: f64,
    pub confidence: f64,
    pub review_state: RelationshipReviewState,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewRelationship {
    pub fn between_personas(
        source_persona_id: impl Into<String>,
        target_persona_id: impl Into<String>,
        relationship_type: impl Into<String>,
        trust_score: f64,
        strength_score: f64,
        confidence: f64,
        review_state: RelationshipReviewState,
    ) -> Self {
        Self {
            source_entity_kind: RelationshipEntityKind::Persona,
            source_entity_id: source_persona_id.into(),
            target_entity_kind: RelationshipEntityKind::Persona,
            target_entity_id: target_persona_id.into(),
            relationship_type: relationship_type.into(),
            trust_score,
            strength_score,
            confidence,
            review_state,
            valid_from: None,
            valid_to: None,
            metadata: json!({}),
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewRelationshipEvidence {
    pub source_kind: RelationshipEvidenceSourceKind,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

impl NewRelationshipEvidence {
    pub fn new(source_kind: RelationshipEvidenceSourceKind, source_id: impl Into<String>) -> Self {
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
            source_kind: RelationshipEvidenceSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id),
            excerpt: None,
            metadata: json!({}),
        }
    }

    pub fn excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Relationship {
    pub relationship_id: String,
    pub source_entity_kind: RelationshipEntityKind,
    pub source_entity_id: String,
    pub target_entity_kind: RelationshipEntityKind,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub trust_score: f64,
    pub strength_score: f64,
    pub confidence: f64,
    pub review_state: RelationshipReviewState,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### `backend/src/domains/relationships/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/ports.rs`
- Size bytes / Размер в байтах: `67`
- Included characters / Включено символов: `67`
- Truncated / Обрезано: `no`

```rust
pub use super::store::RelationshipStore as RelationshipReviewPort;
```

### `backend/src/domains/relationships/row_mapping.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/row_mapping.rs`
- Size bytes / Размер в байтах: `1448`
- Included characters / Включено символов: `1448`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::RelationshipStoreError;
use super::models::{Relationship, RelationshipEntityKind, RelationshipReviewState};

pub(super) fn row_to_relationship(row: PgRow) -> Result<Relationship, RelationshipStoreError> {
    Ok(Relationship {
        relationship_id: row.try_get("relationship_id")?,
        source_entity_kind: parse_entity_kind(row.try_get("source_entity_kind")?)?,
        source_entity_id: row.try_get("source_entity_id")?,
        target_entity_kind: parse_entity_kind(row.try_get("target_entity_kind")?)?,
        target_entity_id: row.try_get("target_entity_id")?,
        relationship_type: row.try_get("relationship_type")?,
        trust_score: row.try_get("trust_score")?,
        strength_score: row.try_get("strength_score")?,
        confidence: row.try_get("confidence")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<RelationshipEntityKind, RelationshipStoreError> {
    RelationshipEntityKind::parse(value)
}

fn parse_review_state(value: String) -> Result<RelationshipReviewState, RelationshipStoreError> {
    RelationshipReviewState::parse(value)
}
```
