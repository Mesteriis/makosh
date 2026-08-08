use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

use super::errors::MailSyncError;
use super::models::runs::MailSyncRun;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;

pub(super) async fn capture_mail_sync_run_observation(
    transaction: &mut Transaction<'_, Postgres>,
    run: &MailSyncRun,
    kind_code: &str,
    relationship_kind: &str,
    observed_at: DateTime<Utc>,
    actor: &str,
) -> Result<(), MailSyncError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            kind_code,
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "run_id": run.run_id,
                "account_id": run.account_id,
                "trigger": run.trigger,
                "status": run.status,
                "phase": run.phase,
                "progress_mode": run.progress_mode,
                "progress_percent": run.progress_percent,
                "processed_messages": run.processed_messages,
                "estimated_total_messages": run.estimated_total_messages,
                "current_batch_size": run.current_batch_size,
                "fetched_messages": run.fetched_messages,
                "projected_messages": run.projected_messages,
                "upserted_personas": run.upserted_personas,
                "upserted_organizations": run.upserted_organizations,
                "checkpoint_before": run.checkpoint_before,
                "checkpoint_after": run.checkpoint_after,
                "checkpoint_saved": run.checkpoint_saved,
                "error_code": run.error_code,
                "error_message": run.error_message,
                "started_at": run.started_at,
                "completed_at": run.completed_at,
                "next_run_at": run.next_run_at,
                "operation": relationship_kind,
            }),
            format!("mail-sync-run://{}/{}", run.run_id, relationship_kind),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "mail_sync_run",
        run.run_id.clone(),
        relationship_kind,
        json!({
            "account_id": run.account_id,
            "status": run.status,
            "phase": run.phase,
            "progress_mode": run.progress_mode,
        }),
        None,
    )
    .await?;
    Ok(())
}
