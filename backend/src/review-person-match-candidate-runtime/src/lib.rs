#![forbid(unsafe_code)]

mod admission;
mod client;
mod consumer;
mod execution;
mod managed_runtime;

pub use admission::{
    REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
    review_person_match_candidate_module_descriptor_v1,
    review_person_match_candidate_settings_schema_bytes_v1,
    review_person_match_candidate_settings_schema_v1,
};
pub use client::dispatch_review_person_match_candidate_client_request_v1;
pub use consumer::{
    consume_person_match_candidate_decision_once_v1,
    consume_person_match_candidate_promotion_result_once_v1,
    consume_persons_review_candidate_once_v1,
};
pub use execution::{
    ReviewPersonMatchCandidateExecutionContextV1, ReviewPersonMatchCandidateExecutionErrorV1,
    process_person_match_candidate_decision_v1, process_persons_review_candidate_v1,
};
pub use managed_runtime::{
    ReviewPersonMatchCandidateManagedRuntimeErrorV1, ReviewPersonMatchCandidateManagedRuntimeV1,
    ReviewPersonMatchCandidateRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-review-person-match-candidate-runtime";
