#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-timeline-api";
pub const TIMELINE_OWNER_ID_V1: &str = "timeline";
pub const TIMELINE_MODULE_ID_V1: &str = "makosh-timeline-runtime";
pub const TIMELINE_CLIENT_CAPABILITY_ID_V1: &str = "timeline.client.v1";
pub const TIMELINE_PROJECTION_CAPABILITY_ID_V1: &str = "timeline.projection.v1";
pub const TIMELINE_STORAGE_CAPABILITY_ID_V1: &str = "timeline.storage.v1";
pub const TIMELINE_LIST_CONNECT_PATH_V1: &str = "/makosh.timeline.v1.TimelineQueryService/List";
pub const TIMELINE_STATUS_CONNECT_PATH_V1: &str =
    "/makosh.timeline.v1.TimelineQueryService/GetStatus";

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.timeline.v1.rs"));
}
include!(concat!(env!("OUT_DIR"), "/timeline_schema.rs"));
pub const TIMELINE_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/timeline-v1.bin"));

use makosh_runtime_protocol::v1::ContractReferenceV1;
fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: TIMELINE_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: TIMELINE_SCHEMA_SHA256_V1.to_vec(),
    }
}
#[must_use]
pub fn timeline_list_contract_reference_v1() -> ContractReferenceV1 {
    contract("timeline_list")
}
#[must_use]
pub fn timeline_status_contract_reference_v1() -> ContractReferenceV1 {
    contract("timeline_status")
}
#[must_use]
pub fn timeline_client_routes_v1() -> [(ContractReferenceV1, &'static str); 2] {
    [
        (
            timeline_list_contract_reference_v1(),
            TIMELINE_LIST_CONNECT_PATH_V1,
        ),
        (
            timeline_status_contract_reference_v1(),
            TIMELINE_STATUS_CONNECT_PATH_V1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contract_is_read_only_and_private_free() {
        let schema =
            include_str!("../proto/makosh/timeline/v1/timeline.proto").to_ascii_lowercase();
        assert_eq!(timeline_client_routes_v1().len(), 2);
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
