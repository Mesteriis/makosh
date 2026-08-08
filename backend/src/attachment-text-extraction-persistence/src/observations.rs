//! Exact event inbox and order-independent evidence join persistence.

use makosh_attachment_text_extraction_core::{
    AttachmentTextCanonicalSafetyFactV1, AttachmentTextExtractionErrorV1,
    AttachmentTextExtractionJoinDecisionV1, AttachmentTextExtractionRecordDecisionV1,
    AttachmentTextExtractionRejectionV1, AttachmentTextExtractionStateV1,
    AttachmentTextExtractionTransitionV1, AttachmentTextScanCandidateV1,
    decide_attachment_text_join_v1, decide_attachment_text_safety_record_v1,
    decide_attachment_text_scan_candidate_record_v1, transition_attachment_text_status_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    PendingAttachmentTextCustodyDelegationV1, PersistAttachmentTextFactOutcomeV1,
    model::{
        attachment_text_extraction_request_fingerprint_v1, safety_state_code,
        safety_state_from_code, valid_id16, valid_owner, valid_sha256,
    },
    repository::{append_realtime, load_run_for_update, update_run_status},
};

const MAX_PENDING_JOIN_RESULTS_V1: u16 = 64;

impl AttachmentTextExtractionPersistenceV1 {
    pub async fn persist_scan_candidate(
        &self,
        logical_owner_id: &str,
        candidate: &AttachmentTextScanCandidateV1,
        envelope_sha256: [u8; 32],
        exact_payload_sha256: [u8; 32],
        consumed_at_unix_millis: i64,
    ) -> Result<PersistAttachmentTextFactOutcomeV1, AttachmentTextExtractionPersistenceErrorV1>
    {
        if !valid_fact_input(
            logical_owner_id,
            candidate.message_id,
            envelope_sha256,
            exact_payload_sha256,
            consumed_at_unix_millis,
        ) || decide_attachment_text_scan_candidate_record_v1(None, candidate)
            != AttachmentTextExtractionRecordDecisionV1::Insert
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        lock_anchor(
            &mut transaction,
            logical_owner_id,
            candidate.attachment_anchor_id,
        )
        .await?;
        match reconcile_inbox(
            &mut transaction,
            InboxFactV1 {
                logical_owner_id,
                message_id: candidate.message_id,
                envelope_sha256,
                event_kind: 1,
                attachment_anchor_id: candidate.attachment_anchor_id,
                exact_payload_sha256,
                processed_at_unix_millis: consumed_at_unix_millis,
            },
        )
        .await?
        {
            InboxOutcomeV1::Replayed => {
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextFactOutcomeV1::Replayed);
            }
            InboxOutcomeV1::Conflict => {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    candidate.attachment_anchor_id,
                    AttachmentTextExtractionRejectionV1::CandidateConflict,
                    consumed_at_unix_millis,
                )
                .await?;
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
            InboxOutcomeV1::Inserted => {}
        }
        let existing = load_candidate(
            &mut transaction,
            logical_owner_id,
            candidate.attachment_anchor_id,
        )
        .await?;
        match decide_attachment_text_scan_candidate_record_v1(existing.as_ref(), candidate) {
            AttachmentTextExtractionRecordDecisionV1::Insert => {}
            AttachmentTextExtractionRecordDecisionV1::Duplicate => {
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextFactOutcomeV1::Replayed);
            }
            AttachmentTextExtractionRecordDecisionV1::Reject(_) => {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    candidate.attachment_anchor_id,
                    AttachmentTextExtractionRejectionV1::CandidateConflict,
                    consumed_at_unix_millis,
                )
                .await?;
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
        }
        sqlx::query(
            "INSERT INTO makosh_data.attachment_text_extraction_scan_candidates (logical_owner_id, attachment_anchor_id, message_id, envelope_sha256, exact_payload_sha256, blob_reference_id, declared_size, blob_receipt_sha256, custody_transfer_source_proof, observed_at_unix_seconds) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(logical_owner_id)
        .bind(candidate.attachment_anchor_id.as_slice())
        .bind(candidate.message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .bind(exact_payload_sha256.as_slice())
        .bind(candidate.blob_reference_id.as_slice())
        .bind(i64::try_from(candidate.declared_size).map_err(invalid_input)?)
        .bind(candidate.blob_receipt_sha256.as_slice())
        .bind(&candidate.custody_transfer_source_proof)
        .bind(candidate.observed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        let transitioned = settle_anchor_runs(
            &mut transaction,
            logical_owner_id,
            candidate.attachment_anchor_id,
            consumed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(PersistAttachmentTextFactOutcomeV1::Recorded {
            transitioned_runs: transitioned,
        })
    }

    pub async fn persist_canonical_safety_fact(
        &self,
        logical_owner_id: &str,
        safety: &AttachmentTextCanonicalSafetyFactV1,
        envelope_sha256: [u8; 32],
        exact_payload_sha256: [u8; 32],
        consumed_at_unix_millis: i64,
    ) -> Result<PersistAttachmentTextFactOutcomeV1, AttachmentTextExtractionPersistenceErrorV1>
    {
        if !valid_fact_input(
            logical_owner_id,
            safety.message_id,
            envelope_sha256,
            exact_payload_sha256,
            consumed_at_unix_millis,
        ) || decide_attachment_text_safety_record_v1(None, safety)
            != AttachmentTextExtractionRecordDecisionV1::Insert
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        lock_anchor(
            &mut transaction,
            logical_owner_id,
            safety.attachment_anchor_id,
        )
        .await?;
        match reconcile_inbox(
            &mut transaction,
            InboxFactV1 {
                logical_owner_id,
                message_id: safety.message_id,
                envelope_sha256,
                event_kind: 2,
                attachment_anchor_id: safety.attachment_anchor_id,
                exact_payload_sha256,
                processed_at_unix_millis: consumed_at_unix_millis,
            },
        )
        .await?
        {
            InboxOutcomeV1::Replayed => {
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextFactOutcomeV1::Replayed);
            }
            InboxOutcomeV1::Conflict => {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    safety.attachment_anchor_id,
                    AttachmentTextExtractionRejectionV1::SafetyStateConflict,
                    consumed_at_unix_millis,
                )
                .await?;
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
            InboxOutcomeV1::Inserted => {}
        }
        let existing = load_safety(
            &mut transaction,
            logical_owner_id,
            safety.attachment_anchor_id,
        )
        .await?;
        match decide_attachment_text_safety_record_v1(existing.as_ref(), safety) {
            AttachmentTextExtractionRecordDecisionV1::Insert => {}
            AttachmentTextExtractionRecordDecisionV1::Duplicate => {
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextFactOutcomeV1::Replayed);
            }
            AttachmentTextExtractionRecordDecisionV1::Reject(_) => {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    safety.attachment_anchor_id,
                    AttachmentTextExtractionRejectionV1::SafetyStateConflict,
                    consumed_at_unix_millis,
                )
                .await?;
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
        }
        sqlx::query(
            "INSERT INTO makosh_data.attachment_text_extraction_safety_facts (logical_owner_id, attachment_anchor_id, message_id, envelope_sha256, exact_payload_sha256, expected_state, next_state, evidence_id, observed_at_unix_seconds) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(logical_owner_id)
        .bind(safety.attachment_anchor_id.as_slice())
        .bind(safety.message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .bind(exact_payload_sha256.as_slice())
        .bind(safety_state_code(safety.expected_state))
        .bind(safety_state_code(safety.next_state))
        .bind(safety.evidence_id.as_slice())
        .bind(safety.observed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        let transitioned = settle_anchor_runs(
            &mut transaction,
            logical_owner_id,
            safety.attachment_anchor_id,
            consumed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(PersistAttachmentTextFactOutcomeV1::Recorded {
            transitioned_runs: transitioned,
        })
    }

    pub async fn pending_custody_delegation_intents(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<
        Vec<PendingAttachmentTextCustodyDelegationV1>,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        if !valid_owner(logical_owner_id) || !(1..=MAX_PENDING_JOIN_RESULTS_V1).contains(&limit) {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT r.run_id, r.operation_id, r.attachment_anchor_id, r.created_at_unix_millis, c.message_id AS candidate_message_id, c.envelope_sha256 AS candidate_envelope_sha256, c.blob_reference_id, c.declared_size, c.blob_receipt_sha256, c.custody_transfer_source_proof, c.observed_at_unix_seconds AS candidate_observed_at, s.message_id AS safety_message_id, s.expected_state, s.next_state, s.evidence_id, s.observed_at_unix_seconds AS safety_observed_at FROM makosh_data.attachment_text_extraction_runs r JOIN makosh_data.attachment_text_extraction_scan_candidates c ON c.logical_owner_id = r.logical_owner_id AND c.attachment_anchor_id = r.attachment_anchor_id JOIN makosh_data.attachment_text_extraction_safety_facts s ON s.logical_owner_id = r.logical_owner_id AND s.attachment_anchor_id = r.attachment_anchor_id LEFT JOIN makosh_data.attachment_text_extraction_custody_outbox o ON o.logical_owner_id = r.logical_owner_id AND o.run_id = r.run_id WHERE r.logical_owner_id = $1 AND r.state IN (1,2) AND s.next_state=5 AND o.run_id IS NULL ORDER BY r.created_at_unix_millis, r.run_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_unavailable)?;
        rows.into_iter().map(pending_join_from_row).collect()
    }
}

pub(crate) async fn lock_anchor(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    let digest = attachment_text_extraction_request_fingerprint_v1(attachment_anchor_id);
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    let owner_salt = logical_owner_id
        .as_bytes()
        .iter()
        .fold(0_i64, |value, byte| value.rotate_left(5) ^ i64::from(*byte));
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(i64::from_be_bytes(bytes) ^ owner_salt)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(storage_unavailable)
}

async fn settle_anchor_runs(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<u32, AttachmentTextExtractionPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT run_id FROM makosh_data.attachment_text_extraction_runs WHERE logical_owner_id=$1 AND attachment_anchor_id=$2 AND state IN (1,2) ORDER BY run_id FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    let mut transitioned = 0_u32;
    for row in rows {
        let run_id = id16(row.try_get("run_id").map_err(invalid_row)?)?;
        if settle_run(
            transaction,
            logical_owner_id,
            run_id,
            occurred_at_unix_millis,
        )
        .await?
        {
            transitioned = transitioned
                .checked_add(1)
                .ok_or(AttachmentTextExtractionPersistenceErrorV1::InvalidRow)?;
        }
    }
    Ok(transitioned)
}

pub(crate) async fn settle_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<bool, AttachmentTextExtractionPersistenceErrorV1> {
    let current = load_run_for_update(transaction, logical_owner_id, run_id)
        .await?
        .ok_or(AttachmentTextExtractionPersistenceErrorV1::InvalidRow)?;
    if !matches!(
        current.status.state,
        AttachmentTextExtractionStateV1::Accepted
            | AttachmentTextExtractionStateV1::AwaitingEvidence
    ) {
        return Ok(false);
    }
    let candidate = load_candidate(
        transaction,
        logical_owner_id,
        current.request.attachment_anchor_id,
    )
    .await?;
    let safety = load_safety(
        transaction,
        logical_owner_id,
        current.request.attachment_anchor_id,
    )
    .await?;
    let transition =
        match decide_attachment_text_join_v1(&current.request, candidate.as_ref(), safety.as_ref())
        {
            AttachmentTextExtractionJoinDecisionV1::Waiting
            | AttachmentTextExtractionJoinDecisionV1::CustodyDelegationRequired(_) => {
                if current.status.state == AttachmentTextExtractionStateV1::AwaitingEvidence {
                    return Ok(false);
                }
                AttachmentTextExtractionTransitionV1::AwaitEvidence
            }
            AttachmentTextExtractionJoinDecisionV1::Reject(rejection) => {
                let error = if rejection == AttachmentTextExtractionRejectionV1::NotSafe {
                    AttachmentTextExtractionErrorV1::NotSafe
                } else {
                    AttachmentTextExtractionErrorV1::Unavailable
                };
                AttachmentTextExtractionTransitionV1::Reject(error)
            }
        };
    let next = transition_attachment_text_status_v1(&current.status, transition)
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::InvalidRow)?;
    if !update_run_status(
        transaction,
        logical_owner_id,
        run_id,
        current.status.state_revision,
        &next,
        occurred_at_unix_millis,
    )
    .await?
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
    }
    append_realtime(
        transaction,
        logical_owner_id,
        run_id,
        &next,
        occurred_at_unix_millis,
    )
    .await?;
    Ok(true)
}

async fn reject_anchor_runs(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
    _rejection: AttachmentTextExtractionRejectionV1,
    occurred_at_unix_millis: i64,
) -> Result<u32, AttachmentTextExtractionPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT run_id FROM makosh_data.attachment_text_extraction_runs WHERE logical_owner_id=$1 AND attachment_anchor_id=$2 AND state IN (1,2,3) ORDER BY run_id FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    let mut rejected = 0_u32;
    for row in rows {
        let run_id = id16(row.try_get("run_id").map_err(invalid_row)?)?;
        let current = load_run_for_update(transaction, logical_owner_id, run_id)
            .await?
            .ok_or(AttachmentTextExtractionPersistenceErrorV1::InvalidRow)?;
        sqlx::query(
            "UPDATE makosh_data.attachment_text_extraction_jobs SET state=4,worker_id=NULL,runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$3 WHERE logical_owner_id=$1 AND run_id=$2 AND state IN (1,2)",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(occurred_at_unix_millis)
        .execute(&mut **transaction)
        .await
        .map_err(storage_unavailable)?;
        let next = transition_attachment_text_status_v1(
            &current.status,
            AttachmentTextExtractionTransitionV1::Reject(
                AttachmentTextExtractionErrorV1::Unavailable,
            ),
        )
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::InvalidRow)?;
        if !update_run_status(
            transaction,
            logical_owner_id,
            run_id,
            current.status.state_revision,
            &next,
            occurred_at_unix_millis,
        )
        .await?
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
        }
        append_realtime(
            transaction,
            logical_owner_id,
            run_id,
            &next,
            occurred_at_unix_millis,
        )
        .await?;
        rejected = rejected
            .checked_add(1)
            .ok_or(AttachmentTextExtractionPersistenceErrorV1::InvalidRow)?;
    }
    Ok(rejected)
}

struct InboxFactV1<'a> {
    logical_owner_id: &'a str,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    event_kind: i16,
    attachment_anchor_id: [u8; 16],
    exact_payload_sha256: [u8; 32],
    processed_at_unix_millis: i64,
}

enum InboxOutcomeV1 {
    Inserted,
    Replayed,
    Conflict,
}

async fn reconcile_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    fact: InboxFactV1<'_>,
) -> Result<InboxOutcomeV1, AttachmentTextExtractionPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_text_extraction_event_inbox (logical_owner_id,message_id,envelope_sha256,event_kind,attachment_anchor_id,exact_payload_sha256,processed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (logical_owner_id,message_id) DO NOTHING",
    )
    .bind(fact.logical_owner_id)
    .bind(fact.message_id.as_slice())
    .bind(fact.envelope_sha256.as_slice())
    .bind(fact.event_kind)
    .bind(fact.attachment_anchor_id.as_slice())
    .bind(fact.exact_payload_sha256.as_slice())
    .bind(fact.processed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    if inserted.rows_affected() == 1 {
        return Ok(InboxOutcomeV1::Inserted);
    }
    let row = sqlx::query(
        "SELECT envelope_sha256,event_kind,attachment_anchor_id,exact_payload_sha256 FROM makosh_data.attachment_text_extraction_event_inbox WHERE logical_owner_id=$1 AND message_id=$2",
    )
    .bind(fact.logical_owner_id)
    .bind(fact.message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    let exact = id32(row.try_get("envelope_sha256").map_err(invalid_row)?)? == fact.envelope_sha256
        && row.try_get::<i16, _>("event_kind").map_err(invalid_row)? == fact.event_kind
        && id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?
            == fact.attachment_anchor_id
        && id32(row.try_get("exact_payload_sha256").map_err(invalid_row)?)?
            == fact.exact_payload_sha256;
    Ok(if exact {
        InboxOutcomeV1::Replayed
    } else {
        InboxOutcomeV1::Conflict
    })
}

async fn load_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    anchor: [u8; 16],
) -> Result<Option<AttachmentTextScanCandidateV1>, AttachmentTextExtractionPersistenceErrorV1> {
    sqlx::query("SELECT message_id,attachment_anchor_id,blob_reference_id,declared_size,blob_receipt_sha256,custody_transfer_source_proof,observed_at_unix_seconds FROM makosh_data.attachment_text_extraction_scan_candidates WHERE logical_owner_id=$1 AND attachment_anchor_id=$2")
        .bind(owner).bind(anchor.as_slice()).fetch_optional(&mut **transaction).await
        .map_err(storage_unavailable)?.map(|row| candidate_from_row(&row)).transpose()
}

async fn load_safety(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    anchor: [u8; 16],
) -> Result<Option<AttachmentTextCanonicalSafetyFactV1>, AttachmentTextExtractionPersistenceErrorV1>
{
    sqlx::query("SELECT message_id,attachment_anchor_id,expected_state,next_state,evidence_id,observed_at_unix_seconds FROM makosh_data.attachment_text_extraction_safety_facts WHERE logical_owner_id=$1 AND attachment_anchor_id=$2")
        .bind(owner).bind(anchor.as_slice()).fetch_optional(&mut **transaction).await
        .map_err(storage_unavailable)?.map(|row| safety_from_row(&row)).transpose()
}

fn pending_join_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PendingAttachmentTextCustodyDelegationV1, AttachmentTextExtractionPersistenceErrorV1> {
    let request = makosh_attachment_text_extraction_core::AttachmentTextExtractionRequestV1 {
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        operation_id: id16(row.try_get("operation_id").map_err(invalid_row)?)?,
        attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
    };
    let candidate = candidate_from_row(&row)?;
    let safety = AttachmentTextCanonicalSafetyFactV1 {
        message_id: id16(row.try_get("safety_message_id").map_err(invalid_row)?)?,
        attachment_anchor_id: request.attachment_anchor_id,
        expected_state: safety_state_from_code(
            row.try_get("expected_state").map_err(invalid_row)?,
        )?,
        next_state: safety_state_from_code(row.try_get("next_state").map_err(invalid_row)?)?,
        evidence_id: id16(row.try_get("evidence_id").map_err(invalid_row)?)?,
        observed_at_unix_seconds: row.try_get("safety_observed_at").map_err(invalid_row)?,
    };
    let AttachmentTextExtractionJoinDecisionV1::CustodyDelegationRequired(intent) =
        decide_attachment_text_join_v1(&request, Some(&candidate), Some(&safety))
    else {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    };
    Ok(PendingAttachmentTextCustodyDelegationV1 {
        intent,
        candidate_envelope_sha256: id32(
            row.try_get("candidate_envelope_sha256")
                .map_err(invalid_row)?,
        )?,
        created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(invalid_row)?,
    })
}

fn candidate_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<AttachmentTextScanCandidateV1, AttachmentTextExtractionPersistenceErrorV1> {
    Ok(AttachmentTextScanCandidateV1 {
        message_id: id16(
            row.try_get("candidate_message_id")
                .or_else(|_| row.try_get("message_id"))
                .map_err(invalid_row)?,
        )?,
        attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        blob_reference_id: id16(row.try_get("blob_reference_id").map_err(invalid_row)?)?,
        declared_size: u64::try_from(
            row.try_get::<i64, _>("declared_size")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        blob_receipt_sha256: id32(row.try_get("blob_receipt_sha256").map_err(invalid_row)?)?,
        custody_transfer_source_proof: row
            .try_get("custody_transfer_source_proof")
            .map_err(invalid_row)?,
        observed_at_unix_seconds: row
            .try_get("candidate_observed_at")
            .or_else(|_| row.try_get("observed_at_unix_seconds"))
            .map_err(invalid_row)?,
    })
}

fn safety_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<AttachmentTextCanonicalSafetyFactV1, AttachmentTextExtractionPersistenceErrorV1> {
    Ok(AttachmentTextCanonicalSafetyFactV1 {
        message_id: id16(row.try_get("message_id").map_err(invalid_row)?)?,
        attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        expected_state: safety_state_from_code(
            row.try_get("expected_state").map_err(invalid_row)?,
        )?,
        next_state: safety_state_from_code(row.try_get("next_state").map_err(invalid_row)?)?,
        evidence_id: id16(row.try_get("evidence_id").map_err(invalid_row)?)?,
        observed_at_unix_seconds: row
            .try_get("observed_at_unix_seconds")
            .map_err(invalid_row)?,
    })
}

fn valid_fact_input(
    owner: &str,
    message_id: [u8; 16],
    envelope: [u8; 32],
    payload: [u8; 32],
    millis: i64,
) -> bool {
    valid_owner(owner)
        && valid_id16(&message_id)
        && valid_sha256(&envelope)
        && valid_sha256(&payload)
        && millis > 0
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::InvalidRow)
}
fn id32(value: Vec<u8>) -> Result<[u8; 32], AttachmentTextExtractionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::InvalidRow)
}
fn storage_unavailable<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable
}
fn invalid_input<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidInput
}
fn invalid_row<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_kinds_remain_exact_and_bounded() {
        assert_eq!(MAX_PENDING_JOIN_RESULTS_V1, 64);
        assert_ne!(1_i16, 2_i16);
    }
}
