use makosh_communication_summary_api::{
    COMMUNICATION_SUMMARY_COMMAND_CONTRACT_NAME_V1, COMMUNICATION_SUMMARY_CONTRACT_MAJOR_V1,
    COMMUNICATION_SUMMARY_CONTRACT_REVISION_V1, COMMUNICATION_SUMMARY_OWNER_V1,
    COMMUNICATION_SUMMARY_QUERY_CONTRACT_NAME_V1, COMMUNICATION_SUMMARY_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_SUMMARY_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn communication_summary_command_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_SUMMARY_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn communication_summary_query_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_SUMMARY_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn communication_summary_realtime_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_SUMMARY_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_SUMMARY_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_SUMMARY_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_SUMMARY_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_SUMMARY_SCHEMA_SHA256.to_vec(),
    }
}
