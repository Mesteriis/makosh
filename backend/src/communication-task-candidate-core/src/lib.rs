#![forbid(unsafe_code)]

mod extraction;
mod lifecycle;
mod model;

pub use extraction::{
    CommunicationTaskExtractionErrorV1, extract_communication_task_candidates_v1,
};
pub use lifecycle::{
    CommunicationTaskCandidateTransitionErrorV1, CommunicationTaskCandidateTransitionV1,
    accepted_communication_task_candidate_status_v1, transition_communication_task_candidate_v1,
    validate_communication_task_candidate_status_v1,
};
pub use model::{
    CommunicationTaskCandidateCompletenessV1, CommunicationTaskCandidateDraftV1,
    CommunicationTaskCandidateRejectionCodeV1, CommunicationTaskCandidateStateV1,
    CommunicationTaskCandidateStatusV1, CommunicationTaskCandidateV1,
    CommunicationTaskCandidateValidationErrorV1, CommunicationTaskSignalKindV1,
    CommunicationTaskSourceBasisV1, CommunicationTaskSourceContentV1,
    validate_communication_task_candidate_draft_v1,
};

pub const PACKAGE: &str = "makosh-communication-task-candidate-core";
pub const COMMUNICATION_TASK_SOURCE_MAX_BYTES_V1: usize = 256 * 1024;
pub const COMMUNICATION_TASK_MAX_CANDIDATES_V1: usize = 16;
pub const COMMUNICATION_TASK_MAX_TITLE_CHARS_V1: usize = 240;
pub const COMMUNICATION_TASK_MAX_HINT_CHARS_V1: usize = 120;
pub const COMMUNICATION_TASK_MAX_CONFIDENCE_BASIS_POINTS_V1: u32 = 10_000;
