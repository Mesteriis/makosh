#![forbid(unsafe_code)]
//! Owner-local durable state for the call transcription workflow.

mod jobs;
mod model;
mod outbox;
mod realtime;
mod repository;
pub mod schema;
mod tickets;

pub use model::{
    CALL_TRANSCRIPTION_MAX_ATTEMPTS_V1, CALL_TRANSCRIPTION_MAX_LEASE_MILLIS_V1,
    CALL_TRANSCRIPTION_OUTBOX_LIMIT_V1, CALL_TRANSCRIPTION_REALTIME_LIMIT_V1,
    CALL_TRANSCRIPTION_RECOVERY_LIMIT_V1, CallTranscriptionInboxOutcomeV1,
    CallTranscriptionJobLeaseV1, CallTranscriptionPersistenceErrorV1,
    CallTranscriptionRealtimeTransitionV1, ClaimedCallTranscriptionJobV1, CompleteSourceCleanupV1,
    CreateCallTranscriptionRunOutcomeV1, CreateCallTranscriptionRunV1, DurableOutboxRecordV1,
    IssueCallTranscriptTicketV1, IssuedCallTranscriptTicketV1, MaterializeTranscriptV1,
    PersistRecordingIngressV1, PersistSttResultV1, PersistedCallTranscriptionRunV1,
    PersistedRecordingSourceV1, PersistedTranscriptBlobV1, RebindTranscriptMaterializationV1,
    RecordingIngressOutcomeV1, RedeemedCallTranscriptTicketV1, UnpublishedCallTranscriptionEventV1,
    call_transcription_job_id_v1,
};
pub use repository::CallTranscriptionPersistenceV1;
pub use schema::{
    CALL_TRANSCRIPTION_SCHEMA_V1, CALL_TRANSCRIPTION_STORAGE_BUNDLE_REVISION_V1,
    call_transcription_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-call-transcription-persistence";
