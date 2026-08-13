#![forbid(unsafe_code)]

mod admission;
mod approval;
mod event_outbox;
mod managed_runtime;
mod obligation_results;
mod validation;

pub use admission::{
    REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
    reviewed_obligation_candidate_promotion_module_descriptor_v1,
    reviewed_obligation_candidate_promotion_settings_schema_bytes_v1,
    reviewed_obligation_candidate_promotion_settings_schema_v1,
};
pub use managed_runtime::{
    ReviewedObligationCandidatePromotionManagedRuntimeErrorV1,
    ReviewedObligationCandidatePromotionManagedRuntimeV1,
    ReviewedObligationCandidatePromotionRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-reviewed-obligation-candidate-promotion-runtime";
