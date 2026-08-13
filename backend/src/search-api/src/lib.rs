#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-search-api";
pub const SEARCH_OWNER_ID_V1: &str = "search";
pub const SEARCH_MODULE_ID_V1: &str = "makosh-search-runtime";
pub const SEARCH_CLIENT_CAPABILITY_ID_V1: &str = "search.client.v1";
pub const SEARCH_PROJECTION_CAPABILITY_ID_V1: &str = "search.projection.v1";
pub const SEARCH_STORAGE_CAPABILITY_ID_V1: &str = "search.storage.v1";
pub const SEARCH_QUERY_CONNECT_PATH_V1: &str = "/makosh.search.v1.SearchQueryService/Query";
pub const SEARCH_STATUS_CONNECT_PATH_V1: &str = "/makosh.search.v1.SearchQueryService/GetStatus";
pub const SEARCH_CONTRACT_MAJOR_V1: u32 = 1;
pub const SEARCH_CONTRACT_REVISION_V1: u32 = 1;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.search.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/search_schema.rs"));
pub const SEARCH_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/search-v1.bin"));

use makosh_runtime_protocol::v1::ContractReferenceV1;

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: SEARCH_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: SEARCH_CONTRACT_MAJOR_V1,
        revision: SEARCH_CONTRACT_REVISION_V1,
        schema_sha256: SEARCH_SCHEMA_SHA256_V1.to_vec(),
    }
}

#[must_use]
pub fn search_query_contract_reference_v1() -> ContractReferenceV1 {
    contract("search_query")
}

#[must_use]
pub fn search_status_contract_reference_v1() -> ContractReferenceV1 {
    contract("search_projection_status")
}

#[must_use]
pub fn search_client_routes_v1() -> [(ContractReferenceV1, &'static str); 2] {
    [
        (
            search_query_contract_reference_v1(),
            SEARCH_QUERY_CONNECT_PATH_V1,
        ),
        (
            search_status_contract_reference_v1(),
            SEARCH_STATUS_CONNECT_PATH_V1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_contract_is_read_only_and_private_free() {
        let schema = include_str!("../proto/makosh/search/v1/search.proto").to_ascii_lowercase();
        assert!(SEARCH_SCHEMA_SHA256_V1.iter().any(|value| *value != 0));
        assert_eq!(search_client_routes_v1().len(), 2);
        for forbidden in [
            "rpc create",
            "rpc update",
            "rpc delete",
            "credential",
            "provider_payload",
            "private_locator",
            "confidence",
            "risk",
            "map<",
            "json",
        ] {
            assert!(!schema.contains(forbidden), "{forbidden}");
        }
    }
}
