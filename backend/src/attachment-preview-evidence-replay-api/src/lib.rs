#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-attachment-preview-evidence-replay-api";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1: &str = "attachment_preview_evidence_replay";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_ID_V1: &str =
    "makosh-attachment-preview-evidence-replay-runtime";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CAPABILITY_ID_V1: &str =
    "attachment-preview-evidence-replay.command.v1";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONTRACT_NAME_V1: &str =
    "attachment_preview_evidence_replay.command";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONTRACT_MAJOR_V1: u32 = 1;
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONTRACT_REVISION_V1: u32 = 2;
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONNECT_PATH_V1: &str = "/makosh.attachment_preview_evidence_replay.v1.AttachmentPreviewEvidenceReplayCommandService/Start";
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MAX_MESSAGES_PER_PRODUCER_V1: usize = 16;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.attachment_preview_evidence_replay.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/attachment_preview_evidence_replay_schema.rs"
));

pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/attachment-preview-evidence-replay-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_surface_is_one_exact_bounded_command() {
        assert!(ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONNECT_PATH_V1.starts_with('/'));
        assert_eq!(
            ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MAX_MESSAGES_PER_PRODUCER_V1,
            16
        );
        let source =
            include_str!("../proto/makosh/attachment_preview_evidence_replay/v1/replay.proto");
        assert!(!source.contains("subject"));
        assert!(!source.contains("predicate"));
        assert!(!source.contains("payload_bytes"));
        assert!(!source.contains("logical_owner_id"));
        assert!(!source.contains("owner_device_actor"));
        assert!(!source.contains("producer_registration_id"));
        assert!(!source.contains("original_message_ids"));
    }
}
