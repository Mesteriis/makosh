use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::storage::models::{
    CommunicationAttachmentDisposition, NewCommunicationAttachment, NewCommunicationBlob,
};
use crate::domains::communications::storage::port::{CommunicationAttachmentPort, LocalBlobPort};
use crate::domains::communications::storage::scanner::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanStatus, scan_attachment_with_configured_clamav,
};
use crate::platform::communications::rfc822::models::{
    ParsedEmailAttachment, ParsedEmailAttachmentDisposition,
};
use makosh_communications_api::evidence::StoredRawCommunicationRecord;

use super::errors::EmailSyncPipelineError;

#[derive(Default)]
pub(crate) struct AttachmentProjectionReport {
    pub(crate) attachment_blobs_upserted: usize,
    pub(crate) attachments_extracted: usize,
    pub(crate) attachments_not_scanned: usize,
}

pub(crate) async fn project_attachments(
    mail_store: &CommunicationAttachmentPort,
    blob_store: &LocalBlobPort,
    raw_record: &StoredRawCommunicationRecord,
    message: &ProjectedMessage,
    attachments: &[ParsedEmailAttachment],
) -> Result<AttachmentProjectionReport, EmailSyncPipelineError> {
    let mut report = AttachmentProjectionReport::default();

    for parsed_attachment in attachments {
        let local_blob = blob_store.put_blob(&parsed_attachment.body_bytes).await?;
        let blob = mail_store
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob)
                    .content_type(&parsed_attachment.content_type),
            )
            .await?;
        let scan_report = scan_attachment_with_configured_clamav(&AttachmentSafetyScanRequest {
            provider_attachment_id: &parsed_attachment.provider_attachment_id,
            filename: parsed_attachment.filename.as_deref(),
            content_type: &parsed_attachment.content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &blob.sha256,
            storage_kind: &blob.storage_kind,
            storage_path: &blob.storage_path,
            bytes: &parsed_attachment.body_bytes,
        })
        .await?;
        let scan_status = scan_report.status;

        let mut attachment = NewCommunicationAttachment::new(
            &message.message_id,
            &raw_record.raw_record_id,
            &blob.blob_id,
            &parsed_attachment.provider_attachment_id,
            &parsed_attachment.content_type,
            local_blob.size_bytes,
            &blob.sha256,
        )
        .disposition(mail_attachment_disposition(parsed_attachment.disposition))
        .scan_report(scan_report);

        if let Some(filename) = &parsed_attachment.filename {
            attachment = attachment.filename(filename);
        }

        mail_store.upsert_attachment(&attachment).await?;
        report.attachment_blobs_upserted += 1;
        report.attachments_extracted += 1;
        if scan_status == AttachmentSafetyScanStatus::NotScanned {
            report.attachments_not_scanned += 1;
        }
    }

    Ok(report)
}

fn mail_attachment_disposition(
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
