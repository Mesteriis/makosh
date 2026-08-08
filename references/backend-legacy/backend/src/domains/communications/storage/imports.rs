use super::errors::CommunicationStorageError;
use super::ids::communication_attachment_import_id;
use super::models::{
    ImportedCommunicationAttachment, ImportedCommunicationAttachmentRemovalResult,
    NewCommunicationAttachmentImport,
};
use super::rows::{row_to_imported_attachment, row_to_mail_blob};
use super::store::CommunicationStorageStore;
use super::validation::validate_non_empty;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::platform::storage::communication_media::{
    ImportedAttachmentRecord, ImportedAttachmentRemovalResult, ImportedAttachmentStoragePort,
    ImportedAttachmentUpsert, LocalBlobRecord, SafetyScanReport, SafetyScanStatus,
    StoredBlobRecord,
};
use crate::platform::storage::errors::StorageError;
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::store::EventStore;

impl CommunicationStorageStore {
    pub async fn upsert_imported_attachment(
        &self,
        import: &NewCommunicationAttachmentImport,
    ) -> Result<ImportedCommunicationAttachment, CommunicationStorageError> {
        self.upsert_imported_attachment_with_observation(import, None, "attachment_import", None)
            .await
    }

    pub async fn upsert_imported_attachment_with_observation(
        &self,
        import: &NewCommunicationAttachmentImport,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ImportedCommunicationAttachment, CommunicationStorageError> {
        let import = import.validate()?;
        let mut transaction = self.pool.begin().await?;
        sqlx::query(imported_attachment_upsert_sql())
            .bind(&import.attachment_id)
            .bind(&import.account_id)
            .bind(&import.channel_kind)
            .bind(&import.blob_id)
            .bind(&import.filename)
            .bind(&import.content_type)
            .bind(import.size_bytes)
            .bind(&import.sha256)
            .bind(&import.source_kind)
            .bind(&import.imported_by)
            .bind(import.scan_report.status.as_str())
            .bind(&import.scan_report.engine)
            .bind(import.scan_report.checked_at)
            .bind(&import.scan_report.summary)
            .bind(&import.scan_report.metadata)
            .bind(&import.metadata)
            .execute(&mut *transaction)
            .await?;

        let sql = imported_attachment_select_sql("i.attachment_id = $1");
        let row = sqlx::query(&sql)
            .bind(&import.attachment_id)
            .fetch_optional(&mut *transaction)
            .await?;
        let imported = row
            .map(row_to_imported_attachment)
            .transpose()?
            .ok_or_else(|| CommunicationStorageError::Sqlx(sqlx::Error::RowNotFound))?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "attachment_import",
                imported.attachment_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "blob_id": imported.blob_id,
                    "scan_status": imported.scan_status.as_str(),
                    "content_type": imported.content_type,
                    "sha256": imported.sha256,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(imported)
    }

    pub async fn imported_attachment_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<Option<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let sql = imported_attachment_select_sql("i.attachment_id = $1");
        let row = sqlx::query(&sql)
            .bind(attachment_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_imported_attachment).transpose()
    }

    pub async fn imported_attachment_by_blob_id(
        &self,
        blob_id: &str,
    ) -> Result<Option<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let blob_id = validate_non_empty("blob_id", blob_id)?;
        let sql = imported_attachment_select_sql("i.blob_id = $1");
        let row = sqlx::query(&sql)
            .bind(blob_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_imported_attachment).transpose()
    }

    pub async fn list_imported_attachments(
        &self,
        account_id: &str,
        source_kind: &str,
        limit: i64,
    ) -> Result<Vec<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let source_kind = validate_non_empty("source_kind", source_kind)?;
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query(
            r#"
            SELECT
                i.attachment_id,
                i.account_id,
                i.channel_kind,
                i.blob_id,
                i.filename,
                i.content_type,
                i.size_bytes,
                i.sha256,
                i.source_kind,
                i.imported_by,
                i.scan_status,
                i.scan_engine,
                i.scan_checked_at,
                i.scan_summary,
                i.scan_metadata,
                i.metadata,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path,
                i.created_at,
                i.updated_at
            FROM communication_attachment_imports i
            JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
            WHERE i.account_id = $1
              AND i.source_kind = $2
            ORDER BY i.created_at DESC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(source_kind)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_imported_attachment).collect()
    }

    pub async fn list_not_scanned_imported_attachments(
        &self,
        limit: i64,
    ) -> Result<Vec<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let rows = sqlx::query(
            r#"
            SELECT
                i.attachment_id,
                i.account_id,
                i.channel_kind,
                i.blob_id,
                i.filename,
                i.content_type,
                i.size_bytes,
                i.sha256,
                i.source_kind,
                i.imported_by,
                i.scan_status,
                i.scan_engine,
                i.scan_checked_at,
                i.scan_summary,
                i.scan_metadata,
                i.metadata,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path,
                i.created_at,
                i.updated_at
            FROM communication_attachment_imports i
            JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
            WHERE i.scan_status = 'not_scanned'
            ORDER BY i.created_at ASC, i.attachment_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_imported_attachment).collect()
    }

    /// Stores a retry verdict only while the import still references the scanned immutable blob.
    pub async fn persist_not_scanned_imported_attachment_verdict(
        &self,
        attachment_id: &str,
        expected_sha256: &str,
        report: &super::scanner::AttachmentSafetyScanReport,
    ) -> Result<Option<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let expected_sha256 = validate_non_empty("expected_sha256", expected_sha256)?;
        let report = report.validate()?;
        let mut transaction = self.pool.begin().await?;
        let updated = sqlx::query_scalar::<_, String>(
            r#"
            UPDATE communication_attachment_imports
            SET
                scan_status = $3,
                scan_engine = $4,
                scan_checked_at = $5,
                scan_summary = $6,
                scan_metadata = $7,
                updated_at = now()
            WHERE attachment_id = $1
              AND sha256 = $2
              AND scan_status = 'not_scanned'
            RETURNING attachment_id
            "#,
        )
        .bind(&attachment_id)
        .bind(&expected_sha256)
        .bind(report.status.as_str())
        .bind(&report.engine)
        .bind(report.checked_at)
        .bind(&report.summary)
        .bind(&report.metadata)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(updated) = updated else {
            transaction.commit().await?;
            return Ok(None);
        };
        let row = sqlx::query(&imported_attachment_select_sql("i.attachment_id = $1"))
            .bind(&updated)
            .fetch_one(&mut *transaction)
            .await?;
        let imported = row_to_imported_attachment(row)?;
        let occurred_at = Utc::now();
        let event = NewEventEnvelope::builder(
            format!(
                "communication_attachment_processing:{}:{}:{}",
                imported.attachment_id,
                imported.scan_status.as_str(),
                occurred_at.timestamp_micros()
            ),
            "communication.attachment.processing_changed.v1",
            occurred_at,
            serde_json::json!({ "kind": "communication_attachment_import" }),
            serde_json::json!({
                "kind": "communication_attachment_import",
                "id": imported.attachment_id,
            }),
        )
        .actor(serde_json::json!({ "actor_id": "makosh-attachment-scanner" }))
        .payload(serde_json::json!({
            "attachment_id": imported.attachment_id,
            "previous_scan_status": "not_scanned",
            "scan_status": imported.scan_status.as_str(),
            "scan_engine": imported.scan_engine,
            "scan_checked_at": imported.scan_checked_at,
        }))
        .provenance(serde_json::json!({
            "source_kind": "communication_attachment_import_rescan",
            "source_id": imported.attachment_id,
        }))
        .build()?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;
        Ok(Some(imported))
    }

    pub async fn list_expired_imported_attachments(
        &self,
        account_id: &str,
        source_kind: &str,
        limit: i64,
    ) -> Result<Vec<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let source_kind = validate_non_empty("source_kind", source_kind)?;
        let limit = limit.clamp(1, 500);
        let rows = sqlx::query(
            r#"
            SELECT
                i.attachment_id,
                i.account_id,
                i.channel_kind,
                i.blob_id,
                i.filename,
                i.content_type,
                i.size_bytes,
                i.sha256,
                i.source_kind,
                i.imported_by,
                i.scan_status,
                i.scan_engine,
                i.scan_checked_at,
                i.scan_summary,
                i.scan_metadata,
                i.metadata,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path,
                i.created_at,
                i.updated_at
            FROM communication_attachment_imports i
            JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
            WHERE i.account_id = $1
              AND i.source_kind = $2
              AND NULLIF(i.metadata -> 'retention_policy' ->> 'expires_at', '') IS NOT NULL
              AND (i.metadata -> 'retention_policy' ->> 'expires_at')::timestamptz <= now()
            ORDER BY (i.metadata -> 'retention_policy' ->> 'expires_at')::timestamptz ASC,
                     i.created_at ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(source_kind)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_imported_attachment).collect()
    }

    pub async fn blob_by_id(
        &self,
        blob_id: &str,
    ) -> Result<Option<super::models::StoredCommunicationBlob>, CommunicationStorageError> {
        let blob_id = validate_non_empty("blob_id", blob_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type,
                created_at
            FROM communication_mail_blobs
            WHERE blob_id = $1
            "#,
        )
        .bind(blob_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_mail_blob).transpose()
    }

    pub async fn remove_imported_attachment(
        &self,
        attachment_id: &str,
        account_id: &str,
        source_kind: &str,
    ) -> Result<Option<ImportedCommunicationAttachmentRemovalResult>, CommunicationStorageError>
    {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let account_id = validate_non_empty("account_id", account_id)?;
        let source_kind = validate_non_empty("source_kind", source_kind)?;
        let mut transaction = self.pool.begin().await?;
        let sql = imported_attachment_select_sql(
            "i.attachment_id = $1 AND i.account_id = $2 AND i.source_kind = $3",
        );
        let imported = sqlx::query(&sql)
            .bind(&attachment_id)
            .bind(&account_id)
            .bind(&source_kind)
            .fetch_optional(&mut *transaction)
            .await?
            .map(row_to_imported_attachment)
            .transpose()?;
        let Some(imported_attachment) = imported else {
            transaction.commit().await?;
            return Ok(None);
        };

        sqlx::query(
            r#"
            DELETE FROM communication_attachment_imports
            WHERE attachment_id = $1
              AND account_id = $2
              AND source_kind = $3
            "#,
        )
        .bind(&attachment_id)
        .bind(&account_id)
        .bind(&source_kind)
        .execute(&mut *transaction)
        .await?;

        let blob_still_referenced = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM communication_attachment_imports
                WHERE blob_id = $1
            ) OR EXISTS(
                SELECT 1
                FROM communication_attachments
                WHERE blob_id = $1
            )
            "#,
        )
        .bind(&imported_attachment.blob_id)
        .fetch_one(&mut *transaction)
        .await?;

        let blob_metadata_removed = if blob_still_referenced {
            false
        } else {
            sqlx::query(
                r#"
                DELETE FROM communication_mail_blobs
                WHERE blob_id = $1
                "#,
            )
            .bind(&imported_attachment.blob_id)
            .execute(&mut *transaction)
            .await?;
            true
        };

        transaction.commit().await?;
        Ok(Some(ImportedCommunicationAttachmentRemovalResult {
            imported_attachment,
            blob_metadata_removed,
        }))
    }
}

pub fn new_communication_attachment_import_id(seed: &str) -> String {
    communication_attachment_import_id(seed)
}

fn imported_attachment_upsert_sql() -> &'static str {
    r#"
    INSERT INTO communication_attachment_imports (
        attachment_id,
        account_id,
        channel_kind,
        blob_id,
        filename,
        content_type,
        size_bytes,
        sha256,
        source_kind,
        imported_by,
        scan_status,
        scan_engine,
        scan_checked_at,
        scan_summary,
        scan_metadata,
        metadata,
        updated_at
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, now())
    ON CONFLICT (attachment_id)
    DO UPDATE SET
        account_id = EXCLUDED.account_id,
        channel_kind = EXCLUDED.channel_kind,
        blob_id = EXCLUDED.blob_id,
        filename = EXCLUDED.filename,
        content_type = EXCLUDED.content_type,
        size_bytes = EXCLUDED.size_bytes,
        sha256 = EXCLUDED.sha256,
        source_kind = EXCLUDED.source_kind,
        imported_by = EXCLUDED.imported_by,
        scan_status = EXCLUDED.scan_status,
        scan_engine = EXCLUDED.scan_engine,
        scan_checked_at = EXCLUDED.scan_checked_at,
        scan_summary = EXCLUDED.scan_summary,
        scan_metadata = EXCLUDED.scan_metadata,
        metadata = EXCLUDED.metadata,
        updated_at = now()
    "#
}

impl ImportedAttachmentStoragePort for CommunicationStorageStore {
    fn upsert_blob_record<'a>(
        &'a self,
        blob: &'a LocalBlobRecord,
        content_type: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<StoredBlobRecord, StorageError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let stored = self
                .upsert_blob(
                    &super::models::NewCommunicationBlob::new(
                        &blob.storage_kind,
                        &blob.storage_path,
                        &blob.sha256,
                        blob.size_bytes,
                    )
                    .content_type(content_type),
                )
                .await
                .map_err(storage_error_from_communication)?;
            Ok(StoredBlobRecord {
                blob_id: stored.blob_id,
            })
        })
    }

    fn upsert_imported_attachment_record<'a>(
        &'a self,
        import: &'a ImportedAttachmentUpsert,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<ImportedAttachmentRecord, StorageError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let mut upsert = NewCommunicationAttachmentImport::new(
                &import.attachment_id,
                &import.blob_id,
                &import.content_type,
                import.size_bytes,
                &import.sha256,
                &import.imported_by,
            )
            .account_id(import.account_id.clone())
            .channel_kind(import.channel_kind.clone())
            .source_kind(import.source_kind.clone())
            .scan_report(scan_report_to_domain(&import.scan_report)?)
            .metadata(import.metadata.clone());
            if let Some(filename) = &import.filename {
                upsert = upsert.filename(filename.clone());
            }
            let stored = self
                .upsert_imported_attachment(&upsert)
                .await
                .map_err(storage_error_from_communication)?;
            imported_attachment_to_platform(stored)
        })
    }

    fn list_imported_attachment_records<'a>(
        &'a self,
        account_id: &'a str,
        source_kind: &'a str,
        limit: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ImportedAttachmentRecord>, StorageError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let items = self
                .list_imported_attachments(account_id, source_kind, limit)
                .await
                .map_err(storage_error_from_communication)?;
            items
                .into_iter()
                .map(imported_attachment_to_platform)
                .collect()
        })
    }

    fn list_expired_imported_attachment_records<'a>(
        &'a self,
        account_id: &'a str,
        source_kind: &'a str,
        limit: i64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ImportedAttachmentRecord>, StorageError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let items = self
                .list_expired_imported_attachments(account_id, source_kind, limit)
                .await
                .map_err(storage_error_from_communication)?;
            items
                .into_iter()
                .map(imported_attachment_to_platform)
                .collect()
        })
    }

    fn remove_imported_attachment_record<'a>(
        &'a self,
        attachment_id: &'a str,
        account_id: &'a str,
        source_kind: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<ImportedAttachmentRemovalResult>, StorageError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let removed = self
                .remove_imported_attachment(attachment_id, account_id, source_kind)
                .await
                .map_err(storage_error_from_communication)?;
            removed
                .map(|item| {
                    Ok(ImportedAttachmentRemovalResult {
                        imported_attachment: imported_attachment_to_platform(
                            item.imported_attachment,
                        )?,
                        blob_metadata_removed: item.blob_metadata_removed,
                    })
                })
                .transpose()
        })
    }
}

fn storage_error_from_communication(error: CommunicationStorageError) -> StorageError {
    match error {
        CommunicationStorageError::Sqlx(err) => StorageError::Connect(err),
        CommunicationStorageError::ObservationStore(err) => StorageError::Invalid(err.to_string()),
        CommunicationStorageError::Io(err) => StorageError::Io(err),
        other => StorageError::Invalid(other.to_string()),
    }
}

fn scan_status_to_domain(status: SafetyScanStatus) -> super::scanner::AttachmentSafetyScanStatus {
    match status {
        SafetyScanStatus::NotScanned => super::scanner::AttachmentSafetyScanStatus::NotScanned,
        SafetyScanStatus::Clean => super::scanner::AttachmentSafetyScanStatus::Clean,
        SafetyScanStatus::Suspicious => super::scanner::AttachmentSafetyScanStatus::Suspicious,
        SafetyScanStatus::Malicious => super::scanner::AttachmentSafetyScanStatus::Malicious,
        SafetyScanStatus::Failed => super::scanner::AttachmentSafetyScanStatus::Failed,
    }
}

fn scan_status_to_platform(status: super::scanner::AttachmentSafetyScanStatus) -> SafetyScanStatus {
    match status {
        super::scanner::AttachmentSafetyScanStatus::NotScanned => SafetyScanStatus::NotScanned,
        super::scanner::AttachmentSafetyScanStatus::Clean => SafetyScanStatus::Clean,
        super::scanner::AttachmentSafetyScanStatus::Suspicious => SafetyScanStatus::Suspicious,
        super::scanner::AttachmentSafetyScanStatus::Malicious => SafetyScanStatus::Malicious,
        super::scanner::AttachmentSafetyScanStatus::Failed => SafetyScanStatus::Failed,
    }
}

fn scan_report_to_domain(
    report: &SafetyScanReport,
) -> Result<super::scanner::AttachmentSafetyScanReport, StorageError> {
    if !report.metadata.is_object() {
        return Err(StorageError::Invalid(
            "scan_metadata must be a JSON object".to_owned(),
        ));
    }
    Ok(super::scanner::AttachmentSafetyScanReport {
        status: scan_status_to_domain(report.status),
        engine: report.engine.clone(),
        checked_at: report.checked_at,
        summary: report.summary.clone(),
        metadata: report.metadata.clone(),
    })
}

fn imported_attachment_to_platform(
    item: ImportedCommunicationAttachment,
) -> Result<ImportedAttachmentRecord, StorageError> {
    Ok(ImportedAttachmentRecord {
        attachment_id: item.attachment_id,
        account_id: item.account_id,
        channel_kind: item.channel_kind,
        blob_id: item.blob_id,
        filename: item.filename,
        content_type: item.content_type,
        size_bytes: item.size_bytes,
        sha256: item.sha256,
        source_kind: item.source_kind,
        imported_by: item.imported_by,
        scan_status: scan_status_to_platform(item.scan_status),
        scan_engine: item.scan_engine,
        scan_checked_at: item.scan_checked_at,
        scan_summary: item.scan_summary,
        scan_metadata: item.scan_metadata,
        metadata: item.metadata,
        storage_kind: item.storage_kind,
        storage_path: item.storage_path,
        created_at: item.created_at,
        updated_at: item.updated_at,
    })
}

fn imported_attachment_select_sql(predicate: &str) -> String {
    format!(
        r#"
        SELECT
            i.attachment_id,
            i.account_id,
            i.channel_kind,
            i.blob_id,
            i.filename,
            i.content_type,
            i.size_bytes,
            i.sha256,
            i.source_kind,
            i.imported_by,
            i.scan_status,
            i.scan_engine,
            i.scan_checked_at,
            i.scan_summary,
            i.scan_metadata,
            i.metadata,
            b.storage_kind AS blob_storage_kind,
            b.storage_path AS blob_storage_path,
            i.created_at,
            i.updated_at
        FROM communication_attachment_imports i
        JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
        WHERE {predicate}
        ORDER BY i.created_at DESC
        LIMIT 1
        "#
    )
}
