//! Public typed input contract owned by the Attachment Security engine.

pub mod admission;
mod candidate;

pub use candidate::{
    ATTACHMENT_SECURITY_MAX_SCAN_CANDIDATE_BYTES_V1, AttachmentSecurityObservationBuildErrorV1,
    AttachmentSecurityObservationContextV1, AttachmentSecurityScanCandidateFactV1,
    build_attachment_security_scan_candidate_outbox_record_v1,
};

pub mod v1 {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.attachment_security.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/attachment_security_scan_candidate_schema.rs"
));

pub const PACKAGE: &str = "makosh-attachment-security-contract";
