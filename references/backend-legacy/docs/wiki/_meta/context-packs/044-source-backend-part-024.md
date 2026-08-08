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

- Chunk ID / ID чанка: `044-source-backend-part-024`
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

### `backend/src/domains/decisions/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/service.rs`
- Size bytes / Размер в байтах: `2136`
- Included characters / Включено символов: `2136`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{Decision, DecisionReviewState, DecisionStore, DecisionStoreError};

#[derive(Clone)]
pub struct DecisionCommandService {
    pool: PgPool,
}

impl DecisionCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionCommandServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "decision_id": decision_id,
                        "review_state": review_state.as_str(),
                        "operation": "decision_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("decision://{decision_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "decisions_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let decision = DecisionStore::new(self.pool.clone())
            .set_review_state_with_observation(
                decision_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "decisions_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(decision)
    }
}

#[derive(Debug, Error)]
pub enum DecisionCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Decision(#[from] DecisionStoreError),
}
```

### `backend/src/domains/decisions/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/store.rs`
- Size bytes / Размер в байтах: `12139`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashSet;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use super::errors::DecisionStoreError;
use super::evidence::{
    link_decision_review_transition_in_transaction, link_decision_support_in_transaction,
};
use super::ids::{decision_id, evidence_id};
use super::models::{
    Decision, DecisionEntityKind, DecisionReviewState, NewDecision, NewDecisionEvidence,
    NewDecisionImpactedEntity,
};
use super::row_mapping::row_to_decision;
use super::validation::{validate_decision_with_evidence, validate_non_empty};

#[derive(Clone)]
pub struct DecisionStore {
    pub(super) pool: PgPool,
}

impl DecisionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        decision: &NewDecision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<Decision, DecisionStoreError> {
        validate_decision_with_evidence(decision, evidence, impacted_entities)?;

        let mut transaction = self.pool.begin().await?;
        let stored = Self::upsert_with_evidence_in_transaction(
            &mut transaction,
            decision,
            evidence,
            impacted_entities,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        decision: &NewDecision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<Decision, DecisionStoreError> {
        validate_evidence_observations_exist(transaction, evidence).await?;
        let decision_id = decision_id(decision);
        let row = sqlx::query(
            r#"
            INSERT INTO decisions (
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
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
                CAST($10 AS NUMERIC(5,4)),
                $11
            )
            ON CONFLICT (decision_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                status = EXCLUDED.status,
                rationale = EXCLUDED.rationale,
                alternatives = EXCLUDED.alternatives,
                decided_by_entity_kind = EXCLUDED.decided_by_entity_kind,
                decided_by_entity_id = EXCLUDED.decided_by_entity_id,
                decided_at = EXCLUDED.decided_at,
                review_state = EXCLUDED.review_state,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&decision_id)
        .bind(&decision.title)
        .bind(decision.status.as_str())
        .bind(&decision.rationale)
        .bind(&decision.alternatives)
        .bind(decision.decided_by_entity_kind.map(|kind| kind.as_str()))
        .bind(&decision.decided_by_entity_id)
        .bind(decision.decided_at)
        .bind(decision.review_state.as_str())
        .bind(decision.confidence)
        .bind(&decision.metadata)
        .fetch_one(&mut **transaction)
        .await?;

        let stored = row_to_decision(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&decision_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO decision_evidence (
                    evidence_id,
                    decision_id,
                    source_kind,
                    source_id,
                    observation_id,
                    quote,
                    confidence,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS NUMERIC(5,4)), $8)
                ON CONFLICT (decision_id, source_kind, source_id)
                DO UPDATE SET
                    observation_id = EXCLUDED.observation_id,
                    quote = EXCLUDED.quote,
                    confidence = EXCLUDED.confidence,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&decision_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(item.observation_id.as_deref())
            .bind(&item.quote)
            .bind(item.confidence)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;

            if let Some(observation_id) = item.observation_id.as_deref() {
                link_decision_support_in_transaction(
                    transaction,
                    observation_id,
                    decision_id.clone(),
                    item.confidence,
                    json!({
                        "source_kind": item.source_kind.as_str(),
                        "source_id": item.source_id,
                    }),
                )
                .await?;
            }
        }

        for item in impacted_entities {
            sqlx::query(
                r#"
                INSERT INTO decision_impacted_entities (
                    decision_id,
                    entity_kind,
                    entity_id,
                    impact_type,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (decision_id, entity_kind, entity_id)
                DO UPDATE SET
                    impact_type = EXCLUDED.impact_type,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(&decision_id)
            .bind(item.entity_kind.as_str())
            .bind(&item.entity_id)
            .bind(&item.impact_type)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        Ok(stored)
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: DecisionEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Decision>, DecisionStoreError> {
        validate_non_empty("entity_id", entity_id)?;
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                decision.decision_id,
                decision.title,
                decision.status,
                decision.rationale,
                decision.alternatives,
                decision.decided_by_entity_kind,
                decision.decided_by_entity_id,
                decision.decided_at,
                decision.review_state,
                decision.confidence::float8 AS confidence,
                decision.metadata,
                decision.created_at,
                decision.updated_at
            FROM decisions decision
            LEFT JOIN decision_impacted_entities impacted
              ON impacted.decision_id = decision.decision_id
            WHERE (decision.decided_by_entity_kind = $1 AND decision.decided_by_entity_id = $2)
               OR (impacted.entity_kind = $1 AND impacted.entity_id = $2)
            ORDER BY decision.updated_at DESC, decision.decision_id ASC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_decision).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: DecisionReviewState,
        limit: i64,
    ) -> Result<Vec<Decision>, DecisionStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM decisions
            WHERE review_state = $1
            ORDER BY updated_at DESC, decision_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_decision).collect()
    }

    pub async fn set_review_state(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionStoreError> {
        self.set_review_state_with_observation(decision_id, review_state, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Decision, DecisionStoreError> {
        validate_non_empty("decision_id", decision_id)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE decisions
            SET
                review_state = $1,
                updated_at = now()
            WHERE decision_id = $2
            RETURNING
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(decision_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(DecisionStoreError::DecisionNotFound)?;

        let decision = row_to_decision(row)?;
        link_decision_review_transition_in_transaction(
            &mut transaction,
            observation_id,
            &decision.decision_id,
            decision.review_state,
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(decision)
    }
}

async fn validate_evidence_observations_exist(
    transaction: &mut Transaction<'_, Postgres>,
    evidence: &[NewDecisionEvidence],
) -> Result<(), DecisionStoreError> {
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
        if !stored_observation_ids.
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/decisions/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/validation.rs`
- Size bytes / Размер в байтах: `2649`
- Included characters / Включено символов: `2649`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::postgres::PgPool;

use super::constants::{MAX_REFRESH_LIMIT, MIN_REFRESH_LIMIT};
use super::errors::DecisionStoreError;
use super::ids::decision_id;
use super::models::{
    DecisionReviewState, NewDecision, NewDecisionEvidence, NewDecisionImpactedEntity,
};

pub(super) fn validate_decision_with_evidence(
    decision: &NewDecision,
    evidence: &[NewDecisionEvidence],
    impacted_entities: &[NewDecisionImpactedEntity],
) -> Result<(), DecisionStoreError> {
    decision.validate()?;
    if evidence.is_empty() {
        return Err(DecisionStoreError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }
    for item in impacted_entities {
        item.validate()?;
    }

    Ok(())
}

pub(super) async fn preserve_existing_review_state(
    pool: &PgPool,
    decision: &mut NewDecision,
) -> Result<(), DecisionStoreError> {
    let existing_review_state: Option<String> =
        sqlx::query_scalar("SELECT review_state FROM decisions WHERE decision_id = $1")
            .bind(decision_id(decision))
            .fetch_optional(pool)
            .await?;
    let Some(existing_review_state) = existing_review_state else {
        return Ok(());
    };
    let existing_review_state = DecisionReviewState::parse(existing_review_state)?;
    if existing_review_state != DecisionReviewState::Suggested {
        decision.review_state = existing_review_state;
    }

    Ok(())
}

pub(super) fn validate_refresh_limit(limit: i64) -> Result<i64, DecisionStoreError> {
    if !(MIN_REFRESH_LIMIT..=MAX_REFRESH_LIMIT).contains(&limit) {
        return Err(DecisionStoreError::InvalidLimit);
    }

    Ok(limit)
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), DecisionStoreError> {
    if value.trim().is_empty() {
        return Err(DecisionStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_score(
    field_name: &'static str,
    value: f64,
) -> Result<(), DecisionStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(DecisionStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), DecisionStoreError> {
    if !value.is_object() {
        return Err(DecisionStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_array(
    field_name: &'static str,
    value: &Value,
) -> Result<(), DecisionStoreError> {
    if !value.is_array() {
        return Err(DecisionStoreError::InvalidJsonArray(field_name));
    }

    Ok(())
}
```

### `backend/src/domains/documents/attachment_intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/attachment_intelligence.rs`
- Size bytes / Размер в байтах: `1641`
- Included characters / Включено символов: `1641`
- Truncated / Обрезано: `no`

```rust
mod classification;
mod file_kinds;
mod models;

#[cfg(test)]
mod tests;

use classification::classify_by_name_and_type;
use file_kinds::{is_archive_type, is_document_type, is_executable_type};

pub use models::{
    AttachmentCategory, AttachmentClassification, AttachmentIntelligenceError,
    AttachmentIntelligenceInput, RiskLevel,
};

pub struct AttachmentIntelligenceService;

impl AttachmentIntelligenceService {
    /// Classify an attachment by filename and content type.
    pub fn classify(attachment: &AttachmentIntelligenceInput) -> AttachmentClassification {
        let filename = attachment.filename.as_deref().unwrap_or("");
        let content_type = &attachment.content_type;
        let filename_lower = filename.to_lowercase();

        let category = classify_by_name_and_type(&filename_lower, content_type);
        let is_executable = is_executable_type(content_type, &filename_lower);
        let is_archive = is_archive_type(content_type, &filename_lower);
        let is_document = is_document_type(content_type, &filename_lower);
        let risk_level = if is_executable {
            RiskLevel::High
        } else if is_archive {
            RiskLevel::Medium
        } else {
            RiskLevel::Safe
        };

        let size_mb = attachment.size_bytes as f64 / 1_048_576.0;

        AttachmentClassification {
            attachment_id: attachment.attachment_id.clone(),
            category,
            is_executable,
            is_archive,
            is_document,
            risk_level,
            summary: format!("{} ({:.1} MB) - {}", filename, size_mb, category.as_str()),
        }
    }
}
```

### `backend/src/domains/documents/attachment_intelligence/classification.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/attachment_intelligence/classification.rs`
- Size bytes / Размер в байтах: `2141`
- Included characters / Включено символов: `2141`
- Truncated / Обрезано: `no`

```rust
use super::models::AttachmentCategory;

pub(super) fn classify_by_name_and_type(filename: &str, content_type: &str) -> AttachmentCategory {
    let lower = filename.to_lowercase();

    if lower.contains("invoice") || lower.contains("factura") || lower.contains("receipt") {
        return AttachmentCategory::Invoice;
    }
    if lower.contains("contract") || lower.contains("agreement") || lower.contains("nda") {
        return AttachmentCategory::Contract;
    }
    if lower.contains("certificate") || lower.contains("cert") {
        return AttachmentCategory::Certificate;
    }
    if lower.contains("tax") || lower.contains("hacienda") || lower.contains("aeat") {
        return AttachmentCategory::TaxDocument;
    }
    if lower.contains("passport") || lower.contains("dni") || lower.contains("nie") {
        return AttachmentCategory::IdentityDocument;
    }
    if lower.contains("report") {
        return AttachmentCategory::Report;
    }
    if lower.contains("presentation") || lower.ends_with(".pptx") || lower.ends_with(".ppt") {
        return AttachmentCategory::Presentation;
    }
    if lower.ends_with(".xlsx") || lower.ends_with(".xls") || lower.ends_with(".csv") {
        return AttachmentCategory::Spreadsheet;
    }
    if is_source_code_filename(&lower) {
        return AttachmentCategory::SourceCode;
    }
    if is_archive_filename(&lower) {
        return AttachmentCategory::Archive;
    }
    if content_type.starts_with("image/") {
        if lower.contains("screenshot") || lower.contains("screen") {
            return AttachmentCategory::Screenshot;
        }
        return AttachmentCategory::Image;
    }
    if content_type == "application/pdf" {
        return AttachmentCategory::Report;
    }

    AttachmentCategory::Unknown
}

fn is_source_code_filename(filename: &str) -> bool {
    [".rs", ".py", ".js", ".ts", ".go", ".java", ".c", ".cpp"]
        .iter()
        .any(|extension| filename.ends_with(extension))
}

fn is_archive_filename(filename: &str) -> bool {
    [".zip", ".rar", ".7z", ".tar.gz", ".tar"]
        .iter()
        .any(|extension| filename.ends_with(extension))
}
```

### `backend/src/domains/documents/attachment_intelligence/file_kinds.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/attachment_intelligence/file_kinds.rs`
- Size bytes / Размер в байтах: `1758`
- Included characters / Включено символов: `1758`
- Truncated / Обрезано: `no`

```rust
pub(super) fn is_executable_type(content_type: &str, filename: &str) -> bool {
    let executable_types = [
        "application/x-msdownload",
        "application/x-executable",
        "application/x-mach-binary",
        "application/x-sh",
        "application/x-bat",
        "application/x-msi",
    ];
    let executable_exts = [
        ".exe", ".dll", ".sh", ".bat", ".cmd", ".msi", ".app", ".bin",
    ];
    executable_types.contains(&content_type)
        || executable_exts
            .iter()
            .any(|extension| filename.ends_with(extension))
}

pub(super) fn is_archive_type(content_type: &str, filename: &str) -> bool {
    let archive_types = [
        "application/zip",
        "application/x-rar-compressed",
        "application/x-7z-compressed",
        "application/x-tar",
        "application/gzip",
        "application/x-bzip2",
    ];
    let archive_exts = [
        ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz", ".tar.gz",
    ];
    archive_types.contains(&content_type)
        || archive_exts
            .iter()
            .any(|extension| filename.ends_with(extension))
}

pub(super) fn is_document_type(content_type: &str, filename: &str) -> bool {
    let doc_types = [
        "application/pdf",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "text/plain",
        "text/markdown",
        "text/csv",
    ];
    let doc_exts = [
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".txt", ".md", ".csv",
    ];
    doc_types.contains(&content_type)
        || doc_exts
            .iter()
            .any(|extension| filename.ends_with(extension))
}
```

### `backend/src/domains/documents/attachment_intelligence/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/attachment_intelligence/models.rs`
- Size bytes / Размер в байтах: `2255`
- Included characters / Включено символов: `2255`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentIntelligenceInput {
    pub attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentClassification {
    pub attachment_id: String,
    pub category: AttachmentCategory,
    pub is_executable: bool,
    pub is_archive: bool,
    pub is_document: bool,
    pub risk_level: RiskLevel,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentCategory {
    Invoice,
    Contract,
    LegalDocument,
    TaxDocument,
    IdentityDocument,
    BankDocument,
    Certificate,
    Report,
    Presentation,
    Spreadsheet,
    SourceCode,
    Image,
    Screenshot,
    Archive,
    Unknown,
}

impl AttachmentCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Invoice => "invoice",
            Self::Contract => "contract",
            Self::LegalDocument => "legal_document",
            Self::TaxDocument => "tax_document",
            Self::IdentityDocument => "identity_document",
            Self::BankDocument => "bank_document",
            Self::Certificate => "certificate",
            Self::Report => "report",
            Self::Presentation => "presentation",
            Self::Spreadsheet => "spreadsheet",
            Self::SourceCode => "source_code",
            Self::Image => "image",
            Self::Screenshot => "screenshot",
            Self::Archive => "archive",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Error)]
pub enum AttachmentIntelligenceError {
    #[error("attachment not found")]
    NotFound,
}
```

### `backend/src/domains/documents/attachment_intelligence/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/attachment_intelligence/tests.rs`
- Size bytes / Размер в байтах: `2323`
- Included characters / Включено символов: `2323`
- Truncated / Обрезано: `no`

```rust
use super::file_kinds::{is_archive_type, is_executable_type};
use super::*;

fn test_attachment(filename: &str, content_type: &str, size: i64) -> AttachmentIntelligenceInput {
    AttachmentIntelligenceInput {
        attachment_id: "att:1".into(),
        filename: Some(filename.into()),
        content_type: content_type.into(),
        size_bytes: size,
    }
}

#[test]
fn classify_invoice_by_filename() {
    let att = test_attachment("Invoice_2026_001.pdf", "application/pdf", 100_000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.category.as_str(), "invoice");
}

#[test]
fn classify_contract_by_filename() {
    let att = test_attachment(
        "NDA_Acme_Corp.docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        50_000,
    );
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.category.as_str(), "contract");
}

#[test]
fn classify_archive_by_extension() {
    let att = test_attachment("documents.zip", "application/zip", 1_000_000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.category.as_str(), "archive");
    assert_eq!(result.risk_level.as_str(), "medium");
}

#[test]
fn classify_executable_as_high_risk() {
    let att = test_attachment("setup.exe", "application/x-msdownload", 5_000_000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert!(result.is_executable);
    assert_eq!(result.risk_level.as_str(), "high");
}

#[test]
fn classify_image_as_safe() {
    let att = test_attachment("photo.jpg", "image/jpeg", 200_000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.risk_level.as_str(), "safe");
}

#[test]
fn classify_source_code() {
    let att = test_attachment("main.rs", "text/plain", 5000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.category.as_str(), "source_code");
}

#[test]
fn is_executable_detects_exe() {
    assert!(is_executable_type("application/x-msdownload", "setup.exe"));
    assert!(!is_executable_type("application/pdf", "doc.pdf"));
}

#[test]
fn is_archive_detects_zip() {
    assert!(is_archive_type("application/zip", "archive.zip"));
    assert!(!is_archive_type("application/pdf", "doc.pdf"));
}
```

### `backend/src/domains/documents/core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core.rs`
- Size bytes / Размер в байтах: `424`
- Included characters / Включено символов: `424`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod evidence;
mod fingerprint;
mod markdown;
mod models;
mod rows;
mod store;
mod validation;

pub use errors::{DocumentImportError, DocumentImportWithProcessingError};
pub(crate) use evidence::link_document_entity_in_transaction;
pub use models::{ImportedDocument, ImportedDocumentWithProcessing, NewDocumentImport};
pub use store::DocumentImportStore;
pub use store::DocumentImportStore as DocumentImportPort;
```

### `backend/src/domains/documents/core/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core/errors.rs`
- Size bytes / Размер в байтах: `1031`
- Included characters / Включено символов: `1031`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocumentImportError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ObservationStore(#[from] crate::platform::observations::ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("document_kind must be markdown or pdf: {0}")]
    InvalidDocumentKind(String),

    #[error(
        "document_kind change rejected for document_id={document_id}: existing={existing_kind}, new={new_kind}"
    )]
    DocumentKindChange {
        document_id: String,
        existing_kind: String,
        new_kind: String,
    },

    #[error("document import upsert skipped unexpectedly for document_id={0}")]
    UpsertSkipped(String),
}

#[derive(Debug, Error)]
pub enum DocumentImportWithProcessingError {
    #[error(transparent)]
    DocumentImport(#[from] DocumentImportError),

    #[error(transparent)]
    Processing(#[from] crate::domains::documents::processing::DocumentProcessingError),
}
```

### `backend/src/domains/documents/core/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core/evidence.rs`
- Size bytes / Размер в байтах: `679`
- Included characters / Включено символов: `679`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::{ObservationStoreError, link_domain_entity_in_transaction};

pub(crate) async fn link_document_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    document_id: impl Into<String>,
    relationship_kind: Option<&str>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "documents",
        "document",
        document_id.into(),
        relationship_kind,
        None,
        metadata,
    )
    .await
}
```

### `backend/src/domains/documents/core/fingerprint.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core/fingerprint.rs`
- Size bytes / Размер в байтах: `511`
- Included characters / Включено символов: `511`
- Truncated / Обрезано: `no`

```rust
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

// V1 local boundary fingerprint only. This is deterministic for idempotence but
// is not cryptographic evidence of source content.
pub(super) fn local_markdown_fingerprint(extracted_text: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in extracted_text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("local-v1:markdown:{hash:016x}")
}
```

### `backend/src/domains/documents/core/markdown.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core/markdown.rs`
- Size bytes / Размер в байтах: `777`
- Included characters / Включено символов: `777`
- Truncated / Обрезано: `no`

```rust
pub(super) fn extract_markdown_text(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| match markdown_heading_text(line.trim_end()) {
            Some(heading_text) => heading_text,
            None => line.trim_end(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn markdown_heading_text(line: &str) -> Option<&str> {
    let mut hash_count = 0;
    for character in line.chars() {
        if character == '#' {
            hash_count += 1;
            continue;
        }
        break;
    }

    if !(1..=6).contains(&hash_count) {
        return None;
    }

    line.as_bytes()
        .get(hash_count)
        .filter(|byte| **byte == b' ')
        .map(|_| &line[hash_count + 1..])
}
```

### `backend/src/domains/documents/core/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core/models.rs`
- Size bytes / Размер в байтах: `2446`
- Included characters / Включено символов: `2446`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::errors::DocumentImportError;
use super::fingerprint::local_markdown_fingerprint;
use super::markdown::extract_markdown_text;
use super::validation::{ValidatedDocumentImport, validate_document_import};
use crate::domains::documents::processing::DocumentProcessingJob;

pub(super) const DOCUMENT_KIND_MARKDOWN: &str = "markdown";
pub(super) const DOCUMENT_KIND_PDF: &str = "pdf";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewDocumentImport {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub source_fingerprint: String,
    pub extracted_text: String,
}

impl NewDocumentImport {
    /// Creates a Markdown import with extracted text and a deterministic V1
    /// local fingerprint of that extracted text. The fingerprint is for local
    /// idempotence only and is not cryptographic evidence of source content.
    pub fn markdown(
        document_id: impl Into<String>,
        title: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        let extracted_text = extract_markdown_text(&markdown.into());
        let source_fingerprint = local_markdown_fingerprint(&extracted_text);

        Self {
            document_id: document_id.into(),
            document_kind: DOCUMENT_KIND_MARKDOWN.to_owned(),
            title: title.into(),
            source_fingerprint,
            extracted_text,
        }
    }

    pub fn pdf_metadata(
        document_id: impl Into<String>,
        title: impl Into<String>,
        source_fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            document_kind: DOCUMENT_KIND_PDF.to_owned(),
            title: title.into(),
            source_fingerprint: source_fingerprint.into(),
            extracted_text: String::new(),
        }
    }

    pub(super) fn validate(&self) -> Result<ValidatedDocumentImport, DocumentImportError> {
        validate_document_import(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct ImportedDocumentWithProcessing {
    pub imported: ImportedDocument,
    pub jobs: Vec<DocumentProcessingJob>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedDocument {
    pub document_id: String,
    pub document_kind: String,
    pub observation_id: String,
    pub title: String,
    pub source_fingerprint: String,
    pub extracted_text: String,
    pub imported_at: DateTime<Utc>,
}
```

### `backend/src/domains/documents/core/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core/rows.rs`
- Size bytes / Размер в байтах: `633`
- Included characters / Включено символов: `633`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::DocumentImportError;
use super::models::ImportedDocument;

pub(super) fn row_to_imported_document(
    row: PgRow,
) -> Result<ImportedDocument, DocumentImportError> {
    Ok(ImportedDocument {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        observation_id: row.try_get("observation_id")?,
        title: row.try_get("title")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        extracted_text: row.try_get("extracted_text")?,
        imported_at: row.try_get("imported_at")?,
    })
}
```

### `backend/src/domains/documents/core/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core/store.rs`
- Size bytes / Размер в байтах: `9067`
- Included characters / Включено символов: `9067`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::{DocumentImportError, DocumentImportWithProcessingError};
use super::link_document_entity_in_transaction;
use super::models::{ImportedDocument, ImportedDocumentWithProcessing, NewDocumentImport};
use super::rows::row_to_imported_document;
use crate::domains::documents::processing::DocumentProcessingStore;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};
use chrono::Utc;

#[derive(Clone)]
pub struct DocumentImportStore {
    pool: PgPool,
}

impl DocumentImportStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn import_document_and_enqueue_processing(
        &self,
        document: &NewDocumentImport,
        processing_store: &DocumentProcessingStore,
    ) -> Result<ImportedDocumentWithProcessing, DocumentImportWithProcessingError> {
        let imported = self.import_document(document).await?;
        let jobs = processing_store
            .enqueue_for_document(&imported.document_id)
            .await?;

        Ok(ImportedDocumentWithProcessing { imported, jobs })
    }

    pub async fn import_document(
        &self,
        document: &NewDocumentImport,
    ) -> Result<ImportedDocument, DocumentImportError> {
        document.validate()?;
        let mut transaction = self.pool.begin().await?;
        let imported = Self::import_document_in_transaction(&mut transaction, document).await?;
        transaction.commit().await?;
        Ok(imported)
    }

    pub(crate) async fn import_document_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
    ) -> Result<ImportedDocument, DocumentImportError> {
        Self::import_document_with_origin_in_transaction(
            transaction,
            document,
            ObservationOriginKind::FileImport,
            format!("document://{}", document.document_id),
            json!({
                "ingested_by": "documents_domain"
            }),
            None,
            None,
            None,
        )
        .await
    }

    pub(crate) async fn import_document_manual_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
        source_ref: String,
        provenance: serde_json::Value,
    ) -> Result<ImportedDocument, DocumentImportError> {
        Self::import_document_manual_with_observation_in_transaction(
            transaction,
            document,
            source_ref,
            provenance,
            None,
            None,
            None,
        )
        .await
    }

    pub(crate) async fn import_document_manual_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
        source_ref: String,
        provenance: serde_json::Value,
        source_observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ImportedDocument, DocumentImportError> {
        Self::import_document_with_origin_in_transaction(
            transaction,
            document,
            ObservationOriginKind::Manual,
            source_ref,
            provenance,
            source_observation_id,
            relationship_kind,
            metadata,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn import_document_with_origin_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
        origin_kind: ObservationOriginKind,
        source_ref: String,
        provenance: serde_json::Value,
        source_observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ImportedDocument, DocumentImportError> {
        let document = document.validate()?;
        let observation = NewObservation::new(
            "DOCUMENT",
            origin_kind,
            Utc::now(),
            json!({
                "document_id": document.document_id,
                "document_kind": document.document_kind,
                "title": document.title,
                "source_fingerprint": document.source_fingerprint,
                "extracted_text": document.extracted_text,
            }),
            source_ref,
        )
        .provenance(provenance);

        let observation = ObservationStore::capture_in_transaction(transaction, &observation)
            .await
            .map_err(DocumentImportError::from)?;

        let row = sqlx::query(
            r#"
            INSERT INTO documents (
                document_id,
                document_kind,
                observation_id,
                title,
                source_fingerprint,
                extracted_text
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (document_id)
            DO UPDATE SET
                observation_id = EXCLUDED.observation_id,
                title = EXCLUDED.title,
                source_fingerprint = EXCLUDED.source_fingerprint,
                extracted_text = EXCLUDED.extracted_text
            WHERE documents.document_kind = EXCLUDED.document_kind
            RETURNING
                document_id,
                document_kind,
                observation_id,
                title,
                source_fingerprint,
                extracted_text,
                imported_at
            "#,
        )
        .bind(&document.document_id)
        .bind(&document.document_kind)
        .bind(&observation.observation_id)
        .bind(&document.title)
        .bind(&document.source_fingerprint)
        .bind(&document.extracted_text)
        .fetch_optional(&mut **transaction)
        .await?;

        if let Some(row) = row {
            let imported = row_to_imported_document(row)?;
            link_document_entity_in_transaction(
                transaction,
                &imported.observation_id,
                imported.document_id.clone(),
                Some("import"),
                Some(json!({
                    "document_kind": imported.document_kind,
                    "source_fingerprint": imported.source_fingerprint,
                })),
            )
            .await
            .map_err(DocumentImportError::from)?;
            if let Some(source_observation_id) =
                source_observation_id.filter(|value| !value.trim().is_empty())
            {
                let metadata = match metadata {
                    Some(extra)
                        if json!({
                            "document_kind": imported.document_kind,
                            "source_document_observation_id": imported.observation_id,
                        })
                        .is_object()
                            && extra.is_object() =>
                    {
                        let mut merged = json!({
                            "document_kind": imported.document_kind,
                            "source_document_observation_id": imported.observation_id,
                        });
                        if let (Some(base), Some(extra)) =
                            (merged.as_object_mut(), extra.as_object())
                        {
                            for (key, value) in extra {
                                base.insert(key.clone(), value.clone());
                            }
                        }
                        merged
                    }
                    Some(extra) => extra,
                    None => json!({
                        "document_kind": imported.document_kind,
                        "source_document_observation_id": imported.observation_id,
                    }),
                };
                link_document_entity_in_transaction(
                    transaction,
                    source_observation_id,
                    imported.document_id.clone(),
                    Some(
                        relationship_kind
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or("workflow_action_projection"),
                    ),
                    Some(metadata),
                )
                .await
                .map_err(DocumentImportError::from)?;
            }
            return Ok(imported);
        }

        let existing_kind = sqlx::query_scalar::<_, String>(
            "SELECT document_kind FROM documents WHERE document_id = $1",
        )
        .bind(&document.document_id)
        .fetch_optional(&mut **transaction)
        .await?;

        match existing_kind {
            Some(existing_kind) => Err(DocumentImportError::DocumentKindChange {
                document_id: document.document_id,
                existing_kind,
                new_kind: document.document_kind,
            }),
            None => Err(DocumentImportError::UpsertSkipped(document.document_id)),
        }
    }
}
```

### `backend/src/domains/documents/core/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/core/validation.rs`
- Size bytes / Размер в байтах: `1891`
- Included characters / Включено символов: `1891`
- Truncated / Обрезано: `no`

```rust
use super::errors::DocumentImportError;
use super::models::{DOCUMENT_KIND_MARKDOWN, DOCUMENT_KIND_PDF, NewDocumentImport};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ValidatedDocumentImport {
    pub(super) document_id: String,
    pub(super) document_kind: String,
    pub(super) title: String,
    pub(super) source_fingerprint: String,
    pub(super) extracted_text: String,
}

pub(super) fn validate_document_import(
    document: &NewDocumentImport,
) -> Result<ValidatedDocumentImport, DocumentImportError> {
    let document_id = validate_non_empty("document_id", &document.document_id)?;
    let document_kind = validate_non_empty("document_kind", &document.document_kind)?;
    let title = validate_non_empty("title", &document.title)?;
    let source_fingerprint =
        validate_non_empty("source_fingerprint", &document.source_fingerprint)?;

    match document_kind.as_str() {
        DOCUMENT_KIND_MARKDOWN => {
            let extracted_text = document.extracted_text.trim_end().to_owned();
            validate_non_empty("extracted_text", &extracted_text)?;
            Ok(ValidatedDocumentImport {
                document_id,
                document_kind,
                title,
                source_fingerprint,
                extracted_text,
            })
        }
        DOCUMENT_KIND_PDF => Ok(ValidatedDocumentImport {
            document_id,
            document_kind,
            title,
            source_fingerprint,
            extracted_text: String::new(),
        }),
        _ => Err(DocumentImportError::InvalidDocumentKind(document_kind)),
    }
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<String, DocumentImportError> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        return Err(DocumentImportError::EmptyField(field_name));
    }

    Ok(normalized)
}
```

### `backend/src/domains/documents/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/mod.rs`
- Size bytes / Размер в байтах: `67`
- Included characters / Включено символов: `67`
- Truncated / Обрезано: `no`

```rust
pub mod attachment_intelligence;
pub mod core;
pub mod processing;
```

### `backend/src/domains/documents/processing.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing.rs`
- Size bytes / Размер в байтах: `630`
- Included characters / Включено символов: `630`
- Truncated / Обрезано: `no`

```rust
mod artifacts;
mod constants;
mod documents;
mod errors;
mod evidence;
mod ids;
mod jobs;
mod models;
mod retry;
mod rows;
mod runner;
mod service;
mod store;
mod validation;

pub use errors::DocumentProcessingError;
pub use models::{
    DocumentArtifactKind, DocumentProcessingArtifact, DocumentProcessingJob,
    DocumentProcessingRecord, DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult,
    DocumentProcessingRunReport, DocumentProcessingStatus, DocumentProcessingStep,
};
pub use service::{DocumentProcessingCommandService, DocumentProcessingCommandServiceError};
pub use store::DocumentProcessingStore;
```

### `backend/src/domains/documents/processing/artifacts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/artifacts.rs`
- Size bytes / Размер в байтах: `2026`
- Included characters / Включено символов: `2026`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Transaction};

use super::constants::ARTIFACT_METADATA_KIND;
use super::errors::DocumentProcessingError;
use super::ids::artifact_id;
use super::models::{DocumentArtifactKind, DocumentProcessingJob};
use super::store::DocumentProcessingStore;

impl DocumentProcessingStore {
    pub(super) async fn upsert_artifact(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: &DocumentProcessingJob,
        artifact_kind: DocumentArtifactKind,
        text_content: Option<String>,
    ) -> Result<(), DocumentProcessingError> {
        let artifact_id = artifact_id(&job.document_id, artifact_kind);
        let text = text_content.as_deref().unwrap_or("");
        let content_sha256 = content_sha256_hex(text);
        let metadata = json!({
            "source": ARTIFACT_METADATA_KIND,
            "artifact_kind": artifact_kind.as_str(),
        });

        sqlx::query(
            r#"
            INSERT INTO document_artifacts (
                artifact_id,
                document_id,
                job_id,
                artifact_kind,
                content_sha256,
                text_content,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (document_id, artifact_kind)
            DO UPDATE SET
                content_sha256 = EXCLUDED.content_sha256,
                text_content = EXCLUDED.text_content,
                metadata = EXCLUDED.metadata,
                job_id = EXCLUDED.job_id
            "#,
        )
        .bind(artifact_id)
        .bind(&job.document_id)
        .bind(&job.job_id)
        .bind(artifact_kind.as_str())
        .bind(content_sha256)
        .bind(text_content)
        .bind(metadata)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

fn content_sha256_hex(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}
```

### `backend/src/domains/documents/processing/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/constants.rs`
- Size bytes / Размер в байтах: `681`
- Included characters / Включено символов: `681`
- Truncated / Обрезано: `no`

```rust
pub(super) const DEFAULT_LIST_LIMIT: i64 = 50;
pub(super) const MAX_LIST_LIMIT: i64 = 100;
pub(super) const MIN_LIST_LIMIT: i64 = 1;
pub(super) const ARTIFACT_METADATA_KIND: &str = "document_processing";
pub(super) const DEFAULT_MAX_ATTEMPTS: i32 = 3;
pub(super) const JOB_ID_PREFIX: &str = "document_processing_job:v1:";
pub(super) const ARTIFACT_ID_PREFIX: &str = "document_artifact:v1:";
pub(super) const RETRY_EVENT_TYPE: &str = "document_processing.retry_requested";
pub(super) const RETRY_EVENT_ID_PREFIX: &str = "document_processing_retry:";
pub(super) const RETRY_SOURCE_KIND: &str = "document_processing_retry";
pub(super) const RETRY_SOURCE_PROVIDER: &str = "local_api";
```

### `backend/src/domains/documents/processing/documents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/documents.rs`
- Size bytes / Размер в байтах: `2529`
- Included characters / Включено символов: `2529`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::Postgres;
use sqlx::{Row, Transaction};

use super::errors::DocumentProcessingError;
use super::store::DocumentProcessingStore;

impl DocumentProcessingStore {
    pub(super) async fn ensure_document_exists(
        &self,
        document_id: &str,
    ) -> Result<(), DocumentProcessingError> {
        if self.document_exists(document_id).await? {
            Ok(())
        } else {
            Err(DocumentProcessingError::DocumentNotFound)
        }
    }

    pub(super) async fn document_for_id(
        &self,
        tx_or_pool: &mut Transaction<'_, Postgres>,
        document_id: &str,
    ) -> Result<Option<DocumentRecord>, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            SELECT
                document_id,
                document_kind,
                extracted_text
            FROM documents
            WHERE document_id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&mut **tx_or_pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(DocumentRecord {
            kind: row.try_get("document_kind")?,
            extracted_text: row.try_get("extracted_text")?,
        }))
    }

    async fn document_exists(&self, document_id: &str) -> Result<bool, DocumentProcessingError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM documents
                WHERE document_id = $1
            )
            "#,
        )
        .bind(document_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    pub(super) async fn document_record_by_id(
        &self,
        document_id: &str,
    ) -> Result<Option<DocumentRecord>, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            SELECT
                document_id,
                document_kind,
                extracted_text
            FROM documents
            WHERE document_id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(DocumentRecord {
            kind: row.try_get("document_kind")?,
            extracted_text: row.try_get("extracted_text")?,
        }))
    }
}

#[derive(Debug)]
pub(super) struct DocumentRecord {
    pub(super) kind: String,
    pub(super) extracted_text: String,
}
```

### `backend/src/domains/documents/processing/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/errors.rs`
- Size bytes / Размер в байтах: `1320`
- Included characters / Включено символов: `1320`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum DocumentProcessingError {
    #[error("document processing limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(&'static str),

    #[error("document processing job not found")]
    JobNotFound,

    #[error("document processing retry requires a failed job")]
    RetryRequiresFailedJob,

    #[error("document processing retry command conflicts with existing event")]
    RetryCommandConflict,

    #[error("document not found")]
    DocumentNotFound,

    #[error("invalid document kind")]
    InvalidStep(String),

    #[error("invalid step value")]
    InvalidStatus(String),

    #[error("invalid artifact kind")]
    InvalidArtifactKind(String),

    #[error("missing document source text")]
    MissingSourceText,

    #[error("OCR backend is not available")]
    OcrBackendUnavailable,

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/domains/documents/processing/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/evidence.rs`
- Size bytes / Размер в байтах: `706`
- Included characters / Включено символов: `706`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::{ObservationStoreError, link_domain_entity_in_transaction};

pub(super) async fn link_document_processing_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    metadata: Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "documents",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}
```

### `backend/src/domains/documents/processing/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/documents/processing/ids.rs`
- Size bytes / Размер в байтах: `472`
- Included characters / Включено символов: `472`
- Truncated / Обрезано: `no`

```rust
use super::constants::{ARTIFACT_ID_PREFIX, JOB_ID_PREFIX};
use super::models::{DocumentArtifactKind, DocumentProcessingStep};

pub(super) fn job_id(document_id: &str, step: DocumentProcessingStep) -> String {
    format!("{JOB_ID_PREFIX}{document_id}:{:0}", step.as_str())
}

pub(super) fn artifact_id(document_id: &str, artifact_kind: DocumentArtifactKind) -> String {
    format!(
        "{ARTIFACT_ID_PREFIX}{document_id}:{:0}",
        artifact_kind.as_str()
    )
}
```
