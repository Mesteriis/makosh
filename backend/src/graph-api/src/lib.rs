#![forbid(unsafe_code)]
pub const PACKAGE: &str = "makosh-graph-api";
pub const GRAPH_OWNER_ID_V1: &str = "graph";
pub const GRAPH_MODULE_ID_V1: &str = "makosh-graph-runtime";
pub const GRAPH_CLIENT_CAPABILITY_ID_V1: &str = "graph.client.v1";
pub const GRAPH_PROJECTION_CAPABILITY_ID_V1: &str = "graph.projection.v1";
pub const GRAPH_STORAGE_CAPABILITY_ID_V1: &str = "graph.storage.v1";
pub const GRAPH_NEIGHBORS_PATH_V1: &str = "/makosh.graph.v1.GraphQueryService/Neighbors";
pub const GRAPH_PATH_PATH_V1: &str = "/makosh.graph.v1.GraphQueryService/Path";
pub const GRAPH_STATUS_PATH_V1: &str = "/makosh.graph.v1.GraphQueryService/GetStatus";
pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.graph.v1.rs"));
}
include!(concat!(env!("OUT_DIR"), "/graph_schema.rs"));
pub const GRAPH_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/graph-v1.bin"));
use makosh_runtime_protocol::v1::ContractReferenceV1;
fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: GRAPH_OWNER_ID_V1.into(),
        name: name.into(),
        major: 1,
        revision: 1,
        schema_sha256: GRAPH_SCHEMA_SHA256_V1.to_vec(),
    }
}
#[must_use]
pub fn graph_neighbors_contract_reference_v1() -> ContractReferenceV1 {
    contract("graph_neighbors")
}
#[must_use]
pub fn graph_path_contract_reference_v1() -> ContractReferenceV1 {
    contract("graph_path")
}
#[must_use]
pub fn graph_status_contract_reference_v1() -> ContractReferenceV1 {
    contract("graph_status")
}
#[must_use]
pub fn graph_client_routes_v1() -> [(ContractReferenceV1, &'static str); 3] {
    [
        (
            graph_neighbors_contract_reference_v1(),
            GRAPH_NEIGHBORS_PATH_V1,
        ),
        (graph_path_contract_reference_v1(), GRAPH_PATH_PATH_V1),
        (graph_status_contract_reference_v1(), GRAPH_STATUS_PATH_V1),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn api_is_read_only_and_has_no_inference() {
        let schema = include_str!("../proto/makosh/graph/v1/graph.proto").to_ascii_lowercase();
        assert_eq!(graph_client_routes_v1().len(), 3);
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
