#![forbid(unsafe_code)]

mod extraction;
mod lifecycle;
mod model;

pub use extraction::{
    CommunicationNoteExtractionErrorV1, extract_communication_note_candidates_v1,
};
pub use lifecycle::{
    CommunicationNoteCandidateTransitionErrorV1, CommunicationNoteCandidateTransitionV1,
    accepted_communication_note_candidate_status_v1, transition_communication_note_candidate_v1,
    validate_communication_note_candidate_status_v1,
};
pub use model::{
    CommunicationNoteCandidateCompletenessV1, CommunicationNoteCandidateDraftV1,
    CommunicationNoteCandidateRejectionCodeV1, CommunicationNoteCandidateStateV1,
    CommunicationNoteCandidateStatusV1, CommunicationNoteCandidateV1,
    CommunicationNoteCandidateValidationErrorV1, CommunicationNoteSourceBasisV1,
    CommunicationNoteSourceContentV1, CommunicationNoteTopicHintV1,
    validate_communication_note_candidate_draft_v1,
};

pub const PACKAGE: &str = "makosh-communication-note-candidate-core";
pub const COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1: usize = 256 * 1024;
pub const COMMUNICATION_NOTE_MAX_CANDIDATES_V1: usize = 1;
pub const COMMUNICATION_NOTE_MAX_TITLE_CHARS_V1: usize = 240;
pub const COMMUNICATION_NOTE_MAX_EXCERPT_CHARS_V1: usize = 2_000;
pub const COMMUNICATION_NOTE_MAX_CONFIDENCE_BASIS_POINTS_V1: u32 = 10_000;
