#![forbid(unsafe_code)]

mod repository;
mod schema;

pub use repository::{
    CallEvidenceConsumeOutcomeV1, CallEvidenceListFilterV1, CallEvidencePageV1,
    CallEvidencePersistenceErrorV1, CallEvidenceRealtimeRecordV1, CallEvidenceRejectionCodeV1,
    CommunicationsCallEvidencePersistenceV1,
};
pub use schema::{
    COMMUNICATIONS_CALL_EVIDENCE_SCHEMA_V1,
    COMMUNICATIONS_CALL_EVIDENCE_STORAGE_BUNDLE_REVISION_V1,
    CommunicationsCallEvidenceSchemaErrorV1, append_communications_call_evidence_storage_v1,
};

pub const PACKAGE: &str = "makosh-communications-call-evidence-persistence";
