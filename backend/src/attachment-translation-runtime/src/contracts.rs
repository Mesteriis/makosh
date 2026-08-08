use makosh_attachment_translation_api::{
    ATTACHMENT_TRANSLATION_COMMAND_CONTRACT_NAME_V1, ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1,
    ATTACHMENT_TRANSLATION_CONTRACT_REVISION_V1, ATTACHMENT_TRANSLATION_CONTROL_SCHEMA_SHA256,
    ATTACHMENT_TRANSLATION_OWNER_V1, ATTACHMENT_TRANSLATION_QUERY_CONTRACT_NAME_V1,
    ATTACHMENT_TRANSLATION_READ_CONTRACT_NAME_V1, ATTACHMENT_TRANSLATION_READ_SCHEMA_SHA256,
    ATTACHMENT_TRANSLATION_REALTIME_CONTRACT_NAME_V1,
    ATTACHMENT_TRANSLATION_TICKET_CONTRACT_NAME_V1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn attachment_translation_command_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_TRANSLATION_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn attachment_translation_query_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_TRANSLATION_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn attachment_translation_realtime_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_TRANSLATION_REALTIME_CONTRACT_NAME_V1)
}

pub(crate) fn attachment_translation_ticket_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_TRANSLATION_TICKET_CONTRACT_NAME_V1)
}

pub(crate) fn attachment_translation_read_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ATTACHMENT_TRANSLATION_OWNER_V1.to_owned(),
        name: ATTACHMENT_TRANSLATION_READ_CONTRACT_NAME_V1.to_owned(),
        major: ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1,
        revision: ATTACHMENT_TRANSLATION_CONTRACT_REVISION_V1,
        schema_sha256: ATTACHMENT_TRANSLATION_READ_SCHEMA_SHA256.to_vec(),
    }
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ATTACHMENT_TRANSLATION_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1,
        revision: ATTACHMENT_TRANSLATION_CONTRACT_REVISION_V1,
        schema_sha256: ATTACHMENT_TRANSLATION_CONTROL_SCHEMA_SHA256.to_vec(),
    }
}
