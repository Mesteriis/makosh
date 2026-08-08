//! Exact event inbox and order-independent Preview evidence join persistence.

use makosh_attachment_preview_api::wire::{AttachmentPreviewErrorCodeV1, AttachmentPreviewStateV1};
use makosh_attachment_preview_core::{
    AttachmentPreviewEvidenceJoinV1, AttachmentPreviewRequestFactV1, AttachmentPreviewSafetyFactV1,
    AttachmentPreviewScanCandidateFactV1, AttachmentPreviewTransitionV1,
    transition_attachment_preview_status_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentPreviewPersistenceErrorV1, AttachmentPreviewPersistenceV1,
    PendingAttachmentPreviewCustodyDelegationV1, PersistAttachmentPreviewFactOutcomeV1,
    model::{
        safety_state_code, safety_state_from_code, valid_owner, valid_sha256,
        valid_timestamp_millis,
    },
    repository::{
        append_realtime, id16, id32, invalid_row, load_run_for_update, lock_anchor,
        storage_unavailable, update_run_status,
    },
};

const MAX_PENDING_JOIN_RESULTS_V1: u16 = 64;

impl AttachmentPreviewPersistenceV1 {
    pub async fn persist_scan_candidate(
        &self,
        logical_owner_id: &str,
        candidate: &AttachmentPreviewScanCandidateFactV1,
        exact_payload_sha256: [u8; 32],
        processed_at_unix_millis: i64,
    ) -> Result<PersistAttachmentPreviewFactOutcomeV1, AttachmentPreviewPersistenceErrorV1> {
        let mut validation = AttachmentPreviewEvidenceJoinV1::default();
        if !valid_owner(logical_owner_id)
            || !valid_sha256(&exact_payload_sha256)
            || !valid_timestamp_millis(processed_at_unix_millis)
            || validation.observe_candidate(candidate.clone()).is_err()
        {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        lock_anchor(
            &mut transaction,
            logical_owner_id,
            candidate.attachment_anchor_id,
        )
        .await?;
        let inbox = reconcile_inbox(
            &mut transaction,
            InboxFactV1 {
                logical_owner_id,
                message_id: candidate.candidate_message_id,
                envelope_sha256: candidate.candidate_envelope_sha256,
                event_kind: 1,
                attachment_anchor_id: candidate.attachment_anchor_id,
                exact_payload_sha256,
                processed_at_unix_millis,
            },
        )
        .await?;
        if inbox == InboxOutcomeV1::Conflict {
            let rejected = reject_anchor_runs(
                &mut transaction,
                logical_owner_id,
                candidate.attachment_anchor_id,
                AttachmentPreviewErrorCodeV1::Unavailable,
                processed_at_unix_millis,
            )
            .await?;
            transaction.commit().await.map_err(storage_unavailable)?;
            return Ok(PersistAttachmentPreviewFactOutcomeV1::Conflict {
                rejected_runs: rejected,
            });
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_preview_scan_candidates (logical_owner_id,attachment_anchor_id,message_id,envelope_sha256,exact_payload_sha256,source_reference_id,declared_size,source_receipt_sha256,custody_transfer_source_proof,observed_at_unix_seconds) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT (logical_owner_id,attachment_anchor_id) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(candidate.attachment_anchor_id.as_slice())
        .bind(candidate.candidate_message_id.as_slice())
        .bind(candidate.candidate_envelope_sha256.as_slice())
        .bind(exact_payload_sha256.as_slice())
        .bind(candidate.source_reference_id.as_slice())
        .bind(i64::try_from(candidate.declared_size).map_err(invalid_input)?)
        .bind(candidate.source_receipt_sha256.as_slice())
        .bind(&candidate.custody_transfer_source_proof)
        .bind(candidate.observed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() == 0 {
            let existing = load_candidate(
                &mut transaction,
                logical_owner_id,
                candidate.attachment_anchor_id,
            )
            .await?
            .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
            if existing != *candidate {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    candidate.attachment_anchor_id,
                    AttachmentPreviewErrorCodeV1::Unavailable,
                    processed_at_unix_millis,
                )
                .await?;
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentPreviewFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
        }
        let transitioned = settle_anchor_runs(
            &mut transaction,
            logical_owner_id,
            candidate.attachment_anchor_id,
            processed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        if inbox == InboxOutcomeV1::Replayed && inserted.rows_affected() == 0 && transitioned == 0 {
            Ok(PersistAttachmentPreviewFactOutcomeV1::Replayed)
        } else {
            Ok(PersistAttachmentPreviewFactOutcomeV1::Recorded {
                transitioned_runs: transitioned,
            })
        }
    }

    pub async fn persist_safety_fact(
        &self,
        logical_owner_id: &str,
        safety: AttachmentPreviewSafetyFactV1,
        envelope_sha256: [u8; 32],
        exact_payload_sha256: [u8; 32],
        processed_at_unix_millis: i64,
    ) -> Result<PersistAttachmentPreviewFactOutcomeV1, AttachmentPreviewPersistenceErrorV1> {
        let mut validation = AttachmentPreviewEvidenceJoinV1::default();
        if !valid_owner(logical_owner_id)
            || !valid_sha256(&envelope_sha256)
            || !valid_sha256(&exact_payload_sha256)
            || !valid_timestamp_millis(processed_at_unix_millis)
            || validation.observe_safety(safety).is_err()
        {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        lock_anchor(
            &mut transaction,
            logical_owner_id,
            safety.attachment_anchor_id,
        )
        .await?;
        let inbox = reconcile_inbox(
            &mut transaction,
            InboxFactV1 {
                logical_owner_id,
                message_id: safety.safety_message_id,
                envelope_sha256,
                event_kind: 2,
                attachment_anchor_id: safety.attachment_anchor_id,
                exact_payload_sha256,
                processed_at_unix_millis,
            },
        )
        .await?;
        if inbox == InboxOutcomeV1::Conflict {
            let rejected = reject_anchor_runs(
                &mut transaction,
                logical_owner_id,
                safety.attachment_anchor_id,
                AttachmentPreviewErrorCodeV1::Unavailable,
                processed_at_unix_millis,
            )
            .await?;
            transaction.commit().await.map_err(storage_unavailable)?;
            return Ok(PersistAttachmentPreviewFactOutcomeV1::Conflict {
                rejected_runs: rejected,
            });
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_preview_safety_facts (logical_owner_id,attachment_anchor_id,message_id,envelope_sha256,exact_payload_sha256,expected_state,next_state,evidence_id,observed_at_unix_seconds) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (logical_owner_id,attachment_anchor_id) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(safety.attachment_anchor_id.as_slice())
        .bind(safety.safety_message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .bind(exact_payload_sha256.as_slice())
        .bind(safety_state_code(safety.expected_state))
        .bind(safety_state_code(safety.next_state))
        .bind(safety.safety_evidence_id.as_slice())
        .bind(safety.observed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() == 0 {
            let existing = load_safety(
                &mut transaction,
                logical_owner_id,
                safety.attachment_anchor_id,
            )
            .await?
            .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
            if existing.fact != safety || existing.envelope_sha256 != envelope_sha256 {
                let rejected = reject_anchor_runs(
                    &mut transaction,
                    logical_owner_id,
                    safety.attachment_anchor_id,
                    AttachmentPreviewErrorCodeV1::Unavailable,
                    processed_at_unix_millis,
                )
                .await?;
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentPreviewFactOutcomeV1::Conflict {
                    rejected_runs: rejected,
                });
            }
        }
        let transitioned = settle_anchor_runs(
            &mut transaction,
            logical_owner_id,
            safety.attachment_anchor_id,
            processed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        if inbox == InboxOutcomeV1::Replayed && inserted.rows_affected() == 0 && transitioned == 0 {
            Ok(PersistAttachmentPreviewFactOutcomeV1::Replayed)
        } else {
            Ok(PersistAttachmentPreviewFactOutcomeV1::Recorded {
                transitioned_runs: transitioned,
            })
        }
    }

    pub async fn pending_custody_delegations(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<Vec<PendingAttachmentPreviewCustodyDelegationV1>, AttachmentPreviewPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id) || !(1..=MAX_PENDING_JOIN_RESULTS_V1).contains(&limit) {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT r.run_id,r.operation_id,r.attachment_anchor_id,r.created_at_unix_millis,c.message_id AS candidate_message_id,c.envelope_sha256 AS candidate_envelope_sha256,c.source_reference_id,c.declared_size,c.source_receipt_sha256,c.custody_transfer_source_proof,c.observed_at_unix_seconds AS candidate_observed_at,s.message_id AS safety_message_id,s.envelope_sha256 AS safety_envelope_sha256,s.evidence_id,s.expected_state,s.next_state,s.observed_at_unix_seconds AS safety_observed_at FROM makosh_data.attachment_preview_runs r JOIN makosh_data.attachment_preview_scan_candidates c ON c.logical_owner_id=r.logical_owner_id AND c.attachment_anchor_id=r.attachment_anchor_id JOIN makosh_data.attachment_preview_safety_facts s ON s.logical_owner_id=r.logical_owner_id AND s.attachment_anchor_id=r.attachment_anchor_id LEFT JOIN makosh_data.attachment_preview_custody_outbox o ON o.logical_owner_id=r.logical_owner_id AND o.run_id=r.run_id WHERE r.logical_owner_id=$1 AND r.state=2 AND s.expected_state=3 AND s.next_state=4 AND o.run_id IS NULL ORDER BY r.created_at_unix_millis,r.run_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_unavailable)?;
        rows.into_iter()
            .map(|row| pending_from_row(logical_owner_id, row))
            .collect()
    }
}

pub(crate) async fn settle_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<bool, AttachmentPreviewPersistenceErrorV1> {
    let current = load_run_for_update(transaction, logical_owner_id, run_id)
        .await?
        .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
    if !matches!(
        current.status.state,
        AttachmentPreviewStateV1::Accepted | AttachmentPreviewStateV1::AwaitingEvidence
    ) {
        return Ok(false);
    }
    let candidate =
        load_candidate(transaction, logical_owner_id, current.attachment_anchor_id).await?;
    let safety = load_safety(transaction, logical_owner_id, current.attachment_anchor_id).await?;
    let mut join = AttachmentPreviewEvidenceJoinV1::default();
    join.observe_request(AttachmentPreviewRequestFactV1 {
        run_id: current.run_id,
        operation_id: current.operation_id,
        attachment_anchor_id: current.attachment_anchor_id,
        logical_owner_id: logical_owner_id.to_owned(),
    })
    .map_err(|_| AttachmentPreviewPersistenceErrorV1::EvidenceConflict)?;
    if let Some(candidate) = candidate {
        join.observe_candidate(candidate)
            .map_err(|_| AttachmentPreviewPersistenceErrorV1::EvidenceConflict)?;
    }
    if let Some(safety) = safety {
        join.observe_safety(safety.fact)
            .map_err(|_| AttachmentPreviewPersistenceErrorV1::EvidenceConflict)?;
    }
    let transition = match join.delegation_intent() {
        Ok(Some(_)) if current.status.state == AttachmentPreviewStateV1::Accepted => {
            Some(AttachmentPreviewTransitionV1::AwaitEvidence)
        }
        Ok(_) => None,
        Err(makosh_attachment_preview_core::AttachmentPreviewJoinErrorV1::NotSafe) => Some(
            AttachmentPreviewTransitionV1::Reject(AttachmentPreviewErrorCodeV1::NotSafe),
        ),
        Err(_) => Some(AttachmentPreviewTransitionV1::Reject(
            AttachmentPreviewErrorCodeV1::Unavailable,
        )),
    };
    let Some(transition) = transition else {
        return Ok(false);
    };
    let next = transition_attachment_preview_status_v1(&current.status, transition)
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
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
        return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
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

async fn settle_anchor_runs(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<u32, AttachmentPreviewPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT run_id FROM makosh_data.attachment_preview_runs WHERE logical_owner_id=$1 AND attachment_anchor_id=$2 AND state IN (1,2) ORDER BY run_id FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    let mut transitioned = 0_u32;
    for row in rows {
        let run_id = id16(row.try_get("run_id").map_err(invalid_row)?)?;
        transitioned += u32::from(
            settle_run(
                transaction,
                logical_owner_id,
                run_id,
                occurred_at_unix_millis,
            )
            .await?,
        );
    }
    Ok(transitioned)
}

async fn reject_anchor_runs(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
    error: AttachmentPreviewErrorCodeV1,
    occurred_at_unix_millis: i64,
) -> Result<u32, AttachmentPreviewPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT run_id FROM makosh_data.attachment_preview_runs WHERE logical_owner_id=$1 AND attachment_anchor_id=$2 AND state IN (1,2,3) ORDER BY run_id FOR UPDATE",
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
            .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
        let next = transition_attachment_preview_status_v1(
            &current.status,
            AttachmentPreviewTransitionV1::Reject(error),
        )
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
        sqlx::query(
            "UPDATE makosh_data.attachment_preview_jobs SET state=4,worker_id=NULL,runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$3 WHERE logical_owner_id=$1 AND run_id=$2 AND state IN (1,2)",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(occurred_at_unix_millis)
        .execute(&mut **transaction)
        .await
        .map_err(storage_unavailable)?;
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
            return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
        }
        append_realtime(
            transaction,
            logical_owner_id,
            run_id,
            &next,
            occurred_at_unix_millis,
        )
        .await?;
        rejected += 1;
    }
    Ok(rejected)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InboxOutcomeV1 {
    Inserted,
    Replayed,
    Conflict,
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

async fn reconcile_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    fact: InboxFactV1<'_>,
) -> Result<InboxOutcomeV1, AttachmentPreviewPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_preview_event_inbox (logical_owner_id,message_id,envelope_sha256,event_kind,attachment_anchor_id,exact_payload_sha256,processed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (logical_owner_id,message_id) DO NOTHING",
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
        "SELECT envelope_sha256,event_kind,attachment_anchor_id,exact_payload_sha256 FROM makosh_data.attachment_preview_event_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
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

struct StoredSafetyV1 {
    fact: AttachmentPreviewSafetyFactV1,
    envelope_sha256: [u8; 32],
}

async fn load_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
) -> Result<Option<AttachmentPreviewScanCandidateFactV1>, AttachmentPreviewPersistenceErrorV1> {
    sqlx::query(
        "SELECT attachment_anchor_id,message_id,envelope_sha256,source_reference_id,declared_size,source_receipt_sha256,custody_transfer_source_proof,observed_at_unix_seconds FROM makosh_data.attachment_preview_scan_candidates WHERE logical_owner_id=$1 AND attachment_anchor_id=$2",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .map(|row| {
        Ok(AttachmentPreviewScanCandidateFactV1 {
            attachment_anchor_id: id16(
                row.try_get("attachment_anchor_id").map_err(invalid_row)?,
            )?,
            candidate_message_id: id16(row.try_get("message_id").map_err(invalid_row)?)?,
            candidate_envelope_sha256: id32(
                row.try_get("envelope_sha256").map_err(invalid_row)?,
            )?,
            source_reference_id: id16(
                row.try_get("source_reference_id").map_err(invalid_row)?,
            )?,
            declared_size: u64::try_from(
                row.try_get::<i64, _>("declared_size")
                    .map_err(invalid_row)?,
            )
            .map_err(invalid_row)?,
            source_receipt_sha256: id32(
                row.try_get("source_receipt_sha256")
                    .map_err(invalid_row)?,
            )?,
            custody_transfer_source_proof: row
                .try_get("custody_transfer_source_proof")
                .map_err(invalid_row)?,
            observed_at_unix_seconds: row
                .try_get("observed_at_unix_seconds")
                .map_err(invalid_row)?,
        })
    })
    .transpose()
}

async fn load_safety(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
) -> Result<Option<StoredSafetyV1>, AttachmentPreviewPersistenceErrorV1> {
    sqlx::query(
        "SELECT attachment_anchor_id,message_id,envelope_sha256,expected_state,next_state,evidence_id,observed_at_unix_seconds FROM makosh_data.attachment_preview_safety_facts WHERE logical_owner_id=$1 AND attachment_anchor_id=$2",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .map(|row| {
        Ok(StoredSafetyV1 {
            fact: AttachmentPreviewSafetyFactV1 {
                attachment_anchor_id: id16(
                    row.try_get("attachment_anchor_id").map_err(invalid_row)?,
                )?,
                safety_message_id: id16(row.try_get("message_id").map_err(invalid_row)?)?,
                safety_evidence_id: id16(row.try_get("evidence_id").map_err(invalid_row)?)?,
                expected_state: safety_state_from_code(
                    row.try_get("expected_state").map_err(invalid_row)?,
                )?,
                next_state: safety_state_from_code(
                    row.try_get("next_state").map_err(invalid_row)?,
                )?,
                observed_at_unix_seconds: row
                    .try_get("observed_at_unix_seconds")
                    .map_err(invalid_row)?,
            },
            envelope_sha256: id32(
                row.try_get("envelope_sha256").map_err(invalid_row)?,
            )?,
        })
    })
    .transpose()
}

fn pending_from_row(
    logical_owner_id: &str,
    row: sqlx::postgres::PgRow,
) -> Result<PendingAttachmentPreviewCustodyDelegationV1, AttachmentPreviewPersistenceErrorV1> {
    let request = AttachmentPreviewRequestFactV1 {
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        operation_id: id16(row.try_get("operation_id").map_err(invalid_row)?)?,
        attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        logical_owner_id: logical_owner_id.to_owned(),
    };
    let candidate = AttachmentPreviewScanCandidateFactV1 {
        attachment_anchor_id: request.attachment_anchor_id,
        candidate_message_id: id16(row.try_get("candidate_message_id").map_err(invalid_row)?)?,
        candidate_envelope_sha256: id32(
            row.try_get("candidate_envelope_sha256")
                .map_err(invalid_row)?,
        )?,
        source_reference_id: id16(row.try_get("source_reference_id").map_err(invalid_row)?)?,
        declared_size: u64::try_from(
            row.try_get::<i64, _>("declared_size")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        source_receipt_sha256: id32(row.try_get("source_receipt_sha256").map_err(invalid_row)?)?,
        custody_transfer_source_proof: row
            .try_get("custody_transfer_source_proof")
            .map_err(invalid_row)?,
        observed_at_unix_seconds: row.try_get("candidate_observed_at").map_err(invalid_row)?,
    };
    let safety = AttachmentPreviewSafetyFactV1 {
        attachment_anchor_id: request.attachment_anchor_id,
        safety_message_id: id16(row.try_get("safety_message_id").map_err(invalid_row)?)?,
        safety_evidence_id: id16(row.try_get("evidence_id").map_err(invalid_row)?)?,
        expected_state: safety_state_from_code(
            row.try_get("expected_state").map_err(invalid_row)?,
        )?,
        next_state: safety_state_from_code(row.try_get("next_state").map_err(invalid_row)?)?,
        observed_at_unix_seconds: row.try_get("safety_observed_at").map_err(invalid_row)?,
    };
    let mut join = AttachmentPreviewEvidenceJoinV1::default();
    join.observe_request(request)
        .and_then(|()| join.observe_candidate(candidate))
        .and_then(|()| join.observe_safety(safety))
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
    let intent = join
        .delegation_intent()
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidRow)?
        .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
    Ok(PendingAttachmentPreviewCustodyDelegationV1 {
        intent,
        created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(invalid_row)?,
    })
}

fn invalid_input<T>(_: T) -> AttachmentPreviewPersistenceErrorV1 {
    AttachmentPreviewPersistenceErrorV1::InvalidInput
}
