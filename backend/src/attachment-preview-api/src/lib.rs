#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-attachment-preview-api";
pub const ATTACHMENT_PREVIEW_OWNER_V1: &str = "attachment_preview";
pub const ATTACHMENT_PREVIEW_MODULE_ID_V1: &str = "makosh-attachment-preview-runtime";
pub const ATTACHMENT_PREVIEW_COMMAND_CONTRACT_NAME_V1: &str = "attachment_preview.command";
pub const ATTACHMENT_PREVIEW_QUERY_CONTRACT_NAME_V1: &str = "attachment_preview.query";
pub const ATTACHMENT_PREVIEW_TICKET_CONTRACT_NAME_V1: &str = "attachment_preview.ticket";
pub const ATTACHMENT_PREVIEW_REALTIME_CONTRACT_NAME_V1: &str = "attachment_preview.realtime";
pub const ATTACHMENT_PREVIEW_READ_CONTRACT_NAME_V1: &str = "attachment_preview.read";
pub const ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1: u32 = 1;
pub const ATTACHMENT_PREVIEW_CONTRACT_REVISION_V1: u32 = 1;
pub const ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.attachment_preview.v1.AttachmentPreviewCommandService/Start";
pub const ATTACHMENT_PREVIEW_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.attachment_preview.v1.AttachmentPreviewQueryService/Get";
pub const ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1: &str =
    "/makosh.attachment_preview.v1.AttachmentPreviewTicketService/IssueRead";
pub const ATTACHMENT_PREVIEW_READ_BLOB_PATH_V1: &str = "/api/blobs/attachment-preview/v1/artifact";
pub const ATTACHMENT_PREVIEW_REALTIME_EVENT_KIND_V1: &str = "attachment_preview.status_changed.v1";
pub const ATTACHMENT_PREVIEW_READ_TICKET_BYTES_V1: usize = 32;
pub const ATTACHMENT_PREVIEW_READ_TICKET_TTL_SECONDS_V1: i64 = 30;
pub const ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1: u64 = 64 * 1024;
pub const ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1: u64 = 5 * 1024 * 1024;
pub const ATTACHMENT_PREVIEW_MAX_AUDIO_BYTES_V1: u64 = 24 * 1024 * 1024;
pub const ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1: u64 = 32 * 1024 * 1024;
const _: () = {
    assert!(ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1 > ATTACHMENT_PREVIEW_MAX_AUDIO_BYTES_V1);
    assert!(ATTACHMENT_PREVIEW_MAX_AUDIO_BYTES_V1 > ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1);
    assert!(ATTACHMENT_PREVIEW_MAX_IMAGE_BYTES_V1 > ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1);
};

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.attachment_preview.v1.rs"));
}

pub mod read_wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.attachment_preview.read.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/attachment_preview_control_schema.rs"
));
include!(concat!(
    env!("OUT_DIR"),
    "/attachment_preview_read_schema.rs"
));

pub const ATTACHMENT_PREVIEW_CONTROL_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/attachment-preview-control-v1.bin"
));
pub const ATTACHMENT_PREVIEW_READ_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/attachment-preview-read-v1.bin"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_and_bounds_are_exact() {
        assert!(ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_PREVIEW_QUERY_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_PREVIEW_READ_BLOB_PATH_V1.starts_with("/api/blobs/"));
        assert_eq!(ATTACHMENT_PREVIEW_READ_TICKET_BYTES_V1, 32);
        assert_eq!(ATTACHMENT_PREVIEW_READ_TICKET_TTL_SECONDS_V1, 30);
    }
}
