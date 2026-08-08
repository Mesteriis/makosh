#![forbid(unsafe_code)]

mod admission;
mod blob_materialization;
mod client_port;
mod client_realtime;
mod contracts;
mod event_outbox;
mod managed_runtime;
mod promotion_result;
mod submission;

pub use admission::{
    REVIEW_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1, review_task_candidate_module_descriptor_v1,
    review_task_candidate_settings_schema_bytes_v1, review_task_candidate_settings_schema_v1,
};
pub use managed_runtime::{
    ReviewTaskCandidateManagedRuntimeErrorV1, ReviewTaskCandidateManagedRuntimeV1,
    ReviewTaskCandidateRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-review-task-candidate-runtime";
