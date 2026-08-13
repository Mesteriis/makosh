//! Typed Mail managed-runtime admission contract.

pub mod account_lifecycle;
pub mod address_book_fetch_worker;
mod address_book_provider;
pub mod admission;
pub mod attachment_anchor_mapping;
pub mod attachment_safety_projection;
pub mod attachment_security_outbox;
pub mod client_port;
pub mod communications_outbox;
pub mod delivery_intent_consumer;
pub mod delivery_intent_execution;
pub mod delivery_intent_outbox;
pub mod delivery_intent_result;
pub mod delivery_intent_worker;
pub mod gmail_oauth;
pub mod gmail_sync_worker;
pub mod managed;
pub mod person_source_fetch_worker;
pub mod person_source_producer;
pub mod retained_evidence_replay;
pub mod retained_evidence_replay_consumer;
pub mod retained_evidence_replay_result;
pub mod settings;
pub mod storage_bundle;

use makosh_mail_api::{
    GmailOAuthConfigurationV1, MailAccountConfigurationV1, MailAddressBookConfigurationV1,
};

#[derive(Clone)]
pub struct MailRuntimeAdmission {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub configuration_instance_id: String,
    pub module_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub vault_runtime_generation: u64,
    pub settings_revision: u64,
    pub account: MailAccountConfigurationV1,
    pub address_book: MailAddressBookConfigurationV1,
    pub gmail_oauth: Option<GmailOAuthConfigurationV1>,
}
