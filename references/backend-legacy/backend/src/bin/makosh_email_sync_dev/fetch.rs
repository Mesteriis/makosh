use makosh_communications_api::email_sync::EmailSyncBatch;
use makosh_hub_backend::integrations::mail::gmail::client::{
    imap::ImapNetworkClient, options::ImapFetchOptions,
};

use crate::config::DevEmailSyncConfig;
use crate::errors::DevEmailSyncError;

pub(super) async fn fetch_raw_messages(
    config: &DevEmailSyncConfig,
    last_seen_uid: Option<u32>,
) -> Result<EmailSyncBatch, DevEmailSyncError> {
    let mut fetch_options = ImapFetchOptions::new(
        &config.host,
        config.port,
        config.tls,
        config.mailbox.clone(),
        &config.username,
    )
    .provider_kind(config.provider_kind)
    .max_messages(config.max_messages)
    .latest_messages();

    if let Some(checkpoint) = last_seen_uid {
        fetch_options = fetch_options.last_seen_uid(checkpoint);
    }

    Ok(ImapNetworkClient::new()
        .fetch_raw_messages(&config.password, &fetch_options)
        .await?)
}
