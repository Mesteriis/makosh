use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use serde_json::json;

use super::command_service::{
    CommunicationAttachmentImportCommand, CommunicationCommandService,
    CommunicationCommandServiceError,
};
use super::storage::blob_store::LocalCommunicationBlobStore;
use super::storage::imports::new_communication_attachment_import_id;
use super::storage::models::{
    ImportedCommunicationAttachment, NewCommunicationAttachmentImport, NewCommunicationBlob,
};
use super::storage::scanner::{
    AttachmentSafetyScanRequest, scan_attachment_with_configured_clamav,
};
use super::storage::store::CommunicationStorageStore;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;

const MAX_ATTACHMENT_IMPORT_BYTES: usize = 50 * 1024 * 1024;
const LOCAL_IMPORT_ACTOR_ID: &str = "makosh-frontend";

impl CommunicationCommandService {
    pub async fn import_attachment(
        &self,
        request: CommunicationAttachmentImportCommand,
    ) -> Result<ImportedCommunicationAttachment, CommunicationCommandServiceError> {
        let bytes = decode_import_bytes(&request.content_base64)?;
        if bytes.is_empty() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "attachment import bytes must not be empty",
            ));
        }
        if bytes.len() > MAX_ATTACHMENT_IMPORT_BYTES {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "attachment import exceeds the local size limit",
            ));
        }

        let content_type = request
            .content_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("application/octet-stream")
            .to_owned();
        let filename = request
            .filename
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let metadata = request.metadata.clone().unwrap_or_else(|| json!({}));
        if !metadata.is_object() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "attachment import metadata must be an object",
            ));
        }

        let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
        let local_blob = blob_store.put_blob(&bytes).await?;
        let mail_store = CommunicationStorageStore::new(self.pool.clone());
        let stored_blob = mail_store
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob).content_type(&content_type),
            )
            .await?;
        let scan_request = AttachmentSafetyScanRequest {
            provider_attachment_id: "local-import",
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &local_blob.sha256,
            storage_kind: &local_blob.storage_kind,
            storage_path: &local_blob.storage_path,
            bytes: &bytes,
        };
        let scan_report = scan_attachment_with_configured_clamav(&scan_request).await?;
        let seed = format!(
            "{}:{}:{}:{}",
            local_blob.sha256,
            filename.as_deref().unwrap_or(""),
            request.account_id.as_deref().unwrap_or(""),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let attachment_id = new_communication_attachment_import_id(&seed);
        let source_kind = request
            .source_kind
            .clone()
            .unwrap_or_else(|| "local_import".to_owned());
        let observation = self
            .capture_observation(
                "attachment import",
                "COMMUNICATION_ATTACHMENT",
                json!({
                    "attachment_id": attachment_id,
                    "account_id": request.account_id.clone(),
                    "channel_kind": request.channel_kind.clone(),
                    "filename": filename.clone(),
                    "content_type": content_type.clone(),
                    "size_bytes": local_blob.size_bytes,
                    "sha256": local_blob.sha256.clone(),
                    "source_kind": source_kind.clone(),
                    "metadata": metadata.clone(),
                }),
                format!("communications://attachments/import/{attachment_id}"),
                json!({
                    "captured_by": "mail_service.import_attachment",
                    "operation": "attachment_import",
                    "storage_kind": local_blob.storage_kind,
                    "blob_id": stored_blob.blob_id,
                }),
            )
            .await?;

        let mut import = NewCommunicationAttachmentImport::new(
            attachment_id,
            stored_blob.blob_id,
            &content_type,
            local_blob.size_bytes,
            &local_blob.sha256,
            LOCAL_IMPORT_ACTOR_ID,
        )
        .source_kind(
            request
                .source_kind
                .unwrap_or_else(|| "local_import".to_owned()),
        )
        .scan_report(scan_report)
        .metadata(metadata);
        if let Some(account_id) = request.account_id {
            import = import.account_id(account_id);
        }
        if let Some(channel_kind) = request.channel_kind {
            import = import.channel_kind(channel_kind);
        }
        if let Some(filename) = filename {
            import = import.filename(filename);
        }

        Ok(mail_store
            .upsert_imported_attachment_with_observation(
                &import,
                Some(&observation.observation_id),
                "attachment_import",
                None,
            )
            .await?)
    }
}

fn decode_import_bytes(content_base64: &str) -> Result<Vec<u8>, CommunicationCommandServiceError> {
    let encoded = content_base64
        .split_once(',')
        .map(|(_, value)| value)
        .unwrap_or(content_base64)
        .trim();
    BASE64_STANDARD.decode(encoded).map_err(|_| {
        CommunicationCommandServiceError::InvalidRequest("invalid attachment import base64")
    })
}
