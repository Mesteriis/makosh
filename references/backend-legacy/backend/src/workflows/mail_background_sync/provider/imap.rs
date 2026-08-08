use serde_json::Value;

use crate::platform::communications::email_sync::{IMAP_ALL_MAILBOXES, imap_mailbox_stream_id};
use makosh_communications_api::mail_resources::{ImapMailboxListRequest, ImapMessageFetchRequest};

use super::super::errors::ProviderSyncError;
use super::super::models::progress::{MailSyncPhase, ProgressMode, ProgressUpdate};
use super::super::service::MailBackgroundSyncService;
use super::summary::ProviderSyncSummary;
use super::types::ImapAccountConfig;
use super::types::ProviderSyncContext;

impl MailBackgroundSyncService {
    pub(in crate::workflows::mail_background_sync::provider) async fn sync_imap(
        &self,
        context: ProviderSyncContext<'_>,
        config: ImapAccountConfig<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        let mut summary = ProviderSyncSummary::default();
        let mailboxes = self.resolve_imap_mailboxes(&context, config).await?;

        for mailbox in &mailboxes {
            self.sync_imap_mailbox(&context, config, mailbox, &mut summary)
                .await?;
        }

        Ok(summary)
    }

    async fn resolve_imap_mailboxes(
        &self,
        context: &ProviderSyncContext<'_>,
        config: ImapAccountConfig<'_>,
    ) -> Result<Vec<String>, ProviderSyncError> {
        if !config
            .mailboxes
            .iter()
            .any(|mailbox| mailbox == IMAP_ALL_MAILBOXES)
        {
            return Ok(config.mailboxes.to_vec());
        }

        Ok(self
            .provider_sync
            .list_imap_mailboxes(ImapMailboxListRequest {
                account_id: context.account.account_id.clone(),
                host: config.host.to_owned(),
                port: config.port,
                tls: config.tls,
                username: context.account.external_account_id.clone(),
            })
            .await?)
    }

    async fn sync_imap_mailbox(
        &self,
        context: &ProviderSyncContext<'_>,
        config: ImapAccountConfig<'_>,
        mailbox: &str,
        summary: &mut ProviderSyncSummary,
    ) -> Result<(), ProviderSyncError> {
        let stream_id = imap_mailbox_stream_id(mailbox);
        let checkpoint_before = context
            .communication_evidence
            .checkpoint(&context.account.account_id, &stream_id)
            .await?
            .map(|checkpoint| checkpoint.checkpoint);
        let mut last_seen_uid = checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("last_seen_uid"))
            .and_then(Value::as_u64)
            .and_then(|uid| u32::try_from(uid).ok());
        let checkpoint_uid_validity = checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("uid_validity"))
            .and_then(Value::as_u64)
            .and_then(|uid_validity| u32::try_from(uid_validity).ok());
        let mut retried_after_uid_validity_reset = false;

        loop {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Fetching,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let batch = self
                .provider_sync
                .fetch_imap_messages(ImapMessageFetchRequest {
                    account_id: context.account.account_id.clone(),
                    provider_kind: context.account.provider_kind,
                    host: config.host.to_owned(),
                    port: config.port,
                    tls: config.tls,
                    mailbox: mailbox.to_owned(),
                    username: context.account.external_account_id.clone(),
                    max_messages: context.settings.batch_size as usize,
                    last_seen_uid,
                })
                .await?;
            let fetched_count = batch.messages.len();
            let batch_uid_validity = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("uid_validity"))
                .and_then(Value::as_u64)
                .and_then(|uid_validity| u32::try_from(uid_validity).ok());
            if !retried_after_uid_validity_reset
                && checkpoint_uid_validity.is_some()
                && batch_uid_validity.is_some()
                && checkpoint_uid_validity != batch_uid_validity
            {
                retried_after_uid_validity_reset = true;
                last_seen_uid = None;
                continue;
            }

            last_seen_uid = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("last_seen_uid"))
                .and_then(Value::as_u64)
                .and_then(|uid| u32::try_from(uid).ok())
                .or(last_seen_uid);
            self.project_batch(
                context.store,
                context.run_id,
                context.settings,
                summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if fetched_count == 0 {
                break;
            }
        }

        Ok(())
    }
}
