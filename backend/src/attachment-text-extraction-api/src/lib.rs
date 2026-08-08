#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-attachment-text-extraction-api";
pub const ATTACHMENT_TEXT_EXTRACTION_OWNER_V1: &str = "attachment_text_extraction";
pub const ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1: &str =
    "makosh-attachment-text-extraction-runtime";
pub const ATTACHMENT_TEXT_EXTRACTION_CAPABILITY_ID_V1: &str = "attachment.text_extraction.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONTRACT_NAME_V1: &str =
    "attachment.text_extraction.command";
pub const ATTACHMENT_TEXT_EXTRACTION_QUERY_CONTRACT_NAME_V1: &str =
    "attachment.text_extraction.query";
pub const ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONTRACT_NAME_V1: &str =
    "attachment.text_extraction.content";
pub const ATTACHMENT_TEXT_EXTRACTION_REALTIME_CONTRACT_NAME_V1: &str =
    "attachment.text_extraction.status_changed";
pub const ATTACHMENT_TEXT_EXTRACTION_REALTIME_EVENT_KIND_V1: &str =
    "attachment.text_extraction.status_changed";
pub const ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.attachment_text_extraction.v1.AttachmentTextExtractionCommandService/Start";
pub const ATTACHMENT_TEXT_EXTRACTION_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.attachment_text_extraction.v1.AttachmentTextExtractionQueryService/Get";
pub const ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1: &str =
    "/makosh.attachment_text_extraction.v1.AttachmentTextExtractionContentService/ReadText";
pub const ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const ATTACHMENT_TEXT_EXTRACTION_CONTRACT_REVISION_V1: u32 = 1;
pub const ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1: usize = 1024 * 1024;
pub const ATTACHMENT_TEXT_EXTRACTION_MAX_VISIBLE_BYTES_V1: usize = 64 * 1024;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.attachment_text_extraction.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/attachment_text_extraction_schema.rs"
));

pub const ATTACHMENT_TEXT_EXTRACTION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/attachment-text-extraction-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_separates_status_from_private_content() {
        assert!(ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_TEXT_EXTRACTION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONNECT_PATH_V1.starts_with('/'));
        let source =
            include_str!("../proto/makosh/attachment_text_extraction/v1/text_extraction.proto");
        assert!(source.contains("rpc ReadText"));
        let status = source
            .split("message GetAttachmentTextExtractionResponseV1")
            .nth(1)
            .and_then(|value| value.split('}').next())
            .expect("status message");
        let realtime = source
            .split("message AttachmentTextExtractionStatusChangedV1")
            .nth(1)
            .and_then(|value| value.split('}').next())
            .expect("realtime message");
        assert!(!status.contains("text_utf8"));
        assert!(!realtime.contains("text_utf8"));
        for forbidden in [
            "blob_reference",
            "provider",
            "account_id",
            "filename",
            "content_type",
            "filesystem",
            "source_path",
            "map<",
        ] {
            assert!(!source.contains(forbidden), "forbidden field: {forbidden}");
        }
    }

    #[test]
    fn content_bounds_are_explicit_and_distinct() {
        let derived_limit = ATTACHMENT_TEXT_EXTRACTION_MAX_DERIVED_BYTES_V1;
        let visible_limit = ATTACHMENT_TEXT_EXTRACTION_MAX_VISIBLE_BYTES_V1;
        assert_eq!(derived_limit, 1_048_576);
        assert_eq!(visible_limit, 65_536);
        assert!(visible_limit < derived_limit);
    }
}
