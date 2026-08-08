#![forbid(unsafe_code)]

mod creation;
mod model;

pub use creation::{
    KnowledgeNoteCreationErrorV1, create_verified_knowledge_note_from_reviewed_candidate_v1,
};
pub use model::{
    KnowledgeNoteProvenanceV1, KnowledgeNoteSourceBasisV1, KnowledgeNoteTimestampV1,
    KnowledgeNoteTopicHintV1, KnowledgeValidationErrorV1, ReviewedCandidateKnowledgeNoteDraftV1,
    VerifiedKnowledgeNoteStatusV1, VerifiedKnowledgeNoteV1, derive_verified_knowledge_note_id_v1,
    knowledge_note_creation_fingerprint_v1, validate_verified_knowledge_note_v1,
};

pub const PACKAGE: &str = "makosh-knowledge-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_TITLE_CHARS_V1: usize = 240;
pub const MAX_EXCERPT_CHARS_V1: usize = 2_000;
pub const MAX_TOPIC_HINTS_V1: usize = 4;
