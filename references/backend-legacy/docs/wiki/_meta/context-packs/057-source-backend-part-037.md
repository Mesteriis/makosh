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

- Chunk ID / ID чанка: `057-source-backend-part-037`
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

### `backend/src/engines/search/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/search/engine.rs`
- Size bytes / Размер в байтах: `3435`
- Included characters / Включено символов: `3435`
- Truncated / Обрезано: `no`

```rust
use std::path::Path;
use std::sync::Mutex;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, doc};

use crate::engines::search::errors::SearchError;
use crate::engines::search::models::{
    SearchDocument, SearchFields, SearchResult, document_identity_term, object_identity,
    required_stored_text, validate_non_empty,
};

const INDEX_WRITER_MEMORY_BUDGET_BYTES: usize = 50_000_000;

pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    writer: Mutex<IndexWriter>,
    fields: SearchFields,
}

impl SearchIndex {
    pub fn open_or_create(path: &Path) -> Result<Self, SearchError> {
        let fields = SearchFields::schema();
        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(path)?,
            fields.schema.clone(),
        )?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        let writer = Mutex::new(index.writer(INDEX_WRITER_MEMORY_BUDGET_BYTES)?);

        Ok(Self {
            index,
            reader,
            writer,
            fields,
        })
    }

    pub fn upsert_document(&self, document: &SearchDocument) -> Result<(), SearchError> {
        document.validate()?;

        let writer = self
            .writer
            .lock()
            .map_err(|_| SearchError::WriterLockPoisoned)?;
        writer.delete_term(document_identity_term(
            self.fields.object_identity,
            document,
        ));
        writer.add_document(doc!(
            self.fields.object_identity => object_identity(document),
            self.fields.object_id => document.object_id.clone(),
            self.fields.object_kind => document.object_kind.clone(),
            self.fields.title => document.title.clone(),
            self.fields.body => document.body.clone(),
        ))?;

        Ok(())
    }

    pub fn commit(&self) -> Result<(), SearchError> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| SearchError::WriterLockPoisoned)?;
        writer.commit()?;
        self.reader.reload()?;

        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        validate_non_empty("query", query)?;
        if limit == 0 {
            return Err(SearchError::InvalidLimit);
        }

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.fields.title, self.fields.body]);
        let query = query_parser.parse_query(query.trim())?;
        let searcher = self.reader.searcher();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        top_docs
            .into_iter()
            .map(|(_score, doc_address)| {
                let document = searcher.doc::<TantivyDocument>(doc_address)?;
                Ok(SearchResult {
                    object_id: required_stored_text(&document, self.fields.object_id, "object_id")?,
                    object_kind: required_stored_text(
                        &document,
                        self.fields.object_kind,
                        "object_kind",
                    )?,
                    title: required_stored_text(&document, self.fields.title, "title")?,
                })
            })
            .collect()
    }
}
```

### `backend/src/engines/search/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/search/errors.rs`
- Size bytes / Размер в байтах: `646`
- Included characters / Включено символов: `646`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Tantivy(#[from] tantivy::TantivyError),

    #[error(transparent)]
    OpenDirectory(#[from] tantivy::directory::error::OpenDirectoryError),

    #[error(transparent)]
    QueryParser(#[from] tantivy::query::QueryParserError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("search limit must be greater than zero")]
    InvalidLimit,

    #[error("search index writer lock was poisoned")]
    WriterLockPoisoned,

    #[error("search result missing stored field: {0}")]
    MissingStoredField(&'static str),
}
```

### `backend/src/engines/search/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/search/mod.rs`
- Size bytes / Размер в байтах: `143`
- Included characters / Включено символов: `143`
- Truncated / Обрезано: `no`

```rust
mod engine;
mod errors;
mod models;

pub use engine::SearchIndex;
pub use errors::SearchError;
pub use models::{SearchDocument, SearchResult};
```

### `backend/src/engines/search/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/search/models.rs`
- Size bytes / Размер в байтах: `2751`
- Included characters / Включено символов: `2751`
- Truncated / Обрезано: `no`

```rust
use tantivy::Term;
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT, TantivyDocument, Value};

use crate::engines::search::errors::SearchError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchDocument {
    pub object_id: String,
    pub object_kind: String,
    pub title: String,
    pub body: String,
}

impl SearchDocument {
    pub fn validate(&self) -> Result<(), SearchError> {
        validate_non_empty("object_id", &self.object_id)?;
        validate_non_empty("object_kind", &self.object_kind)?;
        validate_non_empty("title", &self.title)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub object_id: String,
    pub object_kind: String,
    pub title: String,
}

pub struct SearchFields {
    pub schema: Schema,
    pub object_identity: Field,
    pub object_id: Field,
    pub object_kind: Field,
    pub title: Field,
    pub body: Field,
}

impl SearchFields {
    pub fn schema() -> Self {
        let mut schema_builder = Schema::builder();
        let object_identity = schema_builder.add_text_field("object_identity", STRING);
        let object_id = schema_builder.add_text_field("object_id", STRING | STORED);
        let object_kind = schema_builder.add_text_field("object_kind", STRING | STORED);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();

        Self {
            schema,
            object_identity,
            object_id,
            object_kind,
            title,
            body,
        }
    }
}

pub fn object_identity(document: &SearchDocument) -> String {
    let mut encoded = String::from("search:v1:");
    append_identity_component(&mut encoded, &document.object_kind);
    encoded.push(':');
    append_identity_component(&mut encoded, &document.object_id);
    encoded
}

fn append_identity_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}

pub fn document_identity_term(field: Field, document: &SearchDocument) -> Term {
    Term::from_field_text(field, &object_identity(document))
}

pub fn required_stored_text(
    document: &TantivyDocument,
    field: Field,
    field_name: &'static str,
) -> Result<String, SearchError> {
    document
        .get_first(field)
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
        .ok_or(SearchError::MissingStoredField(field_name))
}

pub fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), SearchError> {
    if value.trim().is_empty() {
        return Err(SearchError::EmptyField(field_name));
    }

    Ok(())
}
```

### `backend/src/engines/speaker_identity/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/speaker_identity/engine.rs`
- Size bytes / Размер в байтах: `3152`
- Included characters / Включено символов: `3152`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeMap;

use super::models::{
    SpeakerEvidence, SpeakerIdentityCandidate, SpeakerIdentityMergePlan, SpeakerIdentitySource,
};

#[derive(Clone, Debug, Default)]
pub struct SpeakerIdentityEngine;

impl SpeakerIdentityEngine {
    pub fn merge(&self, evidence: &[SpeakerEvidence]) -> SpeakerIdentityMergePlan {
        let mut grouped: BTreeMap<String, Vec<&SpeakerEvidence>> = BTreeMap::new();
        for item in evidence {
            let key = item
                .person_id
                .clone()
                .unwrap_or_else(|| normalize_label(&item.label));
            grouped.entry(key).or_default().push(item);
        }

        let candidates = grouped
            .into_iter()
            .map(|(key, items)| {
                let evidence_count = items.len();
                let weighted_sum: f32 = items
                    .iter()
                    .map(|item| source_weight(item.source) * item.confidence.clamp(0.0, 1.0))
                    .sum();
                let weight_total: f32 = items.iter().map(|item| source_weight(item.source)).sum();
                let confidence = if weight_total > 0.0 {
                    weighted_sum / weight_total
                } else {
                    0.0
                };
                let display_label = items
                    .iter()
                    .find(|item| !item.label.trim().is_empty())
                    .map(|item| item.label.trim().to_owned())
                    .unwrap_or_else(|| "Unknown speaker".to_owned());
                SpeakerIdentityCandidate {
                    speaker_key: key,
                    display_label,
                    person_id: items.iter().find_map(|item| item.person_id.clone()),
                    confidence,
                    evidence_count,
                    requires_review: confidence < 0.8,
                }
            })
            .collect::<Vec<_>>();
        let unknown_speaker_count = candidates
            .iter()
            .filter(|candidate| candidate.person_id.is_none())
            .count();
        SpeakerIdentityMergePlan {
            candidates,
            unknown_speaker_count,
            policy: "dom_webview_hints_are_supporting_evidence_not_truth".to_owned(),
        }
    }
}

fn source_weight(source: SpeakerIdentitySource) -> f32 {
    match source {
        SpeakerIdentitySource::ManualConfirmation => 1.0,
        SpeakerIdentitySource::VoiceEmbedding => 0.85,
        SpeakerIdentitySource::CalendarAttendee => 0.55,
        SpeakerIdentitySource::ProviderParticipant => 0.5,
        SpeakerIdentitySource::WhisperDiarization => 0.45,
        SpeakerIdentitySource::WebviewDomHint => 0.25,
    }
}

fn normalize_label(value: &str) -> String {
    let normalized = value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if normalized.is_empty() {
        "unknown-speaker".to_owned()
    } else {
        normalized
    }
}
```

### `backend/src/engines/speaker_identity/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/speaker_identity/mod.rs`
- Size bytes / Размер в байтах: `181`
- Included characters / Включено символов: `181`
- Truncated / Обрезано: `no`

```rust
mod engine;
mod models;

pub use engine::SpeakerIdentityEngine;
pub use models::{
    SpeakerEvidence, SpeakerIdentityCandidate, SpeakerIdentityMergePlan, SpeakerIdentitySource,
};
```

### `backend/src/engines/speaker_identity/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/speaker_identity/models.rs`
- Size bytes / Размер в байтах: `1137`
- Included characters / Включено символов: `1137`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerIdentitySource {
    WebviewDomHint,
    WhisperDiarization,
    VoiceEmbedding,
    CalendarAttendee,
    ProviderParticipant,
    ManualConfirmation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerEvidence {
    pub source: SpeakerIdentitySource,
    pub label: String,
    pub person_id: Option<String>,
    pub starts_at_ms: Option<i64>,
    pub ends_at_ms: Option<i64>,
    pub confidence: f32,
    pub evidence: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerIdentityCandidate {
    pub speaker_key: String,
    pub display_label: String,
    pub person_id: Option<String>,
    pub confidence: f32,
    pub evidence_count: usize,
    pub requires_review: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerIdentityMergePlan {
    pub candidates: Vec<SpeakerIdentityCandidate>,
    pub unknown_speaker_count: usize,
    pub policy: String,
}
```

### `backend/src/engines/timeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline.rs`
- Size bytes / Размер в байтах: `3402`
- Included characters / Включено символов: `3402`
- Truncated / Обрезано: `no`

```rust
mod analysis;
mod cross_domain;
mod errors;
mod models;
mod policy;
mod projection;
mod replay;
mod summaries;
mod validation;

use chrono::{DateTime, Utc};

use crate::platform::events::{EventStore, ProjectionCursorStore, StoredEventEnvelope};

pub use errors::{TimelineEngineError, TimelineProjectionError};
pub use models::{
    TimelineChange, TimelineChangeDiff, TimelineEntry, TimelineEventDraft, TimelineGap,
    TimelinePeriodSummary, TimelineProjectionRun, TimelineRecencySignal, TimelineReplay,
};

pub struct TimelineEngine;

impl TimelineEngine {
    pub fn bounded_entity_limit(limit: i64) -> i64 {
        policy::bounded_entity_limit(limit)
    }

    pub fn validate_event(event: &TimelineEventDraft<'_>) -> Result<(), TimelineEngineError> {
        policy::validate_event(event)
    }

    pub fn period_summary(
        events: &[TimelineEventDraft<'_>],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<TimelinePeriodSummary, TimelineEngineError> {
        summaries::period_summary(events, period_start, period_end)
    }

    pub fn recency_signal(
        events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
        as_of: DateTime<Utc>,
    ) -> Result<TimelineRecencySignal, TimelineEngineError> {
        analysis::recency_signal(events, entity_kind, entity_id, as_of)
    }

    pub fn timeline_gaps(
        events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        max_gap_seconds: i64,
    ) -> Result<Vec<TimelineGap>, TimelineEngineError> {
        analysis::timeline_gaps(
            events,
            entity_kind,
            entity_id,
            period_start,
            period_end,
            max_gap_seconds,
        )
    }

    pub fn change_diff(
        previous_events: &[TimelineEventDraft<'_>],
        current_events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
    ) -> Result<TimelineChangeDiff, TimelineEngineError> {
        analysis::change_diff(previous_events, current_events, entity_kind, entity_id)
    }

    pub fn cross_domain_timeline(
        events: &[TimelineEventDraft<'_>],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<TimelineEntry>, TimelineEngineError> {
        cross_domain::cross_domain_timeline(events, period_start, period_end, limit)
    }

    pub fn replay_event_log(
        stored_events: &[StoredEventEnvelope],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit: i64,
    ) -> Result<TimelineReplay, TimelineEngineError> {
        replay::replay_event_log(stored_events, period_start, period_end, limit)
    }

    pub async fn run_event_log_projection(
        events: &EventStore,
        cursors: &ProjectionCursorStore,
        projection_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        batch_size: u32,
        timeline_limit: i64,
    ) -> Result<TimelineProjectionRun, TimelineProjectionError> {
        projection::run_event_log_projection(
            events,
            cursors,
            projection_name,
            period_start,
            period_end,
            batch_size,
            timeline_limit,
        )
        .await
    }
}
```

### `backend/src/engines/timeline/analysis.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/analysis.rs`
- Size bytes / Размер в байтах: `5258`
- Included characters / Включено символов: `5258`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

use super::errors::TimelineEngineError;
use super::models::{
    TimelineChange, TimelineChangeDiff, TimelineEventDraft, TimelineGap, TimelineRecencySignal,
};
use super::policy::validate_event;
use super::validation::validate_non_empty;

pub(super) fn recency_signal(
    events: &[TimelineEventDraft<'_>],
    entity_kind: &str,
    entity_id: &str,
    as_of: DateTime<Utc>,
) -> Result<TimelineRecencySignal, TimelineEngineError> {
    validate_non_empty("entity_kind", entity_kind)?;
    validate_non_empty("entity_id", entity_id)?;

    let entity_kind = entity_kind.trim();
    let entity_id = entity_id.trim();
    let mut latest_event: Option<&TimelineEventDraft<'_>> = None;

    for event in events {
        validate_event(event)?;
        if event.occurred_at > as_of
            || event.entity_kind.trim() != entity_kind
            || event.entity_id.trim() != entity_id
        {
            continue;
        }

        match latest_event {
            Some(current) if current.occurred_at >= event.occurred_at => {}
            _ => latest_event = Some(event),
        }
    }

    let (last_event_at, last_event_type, last_event_source, age_seconds) =
        if let Some(event) = latest_event {
            (
                Some(event.occurred_at),
                Some(event.event_type.trim().to_owned()),
                Some(event.source.trim().to_owned()),
                Some(as_of.signed_duration_since(event.occurred_at).num_seconds()),
            )
        } else {
            (None, None, None, None)
        };

    Ok(TimelineRecencySignal {
        entity_kind: entity_kind.to_owned(),
        entity_id: entity_id.to_owned(),
        last_event_at,
        last_event_type,
        last_event_source,
        age_seconds,
    })
}

pub(super) fn timeline_gaps(
    events: &[TimelineEventDraft<'_>],
    entity_kind: &str,
    entity_id: &str,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    max_gap_seconds: i64,
) -> Result<Vec<TimelineGap>, TimelineEngineError> {
    if period_start > period_end {
        return Err(TimelineEngineError::InvalidPeriod);
    }
    if max_gap_seconds <= 0 {
        return Err(TimelineEngineError::InvalidGapThreshold);
    }
    validate_non_empty("entity_kind", entity_kind)?;
    validate_non_empty("entity_id", entity_id)?;

    let entity_kind = entity_kind.trim();
    let entity_id = entity_id.trim();
    let mut entity_events = Vec::new();

    for event in events {
        validate_event(event)?;
        if event.occurred_at < period_start
            || event.occurred_at > period_end
            || event.entity_kind.trim() != entity_kind
            || event.entity_id.trim() != entity_id
        {
            continue;
        }
        entity_events.push(event);
    }

    entity_events.sort_by_key(|event| event.occurred_at);

    let mut gaps = Vec::new();
    for pair in entity_events.windows(2) {
        let previous = pair[0];
        let next = pair[1];
        let gap_seconds = next
            .occurred_at
            .signed_duration_since(previous.occurred_at)
            .num_seconds();
        if gap_seconds <= max_gap_seconds {
            continue;
        }

        gaps.push(TimelineGap {
            entity_kind: entity_kind.to_owned(),
            entity_id: entity_id.to_owned(),
            gap_start: previous.occurred_at,
            gap_end: next.occurred_at,
            gap_seconds,
            previous_event_source: Some(previous.source.trim().to_owned()),
            next_event_source: Some(next.source.trim().to_owned()),
        });
    }

    Ok(gaps)
}

pub(super) fn change_diff(
    previous_events: &[TimelineEventDraft<'_>],
    current_events: &[TimelineEventDraft<'_>],
    entity_kind: &str,
    entity_id: &str,
) -> Result<TimelineChangeDiff, TimelineEngineError> {
    validate_non_empty("entity_kind", entity_kind)?;
    validate_non_empty("entity_id", entity_id)?;

    let entity_kind = entity_kind.trim();
    let entity_id = entity_id.trim();
    let previous = events_by_source(previous_events, entity_kind, entity_id)?;
    let current = events_by_source(current_events, entity_kind, entity_id)?;

    let mut added = Vec::new();
    for (source, event) in &current {
        if !previous.contains_key(source) {
            added.push(TimelineChange::from_event(event));
        }
    }

    let mut removed = Vec::new();
    for (source, event) in &previous {
        if !current.contains_key(source) {
            removed.push(TimelineChange::from_event(event));
        }
    }

    Ok(TimelineChangeDiff {
        entity_kind: entity_kind.to_owned(),
        entity_id: entity_id.to_owned(),
        added,
        removed,
    })
}

fn events_by_source<'a>(
    events: &'a [TimelineEventDraft<'a>],
    entity_kind: &str,
    entity_id: &str,
) -> Result<BTreeMap<String, &'a TimelineEventDraft<'a>>, TimelineEngineError> {
    let mut by_source = BTreeMap::new();
    for event in events {
        validate_event(event)?;
        if event.entity_kind.trim() == entity_kind && event.entity_id.trim() == entity_id {
            by_source.insert(event.source.trim().to_owned(), event);
        }
    }
    Ok(by_source)
}
```

### `backend/src/engines/timeline/cross_domain.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/cross_domain.rs`
- Size bytes / Размер в байтах: `1034`
- Included characters / Включено символов: `1034`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::errors::TimelineEngineError;
use super::models::{TimelineEntry, TimelineEventDraft};
use super::policy::{bounded_entity_limit, validate_event};

pub(super) fn cross_domain_timeline(
    events: &[TimelineEventDraft<'_>],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<TimelineEntry>, TimelineEngineError> {
    if period_start > period_end {
        return Err(TimelineEngineError::InvalidPeriod);
    }

    let limit = bounded_entity_limit(limit) as usize;
    let mut timeline = Vec::new();

    for event in events {
        validate_event(event)?;
        if event.occurred_at < period_start || event.occurred_at > period_end {
            continue;
        }
        timeline.push(TimelineEntry::from_event(event));
    }

    timeline.sort_by(|left, right| {
        left.occurred_at
            .cmp(&right.occurred_at)
            .then_with(|| left.source.cmp(&right.source))
    });
    timeline.truncate(limit);

    Ok(timeline)
}
```

### `backend/src/engines/timeline/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/errors.rs`
- Size bytes / Размер в байтах: `841`
- Included characters / Включено символов: `841`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::projections::ProjectionRunnerError;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TimelineEngineError {
    #[error("timeline event {0} must not be empty")]
    EmptyField(&'static str),
    #[error("timeline period start must not be after period end")]
    InvalidPeriod,
    #[error("timeline gap threshold must be greater than zero")]
    InvalidGapThreshold,
    #[error("event log event `{event_id}` {object_name}.{field_name} must be a non-empty string")]
    InvalidEventLogField {
        event_id: String,
        object_name: &'static str,
        field_name: &'static str,
    },
}

#[derive(Debug, Error)]
pub enum TimelineProjectionError {
    #[error(transparent)]
    Runner(#[from] ProjectionRunnerError),

    #[error(transparent)]
    Timeline(#[from] TimelineEngineError),
}
```

### `backend/src/engines/timeline/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/models.rs`
- Size bytes / Размер в байтах: `2840`
- Included characters / Включено символов: `2840`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct TimelineEventDraft<'a> {
    pub entity_kind: &'a str,
    pub entity_id: &'a str,
    pub event_type: &'a str,
    pub title: &'a str,
    pub occurred_at: DateTime<Utc>,
    pub source: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelinePeriodSummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: usize,
    pub by_entity_kind: BTreeMap<String, usize>,
    pub by_event_type: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineRecencySignal {
    pub entity_kind: String,
    pub entity_id: String,
    pub last_event_at: Option<DateTime<Utc>>,
    pub last_event_type: Option<String>,
    pub last_event_source: Option<String>,
    pub age_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineGap {
    pub entity_kind: String,
    pub entity_id: String,
    pub gap_start: DateTime<Utc>,
    pub gap_end: DateTime<Utc>,
    pub gap_seconds: i64,
    pub previous_event_source: Option<String>,
    pub next_event_source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineChangeDiff {
    pub entity_kind: String,
    pub entity_id: String,
    pub added: Vec<TimelineChange>,
    pub removed: Vec<TimelineChange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineChange {
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
}

impl TimelineChange {
    pub(super) fn from_event(event: &TimelineEventDraft<'_>) -> Self {
        Self {
            event_type: event.event_type.trim().to_owned(),
            occurred_at: event.occurred_at,
            source: event.source.trim().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineEntry {
    pub entity_kind: String,
    pub entity_id: String,
    pub event_type: String,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
}

impl TimelineEntry {
    pub(super) fn from_event(event: &TimelineEventDraft<'_>) -> Self {
        Self {
            entity_kind: event.entity_kind.trim().to_owned(),
            entity_id: event.entity_id.trim().to_owned(),
            event_type: event.event_type.trim().to_owned(),
            title: event.title.trim().to_owned(),
            occurred_at: event.occurred_at,
            source: event.source.trim().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineReplay {
    pub last_replayed_position: i64,
    pub entries: Vec<TimelineEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineProjectionRun {
    pub processed_count: usize,
    pub last_processed_position: i64,
    pub entries: Vec<TimelineEntry>,
}
```

### `backend/src/engines/timeline/policy.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/policy.rs`
- Size bytes / Размер в байтах: `673`
- Included characters / Включено символов: `673`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::errors::TimelineEngineError;
use super::models::TimelineEventDraft;
use super::validation::validate_non_empty;

pub(super) fn bounded_entity_limit(limit: i64) -> i64 {
    limit.clamp(1, 100)
}

pub(super) fn validate_event(event: &TimelineEventDraft<'_>) -> Result<(), TimelineEngineError> {
    validate_non_empty("entity_kind", event.entity_kind)?;
    validate_non_empty("entity_id", event.entity_id)?;
    validate_non_empty("event_type", event.event_type)?;
    validate_non_empty("title", event.title)?;
    validate_non_empty("source", event.source)?;

    let _occurred_at: DateTime<Utc> = event.occurred_at;

    Ok(())
}
```

### `backend/src/engines/timeline/projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/projection.rs`
- Size bytes / Размер в байтах: `1402`
- Included characters / Включено символов: `1402`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use std::future;

use super::errors::TimelineProjectionError;
use super::models::TimelineProjectionRun;
use super::replay::replay_event_log;
use crate::platform::events::{EventStore, ProjectionCursorStore};
use crate::platform::projections::{ProjectionHandlerError, run_projection_batch};

pub(super) async fn run_event_log_projection(
    events: &EventStore,
    cursors: &ProjectionCursorStore,
    projection_name: &str,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    batch_size: u32,
    timeline_limit: i64,
) -> Result<TimelineProjectionRun, TimelineProjectionError> {
    let mut replay_batch = Vec::new();
    let outcome = run_projection_batch(events, cursors, projection_name, batch_size, |event| {
        let validation =
            replay_event_log(std::slice::from_ref(&event), period_start, period_end, 1)
                .map(|_| ())
                .map_err(|error| ProjectionHandlerError::new(error.to_string()));
        if validation.is_ok() {
            replay_batch.push(event);
        }
        future::ready(validation)
    })
    .await?;

    let replay = replay_event_log(&replay_batch, period_start, period_end, timeline_limit)?;

    Ok(TimelineProjectionRun {
        processed_count: outcome.processed_count,
        last_processed_position: outcome.last_processed_position,
        entries: replay.entries,
    })
}
```

### `backend/src/engines/timeline/replay.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/replay.rs`
- Size bytes / Размер в байтах: `2005`
- Included characters / Включено символов: `2005`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::errors::TimelineEngineError;
use super::models::{TimelineEntry, TimelineReplay};
use super::policy::bounded_entity_limit;
use super::validation::{event_log_source_ref, optional_json_string, required_json_string};
use crate::platform::events::StoredEventEnvelope;

pub(super) fn replay_event_log(
    stored_events: &[StoredEventEnvelope],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    limit: i64,
) -> Result<TimelineReplay, TimelineEngineError> {
    if period_start > period_end {
        return Err(TimelineEngineError::InvalidPeriod);
    }

    let last_replayed_position = stored_events
        .iter()
        .map(|stored| stored.position)
        .max()
        .unwrap_or(0);
    let limit = bounded_entity_limit(limit) as usize;
    let mut entries = Vec::new();

    for stored in stored_events {
        let event = &stored.event;
        super::validation::validate_non_empty("event_type", &event.event_type)?;
        if event.occurred_at < period_start || event.occurred_at > period_end {
            continue;
        }

        entries.push(TimelineEntry {
            entity_kind: required_json_string(&event.subject, "subject", "kind", &event.event_id)?,
            entity_id: required_json_string(
                &event.subject,
                "subject",
                "entity_id",
                &event.event_id,
            )?,
            event_type: event.event_type.trim().to_owned(),
            title: optional_json_string(&event.payload, "title")
                .unwrap_or_else(|| event.event_type.trim().to_owned()),
            occurred_at: event.occurred_at,
            source: event_log_source_ref(event),
        });
    }

    entries.sort_by(|left, right| {
        left.occurred_at
            .cmp(&right.occurred_at)
            .then_with(|| left.source.cmp(&right.source))
    });
    entries.truncate(limit);

    Ok(TimelineReplay {
        last_replayed_position,
        entries,
    })
}
```

### `backend/src/engines/timeline/summaries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/summaries.rs`
- Size bytes / Размер в байтах: `1188`
- Included characters / Включено символов: `1188`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

use super::errors::TimelineEngineError;
use super::models::{TimelineEventDraft, TimelinePeriodSummary};
use super::policy::validate_event;

pub(super) fn period_summary(
    events: &[TimelineEventDraft<'_>],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> Result<TimelinePeriodSummary, TimelineEngineError> {
    if period_start > period_end {
        return Err(TimelineEngineError::InvalidPeriod);
    }

    let mut summary = TimelinePeriodSummary {
        period_start,
        period_end,
        total_events: 0,
        by_entity_kind: BTreeMap::new(),
        by_event_type: BTreeMap::new(),
    };

    for event in events {
        validate_event(event)?;
        if event.occurred_at < period_start || event.occurred_at > period_end {
            continue;
        }

        summary.total_events += 1;
        *summary
            .by_entity_kind
            .entry(event.entity_kind.trim().to_owned())
            .or_insert(0) += 1;
        *summary
            .by_event_type
            .entry(event.event_type.trim().to_owned())
            .or_insert(0) += 1;
    }

    Ok(summary)
}
```

### `backend/src/engines/timeline/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/timeline/validation.rs`
- Size bytes / Размер в байтах: `1468`
- Included characters / Включено символов: `1468`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::TimelineEngineError;

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), TimelineEngineError> {
    if value.trim().is_empty() {
        return Err(TimelineEngineError::EmptyField(field));
    }
    Ok(())
}

pub(super) fn required_json_string(
    value: &Value,
    object_name: &'static str,
    field_name: &'static str,
    event_id: &str,
) -> Result<String, TimelineEngineError> {
    let field_value = value
        .get(field_name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| TimelineEngineError::InvalidEventLogField {
            event_id: event_id.to_owned(),
            object_name,
            field_name,
        })?;

    Ok(field_value.to_owned())
}

pub(super) fn optional_json_string(value: &Value, field_name: &str) -> Option<String> {
    value
        .get(field_name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn event_log_source_ref(event: &crate::platform::events::EventEnvelope) -> String {
    let Some(kind) = optional_json_string(&event.source, "kind") else {
        return event.event_id.clone();
    };
    let Some(source_id) = optional_json_string(&event.source, "source_id") else {
        return event.event_id.clone();
    };

    format!("{kind}:{source_id}")
}
```

### `backend/src/engines/trust/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/trust/engine.rs`
- Size bytes / Размер в байтах: `1482`
- Included characters / Включено символов: `1482`
- Truncated / Обрезано: `no`

```rust
use crate::engines::trust::errors::TrustEngineError;
use crate::engines::trust::models::{
    TrustImpactDirection, TrustRelationshipSignal, TrustSignalKind, TrustSourceReliabilitySignal,
    normalize_compatibility_score, validate_confidence, validate_non_empty,
};

pub struct TrustEngine;

impl TrustEngine {
    pub fn persona_compatibility_score_signal(score: i16) -> TrustRelationshipSignal {
        TrustRelationshipSignal {
            kind: TrustSignalKind::PersonaCompatibilityScore,
            relationship_type: "trusts",
            trust_score: normalize_compatibility_score(score),
            strength_score: 0.5,
            confidence: 1.0,
            explanation: "compatibility persons.trust_score signal",
        }
    }

    pub fn source_reliability_signal(
        affected_source: &str,
        evidence: &str,
        confidence: f64,
    ) -> Result<TrustSourceReliabilitySignal, TrustEngineError> {
        validate_non_empty("affected source", affected_source)?;
        validate_non_empty("evidence", evidence)?;
        validate_confidence(confidence)?;

        Ok(TrustSourceReliabilitySignal {
            kind: TrustSignalKind::SourceReliability,
            affected_source: affected_source.trim().to_owned(),
            evidence: evidence.trim().to_owned(),
            confidence,
            direction: TrustImpactDirection::from_confidence(confidence),
            explanation: "source reliability signal for review",
        })
    }
}
```

### `backend/src/engines/trust/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/trust/errors.rs`
- Size bytes / Размер в байтах: `267`
- Included characters / Включено символов: `267`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TrustEngineError {
    #[error("trust signal {0} must not be empty")]
    EmptyField(&'static str),

    #[error("trust signal confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),
}
```

### `backend/src/engines/trust/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/trust/mod.rs`
- Size bytes / Размер в байтах: `133`
- Included characters / Включено символов: `133`
- Truncated / Обрезано: `no`

```rust
mod engine;
mod errors;
mod models;

pub use engine::TrustEngine;
pub use errors::TrustEngineError;
pub use models::TrustSignalKind;
```

### `backend/src/engines/trust/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/trust/models.rs`
- Size bytes / Размер в байтах: `1989`
- Included characters / Включено символов: `1989`
- Truncated / Обрезано: `no`

```rust
use crate::engines::trust::errors::TrustEngineError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustSignalKind {
    PersonaCompatibilityScore,
    SourceReliability,
}

impl TrustSignalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersonaCompatibilityScore => "persona_compatibility_score",
            Self::SourceReliability => "source_reliability",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrustRelationshipSignal {
    pub kind: TrustSignalKind,
    pub relationship_type: &'static str,
    pub trust_score: f64,
    pub strength_score: f64,
    pub confidence: f64,
    pub explanation: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrustSourceReliabilitySignal {
    pub kind: TrustSignalKind,
    pub affected_source: String,
    pub evidence: String,
    pub confidence: f64,
    pub direction: TrustImpactDirection,
    pub explanation: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustImpactDirection {
    Positive,
    Negative,
}

impl TrustImpactDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }

    pub fn from_confidence(confidence: f64) -> Self {
        if confidence >= 0.5 {
            Self::Positive
        } else {
            Self::Negative
        }
    }
}

pub fn normalize_compatibility_score(score: i16) -> f64 {
    (f64::from(score.clamp(0, 100)) / 100.0 * 10000.0).round() / 10000.0
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), TrustEngineError> {
    if value.trim().is_empty() {
        return Err(TrustEngineError::EmptyField(field));
    }
    Ok(())
}

pub fn validate_confidence(confidence: f64) -> Result<(), TrustEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(TrustEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}
```

### `backend/src/integrations/ai_runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ai_runtime.rs`
- Size bytes / Размер в байтах: `5054`
- Included characters / Включено символов: `5054`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;
use std::pin::Pin;

use thiserror::Error;

use crate::integrations::ollama::client::{OllamaClient, OllamaError};
use crate::integrations::omniroute::client::{OmniRouteClient, OmniRouteError};
use crate::platform::ai_runtime::{AiChatResult, AiEmbedResult, AiRuntimePort, AiRuntimePortError};

#[derive(Clone)]
pub enum AiRuntimeClient {
    Ollama(OllamaClient),
    OmniRoute(OmniRouteClient),
}

impl AiRuntimeClient {
    pub fn runtime_name(&self) -> &'static str {
        match self {
            Self::Ollama(_) => "ollama",
            Self::OmniRoute(_) => "omniroute",
        }
    }

    pub fn chat_model(&self) -> &str {
        match self {
            Self::Ollama(client) => client.chat_model(),
            Self::OmniRoute(client) => client.chat_model(),
        }
    }

    pub fn embedding_model(&self) -> &str {
        match self {
            Self::Ollama(client) => client.embedding_model(),
            Self::OmniRoute(client) => client.embedding_model(),
        }
    }

    pub async fn version(&self) -> Result<Option<String>, AiRuntimeError> {
        match self {
            Self::Ollama(client) => client.version().await.map(Some).map_err(Into::into),
            Self::OmniRoute(_) => Ok(None),
        }
    }

    pub async fn models(&self) -> Result<Vec<String>, AiRuntimeError> {
        match self {
            Self::Ollama(client) => client.tags().await.map_err(Into::into),
            Self::OmniRoute(client) => client.models().await.map_err(Into::into),
        }
    }

    pub async fn validate_required_models(&self) -> Result<(), AiRuntimeError> {
        match self {
            Self::Ollama(client) => client.validate_required_models().await.map_err(Into::into),
            Self::OmniRoute(client) => client.validate_required_models().await.map_err(Into::into),
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<AiChatResult, AiRuntimeError> {
        self.chat_with_model(prompt, self.chat_model()).await
    }

    pub async fn chat_with_model(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<AiChatResult, AiRuntimeError> {
        match self {
            Self::Ollama(client) => {
                let result = client.chat_with_model(prompt, model).await?;
                Ok(AiChatResult {
                    model: result.model,
                    content: result.content,
                    total_duration_ns: result.total_duration_ns,
                })
            }
            Self::OmniRoute(client) => {
                let result = client.chat_with_model(prompt, model).await?;
                Ok(AiChatResult {
                    model: result.model,
                    content: result.content,
                    total_duration_ns: None,
                })
            }
        }
    }

    pub async fn embed(&self, input: &str) -> Result<AiEmbedResult, AiRuntimeError> {
        self.embed_with_model(input, self.embedding_model()).await
    }

    pub async fn embed_with_model(
        &self,
        input: &str,
        model: &str,
    ) -> Result<AiEmbedResult, AiRuntimeError> {
        match self {
            Self::Ollama(client) => {
                let result = client.embed_with_model(input, model).await?;
                Ok(AiEmbedResult {
                    model: result.model,
                    embedding: result.embedding,
                    total_duration_ns: result.total_duration_ns,
                })
            }
            Self::OmniRoute(client) => {
                let result = client.embed_with_model(input, model).await?;
                Ok(AiEmbedResult {
                    model: result.model,
                    embedding: result.embedding,
                    total_duration_ns: None,
                })
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum AiRuntimeError {
    #[error(transparent)]
    Ollama(#[from] OllamaError),

    #[error(transparent)]
    OmniRoute(#[from] OmniRouteError),
}

impl From<AiRuntimeError> for AiRuntimePortError {
    fn from(error: AiRuntimeError) -> Self {
        match error {
            AiRuntimeError::Ollama(error) => Self::provider("ollama", error.to_string()),
            AiRuntimeError::OmniRoute(error) => Self::provider("omniroute", error.to_string()),
        }
    }
}

impl AiRuntimePort for AiRuntimeClient {
    fn runtime_name(&self) -> &'static str {
        AiRuntimeClient::runtime_name(self)
    }

    fn chat<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiChatResult, AiRuntimePortError>> + Send + 'a>> {
        Box::pin(async move { self.chat(prompt).await.map_err(Into::into) })
    }

    fn embed_with_model<'a>(
        &'a self,
        input: &'a str,
        model: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiEmbedResult, AiRuntimePortError>> + Send + 'a>> {
        Box::pin(async move {
            self.embed_with_model(input, model)
                .await
                .map_err(Into::into)
        })
    }
}
```

### `backend/src/integrations/mail/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts.rs`
- Size bytes / Размер в байтах: `303`
- Included characters / Включено символов: `303`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod errors;
mod helpers;
mod models;
mod service;
mod validation;
mod vault;

pub use errors::EmailAccountSetupError;
pub use models::{
    EmailAccountSetupResult, GmailOAuthPendingGrant, GmailOAuthSetupRequest,
    ImapAccountSetupRequest,
};
pub use service::EmailAccountSetupService;
```

### `backend/src/integrations/mail/accounts/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/constants.rs`
- Size bytes / Размер в байтах: `795`
- Included characters / Включено символов: `795`
- Truncated / Обрезано: `no`

```rust
pub(super) const DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT: &str =
    "https://accounts.google.com/o/oauth2/v2/auth";
pub(super) const DEFAULT_GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

const GOOGLE_GMAIL_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/gmail.readonly";
pub(crate) const GOOGLE_GMAIL_SEND_SCOPE: &str = "https://www.googleapis.com/auth/gmail.send";
const GOOGLE_CALENDAR_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const GOOGLE_CONTACTS_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/contacts.readonly";

pub(super) const DEFAULT_GOOGLE_WORKSPACE_SCOPES: [&str; 4] = [
    GOOGLE_GMAIL_READONLY_SCOPE,
    GOOGLE_GMAIL_SEND_SCOPE,
    GOOGLE_CALENDAR_READONLY_SCOPE,
    GOOGLE_CONTACTS_READONLY_SCOPE,
];
```

### `backend/src/integrations/mail/accounts/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/errors.rs`
- Size bytes / Размер в байтах: `1110`
- Included characters / Включено символов: `1110`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::secrets::{
    DatabaseEncryptedVaultError, SecretReferenceError, SecretResolutionError,
};
use crate::vault::HostVaultError;

#[derive(Debug, Error)]
pub enum EmailAccountSetupError {
    #[error("invalid account setup request field {field}: {message}")]
    InvalidRequest {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider response is missing required field: {field}")]
    MissingProviderField { field: &'static str },

    #[error("account setup stores are not configured")]
    StoresNotConfigured,

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    DatabaseVault(#[from] DatabaseEncryptedVaultError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    Secret(#[from] SecretResolutionError),

    #[error("provider account store operation failed: {0}")]
    ProviderAccountStore(String),
}
```
