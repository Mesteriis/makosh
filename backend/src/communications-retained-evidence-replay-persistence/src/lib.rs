#![forbid(unsafe_code)]

mod delivery;
mod repository;
mod schema;

pub use delivery::{
    CommunicationsReplayCommandAdmissionV1, CommunicationsReplayCommandInboxOutcomeV1,
    CommunicationsReplayResultStoreOutcomeV1,
};
pub use repository::{
    CommunicationsRetainedEvidenceReplayPersistenceV1, RetainedCommunicationsEvidenceV1,
    RetainedCommunicationsReplayAuditV1, RetainedCommunicationsReplayErrorV1,
    RetainedCommunicationsReplayPhaseV1,
};
pub use schema::{
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_SCHEMA_V1,
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1,
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_SCHEMA_V1,
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCAN_STORAGE_BUNDLE_REVISION_V1,
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_SCHEMA_V1,
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
    CommunicationsRetainedEvidenceReplayDeliverySchemaErrorV1,
    CommunicationsRetainedEvidenceReplayScanSchemaErrorV1,
    CommunicationsRetainedEvidenceReplaySchemaErrorV1,
    append_communications_retained_evidence_replay_delivery_storage_v1,
    append_communications_retained_evidence_replay_scan_storage_v1,
    append_communications_retained_evidence_replay_storage_v1,
};

pub const PACKAGE: &str = "makosh-communications-retained-evidence-replay-persistence";
