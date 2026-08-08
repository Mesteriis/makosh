#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-bulk-action-api";
pub const COMMUNICATION_BULK_ACTION_OWNER_V1: &str = "communication_bulk_action";
pub const COMMUNICATION_BULK_ACTION_MODULE_ID_V1: &str = "makosh-communication-bulk-action-runtime";
pub const COMMUNICATION_BULK_ACTION_CAPABILITY_ID_V1: &str = "communication.bulk_action.v1";
pub const COMMUNICATION_BULK_ACTION_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.bulk_action.command";
pub const COMMUNICATION_BULK_ACTION_QUERY_CONTRACT_NAME_V1: &str =
    "communication.bulk_action.query";
pub const COMMUNICATION_BULK_ACTION_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.bulk_action.status_changed";
pub const COMMUNICATION_BULK_ACTION_REALTIME_EVENT_KIND_V1: &str =
    "communication.bulk_action.status_changed";
pub const COMMUNICATION_BULK_ACTION_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.communication_bulk_action.v1.CommunicationBulkDeliveryCommandService/Start";
pub const COMMUNICATION_BULK_ACTION_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communication_bulk_action.v1.CommunicationBulkDeliveryQueryService/GetStatus";
pub const COMMUNICATION_BULK_ACTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_BULK_ACTION_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_BULK_ACTION_MAX_TARGETS_V1: usize = 100;
pub const COMMUNICATION_BULK_ACTION_MAX_STATUS_PAGE_V1: u32 = 100;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_bulk_action.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_bulk_action_schema.rs"
));

pub const COMMUNICATION_BULK_ACTION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-bulk-action-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_contract_is_bounded_and_provider_neutral() {
        assert_eq!(COMMUNICATION_BULK_ACTION_MAX_TARGETS_V1, 100);
        assert_eq!(COMMUNICATION_BULK_ACTION_MAX_STATUS_PAGE_V1, 100);
        assert!(COMMUNICATION_BULK_ACTION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_BULK_ACTION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source = include_str!("../proto/makosh/communication_bulk_action/v1/bulk_action.proto");
        assert!(!source.contains("provider_id"));
        assert!(!source.contains("account_id"));
        assert!(!source.contains("map<"));
    }
}
