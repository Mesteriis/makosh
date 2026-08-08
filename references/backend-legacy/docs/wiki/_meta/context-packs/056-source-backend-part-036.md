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

- Chunk ID / ID чанка: `056-source-backend-part-036`
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

### `backend/src/engines/enrichment/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/enrichment/errors.rs`
- Size bytes / Размер в байтах: `370`
- Included characters / Включено символов: `370`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum EnrichmentEngineError {
    #[error("enrichment candidate {0} must not be empty")]
    EmptyField(&'static str),

    #[error("enrichment candidate confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),

    #[error("enrichment candidate data must be a JSON object")]
    InvalidData,
}
```

### `backend/src/engines/enrichment/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/enrichment/mod.rs`
- Size bytes / Размер в байтах: `110`
- Included characters / Включено символов: `110`
- Truncated / Обрезано: `no`

```rust
mod engine;
mod errors;
mod models;

pub use engine::EnrichmentEngine;
pub use errors::EnrichmentEngineError;
```

### `backend/src/engines/enrichment/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/enrichment/models.rs`
- Size bytes / Размер в байтах: `1017`
- Included characters / Включено символов: `1017`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::engines::enrichment::errors::EnrichmentEngineError;

#[derive(Clone, Debug, PartialEq)]
pub struct PreferenceDraft {
    pub preference_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnrichmentCandidateDraft {
    pub entity_kind: String,
    pub entity_id: String,
    pub source: String,
    pub extracted_claim: String,
    pub data: Value,
    pub confidence: f64,
    pub review_state: String,
    pub freshness: String,
    pub conflict_marker: bool,
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), EnrichmentEngineError> {
    if value.trim().is_empty() {
        return Err(EnrichmentEngineError::EmptyField(field));
    }
    Ok(())
}

pub fn validate_confidence(confidence: f64) -> Result<(), EnrichmentEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(EnrichmentEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}
```

### `backend/src/engines/identity_resolution/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/identity_resolution/mod.rs`
- Size bytes / Размер в байтах: `3649`
- Included characters / Включено символов: `3649`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IdentityResolutionSubject {
    pub entity_kind: String,
    pub entity_id: String,
}

impl IdentityResolutionSubject {
    pub fn new(entity_kind: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self {
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdentityResolutionCandidate {
    pub candidate_id: String,
    pub left: IdentityResolutionSubject,
    pub right: IdentityResolutionSubject,
    pub confidence: f64,
    pub evidence_observation_ids: Vec<String>,
}

impl IdentityResolutionCandidate {
    pub fn same_entity_candidate(
        left: IdentityResolutionSubject,
        right: IdentityResolutionSubject,
        confidence: f64,
        evidence_observation_ids: Vec<String>,
    ) -> Result<Self, IdentityResolutionError> {
        validate_subject(&left)?;
        validate_subject(&right)?;
        if left == right {
            return Err(IdentityResolutionError::SameSubject);
        }
        validate_confidence(confidence)?;
        validate_evidence(&evidence_observation_ids)?;

        Ok(Self {
            candidate_id: format!(
                "identity_resolution_candidate:v1:{}:{}:{}:{}",
                left.entity_kind, left.entity_id, right.entity_kind, right.entity_id
            ),
            left,
            right,
            confidence,
            evidence_observation_ids,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum IdentityResolutionError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("identity resolution candidate must compare two different subjects")]
    SameSubject,

    #[error("identity resolution candidate evidence is required")]
    MissingEvidence,

    #[error("confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(String),
}

fn validate_subject(subject: &IdentityResolutionSubject) -> Result<(), IdentityResolutionError> {
    validate_non_empty("entity_kind", &subject.entity_kind)?;
    validate_non_empty("entity_id", &subject.entity_id)?;
    Ok(())
}

fn validate_evidence(evidence_observation_ids: &[String]) -> Result<(), IdentityResolutionError> {
    if evidence_observation_ids.is_empty() {
        return Err(IdentityResolutionError::MissingEvidence);
    }
    for observation_id in evidence_observation_ids {
        validate_non_empty("evidence_observation_id", observation_id)?;
    }
    Ok(())
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), IdentityResolutionError> {
    if value.trim().is_empty() {
        return Err(IdentityResolutionError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), IdentityResolutionError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(IdentityResolutionError::InvalidConfidence(
            confidence.to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_resolution_candidate_requires_evidence() {
        let error = IdentityResolutionCandidate::same_entity_candidate(
            IdentityResolutionSubject::new("persona", "person:v1:ivan-a"),
            IdentityResolutionSubject::new("persona", "person:v1:ivan-b"),
            0.82,
            vec![],
        )
        .expect_err("missing evidence must be rejected");

        assert_eq!(error, IdentityResolutionError::MissingEvidence);
    }
}
```

### `backend/src/engines/memory.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory.rs`
- Size bytes / Размер в байтах: `2657`
- Included characters / Включено символов: `2657`
- Truncated / Обрезано: `no`

```rust
mod cards;
mod context;
mod cross_domain;
mod errors;
mod facts;
mod gaps;
mod models;
mod stale;
mod validation;

use chrono::{DateTime, Utc};

pub use errors::MemoryEngineError;
pub use models::{
    CrossDomainMemoryContextPack, MemoryCardDraft, MemoryContextItem, MemoryContextPack,
    MemoryContextSource, MemoryCrossDomainContextItem, MemoryEntityRef, MemoryFactDraft,
    MemoryFactState, MemoryGap, MemoryStaleCandidate,
};

pub struct MemoryEngine;

impl MemoryEngine {
    pub fn persona_notes_memory_card(person_id: &str, notes: &str) -> Option<MemoryCardDraft> {
        cards::persona_notes_memory_card(person_id, notes)
    }

    pub fn persona_fact_memory(
        person_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
    ) -> Result<MemoryFactDraft, MemoryEngineError> {
        facts::persona_fact_memory(person_id, fact_type, value, source, confidence)
    }

    pub fn context_pack(
        affected_entity_kind: &str,
        affected_entity_id: &str,
        facts: &[MemoryFactDraft],
        cards: &[MemoryCardDraft],
        limit: i64,
    ) -> Result<MemoryContextPack, MemoryEngineError> {
        context::context_pack(
            affected_entity_kind,
            affected_entity_id,
            facts,
            cards,
            limit,
        )
    }

    pub fn memory_gaps(
        affected_entity_kind: &str,
        affected_entity_id: &str,
        required_fact_types: &[&str],
        facts: &[MemoryFactDraft],
    ) -> Result<Vec<MemoryGap>, MemoryEngineError> {
        gaps::memory_gaps(
            affected_entity_kind,
            affected_entity_id,
            required_fact_types,
            facts,
        )
    }

    pub fn stale_memory_candidates(
        affected_entity_kind: &str,
        affected_entity_id: &str,
        facts: &[MemoryFactState],
        as_of: DateTime<Utc>,
        stale_after_days: i64,
    ) -> Result<Vec<MemoryStaleCandidate>, MemoryEngineError> {
        stale::stale_memory_candidates(
            affected_entity_kind,
            affected_entity_id,
            facts,
            as_of,
            stale_after_days,
        )
    }

    pub fn cross_domain_context_pack(
        root_entity_kind: &str,
        root_entity_id: &str,
        related_entities: &[MemoryEntityRef],
        sources: &[MemoryContextSource],
        limit: i64,
    ) -> Result<CrossDomainMemoryContextPack, MemoryEngineError> {
        cross_domain::cross_domain_context_pack(
            root_entity_kind,
            root_entity_id,
            related_entities,
            sources,
            limit,
        )
    }
}
```

### `backend/src/engines/memory/cards.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/cards.rs`
- Size bytes / Размер в байтах: `464`
- Included characters / Включено символов: `464`
- Truncated / Обрезано: `no`

```rust
use super::models::MemoryCardDraft;

pub(super) fn persona_notes_memory_card(person_id: &str, notes: &str) -> Option<MemoryCardDraft> {
    let description = notes.trim();
    if description.is_empty() {
        return None;
    }

    Some(MemoryCardDraft {
        title: "Compatibility notes".to_owned(),
        description: description.to_owned(),
        source: format!("persons.notes:{person_id}"),
        confidence: 1.0,
        importance: 5,
    })
}
```

### `backend/src/engines/memory/context.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/context.rs`
- Size bytes / Размер в байтах: `2851`
- Included characters / Включено символов: `2851`
- Truncated / Обрезано: `no`

```rust
use std::cmp::Ordering;

use super::errors::MemoryEngineError;
use super::models::{MemoryCardDraft, MemoryContextItem, MemoryContextPack, MemoryFactDraft};
use super::validation::{validate_memory_card, validate_memory_fact, validate_non_empty};

pub(super) fn context_pack(
    affected_entity_kind: &str,
    affected_entity_id: &str,
    facts: &[MemoryFactDraft],
    cards: &[MemoryCardDraft],
    limit: i64,
) -> Result<MemoryContextPack, MemoryEngineError> {
    validate_non_empty("affected entity kind", affected_entity_kind)?;
    validate_non_empty("affected entity", affected_entity_id)?;

    let affected_entity_kind = affected_entity_kind.trim();
    let affected_entity_id = affected_entity_id.trim();
    let mut items = Vec::new();

    for card in cards {
        validate_memory_card(card)?;
        items.push(MemoryContextItem {
            item_kind: "memory_card".to_owned(),
            title: card.title.trim().to_owned(),
            body: card.description.trim().to_owned(),
            source: card.source.trim().to_owned(),
            confidence: card.confidence,
            review_state: "accepted".to_owned(),
        });
    }

    for fact in facts {
        validate_memory_fact(fact)?;
        if fact.affected_entity_kind.trim() != affected_entity_kind
            || fact.affected_entity_id.trim() != affected_entity_id
        {
            continue;
        }

        items.push(MemoryContextItem {
            item_kind: "fact".to_owned(),
            title: fact.fact_type.trim().to_owned(),
            body: fact.value.trim().to_owned(),
            source: fact.source.trim().to_owned(),
            confidence: fact.confidence,
            review_state: fact.review_state.trim().to_owned(),
        });
    }

    items.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.item_kind.cmp(&right.item_kind))
            .then_with(|| left.source.cmp(&right.source))
    });
    items.truncate(limit.clamp(1, 50) as usize);

    let mut source_citations = Vec::new();
    for item in &items {
        if !source_citations.contains(&item.source) {
            source_citations.push(item.source.clone());
        }
    }

    let confidence = aggregate_confidence(&items);

    Ok(MemoryContextPack {
        affected_entity_kind: affected_entity_kind.to_owned(),
        affected_entity_id: affected_entity_id.to_owned(),
        items,
        source_citations,
        confidence,
        produced_by: "memory_engine".to_owned(),
    })
}

fn aggregate_confidence(items: &[MemoryContextItem]) -> f64 {
    if items.is_empty() {
        return 0.0;
    }

    let sum: f64 = items.iter().map(|item| item.confidence).sum();
    ((sum / items.len() as f64) * 100.0).round() / 100.0
}
```

### `backend/src/engines/memory/cross_domain.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/cross_domain.rs`
- Size bytes / Размер в байтах: `5120`
- Included characters / Включено символов: `5120`
- Truncated / Обрезано: `no`

```rust
use std::cmp::Ordering;

use super::errors::MemoryEngineError;
use super::models::{
    CrossDomainMemoryContextPack, MemoryContextSource, MemoryCrossDomainContextItem,
    MemoryEntityRef,
};
use super::validation::{
    validate_memory_context_source, validate_memory_entity_ref, validate_non_empty,
};

pub(super) fn cross_domain_context_pack(
    root_entity_kind: &str,
    root_entity_id: &str,
    related_entities: &[MemoryEntityRef],
    sources: &[MemoryContextSource],
    limit: i64,
) -> Result<CrossDomainMemoryContextPack, MemoryEngineError> {
    validate_non_empty("root entity kind", root_entity_kind)?;
    validate_non_empty("root entity", root_entity_id)?;

    let root_entity_kind = root_entity_kind.trim();
    let root_entity_id = root_entity_id.trim();
    for entity in related_entities {
        validate_memory_entity_ref(entity)?;
    }

    let mut items = Vec::new();
    for source in sources {
        validate_memory_context_source(source)?;
        if source.review_state.trim() != "accepted" {
            continue;
        }

        let entity_kind = source.entity_kind.trim();
        let entity_id = source.entity_id.trim();
        let Some((entity_rank, relation_kind)) =
            context_entity_rank(root_entity_kind, root_entity_id, related_entities, source)
        else {
            continue;
        };

        items.push(RankedCrossDomainMemoryContextItem {
            entity_kind: entity_kind.to_owned(),
            entity_id: entity_id.to_owned(),
            relation_kind,
            entity_rank,
            item_kind: source.item_kind.trim().to_owned(),
            title: source.title.trim().to_owned(),
            body: source.body.trim().to_owned(),
            source: source.source.trim().to_owned(),
            confidence: source.confidence,
            review_state: source.review_state.trim().to_owned(),
        });
    }

    items.sort_by(|left, right| {
        left.entity_rank
            .cmp(&right.entity_rank)
            .then_with(|| {
                right
                    .confidence
                    .partial_cmp(&left.confidence)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| left.source.cmp(&right.source))
    });
    items.truncate(limit.clamp(1, 50) as usize);

    let entity_citations = entity_citations(&items);
    let source_citations = source_citations(&items);
    let confidence = aggregate_confidence(&items);
    let items = items
        .into_iter()
        .map(|item| MemoryCrossDomainContextItem {
            entity_kind: item.entity_kind,
            entity_id: item.entity_id,
            relation_kind: item.relation_kind,
            item_kind: item.item_kind,
            title: item.title,
            body: item.body,
            source: item.source,
            confidence: item.confidence,
            review_state: item.review_state,
        })
        .collect();

    Ok(CrossDomainMemoryContextPack {
        root_entity_kind: root_entity_kind.to_owned(),
        root_entity_id: root_entity_id.to_owned(),
        items,
        entity_citations,
        source_citations,
        confidence,
        produced_by: "memory_engine".to_owned(),
    })
}

struct RankedCrossDomainMemoryContextItem {
    entity_kind: String,
    entity_id: String,
    relation_kind: String,
    entity_rank: usize,
    item_kind: String,
    title: String,
    body: String,
    source: String,
    confidence: f64,
    review_state: String,
}

fn context_entity_rank(
    root_entity_kind: &str,
    root_entity_id: &str,
    related_entities: &[MemoryEntityRef],
    source: &MemoryContextSource,
) -> Option<(usize, String)> {
    let entity_kind = source.entity_kind.trim();
    let entity_id = source.entity_id.trim();
    if entity_kind == root_entity_kind && entity_id == root_entity_id {
        return Some((0, "self".to_owned()));
    }

    related_entities
        .iter()
        .enumerate()
        .find(|(_, entity)| {
            entity.entity_kind.trim() == entity_kind && entity.entity_id.trim() == entity_id
        })
        .map(|(index, entity)| (index + 1, entity.relation_kind.trim().to_owned()))
}

fn entity_citations(items: &[RankedCrossDomainMemoryContextItem]) -> Vec<String> {
    let mut entity_citations = Vec::new();
    for item in items {
        let entity_citation = format!("{}:{}", item.entity_kind, item.entity_id);
        if !entity_citations.contains(&entity_citation) {
            entity_citations.push(entity_citation);
        }
    }
    entity_citations
}

fn source_citations(items: &[RankedCrossDomainMemoryContextItem]) -> Vec<String> {
    let mut source_citations = Vec::new();
    for item in items {
        if !source_citations.contains(&item.source) {
            source_citations.push(item.source.clone());
        }
    }
    source_citations
}

fn aggregate_confidence(items: &[RankedCrossDomainMemoryContextItem]) -> f64 {
    if items.is_empty() {
        return 0.0;
    }

    let sum: f64 = items.iter().map(|item| item.confidence).sum();
    ((sum / items.len() as f64) * 100.0).round() / 100.0
}
```

### `backend/src/engines/memory/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/errors.rs`
- Size bytes / Размер в байтах: `352`
- Included characters / Включено символов: `352`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum MemoryEngineError {
    #[error("memory {0} must not be empty")]
    EmptyField(&'static str),
    #[error("memory confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),
    #[error("memory stale threshold days must be greater than zero")]
    InvalidStaleThreshold,
}
```

### `backend/src/engines/memory/facts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/facts.rs`
- Size bytes / Размер в байтах: `938`
- Included characters / Включено символов: `938`
- Truncated / Обрезано: `no`

```rust
use super::errors::MemoryEngineError;
use super::models::MemoryFactDraft;
use super::validation::{validate_confidence, validate_non_empty};

pub(super) fn persona_fact_memory(
    person_id: &str,
    fact_type: &str,
    value: &str,
    source: &str,
    confidence: f64,
) -> Result<MemoryFactDraft, MemoryEngineError> {
    validate_non_empty("affected entity", person_id)?;
    validate_non_empty("fact type", fact_type)?;
    validate_non_empty("value", value)?;
    validate_non_empty("source", source)?;
    validate_confidence(confidence)?;

    Ok(MemoryFactDraft {
        affected_entity_kind: "persona".to_owned(),
        affected_entity_id: person_id.trim().to_owned(),
        fact_type: fact_type.trim().to_owned(),
        value: value.trim().to_owned(),
        source: source.trim().to_owned(),
        confidence,
        review_state: "accepted".to_owned(),
        produced_by: "memory_engine".to_owned(),
    })
}
```

### `backend/src/engines/memory/gaps.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/gaps.rs`
- Size bytes / Размер в байтах: `2467`
- Included characters / Включено символов: `2467`
- Truncated / Обрезано: `no`

```rust
use super::errors::MemoryEngineError;
use super::models::{MemoryFactDraft, MemoryGap};
use super::validation::{validate_memory_fact, validate_non_empty};

pub(super) fn memory_gaps(
    affected_entity_kind: &str,
    affected_entity_id: &str,
    required_fact_types: &[&str],
    facts: &[MemoryFactDraft],
) -> Result<Vec<MemoryGap>, MemoryEngineError> {
    validate_non_empty("affected entity kind", affected_entity_kind)?;
    validate_non_empty("affected entity", affected_entity_id)?;

    let affected_entity_kind = affected_entity_kind.trim();
    let affected_entity_id = affected_entity_id.trim();
    let required = unique_required_fact_types(required_fact_types)?;
    let present = accepted_fact_types_for_entity(affected_entity_kind, affected_entity_id, facts)?;

    let gaps = required
        .into_iter()
        .filter(|fact_type| !present.contains(fact_type))
        .map(|fact_type| MemoryGap {
            affected_entity_kind: affected_entity_kind.to_owned(),
            affected_entity_id: affected_entity_id.to_owned(),
            missing_fact_type: fact_type.clone(),
            source: format!(
                "memory_engine:gap:{affected_entity_kind}:{affected_entity_id}:{fact_type}"
            ),
            review_state: "suggested".to_owned(),
            produced_by: "memory_engine".to_owned(),
        })
        .collect();

    Ok(gaps)
}

fn unique_required_fact_types(
    required_fact_types: &[&str],
) -> Result<Vec<String>, MemoryEngineError> {
    let mut required = Vec::new();
    for fact_type in required_fact_types {
        validate_non_empty("fact type", fact_type)?;
        let fact_type = fact_type.trim().to_owned();
        if !required.contains(&fact_type) {
            required.push(fact_type);
        }
    }
    Ok(required)
}

fn accepted_fact_types_for_entity(
    affected_entity_kind: &str,
    affected_entity_id: &str,
    facts: &[MemoryFactDraft],
) -> Result<Vec<String>, MemoryEngineError> {
    let mut present = Vec::new();
    for fact in facts {
        validate_memory_fact(fact)?;
        if fact.affected_entity_kind.trim() == affected_entity_kind
            && fact.affected_entity_id.trim() == affected_entity_id
            && fact.review_state.trim() == "accepted"
        {
            let fact_type = fact.fact_type.trim().to_owned();
            if !present.contains(&fact_type) {
                present.push(fact_type);
            }
        }
    }
    Ok(present)
}
```

### `backend/src/engines/memory/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/models.rs`
- Size bytes / Размер в байтах: `3019`
- Included characters / Включено символов: `3019`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryCardDraft {
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryFactDraft {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryFactState {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextPack {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub items: Vec<MemoryContextItem>,
    pub source_citations: Vec<String>,
    pub confidence: f64,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextItem {
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryGap {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub missing_fact_type: String,
    pub source: String,
    pub review_state: String,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryStaleCandidate {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub review_state: String,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryEntityRef {
    pub entity_kind: String,
    pub entity_id: String,
    pub relation_kind: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryContextSource {
    pub entity_kind: String,
    pub entity_id: String,
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrossDomainMemoryContextPack {
    pub root_entity_kind: String,
    pub root_entity_id: String,
    pub items: Vec<MemoryCrossDomainContextItem>,
    pub entity_citations: Vec<String>,
    pub source_citations: Vec<String>,
    pub confidence: f64,
    pub produced_by: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryCrossDomainContextItem {
    pub entity_kind: String,
    pub entity_id: String,
    pub relation_kind: String,
    pub item_kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub confidence: f64,
    pub review_state: String,
}
```

### `backend/src/engines/memory/stale.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/stale.rs`
- Size bytes / Размер в байтах: `2267`
- Included characters / Включено символов: `2267`
- Truncated / Обрезано: `no`

```rust
use std::cmp::Ordering;

use chrono::{DateTime, Utc};

use super::errors::MemoryEngineError;
use super::models::{MemoryFactState, MemoryStaleCandidate};
use super::validation::{validate_memory_fact_state, validate_non_empty};

pub(super) fn stale_memory_candidates(
    affected_entity_kind: &str,
    affected_entity_id: &str,
    facts: &[MemoryFactState],
    as_of: DateTime<Utc>,
    stale_after_days: i64,
) -> Result<Vec<MemoryStaleCandidate>, MemoryEngineError> {
    validate_non_empty("affected entity kind", affected_entity_kind)?;
    validate_non_empty("affected entity", affected_entity_id)?;
    if stale_after_days <= 0 {
        return Err(MemoryEngineError::InvalidStaleThreshold);
    }

    let affected_entity_kind = affected_entity_kind.trim();
    let affected_entity_id = affected_entity_id.trim();
    let stale_cutoff = as_of - chrono::Duration::days(stale_after_days);
    let mut candidates = Vec::new();

    for fact in facts {
        validate_memory_fact_state(fact)?;
        if fact.affected_entity_kind.trim() != affected_entity_kind
            || fact.affected_entity_id.trim() != affected_entity_id
            || fact.review_state.trim() != "accepted"
        {
            continue;
        }

        if fact
            .last_verified_at
            .is_some_and(|verified_at| verified_at >= stale_cutoff)
        {
            continue;
        }

        candidates.push(MemoryStaleCandidate {
            affected_entity_kind: affected_entity_kind.to_owned(),
            affected_entity_id: affected_entity_id.to_owned(),
            fact_type: fact.fact_type.trim().to_owned(),
            value: fact.value.trim().to_owned(),
            source: fact.source.trim().to_owned(),
            confidence: fact.confidence,
            last_verified_at: fact.last_verified_at,
            review_state: "suggested".to_owned(),
            produced_by: "memory_engine".to_owned(),
        });
    }

    candidates.sort_by(|left, right| {
        compare_optional_time(left.last_verified_at, right.last_verified_at)
            .then_with(|| left.source.cmp(&right.source))
    });

    Ok(candidates)
}

fn compare_optional_time(left: Option<DateTime<Utc>>, right: Option<DateTime<Utc>>) -> Ordering {
    left.cmp(&right)
}
```

### `backend/src/engines/memory/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/memory/validation.rs`
- Size bytes / Размер в байтах: `2860`
- Included characters / Включено символов: `2860`
- Truncated / Обрезано: `no`

```rust
use super::errors::MemoryEngineError;
use super::models::MemoryFactState;
use super::models::{MemoryCardDraft, MemoryContextSource, MemoryEntityRef, MemoryFactDraft};

pub(super) fn validate_memory_card(card: &MemoryCardDraft) -> Result<(), MemoryEngineError> {
    validate_non_empty("title", &card.title)?;
    validate_non_empty("description", &card.description)?;
    validate_non_empty("source", &card.source)?;
    validate_confidence(card.confidence)?;
    Ok(())
}

pub(super) fn validate_memory_fact(fact: &MemoryFactDraft) -> Result<(), MemoryEngineError> {
    validate_non_empty("affected entity kind", &fact.affected_entity_kind)?;
    validate_non_empty("affected entity", &fact.affected_entity_id)?;
    validate_non_empty("fact type", &fact.fact_type)?;
    validate_non_empty("value", &fact.value)?;
    validate_non_empty("source", &fact.source)?;
    validate_non_empty("review state", &fact.review_state)?;
    validate_non_empty("producer", &fact.produced_by)?;
    validate_confidence(fact.confidence)?;
    Ok(())
}

pub(super) fn validate_memory_fact_state(fact: &MemoryFactState) -> Result<(), MemoryEngineError> {
    validate_non_empty("affected entity kind", &fact.affected_entity_kind)?;
    validate_non_empty("affected entity", &fact.affected_entity_id)?;
    validate_non_empty("fact type", &fact.fact_type)?;
    validate_non_empty("value", &fact.value)?;
    validate_non_empty("source", &fact.source)?;
    validate_non_empty("review state", &fact.review_state)?;
    validate_confidence(fact.confidence)?;
    Ok(())
}

pub(super) fn validate_memory_entity_ref(
    entity: &MemoryEntityRef,
) -> Result<(), MemoryEngineError> {
    validate_non_empty("entity kind", &entity.entity_kind)?;
    validate_non_empty("entity", &entity.entity_id)?;
    validate_non_empty("relation kind", &entity.relation_kind)?;
    Ok(())
}

pub(super) fn validate_memory_context_source(
    source: &MemoryContextSource,
) -> Result<(), MemoryEngineError> {
    validate_non_empty("entity kind", &source.entity_kind)?;
    validate_non_empty("entity", &source.entity_id)?;
    validate_non_empty("item kind", &source.item_kind)?;
    validate_non_empty("title", &source.title)?;
    validate_non_empty("body", &source.body)?;
    validate_non_empty("source", &source.source)?;
    validate_non_empty("review state", &source.review_state)?;
    validate_confidence(source.confidence)?;
    Ok(())
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), MemoryEngineError> {
    if value.trim().is_empty() {
        return Err(MemoryEngineError::EmptyField(field));
    }
    Ok(())
}

pub(super) fn validate_confidence(confidence: f64) -> Result<(), MemoryEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(MemoryEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}
```

### `backend/src/engines/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/mod.rs`
- Size bytes / Размер в байтах: `288`
- Included characters / Включено символов: `288`
- Truncated / Обрезано: `no`

```rust
pub mod automation;
pub mod call_intelligence;
pub mod consistency;
pub mod context_packs;
pub mod enrichment;
pub mod identity_resolution;
pub mod memory;
pub mod obligation;
pub mod relationships;
pub mod risk;
pub mod search;
pub mod speaker_identity;
pub mod timeline;
pub mod trust;
```

### `backend/src/engines/obligation/detection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/obligation/detection.rs`
- Size bytes / Размер в байтах: `3459`
- Included characters / Включено символов: `3459`
- Truncated / Обрезано: `no`

```rust
use super::errors::ObligationEngineError;
use super::models::{
    ObligationCandidate, ObligationCandidateKind, ObligationExtractionInput, ObligationReviewState,
    validate_non_empty,
};

pub(crate) fn detect_commitment(
    input: &ObligationExtractionInput,
    sentence: &str,
) -> Option<ObligationCandidate> {
    let normalized_sentence = sentence.trim();
    let lower = normalized_sentence.to_lowercase();

    let (kind, statement_start, confidence) = if lower.starts_with("i will ") {
        (ObligationCandidateKind::Commitment, "i will ".len(), 0.84)
    } else if lower.starts_with("i'll ") {
        (ObligationCandidateKind::Commitment, "i'll ".len(), 0.84)
    } else if lower.starts_with("please ") {
        (ObligationCandidateKind::Request, "please ".len(), 0.76)
    } else {
        return None;
    };

    let body = normalized_sentence[statement_start..]
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .trim();
    if body.is_empty() {
        return None;
    }

    let (statement, due_text) = split_due_text(body);
    let (statement, condition) = split_condition(&statement);
    let statement = statement.trim();
    if statement.len() < 3 {
        return None;
    }

    Some(ObligationCandidate {
        kind,
        obligated_entity_kind: input.obligated_entity_kind,
        obligated_entity_id: input.obligated_entity_id.clone(),
        beneficiary_entity_kind: input.beneficiary_entity_kind,
        beneficiary_entity_id: input.beneficiary_entity_id.clone(),
        statement: statement.to_owned(),
        quote: ensure_sentence_terminator(normalized_sentence),
        due_text,
        condition,
        confidence,
        review_state: ObligationReviewState::Suggested,
        evidence_source_kind: input.source_kind,
        evidence_source_id: input.source_id.clone(),
    })
}

fn split_due_text(value: &str) -> (String, Option<String>) {
    let lower = value.to_lowercase();
    for marker in [" by ", " before "] {
        if let Some(index) = lower.find(marker) {
            let statement = value[..index].trim().to_owned();
            let due_text = value[index + marker.len()..]
                .trim()
                .trim_end_matches(['.', '!', '?'])
                .trim()
                .to_owned();
            if !due_text.is_empty() {
                return (statement, Some(due_text));
            }
        }
    }

    (value.trim().to_owned(), None)
}

fn split_condition(value: &str) -> (String, Option<String>) {
    let lower = value.to_lowercase();
    for marker in [" when ", " once ", " if "] {
        if let Some(index) = lower.find(marker) {
            let statement = value[..index].trim().to_owned();
            let condition = value[index + marker.len()..]
                .trim()
                .trim_end_matches(['.', '!', '?'])
                .trim()
                .to_owned();
            if !condition.is_empty() {
                return (statement, Some(condition));
            }
        }
    }

    (value.trim().to_owned(), None)
}

pub(crate) fn sentences(text: &str) -> Vec<&str> {
    text.split_terminator(['\n', '.', '!', '?'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

fn ensure_sentence_terminator(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.ends_with(['.', '!', '?']) {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.")
    }
}
```

### `backend/src/engines/obligation/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/obligation/engine.rs`
- Size bytes / Размер в байтах: `1014`
- Included characters / Включено символов: `1014`
- Truncated / Обрезано: `no`

```rust
use super::detection::{detect_commitment, sentences};
use super::errors::ObligationEngineError;
use super::models::{
    FollowUpCandidate, ObligationExtractionInput, ObligationExtractionResult,
    ObligationTaskCandidate,
};

pub struct ObligationEngine;

impl ObligationEngine {
    pub fn detect_candidates(
        input: &ObligationExtractionInput,
    ) -> Result<ObligationExtractionResult, ObligationEngineError> {
        input.validate()?;

        let mut result = ObligationExtractionResult::default();
        for sentence in sentences(&input.text) {
            if let Some(candidate) = detect_commitment(input, sentence) {
                result
                    .task_candidates
                    .push(ObligationTaskCandidate::from_obligation(&candidate));
                result
                    .follow_ups
                    .push(FollowUpCandidate::from_obligation(&candidate));
                result.obligations.push(candidate);
            }
        }

        Ok(result)
    }
}
```

### `backend/src/engines/obligation/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/obligation/errors.rs`
- Size bytes / Размер в байтах: `248`
- Included characters / Включено символов: `248`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObligationEngineError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("beneficiary entity kind and id must be provided together")]
    PartialBeneficiary,
}
```

### `backend/src/engines/obligation/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/obligation/mod.rs`
- Size bytes / Размер в байтах: `378`
- Included characters / Включено символов: `378`
- Truncated / Обрезано: `no`

```rust
mod detection;
mod engine;
mod errors;
mod models;

pub use engine::ObligationEngine;
pub use errors::ObligationEngineError;
pub use models::{
    FollowUpCandidate, ObligationCandidate, ObligationCandidateKind, ObligationEntityKind,
    ObligationEvidenceSourceKind, ObligationExtractionInput, ObligationExtractionResult,
    ObligationReviewState, ObligationTaskCandidate,
};
```

### `backend/src/engines/obligation/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/obligation/models.rs`
- Size bytes / Размер в байтах: `7421`
- Included characters / Включено символов: `7421`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub struct ObligationExtractionInput {
    pub source_kind: ObligationEvidenceSourceKind,
    pub source_id: String,
    pub text: String,
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
}

impl ObligationExtractionInput {
    pub fn communication(
        source_id: impl Into<String>,
        text: impl Into<String>,
        obligated_entity_kind: ObligationEntityKind,
        obligated_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_kind: ObligationEvidenceSourceKind::Communication,
            source_id: source_id.into(),
            text: text.into(),
            obligated_entity_kind,
            obligated_entity_id: obligated_entity_id.into(),
            beneficiary_entity_kind: None,
            beneficiary_entity_id: None,
        }
    }

    pub fn document(
        source_id: impl Into<String>,
        text: impl Into<String>,
        obligated_entity_kind: ObligationEntityKind,
        obligated_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_kind: ObligationEvidenceSourceKind::Document,
            source_id: source_id.into(),
            text: text.into(),
            obligated_entity_kind,
            obligated_entity_id: obligated_entity_id.into(),
            beneficiary_entity_kind: None,
            beneficiary_entity_id: None,
        }
    }

    pub fn beneficiary(
        mut self,
        beneficiary_entity_kind: ObligationEntityKind,
        beneficiary_entity_id: impl Into<String>,
    ) -> Self {
        self.beneficiary_entity_kind = Some(beneficiary_entity_kind);
        self.beneficiary_entity_id = Some(beneficiary_entity_id.into());
        self
    }

    pub(crate) fn validate(
        &self,
    ) -> Result<(), crate::engines::obligation::errors::ObligationEngineError> {
        use crate::engines::obligation::errors::ObligationEngineError;
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("obligated_entity_id", &self.obligated_entity_id)?;
        match (
            self.beneficiary_entity_kind,
            self.beneficiary_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(beneficiary_entity_id)) => {
                validate_non_empty("beneficiary_entity_id", beneficiary_entity_id)?;
            }
            _ => return Err(ObligationEngineError::PartialBeneficiary),
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationEntityKind {
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

impl ObligationEntityKind {
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
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationEvidenceSourceKind {
    Communication,
    Document,
    CalendarEvent,
    Observation,
    Manual,
}

impl ObligationEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Communication => "communication",
            Self::Document => "document",
            Self::CalendarEvent => "calendar_event",
            Self::Observation => "observation",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ObligationReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObligationExtractionResult {
    pub obligations: Vec<ObligationCandidate>,
    pub task_candidates: Vec<ObligationTaskCandidate>,
    pub follow_ups: Vec<FollowUpCandidate>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationCandidateKind {
    Commitment,
    Request,
}

impl ObligationCandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Commitment => "commitment",
            Self::Request => "request",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObligationCandidate {
    pub kind: ObligationCandidateKind,
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub quote: String,
    pub due_text: Option<String>,
    pub condition: Option<String>,
    pub confidence: f64,
    pub review_state: ObligationReviewState,
    pub evidence_source_kind: ObligationEvidenceSourceKind,
    pub evidence_source_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObligationTaskCandidate {
    pub source_obligation_statement: String,
    pub statement: String,
    pub suggested_title: String,
    pub due_text: Option<String>,
    pub confidence: f64,
}

impl ObligationTaskCandidate {
    pub(crate) fn from_obligation(candidate: &ObligationCandidate) -> Self {
        Self {
            source_obligation_statement: candidate.statement.clone(),
            statement: candidate.statement.clone(),
            suggested_title: candidate.statement.clone(),
            due_text: candidate.due_text.clone(),
            confidence: (candidate.confidence - 0.08).max(0.0),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FollowUpCandidate {
    pub source_obligation_statement: String,
    pub prompt: String,
    pub due_text: Option<String>,
    pub confidence: f64,
}

impl FollowUpCandidate {
    pub(crate) fn from_obligation(candidate: &ObligationCandidate) -> Self {
        Self {
            source_obligation_statement: candidate.statement.clone(),
            prompt: format!("Follow up on: {}", candidate.statement),
            due_text: candidate.due_text.clone(),
            confidence: (candidate.confidence - 0.12).max(0.0),
        }
    }
}

pub(crate) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), crate::engines::obligation::errors::ObligationEngineError> {
    use crate::engines::obligation::errors::ObligationEngineError;
    if value.trim().is_empty() {
        return Err(ObligationEngineError::EmptyField(field_name));
    }
    Ok(())
}
```

### `backend/src/engines/relationships/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/relationships/mod.rs`
- Size bytes / Размер в байтах: `3775`
- Included characters / Включено символов: `3775`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelationshipSubject {
    pub entity_kind: String,
    pub entity_id: String,
}

impl RelationshipSubject {
    pub fn new(entity_kind: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self {
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelationshipCandidate {
    pub candidate_id: String,
    pub source: RelationshipSubject,
    pub target: RelationshipSubject,
    pub relationship_type: String,
    pub confidence: f64,
    pub evidence_observation_ids: Vec<String>,
}

impl RelationshipCandidate {
    pub fn linked_entities_candidate(
        source: RelationshipSubject,
        target: RelationshipSubject,
        relationship_type: impl Into<String>,
        confidence: f64,
        evidence_observation_ids: Vec<String>,
    ) -> Result<Self, RelationshipEngineError> {
        validate_subject(&source)?;
        validate_subject(&target)?;
        validate_confidence(confidence)?;
        validate_evidence(&evidence_observation_ids)?;
        let relationship_type = relationship_type.into();
        validate_non_empty("relationship_type", &relationship_type)?;

        Ok(Self {
            candidate_id: format!(
                "relationship_candidate:v1:{}:{}:{}:{}:{}",
                source.entity_kind,
                source.entity_id,
                relationship_type.trim(),
                target.entity_kind,
                target.entity_id
            ),
            source,
            target,
            relationship_type: relationship_type.trim().to_owned(),
            confidence,
            evidence_observation_ids,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RelationshipEngineError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("relationship candidate evidence is required")]
    MissingEvidence,

    #[error("confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(String),
}

fn validate_subject(subject: &RelationshipSubject) -> Result<(), RelationshipEngineError> {
    validate_non_empty("entity_kind", &subject.entity_kind)?;
    validate_non_empty("entity_id", &subject.entity_id)?;
    Ok(())
}

fn validate_evidence(evidence_observation_ids: &[String]) -> Result<(), RelationshipEngineError> {
    if evidence_observation_ids.is_empty() {
        return Err(RelationshipEngineError::MissingEvidence);
    }
    for observation_id in evidence_observation_ids {
        validate_non_empty("evidence_observation_id", observation_id)?;
    }
    Ok(())
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), RelationshipEngineError> {
    if value.trim().is_empty() {
        return Err(RelationshipEngineError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), RelationshipEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(RelationshipEngineError::InvalidConfidence(
            confidence.to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationship_candidate_requires_evidence() {
        let error = RelationshipCandidate::linked_entities_candidate(
            RelationshipSubject::new("persona", "person:v1:ivan"),
            RelationshipSubject::new("organization", "org:v1:acme"),
            "works_at",
            0.77,
            vec![],
        )
        .expect_err("missing evidence must be rejected");

        assert_eq!(error, RelationshipEngineError::MissingEvidence);
    }
}
```

### `backend/src/engines/risk/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/risk/engine.rs`
- Size bytes / Размер в байтах: `1799`
- Included characters / Включено символов: `1799`
- Truncated / Обрезано: `no`

```rust
use crate::engines::risk::errors::RiskEngineError;
use crate::engines::risk::models::{
    RiskAttentionStatus, RiskObservationDraft, RiskSeverity, RiskSignal, validate_non_empty,
};

pub struct RiskEngine;

impl RiskEngine {
    pub fn derive_attention_status(risks: &[RiskSignal]) -> RiskAttentionStatus {
        let mut has_attention_risk = false;

        for risk in risks.iter().filter(|risk| !risk.resolved) {
            match risk.severity {
                RiskSeverity::Critical | RiskSeverity::High => return RiskAttentionStatus::AtRisk,
                RiskSeverity::Medium | RiskSeverity::Low => has_attention_risk = true,
            }
        }

        if has_attention_risk {
            RiskAttentionStatus::NeedsAttention
        } else {
            RiskAttentionStatus::Healthy
        }
    }

    pub fn persona_observation(
        person_id: &str,
        risk_type: &str,
        evidence: &str,
        severity: &str,
        source: &str,
    ) -> Result<RiskObservationDraft, RiskEngineError> {
        validate_non_empty("affected entity", person_id)?;
        validate_non_empty("risk type", risk_type)?;
        validate_non_empty("evidence", evidence)?;
        validate_non_empty("source", source)?;

        let severity = RiskSeverity::parse(severity)?;

        Ok(RiskObservationDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: person_id.trim().to_owned(),
            risk_type: risk_type.trim().to_owned(),
            evidence: evidence.trim().to_owned(),
            source: source.trim().to_owned(),
            confidence: 0.5,
            severity,
            suggested_handling_state: severity.suggested_handling_state().to_owned(),
            review_state: "suggested".to_owned(),
        })
    }
}
```

### `backend/src/engines/risk/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/risk/errors.rs`
- Size bytes / Размер в байтах: `235`
- Included characters / Включено символов: `235`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiskEngineError {
    #[error("invalid risk severity `{0}`")]
    InvalidSeverity(String),

    #[error("risk observation {0} must not be empty")]
    EmptyField(&'static str),
}
```

### `backend/src/engines/risk/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/risk/mod.rs`
- Size bytes / Размер в байтах: `185`
- Included characters / Включено символов: `185`
- Truncated / Обрезано: `no`

```rust
mod engine;
mod errors;
mod models;

pub use engine::RiskEngine;
pub use errors::RiskEngineError;
pub use models::{RiskAttentionStatus, RiskObservationDraft, RiskSeverity, RiskSignal};
```

### `backend/src/engines/risk/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/risk/models.rs`
- Size bytes / Размер в байтах: `2365`
- Included characters / Включено символов: `2365`
- Truncated / Обрезано: `no`

```rust
use crate::engines::risk::errors::RiskEngineError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskAttentionStatus {
    Healthy,
    NeedsAttention,
    AtRisk,
}

impl RiskAttentionStatus {
    pub fn as_persona_health_status(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::NeedsAttention => "needs_attention",
            Self::AtRisk => "at_risk",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskSeverity {
    pub fn parse(value: &str) -> Result<Self, RiskEngineError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            other => Err(RiskEngineError::InvalidSeverity(other.to_owned())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn suggested_handling_state(self) -> &'static str {
        match self {
            Self::Critical | Self::High => "review_now",
            Self::Medium | Self::Low => "monitor",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RiskObservationDraft {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub risk_type: String,
    pub evidence: String,
    pub source: String,
    pub confidence: f64,
    pub severity: RiskSeverity,
    pub suggested_handling_state: String,
    pub review_state: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RiskSignal {
    pub severity: RiskSeverity,
    pub resolved: bool,
}

impl RiskSignal {
    pub fn unresolved(severity: RiskSeverity) -> Self {
        Self {
            severity,
            resolved: false,
        }
    }

    pub fn resolved(severity: RiskSeverity) -> Self {
        Self {
            severity,
            resolved: true,
        }
    }
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), RiskEngineError> {
    if value.trim().is_empty() {
        return Err(RiskEngineError::EmptyField(field));
    }
    Ok(())
}
```
