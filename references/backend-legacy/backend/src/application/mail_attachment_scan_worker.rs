use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use crate::domains::communications::storage::errors::{
    AttachmentSafetyScanError, CommunicationStorageError,
};
use crate::domains::communications::storage::scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanRequest, AttachmentSafetyScanStatus,
    scan_attachment_with_clamav,
};
use crate::domains::communications::storage::store::CommunicationStorageStore;
use crate::platform::attachment_scanning::ClamAvClient;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;

const MAX_ATTACHMENT_RESCAN_BATCH_SIZE: i64 = 100;

#[derive(Clone)]
pub(crate) struct MailAttachmentScanWorker {
    storage: CommunicationStorageStore,
    blobs: LocalCommunicationBlobStore,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct MailAttachmentScanReport {
    pub(crate) candidates_seen: usize,
    pub(crate) verdicts_persisted: usize,
    pub(crate) retry_deferred: usize,
    pub(crate) invalid_or_stale: usize,
    pub(crate) failures: usize,
}

#[derive(Debug, Error)]
pub(crate) enum MailAttachmentScanWorkerError {
    #[error(transparent)]
    Storage(#[from] CommunicationStorageError),

    #[error(transparent)]
    Scan(#[from] AttachmentSafetyScanError),
}

enum RescanAttempt {
    Verdict(AttachmentSafetyScanReport),
    RetryDeferred,
    InvalidOrStale,
}

struct RescanBlob<'a> {
    provider_attachment_id: &'a str,
    filename: Option<&'a str>,
    content_type: &'a str,
    size_bytes: i64,
    sha256: &'a str,
    storage_kind: &'a str,
    storage_path: &'a str,
}

impl MailAttachmentScanWorker {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self::with_blob_root(pool, DEFAULT_MAIL_SYNC_BLOB_ROOT)
    }

    pub(crate) fn with_blob_root(pool: PgPool, blob_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            storage: CommunicationStorageStore::new(pool),
            blobs: LocalCommunicationBlobStore::new(blob_root),
        }
    }

    pub(crate) async fn scan_due(
        &self,
        limit: i64,
    ) -> Result<MailAttachmentScanReport, MailAttachmentScanWorkerError> {
        let clamav = match ClamAvClient::from_env() {
            Ok(Some(client)) => client,
            Ok(None) => return Ok(MailAttachmentScanReport::default()),
            Err(error) => {
                tracing::warn!(error = %error, "mail attachment rescan is disabled by invalid ClamAV configuration");
                return Ok(MailAttachmentScanReport::default());
            }
        };
        self.scan_due_with_clamav(&clamav, limit).await
    }

    async fn scan_due_with_clamav(
        &self,
        clamav: &ClamAvClient,
        limit: i64,
    ) -> Result<MailAttachmentScanReport, MailAttachmentScanWorkerError> {
        let limit = limit.clamp(1, MAX_ATTACHMENT_RESCAN_BATCH_SIZE);
        let mut report = MailAttachmentScanReport::default();
        let message_attachments = self.storage.list_not_scanned_attachments(limit).await?;

        for candidate in message_attachments {
            report.candidates_seen += 1;
            let attachment = candidate.attachment;
            match self
                .scan_blob(
                    clamav,
                    RescanBlob {
                        provider_attachment_id: &attachment.provider_attachment_id,
                        filename: attachment.filename.as_deref(),
                        content_type: &attachment.content_type,
                        size_bytes: attachment.size_bytes,
                        sha256: &attachment.sha256,
                        storage_kind: &candidate.storage_kind,
                        storage_path: &candidate.storage_path,
                    },
                )
                .await
            {
                Ok(RescanAttempt::Verdict(verdict)) => match self
                    .storage
                    .persist_not_scanned_attachment_verdict(
                        &attachment.attachment_id,
                        &attachment.sha256,
                        &verdict,
                    )
                    .await
                {
                    Ok(Some(_)) => report.verdicts_persisted += 1,
                    Ok(None) => report.invalid_or_stale += 1,
                    Err(error) => {
                        report.failures += 1;
                        tracing::warn!(attachment_id = %attachment.attachment_id, error = %error, "mail attachment rescan verdict was not persisted");
                    }
                },
                Ok(RescanAttempt::RetryDeferred) => report.retry_deferred += 1,
                Ok(RescanAttempt::InvalidOrStale) => report.invalid_or_stale += 1,
                Err(error) => {
                    report.failures += 1;
                    tracing::warn!(attachment_id = %attachment.attachment_id, error = %error, "mail attachment rescan failed");
                }
            }
        }

        let remaining =
            limit.saturating_sub(i64::try_from(report.candidates_seen).unwrap_or(limit));
        if remaining == 0 {
            return Ok(report);
        }
        let imported_attachments = self
            .storage
            .list_not_scanned_imported_attachments(remaining)
            .await?;
        for imported in imported_attachments {
            report.candidates_seen += 1;
            match self
                .scan_blob(
                    clamav,
                    RescanBlob {
                        provider_attachment_id: &imported.attachment_id,
                        filename: imported.filename.as_deref(),
                        content_type: &imported.content_type,
                        size_bytes: imported.size_bytes,
                        sha256: &imported.sha256,
                        storage_kind: &imported.storage_kind,
                        storage_path: &imported.storage_path,
                    },
                )
                .await
            {
                Ok(RescanAttempt::Verdict(verdict)) => match self
                    .storage
                    .persist_not_scanned_imported_attachment_verdict(
                        &imported.attachment_id,
                        &imported.sha256,
                        &verdict,
                    )
                    .await
                {
                    Ok(Some(_)) => report.verdicts_persisted += 1,
                    Ok(None) => report.invalid_or_stale += 1,
                    Err(error) => {
                        report.failures += 1;
                        tracing::warn!(attachment_id = %imported.attachment_id, error = %error, "imported attachment rescan verdict was not persisted");
                    }
                },
                Ok(RescanAttempt::RetryDeferred) => report.retry_deferred += 1,
                Ok(RescanAttempt::InvalidOrStale) => report.invalid_or_stale += 1,
                Err(error) => {
                    report.failures += 1;
                    tracing::warn!(attachment_id = %imported.attachment_id, error = %error, "imported attachment rescan failed");
                }
            }
        }
        Ok(report)
    }

    async fn scan_blob(
        &self,
        clamav: &ClamAvClient,
        candidate: RescanBlob<'_>,
    ) -> Result<RescanAttempt, MailAttachmentScanWorkerError> {
        if candidate.storage_kind != "local_fs" {
            return Ok(RescanAttempt::InvalidOrStale);
        }
        let bytes = self.blobs.read_blob(candidate.storage_path).await?;
        if i64::try_from(bytes.len()).ok() != Some(candidate.size_bytes)
            || LocalCommunicationBlobStore::sha256_for_bytes(&bytes) != candidate.sha256
        {
            return Ok(RescanAttempt::InvalidOrStale);
        }
        let scan = scan_attachment_with_clamav(
            &AttachmentSafetyScanRequest {
                provider_attachment_id: candidate.provider_attachment_id,
                filename: candidate.filename,
                content_type: candidate.content_type,
                size_bytes: candidate.size_bytes,
                sha256: candidate.sha256,
                storage_kind: candidate.storage_kind,
                storage_path: candidate.storage_path,
                bytes: &bytes,
            },
            Some(clamav),
        )
        .await?;
        if scan.status == AttachmentSafetyScanStatus::NotScanned {
            return Ok(RescanAttempt::RetryDeferred);
        }
        Ok(RescanAttempt::Verdict(scan))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sqlx::postgres::PgPoolOptions;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    async fn fake_clamd(response: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake ClamAV");
        let address = listener.local_addr().expect("listener address").to_string();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept scan");
            let mut command = [0_u8; 10];
            socket.read_exact(&mut command).await.expect("read command");
            assert_eq!(&command, b"zINSTREAM\0");
            loop {
                let length = socket.read_u32().await.expect("read frame") as usize;
                if length == 0 {
                    break;
                }
                let mut body = vec![0_u8; length];
                socket.read_exact(&mut body).await.expect("read body");
            }
            socket.write_all(response).await.expect("write verdict");
        });
        address
    }

    fn worker(blob_root: &std::path::Path) -> MailAttachmentScanWorker {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://makosh:makosh@127.0.0.1:30432/makosh")
            .expect("lazy test pool");
        MailAttachmentScanWorker::with_blob_root(pool, blob_root)
    }

    #[tokio::test]
    async fn verified_blob_receives_a_clean_rescan_verdict() {
        let directory = tempfile::tempdir().expect("temporary blob root");
        let blobs = LocalCommunicationBlobStore::new(directory.path());
        let blob = blobs
            .put_blob(b"safe attachment")
            .await
            .expect("store blob");
        let address = fake_clamd(b"stream: OK\0").await;
        let clamav = ClamAvClient::new(address, Duration::from_secs(1)).expect("client");

        let attempt = worker(directory.path())
            .scan_blob(
                &clamav,
                RescanBlob {
                    provider_attachment_id: "provider-attachment-1",
                    filename: Some("invoice.txt"),
                    content_type: "text/plain",
                    size_bytes: blob.size_bytes,
                    sha256: &blob.sha256,
                    storage_kind: &blob.storage_kind,
                    storage_path: &blob.storage_path,
                },
            )
            .await
            .expect("rescan attempt");

        assert!(matches!(
            attempt,
            RescanAttempt::Verdict(AttachmentSafetyScanReport {
                status: AttachmentSafetyScanStatus::Clean,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn sha_mismatch_stays_quarantined_without_contacting_clamav() {
        let directory = tempfile::tempdir().expect("temporary blob root");
        let blobs = LocalCommunicationBlobStore::new(directory.path());
        let blob = blobs
            .put_blob(b"safe attachment")
            .await
            .expect("store blob");
        let clamav = ClamAvClient::new("127.0.0.1:1", Duration::from_millis(20)).expect("client");

        let attempt = worker(directory.path())
            .scan_blob(
                &clamav,
                RescanBlob {
                    provider_attachment_id: "provider-attachment-1",
                    filename: Some("invoice.txt"),
                    content_type: "text/plain",
                    size_bytes: blob.size_bytes,
                    sha256: "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                    storage_kind: &blob.storage_kind,
                    storage_path: &blob.storage_path,
                },
            )
            .await
            .expect("rescan attempt");

        assert!(matches!(attempt, RescanAttempt::InvalidOrStale));
    }
}
