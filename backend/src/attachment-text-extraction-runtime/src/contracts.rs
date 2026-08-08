use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1, ATTACHMENT_TEXT_EXTRACTION_CONTRACT_REVISION_V1,
    ATTACHMENT_TEXT_EXTRACTION_OWNER_V1, ATTACHMENT_TEXT_EXTRACTION_QUERY_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_REALTIME_CONTRACT_NAME_V1, ATTACHMENT_TEXT_EXTRACTION_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn command_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_TEXT_EXTRACTION_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn query_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_TEXT_EXTRACTION_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn content_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_TEXT_EXTRACTION_CONTENT_CONTRACT_NAME_V1)
}

pub(crate) fn realtime_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_TEXT_EXTRACTION_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ATTACHMENT_TEXT_EXTRACTION_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1,
        revision: ATTACHMENT_TEXT_EXTRACTION_CONTRACT_REVISION_V1,
        schema_sha256: ATTACHMENT_TEXT_EXTRACTION_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contracts_are_exact_and_share_one_schema() {
        let contracts = [
            command_contract_v1(),
            query_contract_v1(),
            content_contract_v1(),
            realtime_contract_v1(),
        ];
        assert!(contracts.iter().all(|contract| {
            contract.owner == ATTACHMENT_TEXT_EXTRACTION_OWNER_V1
                && contract.major == ATTACHMENT_TEXT_EXTRACTION_CONTRACT_MAJOR_V1
                && contract.revision == ATTACHMENT_TEXT_EXTRACTION_CONTRACT_REVISION_V1
                && contract.schema_sha256 == ATTACHMENT_TEXT_EXTRACTION_SCHEMA_SHA256
        }));
    }
}
