#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-review-attention-api";
pub const REVIEW_ATTENTION_OWNER_V1: &str = "review";
pub const REVIEW_ATTENTION_MODULE_ID_V1: &str = "makosh-review-runtime";
pub const REVIEW_ATTENTION_COMMAND_CAPABILITY_ID_V1: &str =
    "review.communication-attention.command.v1";
pub const REVIEW_ATTENTION_QUERY_CAPABILITY_ID_V1: &str = "review.communication-attention.query.v1";
pub const REVIEW_ATTENTION_REALTIME_CAPABILITY_ID_V1: &str =
    "review.communication-attention.realtime.v1";
pub const REVIEW_ATTENTION_COMMAND_CONTRACT_NAME_V1: &str =
    "review.communication-attention.command";
pub const REVIEW_ATTENTION_QUERY_CONTRACT_NAME_V1: &str = "review.communication-attention.query";
pub const REVIEW_ATTENTION_REALTIME_CONTRACT_NAME_V1: &str =
    "review.communication-attention.changed";
pub const REVIEW_ATTENTION_REALTIME_EVENT_KIND_V1: &str = "review_attention_changed";
pub const REVIEW_ATTENTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const REVIEW_ATTENTION_CONTRACT_REVISION_V1: u32 = 1;
pub const REVIEW_ATTENTION_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.review.attention.client.v1.ReviewAttentionCommandService/Execute";
pub const REVIEW_ATTENTION_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.review.attention.client.v1.ReviewAttentionQueryService/Query";
pub const REVIEW_ATTENTION_MAX_PAGE_SIZE_V1: u16 = 100;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.review.attention.client.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/review_attention_client_schema.rs"
));

pub const REVIEW_ATTENTION_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/review-attention-client-v1.bin"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contracts_are_exact_separate_capabilities() {
        let capabilities = [
            REVIEW_ATTENTION_COMMAND_CAPABILITY_ID_V1,
            REVIEW_ATTENTION_QUERY_CAPABILITY_ID_V1,
            REVIEW_ATTENTION_REALTIME_CAPABILITY_ID_V1,
        ];
        assert_eq!(
            capabilities
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            capabilities.len(),
        );
        assert!(REVIEW_ATTENTION_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(REVIEW_ATTENTION_QUERY_CONNECT_PATH_V1.starts_with('/'));
        assert_eq!(REVIEW_ATTENTION_MAX_PAGE_SIZE_V1, 100);
    }

    #[test]
    fn client_contract_contains_no_provider_action_or_private_content() {
        let source = include_str!("../proto/makosh/review/attention/client/v1/client.proto");
        for forbidden in [
            "provider",
            "archive_message",
            "mute_chat",
            "mark_provider_read",
            "message_body",
            "subject",
            "phone_number",
            "email_address",
            "credential",
            "session",
            "map<",
            "google.protobuf.Any",
        ] {
            assert!(!source.contains(forbidden), "{forbidden}");
        }
    }
}
