use crate::platform::secrets::store::SecretReferenceStore;
use chrono::{DateTime, Utc};
use makosh_blob_api::BlobReadPort;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use makosh_communications_api::attachments::CommunicationAttachmentLookupPort;
use makosh_communications_api::commands::{
    CommunicationProviderCommand, ProviderCommandQueuePort, ProviderCommandQueuePortError,
};
use makosh_provider_api::{CredentialLease, ProviderId};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::application::communications_attachment_lookup::{
    LocalBlobReadAdapter, PostgresCommunicationAttachmentLookup,
};
use crate::domains::communications::credentials::ProviderCredentialReader;
use crate::domains::communications::storage::errors::CommunicationStorageError;
use crate::domains::communications::storage::models::{
    ImportedCommunicationAttachment, StoredCommunicationBlob,
};
use crate::domains::communications::storage::port::CommunicationAttachmentPort;
use crate::domains::communications::storage::store::CommunicationStorageStore;
use makosh_communications_postgres::errors::CommunicationIngestionError;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};

use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::secrets::resolver::SecretResolver;
use makosh_provider_orchestration::{
    communication_command_to_provider_envelope, execute_provider_command,
};
use makosh_provider_zulip::client::{ZulipApiClient, ZulipClientConfig};
use makosh_provider_zulip::command_execution::{
    ZulipCommandExecutionError, ZulipExecutableCommand, ZulipPreparedUpload, execute_zulip_command,
};
use makosh_provider_zulip::runtime::{ZulipInProcessRuntime, ZulipRuntimeConfig};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ZulipCommandExecutionReport {
    pub accounts_scanned: usize,
    pub accounts_failed: usize,
    pub claimed: usize,
    pub completed: usize,
    pub retrying: usize,
    pub dead_lettered: usize,
}

pub struct ZulipCommandWorker<R, Q = CommunicationProviderCommandStore> {
    pool: PgPool,
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    command_queue: Q,
    resolver: R,
    attachment_lookup: std::sync::Arc<dyn CommunicationAttachmentLookupPort>,
    blob_reader: std::sync::Arc<LocalBlobReadAdapter>,
}

impl<R> ZulipCommandWorker<R, CommunicationProviderCommandStore>
where
    R: SecretResolver,
{
    pub fn new(pool: PgPool, resolver: R) -> Self {
        Self::with_command_queue(
            pool.clone(),
            resolver,
            CommunicationProviderCommandStore::new(pool),
        )
    }
}

impl<R, Q> ZulipCommandWorker<R, Q>
where
    R: SecretResolver,
{
    pub fn with_command_queue(pool: PgPool, resolver: R, command_queue: Q) -> Self {
        Self {
            pool: pool.clone(),
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            secret_store: SecretReferenceStore::new(pool.clone()),
            command_queue,
            resolver,
            attachment_lookup: std::sync::Arc::new(PostgresCommunicationAttachmentLookup::new(
                CommunicationAttachmentPort::new(pool),
            )),
            blob_reader: std::sync::Arc::new(LocalBlobReadAdapter::new(
                DEFAULT_MAIL_SYNC_BLOB_ROOT,
            )),
        }
    }
}

impl<R, Q> ZulipCommandWorker<R, Q>
where
    R: SecretResolver + Send + Sync,
    Q: ProviderCommandQueuePort,
{
    pub async fn execute_due(
        &self,
        now: DateTime<Utc>,
        limit_per_account: i64,
    ) -> Result<ZulipCommandExecutionReport, ZulipCommandWorkerError> {
        let accounts = self.provider_account_store.list().await?;
        let mut report = ZulipCommandExecutionReport::default();

        for account in accounts
            .into_iter()
            .filter(|account| account.provider_kind == CommunicationProviderKind::ZulipBot)
        {
            report.accounts_scanned += 1;
            match self
                .execute_due_for_account(&account.account_id, now, limit_per_account)
                .await
            {
                Ok(account_report) => report.merge(account_report),
                Err(error) => {
                    report.accounts_failed += 1;
                    tracing::warn!(
                        error = %error,
                        account_id = %account.account_id,
                        "zulip command worker account tick failed"
                    );
                }
            }
        }

        Ok(report)
    }

    pub async fn execute_due_for_account(
        &self,
        account_id: &str,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<ZulipCommandExecutionReport, ZulipCommandWorkerError> {
        let account = self.zulip_account(account_id).await?;
        let base_url = zulip_base_url(&account)?;
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
            .map_err(|_| ZulipCommandWorkerError::CredentialUnavailable {
                account_id: account.account_id.clone(),
            })?;
        let client = ZulipApiClient::new(
            ZulipClientConfig::new(
                base_url,
                account.external_account_id.as_str(),
                credential.secret.expose_for_runtime(),
            )
            .map_err(|_| ZulipCommandWorkerError::InvalidAccountConfig {
                account_id: account.account_id.clone(),
                field: "base_url",
            })?,
        );
        let runtime = ZulipInProcessRuntime::new(
            ZulipRuntimeConfig::new(
                account.account_id.clone(),
                base_url,
                account.external_account_id.clone(),
            )
            .map_err(|_| ZulipCommandWorkerError::InvalidAccountConfig {
                account_id: account.account_id.clone(),
                field: "base_url",
            })?,
        );
        let claimed = self
            .command_queue
            .claim_due(&account.account_id, "zulip", now, limit)
            .await?;
        let mut report = ZulipCommandExecutionReport {
            claimed: claimed.len(),
            ..ZulipCommandExecutionReport::default()
        };

        for command in claimed {
            if !command_requires_upload(&command.command_kind) {
                let issued_at = Utc::now();
                let deadline = issued_at + chrono::Duration::minutes(5);
                let envelope = communication_command_to_provider_envelope(
                    &command,
                    ProviderId::parse("zulip").expect("static provider id is valid"),
                    issued_at,
                    deadline,
                    0,
                )
                .map_err(|_| ZulipCommandWorkerError::InvalidCommand {
                    command_id: command.command_id.clone(),
                });
                let lease = CredentialLease::new(
                    "zulip",
                    account.account_id.clone(),
                    "api_key",
                    0,
                    issued_at,
                    deadline,
                    credential.secret.expose_for_runtime().as_bytes(),
                )
                .map_err(|_| ZulipCommandWorkerError::CredentialUnavailable {
                    account_id: account.account_id.clone(),
                });
                match (envelope, lease) {
                    (Ok(envelope), Ok(lease)) => {
                        match execute_provider_command(&runtime, &envelope, lease).await {
                            Ok(result) => {
                                self.command_queue
                                    .mark_completed_at_epoch(
                                        &command.command_id,
                                        "zulip",
                                        Utc::now(),
                                        result.payload,
                                        command.lease_epoch,
                                    )
                                    .await?;
                                report.completed += 1;
                            }
                            Err(error) => {
                                let error_code = error.code.clone();
                                self.command_queue
                                    .mark_failed_at_epoch(
                                        &command.command_id,
                                        "zulip",
                                        Utc::now(),
                                        error_code.as_str(),
                                        json!({"error": error_code}),
                                        command.lease_epoch,
                                    )
                                    .await?;
                                report.retrying += 1;
                            }
                        }
                    }
                    (Err(error), _) | (_, Err(error)) => {
                        self.command_queue
                            .mark_failed_at_epoch(
                                &command.command_id,
                                "zulip",
                                Utc::now(),
                                &error.to_string(),
                                json!({"error": "invalid_provider_command"}),
                                command.lease_epoch,
                            )
                            .await?;
                        report.retrying += 1;
                    }
                }
                continue;
            }
            let executable = match self.executable_command(&command).await {
                Ok(executable) => executable,
                Err(error) => {
                    let updated = self
                        .command_queue
                        .mark_failed_at_epoch(
                            &command.command_id,
                            "zulip",
                            Utc::now(),
                            &error.to_string(),
                            failure_result_payload(&error),
                            command.lease_epoch,
                        )
                        .await?;
                    if let Some(updated) = updated {
                        match updated.status.as_str() {
                            "dead_letter" => report.dead_lettered += 1,
                            _ => report.retrying += 1,
                        }
                    }
                    continue;
                }
            };
            match execute_zulip_command(&client, &executable).await {
                Ok(outcome) => {
                    self.command_queue
                        .mark_completed_at_epoch(
                            &command.command_id,
                            "zulip",
                            Utc::now(),
                            outcome.result_payload,
                            command.lease_epoch,
                        )
                        .await?;
                    report.completed += 1;
                }
                Err(error) => {
                    let updated = self
                        .command_queue
                        .mark_failed_at_epoch(
                            &command.command_id,
                            "zulip",
                            Utc::now(),
                            &error.to_string(),
                            failure_result_payload(&error),
                            command.lease_epoch,
                        )
                        .await?;
                    if let Some(updated) = updated {
                        match updated.status.as_str() {
                            "dead_letter" => report.dead_lettered += 1,
                            _ => report.retrying += 1,
                        }
                    }
                }
            }
        }

        Ok(report)
    }

    async fn zulip_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, ZulipCommandWorkerError> {
        let account = self
            .provider_account_store
            .get(account_id)
            .await?
            .ok_or_else(|| ZulipCommandWorkerError::AccountNotFound {
                account_id: account_id.trim().to_owned(),
            })?;
        if account.provider_kind != CommunicationProviderKind::ZulipBot {
            return Err(ZulipCommandWorkerError::UnsupportedProvider {
                account_id: account.account_id,
                provider_kind: account.provider_kind.as_str(),
            });
        }
        Ok(account)
    }

    async fn executable_command(
        &self,
        command: &CommunicationProviderCommand,
    ) -> Result<ZulipExecutableCommand, ZulipCommandExecutionError> {
        let mut executable = executable_command(command);
        if command_requires_upload(&command.command_kind) {
            executable = executable.prepared_upload(self.prepared_upload(command).await?);
        }
        Ok(executable)
    }

    async fn prepared_upload(
        &self,
        command: &CommunicationProviderCommand,
    ) -> Result<ZulipPreparedUpload, ZulipCommandExecutionError> {
        let storage = CommunicationStorageStore::new(self.pool.clone());
        let resolved =
            resolve_upload_blob(command, &storage, self.attachment_lookup.as_ref()).await?;
        if resolved.storage_kind != "local_fs" {
            return Err(invalid_command(
                command,
                "Zulip upload requires a local filesystem blob",
            ));
        }
        if resolved.scan_status.as_deref() == Some("malicious") {
            return Err(invalid_command(
                command,
                "Zulip upload rejected a malicious attachment import",
            ));
        }

        let blob_ref = self
            .blob_reader
            .issue(
                resolved.blob_id.clone(),
                command.account_id.clone(),
                resolved.storage_path.clone(),
                Utc::now() + chrono::Duration::minutes(5),
            )
            .map_err(|_| invalid_command(command, "invalid blob capability"))?;
        let bytes = self
            .blob_reader
            .read_bounded(&blob_ref, 25 * 1024 * 1024)
            .await
            .map_err(|_| invalid_command(command, "blob capability read failed"))?;

        Ok(ZulipPreparedUpload {
            filename: resolved
                .filename
                .unwrap_or_else(|| "attachment.bin".to_owned()),
            bytes,
            attachment_id: resolved.attachment_id,
            blob_id: resolved.blob_id,
            content_type: resolved.content_type,
            size_bytes: resolved.size_bytes,
            sha256: resolved.sha256,
        })
    }
}

impl ZulipCommandExecutionReport {
    fn merge(&mut self, other: Self) {
        self.claimed += other.claimed;
        self.completed += other.completed;
        self.retrying += other.retrying;
        self.dead_lettered += other.dead_lettered;
    }
}

fn executable_command(command: &CommunicationProviderCommand) -> ZulipExecutableCommand {
    ZulipExecutableCommand::new(
        command.command_id.clone(),
        command.command_kind.clone(),
        command.provider_message_id.clone(),
        command.payload.clone(),
    )
}

fn command_requires_upload(command_kind: &str) -> bool {
    matches!(
        command_kind,
        "upload_file" | "send_stream_message_with_upload" | "send_direct_message_with_upload"
    )
}

fn zulip_base_url(account: &ProviderAccount) -> Result<&str, ZulipCommandWorkerError> {
    account
        .config
        .get("base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZulipCommandWorkerError::InvalidAccountConfig {
            account_id: account.account_id.clone(),
            field: "base_url",
        })
}

#[derive(Clone, Debug)]
struct ResolvedUploadBlob {
    attachment_id: Option<String>,
    blob_id: String,
    filename: Option<String>,
    content_type: String,
    size_bytes: i64,
    sha256: String,
    scan_status: Option<String>,
    storage_kind: String,
    storage_path: String,
}

async fn resolve_upload_blob(
    command: &CommunicationProviderCommand,
    storage: &CommunicationStorageStore,
    attachment_lookup: &dyn CommunicationAttachmentLookupPort,
) -> Result<ResolvedUploadBlob, ZulipCommandExecutionError> {
    if let Some(attachment_id) = optional_payload_string(command, "attachment_id") {
        let reference = attachment_lookup
            .lookup_by_id(&attachment_id)
            .await
            .map_err(|error| invalid_command_owned(command, error.to_string()))?
            .ok_or_else(|| {
                invalid_command_owned(
                    command,
                    format!("attachment import `{attachment_id}` was not found"),
                )
            })?;
        if reference
            .account_id
            .as_deref()
            .is_some_and(|id| id != command.account_id)
            || reference
                .channel_kind
                .as_deref()
                .is_some_and(|kind| kind != "zulip")
        {
            return Err(invalid_command(
                command,
                "attachment import is outside the Zulip account scope",
            ));
        }
        let imported = storage
            .imported_attachment_by_id(&attachment_id)
            .await
            .map_err(|error| storage_error(command, error))?
            .ok_or_else(|| {
                invalid_command_owned(
                    command,
                    format!("attachment import `{attachment_id}` was not found"),
                )
            })?;
        if let Some(account_id) = imported.account_id.as_deref()
            && account_id != command.account_id
        {
            return Err(invalid_command(
                command,
                "attachment import belongs to a different account",
            ));
        }
        if let Some(channel_kind) = imported.channel_kind.as_deref()
            && channel_kind != "zulip"
        {
            return Err(invalid_command(
                command,
                "attachment import is not scoped to Zulip",
            ));
        }
        if let Some(blob_id) = optional_payload_string(command, "blob_id")
            && blob_id != imported.blob_id
        {
            return Err(invalid_command(
                command,
                "blob_id does not match attachment import",
            ));
        }
        return Ok(resolved_upload_from_import(imported, command));
    }

    let blob_id = optional_payload_string(command, "blob_id").ok_or_else(|| {
        invalid_command(command, "upload command requires attachment_id or blob_id")
    })?;
    let blob = storage
        .blob_by_id(&blob_id)
        .await
        .map_err(|error| storage_error(command, error))?
        .ok_or_else(|| invalid_command_owned(command, format!("blob `{blob_id}` was not found")))?;
    Ok(resolved_upload_from_blob(blob, command))
}

fn resolved_upload_from_import(
    imported: ImportedCommunicationAttachment,
    command: &CommunicationProviderCommand,
) -> ResolvedUploadBlob {
    ResolvedUploadBlob {
        attachment_id: Some(imported.attachment_id),
        blob_id: imported.blob_id,
        filename: optional_payload_string(command, "filename").or(imported.filename),
        content_type: imported.content_type,
        size_bytes: imported.size_bytes,
        sha256: imported.sha256,
        scan_status: Some(imported.scan_status.as_str().to_owned()),
        storage_kind: imported.storage_kind,
        storage_path: imported.storage_path,
    }
}

fn resolved_upload_from_blob(
    blob: StoredCommunicationBlob,
    command: &CommunicationProviderCommand,
) -> ResolvedUploadBlob {
    ResolvedUploadBlob {
        attachment_id: None,
        blob_id: blob.blob_id,
        filename: optional_payload_string(command, "filename"),
        content_type: blob
            .content_type
            .unwrap_or_else(|| "application/octet-stream".to_owned()),
        size_bytes: blob.size_bytes,
        sha256: blob.sha256,
        scan_status: None,
        storage_kind: blob.storage_kind,
        storage_path: blob.storage_path,
    }
}

fn optional_payload_string(command: &CommunicationProviderCommand, key: &str) -> Option<String> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn storage_error(
    command: &CommunicationProviderCommand,
    error: CommunicationStorageError,
) -> ZulipCommandExecutionError {
    invalid_command_owned(command, format!("storage error: {error}"))
}

fn invalid_command(
    command: &CommunicationProviderCommand,
    reason: &'static str,
) -> ZulipCommandExecutionError {
    invalid_command_owned(command, reason.to_owned())
}

fn invalid_command_owned(
    command: &CommunicationProviderCommand,
    reason: String,
) -> ZulipCommandExecutionError {
    ZulipCommandExecutionError::InvalidCommand {
        command_id: command.command_id.clone(),
        reason,
    }
}

fn failure_result_payload(error: &ZulipCommandExecutionError) -> Value {
    json!({
        "provider": "zulip",
        "result": "error",
        "error_kind": error.error_kind(),
    })
}

#[derive(Debug, Error)]
pub enum ZulipCommandWorkerError {
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
    #[error("Zulip command `{command_id}` is invalid")]
    InvalidCommand { command_id: String },
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),
    #[error(transparent)]
    ProviderCommand(#[from] ProviderCommandQueuePortError),
}
