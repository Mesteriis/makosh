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

- Chunk ID / ID чанка: `055-source-backend-part-035`
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

### `backend/src/engines/automation/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/store.rs`
- Size bytes / Размер в байтах: `8917`
- Included characters / Включено символов: `8917`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::AutomationError;
use super::evidence::{capture_policy_observation, capture_template_observation};
use super::models::{
    AutomationPolicy, AutomationTemplate, NewAutomationPolicy, NewAutomationTemplate,
    TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use super::rows::{row_to_policy, row_to_template, string_vec_from_value};
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct AutomationStore {
    pool: PgPool,
}

impl AutomationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_template(
        &self,
        template: &NewAutomationTemplate,
        actor_id: &str,
    ) -> Result<AutomationTemplate, AutomationError> {
        template.validate()?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO automation_templates (
                template_id,
                name,
                body_template,
                required_variables,
                updated_at
            )
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (template_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                body_template = EXCLUDED.body_template,
                required_variables = EXCLUDED.required_variables,
                updated_at = now()
            RETURNING
                template_id,
                name,
                body_template,
                required_variables,
                created_at,
                updated_at
            "#,
        )
        .bind(template.template_id.trim())
        .bind(template.name.trim())
        .bind(template.body_template.trim())
        .bind(json!(template.required_variables))
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_template(row)?;
        capture_template_observation(
            &mut transaction,
            &stored,
            "upsert",
            actor_id,
            stored.updated_at,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn upsert_policy(
        &self,
        policy: &NewAutomationPolicy,
        actor_id: &str,
    ) -> Result<AutomationPolicy, AutomationError> {
        policy.validate()?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO automation_policies (
                policy_id,
                template_id,
                name,
                enabled,
                account_id,
                allowed_chat_ids,
                trigger_kind,
                max_sends_per_hour,
                quiet_hours,
                expires_at,
                conditions,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
            ON CONFLICT (policy_id)
            DO UPDATE SET
                template_id = EXCLUDED.template_id,
                name = EXCLUDED.name,
                enabled = EXCLUDED.enabled,
                account_id = EXCLUDED.account_id,
                allowed_chat_ids = EXCLUDED.allowed_chat_ids,
                trigger_kind = EXCLUDED.trigger_kind,
                max_sends_per_hour = EXCLUDED.max_sends_per_hour,
                quiet_hours = EXCLUDED.quiet_hours,
                expires_at = EXCLUDED.expires_at,
                conditions = EXCLUDED.conditions,
                updated_at = now()
            RETURNING
                policy_id,
                template_id,
                name,
                enabled,
                account_id,
                allowed_chat_ids,
                trigger_kind,
                max_sends_per_hour,
                quiet_hours,
                expires_at,
                conditions,
                created_at,
                updated_at
            "#,
        )
        .bind(policy.policy_id.trim())
        .bind(policy.template_id.trim())
        .bind(policy.name.trim())
        .bind(policy.enabled)
        .bind(policy.account_id.trim())
        .bind(json!(policy.allowed_chat_ids))
        .bind(policy.trigger_kind.trim())
        .bind(policy.max_sends_per_hour)
        .bind(&policy.quiet_hours)
        .bind(policy.expires_at)
        .bind(&policy.conditions)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_policy(row)?;
        capture_policy_observation(
            &mut transaction,
            &stored,
            "upsert",
            actor_id,
            stored.updated_at,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_templates(&self) -> Result<Vec<AutomationTemplate>, AutomationError> {
        let rows = sqlx::query(
            r#"
            SELECT
                template_id,
                name,
                body_template,
                required_variables,
                created_at,
                updated_at
            FROM automation_templates
            ORDER BY updated_at DESC, template_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_template).collect()
    }

    pub async fn list_policies(&self) -> Result<Vec<AutomationPolicy>, AutomationError> {
        let rows = sqlx::query(
            r#"
            SELECT
                policy_id,
                template_id,
                name,
                enabled,
                account_id,
                allowed_chat_ids,
                trigger_kind,
                max_sends_per_hour,
                quiet_hours,
                expires_at,
                conditions,
                created_at,
                updated_at
            FROM automation_policies
            ORDER BY updated_at DESC, policy_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_policy).collect()
    }

    pub async fn dry_run_send(
        &self,
        request: &TelegramSendDryRunRequest,
        actor_id: &str,
    ) -> Result<TelegramSendDryRunResponse, AutomationError> {
        super::dry_run::dry_run_send(&self.pool, request, actor_id).await
    }

    pub(super) async fn policy_with_template(
        pool: &PgPool,
        policy_id: &str,
    ) -> Result<(AutomationPolicy, AutomationTemplate), AutomationError> {
        let policy_id = validate_non_empty("policy_id", policy_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                p.policy_id,
                p.template_id,
                p.name AS policy_name,
                p.enabled,
                p.account_id,
                p.allowed_chat_ids,
                p.trigger_kind,
                p.max_sends_per_hour,
                p.quiet_hours,
                p.expires_at,
                p.conditions,
                p.created_at AS policy_created_at,
                p.updated_at AS policy_updated_at,
                t.name AS template_name,
                t.body_template,
                t.required_variables,
                t.created_at AS template_created_at,
                t.updated_at AS template_updated_at
            FROM automation_policies p
            JOIN automation_templates t ON t.template_id = p.template_id
            WHERE p.policy_id = $1
            "#,
        )
        .bind(&policy_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AutomationError::PolicyNotFound)?;

        Ok((
            AutomationPolicy {
                policy_id: row.try_get("policy_id")?,
                template_id: row.try_get("template_id")?,
                name: row.try_get("policy_name")?,
                enabled: row.try_get("enabled")?,
                account_id: row.try_get("account_id")?,
                allowed_chat_ids: string_vec_from_value(row.try_get("allowed_chat_ids")?)?,
                trigger_kind: row.try_get("trigger_kind")?,
                max_sends_per_hour: row.try_get("max_sends_per_hour")?,
                quiet_hours: row.try_get("quiet_hours")?,
                expires_at: row.try_get("expires_at")?,
                conditions: row.try_get("conditions")?,
                created_at: row.try_get("policy_created_at")?,
                updated_at: row.try_get("policy_updated_at")?,
            },
            AutomationTemplate {
                template_id: row.try_get("template_id")?,
                name: row.try_get("template_name")?,
                body_template: row.try_get("body_template")?,
                required_variables: string_vec_from_value(row.try_get("required_variables")?)?,
                created_at: row.try_get("template_created_at")?,
                updated_at: row.try_get("template_updated_at")?,
            },
        ))
    }
}
```

### `backend/src/engines/automation/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/validation.rs`
- Size bytes / Размер в байтах: `2942`
- Included characters / Включено символов: `2942`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::AutomationError;
use super::models::{NewAutomationPolicy, NewAutomationTemplate, TelegramSendDryRunRequest};

impl NewAutomationTemplate {
    pub(super) fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("template_id", &self.template_id)?;
        validate_non_empty("name", &self.name)?;
        validate_non_empty("body_template", &self.body_template)?;
        for variable in &self.required_variables {
            validate_variable_name(variable)?;
        }
        Ok(())
    }
}

impl NewAutomationPolicy {
    pub(super) fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("policy_id", &self.policy_id)?;
        validate_non_empty("template_id", &self.template_id)?;
        validate_non_empty("name", &self.name)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("trigger_kind", &self.trigger_kind)?;
        if self.max_sends_per_hour <= 0 {
            return Err(AutomationError::InvalidRequest(
                "max_sends_per_hour must be greater than zero".to_owned(),
            ));
        }
        if self.allowed_chat_ids.is_empty() {
            return Err(AutomationError::InvalidRequest(
                "allowed_chat_ids must not be empty".to_owned(),
            ));
        }
        validate_object("quiet_hours", &self.quiet_hours)?;
        validate_object("conditions", &self.conditions)?;
        Ok(())
    }
}

impl TelegramSendDryRunRequest {
    pub(super) fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("policy_id", &self.policy_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_object("variables", &self.variables)?;
        validate_object("source_context", &self.source_context)?;
        Ok(())
    }
}

pub(super) fn validate_variable_name(value: &str) -> Result<String, AutomationError> {
    let value = validate_non_empty("required_variable", value)?;
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(AutomationError::InvalidRequest(
            "template variables must be ASCII letters, numbers or underscores".to_owned(),
        ));
    }
    Ok(value)
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, AutomationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AutomationError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), AutomationError> {
    if !matches!(value, Value::Object(_)) {
        return Err(AutomationError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}
```

### `backend/src/engines/call_intelligence/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/call_intelligence/engine.rs`
- Size bytes / Размер в байтах: `4915`
- Included characters / Включено символов: `4915`
- Truncated / Обрезано: `no`

```rust
use crate::platform::realtime_conversation::CallBundleManifest;

use super::models::{
    CallIntelligenceArtifactRequirement, CallIntelligencePipelinePlan, CallIntelligenceStep,
};

#[derive(Clone, Debug, Default)]
pub struct CallIntelligenceEngine;

impl CallIntelligenceEngine {
    pub fn plan_from_bundle(&self, manifest: &CallBundleManifest) -> CallIntelligencePipelinePlan {
        CallIntelligencePipelinePlan {
            bundle_id: manifest.bundle_id.clone(),
            requirements: vec![
                CallIntelligenceArtifactRequirement {
                    kind: "audio.mp3".to_owned(),
                    required: true,
                    purpose: "transcription and diarization".to_owned(),
                },
                CallIntelligenceArtifactRequirement {
                    kind: "speaker-hints.jsonl".to_owned(),
                    required: false,
                    purpose: "warm-start speaker count and possible human labels".to_owned(),
                },
                CallIntelligenceArtifactRequirement {
                    kind: "screenshots".to_owned(),
                    required: false,
                    purpose: "screen intelligence, OCR and visual evidence".to_owned(),
                },
                CallIntelligenceArtifactRequirement {
                    kind: "chat.json".to_owned(),
                    required: false,
                    purpose: "meeting chat evidence and shared links/files".to_owned(),
                },
            ],
            steps: vec![
                step(
                    "transcribe",
                    "Transcribe MP3",
                    ["audio.mp3"],
                    ["transcript.json", "transcript.md"],
                    "audio_is_capture_artifact",
                ),
                step(
                    "diarize",
                    "Diarize speakers",
                    ["audio.mp3", "speaker-hints.jsonl"],
                    ["speaker-timeline.json"],
                    "speaker_hints_are_not_truth",
                ),
                step(
                    "identify_speakers",
                    "Merge speaker identities",
                    [
                        "speaker-timeline.json",
                        "participants.json",
                        "calendar_event",
                    ],
                    ["speaker-identities.json"],
                    "confidence_weighted_identity_merge",
                ),
                step(
                    "topics",
                    "Build topic timeline",
                    ["transcript.json"],
                    ["topics.json"],
                    "ai_candidate_with_evidence",
                ),
                step(
                    "decisions",
                    "Detect decisions",
                    ["transcript.json", "topics.json"],
                    ["decisions.json"],
                    "candidate_not_domain_truth",
                ),
                step(
                    "actions",
                    "Detect action items",
                    ["transcript.json", "speaker-identities.json"],
                    ["tasks.json"],
                    "radar_review_before_task",
                ),
                step(
                    "screen_intelligence",
                    "Analyze screenshots and OCR",
                    ["screenshots"],
                    ["ocr/", "visual-evidence.json"],
                    "screenshot_is_evidence_not_context_by_itself",
                ),
                step(
                    "knowledge",
                    "Extract meeting knowledge",
                    [
                        "transcript.json",
                        "decisions.json",
                        "tasks.json",
                        "visual-evidence.json",
                    ],
                    ["knowledge.json", "summary.md"],
                    "memory_candidate_requires_provenance",
                ),
                step(
                    "radar",
                    "Project important findings to Radar",
                    ["knowledge.json", "tasks.json", "decisions.json"],
                    ["radar-signals.json"],
                    "review_required_before_promotion",
                ),
            ],
        }
    }
}

fn step<const I: usize, const O: usize>(
    step_id: &'static str,
    title: &'static str,
    input_artifacts: [&'static str; I],
    output_artifacts: [&'static str; O],
    source_of_truth_policy: &'static str,
) -> CallIntelligenceStep {
    CallIntelligenceStep {
        step_id: step_id.to_owned(),
        title: title.to_owned(),
        input_artifacts: input_artifacts.into_iter().map(str::to_owned).collect(),
        output_artifacts: output_artifacts.into_iter().map(str::to_owned).collect(),
        source_of_truth_policy: source_of_truth_policy.to_owned(),
    }
}
```

### `backend/src/engines/call_intelligence/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/call_intelligence/mod.rs`
- Size bytes / Размер в байтах: `216`
- Included characters / Включено символов: `216`
- Truncated / Обрезано: `no`

```rust
mod engine;
mod models;

pub use engine::CallIntelligenceEngine;
pub use models::{
    CallIntelligenceArtifactRequirement, CallIntelligenceOutputCandidate,
    CallIntelligencePipelinePlan, CallIntelligenceStep,
};
```

### `backend/src/engines/call_intelligence/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/call_intelligence/models.rs`
- Size bytes / Размер в байтах: `964`
- Included characters / Включено символов: `964`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallIntelligenceArtifactRequirement {
    pub kind: String,
    pub required: bool,
    pub purpose: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallIntelligenceStep {
    pub step_id: String,
    pub title: String,
    pub input_artifacts: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub source_of_truth_policy: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallIntelligencePipelinePlan {
    pub bundle_id: String,
    pub requirements: Vec<CallIntelligenceArtifactRequirement>,
    pub steps: Vec<CallIntelligenceStep>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CallIntelligenceOutputCandidate {
    pub candidate_kind: String,
    pub title: String,
    pub confidence: f32,
    pub evidence: Value,
}
```

### `backend/src/engines/consistency.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency.rs`
- Size bytes / Размер в байтах: `605`
- Included characters / Включено символов: `605`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod engine;
mod errors;
pub(crate) mod evidence;
mod helpers;
mod models;
mod parsing;
mod rows;
mod store;
mod validation;

pub use engine::ConsistencyEngine;
pub use errors::ConsistencyError;
pub use helpers::contradiction_observation_id;
pub use models::{
    AcceptedClaim, ContradictionObservation, ContradictionReviewState, ContradictionSeverity,
    ContradictionSourceKind, EvidenceClaimExtractionInput, NewContradictionObservation,
    NewEvidenceClaim,
};
pub use store::ContradictionObservationStore;
pub use store::ContradictionObservationStore as ContradictionObservationPort;
```

### `backend/src/engines/consistency/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/constants.rs`
- Size bytes / Размер в байтах: `158`
- Included characters / Включено символов: `158`
- Truncated / Обрезано: `no`

```rust
pub(super) const MAX_REFRESH_LIMIT: i64 = 100;
pub(super) const MIN_REFRESH_LIMIT: i64 = 1;
pub(super) const STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE: f64 = 0.8;
```

### `backend/src/engines/consistency/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/engine.rs`
- Size bytes / Размер в байтах: `4120`
- Included characters / Включено символов: `4120`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::errors::ConsistencyError;
use super::helpers::{
    claim_text, contradiction_metadata, normalize_claim_value, severity_for_confidence,
};
use super::models::{
    AcceptedClaim, ContradictionReviewState, EvidenceClaimExtractionInput,
    NewContradictionObservation, NewEvidenceClaim,
};
use super::parsing::parse_evidence_claim_line;

pub struct ConsistencyEngine;

impl ConsistencyEngine {
    pub fn detect_claim_contradictions(
        accepted_claims: &[AcceptedClaim],
        new_claims: &[NewEvidenceClaim],
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        Self::detect_claim_contradictions_with_detector(
            accepted_claims,
            new_claims,
            "structured_claim",
        )
    }

    pub fn extract_evidence_claims(
        input: &EvidenceClaimExtractionInput,
    ) -> Result<Vec<NewEvidenceClaim>, ConsistencyError> {
        input.validate()?;

        let mut claims = Vec::new();
        for line in input.text.lines() {
            let Some((claim_type, value)) = parse_evidence_claim_line(line) else {
                continue;
            };

            claims.push(NewEvidenceClaim {
                subject_id: input.subject_id.clone(),
                claim_type,
                value,
                source_kind: input.source_kind,
                source_id: input.source_id.clone(),
                confidence: input.confidence,
            });
        }

        Ok(claims)
    }

    pub fn detect_evidence_contradictions(
        accepted_claims: &[AcceptedClaim],
        evidence_inputs: &[EvidenceClaimExtractionInput],
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        let mut extracted_claims = Vec::new();
        for input in evidence_inputs {
            extracted_claims.extend(Self::extract_evidence_claims(input)?);
        }

        Self::detect_claim_contradictions_with_detector(
            accepted_claims,
            &extracted_claims,
            "structured_evidence_claim",
        )
    }

    fn detect_claim_contradictions_with_detector(
        accepted_claims: &[AcceptedClaim],
        new_claims: &[NewEvidenceClaim],
        detector: &str,
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        for claim in accepted_claims {
            claim.validate()?;
        }
        for claim in new_claims {
            claim.validate()?;
        }

        let mut observations = Vec::new();
        for accepted in accepted_claims {
            for new_claim in new_claims {
                if accepted.subject_id != new_claim.subject_id {
                    continue;
                }
                if accepted.claim_type.trim() != new_claim.claim_type.trim() {
                    continue;
                }
                if normalize_claim_value(&accepted.value) == normalize_claim_value(&new_claim.value)
                {
                    continue;
                }

                let confidence = accepted.confidence.min(new_claim.confidence);
                observations.push(NewContradictionObservation {
                    old_source_kind: accepted.source_kind,
                    old_source_id: accepted.source_id.clone(),
                    new_source_kind: new_claim.source_kind,
                    new_source_id: new_claim.source_id.clone(),
                    affected_entities: json!([{
                        "entity_kind": "subject",
                        "entity_id": accepted.subject_id,
                    }]),
                    conflict_type: "direct_contradiction".to_owned(),
                    old_claim: claim_text(&accepted.claim_type, &accepted.value),
                    new_claim: claim_text(&new_claim.claim_type, &new_claim.value),
                    confidence,
                    severity: severity_for_confidence(confidence),
                    review_state: ContradictionReviewState::Suggested,
                    metadata: contradiction_metadata(detector, accepted, new_claim),
                });
            }
        }

        Ok(observations)
    }
}
```

### `backend/src/engines/consistency/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/errors.rs`
- Size bytes / Размер в байтах: `1013`
- Included characters / Включено символов: `1013`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum ConsistencyError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(f64),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be a JSON array or object")]
    InvalidJsonArrayOrObject(&'static str),

    #[error("unknown contradiction source kind stored in database: {0}")]
    UnknownSourceKind(String),

    #[error("unknown contradiction severity stored in database: {0}")]
    UnknownSeverity(String),

    #[error("unknown contradiction review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("contradiction observation not found: {0}")]
    ObservationNotFound(String),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
}
```

### `backend/src/engines/consistency/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/evidence.rs`
- Size bytes / Размер в байтах: `4935`
- Included characters / Включено символов: `4935`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgRow;
use sqlx::postgres::Postgres;

use super::errors::ConsistencyError;
use crate::platform::observations::{ObservationStoreError, link_domain_entity_in_transaction};

pub(super) struct ActivePersonFactClaim {
    pub(super) fact_id: String,
    pub(super) person_id: String,
    pub(super) claim_type: String,
    pub(super) value: String,
    pub(super) confidence: f64,
    pub(super) email_address: String,
}

pub(super) fn row_to_active_person_fact_claim(
    row: PgRow,
) -> Result<ActivePersonFactClaim, ConsistencyError> {
    Ok(ActivePersonFactClaim {
        fact_id: row.try_get("fact_id")?,
        person_id: row.try_get("person_id")?,
        claim_type: row.try_get("fact_type")?,
        value: row.try_get("value")?,
        confidence: row.try_get("confidence")?,
        email_address: normalize_email_address_for_match(
            row.try_get::<String, _>("email_address")?.as_str(),
        ),
    })
}

pub(super) struct MessageEvidence {
    pub(super) message_id: String,
    pub(super) sender_email_address: String,
    pub(super) text: String,
}

pub(super) fn row_to_message_evidence(row: PgRow) -> Result<MessageEvidence, ConsistencyError> {
    let subject = row.try_get::<String, _>("subject")?;
    let body_text = row.try_get::<String, _>("body_text")?;

    Ok(MessageEvidence {
        message_id: row.try_get("message_id")?,
        sender_email_address: normalize_email_address_for_match(
            row.try_get::<String, _>("sender")?.as_str(),
        ),
        text: format!("{subject}\n{body_text}"),
    })
}

pub(super) struct ChannelMessageEvidence {
    pub(super) message_id: String,
    pub(super) person_id: String,
    pub(super) text: String,
}

pub(super) fn row_to_channel_message_evidence(
    row: PgRow,
) -> Result<ChannelMessageEvidence, ConsistencyError> {
    let subject = row.try_get::<String, _>("subject")?;
    let body_text = row.try_get::<String, _>("body_text")?;

    Ok(ChannelMessageEvidence {
        message_id: row.try_get("message_id")?,
        person_id: row.try_get("person_id")?,
        text: format!("{subject}\n{body_text}"),
    })
}

pub(super) struct DocumentEvidence {
    pub(super) document_id: String,
    pub(super) observation_id: Option<String>,
    pub(super) normalized_text: String,
    pub(super) text: String,
}

impl DocumentEvidence {
    pub(super) fn references_email_address(&self, email_address: &str) -> bool {
        self.normalized_text.contains(email_address)
    }
}

pub(super) fn row_to_document_evidence(row: PgRow) -> Result<DocumentEvidence, ConsistencyError> {
    let title = row.try_get::<String, _>("title")?;
    let extracted_text = row.try_get::<String, _>("extracted_text")?;
    let text = format!("{title}\n{extracted_text}");

    Ok(DocumentEvidence {
        document_id: row.try_get("document_id")?,
        observation_id: row.try_get("observation_id")?,
        normalized_text: text.to_ascii_lowercase(),
        text,
    })
}

pub(super) struct MeetingNoteEvidence {
    pub(super) note_id: String,
    pub(super) person_id: String,
    pub(super) text: String,
}

pub(super) fn row_to_meeting_note_evidence(
    row: PgRow,
) -> Result<MeetingNoteEvidence, ConsistencyError> {
    let title = row.try_get::<String, _>("title")?;
    let content = row.try_get::<String, _>("content")?;

    Ok(MeetingNoteEvidence {
        note_id: row.try_get("note_id")?,
        person_id: row.try_get("person_id")?,
        text: format!("{title}\n{content}"),
    })
}

pub(super) struct CallTranscriptEvidence {
    pub(super) transcript_id: String,
    pub(super) person_id: String,
    pub(super) text: String,
}

pub(super) fn row_to_call_transcript_evidence(
    row: PgRow,
) -> Result<CallTranscriptEvidence, ConsistencyError> {
    Ok(CallTranscriptEvidence {
        transcript_id: row.try_get("transcript_id")?,
        person_id: row.try_get("person_id")?,
        text: row.try_get("transcript_text")?,
    })
}

fn normalize_email_address_for_match(email_address: &str) -> String {
    email_addr_spec(email_address).trim().to_ascii_lowercase()
}

fn email_addr_spec(value: &str) -> &str {
    let value = value.trim();
    if let Some((_, tail)) = value.rsplit_once('<')
        && let Some((addr, _)) = tail.split_once('>')
    {
        return addr.trim();
    }
    value.trim_matches('"')
}

pub(crate) async fn link_consistency_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    metadata: serde_json::Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "consistency",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}
```

### `backend/src/engines/consistency/helpers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/helpers.rs`
- Size bytes / Размер в байтах: `1934`
- Included characters / Включено символов: `1934`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::models::{
    AcceptedClaim, ContradictionSeverity, NewContradictionObservation, NewEvidenceClaim,
};

pub fn contradiction_observation_id(observation: &NewContradictionObservation) -> String {
    format!(
        "contradiction:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        observation.old_source_kind.as_str().len(),
        observation.old_source_kind.as_str(),
        observation.old_source_id.len(),
        observation.old_source_id,
        observation.new_source_kind.as_str().len(),
        observation.new_source_kind.as_str(),
        observation.new_source_id.len(),
        observation.new_source_id,
        observation.conflict_type.len(),
        observation.conflict_type
    )
}

pub(super) fn claim_text(claim_type: &str, value: &str) -> String {
    let claim_type = claim_type.trim();
    let value = value.trim();
    format!("{claim_type}={value}")
}

pub(super) fn normalize_claim_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(super) fn severity_for_confidence(confidence: f64) -> ContradictionSeverity {
    if confidence >= 0.95 {
        ContradictionSeverity::Critical
    } else if confidence >= 0.9 {
        ContradictionSeverity::High
    } else if confidence >= 0.7 {
        ContradictionSeverity::Medium
    } else {
        ContradictionSeverity::Low
    }
}

pub(super) fn contradiction_metadata(
    detector: &str,
    accepted: &AcceptedClaim,
    new_claim: &NewEvidenceClaim,
) -> Value {
    if detector == "structured_evidence_claim" {
        json!({
            "detector": detector,
            "claim_type": accepted.claim_type.trim(),
            "source_kind": new_claim.source_kind.as_str(),
        })
    } else {
        json!({
            "detector": detector,
            "claim_type": accepted.claim_type.trim(),
        })
    }
}
```

### `backend/src/engines/consistency/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/models.rs`
- Size bytes / Размер в байтах: `5967`
- Included characters / Включено символов: `5967`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::ConsistencyError;
use super::validation::{
    validate_confidence, validate_json_array_or_object, validate_json_object, validate_non_empty,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSourceKind {
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Relationship,
}

impl ContradictionSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Event => "event",
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Task => "task",
            Self::Relationship => "relationship",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ContradictionSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ContradictionReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ConsistencyError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ConsistencyError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AcceptedClaim {
    pub subject_id: String,
    pub claim_type: String,
    pub value: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub confidence: f64,
}

impl AcceptedClaim {
    pub(super) fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("claim_type", &self.claim_type)?;
        validate_non_empty("value", &self.value)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewEvidenceClaim {
    pub subject_id: String,
    pub claim_type: String,
    pub value: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub confidence: f64,
}

impl NewEvidenceClaim {
    pub(super) fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("claim_type", &self.claim_type)?;
        validate_non_empty("value", &self.value)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceClaimExtractionInput {
    pub subject_id: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub text: String,
    pub confidence: f64,
}

impl EvidenceClaimExtractionInput {
    pub(super) fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("text", &self.text)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewContradictionObservation {
    pub old_source_kind: ContradictionSourceKind,
    pub old_source_id: String,
    pub new_source_kind: ContradictionSourceKind,
    pub new_source_id: String,
    pub affected_entities: Value,
    pub conflict_type: String,
    pub old_claim: String,
    pub new_claim: String,
    pub confidence: f64,
    pub severity: ContradictionSeverity,
    pub review_state: ContradictionReviewState,
    pub metadata: Value,
}

impl NewContradictionObservation {
    pub fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("old_source_id", &self.old_source_id)?;
        validate_non_empty("new_source_id", &self.new_source_id)?;
        validate_non_empty("conflict_type", &self.conflict_type)?;
        validate_non_empty("old_claim", &self.old_claim)?;
        validate_non_empty("new_claim", &self.new_claim)?;
        validate_confidence(self.confidence)?;
        validate_json_array_or_object("affected_entities", &self.affected_entities)?;
        validate_json_object("metadata", &self.metadata)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContradictionObservation {
    pub observation_id: String,
    pub old_source_kind: ContradictionSourceKind,
    pub old_source_id: String,
    pub new_source_kind: ContradictionSourceKind,
    pub new_source_id: String,
    pub affected_entities: Value,
    pub conflict_type: String,
    pub old_claim: String,
    pub new_claim: String,
    pub confidence: f64,
    pub severity: ContradictionSeverity,
    pub review_state: ContradictionReviewState,
    pub metadata: Value,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### `backend/src/engines/consistency/parsing.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/parsing.rs`
- Size bytes / Размер в байтах: `2882`
- Included characters / Включено символов: `2882`
- Truncated / Обрезано: `no`

```rust
pub(super) fn parse_evidence_claim_line(line: &str) -> Option<(String, String)> {
    parse_structured_claim_line(line).or_else(|| parse_natural_language_claim_line(line))
}

fn parse_structured_claim_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let delimiter_index = match (trimmed.find(':'), trimmed.find('=')) {
        (Some(colon), Some(equals)) => Some(colon.min(equals)),
        (Some(colon), None) => Some(colon),
        (None, Some(equals)) => Some(equals),
        (None, None) => None,
    }?;

    let raw_claim_type = trimmed[..delimiter_index].trim();
    let value = trimmed[delimiter_index + 1..].trim();
    if raw_claim_type.is_empty() || value.is_empty() {
        return None;
    }

    let claim_type = raw_claim_type
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .to_lowercase();
    if claim_type.is_empty()
        || !claim_type
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        || !is_supported_deterministic_claim_type(&claim_type)
    {
        return None;
    }

    Some((claim_type, normalize_extracted_claim_value(value)?))
}

fn parse_natural_language_claim_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    for prefix in [
        "i am now in ",
        "i am in ",
        "i'm now in ",
        "i'm in ",
        "location is ",
        "location changed to ",
        "location became ",
    ] {
        if let Some(value) = value_after_case_insensitive_pattern(trimmed, &lower, prefix) {
            return Some(("location".to_owned(), value));
        }
    }

    for prefix in ["status is ", "status changed to ", "status became "] {
        if let Some(value) = value_after_case_insensitive_pattern(trimmed, &lower, prefix) {
            return Some(("status".to_owned(), value));
        }
    }

    None
}

fn value_after_case_insensitive_pattern(
    original: &str,
    lower: &str,
    pattern: &str,
) -> Option<String> {
    let start = lower.find(pattern)? + pattern.len();
    normalize_extracted_claim_value(&original[start..])
}

fn normalize_extracted_claim_value(value: &str) -> Option<String> {
    let value = value
        .trim()
        .trim_matches(|character: char| {
            matches!(
                character,
                '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | ')' | '('
            )
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if value.is_empty() { None } else { Some(value) }
}

fn is_supported_deterministic_claim_type(claim_type: &str) -> bool {
    matches!(claim_type, "location" | "status")
}
```

### `backend/src/engines/consistency/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/rows.rs`
- Size bytes / Размер в байтах: `2681`
- Included characters / Включено символов: `2681`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::ConsistencyError;
use super::models::{
    ContradictionObservation, ContradictionReviewState, ContradictionSeverity,
    ContradictionSourceKind,
};

pub(super) fn row_to_observation(row: PgRow) -> Result<ContradictionObservation, ConsistencyError> {
    Ok(ContradictionObservation {
        observation_id: row.try_get("observation_id")?,
        old_source_kind: parse_source_kind(row.try_get("old_source_kind")?)?,
        old_source_id: row.try_get("old_source_id")?,
        new_source_kind: parse_source_kind(row.try_get("new_source_kind")?)?,
        new_source_id: row.try_get("new_source_id")?,
        affected_entities: row.try_get("affected_entities")?,
        conflict_type: row.try_get("conflict_type")?,
        old_claim: row.try_get("old_claim")?,
        new_claim: row.try_get("new_claim")?,
        confidence: row.try_get("confidence")?,
        severity: parse_severity(row.try_get("severity")?)?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        metadata: row.try_get("metadata")?,
        reviewed_by: row.try_get("reviewed_by")?,
        reviewed_at: row.try_get("reviewed_at")?,
        resolution: row.try_get("resolution")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn parse_source_kind(
    value: String,
) -> Result<ContradictionSourceKind, ConsistencyError> {
    match value.as_str() {
        "communication" => Ok(ContradictionSourceKind::Communication),
        "document" => Ok(ContradictionSourceKind::Document),
        "event" => Ok(ContradictionSourceKind::Event),
        "memory" => Ok(ContradictionSourceKind::Memory),
        "knowledge" => Ok(ContradictionSourceKind::Knowledge),
        "decision" => Ok(ContradictionSourceKind::Decision),
        "obligation" => Ok(ContradictionSourceKind::Obligation),
        "task" => Ok(ContradictionSourceKind::Task),
        "relationship" => Ok(ContradictionSourceKind::Relationship),
        _ => Err(ConsistencyError::UnknownSourceKind(value)),
    }
}

pub(super) fn parse_severity(value: String) -> Result<ContradictionSeverity, ConsistencyError> {
    match value.as_str() {
        "low" => Ok(ContradictionSeverity::Low),
        "medium" => Ok(ContradictionSeverity::Medium),
        "high" => Ok(ContradictionSeverity::High),
        "critical" => Ok(ContradictionSeverity::Critical),
        _ => Err(ConsistencyError::UnknownSeverity(value)),
    }
}

pub(super) fn parse_review_state(
    value: String,
) -> Result<ContradictionReviewState, ConsistencyError> {
    ContradictionReviewState::parse(value)
}
```

### `backend/src/engines/consistency/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/store.rs`
- Size bytes / Размер в байтах: `2110`
- Included characters / Включено символов: `2110`
- Truncated / Обрезано: `no`

```rust
mod observations;
mod refresh;
mod review;
mod sources;

use sqlx::postgres::PgPool;

use super::errors::ConsistencyError;
use super::models::{
    ContradictionObservation, ContradictionReviewState, NewContradictionObservation,
};

pub struct ContradictionObservationStore {
    pool: PgPool,
}

impl ContradictionObservationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_deterministic_observations(
        &self,
        limit: i64,
    ) -> Result<usize, ConsistencyError> {
        refresh::refresh_deterministic_observations(&self.pool, limit).await
    }

    pub async fn upsert(
        &self,
        observation: &NewContradictionObservation,
    ) -> Result<ContradictionObservation, ConsistencyError> {
        observations::upsert(&self.pool, observation).await
    }

    pub async fn list_open(
        &self,
        limit: i64,
    ) -> Result<Vec<ContradictionObservation>, ConsistencyError> {
        observations::list_open(&self.pool, limit).await
    }

    pub async fn set_review_state(
        &self,
        observation_id: &str,
        review_state: ContradictionReviewState,
        reviewed_by: &str,
        resolution: Option<&str>,
    ) -> Result<ContradictionObservation, ConsistencyError> {
        self.set_review_state_with_observation(
            observation_id,
            review_state,
            reviewed_by,
            resolution,
            None,
            None,
        )
        .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        observation_id: &str,
        review_state: ContradictionReviewState,
        reviewed_by: &str,
        resolution: Option<&str>,
        review_observation_id: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ContradictionObservation, ConsistencyError> {
        review::set_review_state(
            &self.pool,
            observation_id,
            review_state,
            reviewed_by,
            resolution,
            review_observation_id,
            metadata,
        )
        .await
    }
}
```

### `backend/src/engines/consistency/store/observations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/store/observations.rs`
- Size bytes / Размер в байтах: `6333`
- Included characters / Включено символов: `6333`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use super::super::errors::ConsistencyError;
use super::super::evidence::link_consistency_entity_in_transaction;
use super::super::helpers::contradiction_observation_id;
use super::super::models::{ContradictionObservation, NewContradictionObservation};
use super::super::rows::row_to_observation;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationPort};

pub(super) async fn upsert(
    pool: &PgPool,
    observation: &NewContradictionObservation,
) -> Result<ContradictionObservation, ConsistencyError> {
    observation.validate()?;
    let observation_id = contradiction_observation_id(observation);
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO contradiction_observations (
            observation_id,
            old_source_kind,
            old_source_id,
            new_source_kind,
            new_source_id,
            affected_entities,
            conflict_type,
            old_claim,
            new_claim,
            confidence,
            severity,
            review_state,
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
            $11,
            $12,
            $13
        )
        ON CONFLICT (observation_id)
        DO UPDATE SET
            affected_entities = EXCLUDED.affected_entities,
            old_claim = EXCLUDED.old_claim,
            new_claim = EXCLUDED.new_claim,
            confidence = EXCLUDED.confidence,
            severity = EXCLUDED.severity,
            metadata = EXCLUDED.metadata,
            updated_at = now()
        RETURNING
            observation_id,
            old_source_kind,
            old_source_id,
            new_source_kind,
            new_source_id,
            affected_entities,
            conflict_type,
            old_claim,
            new_claim,
            confidence::float8 AS confidence,
            severity,
            review_state,
            metadata,
            reviewed_by,
            reviewed_at,
            resolution,
            created_at,
            updated_at
        "#,
    )
    .bind(&observation_id)
    .bind(observation.old_source_kind.as_str())
    .bind(&observation.old_source_id)
    .bind(observation.new_source_kind.as_str())
    .bind(&observation.new_source_id)
    .bind(&observation.affected_entities)
    .bind(&observation.conflict_type)
    .bind(&observation.old_claim)
    .bind(&observation.new_claim)
    .bind(observation.confidence)
    .bind(observation.severity.as_str())
    .bind(observation.review_state.as_str())
    .bind(&observation.metadata)
    .fetch_one(&mut *transaction)
    .await?;

    let stored = row_to_observation(row)?;
    link_contradiction_observation_in_transaction(&mut transaction, &stored).await?;
    transaction.commit().await?;
    Ok(stored)
}

pub(super) async fn list_open(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<ContradictionObservation>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            observation_id,
            old_source_kind,
            old_source_id,
            new_source_kind,
            new_source_id,
            affected_entities,
            conflict_type,
            old_claim,
            new_claim,
            confidence::float8 AS confidence,
            severity,
            review_state,
            metadata,
            reviewed_by,
            reviewed_at,
            resolution,
            created_at,
            updated_at
        FROM contradiction_observations
        WHERE review_state = 'suggested'
        ORDER BY updated_at DESC, observation_id ASC
        LIMIT $1
        "#,
    )
    .bind(limit.clamp(1, 100))
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_observation).collect()
}

pub(crate) async fn link_contradiction_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<(), ConsistencyError> {
    let evidence_observation =
        capture_contradiction_observation_in_transaction(transaction, contradiction).await?;
    link_consistency_entity_in_transaction(
        transaction,
        &evidence_observation.observation_id,
        "contradiction_observation",
        contradiction.observation_id.clone(),
        "upsert",
        json!({
            "conflict_type": contradiction.conflict_type,
            "review_state": contradiction.review_state.as_str(),
            "severity": contradiction.severity.as_str(),
            "old_source_kind": contradiction.old_source_kind.as_str(),
            "old_source_id": contradiction.old_source_id,
            "new_source_kind": contradiction.new_source_kind.as_str(),
            "new_source_id": contradiction.new_source_id,
        }),
    )
    .await?;
    Ok(())
}

async fn capture_contradiction_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<crate::platform::observations::Observation, ConsistencyError> {
    let observation = ObservationPort::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "CONTRADICTION_OBSERVATION",
            ObservationOriginKind::LocalRuntime,
            contradiction.created_at,
            json!({
                "contradiction_observation_id": contradiction.observation_id,
                "conflict_type": contradiction.conflict_type,
                "old_claim": contradiction.old_claim,
                "new_claim": contradiction.new_claim,
                "severity": contradiction.severity.as_str(),
                "review_state": contradiction.review_state.as_str(),
                "affected_entities": contradiction.affected_entities,
            }),
            format!("contradiction://{}", contradiction.observation_id),
        )
        .confidence(contradiction.confidence)
        .provenance(json!({
            "engine": "consistency",
            "pipeline": "contradiction_observations",
        })),
    )
    .await?;
    Ok(observation)
}
```

### `backend/src/engines/consistency/store/refresh.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/store/refresh.rs`
- Size bytes / Размер в байтах: `4927`
- Included characters / Включено символов: `4927`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::super::constants::STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE;
use super::super::engine::ConsistencyEngine;
use super::super::errors::ConsistencyError;
use super::super::evidence::ActivePersonFactClaim;
use super::super::models::{AcceptedClaim, ContradictionSourceKind, EvidenceClaimExtractionInput};
use super::super::validation::validate_refresh_limit;
use super::observations;
use super::sources;

pub(super) async fn refresh_deterministic_observations(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, ConsistencyError> {
    let limit = validate_refresh_limit(limit);
    let facts = sources::active_person_fact_claims(pool, limit).await?;
    let messages = sources::recent_message_evidence(pool, limit).await?;
    let channel_messages = sources::recent_channel_message_evidence(pool, limit).await?;
    let documents = sources::recent_document_evidence(pool, limit).await?;
    let meeting_notes = sources::recent_meeting_note_evidence(pool, limit).await?;
    let call_transcripts = sources::recent_call_transcript_evidence(pool, limit).await?;
    let mut count = 0usize;

    for fact in &facts {
        let accepted = accepted_claim(fact);

        for message in &messages {
            if fact.email_address != message.sender_email_address {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.person_id,
                    ContradictionSourceKind::Communication,
                    &message.message_id,
                    &message.text,
                ),
            )
            .await?;
        }

        for message in &channel_messages {
            if message.person_id != fact.person_id {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.person_id,
                    ContradictionSourceKind::Communication,
                    &message.message_id,
                    &message.text,
                ),
            )
            .await?;
        }

        for document in &documents {
            if !document.references_email_address(&fact.email_address) {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.person_id,
                    ContradictionSourceKind::Document,
                    &document.document_id,
                    &document.text,
                ),
            )
            .await?;
        }

        for note in &meeting_notes {
            if note.person_id != fact.person_id {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.person_id,
                    ContradictionSourceKind::Event,
                    &note.note_id,
                    &note.text,
                ),
            )
            .await?;
        }

        for transcript in &call_transcripts {
            if transcript.person_id != fact.person_id {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.person_id,
                    ContradictionSourceKind::Communication,
                    &transcript.transcript_id,
                    &transcript.text,
                ),
            )
            .await?;
        }
    }

    Ok(count)
}

fn accepted_claim(fact: &ActivePersonFactClaim) -> AcceptedClaim {
    AcceptedClaim {
        subject_id: fact.person_id.clone(),
        claim_type: fact.claim_type.clone(),
        value: fact.value.clone(),
        source_kind: ContradictionSourceKind::Memory,
        source_id: fact.fact_id.clone(),
        confidence: fact.confidence,
    }
}

fn evidence_claim(
    subject_id: &str,
    source_kind: ContradictionSourceKind,
    source_id: &str,
    text: &str,
) -> EvidenceClaimExtractionInput {
    EvidenceClaimExtractionInput {
        subject_id: subject_id.to_owned(),
        source_kind,
        source_id: source_id.to_owned(),
        text: text.to_owned(),
        confidence: STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE,
    }
}

async fn detect_and_upsert(
    pool: &PgPool,
    accepted: &AcceptedClaim,
    evidence: EvidenceClaimExtractionInput,
) -> Result<usize, ConsistencyError> {
    let observations = ConsistencyEngine::detect_evidence_contradictions(
        std::slice::from_ref(accepted),
        std::slice::from_ref(&evidence),
    )?;
    let count = observations.len();

    for observation in observations {
        observations::upsert(pool, &observation).await?;
    }

    Ok(count)
}
```

### `backend/src/engines/consistency/store/review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/store/review.rs`
- Size bytes / Размер в байтах: `2806`
- Included characters / Включено символов: `2806`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::super::errors::ConsistencyError;
use super::super::evidence::link_consistency_entity_in_transaction;
use super::super::models::{ContradictionObservation, ContradictionReviewState};
use super::super::rows::row_to_observation;
use super::super::validation::validate_non_empty;

pub(super) async fn set_review_state(
    pool: &PgPool,
    observation_id: &str,
    review_state: ContradictionReviewState,
    reviewed_by: &str,
    resolution: Option<&str>,
    review_observation_id: Option<&str>,
    metadata: Option<serde_json::Value>,
) -> Result<ContradictionObservation, ConsistencyError> {
    validate_non_empty("observation_id", observation_id)?;
    validate_non_empty("reviewed_by", reviewed_by)?;
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        UPDATE contradiction_observations
        SET review_state = $2,
            reviewed_by = $3,
            reviewed_at = now(),
            resolution = $4,
            updated_at = now()
        WHERE observation_id = $1
        RETURNING
            observation_id,
            old_source_kind,
            old_source_id,
            new_source_kind,
            new_source_id,
            affected_entities,
            conflict_type,
            old_claim,
            new_claim,
            confidence::float8 AS confidence,
            severity,
            review_state,
            metadata,
            reviewed_by,
            reviewed_at,
            resolution,
            created_at,
            updated_at
        "#,
    )
    .bind(observation_id)
    .bind(review_state.as_str())
    .bind(reviewed_by)
    .bind(resolution)
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(row) = row else {
        return Err(ConsistencyError::ObservationNotFound(
            observation_id.to_owned(),
        ));
    };

    let stored = row_to_observation(row)?;
    if let Some(review_observation_id) = review_observation_id.filter(|value| !value.is_empty()) {
        let link_metadata = if let Some(extra) = metadata {
            serde_json::json!({
                "review_state": stored.review_state.as_str(),
                "resolution": stored.resolution,
                "context": extra,
            })
        } else {
            serde_json::json!({
                "review_state": stored.review_state.as_str(),
                "resolution": stored.resolution,
            })
        };
        link_consistency_entity_in_transaction(
            &mut transaction,
            review_observation_id,
            "contradiction_observation",
            stored.observation_id.clone(),
            "review_transition",
            link_metadata,
        )
        .await?;
    }
    transaction.commit().await?;
    Ok(stored)
}
```

### `backend/src/engines/consistency/store/sources.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/store/sources.rs`
- Size bytes / Размер в байтах: `5512`
- Included characters / Включено символов: `5512`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::super::errors::ConsistencyError;
use super::super::evidence::{
    ActivePersonFactClaim, CallTranscriptEvidence, ChannelMessageEvidence, DocumentEvidence,
    MeetingNoteEvidence, MessageEvidence, row_to_active_person_fact_claim,
    row_to_call_transcript_evidence, row_to_channel_message_evidence, row_to_document_evidence,
    row_to_meeting_note_evidence, row_to_message_evidence,
};

pub(super) async fn active_person_fact_claims(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<ActivePersonFactClaim>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            fact.id::text AS fact_id,
            fact.person_id,
            fact.fact_type,
            fact.value,
            fact.confidence::float8 AS confidence,
            person.email_address
        FROM person_facts fact
        JOIN persons person ON person.person_id = fact.person_id
        WHERE fact.is_active = true
          AND length(trim(person.email_address)) > 0
        ORDER BY fact.updated_at DESC, fact.id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_active_person_fact_claim)
        .collect()
}

pub(super) async fn recent_message_evidence(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<MessageEvidence>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            message_id,
            sender,
            subject,
            body_text
        FROM communication_messages
        ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_message_evidence).collect()
}

pub(super) async fn recent_channel_message_evidence(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<ChannelMessageEvidence>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            message.message_id,
            identity.person_id,
            message.subject,
            message.body_text
        FROM communication_messages message
        JOIN person_identities identity
          ON identity.status = 'active'
         AND identity.identity_value = message.message_metadata->>'sender_id'
         AND (
                (
                    message.channel_kind IN ('telegram_user', 'telegram_bot')
                AND identity.identity_type = 'telegram'
                )
             OR (
                    message.channel_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
                AND identity.identity_type = 'whatsapp'
                )
             )
        WHERE message.channel_kind IN (
            'telegram_user',
            'telegram_bot',
            'whatsapp_web',
            'whatsapp_business_cloud'
        )
          AND length(trim(message.body_text)) > 0
          AND length(trim(message.message_metadata->>'sender_id')) > 0
        ORDER BY COALESCE(message.occurred_at, message.projected_at) DESC, message.message_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_channel_message_evidence)
        .collect()
}

pub(super) async fn recent_document_evidence(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<DocumentEvidence>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            document_id,
            observation_id,
            title,
            extracted_text
        FROM documents
        WHERE length(trim(extracted_text)) > 0
        ORDER BY imported_at DESC, document_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_document_evidence).collect()
}

pub(super) async fn recent_meeting_note_evidence(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<MeetingNoteEvidence>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            note.id::text AS note_id,
            participant.person_id,
            event.title,
            note.content
        FROM meeting_notes note
        JOIN calendar_events event ON event.event_id = note.event_id
        JOIN event_participants participant ON participant.event_id = note.event_id
        WHERE participant.person_id IS NOT NULL
          AND length(trim(note.content)) > 0
        ORDER BY note.updated_at DESC, note.id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_meeting_note_evidence).collect()
}

pub(super) async fn recent_call_transcript_evidence(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<CallTranscriptEvidence>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            transcript.transcript_id,
            identity.person_id,
            transcript.transcript_text
        FROM call_transcripts transcript
        JOIN person_identities identity
          ON identity.identity_type = 'telegram'
         AND identity.status = 'active'
         AND identity.identity_value = transcript.provider_chat_id
        WHERE transcript.transcript_status = 'succeeded'
          AND length(trim(transcript.transcript_text)) > 0
        ORDER BY transcript.updated_at DESC, transcript.transcript_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_call_transcript_evidence)
        .collect()
}
```

### `backend/src/engines/consistency/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/consistency/validation.rs`
- Size bytes / Размер в байтах: `1205`
- Included characters / Включено символов: `1205`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::constants::{MAX_REFRESH_LIMIT, MIN_REFRESH_LIMIT};
use super::errors::ConsistencyError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ConsistencyError> {
    if value.trim().is_empty() {
        return Err(ConsistencyError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_confidence(confidence: f64) -> Result<(), ConsistencyError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(ConsistencyError::InvalidConfidence(confidence));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ConsistencyError> {
    if !value.is_object() {
        return Err(ConsistencyError::InvalidJsonObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_array_or_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ConsistencyError> {
    if !value.is_array() && !value.is_object() {
        return Err(ConsistencyError::InvalidJsonArrayOrObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_refresh_limit(limit: i64) -> i64 {
    limit.clamp(MIN_REFRESH_LIMIT, MAX_REFRESH_LIMIT)
}
```

### `backend/src/engines/context_packs/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/context_packs/errors.rs`
- Size bytes / Размер в байтах: `642`
- Included characters / Включено символов: `642`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextPackStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("context pack sources are required")]
    MissingSources,

    #[error("unknown context pack kind stored in database: {0}")]
    UnknownContextPackKind(String),

    #[error("unknown context pack source kind stored in database: {0}")]
    UnknownContextPackSourceKind(String),
}
```

### `backend/src/engines/context_packs/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/context_packs/mod.rs`
- Size bytes / Размер в байтах: `247`
- Included characters / Включено символов: `247`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod models;
mod store;

pub use errors::ContextPackStoreError;
pub use models::{
    ContextPack, ContextPackKind, ContextPackSource, ContextPackSourceKind, NewContextPack,
    NewContextPackSource,
};
pub use store::ContextPackStore;
```

### `backend/src/engines/context_packs/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/context_packs/models.rs`
- Size bytes / Размер в байтах: `6117`
- Included characters / Включено символов: `6117`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::ContextPackStoreError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPackKind {
    Persona,
    Meeting,
    Task,
    Calendar,
    Project,
}

impl ContextPackKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Persona => "persona",
            Self::Meeting => "meeting",
            Self::Task => "task",
            Self::Calendar => "calendar",
            Self::Project => "project",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ContextPackStoreError> {
        match value.as_ref() {
            "persona" => Ok(Self::Persona),
            "meeting" => Ok(Self::Meeting),
            "task" => Ok(Self::Task),
            "calendar" => Ok(Self::Calendar),
            "project" => Ok(Self::Project),
            unknown => Err(ContextPackStoreError::UnknownContextPackKind(
                unknown.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPackSourceKind {
    Observation,
    DomainEntity,
    Knowledge,
    Relationship,
    Decision,
    Task,
    Obligation,
    Document,
    CalendarEvent,
    Project,
}

impl ContextPackSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::DomainEntity => "domain_entity",
            Self::Knowledge => "knowledge",
            Self::Relationship => "relationship",
            Self::Decision => "decision",
            Self::Task => "task",
            Self::Obligation => "obligation",
            Self::Document => "document",
            Self::CalendarEvent => "calendar_event",
            Self::Project => "project",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ContextPackStoreError> {
        match value.as_ref() {
            "observation" => Ok(Self::Observation),
            "domain_entity" => Ok(Self::DomainEntity),
            "knowledge" => Ok(Self::Knowledge),
            "relationship" => Ok(Self::Relationship),
            "decision" => Ok(Self::Decision),
            "task" => Ok(Self::Task),
            "obligation" => Ok(Self::Obligation),
            "document" => Ok(Self::Document),
            "calendar_event" => Ok(Self::CalendarEvent),
            "project" => Ok(Self::Project),
            unknown => Err(ContextPackStoreError::UnknownContextPackSourceKind(
                unknown.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContextPack {
    pub context_pack_id: String,
    pub kind: ContextPackKind,
    pub subject_id: String,
    pub content: Value,
    pub metadata: Value,
    pub rebuildable: bool,
    pub built_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewContextPack {
    pub kind: ContextPackKind,
    pub subject_id: String,
    pub content: Value,
    pub metadata: Value,
    pub rebuildable: bool,
}

impl NewContextPack {
    pub fn new(kind: ContextPackKind, subject_id: impl Into<String>, content: Value) -> Self {
        Self {
            kind,
            subject_id: subject_id.into(),
            content,
            metadata: json!({}),
            rebuildable: true,
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn rebuildable(mut self, rebuildable: bool) -> Self {
        self.rebuildable = rebuildable;
        self
    }

    pub fn validate(&self) -> Result<(), ContextPackStoreError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_json_object("content", &self.content)?;
        validate_json_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContextPackSource {
    pub context_pack_id: String,
    pub source_kind: ContextPackSourceKind,
    pub source_id: String,
    pub role: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewContextPackSource {
    pub source_kind: ContextPackSourceKind,
    pub source_id: String,
    pub role: String,
    pub metadata: Value,
}

impl NewContextPackSource {
    pub fn new(source_kind: ContextPackSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            role: "source".to_owned(),
            metadata: json!({}),
        }
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> Result<(), ContextPackStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("role", &self.role)?;
        validate_json_object("metadata", &self.metadata)?;
        Ok(())
    }
}

pub(super) fn validate_context_pack_with_sources(
    pack: &NewContextPack,
    sources: &[NewContextPackSource],
) -> Result<(), ContextPackStoreError> {
    pack.validate()?;
    if sources.is_empty() {
        return Err(ContextPackStoreError::MissingSources);
    }
    for source in sources {
        source.validate()?;
    }
    Ok(())
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ContextPackStoreError> {
    if value.trim().is_empty() {
        return Err(ContextPackStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ContextPackStoreError> {
    if !value.is_object() {
        return Err(ContextPackStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
```

### `backend/src/engines/context_packs/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/context_packs/store.rs`
- Size bytes / Размер в байтах: `6785`
- Included characters / Включено символов: `6785`
- Truncated / Обрезано: `no`

```rust
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Row, Transaction};

use super::errors::ContextPackStoreError;
use super::models::{
    ContextPack, ContextPackKind, ContextPackSource, ContextPackSourceKind, NewContextPack,
    NewContextPackSource, validate_context_pack_with_sources, validate_non_empty,
};

#[derive(Clone)]
pub struct ContextPackStore {
    pool: PgPool,
}

impl ContextPackStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(
        &self,
        kind: ContextPackKind,
        subject_id: &str,
    ) -> Result<Option<ContextPack>, ContextPackStoreError> {
        validate_non_empty("subject_id", subject_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                context_pack_id,
                kind,
                subject_id,
                content,
                metadata,
                rebuildable,
                built_at,
                updated_at
            FROM context_packs
            WHERE kind = $1 AND subject_id = $2
            LIMIT 1
            "#,
        )
        .bind(kind.as_str())
        .bind(subject_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_context_pack).transpose()
    }

    pub async fn exists(
        &self,
        kind: ContextPackKind,
        subject_id: &str,
    ) -> Result<bool, ContextPackStoreError> {
        validate_non_empty("subject_id", subject_id)?;
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM context_packs
                WHERE kind = $1 AND subject_id = $2
            )
            "#,
        )
        .bind(kind.as_str())
        .bind(subject_id.trim())
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    pub async fn upsert_with_sources(
        &self,
        pack: &NewContextPack,
        sources: &[NewContextPackSource],
    ) -> Result<ContextPack, ContextPackStoreError> {
        validate_context_pack_with_sources(pack, sources)?;

        let mut transaction = self.pool.begin().await?;
        let stored = self
            .upsert_with_sources_in_transaction(&mut transaction, pack, sources)
            .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_sources(
        &self,
        context_pack_id: &str,
    ) -> Result<Vec<ContextPackSource>, ContextPackStoreError> {
        validate_non_empty("context_pack_id", context_pack_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                context_pack_id,
                source_kind,
                source_id,
                role,
                metadata,
                created_at
            FROM context_pack_sources
            WHERE context_pack_id = $1
            ORDER BY role ASC, source_kind ASC, source_id ASC
            "#,
        )
        .bind(context_pack_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_context_pack_source).collect()
    }

    async fn upsert_with_sources_in_transaction(
        &self,
        transaction: &mut Transaction<'_, sqlx::Postgres>,
        pack: &NewContextPack,
        sources: &[NewContextPackSource],
    ) -> Result<ContextPack, ContextPackStoreError> {
        let context_pack_id = context_pack_id(pack)?;
        let row = sqlx::query(
            r#"
            INSERT INTO context_packs (
                context_pack_id,
                kind,
                subject_id,
                content,
                metadata,
                rebuildable
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (kind, subject_id)
            DO UPDATE SET
                content = EXCLUDED.content,
                metadata = EXCLUDED.metadata,
                rebuildable = EXCLUDED.rebuildable,
                built_at = now(),
                updated_at = now()
            RETURNING
                context_pack_id,
                kind,
                subject_id,
                content,
                metadata,
                rebuildable,
                built_at,
                updated_at
            "#,
        )
        .bind(&context_pack_id)
        .bind(pack.kind.as_str())
        .bind(pack.subject_id.trim())
        .bind(&pack.content)
        .bind(&pack.metadata)
        .bind(pack.rebuildable)
        .fetch_one(&mut **transaction)
        .await?;
        let stored = row_to_context_pack(row)?;

        sqlx::query("DELETE FROM context_pack_sources WHERE context_pack_id = $1")
            .bind(&stored.context_pack_id)
            .execute(&mut **transaction)
            .await?;

        for source in sources {
            sqlx::query(
                r#"
                INSERT INTO context_pack_sources (
                    context_pack_id,
                    source_kind,
                    source_id,
                    role,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(&stored.context_pack_id)
            .bind(source.source_kind.as_str())
            .bind(source.source_id.trim())
            .bind(source.role.trim())
            .bind(&source.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        Ok(stored)
    }
}

fn row_to_context_pack(row: PgRow) -> Result<ContextPack, ContextPackStoreError> {
    let kind: String = row.try_get("kind")?;
    Ok(ContextPack {
        context_pack_id: row.try_get("context_pack_id")?,
        kind: ContextPackKind::parse(kind)?,
        subject_id: row.try_get("subject_id")?,
        content: row.try_get("content")?,
        metadata: row.try_get("metadata")?,
        rebuildable: row.try_get("rebuildable")?,
        built_at: row.try_get("built_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_context_pack_source(row: PgRow) -> Result<ContextPackSource, ContextPackStoreError> {
    let source_kind: String = row.try_get("source_kind")?;
    Ok(ContextPackSource {
        context_pack_id: row.try_get("context_pack_id")?,
        source_kind: ContextPackSourceKind::parse(source_kind)?,
        source_id: row.try_get("source_id")?,
        role: row.try_get("role")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}

fn context_pack_id(pack: &NewContextPack) -> Result<String, ContextPackStoreError> {
    let mut digest = Sha256::new();
    digest.update(pack.kind.as_str().as_bytes());
    digest.update(b"\n");
    digest.update(pack.subject_id.trim().as_bytes());
    Ok(format!("context_pack:v1:{:x}", digest.finalize()))
}
```

### `backend/src/engines/enrichment/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/enrichment/engine.rs`
- Size bytes / Размер в байтах: `2355`
- Included characters / Включено символов: `2355`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use crate::engines::enrichment::errors::EnrichmentEngineError;
use crate::engines::enrichment::models::{
    EnrichmentCandidateDraft, PreferenceDraft, validate_confidence, validate_non_empty,
};

pub struct EnrichmentEngine;

impl EnrichmentEngine {
    pub fn persona_favorite_preference(
        person_id: &str,
        is_favorite: bool,
    ) -> Option<PreferenceDraft> {
        if !is_favorite {
            return None;
        }

        Some(PreferenceDraft {
            preference_type: "ui:favorite".to_owned(),
            value: "true".to_owned(),
            source: format!("persons.is_favorite:{person_id}"),
            confidence: 1.0,
        })
    }

    pub fn persona_observation_candidate(
        person_id: &str,
        source: &str,
        extracted_claim: &str,
        data: Value,
        confidence: f64,
    ) -> Result<EnrichmentCandidateDraft, EnrichmentEngineError> {
        validate_non_empty("affected entity", person_id)?;
        validate_non_empty("source", source)?;
        validate_non_empty("extracted claim", extracted_claim)?;
        validate_confidence(confidence)?;

        let Value::Object(mut data_object) = data else {
            return Err(EnrichmentEngineError::InvalidData);
        };
        let conflict_marker = data_object
            .get("conflict_marker")
            .or_else(|| data_object.get("conflict"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        data_object.insert(
            "_enrichment".to_owned(),
            json!({
                "affected_entity_kind": "persona",
                "affected_entity_id": person_id,
                "extracted_claim": extracted_claim,
                "source": source,
                "review_state": "pending",
                "freshness": "current",
                "conflict_marker": conflict_marker,
            }),
        );

        Ok(EnrichmentCandidateDraft {
            entity_kind: "persona".to_owned(),
            entity_id: person_id.to_owned(),
            source: source.to_owned(),
            extracted_claim: extracted_claim.to_owned(),
            data: Value::Object(data_object),
            confidence,
            review_state: "pending".to_owned(),
            freshness: "current".to_owned(),
            conflict_marker,
        })
    }
}
```
