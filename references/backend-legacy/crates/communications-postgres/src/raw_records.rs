use super::rows::row_to_raw_record;
use super::store::CommunicationIngestionStore;
use super::validation::validate_non_empty;
use crate::errors::CommunicationIngestionError;
use chrono::Utc;
use makosh_communications_api::evidence::{
    CommunicationEvidencePortError, CommunicationEvidencePortFuture,
    CommunicationRawEvidenceCommandPort, NewRawCommunicationRecord, StoredRawCommunicationRecord,
};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::json;

impl CommunicationIngestionStore {
    pub async fn record_raw_source(
        &self,
        record: &NewRawCommunicationRecord,
    ) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
        record.validate()?;

        let mut transaction = self.pool.begin().await?;
        let existing = sqlx::query(raw_record_by_id_sql())
            .bind(record.raw_record_id.trim())
            .fetch_optional(&mut *transaction)
            .await?;
        if let Some(row) = existing {
            let stored = row_to_raw_record(row)?;
            transaction.commit().await?;
            return Ok(stored);
        }

        let existing_observation_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT observation_id
            FROM communication_raw_records
            WHERE account_id = $1
              AND record_kind = $2
              AND provider_record_id = $3
            ORDER BY captured_at ASC, raw_record_id ASC
            LIMIT 1
            FOR UPDATE
            "#,
        )
        .bind(record.account_id.trim())
        .bind(record.record_kind.trim())
        .bind(record.provider_record_id.trim())
        .fetch_optional(&mut *transaction)
        .await?;
        let observation_id = match existing_observation_id {
            Some(observation_id) => observation_id,
            None => {
                ObservationStore::capture_in_transaction(
                    &mut transaction,
                    &raw_record_observation(record),
                )
                .await?
                .observation_id
            }
        };
        let inserted = sqlx::query(
            r#"
            INSERT INTO communication_raw_records (
                raw_record_id,
                observation_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                payload,
                provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (raw_record_id)
            DO NOTHING
            RETURNING
                raw_record_id,
                observation_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            "#,
        )
        .bind(record.raw_record_id.trim())
        .bind(&observation_id)
        .bind(record.account_id.trim())
        .bind(record.record_kind.trim())
        .bind(record.provider_record_id.trim())
        .bind(record.source_fingerprint.trim())
        .bind(record.import_batch_id.trim())
        .bind(record.occurred_at)
        .bind(&record.payload)
        .bind(&record.provenance)
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(row) = inserted {
            let stored = row_to_raw_record(row)?;
            transaction.commit().await?;
            return Ok(stored);
        }

        let row = sqlx::query(raw_record_by_id_sql())
            .bind(record.raw_record_id.trim())
            .fetch_one(&mut *transaction)
            .await?;

        let stored = row_to_raw_record(row)?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn raw_record(
        &self,
        raw_record_id: &str,
    ) -> Result<Option<StoredRawCommunicationRecord>, CommunicationIngestionError> {
        validate_non_empty("raw_record_id", raw_record_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                raw_record_id,
                observation_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            FROM communication_raw_records
            WHERE raw_record_id = $1
            "#,
        )
        .bind(raw_record_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_raw_record).transpose()
    }
}

impl CommunicationRawEvidenceCommandPort for CommunicationIngestionStore {
    fn record_raw_source<'a>(
        &'a self,
        record: &'a NewRawCommunicationRecord,
    ) -> CommunicationEvidencePortFuture<'a, StoredRawCommunicationRecord> {
        Box::pin(async move {
            CommunicationIngestionStore::record_raw_source(self, record)
                .await
                .map_err(CommunicationEvidencePortError::new)
        })
    }
}

fn raw_record_by_id_sql() -> &'static str {
    r#"
    SELECT
        raw_record_id,
        observation_id,
        account_id,
        record_kind,
        provider_record_id,
        source_fingerprint,
        import_batch_id,
        occurred_at,
        captured_at,
        payload,
        provenance
    FROM communication_raw_records
    WHERE raw_record_id = $1
    "#
}

fn raw_record_observation(record: &NewRawCommunicationRecord) -> NewObservation {
    let kind_code = if record.record_kind.contains("attachment") {
        "COMMUNICATION_ATTACHMENT"
    } else {
        "COMMUNICATION_MESSAGE"
    };
    let observed_at = record.occurred_at.unwrap_or_else(Utc::now);
    NewObservation::new(
        kind_code,
        ObservationOriginKind::VaultSource,
        observed_at,
        record.payload.clone(),
        format!(
            "communication://{}/{}/{}",
            record.account_id.trim(),
            record.record_kind.trim(),
            record.provider_record_id.trim()
        ),
    )
    .confidence(1.0)
    .provenance(json!({
        "communication_raw_record": true,
        "raw_record_id": record.raw_record_id.trim(),
        "account_id": record.account_id.trim(),
        "record_kind": record.record_kind.trim(),
        "provider_record_id": record.provider_record_id.trim(),
        "import_batch_id": record.import_batch_id.trim(),
        "source_fingerprint": record.source_fingerprint.trim(),
        "raw_provenance": record.provenance
    }))
}
