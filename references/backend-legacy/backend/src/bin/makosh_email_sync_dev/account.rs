use makosh_communications_api::accounts::NewProviderAccount;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use serde_json::json;

use crate::config::DevEmailSyncConfig;
use crate::errors::DevEmailSyncError;

pub(super) async fn upsert_dev_provider_account(
    store: &CommunicationProviderAccountStore,
    config: &DevEmailSyncConfig,
) -> Result<(), DevEmailSyncError> {
    let account = NewProviderAccount::new(
        &config.account_id,
        config.provider_kind,
        &config.display_name,
        &config.external_account_id,
    )
    .config(json!({
        "host": config.host,
        "port": config.port,
        "tls": config.tls,
        "mailbox": config.mailbox
    }));

    store.upsert(&account).await?;

    Ok(())
}
