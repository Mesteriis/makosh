#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-attachment-archive-inspection-api";
pub const ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1: &str = "attachment_archive_inspection";
pub const ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1: &str =
    "makosh-attachment-archive-inspection-runtime";
pub const ATTACHMENT_ARCHIVE_INSPECTION_CAPABILITY_ID_V1: &str = "attachment.archive_inspection.v1";
pub const ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONTRACT_NAME_V1: &str =
    "attachment.archive_inspection.command";
pub const ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONTRACT_NAME_V1: &str =
    "attachment.archive_inspection.query";
pub const ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_CONTRACT_NAME_V1: &str =
    "attachment.archive_inspection.status_changed";
pub const ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_EVENT_KIND_V1: &str =
    "attachment.archive_inspection.status_changed";
pub const ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.attachment_archive_inspection.v1.AttachmentArchiveInspectionCommandService/Start";
pub const ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.attachment_archive_inspection.v1.AttachmentArchiveInspectionQueryService/Get";
pub const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_REVISION_V1: u32 = 1;
pub const ATTACHMENT_ARCHIVE_INSPECTION_MAX_REPORT_ENTRIES_V1: usize = 1_000;
pub const ATTACHMENT_ARCHIVE_INSPECTION_MAX_PATH_BYTES_V1: usize = 1_024;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.attachment_archive_inspection.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/attachment_archive_inspection_schema.rs"
));

pub const ATTACHMENT_ARCHIVE_INSPECTION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/attachment-archive-inspection-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_concrete_bounded_and_has_no_blob_or_provider_authority() {
        assert!(ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source = include_str!(
            "../proto/makosh/attachment_archive_inspection/v1/archive_inspection.proto"
        );
        assert!(source.contains("ArchiveInspectionReportV1"));
        assert!(source.contains("normalized_path_utf8"));
        assert!(!source.contains("blob_reference"));
        assert!(!source.contains("provider"));
        assert!(!source.contains("account_id"));
        assert!(!source.contains("filesystem"));
        assert!(!source.contains("source_bytes"));
        assert!(!source.contains("map<"));
    }
}
