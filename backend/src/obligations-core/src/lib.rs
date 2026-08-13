#![forbid(unsafe_code)]

mod creation;
mod lifecycle;
mod model;

pub use creation::{ObligationCreationErrorV1, create_obligation_from_reviewed_candidate_v1};
pub use lifecycle::{
    MAX_CONDITION_CHARS_V1, MAX_EVIDENCE_OWNER_ID_BYTES_V1, ObligationEvidenceLinkV1,
    ObligationLifecycleErrorV1, ObligationLifecycleStateV1, ObligationRecordV1,
    add_obligation_evidence_v1, remove_obligation_evidence_v1, set_obligation_state_v1,
    update_obligation_content_v1, validate_obligation_record_v1,
};
pub use model::{
    ObligationProvenanceV1, ObligationStatusV1, ObligationTimestampV1, ObligationV1,
    ObligationsValidationErrorV1, ReviewedCandidateObligationDraftV1, derive_obligation_id_v1,
    obligation_creation_fingerprint_v1, validate_obligation_v1,
};

pub const PACKAGE: &str = "makosh-obligations-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_STATEMENT_CHARS_V1: usize = 240;
