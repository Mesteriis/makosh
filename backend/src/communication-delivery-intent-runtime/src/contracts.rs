//! Exact public contract references served by this workflow runtime.

use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1, COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
    COMMUNICATION_DELIVERY_INTENT_QUERY_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn command_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn query_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_DELIVERY_INTENT_QUERY_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_and_query_contracts_are_exact_and_distinct() {
        assert_ne!(command_contract_v1(), query_contract_v1());
        assert_eq!(
            command_contract_v1().schema_sha256,
            COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256
        );
    }
}
