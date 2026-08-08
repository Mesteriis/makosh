use makosh_hub_backend::workflows::email_sync_pipeline::report::EmailSyncPipelineReport;
use serde::Serialize;

use crate::config::DevEmailSyncConfig;

#[derive(Serialize)]
pub(super) struct DevEmailSyncReport {
    account_id: String,
    provider: String,
    mailbox: String,
    fetched_messages: usize,
    blob_root: String,
    checkpoint: Option<serde_json::Value>,
    pipeline: EmailSyncPipelineReport,
}

impl DevEmailSyncReport {
    pub(super) fn new(
        config: &DevEmailSyncConfig,
        fetched_messages: usize,
        checkpoint: Option<serde_json::Value>,
        pipeline: EmailSyncPipelineReport,
    ) -> Self {
        Self {
            account_id: config.account_id.clone(),
            provider: config.provider_kind.as_str().to_owned(),
            mailbox: config.mailbox.clone(),
            fetched_messages,
            blob_root: config.blob_root.display().to_string(),
            checkpoint,
            pipeline,
        }
    }
}
