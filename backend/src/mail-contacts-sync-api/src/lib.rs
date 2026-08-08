#![forbid(unsafe_code)]

use makosh_runtime_protocol::v1::ContractReferenceV1;

pub const PACKAGE: &str = "makosh-mail-contacts-sync-api";
pub const MAIL_CONTACTS_SYNC_OWNER_ID_V1: &str = "mail_contacts_sync";
pub const MAIL_CONTACTS_SYNC_MODULE_ID_V1: &str = "makosh-mail-contacts-sync-runtime";
pub const MAIL_CONTACTS_SYNC_CAPABILITY_ID_V1: &str = "mail.contacts-sync.v1";
pub const MAIL_CONTACTS_SYNC_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.mail_contacts_sync.v1.MailContactsSyncCommandService/Start";
pub const MAIL_CONTACTS_SYNC_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.mail_contacts_sync.v1.MailContactsSyncQueryService/Get";
pub const MAIL_CONTACTS_SYNC_REALTIME_EVENT_KIND_V1: &str = "mail.contacts-sync.status-changed.v1";
pub const MAIL_CONTACTS_SYNC_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_CONTACTS_SYNC_CONTRACT_REVISION_V1: u32 = 1;
pub const MAIL_CONTACTS_SYNC_START_CONTRACT_NAME_V1: &str = "mail_contacts_sync_start";
pub const MAIL_CONTACTS_SYNC_QUERY_CONTRACT_NAME_V1: &str = "mail_contacts_sync_query";
pub const MAIL_CONTACTS_SYNC_REALTIME_CONTRACT_NAME_V1: &str = "mail_contacts_sync_realtime";

#[must_use]
pub fn mail_contacts_sync_start_contract_v1() -> ContractReferenceV1 {
    contract(MAIL_CONTACTS_SYNC_START_CONTRACT_NAME_V1)
}

#[must_use]
pub fn mail_contacts_sync_query_contract_v1() -> ContractReferenceV1 {
    contract(MAIL_CONTACTS_SYNC_QUERY_CONTRACT_NAME_V1)
}

#[must_use]
pub fn mail_contacts_sync_realtime_contract_v1() -> ContractReferenceV1 {
    contract(MAIL_CONTACTS_SYNC_REALTIME_CONTRACT_NAME_V1)
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: MAIL_CONTACTS_SYNC_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: MAIL_CONTACTS_SYNC_CONTRACT_MAJOR_V1,
        revision: MAIL_CONTACTS_SYNC_CONTRACT_REVISION_V1,
        schema_sha256: MAIL_CONTACTS_SYNC_SCHEMA_SHA256_V1.to_vec(),
    }
}

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail_contacts_sync.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/mail_contacts_sync_schema.rs"));

pub const MAIL_CONTACTS_SYNC_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/mail-contacts-sync-v1.bin"));

#[cfg(test)]
mod tests {
    #[test]
    fn client_surface_is_generated_start_get_and_realtime_without_polling_contract() {
        let source = include_str!("../proto/makosh/mail_contacts_sync/v1/sync.proto");
        assert!(source.contains("rpc Start"));
        assert!(source.contains("rpc Get"));
        assert!(source.contains("MailContactsSyncStatusChangedV1"));
        for forbidden in [
            "Poll",
            "provider_entry_id",
            "provider_etag",
            "credential",
            "map<",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden client surface: {forbidden}"
            );
        }
    }
}
