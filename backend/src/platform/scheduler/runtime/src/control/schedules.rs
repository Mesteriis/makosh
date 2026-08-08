//! Exact schedule mutation conversion at the authenticated Scheduler boundary.

use makosh_clock_protocol::UtcMillisV1;
use makosh_runtime_protocol::v1::{
    SchedulerScheduleUpsertOutcomeV1, UpsertSchedulerScheduleRequestV1,
    UpsertSchedulerScheduleResponseV1,
};
use makosh_scheduler_persistence::{
    SchedulerScheduleUpsertOutcomeV1 as StoreOutcome, SchedulerScheduleUpsertV1,
};
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobContractBindingV1, JobKindV1, OpaqueScheduleScopeV1, ScheduleIdV1,
    SchedulePolicyV1, ScheduleRevisionV1, ScheduleSpecV1,
};

pub(super) fn upsert_from_request(
    request: UpsertSchedulerScheduleRequestV1,
) -> Result<SchedulerScheduleUpsertV1, String> {
    let schedule_id = ScheduleIdV1::new(
        request
            .schedule_id
            .try_into()
            .map_err(|_| "schedule_rejected".to_owned())?,
    )
    .map_err(|_| "schedule_rejected".to_owned())?;
    let revision = ScheduleRevisionV1::new(request.schedule_revision)
        .map_err(|_| "schedule_rejected".to_owned())?;
    let job_kind = JobKindV1::new(
        request.job_owner,
        request.job_name,
        u16::try_from(request.job_major).map_err(|_| "schedule_rejected".to_owned())?,
    )
    .map_err(|_| "schedule_rejected".to_owned())?;
    let binding = JobContractBindingV1::new(
        job_kind,
        request.contract_name,
        request.contract_revision,
        request
            .contract_schema_sha256
            .try_into()
            .map_err(|_| "schedule_rejected".to_owned())?,
    )
    .map_err(|_| "schedule_rejected".to_owned())?;
    let scope =
        OpaqueScheduleScopeV1::new(request.scope_id).map_err(|_| "schedule_rejected".to_owned())?;
    let concurrency_key = ConcurrencyKeyV1::new(request.concurrency_key)
        .map_err(|_| "schedule_rejected".to_owned())?;
    let policy = SchedulePolicyV1::from_canonical_bytes(&request.policy_canonical_bytes)
        .map_err(|_| "schedule_rejected".to_owned())?;
    Ok(SchedulerScheduleUpsertV1::new(
        ScheduleSpecV1::new(
            schedule_id,
            revision,
            binding,
            scope,
            concurrency_key,
            request.enabled,
            policy,
        ),
        UtcMillisV1::new(request.next_due_at_unix_millis),
        UtcMillisV1::new(request.updated_at_unix_millis),
    ))
}

pub(super) fn response(
    outcome: StoreOutcome,
    schedule_revision: u64,
) -> UpsertSchedulerScheduleResponseV1 {
    let outcome = match outcome {
        StoreOutcome::Inserted => SchedulerScheduleUpsertOutcomeV1::Inserted,
        StoreOutcome::Updated => SchedulerScheduleUpsertOutcomeV1::Updated,
        StoreOutcome::Unchanged => SchedulerScheduleUpsertOutcomeV1::Unchanged,
    };
    UpsertSchedulerScheduleResponseV1 {
        outcome: outcome as i32,
        schedule_revision,
    }
}
