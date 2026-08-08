use serde_json::Value;

use makosh_communications_api::accounts::ProviderAccount;
use makosh_communications_api::evidence::CommunicationEvidencePort;

use super::super::models::settings::MailSyncSettings;
use super::super::store::MailSyncStatePort;

pub(in crate::workflows::mail_background_sync) struct ProviderSyncContext<'a> {
    pub(in crate::workflows::mail_background_sync) store: &'a MailSyncStatePort,
    pub(in crate::workflows::mail_background_sync) communication_evidence:
        &'a dyn CommunicationEvidencePort,
    pub(in crate::workflows::mail_background_sync) account: &'a ProviderAccount,
    pub(in crate::workflows::mail_background_sync) run_id: &'a str,
    pub(in crate::workflows::mail_background_sync) settings: &'a MailSyncSettings,
    pub(in crate::workflows::mail_background_sync) checkpoint_before: Option<Value>,
}

#[derive(Clone, Copy)]
pub(in crate::workflows::mail_background_sync::provider) struct ImapAccountConfig<'a> {
    pub(in crate::workflows::mail_background_sync::provider) host: &'a str,
    pub(in crate::workflows::mail_background_sync::provider) port: u16,
    pub(in crate::workflows::mail_background_sync::provider) tls: bool,
    pub(in crate::workflows::mail_background_sync::provider) mailboxes: &'a [String],
}
