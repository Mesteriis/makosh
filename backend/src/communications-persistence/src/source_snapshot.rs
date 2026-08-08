use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::Row;

use crate::{CommunicationsConsumeOutcomeV1, CommunicationsDurablePersistence};

const MAX_SOURCE_BYTES_V1: u64 = 256 * 1024;
const MAX_SOURCE_SENDER_BYTES_V1: usize = 256;
const MAX_SOURCE_SUBJECT_BYTES_V1: usize = 998;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsBodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSourceSnapshotV1 {
    pub source_message_id: [u8; 16],
    pub evidence_id: [u8; 16],
    pub evidence_revision: u64,
    pub sender_utf8: Vec<u8>,
    pub subject_utf8: Vec<u8>,
    pub body: CommunicationsBodyReceiptV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSourceErrorV1 {
    InvalidRequest,
    SourceMissingOrInactive,
    ContentUnavailable,
    ContentLimit,
    StaleRevision,
    InvalidRow,
    StorageUnavailable,
    InboxHashConflict,
    OutboxConflict,
}

impl CommunicationsDurablePersistence {
    pub async fn source_snapshot(
        &self,
        source_message_id: [u8; 16],
        expected_source_revision: u64,
    ) -> Result<CommunicationsSourceSnapshotV1, CommunicationsSourceErrorV1> {
        if !valid_id16(&source_message_id) || expected_source_revision == 0 {
            return Err(CommunicationsSourceErrorV1::InvalidRequest);
        }
        let row = sqlx::query(
            "SELECT message.message_id, message.last_evidence_id AS evidence_id,
               message.canonical_revision, evidence.body_state,
               evidence.participant_display_label, evidence.message_subject,
               evidence.body_blob_reference_id, evidence.body_blob_declared_bytes,
               evidence.body_blob_sha256
             FROM makosh_data.communications_messages message
             JOIN makosh_data.communications_evidence_summaries evidence
               ON evidence.observation_id = message.last_evidence_id
             WHERE message.message_id = $1
               AND message.lifecycle_state = 1",
        )
        .bind(source_message_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsSourceErrorV1::StorageUnavailable)?
        .ok_or(CommunicationsSourceErrorV1::SourceMissingOrInactive)?;
        let revision = positive_revision(
            row.try_get("canonical_revision")
                .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
        )?;
        if revision != expected_source_revision {
            return Err(CommunicationsSourceErrorV1::StaleRevision);
        }
        let body_state: i16 = row
            .try_get("body_state")
            .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?;
        if body_state != 4 {
            return Err(CommunicationsSourceErrorV1::ContentUnavailable);
        }
        let declared_bytes = u64::try_from(
            row.try_get::<i64, _>("body_blob_declared_bytes")
                .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
        )
        .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?;
        if !(1..=MAX_SOURCE_BYTES_V1).contains(&declared_bytes) {
            return Err(CommunicationsSourceErrorV1::ContentLimit);
        }
        let sender = row
            .try_get::<Option<String>, _>("participant_display_label")
            .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?
            .unwrap_or_default()
            .into_bytes();
        let subject = row
            .try_get::<Option<String>, _>("message_subject")
            .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?
            .unwrap_or_default()
            .into_bytes();
        Ok(CommunicationsSourceSnapshotV1 {
            source_message_id: id16(
                &row.try_get::<Vec<u8>, _>("message_id")
                    .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
            )?,
            evidence_id: id16(
                &row.try_get::<Vec<u8>, _>("evidence_id")
                    .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
            )?,
            evidence_revision: revision,
            sender_utf8: sender,
            subject_utf8: subject,
            body: CommunicationsBodyReceiptV1 {
                reference_id: id16(
                    &row.try_get::<Vec<u8>, _>("body_blob_reference_id")
                        .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
                )?,
                declared_bytes,
                sha256: id32(
                    &row.try_get::<Vec<u8>, _>("body_blob_sha256")
                        .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
                )?,
            },
        })
    }

    pub async fn persist_source_result(
        &self,
        command_message_id: [u8; 16],
        command_envelope_sha256: [u8; 32],
        expected_current_snapshot: Option<&CommunicationsSourceSnapshotV1>,
        result_outbox: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsSourceErrorV1> {
        if !valid_id16(&command_message_id)
            || !valid_sha256(&command_envelope_sha256)
            || !valid_id16(result_outbox.message_id())
            || created_at_unix_seconds <= 0
        {
            return Err(CommunicationsSourceErrorV1::InvalidRequest);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsSourceErrorV1::StorageUnavailable)?;
        if let Some(snapshot) = expected_current_snapshot {
            validate_snapshot(snapshot)?;
            let current = sqlx::query(
                "SELECT message.canonical_revision,
                   message.last_evidence_id AS evidence_id,
                   evidence.body_blob_reference_id,
                   evidence.body_blob_declared_bytes,
                   evidence.body_blob_sha256
                 FROM makosh_data.communications_messages message
                 JOIN makosh_data.communications_evidence_summaries evidence
                   ON evidence.observation_id = message.last_evidence_id
                 WHERE message.message_id = $1
                   AND message.lifecycle_state = 1
                   AND evidence.body_state = 4",
            )
            .bind(snapshot.source_message_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| CommunicationsSourceErrorV1::StorageUnavailable)?;
            let Some(current) = current else {
                return Err(CommunicationsSourceErrorV1::StaleRevision);
            };
            let revision = positive_revision(
                current
                    .try_get("canonical_revision")
                    .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
            )?;
            let current_evidence_id = id16(
                &current
                    .try_get::<Vec<u8>, _>("evidence_id")
                    .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
            )?;
            let current_reference_id = id16(
                &current
                    .try_get::<Vec<u8>, _>("body_blob_reference_id")
                    .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
            )?;
            let current_declared_bytes = u64::try_from(
                current
                    .try_get::<i64, _>("body_blob_declared_bytes")
                    .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
            )
            .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?;
            let current_sha256 = id32(
                &current
                    .try_get::<Vec<u8>, _>("body_blob_sha256")
                    .map_err(|_| CommunicationsSourceErrorV1::InvalidRow)?,
            )?;
            if revision != snapshot.evidence_revision
                || current_evidence_id != snapshot.evidence_id
                || current_reference_id != snapshot.body.reference_id
                || current_declared_bytes != snapshot.body.declared_bytes
                || current_sha256 != snapshot.body.sha256
            {
                return Err(CommunicationsSourceErrorV1::StaleRevision);
            }
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communications_event_inbox
               (message_id, envelope_sha256)
             VALUES ($1, $2)
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(command_message_id.as_slice())
        .bind(command_envelope_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsSourceErrorV1::StorageUnavailable)?;
        if inserted.rows_affected() == 0 {
            let existing: Option<Vec<u8>> = sqlx::query_scalar(
                "SELECT envelope_sha256
                 FROM makosh_data.communications_event_inbox
                 WHERE message_id = $1",
            )
            .bind(command_message_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| CommunicationsSourceErrorV1::StorageUnavailable)?;
            return if existing.as_deref() == Some(command_envelope_sha256.as_slice()) {
                Ok(CommunicationsConsumeOutcomeV1::Duplicate)
            } else {
                Err(CommunicationsSourceErrorV1::InboxHashConflict)
            };
        }
        let result = sqlx::query(
            "INSERT INTO makosh_data.communications_domain_outbox (
               message_id, envelope_sha256, exact_envelope_bytes,
               created_at_unix_seconds, published_at_unix_seconds
             ) VALUES ($1, $2, $3, $4, NULL)
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(result_outbox.message_id().as_slice())
        .bind(result_outbox.envelope_sha256().as_slice())
        .bind(result_outbox.exact_bytes())
        .bind(created_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsSourceErrorV1::StorageUnavailable)?;
        if result.rows_affected() != 1 {
            return Err(CommunicationsSourceErrorV1::OutboxConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsSourceErrorV1::StorageUnavailable)?;
        Ok(CommunicationsConsumeOutcomeV1::Applied)
    }
}

fn validate_snapshot(
    snapshot: &CommunicationsSourceSnapshotV1,
) -> Result<(), CommunicationsSourceErrorV1> {
    if !valid_id16(&snapshot.source_message_id)
        || !valid_id16(&snapshot.evidence_id)
        || snapshot.evidence_revision == 0
        || snapshot.sender_utf8.len() > MAX_SOURCE_SENDER_BYTES_V1
        || snapshot.subject_utf8.len() > MAX_SOURCE_SUBJECT_BYTES_V1
        || std::str::from_utf8(&snapshot.sender_utf8).is_err()
        || std::str::from_utf8(&snapshot.subject_utf8).is_err()
        || !valid_id16(&snapshot.body.reference_id)
        || !(1..=MAX_SOURCE_BYTES_V1).contains(&snapshot.body.declared_bytes)
        || !valid_sha256(&snapshot.body.sha256)
    {
        return Err(CommunicationsSourceErrorV1::InvalidRequest);
    }
    Ok(())
}

fn positive_revision(value: i64) -> Result<u64, CommunicationsSourceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|revision| *revision > 0)
        .ok_or(CommunicationsSourceErrorV1::InvalidRow)
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsSourceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id16)
        .ok_or(CommunicationsSourceErrorV1::InvalidRow)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsSourceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_sha256)
        .ok_or(CommunicationsSourceErrorV1::InvalidRow)
}

fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_is_bounded_and_provider_neutral() {
        let snapshot = CommunicationsSourceSnapshotV1 {
            source_message_id: [1; 16],
            evidence_id: [2; 16],
            evidence_revision: 3,
            sender_utf8: b"Ada <ada@example.test>".to_vec(),
            subject_utf8: b"Quarterly update".to_vec(),
            body: CommunicationsBodyReceiptV1 {
                reference_id: [4; 16],
                declared_bytes: 5,
                sha256: [6; 32],
            },
        };
        assert_eq!(validate_snapshot(&snapshot), Ok(()));
    }
}
