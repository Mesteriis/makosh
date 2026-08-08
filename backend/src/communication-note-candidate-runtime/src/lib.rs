#![forbid(unsafe_code)]

mod admission;
mod blob_materialization;
mod client_port;
mod client_realtime;
mod contracts;
mod event_outbox;
mod extraction;
mod managed_runtime;
mod review_submission;
mod source_results;

pub use admission::{
    COMMUNICATION_NOTE_CANDIDATE_BLOB_CAPABILITY_ID_V1,
    COMMUNICATION_NOTE_CANDIDATE_REVIEW_SUBMISSION_CAPABILITY_ID_V1,
    COMMUNICATION_NOTE_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
    communication_note_candidate_module_descriptor_v1,
    communication_note_candidate_settings_schema_bytes_v1,
    communication_note_candidate_settings_schema_v1,
};
pub use blob_materialization::{
    CommunicationNoteCandidateBlobErrorV1, CommunicationNoteCandidateSourceBlobReceiptV1,
};
pub use client_port::{
    get_communication_note_candidate_payload_v1, start_communication_note_candidate_payload_v1,
};
pub use event_outbox::{CommunicationNoteCandidateEventRelayErrorV1, relay_outbox_once_v1};
pub use managed_runtime::{
    CommunicationNoteCandidateManagedRuntimeErrorV1, CommunicationNoteCandidateManagedRuntimeV1,
    CommunicationNoteCandidateRuntimeAdmissionV1,
};
pub use source_results::{
    CommunicationNoteCandidateSourceResultErrorV1, consume_note_source_rejected_once_v1,
};

pub const PACKAGE: &str = "makosh-communication-note-candidate-runtime";
