//! Owner-local canonical snapshot and exact inbox/outbox persistence for the
//! Communications evidence-export source port.

use makosh_communications_api::CommunicationDirectionV1;
use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::Row;

use crate::{CommunicationsConsumeOutcomeV1, CommunicationsDurablePersistence};

const MAX_EXPORT_MESSAGES_V1: usize = 64;
const MAX_EXPORT_SOURCE_BYTES_V1: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationsEvidenceExportBodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsEvidenceExportSourceItemV1 {
    pub message_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub evidence_id: [u8; 16],
    pub evidence_revision: u64,
    pub direction: CommunicationDirectionV1,
    pub occurred_at_unix_seconds: i64,
    pub observed_at_unix_seconds: i64,
    pub participant_display_label: Option<String>,
    pub body: Option<CommunicationsEvidenceExportBodyReceiptV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsEvidenceExportSourceErrorV1 {
    InvalidRequest,
    NotFound,
    StaleRevision,
    ContentLimit,
    InvalidRow,
    StorageUnavailable,
    InboxHashConflict,
    OutboxConflict,
}

impl CommunicationsDurablePersistence {
    pub async fn evidence_export_source_snapshot(
        &self,
        message_ids: &[[u8; 16]],
    ) -> Result<
        Vec<CommunicationsEvidenceExportSourceItemV1>,
        CommunicationsEvidenceExportSourceErrorV1,
    > {
        validate_message_ids(message_ids)?;
        let requested = message_ids
            .iter()
            .map(|value| value.to_vec())
            .collect::<Vec<_>>();
        let rows = sqlx::query(
            "WITH requested(message_id, ordinal) AS (
               SELECT message_id, ordinal
               FROM unnest($1::BYTEA[]) WITH ORDINALITY AS input(message_id, ordinal)
             )
             SELECT requested.ordinal, message.message_id, message.conversation_id,
               message.last_evidence_id AS evidence_id, message.canonical_revision,
               message.direction, evidence.observed_at_unix_seconds
                 AS occurred_at_unix_seconds,
               COALESCE(lineage.recorded_at_unix_seconds,
                 evidence.observed_at_unix_seconds) AS observed_at_unix_seconds,
               sender.display_label AS participant_display_label,
               evidence.body_state, evidence.body_blob_reference_id,
               evidence.body_blob_declared_bytes, evidence.body_blob_sha256
             FROM requested
             JOIN makosh_data.communications_messages message
               ON message.message_id = requested.message_id
             JOIN makosh_data.communications_evidence_summaries evidence
               ON evidence.observation_id = message.last_evidence_id
             LEFT JOIN makosh_data.communications_evidence_audit_lineage lineage
               ON lineage.evidence_id = evidence.observation_id
             LEFT JOIN makosh_data.communications_message_sender_facts sender_fact
               ON sender_fact.message_id = message.message_id
             LEFT JOIN makosh_data.communications_sender_profiles sender
               ON sender.sender_id = sender_fact.sender_id
             WHERE message.lifecycle_state = 1
             ORDER BY requested.ordinal",
        )
        .bind(requested)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable)?;
        if rows.len() != message_ids.len() {
            return Err(CommunicationsEvidenceExportSourceErrorV1::NotFound);
        }
        let mut total_source_bytes = 0_u64;
        let mut items = Vec::with_capacity(rows.len());
        for (index, row) in rows.into_iter().enumerate() {
            let ordinal: i64 = row
                .try_get("ordinal")
                .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?;
            if usize::try_from(ordinal).ok() != Some(index + 1) {
                return Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRow);
            }
            let body_state: i16 = row
                .try_get("body_state")
                .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?;
            let body = match body_state {
                4 => {
                    let declared_bytes = u64::try_from(
                        row.try_get::<i64, _>("body_blob_declared_bytes")
                            .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                    )
                    .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?;
                    total_source_bytes =
                        checked_export_source_bytes(total_source_bytes, declared_bytes)?;
                    Some(CommunicationsEvidenceExportBodyReceiptV1 {
                        reference_id: id16(
                            &row.try_get::<Vec<u8>, _>("body_blob_reference_id")
                                .map_err(|_| {
                                    CommunicationsEvidenceExportSourceErrorV1::InvalidRow
                                })?,
                        )?,
                        declared_bytes,
                        sha256: id32(
                            &row.try_get::<Vec<u8>, _>("body_blob_sha256").map_err(|_| {
                                CommunicationsEvidenceExportSourceErrorV1::InvalidRow
                            })?,
                        )?,
                    })
                }
                1..=3 => None,
                _ => return Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRow),
            };
            items.push(CommunicationsEvidenceExportSourceItemV1 {
                message_id: id16(
                    &row.try_get::<Vec<u8>, _>("message_id")
                        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                )?,
                conversation_id: id16(
                    &row.try_get::<Vec<u8>, _>("conversation_id")
                        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                )?,
                evidence_id: id16(
                    &row.try_get::<Vec<u8>, _>("evidence_id")
                        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                )?,
                evidence_revision: u64::try_from(
                    row.try_get::<i64, _>("canonical_revision")
                        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                )
                .ok()
                .filter(|value| *value > 0)
                .ok_or(CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                direction: direction_from_value(
                    row.try_get("direction")
                        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                )?,
                occurred_at_unix_seconds: positive_timestamp(
                    row.try_get("occurred_at_unix_seconds")
                        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                )?,
                observed_at_unix_seconds: positive_timestamp(
                    row.try_get("observed_at_unix_seconds")
                        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                )?,
                participant_display_label: row
                    .try_get("participant_display_label")
                    .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRow)?,
                body,
            });
        }
        Ok(items)
    }

    pub async fn persist_evidence_export_source_result(
        &self,
        command_message_id: [u8; 16],
        command_envelope_sha256: [u8; 32],
        expected_current_snapshot: Option<&[CommunicationsEvidenceExportSourceItemV1]>,
        result_outbox: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsEvidenceExportSourceErrorV1> {
        if !valid_id16(&command_message_id)
            || !valid_sha256(&command_envelope_sha256)
            || !valid_id16(result_outbox.message_id())
            || created_at_unix_seconds <= 0
        {
            return Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRequest);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable)?;
        if let Some(snapshot) = expected_current_snapshot {
            validate_current_snapshot(snapshot)?;
            let message_ids = snapshot
                .iter()
                .map(|item| item.message_id.to_vec())
                .collect::<Vec<_>>();
            let revisions = snapshot
                .iter()
                .map(|item| {
                    i64::try_from(item.evidence_revision)
                        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::InvalidRequest)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let current_count: i64 = sqlx::query_scalar(
                "WITH expected(message_id, canonical_revision) AS (
                   SELECT message_id, canonical_revision
                   FROM unnest($1::BYTEA[], $2::BIGINT[])
                     AS input(message_id, canonical_revision)
                 )
                 SELECT COUNT(*)
                 FROM expected
                 JOIN makosh_data.communications_messages message
                   ON message.message_id = expected.message_id
                  AND message.canonical_revision = expected.canonical_revision
                  AND message.lifecycle_state = 1",
            )
            .bind(message_ids)
            .bind(revisions)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable)?;
            if usize::try_from(current_count).ok() != Some(snapshot.len()) {
                return Err(CommunicationsEvidenceExportSourceErrorV1::StaleRevision);
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
        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable)?;
        if inserted.rows_affected() == 0 {
            let existing: Option<Vec<u8>> = sqlx::query_scalar(
                "SELECT envelope_sha256
                 FROM makosh_data.communications_event_inbox
                 WHERE message_id = $1",
            )
            .bind(command_message_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable)?;
            return if existing.as_deref() == Some(command_envelope_sha256.as_slice()) {
                Ok(CommunicationsConsumeOutcomeV1::Duplicate)
            } else {
                Err(CommunicationsEvidenceExportSourceErrorV1::InboxHashConflict)
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
        .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable)?;
        if result.rows_affected() != 1 {
            return Err(CommunicationsEvidenceExportSourceErrorV1::OutboxConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable)?;
        Ok(CommunicationsConsumeOutcomeV1::Applied)
    }
}

fn validate_current_snapshot(
    snapshot: &[CommunicationsEvidenceExportSourceItemV1],
) -> Result<(), CommunicationsEvidenceExportSourceErrorV1> {
    let message_ids = snapshot
        .iter()
        .map(|item| item.message_id)
        .collect::<Vec<_>>();
    validate_message_ids(&message_ids)?;
    if snapshot.iter().any(|item| item.evidence_revision == 0) {
        return Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRequest);
    }
    Ok(())
}

fn validate_message_ids(
    message_ids: &[[u8; 16]],
) -> Result<(), CommunicationsEvidenceExportSourceErrorV1> {
    if message_ids.is_empty()
        || message_ids.len() > MAX_EXPORT_MESSAGES_V1
        || message_ids.iter().any(|value| !valid_id16(value))
        || message_ids
            .iter()
            .enumerate()
            .any(|(index, value)| message_ids[..index].contains(value))
    {
        return Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRequest);
    }
    Ok(())
}

fn checked_export_source_bytes(
    current: u64,
    declared_bytes: u64,
) -> Result<u64, CommunicationsEvidenceExportSourceErrorV1> {
    let total = current
        .checked_add(declared_bytes)
        .ok_or(CommunicationsEvidenceExportSourceErrorV1::ContentLimit)?;
    if total > MAX_EXPORT_SOURCE_BYTES_V1 {
        return Err(CommunicationsEvidenceExportSourceErrorV1::ContentLimit);
    }
    Ok(total)
}

fn direction_from_value(
    value: i16,
) -> Result<CommunicationDirectionV1, CommunicationsEvidenceExportSourceErrorV1> {
    match value {
        1 => Ok(CommunicationDirectionV1::Incoming),
        2 => Ok(CommunicationDirectionV1::Outgoing),
        3 => Ok(CommunicationDirectionV1::Unknown),
        _ => Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRow),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsEvidenceExportSourceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id16)
        .ok_or(CommunicationsEvidenceExportSourceErrorV1::InvalidRow)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsEvidenceExportSourceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_sha256)
        .ok_or(CommunicationsEvidenceExportSourceErrorV1::InvalidRow)
}

fn positive_timestamp(value: i64) -> Result<i64, CommunicationsEvidenceExportSourceErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(CommunicationsEvidenceExportSourceErrorV1::InvalidRow)
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
    fn request_is_bounded_ordered_and_unique() {
        assert_eq!(
            validate_message_ids(&[]),
            Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRequest)
        );
        assert_eq!(validate_message_ids(&[[1; 16], [2; 16]]), Ok(()));
        assert_eq!(
            validate_message_ids(&[[1; 16], [1; 16]]),
            Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRequest)
        );
    }

    #[test]
    fn aggregate_source_bytes_reject_the_first_byte_above_the_exact_limit() {
        assert_eq!(
            checked_export_source_bytes(0, MAX_EXPORT_SOURCE_BYTES_V1),
            Ok(MAX_EXPORT_SOURCE_BYTES_V1)
        );
        assert_eq!(
            checked_export_source_bytes(MAX_EXPORT_SOURCE_BYTES_V1, 1),
            Err(CommunicationsEvidenceExportSourceErrorV1::ContentLimit)
        );
        assert_eq!(
            checked_export_source_bytes(u64::MAX, 1),
            Err(CommunicationsEvidenceExportSourceErrorV1::ContentLimit)
        );
    }

    #[test]
    fn prepared_result_requires_a_nonempty_unique_revision_snapshot() {
        let mut item = CommunicationsEvidenceExportSourceItemV1 {
            message_id: [1; 16],
            conversation_id: [2; 16],
            evidence_id: [3; 16],
            evidence_revision: 1,
            direction: CommunicationDirectionV1::Incoming,
            occurred_at_unix_seconds: 1,
            observed_at_unix_seconds: 1,
            participant_display_label: None,
            body: None,
        };
        assert_eq!(validate_current_snapshot(&[item.clone()]), Ok(()));
        assert_eq!(
            validate_current_snapshot(&[]),
            Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRequest)
        );
        assert_eq!(
            validate_current_snapshot(&[item.clone(), item.clone()]),
            Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRequest)
        );
        item.evidence_revision = 0;
        assert_eq!(
            validate_current_snapshot(&[item]),
            Err(CommunicationsEvidenceExportSourceErrorV1::InvalidRequest)
        );
    }
}
