#![forbid(unsafe_code)]

mod content;
mod join;
mod lifecycle;

pub use content::{
    AttachmentTextContentErrorV1, NormalizedAttachmentTextV1, normalize_attachment_text_v1,
    visible_attachment_text_v1,
};
pub use join::{
    AttachmentTextCanonicalSafetyFactV1, AttachmentTextCustodyDelegationIntentV1,
    AttachmentTextExtractionJoinDecisionV1, AttachmentTextExtractionRecordDecisionV1,
    AttachmentTextExtractionRejectionV1, AttachmentTextExtractionRequestV1,
    AttachmentTextSafetyStateV1, AttachmentTextScanCandidateV1,
    attachment_text_rejection_evidence_id_v1, decide_attachment_text_join_v1,
    decide_attachment_text_safety_record_v1, decide_attachment_text_scan_candidate_record_v1,
    validate_attachment_text_request_v1,
};
pub use lifecycle::{
    AttachmentTextExtractionErrorV1, AttachmentTextExtractionStateV1,
    AttachmentTextExtractionStatusV1, AttachmentTextExtractionTransitionErrorV1,
    AttachmentTextExtractionTransitionV1, AttachmentTextFormatV1,
    accepted_attachment_text_status_v1, transition_attachment_text_status_v1,
    validate_attachment_text_status_v1,
};

pub const PACKAGE: &str = "makosh-attachment-text-extraction-core";
pub const ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1: u64 = 100 * 1024 * 1024;
