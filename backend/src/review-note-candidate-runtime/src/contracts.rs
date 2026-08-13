use makosh_review_note_candidate_api::{
    REVIEW_NOTE_CANDIDATE_COMMAND_CONTRACT_NAME_V1, REVIEW_NOTE_CANDIDATE_CONTRACT_MAJOR_V1,
    REVIEW_NOTE_CANDIDATE_CONTRACT_REVISION_V1, REVIEW_NOTE_CANDIDATE_LIST_CONTRACT_NAME_V1,
    REVIEW_NOTE_CANDIDATE_OWNER_V1, REVIEW_NOTE_CANDIDATE_QUERY_CONTRACT_NAME_V1,
    REVIEW_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1, REVIEW_NOTE_CANDIDATE_SCHEMA_SHA256_V1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn command_contract_v1() -> ContractReferenceV1 {
    contract(REVIEW_NOTE_CANDIDATE_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn query_contract_v1() -> ContractReferenceV1 {
    contract(REVIEW_NOTE_CANDIDATE_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn list_contract_v1() -> ContractReferenceV1 {
    contract(REVIEW_NOTE_CANDIDATE_LIST_CONTRACT_NAME_V1)
}

pub(crate) fn realtime_contract_v1() -> ContractReferenceV1 {
    contract(REVIEW_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: REVIEW_NOTE_CANDIDATE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: REVIEW_NOTE_CANDIDATE_CONTRACT_MAJOR_V1,
        revision: REVIEW_NOTE_CANDIDATE_CONTRACT_REVISION_V1,
        schema_sha256: REVIEW_NOTE_CANDIDATE_SCHEMA_SHA256_V1.to_vec(),
    }
}
