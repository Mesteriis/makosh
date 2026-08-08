use makosh_review_attention_api::{
    REVIEW_ATTENTION_CLIENT_SCHEMA_SHA256_V1, REVIEW_ATTENTION_COMMAND_CONTRACT_NAME_V1,
    REVIEW_ATTENTION_CONTRACT_MAJOR_V1, REVIEW_ATTENTION_CONTRACT_REVISION_V1,
    REVIEW_ATTENTION_OWNER_V1, REVIEW_ATTENTION_QUERY_CONTRACT_NAME_V1,
    REVIEW_ATTENTION_REALTIME_CONTRACT_NAME_V1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

#[must_use]
pub fn review_attention_command_contract_v1() -> ContractReferenceV1 {
    contract(REVIEW_ATTENTION_COMMAND_CONTRACT_NAME_V1)
}

#[must_use]
pub fn review_attention_query_contract_v1() -> ContractReferenceV1 {
    contract(REVIEW_ATTENTION_QUERY_CONTRACT_NAME_V1)
}

#[must_use]
pub fn review_attention_realtime_contract_v1() -> ContractReferenceV1 {
    contract(REVIEW_ATTENTION_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: REVIEW_ATTENTION_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: REVIEW_ATTENTION_CONTRACT_MAJOR_V1,
        revision: REVIEW_ATTENTION_CONTRACT_REVISION_V1,
        schema_sha256: REVIEW_ATTENTION_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}
