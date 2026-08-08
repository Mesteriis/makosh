use crate::domains::communications::storage::port::{CommunicationAttachmentPort, LocalBlobPort};
use makosh_communications_api::evidence::{
    CommunicationEvidencePort, NewIngestionCheckpoint, NewRawCommunicationRecord,
};
use serde_json::json;

use super::errors::EmailSyncRecordError;
use super::ids::{EMAIL_MESSAGE_RECORD_KIND, raw_record_id};
use super::raw_payload::{payload_with_raw_blob_reference, raw_message_bytes};
use crate::domains::communications::storage::models::NewCommunicationBlob;
use makosh_communications_api::email_sync::EmailSyncBatch;
use makosh_communications_api::email_sync::{EmailSyncBlobImportReport, EmailSyncImportReport};

pub async fn record_email_sync_batch(
    store: &dyn CommunicationEvidencePort,
    account_id: &str,
    import_batch_id: &str,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncImportReport, EmailSyncRecordError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let import_batch_id = validate_non_empty("import_batch_id", import_batch_id)?;
    validate_non_empty("stream_id", &batch.stream_id)?;

    let mut inserted_or_existing_records = 0;
    for message in &batch.messages {
        let mut raw_record = NewRawCommunicationRecord::new(
            raw_record_id(
                &account_id,
                EMAIL_MESSAGE_RECORD_KIND,
                &message.provider_record_id,
                &message.source_fingerprint,
                &message.payload,
            ),
            &account_id,
            EMAIL_MESSAGE_RECORD_KIND,
            &message.provider_record_id,
            &message.source_fingerprint,
            &import_batch_id,
            message.payload.clone(),
        )
        .provenance(json!({
            "source": "email_provider_sync",
            "provider": batch.provider_kind.as_str(),
            "stream_id": batch.stream_id
        }));

        if let Some(occurred_at) = message.occurred_at {
            raw_record = raw_record.occurred_at(occurred_at);
        }

        store.record_raw_source(&raw_record).await?;
        inserted_or_existing_records += 1;
    }

    let checkpoint_saved = save_checkpoint_if_present(store, &account_id, batch).await?;

    Ok(EmailSyncImportReport {
        inserted_or_existing_records,
        checkpoint_saved,
    })
}

pub async fn record_email_sync_batch_with_mail_blobs(
    store: &dyn CommunicationEvidencePort,
    mail_store: &CommunicationAttachmentPort,
    blob_store: &LocalBlobPort,
    account_id: &str,
    import_batch_id: &str,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncBlobImportReport, EmailSyncRecordError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let import_batch_id = validate_non_empty("import_batch_id", import_batch_id)?;
    validate_non_empty("stream_id", &batch.stream_id)?;

    let mut inserted_or_existing_records = 0;
    let mut blobs_upserted = 0;
    let mut raw_records = Vec::new();
    for message in &batch.messages {
        let raw_bytes = raw_message_bytes(batch.provider_kind, &message.payload)?;
        let local_blob = blob_store.put_blob(&raw_bytes).await?;
        let stored_blob = mail_store
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob).content_type("message/rfc822"),
            )
            .await?;
        let payload = payload_with_raw_blob_reference(&message.payload, &stored_blob)?;

        let mut raw_record = NewRawCommunicationRecord::new(
            raw_record_id(
                &account_id,
                EMAIL_MESSAGE_RECORD_KIND,
                &message.provider_record_id,
                &message.source_fingerprint,
                &payload,
            ),
            &account_id,
            EMAIL_MESSAGE_RECORD_KIND,
            &message.provider_record_id,
            &message.source_fingerprint,
            &import_batch_id,
            payload,
        )
        .provenance(json!({
            "source": "email_provider_sync",
            "provider": batch.provider_kind.as_str(),
            "stream_id": batch.stream_id,
            "raw_storage": stored_blob.storage_kind
        }));

        if let Some(occurred_at) = message.occurred_at {
            raw_record = raw_record.occurred_at(occurred_at);
        }

        raw_records.push(store.record_raw_source(&raw_record).await?);
        inserted_or_existing_records += 1;
        blobs_upserted += 1;
    }

    let checkpoint_saved = save_checkpoint_if_present(store, &account_id, batch).await?;

    Ok(EmailSyncBlobImportReport {
        inserted_or_existing_records,
        checkpoint_saved,
        blobs_upserted,
        raw_records,
    })
}

async fn save_checkpoint_if_present(
    store: &dyn CommunicationEvidencePort,
    account_id: &str,
    batch: &EmailSyncBatch,
) -> Result<bool, EmailSyncRecordError> {
    if let Some(checkpoint) = &batch.checkpoint {
        store
            .save_checkpoint(&NewIngestionCheckpoint::new(
                account_id,
                &batch.stream_id,
                checkpoint.clone(),
            ))
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, EmailSyncRecordError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EmailSyncRecordError::EmptyField(field));
    }
    Ok(value.to_owned())
}
