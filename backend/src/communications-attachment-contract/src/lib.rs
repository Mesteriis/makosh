//! Public typed attachment contracts owned by Communications.

pub mod admission;
mod observation;

pub use observation::{
    AttachmentBlobAdmissionFactV1, AttachmentBlobAdmissionTransitionV1,
    AttachmentBlobExpectedStateV1, AttachmentObservationEnvelopeBuildErrorV1,
    AttachmentObservationEnvelopeContextV1, AttachmentSafetyExpectedStateV1,
    AttachmentSafetyVerdictFactV1, AttachmentSafetyVerdictOutboxRecordV1,
    AttachmentSafetyVerdictV1, build_attachment_blob_admission_outbox_record_v1,
    build_attachment_safety_verdict_outbox_record_v1,
};

pub mod blob_admission_v1 {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.ingress.attachment.blob.v1.rs"
    ));
}

pub mod safety_verdict_v1 {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.ingress.attachment.safety.v1.rs"
    ));
}

pub mod anchor_recorded_v1 {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.ingress.attachment.anchor.v1.rs"
    ));
}

pub mod lifecycle_v1 {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.attachment.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communications_attachment_blob_admission_observation_schema.rs"
));
include!(concat!(
    env!("OUT_DIR"),
    "/communications_attachment_safety_verdict_observation_schema.rs"
));
include!(concat!(
    env!("OUT_DIR"),
    "/communications_attachment_anchor_recorded_schema.rs"
));
include!(concat!(
    env!("OUT_DIR"),
    "/communications_attachment_lifecycle_schema.rs"
));

pub const PACKAGE: &str = "makosh-communications-attachment-contract";
