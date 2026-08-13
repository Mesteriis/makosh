#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-memory-api";
pub const MEMORY_OWNER_ID_V1: &str = "memory";
pub const MEMORY_MODULE_ID_V1: &str = "makosh-memory-runtime";
pub const MEMORY_CLIENT_CAPABILITY_ID_V1: &str = "memory.client.v1";
pub const MEMORY_EVIDENCE_CAPABILITY_ID_V1: &str = "memory.evidence.consume.v1";
pub const MEMORY_STORAGE_CAPABILITY_ID_V1: &str = "memory.storage.v1";
pub const MEMORY_LIST_CONNECT_PATH_V1: &str = "/makosh.memory.v1.MemoryQueryService/List";
pub const MEMORY_STATUS_CONNECT_PATH_V1: &str = "/makosh.memory.v1.MemoryQueryService/GetStatus";

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.memory.v1.rs"));
}
include!(concat!(env!("OUT_DIR"), "/memory_schema.rs"));
pub const MEMORY_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/memory-v1.bin"));

use makosh_runtime_protocol::v1::ContractReferenceV1;
fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: MEMORY_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: MEMORY_SCHEMA_SHA256_V1.to_vec(),
    }
}
#[must_use]
pub fn memory_list_contract_reference_v1() -> ContractReferenceV1 {
    contract("memory_list")
}
#[must_use]
pub fn memory_status_contract_reference_v1() -> ContractReferenceV1 {
    contract("memory_status")
}
#[must_use]
pub fn memory_client_routes_v1() -> [(ContractReferenceV1, &'static str); 2] {
    [
        (
            memory_list_contract_reference_v1(),
            MEMORY_LIST_CONNECT_PATH_V1,
        ),
        (
            memory_status_contract_reference_v1(),
            MEMORY_STATUS_CONNECT_PATH_V1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contract_is_read_only_and_private_free() {
        let schema = include_str!("../proto/makosh/memory/v1/memory.proto").to_ascii_lowercase();
        assert_eq!(memory_client_routes_v1().len(), 2);
        for forbidden in [
            "rpc create",
            "rpc update",
            "rpc delete",
            "credential",
            "private_locator",
            "provider_payload",
            "confidence",
            "risk",
            "map<",
            "json",
        ] {
            assert!(!schema.contains(forbidden), "{forbidden}");
        }
    }
}
