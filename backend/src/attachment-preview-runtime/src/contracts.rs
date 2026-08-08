use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_COMMAND_CONTRACT_NAME_V1, ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1,
    ATTACHMENT_PREVIEW_CONTRACT_REVISION_V1, ATTACHMENT_PREVIEW_CONTROL_SCHEMA_SHA256,
    ATTACHMENT_PREVIEW_OWNER_V1, ATTACHMENT_PREVIEW_QUERY_CONTRACT_NAME_V1,
    ATTACHMENT_PREVIEW_READ_CONTRACT_NAME_V1, ATTACHMENT_PREVIEW_READ_SCHEMA_SHA256,
    ATTACHMENT_PREVIEW_TICKET_CONTRACT_NAME_V1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn command_contract_v1() -> ContractReferenceV1 {
    control_contract(ATTACHMENT_PREVIEW_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn query_contract_v1() -> ContractReferenceV1 {
    control_contract(ATTACHMENT_PREVIEW_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn ticket_contract_v1() -> ContractReferenceV1 {
    control_contract(ATTACHMENT_PREVIEW_TICKET_CONTRACT_NAME_V1)
}

pub(crate) fn realtime_contract_v1() -> ContractReferenceV1 {
    control_contract(makosh_attachment_preview_api::ATTACHMENT_PREVIEW_REALTIME_CONTRACT_NAME_V1)
}

pub(crate) fn read_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ATTACHMENT_PREVIEW_OWNER_V1.to_owned(),
        name: ATTACHMENT_PREVIEW_READ_CONTRACT_NAME_V1.to_owned(),
        major: ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1,
        revision: ATTACHMENT_PREVIEW_CONTRACT_REVISION_V1,
        schema_sha256: ATTACHMENT_PREVIEW_READ_SCHEMA_SHA256.to_vec(),
    }
}

fn control_contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ATTACHMENT_PREVIEW_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1,
        revision: ATTACHMENT_PREVIEW_CONTRACT_REVISION_V1,
        schema_sha256: ATTACHMENT_PREVIEW_CONTROL_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_read_contract_has_a_distinct_schema() {
        assert_eq!(
            command_contract_v1().schema_sha256,
            query_contract_v1().schema_sha256
        );
        assert_eq!(
            command_contract_v1().schema_sha256,
            ticket_contract_v1().schema_sha256
        );
        assert_ne!(
            command_contract_v1().schema_sha256,
            read_contract_v1().schema_sha256
        );
    }
}
