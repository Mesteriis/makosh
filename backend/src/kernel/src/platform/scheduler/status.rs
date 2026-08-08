//! Verifies the exact live Scheduler runtime before exposing it to clients.

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{
        GetSchedulerRuntimeStatusRequestV1, SchedulerRuntimeControlRequestV1,
        SchedulerRuntimeControlResponseV1, SchedulerRuntimeStateV1,
        scheduler_runtime_control_request_v1::Operation,
        scheduler_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::scheduler::validate_scheduler_runtime_control_response,
};
use prost::Message;

use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

const SCHEDULER_PROCESS_ID: &str = "scheduler_developer";

pub(crate) fn read_current(
    store: &SqliteControlStore,
    relay: &ManagedRuntimeRelayPort,
) -> Result<(), String> {
    let launch = store
        .effective_managed_launch_record(SCHEDULER_PROCESS_ID)
        .map_err(|_| "managed Scheduler launch is unavailable".to_owned())?
        .ok_or_else(|| "managed Scheduler launch is unavailable".to_owned())?;
    let request = SchedulerRuntimeControlRequestV1 {
        operation: Some(Operation::GetStatus(GetSchedulerRuntimeStatusRequestV1 {})),
    };
    let response = relay.relay(SCHEDULER_PROCESS_ID, request.encode_to_vec())?;
    let response = SchedulerRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed Scheduler status response is invalid".to_owned())?;
    validate_scheduler_runtime_control_response(&response)
        .map_err(|_| "managed Scheduler status response is invalid".to_owned())?;
    let status = match response.result {
        Some(ResponseResult::Status(status)) if response.error_code.is_empty() => status,
        _ => return Err("managed Scheduler status is unavailable".to_owned()),
    };
    (SchedulerRuntimeStateV1::try_from(status.state).ok() == Some(SchedulerRuntimeStateV1::Ready)
        && status.runtime_generation == launch.runtime_generation()
        && status.grant_epoch == launch.grant_epoch())
    .then_some(())
    .ok_or_else(|| "managed Scheduler status is stale or unavailable".to_owned())
}
