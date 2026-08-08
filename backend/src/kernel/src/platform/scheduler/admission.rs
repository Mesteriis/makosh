//! Current approved JobKind admission for Scheduler-owned schedule mutations.

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::UpsertSchedulerScheduleRequestV1;

use super::scheduler_catalog;

pub(crate) fn require_current_job_contract(
    store: &SqliteControlStore,
    schedule: &UpsertSchedulerScheduleRequestV1,
) -> Result<(), String> {
    let contract_name = format!("{}.{}", schedule.job_owner, schedule.job_name);
    if schedule.contract_name != contract_name {
        return Err("Scheduler schedule contract is unavailable".to_owned());
    }
    let expected_schema: [u8; 32] = schedule
        .contract_schema_sha256
        .as_slice()
        .try_into()
        .map_err(|_| "Scheduler schedule contract is unavailable".to_owned())?;
    scheduler_catalog::resolve(store)?
        .iter()
        .any(|entry| {
            let request = entry.request();
            request.owner() == schedule.job_owner
                && request.name() == schedule.job_name
                && request.major() == schedule.job_major
                && request.revision() == schedule.contract_revision
                && request.schema_sha256() == &expected_schema
        })
        .then_some(())
        .ok_or_else(|| "Scheduler schedule contract is unavailable".to_owned())
}
