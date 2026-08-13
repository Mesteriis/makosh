#![forbid(unsafe_code)]

mod admission;
mod approval;
mod consumer;
mod managed_runtime;

pub use admission::{
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
    reviewed_person_match_candidate_promotion_module_descriptor_v1,
    reviewed_person_match_candidate_promotion_settings_schema_bytes_v1,
    reviewed_person_match_candidate_promotion_settings_schema_v1,
};
pub use approval::{
    ReviewedPersonMatchCandidatePromotionExecutionContextV1,
    ReviewedPersonMatchCandidatePromotionExecutionErrorV1, build_persons_command_outbox_record_v1,
    process_person_match_candidate_approval_v1, process_persons_terminal_v1,
};
pub use consumer::{
    consume_person_match_candidate_approval_once_v1, consume_persons_rejected_terminal_once_v1,
    consume_persons_succeeded_terminal_once_v1,
};
pub use managed_runtime::{
    ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1,
    ReviewedPersonMatchCandidatePromotionManagedRuntimeV1,
    ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-reviewed-person-match-candidate-promotion-runtime";
