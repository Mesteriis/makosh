#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-reply-suggestion-api";
pub const COMMUNICATION_REPLY_SUGGESTION_OWNER_V1: &str = "communication_reply_suggestion";
pub const COMMUNICATION_REPLY_SUGGESTION_MODULE_ID_V1: &str =
    "makosh-communication-reply-suggestion-runtime";
pub const COMMUNICATION_REPLY_SUGGESTION_CAPABILITY_ID_V1: &str =
    "communication.reply_suggestion.v1";
pub const COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.reply_suggestion.command";
pub const COMMUNICATION_REPLY_SUGGESTION_QUERY_CONTRACT_NAME_V1: &str =
    "communication.reply_suggestion.query";
pub const COMMUNICATION_REPLY_SUGGESTION_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.reply_suggestion.status_changed";
pub const COMMUNICATION_REPLY_SUGGESTION_REALTIME_EVENT_KIND_V1: &str =
    "communication.reply_suggestion.status_changed";
pub const COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.communication_reply_suggestion.v1.CommunicationReplySuggestionCommandService/Start";
pub const COMMUNICATION_REPLY_SUGGESTION_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communication_reply_suggestion.v1.CommunicationReplySuggestionQueryService/Get";
pub const COMMUNICATION_REPLY_SUGGESTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_REPLY_SUGGESTION_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_REPLY_SUGGESTION_MAX_SUBJECT_BYTES_V1: usize = 998;
pub const COMMUNICATION_REPLY_SUGGESTION_MAX_BODY_BYTES_V1: usize = 64 * 1024;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_reply_suggestion.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_reply_suggestion_schema.rs"
));

pub const COMMUNICATION_REPLY_SUGGESTION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-reply-suggestion-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_concrete_provider_neutral_and_has_no_source_body() {
        assert!(COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_REPLY_SUGGESTION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source = include_str!(
            "../proto/makosh/communication_reply_suggestion/v1/reply_suggestion.proto"
        );
        assert!(source.contains("ReplySuggestionCandidateV1"));
        assert!(source.contains("REPLY_SUGGESTION_TONE_PROFESSIONAL"));
        assert!(source.contains("REPLY_SUGGESTION_LANGUAGE_SPANISH"));
        assert!(!source.contains("provider_id"));
        assert!(!source.contains("model_id"));
        assert!(!source.contains("endpoint"));
        assert!(!source.contains("prompt"));
        assert!(!source.contains("source_body"));
        assert!(!source.contains("map<"));
    }
}
