//! Owner-authorized Scheduler lifecycle and schedule-control routes.

use std::path::Path;

use makosh_gateway_protocol::v1::{
    RestartSchedulerRuntimeRequestV1, RestartSchedulerRuntimeResponseV1,
    StartReservedSchedulerRuntimeRequestV1, StartReservedSchedulerRuntimeResponseV1,
    UpsertSchedulerScheduleRequestV1, UpsertSchedulerScheduleResponseV1,
};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{SchedulerRuntimeControlRequestV1, SchedulerRuntimeControlResponseV1},
    validation::scheduler::{
        validate_scheduler_runtime_control_request, validate_scheduler_runtime_control_response,
    },
};
use prost::Message;

use super::{OwnerControlSessions, OwnerResult};
use crate::platform::macos::managed_launch;
use crate::platform::scheduler::{admission as scheduler_admission, launch as scheduler_launch};
use crate::platform::storage::issuance::StorageBindingIssueV1;
use crate::platform::storage::successor as storage_successor;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(super) fn start_reserved(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartReservedSchedulerRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        require_scheduler_registration(store, &request.registration_id)?;
        let reservation = managed_launch::load(supervisor, store, &request.registration_id)?;
        let binding = active_storage_binding(
            store,
            &request.registration_id,
            &request.storage_capability_id,
        )?;
        start_from_reservation(supervisor, store, runtime_dir, reservation, &binding)
    })()
    .map(|runtime_generation| {
        OwnerResult::StartReservedSchedulerRuntime(StartReservedSchedulerRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

pub(super) fn restart(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: RestartSchedulerRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        require_scheduler_registration(store, &request.registration_id)?;
        let issue = StorageBindingIssueV1::new(
            request.role_epoch,
            request.credential_lease_revision,
            request.storage_bundle_revision,
            request
                .storage_bundle_digest
                .try_into()
                .map_err(|_| "Scheduler Storage binding is unavailable".to_owned())?,
        )?;
        let (reservation, binding) = storage_successor::reserve(
            supervisor,
            store,
            &request.registration_id,
            &request.storage_capability_id,
            issue,
        )?;
        let storage_binding_revision = binding.binding_revision();
        let runtime_generation =
            start_from_reservation(supervisor, store, runtime_dir, reservation, &binding)?;
        Ok((runtime_generation, storage_binding_revision))
    })()
    .map(|(runtime_generation, storage_binding_revision)| {
        OwnerResult::RestartSchedulerRuntime(RestartSchedulerRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            storage_binding_revision,
            launch_state: "accepted".to_owned(),
        })
    })
}

pub(super) fn upsert(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: UpsertSchedulerScheduleRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        require_scheduler_registration(store, &request.scheduler_registration_id)?;
        let schedule = request
            .schedule
            .ok_or_else(|| "Scheduler schedule is unavailable".to_owned())?;
        let expected_schedule_revision = schedule.schedule_revision;
        let control = SchedulerRuntimeControlRequestV1 {
            operation: Some(
                makosh_runtime_protocol::v1::scheduler_runtime_control_request_v1::Operation::UpsertSchedule(schedule.clone()),
            ),
        };
        validate_scheduler_runtime_control_request(&control)
            .map_err(|_| "Scheduler schedule is unavailable".to_owned())?;
        scheduler_admission::require_current_job_contract(store, &schedule)?;
        let response = SchedulerRuntimeControlResponseV1::decode(
            supervisor
                .relay(&request.scheduler_registration_id, control.encode_to_vec())?
                .as_slice(),
        )
        .map_err(|_| "Scheduler control response is unavailable".to_owned())?;
        validate_scheduler_runtime_control_response(&response)
            .map_err(|_| "Scheduler control response is unavailable".to_owned())?;
        scheduler_upsert_response(
            request.scheduler_registration_id,
            expected_schedule_revision,
            response.result,
        )
    })()
    .map(OwnerResult::UpsertSchedulerSchedule)
}

fn require_scheduler_registration(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<(), String> {
    let registration = store
        .module_registration(registration_id)
        .map_err(|_| "Scheduler registration is unavailable".to_owned())?
        .ok_or_else(|| "Scheduler registration is unavailable".to_owned())?;
    (registration.module_id() == "scheduler")
        .then_some(())
        .ok_or_else(|| "Scheduler control requires Scheduler registration".to_owned())
}

fn active_storage_binding(
    store: &SqliteControlStore,
    registration_id: &str,
    storage_capability_id: &str,
) -> Result<makosh_kernel_control_store::PlatformStorageBindingV1, String> {
    store
        .platform_storage_binding(registration_id, storage_capability_id)
        .map_err(|_| "Scheduler Storage binding is unavailable".to_owned())?
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .ok_or_else(|| "Scheduler Storage binding is unavailable".to_owned())
}

fn start_from_reservation(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    binding: &makosh_kernel_control_store::PlatformStorageBindingV1,
) -> Result<u64, String> {
    let kernel =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    scheduler_launch::start_from_reservation(
        supervisor,
        store,
        &kernel,
        runtime_dir,
        reservation,
        binding,
    )
}

fn scheduler_upsert_response(
    scheduler_registration_id: String,
    expected_schedule_revision: u64,
    result: Option<makosh_runtime_protocol::v1::scheduler_runtime_control_response_v1::Result>,
) -> Result<UpsertSchedulerScheduleResponseV1, String> {
    match result {
        Some(
            makosh_runtime_protocol::v1::scheduler_runtime_control_response_v1::Result::UpsertSchedule(schedule),
        ) if schedule.schedule_revision == expected_schedule_revision => Ok(
            UpsertSchedulerScheduleResponseV1 {
                scheduler_registration_id,
                schedule: Some(schedule),
            },
        ),
        Some(
            makosh_runtime_protocol::v1::scheduler_runtime_control_response_v1::Result::UpsertSchedule(_),
        ) => Err("Scheduler control response is unavailable".to_owned()),
        _ => Err("Scheduler schedule was not accepted".to_owned()),
    }
}
