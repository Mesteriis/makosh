#![forbid(unsafe_code)]

mod creation;
mod model;

pub use creation::{TaskCreationErrorV1, create_task_from_reviewed_candidate_v1};
pub use model::{
    ReviewedCandidateTaskDraftV1, TaskProvenanceV1, TaskStatusV1, TaskTimestampV1, TaskV1,
    TasksValidationErrorV1, derive_task_id_v1, task_creation_fingerprint_v1, validate_task_v1,
};

pub const PACKAGE: &str = "makosh-tasks-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_TITLE_CHARS_V1: usize = 240;
pub const MAX_HINT_CHARS_V1: usize = 120;
