#![forbid(unsafe_code)]

mod admission;
mod approval;
mod blob_handoff;
mod event_outbox;
mod managed_runtime;
mod note_results;
mod validation;

pub use admission::{
    REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_CAPABILITY_ID_V1,
    REVIEWED_NOTE_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
    reviewed_note_candidate_promotion_module_descriptor_v1,
    reviewed_note_candidate_promotion_settings_schema_bytes_v1,
    reviewed_note_candidate_promotion_settings_schema_v1,
};
pub use managed_runtime::{
    ReviewedNoteCandidatePromotionManagedRuntimeErrorV1,
    ReviewedNoteCandidatePromotionManagedRuntimeV1,
    ReviewedNoteCandidatePromotionRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-reviewed-note-candidate-promotion-runtime";
