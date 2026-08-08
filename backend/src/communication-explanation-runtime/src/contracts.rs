use makosh_communication_explanation_api::{
    COMMUNICATION_EXPLANATION_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_EXPLANATION_CONTRACT_MAJOR_V1, COMMUNICATION_EXPLANATION_CONTRACT_REVISION_V1,
    COMMUNICATION_EXPLANATION_OWNER_V1, COMMUNICATION_EXPLANATION_QUERY_CONTRACT_NAME_V1,
    COMMUNICATION_EXPLANATION_REALTIME_CONTRACT_NAME_V1, COMMUNICATION_EXPLANATION_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn communication_explanation_command_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_EXPLANATION_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn communication_explanation_query_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_EXPLANATION_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn communication_explanation_realtime_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_EXPLANATION_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_EXPLANATION_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_EXPLANATION_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_EXPLANATION_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_EXPLANATION_SCHEMA_SHA256.to_vec(),
    }
}
