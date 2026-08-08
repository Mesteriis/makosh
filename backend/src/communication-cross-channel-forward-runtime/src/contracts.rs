use makosh_communication_cross_channel_forward_api::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_MAJOR_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_REVISION_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONTRACT_NAME_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn cross_channel_forward_command_contract_v1() -> ContractReferenceV1 {
    cross_channel_forward_contract_v1(COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn cross_channel_forward_query_contract_v1() -> ContractReferenceV1 {
    cross_channel_forward_contract_v1(COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn cross_channel_forward_realtime_contract_v1() -> ContractReferenceV1 {
    cross_channel_forward_contract_v1(COMMUNICATION_CROSS_CHANNEL_FORWARD_REALTIME_CONTRACT_NAME_V1)
}

fn cross_channel_forward_contract_v1(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_CROSS_CHANNEL_FORWARD_SCHEMA_SHA256.to_vec(),
    }
}
