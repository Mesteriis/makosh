#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-translation-api";
pub const COMMUNICATION_TRANSLATION_OWNER_V1: &str = "communication_translation";
pub const COMMUNICATION_TRANSLATION_MODULE_ID_V1: &str = "makosh-communication-translation-runtime";
pub const COMMUNICATION_TRANSLATION_CAPABILITY_ID_V1: &str = "communication.translation.v1";
pub const COMMUNICATION_TRANSLATION_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.translation.command";
pub const COMMUNICATION_TRANSLATION_QUERY_CONTRACT_NAME_V1: &str =
    "communication.translation.query";
pub const COMMUNICATION_TRANSLATION_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.translation.status_changed";
pub const COMMUNICATION_TRANSLATION_REALTIME_EVENT_KIND_V1: &str =
    "communication.translation.status_changed";
pub const COMMUNICATION_TRANSLATION_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.communication_translation.v1.CommunicationTranslationCommandService/Start";
pub const COMMUNICATION_TRANSLATION_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communication_translation.v1.CommunicationTranslationQueryService/Get";
pub const COMMUNICATION_TRANSLATION_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_TRANSLATION_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_TRANSLATION_MAX_BYTES_V1: usize = 64 * 1024;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_translation.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_translation_schema.rs"
));

pub const COMMUNICATION_TRANSLATION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-translation-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_exact_provider_neutral_and_single_message_only() {
        assert!(COMMUNICATION_TRANSLATION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_TRANSLATION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source = include_str!("../proto/makosh/communication_translation/v1/translation.proto");
        assert!(source.contains("CommunicationTranslationCandidateV1"));
        assert!(source.contains("COMMUNICATION_TRANSLATION_LANGUAGE_SPANISH"));
        for forbidden in [
            "provider_id",
            "model_id",
            "endpoint",
            "prompt",
            "source_body",
            "thread_id",
            "attachment_id",
            "map<",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden client field {forbidden}"
            );
        }
    }
}
