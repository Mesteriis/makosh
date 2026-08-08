#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-explanation-api";
pub const COMMUNICATION_EXPLANATION_OWNER_V1: &str = "communication_explanation";
pub const COMMUNICATION_EXPLANATION_MODULE_ID_V1: &str = "makosh-communication-explanation-runtime";
pub const COMMUNICATION_EXPLANATION_CAPABILITY_ID_V1: &str = "communication.explanation.v1";
pub const COMMUNICATION_EXPLANATION_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.explanation.command";
pub const COMMUNICATION_EXPLANATION_QUERY_CONTRACT_NAME_V1: &str =
    "communication.explanation.query";
pub const COMMUNICATION_EXPLANATION_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.explanation.status_changed";
pub const COMMUNICATION_EXPLANATION_REALTIME_EVENT_KIND_V1: &str =
    "communication.explanation.status_changed";
pub const COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.communication_explanation.v1.CommunicationExplanationCommandService/Start";
pub const COMMUNICATION_EXPLANATION_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communication_explanation.v1.CommunicationExplanationQueryService/Get";
pub const COMMUNICATION_EXPLANATION_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_EXPLANATION_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_EXPLANATION_MAX_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_EXPLANATION_MAX_REASONS_V1: usize = 8;
pub const COMMUNICATION_EXPLANATION_MAX_REASON_TEXT_BYTES_V1: usize = 512;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_explanation.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_explanation_schema.rs"
));

pub const COMMUNICATION_EXPLANATION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-explanation-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_bounded_typed_and_provider_neutral() {
        assert!(COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_EXPLANATION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source = include_str!("../proto/makosh/communication_explanation/v1/explanation.proto");
        assert!(source.contains("CommunicationExplanationReasonV1"));
        assert!(source.contains("COMMUNICATION_EXPLANATION_REASON_KIND_URGENCY"));
        assert!(source.contains("COMMUNICATION_EXPLANATION_SOURCE_BASIS_COMBINED"));
        for forbidden in [
            "provider_id",
            "model_id",
            "endpoint",
            "prompt",
            "source_body",
            "recipient",
            "task",
            "note",
            "map<",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden client field {forbidden}"
            );
        }
    }
}
