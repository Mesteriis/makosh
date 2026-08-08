//! Versioned, owner-neutral Scheduler contracts.

mod contracts;
mod transport;
pub mod validation;

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/makosh.scheduler.v1.rs"));
}

pub const SCHEDULER_JOB_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/makosh.scheduler.v1.bin"));
pub const SCHEDULER_RUNTIME_MODULE_ID_V1: &str = "makosh-scheduler-runtime";

pub use contracts::command::{ScheduledJobCommandBuildErrorV1, build_scheduled_job_command_v1};
pub use contracts::job::{JobContractBindingV1, JobKindErrorV1, JobKindV1};
pub use contracts::owner_command::{
    OpaqueOwnerJobScopeV1, OwnerJobCommandBuildErrorV1, OwnerJobLeaseV1, build_owner_job_command_v1,
};
pub use contracts::run::{JobRunErrorV1, JobRunIdV1, ScheduleRunLeaseV1};
pub use contracts::schedule::{
    ConcurrencyKeyV1, MisfirePolicyV1, OpaqueScheduleScopeV1, OverlapPolicyV1, RetryPolicyV1,
    ScheduleCodecErrorV1, ScheduleErrorV1, ScheduleIdV1, SchedulePolicyV1, ScheduleRevisionV1,
    ScheduleSpecV1, ScheduleTriggerV1,
};
pub use transport::{
    SchedulerReceiptDeliveryErrorV1, SchedulerReceiptDeliveryPortV1, SchedulerReceiptDeliveryV1,
    SchedulerScheduleControlDeliveryErrorV1, SchedulerScheduleControlDeliveryPortV1,
    SchedulerScheduleControlDeliveryV1,
};
pub use validation::{
    OwnerJobCommandValidationErrorV1, SchedulerCommandValidationErrorV1,
    SchedulerReceiptValidationErrorV1, SchedulerScheduleControlValidationErrorV1,
    validate_job_run_receipt_v1, validate_owner_job_command_v1, validate_scheduled_job_command_v1,
    validate_scheduler_schedule_control_command_v1, validate_scheduler_schedule_control_result_v1,
};
