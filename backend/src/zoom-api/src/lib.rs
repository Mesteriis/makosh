#![forbid(unsafe_code)]
use makosh_runtime_protocol::v1::{ContractReferenceV1, SettingsSchemaV1};
use prost::Message;
pub const PACKAGE: &str = "makosh-zoom-api";
pub const ZOOM_OWNER_ID_V1: &str = "zoom";
pub const ZOOM_MODULE_ID_V1: &str = "makosh-zoom-runtime";
pub const ZOOM_ACCOUNT_CLIENT_CAPABILITY_ID_V1: &str = "zoom.account.client.v1";
pub const ZOOM_PROVIDER_CAPABILITY_ID_V1: &str = "zoom.provider.v1";
pub const ZOOM_CREDENTIAL_PROVISION_CAPABILITY_ID_V1: &str = "zoom.credential.provision.v1";
pub const ZOOM_CREDENTIAL_RESOLVE_CAPABILITY_ID_V1: &str = "zoom.credential.resolve.v1";
pub const ZOOM_CALL_EVIDENCE_CAPABILITY_ID_V1: &str = "zoom.call-evidence.publish.v1";
pub const ZOOM_STORAGE_CAPABILITY_ID_V1: &str = "zoom.storage.v1";
pub const ZOOM_CREDENTIAL_PURPOSE_ID_V1: &str = "zoom.provider-credential.v1";
pub const ZOOM_LIST_ACCOUNTS_PATH_V1: &str = "/makosh.zoom.v1.ZoomQueryService/ListAccounts";
pub const ZOOM_GET_STATUS_PATH_V1: &str = "/makosh.zoom.v1.ZoomQueryService/GetStatus";
pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.zoom.v1.rs"));
}
include!(concat!(env!("OUT_DIR"), "/zoom_schema.rs"));
pub const ZOOM_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/zoom-v1.bin"));
#[must_use]
pub fn zoom_settings_schema_bytes_v1() -> Vec<u8> {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
    .encode_to_vec()
}
fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ZOOM_OWNER_ID_V1.into(),
        name: name.into(),
        major: 1,
        revision: 1,
        schema_sha256: ZOOM_SCHEMA_SHA256_V1.to_vec(),
    }
}
#[must_use]
pub fn zoom_client_routes_v1() -> [(ContractReferenceV1, &'static str); 2] {
    [
        (contract("zoom_list_accounts"), ZOOM_LIST_ACCOUNTS_PATH_V1),
        (contract("zoom_status"), ZOOM_GET_STATUS_PATH_V1),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contract_is_read_only_and_private_free() {
        let schema = include_str!("../proto/makosh/zoom/v1/zoom.proto").to_ascii_lowercase();
        assert_eq!(zoom_client_routes_v1().len(), 2);
        for forbidden in [
            "rpc create",
            "rpc update",
            "rpc delete",
            "credential",
            "token",
            "provider_payload",
            "join_url",
            "webhook",
            "map<",
            "json",
        ] {
            assert!(!schema.contains(forbidden), "{forbidden}");
        }
    }
}
