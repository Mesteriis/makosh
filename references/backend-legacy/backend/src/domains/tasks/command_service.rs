use chrono::Utc;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::api::{NewTask, Task, TaskError, TaskStore, TaskUpdate};
use super::core::checklists::{TaskChecklist, TaskChecklistStore};
use super::core::context_packs::TaskContextPackStore;
use super::core::errors::TaskCoreError;
use super::core::evidence::{TaskEvidence, TaskEvidenceStore};
use super::core::relations::TaskRelationStore;
use super::core::subtasks::{TaskSubtask, TaskSubtaskStore};
use super::intelligence::TaskIntelligenceService;

#[derive(Clone)]
pub struct TaskCommandService {
    pool: PgPool,
}

impl TaskCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_task_manual(&self, req: &NewTask) -> Result<Task, TaskCommandServiceError> {
        let resolved = self.resolve_provenance(req).await?;
        let mut transaction = self.pool.begin().await?;
        let task = TaskStore::new(self.pool.clone())
            .create_in_transaction(&mut transaction, &resolved)
            .await?;
        transaction.commit().await?;
        Ok(task)
    }

    pub async fn update_task_manual(
        &self,
        task_id: &str,
        update: &TaskUpdate,
    ) -> Result<Task, TaskCommandServiceError> {
        let task_store = TaskStore::new(self.pool.clone());
        task_store.get(task_id).await?.ok_or(TaskError::NotFound)?;

        let observation = self
            .capture_observation(
                "task update",
                ObservationOriginKind::Manual,
                json!({
                    "task_id": task_id,
                    "title": update.title.as_deref(),
                    "description": update.description.as_deref(),
                    "makosh_status": update.makosh_status.as_deref(),
                    "priority_score": update.priority_score,
                    "risk_score": update.risk_score,
                    "readiness_score": update.readiness_score,
                    "area": update.area.as_deref(),
                    "why": update.why.as_deref(),
                    "outcome": update.outcome.as_deref(),
                    "due_at": update.due_at,
                    "waiting_reason": update.waiting_reason.as_deref(),
                    "energy_type": update.energy_type.as_deref(),
                    "confidentiality": update.confidentiality.as_deref(),
                    "tags": update.tags.clone(),
                    "task_metadata": update.task_metadata.clone(),
                    "linked_person_id": update.linked_person_id.as_deref(),
                    "linked_organization_id": update.linked_organization_id.as_deref(),
                    "completed_at": update.completed_at,
                }),
                format!("task://{task_id}/update"),
                json!({
                    "captured_by": "tasks_service.update_task_manual",
                    "operation": "update_task_manual",
                }),
            )
            .await?;

        Ok(task_store
            .update_with_observation(
                task_id,
                update,
                &observation.observation_id,
                "task_update",
                json!({}),
            )
            .await?)
    }

    pub async fn set_status_manual(
        &self,
        task_id: &str,
        status: &str,
    ) -> Result<Task, TaskCommandServiceError> {
        let task_store = TaskStore::new(self.pool.clone());
        task_store.get(task_id).await?.ok_or(TaskError::NotFound)?;

        let observation = self
            .capture_observation(
                "task status",
                ObservationOriginKind::Manual,
                json!({
                    "task_id": task_id,
                    "status": status,
                    "operation": "set_status",
                }),
                format!("task://{task_id}/status"),
                json!({
                    "captured_by": "tasks_service.set_status_manual",
                    "operation": "set_status_manual",
                }),
            )
            .await?;

        Ok(task_store
            .set_status_with_observation(
                task_id,
                status,
                &observation.observation_id,
                "status_update",
                json!({
                    "status": status,
                }),
            )
            .await?)
    }

    pub async fn archive_manual(&self, task_id: &str) -> Result<Task, TaskCommandServiceError> {
        let task_store = TaskStore::new(self.pool.clone());
        task_store.get(task_id).await?.ok_or(TaskError::NotFound)?;

        let observation = self
            .capture_observation(
                "task archive",
                ObservationOriginKind::Manual,
                json!({
                    "task_id": task_id,
                    "status": "archived",
                    "operation": "archive",
                }),
                format!("task://{task_id}/archive"),
                json!({
                    "captured_by": "tasks_service.archive_manual",
                    "operation": "archive_manual",
                }),
            )
            .await?;

        Ok(task_store
            .archive_with_observation(
                task_id,
                &observation.observation_id,
                "status_update",
                json!({
                    "status": "archived",
                }),
            )
            .await?)
    }

    pub async fn analyze_runtime(
        &self,
        task_id: &str,
    ) -> Result<TaskAnalysisResult, TaskCommandServiceError> {
        let task = TaskStore::new(self.pool.clone())
            .get(task_id)
            .await?
            .ok_or(TaskError::NotFound)?;
        let has_ctx = TaskContextPackStore::new(self.pool.clone())
            .get(task_id)
            .await
            .map(|c| c.is_some())
            .unwrap_or(false);
        let _has_relations = TaskRelationStore::new(self.pool.clone())
            .list(task_id)
            .await
            .map(|r| !r.is_empty())
            .unwrap_or(false);
        let is_legal = task.area.as_deref() == Some("legal") || task.area.as_deref() == Some("tax");
        let is_tax = task.area.as_deref() == Some("tax");
        let has_persona_link = task.linked_person_id.is_some();
        let has_org = task.linked_organization_id.is_some();
        let priority = TaskIntelligenceService::calculate_priority(
            task.due_at,
            has_persona_link,
            has_org,
            task.project_id.is_some(),
            is_legal,
            is_tax,
            false,
        );
        let risk = TaskIntelligenceService::calculate_risk(
            task.due_at
                .map(|d| (d - Utc::now()).num_hours() < 24)
                .unwrap_or(false),
            false,
            false,
            false,
            is_legal,
            &task.title,
        );
        let readiness = TaskIntelligenceService::calculate_readiness(
            task.description.is_some(),
            has_ctx,
            false,
            task.due_at.is_some(),
            true,
            has_persona_link,
        );
        let missing_context = TaskIntelligenceService::detect_missing_context(
            task.description.is_some(),
            has_ctx,
            task.due_at.is_some(),
            has_persona_link,
            task.project_id.is_some(),
        );
        let next_action = TaskIntelligenceService::suggest_next_action(
            &task.makosh_status,
            false,
            false,
            task.waiting_reason.as_deref(),
        );
        let update = TaskUpdate {
            priority_score: Some(priority),
            risk_score: Some(risk),
            readiness_score: Some(readiness),
            ..Default::default()
        };

        let observation = self
            .capture_observation(
                "task analyze",
                ObservationOriginKind::LocalRuntime,
                json!({
                    "task_id": task_id,
                    "priority_score": priority,
                    "risk_score": risk,
                    "readiness_score": readiness,
                    "missing_context": missing_context,
                    "next_action": next_action,
                }),
                format!("task://{task_id}/analyze"),
                json!({
                    "captured_by": "tasks_service.analyze_runtime",
                    "operation": "analyze_runtime",
                    "engine": "task_intelligence",
                }),
            )
            .await?;

        TaskStore::new(self.pool.clone())
            .update_with_observation(
                task_id,
                &update,
                &observation.observation_id,
                "analysis_update",
                json!({}),
            )
            .await?;

        Ok(TaskAnalysisResult {
            priority,
            risk,
            readiness,
            missing_context,
            next_action,
        })
    }

    pub async fn add_evidence(
        &self,
        task_id: &str,
        requested_source_type: Option<&str>,
        requested_source_id: Option<&str>,
        quote: Option<String>,
        confidence: Option<f64>,
    ) -> Result<TaskEvidence, TaskCommandServiceError> {
        let requested_source_type = requested_source_type.unwrap_or("manual").trim();
        let requested_source_id = requested_source_id.map(str::trim);

        let (source_type, source_id) =
            if requested_source_type.is_empty() || requested_source_type == "manual" {
                let observation = self
                    .capture_observation(
                        "task evidence",
                        ObservationOriginKind::Manual,
                        json!({
                            "task_id": task_id,
                            "quote": quote,
                            "confidence": confidence,
                        }),
                        format!("task://{task_id}/evidence"),
                        json!({
                            "captured_by": "tasks_service.add_evidence",
                            "operation": "add_evidence",
                        }),
                    )
                    .await?;
                ("observation".to_owned(), observation.observation_id)
            } else {
                let source_id =
                    requested_source_id.ok_or(TaskCommandServiceError::MissingEvidenceSourceId)?;
                if source_id.is_empty() {
                    return Err(TaskCommandServiceError::MissingEvidenceSourceId);
                }
                (requested_source_type.to_owned(), source_id.to_owned())
            };

        Ok(TaskEvidenceStore::new(self.pool.clone())
            .add(
                task_id,
                &source_type,
                &source_id,
                quote.as_deref(),
                confidence,
            )
            .await?)
    }

    pub async fn set_checklist_manual(
        &self,
        task_id: &str,
        items: Value,
        requested_source: Option<&str>,
    ) -> Result<TaskChecklist, TaskCommandServiceError> {
        let requested_source = requested_source.unwrap_or("manual");
        let observation = self
            .capture_observation(
                "task checklist",
                ObservationOriginKind::Manual,
                json!({
                    "task_id": task_id,
                    "items": items,
                    "source": requested_source,
                }),
                format!("task://{task_id}/checklist"),
                json!({
                    "captured_by": "tasks_service.set_checklist_manual",
                    "operation": "set_checklist_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(TaskChecklistStore::new(self.pool.clone())
            .set(
                task_id,
                items,
                &format!("observation:{}", observation.observation_id),
            )
            .await?)
    }

    pub async fn add_subtask_manual(
        &self,
        parent_task_id: &str,
        child_task_id: &str,
        sort_order: i32,
    ) -> Result<TaskSubtask, TaskCommandServiceError> {
        let task_store = TaskStore::new(self.pool.clone());
        task_store
            .get(parent_task_id)
            .await?
            .ok_or(TaskError::NotFound)?;
        task_store
            .get(child_task_id)
            .await?
            .ok_or(TaskError::NotFound)?;

        let observation = self
            .capture_observation(
                "task subtask",
                ObservationOriginKind::Manual,
                json!({
                    "parent_task_id": parent_task_id,
                    "child_task_id": child_task_id,
                    "sort_order": sort_order,
                }),
                format!("task://{parent_task_id}/subtask/{child_task_id}"),
                json!({
                    "captured_by": "tasks_service.add_subtask_manual",
                    "operation": "add_subtask_manual",
                }),
            )
            .await?;

        Ok(TaskSubtaskStore::new(self.pool.clone())
            .add_with_source(
                parent_task_id,
                child_task_id,
                sort_order,
                &format!("observation:{}", observation.observation_id),
            )
            .await?)
    }

    async fn capture_observation(
        &self,
        operation: &'static str,
        origin_kind: ObservationOriginKind,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<makosh_observations_api::models::Observation, TaskCommandServiceError> {
        ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "TASK_MUTATION",
                    origin_kind,
                    Utc::now(),
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await
            .map_err(|source| TaskCommandServiceError::ObservationCapture { operation, source })
    }

    async fn resolve_provenance(&self, req: &NewTask) -> Result<NewTask, TaskCommandServiceError> {
        let provenance_kind: Option<String> = req.provenance_kind.as_deref().map(normalize_kind);
        let provenance_id = req
            .provenance_id
            .clone()
            .map(|value| value.trim().to_owned());

        if provenance_kind.is_some() || provenance_id.is_some() {
            let provenance_kind = provenance_kind.ok_or(TaskError::InvalidProvenanceSpec)?;
            let provenance_id = provenance_id.ok_or(TaskError::InvalidProvenanceSpec)?;

            if provenance_id.is_empty() {
                return Err(TaskError::InvalidProvenanceSpec.into());
            }

            match provenance_kind.as_str() {
                "observation" => {
                    let exists = ObservationStore::new(self.pool.clone())
                        .get(&provenance_id)
                        .await?;
                    if exists.is_none() {
                        return Err(TaskError::MissingProvenanceObservation.into());
                    }
                }
                "review_item" | "decision" | "obligation" => {
                    if !self
                        .provenance_target_exists(&provenance_kind, &provenance_id)
                        .await?
                    {
                        return Err(TaskError::MissingProvenanceReference.into());
                    }
                }
                _ => return Err(TaskError::UnknownProvenanceKind.into()),
            }

            let resolved = req
                .clone()
                .with_provenance(&provenance_kind, &provenance_id);

            if provenance_kind == "observation" {
                return Ok(resolved.with_observation_source(provenance_id));
            }

            if let Some(observation_id) = self
                .find_observation_id_for_provenance(&provenance_kind, &provenance_id)
                .await?
            {
                return Ok(resolved.with_observation_source(observation_id));
            }

            return Err(TaskError::MissingProvenanceEvidence.into());
        }

        if let (Some(source_kind), Some(source_id)) = (&req.source_kind, &req.source_id) {
            let source_kind = normalize_kind(source_kind);
            let source_id = source_id.trim().to_owned();

            if source_id.is_empty() {
                return Err(TaskError::MissingSourceIdentifier.into());
            }

            let provenance_id = match self
                .find_observation_id_for_source(&source_kind, &source_id)
                .await?
            {
                Some(observation_id) => observation_id,
                None => self.seed_observation_from_task(req).await?.observation_id,
            };

            if source_kind == "manual" || source_kind == "observation" {
                return Ok(req.clone().with_observation_source(provenance_id));
            }

            return Ok(req
                .clone()
                .with_provenance("observation", provenance_id)
                .with_source_defaults(source_kind, source_id));
        }

        if req.source_kind.is_some() || req.source_id.is_some() {
            return Err(TaskError::MissingSourceIdentifier.into());
        }

        let observation = self.seed_observation_from_task(req).await?;

        Ok(req
            .clone()
            .with_provenance("observation", observation.observation_id.clone())
            .with_observation_source(observation.observation_id))
    }

    async fn provenance_target_exists(
        &self,
        provenance_kind: &str,
        provenance_id: &str,
    ) -> Result<bool, TaskCommandServiceError> {
        let row_count = match provenance_kind {
            "review_item" => sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM review_items WHERE review_item_id = $1",
            )
            .bind(provenance_id),
            "decision" => sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM decisions WHERE decision_id = $1",
            )
            .bind(provenance_id),
            "obligation" => sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM obligations WHERE obligation_id = $1",
            )
            .bind(provenance_id),
            _ => unreachable!("unsupported provenance kind"),
        }
        .fetch_one(&self.pool)
        .await?;

        Ok(row_count > 0)
    }

    async fn find_observation_id_for_source(
        &self,
        source_kind: &str,
        source_id: &str,
    ) -> Result<Option<String>, TaskCommandServiceError> {
        let observation_id =
            match source_kind.trim() {
                "observation" => {
                    return match ObservationStore::new(self.pool.clone())
                        .get(source_id)
                        .await?
                    {
                        Some(observation) => Ok(Some(observation.observation_id)),
                        None => Ok(None),
                    };
                }
                "message" | "communication_message" | "email" | "telegram" | "whatsapp" => {
                    select_observation_id(
                        &self.pool,
                        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
                        source_id,
                    )
                    .await?
                }
                "document" => {
                    select_observation_id(
                        &self.pool,
                        "SELECT observation_id FROM documents WHERE document_id = $1",
                        source_id,
                    )
                    .await?
                }
                "raw_record" | "communication_raw_record" => select_observation_id(
                    &self.pool,
                    "SELECT observation_id FROM communication_raw_records WHERE raw_record_id = $1",
                    source_id,
                )
                .await?,
                "task" => {
                    select_observation_id(
                        &self.pool,
                        "SELECT observation_id FROM tasks WHERE task_id = $1",
                        source_id,
                    )
                    .await?
                }
                _ => None,
            };

        Ok(observation_id)
    }

    async fn find_observation_id_for_provenance(
        &self,
        provenance_kind: &str,
        provenance_id: &str,
    ) -> Result<Option<String>, TaskCommandServiceError> {
        let observation_id = match provenance_kind {
            "review_item" => {
                select_observation_id(
                    &self.pool,
                    r#"
                SELECT observation_id
                FROM review_item_evidence
                WHERE review_item_id = $1
                ORDER BY created_at ASC, observation_id ASC
                LIMIT 1
                "#,
                    provenance_id,
                )
                .await?
            }
            "decision" => {
                select_observation_id(
                    &self.pool,
                    r#"
                SELECT observation_id
                FROM decision_evidence
                WHERE decision_id = $1
                  AND source_kind = 'observation'
                  AND observation_id IS NOT NULL
                ORDER BY created_at ASC, source_id ASC
                LIMIT 1
                "#,
                    provenance_id,
                )
                .await?
            }
            "obligation" => {
                select_observation_id(
                    &self.pool,
                    r#"
                SELECT observation_id
                FROM obligation_evidence
                WHERE obligation_id = $1
                  AND source_kind = 'observation'
                  AND observation_id IS NOT NULL
                ORDER BY created_at ASC, source_id ASC
                LIMIT 1
                "#,
                    provenance_id,
                )
                .await?
            }
            _ => None,
        };

        Ok(observation_id)
    }

    async fn seed_observation_from_task(
        &self,
        req: &NewTask,
    ) -> Result<makosh_observations_api::models::Observation, TaskCommandServiceError> {
        let title_words = req
            .title
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        let source_ref = if title_words.is_empty() {
            format!(
                "manual://tasks/{:x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            )
        } else {
            format!("manual://tasks/{title_words}")
        };

        self.capture_observation(
            "task create",
            ObservationOriginKind::Manual,
            json!({
                "task_title": req.title,
                "task_description": req.description,
                "captured_from": "task_create",
            }),
            source_ref,
            json!({
                "created_by": "tasks",
                "mode": "manual_task_seed",
            }),
        )
        .await
    }
}

#[derive(Debug, Serialize)]
pub struct TaskAnalysisResult {
    pub priority: f64,
    pub risk: f64,
    pub readiness: f64,
    pub missing_context: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Error)]
pub enum TaskCommandServiceError {
    #[error("{operation} observation capture failed")]
    ObservationCapture {
        operation: &'static str,
        #[source]
        source: ObservationStoreError,
    },

    #[error("task evidence source id is required")]
    MissingEvidenceSourceId,

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Task(#[from] TaskError),

    #[error(transparent)]
    Core(#[from] TaskCoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

fn normalize_kind(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

async fn select_observation_id(
    pool: &PgPool,
    sql: &str,
    source_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(sql)
        .bind(source_id)
        .fetch_optional(pool)
        .await
}
