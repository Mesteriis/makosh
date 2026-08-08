use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_blob_api::{BlobReadError, BlobReadFuture, BlobReadPort, BlobRef};
use makosh_communications_api::attachments::{
    AttachmentLookupPortFuture, AttachmentReference, CommunicationAttachmentLookupPort,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::domains::communications::storage::port::CommunicationAttachmentPort;

/// PostgreSQL-backed adapter for the provider-neutral attachment lookup port.
/// It intentionally drops storage paths; providers receive only metadata and
/// later a scoped BlobRef capability.
#[derive(Clone)]
pub struct PostgresCommunicationAttachmentLookup {
    port: CommunicationAttachmentPort,
}

impl PostgresCommunicationAttachmentLookup {
    pub fn new(port: CommunicationAttachmentPort) -> Self {
        Self { port }
    }
}

impl CommunicationAttachmentLookupPort for PostgresCommunicationAttachmentLookup {
    fn lookup_by_id<'a>(
        &'a self,
        attachment_id: &'a str,
    ) -> AttachmentLookupPortFuture<'a, Option<AttachmentReference>> {
        Box::pin(async move {
            self.port
                .imported_attachment_by_id(attachment_id)
                .await
                .map(|attachment| {
                    attachment.map(|attachment| AttachmentReference {
                        attachment_id: attachment.attachment_id,
                        blob_id: attachment.blob_id,
                        account_id: attachment.account_id,
                        channel_kind: attachment.channel_kind,
                        filename: attachment.filename,
                        content_type: attachment.content_type,
                        size_bytes: attachment.size_bytes,
                        sha256: attachment.sha256,
                        scan_status: Some(attachment.scan_status.as_str().to_owned()),
                    })
                })
                .map_err(|error| {
                    makosh_communications_api::attachments::AttachmentLookupPortError(
                        error.to_string(),
                    )
                })
        })
    }
}

/// Transitional composition adapter. The path is retained only inside the
/// opaque capability and is never exposed to provider crates or envelopes.
#[derive(Clone, Debug)]
pub struct LocalBlobReadAdapter {
    store: LocalCommunicationBlobStore,
    capabilities: Arc<Mutex<HashMap<String, String>>>,
}

impl LocalBlobReadAdapter {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            store: LocalCommunicationBlobStore::new(root.into()),
            capabilities: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn issue(
        &self,
        blob_id: impl Into<String>,
        account_id: impl Into<String>,
        path: impl Into<String>,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<BlobRef, makosh_blob_api::BlobApiError> {
        let token = uuid::Uuid::new_v4().to_string();
        self.capabilities
            .lock()
            .expect("blob capability mutex is not poisoned")
            .insert(token.clone(), path.into());
        BlobRef::new(blob_id, account_id, token, expires_at)
    }
}

impl BlobReadPort for LocalBlobReadAdapter {
    fn read_bounded<'a>(&'a self, reference: &'a BlobRef, max_bytes: usize) -> BlobReadFuture<'a> {
        Box::pin(async move {
            if reference.is_expired_at(chrono::Utc::now()) {
                self.capabilities
                    .lock()
                    .expect("blob capability mutex is not poisoned")
                    .remove(reference.capability());
                return Err(BlobReadError::Expired);
            }
            let path = self
                .capabilities
                .lock()
                .expect("blob capability mutex is not poisoned")
                .get(reference.capability())
                .cloned()
                .ok_or(BlobReadError::Unavailable)?;
            let bytes = self
                .store
                .read_blob(&path)
                .await
                .map_err(|_| BlobReadError::Unavailable)?;
            self.capabilities
                .lock()
                .expect("blob capability mutex is not poisoned")
                .remove(reference.capability());
            if bytes.len() > max_bytes {
                return Err(BlobReadError::TooLarge);
            }
            Ok(bytes)
        })
    }
}
