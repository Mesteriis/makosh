use serde_json::Value;

use makosh_communications_api::mail_resources::GmailMessageListFetchRequest;

use super::super::super::errors::ProviderSyncError;
use super::super::super::models::progress::{MailSyncPhase, ProgressMode, ProgressUpdate};
use super::super::super::service::MailBackgroundSyncService;
use super::super::summary::ProviderSyncSummary;
use super::super::types::ProviderSyncContext;

impl MailBackgroundSyncService {
    pub(in crate::workflows::mail_background_sync::provider::gmail) async fn sync_gmail_message_list_pages(
        &self,
        context: &ProviderSyncContext<'_>,
        summary: &mut ProviderSyncSummary,
        mut page_token: Option<String>,
    ) -> Result<(), ProviderSyncError> {
        loop {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Listing,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let batch = self
                .provider_sync
                .fetch_gmail_message_list(GmailMessageListFetchRequest {
                    account_id: context.account.account_id.clone(),
                    max_results: context.settings.batch_size as u16,
                    page_token,
                })
                .await?;
            page_token = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("next_page_token"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let fetched_count = batch.messages.len();
            self.project_batch(
                context.store,
                context.run_id,
                context.settings,
                summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if page_token.is_none() || fetched_count == 0 {
                break;
            }
        }

        Ok(())
    }
}
