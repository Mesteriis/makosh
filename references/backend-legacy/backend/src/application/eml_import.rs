use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::projection::project_parsed_raw_email_message;
use crate::domains::communications::messages::store::MessageProjectionStore;
use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use crate::domains::communications::storage::errors::{
    AttachmentSafetyScanError, CommunicationStorageError,
};
use crate::domains::communications::storage::models::{
    CommunicationAttachmentDisposition, NewCommunicationAttachment, NewCommunicationBlob,
};
use crate::domains::communications::storage::scanner::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanner, HeuristicAttachmentSafetyScanner,
};
use crate::domains::communications::storage::store::CommunicationStorageStore;
use crate::platform::communications::mbox::{MboxParseError, split_mbox_messages};
use crate::platform::communications::rfc822::errors::EmailRfc822ParseError;
use crate::platform::communications::rfc822::models::ParsedEmailAttachmentDisposition;
use crate::platform::communications::rfc822::parser::parse_rfc822_message;
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use makosh_communications_api::evidence::StoredRawCommunicationRecord;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use serde_json::json;

#[derive(Clone)]
pub(crate) struct EmlImportService {
    ingestion_store: CommunicationIngestionStore,
    account_store: CommunicationProviderAccountStore,
    message_store: MessageProjectionStore,
    storage_store: CommunicationStorageStore,
    blob_store: LocalCommunicationBlobStore,
}

#[derive(Clone, Debug)]
pub(crate) struct EmlImportResult {
    pub message: ProjectedMessage,
    pub raw_record: StoredRawCommunicationRecord,
    pub attachment_count: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct MboxImportResult {
    pub imported: Vec<EmlImportResult>,
    pub failures: Vec<MboxImportFailure>,
}

#[derive(Clone, Debug)]
pub(crate) struct MboxImportFailure {
    pub message_index: usize,
    pub reason: &'static str,
}

pub(crate) const MAX_MBOX_IMPORT_MESSAGES: usize = 500;

impl EmlImportService {
    pub(crate) fn new(
        ingestion_store: CommunicationIngestionStore,
        account_store: CommunicationProviderAccountStore,
        message_store: MessageProjectionStore,
        storage_store: CommunicationStorageStore,
        blob_store: LocalCommunicationBlobStore,
    ) -> Self {
        Self {
            ingestion_store,
            account_store,
            message_store,
            storage_store,
            blob_store,
        }
    }

    pub(crate) async fn import_eml(
        &self,
        account_id: &str,
        raw_eml: &[u8],
    ) -> Result<EmlImportResult, EmlImportError> {
        self.ensure_mail_account(account_id).await?;

        let parsed = parse_rfc822_message(raw_eml)?;
        let raw_blob = self.blob_store.put_blob(raw_eml).await?;
        let fingerprint = raw_blob.sha256.clone();
        let digest = fingerprint
            .strip_prefix("sha256:")
            .unwrap_or(fingerprint.as_str());
        let provider_record_id = format!("eml-import:{digest}");
        let raw_record = self
            .ingestion_store
            .record_raw_source(
                &NewRawCommunicationRecord::new(
                    format!("raw:eml-import:{account_id}:{digest}"),
                    account_id,
                    "email_message",
                    &provider_record_id,
                    &fingerprint,
                    format!("eml-import:{digest}"),
                    json!({
                        "subject": parsed.subject,
                        "from": parsed.from,
                        "to": parsed.to,
                        "body_text": parsed.body_text,
                        "body_html": parsed.body_html,
                        "provider": "eml_import",
                        "transport": "local_import",
                        "raw_blob_storage_kind": raw_blob.storage_kind,
                        "raw_blob_storage_path": raw_blob.storage_path,
                        "raw_blob_sha256": fingerprint,
                        "rfc822_size": raw_blob.size_bytes,
                        "is_read": true
                    }),
                )
                .provenance(json!({
                    "source": "eml_import",
                    "format": "eml",
                    "content_sha256": raw_blob.sha256,
                })),
            )
            .await?;
        let message =
            project_parsed_raw_email_message(&self.message_store, &raw_record, &parsed).await?;

        let scanner = HeuristicAttachmentSafetyScanner;
        for attachment in &parsed.attachments {
            let local_blob = self.blob_store.put_blob(&attachment.body_bytes).await?;
            let blob = self
                .storage_store
                .upsert_blob(
                    &NewCommunicationBlob::from_local_blob(&local_blob)
                        .content_type(&attachment.content_type),
                )
                .await?;
            let scan_report = scanner.scan(&AttachmentSafetyScanRequest {
                provider_attachment_id: &attachment.provider_attachment_id,
                filename: attachment.filename.as_deref(),
                content_type: &attachment.content_type,
                size_bytes: local_blob.size_bytes,
                sha256: &blob.sha256,
                storage_kind: &blob.storage_kind,
                storage_path: &blob.storage_path,
                bytes: &attachment.body_bytes,
            })?;
            let mut new_attachment = NewCommunicationAttachment::new(
                &message.message_id,
                &raw_record.raw_record_id,
                &blob.blob_id,
                &attachment.provider_attachment_id,
                &attachment.content_type,
                local_blob.size_bytes,
                &blob.sha256,
            )
            .disposition(import_attachment_disposition(attachment.disposition))
            .scan_report(scan_report);
            if let Some(filename) = &attachment.filename {
                new_attachment = new_attachment.filename(filename);
            }
            self.storage_store
                .upsert_attachment(&new_attachment)
                .await?;
        }

        Ok(EmlImportResult {
            message,
            raw_record,
            attachment_count: parsed.attachments.len(),
        })
    }

    pub(crate) async fn import_mbox(
        &self,
        account_id: &str,
        source: &[u8],
    ) -> Result<MboxImportResult, EmlImportError> {
        let messages = split_mbox_messages(source, MAX_MBOX_IMPORT_MESSAGES)?;
        self.ensure_mail_account(account_id).await?;
        let mut imported = Vec::with_capacity(messages.len());
        let mut failures = Vec::new();
        for (message_index, message) in messages.into_iter().enumerate() {
            match self.import_eml(account_id, &message).await {
                Ok(result) => imported.push(result),
                Err(error) => failures.push(MboxImportFailure {
                    message_index,
                    reason: mbox_failure_reason(&error),
                }),
            }
        }
        Ok(MboxImportResult { imported, failures })
    }

    async fn ensure_mail_account(&self, account_id: &str) -> Result<(), EmlImportError> {
        let account = self
            .account_store
            .get(account_id)
            .await?
            .ok_or(EmlImportError::AccountNotFound)?;
        if !account.provider_kind.is_email() {
            return Err(EmlImportError::UnsupportedAccountKind);
        }
        Ok(())
    }
}

fn mbox_failure_reason(error: &EmlImportError) -> &'static str {
    match error {
        EmlImportError::Rfc822(_) | EmlImportError::Mbox(_) => "invalid_message",
        EmlImportError::Scan(_) => "attachment_scan_failed",
        EmlImportError::Storage(_) => "attachment_storage_failed",
        EmlImportError::Ingestion(_) | EmlImportError::MessageProjection(_) => {
            "message_recording_failed"
        }
        EmlImportError::AccountNotFound | EmlImportError::UnsupportedAccountKind => {
            "account_unavailable"
        }
    }
}

fn import_attachment_disposition(
    disposition: ParsedEmailAttachmentDisposition,
) -> CommunicationAttachmentDisposition {
    match disposition {
        ParsedEmailAttachmentDisposition::Attachment => {
            CommunicationAttachmentDisposition::Attachment
        }
        ParsedEmailAttachmentDisposition::Inline => CommunicationAttachmentDisposition::Inline,
        ParsedEmailAttachmentDisposition::Unknown => CommunicationAttachmentDisposition::Unknown,
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum EmlImportError {
    #[error("mail account was not found")]
    AccountNotFound,
    #[error("EML imports require a mail account")]
    UnsupportedAccountKind,
    #[error(transparent)]
    Rfc822(#[from] EmailRfc822ParseError),
    #[error(transparent)]
    Mbox(#[from] MboxParseError),
    #[error(transparent)]
    Ingestion(#[from] makosh_communications_postgres::errors::CommunicationIngestionError),
    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),
    #[error(transparent)]
    Storage(#[from] CommunicationStorageError),
    #[error(transparent)]
    Scan(#[from] AttachmentSafetyScanError),
}
