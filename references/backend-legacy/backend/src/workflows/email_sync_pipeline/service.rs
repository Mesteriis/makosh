use crate::domains::communications::storage::port::{CommunicationAttachmentPort, LocalBlobPort};
use sqlx::postgres::PgPool;

use makosh_communications_api::email_sync::EmailSyncBatch;
use makosh_communications_api::evidence::CommunicationEvidencePort;

use super::errors::EmailSyncPipelineError;
use super::knowledge::project_message_knowledge;
use super::raw_records::project_raw_records;
use super::recording::record_email_sync_batch_with_mail_blobs;
use super::report::EmailSyncPipelineReport;

pub async fn project_email_sync_batch_with_mail_blobs(
    pool: PgPool,
    communication_evidence: &dyn CommunicationEvidencePort,
    blob_store: &LocalBlobPort,
    account_id: &str,
    import_batch_id: impl AsRef<str>,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncPipelineReport, EmailSyncPipelineError> {
    let mail_store = CommunicationAttachmentPort::new(pool.clone());
    let import_report = record_email_sync_batch_with_mail_blobs(
        communication_evidence,
        &mail_store,
        blob_store,
        account_id,
        import_batch_id.as_ref(),
        batch,
    )
    .await?;

    let projection_report =
        project_raw_records(&pool, &mail_store, blob_store, &import_report.raw_records).await?;
    let knowledge_report =
        project_message_knowledge(&pool, &projection_report.projected_messages).await?;
    Ok(EmailSyncPipelineReport {
        imported_records: import_report.inserted_or_existing_records,
        raw_blobs_upserted: import_report.blobs_upserted,
        projected_messages: projection_report.projected_messages.len(),
        attachment_blobs_upserted: projection_report.attachment_blobs_upserted,
        attachments_extracted: projection_report.attachments_extracted,
        attachments_not_scanned: projection_report.attachments_not_scanned,
        upserted_personas: knowledge_report.upserted_personas,
        upserted_persona_identities: knowledge_report.upserted_persona_identities,
        upserted_message_participants: knowledge_report.upserted_message_participants,
        upserted_relationship_events: knowledge_report.upserted_relationship_events,
        upserted_organizations: knowledge_report.upserted_organizations,
        upserted_organization_persona_links: knowledge_report.upserted_organization_persona_links,
        refreshed_decision_candidates: 0,
        refreshed_knowledge_candidates: 0,
        refreshed_task_candidates: 0,
        checkpoint_saved: import_report.checkpoint_saved,
    })
}
