#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-cross-channel-forward-api";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1: &str =
    "communication_cross_channel_forward";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1: &str =
    "makosh-communication-cross-channel-forward-runtime";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_CAPABILITY_ID_V1: &str =
    "communication.cross_channel_forward.v1";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.cross_channel_forward.command";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONTRACT_NAME_V1: &str =
    "communication.cross_channel_forward.query";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.cross_channel_forward.status_changed";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_REALTIME_EVENT_KIND_V1: &str =
    "communication.cross_channel_forward.status_changed";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONNECT_PATH_V1: &str = "/makosh.communication_cross_channel_forward.v1.CommunicationCrossChannelForwardCommandService/Start";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONNECT_PATH_V1: &str = "/makosh.communication_cross_channel_forward.v1.CommunicationCrossChannelForwardQueryService/GetStatus";
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_REVISION_V1: u32 = 1;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_cross_channel_forward.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_cross_channel_forward_schema.rs"
));

pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-cross-channel-forward-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_contract_is_provider_neutral_and_contains_no_private_body() {
        assert!(COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source =
            include_str!("../proto/makosh/communication_cross_channel_forward/v1/forward.proto");
        assert!(!source.contains("provider_id"));
        assert!(!source.contains("account_id"));
        assert!(!source.contains("body_utf8"));
        assert!(!source.contains("blob_reference"));
        assert!(!source.contains("map<"));
    }
}
