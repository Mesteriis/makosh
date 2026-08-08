#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communications-export-api";
pub const COMMUNICATIONS_EXPORT_OWNER_V1: &str = "communications_export";
pub const COMMUNICATIONS_EXPORT_MODULE_ID_V1: &str = "makosh-communications-export-runtime";
pub const COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1: &str = "communications.export.v1";
pub const COMMUNICATIONS_EXPORT_COMMAND_CONTRACT_NAME_V1: &str = "communications.export.command";
pub const COMMUNICATIONS_EXPORT_QUERY_CONTRACT_NAME_V1: &str = "communications.export.query";
pub const COMMUNICATIONS_EXPORT_TICKET_CONTRACT_NAME_V1: &str = "communications.export.ticket";
pub const COMMUNICATIONS_EXPORT_READ_CONTRACT_NAME_V1: &str = "communications.export.read";
pub const COMMUNICATIONS_EXPORT_REALTIME_CONTRACT_NAME_V1: &str =
    "communications.export.status_changed";
pub const COMMUNICATIONS_EXPORT_REALTIME_EVENT_KIND_V1: &str =
    "communications.export.status_changed";
pub const COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATIONS_EXPORT_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATIONS_EXPORT_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.communications_export.v1.CommunicationsExportCommandService/Start";
pub const COMMUNICATIONS_EXPORT_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communications_export.v1.CommunicationsExportQueryService/GetStatus";
pub const COMMUNICATIONS_EXPORT_TICKET_CONNECT_PATH_V1: &str =
    "/makosh.communications_export.v1.CommunicationsExportTicketService/IssueRead";
pub const COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1: &str =
    "/api/blobs/communications-export/v1/artifact";
pub const COMMUNICATIONS_EXPORT_MAX_MESSAGES_V1: usize = 64;
pub const COMMUNICATIONS_EXPORT_MAX_SOURCE_BYTES_V1: u64 = 16 * 1024 * 1024;
pub const COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1: u64 = 24 * 1024 * 1024;
pub const COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1: usize = 32;
pub const COMMUNICATIONS_EXPORT_READ_TICKET_TTL_SECONDS_V1: i64 = 30;
const _: () = assert!(
    COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1 > COMMUNICATIONS_EXPORT_MAX_SOURCE_BYTES_V1
);

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications_export.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/communications_export_schema.rs"));

pub const COMMUNICATIONS_EXPORT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/communications-export-v1.bin"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_routes_are_exact_and_do_not_expose_internal_blob_identity() {
        assert!(COMMUNICATIONS_EXPORT_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATIONS_EXPORT_QUERY_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATIONS_EXPORT_TICKET_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1.starts_with("/api/blobs/"));
        assert_ne!(
            COMMUNICATIONS_EXPORT_COMMAND_CONNECT_PATH_V1,
            COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1
        );
    }

    #[test]
    fn bounds_are_explicit() {
        assert_eq!(COMMUNICATIONS_EXPORT_MAX_MESSAGES_V1, 64);
        assert_eq!(COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1, 32);
        assert_eq!(COMMUNICATIONS_EXPORT_READ_TICKET_TTL_SECONDS_V1, 30);
    }
}
