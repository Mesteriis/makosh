#![forbid(unsafe_code)]

mod join;
mod lifecycle;
mod policy;

pub use join::{
    AttachmentPreviewCustodyDelegationIntentV1, AttachmentPreviewEvidenceJoinV1,
    AttachmentPreviewJoinErrorV1, AttachmentPreviewRequestFactV1, AttachmentPreviewSafetyFactV1,
    AttachmentPreviewSafetyStateV1, AttachmentPreviewScanCandidateFactV1,
};
pub use lifecycle::{
    AttachmentPreviewStatusV1, AttachmentPreviewTransitionErrorV1, AttachmentPreviewTransitionV1,
    accepted_attachment_preview_status_v1, transition_attachment_preview_status_v1,
    transition_attachment_preview_v1, validate_attachment_preview_status_v1,
};
pub use policy::{
    AttachmentPreviewOutputPolicyErrorV1, preview_output_limit_v1, validate_preview_output_v1,
};

pub const PACKAGE: &str = "makosh-attachment-preview-core";
