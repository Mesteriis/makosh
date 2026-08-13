#![forbid(unsafe_code)]
pub const PACKAGE: &str = "makosh-consistency-api";
pub const CONSISTENCY_OWNER_ID_V1: &str = "consistency";
pub const CONSISTENCY_MODULE_ID_V1: &str = "makosh-consistency-runtime";
pub const CONSISTENCY_CLIENT_CAPABILITY_ID_V1: &str = "consistency.client.v1";
pub const CONSISTENCY_CLAIM_CAPABILITY_ID_V1: &str = "consistency.claim.consume.v1";
pub const CONSISTENCY_STORAGE_CAPABILITY_ID_V1: &str = "consistency.storage.v1";
pub const CONSISTENCY_CONTRADICTIONS_PATH_V1: &str =
    "/makosh.consistency.v1.ConsistencyQueryService/ListContradictions";
pub const CONSISTENCY_STATUS_PATH_V1: &str =
    "/makosh.consistency.v1.ConsistencyQueryService/GetStatus";
pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.consistency.v1.rs"));
}
include!(concat!(env!("OUT_DIR"), "/consistency_schema.rs"));
pub const CONSISTENCY_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/consistency-v1.bin"));
use makosh_runtime_protocol::v1::ContractReferenceV1;
fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CONSISTENCY_OWNER_ID_V1.into(),
        name: name.into(),
        major: 1,
        revision: 1,
        schema_sha256: CONSISTENCY_SCHEMA_SHA256_V1.to_vec(),
    }
}
#[must_use]
pub fn consistency_contradictions_contract_reference_v1() -> ContractReferenceV1 {
    contract("consistency_contradictions")
}
#[must_use]
pub fn consistency_status_contract_reference_v1() -> ContractReferenceV1 {
    contract("consistency_status")
}
#[must_use]
pub fn consistency_client_routes_v1() -> [(ContractReferenceV1, &'static str); 2] {
    [
        (
            consistency_contradictions_contract_reference_v1(),
            CONSISTENCY_CONTRADICTIONS_PATH_V1,
        ),
        (
            consistency_status_contract_reference_v1(),
            CONSISTENCY_STATUS_PATH_V1,
        ),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn api_is_read_only_and_has_no_inference() {
        let schema =
            include_str!("../proto/makosh/consistency/v1/consistency.proto").to_ascii_lowercase();
        assert_eq!(consistency_client_routes_v1().len(), 2);
        for forbidden in [
            "rpc create",
            "rpc update",
            "rpc delete",
            "confidence",
            "risk",
            "inference",
            "credential",
            "provider_payload",
            "private_locator",
            "map<",
            "json",
        ] {
            assert!(!schema.contains(forbidden), "{forbidden}");
        }
    }
}
