#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-risk-api";
pub const RISK_OWNER_ID_V1: &str = "risk";
pub const RISK_MODULE_ID_V1: &str = "makosh-risk-runtime";
pub const RISK_CLIENT_CAPABILITY_ID_V1: &str = "risk.client.v1";
pub const RISK_SIGNAL_CAPABILITY_ID_V1: &str = "risk.signal.consume.v1";
pub const RISK_STORAGE_CAPABILITY_ID_V1: &str = "risk.storage.v1";
pub const RISK_LIST_CONNECT_PATH_V1: &str = "/makosh.risk.v1.RiskQueryService/List";
pub const RISK_STATUS_CONNECT_PATH_V1: &str = "/makosh.risk.v1.RiskQueryService/GetStatus";

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.risk.v1.rs"));
}
include!(concat!(env!("OUT_DIR"), "/risk_schema.rs"));
pub const RISK_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/risk-v1.bin"));

use makosh_runtime_protocol::v1::ContractReferenceV1;
fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: RISK_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: RISK_SCHEMA_SHA256_V1.to_vec(),
    }
}
#[must_use]
pub fn risk_list_contract_reference_v1() -> ContractReferenceV1 {
    contract("risk_list")
}
#[must_use]
pub fn risk_status_contract_reference_v1() -> ContractReferenceV1 {
    contract("risk_status")
}
#[must_use]
pub fn risk_client_routes_v1() -> [(ContractReferenceV1, &'static str); 2] {
    [
        (risk_list_contract_reference_v1(), RISK_LIST_CONNECT_PATH_V1),
        (
            risk_status_contract_reference_v1(),
            RISK_STATUS_CONNECT_PATH_V1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contract_is_read_only_and_private_free() {
        let schema = include_str!("../proto/makosh/risk/v1/risk.proto").to_ascii_lowercase();
        assert_eq!(risk_client_routes_v1().len(), 2);
        for forbidden in [
            "rpc create",
            "rpc update",
            "rpc delete",
            "credential",
            "private_locator",
            "provider_payload",
            "confidence",
            "map<",
            "json",
        ] {
            assert!(!schema.contains(forbidden), "{forbidden}");
        }
    }
}
