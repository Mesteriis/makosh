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

- Chunk ID / ID чанка: `071-source-backend-part-051`
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

### `backend/src/platform/events/trace_context.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/trace_context.rs`
- Size bytes / Размер в байтах: `1135`
- Included characters / Включено символов: `1135`
- Truncated / Обрезано: `no`

```rust
use super::builder::NewEventEnvelopeBuilder;
use super::models::{EventEnvelope, StoredEventEnvelope};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceContext {
    pub correlation_id: String,
    pub causation_id: Option<String>,
}

impl TraceContext {
    pub fn root(root_id: impl Into<String>) -> Self {
        Self {
            correlation_id: root_id.into(),
            causation_id: None,
        }
    }

    pub fn child_of(parent: &EventEnvelope) -> Self {
        Self {
            correlation_id: parent
                .correlation_id
                .clone()
                .unwrap_or_else(|| parent.event_id.clone()),
            causation_id: Some(parent.event_id.clone()),
        }
    }

    pub fn child_of_stored(parent: &StoredEventEnvelope) -> Self {
        Self::child_of(&parent.event)
    }

    pub fn apply(self, builder: NewEventEnvelopeBuilder) -> NewEventEnvelopeBuilder {
        let builder = builder.correlation_id(self.correlation_id);
        match self.causation_id {
            Some(causation_id) => builder.causation_id(causation_id),
            None => builder,
        }
    }
}
```

### `backend/src/platform/events/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/validation.rs`
- Size bytes / Размер в байтах: `534`
- Included characters / Включено символов: `534`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::EventEnvelopeError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), EventEnvelopeError> {
    if value.trim().is_empty() {
        return Err(EventEnvelopeError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), EventEnvelopeError> {
    if !value.is_object() {
        return Err(EventEnvelopeError::NonObjectJson(field_name));
    }

    Ok(())
}
```

### `backend/src/platform/formatting.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/formatting.rs`
- Size bytes / Размер в байтах: `240`
- Included characters / Включено символов: `240`
- Truncated / Обрезано: `no`

```rust
pub(crate) fn text_preview(value: &str, max_chars: usize) -> String {
    let mut preview = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        preview.push_str("...");
    }

    preview
}
```

### `backend/src/platform/graph.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/graph.rs`
- Size bytes / Размер в байтах: `1019`
- Included characters / Включено символов: `1019`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    Person,
    EmailAddress,
    Message,
    Document,
    Project,
    Organization,
    Task,
    Event,
    Decision,
    Obligation,
    Knowledge,
}

impl GraphNodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::EmailAddress => "email_address",
            Self::Message => "message",
            Self::Document => "document",
            Self::Project => "project",
            Self::Organization => "organization",
            Self::Task => "task",
            Self::Event => "event",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Knowledge => "knowledge",
        }
    }
}

pub fn node_id(kind: GraphNodeKind, stable_key: &str) -> String {
    format!("graph:node:v1:{}:{stable_key}", kind.as_str())
}
```

### `backend/src/platform/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/mod.rs`
- Size bytes / Размер в байтах: `289`
- Included characters / Включено символов: `289`
- Truncated / Обрезано: `no`

```rust
pub mod ai_runtime;
pub mod audit;
pub mod calls;
pub mod capabilities;
pub mod communications;
pub mod config;
pub mod events;
pub mod formatting;
pub mod graph;
pub mod observations;
pub mod projections;
pub mod realtime_conversation;
pub mod secrets;
pub mod settings;
pub mod storage;
```

### `backend/src/platform/observations/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/observations/errors.rs`
- Size bytes / Размер в байтах: `980`
- Included characters / Включено символов: `980`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStoreError};

#[derive(Debug, Error)]
pub enum ObservationStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("unknown observation origin kind stored in database: {0}")]
    UnknownOriginKind(String),

    #[error("unknown observation ingestion run status stored in database: {0}")]
    UnknownIngestionRunStatus(String),

    #[error("observation kind definition was not found: {0}")]
    ObservationKindNotFound(String),
}
```

### `backend/src/platform/observations/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/observations/mod.rs`
- Size bytes / Размер в байтах: `691`
- Included characters / Включено символов: `691`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod models;
mod review_links;
mod store;

pub use errors::ObservationStoreError;
pub use errors::ObservationStoreError as ObservationPortError;
pub use models::{
    NewObservation, NewObservationIngestionRun, NewObservationLink, Observation,
    ObservationIngestionRun, ObservationIngestionRunStatus, ObservationKindDefinition,
    ObservationLink, ObservationOriginKind,
};
pub(crate) use review_links::{
    link_domain_entity, link_domain_entity_in_transaction, materialize_review_transition_link,
    materialize_review_transition_link_in_transaction,
};
pub use store::ObservationStore as ObservationPort;
pub use store::{ObservationStore, observation_captured_event_id};
```

### `backend/src/platform/observations/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/observations/models.rs`
- Size bytes / Размер в байтах: `8916`
- Included characters / Включено символов: `8916`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::ObservationStoreError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationOriginKind {
    VaultSource,
    Manual,
    BrowserCapture,
    VoiceMemo,
    FileImport,
    LocalRuntime,
    TestFixture,
}

impl ObservationOriginKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VaultSource => "vault_source",
            Self::Manual => "manual",
            Self::BrowserCapture => "browser_capture",
            Self::VoiceMemo => "voice_memo",
            Self::FileImport => "file_import",
            Self::LocalRuntime => "local_runtime",
            Self::TestFixture => "test_fixture",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObservationStoreError> {
        match value.as_ref() {
            "vault_source" => Ok(Self::VaultSource),
            "manual" => Ok(Self::Manual),
            "browser_capture" => Ok(Self::BrowserCapture),
            "voice_memo" => Ok(Self::VoiceMemo),
            "file_import" => Ok(Self::FileImport),
            "local_runtime" => Ok(Self::LocalRuntime),
            "test_fixture" => Ok(Self::TestFixture),
            unknown => Err(ObservationStoreError::UnknownOriginKind(unknown.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObservationKindDefinition {
    pub kind_definition_id: String,
    pub code: String,
    pub name: String,
    pub version: i32,
    pub category: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Observation {
    pub observation_id: String,
    pub kind_definition_id: String,
    pub kind_code: String,
    pub origin_kind: ObservationOriginKind,
    pub vault_source_id: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub captured_at: DateTime<Utc>,
    pub payload: Value,
    pub confidence: f64,
    pub content_hash: String,
    pub source_ref: String,
    pub provenance: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewObservation {
    pub kind_code: String,
    pub origin_kind: ObservationOriginKind,
    pub vault_source_id: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub payload: Value,
    pub confidence: f64,
    pub source_ref: String,
    pub provenance: Value,
}

impl NewObservation {
    pub fn new(
        kind_code: impl Into<String>,
        origin_kind: ObservationOriginKind,
        observed_at: DateTime<Utc>,
        payload: Value,
        source_ref: impl Into<String>,
    ) -> Self {
        Self {
            kind_code: kind_code.into(),
            origin_kind,
            vault_source_id: None,
            observed_at,
            payload,
            confidence: 1.0,
            source_ref: source_ref.into(),
            provenance: json!({}),
        }
    }

    pub fn vault_source_id(mut self, vault_source_id: impl Into<String>) -> Self {
        self.vault_source_id = Some(vault_source_id.into());
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn provenance(mut self, provenance: Value) -> Self {
        self.provenance = provenance;
        self
    }

    pub fn validate(&self) -> Result<(), ObservationStoreError> {
        validate_non_empty("kind_code", &self.kind_code)?;
        validate_non_empty("source_ref", &self.source_ref)?;
        if let Some(vault_source_id) = &self.vault_source_id {
            validate_non_empty("vault_source_id", vault_source_id)?;
        }
        validate_json_object("payload", &self.payload)?;
        validate_json_object("provenance", &self.provenance)?;
        validate_score("confidence", self.confidence)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObservationLink {
    pub observation_id: String,
    pub domain: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub relationship_kind: String,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewObservationLink {
    pub observation_id: String,
    pub domain: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub relationship_kind: String,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObservationLink {
    pub fn new(
        observation_id: impl Into<String>,
        domain: impl Into<String>,
        entity_kind: impl Into<String>,
        entity_id: impl Into<String>,
    ) -> Self {
        Self {
            observation_id: observation_id.into(),
            domain: domain.into(),
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
            relationship_kind: "evidence_for".to_owned(),
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn relationship_kind(mut self, relationship_kind: impl Into<String>) -> Self {
        self.relationship_kind = relationship_kind.into();
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> Result<(), ObservationStoreError> {
        validate_non_empty("observation_id", &self.observation_id)?;
        validate_non_empty("domain", &self.domain)?;
        validate_non_empty("entity_kind", &self.entity_kind)?;
        validate_non_empty("entity_id", &self.entity_id)?;
        validate_non_empty("relationship_kind", &self.relationship_kind)?;
        validate_json_object("metadata", &self.metadata)?;
        validate_score("confidence", self.confidence)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationIngestionRunStatus {
    Running,
    Succeeded,
    Failed,
    Skipped,
}

impl ObservationIngestionRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObservationStoreError> {
        match value.as_ref() {
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            unknown => Err(ObservationStoreError::UnknownIngestionRunStatus(
                unknown.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObservationIngestionRun {
    pub ingestion_run_id: String,
    pub observation_id: String,
    pub pipeline: String,
    pub status: ObservationIngestionRunStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub output: Value,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewObservationIngestionRun {
    pub ingestion_run_id: String,
    pub observation_id: String,
    pub pipeline: String,
}

impl NewObservationIngestionRun {
    pub fn new(
        ingestion_run_id: impl Into<String>,
        observation_id: impl Into<String>,
        pipeline: impl Into<String>,
    ) -> Self {
        Self {
            ingestion_run_id: ingestion_run_id.into(),
            observation_id: observation_id.into(),
            pipeline: pipeline.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ObservationStoreError> {
        validate_non_empty("ingestion_run_id", &self.ingestion_run_id)?;
        validate_non_empty("observation_id", &self.observation_id)?;
        validate_non_empty("pipeline", &self.pipeline)?;
        Ok(())
    }
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ObservationStoreError> {
    if value.trim().is_empty() {
        return Err(ObservationStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ObservationStoreError> {
    if !value.is_object() {
        return Err(ObservationStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_score(
    field_name: &'static str,
    value: f64,
) -> Result<(), ObservationStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(ObservationStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}
```

### `backend/src/platform/observations/review_links.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/observations/review_links.rs`
- Size bytes / Размер в байтах: `4634`
- Included characters / Включено символов: `4634`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use super::{NewObservationLink, ObservationStore, ObservationStoreError};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn materialize_review_transition_link(
    pool: &PgPool,
    observation_id: Option<&str>,
    domain: &str,
    entity_kind: &str,
    entity_id: &str,
    state_field: &str,
    state_value: &str,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let Some(link) = build_review_transition_link(
        observation_id,
        domain,
        entity_kind,
        entity_id,
        state_field,
        state_value,
        metadata,
    ) else {
        return Ok(());
    };

    ObservationStore::new(pool.clone())
        .upsert_link(&link)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn materialize_review_transition_link_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    domain: &str,
    entity_kind: &str,
    entity_id: &str,
    state_field: &str,
    state_value: &str,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let Some(link) = build_review_transition_link(
        observation_id,
        domain,
        entity_kind,
        entity_id,
        state_field,
        state_value,
        metadata,
    ) else {
        return Ok(());
    };

    ObservationStore::upsert_link_in_transaction(transaction, &link).await?;
    Ok(())
}

fn build_review_transition_link(
    observation_id: Option<&str>,
    domain: &str,
    entity_kind: &str,
    entity_id: &str,
    state_field: &str,
    state_value: &str,
    metadata: Option<Value>,
) -> Option<NewObservationLink> {
    let observation_id = observation_id.filter(|value| !value.is_empty())?;

    let mut link = NewObservationLink::new(
        observation_id.to_owned(),
        domain,
        entity_kind,
        entity_id.to_owned(),
    )
    .relationship_kind("review_transition")
    .metadata(json!({
        state_field: state_value,
    }));

    if let Some(extra) = metadata {
        if let (Some(base), Some(extra)) = (link.metadata.as_object_mut(), extra.as_object()) {
            for (key, value) in extra {
                base.insert(key.clone(), value.clone());
            }
        } else {
            link = link.metadata(extra);
        }
    }

    Some(link)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn link_domain_entity(
    pool: &PgPool,
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let link = build_domain_entity_link(
        observation_id,
        domain,
        entity_kind,
        entity_id.into(),
        relationship_kind,
        confidence,
        metadata,
    )?;
    ObservationStore::new(pool.clone())
        .upsert_link(&link)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn link_domain_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let link = build_domain_entity_link(
        observation_id,
        domain,
        entity_kind,
        entity_id.into(),
        relationship_kind,
        confidence,
        metadata,
    )?;
    ObservationStore::upsert_link_in_transaction(transaction, &link).await?;
    Ok(())
}

fn build_domain_entity_link(
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: String,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<NewObservationLink, ObservationStoreError> {
    let observation_id = observation_id.trim();
    if observation_id.is_empty() {
        return Err(ObservationStoreError::EmptyField("observation_id"));
    }

    let mut link =
        NewObservationLink::new(observation_id.to_owned(), domain, entity_kind, entity_id);

    if let Some(relationship_kind) = relationship_kind.filter(|value| !value.is_empty()) {
        link = link.relationship_kind(relationship_kind);
    }
    if let Some(confidence) = confidence {
        link = link.confidence(confidence);
    }
    if let Some(metadata) = metadata {
        link = link.metadata(metadata);
    }

    Ok(link)
}
```

### `backend/src/platform/observations/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/observations/store.rs`
- Size bytes / Размер в байтах: `16313`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};

use crate::platform::events::{EventStore, NewEventEnvelope};

use super::errors::ObservationStoreError;
use super::models::{
    NewObservation, NewObservationIngestionRun, NewObservationLink, Observation,
    ObservationIngestionRun, ObservationIngestionRunStatus, ObservationKindDefinition,
    ObservationLink, ObservationOriginKind, validate_json_object, validate_non_empty,
};

#[derive(Clone)]
pub struct ObservationStore {
    pool: PgPool,
}

impl ObservationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn capture(
        &self,
        observation: &NewObservation,
    ) -> Result<Observation, ObservationStoreError> {
        observation.validate()?;

        let mut transaction = self.pool.begin().await?;
        let stored = Self::capture_in_transaction(&mut transaction, observation).await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_kind_definitions(
        &self,
    ) -> Result<Vec<ObservationKindDefinition>, ObservationStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                kind_definition_id,
                code,
                name,
                version,
                category,
                description,
                created_at,
                updated_at
            FROM observation_kind_definitions
            ORDER BY category ASC, code ASC, version ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_kind_definition).collect()
    }

    pub async fn get(
        &self,
        observation_id: &str,
    ) -> Result<Option<Observation>, ObservationStoreError> {
        validate_non_empty("observation_id", observation_id)?;
        let sql = observation_select_sql("WHERE observation.observation_id = $1");
        let row = sqlx::query(&sql)
            .bind(observation_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_observation).transpose()
    }

    pub async fn upsert_link(
        &self,
        link: &NewObservationLink,
    ) -> Result<ObservationLink, ObservationStoreError> {
        link.validate()?;
        let mut transaction = self.pool.begin().await?;
        let stored = Self::upsert_link_in_transaction(&mut transaction, link).await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_link_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        link: &NewObservationLink,
    ) -> Result<ObservationLink, ObservationStoreError> {
        link.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO observation_links (
                observation_id,
                domain,
                entity_kind,
                entity_id,
                relationship_kind,
                confidence,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (observation_id, domain, entity_kind, entity_id, relationship_kind)
            DO UPDATE SET
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata
            RETURNING
                observation_id,
                domain,
                entity_kind,
                entity_id,
                relationship_kind,
                confidence::float8 AS confidence,
                metadata,
                created_at
            "#,
        )
        .bind(link.observation_id.trim())
        .bind(link.domain.trim())
        .bind(link.entity_kind.trim())
        .bind(link.entity_id.trim())
        .bind(link.relationship_kind.trim())
        .bind(link.confidence)
        .bind(&link.metadata)
        .fetch_one(&mut **transaction)
        .await?;
        row_to_observation_link(row)
    }

    pub async fn list_links(
        &self,
        observation_id: &str,
    ) -> Result<Vec<ObservationLink>, ObservationStoreError> {
        validate_non_empty("observation_id", observation_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                observation_id,
                domain,
                entity_kind,
                entity_id,
                relationship_kind,
                confidence::float8 AS confidence,
                metadata,
                created_at
            FROM observation_links
            WHERE observation_id = $1
            ORDER BY domain ASC, entity_kind ASC, entity_id ASC, relationship_kind ASC
            "#,
        )
        .bind(observation_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_observation_link).collect()
    }

    pub async fn start_ingestion_run(
        &self,
        run: &NewObservationIngestionRun,
    ) -> Result<ObservationIngestionRun, ObservationStoreError> {
        run.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO observation_ingestion_runs (
                ingestion_run_id,
                observation_id,
                pipeline,
                status
            )
            VALUES ($1, $2, $3, 'running')
            ON CONFLICT (ingestion_run_id)
            DO UPDATE SET
                observation_id = EXCLUDED.observation_id,
                pipeline = EXCLUDED.pipeline
            RETURNING
                ingestion_run_id,
                observation_id,
                pipeline,
                status,
                started_at,
                finished_at,
                output,
                error_message
            "#,
        )
        .bind(run.ingestion_run_id.trim())
        .bind(run.observation_id.trim())
        .bind(run.pipeline.trim())
        .fetch_one(&self.pool)
        .await?;
        row_to_observation_ingestion_run(row)
    }

    pub async fn finish_ingestion_run(
        &self,
        ingestion_run_id: &str,
        status: ObservationIngestionRunStatus,
        output: &serde_json::Value,
        error_message: Option<&str>,
    ) -> Result<ObservationIngestionRun, ObservationStoreError> {
        validate_non_empty("ingestion_run_id", ingestion_run_id)?;
        validate_json_object("output", output)?;
        if let Some(error_message) = error_message {
            validate_non_empty("error_message", error_message)?;
        }

        let row = sqlx::query(
            r#"
            UPDATE observation_ingestion_runs
            SET
                status = $2,
                finished_at = now(),
                output = $3,
                error_message = $4
            WHERE ingestion_run_id = $1
            RETURNING
                ingestion_run_id,
                observation_id,
                pipeline,
                status,
                started_at,
                finished_at,
                output,
                error_message
            "#,
        )
        .bind(ingestion_run_id)
        .bind(status.as_str())
        .bind(output)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;
        row_to_observation_ingestion_run(row)
    }

    pub async fn list_ingestion_runs(
        &self,
        observation_id: &str,
    ) -> Result<Vec<ObservationIngestionRun>, ObservationStoreError> {
        validate_non_empty("observation_id", observation_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                ingestion_run_id,
                observation_id,
                pipeline,
                status,
                started_at,
                finished_at,
                output,
                error_message
            FROM observation_ingestion_runs
            WHERE observation_id = $1
            ORDER BY started_at DESC, ingestion_run_id ASC
            "#,
        )
        .bind(observation_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(row_to_observation_ingestion_run)
            .collect()
    }

    pub(crate) async fn capture_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        observation: &NewObservation,
    ) -> Result<Observation, ObservationStoreError> {
        let kind_definition_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT kind_definition_id
            FROM observation_kind_definitions
            WHERE code = $1
              AND version = 1
            "#,
        )
        .bind(observation.kind_code.trim())
        .fetch_optional(&mut **transaction)
        .await?
        .ok_or_else(|| {
            ObservationStoreError::ObservationKindNotFound(observation.kind_code.trim().to_owned())
        })?;

        let content_hash = content_hash(observation)?;
        let observation_id = observation_id(observation, &content_hash)?;
        let inserted = sqlx::query(observation_insert_sql())
            .bind(&observation_id)
            .bind(&kind_definition_id)
            .bind(observation.origin_kind.as_str())
            .bind(&observation.vault_source_id)
            .bind(observation.observed_at)
            .bind(&observation.payload)
            .bind(observation.confidence)
            .bind(&content_hash)
            .bind(observation.source_ref.trim())
            .bind(&observation.provenance)
            .bind(observation.kind_code.trim())
            .fetch_optional(&mut **transaction)
            .await?;

        if let Some(row) = inserted {
            let stored = row_to_observation(row)?;
            append_observation_captured_event(transaction, &stored).await?;
            return Ok(stored);
        }

        let sql = observation_select_sql("WHERE observation.observation_id = $1");
        let row = sqlx::query(&sql)
            .bind(&observation_id)
            .fetch_one(&mut **transaction)
            .await?;
        row_to_observation(row)
    }
}

fn observation_insert_sql() -> &'static str {
    r#"
    INSERT INTO observations (
        observation_id,
        kind_definition_id,
        origin_kind,
        vault_source_id,
        observed_at,
        payload,
        confidence,
        content_hash,
        source_ref,
        provenance
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    ON CONFLICT (observation_id) DO NOTHING
    RETURNING
        observation_id,
        kind_definition_id,
        $11::text AS kind_code,
        origin_kind,
        vault_source_id,
        observed_at,
        captured_at,
        payload,
        confidence::float8 AS confidence,
        content_hash,
        source_ref,
        provenance
    "#
}

fn observation_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            observation.observation_id,
            observation.kind_definition_id,
            kind.code AS kind_code,
            observation.origin_kind,
            observation.vault_source_id,
            observation.observed_at,
            observation.captured_at,
            observation.payload,
            observation.confidence::float8 AS confidence,
            observation.content_hash,
            observation.source_ref,
            observation.provenance
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        {where_clause}
        "#
    )
}

fn row_to_kind_definition(row: PgRow) -> Result<ObservationKindDefinition, ObservationStoreError> {
    Ok(ObservationKindDefinition {
        kind_definition_id: row.try_get("kind_definition_id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        version: row.try_get("version")?,
        category: row.try_get("category")?,
        description: row.try_get("description")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_t
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/platform/projections.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/projections.rs`
- Size bytes / Размер в байтах: `1904`
- Included characters / Включено символов: `1904`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;

use thiserror::Error;

use crate::platform::events::{
    EventStore, EventStoreError, ProjectionCursorStore, StoredEventEnvelope,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionBatchOutcome {
    pub processed_count: usize,
    pub last_processed_position: i64,
}

pub async fn run_projection_batch<F, Fut>(
    events: &EventStore,
    cursors: &ProjectionCursorStore,
    projection_name: &str,
    batch_size: u32,
    mut handler: F,
) -> Result<ProjectionBatchOutcome, ProjectionRunnerError>
where
    F: FnMut(StoredEventEnvelope) -> Fut,
    Fut: Future<Output = Result<(), ProjectionHandlerError>>,
{
    if batch_size == 0 {
        return Err(ProjectionRunnerError::InvalidBatchSize);
    }

    let start_position = cursors.last_processed_position(projection_name).await?;
    let batch = events
        .list_after_position(start_position, batch_size)
        .await?;

    let mut processed_count = 0;
    let mut last_processed_position = start_position;

    for event in batch {
        let position = event.position;
        handler(event).await?;
        last_processed_position = cursors.save_position(projection_name, position).await?;
        processed_count += 1;
    }

    Ok(ProjectionBatchOutcome {
        processed_count,
        last_processed_position,
    })
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct ProjectionHandlerError {
    message: String,
}

impl ProjectionHandlerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProjectionRunnerError {
    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Handler(#[from] ProjectionHandlerError),

    #[error("projection batch size must be greater than zero")]
    InvalidBatchSize,
}
```

### `backend/src/platform/realtime_conversation/bundle.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/realtime_conversation/bundle.rs`
- Size bytes / Размер в байтах: `4126`
- Included characters / Включено символов: `4126`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;

use super::models::{
    CallBundleArtifact, CallBundleLayout, CallBundleManifest, CallBundlePipelineState,
    CallBundlePrivacyPolicy, RealtimeConversationProviderKind,
};

pub fn default_call_bundle_layout(root: impl Into<String>) -> CallBundleLayout {
    let root = root.into();
    CallBundleLayout {
        root,
        manifest: "manifest.json".to_owned(),
        meeting_json: "meeting.json".to_owned(),
        provider_json: "provider.json".to_owned(),
        participants_json: "participants.json".to_owned(),
        audio_mp3: "audio.mp3".to_owned(),
        speaker_hints_jsonl: "speaker-hints.jsonl".to_owned(),
        speaker_timeline_txt: "speaker-timeline.txt".to_owned(),
        event_track_jsonl: "event-track.jsonl".to_owned(),
        chat_json: "chat.json".to_owned(),
        transcript_json: "transcript.json".to_owned(),
        transcript_markdown: "transcript.md".to_owned(),
        summary_markdown: "summary.md".to_owned(),
        topics_json: "topics.json".to_owned(),
        entities_json: "entities.json".to_owned(),
        decisions_json: "decisions.json".to_owned(),
        tasks_json: "tasks.json".to_owned(),
        knowledge_json: "knowledge.json".to_owned(),
        metrics_json: "metrics.json".to_owned(),
        radar_signals_json: "radar-signals.json".to_owned(),
        screenshots_dir: "screenshots".to_owned(),
        attachments_dir: "attachments".to_owned(),
        ocr_dir: "ocr".to_owned(),
    }
}

pub fn build_call_bundle_manifest(
    bundle_id: impl Into<String>,
    provider_kind: RealtimeConversationProviderKind,
    provider_shape: impl Into<String>,
    account_id: impl Into<String>,
    provider_conference_id: Option<String>,
    join_url: Option<String>,
    root: impl Into<String>,
) -> CallBundleManifest {
    let layout = default_call_bundle_layout(root);
    CallBundleManifest {
        schema_version: 1,
        bundle_id: bundle_id.into(),
        provider_kind,
        provider_shape: provider_shape.into(),
        account_id: account_id.into(),
        provider_conference_id,
        join_url,
        calendar_event_id: None,
        project_id: None,
        organization_id: None,
        created_at: Utc::now(),
        artifacts: vec![
            CallBundleArtifact {
                kind: "audio".to_owned(),
                relative_path: layout.audio_mp3.clone(),
                source: "local_audio_loopback".to_owned(),
                truth_status: "capture_artifact".to_owned(),
                media_type: Some("audio/mpeg".to_owned()),
                description: Some(
                    "Local MP3 recording used by the transcription pipeline".to_owned(),
                ),
            },
            CallBundleArtifact {
                kind: "speaker_hints".to_owned(),
                relative_path: layout.speaker_hints_jsonl.clone(),
                source: "visible_webview_dom_heuristic".to_owned(),
                truth_status: "hint_not_truth".to_owned(),
                media_type: Some("application/x-ndjson".to_owned()),
                description: Some(
                    "Warm-start hints for diarization and speaker identity merging".to_owned(),
                ),
            },
            CallBundleArtifact {
                kind: "event_track".to_owned(),
                relative_path: layout.event_track_jsonl.clone(),
                source: "local_runtime".to_owned(),
                truth_status: "observed_runtime_event".to_owned(),
                media_type: Some("application/x-ndjson".to_owned()),
                description: Some("Meeting lifecycle and capture events".to_owned()),
            },
        ],
        layout,
        pipeline_state: CallBundlePipelineState::queued_from_local_recording(),
        privacy_policy: CallBundlePrivacyPolicy::local_visible_capture(),
        provenance: json!({
            "source": "makosh_realtime_conversation_bundle_builder",
            "single_source_of_truth": false,
            "notes": "Provider DOM speaker state is only a hint for later AI processing."
        }),
    }
}
```

### `backend/src/platform/realtime_conversation/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/realtime_conversation/events.rs`
- Size bytes / Размер в байтах: `1316`
- Included characters / Включено символов: `1316`
- Truncated / Обрезано: `no`

```rust
pub const REALTIME_CONVERSATION_SESSION_OPENED: &str = "realtime_conversation.session.opened";
pub const REALTIME_CONVERSATION_SESSION_ENDED: &str = "realtime_conversation.session.ended";
pub const REALTIME_CONVERSATION_CALL_BUNDLE_CREATED: &str =
    "realtime_conversation.call_bundle.created";
pub const REALTIME_CONVERSATION_AUDIO_CAPTURE_STARTED: &str =
    "realtime_conversation.audio_capture.started";
pub const REALTIME_CONVERSATION_AUDIO_CAPTURE_COMPLETED: &str =
    "realtime_conversation.audio_capture.completed";
pub const REALTIME_CONVERSATION_SPEAKER_HINT_OBSERVED: &str =
    "realtime_conversation.speaker_hint.observed";
pub const REALTIME_CONVERSATION_EVENT_TRACK_OBSERVED: &str =
    "realtime_conversation.event_track.observed";
pub const REALTIME_CONVERSATION_SCREENSHOT_HINT_CAPTURED: &str =
    "realtime_conversation.screenshot_hint.captured";
pub const REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED: &str =
    "realtime_conversation.transcript.requested";
pub const REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED: &str =
    "realtime_conversation.transcript.completed";
pub const REALTIME_CONVERSATION_KNOWLEDGE_EXTRACTED: &str =
    "realtime_conversation.knowledge.extracted";
pub const REALTIME_CONVERSATION_RADAR_SIGNALS_DETECTED: &str =
    "realtime_conversation.radar_signals.detected";
```

### `backend/src/platform/realtime_conversation/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/realtime_conversation/mod.rs`
- Size bytes / Размер в байтах: `593`
- Included characters / Включено символов: `593`
- Truncated / Обрезано: `no`

```rust
mod bundle;
mod events;
mod models;
mod provider;

pub use bundle::{build_call_bundle_manifest, default_call_bundle_layout};
pub use events::*;
pub use models::{
    CallBundleArtifact, CallBundleLayout, CallBundleManifest, CallBundlePipelineState,
    CallBundlePrivacyPolicy, MeetingTimelineEvent, RealtimeConversationProviderCapabilities,
    RealtimeConversationProviderKind, SpeakerTimelineHint, TopicTimelineSegment,
};
pub use provider::{
    RealtimeConversationProvider, generic_webview_provider_capabilities,
    yandex_telemost_provider_capabilities, zoom_provider_capabilities,
};
```

### `backend/src/platform/realtime_conversation/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/realtime_conversation/models.rs`
- Size bytes / Размер в байтах: `6047`
- Included characters / Включено символов: `6047`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeConversationProviderKind {
    YandexTelemost,
    Zoom,
    GoogleMeet,
    Jitsi,
    Discord,
    SignalCalls,
    Unknown,
}

impl RealtimeConversationProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::YandexTelemost => "yandex_telemost",
            Self::Zoom => "zoom",
            Self::GoogleMeet => "google_meet",
            Self::Jitsi => "jitsi",
            Self::Discord => "discord",
            Self::SignalCalls => "signal_calls",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RealtimeConversationProviderCapabilities {
    pub provider_kind: RealtimeConversationProviderKind,
    pub provider_shape: String,
    pub supports_conference_create: bool,
    pub supports_visible_webview: bool,
    pub supports_audio_capture: bool,
    pub supports_participant_events: bool,
    pub supports_speaker_hints: bool,
    pub supports_chat_capture: bool,
    pub supports_screen_share_detection: bool,
    pub supports_screenshot_hints: bool,
    pub supports_recording: bool,
    pub supports_provider_transcript: bool,
    pub supports_reactions: bool,
    pub evidence: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundleArtifact {
    pub kind: String,
    pub relative_path: String,
    pub source: String,
    pub truth_status: String,
    pub media_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundleLayout {
    pub root: String,
    pub manifest: String,
    pub meeting_json: String,
    pub provider_json: String,
    pub participants_json: String,
    pub audio_mp3: String,
    pub speaker_hints_jsonl: String,
    pub speaker_timeline_txt: String,
    pub event_track_jsonl: String,
    pub chat_json: String,
    pub transcript_json: String,
    pub transcript_markdown: String,
    pub summary_markdown: String,
    pub topics_json: String,
    pub entities_json: String,
    pub decisions_json: String,
    pub tasks_json: String,
    pub knowledge_json: String,
    pub metrics_json: String,
    pub radar_signals_json: String,
    pub screenshots_dir: String,
    pub attachments_dir: String,
    pub ocr_dir: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundlePipelineState {
    pub audio_capture: String,
    pub speaker_hints: String,
    pub transcription: String,
    pub diarization: String,
    pub speaker_identity: String,
    pub topic_timeline: String,
    pub decision_detection: String,
    pub action_detection: String,
    pub screen_intelligence: String,
    pub knowledge_extraction: String,
    pub radar_projection: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundlePrivacyPolicy {
    pub owner_visible_capture_only: bool,
    pub hidden_headless_capture: String,
    pub consent_required: bool,
    pub local_first: bool,
    pub provider_dom_truth_status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundleManifest {
    pub schema_version: u16,
    pub bundle_id: String,
    pub provider_kind: RealtimeConversationProviderKind,
    pub provider_shape: String,
    pub account_id: String,
    pub provider_conference_id: Option<String>,
    pub join_url: Option<String>,
    pub calendar_event_id: Option<String>,
    pub project_id: Option<String>,
    pub organization_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub layout: CallBundleLayout,
    pub artifacts: Vec<CallBundleArtifact>,
    pub pipeline_state: CallBundlePipelineState,
    pub privacy_policy: CallBundlePrivacyPolicy,
    pub provenance: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerTimelineHint {
    pub observed_at_ms: i64,
    pub speaker_label: String,
    pub source: String,
    pub confidence: f32,
    pub truth_status: String,
    pub provider_participant_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MeetingTimelineEvent {
    pub occurred_at_ms: i64,
    pub event_kind: String,
    pub label: String,
    pub confidence: f32,
    pub source: String,
    pub evidence: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TopicTimelineSegment {
    pub starts_at_ms: i64,
    pub ends_at_ms: Option<i64>,
    pub title: String,
    pub summary: String,
    pub confidence: f32,
    pub evidence: Value,
}

impl CallBundlePipelineState {
    pub fn queued_from_local_recording() -> Self {
        Self {
            audio_capture: "running_or_completed".to_owned(),
            speaker_hints: "collecting_hint_not_truth".to_owned(),
            transcription: "queued".to_owned(),
            diarization: "queued".to_owned(),
            speaker_identity: "queued".to_owned(),
            topic_timeline: "queued".to_owned(),
            decision_detection: "queued".to_owned(),
            action_detection: "queued".to_owned(),
            screen_intelligence: "queued".to_owned(),
            knowledge_extraction: "queued".to_owned(),
            radar_projection: "queued".to_owned(),
        }
    }
}

impl CallBundlePrivacyPolicy {
    pub fn local_visible_capture() -> Self {
        Self {
            owner_visible_capture_only: true,
            hidden_headless_capture: "forbidden".to_owned(),
            consent_required: true,
            local_first: true,
            provider_dom_truth_status: "hint_not_truth".to_owned(),
        }
    }
}

impl RealtimeConversationProviderCapabilities {
    pub fn evidence_source(source: &'static str) -> Value {
        json!({ "source": source, "confidence": "provider_contract_declared" })
    }
}
```

### `backend/src/platform/realtime_conversation/provider.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/realtime_conversation/provider.rs`
- Size bytes / Размер в байтах: `2912`
- Included characters / Включено символов: `2912`
- Truncated / Обрезано: `no`

```rust
use super::models::{RealtimeConversationProviderCapabilities, RealtimeConversationProviderKind};

pub trait RealtimeConversationProvider {
    fn provider_kind(&self) -> RealtimeConversationProviderKind;
    fn provider_shape(&self) -> &'static str;
    fn capabilities(&self) -> RealtimeConversationProviderCapabilities;
}

pub fn yandex_telemost_provider_capabilities() -> RealtimeConversationProviderCapabilities {
    RealtimeConversationProviderCapabilities {
        provider_kind: RealtimeConversationProviderKind::YandexTelemost,
        provider_shape: "yandex_telemost_user".to_owned(),
        supports_conference_create: true,
        supports_visible_webview: true,
        supports_audio_capture: true,
        supports_participant_events: false,
        supports_speaker_hints: true,
        supports_chat_capture: false,
        supports_screen_share_detection: false,
        supports_screenshot_hints: true,
        supports_recording: true,
        supports_provider_transcript: false,
        supports_reactions: false,
        evidence: RealtimeConversationProviderCapabilities::evidence_source(
            "yandex_telemost_api_and_visible_webview_runtime",
        ),
    }
}

pub fn zoom_provider_capabilities() -> RealtimeConversationProviderCapabilities {
    RealtimeConversationProviderCapabilities {
        provider_kind: RealtimeConversationProviderKind::Zoom,
        provider_shape: "zoom_user".to_owned(),
        supports_conference_create: true,
        supports_visible_webview: true,
        supports_audio_capture: true,
        supports_participant_events: true,
        supports_speaker_hints: true,
        supports_chat_capture: true,
        supports_screen_share_detection: true,
        supports_screenshot_hints: true,
        supports_recording: true,
        supports_provider_transcript: true,
        supports_reactions: true,
        evidence: RealtimeConversationProviderCapabilities::evidence_source(
            "zoom_provider_runtime_contract",
        ),
    }
}

pub fn generic_webview_provider_capabilities(
    provider_kind: RealtimeConversationProviderKind,
    provider_shape: impl Into<String>,
) -> RealtimeConversationProviderCapabilities {
    RealtimeConversationProviderCapabilities {
        provider_kind,
        provider_shape: provider_shape.into(),
        supports_conference_create: false,
        supports_visible_webview: true,
        supports_audio_capture: true,
        supports_participant_events: false,
        supports_speaker_hints: false,
        supports_chat_capture: false,
        supports_screen_share_detection: false,
        supports_screenshot_hints: true,
        supports_recording: true,
        supports_provider_transcript: false,
        supports_reactions: false,
        evidence: RealtimeConversationProviderCapabilities::evidence_source(
            "generic_visible_webview_runtime",
        ),
    }
}
```

### `backend/src/platform/secrets.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets.rs`
- Size bytes / Размер в байтах: `601`
- Included characters / Включено символов: `601`
- Truncated / Обрезано: `no`

```rust
mod crypto;
mod database_vault;
mod errors;
mod file_vault;
mod models;
mod paths;
mod resolver;
mod store;
mod validation;

pub use database_vault::{DatabaseEncryptedSecretVault, DatabaseEncryptedVaultError};
pub use errors::{SecretReferenceError, SecretResolutionError};
pub use file_vault::{EncryptedSecretVault, EncryptedVaultError};
pub use models::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReference, SecretStoreKind,
};
pub use paths::default_vault_path;
pub use resolver::{InMemorySecretResolver, SecretResolutionFuture, SecretResolver};
pub use store::SecretReferenceStore;
```

### `backend/src/platform/secrets/crypto.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/crypto.rs`
- Size bytes / Размер в байтах: `1747`
- Included characters / Включено символов: `1747`
- Truncated / Обрезано: `no`

```rust
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key};
use argon2::Argon2;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use super::models::ResolvedSecret;

pub(super) const VAULT_VERSION: u8 = 1;
pub(super) const VAULT_KDF: &str = "argon2id:v1";
pub(super) const SALT_LEN: usize = 16;
pub(super) const NONCE_LEN: usize = 12;

pub(super) fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0_u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub(super) fn encrypted_vault_cipher(
    master_key: &ResolvedSecret,
    encoded_salt: &str,
) -> Result<Aes256Gcm, super::file_vault::EncryptedVaultError> {
    let salt = BASE64_STANDARD
        .decode(encoded_salt)
        .map_err(|_| super::file_vault::EncryptedVaultError::InvalidEncoding)?;
    let mut key = [0_u8; 32];
    Argon2::default()
        .hash_password_into(master_key.expose_for_runtime().as_bytes(), &salt, &mut key)
        .map_err(|_| super::file_vault::EncryptedVaultError::Crypto)?;

    Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)))
}

pub(super) fn database_vault_cipher(
    master_key: &ResolvedSecret,
    encoded_salt: &str,
) -> Result<Aes256Gcm, super::database_vault::DatabaseEncryptedVaultError> {
    let salt = BASE64_STANDARD
        .decode(encoded_salt)
        .map_err(|_| super::database_vault::DatabaseEncryptedVaultError::InvalidEncoding)?;
    let mut key = [0_u8; 32];
    Argon2::default()
        .hash_password_into(master_key.expose_for_runtime().as_bytes(), &salt, &mut key)
        .map_err(|_| super::database_vault::DatabaseEncryptedVaultError::Crypto)?;

    Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)))
}
```

### `backend/src/platform/secrets/database_vault.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/database_vault.rs`
- Size bytes / Размер в байтах: `6992`
- Included characters / Включено символов: `6992`
- Truncated / Обрезано: `no`

```rust
use aes_gcm::Nonce;
use aes_gcm::aead::{Aead, Payload};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use super::crypto::{NONCE_LEN, SALT_LEN, VAULT_KDF, database_vault_cipher, random_bytes};
use super::errors::SecretResolutionError;
use super::models::{ResolvedSecret, SecretReference, SecretStoreKind};
use super::resolver::{SecretResolutionFuture, SecretResolver};
use super::validation::validate_database_non_empty;

#[derive(Clone)]
pub struct DatabaseEncryptedSecretVault {
    pool: PgPool,
    master_key: ResolvedSecret,
}

impl DatabaseEncryptedSecretVault {
    pub fn new(pool: PgPool, master_key: ResolvedSecret) -> Self {
        Self { pool, master_key }
    }

    pub async fn store_secret(
        &self,
        secret_ref: &str,
        value: &str,
    ) -> Result<(), DatabaseEncryptedVaultError> {
        validate_database_non_empty("secret_ref", secret_ref)?;
        validate_database_non_empty("secret value", value)?;

        let secret_ref = secret_ref.trim();
        let salt = random_bytes::<SALT_LEN>();
        let encoded_salt = BASE64_STANDARD.encode(salt);
        let cipher = database_vault_cipher(&self.master_key, &encoded_salt)?;
        let nonce = random_bytes::<NONCE_LEN>();
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: value.as_bytes(),
                    aad: secret_ref.as_bytes(),
                },
            )
            .map_err(|_| DatabaseEncryptedVaultError::Crypto)?;

        sqlx::query(
            r#"
            INSERT INTO encrypted_secret_vault_entries (
                secret_ref,
                kdf,
                salt,
                nonce,
                ciphertext,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (secret_ref)
            DO UPDATE SET
                kdf = EXCLUDED.kdf,
                salt = EXCLUDED.salt,
                nonce = EXCLUDED.nonce,
                ciphertext = EXCLUDED.ciphertext,
                updated_at = now()
            "#,
        )
        .bind(secret_ref)
        .bind(VAULT_KDF)
        .bind(encoded_salt)
        .bind(BASE64_STANDARD.encode(nonce))
        .bind(BASE64_STANDARD.encode(ciphertext))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn resolve_secret(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::DatabaseEncryptedVault {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }

        let secret_ref = reference.secret_ref.trim();
        if secret_ref.is_empty() {
            return Err(SecretResolutionError::EmptySecretRef);
        }

        let Some(entry) = self
            .vault_entry(secret_ref)
            .await
            .map_err(database_secret_store_failure)?
        else {
            return Err(SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            });
        };
        if entry.kdf != VAULT_KDF {
            return Err(database_secret_store_failure(
                DatabaseEncryptedVaultError::UnsupportedVaultFormat,
            ));
        }

        let cipher = database_vault_cipher(&self.master_key, &entry.salt)
            .map_err(database_secret_store_failure)?;
        let nonce = BASE64_STANDARD.decode(&entry.nonce).map_err(|_| {
            database_secret_store_failure(DatabaseEncryptedVaultError::InvalidEncoding)
        })?;
        let ciphertext = BASE64_STANDARD.decode(&entry.ciphertext).map_err(|_| {
            database_secret_store_failure(DatabaseEncryptedVaultError::InvalidEncoding)
        })?;
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: secret_ref.as_bytes(),
                },
            )
            .map_err(|_| database_secret_store_failure(DatabaseEncryptedVaultError::Crypto))?;
        let value = String::from_utf8(plaintext).map_err(|_| {
            database_secret_store_failure(DatabaseEncryptedVaultError::InvalidEncoding)
        })?;

        ResolvedSecret::new(value)
    }

    async fn vault_entry(
        &self,
        secret_ref: &str,
    ) -> Result<Option<DatabaseEncryptedVaultEntry>, DatabaseEncryptedVaultError> {
        let row = sqlx::query(
            r#"
            SELECT
                kdf,
                salt,
                nonce,
                ciphertext
            FROM encrypted_secret_vault_entries
            WHERE secret_ref = $1
            "#,
        )
        .bind(secret_ref)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_database_entry).transpose()
    }
}

impl SecretResolver for DatabaseEncryptedSecretVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(self.resolve_secret(reference))
    }
}

#[derive(Debug)]
struct DatabaseEncryptedVaultEntry {
    kdf: String,
    salt: String,
    nonce: String,
    ciphertext: String,
}

fn row_to_database_entry(
    row: PgRow,
) -> Result<DatabaseEncryptedVaultEntry, DatabaseEncryptedVaultError> {
    Ok(DatabaseEncryptedVaultEntry {
        kdf: row.try_get("kdf")?,
        salt: row.try_get("salt")?,
        nonce: row.try_get("nonce")?,
        ciphertext: row.try_get("ciphertext")?,
    })
}

fn database_secret_store_failure(error: DatabaseEncryptedVaultError) -> SecretResolutionError {
    SecretResolutionError::StoreFailure {
        message: error.public_message(),
    }
}

#[derive(Debug, Error)]
pub enum DatabaseEncryptedVaultError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported database encrypted vault format")]
    UnsupportedVaultFormat,

    #[error("invalid database encrypted vault encoding")]
    InvalidEncoding,

    #[error("database encrypted vault cryptographic operation failed")]
    Crypto,

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl DatabaseEncryptedVaultError {
    pub(super) fn public_message(&self) -> String {
        match self {
            Self::Crypto => "invalid vault key or corrupted encrypted database vault".to_owned(),
            Self::InvalidEncoding => "invalid encrypted database vault encoding".to_owned(),
            Self::UnsupportedVaultFormat => {
                "unsupported encrypted database vault format".to_owned()
            }
            Self::EmptyField(field) => format!("{field} must not be empty"),
            Self::Sqlx(_) => "encrypted database vault operation failed".to_owned(),
        }
    }
}
```

### `backend/src/platform/secrets/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/errors.rs`
- Size bytes / Размер в байтах: `980`
- Included characters / Включено символов: `980`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecretReferenceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported secret kind: {0}")]
    UnsupportedSecretKind(String),

    #[error("unsupported secret store kind: {0}")]
    UnsupportedStoreKind(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SecretResolutionError {
    #[error("secret_ref must not be empty")]
    EmptySecretRef,

    #[error("secret value must not be empty")]
    EmptySecretValue,

    #[error("secret reference was not found: {secret_ref}")]
    MissingSecret { secret_ref: String },

    #[error("secret store kind is not supported by in-memory resolver: {0}")]
    UnsupportedStoreKind(String),

    #[error("secret store operation failed: {message}")]
    StoreFailure { message: String },
}
```

### `backend/src/platform/secrets/file_vault.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/file_vault.rs`
- Size bytes / Размер в байтах: `6795`
- Included characters / Включено символов: `6795`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use aes_gcm::Nonce;
use aes_gcm::aead::{Aead, Payload};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::crypto::{
    NONCE_LEN, SALT_LEN, VAULT_KDF, VAULT_VERSION, encrypted_vault_cipher, random_bytes,
};
use super::errors::SecretResolutionError;
use super::models::{ResolvedSecret, SecretReference, SecretStoreKind};
use super::resolver::{SecretResolutionFuture, SecretResolver};
use super::validation::validate_vault_field;

#[derive(Clone)]
pub struct EncryptedSecretVault {
    path: PathBuf,
    master_key: ResolvedSecret,
}

impl EncryptedSecretVault {
    pub fn new(path: impl Into<PathBuf>, master_key: ResolvedSecret) -> Self {
        Self {
            path: path.into(),
            master_key,
        }
    }

    pub fn store_secret(&self, secret_ref: &str, value: &str) -> Result<(), EncryptedVaultError> {
        validate_vault_field("secret_ref", secret_ref)?;
        validate_vault_field("secret value", value)?;

        let mut file = self.load_or_create_file()?;
        let cipher = encrypted_vault_cipher(&self.master_key, &file.salt)?;
        let nonce = random_bytes::<NONCE_LEN>();
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: value.as_bytes(),
                    aad: secret_ref.trim().as_bytes(),
                },
            )
            .map_err(|_| EncryptedVaultError::Crypto)?;

        file.entries.insert(
            secret_ref.trim().to_owned(),
            EncryptedVaultEntry {
                nonce: BASE64_STANDARD.encode(nonce),
                ciphertext: BASE64_STANDARD.encode(ciphertext),
            },
        );
        self.save_file(&file)
    }

    fn load_or_create_file(&self) -> Result<EncryptedVaultFile, EncryptedVaultError> {
        if !self.path.exists() {
            return Ok(EncryptedVaultFile {
                version: VAULT_VERSION,
                kdf: VAULT_KDF.to_owned(),
                salt: BASE64_STANDARD.encode(random_bytes::<SALT_LEN>()),
                entries: BTreeMap::new(),
            });
        }

        let raw = fs::read_to_string(&self.path)?;
        let file: EncryptedVaultFile = serde_json::from_str(&raw)?;
        if file.version != VAULT_VERSION || file.kdf != VAULT_KDF {
            return Err(EncryptedVaultError::UnsupportedVaultFormat);
        }

        Ok(file)
    }

    fn load_file(&self) -> Result<Option<EncryptedVaultFile>, EncryptedVaultError> {
        if !self.path.exists() {
            return Ok(None);
        }

        self.load_or_create_file().map(Some)
    }

    fn save_file(&self, file: &EncryptedVaultFile) -> Result<(), EncryptedVaultError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = self.path.with_extension("tmp");
        let serialized = serde_json::to_vec_pretty(file)?;
        fs::write(&temp_path, serialized)?;
        fs::rename(temp_path, &self.path)?;

        Ok(())
    }

    fn resolve_secret(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::EncryptedVault {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }

        let secret_ref = reference.secret_ref.trim();
        if secret_ref.is_empty() {
            return Err(SecretResolutionError::EmptySecretRef);
        }

        let Some(file) = self.load_file().map_err(secret_store_failure)? else {
            return Err(SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            });
        };
        let Some(entry) = file.entries.get(secret_ref) else {
            return Err(SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            });
        };

        let cipher =
            encrypted_vault_cipher(&self.master_key, &file.salt).map_err(secret_store_failure)?;
        let nonce = BASE64_STANDARD
            .decode(&entry.nonce)
            .map_err(|_| secret_store_failure(EncryptedVaultError::InvalidEncoding))?;
        let ciphertext = BASE64_STANDARD
            .decode(&entry.ciphertext)
            .map_err(|_| secret_store_failure(EncryptedVaultError::InvalidEncoding))?;
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: secret_ref.as_bytes(),
                },
            )
            .map_err(|_| secret_store_failure(EncryptedVaultError::Crypto))?;
        let value = String::from_utf8(plaintext)
            .map_err(|_| secret_store_failure(EncryptedVaultError::InvalidEncoding))?;

        ResolvedSecret::new(value)
    }
}

impl SecretResolver for EncryptedSecretVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(std::future::ready(self.resolve_secret(reference)))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct EncryptedVaultFile {
    version: u8,
    kdf: String,
    salt: String,
    entries: BTreeMap<String, EncryptedVaultEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EncryptedVaultEntry {
    nonce: String,
    ciphertext: String,
}

fn secret_store_failure(error: EncryptedVaultError) -> SecretResolutionError {
    SecretResolutionError::StoreFailure {
        message: error.public_message(),
    }
}

#[derive(Debug, Error)]
pub enum EncryptedVaultError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("unsupported encrypted vault format")]
    UnsupportedVaultFormat,

    #[error("invalid encrypted vault encoding")]
    InvalidEncoding,

    #[error("encrypted vault cryptographic operation failed")]
    Crypto,

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl EncryptedVaultError {
    pub(super) fn public_message(&self) -> String {
        match self {
            Self::Crypto => "invalid vault key or corrupted encrypted vault".to_owned(),
            Self::InvalidEncoding => "invalid encrypted vault encoding".to_owned(),
            Self::UnsupportedVaultFormat => "unsupported encrypted vault format".to_owned(),
            Self::EmptyField(field) => format!("{field} must not be empty"),
            Self::Io(_) | Self::Json(_) => "encrypted vault read/write failed".to_owned(),
        }
    }
}
```

### `backend/src/platform/secrets/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/models.rs`
- Size bytes / Размер в байтах: `4660`
- Included characters / Включено символов: `4660`
- Truncated / Обрезано: `no`

```rust
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::{SecretReferenceError, SecretResolutionError};
use super::validation::{validate_non_empty, validate_object};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    OauthToken,
    AppPassword,
    Password,
    ApiToken,
    PrivateKey,
    Other,
}

impl SecretKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OauthToken => "oauth_token",
            Self::AppPassword => "app_password",
            Self::Password => "password",
            Self::ApiToken => "api_token",
            Self::PrivateKey => "private_key",
            Self::Other => "other",
        }
    }
}

impl TryFrom<&str> for SecretKind {
    type Error = SecretReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "oauth_token" => Ok(Self::OauthToken),
            "app_password" => Ok(Self::AppPassword),
            "password" => Ok(Self::Password),
            "api_token" => Ok(Self::ApiToken),
            "private_key" => Ok(Self::PrivateKey),
            "other" => Ok(Self::Other),
            other => Err(SecretReferenceError::UnsupportedSecretKind(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretStoreKind {
    OsKeychain,
    EncryptedVault,
    DatabaseEncryptedVault,
    HostVault,
    ExternalVault,
    TestDouble,
}

impl SecretStoreKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OsKeychain => "os_keychain",
            Self::EncryptedVault => "encrypted_vault",
            Self::DatabaseEncryptedVault => "database_encrypted_vault",
            Self::HostVault => "host_vault",
            Self::ExternalVault => "external_vault",
            Self::TestDouble => "test_double",
        }
    }
}

impl TryFrom<&str> for SecretStoreKind {
    type Error = SecretReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "os_keychain" => Ok(Self::OsKeychain),
            "encrypted_vault" => Ok(Self::EncryptedVault),
            "database_encrypted_vault" => Ok(Self::DatabaseEncryptedVault),
            "host_vault" => Ok(Self::HostVault),
            "external_vault" => Ok(Self::ExternalVault),
            "test_double" => Ok(Self::TestDouble),
            other => Err(SecretReferenceError::UnsupportedStoreKind(other.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SecretReference {
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
    pub label: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ResolvedSecret {
    value: String,
}

impl ResolvedSecret {
    pub fn new(value: impl Into<String>) -> Result<Self, SecretResolutionError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SecretResolutionError::EmptySecretValue);
        }

        Ok(Self { value })
    }

    pub fn expose_for_runtime(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for ResolvedSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedSecret")
            .field("value", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewSecretReference {
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
    pub label: String,
    pub metadata: Value,
}

impl NewSecretReference {
    pub fn new(
        secret_ref: impl Into<String>,
        secret_kind: SecretKind,
        store_kind: SecretStoreKind,
        label: impl Into<String>,
    ) -> Self {
        Self {
            secret_ref: secret_ref.into(),
            secret_kind,
            store_kind,
            label: label.into(),
            metadata: json!({}),
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(super) fn validate(&self) -> Result<(), SecretReferenceError> {
        validate_non_empty("secret_ref", &self.secret_ref)?;
        validate_non_empty("label", &self.label)?;
        validate_object("metadata", &self.metadata)
    }
}
```

### `backend/src/platform/secrets/paths.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/paths.rs`
- Size bytes / Размер в байтах: `193`
- Included characters / Включено символов: `193`
- Truncated / Обрезано: `no`

```rust
use std::path::{Path, PathBuf};

pub fn default_vault_path(home_dir: &Path) -> PathBuf {
    home_dir
        .join(".config")
        .join("makosh")
        .join("secrets.vault.json")
}
```

### `backend/src/platform/secrets/resolver.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/resolver.rs`
- Size bytes / Размер в байтах: `2036`
- Included characters / Включено символов: `2036`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::errors::SecretResolutionError;
use super::models::{ResolvedSecret, SecretReference, SecretStoreKind};
use super::validation::validate_secret_resolution_ref;

pub type SecretResolutionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ResolvedSecret, SecretResolutionError>> + Send + 'a>>;

pub trait SecretResolver {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemorySecretResolver {
    values: HashMap<String, ResolvedSecret>,
}

impl InMemorySecretResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        secret_ref: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), SecretResolutionError> {
        let secret_ref = secret_ref.into();
        validate_secret_resolution_ref(&secret_ref)?;
        let resolved_secret = ResolvedSecret::new(value)?;

        self.values
            .insert(secret_ref.trim().to_owned(), resolved_secret);

        Ok(())
    }

    fn resolve_reference(
        &self,
        reference: &SecretReference,
    ) -> Result<ResolvedSecret, SecretResolutionError> {
        if reference.store_kind != SecretStoreKind::TestDouble {
            return Err(SecretResolutionError::UnsupportedStoreKind(
                reference.store_kind.as_str().to_owned(),
            ));
        }

        validate_secret_resolution_ref(&reference.secret_ref)?;
        let secret_ref = reference.secret_ref.trim();

        self.values
            .get(secret_ref)
            .cloned()
            .ok_or_else(|| SecretResolutionError::MissingSecret {
                secret_ref: secret_ref.to_owned(),
            })
    }
}

impl SecretResolver for InMemorySecretResolver {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        Box::pin(std::future::ready(self.resolve_reference(reference)))
    }
}
```

### `backend/src/platform/secrets/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/store.rs`
- Size bytes / Размер в байтах: `3463`
- Included characters / Включено символов: `3463`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::SecretReferenceError;
use super::models::{NewSecretReference, SecretKind, SecretReference, SecretStoreKind};
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct SecretReferenceStore {
    pool: PgPool,
}

impl SecretReferenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_secret_reference(
        &self,
        reference: &NewSecretReference,
    ) -> Result<SecretReference, SecretReferenceError> {
        reference.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO secret_references (
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (secret_ref)
            DO UPDATE SET
                secret_kind = EXCLUDED.secret_kind,
                store_kind = EXCLUDED.store_kind,
                label = EXCLUDED.label,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(reference.secret_ref.trim())
        .bind(reference.secret_kind.as_str())
        .bind(reference.store_kind.as_str())
        .bind(reference.label.trim())
        .bind(&reference.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_secret_reference(row)
    }

    pub async fn secret_reference(
        &self,
        secret_ref: &str,
    ) -> Result<Option<SecretReference>, SecretReferenceError> {
        validate_non_empty("secret_ref", secret_ref)?;

        let row = sqlx::query(
            r#"
            SELECT
                secret_ref,
                secret_kind,
                store_kind,
                label,
                metadata,
                created_at,
                updated_at
            FROM secret_references
            WHERE secret_ref = $1
            "#,
        )
        .bind(secret_ref.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_secret_reference).transpose()
    }

    pub async fn delete_secret_reference(
        &self,
        secret_ref: &str,
    ) -> Result<bool, SecretReferenceError> {
        validate_non_empty("secret_ref", secret_ref)?;

        let deleted = sqlx::query(
            r#"
            DELETE FROM secret_references
            WHERE secret_ref = $1
            "#,
        )
        .bind(secret_ref.trim())
        .execute(&self.pool)
        .await?;

        Ok(deleted.rows_affected() > 0)
    }
}

fn row_to_secret_reference(row: PgRow) -> Result<SecretReference, SecretReferenceError> {
    let secret_kind = SecretKind::try_from(row.try_get::<String, _>("secret_kind")?.as_str())?;
    let store_kind = SecretStoreKind::try_from(row.try_get::<String, _>("store_kind")?.as_str())?;

    Ok(SecretReference {
        secret_ref: row.try_get("secret_ref")?,
        secret_kind,
        store_kind,
        label: row.try_get("label")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```
