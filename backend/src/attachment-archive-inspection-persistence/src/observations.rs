use makosh_attachment_archive_inspection_core::{
    ArchiveInspectionCanonicalSafetyFactV1, ArchiveInspectionErrorV1,
    ArchiveInspectionRecordDecisionV1, ArchiveInspectionRejectionV1, ArchiveInspectionRequestV1,
    ArchiveInspectionSafetyStateV1, ArchiveInspectionScanCandidateV1, ArchiveInspectionStatusV1,
    ArchiveInspectionTransitionV1, archive_inspection_rejection_evidence_id_v1,
    decide_archive_inspection_safety_record_v1, decide_archive_scan_candidate_record_v1,
    transition_archive_inspection_status_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    ArchiveInspectionPersistenceErrorV1, AttachmentArchiveInspectionPersistenceV1,
    PersistArchiveInspectionFactOutcomeV1, id16, id32,
    model::{state_from_code, valid_owner, valid_sha256, valid_timestamp_millis},
    runs::{lock_anchor, persist_status, settle_anchor_runs},
    unsigned,
};

enum InboxInsertV1 {
    New,
    Duplicate,
    Conflict,
}

impl AttachmentArchiveInspectionPersistenceV1 {
    pub async fn persist_scan_candidate(
        &self,
        logical_owner_id: &str,
        candidate: &ArchiveInspectionScanCandidateV1,
        envelope_sha256: [u8; 32],
        consumed_at_unix_millis: i64,
    ) -> Result<PersistArchiveInspectionFactOutcomeV1, ArchiveInspectionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_sha256(&envelope_sha256)
            || !valid_timestamp_millis(consumed_at_unix_millis)
            || decide_archive_scan_candidate_record_v1(None, candidate)
                != ArchiveInspectionRecordDecisionV1::Insert
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        lock_anchor(
            &mut transaction,
            logical_owner_id,
            candidate.attachment_anchor_id,
        )
        .await?;
        match insert_inbox(
            &mut transaction,
            logical_owner_id,
            candidate.message_id,
            envelope_sha256,
            1,
            candidate.attachment_anchor_id,
            consumed_at_unix_millis,
        )
        .await?
        {
            InboxInsertV1::Duplicate => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistArchiveInspectionFactOutcomeV1::Duplicate);
            }
            InboxInsertV1::Conflict => {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    candidate.attachment_anchor_id,
                    ArchiveInspectionRejectionV1::CandidateConflict,
                    consumed_at_unix_millis,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistArchiveInspectionFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
            InboxInsertV1::New => {}
        }
        let existing = load_candidate(
            &mut transaction,
            logical_owner_id,
            candidate.attachment_anchor_id,
        )
        .await?;
        match decide_archive_scan_candidate_record_v1(existing.as_ref(), candidate) {
            ArchiveInspectionRecordDecisionV1::Insert => {
                insert_candidate(
                    &mut transaction,
                    logical_owner_id,
                    candidate,
                    envelope_sha256,
                )
                .await?;
            }
            ArchiveInspectionRecordDecisionV1::Duplicate => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistArchiveInspectionFactOutcomeV1::Duplicate);
            }
            ArchiveInspectionRecordDecisionV1::Reject(reason) => {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    candidate.attachment_anchor_id,
                    reason,
                    consumed_at_unix_millis,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistArchiveInspectionFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
        }
        let transitioned = settle_anchor_runs(
            &mut transaction,
            logical_owner_id,
            candidate.attachment_anchor_id,
            consumed_at_unix_millis,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        Ok(PersistArchiveInspectionFactOutcomeV1::Recorded {
            transitioned_runs: transitioned,
        })
    }

    pub async fn persist_canonical_safety_fact(
        &self,
        logical_owner_id: &str,
        safety: &ArchiveInspectionCanonicalSafetyFactV1,
        envelope_sha256: [u8; 32],
        consumed_at_unix_millis: i64,
    ) -> Result<PersistArchiveInspectionFactOutcomeV1, ArchiveInspectionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_sha256(&envelope_sha256)
            || !valid_timestamp_millis(consumed_at_unix_millis)
            || decide_archive_inspection_safety_record_v1(None, safety)
                != ArchiveInspectionRecordDecisionV1::Insert
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        lock_anchor(
            &mut transaction,
            logical_owner_id,
            safety.attachment_anchor_id,
        )
        .await?;
        match insert_inbox(
            &mut transaction,
            logical_owner_id,
            safety.message_id,
            envelope_sha256,
            2,
            safety.attachment_anchor_id,
            consumed_at_unix_millis,
        )
        .await?
        {
            InboxInsertV1::Duplicate => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistArchiveInspectionFactOutcomeV1::Duplicate);
            }
            InboxInsertV1::Conflict => {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    safety.attachment_anchor_id,
                    ArchiveInspectionRejectionV1::SafetyStateConflict,
                    consumed_at_unix_millis,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistArchiveInspectionFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
            InboxInsertV1::New => {}
        }
        let existing = load_safety(
            &mut transaction,
            logical_owner_id,
            safety.attachment_anchor_id,
        )
        .await?;
        match decide_archive_inspection_safety_record_v1(existing.as_ref(), safety) {
            ArchiveInspectionRecordDecisionV1::Insert => {
                insert_safety(&mut transaction, logical_owner_id, safety, envelope_sha256).await?;
            }
            ArchiveInspectionRecordDecisionV1::Duplicate => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistArchiveInspectionFactOutcomeV1::Duplicate);
            }
            ArchiveInspectionRecordDecisionV1::Reject(reason) => {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    safety.attachment_anchor_id,
                    reason,
                    consumed_at_unix_millis,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistArchiveInspectionFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
        }
        let transitioned = settle_anchor_runs(
            &mut transaction,
            logical_owner_id,
            safety.attachment_anchor_id,
            consumed_at_unix_millis,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        Ok(PersistArchiveInspectionFactOutcomeV1::Recorded {
            transitioned_runs: transitioned,
        })
    }
}

pub(crate) async fn load_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
) -> Result<Option<ArchiveInspectionScanCandidateV1>, ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT message_id, blob_reference_id, declared_size, blob_receipt_sha256, custody_transfer_source_proof, observed_at_unix_seconds FROM makosh_data.attachment_archive_inspection_scan_candidates WHERE logical_owner_id = $1 AND attachment_anchor_id = $2",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    row.map(|row| {
        Ok(ArchiveInspectionScanCandidateV1 {
            message_id: id16(
                row.try_get::<Vec<u8>, _>("message_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            attachment_anchor_id,
            blob_reference_id: id16(
                row.try_get::<Vec<u8>, _>("blob_reference_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            declared_size: unsigned(
                row.try_get("declared_size")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
            )?,
            blob_receipt_sha256: id32(
                row.try_get::<Vec<u8>, _>("blob_receipt_sha256")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            custody_transfer_source_proof: row
                .try_get("custody_transfer_source_proof")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
            observed_at_unix_seconds: row
                .try_get("observed_at_unix_seconds")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        })
    })
    .transpose()
}

pub(crate) async fn load_candidate_envelope_sha256(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
) -> Result<Option<[u8; 32]>, ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256
         FROM makosh_data.attachment_archive_inspection_scan_candidates
         WHERE logical_owner_id = $1 AND attachment_anchor_id = $2",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    row.map(|row| {
        id32(
            row.try_get::<Vec<u8>, _>("envelope_sha256")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )
    })
    .transpose()
}

pub(crate) async fn load_safety(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
) -> Result<Option<ArchiveInspectionCanonicalSafetyFactV1>, ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT message_id, expected_state, next_state, evidence_id, observed_at_unix_seconds FROM makosh_data.attachment_archive_inspection_safety_facts WHERE logical_owner_id = $1 AND attachment_anchor_id = $2",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    row.map(|row| {
        Ok(ArchiveInspectionCanonicalSafetyFactV1 {
            message_id: id16(
                row.try_get::<Vec<u8>, _>("message_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            attachment_anchor_id,
            expected_state: safety_state_from_code(
                row.try_get("expected_state")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
            )?,
            next_state: safety_state_from_code(
                row.try_get("next_state")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
            )?,
            evidence_id: id16(
                row.try_get::<Vec<u8>, _>("evidence_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            observed_at_unix_seconds: row
                .try_get("observed_at_unix_seconds")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        })
    })
    .transpose()
}

async fn insert_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    event_kind: i16,
    attachment_anchor_id: [u8; 16],
    processed_at_unix_millis: i64,
) -> Result<InboxInsertV1, ArchiveInspectionPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_archive_inspection_event_inbox (logical_owner_id, message_id, envelope_sha256, event_kind, attachment_anchor_id, processed_at_unix_millis) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (logical_owner_id, message_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(event_kind)
    .bind(attachment_anchor_id.as_slice())
    .bind(processed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
    .rows_affected()
        == 1;
    if inserted {
        return Ok(InboxInsertV1::New);
    }
    let row = sqlx::query(
        "SELECT envelope_sha256, event_kind, attachment_anchor_id FROM makosh_data.attachment_archive_inspection_event_inbox WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let existing_hash = id32(
        row.try_get::<Vec<u8>, _>("envelope_sha256")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
            .as_slice(),
    )?;
    let existing_kind: i16 = row
        .try_get("event_kind")
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    let existing_anchor = id16(
        row.try_get::<Vec<u8>, _>("attachment_anchor_id")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
            .as_slice(),
    )?;
    if existing_hash == envelope_sha256
        && existing_kind == event_kind
        && existing_anchor == attachment_anchor_id
    {
        Ok(InboxInsertV1::Duplicate)
    } else {
        Ok(InboxInsertV1::Conflict)
    }
}

async fn insert_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    candidate: &ArchiveInspectionScanCandidateV1,
    envelope_sha256: [u8; 32],
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_archive_inspection_scan_candidates (logical_owner_id, attachment_anchor_id, message_id, envelope_sha256, blob_reference_id, declared_size, blob_receipt_sha256, custody_transfer_source_proof, observed_at_unix_seconds) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(logical_owner_id)
    .bind(candidate.attachment_anchor_id.as_slice())
    .bind(candidate.message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(candidate.blob_reference_id.as_slice())
    .bind(i64::try_from(candidate.declared_size).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
    .bind(candidate.blob_receipt_sha256.as_slice())
    .bind(&candidate.custody_transfer_source_proof)
    .bind(candidate.observed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)
}

async fn insert_safety(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    safety: &ArchiveInspectionCanonicalSafetyFactV1,
    envelope_sha256: [u8; 32],
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_archive_inspection_safety_facts (logical_owner_id, attachment_anchor_id, message_id, envelope_sha256, expected_state, next_state, evidence_id, observed_at_unix_seconds) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(logical_owner_id)
    .bind(safety.attachment_anchor_id.as_slice())
    .bind(safety.message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(safety_state_code(safety.expected_state))
    .bind(safety_state_code(safety.next_state))
    .bind(safety.evidence_id.as_slice())
    .bind(safety.observed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)
}

async fn reject_anchor_runs(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
    rejection: ArchiveInspectionRejectionV1,
    occurred_at_unix_millis: i64,
) -> Result<u32, ArchiveInspectionPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT run_id, operation_id, state, state_revision FROM makosh_data.attachment_archive_inspection_runs WHERE logical_owner_id = $1 AND attachment_anchor_id = $2 AND state IN (1, 2, 3) ORDER BY run_id FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let mut rejected = 0_u32;
    for row in rows {
        let request = ArchiveInspectionRequestV1 {
            run_id: id16(
                row.try_get::<Vec<u8>, _>("run_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            operation_id: id16(
                row.try_get::<Vec<u8>, _>("operation_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            attachment_anchor_id,
        };
        let state = state_from_code(
            row.try_get("state")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?;
        let revision = unsigned(
            row.try_get("state_revision")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?;
        let next = transition_archive_inspection_status_v1(
            &ArchiveInspectionStatusV1 {
                state,
                state_revision: revision,
                report: None,
                error: None,
            },
            ArchiveInspectionTransitionV1::Reject(ArchiveInspectionErrorV1::Unavailable),
        )
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
        persist_status(
            transaction,
            logical_owner_id,
            request.run_id,
            revision,
            &next,
            Some(archive_inspection_rejection_evidence_id_v1(
                &request, rejection,
            )),
            occurred_at_unix_millis,
        )
        .await?;
        rejected = rejected
            .checked_add(1)
            .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    }
    Ok(rejected)
}

const fn safety_state_code(value: ArchiveInspectionSafetyStateV1) -> i16 {
    match value {
        ArchiveInspectionSafetyStateV1::DescriptorOnly => 1,
        ArchiveInspectionSafetyStateV1::BlobPending => 2,
        ArchiveInspectionSafetyStateV1::BlobAdmitted => 3,
        ArchiveInspectionSafetyStateV1::Quarantined => 4,
        ArchiveInspectionSafetyStateV1::SafeForDelivery => 5,
        ArchiveInspectionSafetyStateV1::Rejected => 6,
    }
}

fn safety_state_from_code(
    value: i16,
) -> Result<ArchiveInspectionSafetyStateV1, ArchiveInspectionPersistenceErrorV1> {
    match value {
        1 => Ok(ArchiveInspectionSafetyStateV1::DescriptorOnly),
        2 => Ok(ArchiveInspectionSafetyStateV1::BlobPending),
        3 => Ok(ArchiveInspectionSafetyStateV1::BlobAdmitted),
        4 => Ok(ArchiveInspectionSafetyStateV1::Quarantined),
        5 => Ok(ArchiveInspectionSafetyStateV1::SafeForDelivery),
        6 => Ok(ArchiveInspectionSafetyStateV1::Rejected),
        _ => Err(ArchiveInspectionPersistenceErrorV1::InvalidRow),
    }
}
