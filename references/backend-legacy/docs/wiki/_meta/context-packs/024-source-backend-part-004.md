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

- Chunk ID / ID чанка: `024-source-backend-part-004`
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

### `backend/src/app/api_support/query_parsing.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/query_parsing.rs`
- Size bytes / Размер в байтах: `251`
- Included characters / Включено символов: `251`
- Truncated / Обрезано: `no`

```rust
mod communication;
mod documents;
mod graph;
mod persons;
mod projects;
mod tasks;

pub(crate) use communication::*;
pub(crate) use documents::*;
pub(crate) use graph::*;
pub(crate) use persons::*;
pub(crate) use projects::*;
pub(crate) use tasks::*;
```

### `backend/src/app/api_support/query_parsing/communication.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/query_parsing/communication.rs`
- Size bytes / Размер в байтах: `2654`
- Included characters / Включено символов: `2654`
- Truncated / Обрезано: `no`

```rust
use url::form_urlencoded;

use crate::app::ApiError;
use crate::domains::communications::messages::{
    MessageSearchMatchMode, MessageSearchQuery, parse_communication_message_search_query,
};

#[derive(Debug)]
pub(crate) struct CommunicationMessagesQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) workflow_state: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) conversation_id: Option<String>,
    pub(crate) q: Option<String>,
    pub(crate) match_mode: MessageSearchMatchMode,
    pub(crate) search: MessageSearchQuery,
    pub(crate) local_state: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_communication_messages_query(
    raw_query: Option<&str>,
) -> Result<CommunicationMessagesQuery, ApiError> {
    let mut query = CommunicationMessagesQuery {
        account_id: None,
        workflow_state: None,
        channel_kind: None,
        conversation_id: None,
        q: None,
        local_state: None,
        cursor: None,
        limit: None,
        match_mode: MessageSearchMatchMode::All,
        search: MessageSearchQuery::default(),
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "account_id" => query.account_id = non_empty_query_value(value.as_ref()),
                "workflow_state" => query.workflow_state = non_empty_query_value(value.as_ref()),
                "channel_kind" => query.channel_kind = non_empty_query_value(value.as_ref()),
                "conversation_id" => query.conversation_id = non_empty_query_value(value.as_ref()),
                "q" => query.q = non_empty_query_value(value.as_ref()),
                "local_state" => query.local_state = non_empty_query_value(value.as_ref()),
                "cursor" => query.cursor = non_empty_query_value(value.as_ref()),
                "limit" => {
                    query.limit = Some(value.parse::<i64>().map_err(|_| {
                        ApiError::InvalidCommunicationQuery("limit must be an integer")
                    })?);
                }
                _ => {}
            }
        }
    }

    if let Some(raw_query) = query.q.as_deref() {
        let parsed = parse_communication_message_search_query(Some(raw_query));
        query.match_mode = parsed.match_mode;
        query.search = parsed;
    }

    Ok(query)
}

fn non_empty_query_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
```

### `backend/src/app/api_support/query_parsing/documents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/query_parsing/documents.rs`
- Size bytes / Размер в байтах: `1664`
- Included characters / Включено символов: `1664`
- Truncated / Обрезано: `no`

```rust
use url::form_urlencoded;

use crate::app::ApiError;

pub(crate) struct DocumentProcessingJobsQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_document_processing_jobs_query(
    raw_query: Option<&str>,
) -> Result<DocumentProcessingJobsQuery, ApiError> {
    let mut query = DocumentProcessingJobsQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidDocumentProcessingQuery("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn validate_non_empty_document_id(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidDocumentProcessingQuery(
            "document_id must not be empty",
        ));
    }

    Ok(normalized.to_owned())
}

pub(crate) fn validate_non_empty_document_processing_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidDocumentProcessingQuery(match field {
            "command_id" => "command_id must not be empty",
            "job_id" => "job_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}
```

### `backend/src/app/api_support/query_parsing/graph.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/query_parsing/graph.rs`
- Size bytes / Размер в байтах: `2468`
- Included characters / Включено символов: `2468`
- Truncated / Обрезано: `no`

```rust
use url::form_urlencoded;

use crate::app::ApiError;

pub(crate) struct GraphNeighborhoodQuery {
    pub(crate) node_id: Option<String>,
    pub(crate) depth: Option<u8>,
}

pub(crate) struct GraphNodesQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) struct GraphSearchQuery {
    pub(crate) q: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_graph_neighborhood_query(
    raw_query: Option<&str>,
) -> Result<GraphNeighborhoodQuery, ApiError> {
    let mut query = GraphNeighborhoodQuery {
        node_id: None,
        depth: None,
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "node_id" => query.node_id = Some(value.into_owned()),
                "depth" => {
                    query.depth = Some(
                        value
                            .parse::<u8>()
                            .map_err(|_| ApiError::InvalidGraphQuery("depth supports only 1"))?,
                    );
                }
                _ => {}
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_graph_nodes_query(
    raw_query: Option<&str>,
) -> Result<GraphNodesQuery, ApiError> {
    let mut query = GraphNodesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| ApiError::InvalidGraphQuery("limit must be an integer"))?,
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_graph_search_query(
    raw_query: Option<&str>,
) -> Result<GraphSearchQuery, ApiError> {
    let mut query = GraphSearchQuery {
        q: None,
        limit: None,
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "q" => query.q = Some(value.into_owned()),
                "limit" => {
                    query.limit =
                        Some(value.parse::<i64>().map_err(|_| {
                            ApiError::InvalidGraphQuery("limit must be an integer")
                        })?);
                }
                _ => {}
            }
        }
    }

    Ok(query)
}
```

### `backend/src/app/api_support/query_parsing/persons.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/query_parsing/persons.rs`
- Size bytes / Размер в байтах: `2015`
- Included characters / Включено символов: `2015`
- Truncated / Обрезано: `no`

```rust
use url::form_urlencoded;

use crate::app::ApiError;
use crate::domains::persons::identity::PersonIdentityReviewState;

pub(crate) struct PersonIdentityCandidatesQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_person_identity_candidates_query(
    raw_query: Option<&str>,
) -> Result<PersonIdentityCandidatesQuery, ApiError> {
    let mut query = PersonIdentityCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidPersonIdentityReview("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_person_identity_review_state(
    value: &str,
) -> Result<PersonIdentityReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(PersonIdentityReviewState::Suggested),
        "user_confirmed" => Ok(PersonIdentityReviewState::UserConfirmed),
        "user_rejected" => Ok(PersonIdentityReviewState::UserRejected),
        _ => Err(ApiError::InvalidPersonIdentityReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn validate_non_empty_person_identity_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidPersonIdentityReview(match field {
            "command_id" => "command_id must not be empty",
            "identity_candidate_id" => "identity_candidate_id must not be empty",
            "person_id" => "person_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}
```

### `backend/src/app/api_support/query_parsing/projects.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/query_parsing/projects.rs`
- Size bytes / Размер в байтах: `2817`
- Included characters / Включено символов: `2817`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;
use url::form_urlencoded;

use crate::app::ApiError;
use crate::domains::projects::link_reviews::{ProjectLinkReviewState, ProjectLinkTargetKind};

#[derive(Deserialize)]
pub(crate) struct ProjectLinkCandidatesQuery {
    pub(crate) limit: Option<usize>,
}

pub(crate) struct ProjectsQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_projects_query(raw_query: Option<&str>) -> Result<ProjectsQuery, ApiError> {
    let mut query = ProjectsQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| ApiError::InvalidProjectQuery("limit must be an integer"))?,
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_project_link_candidates_query(
    raw_query: Option<&str>,
) -> Result<ProjectLinkCandidatesQuery, ApiError> {
    let mut query = ProjectLinkCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| {
                            ApiError::InvalidProjectLinkReview("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_project_link_target_kind(
    value: &str,
) -> Result<ProjectLinkTargetKind, ApiError> {
    match value.trim() {
        "message" => Ok(ProjectLinkTargetKind::Message),
        "document" => Ok(ProjectLinkTargetKind::Document),
        _ => Err(ApiError::InvalidProjectLinkReview(
            "target_kind must be message or document",
        )),
    }
}

pub(crate) fn parse_project_link_review_state(
    value: &str,
) -> Result<ProjectLinkReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(ProjectLinkReviewState::Suggested),
        "user_confirmed" => Ok(ProjectLinkReviewState::UserConfirmed),
        "user_rejected" => Ok(ProjectLinkReviewState::UserRejected),
        _ => Err(ApiError::InvalidProjectLinkReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn validate_non_empty_project_link_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidProjectLinkReview(field));
    }

    Ok(normalized.to_owned())
}
```

### `backend/src/app/api_support/query_parsing/tasks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/query_parsing/tasks.rs`
- Size bytes / Размер в байтах: `1961`
- Included characters / Включено символов: `1961`
- Truncated / Обрезано: `no`

```rust
use url::form_urlencoded;

use crate::app::ApiError;
use crate::domains::tasks::candidates::TaskCandidateReviewState;

pub(crate) struct TaskCandidatesQuery {
    pub(crate) limit: Option<i64>,
}

pub(crate) fn parse_task_candidates_query(
    raw_query: Option<&str>,
) -> Result<TaskCandidatesQuery, ApiError> {
    let mut query = TaskCandidatesQuery { limit: None };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            if key.as_ref() == "limit" {
                query.limit = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| {
                            ApiError::InvalidTaskCandidateQuery("limit must be an integer")
                        })?
                        .clamp(1, 100),
                );
            }
        }
    }

    Ok(query)
}

pub(crate) fn parse_task_candidate_review_state(
    value: &str,
) -> Result<TaskCandidateReviewState, ApiError> {
    match value.trim() {
        "suggested" => Ok(TaskCandidateReviewState::Suggested),
        "user_confirmed" => Ok(TaskCandidateReviewState::UserConfirmed),
        "user_rejected" => Ok(TaskCandidateReviewState::UserRejected),
        _ => Err(ApiError::InvalidTaskCandidateReview(
            "review_state must be suggested, user_confirmed, or user_rejected",
        )),
    }
}

pub(crate) fn validate_non_empty_task_candidate_field(
    field: &'static str,
    value: &str,
) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::InvalidTaskCandidateReview(match field {
            "command_id" => "command_id must not be empty",
            "review_state" => "review_state must not be empty",
            "task_candidate_id" => "task_candidate_id must not be empty",
            _ => "field must not be empty",
        }));
    }

    Ok(normalized.to_owned())
}
```

### `backend/src/app/api_support/review_commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/review_commands.rs`
- Size bytes / Размер в байтах: `6148`
- Included characters / Включено символов: `6148`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[derive(Serialize)]
pub(crate) struct PersonIdentityCandidateListResponse {
    pub(crate) items: Vec<PersonIdentityCandidate>,
}

#[derive(Deserialize)]
pub(crate) struct PersonIdentityReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) review_state: String,
}

impl PersonIdentityReviewApiRequest {
    pub(crate) fn into_command(
        self,
        identity_candidate_id: String,
        actor_id: String,
    ) -> Result<PersonIdentityReviewCommand, ApiError> {
        let command_id = validate_non_empty_person_identity_field("command_id", &self.command_id)?;
        let identity_candidate_id = validate_non_empty_person_identity_field(
            "identity_candidate_id",
            &identity_candidate_id,
        )?;
        let review_state = parse_person_identity_review_state(&self.review_state)?;

        Ok(PersonIdentityReviewCommand {
            command_id,
            identity_candidate_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct PersonIdentityReviewApiResponse {
    pub(crate) identity_candidate_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::persons::identity::PersonIdentityReviewCommandResult>
    for PersonIdentityReviewApiResponse
{
    fn from(result: crate::domains::persons::identity::PersonIdentityReviewCommandResult) -> Self {
        Self {
            identity_candidate_id: result.identity_candidate_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct DocumentProcessingJobsResponse {
    pub(crate) items: Vec<DocumentProcessingJob>,
}

#[derive(Deserialize)]
pub(crate) struct DocumentProcessingRetryApiRequest {
    pub(crate) command_id: String,
}

impl DocumentProcessingRetryApiRequest {
    pub(crate) fn into_command(
        self,
        job_id: String,
        actor_id: String,
    ) -> Result<DocumentProcessingRetryCommand, ApiError> {
        let command_id =
            validate_non_empty_document_processing_field("command_id", &self.command_id)?;
        let job_id = validate_non_empty_document_processing_field("job_id", &job_id)?;

        Ok(DocumentProcessingRetryCommand {
            command_id,
            job_id,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct DocumentProcessingRetryApiResponse {
    pub(crate) job_id: String,
    pub(crate) status: DocumentProcessingStatus,
    pub(crate) event_id: String,
}

impl From<DocumentProcessingRetryCommandResult> for DocumentProcessingRetryApiResponse {
    fn from(result: DocumentProcessingRetryCommandResult) -> Self {
        Self {
            job_id: result.job_id,
            status: result.status,
            event_id: result.event_id,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct TaskCandidateReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) review_state: String,
}

impl TaskCandidateReviewApiRequest {
    pub(crate) fn into_command(
        self,
        task_candidate_id: String,
        actor_id: String,
    ) -> Result<TaskCandidateReviewCommand, ApiError> {
        let command_id = validate_non_empty_task_candidate_field("command_id", &self.command_id)?;
        let task_candidate_id =
            validate_non_empty_task_candidate_field("task_candidate_id", &task_candidate_id)?;
        let review_state = parse_task_candidate_review_state(&self.review_state)?;

        Ok(TaskCandidateReviewCommand {
            command_id,
            task_candidate_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct TaskCandidateReviewApiResponse {
    pub(crate) task_candidate_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::tasks::candidates::TaskCandidateReviewCommandResult>
    for TaskCandidateReviewApiResponse
{
    fn from(result: crate::domains::tasks::candidates::TaskCandidateReviewCommandResult) -> Self {
        Self {
            task_candidate_id: result.task_candidate_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ProjectLinkReviewApiRequest {
    pub(crate) command_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) review_state: String,
}

impl ProjectLinkReviewApiRequest {
    pub(crate) fn into_command(
        self,
        project_id: String,
        actor_id: String,
    ) -> Result<ProjectLinkReviewCommand, ApiError> {
        let command_id = validate_non_empty_project_link_field("command_id", &self.command_id)?;
        let project_id = validate_non_empty_project_link_field("project_id", &project_id)?;
        let target_id = validate_non_empty_project_link_field("target_id", &self.target_id)?;
        let target_kind = parse_project_link_target_kind(&self.target_kind)?;
        let review_state = parse_project_link_review_state(&self.review_state)?;

        Ok(ProjectLinkReviewCommand {
            command_id,
            project_id,
            target_kind,
            target_id,
            review_state,
            actor_id,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct ProjectLinkReviewApiResponse {
    pub(crate) project_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) review_state: String,
    pub(crate) event_id: String,
}

impl From<crate::domains::projects::link_reviews::ProjectLinkReviewCommandResult>
    for ProjectLinkReviewApiResponse
{
    fn from(
        result: crate::domains::projects::link_reviews::ProjectLinkReviewCommandResult,
    ) -> Self {
        Self {
            project_id: result.project_id,
            target_kind: result.target_kind.as_str().to_owned(),
            target_id: result.target_id,
            review_state: result.review_state.as_str().to_owned(),
            event_id: result.event_id,
        }
    }
}
```

### `backend/src/app/api_support/review_lists.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/review_lists.rs`
- Size bytes / Размер в байтах: `883`
- Included characters / Включено символов: `883`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[derive(Serialize)]
pub(crate) struct ProjectLinkCandidate {
    pub(crate) project_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) graph_node_id: String,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) source_label: String,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) review_state: String,
    pub(crate) evidence_excerpt: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ProjectLinkCandidateListResponse {
    pub(crate) items: Vec<ProjectLinkCandidate>,
}

#[derive(Serialize)]
pub(crate) struct TaskCandidateListResponse {
    pub(crate) items: Vec<TaskCandidate>,
}

#[derive(Deserialize)]
pub(crate) struct AiRunsQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct AiRunListResponse {
    pub(crate) items: Vec<AiAgentRun>,
}
```

### `backend/src/app/api_support/stores.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/stores.rs`
- Size bytes / Размер в байтах: `245`
- Included characters / Включено символов: `245`
- Truncated / Обрезано: `no`

```rust
mod ai_routing;
mod ai_runtime;
mod database;
mod domain_stores;
mod integration_stores;
mod settings_vault;

pub(crate) use ai_runtime::*;
pub(crate) use domain_stores::*;
pub(crate) use integration_stores::*;
pub(crate) use settings_vault::*;
```

### `backend/src/app/api_support/stores/ai_routing.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/stores/ai_routing.rs`
- Size bytes / Размер в байтах: `3209`
- Included characters / Включено символов: `3209`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(in crate::app::api_support::stores) async fn ai_model_routing(
    state: &AppState,
    settings: &AiRuntimeSettings,
) -> Result<AiModelRouting, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(AiModelRouting::fallback(
            &settings.chat_model,
            &settings.embedding_model,
        ));
    };
    let store = AiControlCenterStore::new(pool.clone());
    match resolve_ai_model_routing(&store, settings).await {
        Ok(routing) => Ok(routing),
        Err(error) => {
            tracing::warn!(error = %error, "AI model routing resolution fell back to legacy ai.* settings");
            Ok(AiModelRouting::fallback(
                &settings.chat_model,
                &settings.embedding_model,
            ))
        }
    }
}

async fn resolve_ai_model_routing(
    store: &AiControlCenterStore,
    settings: &AiRuntimeSettings,
) -> Result<AiModelRouting, AiControlCenterError> {
    Ok(AiModelRouting {
        default_chat: resolve_ai_slot_model(store, settings, "default_chat", &settings.chat_model)
            .await?,
        reasoning: resolve_ai_slot_model(store, settings, "reasoning", &settings.chat_model)
            .await?,
        summarization: resolve_ai_slot_model(
            store,
            settings,
            "summarization",
            &settings.chat_model,
        )
        .await?,
        mail_intelligence: resolve_ai_slot_model(
            store,
            settings,
            "mail_intelligence",
            &settings.chat_model,
        )
        .await?,
        reply_draft: resolve_ai_slot_model(store, settings, "reply_draft", &settings.chat_model)
            .await?,
        extraction: resolve_ai_slot_model(store, settings, "extraction", &settings.chat_model)
            .await?,
        embeddings: resolve_ai_slot_model(store, settings, "embeddings", &settings.embedding_model)
            .await?,
        meeting_prep: resolve_ai_slot_model(store, settings, "meeting_prep", &settings.chat_model)
            .await?,
    })
}

async fn resolve_ai_slot_model(
    store: &AiControlCenterStore,
    settings: &AiRuntimeSettings,
    slot: &str,
    fallback_model: &str,
) -> Result<String, AiControlCenterError> {
    let Some(route) = store.route_for_slot(slot).await? else {
        return Ok(fallback_model.to_owned());
    };
    let Some(provider) = store.provider(&route.provider_id).await? else {
        return Ok(fallback_model.to_owned());
    };
    if ai_provider_matches_runtime(&provider, settings.provider)
        && store
            .model_ready_for_private_context(&route.provider_id, &route.model_key)
            .await?
    {
        Ok(route.model_key)
    } else {
        Ok(fallback_model.to_owned())
    }
}

fn ai_provider_matches_runtime(
    provider: &AiProviderAccount,
    runtime_provider: AiRuntimeProvider,
) -> bool {
    match runtime_provider {
        AiRuntimeProvider::Ollama => {
            provider.provider_kind == "built_in" && provider.provider_key == "ollama"
        }
        AiRuntimeProvider::OmniRoute => {
            provider.provider_kind == "api" && provider.provider_key == "omniroute"
        }
    }
}
```

### `backend/src/app/api_support/stores/ai_runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/stores/ai_runtime.rs`
- Size bytes / Размер в байтах: `6878`
- Included characters / Включено символов: `6878`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use super::ai_routing::ai_model_routing;
use super::database::database_pool;
use crate::domains::signal_hub::{SignalHubError, SignalHubStore};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const AI_REQUEST_RUNTIME: &str = "ai_request_runtime";

#[derive(Clone)]
struct PersonProjectionAiPersonaAttributionPort {
    pool: sqlx::postgres::PgPool,
}

impl PersonProjectionAiPersonaAttributionPort {
    fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self { pool }
    }
}

impl crate::ai::core::AiPersonaAttributionPort for PersonProjectionAiPersonaAttributionPort {
    fn upsert_ai_agent_persona<'a>(
        &'a self,
        agent_id: &'a str,
        display_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        crate::ai::core::AiAgentPersonaAttribution,
                        crate::ai::core::AiPersonaAttributionError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let persona =
                crate::domains::persons::api::PersonProjectionStore::new(self.pool.clone())
                    .upsert_ai_agent_persona(agent_id, display_name)
                    .await
                    .map_err(|error| {
                        crate::ai::core::AiPersonaAttributionError::Store(error.to_string())
                    })?;

            Ok(crate::ai::core::AiAgentPersonaAttribution {
                persona_id: persona.person_id,
                persona_type: persona.persona_type.as_str(),
                persona_email: persona.email_address,
            })
        })
    }

    fn owner_persona_id<'a>(
        &'a self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<String>, crate::ai::core::AiPersonaAttributionError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            Ok(
                crate::domains::persons::api::PersonProjectionStore::new(self.pool.clone())
                    .owner_persona()
                    .await
                    .map_err(|error| {
                        crate::ai::core::AiPersonaAttributionError::Store(error.to_string())
                    })?
                    .map(|persona| persona.person_id),
            )
        })
    }
}

pub(crate) fn ai_run_store(state: &AppState) -> Result<crate::ai::core::AiRunStore, ApiError> {
    Ok(crate::ai::core::AiRunStore::new(database_pool(state)?))
}

pub(crate) async fn ai_service(state: &AppState) -> Result<AiService, ApiError> {
    let pool = database_pool(state)?;
    let runtime_settings = ai_runtime_settings(state).await?;
    let model_routing = ai_model_routing(state, &runtime_settings).await?;
    let runtime = ai_runtime_client(state, &runtime_settings)?;

    Ok(
        AiService::new_with_routing(pool.clone(), runtime, model_routing)
            .with_persona_attribution(ai_persona_attribution_port_from_pool(pool)),
    )
}

pub(crate) async fn ai_requests_allowed(state: &AppState) -> Result<bool, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(true);
    };

    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await?;
    crate::platform::events::runtime_allows_processing(
        pool,
        "ai",
        AI_REQUEST_RUNTIME,
        &serde_json::json!({
            "label": "AI request runtime",
            "scope": "runtime",
        }),
    )
    .await
    .map_err(SignalHubError::from)
    .map_err(ApiError::from)
}

pub(crate) fn ai_persona_attribution_port_from_pool(
    pool: sqlx::postgres::PgPool,
) -> crate::ai::core::SharedAiPersonaAttributionPort {
    Arc::new(PersonProjectionAiPersonaAttributionPort::new(pool))
        as crate::ai::core::SharedAiPersonaAttributionPort
}

pub(crate) fn ai_persona_attribution_port_optional(
    state: &AppState,
) -> Option<crate::ai::core::SharedAiPersonaAttributionPort> {
    state
        .database
        .pool()
        .map(|pool| ai_persona_attribution_port_from_pool(pool.clone()))
}

pub(crate) async fn ai_runtime_settings(state: &AppState) -> Result<AiRuntimeSettings, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(AiRuntimeSettings::from_config(&state.config));
    };

    Ok(ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(&state.config)
        .await?)
}

pub(crate) fn ai_runtime_client(
    state: &AppState,
    settings: &AiRuntimeSettings,
) -> Result<AiRuntimeClient, ApiError> {
    match settings.provider {
        AiRuntimeProvider::Ollama => Ok(AiRuntimeClient::Ollama(OllamaClient::new(
            OllamaClientConfig::new(
                &settings.base_url,
                &settings.chat_model,
                &settings.embedding_model,
            )
            .with_timeout_seconds(settings.timeout_seconds),
        )?)),
        AiRuntimeProvider::OmniRoute => {
            let api_key = state.config.omniroute_api_key().cloned().ok_or_else(|| {
                ApiError::Ai(AiError::Runtime(AiRuntimeError::OmniRoute(
                    OmniRouteError::MissingApiKey,
                )))
            })?;
            Ok(AiRuntimeClient::OmniRoute(OmniRouteClient::new(
                OmniRouteClientConfig::new(
                    &settings.base_url,
                    &settings.chat_model,
                    &settings.embedding_model,
                    api_key,
                )
                .with_timeout_seconds(settings.timeout_seconds),
            )?))
        }
    }
}

pub(crate) fn ai_runtime_port(
    state: &AppState,
    settings: &AiRuntimeSettings,
) -> Option<crate::platform::ai_runtime::SharedAiRuntimePort> {
    ai_runtime_client(state, settings)
        .ok()
        .map(|runtime| Arc::new(runtime) as crate::platform::ai_runtime::SharedAiRuntimePort)
}

pub(crate) async fn ai_runtime_port_optional(
    state: &AppState,
) -> Result<Option<crate::platform::ai_runtime::SharedAiRuntimePort>, ApiError> {
    if !ai_requests_allowed(state).await? {
        return Ok(None);
    }

    let settings = ai_runtime_settings(state).await?;
    Ok(ai_runtime_port(state, &settings))
}

pub(crate) async fn email_multilingual_service(
    state: &AppState,
) -> Result<crate::domains::communications::multilingual::MultilingualService, ApiError> {
    Ok(
        crate::domains::communications::multilingual::MultilingualService::new(
            ai_runtime_port_optional(state).await?,
        ),
    )
}

pub(crate) async fn email_ai_reply_service(
    state: &AppState,
) -> Result<crate::domains::communications::ai_reply::AiReplyService, ApiError> {
    Ok(
        crate::domains::communications::ai_reply::AiReplyService::new(
            ai_runtime_port_optional(state).await?,
        ),
    )
}
```

### `backend/src/app/api_support/stores/database.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/stores/database.rs`
- Size bytes / Размер в байтах: `284`
- Included characters / Включено символов: `284`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(in crate::app::api_support::stores) fn database_pool(
    state: &AppState,
) -> Result<sqlx::postgres::PgPool, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(pool.clone())
}
```

### `backend/src/app/api_support/stores/domain_stores.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/stores/domain_stores.rs`
- Size bytes / Размер в байтах: `9125`
- Included characters / Включено символов: `9125`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use super::database::database_pool;
use crate::application::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::domains::communications::storage::LocalCommunicationBlobStore;
use sqlx::PgPool;

pub(crate) trait AppStoreFactory: Sized {
    fn from_pool(pool: PgPool) -> Self;
}

pub(crate) fn app_store<S: AppStoreFactory>(pool: PgPool) -> S {
    S::from_pool(pool)
}

macro_rules! impl_app_store_factory {
    ($($store:path),+ $(,)?) => {
        $(
            impl AppStoreFactory for $store {
                fn from_pool(pool: PgPool) -> Self {
                    <$store>::new(pool)
                }
            }
        )+
    };
}

impl_app_store_factory!(
    crate::domains::calendar::core::EventAgendaStore,
    crate::domains::calendar::core::EventChecklistStore,
    crate::domains::calendar::core::EventContextPackStore,
    crate::domains::calendar::core::EventParticipantStore,
    crate::domains::calendar::core::EventRelationStore,
    crate::domains::calendar::events::CalendarAccountStore,
    crate::domains::calendar::events::CalendarEventStore,
    crate::domains::calendar::events::CalendarSourceStore,
    crate::domains::calendar::meetings::EventRecordingStore,
    crate::domains::calendar::meetings::EventTranscriptStore,
    crate::domains::calendar::meetings::MeetingNoteStore,
    crate::domains::calendar::meetings::MeetingOutcomeStore,
    crate::domains::calendar::reminders::CalendarReminderStore,
    crate::domains::calendar::rules::CalendarRuleStore,
    crate::domains::calendar::scheduling::DeadlineStore,
    crate::domains::calendar::scheduling::FocusBlockStore,
    crate::domains::communications::ai_state::CommunicationAiStateStore,
    crate::domains::communications::analytics::EmailAnalyticsStore,
    crate::domains::communications::attachment_dedup::AttachmentDedupStore,
    crate::domains::communications::attachment_search::AttachmentSearchStore,
    crate::domains::communications::bulk_actions::BulkMessageActionStore,
    crate::domains::communications::core::CommunicationProviderAccountStore,
    crate::domains::communications::core::CommunicationProviderSecretBindingStore,
    crate::domains::communications::delivery_notifications::CommunicationDeliveryNotificationStore,
    crate::domains::communications::drafts::CommunicationDraftStore,
    crate::domains::communications::finance::CommunicationFinanceStore,
    crate::domains::communications::folders::CommunicationFolderStore,
    crate::domains::communications::legal::LegalDocumentStore,
    crate::domains::communications::messages::MessageProjectionStore,
    crate::domains::communications::outbox::CommunicationOutboxStore,
    crate::domains::communications::personas::CommunicationPersonaStore,
    crate::domains::communications::read_receipts::CommunicationReadReceiptStore,
    crate::domains::communications::saved_searches::CommunicationSavedSearchStore,
    crate::domains::communications::signatures::CertificateStore,
    crate::domains::communications::subscriptions::SubscriptionStore,
    crate::domains::communications::templates::CommunicationTemplateStore,
    crate::domains::communications::threads::CommunicationThreadStore,
    crate::domains::decisions::DecisionStore,
    crate::domains::documents::processing::DocumentProcessingStore,
    crate::domains::obligations::ObligationStore,
    crate::domains::organizations::api::OrganizationStore,
    crate::domains::organizations::core::OrgAliasStore,
    crate::domains::organizations::core::OrgContactLinkStore,
    crate::domains::organizations::core::OrgDepartmentStore,
    crate::domains::organizations::core::OrgDomainStore,
    crate::domains::organizations::core::OrgIdentityStore,
    crate::domains::organizations::core::RelatedOrgStore,
    crate::domains::organizations::enrichment::OrgEnrichmentStore,
    crate::domains::organizations::finance::OrgComplianceStore,
    crate::domains::organizations::finance::OrgContractStore,
    crate::domains::organizations::finance::OrgFinancialStore,
    crate::domains::organizations::finance::OrgProductStore,
    crate::domains::organizations::finance::OrgServiceStore,
    crate::domains::organizations::health::OrgHealthStore,
    crate::domains::organizations::health::OrgRiskStore,
    crate::domains::organizations::workflows::OrgPlaybookStore,
    crate::domains::organizations::workflows::OrgPortalStore,
    crate::domains::organizations::workflows::OrgProcedureStore,
    crate::domains::organizations::workflows::OrgTemplateStore,
    crate::domains::organizations::workflows::OrgTimelineStore,
    crate::domains::persons::api::PersonProjectionStore,
    crate::domains::persons::core::PersonPersonaStore,
    crate::domains::persons::core::PersonRoleStore,
    crate::domains::persons::core::PersonsIdentityStore,
    crate::domains::persons::enrichment::PersonEnrichmentStore,
    crate::domains::persons::enrichment_engine::EnrichmentResultStore,
    crate::domains::persons::expertise::PersonExpertiseStore,
    crate::domains::persons::health::PersonHealthStore,
    crate::domains::persons::memory::PersonFactStore,
    crate::domains::persons::memory::PersonMemoryCardStore,
    crate::domains::persons::memory::PersonPreferenceStore,
    crate::domains::persons::memory::PersonSnapshotStore,
    crate::domains::persons::memory::RelationshipEventStore,
    crate::domains::persons::trust::PersonPromiseStore,
    crate::domains::persons::trust::PersonRiskStore,
    crate::domains::relationships::RelationshipStore,
    crate::domains::review::ReviewInboxStore,
    crate::domains::tasks::api::TaskStore,
    crate::domains::tasks::core::ExternalTaskIdentityStore,
    crate::domains::tasks::core::TaskChecklistStore,
    crate::domains::tasks::core::TaskContextPackStore,
    crate::domains::tasks::core::TaskEvidenceStore,
    crate::domains::tasks::core::TaskProviderStore,
    crate::domains::tasks::core::TaskRelationStore,
    crate::domains::tasks::core::TaskSubtaskStore,
    crate::domains::tasks::rules::TaskRuleStore,
    crate::domains::tasks::rules::TaskTemplateStore,
    crate::engines::consistency::ContradictionObservationStore,
    crate::platform::events::EventStore,
    crate::platform::observations::ObservationStore,
    crate::application::mail_background_sync::MailSyncStore,
);

pub(crate) fn communication_blob_store() -> LocalCommunicationBlobStore {
    LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
}

pub(crate) fn event_store(state: &AppState) -> Result<EventStore, ApiError> {
    Ok(EventStore::new(database_pool(state)?))
}

pub(crate) fn graph_store(
    state: &AppState,
) -> Result<crate::domains::graph::core::GraphStore, ApiError> {
    Ok(crate::domains::graph::core::GraphStore::new(database_pool(
        state,
    )?))
}

pub(crate) fn message_store(state: &AppState) -> Result<MessageProjectionStore, ApiError> {
    Ok(MessageProjectionStore::new(database_pool(state)?))
}

pub(crate) fn observation_store(
    state: &AppState,
) -> Result<crate::platform::observations::ObservationStore, ApiError> {
    Ok(crate::platform::observations::ObservationStore::new(
        database_pool(state)?,
    ))
}

pub(crate) fn communication_storage_store(
    state: &AppState,
) -> Result<CommunicationStorageStore, ApiError> {
    Ok(CommunicationStorageStore::new(database_pool(state)?))
}

pub(crate) fn communication_ingestion_store(
    state: &AppState,
) -> Result<CommunicationIngestionStore, ApiError> {
    Ok(CommunicationIngestionStore::new(database_pool(state)?))
}

pub(crate) fn communication_provider_account_store(
    state: &AppState,
) -> Result<crate::domains::communications::core::CommunicationProviderAccountStore, ApiError> {
    Ok(
        crate::domains::communications::core::CommunicationProviderAccountStore::new(
            database_pool(state)?,
        ),
    )
}

pub(crate) fn communication_provider_secret_binding_store(
    state: &AppState,
) -> Result<crate::domains::communications::core::CommunicationProviderSecretBindingStore, ApiError>
{
    Ok(
        crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            database_pool(state)?,
        ),
    )
}

pub(crate) fn project_store(state: &AppState) -> Result<ProjectStore, ApiError> {
    Ok(ProjectStore::new(database_pool(state)?))
}

pub(crate) fn project_link_review_store(
    state: &AppState,
) -> Result<ProjectLinkReviewStore, ApiError> {
    Ok(ProjectLinkReviewStore::new(database_pool(state)?))
}

pub(crate) fn task_candidate_store(state: &AppState) -> Result<TaskCandidateStore, ApiError> {
    Ok(TaskCandidateStore::new(database_pool(state)?))
}

pub(crate) fn document_processing_store(
    state: &AppState,
) -> Result<DocumentProcessingStore, ApiError> {
    Ok(DocumentProcessingStore::new(database_pool(state)?))
}

pub(crate) fn person_identity_store(state: &AppState) -> Result<PersonIdentityStore, ApiError> {
    Ok(PersonIdentityStore::new(database_pool(state)?))
}

pub(crate) fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    Ok(ApiAuditLog::new(database_pool(state)?))
}
```

### `backend/src/app/api_support/stores/integration_stores.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/stores/integration_stores.rs`
- Size bytes / Размер в байтах: `6761`
- Included characters / Включено символов: `6761`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use super::database::database_pool;
use std::sync::Arc;

fn build_telegram_provider_store(
    state: &AppState,
) -> Result<crate::application::TelegramProviderRuntimeStore, ApiError> {
    Ok(crate::application::telegram_provider_runtime_store(
        database_pool(state)?,
    ))
}

pub(crate) fn telegram_provider_runtime_service(
    state: &AppState,
) -> Result<crate::application::TelegramProviderRuntimeApplicationService, ApiError> {
    Ok(crate::application::telegram_provider_runtime_service(
        database_pool(state)?,
    ))
}

pub(crate) fn telegram_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn whatsapp_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn zoom_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn telegram_runtime_use_case_context(
    state: &AppState,
) -> Result<crate::application::telegram_runtime::TelegramRuntimeUseCaseContext<'_>, ApiError> {
    let pool = database_pool(state)?;
    Ok(
        crate::application::telegram_runtime::TelegramRuntimeUseCaseContext::new(
            crate::application::telegram_runtime::TelegramRuntimeUseCaseStores {
                provider_account_store:
                    crate::domains::communications::core::CommunicationProviderAccountStore::new(
                        pool.clone(),
                    ),
                provider_secret_binding_store:
                    crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                        pool.clone(),
                    ),
                telegram_store: build_telegram_provider_store(state)?,
                secret_store: SecretReferenceStore::new(pool),
            },
            crate::application::telegram_runtime::TelegramRuntimeUseCaseRuntime {
                secret_resolver: &state.vault,
                config: &state.config,
                event_bus: &state.event_bus,
                runtime: &state.telegram_runtime,
            },
        ),
    )
}

pub(crate) fn telegram_message_write_service(
    state: &AppState,
) -> Result<
    crate::application::communication_provider_writes::TelegramMessageWriteApplicationService,
    ApiError,
> {
    Ok(
        crate::application::communication_provider_writes::TelegramMessageWriteApplicationService::new(
            build_telegram_provider_store(state)?,
            api_audit_log(state)?,
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn telegram_fixture_ingest_service(
    state: &AppState,
) -> Result<
    crate::application::communication_fixture_ingest::TelegramFixtureIngestApplicationService,
    ApiError,
> {
    Ok(
        crate::application::communication_fixture_ingest::TelegramFixtureIngestApplicationService::new(
            database_pool(state)?,
            build_telegram_provider_store(state)?,
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

fn build_whatsapp_provider_store(
    state: &AppState,
) -> Result<crate::application::WhatsAppProviderRuntimeRef, ApiError> {
    Ok(crate::application::whatsapp_provider_runtime(
        database_pool(state)?,
    ))
}

pub(crate) fn whatsapp_provider_runtime_service(
    state: &AppState,
) -> Result<crate::application::WhatsappProviderRuntimeApplicationService, ApiError> {
    Ok(crate::application::whatsapp_provider_runtime_service(
        database_pool(state)?,
    ))
}

pub(crate) fn zoom_provider_runtime_service(
    state: &AppState,
) -> Result<crate::application::ZoomProviderRuntimeApplicationService, ApiError> {
    Ok(crate::application::zoom_provider_runtime_service(
        database_pool(state)?,
        state.event_bus.clone(),
    ))
}

pub(crate) fn yandex_telemost_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn yandex_telemost_provider_runtime_store(
    state: &AppState,
) -> Result<crate::integrations::yandex_telemost::client::YandexTelemostStore, ApiError> {
    let pool = database_pool(state)?;
    Ok(
        crate::integrations::yandex_telemost::client::YandexTelemostStore::new(
            Arc::new(
                crate::domains::communications::core::CommunicationProviderAccountStore::new(
                    pool.clone(),
                ),
            ),
            Arc::new(
                crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                    pool.clone(),
                ),
            ),
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn yandex_telemost_provider_runtime_service(
    state: &AppState,
) -> Result<crate::application::YandexTelemostProviderRuntimeApplicationService, ApiError> {
    Ok(
        crate::application::yandex_telemost_provider_runtime_service(
            database_pool(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn whatsapp_fixture_ingest_service(
    state: &AppState,
) -> Result<
    crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService,
    ApiError,
> {
    Ok(
        crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService::new(
            database_pool(state)?,
            build_whatsapp_provider_store(state)?,
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn automation_store(state: &AppState) -> Result<AutomationStore, ApiError> {
    Ok(AutomationStore::new(database_pool(state)?))
}

pub(crate) fn call_intelligence_store(state: &AppState) -> Result<CallIntelligenceStore, ApiError> {
    Ok(CallIntelligenceStore::new(database_pool(state)?))
}

pub(crate) fn account_setup_service(
    state: &AppState,
) -> Result<EmailAccountSetupService, ApiError> {
    let pool = database_pool(state)?;
    Ok(EmailAccountSetupService::new_with_host_vault(
        pool.clone(),
        SecretReferenceStore::new(pool.clone()),
        state.vault.clone(),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                pool,
            ),
        ),
    ))
}
```

### `backend/src/app/api_support/stores/settings_vault.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/stores/settings_vault.rs`
- Size bytes / Размер в байтах: `470`
- Included characters / Включено символов: `470`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use super::database::database_pool;

pub(crate) fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    Ok(ApplicationSettingsStore::new(database_pool(state)?))
}

pub(crate) fn database_encrypted_vault(
    config: &AppConfig,
    pool: sqlx::postgres::PgPool,
) -> Option<DatabaseEncryptedSecretVault> {
    Some(DatabaseEncryptedSecretVault::new(
        pool,
        config.secret_vault_key()?.clone(),
    ))
}
```

### `backend/src/app/api_support/telegram_capabilities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/telegram_capabilities.rs`
- Size bytes / Размер в байтах: `16037`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::*;
use crate::domains::communications::core::ProviderAccount;

// ---------------------------------------------------------------------------

/// Capability states per ADR-0091.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramCapabilityState {
    Available,
    Blocked,
    Degraded,
    Planned,
    Unsupported,
}

impl TelegramCapabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Planned => "planned",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Action classes per ADR-0052.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    Export,
    SecretAccess,
    Automation,
}

impl TelegramActionClass {
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

/// A single operation capability entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelegramOperationCapability {
    pub operation: String,
    pub category: String,
    pub status: String,
    pub action_class: String,
    pub reason: String,
    pub confirmation_required: bool,
    pub closure_gate: bool,
}

impl TelegramOperationCapability {
    pub(super) fn new(
        operation: &str,
        category: &str,
        state: TelegramCapabilityState,
        action_class: TelegramActionClass,
        reason: &str,
        confirmation_required: bool,
        closure_gate: bool,
    ) -> Self {
        Self {
            operation: operation.to_owned(),
            category: category.to_owned(),
            status: state.as_str().to_owned(),
            action_class: action_class.as_str().to_owned(),
            reason: reason.to_owned(),
            confirmation_required,
            closure_gate,
        }
    }
}

/// Detailed per-operation Telegram capability response per ADR-0091.
#[derive(Serialize)]
pub(crate) struct TelegramCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) account_scope: Option<TelegramCapabilityAccountScope>,
    pub(crate) telegram_app_credentials_configured: bool,
    pub(crate) tdjson_runtime_available: bool,
    pub(crate) qr_login_ready: bool,
    pub(crate) bot_runtime_available: bool,
    pub(crate) capabilities: Vec<TelegramOperationCapability>,
    pub(crate) planned_features: Vec<&'static str>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelegramCapabilityAccountScope {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub lifecycle_state: String,
}

impl TelegramCapabilitiesResponse {
    pub(crate) fn current(config: &AppConfig) -> Self {
        Self::build(config, None)
    }

    pub(crate) fn current_for_account(config: &AppConfig, account: &ProviderAccount) -> Self {
        Self::build(config, Some(account))
    }

    fn build(config: &AppConfig, account: Option<&ProviderAccount>) -> Self {
        let app_creds = config.telegram_api_id().is_some() && config.telegram_api_hash().is_some();
        let tdjson_ok = tdjson::runtime_available(config.tdjson_path());
        let qr_ready = app_creds && tdjson_ok;
        let bot_ok = false; // Bot API runtime not implemented per ADR-0091
        let account_scope = account.map(TelegramCapabilityAccountScope::from_account);

        let capabilities = super::telegram_capability_catalog::telegram_capability_rows(qr_ready);
        let mut response = Self {
            version: "2.1",
            runtime_mode: if let Some(scope) = account_scope.as_ref() {
                match scope.runtime_kind.as_str() {
                    "tdlib_qr_authorized" => "tdlib_qr_authorized",
                    "live_blocked" => "live_blocked",
                    "fixture" => "fixture",
                    _ => "unknown",
                }
            } else if qr_ready {
                "tdlib_qr"
            } else {
                "fixture"
            },
            account_scope,
            telegram_app_credentials_configured: app_creds,
            tdjson_runtime_available: tdjson_ok,
            qr_login_ready: qr_ready,
            bot_runtime_available: bot_ok,
            capabilities,
            planned_features: vec![
                "bot_runtime",
                "voice_recording",
                "voice_send",
                "video_recording",
                "live_calls",
                "session_export",
                "session_import",
                "mtproxy",
                "socks5",
                "ai_summary",
                "translation",
                "bilingual_reply",
                "ai_review_flows",
            ],
            unsupported_features: vec![
                "group_calls",
                "screen_sharing",
                "hidden_recording",
                "telegram_data_fine_tuning",
                "third_party_plugin_execution",
                "chat_export",
            ],
        };
        response.apply_account_scope_overrides();
        response
    }

    fn apply_account_scope_overrides(&mut self) {
        let Some(scope) = self.account_scope.as_ref() else {
            return;
        };
        let provider_kind = scope.provider_kind.as_str();
        let lifecycle_state = scope.lifecycle_state.as_str();
        let runtime_kind = scope.runtime_kind.as_str();
        let is_bot = provider_kind == "telegram_bot";
        let is_removed = lifecycle_state == "removed";
        let is_logged_out = lifecycle_state == "logged_out";

        for capability in &mut self.capabilities {
            match capability.operation.as_str() {
                "auth.qr_start" | "auth.qr_status" | "auth.qr_password" | "auth.qr_cancel"
                    if is_bot =>
                {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use TDLib QR authorization.".to_owned();
                }
                "runtime.tdlib_live" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib user runtime.".to_owned();
                }
                "runtime.bot_live" if !is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason = "User accounts do not use the Bot API runtime.".to_owned();
                }
                "messages.send_text" if is_bot && runtime_kind == "fixture" => {
                    capability.status = TelegramCapabilityState::Degraded.as_str().to_owned();
                    capability.reason = "Fixture bot accounts can validate local command flow, but the live Bot API runtime is still missing.".to_owned();
                }
                "messages.send_media" | "media.upload_send" if is_bot => {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "Bot media upload/send requires the separate Bot API runtime.".to_owned();
                }
                "messages.send_media" | "media.upload_send"
                    if runtime_kind != "tdlib_qr_authorized" =>
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before media upload/send is available.",
                        scope.account_id
                    );
                }
                "participants.sync" | "participants.join" | "participants.leave" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib participant lifecycle runtime."
                            .to_owned();
                }
                "dialogs.folder_add" | "dialogs.folder_remove" | "dialogs.folder_reassign"
                    if is_bot =>
                {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib user dialog-folder runtime.".to_owned();
                }
                "topics.list" | "topics.create" | "topics.close" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib forum topic projection/runtime."
                            .to_owned();
                }
                "participants.sync" | "participants.join" | "participants.leave"
                    if runtime_kind != "tdlib_qr_authorized" =>
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before participant lifecycle commands are available.",
                        scope.account_id
                    );
                }
                "participants.sync" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "TDLib provider member roster sync uses provider-observed roster snapshots for groups and TDLib chat metadata for private/saved-message chats.".to_owned();
                }
                "participants.join" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "Joining Telegram chats uses the durable provider-write outbox and reconciles completion from provider-observed membership evidence.".to_owned();
                }
                "participants.leave" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "Leaving Telegram chats uses the durable provider-write outbox and reconciles completion from provider-observed membership or exhaustive absence evidence.".to_owned();
                }
                "dialogs.folder_add" | "dialogs.folder_remove" | "dialogs.folder_reassign"
                    if runtime_kind != "tdlib_qr_authorized" =>
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before Telegram folder provider-write commands are available.",
                        scope.account_id
                    );
                }
                "dialogs.folder_add" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "Adding a chat to a Telegram folder uses the durable provider-write outbox and TDLib chat-position reconciliation for the target folder.".to_owned();
                }

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/api_support/telegram_capability_catalog.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/telegram_capability_catalog.rs`
- Size bytes / Размер в байтах: `707`
- Included characters / Включено символов: `707`
- Truncated / Обрезано: `no`

```rust
use super::telegram_capabilities::TelegramOperationCapability;

#[cfg(test)]
#[path = "telegram_capability_catalog_tests.rs"]
mod telegram_capability_catalog_tests;

pub(super) fn telegram_capability_rows(qr_ready: bool) -> Vec<TelegramOperationCapability> {
    let mut capabilities = Vec::new();
    super::telegram_capability_catalog_foundation::push_foundation_capabilities(
        &mut capabilities,
        qr_ready,
    );
    super::telegram_capability_catalog_messages::push_message_capabilities(
        &mut capabilities,
        qr_ready,
    );
    super::telegram_capability_catalog_extended::push_extended_capabilities(
        &mut capabilities,
        qr_ready,
    );
    capabilities
}
```

### `backend/src/app/api_support/telegram_capability_catalog_extended.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/telegram_capability_catalog_extended.rs`
- Size bytes / Размер в байтах: `11154`
- Included characters / Включено символов: `11090`
- Truncated / Обрезано: `no`

```rust
use super::telegram_capabilities::{
    TelegramActionClass, TelegramCapabilityState, TelegramOperationCapability,
};

pub(super) fn push_extended_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    // ── dialogs ──
    let cat_dialogs = "dialogs";
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.mark_unread",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "Mark-unread relies on durable provider-write commands and TDLib unread-state reconciliation.",
        false,
        false,
    ));

    // ── media ──
    let cat_media = "media";
    capabilities.push(TelegramOperationCapability::new(
        "media.download",
        cat_media,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::Read,
        if qr_ready {
            "TDLib media download is available."
        } else {
            "Media download limited to fixture runtime (fails closed)."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "media.upload_send",
        cat_media,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Provider-side media upload/send is available through local attachment import and durable outbox."
        } else {
            "Media upload/send requires TDLib QR runtime, local attachment import and durable outbox."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "media.gallery",
        cat_media,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Media gallery is backed by projected Telegram attachment metadata plus query-backed media search results.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "media.preview",
        cat_media,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Telegram media preview uses the shared Communication attachment preview boundary and local downloaded media paths.",
        false,
        false,
    ));

    // ── voice / calls ──
    let cat_voice = "voice_calls";
    capabilities.push(TelegramOperationCapability::new(
        "voice.playback",
        cat_voice,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Projected Telegram voice/audio attachments play from local downloaded media through the shared media viewer.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "voice.record_send",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Voice recording/send is deferred to the separate Voice initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "voice.record",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Voice recording is deferred to the separate Voice initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "voice.send",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Voice send is deferred to the separate Voice initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "video.record",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Video recording is deferred to a separate Voice/Calls initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "calls.metadata",
        cat_voice,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Call metadata and fixture transcript storage are available.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "calls.live_control",
        cat_voice,
        TelegramCapabilityState::Planned,
        TelegramActionClass::ProviderWrite,
        "Live call control is deferred to the separate Calls initiative.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "calls.transcription_live",
        cat_voice,
        TelegramCapabilityState::Blocked,
        TelegramActionClass::Read,
        "Live call transcription remains blocked until the separate Calls/AI runtime, permission and validation work is implemented.",
        false,
        true,
    ));

    // ── search ──
    let cat_search = "search";
    capabilities.push(TelegramOperationCapability::new(
        "search.local_messages",
        cat_search,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Local thread search/filter and shared Communication search are available.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "search.local_dialogs",
        cat_search,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Local chat title filter is available.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "search.provider",
        cat_search,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::Read,
        "Provider-side TDLib search refreshes provider results into projection before returning UI-visible results.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "search.media",
        cat_search,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Media search reads projected Telegram attachment metadata and attempts provider refresh when account and query are available.",
        false,
        false,
    ));

    // ── realtime ──
    let cat_rt = "realtime";
    capabilities.push(TelegramOperationCapability::new(
        "realtime.generic_transport",
        cat_rt,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Generic WebSocket/SSE/long-poll transports exist at platform level.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "realtime.message_created",
        cat_rt,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "telegram.message.created realtime events are emitted for fixture ingest and manual sends.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
            "realtime.message_updated", cat_rt,
            TelegramCapabilityState::Available, TelegramActionClass::Read,
            "telegram.message.updated realtime events are emitted when lifecycle edit records are created.",
            false, false,
        ));
    capabilities.push(TelegramOperationCapability::new(
        "realtime.message_deleted",
        cat_rt,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "telegram.message.deleted realtime events are emitted when tombstones are recorded.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
            "realtime.reaction_changed", cat_rt,
            TelegramCapabilityState::Available, TelegramActionClass::Read,
            "telegram.reaction.changed realtime events are emitted for local reaction add/remove actions.",
            false, false,
        ));
    capabilities.push(TelegramOperationCapability::new(
        "realtime.sync_progress",
        cat_rt,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "telegram.sync progress events are emitted for chat and history sync requests.",
        false,
        false,
    ));

    // ── automation ──
    let cat_auto = "automation";
    capabilities.push(TelegramOperationCapability::new(
        "automation.dry_run",
        cat_auto,
        TelegramCapabilityState::Available,
        TelegramActionClass::Automation,
        "Policy/template validation and audited dry-run records are available.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
            "automation.live_send", cat_auto,
            TelegramCapabilityState::Blocked, TelegramActionClass::Automation,
            "Live automated sends blocked until live runtime passes the same policy evaluator and audit contract.",
            true, true,
        ));

    // ── ai ──
    let cat_ai = "ai";
    capabilities.push(TelegramOperationCapability::new(
        "ai.summary",
        cat_ai,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Telegram-specific summary is deferred to the separate AI Layer initiative.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "ai.translation",
        cat_ai,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Telegram-specific translation is deferred to the separate AI Layer initiative.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "ai.bilingual_reply",
        cat_ai,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Telegram-specific bilingual reply is deferred to the separate AI Layer initiative.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "ai.review_flows",
        cat_ai,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Telegram-specific AI review flows are deferred to the separate AI Layer initiative.",
        false,
        false,
    ));

    // ── export ──
    let cat_export = "export";
    capabilities.push(TelegramOperationCapability::new(
        "export.chat",
        cat_export,
        TelegramCapabilityState::Unsupported,
        TelegramActionClass::Export,
        "Chat export requires scope selection, audit and manifest contract.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "export.markdown",
        cat_export,
        TelegramCapabilityState::Unsupported,
        TelegramActionClass::Export,
        "Markdown export requires message-order, sender, time and attachment referencing.",
        true,
        true,
    ));
}
```

### `backend/src/app/api_support/telegram_capability_catalog_foundation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/telegram_capability_catalog_foundation.rs`
- Size bytes / Размер в байтах: `13317`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::telegram_capabilities::{
    TelegramActionClass, TelegramCapabilityState, TelegramOperationCapability,
};

pub(super) fn push_foundation_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_account = "account";
    capabilities.push(TelegramOperationCapability::new(
        "account.create_user",
        cat_account,
        TelegramCapabilityState::Available,
        TelegramActionClass::SecretAccess,
        "User account setup with host-vault secret binding is available.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "account.create_bot",
        cat_account,
        TelegramCapabilityState::Available,
        TelegramActionClass::SecretAccess,
        "Bot account metadata and bot-token secret binding are available.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "account.list",
        cat_account,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Account list with lifecycle state and runtime mode is available.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "account.logout",
        cat_account,
        TelegramCapabilityState::Available,
        TelegramActionClass::Destructive,
        "Account logout stops runtime actor and marks lifecycle state.",
        true,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "account.remove",
        cat_account,
        TelegramCapabilityState::Available,
        TelegramActionClass::Destructive,
        "Account removal preserves local evidence, marks account removed and stops runtime.",
        true,
        false,
    ));

    let cat_runtime = "runtime";
    capabilities.push(TelegramOperationCapability::new(
        "runtime.fixture",
        cat_runtime,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Fixture runtime is available for CI and local smoke validation.",
        false,
        true,
    ));
    let tdlib_live = if qr_ready {
        TelegramCapabilityState::Available
    } else {
        TelegramCapabilityState::Blocked
    };
    let tdlib_reason = if qr_ready {
        "TDLib QR login runtime is configured for local development."
    } else {
        "Live TDLib sessions require native TDLib JSON runtime and Telegram app credentials."
    };
    capabilities.push(TelegramOperationCapability::new(
        "runtime.tdlib_live",
        cat_runtime,
        tdlib_live,
        TelegramActionClass::Read,
        tdlib_reason,
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "runtime.bot_live",
        cat_runtime,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Read,
        "Bot API runtime is deferred to the separate Bot Runtime initiative.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "runtime.status",
        cat_runtime,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Account-scoped runtime status is available.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "runtime.stop",
        cat_runtime,
        TelegramCapabilityState::Available,
        TelegramActionClass::LocalWrite,
        "Account-scoped runtime actor stop is available and preserves local evidence.",
        true,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "runtime.restart",
        cat_runtime,
        TelegramCapabilityState::Available,
        TelegramActionClass::LocalWrite,
        "Account-scoped runtime actor restart is available and preserves local evidence.",
        true,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "runtime.health_details",
        cat_runtime,
        TelegramCapabilityState::Blocked,
        TelegramActionClass::Read,
        "Detailed TDLib/native dependency health diagnostics are not yet implemented.",
        false,
        false,
    ));

    let cat_auth = "authorization";
    push_auth_capabilities(capabilities, cat_auth, qr_ready);
    push_session_and_sync_capabilities(capabilities, qr_ready);
    push_dialog_capabilities(capabilities, qr_ready);
}

fn push_auth_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    cat_auth: &str,
    qr_ready: bool,
) {
    capabilities.push(TelegramOperationCapability::new(
        "auth.qr_start",
        cat_auth,
        qr_state(qr_ready),
        TelegramActionClass::SecretAccess,
        if qr_ready {
            "QR login start is available."
        } else {
            "QR login requires native TDLib and app credentials."
        },
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "auth.qr_status",
        cat_auth,
        qr_state(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "QR status polling is available."
        } else {
            "QR status requires native TDLib and app credentials."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "auth.qr_password",
        cat_auth,
        qr_state(qr_ready),
        TelegramActionClass::SecretAccess,
        if qr_ready {
            "2FA password submission is available."
        } else {
            "2FA submission requires native TDLib and app credentials."
        },
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "auth.qr_cancel",
        cat_auth,
        qr_state(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "QR login cancellation is available."
        } else {
            "QR cancel requires native TDLib and app credentials."
        },
        false,
        false,
    ));
}

fn push_session_and_sync_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_session = "session";
    capabilities.push(TelegramOperationCapability::new(
        "session.import",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::SecretAccess,
        "Session import is deferred to a separate encrypted session-portability initiative.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "session.export",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::Export,
        "Session export is deferred to a separate encrypted session-portability initiative.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "proxy.configure",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::SecretAccess,
        "Proxy profiles are deferred to separate MTProxy/SOCKS5 runtime work.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "proxy.mtproxy",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::SecretAccess,
        "MTProxy support is deferred to a separate proxy initiative.",
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "proxy.socks5",
        cat_session,
        TelegramCapabilityState::Planned,
        TelegramActionClass::SecretAccess,
        "SOCKS5 support is deferred to a separate proxy initiative.",
        false,
        true,
    ));

    let cat_sync = "sync";
    capabilities.push(TelegramOperationCapability::new(
        "sync.chats",
        cat_sync,
        sync_state(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "Chat sync through TDLib runtime is available."
        } else {
            "Chat sync limited to fixture runtime."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "sync.history_latest",
        cat_sync,
        sync_state(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "Latest history sync is available."
        } else {
            "History sync limited to fixture runtime."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "sync.history_older",
        cat_sync,
        sync_state(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "Older history pagination is available."
        } else {
            "Older history pagination limited to fixture runtime."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "sync.history_full",
        cat_sync,
        sync_state(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "Full history sync is available."
        } else {
            "Full history sync limited to fixture runtime."
        },
        false,
        false,
    ));
}

fn push_dialog_capabilities(capabilities: &mut Vec<TelegramOperationCapability>, qr_ready: bool) {
    let cat_dialogs = "dialogs";
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.list",
        cat_dialogs,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Projected chat list is available.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.pin",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Dialog pin/unpin uses the durable provider-write outbox and provider-observed TDLib chat-position reconciliation.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.archive",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Dialog archive/unarchive uses the durable provider-write outbox and provider-observed TDLib chat-position reconciliation.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.mute",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Dialog mute/unmute uses the durable provider-write outbox and provider-observed TDLib notification-settings reconciliation.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.unread_counters",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::Read,
        "Projected unread and mention counters are available and provider-observed TDLib chat-state updates reconcile them into shared chat metadata.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.mark_read",
        cat_dialogs,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Unsupported
        },
        TelegramActionClass::ProviderWrite,
        "Mark-read uses the durable provider-write outbox and provider-observed TDLib read-inbox reconciliation.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.folder_add",
        cat_dialogs,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "Adding a chat to a Telegram folder uses the durable provider-write outbox and TDLib chat-position reconciliation for the target folder.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "dialogs.folder_remove",

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/api_support/telegram_capability_catalog_messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/telegram_capability_catalog_messages.rs`
- Size bytes / Размер в байтах: `11978`
- Included characters / Включено символов: `11978`
- Truncated / Обрезано: `no`

```rust
use super::telegram_capabilities::{
    TelegramActionClass, TelegramCapabilityState, TelegramOperationCapability,
};

pub(super) fn push_message_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    push_message_read_capabilities(capabilities);
    push_message_write_capabilities(capabilities, qr_ready);
    push_reply_forward_capabilities(capabilities, qr_ready);
    push_reaction_capabilities(capabilities, qr_ready);
    push_participant_capabilities(capabilities, qr_ready);
    push_topic_capabilities(capabilities, qr_ready);
}

fn push_message_read_capabilities(capabilities: &mut Vec<TelegramOperationCapability>) {
    let cat_msg_read = "messages:read";
    capabilities.push(TelegramOperationCapability::new(
        "messages.list",
        cat_msg_read,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Projected message list is available.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.get_versions",
        cat_msg_read,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Observed Telegram edit versions are available through the lifecycle history endpoints.",
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.get_raw_evidence",
        cat_msg_read,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "Sanitized raw provider evidence view is available.",
        false,
        false,
    ));
}

fn push_message_write_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_msg_write = "messages:write";
    capabilities.push(TelegramOperationCapability::new(
        "messages.send_text",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Manual text send through TDLib QR runtime is available."
        } else {
            "Manual text send limited to fixture runtime."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.send_media",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Provider-side media upload/send is available through local attachment import and durable outbox."
        } else {
            "Media upload/send requires TDLib QR runtime, local attachment import and durable outbox."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.edit",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Append-only edit history and TDLib provider edit execution are available through the command executor."
        } else {
            "Append-only edit history is available locally; provider edit execution requires TDLib QR runtime."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.delete",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::Destructive,
        if qr_ready {
            "Tombstone recording and TDLib provider delete execution are available through the command executor."
        } else {
            "Tombstone recording is available locally; provider delete execution requires TDLib QR runtime."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.restore_visibility",
        cat_msg_write,
        TelegramCapabilityState::Available,
        TelegramActionClass::LocalWrite,
        "Local visibility restore writes tombstone history and command audit evidence.",
        true,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.mark_read",
        cat_msg_write,
        if qr_ready {
            TelegramCapabilityState::Degraded
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Provider mark-read uses TDLib viewMessages and now has dedicated message-level API/UI, but mark-unread symmetry and richer read-history remain incomplete."
        } else {
            "Provider mark-read requires TDLib QR runtime; mark-unread symmetry and richer read-history remain incomplete."
        },
        true,
        true,
    ));
}

fn push_reply_forward_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_reply = "replies_forwards";
    capabilities.push(TelegramOperationCapability::new(
        "messages.reply",
        cat_reply,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "TDLib reply send is available for QR-authorized user accounts.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.forward",
        cat_reply,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        "TDLib forward send is available for QR-authorized user accounts.",
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "messages.pin",
        cat_reply,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Local pin projection and TDLib provider pin execution are available through the command executor."
        } else {
            "Local pin projection is available; provider pin execution requires TDLib QR runtime."
        },
        true,
        true,
    ));
}

fn push_reaction_capabilities(capabilities: &mut Vec<TelegramOperationCapability>, qr_ready: bool) {
    let cat_reactions = "reactions";
    capabilities.push(TelegramOperationCapability::new(
        "reactions.add",
        cat_reactions,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Local reaction projection and TDLib provider reaction execution are available through the command executor."
        } else {
            "Local reaction projection is available; provider reaction execution requires TDLib QR runtime."
        },
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "reactions.remove",
        cat_reactions,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Local reaction removal and TDLib provider reaction execution are available through the command executor."
        } else {
            "Local reaction removal is available; provider reaction execution requires TDLib QR runtime."
        },
        false,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "reactions.sync",
        cat_reactions,
        TelegramCapabilityState::Available,
        TelegramActionClass::Read,
        "TDLib interaction updates and history sync project provider reaction aggregates and reconcile self reaction commands.",
        false,
        false,
    ));
}

fn push_participant_capabilities(
    capabilities: &mut Vec<TelegramOperationCapability>,
    qr_ready: bool,
) {
    let cat_participants = "participants";
    capabilities.push(TelegramOperationCapability::new(
        "participants.sync",
        cat_participants,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::Read,
        if qr_ready {
            "TDLib provider member roster sync is available for supergroups/channels with recent-member pagination plus administrator snapshots, for basic groups through getBasicGroup/getBasicGroupFullInfo, and for private/saved-message chats through TDLib chat metadata."
        } else {
            "Provider member roster sync requires TDLib QR runtime."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "participants.join",
        cat_participants,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "TDLib chat join is available through the durable provider-write outbox with roster/service-message reconciliation."
        } else {
            "Chat join requires TDLib QR runtime and provider reconciliation."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "participants.leave",
        cat_participants,
        provider_or_unsupported(qr_ready),
        TelegramActionClass::Destructive,
        if qr_ready {
            "TDLib chat leave is available through the durable provider-write outbox with service-message and inactive-roster reconciliation."
        } else {
            "Chat leave requires TDLib QR runtime and provider reconciliation."
        },
        true,
        true,
    ));
}

fn push_topic_capabilities(capabilities: &mut Vec<TelegramOperationCapability>, qr_ready: bool) {
    let cat_topics = "topics";
    capabilities.push(TelegramOperationCapability::new(
        "topics.list",
        cat_topics,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Degraded
        },
        TelegramActionClass::Read,
        if qr_ready {
            "Topic projection, topic search and topic-scoped timeline reads are available."
        } else {
            "Topic projection reads are available from local state, but live TDLib refresh requires QR-authorized runtime."
        },
        false,
        false,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "topics.create",
        cat_topics,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Topic create uses the durable provider-write outbox and TDLib forum topic creation."
        } else {
            "Topic create requires TDLib QR runtime before provider-write execution is available."
        },
        true,
        true,
    ));
    capabilities.push(TelegramOperationCapability::new(
        "topics.close",
        cat_topics,
        if qr_ready {
            TelegramCapabilityState::Available
        } else {
            TelegramCapabilityState::Blocked
        },
        TelegramActionClass::ProviderWrite,
        if qr_ready {
            "Topic close/reopen uses the durable provider-write outbox and provider-observed reconciliation."
        } else {
            "Topic close/reopen requires TDLib QR runtime before provider-write execution is available."
        },
        true,
        true,
    ));
}

fn provider_or_unsupported(qr_ready: bool) -> TelegramCapabilityState {
    if qr_ready {
        TelegramCapabilityState::Available
    } else {
        TelegramCapabilityState::Unsupported
    }
}
```

### `backend/src/app/api_support/telegram_capability_catalog_tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/telegram_capability_catalog_tests.rs`
- Size bytes / Размер в байтах: `2964`
- Included characters / Включено символов: `2964`
- Truncated / Обрезано: `no`

```rust
use super::super::telegram_capabilities::TelegramCapabilityState;
use super::super::telegram_capability_catalog::telegram_capability_rows;

fn capability<'a>(
    capabilities: &'a [super::TelegramOperationCapability],
    operation: &str,
) -> &'a super::TelegramOperationCapability {
    capabilities
        .iter()
        .find(|item| item.operation == operation)
        .expect("capability exists")
}

#[test]
fn deferred_telegram_initiatives_are_api_visible_planned_capabilities() {
    assert_eq!(TelegramCapabilityState::Planned.as_str(), "planned");

    let capabilities = telegram_capability_rows(false);
    for operation in [
        "runtime.bot_live",
        "voice.record",
        "voice.send",
        "video.record",
        "calls.live_control",
        "session.import",
        "session.export",
        "proxy.mtproxy",
        "proxy.socks5",
        "ai.summary",
        "ai.translation",
        "ai.bilingual_reply",
        "ai.review_flows",
    ] {
        let capability = capability(&capabilities, operation);
        assert_eq!(capability.status, "planned", "{operation}");
    }
}

#[test]
fn qr_ready_dialog_capabilities_reflect_provider_write_reconciliation() {
    let capabilities = telegram_capability_rows(true);

    let pin = capability(&capabilities, "dialogs.pin");
    assert_eq!(pin.status, "available");
    assert_eq!(pin.action_class, "provider_write");

    let archive = capability(&capabilities, "dialogs.archive");
    assert_eq!(archive.status, "available");
    assert_eq!(archive.action_class, "provider_write");

    let mute = capability(&capabilities, "dialogs.mute");
    assert_eq!(mute.status, "available");
    assert_eq!(mute.action_class, "provider_write");

    let unread_counters = capability(&capabilities, "dialogs.unread_counters");
    assert_eq!(unread_counters.status, "available");
    assert_eq!(unread_counters.action_class, "read");

    let mark_read = capability(&capabilities, "dialogs.mark_read");
    assert_eq!(mark_read.status, "available");
    assert_eq!(mark_read.action_class, "provider_write");

    let mark_unread = capability(&capabilities, "dialogs.mark_unread");
    assert_eq!(mark_unread.status, "available");
    assert_eq!(mark_unread.action_class, "provider_write");

    let reaction_sync = capability(&capabilities, "reactions.sync");
    assert_eq!(reaction_sync.status, "available");
    assert_eq!(reaction_sync.action_class, "read");
}

#[test]
fn qr_ready_search_and_media_capabilities_reflect_projection_backed_provider_refresh() {
    let capabilities = telegram_capability_rows(true);

    for operation in [
        "search.provider",
        "search.media",
        "media.gallery",
        "media.preview",
        "voice.playback",
    ] {
        let capability = capability(&capabilities, operation);
        assert_eq!(capability.status, "available", "{operation}");
        assert_eq!(capability.action_class, "read", "{operation}");
    }
}
```

### `backend/src/app/api_support/whatsapp_capabilities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/whatsapp_capabilities.rs`
- Size bytes / Размер в байтах: `16711`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde::{Deserialize, Serialize};

use crate::application::provider_runtime_contracts::{
    WhatsAppProviderRuntimeShape, WhatsAppRuntimeStatus,
};

use super::whatsapp_capability_catalog::{
    is_whatsapp_business_cloud_personal_capability,
    is_whatsapp_business_cloud_personal_observe_capability,
    is_whatsapp_business_platform_capability, is_whatsapp_provider_write_capability,
    is_whatsapp_runtime_observe_capability, provider_shape_summary_reason,
    provider_shape_summary_status, whatsapp_capability_rows,
};

// ---------------------------------------------------------------------------
// WhatsApp capability model (unchanged)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WhatsAppCapabilityState {
    Available,
    Blocked,
    Degraded,
    Planned,
    Unsupported,
}

impl WhatsAppCapabilityState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Planned => "planned",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WhatsAppActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    SecretAccess,
}

impl WhatsAppActionClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::LocalWrite => "local_write",
            Self::ProviderWrite => "provider_write",
            Self::Destructive => "destructive",
            Self::SecretAccess => "secret_access",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WhatsappProviderShapeStatus {
    pub(crate) provider_shape: String,
    pub(crate) status: String,
    pub(crate) reason: String,
}

impl WhatsappProviderShapeStatus {
    pub(crate) fn new(
        provider_shape: WhatsAppProviderRuntimeShape,
        status: WhatsAppCapabilityState,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            provider_shape: provider_shape.as_str().to_owned(),
            status: status.as_str().to_owned(),
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WhatsappCapabilityAccountScope {
    pub(crate) account_id: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_shape: String,
    pub(crate) runtime_kind: String,
    pub(crate) lifecycle_state: String,
    pub(crate) live_runtime_available: bool,
    pub(crate) live_send_available: bool,
    pub(crate) media_download_available: bool,
    pub(crate) media_upload_available: bool,
}

impl WhatsappCapabilityAccountScope {
    fn from_runtime_status(status: &WhatsAppRuntimeStatus) -> Self {
        Self {
            account_id: status.account_id.clone(),
            provider_kind: status.provider_kind.clone(),
            provider_shape: status.provider_shape.clone(),
            runtime_kind: status.runtime_kind.clone(),
            lifecycle_state: status.status.clone(),
            live_runtime_available: status.live_runtime_available,
            live_send_available: status.live_send_available,
            media_download_available: status.media_download_available,
            media_upload_available: status.media_upload_available,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: String,
    pub(crate) provider_shapes: Vec<WhatsappProviderShapeStatus>,
    pub(crate) account_scope: Option<WhatsappCapabilityAccountScope>,
    pub(crate) capabilities: Vec<WhatsappCapabilityStatus>,
    pub(crate) planned_features: Vec<&'static str>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

impl WhatsappCapabilitiesResponse {
    pub(crate) fn current(runtime_shape: WhatsAppProviderRuntimeShape) -> Self {
        Self::build(runtime_shape, None)
    }

    pub(crate) fn current_for_account(status: &WhatsAppRuntimeStatus) -> Self {
        let runtime_shape = match status.provider_shape.as_str() {
            "whatsapp_native_md" => WhatsAppProviderRuntimeShape::NativeMultiDevice,
            "whatsapp_business_cloud" => WhatsAppProviderRuntimeShape::BusinessCloud,
            _ => WhatsAppProviderRuntimeShape::WebCompanion,
        };
        Self::build(runtime_shape, Some(status))
    }

    fn build(
        runtime_shape: WhatsAppProviderRuntimeShape,
        status: Option<&WhatsAppRuntimeStatus>,
    ) -> Self {
        let account_scope = status.map(WhatsappCapabilityAccountScope::from_runtime_status);
        let runtime_mode = status
            .map(|item| item.runtime_kind.clone())
            .unwrap_or_else(|| "fixture".to_owned());
        let mut response = Self {
            version: "2.0",
            runtime_mode,
            provider_shapes: vec![
                WhatsappProviderShapeStatus::new(
                    WhatsAppProviderRuntimeShape::WebCompanion,
                    provider_shape_summary_status(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::WebCompanion,
                    ),
                    provider_shape_summary_reason(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::WebCompanion,
                    ),
                ),
                WhatsappProviderShapeStatus::new(
                    WhatsAppProviderRuntimeShape::NativeMultiDevice,
                    provider_shape_summary_status(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::NativeMultiDevice,
                    ),
                    provider_shape_summary_reason(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::NativeMultiDevice,
                    ),
                ),
                WhatsappProviderShapeStatus::new(
                    WhatsAppProviderRuntimeShape::BusinessCloud,
                    provider_shape_summary_status(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::BusinessCloud,
                    ),
                    provider_shape_summary_reason(
                        runtime_shape,
                        WhatsAppProviderRuntimeShape::BusinessCloud,
                    ),
                ),
            ],
            account_scope,
            capabilities: whatsapp_capability_rows(),
            planned_features: vec![
                "live_runtime_execution",
                "native_md_runtime",
                "business_cloud_runtime",
                "live_media_transfer_progress",
                "live_presence_feed",
                "live_call_feed",
                "live_status_feed",
                "manual_smoke_test_checklist",
            ],
            unsupported_features: vec![
                "hidden_web_scraping",
                "bulk_messaging",
                "auto_messaging",
                "auto_dialing",
                "whatsapp_data_fine_tuning",
                "whatsapp_business_cloud_as_personal_provider",
            ],
        };
        response.apply_account_scope_overrides();
        response
    }

    fn apply_account_scope_overrides(&mut self) {
        let Some(scope) = self.account_scope.as_ref() else {
            return;
        };
        let lifecycle_state = scope.lifecycle_state.as_str();
        let runtime_kind = scope.runtime_kind.as_str();
        let provider_shape = scope.provider_shape.as_str();

        for capability in &mut self.capabilities {
            match capability.capability.as_str() {
                "runtime.fixture" if runtime_kind != "fixture" => {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "This account does not use the fixture-only WhatsApp runtime.".to_owned();
                }
                "auth.qr_link_start" if provider_shape == "whatsapp_business_cloud" => {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Business Cloud accounts do not use owner QR pairing.".to_owned();
                }
                "auth.pair_code_link_start" if provider_shape == "whatsapp_business_cloud" => {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Business Cloud accounts do not use pair-code linking.".to_owned();
                }
                capability_name
                    if provider_shape == "whatsapp_business_cloud"
                        && is_whatsapp_business_cloud_personal_capability(capability_name) =>
                {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason = "Business Cloud accounts do not expose personal WhatsApp chat/runtime operations."
                        .to_owned();
                }
                capability_name
                    if provider_shape == "whatsapp_business_cloud"
                        && is_whatsapp_business_cloud_personal_observe_capability(
                            capability_name,
                        ) =>
                {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason = "Business Cloud accounts do not expose companion/native WhatsApp observation and projection surfaces."
                        .to_owned();
                }
                capability_name if is_whatsapp_business_platform_capability(capability_name) => {
                    if provider_shape == "whatsapp_business_cloud" {
                        capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                        capability.reason = "Business Cloud account shape is configured, but Макошь does not execute official Business Platform operations yet."
                            .to_owned();
                    } else {
                        capability.status =
                            WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                        capability.reason =
                            "This capability is only valid for whatsapp_business_cloud.".to_owned();
                    }
                }
                "sessions.manual_state" | "sessions.restore"
                    if provider_shape == "whatsapp_business_cloud" =>
                {
                    capability.status = WhatsAppCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Business Cloud accounts do not use companion session restore material."
                            .to_owned();
                }
                "sessions.restore" if matches!(lifecycle_state, "created" | "link_required") => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "This account has not completed WhatsApp session linking yet.".to_owned();
                }
                "sessions.restore" if lifecycle_state == "revoked" => {
                    capability.status = WhatsAppCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "This account was revoked and must be relinked before restore.".to_owned();
                }
                "sessions.restore" if lifecycle_state == "removed" => {
                    capability
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
