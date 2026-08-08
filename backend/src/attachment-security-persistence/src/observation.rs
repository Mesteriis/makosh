//! Transactional inbox recording and order-independent candidate/state join.

use makosh_attachment_security_core::{
    AttachmentSecurityCanonicalStateFactV1, AttachmentSecurityJoinDecisionV1,
    AttachmentSecurityJoinPolicyV1, AttachmentSecurityQuarantineEvidenceV1,
    AttachmentSecurityQuarantineReasonV1, AttachmentSecurityRecordDecisionV1,
    AttachmentSecurityScanCandidateV1, CanonicalAttachmentSafetyStateV1,
    attachment_security_quarantine_evidence_v1, decide_candidate_record_v1,
    decide_canonical_state_record_v1, decide_scan_join_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentSecurityPersistenceErrorV1, AttachmentSecurityPersistenceV1,
    AttachmentSecurityRetryPolicyV1, PersistAttachmentSecurityObservationOutcomeV1, id16, id32,
    jobs::enqueue_scan_job, valid_sha256, valid_timestamp,
};

enum InboxInsertV1 {
    New,
    Duplicate,
    HashConflict,
}

impl AttachmentSecurityPersistenceV1 {
    pub async fn persist_scan_candidate(
        &self,
        incoming: &AttachmentSecurityScanCandidateV1,
        envelope_sha256: [u8; 32],
        join_policy: AttachmentSecurityJoinPolicyV1,
        retry_policy: AttachmentSecurityRetryPolicyV1,
        consumed_at_unix_seconds: i64,
    ) -> Result<PersistAttachmentSecurityObservationOutcomeV1, AttachmentSecurityPersistenceErrorV1>
    {
        if !valid_sha256(&envelope_sha256)
            || !valid_timestamp(consumed_at_unix_seconds)
            || decide_candidate_record_v1(None, incoming, join_policy)
                != AttachmentSecurityRecordDecisionV1::Insert
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        lock_anchor(&mut transaction, incoming.attachment_anchor_id).await?;
        match insert_inbox(
            &mut transaction,
            incoming.message_id,
            envelope_sha256,
            1,
            consumed_at_unix_seconds,
        )
        .await?
        {
            InboxInsertV1::Duplicate => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistAttachmentSecurityObservationOutcomeV1::Duplicate);
            }
            InboxInsertV1::HashConflict => {
                let evidence = attachment_security_quarantine_evidence_v1(
                    incoming.attachment_anchor_id,
                    incoming.correlation_id,
                    AttachmentSecurityQuarantineReasonV1::CandidateConflict,
                );
                insert_quarantine(
                    &mut transaction,
                    incoming.message_id,
                    &evidence,
                    consumed_at_unix_seconds,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistAttachmentSecurityObservationOutcomeV1::Quarantined(
                    evidence,
                ));
            }
            InboxInsertV1::New => {}
        }

        let existing = load_candidate(&mut transaction, incoming.attachment_anchor_id).await?;
        match decide_candidate_record_v1(existing.as_ref(), incoming, join_policy) {
            AttachmentSecurityRecordDecisionV1::Insert => {
                insert_candidate(&mut transaction, incoming).await?;
            }
            AttachmentSecurityRecordDecisionV1::Duplicate => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistAttachmentSecurityObservationOutcomeV1::Duplicate);
            }
            AttachmentSecurityRecordDecisionV1::Quarantine(evidence) => {
                insert_quarantine(
                    &mut transaction,
                    incoming.message_id,
                    &evidence,
                    consumed_at_unix_seconds,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistAttachmentSecurityObservationOutcomeV1::Quarantined(
                    evidence,
                ));
            }
        }

        let canonical =
            load_canonical_state(&mut transaction, incoming.attachment_anchor_id).await?;
        let outcome = settle_join(
            &mut transaction,
            Some(incoming),
            canonical.as_ref(),
            join_policy,
            retry_policy,
            incoming.message_id,
            consumed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(outcome)
    }

    pub async fn persist_canonical_state(
        &self,
        incoming: &AttachmentSecurityCanonicalStateFactV1,
        envelope_sha256: [u8; 32],
        join_policy: AttachmentSecurityJoinPolicyV1,
        retry_policy: AttachmentSecurityRetryPolicyV1,
        consumed_at_unix_seconds: i64,
    ) -> Result<PersistAttachmentSecurityObservationOutcomeV1, AttachmentSecurityPersistenceErrorV1>
    {
        if !valid_sha256(&envelope_sha256)
            || !valid_timestamp(consumed_at_unix_seconds)
            || decide_canonical_state_record_v1(None, incoming)
                != AttachmentSecurityRecordDecisionV1::Insert
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        lock_anchor(&mut transaction, incoming.attachment_anchor_id).await?;
        match insert_inbox(
            &mut transaction,
            incoming.message_id,
            envelope_sha256,
            2,
            consumed_at_unix_seconds,
        )
        .await?
        {
            InboxInsertV1::Duplicate => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistAttachmentSecurityObservationOutcomeV1::Duplicate);
            }
            InboxInsertV1::HashConflict => {
                let evidence = attachment_security_quarantine_evidence_v1(
                    incoming.attachment_anchor_id,
                    incoming.correlation_id,
                    AttachmentSecurityQuarantineReasonV1::CanonicalStateConflict,
                );
                insert_quarantine(
                    &mut transaction,
                    incoming.message_id,
                    &evidence,
                    consumed_at_unix_seconds,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistAttachmentSecurityObservationOutcomeV1::Quarantined(
                    evidence,
                ));
            }
            InboxInsertV1::New => {}
        }

        let existing =
            load_canonical_state(&mut transaction, incoming.attachment_anchor_id).await?;
        match decide_canonical_state_record_v1(existing.as_ref(), incoming) {
            AttachmentSecurityRecordDecisionV1::Insert => {
                insert_canonical_state(&mut transaction, incoming).await?;
            }
            AttachmentSecurityRecordDecisionV1::Duplicate => {
                transaction
                    .commit()
                    .await
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistAttachmentSecurityObservationOutcomeV1::Duplicate);
            }
            AttachmentSecurityRecordDecisionV1::Quarantine(evidence) => {
                insert_quarantine(
                    &mut transaction,
                    incoming.message_id,
                    &evidence,
                    consumed_at_unix_seconds,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
                return Ok(PersistAttachmentSecurityObservationOutcomeV1::Quarantined(
                    evidence,
                ));
            }
        }

        let candidate = load_candidate(&mut transaction, incoming.attachment_anchor_id).await?;
        let outcome = settle_join(
            &mut transaction,
            candidate.as_ref(),
            Some(incoming),
            join_policy,
            retry_policy,
            incoming.message_id,
            consumed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(outcome)
    }
}

async fn settle_join(
    transaction: &mut Transaction<'_, Postgres>,
    candidate: Option<&AttachmentSecurityScanCandidateV1>,
    canonical: Option<&AttachmentSecurityCanonicalStateFactV1>,
    join_policy: AttachmentSecurityJoinPolicyV1,
    retry_policy: AttachmentSecurityRetryPolicyV1,
    source_message_id: [u8; 16],
    recorded_at_unix_seconds: i64,
) -> Result<PersistAttachmentSecurityObservationOutcomeV1, AttachmentSecurityPersistenceErrorV1> {
    let outcome = match decide_scan_join_v1(candidate, canonical, join_policy) {
        AttachmentSecurityJoinDecisionV1::Waiting => {
            PersistAttachmentSecurityObservationOutcomeV1::Waiting
        }
        AttachmentSecurityJoinDecisionV1::Runnable(job) => {
            let job_id =
                enqueue_scan_job(transaction, &job, retry_policy, recorded_at_unix_seconds).await?;
            PersistAttachmentSecurityObservationOutcomeV1::Runnable { job_id }
        }
        AttachmentSecurityJoinDecisionV1::Quarantine(evidence) => {
            insert_quarantine(
                transaction,
                source_message_id,
                &evidence,
                recorded_at_unix_seconds,
            )
            .await?;
            PersistAttachmentSecurityObservationOutcomeV1::Quarantined(evidence)
        }
    };
    Ok(outcome)
}

async fn lock_anchor(
    transaction: &mut Transaction<'_, Postgres>,
    attachment_anchor_id: [u8; 16],
) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_security_join_locks (attachment_anchor_id) VALUES ($1) ON CONFLICT (attachment_anchor_id) DO NOTHING",
    )
    .bind(attachment_anchor_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    sqlx::query(
        "SELECT attachment_anchor_id FROM makosh_data.attachment_security_join_locks WHERE attachment_anchor_id = $1 FOR UPDATE",
    )
    .bind(attachment_anchor_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    Ok(())
}

async fn insert_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    event_kind: i16,
    consumed_at_unix_seconds: i64,
) -> Result<InboxInsertV1, AttachmentSecurityPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_security_event_inbox (message_id, envelope_sha256, event_kind, consumed_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING",
    )
    .bind(message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(event_kind)
    .bind(consumed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    if inserted.rows_affected() == 1 {
        return Ok(InboxInsertV1::New);
    }
    let row = sqlx::query(
        "SELECT envelope_sha256, event_kind FROM makosh_data.attachment_security_event_inbox WHERE message_id = $1",
    )
    .bind(message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    let stored = id32(
        &row.try_get::<Vec<u8>, _>("envelope_sha256")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
    )?;
    let stored_kind: i16 = row
        .try_get("event_kind")
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    Ok(if stored == envelope_sha256 && stored_kind == event_kind {
        InboxInsertV1::Duplicate
    } else {
        InboxInsertV1::HashConflict
    })
}

async fn insert_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    value: &AttachmentSecurityScanCandidateV1,
) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
    let declared_size = i64::try_from(value.declared_size)
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
    sqlx::query(
        "INSERT INTO makosh_data.attachment_security_scan_candidates (attachment_anchor_id, message_id, blob_reference_id, declared_size, blob_receipt_sha256, custody_transfer_source_proof, causation_message_id, correlation_id, observed_at_unix_seconds) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(value.attachment_anchor_id.as_slice())
    .bind(value.message_id.as_slice())
    .bind(value.blob_reference_id.as_slice())
    .bind(declared_size)
    .bind(value.blob_receipt_sha256.as_slice())
    .bind(&value.custody_transfer_source_proof)
    .bind(value.causation_message_id.as_slice())
    .bind(value.correlation_id.as_slice())
    .bind(value.observed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)
}

async fn insert_canonical_state(
    transaction: &mut Transaction<'_, Postgres>,
    value: &AttachmentSecurityCanonicalStateFactV1,
) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_security_canonical_states (attachment_anchor_id, message_id, expected_state, next_state, evidence_id, correlation_id, observed_at_unix_seconds) VALUES ($1, $2, 2, 3, $3, $4, $5)",
    )
    .bind(value.attachment_anchor_id.as_slice())
    .bind(value.message_id.as_slice())
    .bind(value.evidence_id.as_slice())
    .bind(value.correlation_id.as_slice())
    .bind(value.observed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)
}

async fn load_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    attachment_anchor_id: [u8; 16],
) -> Result<Option<AttachmentSecurityScanCandidateV1>, AttachmentSecurityPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT message_id, blob_reference_id, declared_size, blob_receipt_sha256, custody_transfer_source_proof, causation_message_id, correlation_id, observed_at_unix_seconds FROM makosh_data.attachment_security_scan_candidates WHERE attachment_anchor_id = $1",
    )
    .bind(attachment_anchor_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    row.map(|row| {
        Ok(AttachmentSecurityScanCandidateV1 {
            message_id: id16(
                &row.try_get::<Vec<u8>, _>("message_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            attachment_anchor_id,
            blob_reference_id: id16(
                &row.try_get::<Vec<u8>, _>("blob_reference_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            declared_size: u64::try_from(
                row.try_get::<i64, _>("declared_size")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            blob_receipt_sha256: id32(
                &row.try_get::<Vec<u8>, _>("blob_receipt_sha256")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            custody_transfer_source_proof: row
                .try_get("custody_transfer_source_proof")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            causation_message_id: id16(
                &row.try_get::<Vec<u8>, _>("causation_message_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            correlation_id: id16(
                &row.try_get::<Vec<u8>, _>("correlation_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            observed_at_unix_seconds: row
                .try_get("observed_at_unix_seconds")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        })
    })
    .transpose()
}

async fn load_canonical_state(
    transaction: &mut Transaction<'_, Postgres>,
    attachment_anchor_id: [u8; 16],
) -> Result<Option<AttachmentSecurityCanonicalStateFactV1>, AttachmentSecurityPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT message_id, expected_state, next_state, evidence_id, correlation_id, observed_at_unix_seconds FROM makosh_data.attachment_security_canonical_states WHERE attachment_anchor_id = $1",
    )
    .bind(attachment_anchor_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    row.map(|row| {
        let expected: i16 = row
            .try_get("expected_state")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
        let next: i16 = row
            .try_get("next_state")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
        if expected != 2 || next != 3 {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidRow);
        }
        Ok(AttachmentSecurityCanonicalStateFactV1 {
            message_id: id16(
                &row.try_get::<Vec<u8>, _>("message_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            attachment_anchor_id,
            expected_state: CanonicalAttachmentSafetyStateV1::BlobPending,
            next_state: CanonicalAttachmentSafetyStateV1::BlobAdmitted,
            evidence_id: id16(
                &row.try_get::<Vec<u8>, _>("evidence_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            correlation_id: id16(
                &row.try_get::<Vec<u8>, _>("correlation_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            observed_at_unix_seconds: row
                .try_get("observed_at_unix_seconds")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        })
    })
    .transpose()
}

async fn insert_quarantine(
    transaction: &mut Transaction<'_, Postgres>,
    source_message_id: [u8; 16],
    evidence: &AttachmentSecurityQuarantineEvidenceV1,
    recorded_at_unix_seconds: i64,
) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_security_join_quarantines (evidence_id, source_message_id, attachment_anchor_id, correlation_id, reason, recorded_at_unix_seconds) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (evidence_id, source_message_id) DO NOTHING",
    )
    .bind(evidence.evidence_id.as_slice())
    .bind(source_message_id.as_slice())
    .bind(evidence.attachment_anchor_id.as_slice())
    .bind(evidence.correlation_id.as_slice())
    .bind(quarantine_reason_value(evidence.reason))
    .bind(recorded_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)
}

const fn quarantine_reason_value(reason: AttachmentSecurityQuarantineReasonV1) -> i16 {
    match reason {
        AttachmentSecurityQuarantineReasonV1::InvalidCandidate => 1,
        AttachmentSecurityQuarantineReasonV1::InvalidCanonicalState => 2,
        AttachmentSecurityQuarantineReasonV1::CandidateConflict => 3,
        AttachmentSecurityQuarantineReasonV1::CanonicalStateConflict => 4,
        AttachmentSecurityQuarantineReasonV1::AnchorMismatch => 5,
        AttachmentSecurityQuarantineReasonV1::CorrelationMismatch => 6,
    }
}
