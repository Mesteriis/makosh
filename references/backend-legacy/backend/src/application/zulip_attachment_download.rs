use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::credentials::ProviderCredentialReader;
use crate::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore;
use makosh_communications_api::attachments::CommunicationMessageAttachmentScanPort;
use makosh_communications_postgres::attachment_scan::PostgresCommunicationMessageAttachmentScan;
use makosh_communications_postgres::errors::CommunicationIngestionError;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};

use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::secrets::resolver::SecretResolver;
use crate::workflows::zulip_attachment_storage::{
    ZulipAttachmentBytes, ZulipAttachmentMaterialization, ZulipAttachmentStorageError,
    persist_zulip_attachment_bytes,
};
use makosh_provider_zulip::client::{ZulipApiClient, ZulipClientConfig, ZulipClientError};

const ZULIP_CHANNEL_KIND: &str = "zulip";
const MAX_MESSAGE_SCAN_LIMIT: i64 = 500;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ZulipAttachmentDownloadReport {
    pub accounts_scanned: usize,
    pub accounts_failed: usize,
    pub candidates_seen: usize,
    pub attachments_downloaded: usize,
    pub attachments_materialized: usize,
    pub attachments_skipped: usize,
    pub attachments_failed: usize,
}

pub struct ZulipAttachmentDownloadWorker<R> {
    pool: PgPool,
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    attachment_scan: PostgresCommunicationMessageAttachmentScan,
    resolver: R,
    blob_root: PathBuf,
}

impl<R> ZulipAttachmentDownloadWorker<R>
where
    R: SecretResolver,
{
    pub fn new(pool: PgPool, resolver: R) -> Self {
        Self {
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            secret_store: SecretReferenceStore::new(pool.clone()),
            attachment_scan: PostgresCommunicationMessageAttachmentScan::new(pool.clone()),
            pool,
            resolver,
            blob_root: PathBuf::from(DEFAULT_MAIL_SYNC_BLOB_ROOT),
        }
    }

    pub fn with_blob_root(mut self, blob_root: impl AsRef<Path>) -> Self {
        self.blob_root = blob_root.as_ref().to_path_buf();
        self
    }
}

impl<R> ZulipAttachmentDownloadWorker<R>
where
    R: SecretResolver + Send + Sync,
{
    pub async fn download_due(
        &self,
        now: DateTime<Utc>,
        limit_per_account: i64,
    ) -> Result<ZulipAttachmentDownloadReport, ZulipAttachmentDownloadWorkerError> {
        let accounts = self.provider_account_store.list().await?;
        let mut report = ZulipAttachmentDownloadReport::default();

        for account in accounts
            .into_iter()
            .filter(|account| account.provider_kind == CommunicationProviderKind::ZulipBot)
        {
            report.accounts_scanned += 1;
            match self
                .download_due_for_account(&account.account_id, now, limit_per_account)
                .await
            {
                Ok(account_report) => report.merge(account_report),
                Err(error) => {
                    report.accounts_failed += 1;
                    tracing::warn!(
                        error = %error,
                        account_id = %account.account_id,
                        "zulip attachment download account tick failed"
                    );
                }
            }
        }

        Ok(report)
    }

    pub async fn download_due_for_account(
        &self,
        account_id: &str,
        _now: DateTime<Utc>,
        limit: i64,
    ) -> Result<ZulipAttachmentDownloadReport, ZulipAttachmentDownloadWorkerError> {
        let account = self.zulip_account(account_id).await?;
        let base_url = zulip_base_url(&account)?;
        let client = self.zulip_client(&account, base_url).await?;
        let scan = self.pending_attachments(&account.account_id, limit).await?;
        let mut report = ZulipAttachmentDownloadReport {
            accounts_scanned: 1,
            candidates_seen: scan.pending.len(),
            attachments_skipped: scan.skipped,
            ..ZulipAttachmentDownloadReport::default()
        };

        let message_lookup = ProviderChannelMessageStore::new(self.pool.clone());
        for candidate in scan.pending {
            match self
                .download_and_materialize(&client, &message_lookup, &candidate)
                .await
            {
                Ok(_) => {
                    report.attachments_downloaded += 1;
                    report.attachments_materialized += 1;
                }
                Err(error) => {
                    report.attachments_failed += 1;
                    tracing::warn!(
                        error = %error,
                        account_id = %candidate.account_id,
                        provider_message_id = %candidate.provider_message_id,
                        provider_attachment_id = %candidate.provider_attachment_id,
                        "zulip attachment download failed"
                    );
                }
            }
        }

        Ok(report)
    }

    async fn zulip_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, ZulipAttachmentDownloadWorkerError> {
        let account = self
            .provider_account_store
            .get(account_id)
            .await?
            .ok_or_else(|| ZulipAttachmentDownloadWorkerError::AccountNotFound {
                account_id: account_id.trim().to_owned(),
            })?;
        if account.provider_kind != CommunicationProviderKind::ZulipBot {
            return Err(ZulipAttachmentDownloadWorkerError::UnsupportedProvider {
                account_id: account.account_id,
                provider_kind: account.provider_kind.as_str(),
            });
        }
        Ok(account)
    }

    async fn zulip_client(
        &self,
        account: &ProviderAccount,
        base_url: &str,
    ) -> Result<ZulipApiClient, ZulipAttachmentDownloadWorkerError> {
        let credential_reader = ProviderCredentialReader::new(
            self.provider_secret_binding_store.clone(),
            self.secret_store.clone(),
            &self.resolver,
        );
        let credential = credential_reader
            .read(
                &account.account_id,
                ProviderAccountSecretPurpose::ZulipApiKey,
            )
            .await
            .map_err(
                |_| ZulipAttachmentDownloadWorkerError::CredentialUnavailable {
                    account_id: account.account_id.clone(),
                },
            )?;

        Ok(ZulipApiClient::new(
            ZulipClientConfig::new(
                base_url,
                account.external_account_id.as_str(),
                credential.secret.expose_for_runtime(),
            )
            .map_err(
                |_| ZulipAttachmentDownloadWorkerError::InvalidAccountConfig {
                    account_id: account.account_id.clone(),
                    field: "base_url",
                },
            )?,
        ))
    }

    async fn pending_attachments(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<PendingZulipAttachmentScan, ZulipAttachmentDownloadWorkerError> {
        if limit <= 0 {
            return Ok(PendingZulipAttachmentScan::default());
        }
        let row_limit = limit.saturating_mul(5).clamp(1, MAX_MESSAGE_SCAN_LIMIT);
        let rows = self
            .attachment_scan
            .scan_message_attachments(account_id, ZULIP_CHANNEL_KIND, row_limit)
            .await
            .map_err(|error| ZulipAttachmentDownloadWorkerError::AttachmentScan(error.0))?;

        let mut scan = PendingZulipAttachmentScan::default();
        for row in rows {
            let account_id = row.account_id;
            let provider_message_id = row.provider_record_id;
            let metadata = row.message_metadata;
            let extraction =
                pending_attachments_from_metadata(&account_id, &provider_message_id, &metadata);
            scan.skipped += extraction.skipped;
            for pending in extraction.pending {
                if scan.pending.len() >= limit as usize {
                    return Ok(scan);
                }
                scan.pending.push(pending);
            }
        }

        Ok(scan)
    }

    async fn download_and_materialize(
        &self,
        client: &ZulipApiClient,
        message_lookup: &ProviderChannelMessageStore,
        candidate: &PendingZulipAttachment,
    ) -> Result<ZulipAttachmentMaterialization, ZulipAttachmentDownloadWorkerError> {
        let downloaded = client
            .download_user_upload(&candidate.upload_url)
            .await
            .map_err(client_error)?;
        let content_type = candidate.content_type.clone().or(downloaded.content_type);

        persist_zulip_attachment_bytes(
            self.pool.clone(),
            message_lookup,
            &ZulipAttachmentBytes {
                account_id: candidate.account_id.clone(),
                provider_message_id: candidate.provider_message_id.clone(),
                provider_attachment_id: candidate.provider_attachment_id.clone(),
                filename: candidate.filename.clone(),
                content_type,
                bytes: downloaded.bytes,
            },
            &self.blob_root,
        )
        .await
        .map_err(Into::into)
    }
}

impl ZulipAttachmentDownloadReport {
    fn merge(&mut self, other: Self) {
        self.candidates_seen += other.candidates_seen;
        self.attachments_downloaded += other.attachments_downloaded;
        self.attachments_materialized += other.attachments_materialized;
        self.attachments_skipped += other.attachments_skipped;
        self.attachments_failed += other.attachments_failed;
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PendingZulipAttachmentScan {
    pending: Vec<PendingZulipAttachment>,
    skipped: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PendingZulipAttachmentExtraction {
    pending: Vec<PendingZulipAttachment>,
    skipped: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingZulipAttachment {
    account_id: String,
    provider_message_id: String,
    provider_attachment_id: String,
    filename: Option<String>,
    content_type: Option<String>,
    upload_url: String,
}

fn pending_attachments_from_metadata(
    account_id: &str,
    provider_message_id: &str,
    metadata: &Value,
) -> PendingZulipAttachmentExtraction {
    let Some(attachments) = metadata.get("attachments").and_then(Value::as_array) else {
        return PendingZulipAttachmentExtraction::default();
    };
    let mut extraction = PendingZulipAttachmentExtraction::default();

    for attachment in attachments {
        let Some(object) = attachment.as_object() else {
            extraction.skipped += 1;
            continue;
        };
        if !is_zulip_attachment(object) || is_materialized_attachment(object) {
            extraction.skipped += 1;
            continue;
        }
        let Some(provider_attachment_id) =
            metadata_string(object, &["provider_attachment_id", "attachment_id"])
        else {
            extraction.skipped += 1;
            continue;
        };
        let Some(upload_url) = upload_url_from_metadata(object) else {
            extraction.skipped += 1;
            continue;
        };

        extraction.pending.push(PendingZulipAttachment {
            account_id: account_id.to_owned(),
            provider_message_id: provider_message_id.to_owned(),
            provider_attachment_id,
            filename: metadata_string(object, &["filename", "name"]),
            content_type: metadata_string(object, &["content_type", "mime_type"]),
            upload_url,
        });
    }

    extraction
}

fn is_zulip_attachment(object: &Map<String, Value>) -> bool {
    object
        .get("provider")
        .and_then(Value::as_str)
        .map(|value| value == "zulip")
        .unwrap_or(true)
}

fn is_materialized_attachment(object: &Map<String, Value>) -> bool {
    matches!(
        object.get("materialization_state").and_then(Value::as_str),
        Some("materialized")
    ) || matches!(
        object.get("bytes_state").and_then(Value::as_str),
        Some("transferred")
    ) || metadata_string(object, &["blob_id"]).is_some()
}

fn upload_url_from_metadata(object: &Map<String, Value>) -> Option<String> {
    if let Some(url) = metadata_string(object, &["url"]) {
        return Some(url);
    }
    let path_id = metadata_string(object, &["path_id"])?;
    let path = path_id.trim_start_matches('/');
    if path == "user_uploads" || path.starts_with("user_uploads/") {
        Some(format!("/{path}"))
    } else {
        Some(format!("/user_uploads/{path}"))
    }
}

fn metadata_string(object: &Map<String, Value>, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| {
        object
            .get(*field)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn zulip_base_url(account: &ProviderAccount) -> Result<&str, ZulipAttachmentDownloadWorkerError> {
    account
        .config
        .get("base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZulipAttachmentDownloadWorkerError::InvalidAccountConfig {
            account_id: account.account_id.clone(),
            field: "base_url",
        })
}

fn client_error(error: ZulipClientError) -> ZulipAttachmentDownloadWorkerError {
    match error {
        ZulipClientError::Api { status, .. } => {
            ZulipAttachmentDownloadWorkerError::ProviderApi { status }
        }
        ZulipClientError::InvalidRequest(_) => {
            ZulipAttachmentDownloadWorkerError::InvalidClientRequest
        }
        ZulipClientError::Json(_) => ZulipAttachmentDownloadWorkerError::InvalidProviderResponse,
        ZulipClientError::Http(_) => ZulipAttachmentDownloadWorkerError::Transport,
        ZulipClientError::Url(_) => ZulipAttachmentDownloadWorkerError::InvalidClientRequest,
    }
}

#[derive(Debug, Error)]
pub enum ZulipAttachmentDownloadWorkerError {
    #[error("Zulip provider account `{account_id}` was not found")]
    AccountNotFound { account_id: String },
    #[error("provider account `{account_id}` is `{provider_kind}`, not `zulip_bot`")]
    UnsupportedProvider {
        account_id: String,
        provider_kind: &'static str,
    },
    #[error("Zulip provider account `{account_id}` has invalid `{field}` config")]
    InvalidAccountConfig {
        account_id: String,
        field: &'static str,
    },
    #[error("Zulip API credential is unavailable for account `{account_id}`")]
    CredentialUnavailable { account_id: String },
    #[error("Zulip API returned HTTP {status} while downloading an attachment")]
    ProviderApi { status: u16 },
    #[error("Zulip attachment download request was invalid")]
    InvalidClientRequest,
    #[error("Zulip attachment download HTTP request failed")]
    Transport,
    #[error("Zulip attachment download response was invalid")]
    InvalidProviderResponse,
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),
    #[error("communication attachment scan failed: {0}")]
    AttachmentScan(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Storage(#[from] ZulipAttachmentStorageError),
}
