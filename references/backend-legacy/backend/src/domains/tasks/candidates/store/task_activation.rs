use sqlx::Postgres;
use sqlx::Transaction;

use crate::domains::tasks::candidates::errors::TaskCandidateError;
use crate::domains::tasks::candidates::ids::task_id_from_candidate;
use crate::domains::tasks::candidates::models::StoredCandidateRow;

pub(super) async fn upsert_task_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    event_id: &str,
    actor_id: &str,
) -> Result<(), TaskCandidateError> {
    let observation_id = candidate
        .observation_id
        .clone()
        .or_else(|| {
            (candidate.source_kind == "observation").then(|| candidate.source_id.clone())
        })
        .ok_or_else(|| {
            TaskCandidateError::ObservationRequired(format!(
                "task candidate {task_candidate_id} has source_kind={} and no canonical observation_id",
                candidate.source_kind
            ))
        })?;

    sqlx::query(
        r#"
        INSERT INTO tasks (
            task_id,
            task_candidate_id,
            title,
            provenance_kind,
            provenance_id,
            source_kind,
            source_id,
            source_type,
            project_id,
            status,
            makosh_status,
            created_from_event_id,
            created_by_actor_id
        )
        VALUES ($1, $2, $3, 'observation', $4, $5, $6, $7, $8, 'active', 'ready', $9, $10)
        ON CONFLICT (task_candidate_id)
        DO UPDATE SET
            title = EXCLUDED.title,
            provenance_kind = EXCLUDED.provenance_kind,
            provenance_id = EXCLUDED.provenance_id,
            source_kind = EXCLUDED.source_kind,
            source_id = EXCLUDED.source_id,
            source_type = EXCLUDED.source_type,
            project_id = EXCLUDED.project_id,
            status = EXCLUDED.status,
            makosh_status = EXCLUDED.makosh_status,
            created_from_event_id = EXCLUDED.created_from_event_id,
            created_by_actor_id = EXCLUDED.created_by_actor_id,
            updated_at = now()
        "#,
    )
    .bind(task_id_from_candidate(task_candidate_id))
    .bind(task_candidate_id)
    .bind(&candidate.title)
    .bind(&observation_id)
    .bind("observation")
    .bind(&observation_id)
    .bind("observation")
    .bind(&candidate.project_id)
    .bind(event_id)
    .bind(actor_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
