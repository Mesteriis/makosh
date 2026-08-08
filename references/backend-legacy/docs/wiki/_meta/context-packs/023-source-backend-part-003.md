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

- Chunk ID / ID чанка: `023-source-backend-part-003`
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

### `backend/src/ai/core/semantic/source_persons.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/semantic/source_persons.rs`
- Size bytes / Размер в байтах: `1031`
- Included characters / Включено символов: `1031`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::super::errors::AiError;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_person_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT person_id, display_name, email_address
        FROM persons
        ORDER BY updated_at DESC, person_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let person_id: String = row.try_get("person_id")?;
        let display_name: String = row.try_get("display_name")?;
        let email_address: String = row.try_get("email_address")?;
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Person,
            source_id: person_id,
            observation_id: None,
            title: display_name.clone(),
            source_text: format!("{display_name}\nEmail: {email_address}"),
            graph_node_id: None,
        });
    }

    Ok(())
}
```

### `backend/src/ai/core/semantic/source_projects.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/semantic/source_projects.rs`
- Size bytes / Размер в байтах: `1726`
- Included characters / Включено символов: `1726`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::platform::graph::{GraphNodeKind, node_id};

use super::super::errors::AiError;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_project_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT
            p.project_id,
            p.name,
            p.kind,
            p.status,
            p.description,
            p.owner_display_name,
            COALESCE(string_agg(k.keyword, ', ' ORDER BY k.keyword), '') AS keywords
        FROM projects p
        LEFT JOIN project_keywords k ON k.project_id = p.project_id
        GROUP BY p.project_id
        ORDER BY p.updated_at DESC, p.project_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let project_id: String = row.try_get("project_id")?;
        let name: String = row.try_get("name")?;
        let kind: String = row.try_get("kind")?;
        let status: String = row.try_get("status")?;
        let description: String = row.try_get("description")?;
        let owner: String = row.try_get("owner_display_name")?;
        let keywords: String = row.try_get("keywords")?;
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Project,
            source_id: project_id.clone(),
            observation_id: None,
            title: name.clone(),
            source_text: format!(
                "{name}\nKind: {kind}\nStatus: {status}\nOwner: {owner}\nKeywords: {keywords}\n\n{description}"
            ),
            graph_node_id: Some(node_id(GraphNodeKind::Project, &project_id)),
        });
    }

    Ok(())
}
```

### `backend/src/ai/core/semantic/source_tasks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/semantic/source_tasks.rs`
- Size bytes / Размер в байтах: `1134`
- Included characters / Включено символов: `1134`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::super::errors::AiError;
use super::models::{SemanticSource, SemanticSourceKind};

pub(super) async fn append_task_sources(
    pool: &PgPool,
    sources: &mut Vec<SemanticSource>,
) -> Result<(), AiError> {
    let rows = sqlx::query(
        r#"
        SELECT task_id, title, source_kind, source_id, status
        FROM tasks
        ORDER BY updated_at DESC, task_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let task_id: String = row.try_get("task_id")?;
        let title: String = row.try_get("title")?;
        let source_kind: String = row.try_get("source_kind")?;
        let source_id: String = row.try_get("source_id")?;
        let status: String = row.try_get("status")?;
        sources.push(SemanticSource {
            source_kind: SemanticSourceKind::Task,
            source_id: task_id,
            observation_id: None,
            title: title.clone(),
            source_text: format!("{title}\nStatus: {status}\nSource: {source_kind}:{source_id}"),
            graph_node_id: None,
        });
    }

    Ok(())
}
```

### `backend/src/ai/core/semantic/sources.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/semantic/sources.rs`
- Size bytes / Размер в байтах: `878`
- Included characters / Включено символов: `878`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::AiError;
use super::models::SemanticSource;
use super::source_documents::append_document_sources;
use super::source_messages::append_message_sources;
use super::source_persons::append_person_sources;
use super::source_projects::append_project_sources;
use super::source_tasks::append_task_sources;
use super::store::SemanticEmbeddingStore;

impl SemanticEmbeddingStore {
    pub(super) async fn canonical_sources(&self) -> Result<Vec<SemanticSource>, AiError> {
        let mut sources = Vec::new();

        append_message_sources(&self.pool, &mut sources).await?;
        append_document_sources(&self.pool, &mut sources).await?;
        append_project_sources(&self.pool, &mut sources).await?;
        append_task_sources(&self.pool, &mut sources).await?;
        append_person_sources(&self.pool, &mut sources).await?;

        Ok(sources)
    }
}
```

### `backend/src/ai/core/semantic/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/semantic/store.rs`
- Size bytes / Размер в байтах: `213`
- Included characters / Включено символов: `213`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct SemanticEmbeddingStore {
    pub(super) pool: PgPool,
}

impl SemanticEmbeddingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### `backend/src/ai/core/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service.rs`
- Size bytes / Размер в байтах: `366`
- Included characters / Включено символов: `366`
- Truncated / Обрезано: `no`

```rust
mod answer;
mod attribution;
mod attribution_port;
mod core;
mod events;
mod meeting_prep;
mod model_config;
mod retrieval;
mod status;
mod task_candidate_persistence;
mod task_candidates;

pub use attribution_port::{
    AiAgentPersonaAttribution, AiPersonaAttributionError, AiPersonaAttributionPort,
    SharedAiPersonaAttributionPort,
};
pub use core::AiService;
```

### `backend/src/ai/core/service/answer.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/answer.rs`
- Size bytes / Размер в байтах: `4055`
- Included characters / Включено символов: `4055`
- Truncated / Обрезано: `no`

```rust
use std::time::Instant;

use serde_json::json;

use super::super::agents::validate_agent;
use super::super::constants::AI_PROMPT_TEMPLATE_VERSION;
use super::super::errors::AiError;
use super::super::helpers::{
    elapsed_ms, event_id_from_command, run_id_from_command, validate_non_empty,
};
use super::super::prompts::answer_prompt;
use super::super::runs::{AiRunStore, NewAiRun};
use super::super::types::{AiAnswerRequest, AiAnswerResponse};
use super::core::AiService;
use super::events::AiRunEvent;

impl AiService {
    pub async fn answer(
        &self,
        request: AiAnswerRequest,
        actor_id: &str,
    ) -> Result<AiAnswerResponse, AiError> {
        let command_id = validate_non_empty("command_id", &request.command_id)?;
        let query = validate_non_empty("query", &request.query)?;
        let agent_id = request.agent_id.unwrap_or_else(|| "MNEMOSYNE".to_owned());
        validate_agent(&agent_id)?;
        let started_at = Instant::now();
        let run_id = run_id_from_command("answer", &command_id);
        let requested_event_id = event_id_from_command("ai.run.requested", &command_id);
        let completed_event_id = event_id_from_command("ai.run.completed", &command_id);
        let run_store = AiRunStore::new(self.pool.clone());
        let chat_model = self.model_routing.default_chat.clone();
        let attribution = self.run_attribution(&agent_id).await?;

        run_store
            .start_run(&NewAiRun {
                run_id: run_id.clone(),
                agent_id: agent_id.clone(),
                chat_model: chat_model.clone(),
                embedding_model: self.model_routing.embeddings.clone(),
                prompt_template_version: AI_PROMPT_TEMPLATE_VERSION.to_owned(),
                model_config: self.model_config(),
                query: query.clone(),
                actor_id: actor_id.to_owned(),
                agent_persona_id: Some(attribution.agent_persona_id.clone()),
                owner_persona_id: attribution.owner_persona_id.clone(),
                causation_id: request.causation_id.clone(),
                correlation_id: request.correlation_id.clone(),
                requested_event_id: requested_event_id.clone(),
            })
            .await?;
        self.append_run_event(AiRunEvent {
            event_id: &requested_event_id,
            event_type: "ai.run.requested",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({}),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        let citations = self.retrieve_citations(&query).await?;
        let prompt = answer_prompt(&query, &citations);
        let chat = self.runtime.chat_with_model(&prompt, &chat_model).await?;
        let duration_ms = elapsed_ms(started_at);
        let stored = run_store
            .complete_run(
                &run_id,
                &chat.content,
                &citations,
                duration_ms,
                &completed_event_id,
            )
            .await?;
        self.append_run_event(AiRunEvent {
            event_id: &completed_event_id,
            event_type: "ai.run.completed",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({
                "citation_count": citations.len(),
                "duration_ms": duration_ms,
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        Ok(AiAnswerResponse {
            run_id,
            agent_id,
            agent_persona_id: attribution.agent_persona_id,
            owner_persona_id: attribution.owner_persona_id,
            status: stored.status,
            answer: chat.content,
            citations,
            model: chat.model,
            embedding_model: self.model_routing.embeddings.clone(),
            created_at: stored.started_at,
            duration_ms,
        })
    }
}
```

### `backend/src/ai/core/service/attribution.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/attribution.rs`
- Size bytes / Размер в байтах: `904`
- Included characters / Включено символов: `904`
- Truncated / Обрезано: `no`

```rust
use super::super::agents::ai_agent_display_name;
use super::super::errors::AiError;
use super::core::AiService;

pub(super) struct AiRunAttribution {
    pub(super) agent_persona_id: String,
    pub(super) owner_persona_id: Option<String>,
}

impl AiService {
    pub(super) async fn run_attribution(
        &self,
        agent_id: &str,
    ) -> Result<AiRunAttribution, AiError> {
        let persona_attribution = self
            .persona_attribution
            .as_ref()
            .ok_or(AiError::PersonaAttributionUnavailable)?;
        let agent_persona = persona_attribution
            .upsert_ai_agent_persona(agent_id, ai_agent_display_name(agent_id)?)
            .await?;
        let owner_persona_id = persona_attribution.owner_persona_id().await?;

        Ok(AiRunAttribution {
            agent_persona_id: agent_persona.persona_id,
            owner_persona_id,
        })
    }
}
```

### `backend/src/ai/core/service/attribution_port.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/attribution_port.rs`
- Size bytes / Размер в байтах: `975`
- Included characters / Включено символов: `975`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use thiserror::Error;

pub type SharedAiPersonaAttributionPort = Arc<dyn AiPersonaAttributionPort>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiAgentPersonaAttribution {
    pub persona_id: String,
    pub persona_type: &'static str,
    pub persona_email: String,
}

#[derive(Debug, Error)]
pub enum AiPersonaAttributionError {
    #[error("AI persona attribution failed: {0}")]
    Store(String),
}

pub trait AiPersonaAttributionPort: Send + Sync {
    fn upsert_ai_agent_persona<'a>(
        &'a self,
        agent_id: &'a str,
        display_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AiAgentPersonaAttribution, AiPersonaAttributionError>>
                + Send
                + 'a,
        >,
    >;

    fn owner_persona_id<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, AiPersonaAttributionError>> + Send + 'a>>;
}
```

### `backend/src/ai/core/service/core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/core.rs`
- Size bytes / Размер в байтах: `1565`
- Included characters / Включено символов: `1565`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::integrations::ai_runtime::AiRuntimeClient;

use super::super::types::AiModelRouting;
use super::attribution_port::SharedAiPersonaAttributionPort;

#[derive(Clone)]
pub struct AiService {
    pub(super) pool: PgPool,
    pub(super) runtime: AiRuntimeClient,
    pub(super) chat_model: String,
    pub(super) embedding_model: String,
    pub(super) model_routing: AiModelRouting,
    pub(super) persona_attribution: Option<SharedAiPersonaAttributionPort>,
}

impl AiService {
    pub fn new(
        pool: PgPool,
        runtime: AiRuntimeClient,
        chat_model: impl Into<String>,
        embedding_model: impl Into<String>,
    ) -> Self {
        let chat_model = chat_model.into();
        let embedding_model = embedding_model.into();
        let model_routing = AiModelRouting::fallback(chat_model.clone(), embedding_model.clone());
        Self::new_with_routing(pool, runtime, model_routing)
    }

    pub fn new_with_routing(
        pool: PgPool,
        runtime: AiRuntimeClient,
        model_routing: AiModelRouting,
    ) -> Self {
        Self {
            pool,
            runtime,
            chat_model: model_routing.default_chat.clone(),
            embedding_model: model_routing.embeddings.clone(),
            model_routing,
            persona_attribution: None,
        }
    }

    pub fn with_persona_attribution(
        mut self,
        persona_attribution: SharedAiPersonaAttributionPort,
    ) -> Self {
        self.persona_attribution = Some(persona_attribution);
        self
    }
}
```

### `backend/src/ai/core/service/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/events.rs`
- Size bytes / Размер в байтах: `3961`
- Included characters / Включено символов: `3961`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};

use crate::application::dispatch_ai_runtime_signal;
use crate::platform::events::{EventStore, NewEventEnvelope};

use super::super::constants::AI_PROMPT_TEMPLATE_VERSION;
use super::super::errors::AiError;
use super::super::helpers::text_preview;
use super::core::AiService;

pub(super) struct AiRunEvent<'a> {
    pub(super) event_id: &'a str,
    pub(super) event_type: &'a str,
    pub(super) run_id: &'a str,
    pub(super) agent_id: &'a str,
    pub(super) actor_id: &'a str,
    pub(super) query: &'a str,
    pub(super) payload: Value,
    pub(super) correlation_id: Option<&'a str>,
}

impl AiService {
    pub(super) async fn append_run_event(&self, event: AiRunEvent<'_>) -> Result<(), AiError> {
        let event_store = EventStore::new(self.pool.clone());
        let builder = NewEventEnvelope::builder(
            event.event_id,
            event.event_type,
            Utc::now(),
            json!({
                "kind": "ai_run",
                "source_id": event.run_id,
            }),
            json!({
                "kind": "ai_run",
                "run_id": event.run_id,
                "agent_id": event.agent_id,
            }),
        )
        .actor(json!({ "actor_id": event.actor_id }))
        .payload(json!({
            "agent_id": event.agent_id,
            "query_preview": text_preview(event.query, 160),
            "details": event.payload,
        }))
        .provenance(json!({
            "runtime": self.runtime.runtime_name(),
            "chat_model": self.chat_model,
            "embedding_model": self.embedding_model,
            "prompt_template_version": AI_PROMPT_TEMPLATE_VERSION,
        }));
        let trace_id = event
            .correlation_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(event.run_id);
        let ai_event = builder.correlation_id(trace_id).build()?;
        event_store.append(&ai_event).await?;
        self.append_ai_signal_event(&event, trace_id).await?;
        Ok(())
    }

    async fn append_ai_signal_event(
        &self,
        event: &AiRunEvent<'_>,
        correlation_id: &str,
    ) -> Result<(), AiError> {
        let Some(event_kind) = ai_raw_signal_event_kind(event.event_type) else {
            return Ok(());
        };
        let _ = dispatch_ai_runtime_signal(
            self.pool.clone(),
            event_kind,
            event.run_id,
            json!({
                "kind": "ai_run",
                "source_code": "ai",
                "run_id": event.run_id,
                "agent_id": event.agent_id,
                "event_type": event.event_type,
            }),
            json!({
                "agent_id": event.agent_id,
                "workflow": event.payload.get("workflow").cloned(),
                "details": signal_safe_payload(&event.payload),
            }),
            json!({
                "source": "ai_run_event",
                "source_code": "ai",
                "runtime": self.runtime.runtime_name(),
                "chat_model": self.chat_model,
                "embedding_model": self.embedding_model,
                "prompt_template_version": AI_PROMPT_TEMPLATE_VERSION,
                "ai_event_type": event.event_type,
            }),
            Some(correlation_id),
        )
        .await?;
        Ok(())
    }
}

fn ai_raw_signal_event_kind(event_type: &str) -> Option<&'static str> {
    match event_type {
        "ai.run.requested" => Some("run_requested"),
        "ai.run.completed" => Some("run_completed"),
        "ai.task_extraction.completed" => Some("task_extraction"),
        _ => None,
    }
}

fn signal_safe_payload(payload: &Value) -> Value {
    let mut redacted = payload.clone();
    if let Some(object) = redacted.as_object_mut() {
        object.remove("query");
        object.remove("answer");
        object.remove("briefing");
    }
    redacted
}
```

### `backend/src/ai/core/service/meeting_prep.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/meeting_prep.rs`
- Size bytes / Размер в байтах: `4370`
- Included characters / Включено символов: `4370`
- Truncated / Обрезано: `no`

```rust
use std::time::Instant;

use serde_json::json;

use super::super::constants::AI_PROMPT_TEMPLATE_VERSION;
use super::super::errors::AiError;
use super::super::helpers::{
    elapsed_ms, event_id_from_command, run_id_from_command, validate_non_empty,
};
use super::super::prompts::{meeting_prep_prompt, scoped_meeting_query};
use super::super::runs::{AiRunStore, NewAiRun};
use super::super::types::{AiMeetingPrepRequest, AiMeetingPrepResponse};
use super::core::AiService;
use super::events::AiRunEvent;

impl AiService {
    pub async fn meeting_prep(
        &self,
        request: AiMeetingPrepRequest,
        actor_id: &str,
    ) -> Result<AiMeetingPrepResponse, AiError> {
        let command_id = validate_non_empty("command_id", &request.command_id)?;
        let topic = validate_non_empty("topic", &request.topic)?;
        let agent_id = "HESTIA".to_owned();
        let started_at = Instant::now();
        let run_id = run_id_from_command("meeting-prep", &command_id);
        let requested_event_id = event_id_from_command("ai.run.requested", &command_id);
        let completed_event_id = event_id_from_command("ai.run.completed", &command_id);
        let run_store = AiRunStore::new(self.pool.clone());
        let chat_model = self.model_routing.meeting_prep.clone();
        let query = scoped_meeting_query(
            &topic,
            request.project_id.as_deref(),
            request.person_id.as_deref(),
        );
        let attribution = self.run_attribution(&agent_id).await?;

        run_store
            .start_run(&NewAiRun {
                run_id: run_id.clone(),
                agent_id: agent_id.clone(),
                chat_model: chat_model.clone(),
                embedding_model: self.model_routing.embeddings.clone(),
                prompt_template_version: AI_PROMPT_TEMPLATE_VERSION.to_owned(),
                model_config: self.model_config(),
                query: query.clone(),
                actor_id: actor_id.to_owned(),
                agent_persona_id: Some(attribution.agent_persona_id.clone()),
                owner_persona_id: attribution.owner_persona_id.clone(),
                causation_id: request.causation_id.clone(),
                correlation_id: request.correlation_id.clone(),
                requested_event_id: requested_event_id.clone(),
            })
            .await?;
        self.append_run_event(AiRunEvent {
            event_id: &requested_event_id,
            event_type: "ai.run.requested",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({
                "workflow": "meeting_prep",
                "project_id": request.project_id,
                "person_id": request.person_id,
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        let citations = self.retrieve_citations(&query).await?;
        let prompt = meeting_prep_prompt(&topic, &citations);
        let chat = self.runtime.chat_with_model(&prompt, &chat_model).await?;
        let duration_ms = elapsed_ms(started_at);
        let stored = run_store
            .complete_run(
                &run_id,
                &chat.content,
                &citations,
                duration_ms,
                &completed_event_id,
            )
            .await?;
        self.append_run_event(AiRunEvent {
            event_id: &completed_event_id,
            event_type: "ai.run.completed",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({
                "workflow": "meeting_prep",
                "citation_count": citations.len(),
                "duration_ms": duration_ms,
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        Ok(AiMeetingPrepResponse {
            run_id,
            agent_id,
            agent_persona_id: attribution.agent_persona_id,
            owner_persona_id: attribution.owner_persona_id,
            status: stored.status,
            briefing: chat.content,
            citations,
            model: chat.model,
            embedding_model: self.model_routing.embeddings.clone(),
            created_at: stored.started_at,
            duration_ms,
        })
    }
}
```

### `backend/src/ai/core/service/model_config.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/model_config.rs`
- Size bytes / Размер в байтах: `1011`
- Included characters / Включено символов: `1011`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::super::constants::AI_EMBEDDING_DIMENSION;
use super::core::AiService;

impl AiService {
    pub(super) fn model_config(&self) -> Value {
        json!({
            "runtime": self.runtime.runtime_name(),
            "chat_model": &self.model_routing.default_chat,
            "embedding_model": &self.model_routing.embeddings,
            "embedding_dimension": AI_EMBEDDING_DIMENSION,
            "routes": {
                "default_chat": &self.model_routing.default_chat,
                "reasoning": &self.model_routing.reasoning,
                "summarization": &self.model_routing.summarization,
                "mail_intelligence": &self.model_routing.mail_intelligence,
                "reply_draft": &self.model_routing.reply_draft,
                "extraction": &self.model_routing.extraction,
                "embeddings": &self.model_routing.embeddings,
                "meeting_prep": &self.model_routing.meeting_prep,
            }
        })
    }
}
```

### `backend/src/ai/core/service/retrieval.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/retrieval.rs`
- Size bytes / Размер в байтах: `1645`
- Included characters / Включено символов: `1645`
- Truncated / Обрезано: `no`

```rust
use super::super::constants::{AI_EMBEDDING_DIMENSION, DEFAULT_RETRIEVAL_LIMIT};
use super::super::errors::AiError;
use super::super::helpers::merge_retrieval_results;
use super::super::semantic::SemanticEmbeddingStore;
use super::super::types::AiCitation;
use super::core::AiService;

impl AiService {
    pub(super) async fn retrieve_citations(&self, query: &str) -> Result<Vec<AiCitation>, AiError> {
        let semantic_store = SemanticEmbeddingStore::new(self.pool.clone());
        let embedding_model = &self.model_routing.embeddings;
        semantic_store
            .index_canonical_sources(&self.runtime, embedding_model)
            .await?;
        let query_embedding = self
            .runtime
            .embed_with_model(query, embedding_model)
            .await?;
        if query_embedding.embedding.len() != AI_EMBEDDING_DIMENSION {
            return Err(AiError::InvalidEmbeddingDimension {
                expected: AI_EMBEDDING_DIMENSION,
                actual: query_embedding.embedding.len(),
            });
        }

        let vector_results = semantic_store
            .search(
                embedding_model,
                &query_embedding.embedding,
                DEFAULT_RETRIEVAL_LIMIT,
            )
            .await?;
        let text_results = semantic_store
            .text_search(embedding_model, query, DEFAULT_RETRIEVAL_LIMIT)
            .await?;
        let merged = merge_retrieval_results(vector_results, text_results);

        Ok(merged
            .into_iter()
            .take(DEFAULT_RETRIEVAL_LIMIT as usize)
            .map(AiCitation::from)
            .collect())
    }
}
```

### `backend/src/ai/core/service/status.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/status.rs`
- Size bytes / Размер в байтах: `1528`
- Included characters / Включено символов: `1528`
- Truncated / Обрезано: `no`

```rust
use super::super::constants::AI_EMBEDDING_DIMENSION;
use super::super::types::AiStatusResponse;
use super::core::AiService;

impl AiService {
    pub async fn status(&self) -> AiStatusResponse {
        let version = self.runtime.version().await;
        let models = self.runtime.models().await;
        let chat_model_available = models
            .as_ref()
            .map(|models| {
                models
                    .iter()
                    .any(|model| model == &self.model_routing.default_chat)
            })
            .unwrap_or(false);
        let embedding_model_available = models
            .as_ref()
            .map(|models| {
                models
                    .iter()
                    .any(|model| model == &self.model_routing.embeddings)
            })
            .unwrap_or(false);

        AiStatusResponse {
            runtime: self.runtime.runtime_name().to_owned(),
            status: if version.is_ok()
                && models.is_ok()
                && chat_model_available
                && embedding_model_available
            {
                "ok"
            } else {
                "unavailable"
            }
            .to_owned(),
            version: version.ok().flatten(),
            chat_model: self.model_routing.default_chat.clone(),
            embedding_model: self.model_routing.embeddings.clone(),
            embedding_dimension: AI_EMBEDDING_DIMENSION,
            chat_model_available,
            embedding_model_available,
        }
    }
}
```

### `backend/src/ai/core/service/task_candidate_persistence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/task_candidate_persistence.rs`
- Size bytes / Размер в байтах: `4964`
- Included characters / Включено символов: `4964`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::AiError;
use super::super::helpers::{ai_task_candidate_id, validate_non_empty};
use super::super::prompts::{AiTaskCandidateDraft, citation_for_draft};
use super::super::types::AiCitation;
use super::core::AiService;

impl AiService {
    pub(super) async fn upsert_ai_task_candidates(
        &self,
        run_id: &str,
        drafts: &[AiTaskCandidateDraft],
        citations: &[AiCitation],
    ) -> Result<i64, AiError> {
        let mut created_count = 0;
        for draft in drafts {
            let Some(citation) = citation_for_draft(draft, citations) else {
                continue;
            };
            if citation.source_kind != "message" && citation.source_kind != "document" {
                continue;
            }
            let title = validate_non_empty("title", &draft.title)?;
            let evidence_excerpt = draft
                .evidence_excerpt
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(citation.excerpt.as_str());
            let confidence = draft.confidence.unwrap_or(0.5).clamp(0.0, 1.0);
            let observation_id = task_candidate_observation_id(self, citation).await?;
            let Some(observation_id) = observation_id else {
                continue;
            };
            let task_candidate_id = ai_task_candidate_id("observation", &observation_id, &title);

            let result = sqlx::query(
                r#"
                INSERT INTO task_candidates (
                    task_candidate_id,
                    source_kind,
                    source_id,
                    observation_id,
                    project_id,
                    title,
                    due_text,
                    assignee_label,
                    confidence,
                    review_state,
                    evidence_excerpt,
                    event_id,
                    actor_id,
                    reviewed_at,
                    agent_run_id
                )
                VALUES (
                    $1, $2, $3, $4, NULL, $5, $6, $7, $8, 'suggested', $9, NULL, NULL, NULL, $10
                )
                ON CONFLICT (source_kind, source_id, lower(title))
                DO UPDATE SET
                    observation_id = COALESCE(EXCLUDED.observation_id, task_candidates.observation_id),
                    title = EXCLUDED.title,
                    due_text = COALESCE(EXCLUDED.due_text, task_candidates.due_text),
                    assignee_label = COALESCE(EXCLUDED.assignee_label, task_candidates.assignee_label),
                    confidence = EXCLUDED.confidence,
                    review_state = CASE
                        WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                            THEN task_candidates.review_state
                        ELSE 'suggested'
                    END,
                    evidence_excerpt = EXCLUDED.evidence_excerpt,
                    agent_run_id = CASE
                        WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                            THEN task_candidates.agent_run_id
                        ELSE EXCLUDED.agent_run_id
                    END,
                    updated_at = now()
                "#,
            )
            .bind(task_candidate_id)
            .bind("observation")
            .bind(&observation_id)
            .bind(&observation_id)
            .bind(&title)
            .bind(&draft.due_text)
            .bind(&draft.assignee_label)
            .bind(confidence)
            .bind(evidence_excerpt)
            .bind(run_id)
            .execute(&self.pool)
            .await?;

            if result.rows_affected() > 0 {
                created_count += 1;
            }
        }

        Ok(created_count)
    }
}

async fn task_candidate_observation_id(
    service: &AiService,
    citation: &AiCitation,
) -> Result<Option<String>, AiError> {
    match citation.source_kind.as_str() {
        "message" => {
            let observation_id = sqlx::query_scalar::<_, Option<String>>(
                r#"
                SELECT observation_id
                FROM communication_messages
                WHERE message_id = $1
                "#,
            )
            .bind(&citation.source_id)
            .fetch_optional(&service.pool)
            .await?
            .flatten();
            Ok(observation_id)
        }
        "document" => {
            let observation_id = sqlx::query_scalar::<_, Option<String>>(
                r#"
                SELECT observation_id
                FROM documents
                WHERE document_id = $1
                "#,
            )
            .bind(&citation.source_id)
            .fetch_optional(&service.pool)
            .await?
            .flatten();
            Ok(observation_id)
        }
        _ => Ok(None),
    }
}
```

### `backend/src/ai/core/service/task_candidates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/service/task_candidates.rs`
- Size bytes / Размер в байтах: `5185`
- Included characters / Включено символов: `5185`
- Truncated / Обрезано: `no`

```rust
use std::time::Instant;

use serde_json::json;

use super::super::constants::AI_PROMPT_TEMPLATE_VERSION;
use super::super::errors::AiError;
use super::super::helpers::{
    elapsed_ms, event_id_from_command, run_id_from_command, validate_non_empty,
};
use super::super::prompts::{parse_task_candidate_drafts, task_candidate_prompt};
use super::super::runs::{AiRunStore, NewAiRun};
use super::super::types::{AiTaskCandidateRefreshRequest, AiTaskCandidateRefreshResponse};
use super::core::AiService;
use super::events::AiRunEvent;
use crate::application::review_inbox::sync_ai_run_task_candidates_to_review;

impl AiService {
    pub async fn refresh_task_candidates(
        &self,
        request: AiTaskCandidateRefreshRequest,
        actor_id: &str,
    ) -> Result<AiTaskCandidateRefreshResponse, AiError> {
        let command_id = validate_non_empty("command_id", &request.command_id)?;
        let query = validate_non_empty("query", &request.query)?;
        let agent_id = "MAKOSH".to_owned();
        let started_at = Instant::now();
        let run_id = run_id_from_command("task-refresh", &command_id);
        let requested_event_id = event_id_from_command("ai.run.requested", &command_id);
        let completed_event_id = event_id_from_command("ai.run.completed", &command_id);
        let extraction_event_id =
            event_id_from_command("ai.task_extraction.completed", &command_id);
        let run_store = AiRunStore::new(self.pool.clone());
        let chat_model = self.model_routing.extraction.clone();
        let attribution = self.run_attribution(&agent_id).await?;

        run_store
            .start_run(&NewAiRun {
                run_id: run_id.clone(),
                agent_id: agent_id.clone(),
                chat_model: chat_model.clone(),
                embedding_model: self.model_routing.embeddings.clone(),
                prompt_template_version: AI_PROMPT_TEMPLATE_VERSION.to_owned(),
                model_config: self.model_config(),
                query: query.clone(),
                actor_id: actor_id.to_owned(),
                agent_persona_id: Some(attribution.agent_persona_id.clone()),
                owner_persona_id: attribution.owner_persona_id.clone(),
                causation_id: request.causation_id.clone(),
                correlation_id: request.correlation_id.clone(),
                requested_event_id: requested_event_id.clone(),
            })
            .await?;
        self.append_run_event(AiRunEvent {
            event_id: &requested_event_id,
            event_type: "ai.run.requested",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({ "workflow": "task_candidates" }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        let citations = self.retrieve_citations(&query).await?;
        let prompt = task_candidate_prompt(&query, &citations);
        let chat = self.runtime.chat_with_model(&prompt, &chat_model).await?;
        let drafts = parse_task_candidate_drafts(&chat.content, &citations)?;
        let created_count = self
            .upsert_ai_task_candidates(&run_id, &drafts, &citations)
            .await?;
        let _ = sync_ai_run_task_candidates_to_review(&self.pool, &run_id).await?;
        let duration_ms = elapsed_ms(started_at);
        let answer = format!("Created {created_count} suggested task candidate(s).");
        let stored = run_store
            .complete_run(
                &run_id,
                &answer,
                &citations,
                duration_ms,
                &completed_event_id,
            )
            .await?;
        self.append_run_event(AiRunEvent {
            event_id: &completed_event_id,
            event_type: "ai.run.completed",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({
                "workflow": "task_candidates",
                "created_count": created_count,
                "duration_ms": duration_ms,
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;
        self.append_run_event(AiRunEvent {
            event_id: &extraction_event_id,
            event_type: "ai.task_extraction.completed",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({
                "created_count": created_count,
                "candidate_state": "suggested",
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        Ok(AiTaskCandidateRefreshResponse {
            run_id,
            agent_id,
            agent_persona_id: attribution.agent_persona_id,
            owner_persona_id: attribution.owner_persona_id,
            status: stored.status,
            created_count,
            citations,
            model: chat.model,
            embedding_model: self.model_routing.embeddings.clone(),
            created_at: stored.started_at,
            duration_ms,
        })
    }
}
```

### `backend/src/ai/core/types.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/core/types.rs`
- Size bytes / Размер в байтах: `4196`
- Included characters / Включено символов: `4196`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::constants::AI_EMBEDDING_DIMENSION;
use super::helpers::text_preview;
use super::semantic::SemanticSearchResult;

#[derive(Clone)]
pub struct AiModelRouting {
    pub default_chat: String,
    pub reasoning: String,
    pub summarization: String,
    pub mail_intelligence: String,
    pub reply_draft: String,
    pub extraction: String,
    pub embeddings: String,
    pub meeting_prep: String,
}

impl AiModelRouting {
    pub fn fallback(chat_model: impl Into<String>, embedding_model: impl Into<String>) -> Self {
        let chat_model = chat_model.into();
        let embedding_model = embedding_model.into();
        Self {
            default_chat: chat_model.clone(),
            reasoning: chat_model.clone(),
            summarization: chat_model.clone(),
            mail_intelligence: chat_model.clone(),
            reply_draft: chat_model.clone(),
            extraction: chat_model.clone(),
            embeddings: embedding_model,
            meeting_prep: chat_model,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AiAnswerRequest {
    pub command_id: String,
    pub query: String,
    pub agent_id: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AiTaskCandidateRefreshRequest {
    pub command_id: String,
    pub query: String,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AiMeetingPrepRequest {
    pub command_id: String,
    pub topic: String,
    pub project_id: Option<String>,
    pub person_id: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiCitation {
    pub source_kind: String,
    pub source_id: String,
    pub title: String,
    pub excerpt: String,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_node_id: Option<String>,
}

impl From<SemanticSearchResult> for AiCitation {
    fn from(result: SemanticSearchResult) -> Self {
        Self {
            source_kind: result.source_kind,
            source_id: result.source_id,
            title: result.title,
            excerpt: text_preview(&result.source_text, 320),
            score: result.score,
            graph_node_id: result.graph_node_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AiAnswerResponse {
    pub run_id: String,
    pub agent_id: String,
    pub agent_persona_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_persona_id: Option<String>,
    pub status: String,
    pub answer: String,
    pub citations: Vec<AiCitation>,
    pub model: String,
    pub embedding_model: String,
    pub created_at: DateTime<Utc>,
    pub duration_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiTaskCandidateRefreshResponse {
    pub run_id: String,
    pub agent_id: String,
    pub agent_persona_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_persona_id: Option<String>,
    pub status: String,
    pub created_count: i64,
    pub citations: Vec<AiCitation>,
    pub model: String,
    pub embedding_model: String,
    pub created_at: DateTime<Utc>,
    pub duration_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiMeetingPrepResponse {
    pub run_id: String,
    pub agent_id: String,
    pub agent_persona_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_persona_id: Option<String>,
    pub status: String,
    pub briefing: String,
    pub citations: Vec<AiCitation>,
    pub model: String,
    pub embedding_model: String,
    pub created_at: DateTime<Utc>,
    pub duration_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiStatusResponse {
    pub runtime: String,
    pub status: String,
    pub version: Option<String>,
    pub chat_model: String,
    pub embedding_model: String,
    pub embedding_dimension: usize,
    pub chat_model_available: bool,
    pub embedding_model_available: bool,
}
```

### `backend/src/ai/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/mod.rs`
- Size bytes / Размер в байтах: `51`
- Included characters / Включено символов: `51`
- Truncated / Обрезано: `no`

```rust
pub mod api;
pub mod control_center;
pub mod core;
```

### `backend/src/app/api_support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support.rs`
- Size bytes / Размер в байтах: `8683`
- Included characters / Включено символов: `8683`
- Truncated / Обрезано: `no`

```rust
// ADR-0073: shared API support is split into bounded helper modules while
// route modules still import this facade during the backend decomposition phase.
pub(crate) mod automation_calls;
pub(crate) mod communications;
pub(crate) mod formatting;
pub(crate) mod messaging_integrations;
pub(crate) mod platform_dtos;
pub(crate) mod query_parsing;
pub(crate) mod review_commands;
pub(crate) mod review_lists;
pub(crate) mod stores;
pub(crate) mod telegram_capabilities;
pub(crate) mod telegram_capability_catalog;
pub(crate) mod telegram_capability_catalog_extended;
pub(crate) mod telegram_capability_catalog_foundation;
pub(crate) mod telegram_capability_catalog_messages;
pub(crate) mod whatsapp_capabilities;
pub(crate) mod whatsapp_capability_catalog;

pub(crate) use automation_calls::*;
pub(crate) use communications::*;
pub(crate) use formatting::*;
pub(crate) use messaging_integrations::*;
pub(crate) use platform_dtos::*;
pub(crate) use query_parsing::*;
pub(crate) use review_commands::*;
pub(crate) use review_lists::*;
pub(crate) use stores::*;
pub(crate) use telegram_capabilities::*;
pub(crate) use whatsapp_capabilities::*;

use std::io;

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::Html;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;
use url::form_urlencoded;

use crate::ai::control_center::{AiControlCenterError, AiControlCenterStore, AiProviderAccount};
use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiModelRouting, AiService, AiStatusResponse,
    AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
};
use crate::domains::persons::analytics::{AnalyticsError, PersonAnalyticsService};
use crate::domains::persons::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::persons::expertise::{PersonExpertiseError, PersonExpertiseStore};
use crate::domains::persons::export::{ExportError, ExportFormat, PersonExportService};
use crate::domains::persons::investigator::{InvestigatorError, PersonInvestigator};
use crate::engines::automation::{
    AutomationError, AutomationPolicy, AutomationStore, AutomationTemplate, NewAutomationPolicy,
    NewAutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::platform::calls::{
    CallDirection, CallError, CallIntelligenceStore, CallState, CallTranscript,
    FixtureSpeechToTextProvider, NewCallTranscript, NewProviderCall, ProviderCall,
    SpeechToTextProvider, TranscriptStatus,
};
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};
use crate::platform::config::{AiRuntimeProvider, AppConfig};

use crate::domains::persons::health::{PersonHealthError, PersonHealthStore};

use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore, PersonTrustError};

use crate::domains::persons::memory::{
    NewRelationshipEvent, PersonFactStore, PersonMemoryCardStore, PersonMemoryError,
    PersonPreferenceStore, RelationshipEventStore,
};

use crate::domains::persons::core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonPersona, PersonPersonaStore,
    PersonRole, PersonRoleStore, PersonsIdentityStore,
};
use crate::domains::persons::identity::{
    PersonIdentityCandidate, PersonIdentityDetail, PersonIdentityError,
    PersonIdentityReviewCommand, PersonIdentityReviewState, PersonIdentityStore,
};

use crate::application::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};
use crate::application::provider_runtime_contracts::{
    NewTelegramMessage, NewWhatsappWebMessage, ProviderCommunicationMessage,
    TelegramAccountSetupRequest, TelegramAccountSetupResponse, TelegramChat, TelegramError,
    TelegramMessageIngestResult, WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse,
    WhatsappWebError, WhatsappWebMessage, WhatsappWebMessageIngestResult, WhatsappWebSession,
};
use crate::domains::calendar::brain::{CalendarBrainError, CalendarBrainService};
use crate::domains::calendar::core::{
    CalendarCoreError, ContextPackInput, EventAgendaStore, EventChecklistStore,
    EventContextPackStore, EventParticipantStore, EventRelationStore,
};
use crate::domains::calendar::events::{
    CalendarAccountStore, CalendarAccountUpdate, CalendarError, CalendarEventListQuery,
    CalendarEventStore, CalendarEventUpdate, CalendarSourceStore, NewCalendarEvent,
};
use crate::domains::calendar::health::{CalendarHealthError, CalendarWatchtowerService};
use crate::domains::calendar::intelligence::CalendarIntelligenceService;
use crate::domains::calendar::meetings::{
    EventRecordingStore, EventTranscriptStore, MeetingNoteStore, MeetingOutcomeStore, MeetingsError,
};
use crate::domains::calendar::reminders::{CalendarReminderStore, ReminderError};
use crate::domains::calendar::rules::{CalendarRuleError, CalendarRuleStore, RuleUpdate};
use crate::domains::calendar::scheduling::{
    DeadlineStore, FocusBlockStore, SchedulingError, SmartSchedulingService,
};
use crate::domains::calendar::sync::{export_event_ics, export_event_md};
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
    WorkflowState,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, StoredCommunicationAttachmentWithBlob,
};
use crate::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use crate::domains::graph::core::{GraphNodeKind, node_id};
use crate::domains::organizations::api::{
    OrganizationError, OrganizationStore, OrganizationUpdate,
};
use crate::domains::projects::core::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::domains::tasks::api::{NewTask, TaskError, TaskListQuery, TaskStore, TaskUpdate};
use crate::domains::tasks::brain::{TaskBrainError, TaskBrainService};
use crate::domains::tasks::candidates::{
    TaskCandidate, TaskCandidateError, TaskCandidateReviewCommand, TaskCandidateReviewState,
    TaskCandidateStore,
};
use crate::domains::tasks::core::{
    ExternalTaskIdentityStore, TaskChecklistStore, TaskContextPackStore, TaskCoreError,
    TaskEvidenceStore, TaskProviderStore, TaskRelationStore, TaskSubtaskStore,
};
use crate::domains::tasks::health::{TaskHealthError, TaskWatchtowerService};
use crate::domains::tasks::intelligence::TaskIntelligenceService;
use crate::domains::tasks::rules::{TaskRuleError, TaskRuleStore, TaskTemplateStore};
use crate::domains::tasks::sync::{export_task_json, export_task_md};
use crate::integrations::ai_runtime::{AiRuntimeClient, AiRuntimeError};
use crate::integrations::mail::accounts::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::integrations::omniroute::client::{
    OmniRouteClient, OmniRouteClientConfig, OmniRouteError,
};
use crate::integrations::telegram::tdjson;
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::platform::secrets::DatabaseEncryptedSecretVault;
use crate::platform::secrets::{SecretKind, SecretReferenceStore};
use crate::platform::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::platform::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};
use crate::vault::VaultStatus;

use crate::app::{ApiError, AppState};

pub(crate) fn ensure_fixture_routes_enabled(state: &AppState) -> Result<(), ApiError> {
    if state.config.dev_mode() || cfg!(test) {
        return Ok(());
    }
    Err(ApiError::NotFound)
}
```

### `backend/src/app/api_support/automation_calls.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/automation_calls.rs`
- Size bytes / Размер в байтах: `3611`
- Included characters / Включено символов: `3611`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[derive(Deserialize)]
pub(crate) struct PolicyTemplateApiRequest {
    pub(crate) template_id: String,
    pub(crate) name: String,
    pub(crate) body_template: String,
    #[serde(default)]
    pub(crate) required_variables: Vec<String>,
}

impl PolicyTemplateApiRequest {
    pub(crate) fn into_template(self) -> NewAutomationTemplate {
        NewAutomationTemplate {
            template_id: self.template_id,
            name: self.name,
            body_template: self.body_template,
            required_variables: self.required_variables,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PolicyTemplateListResponse {
    pub(crate) items: Vec<AutomationTemplate>,
}

#[derive(Deserialize)]
pub(crate) struct PolicyApiRequest {
    pub(crate) policy_id: String,
    pub(crate) template_id: String,
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) account_id: String,
    pub(crate) allowed_chat_ids: Vec<String>,
    pub(crate) trigger_kind: String,
    pub(crate) max_sends_per_hour: i32,
    #[serde(default = "empty_json_object")]
    pub(crate) quiet_hours: Value,
    pub(crate) expires_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_json_object")]
    pub(crate) conditions: Value,
}

impl PolicyApiRequest {
    pub(crate) fn into_policy(self) -> NewAutomationPolicy {
        NewAutomationPolicy {
            policy_id: self.policy_id,
            template_id: self.template_id,
            name: self.name,
            enabled: self.enabled,
            account_id: self.account_id,
            allowed_chat_ids: self.allowed_chat_ids,
            trigger_kind: self.trigger_kind,
            max_sends_per_hour: self.max_sends_per_hour,
            quiet_hours: self.quiet_hours,
            expires_at: self.expires_at,
            conditions: self.conditions,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct PolicyListResponse {
    pub(crate) items: Vec<AutomationPolicy>,
}

#[derive(Deserialize)]
pub(crate) struct CallApiRequest {
    pub(crate) call_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_call_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) direction: CallDirection,
    pub(crate) call_state: CallState,
    pub(crate) started_at: Option<DateTime<Utc>>,
    pub(crate) ended_at: Option<DateTime<Utc>>,
    pub(crate) transcription_policy_id: Option<String>,
    #[serde(default = "empty_json_object")]
    pub(crate) metadata: Value,
}

impl CallApiRequest {
    pub(crate) fn into_call(self) -> NewProviderCall {
        NewProviderCall {
            call_id: self.call_id,
            account_id: self.account_id,
            provider_call_id: self.provider_call_id,
            provider_chat_id: self.provider_chat_id,
            direction: self.direction,
            call_state: self.call_state,
            started_at: self.started_at,
            ended_at: self.ended_at,
            transcription_policy_id: self.transcription_policy_id,
            metadata: self.metadata,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CallListResponse {
    pub(crate) items: Vec<ProviderCall>,
}

#[derive(Deserialize)]
pub(crate) struct CallTranscriptFixtureApiRequest {
    pub(crate) transcript_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) source_audio_ref: String,
    pub(crate) language_code: Option<String>,
    #[serde(default)]
    pub(crate) always_on_policy: bool,
}

#[derive(Serialize)]
pub(crate) struct CallTranscriptResponse {
    pub(crate) transcript: Option<CallTranscript>,
}
```

### `backend/src/app/api_support/communications.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/communications.rs`
- Size bytes / Размер в байтах: `7137`
- Included characters / Включено символов: `7137`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[derive(Serialize)]
pub(crate) struct CommunicationMessagesResponse {
    pub(crate) items: Vec<CommunicationMessageSummaryResponse>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) has_more: bool,
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageSummaryResponse {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) observation_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_record_id: String,
    pub(crate) subject: String,
    pub(crate) sender: String,
    pub(crate) recipients: Vec<String>,
    pub(crate) body_text_preview: String,
    pub(crate) occurred_at: Option<DateTime<Utc>>,
    pub(crate) projected_at: DateTime<Utc>,
    pub(crate) channel_kind: String,
    pub(crate) conversation_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) delivery_state: String,
    pub(crate) message_metadata: Value,
    pub(crate) attachment_count: i64,
    pub(crate) local_state: String,
    pub(crate) local_state_changed_at: Option<DateTime<Utc>>,
}

impl From<ProjectedMessageSummary> for CommunicationMessageSummaryResponse {
    fn from(summary: ProjectedMessageSummary) -> Self {
        Self {
            message_id: summary.message.message_id,
            raw_record_id: summary.message.raw_record_id,
            observation_id: summary.message.observation_id,
            account_id: summary.message.account_id,
            provider_record_id: summary.message.provider_record_id,
            subject: summary.message.subject,
            sender: summary.message.sender,
            recipients: summary.message.recipients,
            body_text_preview: text_preview(&summary.message.body_text, 240),
            occurred_at: summary.message.occurred_at,
            projected_at: summary.message.projected_at,
            channel_kind: summary.message.channel_kind,
            conversation_id: summary.message.conversation_id,
            sender_display_name: summary.message.sender_display_name,
            delivery_state: summary.message.delivery_state,
            message_metadata: summary.message.message_metadata,
            attachment_count: summary.attachment_count,
            local_state: summary.message.local_state.as_str().to_owned(),
            local_state_changed_at: summary.message.local_state_changed_at,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageDetailResponse {
    pub(crate) message: CommunicationMessageDetailItem,
    pub(crate) attachments: Vec<CommunicationAttachmentResponse>,
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageDetailItem {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) observation_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_record_id: String,
    pub(crate) subject: String,
    pub(crate) sender: String,
    pub(crate) recipients: Vec<String>,
    pub(crate) body_text: String,
    pub(crate) body_html: Option<String>,
    pub(crate) occurred_at: Option<DateTime<Utc>>,
    pub(crate) projected_at: DateTime<Utc>,
    pub(crate) channel_kind: String,
    pub(crate) conversation_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) delivery_state: String,
    pub(crate) message_metadata: Value,
    pub(crate) local_state: String,
    pub(crate) local_state_changed_at: Option<DateTime<Utc>>,
    pub(crate) local_state_reason: Option<String>,
}

impl CommunicationMessageDetailItem {
    pub(crate) fn from_message(message: ProjectedMessage, body_html: Option<String>) -> Self {
        let message_metadata = message.message_metadata.clone();
        Self::from_message_with_metadata(message, body_html, message_metadata)
    }

    pub(crate) fn from_message_with_metadata(
        message: ProjectedMessage,
        body_html: Option<String>,
        message_metadata: Value,
    ) -> Self {
        Self {
            message_id: message.message_id,
            raw_record_id: message.raw_record_id,
            observation_id: message.observation_id,
            account_id: message.account_id,
            provider_record_id: message.provider_record_id,
            subject: message.subject,
            sender: message.sender,
            recipients: message.recipients,
            body_text: message.body_text,
            body_html,
            occurred_at: message.occurred_at,
            projected_at: message.projected_at,
            channel_kind: message.channel_kind,
            conversation_id: message.conversation_id,
            sender_display_name: message.sender_display_name,
            delivery_state: message.delivery_state,
            message_metadata,
            local_state: message.local_state.as_str().to_owned(),
            local_state_changed_at: message.local_state_changed_at,
            local_state_reason: message.local_state_reason,
        }
    }
}

impl From<ProjectedMessage> for CommunicationMessageDetailItem {
    fn from(message: ProjectedMessage) -> Self {
        Self::from_message(message, None)
    }
}

#[derive(Serialize)]
pub(crate) struct CommunicationAttachmentResponse {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) blob_id: String,
    pub(crate) provider_attachment_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) disposition: &'static str,
    pub(crate) scan_status: &'static str,
    pub(crate) scan_engine: Option<String>,
    pub(crate) scan_checked_at: Option<DateTime<Utc>>,
    pub(crate) scan_summary: Option<String>,
    pub(crate) scan_metadata: Value,
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

impl From<StoredCommunicationAttachmentWithBlob> for CommunicationAttachmentResponse {
    fn from(record: StoredCommunicationAttachmentWithBlob) -> Self {
        let attachment = record.attachment;
        Self {
            attachment_id: attachment.attachment_id,
            message_id: attachment.message_id,
            raw_record_id: attachment.raw_record_id,
            blob_id: attachment.blob_id,
            provider_attachment_id: attachment.provider_attachment_id,
            filename: attachment.filename,
            content_type: attachment.content_type,
            size_bytes: attachment.size_bytes,
            sha256: attachment.sha256,
            disposition: attachment.disposition.as_str(),
            scan_status: attachment.scan_status.as_str(),
            scan_engine: attachment.scan_engine,
            scan_checked_at: attachment.scan_checked_at,
            scan_summary: attachment.scan_summary,
            scan_metadata: attachment.scan_metadata,
            storage_kind: record.storage_kind,
            storage_path: record.storage_path,
            created_at: attachment.created_at,
            updated_at: attachment.updated_at,
        }
    }
}
```

### `backend/src/app/api_support/formatting.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/formatting.rs`
- Size bytes / Размер в байтах: `411`
- Included characters / Включено символов: `411`
- Truncated / Обрезано: `no`

```rust
use super::*;

pub(crate) use crate::platform::formatting::text_preview;

pub(crate) fn default_schema_version() -> i32 {
    1
}

pub(crate) fn empty_json_object() -> Value {
    json!({})
}

pub(crate) fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
```

### `backend/src/app/api_support/messaging_integrations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/messaging_integrations.rs`
- Size bytes / Размер в байтах: `1338`
- Included characters / Включено символов: `1338`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[derive(Deserialize)]
pub(crate) struct TelegramListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramChatListResponse {
    pub(crate) items: Vec<TelegramChat>,
}

#[derive(Serialize)]
pub(crate) struct TelegramMessageListResponse {
    pub(crate) items: Vec<ProviderCommunicationMessage>,
}

#[derive(Deserialize)]
pub(crate) struct WhatsappWebListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct WhatsappWebSessionListResponse {
    pub(crate) items: Vec<WhatsappWebSession>,
}

#[derive(Serialize)]
pub(crate) struct WhatsappWebMessageListResponse {
    pub(crate) items: Vec<WhatsappWebMessage>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramReactionDeleteQuery {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) reaction_emoji: String,
    pub(crate) sender_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) command_id: Option<String>,
}
```

### `backend/src/app/api_support/platform_dtos.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/platform_dtos.rs`
- Size bytes / Размер в байтах: `2628`
- Included characters / Включено символов: `2628`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[derive(Serialize)]
pub(crate) struct ApplicationSettingsResponse {
    pub(crate) items: Vec<ApplicationSetting>,
}

#[derive(Serialize)]
pub(crate) struct ApplicationAccountsResponse {
    pub(crate) items: Vec<ProviderAccount>,
}

#[derive(Deserialize)]
pub(crate) struct ApplicationSettingUpdateRequest {
    pub(crate) value: Value,
}

#[derive(Deserialize)]
pub(crate) struct AppendEventRequest {
    pub(crate) event_id: String,
    pub(crate) event_type: String,
    #[serde(default = "default_schema_version")]
    pub(crate) schema_version: i32,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) source: Value,
    pub(crate) actor: Option<Value>,
    pub(crate) subject: Value,
    #[serde(default = "empty_json_object")]
    pub(crate) payload: Value,
    #[serde(default = "empty_json_object")]
    pub(crate) provenance: Value,
    pub(crate) causation_id: Option<String>,
    pub(crate) correlation_id: Option<String>,
}

impl AppendEventRequest {
    pub(crate) fn into_new_event(self) -> Result<NewEventEnvelope, EventEnvelopeError> {
        let mut builder = NewEventEnvelope::builder(
            self.event_id,
            self.event_type,
            self.occurred_at,
            self.source,
            self.subject,
        )
        .schema_version(self.schema_version)
        .payload(self.payload)
        .provenance(self.provenance);

        if let Some(actor) = self.actor {
            builder = builder.actor(actor);
        }

        if let Some(causation_id) = self.causation_id {
            builder = builder.causation_id(causation_id);
        }

        if let Some(correlation_id) = self.correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        builder.build()
    }
}

#[derive(Serialize)]
pub(crate) struct AppendEventResponse {
    pub(crate) event_id: String,
    pub(crate) position: i64,
}

#[derive(Deserialize)]
pub(crate) struct AuditEventsQuery {
    pub(crate) target_id: Option<String>,
    pub(crate) actor_id: Option<String>,
    pub(crate) after_audit_id: Option<i64>,
    pub(crate) limit: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct AuditEventsResponse {
    pub(crate) items: Vec<ApiAuditRecord>,
}

#[derive(Serialize)]
pub(crate) struct V1StatusResponse {
    pub(crate) version: &'static str,
    pub(crate) surfaces: V1Surfaces,
    pub(crate) vault_status: VaultStatus,
}

#[derive(Serialize)]
pub(crate) struct V1Surfaces {
    pub(crate) messages: bool,
    pub(crate) persons: bool,
    pub(crate) search: bool,
    pub(crate) documents: bool,
    pub(crate) account_setup: bool,
}
```
