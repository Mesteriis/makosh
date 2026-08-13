#![forbid(unsafe_code)]
use makosh_runtime_protocol::v1::{ContractReferenceV1, SettingsSchemaV1};
use prost::Message;
pub const PACKAGE: &str = "makosh-telemost-api";
pub const TELEMOST_OWNER_ID_V1: &str = "telemost";
pub const TELEMOST_MODULE_ID_V1: &str = "makosh-telemost-runtime";
pub const TELEMOST_ACCOUNT_CLIENT_CAPABILITY_ID_V1: &str = "telemost.account.client.v1";
pub const TELEMOST_PROVIDER_CAPABILITY_ID_V1: &str = "telemost.provider.v1";
pub const TELEMOST_CREDENTIAL_PROVISION_CAPABILITY_ID_V1: &str = "telemost.credential.provision.v1";
pub const TELEMOST_CREDENTIAL_RESOLVE_CAPABILITY_ID_V1: &str = "telemost.credential.resolve.v1";
pub const TELEMOST_CALL_EVIDENCE_CAPABILITY_ID_V1: &str = "telemost.call-evidence.publish.v1";
pub const TELEMOST_STORAGE_CAPABILITY_ID_V1: &str = "telemost.storage.v1";
pub const TELEMOST_CREDENTIAL_PURPOSE_ID_V1: &str = "telemost.provider-credential.v1";
pub const TELEMOST_LIST_ACCOUNTS_PATH_V1: &str =
    "/makosh.telemost.v1.YandexTelemostQueryService/ListAccounts";
pub const TELEMOST_GET_STATUS_PATH_V1: &str =
    "/makosh.telemost.v1.YandexTelemostQueryService/GetStatus";
pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.telemost.v1.rs"));
}
include!(concat!(env!("OUT_DIR"), "/telemost_schema.rs"));
pub const TELEMOST_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/telemost-v1.bin"));
#[must_use]
pub fn telemost_settings_schema_bytes_v1() -> Vec<u8> {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
    .encode_to_vec()
}
fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: TELEMOST_OWNER_ID_V1.into(),
        name: name.into(),
        major: 1,
        revision: 1,
        schema_sha256: TELEMOST_SCHEMA_SHA256_V1.to_vec(),
    }
}
#[must_use]
pub fn telemost_client_routes_v1() -> [(ContractReferenceV1, &'static str); 2] {
    [
        (
            contract("telemost_list_accounts"),
            TELEMOST_LIST_ACCOUNTS_PATH_V1,
        ),
        (contract("telemost_status"), TELEMOST_GET_STATUS_PATH_V1),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contract_is_read_only_and_private_free() {
        let schema =
            include_str!("../proto/makosh/telemost/v1/telemost.proto").to_ascii_lowercase();
        assert_eq!(telemost_client_routes_v1().len(), 2);
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
