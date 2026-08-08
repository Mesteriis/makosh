use makosh_communication_recipient_suggestion_api::{
    COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_MAJOR_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_REVISION_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_OWNER_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_QUERY_CONTRACT_NAME_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn communication_recipient_suggestion_command_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn communication_recipient_suggestion_query_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_RECIPIENT_SUGGESTION_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn communication_recipient_suggestion_realtime_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_RECIPIENT_SUGGESTION_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_RECIPIENT_SUGGESTION_SCHEMA_SHA256.to_vec(),
    }
}
