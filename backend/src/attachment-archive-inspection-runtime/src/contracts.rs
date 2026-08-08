use makosh_attachment_archive_inspection_api::{
    ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONTRACT_NAME_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_MAJOR_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_REVISION_V1, ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONTRACT_NAME_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_CONTRACT_NAME_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_SCHEMA_SHA256,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;

pub(crate) fn archive_inspection_command_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONTRACT_NAME_V1)
}

pub(crate) fn archive_inspection_query_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONTRACT_NAME_V1)
}

pub(crate) fn archive_inspection_realtime_contract_v1() -> ContractReferenceV1 {
    contract(ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_MAJOR_V1,
        revision: ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_REVISION_V1,
        schema_sha256: ATTACHMENT_ARCHIVE_INSPECTION_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contracts_share_exact_owner_schema_and_version() {
        let command = archive_inspection_command_contract_v1();
        let query = archive_inspection_query_contract_v1();
        let realtime = archive_inspection_realtime_contract_v1();
        assert_eq!(command.owner, ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1);
        assert_eq!(
            command.major,
            ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_MAJOR_V1
        );
        assert_eq!(
            command.revision,
            ATTACHMENT_ARCHIVE_INSPECTION_CONTRACT_REVISION_V1
        );
        assert_eq!(command.schema_sha256, query.schema_sha256);
        assert_eq!(command.schema_sha256, realtime.schema_sha256);
        assert_ne!(command.name, query.name);
        assert_ne!(query.name, realtime.name);
    }
}
