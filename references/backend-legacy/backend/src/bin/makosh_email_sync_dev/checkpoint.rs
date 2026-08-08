use makosh_communications_postgres::store::CommunicationIngestionStore;

use crate::errors::DevEmailSyncError;

pub(super) async fn last_seen_uid(
    store: &CommunicationIngestionStore,
    account_id: &str,
    stream_id: &str,
) -> Result<Option<u32>, DevEmailSyncError> {
    let checkpoint = store.checkpoint(account_id, stream_id).await?;
    Ok(checkpoint
        .and_then(|checkpoint| checkpoint.checkpoint["last_seen_uid"].as_u64())
        .and_then(|last_seen_uid| u32::try_from(last_seen_uid).ok()))
}
