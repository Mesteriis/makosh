#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-delivery-intent-api";
pub const COMMUNICATION_DELIVERY_INTENT_OWNER_V1: &str = "communication_delivery_intent";
pub const COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1: &str =
    "makosh-communication-delivery-intent-runtime";
pub const COMMUNICATION_DELIVERY_INTENT_CAPABILITY_ID_V1: &str = "communication.delivery_intent.v1";
pub const COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.delivery_intent.command";
pub const COMMUNICATION_DELIVERY_INTENT_QUERY_CONTRACT_NAME_V1: &str =
    "communication.delivery_intent.query";
pub const COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.delivery_intent.status_changed";
pub const COMMUNICATION_DELIVERY_INTENT_REALTIME_EVENT_KIND_V1: &str =
    "delivery_intent_status_changed";
pub const COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.communication_delivery_intent.v1.CommunicationDeliveryIntentCommandService/Submit";
pub const COMMUNICATION_DELIVERY_INTENT_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communication_delivery_intent.v1.CommunicationDeliveryIntentQueryService/GetStatus";
pub const COMMUNICATION_DELIVERY_INTENT_MAX_BODY_BYTES_V1: usize = 64 * 1024;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_delivery_intent.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_delivery_intent_schema.rs"
));

pub const COMMUNICATION_DELIVERY_INTENT_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-delivery-intent-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_routes_and_bounds_are_exact() {
        assert!(COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_DELIVERY_INTENT_QUERY_CONNECT_PATH_V1.starts_with('/'));
        assert_ne!(
            COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1,
            COMMUNICATION_DELIVERY_INTENT_QUERY_CONNECT_PATH_V1
        );
        assert_eq!(COMMUNICATION_DELIVERY_INTENT_MAX_BODY_BYTES_V1, 65_536);
    }

    #[test]
    fn request_is_canonical_and_contains_no_provider_selector() {
        let request = wire::SubmitDeliveryIntentRequestV1 {
            protocol_major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
            operation_id: vec![1; 16],
            conversation_id: vec![2; 16],
            reply_to_message_id: None,
            body_utf8: b"hello".to_vec(),
        };
        assert_eq!(request.conversation_id.len(), 16);
        assert_eq!(request.body_utf8, b"hello");
    }
}
