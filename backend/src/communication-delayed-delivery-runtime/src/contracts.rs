use makosh_communication_delayed_delivery_api::{
    COMMUNICATION_DELAYED_DELIVERY_CANCEL_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_DELAYED_DELIVERY_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELAYED_DELIVERY_CONTRACT_REVISION_V1, COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
    COMMUNICATION_DELAYED_DELIVERY_QUERY_CONTRACT_NAME_V1,
    COMMUNICATION_DELAYED_DELIVERY_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_DELAYED_DELIVERY_SCHEMA_SHA256,
};
use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1, COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
    COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

#[must_use]
pub fn delayed_delivery_schedule_command_contract_v1() -> ContractReferenceV1 {
    delayed_delivery_contract(COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_COMMAND_CONTRACT_NAME_V1)
}

#[must_use]
pub fn delayed_delivery_cancel_command_contract_v1() -> ContractReferenceV1 {
    delayed_delivery_contract(COMMUNICATION_DELAYED_DELIVERY_CANCEL_COMMAND_CONTRACT_NAME_V1)
}

#[must_use]
pub fn delayed_delivery_query_contract_v1() -> ContractReferenceV1 {
    delayed_delivery_contract(COMMUNICATION_DELAYED_DELIVERY_QUERY_CONTRACT_NAME_V1)
}

#[must_use]
pub fn delayed_delivery_realtime_contract_v1() -> ContractReferenceV1 {
    delayed_delivery_contract(COMMUNICATION_DELAYED_DELIVERY_REALTIME_CONTRACT_NAME_V1)
}

#[must_use]
pub fn delivery_intent_command_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
        name: COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1.to_owned(),
        major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
    }
}

fn delayed_delivery_contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_DELAYED_DELIVERY_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_DELAYED_DELIVERY_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_DELAYED_DELIVERY_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_DELAYED_DELIVERY_SCHEMA_SHA256.to_vec(),
    }
}
