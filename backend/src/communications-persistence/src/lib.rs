//! Owner-local idempotency seam for canonical Communications evidence.

pub const PACKAGE: &str = "makosh-communications-persistence";

use makosh_communications_api::CommunicationObservationIdV1;

mod canonical_read;
mod content_read;
mod custody_transfer;
mod durable;
mod export_source;
mod forward_source;
mod saved_search;
mod schema;
mod search;
mod search_job;
mod sender_insights;
mod source_snapshot;
pub use canonical_read::{
    CanonicalReadAfterV1, CanonicalReadPageV1, CanonicalReferenceReadAfterV1,
    CanonicalReferenceReadItemV1,
};
pub use content_read::CommunicationsBodyContentReceiptV1;
pub use custody_transfer::{
    ClaimedCommunicationsBodyCustodyTransferV1, CommunicationsBodyCustodyTransferErrorV1,
};
pub use durable::{CommunicationsDurablePersistence, PersistedCommunicationsObservationV1};
pub use export_source::{
    CommunicationsEvidenceExportBodyReceiptV1, CommunicationsEvidenceExportSourceErrorV1,
    CommunicationsEvidenceExportSourceItemV1,
};
pub use forward_source::{
    CommunicationsCrossChannelForwardBodyReceiptV1, CommunicationsCrossChannelForwardSourceErrorV1,
    CommunicationsCrossChannelForwardSourceSnapshotV1,
};
pub use saved_search::{
    CommunicationsSavedSearchDefinitionV1, CommunicationsSavedSearchListAfterV1,
    CommunicationsSavedSearchMutationErrorV1, CommunicationsSavedSearchSummaryV1,
    CommunicationsSavedSearchWriteV1,
};
pub use schema::{
    COMMUNICATIONS_BODY_MEDIA_TYPE_STORAGE_BUNDLE_REVISION_V1, COMMUNICATIONS_SCHEMA_V1,
    COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1, CommunicationsBodyMediaTypeSchemaErrorV1,
    append_communications_body_media_type_storage_v1, communications_storage_bundle_v1,
};
pub use search::{
    CommunicationsSearchProjectionWriteErrorV1, CommunicationsSearchProjectionWriteV1,
};
pub use search_job::{
    ClaimedCommunicationsDerivedIndexJobV1, CommunicationsDerivedIndexFailureRecordV1,
    CommunicationsDerivedIndexFailureV1, CommunicationsDerivedIndexJobErrorV1,
    CommunicationsDerivedIndexJobOperationV1, CommunicationsDerivedIndexJobV1,
    communications_derived_index_job_id_v1,
};
pub use sender_insights::{
    CommunicationsSenderInsightAfterV1, CommunicationsSenderInsightV1,
    CommunicationsSenderInsightsErrorV1,
};
pub use source_snapshot::{
    CommunicationsBodyReceiptV1, CommunicationsSourceErrorV1, CommunicationsSourceSnapshotV1,
};

/// Private Communications-owned work item for an admitted producer body. It
/// never becomes a canonical Blob reference or public query field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingCommunicationsBodyCustodyTransferV1 {
    pub evidence_id: CommunicationObservationIdV1,
    pub envelope_sha256: [u8; 32],
    pub source_blob_ref: String,
    pub source_reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub plaintext_sha256: [u8; 32],
    pub source_custody_proof: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsConsumeOutcomeV1 {
    Applied,
    Duplicate,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsPersistenceError {
    DuplicateOperation,
    InboxHashConflict,
    InvalidDerivedIndexJob,
    InvalidCustodyTransfer,
    InvalidAttachmentAnchorOutbox,
    MissingCanonicalMessage,
    StorageUnavailable,
    InvalidRow,
}
