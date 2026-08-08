#![forbid(unsafe_code)]

mod admission;
mod approval;
mod event_outbox;
mod managed_runtime;
mod task_results;
mod validation;

pub use admission::{
    REVIEWED_TASK_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
    reviewed_task_candidate_promotion_module_descriptor_v1,
    reviewed_task_candidate_promotion_settings_schema_bytes_v1,
    reviewed_task_candidate_promotion_settings_schema_v1,
};
pub use managed_runtime::{
    ReviewedTaskCandidatePromotionManagedRuntimeErrorV1,
    ReviewedTaskCandidatePromotionManagedRuntimeV1,
    ReviewedTaskCandidatePromotionRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-reviewed-task-candidate-promotion-runtime";
