use makosh_events_protocol::{
    delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
    validation::envelope::decode_envelope_v1,
};
use sqlx::{Postgres, Row, Transaction, query, query_scalar};

use super::request::{
    SchedulerScheduleControlApplyErrorV1, SchedulerScheduleControlApplyOutcomeV1,
    SchedulerScheduleControlAuthorityV1, SchedulerScheduleControlDecisionV1,
    SchedulerScheduleControlMutationV1, SchedulerScheduleControlRejectionV1,
    SchedulerScheduleControlRequestV1,
};
use crate::{
    SchedulerPostgresStoreV1, SchedulerScheduleStoreErrorV1,
    store::{concurrency::ensure_slot, schedules::upsert_locked},
};

impl SchedulerPostgresStoreV1 {
    /// Deduplicates, mutates Scheduler state and persists exact result bytes in
    /// one PostgreSQL transaction. Broker ACK belongs to the caller only after
    /// this method commits.
    pub async fn apply_schedule_control<F>(
        &self,
        request: &SchedulerScheduleControlRequestV1,
        result_factory: F,
    ) -> Result<SchedulerScheduleControlApplyOutcomeV1, SchedulerScheduleControlApplyErrorV1>
    where
        F: FnOnce(
            SchedulerScheduleControlDecisionV1,
        ) -> Result<OutboxRecordV1, SchedulerScheduleControlApplyErrorV1>,
    {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
        if let Some(outcome) = duplicate(&mut transaction, request).await? {
            transaction
                .commit()
                .await
                .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
            return Ok(outcome);
        }

        let decision = apply_mutation(&mut transaction, request).await?;
        let result = result_factory(decision)?;
        validate_result_correlation(request.command(), &result)?;
        persist_acceptance(&mut transaction, request, decision, &result).await?;
        transaction
            .commit()
            .await
            .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
        Ok(SchedulerScheduleControlApplyOutcomeV1::Applied { decision, result })
    }
}

fn validate_result_correlation(
    command: &OutboxRecordV1,
    result: &OutboxRecordV1,
) -> Result<(), SchedulerScheduleControlApplyErrorV1> {
    let command_envelope = decode_envelope_v1(command.exact_bytes())
        .map_err(|_| SchedulerScheduleControlApplyErrorV1::InvalidRequest)?;
    let result_envelope = decode_envelope_v1(result.exact_bytes())
        .map_err(|_| SchedulerScheduleControlApplyErrorV1::InvalidResult)?;
    let Some(Semantics::Command(command_metadata)) = command_envelope.semantics else {
        return Err(SchedulerScheduleControlApplyErrorV1::InvalidRequest);
    };
    let Some(Semantics::Result(result_metadata)) = result_envelope.semantics else {
        return Err(SchedulerScheduleControlApplyErrorV1::InvalidResult);
    };
    (result_metadata.command_id == command_metadata.command_id
        && result_metadata.command_message_id == command.message_id()
        && result_envelope.causation_message_id == command.message_id()
        && result_envelope.correlation_id == command_envelope.correlation_id)
        .then_some(())
        .ok_or(SchedulerScheduleControlApplyErrorV1::InvalidResult)
}

async fn duplicate(
    transaction: &mut Transaction<'_, Postgres>,
    request: &SchedulerScheduleControlRequestV1,
) -> Result<Option<SchedulerScheduleControlApplyOutcomeV1>, SchedulerScheduleControlApplyErrorV1> {
    let row = query(
        "SELECT inbox.command_envelope_sha256, inbox.decision, results.message_id, results.envelope_sha256, results.exact_envelope_bytes FROM makosh_platform.scheduler_schedule_control_inbox AS inbox JOIN makosh_platform.scheduler_schedule_control_results AS results ON results.command_message_id = inbox.command_message_id AND results.message_id = inbox.result_message_id WHERE inbox.command_message_id = $1 FOR UPDATE OF inbox, results",
    )
    .bind(request.command().message_id().to_vec())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let command_hash: Vec<u8> = row
        .try_get("command_envelope_sha256")
        .map_err(|_| SchedulerScheduleControlApplyErrorV1::CorruptState)?;
    if command_hash != request.command().envelope_sha256() {
        return Err(SchedulerScheduleControlApplyErrorV1::HashConflict);
    }
    let decision = decode_decision(
        row.try_get("decision")
            .map_err(|_| SchedulerScheduleControlApplyErrorV1::CorruptState)?,
    )?;
    let result_message_id: Vec<u8> = row
        .try_get("message_id")
        .map_err(|_| SchedulerScheduleControlApplyErrorV1::CorruptState)?;
    let result_hash: Vec<u8> = row
        .try_get("envelope_sha256")
        .map_err(|_| SchedulerScheduleControlApplyErrorV1::CorruptState)?;
    let result_bytes: Vec<u8> = row
        .try_get("exact_envelope_bytes")
        .map_err(|_| SchedulerScheduleControlApplyErrorV1::CorruptState)?;
    let result = OutboxRecordV1::accept(result_bytes)
        .map_err(|_| SchedulerScheduleControlApplyErrorV1::CorruptState)?;
    if result_message_id != result.message_id() || result_hash != result.envelope_sha256() {
        return Err(SchedulerScheduleControlApplyErrorV1::CorruptState);
    }
    Ok(Some(SchedulerScheduleControlApplyOutcomeV1::Duplicate {
        decision,
        result,
    }))
}

async fn apply_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    request: &SchedulerScheduleControlRequestV1,
) -> Result<SchedulerScheduleControlDecisionV1, SchedulerScheduleControlApplyErrorV1> {
    match request.mutation() {
        SchedulerScheduleControlMutationV1::Ensure(change) => {
            let authority = claim_authority(
                transaction,
                change.spec().schedule_id().bytes(),
                request.authority(),
                request.received_at().value(),
            )
            .await?;
            if !authority.matches {
                return Ok(SchedulerScheduleControlDecisionV1::Rejected(
                    SchedulerScheduleControlRejectionV1::ForeignAuthority,
                ));
            }
            let slot = ensure_slot(
                transaction,
                change.spec().concurrency_key(),
                change.spec().policy(),
                change.updated_at(),
            )
            .await;
            match slot {
                Ok(()) => {}
                Err(crate::SchedulerRunClaimErrorV1::ConcurrencyBusy) => {
                    rollback_new_authority(
                        transaction,
                        change.spec().schedule_id().bytes(),
                        authority.inserted,
                    )
                    .await?;
                    return Ok(SchedulerScheduleControlDecisionV1::Rejected(
                        SchedulerScheduleControlRejectionV1::ConcurrencyBusy,
                    ));
                }
                Err(crate::SchedulerRunClaimErrorV1::Unavailable) => {
                    return Err(SchedulerScheduleControlApplyErrorV1::Unavailable);
                }
                Err(_) => return Err(SchedulerScheduleControlApplyErrorV1::CorruptState),
            }
            match upsert_locked(transaction, change).await {
                Ok(_) => Ok(SchedulerScheduleControlDecisionV1::Ensured),
                Err(error) => {
                    rollback_new_authority(
                        transaction,
                        change.spec().schedule_id().bytes(),
                        authority.inserted,
                    )
                    .await?;
                    map_schedule_error(error)
                }
            }
        }
        SchedulerScheduleControlMutationV1::Cancel {
            schedule_id,
            expected_revision,
            cancelled_at,
        } => {
            if !load_authority(transaction, schedule_id.bytes(), request.authority()).await? {
                return Ok(SchedulerScheduleControlDecisionV1::Rejected(
                    SchedulerScheduleControlRejectionV1::ForeignAuthority,
                ));
            }
            cancel(
                transaction,
                schedule_id.bytes(),
                expected_revision.value(),
                cancelled_at.value(),
            )
            .await
        }
    }
}

struct AuthorityClaimV1 {
    matches: bool,
    inserted: bool,
}

async fn claim_authority(
    transaction: &mut Transaction<'_, Postgres>,
    schedule_id: [u8; 16],
    authority: &SchedulerScheduleControlAuthorityV1,
    created_at: i64,
) -> Result<AuthorityClaimV1, SchedulerScheduleControlApplyErrorV1> {
    let inserted = query(
        "INSERT INTO makosh_platform.scheduler_schedule_control_authorities (schedule_id, source_module_id, source_owner, job_owner, job_name, job_major, created_at_unix_ms) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (schedule_id) DO NOTHING",
    )
    .bind(schedule_id.to_vec())
    .bind(authority.source_module_id())
    .bind(authority.source_owner())
    .bind(authority.job_owner())
    .bind(authority.job_name())
    .bind(i32::from(authority.job_major()))
    .bind(created_at)
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?
    .rows_affected()
        == 1;
    let matches = load_authority(transaction, schedule_id, authority).await?;
    Ok(AuthorityClaimV1 { matches, inserted })
}

async fn load_authority(
    transaction: &mut Transaction<'_, Postgres>,
    schedule_id: [u8; 16],
    authority: &SchedulerScheduleControlAuthorityV1,
) -> Result<bool, SchedulerScheduleControlApplyErrorV1> {
    let row = query(
        "SELECT source_module_id, source_owner, job_owner, job_name, job_major FROM makosh_platform.scheduler_schedule_control_authorities WHERE schedule_id = $1 FOR UPDATE",
    )
    .bind(schedule_id.to_vec())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
    Ok(row.is_some_and(|row| {
        row.try_get::<String, _>("source_module_id").ok().as_deref()
            == Some(authority.source_module_id())
            && row.try_get::<String, _>("source_owner").ok().as_deref()
                == Some(authority.source_owner())
            && row.try_get::<String, _>("job_owner").ok().as_deref() == Some(authority.job_owner())
            && row.try_get::<String, _>("job_name").ok().as_deref() == Some(authority.job_name())
            && row.try_get::<i32, _>("job_major").ok() == Some(i32::from(authority.job_major()))
    }))
}

async fn rollback_new_authority(
    transaction: &mut Transaction<'_, Postgres>,
    schedule_id: [u8; 16],
    inserted: bool,
) -> Result<(), SchedulerScheduleControlApplyErrorV1> {
    if inserted {
        query(
            "DELETE FROM makosh_platform.scheduler_schedule_control_authorities WHERE schedule_id = $1",
        )
        .bind(schedule_id.to_vec())
        .execute(&mut **transaction)
        .await
        .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
    }
    Ok(())
}

async fn cancel(
    transaction: &mut Transaction<'_, Postgres>,
    schedule_id: [u8; 16],
    expected_revision: u64,
    cancelled_at: i64,
) -> Result<SchedulerScheduleControlDecisionV1, SchedulerScheduleControlApplyErrorV1> {
    let persisted_revision: Option<i64> = query_scalar(
        "SELECT schedule_revision FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1 FOR UPDATE",
    )
    .bind(schedule_id.to_vec())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
    let Some(persisted_revision) = persisted_revision else {
        return Ok(SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::UnknownSchedule,
        ));
    };
    if i64::try_from(expected_revision).ok() != Some(persisted_revision) {
        return Ok(SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::StaleRevision,
        ));
    }
    let accepted: bool = query_scalar(
        "SELECT EXISTS (SELECT 1 FROM makosh_platform.scheduler_runs WHERE schedule_id = $1 AND schedule_revision = $2)",
    )
    .bind(schedule_id.to_vec())
    .bind(persisted_revision)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
    if accepted {
        return Ok(SchedulerScheduleControlDecisionV1::TooLate);
    }
    query(
        "UPDATE makosh_platform.scheduler_schedules SET enabled = FALSE, updated_at_unix_ms = $2 WHERE schedule_id = $1",
    )
    .bind(schedule_id.to_vec())
    .bind(cancelled_at)
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
    Ok(SchedulerScheduleControlDecisionV1::Cancelled)
}

fn map_schedule_error(
    error: SchedulerScheduleStoreErrorV1,
) -> Result<SchedulerScheduleControlDecisionV1, SchedulerScheduleControlApplyErrorV1> {
    match error {
        SchedulerScheduleStoreErrorV1::StaleRevision => {
            Ok(SchedulerScheduleControlDecisionV1::Rejected(
                SchedulerScheduleControlRejectionV1::StaleRevision,
            ))
        }
        SchedulerScheduleStoreErrorV1::RevisionConflict => {
            Ok(SchedulerScheduleControlDecisionV1::Rejected(
                SchedulerScheduleControlRejectionV1::RevisionConflict,
            ))
        }
        SchedulerScheduleStoreErrorV1::ConcurrencyBusy => {
            Ok(SchedulerScheduleControlDecisionV1::Rejected(
                SchedulerScheduleControlRejectionV1::ConcurrencyBusy,
            ))
        }
        SchedulerScheduleStoreErrorV1::Unavailable => {
            Err(SchedulerScheduleControlApplyErrorV1::Unavailable)
        }
        SchedulerScheduleStoreErrorV1::InvalidLimit
        | SchedulerScheduleStoreErrorV1::CorruptState => {
            Err(SchedulerScheduleControlApplyErrorV1::CorruptState)
        }
    }
}

async fn persist_acceptance(
    transaction: &mut Transaction<'_, Postgres>,
    request: &SchedulerScheduleControlRequestV1,
    decision: SchedulerScheduleControlDecisionV1,
    result: &OutboxRecordV1,
) -> Result<(), SchedulerScheduleControlApplyErrorV1> {
    query(
        "INSERT INTO makosh_platform.scheduler_schedule_control_results (message_id, command_message_id, envelope_sha256, exact_envelope_bytes, state, created_at_unix_ms) VALUES ($1, $2, $3, $4, 'pending', $5)",
    )
    .bind(result.message_id().to_vec())
    .bind(request.command().message_id().to_vec())
    .bind(result.envelope_sha256().to_vec())
    .bind(result.exact_bytes())
    .bind(request.received_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
    query(
        "INSERT INTO makosh_platform.scheduler_schedule_control_inbox (command_message_id, command_envelope_sha256, operation_id, schedule_id, schedule_revision, decision, result_message_id, received_at_unix_ms) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(request.command().message_id().to_vec())
    .bind(request.command().envelope_sha256().to_vec())
    .bind(request.operation_id().to_vec())
    .bind(request.mutation().schedule_id().bytes().to_vec())
    .bind(
        i64::try_from(request.mutation().schedule_revision().value())
            .map_err(|_| SchedulerScheduleControlApplyErrorV1::InvalidRequest)?,
    )
    .bind(encode_decision(decision))
    .bind(result.message_id().to_vec())
    .bind(request.received_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerScheduleControlApplyErrorV1::Unavailable)?;
    Ok(())
}

fn encode_decision(decision: SchedulerScheduleControlDecisionV1) -> &'static str {
    match decision {
        SchedulerScheduleControlDecisionV1::Ensured => "ensured",
        SchedulerScheduleControlDecisionV1::Cancelled => "cancelled",
        SchedulerScheduleControlDecisionV1::TooLate => "too_late",
        SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::ForeignAuthority,
        ) => "rejected_foreign_authority",
        SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::UnknownSchedule,
        ) => "rejected_unknown_schedule",
        SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::StaleRevision,
        ) => "rejected_stale_revision",
        SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::RevisionConflict,
        ) => "rejected_revision_conflict",
        SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::ConcurrencyBusy,
        ) => "rejected_concurrency_busy",
    }
}

fn decode_decision(
    value: &str,
) -> Result<SchedulerScheduleControlDecisionV1, SchedulerScheduleControlApplyErrorV1> {
    match value {
        "ensured" => Ok(SchedulerScheduleControlDecisionV1::Ensured),
        "cancelled" => Ok(SchedulerScheduleControlDecisionV1::Cancelled),
        "too_late" => Ok(SchedulerScheduleControlDecisionV1::TooLate),
        "rejected_foreign_authority" => Ok(SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::ForeignAuthority,
        )),
        "rejected_unknown_schedule" => Ok(SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::UnknownSchedule,
        )),
        "rejected_stale_revision" => Ok(SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::StaleRevision,
        )),
        "rejected_revision_conflict" => Ok(SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::RevisionConflict,
        )),
        "rejected_concurrency_busy" => Ok(SchedulerScheduleControlDecisionV1::Rejected(
            SchedulerScheduleControlRejectionV1::ConcurrencyBusy,
        )),
        _ => Err(SchedulerScheduleControlApplyErrorV1::CorruptState),
    }
}
