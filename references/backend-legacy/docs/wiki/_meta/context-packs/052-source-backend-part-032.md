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

- Chunk ID / ID чанка: `052-source-backend-part-032`
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

### `backend/src/domains/relationships/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/service.rs`
- Size bytes / Размер в байтах: `2190`
- Included characters / Включено символов: `2190`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{Relationship, RelationshipReviewState, RelationshipStore, RelationshipStoreError};

#[derive(Clone)]
pub struct RelationshipCommandService {
    pool: PgPool,
}

impl RelationshipCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
    ) -> Result<Relationship, RelationshipCommandServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "relationship_id": relationship_id,
                        "review_state": review_state.as_str(),
                        "operation": "relationship_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("relationship://{relationship_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "relationships_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(RelationshipStore::new(self.pool.clone())
            .set_review_state_with_observation(
                relationship_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "relationships_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum RelationshipCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
}
```

### `backend/src/domains/relationships/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/store.rs`
- Size bytes / Размер в байтах: `18407`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashSet;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphProjectionPort, GraphReviewState, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType,
};
use crate::platform::observations::materialize_review_transition_link_in_transaction;

use super::errors::RelationshipStoreError;
use super::evidence::link_relationship_entity_in_transaction;
use super::ids::{evidence_id, relationship_id};
use super::models::{
    NewRelationship, NewRelationshipEvidence, Relationship, RelationshipEntityKind,
    RelationshipReviewState,
};
use super::row_mapping::row_to_relationship;
use super::validation::{validate_non_empty, validate_relationship_with_evidence};

#[derive(Clone)]
pub struct RelationshipStore {
    pool: PgPool,
}

impl RelationshipStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipStoreError> {
        validate_relationship_with_evidence(relationship, evidence)?;

        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::upsert_with_evidence_in_transaction(&mut transaction, relationship, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipStoreError> {
        validate_evidence_observations_exist(transaction, evidence).await?;
        let relationship_id = relationship_id(
            relationship.source_entity_kind,
            &relationship.source_entity_id,
            &relationship.relationship_type,
            relationship.target_entity_kind,
            &relationship.target_entity_id,
        );
        let row = sqlx::query(
            r#"
            INSERT INTO relationships (
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score,
                strength_score,
                confidence,
                review_state,
                valid_from,
                valid_to,
                metadata
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                CAST($7 AS NUMERIC(5,4)),
                CAST($8 AS NUMERIC(5,4)),
                CAST($9 AS NUMERIC(5,4)),
                $10,
                $11,
                $12,
                $13
            )
            ON CONFLICT (relationship_id)
            DO UPDATE SET
                trust_score = EXCLUDED.trust_score,
                strength_score = EXCLUDED.strength_score,
                confidence = EXCLUDED.confidence,
                review_state = EXCLUDED.review_state,
                valid_from = EXCLUDED.valid_from,
                valid_to = EXCLUDED.valid_to,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&relationship_id)
        .bind(relationship.source_entity_kind.as_str())
        .bind(&relationship.source_entity_id)
        .bind(relationship.target_entity_kind.as_str())
        .bind(&relationship.target_entity_id)
        .bind(&relationship.relationship_type)
        .bind(relationship.trust_score)
        .bind(relationship.strength_score)
        .bind(relationship.confidence)
        .bind(relationship.review_state.as_str())
        .bind(relationship.valid_from)
        .bind(relationship.valid_to)
        .bind(&relationship.metadata)
        .fetch_one(&mut **transaction)
        .await?;
        let stored = row_to_relationship(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&relationship_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO relationship_evidence (
                    evidence_id,
                    relationship_id,
                    source_kind,
                    source_id,
                    observation_id,
                    excerpt,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (relationship_id, source_kind, source_id)
                DO UPDATE SET
                    observation_id = EXCLUDED.observation_id,
                    excerpt = EXCLUDED.excerpt,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&relationship_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(item.observation_id.as_deref())
            .bind(&item.excerpt)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;

            if let Some(observation_id) = item.observation_id.as_deref() {
                link_relationship_entity_in_transaction(
                    transaction,
                    observation_id,
                    "relationship",
                    relationship_id.clone(),
                    Some("supports"),
                    Some(relationship.confidence),
                    Some(json!({
                        "source_kind": item.source_kind.as_str(),
                        "source_id": item.source_id,
                    })),
                )
                .await?;
            }
        }

        materialize_relationship_graph_in_transaction(transaction, &stored, evidence).await?;

        Ok(stored)
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: RelationshipEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Relationship>, RelationshipStoreError> {
        validate_non_empty("entity_id", entity_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            FROM relationships
            WHERE (source_entity_kind = $1 AND source_entity_id = $2)
               OR (target_entity_kind = $1 AND target_entity_id = $2)
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_relationship).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: RelationshipReviewState,
        limit: i64,
    ) -> Result<Vec<Relationship>, RelationshipStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            FROM relationships
            WHERE review_state = $1
            ORDER BY updated_at DESC, relationship_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_relationship).collect()
    }

    pub async fn set_review_state(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
    ) -> Result<Relationship, RelationshipStoreError> {
        self.set_review_state_with_observation(relationship_id, review_state, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Relationship, RelationshipStoreError> {
        validate_non_empty("relationship_id", relationship_id)?;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE relationships
            SET
                review_state = $1,
                updated_at = now()
            WHERE relationship_id = $2
            RETURNING
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(relationship_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(RelationshipStoreError::RelationshipNotFound)?;

        let relationship = row_to_relationship(row)?;
        materialize_relationship_graph_review_in_transaction(
            &mut transaction,
            &relationship,
            observation_id,
            metadata.as_ref(),
        )
        .await?;
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "relationships",
            "relationship",
            &relationship.relationship_id,
            "review_state",
            relationship.review_state.as_str(),
            metadata,
        )
        .await?;
        transaction.commit().await?;

        Ok(relationship)
    }
}

async fn materialize_relationship_graph_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
    evidence: &[NewRelationshipEvidence],
) -> Result<(), RelationshipStoreError> {
    let graph_evidence = relationship_graph_evidence(relationship, evidence);
    materialize_relationship_graph_with_evidence_in_transaction(
        transaction,
        relationship,
        std::slice::from_ref(&graph_evidence),
    )
    .await
}

async fn materialize_relationship_graph_review_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationsh
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/relationships/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/relationships/validation.rs`
- Size bytes / Размер в байтах: `2917`
- Included characters / Включено символов: `2917`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::RelationshipStoreError;
use super::models::{NewRelationship, NewRelationshipEvidence, RelationshipEvidenceSourceKind};

pub(super) fn validate_relationship_with_evidence(
    relationship: &NewRelationship,
    evidence: &[NewRelationshipEvidence],
) -> Result<(), RelationshipStoreError> {
    validate_relationship(relationship)?;
    if evidence.is_empty() {
        return Err(RelationshipStoreError::MissingEvidence);
    }
    for item in evidence {
        validate_evidence(item)?;
    }

    Ok(())
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), RelationshipStoreError> {
    if value.trim().is_empty() {
        return Err(RelationshipStoreError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_relationship(relationship: &NewRelationship) -> Result<(), RelationshipStoreError> {
    validate_non_empty("source_entity_id", &relationship.source_entity_id)?;
    validate_non_empty("target_entity_id", &relationship.target_entity_id)?;
    validate_non_empty("relationship_type", &relationship.relationship_type)?;
    validate_score("trust_score", relationship.trust_score)?;
    validate_score("strength_score", relationship.strength_score)?;
    validate_score("confidence", relationship.confidence)?;
    validate_json_object("relationship metadata", &relationship.metadata)?;
    if relationship.source_entity_kind == relationship.target_entity_kind
        && relationship.source_entity_id == relationship.target_entity_id
    {
        return Err(RelationshipStoreError::IdenticalEndpoints);
    }
    if let (Some(valid_from), Some(valid_to)) = (relationship.valid_from, relationship.valid_to)
        && valid_to < valid_from
    {
        return Err(RelationshipStoreError::InvalidTemporalRange);
    }

    Ok(())
}

fn validate_evidence(evidence: &NewRelationshipEvidence) -> Result<(), RelationshipStoreError> {
    validate_non_empty("source_id", &evidence.source_id)?;
    if let Some(observation_id) = &evidence.observation_id {
        validate_non_empty("observation_id", observation_id)?;
    }
    if evidence.source_kind == RelationshipEvidenceSourceKind::Observation
        && evidence.observation_id.as_deref() != Some(evidence.source_id.as_str())
    {
        return Err(RelationshipStoreError::InvalidObservationEvidenceSource);
    }
    validate_json_object("evidence metadata", &evidence.metadata)
}

fn validate_score(field_name: &'static str, value: f64) -> Result<(), RelationshipStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(RelationshipStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), RelationshipStoreError> {
    if !value.is_object() {
        return Err(RelationshipStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
```

### `backend/src/domains/review/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/review/errors.rs`
- Size bytes / Размер в байтах: `1312`
- Included characters / Включено символов: `1312`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum ReviewInboxError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("metadata filter must be a JSON object")]
    InvalidMetadataFilter,

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("review item evidence is required")]
    MissingEvidence,

    #[error("review item evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("review item was not found: {0}")]
    ReviewItemNotFound(String),

    #[error("unknown review item kind stored in database: {0}")]
    UnknownItemKind(String),

    #[error("unknown review item status stored in database: {0}")]
    UnknownStatus(String),
}
```

### `backend/src/domains/review/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/review/evidence.rs`
- Size bytes / Размер в байтах: `142`
- Included characters / Включено символов: `142`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::platform::observations::{
    materialize_review_transition_link, materialize_review_transition_link_in_transaction,
};
```

### `backend/src/domains/review/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/review/mod.rs`
- Size bytes / Размер в байтах: `441`
- Included characters / Включено символов: `441`
- Truncated / Обрезано: `no`

```rust
mod errors;
pub(crate) mod evidence;
mod models;
mod service;
mod store;

pub use errors::ReviewInboxError;
pub use models::{
    NewReviewItem, NewReviewItemEvidence, ReviewItem, ReviewItemEvidence, ReviewItemKind,
    ReviewItemStatus, ReviewPromotionTarget,
};
pub use service::{ReviewInboxService, ReviewInboxServiceError};
pub use store::ReviewInboxStore as ReviewInboxPort;
pub use store::{ReviewInboxStore, ReviewItemEvidenceRecord};
```

### `backend/src/domains/review/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/review/models.rs`
- Size bytes / Размер в байтах: `7833`
- Included characters / Включено символов: `7833`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::ReviewInboxError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewItemKind {
    NewPerson,
    NewOrganization,
    IdentityCandidate,
    ProjectLinkCandidate,
    ContradictionCandidate,
    PotentialTask,
    PotentialObligation,
    PotentialDecision,
    PotentialRelationship,
    PotentialProject,
    KnowledgeCandidate,
}

impl ReviewItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NewPerson => "new_person",
            Self::NewOrganization => "new_organization",
            Self::IdentityCandidate => "identity_candidate",
            Self::ProjectLinkCandidate => "project_link_candidate",
            Self::ContradictionCandidate => "contradiction_candidate",
            Self::PotentialTask => "potential_task",
            Self::PotentialObligation => "potential_obligation",
            Self::PotentialDecision => "potential_decision",
            Self::PotentialRelationship => "potential_relationship",
            Self::PotentialProject => "potential_project",
            Self::KnowledgeCandidate => "knowledge_candidate",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReviewInboxError> {
        match value.as_ref() {
            "new_person" => Ok(Self::NewPerson),
            "new_organization" => Ok(Self::NewOrganization),
            "identity_candidate" => Ok(Self::IdentityCandidate),
            "project_link_candidate" => Ok(Self::ProjectLinkCandidate),
            "contradiction_candidate" => Ok(Self::ContradictionCandidate),
            "potential_task" => Ok(Self::PotentialTask),
            "potential_obligation" => Ok(Self::PotentialObligation),
            "potential_decision" => Ok(Self::PotentialDecision),
            "potential_relationship" => Ok(Self::PotentialRelationship),
            "potential_project" => Ok(Self::PotentialProject),
            "knowledge_candidate" => Ok(Self::KnowledgeCandidate),
            unknown => Err(ReviewInboxError::UnknownItemKind(unknown.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewItemStatus {
    New,
    InReview,
    Approved,
    Promoted,
    Dismissed,
    Archived,
}

impl ReviewItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::InReview => "in_review",
            Self::Approved => "approved",
            Self::Promoted => "promoted",
            Self::Dismissed => "dismissed",
            Self::Archived => "archived",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReviewInboxError> {
        match value.as_ref() {
            "new" => Ok(Self::New),
            "in_review" => Ok(Self::InReview),
            "approved" => Ok(Self::Approved),
            "promoted" => Ok(Self::Promoted),
            "dismissed" => Ok(Self::Dismissed),
            "archived" => Ok(Self::Archived),
            unknown => Err(ReviewInboxError::UnknownStatus(unknown.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReviewItem {
    pub review_item_id: String,
    pub item_kind: ReviewItemKind,
    pub title: String,
    pub summary: String,
    pub status: ReviewItemStatus,
    pub target_domain: Option<String>,
    pub target_entity_kind: Option<String>,
    pub target_entity_id: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewReviewItem {
    pub item_kind: ReviewItemKind,
    pub title: String,
    pub summary: String,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewReviewItem {
    pub fn new(
        item_kind: ReviewItemKind,
        title: impl Into<String>,
        summary: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            item_kind,
            title: title.into(),
            summary: summary.into(),
            confidence,
            metadata: json!({}),
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> Result<(), ReviewInboxError> {
        validate_non_empty("title", &self.title)?;
        validate_non_empty("summary", &self.summary)?;
        validate_score("confidence", self.confidence)?;
        validate_json_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReviewItemEvidence {
    pub review_item_id: String,
    pub observation_id: String,
    pub evidence_role: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewReviewItemEvidence {
    pub observation_id: String,
    pub evidence_role: String,
    pub metadata: Value,
}

impl NewReviewItemEvidence {
    pub fn new(observation_id: impl Into<String>) -> Self {
        Self {
            observation_id: observation_id.into(),
            evidence_role: "primary".to_owned(),
            metadata: json!({}),
        }
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.evidence_role = role.into();
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> Result<(), ReviewInboxError> {
        validate_non_empty("observation_id", &self.observation_id)?;
        validate_non_empty("evidence_role", &self.evidence_role)?;
        validate_json_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReviewPromotionTarget {
    pub target_domain: String,
    pub target_entity_kind: String,
    pub target_entity_id: String,
}

impl ReviewPromotionTarget {
    pub fn new(
        target_domain: impl Into<String>,
        target_entity_kind: impl Into<String>,
        target_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            target_domain: target_domain.into(),
            target_entity_kind: target_entity_kind.into(),
            target_entity_id: target_entity_id.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ReviewInboxError> {
        validate_non_empty("target_domain", &self.target_domain)?;
        validate_non_empty("target_entity_kind", &self.target_entity_kind)?;
        validate_non_empty("target_entity_id", &self.target_entity_id)?;
        Ok(())
    }
}

pub(super) fn validate_review_item_with_evidence(
    item: &NewReviewItem,
    evidence: &[NewReviewItemEvidence],
) -> Result<(), ReviewInboxError> {
    item.validate()?;
    if evidence.is_empty() {
        return Err(ReviewInboxError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }
    Ok(())
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ReviewInboxError> {
    if value.trim().is_empty() {
        return Err(ReviewInboxError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ReviewInboxError> {
    if !value.is_object() {
        return Err(ReviewInboxError::InvalidJsonObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_score(field_name: &'static str, value: f64) -> Result<(), ReviewInboxError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(ReviewInboxError::InvalidScore(field_name, value));
    }

    Ok(())
}
```

### `backend/src/domains/review/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/review/service.rs`
- Size bytes / Размер в байтах: `2402`
- Included characters / Включено символов: `2402`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::{ReviewInboxError, ReviewInboxStore, ReviewItem, ReviewItemStatus};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

#[derive(Clone)]
pub struct ReviewInboxService {
    pool: PgPool,
}

impl ReviewInboxService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn transition_status_from_manual(
        &self,
        review_item_id: &str,
        status: ReviewItemStatus,
        captured_by: &'static str,
        endpoint: &'static str,
    ) -> Result<ReviewItem, ReviewInboxServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "review_item_id": review_item_id,
                        "operation": "review_item_status_transition",
                        "status": status.as_str(),
                    }),
                    format!("review-item://{review_item_id}/{}", status.as_str()),
                )
                .provenance(json!({
                    "captured_by": captured_by,
                    "endpoint": endpoint,
                    "status": status.as_str(),
                })),
            )
            .await
            .map_err(ReviewInboxServiceError::StatusObservationCapture)?;

        Ok(ReviewInboxStore::new(self.pool.clone())
            .set_status_with_observation(
                review_item_id,
                status,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": captured_by,
                    "endpoint": endpoint,
                    "status": status.as_str(),
                })),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum ReviewInboxServiceError {
    #[error("review status observation capture failed")]
    StatusObservationCapture(#[source] ObservationStoreError),

    #[error("review promotion observation capture failed")]
    PromotionObservationCapture(#[source] ObservationStoreError),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),
}
```

### `backend/src/domains/review/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/review/store.rs`
- Size bytes / Размер в байтах: `22617`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashSet;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};

use crate::platform::events::{EventStore, NewEventEnvelope};

use super::errors::ReviewInboxError;
use super::evidence::materialize_review_transition_link_in_transaction;
use super::models::{
    NewReviewItem, NewReviewItemEvidence, ReviewItem, ReviewItemKind, ReviewItemStatus,
    ReviewPromotionTarget, validate_non_empty, validate_review_item_with_evidence,
};

#[derive(Clone)]
pub struct ReviewInboxStore {
    pool: PgPool,
}

impl ReviewInboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_with_evidence(
        &self,
        item: &NewReviewItem,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_review_item_with_evidence(item, evidence)?;

        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::create_with_evidence_in_transaction(&mut transaction, item, evidence).await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_by_status(
        &self,
        status: ReviewItemStatus,
        limit: i64,
    ) -> Result<Vec<ReviewItem>, ReviewInboxError> {
        let sql = review_item_select_sql(
            "WHERE status = $1 ORDER BY updated_at DESC, review_item_id ASC LIMIT $2",
        );
        let rows = sqlx::query(&sql)
            .bind(status.as_str())
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(row_to_review_item).collect()
    }

    pub async fn list_open(&self, limit: i64) -> Result<Vec<ReviewItem>, ReviewInboxError> {
        let sql = review_item_select_sql(
            "WHERE status IN ('new', 'in_review') ORDER BY updated_at DESC, review_item_id ASC LIMIT $1",
        );
        let rows = sqlx::query(&sql)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(row_to_review_item).collect()
    }

    pub async fn list_all(&self, limit: i64) -> Result<Vec<ReviewItem>, ReviewInboxError> {
        let sql = review_item_select_sql("ORDER BY updated_at DESC, review_item_id ASC LIMIT $1");
        let rows = sqlx::query(&sql)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_review_item).collect()
    }

    pub async fn set_status(
        &self,
        review_item_id: &str,
        status: ReviewItemStatus,
    ) -> Result<ReviewItem, ReviewInboxError> {
        self.set_status_with_observation(review_item_id, status, None, None)
            .await
    }

    pub async fn set_status_with_observation(
        &self,
        review_item_id: &str,
        status: ReviewItemStatus,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;

        let mut transaction = self.pool.begin().await?;
        let item = Self::transition_status_in_transaction(&mut transaction, review_item_id, status)
            .await?;
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "review",
            "review_item",
            &item.review_item_id,
            "status",
            item.status.as_str(),
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(item)
    }

    pub async fn promote(
        &self,
        review_item_id: &str,
        target: ReviewPromotionTarget,
    ) -> Result<ReviewItem, ReviewInboxError> {
        self.promote_with_observation(review_item_id, target, None, None)
            .await
    }

    pub async fn promote_with_observation(
        &self,
        review_item_id: &str,
        target: ReviewPromotionTarget,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;
        target.validate()?;

        let mut transaction = self.pool.begin().await?;
        let item = Self::promote_in_transaction(&mut transaction, review_item_id, target).await?;
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "review",
            "review_item",
            &item.review_item_id,
            "status",
            item.status.as_str(),
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(item)
    }

    pub(crate) async fn create_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        item: &NewReviewItem,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_evidence_observations_exist(transaction, evidence).await?;
        let review_item_id = review_item_id(item, evidence)?;
        let inserted = sqlx::query(review_item_insert_sql())
            .bind(&review_item_id)
            .bind(item.item_kind.as_str())
            .bind(item.title.trim())
            .bind(item.summary.trim())
            .bind(item.confidence)
            .bind(&item.metadata)
            .fetch_optional(&mut **transaction)
            .await?;
        let was_inserted = inserted.is_some();

        for item in evidence {
            sqlx::query(
                r#"
                INSERT INTO review_item_evidence (
                    review_item_id,
                    observation_id,
                    evidence_role,
                    metadata
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (review_item_id, observation_id, evidence_role)
                DO UPDATE SET metadata = EXCLUDED.metadata
                "#,
            )
            .bind(&review_item_id)
            .bind(item.observation_id.trim())
            .bind(item.evidence_role.trim())
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        let stored = if let Some(row) = inserted {
            row_to_review_item(row)?
        } else {
            Self::fetch_review_item_in_transaction(transaction, &review_item_id).await?
        };

        if was_inserted {
            append_candidate_detected_event(transaction, &stored, evidence).await?;
            append_review_available_event(transaction, &stored, evidence).await?;
        }

        Ok(stored)
    }

    pub(crate) async fn attach_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        evidence: &[NewReviewItemEvidence],
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;
        validate_evidence_observations_exist(transaction, evidence).await?;

        for item in evidence {
            sqlx::query(
                r#"
                INSERT INTO review_item_evidence (
                    review_item_id,
                    observation_id,
                    evidence_role,
                    metadata
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (review_item_id, observation_id, evidence_role)
                DO UPDATE SET metadata = EXCLUDED.metadata
                "#,
            )
            .bind(review_item_id)
            .bind(item.observation_id.trim())
            .bind(item.evidence_role.trim())
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        Self::fetch_review_item_in_transaction(transaction, review_item_id).await
    }

    pub(crate) async fn transition_status_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        status: ReviewItemStatus,
    ) -> Result<ReviewItem, ReviewInboxError> {
        let item = Self::set_status_in_transaction(transaction, review_item_id, status).await?;
        append_review_status_event(transaction, &item).await?;
        Ok(item)
    }

    pub(crate) async fn promote_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        target: ReviewPromotionTarget,
    ) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;
        target.validate()?;

        let sql = review_item_update_returning_sql(
            r#"
            SET
                status = 'promoted',
                target_domain = $2,
                target_entity_kind = $3,
                target_entity_id = $4,
                updated_at = now()
            WHERE review_item_id = $1
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(review_item_id)
            .bind(target.target_domain.trim())
            .bind(target.target_entity_kind.trim())
            .bind(target.target_entity_id.trim())
            .fetch_optional(&mut **transaction)
            .await?;
        let item = row
            .map(row_to_review_item)
            .transpose()?
            .ok_or_else(|| ReviewInboxError::ReviewItemNotFound(review_item_id.to_owned()))?;
        append_review_status_event(transaction, &item).await?;
        Ok(item)
    }

    pub(crate) async fn find_latest_by_kind_and_metadata_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        item_kind: ReviewItemKind,
        metadata_filter: &Value,
    ) -> Result<Option<ReviewItem>, ReviewInboxError> {
        if !metadata_filter.is_object() {
            return Err(ReviewInboxError::InvalidMetadataFilter);
        }

        let sql = review_item_select_sql(
            "WHERE item_kind = $1 AND metadata @> $2::jsonb ORDER BY updated_at DESC, review_item_id ASC LIMIT 1",
        );
        let row = sqlx::query(&sql)
            .bind(item_kind.as_str())
            .bind(metadata_filter)
            .fetch_optional(&mut **transaction)
            .await?;
        row.map(row_to_review_item).transpose()
    }

    async fn set_status_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
        status: ReviewItemStatus,
    ) -> Result<ReviewItem, ReviewInboxError> {
        let sql = review_item_update_returning_sql(
            r#"
            SET
                status = $2,
                updated_at = now()
            WHERE review_item_id = $1
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(review_item_id)
            .bind(status.as_str())
            .fetch_optional(&mut **transaction)
            .await?;

        row.map(row_to_review_item)
            .transpose()?
            .ok_or_else(|| ReviewInboxError::ReviewItemNotFound(review_item_id.to_owned()))
    }

    pub async fn get(&self, review_item_id: &str) -> Result<ReviewItem, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;

        let mut transaction = self.pool.begin().await?;
        let item = Self::fetch_review_item_in_transaction(&mut transaction, review_item_id).await?;
        transaction.commit().await?;
        Ok(item)
    }

    pub async fn list_evidence(
        &self,
        review_item_id: &str,
    ) -> Result<Vec<ReviewItemEvidenceRecord>, ReviewInboxError> {
        validate_non_empty("review_item_id", review_item_id)?;

        let mut transaction = self.pool.begin().await?;
        let evidence = load_review_evidence(&mut transaction, review_item_id).await?;
        transaction.commit().await?;
        Ok(evidence)
    }

    async fn fetch_review_item_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        review_item_id: &str,
    ) -> Result<ReviewItem, ReviewInboxError> {
        let sql = review
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/settings/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/settings/mod.rs`
- Size bytes / Размер в байтах: `1`
- Included characters / Включено символов: `1`
- Truncated / Обрезано: `no`

```rust

```

### `backend/src/domains/signal_hub/ai.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/ai.rs`
- Size bytes / Размер в байтах: `2464`
- Included characters / Включено символов: `2464`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::service::signal_hub_raw_dispatcher_allows_processing;
use super::{SignalHubError, SignalHubSignalService, SignalHubStore, SignalProcessingOutcome};
use crate::platform::events::{EventEnvelope, EventStore, NewEventEnvelope};

pub async fn dispatch_ai_helper_signal(
    pool: PgPool,
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_ai_helper_signal(
        event_kind,
        source_id,
        subject,
        payload,
        provenance,
        correlation_id,
    )?;
    event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = event_store
        .get_by_id(&raw_signal.event_id)
        .await?
        .ok_or_else(|| SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()))?;

    let signal_store = SignalHubStore::new(pool);
    if !signal_hub_raw_dispatcher_allows_processing(&signal_store).await? {
        return Ok(None);
    }

    let service = SignalHubSignalService::new(signal_store, event_store.clone());
    match service.process_raw_signal(&raw_event).await? {
        SignalProcessingOutcome::Accepted { event_id } => {
            Ok(event_store.get_by_id(&event_id).await?)
        }
        SignalProcessingOutcome::Rejected { .. }
        | SignalProcessingOutcome::Muted { .. }
        | SignalProcessingOutcome::Paused { .. } => Ok(None),
    }
}

fn build_ai_helper_signal(
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Result<NewEventEnvelope, SignalHubError> {
    let builder = NewEventEnvelope::builder(
        format!("evt_signal_raw_ai_{}", Uuid::now_v7()),
        format!("signal.raw.ai.{event_kind}.observed"),
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "ai",
            "source_id": source_id,
        }),
        subject,
    )
    .payload(payload)
    .provenance(provenance);

    let builder = match correlation_id {
        Some(value) if !value.trim().is_empty() => builder.correlation_id(value.to_owned()),
        _ => builder,
    };

    Ok(builder.build()?)
}
```

### `backend/src/domains/signal_hub/capabilities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/capabilities.rs`
- Size bytes / Размер в байтах: `8704`
- Included characters / Включено символов: `8704`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;

use super::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use super::store::{SignalCapability, SignalCapabilityUpsert, SignalHubError, SignalHubStore};

#[derive(Clone)]
pub struct SignalHubCapabilityService {
    store: SignalHubStore,
}

impl SignalHubCapabilityService {
    pub fn new(store: SignalHubStore) -> Self {
        Self { store }
    }

    pub async fn list_capabilities(
        &self,
        source_code: Option<&str>,
        connection_id: Option<&str>,
    ) -> Result<Vec<SignalCapability>, SignalHubError> {
        if let Some(source_code) = source_code {
            self.refresh_source_capabilities(source_code).await?;
        } else {
            for source in self.store.list_sources().await? {
                self.refresh_source_capabilities(&source.code).await?;
            }
        }

        self.store
            .list_capabilities(source_code, connection_id)
            .await
    }

    async fn refresh_source_capabilities(&self, source_code: &str) -> Result<(), SignalHubError> {
        let source = self.store.get_source(source_code).await?;
        let policies = self.store.list_active_policies().await?;
        let control_state = source_capability_control_state(&source.code, &policies);
        let mut capabilities = vec![SignalCapabilityUpsert {
            source_code: source.code.clone(),
            connection_id: None,
            capability: "signals.observe".to_owned(),
            state: control_state.state.to_owned(),
            reason: Some(control_state_reason(
                control_state,
                "source is registered in Signal Hub",
            )),
            requires_confirmation: false,
            action_class: "read".to_owned(),
        }];

        if source.supports_connections {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "connections.manage".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source supports operator-managed connection records",
                )),
                requires_confirmation: false,
                action_class: "local_write".to_owned(),
            });
        }

        if source.supports_runtime {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "runtime.health_check".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source runtime can report durable health state",
                )),
                requires_confirmation: false,
                action_class: "read".to_owned(),
            });
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "runtime.pause".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source runtime can be paused or resumed by Signal Hub",
                )),
                requires_confirmation: false,
                action_class: "local_write".to_owned(),
            });
        }

        if source.supports_mute {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "runtime.mute".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source signal publication can be muted without stopping runtime",
                )),
                requires_confirmation: false,
                action_class: "local_write".to_owned(),
            });
        }

        if source.supports_replay {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "runtime.replay".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source events can be replayed from durable Signal Hub history",
                )),
                requires_confirmation: false,
                action_class: "local_write".to_owned(),
            });
        }

        let source_specific = match source.code.as_str() {
            "browser" => Some(("browser.capture", "browser capture source is registered")),
            "filesystem" => Some((
                "files.observe",
                "filesystem observation source is registered",
            )),
            "voice" => Some(("voice.transcribe", "voice capture source is registered")),
            "fixture" => Some((
                "fixture.emit",
                "fixture source can emit deterministic test signals",
            )),
            "ai" => Some(("ai.enrich", "local AI signal source is registered")),
            _ => None,
        };

        if let Some((capability, reason)) = source_specific {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: capability.to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(control_state, reason)),
                requires_confirmation: false,
                action_class: "read".to_owned(),
            });
        }

        self.store
            .replace_source_capabilities(&source.code, None, &capabilities)
            .await
    }
}

#[derive(Clone, Copy)]
struct CapabilityControlState<'a> {
    state: &'a str,
    status_label: Option<&'a str>,
    reason: Option<&'a str>,
}

fn source_capability_control_state<'a>(
    source_code: &str,
    policies: &'a [SignalPolicy],
) -> CapabilityControlState<'a> {
    let now = Utc::now();
    let matching = policies
        .iter()
        .filter(|policy| {
            if policy
                .expires_at
                .is_some_and(|expires_at| expires_at <= now)
            {
                return false;
            }

            if policy.connection_id.is_some() || policy.event_pattern.is_some() {
                return false;
            }

            match policy.scope {
                SignalPolicyScope::Global => true,
                SignalPolicyScope::Source | SignalPolicyScope::Profile => policy
                    .source_code
                    .as_deref()
                    .is_some_and(|policy_source| policy_source == source_code),
                SignalPolicyScope::Connection | SignalPolicyScope::EventPattern => false,
            }
        })
        .collect::<Vec<_>>();

    if let Some(policy) = matching
        .iter()
        .copied()
        .find(|policy| matches!(policy.mode, SignalPolicyMode::Disabled))
    {
        return CapabilityControlState {
            state: "blocked",
            status_label: Some("disabled"),
            reason: Some(policy.reason.as_str()),
        };
    }

    if let Some(policy) = matching
        .iter()
        .copied()
        .find(|policy| matches!(policy.mode, SignalPolicyMode::Paused))
    {
        return CapabilityControlState {
            state: "degraded",
            status_label: Some("paused"),
            reason: Some(policy.reason.as_str()),
        };
    }

    if let Some(policy) = matching
        .iter()
        .copied()
        .find(|policy| matches!(policy.mode, SignalPolicyMode::Muted))
    {
        return CapabilityControlState {
            state: "degraded",
            status_label: Some("muted"),
            reason: Some(policy.reason.as_str()),
        };
    }

    CapabilityControlState {
        state: "available",
        status_label: None,
        reason: None,
    }
}

fn control_state_reason(control_state: CapabilityControlState<'_>, base_reason: &str) -> String {
    match (control_state.status_label, control_state.reason) {
        (Some(status), Some(reason)) => {
            format!("{base_reason}; source is currently {status} by policy: {reason}")
        }
        (Some(status), None) => format!("{base_reason}; source is currently {status} by policy"),
        (None, _) => base_reason.to_owned(),
    }
}
```

### `backend/src/domains/signal_hub/connections.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/connections.rs`
- Size bytes / Размер в байтах: `7069`
- Included characters / Включено символов: `7069`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::Value;
use serde_json::json;
use uuid::Uuid;

use super::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use super::store::{
    SignalConnection, SignalConnectionCreate, SignalConnectionUpdate, SignalHubError,
    SignalHubStore,
};
use crate::platform::events::{EventStore, NewEventEnvelope};

#[derive(Clone)]
pub struct SignalHubConnectionService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalHubConnectionService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn create_connection(
        &self,
        request: &SignalConnectionCreate,
    ) -> Result<SignalConnection, SignalHubError> {
        let connection = self.signal_store.create_connection(request).await?;
        let sync = self.reconcile_operator_status(&connection).await?;
        self.append_connection_event(
            "signal.connection.created",
            &connection,
            sync.cleared_count,
            sync.applied_mode,
        )
        .await?;
        Ok(connection)
    }

    pub async fn update_connection(
        &self,
        request: &SignalConnectionUpdate,
    ) -> Result<SignalConnection, SignalHubError> {
        let connection = self.signal_store.update_connection(request).await?;
        let sync = self.reconcile_operator_status(&connection).await?;
        self.append_connection_event(
            "signal.connection.updated",
            &connection,
            sync.cleared_count,
            sync.applied_mode,
        )
        .await?;
        Ok(connection)
    }

    pub async fn remove_connection(
        &self,
        connection_id: &str,
    ) -> Result<SignalConnection, SignalHubError> {
        let connection = self.signal_store.remove_connection(connection_id).await?;
        let sync = self.reconcile_operator_status(&connection).await?;
        self.append_connection_event(
            "signal.connection.removed",
            &connection,
            sync.cleared_count,
            sync.applied_mode,
        )
        .await?;
        Ok(connection)
    }

    pub async fn upsert_account_connection(
        &self,
        source_code: &str,
        account_id: &str,
        display_name: &str,
        status: &str,
        settings: Value,
        secret_ref: Option<String>,
    ) -> Result<SignalConnection, SignalHubError> {
        if let Some(existing) = self
            .signal_store
            .find_connection_by_account(source_code, account_id)
            .await?
        {
            return self
                .update_connection(&SignalConnectionUpdate {
                    id: existing.id,
                    display_name: Some(display_name.to_owned()),
                    status: Some(status.to_owned()),
                    profile: None,
                    settings: Some(settings),
                    secret_ref,
                })
                .await;
        }

        self.create_connection(&SignalConnectionCreate {
            source_code: source_code.to_owned(),
            display_name: display_name.to_owned(),
            status: status.to_owned(),
            profile: None,
            settings,
            secret_ref,
        })
        .await
    }

    pub async fn remove_account_connection(
        &self,
        source_code: &str,
        account_id: &str,
    ) -> Result<Option<SignalConnection>, SignalHubError> {
        let Some(existing) = self
            .signal_store
            .find_connection_by_account(source_code, account_id)
            .await?
        else {
            return Ok(None);
        };

        self.remove_connection(&existing.id).await.map(Some)
    }

    async fn reconcile_operator_status(
        &self,
        connection: &SignalConnection,
    ) -> Result<ConnectionPolicySync, SignalHubError> {
        let selector = SignalPolicy {
            scope: SignalPolicyScope::Connection,
            source_code: Some(connection.source_code.clone()),
            connection_id: Some(connection.id.clone()),
            event_pattern: None,
            mode: SignalPolicyMode::Enabled,
            reason: format!("connection status {}", connection.status),
            expires_at: None,
        };
        let cleared_count = self
            .signal_store
            .expire_matching_policies(
                &selector,
                &[
                    SignalPolicyMode::Disabled,
                    SignalPolicyMode::Paused,
                    SignalPolicyMode::Muted,
                ],
            )
            .await?;
        let applied_mode = connection_operator_mode(connection.status.as_str());
        if let Some(mode) = applied_mode.clone() {
            self.signal_store
                .create_policy(&SignalPolicy { mode, ..selector })
                .await?;
        }

        Ok(ConnectionPolicySync {
            cleared_count,
            applied_mode,
        })
    }

    async fn append_connection_event(
        &self,
        event_type: &str,
        connection: &SignalConnection,
        cleared_count: u64,
        applied_mode: Option<SignalPolicyMode>,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!(
                "evt_{}_{}_{}",
                event_type.replace('.', "_"),
                connection.id,
                Uuid::now_v7()
            ),
            event_type,
            Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": connection.source_code,
                "source_id": connection.id,
            }),
            json!({
                "kind": "signal_connection",
                "entity_id": connection.id,
                "source_code": connection.source_code,
                "connection_id": connection.id,
            }),
        )
        .payload(json!({
            "display_name": connection.display_name,
            "status": connection.status,
            "profile": connection.profile,
            "cleared_operator_policy_count": cleared_count,
            "applied_operator_policy_mode": applied_mode.as_ref().map(SignalPolicyMode::as_str),
        }))
        .provenance(json!({
            "source": "signal_hub_connection_service",
            "source_code": connection.source_code,
            "connection_id": connection.id,
        }))
        .correlation_id(&connection.id)
        .build()?;
        self.event_store
            .append_for_dispatch_idempotent(&event)
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
struct ConnectionPolicySync {
    cleared_count: u64,
    applied_mode: Option<SignalPolicyMode>,
}

fn connection_operator_mode(status: &str) -> Option<SignalPolicyMode> {
    match status.trim() {
        "disabled" => Some(SignalPolicyMode::Disabled),
        "paused" => Some(SignalPolicyMode::Paused),
        "muted" => Some(SignalPolicyMode::Muted),
        _ => None,
    }
}
```

### `backend/src/domains/signal_hub/controls.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/controls.rs`
- Size bytes / Размер в байтах: `14276`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use super::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use super::store::{SignalConnection, SignalHubError, SignalHubStore, SignalSource};
use crate::platform::events::{EventStore, NewEventEnvelope};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHubControlRequest {
    pub scope: SignalPolicyScope,
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHubControlResult {
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub policy_id: Option<String>,
    pub cleared_count: u64,
}

#[derive(Clone)]
pub struct SignalHubControlService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalHubControlService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn disable_source(
        &self,
        source_code: &str,
        reason: Option<&str>,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        let source = self.signal_store.get_source(source_code).await?;
        let policy = SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some(source.code.clone()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Disabled,
            reason: normalize_reason(reason, "source disabled"),
            expires_at: None,
        };
        let policy_id = self.signal_store.create_policy(&policy).await?;
        self.reconcile_source_runtime_state(&source.code).await?;
        self.append_control_events("signal.source.disabled", &policy, Some(&source), None, 0)
            .await?;

        Ok(SignalHubControlResult {
            source_code: policy.source_code,
            connection_id: None,
            event_pattern: None,
            policy_id: Some(policy_id.to_string()),
            cleared_count: 0,
        })
    }

    pub async fn enable_source(
        &self,
        source_code: &str,
        reason: Option<&str>,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        let source = self.signal_store.get_source(source_code).await?;
        let policy = SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some(source.code.clone()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Disabled,
            reason: normalize_reason(reason, "source enabled"),
            expires_at: None,
        };
        let cleared_count = self
            .signal_store
            .expire_matching_policies(&policy, &[SignalPolicyMode::Disabled])
            .await?;
        self.reconcile_source_runtime_state(&source.code).await?;
        self.append_control_events(
            "signal.source.enabled",
            &policy,
            Some(&source),
            None,
            cleared_count,
        )
        .await?;

        Ok(SignalHubControlResult {
            source_code: policy.source_code,
            connection_id: None,
            event_pattern: None,
            policy_id: None,
            cleared_count,
        })
    }

    pub async fn mute_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.create_scoped_policy(request, SignalPolicyMode::Muted, "signal.source.muted")
            .await
    }

    pub async fn disable_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.create_scoped_policy(
            request,
            SignalPolicyMode::Disabled,
            "signal.signals.disabled",
        )
        .await
    }

    pub async fn enable_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.clear_scoped_policy(
            request,
            SignalPolicyMode::Disabled,
            "signal.signals.enabled",
        )
        .await
    }

    pub async fn unmute_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.clear_scoped_policy(request, SignalPolicyMode::Muted, "signal.source.unmuted")
            .await
    }

    pub async fn pause_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.create_scoped_policy(request, SignalPolicyMode::Paused, "signal.source.paused")
            .await
    }

    pub async fn resume_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.clear_scoped_policy(request, SignalPolicyMode::Paused, "signal.source.resumed")
            .await
    }

    async fn create_scoped_policy(
        &self,
        request: &SignalHubControlRequest,
        mode: SignalPolicyMode,
        event_type: &str,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        let resolved = self.resolve_request(request, Some(mode.clone())).await?;
        let policy_id = self.signal_store.create_policy(&resolved.policy).await?;
        if matches!(resolved.policy.scope, SignalPolicyScope::Source)
            && let Some(source_code) = resolved.policy.source_code.as_deref()
        {
            self.reconcile_source_runtime_state(source_code).await?;
        }
        self.append_control_events(
            event_type,
            &resolved.policy,
            resolved.source.as_ref(),
            resolved.connection.as_ref(),
            0,
        )
        .await?;

        Ok(SignalHubControlResult {
            source_code: resolved.policy.source_code,
            connection_id: resolved.policy.connection_id,
            event_pattern: resolved.policy.event_pattern,
            policy_id: Some(policy_id.to_string()),
            cleared_count: 0,
        })
    }

    async fn clear_scoped_policy(
        &self,
        request: &SignalHubControlRequest,
        mode: SignalPolicyMode,
        event_type: &str,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        let resolved = self.resolve_request(request, Some(mode.clone())).await?;
        let cleared_count = self
            .signal_store
            .expire_matching_policies(&resolved.policy, &[mode])
            .await?;
        if matches!(resolved.policy.scope, SignalPolicyScope::Source)
            && let Some(source_code) = resolved.policy.source_code.as_deref()
        {
            self.reconcile_source_runtime_state(source_code).await?;
        }
        self.append_control_events(
            event_type,
            &resolved.policy,
            resolved.source.as_ref(),
            resolved.connection.as_ref(),
            cleared_count,
        )
        .await?;

        Ok(SignalHubControlResult {
            source_code: resolved.policy.source_code,
            connection_id: resolved.policy.connection_id,
            event_pattern: resolved.policy.event_pattern,
            policy_id: None,
            cleared_count,
        })
    }

    async fn reconcile_source_runtime_state(
        &self,
        source_code: &str,
    ) -> Result<(), SignalHubError> {
        let state = crate::platform::events::source_runtime_state_from_policies(
            self.signal_store.pool(),
            source_code,
        )
        .await?;
        self.signal_store
            .set_source_runtime_state(source_code, state)
            .await?;
        Ok(())
    }

    async fn resolve_request(
        &self,
        request: &SignalHubControlRequest,
        mode: Option<SignalPolicyMode>,
    ) -> Result<ResolvedControlRequest, SignalHubError> {
        let reason = normalize_reason(Some(&request.reason), "owner control");
        let source_code = request
            .source_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let connection_id = request
            .connection_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let event_pattern = request
            .event_pattern
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        let source = match request.scope {
            SignalPolicyScope::Source => Some(
                self.signal_store
                    .get_source(
                        source_code
                            .as_deref()
                            .ok_or(SignalHubError::EmptyField("source_code"))?,
                    )
                    .await?,
            ),
            _ => match source_code.as_deref() {
                Some(code) => Some(self.signal_store.get_source(code).await?),
                None => None,
            },
        };

        let connection = match request.scope {
            SignalPolicyScope::Connection => Some(
                self.signal_store
                    .get_connection(
                        connection_id
                            .as_deref()
                            .ok_or(SignalHubError::EmptyField("connection_id"))?,
                    )
                    .await?,
            ),
            _ => match connection_id.as_deref() {
                Some(id) => Some(self.signal_store.get_connection(id).await?),
                None => None,
            },
        };

        if matches!(request.scope, SignalPolicyScope::EventPattern) && event_pattern.is_none() {
            return Err(SignalHubError::EmptyField("event_pattern"));
        }

        if matches!(request.scope, SignalPolicyScope::Profile) {
            return Err(SignalHubError::InvalidPolicyScope("profile".to_owned()));
        }

        let normalized_source_code = match (&source, &connection) {
            (Some(source), _) => Some(source.code.clone()),
            (None, Some(connection)) => Some(connection.source_code.clone()),
            (None, None) => None,
        };
        if let (Some(source), Some(connection)) = (&source, &connection)
            && connection.source_code != source.code
        {
            return Err(SignalHubError::InvalidConnectionId(connection.id.clone()));
        }

        Ok(ResolvedControlRequest {
            policy: SignalPolicy {
                scope: request.scope.clone(),
                source_code: normalized_source_code,
                connection_id,
                event_pattern,
                mode: mode.unwrap_or(SignalPolicyMode::Enabled),
                reason,
                expires_at: None,
            },
            source,
            connection,
        })
    }

    async fn append_control_events(
        &self,
        control_event_type: &str,
        policy: &SignalPolicy,
        source: Option<&SignalSource>,
        connection: Option<&SignalConnection>,
        cleared_count: u64,
    ) -> Result<(), SignalHubError> {
        let now = Utc::now();
        let control_event_id = Uuid::now_v7();
        let policy_event_id = Uuid::now_v7();
        let control_source_id = format!("signal_hub_control:{control_event_id}");
        let policy_source_id = format!("signal_hub_control:{policy_event_id}");
        let source_code = policy
            .source_code
            .as_deref()
            .or_else(|| connection.map(|item| item.source_code.as_str()))
            .unwrap_or("system");
        let entity_id = connection
            .map(|item| item.id.as_str())
            .or(policy.event_pattern.as_deref())
            .unwrap_or(source_code);
        l
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/signal_hub/fixture_source.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/fixture_source.rs`
- Size bytes / Размер в байтах: `7416`
- Included characters / Включено символов: `7416`
- Truncated / Обрезано: `no`

```rust
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::store::{SignalHubError, SignalHubStore};
use crate::platform::events::{EventStore, NewEventEnvelope};

const TEST_SIGNAL_FIXTURES_TOML: &str =
    include_str!("../../../fixtures/signal_hub/test_signals.toml");

static TEST_SIGNAL_FIXTURE_CATALOG: LazyLock<Result<SignalFixtureCatalog, toml::de::Error>> =
    LazyLock::new(|| toml::from_str(TEST_SIGNAL_FIXTURES_TOML));

#[derive(Clone)]
pub struct SignalFixtureSourceService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalFixtureSourceService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn emit_fixture(
        &self,
        request: &SignalFixtureEmitRequest,
    ) -> Result<SignalFixtureEmission, SignalHubError> {
        let fixture_id = request.fixture_id.trim();
        if fixture_id.is_empty() {
            return Err(SignalHubError::EmptyField("fixture_id"));
        }

        let fixture = fixture_signal_by_id(fixture_id)?;
        self.ensure_source_exists(&fixture.source_code).await?;

        let raw_signal = build_fixture_raw_signal(fixture)?;
        self.event_store
            .append_for_dispatch_idempotent(&raw_signal)
            .await?;

        Ok(SignalFixtureEmission {
            fixture_id: fixture.fixture_id.to_owned(),
            raw_event_id: raw_signal.event_id,
            event_type: fixture.event_type.to_owned(),
            source_code: fixture.source_code.to_owned(),
            correlation_id: fixture.correlation_id.as_deref().map(ToOwned::to_owned),
        })
    }

    pub fn list_fixture_sources(&self) -> Result<Vec<SignalFixtureSource>, SignalHubError> {
        let catalog = fixture_catalog()?;
        Ok(catalog
            .fixtures
            .iter()
            .map(|fixture| SignalFixtureSource {
                fixture_id: fixture.fixture_id.clone(),
                source_code: fixture.source_code.clone(),
                event_type: fixture.event_type.clone(),
                correlation_id: fixture.correlation_id.clone(),
                occurred_at: fixture.occurred_at,
                summary: fixture_summary(fixture),
            })
            .collect())
    }

    async fn ensure_source_exists(&self, source_code: &str) -> Result<(), SignalHubError> {
        let exists = self
            .signal_store
            .list_sources()
            .await?
            .into_iter()
            .any(|source| source.code == source_code);
        if exists {
            Ok(())
        } else {
            Err(SignalHubError::SourceNotFound(source_code.to_owned()))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalFixtureEmitRequest {
    pub fixture_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalFixtureEmission {
    pub fixture_id: String,
    pub raw_event_id: String,
    pub event_type: String,
    pub source_code: String,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalFixtureSource {
    pub fixture_id: String,
    pub source_code: String,
    pub event_type: String,
    pub correlation_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
struct SignalFixtureCatalog {
    schema_version: u32,
    fixtures: Vec<SignalFixtureDefinition>,
}

#[derive(Clone, Debug, Deserialize)]
struct SignalFixtureDefinition {
    fixture_id: String,
    #[serde(rename = "source")]
    source_code: String,
    event_type: String,
    source_id: String,
    subject_kind: String,
    subject_entity_id: String,
    occurred_at: DateTime<Utc>,
    correlation_id: Option<String>,
    payload: Value,
    #[serde(default = "empty_object")]
    provenance: Value,
}

fn fixture_signal_by_id(
    fixture_id: &str,
) -> Result<&'static SignalFixtureDefinition, SignalHubError> {
    let catalog = fixture_catalog()?;

    if catalog.schema_version != 1 {
        return Err(SignalHubError::InvalidFixtureCatalog(format!(
            "unsupported test signal fixture schema version: {}",
            catalog.schema_version
        )));
    }

    catalog
        .fixtures
        .iter()
        .find(|fixture| fixture.fixture_id == fixture_id)
        .ok_or_else(|| SignalHubError::FixtureNotFound(fixture_id.to_owned()))
}

fn fixture_catalog() -> Result<&'static SignalFixtureCatalog, SignalHubError> {
    TEST_SIGNAL_FIXTURE_CATALOG.as_ref().map_err(|error| {
        SignalHubError::InvalidFixtureCatalog(format!(
            "failed to parse test signal fixtures: {error}"
        ))
    })
}

fn build_fixture_raw_signal(
    fixture: &SignalFixtureDefinition,
) -> Result<NewEventEnvelope, SignalHubError> {
    let event = NewEventEnvelope::builder(
        fixture_raw_event_id(fixture),
        &fixture.event_type,
        fixture.occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": fixture.source_code,
            "source_id": fixture.source_id,
            "runtime_kind": "fixture",
            "fixture_id": fixture.fixture_id,
        }),
        json!({
            "kind": fixture.subject_kind,
            "source_code": fixture.source_code,
            "entity_id": fixture.subject_entity_id,
            "fixture_id": fixture.fixture_id,
        }),
    )
    .payload(fixture.payload.clone())
    .provenance(build_fixture_provenance(fixture)?);

    let event = match fixture.correlation_id.as_deref() {
        Some(correlation_id) if !correlation_id.trim().is_empty() => {
            event.correlation_id(correlation_id.trim().to_owned())
        }
        _ => event,
    };

    Ok(event.build()?)
}

fn build_fixture_provenance(fixture: &SignalFixtureDefinition) -> Result<Value, SignalHubError> {
    let mut provenance = match fixture.provenance.clone() {
        Value::Object(map) => map,
        other => {
            return Err(SignalHubError::InvalidFixtureCatalog(format!(
                "fixture provenance must be an object for {} but was {}",
                fixture.fixture_id, other
            )));
        }
    };
    provenance.insert("source".to_owned(), json!("signal_fixture_catalog"));
    provenance.insert("fixture_id".to_owned(), json!(fixture.fixture_id));
    provenance.insert("runtime_kind".to_owned(), json!("fixture"));
    provenance.insert("source_code".to_owned(), json!(fixture.source_code));

    Ok(Value::Object(provenance))
}

fn fixture_raw_event_id(fixture: &SignalFixtureDefinition) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fixture.fixture_id.as_bytes());
    hasher.update([0]);
    hasher.update(fixture.source_code.as_bytes());
    hasher.update([0]);
    hasher.update(fixture.event_type.as_bytes());
    format!("evt_signal_fixture_{:x}", hasher.finalize())
}

fn empty_object() -> Value {
    json!({})
}

fn fixture_summary(fixture: &SignalFixtureDefinition) -> String {
    fixture
        .payload
        .get("summary")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| fixture.event_type.clone())
}
```

### `backend/src/domains/signal_hub/fixtures.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/fixtures.rs`
- Size bytes / Размер в байтах: `9519`
- Included characters / Включено символов: `9519`
- Truncated / Обрезано: `no`

```rust
use std::sync::OnceLock;

use serde::Deserialize;

use super::policies::{SignalPolicyMode, SignalPolicyScope};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemSourceFixture {
    pub code: &'static str,
    pub display_name: &'static str,
    pub category: &'static str,
    pub source_kind: &'static str,
    pub default_enabled: bool,
    pub supports_connections: bool,
    pub supports_runtime: bool,
    pub supports_replay: bool,
    pub supports_pause: bool,
    pub supports_mute: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemProfilePolicyFixture {
    pub scope: SignalPolicyScope,
    pub source_code: Option<&'static str>,
    pub event_pattern: Option<&'static str>,
    pub mode: SignalPolicyMode,
    pub reason: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemProfileFixture {
    pub code: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub is_system: bool,
    pub policies: &'static [SystemProfilePolicyFixture],
}

pub fn system_source_fixtures() -> &'static [SystemSourceFixture] {
    SYSTEM_SOURCE_FIXTURES
        .get_or_init(load_system_source_fixtures)
        .as_slice()
}

pub fn system_profile_fixtures() -> &'static [SystemProfileFixture] {
    &SYSTEM_PROFILE_FIXTURES
}

static SYSTEM_SOURCE_FIXTURES: OnceLock<Vec<SystemSourceFixture>> = OnceLock::new();

fn load_system_source_fixtures() -> Vec<SystemSourceFixture> {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/signal_hub/system_sources.toml"
    ));
    let catalog: RawSystemSourceCatalog =
        toml::from_str(raw).expect("signal_hub system_sources.toml must parse");

    catalog
        .sources
        .into_iter()
        .map(|source| SystemSourceFixture {
            code: leak_string(source.code),
            display_name: leak_string(source.display_name),
            category: leak_string(source.category),
            source_kind: leak_string(source.source_kind),
            default_enabled: source.default_enabled,
            supports_connections: source.supports_connections,
            supports_runtime: source.supports_runtime,
            supports_replay: source.supports_replay,
            supports_pause: source.supports_pause,
            supports_mute: source.supports_mute,
        })
        .collect()
}

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

#[derive(Debug, Deserialize)]
struct RawSystemSourceCatalog {
    sources: Vec<RawSystemSourceFixture>,
}

#[derive(Debug, Deserialize)]
struct RawSystemSourceFixture {
    code: String,
    display_name: String,
    category: String,
    source_kind: String,
    default_enabled: bool,
    supports_connections: bool,
    supports_runtime: bool,
    supports_replay: bool,
    supports_pause: bool,
    supports_mute: bool,
}

const DEVELOPMENT_PROFILE_POLICIES: [SystemProfilePolicyFixture; 2] = [
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("rss"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "development profile mutes noisy RSS capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("browser"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "development profile mutes browser capture by default",
    },
];

const TESTING_PROFILE_POLICIES: [SystemProfilePolicyFixture; 12] = [
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("ai"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes AI runtime signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("browser"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes browser capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("calendar"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes calendar provider signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("filesystem"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes filesystem capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("github"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes GitHub provider signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("home_assistant"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes Home Assistant signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("mail"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes mail provider signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("rss"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes RSS signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("telegram"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes Telegram signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("voice"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes voice capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("whatsapp"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes WhatsApp signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("zoom"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes Zoom signals",
    },
];

const MAINTENANCE_PROFILE_POLICIES: [SystemProfilePolicyFixture; 4] = [
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("mail"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses mail capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("telegram"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses Telegram capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("whatsapp"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses WhatsApp capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("zoom"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses Zoom capture",
    },
];

const PRODUCTION_PROFILE_POLICIES: [SystemProfilePolicyFixture; 0] = [];

const SYSTEM_PROFILE_FIXTURES: [SystemProfileFixture; 4] = [
    SystemProfileFixture {
        code: "production",
        display_name: "Production",
        description: "All configured real sources run according to owner settings.",
        is_system: true,
        policies: &PRODUCTION_PROFILE_POLICIES,
    },
    SystemProfileFixture {
        code: "development",
        display_name: "Development",
        description: "Selected noisy sources stay muted during local development.",
        is_system: true,
        policies: &DEVELOPMENT_PROFILE_POLICIES,
    },
    SystemProfileFixture {
        code: "testing",
        display_name: "Testing",
        description: "Real sources are muted while deterministic fixture signals stay available.",
        is_system: true,
        policies: &TESTING_PROFILE_POLICIES,
    },
    SystemProfileFixture {
        code: "maintenance",
        display_name: "Maintenance",
        description: "Capture pauses while replay and recovery operations remain available.",
        is_system: true,
        policies: &MAINTENANCE_PROFILE_POLICIES,
    },
];

#[cfg(test)]
mod tests {
    use super::system_source_fixtures;

    #[test]
    fn system_source_fixtures_are_loaded_from_canonical_catalog() {
        let codes: Vec<_> = system_source_fixtures()
            .iter()
            .map(|fixture| fixture.code)
            .collect();

        assert_eq!(
            codes,
            vec![
                "system",
                "ai",
                "mail",
                "telegram",
                "whatsapp",
                "zoom",
                "github",
                "browser",
                "rss",
                "calendar",
                "filesystem",
                "home_assistant",
                "voice",
                "fixture",
            ]
        );
    }
}
```

### `backend/src/domains/signal_hub/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/health.rs`
- Size bytes / Размер в байтах: `2704`
- Included characters / Включено символов: `2704`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;

use super::store::{
    SignalHealth, SignalHealthCheckRequest, SignalHealthSnapshotWrite, SignalHubError,
    SignalHubStore,
};
use crate::platform::events::{EventStore, NewEventEnvelope};

#[derive(Clone)]
pub struct SignalHubHealthService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalHubHealthService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn run_health_check(
        &self,
        request: &SignalHealthCheckRequest,
    ) -> Result<SignalHealth, SignalHubError> {
        let health = self.signal_store.run_health_check(request).await?;
        self.append_health_changed_event(&health).await?;
        Ok(health)
    }

    pub async fn record_snapshot(
        &self,
        request: &SignalHealthCheckRequest,
        snapshot: SignalHealthSnapshotWrite,
    ) -> Result<SignalHealth, SignalHubError> {
        let health = self
            .signal_store
            .upsert_health_snapshot(request, snapshot)
            .await?;
        self.append_health_changed_event(&health).await?;
        Ok(health)
    }

    async fn append_health_changed_event(
        &self,
        health: &SignalHealth,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!(
                "evt_signal_health_changed_{}_{}",
                health.id,
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
            "signal.source.health_changed",
            Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": health.source_code,
                "source_id": health.connection_id.clone().unwrap_or_else(|| health.source_code.clone()),
            }),
            json!({
                "kind": "signal_health",
                "entity_id": health.id,
                "source_code": health.source_code,
                "connection_id": health.connection_id,
            }),
        )
        .payload(json!({
            "level": health.level,
            "summary": health.summary,
            "failure_count": health.failure_count,
            "consecutive_failure_count": health.consecutive_failure_count,
            "next_retry_at": health.next_retry_at,
        }))
        .provenance(json!({
            "source": "signal_hub_health_service",
            "source_code": health.source_code,
            "connection_id": health.connection_id,
        }))
        .build()?;
        self.event_store.append_for_dispatch(&event).await?;
        Ok(())
    }
}
```

### `backend/src/domains/signal_hub/mail.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/mail.rs`
- Size bytes / Размер в байтах: `7121`
- Included characters / Включено символов: `7121`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use std::path::Path;

use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;

use super::service::signal_hub_raw_dispatcher_allows_processing;
use super::{SignalHubError, SignalHubSignalService, SignalHubStore, SignalProcessingOutcome};
use crate::platform::communications::StoredRawCommunicationRecord;
use crate::platform::events::{EventEnvelope, EventStore, NewEventEnvelope};
use crate::platform::observations::observation_captured_event_id;

pub struct MailDeliverySignalRequest<'a> {
    pub occurred_at: DateTime<Utc>,
    pub account_id: &'a str,
    pub provider_message_id: &'a str,
    pub event_kind: &'a str,
    pub payload: serde_json::Value,
    pub source_kind: &'a str,
    pub provider_record_id: Option<&'a str>,
    pub raw_record_id: Option<&'a str>,
    pub correlation_id: Option<&'a str>,
}

pub async fn dispatch_mail_raw_signal(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
    raw_blob_root: Option<&Path>,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_mail_raw_signal(raw_record, raw_blob_root)?;
    dispatch_mail_signal(pool, event_store, raw_signal).await
}

pub async fn dispatch_mail_delivery_event_signal(
    pool: PgPool,
    request: MailDeliverySignalRequest<'_>,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_mail_delivery_event_signal(&request)?;
    dispatch_mail_signal(pool, event_store, raw_signal).await
}

async fn dispatch_mail_signal(
    pool: PgPool,
    event_store: EventStore,
    raw_signal: NewEventEnvelope,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = event_store
        .get_by_id(&raw_signal.event_id)
        .await?
        .ok_or_else(|| SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()))?;

    let signal_store = SignalHubStore::new(pool);
    if !signal_hub_raw_dispatcher_allows_processing(&signal_store).await? {
        return Ok(None);
    }

    let service = SignalHubSignalService::new(signal_store, event_store.clone());
    match service.process_raw_signal(&raw_event).await? {
        SignalProcessingOutcome::Accepted { event_id } => {
            Ok(event_store.get_by_id(&event_id).await?)
        }
        SignalProcessingOutcome::Rejected { .. }
        | SignalProcessingOutcome::Muted { .. }
        | SignalProcessingOutcome::Paused { .. } => Ok(None),
    }
}

fn build_mail_raw_signal(
    raw_record: &StoredRawCommunicationRecord,
    raw_blob_root: Option<&Path>,
) -> Result<NewEventEnvelope, SignalHubError> {
    let occurred_at = raw_record.occurred_at.unwrap_or(raw_record.captured_at);
    let source = json!({
        "kind": "signal_source",
        "source_code": "mail",
        "source_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
    });
    let subject = json!({
        "kind": "communication_raw_record",
        "source_code": "mail",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
    });
    let mut provenance = json!({
        "source": "communications_raw_record",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
        "import_batch_id": raw_record.import_batch_id,
        "raw_record_provenance": raw_record.provenance,
    });
    if let Some(root) = raw_blob_root.and_then(|value| value.to_str()) {
        provenance["blob_root"] = json!(root);
    }

    Ok(NewEventEnvelope::builder(
        mail_raw_signal_event_id(&raw_record.raw_record_id),
        "signal.raw.mail.message.observed",
        occurred_at,
        source,
        subject,
    )
    .payload(raw_record.payload.clone())
    .provenance(provenance)
    .causation_id(observation_captured_event_id(&raw_record.observation_id))
    .correlation_id(raw_record.observation_id.clone())
    .build()?)
}

fn build_mail_delivery_event_signal(
    request: &MailDeliverySignalRequest<'_>,
) -> Result<NewEventEnvelope, SignalHubError> {
    let builder = NewEventEnvelope::builder(
        mail_delivery_signal_event_id(
            request.account_id,
            request.provider_message_id,
            request.event_kind,
            request.source_kind,
            request.provider_record_id,
            request.raw_record_id,
        ),
        format!("signal.raw.mail.{}.observed", request.event_kind),
        request.occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "mail",
            "account_id": request.account_id,
            "provider_message_id": request.provider_message_id,
            "source_kind": request.source_kind,
            "provider_record_id": request.provider_record_id,
            "raw_record_id": request.raw_record_id,
        }),
        json!({
            "kind": "mail_provider_delivery_event",
            "source_code": "mail",
            "account_id": request.account_id,
            "provider_message_id": request.provider_message_id,
            "event_kind": request.event_kind,
            "source_kind": request.source_kind,
            "provider_record_id": request.provider_record_id,
            "raw_record_id": request.raw_record_id,
        }),
    )
    .payload(request.payload.clone())
    .provenance(json!({
        "source": "mail_provider_delivery_event",
        "source_kind": request.source_kind,
        "account_id": request.account_id,
        "provider_message_id": request.provider_message_id,
        "provider_record_id": request.provider_record_id,
        "raw_record_id": request.raw_record_id,
    }));
    let builder = match request.correlation_id {
        Some(value) if !value.trim().is_empty() => builder.correlation_id(value.to_owned()),
        _ => builder,
    };
    Ok(builder.build()?)
}

fn mail_raw_signal_event_id(raw_record_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_record_id.as_bytes());
    format!("evt_signal_raw_mail_{:x}", hasher.finalize())
}

fn mail_delivery_signal_event_id(
    account_id: &str,
    provider_message_id: &str,
    event_kind: &str,
    source_kind: &str,
    provider_record_id: Option<&str>,
    raw_record_id: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(provider_message_id.as_bytes());
    hasher.update(event_kind.as_bytes());
    hasher.update(source_kind.as_bytes());
    if let Some(value) = provider_record_id {
        hasher.update(value.as_bytes());
    }
    if let Some(value) = raw_record_id {
        hasher.update(value.as_bytes());
    }
    format!("evt_signal_raw_mail_delivery_{:x}", hasher.finalize())
}
```

### `backend/src/domains/signal_hub/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/mod.rs`
- Size bytes / Размер в байтах: `1905`
- Included characters / Включено символов: `1905`
- Truncated / Обрезано: `no`

```rust
mod ai;
mod capabilities;
mod connections;
mod controls;
mod fixture_source;
mod fixtures;
mod health;
mod mail;
mod policies;
mod profiles;
mod service;
mod store;
mod telegram;
mod whatsapp;

pub use self::ai::dispatch_ai_helper_signal;
pub use self::capabilities::SignalHubCapabilityService;
pub use self::connections::SignalHubConnectionService;
pub use self::controls::{
    SignalHubControlRequest, SignalHubControlResult, SignalHubControlService,
};
pub use self::fixture_source::{
    SignalFixtureEmission, SignalFixtureEmitRequest, SignalFixtureSource,
    SignalFixtureSourceService,
};
pub use self::fixtures::{SystemSourceFixture, system_source_fixtures};
pub use self::health::SignalHubHealthService;
pub use self::mail::{
    MailDeliverySignalRequest, dispatch_mail_delivery_event_signal, dispatch_mail_raw_signal,
};
pub use self::policies::{
    SignalPolicy, SignalPolicyDecision, SignalPolicyEvaluator, SignalPolicyMode, SignalPolicyScope,
};
pub use self::profiles::SignalHubProfileService;
pub use self::service::{
    SIGNAL_HUB_RAW_SIGNAL_CONSUMER, SignalHubSignalService, SignalProcessingOutcome,
    process_signal_hub_raw_event, signal_hub_raw_dispatcher_allows_processing,
};
pub use self::store::SignalHubStore as SignalHubPort;
pub(crate) use self::store::event_type_pattern_matches;
pub use self::store::{
    FixtureRestoreReport, SignalCapability, SignalCapabilityUpsert, SignalConnection,
    SignalConnectionCreate, SignalConnectionUpdate, SignalHealth, SignalHealthCheckRequest,
    SignalHealthSnapshotWrite, SignalHubError, SignalHubStore, SignalProfile, SignalProfileCreate,
    SignalProfilePolicy, SignalProfileSummary, SignalProfileUpdate, SignalReplayRequest,
    SignalReplayRequestCreate, SignalRuntimeState, SignalRuntimeStateUpdate, SignalSource,
};
pub use self::telegram::dispatch_telegram_raw_signal;
pub use self::whatsapp::dispatch_whatsapp_raw_signal;
```

### `backend/src/domains/signal_hub/policies.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/policies.rs`
- Size bytes / Размер в байтах: `5399`
- Included characters / Включено символов: `5399`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalPolicy {
    pub scope: SignalPolicyScope,
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub mode: SignalPolicyMode,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalPolicyScope {
    Global,
    Source,
    Connection,
    EventPattern,
    Profile,
}

impl SignalPolicyScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Source => "source",
            Self::Connection => "connection",
            Self::EventPattern => "event_pattern",
            Self::Profile => "profile",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "global" => Some(Self::Global),
            "source" => Some(Self::Source),
            "connection" => Some(Self::Connection),
            "event_pattern" => Some(Self::EventPattern),
            "profile" => Some(Self::Profile),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalPolicyMode {
    Enabled,
    Disabled,
    Muted,
    Paused,
    ReplayOnly,
    FixtureOnly,
}

impl SignalPolicyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Muted => "muted",
            Self::Paused => "paused",
            Self::ReplayOnly => "replay_only",
            Self::FixtureOnly => "fixture_only",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "enabled" => Some(Self::Enabled),
            "disabled" => Some(Self::Disabled),
            "muted" => Some(Self::Muted),
            "paused" => Some(Self::Paused),
            "replay_only" => Some(Self::ReplayOnly),
            "fixture_only" => Some(Self::FixtureOnly),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalPolicyDecision {
    Allow,
    Rejected { reason: String },
    Paused { reason: String },
    Muted { reason: String },
}

pub struct SignalPolicyEvaluator {
    now: DateTime<Utc>,
}

impl SignalPolicyEvaluator {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self { now }
    }

    pub fn decide(
        &self,
        source_code: &str,
        connection_id: Option<&str>,
        event_type: &str,
        policies: &[SignalPolicy],
    ) -> SignalPolicyDecision {
        let matching: Vec<&SignalPolicy> = policies
            .iter()
            .filter(|policy| self.policy_applies(policy, source_code, connection_id, event_type))
            .collect();

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Disabled))
        {
            return SignalPolicyDecision::Rejected {
                reason: policy.reason.clone(),
            };
        }

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Paused))
        {
            return SignalPolicyDecision::Paused {
                reason: policy.reason.clone(),
            };
        }

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Muted))
        {
            return SignalPolicyDecision::Muted {
                reason: policy.reason.clone(),
            };
        }

        SignalPolicyDecision::Allow
    }

    fn policy_applies(
        &self,
        policy: &SignalPolicy,
        source_code: &str,
        connection_id: Option<&str>,
        event_type: &str,
    ) -> bool {
        if policy
            .expires_at
            .is_some_and(|expires_at| expires_at <= self.now)
        {
            return false;
        }

        if policy
            .source_code
            .as_deref()
            .is_some_and(|policy_source| policy_source != source_code)
        {
            return false;
        }

        if policy
            .connection_id
            .as_deref()
            .is_some_and(|policy_connection| Some(policy_connection) != connection_id)
        {
            return false;
        }

        if policy
            .event_pattern
            .as_deref()
            .is_some_and(|pattern| !event_type_matches(pattern, event_type))
        {
            return false;
        }

        match policy.scope {
            SignalPolicyScope::Global => true,
            SignalPolicyScope::Source => policy.source_code.is_some(),
            SignalPolicyScope::Connection => policy.connection_id.is_some(),
            SignalPolicyScope::EventPattern => policy.event_pattern.is_some(),
            SignalPolicyScope::Profile => true,
        }
    }
}

fn event_type_matches(pattern: &str, event_type: &str) -> bool {
    if pattern == event_type {
        return true;
    }

    let Some(prefix) = pattern.strip_suffix(".*") else {
        return false;
    };

    event_type.starts_with(prefix)
}
```

### `backend/src/domains/signal_hub/profiles.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/profiles.rs`
- Size bytes / Размер в байтах: `6918`
- Included characters / Включено символов: `6918`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;

use super::store::{
    SignalHubError, SignalHubStore, SignalProfile, SignalProfileCreate, SignalProfilePolicy,
    SignalProfileSummary, SignalProfileUpdate,
};
use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::settings::ApplicationSettingsStore;

const ACTIVE_PROFILE_SETTING_KEY: &str = "signal_hub.active_profile";

#[derive(Clone)]
pub struct SignalHubProfileService {
    signal_store: SignalHubStore,
    settings_store: ApplicationSettingsStore,
    event_store: EventStore,
}

impl SignalHubProfileService {
    pub fn new(
        signal_store: SignalHubStore,
        settings_store: ApplicationSettingsStore,
        event_store: EventStore,
    ) -> Self {
        Self {
            signal_store,
            settings_store,
            event_store,
        }
    }

    pub async fn list_profiles(&self) -> Result<Vec<SignalProfileSummary>, SignalHubError> {
        let active_profile_code = self.active_profile_code().await?;
        let profiles = self.signal_store.list_profiles().await?;
        Ok(profiles
            .into_iter()
            .map(|profile| SignalProfileSummary {
                id: profile.id,
                code: profile.code.clone(),
                display_name: profile.display_name,
                description: profile.description,
                policy_count: profile.source_policies.len(),
                source_policies: profile.source_policies,
                is_system: profile.is_system,
                is_active: active_profile_code.as_deref() == Some(profile.code.as_str()),
                created_at: profile.created_at,
                updated_at: profile.updated_at,
            })
            .collect())
    }

    pub async fn create_profile(
        &self,
        request: &SignalProfileCreate,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        let profile = self.signal_store.create_profile(request).await?;
        self.append_profile_event("signal.profile.created", &profile)
            .await?;
        self.summary_for_code(&profile.code).await
    }

    pub async fn update_profile(
        &self,
        request: &SignalProfileUpdate,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        let profile = self.signal_store.update_profile(request).await?;
        self.append_profile_event("signal.profile.updated", &profile)
            .await?;
        self.summary_for_code(&profile.code).await
    }

    pub async fn remove_profile(
        &self,
        profile_code: &str,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        let profile_code = profile_code.trim();
        if profile_code.is_empty() {
            return Err(SignalHubError::EmptyField("profile_code"));
        }

        self.settings_store.repair_declared_settings().await?;
        let active_profile_code = self.active_profile_code().await?;
        let removed = self.signal_store.delete_profile(profile_code).await?;

        if active_profile_code.as_deref() == Some(removed.code.as_str()) {
            self.settings_store
                .update_setting_value(
                    ACTIVE_PROFILE_SETTING_KEY,
                    &json!("production"),
                    "makosh-frontend",
                )
                .await?;
        }

        self.append_profile_event("signal.profile.removed", &removed)
            .await?;

        Ok(SignalProfileSummary {
            id: removed.id,
            code: removed.code,
            display_name: removed.display_name,
            description: removed.description,
            policy_count: removed.source_policies.len(),
            source_policies: removed.source_policies,
            is_system: removed.is_system,
            is_active: false,
            created_at: removed.created_at,
            updated_at: removed.updated_at,
        })
    }

    pub async fn apply_profile(
        &self,
        profile_code: &str,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        let profile_code = profile_code.trim();
        if profile_code.is_empty() {
            return Err(SignalHubError::EmptyField("profile_code"));
        }

        let profile = self
            .signal_store
            .profile_by_code(profile_code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(profile_code.to_owned()))?;

        self.settings_store.repair_declared_settings().await?;
        self.signal_store.expire_managed_profile_policies().await?;
        for policy in &profile.source_policies {
            self.signal_store
                .create_profile_managed_policy(&profile.code, policy)
                .await?;
        }
        self.settings_store
            .update_setting_value(
                ACTIVE_PROFILE_SETTING_KEY,
                &json!(profile.code),
                "makosh-frontend",
            )
            .await?;
        self.append_profile_event("signal.profile.applied", &profile)
            .await?;

        self.summary_for_code(&profile.code).await
    }

    async fn active_profile_code(&self) -> Result<Option<String>, SignalHubError> {
        self.settings_store.repair_declared_settings().await?;
        Ok(self
            .settings_store
            .setting(ACTIVE_PROFILE_SETTING_KEY)
            .await?
            .and_then(|setting| setting.value.as_str().map(ToOwned::to_owned)))
    }

    async fn append_profile_event(
        &self,
        event_type: &str,
        profile: &SignalProfile,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!(
                "evt_{}_{}_{}",
                event_type.replace('.', "_"),
                profile.code,
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
            event_type,
            Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": "system",
                "source_id": profile.code,
            }),
            json!({
                "kind": "signal_profile",
                "entity_id": profile.code,
                "profile_code": profile.code,
            }),
        )
        .payload(json!({
            "profile_code": profile.code,
            "policy_count": profile.source_policies.len(),
        }))
        .provenance(json!({
            "source": "signal_hub_profile_service",
            "profile_code": profile.code,
        }))
        .build()?;
        self.event_store.append_for_dispatch(&event).await?;
        Ok(())
    }

    async fn summary_for_code(
        &self,
        profile_code: &str,
    ) -> Result<SignalProfileSummary, SignalHubError> {
        self.list_profiles()
            .await?
            .into_iter()
            .find(|item| item.code == profile_code)
            .ok_or_else(|| SignalHubError::ProfileNotFound(profile_code.to_owned()))
    }
}
```
