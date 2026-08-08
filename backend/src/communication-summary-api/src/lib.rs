#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-summary-api";
pub const COMMUNICATION_SUMMARY_OWNER_V1: &str = "communication_summary";
pub const COMMUNICATION_SUMMARY_MODULE_ID_V1: &str = "makosh-communication-summary-runtime";
pub const COMMUNICATION_SUMMARY_CAPABILITY_ID_V1: &str = "communication.summary.v1";
pub const COMMUNICATION_SUMMARY_COMMAND_CONTRACT_NAME_V1: &str = "communication.summary.command";
pub const COMMUNICATION_SUMMARY_QUERY_CONTRACT_NAME_V1: &str = "communication.summary.query";
pub const COMMUNICATION_SUMMARY_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.summary.status_changed";
pub const COMMUNICATION_SUMMARY_REALTIME_EVENT_KIND_V1: &str =
    "communication.summary.status_changed";
pub const COMMUNICATION_SUMMARY_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.communication_summary.v1.CommunicationSummaryCommandService/Start";
pub const COMMUNICATION_SUMMARY_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communication_summary.v1.CommunicationSummaryQueryService/Get";
pub const COMMUNICATION_SUMMARY_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_SUMMARY_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_SUMMARY_MAX_BYTES_V1: usize = 64 * 1024;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_summary.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/communication_summary_schema.rs"));

pub const COMMUNICATION_SUMMARY_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/communication-summary-v1.bin"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_concrete_provider_neutral_and_has_no_source_body() {
        assert!(COMMUNICATION_SUMMARY_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_SUMMARY_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source = include_str!("../proto/makosh/communication_summary/v1/summary.proto");
        assert!(source.contains("CommunicationSummaryCandidateV1"));
        assert!(source.contains("COMMUNICATION_SUMMARY_LENGTH_DETAILED"));
        assert!(source.contains("COMMUNICATION_SUMMARY_LANGUAGE_SPANISH"));
        for forbidden in [
            "provider_id",
            "model_id",
            "endpoint",
            "prompt",
            "source_body",
            "action_items",
            "deadlines",
            "map<",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden client field {forbidden}"
            );
        }
    }
}
