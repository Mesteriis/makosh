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

- Chunk ID / ID чанка: `046-source-backend-part-026`
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

### `backend/src/domains/obligations/models/entity_kind.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/models/entity_kind.rs`
- Size bytes / Размер в байтах: `1615`
- Included characters / Включено символов: `1615`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use super::super::errors::ObligationStoreError;

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

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObligationStoreError> {
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
            _ => Err(ObligationStoreError::UnknownEntityKind(value.to_owned())),
        }
    }
}
```

### `backend/src/domains/obligations/models/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/models/evidence.rs`
- Size bytes / Размер в байтах: `2581`
- Included characters / Включено символов: `2581`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::super::errors::ObligationStoreError;
use super::super::validation::{validate_json_object, validate_non_empty, validate_score};
use super::source_kind::ObligationEvidenceSourceKind;

#[derive(Clone, Debug, PartialEq)]
pub struct NewObligationEvidence {
    pub source_kind: ObligationEvidenceSourceKind,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub quote: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObligationEvidence {
    pub fn new(source_kind: ObligationEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            observation_id: None,
            quote: None,
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn observation(observation_id: impl Into<String>) -> Self {
        let observation_id = observation_id.into();
        Self {
            source_kind: ObligationEvidenceSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id),
            quote: None,
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn quote(mut self, quote: impl Into<String>) -> Self {
        self.quote = Some(quote.into());
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

    pub fn with_observation_id<T: Into<String>>(mut self, observation_id: Option<T>) -> Self {
        self.observation_id = observation_id.map(Into::into);
        self
    }

    pub(in crate::domains::obligations) fn validate(&self) -> Result<(), ObligationStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        if let Some(observation_id) = &self.observation_id {
            validate_non_empty("observation_id", observation_id)?;
        }
        if self.source_kind == ObligationEvidenceSourceKind::Observation
            && self.observation_id.as_deref() != Some(self.source_id.as_str())
        {
            return Err(ObligationStoreError::InvalidObservationEvidenceSource);
        }
        validate_score("evidence confidence", self.confidence)?;
        validate_json_object("evidence metadata", &self.metadata)?;
        if let Some(quote) = &self.quote {
            validate_non_empty("quote", quote)?;
        }

        Ok(())
    }
}
```

### `backend/src/domains/obligations/models/obligation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/models/obligation.rs`
- Size bytes / Размер в байтах: `3445`
- Included characters / Включено символов: `3445`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::super::errors::ObligationStoreError;
use super::super::validation::{validate_json_object, validate_non_empty, validate_score};
use super::entity_kind::ObligationEntityKind;
use super::states::{ObligationReviewState, ObligationRiskState, ObligationStatus};

#[derive(Clone, Debug, PartialEq)]
pub struct NewObligation {
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub status: ObligationStatus,
    pub review_state: ObligationReviewState,
    pub due_at: Option<DateTime<Utc>>,
    pub condition: Option<String>,
    pub risk_state: ObligationRiskState,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObligation {
    pub fn new(
        obligated_entity_kind: ObligationEntityKind,
        obligated_entity_id: impl Into<String>,
        statement: impl Into<String>,
        confidence: f64,
        review_state: ObligationReviewState,
    ) -> Self {
        Self {
            obligated_entity_kind,
            obligated_entity_id: obligated_entity_id.into(),
            beneficiary_entity_kind: None,
            beneficiary_entity_id: None,
            statement: statement.into(),
            status: ObligationStatus::Open,
            review_state,
            due_at: None,
            condition: None,
            risk_state: ObligationRiskState::None,
            confidence,
            metadata: json!({}),
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

    pub fn status(mut self, status: ObligationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn due_at(mut self, due_at: DateTime<Utc>) -> Self {
        self.due_at = Some(due_at);
        self
    }

    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    pub fn risk_state(mut self, risk_state: ObligationRiskState) -> Self {
        self.risk_state = risk_state;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(in crate::domains::obligations) fn validate(&self) -> Result<(), ObligationStoreError> {
        validate_non_empty("obligated_entity_id", &self.obligated_entity_id)?;
        validate_non_empty("statement", &self.statement)?;
        validate_score("confidence", self.confidence)?;
        validate_json_object("obligation metadata", &self.metadata)?;

        match (
            self.beneficiary_entity_kind,
            self.beneficiary_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(beneficiary_entity_id)) => {
                validate_non_empty("beneficiary_entity_id", beneficiary_entity_id)?;
            }
            _ => return Err(ObligationStoreError::PartialBeneficiary),
        }

        if let Some(condition) = &self.condition {
            validate_non_empty("condition", condition)?;
        }

        Ok(())
    }
}
```

### `backend/src/domains/obligations/models/read_model.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/models/read_model.rs`
- Size bytes / Размер в байтах: `877`
- Included characters / Включено символов: `877`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::entity_kind::ObligationEntityKind;
use super::states::{ObligationReviewState, ObligationRiskState, ObligationStatus};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Obligation {
    pub obligation_id: String,
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub status: ObligationStatus,
    pub review_state: ObligationReviewState,
    pub due_at: Option<DateTime<Utc>>,
    pub condition: Option<String>,
    pub risk_state: ObligationRiskState,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### `backend/src/domains/obligations/models/source_kind.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/models/source_kind.rs`
- Size bytes / Размер в байтах: `987`
- Included characters / Включено символов: `987`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationEvidenceSourceKind {
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

impl ObligationEvidenceSourceKind {
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
```

### `backend/src/domains/obligations/models/states.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/models/states.rs`
- Size bytes / Размер в байтах: `1896`
- Included characters / Включено символов: `1896`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use super::super::errors::ObligationStoreError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    Open,
    Fulfilled,
    Waived,
    Disputed,
    Canceled,
}

impl ObligationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Fulfilled => "fulfilled",
            Self::Waived => "waived",
            Self::Disputed => "disputed",
            Self::Canceled => "canceled",
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

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObligationStoreError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ObligationStoreError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationRiskState {
    None,
    Watch,
    AtRisk,
    Breached,
}

impl ObligationRiskState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Watch => "watch",
            Self::AtRisk => "at_risk",
            Self::Breached => "breached",
        }
    }
}
```

### `backend/src/domains/obligations/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/ports.rs`
- Size bytes / Размер в байтах: `63`
- Included characters / Включено символов: `63`
- Truncated / Обрезано: `no`

```rust
pub use super::store::ObligationStore as ObligationReviewPort;
```

### `backend/src/domains/obligations/row_mapping.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/row_mapping.rs`
- Size bytes / Размер в байтах: `2430`
- Included characters / Включено символов: `2430`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::ObligationStoreError;
use super::models::{
    Obligation, ObligationEntityKind, ObligationReviewState, ObligationRiskState, ObligationStatus,
};

pub(super) fn row_to_obligation(row: PgRow) -> Result<Obligation, ObligationStoreError> {
    let beneficiary_entity_kind = row
        .try_get::<Option<String>, _>("beneficiary_entity_kind")?
        .map(parse_entity_kind)
        .transpose()?;

    Ok(Obligation {
        obligation_id: row.try_get("obligation_id")?,
        obligated_entity_kind: parse_entity_kind(row.try_get("obligated_entity_kind")?)?,
        obligated_entity_id: row.try_get("obligated_entity_id")?,
        beneficiary_entity_kind,
        beneficiary_entity_id: row.try_get("beneficiary_entity_id")?,
        statement: row.try_get("statement")?,
        status: parse_status(row.try_get("status")?)?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        due_at: row.try_get("due_at")?,
        condition: row.try_get("condition")?,
        risk_state: parse_risk_state(row.try_get("risk_state")?)?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<ObligationEntityKind, ObligationStoreError> {
    ObligationEntityKind::parse(value)
}

fn parse_status(value: String) -> Result<ObligationStatus, ObligationStoreError> {
    match value.as_str() {
        "open" => Ok(ObligationStatus::Open),
        "fulfilled" => Ok(ObligationStatus::Fulfilled),
        "waived" => Ok(ObligationStatus::Waived),
        "disputed" => Ok(ObligationStatus::Disputed),
        "canceled" => Ok(ObligationStatus::Canceled),
        _ => Err(ObligationStoreError::UnknownStatus(value)),
    }
}

fn parse_review_state(value: String) -> Result<ObligationReviewState, ObligationStoreError> {
    ObligationReviewState::parse(value)
}

fn parse_risk_state(value: String) -> Result<ObligationRiskState, ObligationStoreError> {
    match value.as_str() {
        "none" => Ok(ObligationRiskState::None),
        "watch" => Ok(ObligationRiskState::Watch),
        "at_risk" => Ok(ObligationRiskState::AtRisk),
        "breached" => Ok(ObligationRiskState::Breached),
        _ => Err(ObligationStoreError::UnknownRiskState(value)),
    }
}
```

### `backend/src/domains/obligations/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/service.rs`
- Size bytes / Размер в байтах: `2025`
- Included characters / Включено символов: `2025`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{Obligation, ObligationReviewState, ObligationStore, ObligationStoreError};

#[derive(Clone)]
pub struct ObligationCommandService {
    pool: PgPool,
}

impl ObligationCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationCommandServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "obligation_id": obligation_id,
                        "review_state": review_state.as_str(),
                        "operation": "obligation_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("obligation://{obligation_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "obligations_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let obligation = ObligationStore::new(self.pool.clone())
            .set_review_state_with_observation(
                obligation_id,
                review_state,
                Some(&observation.observation_id),
                None,
            )
            .await?;

        Ok(obligation)
    }
}

#[derive(Debug, Error)]
pub enum ObligationCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),
}
```

### `backend/src/domains/obligations/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/store.rs`
- Size bytes / Размер в байтах: `11932`
- Included characters / Включено символов: `11932`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashSet;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use super::errors::ObligationStoreError;
use super::evidence::{
    link_obligation_review_transition_in_transaction, link_obligation_support_in_transaction,
};
use super::ids::{evidence_id, obligation_id};
use super::models::{
    NewObligation, NewObligationEvidence, Obligation, ObligationEntityKind, ObligationReviewState,
};
use super::row_mapping::row_to_obligation;
use super::validation::{validate_non_empty, validate_obligation_with_evidence};

#[derive(Clone)]
pub struct ObligationStore {
    pool: PgPool,
}

impl ObligationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        obligation: &NewObligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<Obligation, ObligationStoreError> {
        validate_obligation_with_evidence(obligation, evidence)?;

        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::upsert_with_evidence_in_transaction(&mut transaction, obligation, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        obligation: &NewObligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<Obligation, ObligationStoreError> {
        validate_evidence_observations_exist(transaction, evidence).await?;
        let obligation_id = obligation_id(obligation);
        let row = sqlx::query(
            r#"
            INSERT INTO obligations (
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence,
                metadata
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                CAST($12 AS NUMERIC(5,4)),
                $13
            )
            ON CONFLICT (obligation_id)
            DO UPDATE SET
                status = EXCLUDED.status,
                review_state = EXCLUDED.review_state,
                due_at = EXCLUDED.due_at,
                condition = EXCLUDED.condition,
                risk_state = EXCLUDED.risk_state,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&obligation_id)
        .bind(obligation.obligated_entity_kind.as_str())
        .bind(&obligation.obligated_entity_id)
        .bind(obligation.beneficiary_entity_kind.map(|kind| kind.as_str()))
        .bind(&obligation.beneficiary_entity_id)
        .bind(&obligation.statement)
        .bind(obligation.status.as_str())
        .bind(obligation.review_state.as_str())
        .bind(obligation.due_at)
        .bind(&obligation.condition)
        .bind(obligation.risk_state.as_str())
        .bind(obligation.confidence)
        .bind(&obligation.metadata)
        .fetch_one(&mut **transaction)
        .await?;

        let stored = row_to_obligation(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&obligation_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO obligation_evidence (
                    evidence_id,
                    obligation_id,
                    source_kind,
                    source_id,
                    observation_id,
                    quote,
                    confidence,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS NUMERIC(5,4)), $8)
                ON CONFLICT (obligation_id, source_kind, source_id)
                DO UPDATE SET
                    observation_id = EXCLUDED.observation_id,
                    quote = EXCLUDED.quote,
                    confidence = EXCLUDED.confidence,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&obligation_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(item.observation_id.as_deref())
            .bind(&item.quote)
            .bind(item.confidence)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;

            if let Some(observation_id) = item.observation_id.as_deref() {
                link_obligation_support_in_transaction(
                    transaction,
                    observation_id,
                    obligation_id.clone(),
                    item.confidence,
                    json!({
                        "source_kind": item.source_kind.as_str(),
                        "source_id": item.source_id,
                    }),
                )
                .await?;
            }
        }

        Ok(stored)
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: ObligationEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Obligation>, ObligationStoreError> {
        validate_non_empty("entity_id", entity_id)?;
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
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM obligations
            WHERE (obligated_entity_kind = $1 AND obligated_entity_id = $2)
               OR (beneficiary_entity_kind = $1 AND beneficiary_entity_id = $2)
            ORDER BY updated_at DESC, obligation_id ASC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_obligation).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: ObligationReviewState,
        limit: i64,
    ) -> Result<Vec<Obligation>, ObligationStoreError> {
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
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM obligations
            WHERE review_state = $1
            ORDER BY updated_at DESC, obligation_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_obligation).collect()
    }

    pub async fn set_review_state(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationStoreError> {
        self.set_review_state_with_observation(obligation_id, review_state, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
        observation_id: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Obligation, ObligationStoreError> {
        validate_non_empty("obligation_id", obligation_id)?;
        let mut transaction = self.pool.begin().await?;
        let obligation = Self::set_review_state_in_transaction(
            &mut transaction,
            obligation_id,
            review_state,
            observation_id,
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(obligation)
    }

    pub(crate) async fn set_review_state_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        obligation_id: &str,
        review_state: ObligationReviewState,
        observation_id: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Obligation, ObligationStoreError> {
        validate_non_empty("obligation_id", obligation_id)?;
        let row = sqlx::query(
            r#"
            UPDATE obligations
            SET
                review_state = $1,
                updated_at = now()
            WHERE obligation_id = $2
            RETURNING
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(obligation_id)
        .fetch_optional(&mut **transaction)
        .await?
        .ok_or(ObligationStoreError::ObligationNotFound)?;

        let obligation = row_to_obligation(row)?;
        link_obligation_review_transition_in_transaction(
            transaction,
            observation_id,
            &obligation.obligation_id,
            obligation.review_state,
            metadata,
        )
        .await?;
        Ok(obligation)
    }
}

async fn validate_evidence_observations_exist(
    transaction: &mut Transaction<'_, Postgres>,
    evidence: &[NewObligationEvidence],
) -> Result<(), ObligationStoreError> {
    let observation_ids: Vec<String> = evidence
        .iter()
        .filter_map(|item| item.observation_id.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if observation_ids.is_empty() {
        return Ok(());
    }

    let stored_observation_ids: HashSet<String> = sqlx::query_scalar::<_, String>(
        r#"
        SELECT observation_id
        FROM observations
        WHERE observation_id = ANY($1)
        "#,
    )
    .bind(&observation_ids)
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .collect();

    for observation_id in observation_ids {
        if !stored_observation_ids.contains(&observation_id) {
            return Err(ObligationStoreError::ObservationNotFound(observation_id));
        }
    }

    Ok(())
}
```

### `backend/src/domains/obligations/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/obligations/validation.rs`
- Size bytes / Размер в байтах: `1220`
- Included characters / Включено символов: `1220`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::ObligationStoreError;
use super::models::{NewObligation, NewObligationEvidence};

pub(super) fn validate_obligation_with_evidence(
    obligation: &NewObligation,
    evidence: &[NewObligationEvidence],
) -> Result<(), ObligationStoreError> {
    obligation.validate()?;
    if evidence.is_empty() {
        return Err(ObligationStoreError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }

    Ok(())
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ObligationStoreError> {
    if value.trim().is_empty() {
        return Err(ObligationStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_score(
    field_name: &'static str,
    value: f64,
) -> Result<(), ObligationStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(ObligationStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ObligationStoreError> {
    if !value.is_object() {
        return Err(ObligationStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
```

### `backend/src/domains/organizations/api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/api.rs`
- Size bytes / Размер в байтах: `22163`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::Postgres;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use super::core::{
    OrgCoreError, OrgDomainStore, OrgIdentityStore, link_email_domain_projection_in_transaction,
    link_organization_in_transaction,
};
use crate::platform::observations::ObservationStoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Organization {
    pub organization_id: String,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub org_type: Option<String>,
    pub status: String,
    pub country: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub timezone: Option<String>,
    pub trust_score: Option<i16>,
    pub health_status: Option<String>,
    pub priority: Option<String>,
    pub notes: Option<String>,
    pub tags: Value,
    pub org_metadata: Value,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub interaction_count: i32,
    pub registration_number: Option<String>,
    pub country_of_registration: Option<String>,
    pub vat: Option<String>,
    pub cif: Option<String>,
    pub nif: Option<String>,
    pub tax_id: Option<String>,
    pub legal_address: Option<String>,
    pub registry_source: Option<String>,
    pub registry_last_verified: Option<DateTime<Utc>>,
    pub communication_style: Option<String>,
    pub verbosity: Option<String>,
    pub formality: Option<String>,
    pub secondary_languages: Option<Value>,
    pub preferred_tone: Option<String>,
    pub official_style_required: Option<bool>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub watchlist: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrganizationStore {
    pool: PgPool,
}

impl OrganizationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        display_name: &str,
        org_type: Option<&str>,
    ) -> Result<Organization, OrganizationError> {
        let mut transaction = self.pool.begin().await?;
        let organization =
            Self::create_in_transaction(&mut transaction, display_name, org_type).await?;
        transaction.commit().await?;
        Ok(organization)
    }

    pub async fn create_with_observation(
        &self,
        display_name: &str,
        org_type: Option<&str>,
        observation_id: &str,
    ) -> Result<Organization, OrganizationError> {
        let mut transaction = self.pool.begin().await?;
        let organization =
            Self::create_in_transaction(&mut transaction, display_name, org_type).await?;
        link_organization_in_transaction(
            &mut transaction,
            observation_id,
            &organization.organization_id,
            "create",
            None,
        )
        .await?;
        transaction.commit().await?;
        Ok(organization)
    }

    async fn create_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        display_name: &str,
        org_type: Option<&str>,
    ) -> Result<Organization, OrganizationError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let org_id = format!("org:v1:{ts:x}");
        let row = sqlx::query(
            "INSERT INTO organizations (organization_id, display_name, org_type)
             VALUES ($1, $2, $3)
             RETURNING organization_id, display_name, legal_name, org_type, status, country, city,
                       address, website, industry, description, primary_language, timezone,
                       trust_score, health_status, priority, notes, tags, org_metadata,
                       last_interaction_at, interaction_count,
                       registration_number, country_of_registration, vat, cif, nif, tax_id,
                       legal_address, registry_source, registry_last_verified,
                       communication_style, verbosity, formality, secondary_languages,
                       preferred_tone, official_style_required,
                       last_health_check, watchlist, created_at, updated_at",
        )
        .bind(&org_id)
        .bind(display_name)
        .bind(org_type)
        .fetch_one(&mut **transaction)
        .await?;
        row_to_org(row)
    }

    pub async fn upsert_review_organization(
        &self,
        organization_id: &str,
        display_name: &str,
        description: Option<&str>,
    ) -> Result<Organization, OrganizationError> {
        if organization_id.trim().is_empty() {
            return Err(OrganizationError::Validation(
                "organization_id must not be empty".to_owned(),
            ));
        }
        if display_name.trim().is_empty() {
            return Err(OrganizationError::Validation(
                "display_name must not be empty".to_owned(),
            ));
        }

        let row = sqlx::query(
            r#"
            INSERT INTO organizations (
                organization_id,
                display_name,
                org_type,
                description
            )
            VALUES ($1, $2, 'derived', $3)
            ON CONFLICT (organization_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                description = EXCLUDED.description,
                updated_at = now()
            RETURNING organization_id, display_name, legal_name, org_type, status, country, city,
                      address, website, industry, description, primary_language, timezone,
                      trust_score, health_status, priority, notes, tags, org_metadata,
                      last_interaction_at, interaction_count,
                      registration_number, country_of_registration, vat, cif, nif, tax_id,
                      legal_address, registry_source, registry_last_verified,
                      communication_style, verbosity, formality, secondary_languages,
                      preferred_tone, official_style_required,
                      last_health_check, watchlist, created_at, updated_at
            "#,
        )
        .bind(organization_id.trim())
        .bind(display_name.trim())
        .bind(description.map(str::trim).filter(|value| !value.is_empty()))
        .fetch_one(&self.pool)
        .await?;
        row_to_org(row)
    }

    pub async fn upsert_email_domain_organization(
        &self,
        domain: &str,
    ) -> Result<(Organization, bool), OrganizationError> {
        self.upsert_email_domain_organization_internal(domain, None)
            .await
    }

    pub async fn upsert_email_domain_organization_with_observation(
        &self,
        domain: &str,
        observation_id: &str,
    ) -> Result<(Organization, bool), OrganizationError> {
        self.upsert_email_domain_organization_internal(domain, Some(observation_id))
            .await
    }

    async fn upsert_email_domain_organization_internal(
        &self,
        domain: &str,
        observation_id: Option<&str>,
    ) -> Result<(Organization, bool), OrganizationError> {
        let domain = domain.trim().to_ascii_lowercase();
        if domain.is_empty() {
            return Err(OrganizationError::Validation(
                "organization domain must not be empty".to_owned(),
            ));
        }

        let organization_id = format!("org:v1:email-domain:{}:{domain}", domain.len());
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO organizations (organization_id, display_name, org_type, website)
            VALUES ($1, $2, 'company', $3)
            ON CONFLICT (organization_id)
            DO UPDATE SET
                updated_at = now(),
                last_interaction_at = now(),
                interaction_count = organizations.interaction_count + 1
            RETURNING
                organization_id, display_name, legal_name, org_type, status, country, city,
                address, website, industry, description, primary_language, timezone,
                trust_score, health_status, priority, notes, tags, org_metadata,
                last_interaction_at, interaction_count,
                registration_number, country_of_registration, vat, cif, nif, tax_id,
                legal_address, registry_source, registry_last_verified,
                communication_style, verbosity, formality, secondary_languages,
                preferred_tone, official_style_required,
                last_health_check, watchlist, created_at, updated_at,
                (xmax = 0) AS inserted
            "#,
        )
        .bind(&organization_id)
        .bind(&domain)
        .bind(format!("https://{domain}"))
        .fetch_one(&mut *transaction)
        .await?;
        let inserted: bool = row.try_get("inserted")?;
        let organization = row_to_org(row)?;
        if let Some(observation_id) = observation_id {
            Self::link_email_domain_projection_evidence(
                &mut transaction,
                observation_id,
                &organization,
                &domain,
                inserted,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok((organization, inserted))
    }

    pub async fn get(
        &self,
        organization_id: &str,
    ) -> Result<Option<Organization>, OrganizationError> {
        let row = sqlx::query(
            "SELECT organization_id, display_name, legal_name, org_type, status, country, city,
                    address, website, industry, description, primary_language, timezone,
                    trust_score, health_status, priority, notes, tags, org_metadata,
                    last_interaction_at, interaction_count,
                    registration_number, country_of_registration, vat, cif, nif, tax_id,
                    legal_address, registry_source, registry_last_verified,
                    communication_style, verbosity, formality, secondary_languages,
                    preferred_tone, official_style_required,
                    last_health_check, watchlist, created_at, updated_at
             FROM organizations WHERE organization_id = $1",
        )
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_org).transpose()
    }

    pub async fn list(
        &self,
        org_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Organization>, OrganizationError> {
        let limit = limit.clamp(1, 100);
        let rows =
            if let Some(t) = org_type {
                sqlx::query(
                "SELECT organization_id, display_name, legal_name, org_type, status, country, city,
                        address, website, industry, description, primary_language, timezone,
                        trust_score, health_status, priority, notes, tags, org_metadata,
                        last_interaction_at, interaction_count,
                        registration_number, country_of_registration, vat, cif, nif, tax_id,
                        legal_address, registry_source, registry_last_verified,
                        communication_style, verbosity, formality, secondary_languages,
                        preferred_tone, official_style_required,
                        last_health_check, watchlist, created_at, updated_at
                 FROM organizations WHERE org_type = $1 ORDER BY interaction_count DESC LIMIT $2"
            ).bind(t).bind(limit).fetch_all(&self.pool).await?
            } else {
                sqlx::query(
                "SELECT organization_id, display_name, legal_name, org_type, status, country, city,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/organizations/core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core.rs`
- Size bytes / Размер в байтах: `750`
- Included characters / Включено символов: `750`
- Truncated / Обрезано: `no`

```rust
mod aliases;
mod contact_links;
mod departments;
mod domains;
mod errors;
mod evidence;
mod identity;
mod related;

pub use aliases::{OrgAliasStore, OrganizationAlias};
pub use contact_links::OrgContactLinkStore as OrganizationContactLinkPort;
pub use contact_links::{OrgContactLink, OrgContactLinkStore};
pub use departments::{OrgDepartment, OrgDepartmentStore};
pub use domains::{OrgDomainStore, OrganizationDomain};
pub use errors::OrgCoreError;
pub(crate) use evidence::{
    link_email_domain_projection_in_transaction, link_entity_in_transaction,
    link_organization_in_transaction, link_review_transition_in_transaction,
};
pub use identity::{OrgIdentityStore, OrganizationIdentity};
pub use related::{RelatedOrgStore, RelatedOrganization};
```

### `backend/src/domains/organizations/core/aliases.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core/aliases.rs`
- Size bytes / Размер в байтах: `4338`
- Included characters / Включено символов: `4338`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;

use super::{OrgCoreError, link_entity_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationAlias {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub alias_type: String,
    pub source: String,
    pub confidence: f64,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgAliasStore {
    pool: PgPool,
}

impl OrgAliasStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationAlias>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, alias_type, source, confidence::float8 AS confidence, valid_from, valid_to, created_at FROM organization_aliases WHERE organization_id=$1 ORDER BY name")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrganizationAlias {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    alias_type: row.try_get("alias_type")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    valid_from: row.try_get("valid_from")?,
                    valid_to: row.try_get("valid_to")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        alias_type: &str,
        source: &str,
    ) -> Result<OrganizationAlias, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let alias =
            Self::add_in_transaction(&mut transaction, org_id, name, alias_type, source).await?;
        transaction.commit().await?;
        Ok(alias)
    }

    pub async fn add_with_observation(
        &self,
        org_id: &str,
        name: &str,
        alias_type: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<OrganizationAlias, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let alias =
            Self::add_in_transaction(&mut transaction, org_id, name, alias_type, source).await?;
        link_entity_in_transaction(
            &mut transaction,
            observation_id,
            "alias",
            &alias.id,
            json!({
                "organization_id": org_id,
                "alias_type": alias.alias_type,
            }),
        )
        .await?;
        transaction.commit().await?;
        Ok(alias)
    }

    pub(crate) async fn add_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        org_id: &str,
        name: &str,
        alias_type: &str,
        source: &str,
    ) -> Result<OrganizationAlias, OrgCoreError> {
        let alias_type = normalize_alias_type(alias_type);
        let row = sqlx::query("INSERT INTO organization_aliases (organization_id, name, alias_type, source) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, name, alias_type, source, confidence::float8 AS confidence, valid_from, valid_to, created_at")
            .bind(org_id)
            .bind(name)
            .bind(alias_type)
            .bind(source)
            .fetch_one(&mut **transaction)
            .await?;

        Ok(OrganizationAlias {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            name: row.try_get("name")?,
            alias_type: row.try_get("alias_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

fn normalize_alias_type(alias_type: &str) -> &str {
    match alias_type {
        "former_name" => "former",
        other => other,
    }
}
```

### `backend/src/domains/organizations/core/contact_links.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core/contact_links.rs`
- Size bytes / Размер в байтах: `7255`
- Included characters / Включено символов: `7255`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::{OrgCoreError, link_entity_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgContactLink {
    pub id: String,
    pub organization_id: String,
    pub person_id: String,
    pub role: Option<String>,
    pub department: Option<String>,
    pub source: String,
    pub confidence: f64,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgContactLinkStore {
    pool: PgPool,
}

impl OrgContactLinkStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_org(&self, org_id: &str) -> Result<Vec<OrgContactLink>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, person_id, role, department, source, confidence::float8 AS confidence, valid_from, valid_to, is_primary, created_at, updated_at FROM organization_contact_links WHERE organization_id=$1 ORDER BY is_primary DESC, role")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgContactLink {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    person_id: row.try_get("person_id")?,
                    role: row.try_get("role")?,
                    department: row.try_get("department")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    valid_from: row.try_get("valid_from")?,
                    valid_to: row.try_get("valid_to")?,
                    is_primary: row.try_get("is_primary")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn link(
        &self,
        org_id: &str,
        person_id: &str,
        role: Option<&str>,
        dept: Option<&str>,
    ) -> Result<OrgContactLink, OrgCoreError> {
        self.link_with_observation(org_id, person_id, role, dept, None, None)
            .await
    }

    pub async fn link_with_observation(
        &self,
        org_id: &str,
        person_id: &str,
        role: Option<&str>,
        dept: Option<&str>,
        source: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<OrgContactLink, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO organization_contact_links (organization_id, person_id, role, department, source) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (organization_id, person_id, role) DO UPDATE SET department=EXCLUDED.department, source=EXCLUDED.source, updated_at=now() RETURNING id::text, organization_id, person_id, role, department, source, confidence::float8 AS confidence, valid_from, valid_to, is_primary, created_at, updated_at")
            .bind(org_id)
            .bind(person_id)
            .bind(role)
            .bind(dept)
            .bind(source.unwrap_or("manual"))
            .fetch_one(&mut *transaction)
            .await?;
        let link = OrgContactLink {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            person_id: row.try_get("person_id")?,
            role: row.try_get("role")?,
            department: row.try_get("department")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            is_primary: row.try_get("is_primary")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };

        if let Some(observation_id) = observation_id {
            link_entity_in_transaction(
                &mut transaction,
                observation_id,
                "contact_link",
                &link.id,
                json!({
                    "organization_id": org_id,
                    "person_id": link.person_id,
                    "role": link.role,
                    "department": link.department,
                }),
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(link)
    }

    pub async fn link_email_participant_with_observation(
        &self,
        org_id: &str,
        person_id: &str,
        message_id: &str,
        observation_id: &str,
    ) -> Result<(OrgContactLink, bool), OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO organization_contact_links (
                organization_id,
                person_id,
                role,
                source,
                confidence
            )
            VALUES ($1, $2, 'email_participant', 'email_sync', 1.0)
            ON CONFLICT (organization_id, person_id, role)
            DO UPDATE SET
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                updated_at = now()
            RETURNING
                id::text,
                organization_id,
                person_id,
                role,
                department,
                source,
                confidence::float8 AS confidence,
                valid_from,
                valid_to,
                is_primary,
                created_at,
                updated_at,
                (xmax = 0) AS inserted
            "#,
        )
        .bind(org_id)
        .bind(person_id)
        .fetch_one(&mut *transaction)
        .await?;
        let link = OrgContactLink {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            person_id: row.try_get("person_id")?,
            role: row.try_get("role")?,
            department: row.try_get("department")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            is_primary: row.try_get("is_primary")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        let inserted: bool = row.try_get("inserted")?;
        transaction.commit().await?;

        Ok((link, inserted))
    }

    pub async fn set_primary(&self, org_id: &str, person_id: &str) -> Result<(), OrgCoreError> {
        sqlx::query(
            "UPDATE organization_contact_links SET is_primary=false WHERE organization_id=$1",
        )
        .bind(org_id)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE organization_contact_links SET is_primary=true WHERE organization_id=$1 AND person_id=$2")
            .bind(org_id)
            .bind(person_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
```

### `backend/src/domains/organizations/core/departments.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core/departments.rs`
- Size bytes / Размер в байтах: `3860`
- Included characters / Включено символов: `3860`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;

use super::{OrgCoreError, link_entity_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgDepartment {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_department_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgDepartmentStore {
    pool: PgPool,
}

impl OrgDepartmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgDepartment>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, description, parent_department_id::text, created_at FROM organization_departments WHERE organization_id=$1 ORDER BY name")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgDepartment {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    parent_department_id: row.try_get("parent_department_id")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<OrgDepartment, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let department =
            Self::add_in_transaction(&mut transaction, org_id, name, description, parent_id)
                .await?;
        transaction.commit().await?;
        Ok(department)
    }

    pub async fn add_with_observation(
        &self,
        org_id: &str,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
        observation_id: &str,
    ) -> Result<OrgDepartment, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let department =
            Self::add_in_transaction(&mut transaction, org_id, name, description, parent_id)
                .await?;
        link_entity_in_transaction(
            &mut transaction,
            observation_id,
            "department",
            &department.id,
            json!({
                "organization_id": org_id,
                "name": department.name,
            }),
        )
        .await?;
        transaction.commit().await?;
        Ok(department)
    }

    pub(crate) async fn add_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        org_id: &str,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<OrgDepartment, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_departments (organization_id, name, description, parent_department_id) VALUES ($1,$2,$3,$4::uuid) RETURNING id::text, organization_id, name, description, parent_department_id::text, created_at")
            .bind(org_id)
            .bind(name)
            .bind(description)
            .bind(parent_id)
            .fetch_one(&mut **transaction)
            .await?;

        Ok(OrgDepartment {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            parent_department_id: row.try_get("parent_department_id")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
```

### `backend/src/domains/organizations/core/domains.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core/domains.rs`
- Size bytes / Размер в байтах: `5035`
- Included characters / Включено символов: `5035`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;

use super::OrgCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationDomain {
    pub id: String,
    pub organization_id: String,
    pub domain: String,
    pub domain_type: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgDomainStore {
    pool: PgPool,
}

impl OrgDomainStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationDomain>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, domain, domain_type, source, confidence::float8 AS confidence, last_verified_at, created_at FROM organization_domains WHERE organization_id=$1 ORDER BY domain_type, domain")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrganizationDomain {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    domain: row.try_get("domain")?,
                    domain_type: row.try_get("domain_type")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    last_verified_at: row.try_get("last_verified_at")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        domain: &str,
        domain_type: &str,
        source: &str,
    ) -> Result<OrganizationDomain, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_domains (organization_id, domain, domain_type, source) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, domain, domain_type, source, confidence::float8 AS confidence, last_verified_at, created_at")
            .bind(org_id)
            .bind(domain)
            .bind(domain_type)
            .bind(source)
            .fetch_one(&self.pool)
            .await?;

        Ok(OrganizationDomain {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            domain: row.try_get("domain")?,
            domain_type: row.try_get("domain_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            last_verified_at: row.try_get("last_verified_at")?,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn upsert_email_domain(
        &self,
        org_id: &str,
        domain: &str,
    ) -> Result<bool, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let (_, inserted) =
            Self::upsert_email_domain_in_transaction(&mut transaction, org_id, domain).await?;
        transaction.commit().await?;
        Ok(inserted)
    }

    pub(crate) async fn upsert_email_domain_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        org_id: &str,
        domain: &str,
    ) -> Result<(OrganizationDomain, bool), OrgCoreError> {
        let result = sqlx::query(
            r#"
            INSERT INTO organization_domains (organization_id, domain, domain_type, source)
            SELECT $1, $2, 'email', 'email_sync'
            WHERE NOT EXISTS (
                SELECT 1
                FROM organization_domains
                WHERE organization_id = $1
                  AND domain = $2
                  AND domain_type != 'former'
            )
            "#,
        )
        .bind(org_id)
        .bind(domain)
        .execute(&mut **transaction)
        .await?;
        let inserted = result.rows_affected() > 0;
        let row = sqlx::query(
            r#"
            SELECT id::text, organization_id, domain, domain_type, source, confidence::float8 AS confidence, last_verified_at, created_at
            FROM organization_domains
            WHERE organization_id = $1
              AND domain = $2
              AND domain_type != 'former'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(domain)
        .fetch_one(&mut **transaction)
        .await?;
        Ok((
            OrganizationDomain {
                id: row.try_get("id")?,
                organization_id: row.try_get("organization_id")?,
                domain: row.try_get("domain")?,
                domain_type: row.try_get("domain_type")?,
                source: row.try_get("source")?,
                confidence: row.try_get("confidence")?,
                last_verified_at: row.try_get("last_verified_at")?,
                created_at: row.try_get("created_at")?,
            },
            inserted,
        ))
    }
}
```

### `backend/src/domains/organizations/core/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core/errors.rs`
- Size bytes / Размер в байтах: `303`
- Included characters / Включено символов: `303`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum OrgCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
```

### `backend/src/domains/organizations/core/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core/evidence.rs`
- Size bytes / Размер в байтах: `3971`
- Included characters / Включено символов: `3971`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::link_domain_entity_in_transaction;

use super::OrgCoreError;

pub(crate) async fn link_organization_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    organization_id: &str,
    action: &str,
    metadata: Option<Value>,
) -> Result<(), OrgCoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        "organization",
        organization_id.to_owned(),
        None,
        None,
        Some(merge_metadata(
            json!({
                "action": action,
            }),
            metadata,
        )),
    )
    .await?;
    Ok(())
}

pub(crate) async fn link_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: &str,
    metadata: Value,
) -> Result<(), OrgCoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        entity_kind,
        entity_id.to_owned(),
        None,
        None,
        Some(metadata),
    )
    .await?;
    Ok(())
}

pub(crate) async fn link_review_transition_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: &str,
    metadata: Value,
) -> Result<(), OrgCoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        entity_kind,
        entity_id.to_owned(),
        Some("review_transition"),
        None,
        Some(metadata),
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn link_email_domain_projection_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    organization_id: &str,
    organization_inserted: bool,
    organization_domain_id: &str,
    domain: &str,
    domain_inserted: bool,
    organization_identity_id: &str,
) -> Result<(), OrgCoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        "organization",
        organization_id.to_owned(),
        Some("email_sync_projection"),
        None,
        Some(json!({
            "projection": "organization",
            "domain": domain,
            "inserted": organization_inserted,
        })),
    )
    .await?;

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        "organization_domain",
        organization_domain_id.to_owned(),
        Some("email_sync_projection"),
        None,
        Some(json!({
            "projection": "organization_domain",
            "organization_id": organization_id,
            "domain": domain,
            "inserted": domain_inserted,
        })),
    )
    .await?;

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        "organization_identity",
        organization_identity_id.to_owned(),
        Some("email_sync_projection"),
        None,
        Some(json!({
            "projection": "organization_identity",
            "organization_id": organization_id,
            "identity_type": "email_domain",
            "identity_value": domain,
        })),
    )
    .await?;

    Ok(())
}

fn merge_metadata(base: Value, extra: Option<Value>) -> Value {
    match extra {
        Some(extra) if base.is_object() && extra.is_object() => {
            let mut merged = base;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => base,
    }
}
```

### `backend/src/domains/organizations/core/identity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core/identity.rs`
- Size bytes / Размер в байтах: `4761`
- Included characters / Включено символов: `4761`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;

use super::{OrgCoreError, link_entity_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationIdentity {
    pub id: String,
    pub organization_id: String,
    pub identity_type: String,
    pub identity_value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub status: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgIdentityStore {
    pool: PgPool,
}

impl OrgIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationIdentity>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, identity_type, identity_value, source, confidence::float8 AS confidence, last_verified_at, status, metadata, created_at, updated_at FROM organization_identities WHERE organization_id = $1 ORDER BY identity_type")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrganizationIdentity {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    identity_type: row.try_get("identity_type")?,
                    identity_value: row.try_get("identity_value")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    last_verified_at: row.try_get("last_verified_at")?,
                    status: row.try_get("status")?,
                    metadata: row.try_get("metadata")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn upsert(
        &self,
        org_id: &str,
        itype: &str,
        ivalue: &str,
        source: &str,
    ) -> Result<OrganizationIdentity, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let identity =
            Self::upsert_in_transaction(&mut transaction, org_id, itype, ivalue, source).await?;
        transaction.commit().await?;
        Ok(identity)
    }

    pub async fn upsert_with_observation(
        &self,
        org_id: &str,
        itype: &str,
        ivalue: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<OrganizationIdentity, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let identity =
            Self::upsert_in_transaction(&mut transaction, org_id, itype, ivalue, source).await?;
        link_entity_in_transaction(
            &mut transaction,
            observation_id,
            "identity",
            &identity.id,
            json!({
                "organization_id": org_id,
                "identity_type": identity.identity_type,
            }),
        )
        .await?;
        transaction.commit().await?;
        Ok(identity)
    }

    pub(crate) async fn upsert_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        org_id: &str,
        itype: &str,
        ivalue: &str,
        source: &str,
    ) -> Result<OrganizationIdentity, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_identities (organization_id, identity_type, identity_value, source) VALUES ($1,$2,$3,$4) ON CONFLICT (identity_type, identity_value) WHERE status='active' DO UPDATE SET updated_at=now() RETURNING id::text, organization_id, identity_type, identity_value, source, confidence::float8 AS confidence, last_verified_at, status, metadata, created_at, updated_at")
            .bind(org_id)
            .bind(itype)
            .bind(ivalue)
            .bind(source)
            .fetch_one(&mut **transaction)
            .await?;

        Ok(OrganizationIdentity {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            identity_type: row.try_get("identity_type")?,
            identity_value: row.try_get("identity_value")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            last_verified_at: row.try_get("last_verified_at")?,
            status: row.try_get("status")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
```

### `backend/src/domains/organizations/core/related.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/core/related.rs`
- Size bytes / Размер в байтах: `2607`
- Included characters / Включено символов: `2607`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::OrgCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelatedOrganization {
    pub id: String,
    pub organization_id: String,
    pub related_organization_id: String,
    pub relation_type: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RelatedOrgStore {
    pool: PgPool,
}

impl RelatedOrgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<RelatedOrganization>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, related_organization_id, relation_type, source, confidence::float8 AS confidence, created_at FROM related_organizations WHERE organization_id=$1")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(RelatedOrganization {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    related_organization_id: row.try_get("related_organization_id")?,
                    relation_type: row.try_get("relation_type")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn relate(
        &self,
        org_id: &str,
        related_id: &str,
        rel_type: &str,
    ) -> Result<RelatedOrganization, OrgCoreError> {
        let row = sqlx::query("INSERT INTO related_organizations (organization_id, related_organization_id, relation_type) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING RETURNING id::text, organization_id, related_organization_id, relation_type, source, confidence::float8 AS confidence, created_at")
            .bind(org_id)
            .bind(related_id)
            .bind(rel_type)
            .fetch_one(&self.pool)
            .await?;

        Ok(RelatedOrganization {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            related_organization_id: row.try_get("related_organization_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
```

### `backend/src/domains/organizations/enrichment.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/enrichment.rs`
- Size bytes / Размер в байтах: `4958`
- Included characters / Включено символов: `4958`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;
use thiserror::Error;

use crate::domains::organizations::core::{OrgCoreError, link_review_transition_in_transaction};
use crate::platform::observations::ObservationStoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgEnrichmentResult {
    pub id: String,
    pub organization_id: String,
    pub source: String,
    pub url: Option<String>,
    pub data: Value,
    pub confidence: f64,
    pub status: String,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgEnrichmentStore {
    pool: PgPool,
}
impl OrgEnrichmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgEnrichmentResult>, OrgEnrichmentError> {
        let rows = sqlx::query("SELECT id::text, organization_id, source, url, data, confidence::float8 AS confidence, status, last_checked_at, applied_at, created_at FROM organization_enrichment_results WHERE organization_id=$1 ORDER BY created_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgEnrichmentResult {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    source: r.try_get("source")?,
                    url: r.try_get("url")?,
                    data: r.try_get("data")?,
                    confidence: r.try_get("confidence")?,
                    status: r.try_get("status")?,
                    last_checked_at: r.try_get("last_checked_at")?,
                    applied_at: r.try_get("applied_at")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn upsert(
        &self,
        org_id: &str,
        source: &str,
        data: Value,
        confidence: f64,
    ) -> Result<OrgEnrichmentResult, OrgEnrichmentError> {
        let row = sqlx::query("INSERT INTO organization_enrichment_results (organization_id, source, data, confidence) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, source, url, data, confidence::float8 AS confidence, status, last_checked_at, applied_at, created_at")
            .bind(org_id).bind(source).bind(&data).bind(confidence).fetch_one(&self.pool).await?;
        Ok(OrgEnrichmentResult {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            source: row.try_get("source")?,
            url: row.try_get("url")?,
            data: row.try_get("data")?,
            confidence: row.try_get("confidence")?,
            status: row.try_get("status")?,
            last_checked_at: row.try_get("last_checked_at")?,
            applied_at: row.try_get("applied_at")?,
            created_at: row.try_get("created_at")?,
        })
    }
    pub async fn apply(&self, id: &str) -> Result<(), OrgEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        Self::apply_in_transaction(&mut transaction, id).await?;
        transaction.commit().await?;
        Ok(())
    }
    pub async fn apply_with_observation(
        &self,
        id: &str,
        observation_id: &str,
    ) -> Result<(), OrgEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        Self::apply_in_transaction(&mut transaction, id).await?;
        link_review_transition_in_transaction(
            &mut transaction,
            observation_id,
            "organization_enrichment_result",
            id,
            json!({
                "operation": "organization_enrichment_apply"
            }),
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }
    pub async fn reject(&self, id: &str) -> Result<(), OrgEnrichmentError> {
        sqlx::query(
            "UPDATE organization_enrichment_results SET status='rejected' WHERE id::text=$1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn apply_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        id: &str,
    ) -> Result<(), OrgEnrichmentError> {
        sqlx::query("UPDATE organization_enrichment_results SET status='applied', applied_at=now() WHERE id::text=$1")
            .bind(id)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum OrgEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Core(#[from] OrgCoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
```

### `backend/src/domains/organizations/finance.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/finance.rs`
- Size bytes / Размер в байтах: `10534`
- Included characters / Включено символов: `10534`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgFinancialInfo {
    pub id: String,
    pub organization_id: String,
    pub bank_name: Option<String>,
    pub iban_masked: Option<String>,
    pub bic: Option<String>,
    pub payment_terms: Option<String>,
    pub currency: Option<String>,
    pub billing_email: Option<String>,
    pub billing_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgFinancialStore {
    pool: PgPool,
}
impl OrgFinancialStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn get(&self, org_id: &str) -> Result<Option<OrgFinancialInfo>, OrgFinanceError> {
        let row = sqlx::query("SELECT id::text, organization_id, bank_name, iban_masked, bic, payment_terms, currency, billing_email, billing_address, created_at, updated_at FROM organization_financial_info WHERE organization_id=$1")
            .bind(org_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(OrgFinancialInfo {
                id: r.try_get("id")?,
                organization_id: r.try_get("organization_id")?,
                bank_name: r.try_get("bank_name")?,
                iban_masked: r.try_get("iban_masked")?,
                bic: r.try_get("bic")?,
                payment_terms: r.try_get("payment_terms")?,
                currency: r.try_get("currency")?,
                billing_email: r.try_get("billing_email")?,
                billing_address: r.try_get("billing_address")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            })
        })
        .transpose()
    }
    pub async fn upsert(
        &self,
        org_id: &str,
        bank: Option<&str>,
        iban: Option<&str>,
        bic: Option<&str>,
    ) -> Result<OrgFinancialInfo, OrgFinanceError> {
        let row = sqlx::query("INSERT INTO organization_financial_info (organization_id, bank_name, iban_masked, bic) VALUES ($1,$2,$3,$4) ON CONFLICT (organization_id) DO UPDATE SET bank_name=EXCLUDED.bank_name, iban_masked=EXCLUDED.iban_masked, bic=EXCLUDED.bic, updated_at=now() RETURNING id::text, organization_id, bank_name, iban_masked, bic, payment_terms, currency, billing_email, billing_address, created_at, updated_at")
            .bind(org_id).bind(bank).bind(iban).bind(bic).fetch_one(&self.pool).await?;
        Ok(OrgFinancialInfo {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            bank_name: row.try_get("bank_name")?,
            iban_masked: row.try_get("iban_masked")?,
            bic: row.try_get("bic")?,
            payment_terms: row.try_get("payment_terms")?,
            currency: row.try_get("currency")?,
            billing_email: row.try_get("billing_email")?,
            billing_address: row.try_get("billing_address")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgContract {
    pub id: String,
    pub organization_id: String,
    pub contract_type: String,
    pub title: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
    pub document_reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgContractStore {
    pool: PgPool,
}
impl OrgContractStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgContract>, OrgFinanceError> {
        let rows = sqlx::query("SELECT id::text, organization_id, contract_type, title, signed_at, expires_at, status, document_reference, notes, created_at, updated_at FROM organization_contracts WHERE organization_id=$1 ORDER BY signed_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgContract {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    contract_type: r.try_get("contract_type")?,
                    title: r.try_get("title")?,
                    signed_at: r.try_get("signed_at")?,
                    expires_at: r.try_get("expires_at")?,
                    status: r.try_get("status")?,
                    document_reference: r.try_get("document_reference")?,
                    notes: r.try_get("notes")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        contract_type: &str,
        title: &str,
    ) -> Result<OrgContract, OrgFinanceError> {
        let row = sqlx::query("INSERT INTO organization_contracts (organization_id, contract_type, title) VALUES ($1,$2,$3) RETURNING id::text, organization_id, contract_type, title, signed_at, expires_at, status, document_reference, notes, created_at, updated_at")
            .bind(org_id).bind(contract_type).bind(title).fetch_one(&self.pool).await?;
        Ok(OrgContract {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            contract_type: row.try_get("contract_type")?,
            title: row.try_get("title")?,
            signed_at: row.try_get("signed_at")?,
            expires_at: row.try_get("expires_at")?,
            status: row.try_get("status")?,
            document_reference: row.try_get("document_reference")?,
            notes: row.try_get("notes")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgCompliance {
    pub id: String,
    pub organization_id: String,
    pub compliance_type: String,
    pub status: String,
    pub document_reference: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgComplianceStore {
    pool: PgPool,
}
impl OrgComplianceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgCompliance>, OrgFinanceError> {
        let rows = sqlx::query("SELECT id::text, organization_id, compliance_type, status, document_reference, expires_at, notes, created_at, updated_at FROM organization_compliance WHERE organization_id=$1 ORDER BY compliance_type")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgCompliance {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    compliance_type: r.try_get("compliance_type")?,
                    status: r.try_get("status")?,
                    document_reference: r.try_get("document_reference")?,
                    expires_at: r.try_get("expires_at")?,
                    notes: r.try_get("notes")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgService {
    pub id: String,
    pub organization_id: String,
    pub service_name: String,
    pub description: Option<String>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgServiceStore {
    pool: PgPool,
}
impl OrgServiceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgService>, OrgFinanceError> {
        let rows = sqlx::query("SELECT id::text, organization_id, service_name, description, status, started_at, created_at, updated_at FROM organization_services WHERE organization_id=$1 ORDER BY service_name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgService {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    service_name: r.try_get("service_name")?,
                    description: r.try_get("description")?,
                    status: r.try_get("status")?,
                    started_at: r.try_get("started_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgProduct {
    pub id: String,
    pub organization_id: String,
    pub product_name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgProductStore {
    pool: PgPool,
}
impl OrgProductStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgProduct>, OrgFinanceError> {
        let rows = sqlx::query("SELECT id::text, organization_id, product_name, description, status, created_at, updated_at FROM organization_products WHERE organization_id=$1 ORDER BY product_name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgProduct {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    product_name: r.try_get("product_name")?,
                    description: r.try_get("description")?,
                    status: r.try_get("status")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum OrgFinanceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/domains/organizations/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/health.rs`
- Size bytes / Размер в байтах: `8846`
- Included characters / Включено символов: `8846`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;
use thiserror::Error;

use crate::domains::organizations::core::{OrgCoreError, link_entity_in_transaction};
use crate::platform::observations::ObservationStoreError;

#[derive(Clone, Debug, Serialize)]
pub struct OrgHealth {
    pub organization_id: String,
    pub display_name: String,
    pub health_status: String,
    pub last_health_check: Option<DateTime<Utc>>,
    pub watchlist: bool,
    pub interaction_count: i32,
    pub trust_score: Option<i16>,
    pub open_risks: i64,
    pub overdue_contracts: i64,
}

#[derive(Clone)]
pub struct OrgHealthStore {
    pool: PgPool,
}
impl OrgHealthStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn get(&self, org_id: &str) -> Result<Option<OrgHealth>, OrgHealthError> {
        let row = sqlx::query("SELECT o.organization_id, o.display_name, o.health_status, o.last_health_check, o.watchlist, o.interaction_count, o.trust_score, (SELECT count(*) FROM organization_risks r WHERE r.organization_id=o.organization_id AND r.resolved_at IS NULL) as open_risks, (SELECT count(*) FROM organization_contracts c WHERE c.organization_id=o.organization_id AND c.expires_at < now() AND c.status='active') as overdue_contracts FROM organizations o WHERE o.organization_id=$1")
            .bind(org_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(OrgHealth {
                organization_id: r.try_get("organization_id").unwrap_or_default(),
                display_name: r.try_get("display_name").unwrap_or_default(),
                health_status: r
                    .try_get("health_status")
                    .unwrap_or_else(|_| "healthy".into()),
                last_health_check: r.try_get("last_health_check").ok(),
                watchlist: r.try_get("watchlist").unwrap_or(false),
                interaction_count: r.try_get("interaction_count").unwrap_or(0),
                trust_score: r.try_get("trust_score").ok(),
                open_risks: r.try_get("open_risks").unwrap_or(0),
                overdue_contracts: r.try_get("overdue_contracts").unwrap_or(0),
            })
        })
        .transpose()
    }
    pub async fn list_unhealthy(&self) -> Result<Vec<OrgHealth>, OrgHealthError> {
        let rows = sqlx::query("SELECT organization_id, display_name, health_status, last_health_check, watchlist, interaction_count, trust_score FROM organizations WHERE health_status IS NOT NULL AND health_status != 'healthy' ORDER BY interaction_count DESC LIMIT 50")
            .fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|r| OrgHealth {
                organization_id: r.try_get("organization_id").unwrap_or_default(),
                display_name: r.try_get("display_name").unwrap_or_default(),
                health_status: r
                    .try_get("health_status")
                    .unwrap_or_else(|_| "healthy".into()),
                last_health_check: r.try_get("last_health_check").ok(),
                watchlist: r.try_get("watchlist").unwrap_or(false),
                interaction_count: r.try_get("interaction_count").unwrap_or(0),
                trust_score: r.try_get("trust_score").ok(),
                open_risks: 0,
                overdue_contracts: 0,
            })
            .collect())
    }
    pub async fn toggle_watchlist(&self, org_id: &str) -> Result<bool, OrgHealthError> {
        self.toggle_watchlist_with_source(org_id, &organization_watchlist_source(org_id))
            .await
    }

    pub async fn toggle_watchlist_with_source(
        &self,
        org_id: &str,
        source: &str,
    ) -> Result<bool, OrgHealthError> {
        let mut transaction = self.pool.begin().await?;
        let watchlist =
            Self::toggle_watchlist_in_transaction(&mut transaction, org_id, source).await?;
        transaction.commit().await?;
        Ok(watchlist)
    }

    pub async fn toggle_watchlist_with_observation(
        &self,
        org_id: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<bool, OrgHealthError> {
        let mut transaction = self.pool.begin().await?;
        let watchlist =
            Self::toggle_watchlist_in_transaction(&mut transaction, org_id, source).await?;
        link_entity_in_transaction(
            &mut transaction,
            observation_id,
            "watchlist_toggle",
            org_id,
            json!({
                "watchlist": watchlist
            }),
        )
        .await?;
        transaction.commit().await?;
        Ok(watchlist)
    }

    async fn toggle_watchlist_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        org_id: &str,
        source: &str,
    ) -> Result<bool, OrgHealthError> {
        let row = sqlx::query("UPDATE organizations SET watchlist = NOT watchlist WHERE organization_id=$1 RETURNING watchlist").bind(org_id).fetch_optional(&mut **transaction).await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let watchlist = row.try_get("watchlist").unwrap_or(false);
        sqlx::query(
            "INSERT INTO organization_preferences (organization_id, preference_type, value, source)
             VALUES ($1, 'ui:watchlist', $2, $3)
             ON CONFLICT (organization_id, preference_type)
             DO UPDATE SET value = $2, source = $3, updated_at = now()",
        )
        .bind(org_id)
        .bind(if watchlist { "true" } else { "false" })
        .bind(source)
        .execute(&mut **transaction)
        .await?;
        Ok(watchlist)
    }
}

fn organization_watchlist_source(org_id: &str) -> String {
    format!("organizations.watchlist:{org_id}")
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgRisk {
    pub id: String,
    pub organization_id: String,
    pub risk_type: String,
    pub description: String,
    pub severity: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

#[derive(Clone)]
pub struct OrgRiskStore {
    pool: PgPool,
}
impl OrgRiskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgRisk>, OrgHealthError> {
        let rows = sqlx::query("SELECT id::text, organization_id, risk_type, description, severity, source, confidence::float8 AS confidence, created_at, resolved_at, resolution FROM organization_risks WHERE organization_id=$1 ORDER BY created_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgRisk {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    risk_type: r.try_get("risk_type")?,
                    description: r.try_get("description")?,
                    severity: r.try_get("severity")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    created_at: r.try_get("created_at")?,
                    resolved_at: r.try_get("resolved_at")?,
                    resolution: r.try_get("resolution")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        risk_type: &str,
        desc: &str,
        severity: &str,
        source: &str,
    ) -> Result<OrgRisk, OrgHealthError> {
        let row = sqlx::query("INSERT INTO organization_risks (organization_id, risk_type, description, severity, source) VALUES ($1,$2,$3,$4,$5) RETURNING id::text, organization_id, risk_type, description, severity, source, confidence::float8 AS confidence, created_at, resolved_at, resolution")
            .bind(org_id).bind(risk_type).bind(desc).bind(severity).bind(source).fetch_one(&self.pool).await?;
        Ok(OrgRisk {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            risk_type: row.try_get("risk_type")?,
            description: row.try_get("description")?,
            severity: row.try_get("severity")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
            resolved_at: row.try_get("resolved_at")?,
            resolution: row.try_get("resolution")?,
        })
    }
}

#[derive(Debug, Error)]
pub enum OrgHealthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Core(#[from] OrgCoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
```

### `backend/src/domains/organizations/investigator.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/investigator.rs`
- Size bytes / Размер в байтах: `4764`
- Included characters / Включено символов: `4764`
- Truncated / Обрезано: `no`

```rust
use crate::domains::organizations::api::OrganizationError;
use crate::domains::organizations::core::OrgCoreError;
use crate::platform::observations::ObservationStoreError;
use serde::Serialize;
use serde_json::Value;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct OrgDossier {
    pub organization: Value,
    pub identities: Vec<Value>,
    pub domains: Vec<Value>,
    pub contacts: Vec<Value>,
    pub facts: Vec<Value>,
    pub memory_cards: Vec<Value>,
    pub timeline: Vec<Value>,
    pub contracts: Vec<Value>,
    pub risks: Vec<Value>,
    pub portals: Vec<Value>,
    pub procedures: Vec<Value>,
    pub enrichment: Vec<Value>,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgBrief {
    pub organization_id: String,
    pub display_name: String,
    pub org_type: Option<String>,
    pub last_interaction_days: Option<i64>,
    pub open_risks: i64,
    pub active_contracts: i64,
    pub primary_contact: Option<String>,
    pub language: Option<String>,
    pub next_deadline: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgContextPack {
    pub brief: OrgBrief,
    pub recent_events: Vec<Value>,
    pub key_contacts: Vec<Value>,
    pub active_contracts: Vec<Value>,
    pub open_risks: Vec<Value>,
    pub portals: Vec<Value>,
    pub procedures: Vec<Value>,
}

#[derive(Clone)]
pub struct OrganizationInvestigator {
    pool: PgPool,
}

impl OrganizationInvestigator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn dossier(&self, org_id: &str) -> Result<OrgDossier, InvestigatorError> {
        use crate::domains::organizations::api::OrganizationStore;
        let org = OrganizationStore::new(self.pool.clone());
        let org_data = org.get(org_id).await?.ok_or(InvestigatorError::NotFound)?;
        let org_json = serde_json::to_value(&org_data).unwrap_or_default();

        let mut parts = Vec::new();
        if let Some(t) = &org_data.org_type {
            parts.push(format!("Type: {t}"));
        }
        if org_data.interaction_count > 0 {
            parts.push(format!("{} interactions", org_data.interaction_count));
        }

        Ok(OrgDossier {
            organization: org_json,
            identities: vec![],
            domains: vec![],
            contacts: vec![],
            facts: vec![],
            memory_cards: vec![],
            timeline: vec![],
            contracts: vec![],
            risks: vec![],
            portals: vec![],
            procedures: vec![],
            enrichment: vec![],
            summary: parts.join(" | "),
        })
    }

    pub async fn brief(&self, org_id: &str) -> Result<OrgBrief, InvestigatorError> {
        use crate::domains::organizations::api::OrganizationStore;
        let org = OrganizationStore::new(self.pool.clone());
        let org_data = org.get(org_id).await?.ok_or(InvestigatorError::NotFound)?;
        let last_days = org_data
            .last_interaction_at
            .map(|dt| (chrono::Utc::now() - dt).num_days());
        Ok(OrgBrief {
            organization_id: org_data.organization_id,
            display_name: org_data.display_name,
            org_type: org_data.org_type,
            last_interaction_days: last_days,
            open_risks: 0,
            active_contracts: 0,
            primary_contact: None,
            language: org_data.primary_language,
            next_deadline: None,
        })
    }

    pub async fn context_pack(&self, org_id: &str) -> Result<OrgContextPack, InvestigatorError> {
        let brief = self.brief(org_id).await?;
        Ok(OrgContextPack {
            brief,
            recent_events: vec![],
            key_contacts: vec![],
            active_contracts: vec![],
            open_risks: vec![],
            portals: vec![],
            procedures: vec![],
        })
    }
}

impl From<OrganizationError> for InvestigatorError {
    fn from(e: OrganizationError) -> Self {
        match e {
            OrganizationError::NotFound => InvestigatorError::NotFound,
            OrganizationError::Validation(message) => InvestigatorError::Validation(message),
            OrganizationError::Sqlx(e) => InvestigatorError::Sqlx(e),
            OrganizationError::Core(e) => InvestigatorError::Core(e),
            OrganizationError::Observation(e) => InvestigatorError::Observation(e),
        }
    }
}

#[derive(Debug, Error)]
pub enum InvestigatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Core(#[from] OrgCoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("{0}")]
    Validation(String),
    #[error("organization not found")]
    NotFound,
}
```
