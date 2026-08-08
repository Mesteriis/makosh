#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-attachment-translation-api";
pub const ATTACHMENT_TRANSLATION_OWNER_V1: &str = "attachment_translation";
pub const ATTACHMENT_TRANSLATION_MODULE_ID_V1: &str = "makosh-attachment-translation-runtime";
pub const ATTACHMENT_TRANSLATION_CAPABILITY_ID_V1: &str = "attachment.translation.v1";
pub const ATTACHMENT_TRANSLATION_COMMAND_CONTRACT_NAME_V1: &str = "attachment.translation.command";
pub const ATTACHMENT_TRANSLATION_QUERY_CONTRACT_NAME_V1: &str = "attachment.translation.query";
pub const ATTACHMENT_TRANSLATION_TICKET_CONTRACT_NAME_V1: &str = "attachment.translation.ticket";
pub const ATTACHMENT_TRANSLATION_READ_CONTRACT_NAME_V1: &str = "attachment.translation.read";
pub const ATTACHMENT_TRANSLATION_REALTIME_CONTRACT_NAME_V1: &str =
    "attachment.translation.status_changed";
pub const ATTACHMENT_TRANSLATION_REALTIME_EVENT_KIND_V1: &str =
    "attachment.translation.status_changed";
pub const ATTACHMENT_TRANSLATION_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.attachment_translation.v1.AttachmentTranslationCommandService/Start";
pub const ATTACHMENT_TRANSLATION_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.attachment_translation.v1.AttachmentTranslationQueryService/Get";
pub const ATTACHMENT_TRANSLATION_TICKET_CONNECT_PATH_V1: &str =
    "/makosh.attachment_translation.v1.AttachmentTranslationTicketService/IssueRead";
pub const ATTACHMENT_TRANSLATION_READ_BLOB_PATH_V1: &str =
    "/api/blobs/attachment-translation/v1/result";
pub const ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1: u32 = 1;
pub const ATTACHMENT_TRANSLATION_CONTRACT_REVISION_V1: u32 = 1;
pub const ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1: u64 = 1024 * 1024;
pub const ATTACHMENT_TRANSLATION_MAX_RESULT_BYTES_V1: u64 = 64 * 1024;
pub const ATTACHMENT_TRANSLATION_READ_TICKET_BYTES_V1: usize = 32;
pub const ATTACHMENT_TRANSLATION_READ_TICKET_TTL_SECONDS_V1: i64 = 30;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.attachment_translation.v1.rs"
    ));
}

pub mod read_wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.attachment_translation.read.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/attachment_translation_control_schema.rs"
));
include!(concat!(
    env!("OUT_DIR"),
    "/attachment_translation_read_schema.rs"
));

pub const ATTACHMENT_TRANSLATION_CONTROL_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/attachment-translation-control-v1.bin"
));
pub const ATTACHMENT_TRANSLATION_READ_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/attachment-translation-read-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_exact_private_content_free_and_provider_neutral() {
        assert!(ATTACHMENT_TRANSLATION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_TRANSLATION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_TRANSLATION_TICKET_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_TRANSLATION_READ_BLOB_PATH_V1.starts_with("/api/blobs/"));
        let source = include_str!("../proto/makosh/attachment_translation/v1/translation.proto");
        assert!(source.contains("IssueAttachmentTranslationReadRequestV1"));
        assert!(source.contains("ATTACHMENT_TRANSLATION_LANGUAGE_SPANISH"));
        for forbidden in [
            "provider_id",
            "model_id",
            "endpoint",
            "prompt",
            "source_text",
            "translated_text_utf8",
            "filename",
            "content_type",
            "map<",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden client field {forbidden}"
            );
        }
    }
}
