#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communications-call-evidence-api";
pub const CALL_EVIDENCE_CLIENT_OWNER_V1: &str = "communications";
pub const CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1: &str = "communications.call-evidence.client.v1";
pub const CALL_EVIDENCE_QUERY_CONTRACT_NAME_V1: &str = "communications.call-evidence.query";
pub const CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1: &str = "communications.call-evidence.changed";
pub const CALL_EVIDENCE_REALTIME_EVENT_KIND_V1: &str = "call_evidence_changed";
pub const CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1: u32 = 1;
pub const CALL_EVIDENCE_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communications.call_evidence.client.v1.CallEvidenceQueryService/Query";
pub const CALL_EVIDENCE_QUERY_MAX_PAGE_SIZE_V1: u16 = 100;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.call_evidence.client.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communications_call_evidence_client_schema.rs"
));

pub const CALL_EVIDENCE_CLIENT_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communications-call-evidence-client-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_metadata_only_and_route_exact() {
        assert!(CALL_EVIDENCE_QUERY_CONNECT_PATH_V1.starts_with('/'));
        assert_eq!(CALL_EVIDENCE_QUERY_MAX_PAGE_SIZE_V1, 100);
        let source =
            include_str!("../proto/makosh/communications/call_evidence/client/v1/client.proto");
        for forbidden in [
            "source_call_cursor",
            "account_cursor",
            "conversation_cursor",
            "participant_cursor",
            "provider_call_id",
            "phone_number",
            "transcript",
            "audio_bytes",
            "credential",
            "session_store",
            "map<",
            "google.protobuf.Any",
        ] {
            assert!(!source.contains(forbidden), "{forbidden}");
        }
    }
}
