use makosh_communication_note_candidate_api::{
    COMMUNICATION_NOTE_CANDIDATE_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_NOTE_CANDIDATE_CONTRACT_MAJOR_V1,
    COMMUNICATION_NOTE_CANDIDATE_CONTRACT_REVISION_V1, COMMUNICATION_NOTE_CANDIDATE_OWNER_V1,
    COMMUNICATION_NOTE_CANDIDATE_QUERY_CONTRACT_NAME_V1,
    COMMUNICATION_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_NOTE_CANDIDATE_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn communication_note_candidate_command_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_NOTE_CANDIDATE_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn communication_note_candidate_query_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_NOTE_CANDIDATE_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn communication_note_candidate_realtime_contract_v1() -> ContractReferenceV1 {
    contract(COMMUNICATION_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_NOTE_CANDIDATE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_NOTE_CANDIDATE_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_NOTE_CANDIDATE_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_NOTE_CANDIDATE_SCHEMA_SHA256.to_vec(),
    }
}
