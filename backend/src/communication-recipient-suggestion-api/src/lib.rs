#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-recipient-suggestion-api";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_OWNER_V1: &str = "communication_recipient_suggestion";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_MODULE_ID_V1: &str =
    "makosh-communication-recipient-suggestion-runtime";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_CAPABILITY_ID_V1: &str =
    "communication.recipient-suggestion.v1";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.recipient-suggestion.command";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_QUERY_CONTRACT_NAME_V1: &str =
    "communication.recipient-suggestion.query";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.recipient-suggestion.status_changed";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_EVENT_KIND_V1: &str =
    "communication.recipient-suggestion.status_changed";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONNECT_PATH_V1: &str = "/makosh.communication_recipient_suggestion.v1.CommunicationRecipientSuggestionCommandService/Start";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_QUERY_CONNECT_PATH_V1: &str = "/makosh.communication_recipient_suggestion.v1.CommunicationRecipientSuggestionQueryService/Get";
pub const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_RECIPIENT_SUGGESTION_MAX_BYTES_V1: usize = 32 * 1024;
pub const COMMUNICATION_RECIPIENT_SUGGESTION_MAX_CANDIDATES_V1: usize = 3;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_recipient_suggestion.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_recipient_suggestion_schema.rs"
));

pub const COMMUNICATION_RECIPIENT_SUGGESTION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-recipient-suggestion-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_bounded_typed_and_provider_neutral() {
        assert!(COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_RECIPIENT_SUGGESTION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source = include_str!(
            "../proto/makosh/communication_recipient_suggestion/v1/recipient_suggestion.proto"
        );
        assert!(source.contains("COMMUNICATION_RECIPIENT_ROLE_ACCOUNTING_OR_BOOKKEEPING"));
        assert!(source.contains("COMMUNICATION_RECIPIENT_ROLE_LEGAL_COUNSEL"));
        assert!(source.contains("COMMUNICATION_RECIPIENT_ROLE_PROJECT_STAKEHOLDER"));
        for forbidden in [
            "email_address",
            "contact_id",
            "person_id",
            "organization_id",
            "provider_id",
            "account_id",
            "model_id",
            "prompt",
            "source_body",
            "map<",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden client field {forbidden}"
            );
        }
    }
}
