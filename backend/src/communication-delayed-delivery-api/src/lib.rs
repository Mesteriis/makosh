#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-delayed-delivery-api";
pub const COMMUNICATION_DELAYED_DELIVERY_OWNER_V1: &str = "communication_delayed_delivery";
pub const COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1: &str =
    "makosh-communication-delayed-delivery-runtime";
pub const COMMUNICATION_DELAYED_DELIVERY_CAPABILITY_ID_V1: &str =
    "communication.delayed_delivery.v1";
pub const COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.delayed_delivery.schedule";
pub const COMMUNICATION_DELAYED_DELIVERY_CANCEL_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.delayed_delivery.cancel";
pub const COMMUNICATION_DELAYED_DELIVERY_QUERY_CONTRACT_NAME_V1: &str =
    "communication.delayed_delivery.query";
pub const COMMUNICATION_DELAYED_DELIVERY_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.delayed_delivery.status_changed";
pub const COMMUNICATION_DELAYED_DELIVERY_REALTIME_EVENT_KIND_V1: &str =
    "communication.delayed_delivery.status_changed";
pub const COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1: &str =
    "/makosh.communication_delayed_delivery.v1.CommunicationDelayedDeliveryCommandService/Schedule";
pub const COMMUNICATION_DELAYED_DELIVERY_CANCEL_CONNECT_PATH_V1: &str =
    "/makosh.communication_delayed_delivery.v1.CommunicationDelayedDeliveryCommandService/Cancel";
pub const COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1: &str =
    "/makosh.communication_delayed_delivery.v1.CommunicationDelayedDeliveryQueryService/GetStatus";
pub const COMMUNICATION_DELAYED_DELIVERY_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_DELAYED_DELIVERY_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_DELAYED_DELIVERY_MAX_BODY_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_DELAYED_DELIVERY_MAX_REQUEST_BYTES_V1: usize = 128 * 1024;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_delayed_delivery.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_delayed_delivery_schema.rs"
));

pub const COMMUNICATION_DELAYED_DELIVERY_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-delayed-delivery-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_contract_is_bounded_and_provider_neutral() {
        assert_eq!(COMMUNICATION_DELAYED_DELIVERY_MAX_BODY_BYTES_V1, 65_536);
        assert_eq!(COMMUNICATION_DELAYED_DELIVERY_MAX_REQUEST_BYTES_V1, 131_072);
        assert!(COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_DELAYED_DELIVERY_CANCEL_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1.starts_with('/'));
        assert_ne!(
            COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_COMMAND_CONTRACT_NAME_V1,
            COMMUNICATION_DELAYED_DELIVERY_CANCEL_COMMAND_CONTRACT_NAME_V1
        );
        let source =
            include_str!("../proto/makosh/communication_delayed_delivery/v1/delivery.proto");
        for forbidden in ["provider_id", "account_id", "blob", "scheduler", "map<"] {
            assert!(!source.contains(forbidden), "{forbidden} leaked into API");
        }
    }
}
