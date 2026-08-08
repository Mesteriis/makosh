use super::rows::row_to_checkpoint;
use super::store::CommunicationIngestionStore;
use super::validation::validate_non_empty;
use crate::errors::CommunicationIngestionError;
use makosh_communications_api::evidence::{
    CommunicationEvidencePortError, CommunicationEvidencePortFuture, IngestionCheckpoint,
    IngestionCheckpointCommandPort, IngestionCheckpointQueryPort, NewIngestionCheckpoint,
};

impl CommunicationIngestionStore {
    pub async fn save_checkpoint(
        &self,
        checkpoint: &NewIngestionCheckpoint,
    ) -> Result<IngestionCheckpoint, CommunicationIngestionError> {
        checkpoint.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_ingestion_checkpoints (
                account_id,
                stream_id,
                checkpoint,
                updated_at
            )
            VALUES ($1, $2, $3, now())
            ON CONFLICT (account_id, stream_id)
            DO UPDATE SET
                checkpoint = EXCLUDED.checkpoint,
                updated_at = now()
            RETURNING
                account_id,
                stream_id,
                checkpoint,
                updated_at
            "#,
        )
        .bind(checkpoint.account_id.trim())
        .bind(checkpoint.stream_id.trim())
        .bind(&checkpoint.checkpoint)
        .fetch_one(&self.pool)
        .await?;

        row_to_checkpoint(row)
    }

    pub async fn checkpoint(
        &self,
        account_id: &str,
        stream_id: &str,
    ) -> Result<Option<IngestionCheckpoint>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("stream_id", stream_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                stream_id,
                checkpoint,
                updated_at
            FROM communication_ingestion_checkpoints
            WHERE account_id = $1
              AND stream_id = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(stream_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_checkpoint).transpose()
    }

    pub async fn delete_checkpoint(
        &self,
        account_id: &str,
        stream_id: &str,
    ) -> Result<bool, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("stream_id", stream_id)?;

        let result = sqlx::query(
            r#"
            DELETE FROM communication_ingestion_checkpoints
            WHERE account_id = $1
              AND stream_id = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(stream_id.trim())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_checkpoints_with_stream_prefix(
        &self,
        account_id: &str,
        stream_prefix: &str,
    ) -> Result<u64, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("stream_prefix", stream_prefix)?;

        let result = sqlx::query(
            r#"
            DELETE FROM communication_ingestion_checkpoints
            WHERE account_id = $1
              AND stream_id LIKE $2
            "#,
        )
        .bind(account_id.trim())
        .bind(format!("{}%", stream_prefix.trim()))
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

impl IngestionCheckpointQueryPort for CommunicationIngestionStore {
    fn checkpoint<'a>(
        &'a self,
        account_id: &'a str,
        stream_id: &'a str,
    ) -> CommunicationEvidencePortFuture<'a, Option<IngestionCheckpoint>> {
        Box::pin(async move {
            CommunicationIngestionStore::checkpoint(self, account_id, stream_id)
                .await
                .map_err(CommunicationEvidencePortError::new)
        })
    }
}

impl IngestionCheckpointCommandPort for CommunicationIngestionStore {
    fn save_checkpoint<'a>(
        &'a self,
        checkpoint: &'a NewIngestionCheckpoint,
    ) -> CommunicationEvidencePortFuture<'a, IngestionCheckpoint> {
        Box::pin(async move {
            CommunicationIngestionStore::save_checkpoint(self, checkpoint)
                .await
                .map_err(CommunicationEvidencePortError::new)
        })
    }

    fn delete_checkpoint<'a>(
        &'a self,
        account_id: &'a str,
        stream_id: &'a str,
    ) -> CommunicationEvidencePortFuture<'a, bool> {
        Box::pin(async move {
            CommunicationIngestionStore::delete_checkpoint(self, account_id, stream_id)
                .await
                .map_err(CommunicationEvidencePortError::new)
        })
    }

    fn delete_checkpoints_with_stream_prefix<'a>(
        &'a self,
        account_id: &'a str,
        stream_prefix: &'a str,
    ) -> CommunicationEvidencePortFuture<'a, u64> {
        Box::pin(async move {
            CommunicationIngestionStore::delete_checkpoints_with_stream_prefix(
                self,
                account_id,
                stream_prefix,
            )
            .await
            .map_err(CommunicationEvidencePortError::new)
        })
    }
}
