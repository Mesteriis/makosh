use chrono::Utc;

use super::super::errors::MailSyncError;
use super::super::events::sync_run_progress_event;
use super::super::evidence::capture_mail_sync_run_observation;
use super::super::models::progress::ProgressUpdate;
use super::super::rows::row_to_run;
use super::MailSyncStore;
use makosh_events_postgres::store::EventStore;

impl MailSyncStore {
    pub(in crate::workflows::mail_background_sync) async fn update_progress(
        &self,
        update: ProgressUpdate<'_>,
    ) -> Result<(), MailSyncError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'running',
                phase = $2,
                progress_mode = $3,
                progress_percent = $4,
                processed_messages = $5,
                estimated_total_messages = $6,
                current_batch_size = $7,
                updated_at = now()
            WHERE run_id = $1
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
        .bind(update.run_id)
        .bind(update.phase.as_str())
        .bind(update.progress_mode.as_str())
        .bind(update.progress_percent)
        .bind(update.processed_messages)
        .bind(update.estimated_total_messages)
        .bind(update.current_batch_size)
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(row) = row {
            let run = row_to_run(row)?;
            capture_mail_sync_run_observation(
                &mut transaction,
                &run,
                "COMMUNICATION_MAIL_SYNC_RUN_STATUS",
                "progress",
                Utc::now(),
                "mail.background_sync.update_progress",
            )
            .await?;
            let event = sync_run_progress_event(&run)?;
            EventStore::append_in_transaction(&mut transaction, &event).await?;
        }
        transaction.commit().await?;

        Ok(())
    }

    pub(in crate::workflows::mail_background_sync) async fn mark_recoverable_full_resync(
        &self,
        run_id: &str,
        error_code: &'static str,
    ) -> Result<(), MailSyncError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'recoverable_full_resync_needed',
                phase = 'listing',
                progress_mode = 'indeterminate',
                error_code = $2,
                error_message = 'Gmail history expired; restarting full mailbox listing',
                updated_at = now()
            WHERE run_id = $1
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
        .bind(run_id)
        .bind(error_code)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let run = row_to_run(row)?;
            capture_mail_sync_run_observation(
                &mut transaction,
                &run,
                "COMMUNICATION_MAIL_SYNC_RUN_STATUS",
                "recoverable_full_resync_needed",
                Utc::now(),
                "mail.background_sync.mark_recoverable_full_resync",
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(())
    }
}
