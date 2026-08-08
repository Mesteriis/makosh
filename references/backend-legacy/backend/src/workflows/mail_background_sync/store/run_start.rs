use serde_json::{Value, json};

use super::super::errors::MailSyncError;
use super::super::events::sync_run_started_event;
use super::super::evidence::capture_mail_sync_run_observation;
use super::super::models::progress::MailSyncTrigger;
use super::super::models::runs::MailSyncRun;
use super::super::models::settings::MailSyncSettings;
use super::super::rows::row_to_run;
use super::super::validation::{mail_sync_run_id, validate_account_id};
use super::MailSyncStore;
use makosh_events_postgres::store::EventStore;

impl MailSyncStore {
    pub(in crate::workflows::mail_background_sync) async fn start_run(
        &self,
        account_id: &str,
        trigger: MailSyncTrigger,
        settings: &MailSyncSettings,
        checkpoint_before: Option<Value>,
    ) -> Result<MailSyncRun, MailSyncError> {
        validate_account_id(account_id)?;
        let run_id = mail_sync_run_id(account_id);
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            INSERT INTO communication_mail_sync_runs (
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                current_batch_size,
                checkpoint_before
            )
            VALUES ($1, $2, $3, 'running', 'listing', 'indeterminate', $4, $5)
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_personas,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(&run_id)
        .bind(account_id.trim())
        .bind(trigger.as_str())
        .bind(settings.batch_size)
        .bind(checkpoint_before.unwrap_or_else(|| json!({})))
        .fetch_one(&mut *transaction)
        .await;

        match result {
            Ok(row) => {
                let run = row_to_run(row)?;
                capture_mail_sync_run_observation(
                    &mut transaction,
                    &run,
                    "COMMUNICATION_MAIL_SYNC_RUN",
                    "started",
                    run.started_at,
                    "mail.background_sync.start_run",
                )
                .await?;
                let event = sync_run_started_event(&run)?;
                EventStore::append_in_transaction(&mut transaction, &event).await?;
                transaction.commit().await?;
                Ok(run)
            }
            Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
                Err(MailSyncError::RunAlreadyActive)
            }
            Err(error) => Err(MailSyncError::Sqlx(error)),
        }
    }
}
