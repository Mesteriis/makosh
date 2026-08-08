use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::storage::port::LocalBlobPort;
use makosh_hub_backend::platform::communications::email_sync::imap_mailbox_stream_id;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::email_sync_pipeline::service::project_email_sync_batch_with_mail_blobs;

use crate::account::upsert_dev_provider_account;
use crate::checkpoint::last_seen_uid;
use crate::config::DevEmailSyncConfig;
use crate::errors::DevEmailSyncError;
use crate::fetch::fetch_raw_messages;
use crate::report::DevEmailSyncReport;

pub(super) async fn run_dev_email_sync(
    config: DevEmailSyncConfig,
) -> Result<DevEmailSyncReport, DevEmailSyncError> {
    let app_config = AppConfig::from_env()?;
    let database_url = app_config
        .database_url()
        .ok_or(DevEmailSyncError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(DevEmailSyncError::MissingDatabaseUrl)?
        .clone();

    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let provider_account_store = CommunicationProviderAccountStore::new(pool.clone());
    upsert_dev_provider_account(&provider_account_store, &config).await?;

    let stream_id = imap_mailbox_stream_id(&config.mailbox);
    let checkpoint_uid =
        last_seen_uid(&communication_store, &config.account_id, &stream_id).await?;
    let batch = fetch_raw_messages(&config, checkpoint_uid).await?;
    let fetched_messages = batch.messages.len();
    let checkpoint = batch.checkpoint.clone();
    let blob_store = LocalBlobPort::new(&config.blob_root);
    let pipeline = project_email_sync_batch_with_mail_blobs(
        pool,
        &communication_store,
        &blob_store,
        &config.account_id,
        &config.import_batch_id,
        &batch,
    )
    .await?;

    Ok(DevEmailSyncReport::new(
        &config,
        fetched_messages,
        checkpoint,
        pipeline,
    ))
}
