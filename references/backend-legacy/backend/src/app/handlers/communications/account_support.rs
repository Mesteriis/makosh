use super::*;
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use makosh_communications_api::sensitive_forwarding::StoredSensitiveForwardingPolicy;

#[derive(Serialize)]
pub(crate) struct MailSyncStatusListResponse {
    pub(super) items: Vec<MailSyncStatus>,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountListResponse {
    pub(super) items: Vec<EmailAccountView>,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountView {
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountCapabilities {
    pub(super) read: bool,
    pub(super) sync: bool,
    pub(super) send: bool,
    pub(super) oauth: bool,
    pub(super) imap: bool,
    pub(super) smtp: bool,
    pub(super) mutate_flags: bool,
    pub(super) mutate_mailboxes: bool,
    pub(super) server_delete: bool,
    pub(super) provider_folders: bool,
    pub(super) local_trash: bool,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountExportResponse {
    pub(super) exported_at: DateTime<Utc>,
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
    pub(super) sync_settings: MailSyncSettings,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountLogoutResponse {
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
    pub(super) sync_settings: MailSyncSettings,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountDeleteResponse {
    pub(super) account_id: String,
    pub(super) deleted: bool,
    pub(super) unbound_secret_refs: Vec<String>,
    pub(super) vault_deleted_secret_refs: Vec<String>,
    pub(super) retained_secret_refs: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct EmailAccountImportRequest {
    pub(super) account: EmailAccountImportAccount,
    pub(super) sync_settings: Option<EmailAccountImportSyncSettings>,
}

#[derive(Deserialize)]
pub(crate) struct EmailAccountImportAccount {
    pub(super) account_id: String,
    pub(super) provider_kind: String,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    #[serde(default)]
    pub(super) config: Value,
}

#[derive(Deserialize)]
pub(crate) struct EmailAccountImportSyncSettings {
    pub(super) sync_enabled: Option<bool>,
    pub(super) batch_size: Option<i32>,
    pub(super) poll_interval_seconds: Option<i32>,
    pub(super) failure_threshold: Option<i32>,
}

#[derive(Deserialize)]
pub(crate) struct MailSyncSettingsPatch {
    pub(super) sync_enabled: Option<bool>,
    pub(super) batch_size: Option<i32>,
    pub(super) poll_interval_seconds: Option<i32>,
    pub(super) failure_threshold: Option<i32>,
}

#[derive(Serialize)]
pub(crate) struct MailContentEgressSettings {
    pub(super) body: bool,
    pub(super) attachments: bool,
    pub(super) extracted_text: bool,
}

#[derive(Deserialize)]
pub(crate) struct MailContentEgressSettingsPatch {
    pub(super) body: Option<bool>,
    pub(super) attachments: Option<bool>,
    pub(super) extracted_text: Option<bool>,
}

#[derive(Serialize)]
pub(crate) struct MailSensitiveForwardingPolicyListResponse {
    pub(super) items: Vec<StoredSensitiveForwardingPolicy>,
}

#[derive(Deserialize)]
pub(crate) struct MailSensitiveForwardingPolicyUpsertRequest {
    pub(super) policy_id: Option<String>,
    pub(super) delivery_account_id: String,
    pub(super) name: String,
    pub(super) enabled: bool,
    #[serde(default)]
    pub(super) include_message_body: bool,
    #[serde(default)]
    pub(super) include_attachments: bool,
    pub(super) fixed_recipients: Vec<String>,
    pub(super) minimum_severity: String,
    pub(super) subject_template: String,
    pub(super) body_template: String,
    pub(super) max_sends_per_hour: i32,
    #[serde(default = "empty_json_object")]
    pub(super) quiet_hours: Value,
    pub(super) expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub(crate) struct MailSensitiveForwardingPolicyDeleteResponse {
    pub(super) policy_id: String,
    pub(super) deleted: bool,
}

fn empty_json_object() -> Value {
    json!({})
}
pub(crate) async fn email_account_or_not_found(
    state: &AppState,
    account_id: &str,
) -> Result<ProviderAccount, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let Some(account) = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationProviderAccountStore,
    >(pool)
    .get(account_id)
    .await?
    else {
        return Err(ApiError::NotFound);
    };
    if !account.provider_kind.is_email() || account.is_deleted() {
        return Err(ApiError::NotFound);
    }

    Ok(account)
}

pub(crate) fn email_account_view(account: ProviderAccount) -> EmailAccountView {
    EmailAccountView {
        capabilities: email_account_capabilities(&account),
        account,
    }
}

pub(crate) fn email_account_capabilities(account: &ProviderAccount) -> EmailAccountCapabilities {
    let logged_out = account
        .config
        .get("auth_state")
        .and_then(Value::as_str)
        .is_some_and(|state| state == "logged_out");
    let smtp = smtp_configured(account);
    let imap = matches!(
        account.provider_kind,
        CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap
    );
    let oauth = matches!(account.provider_kind, CommunicationProviderKind::Gmail)
        || account
            .config
            .get("auth")
            .and_then(Value::as_str)
            .is_some_and(|auth| auth == "oauth");
    let gmail_send_enabled = account
        .config
        .get("gmail_send_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let gmail_modify_enabled = account
        .config
        .get("gmail_modify_enabled")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| {
            account
                .config
                .get("requested_scopes")
                .and_then(Value::as_array)
                .is_some_and(|scopes| {
                    scopes.iter().any(|scope| {
                        scope.as_str() == Some("https://www.googleapis.com/auth/gmail.modify")
                    })
                })
        });
    let provider_mutations_enabled = !logged_out && (imap || gmail_modify_enabled);

    EmailAccountCapabilities {
        read: !logged_out,
        sync: !logged_out,
        send: !logged_out && (smtp || gmail_send_enabled),
        oauth,
        imap,
        smtp,
        mutate_flags: provider_mutations_enabled,
        mutate_mailboxes: provider_mutations_enabled,
        server_delete: false,
        provider_folders: provider_mutations_enabled,
        local_trash: true,
    }
}

pub(crate) fn smtp_configured(account: &ProviderAccount) -> bool {
    let explicit_smtp = account
        .config
        .as_object()
        .is_some_and(has_explicit_smtp_config);

    explicit_smtp
        || (matches!(account.provider_kind, CommunicationProviderKind::Icloud)
            && !account.external_account_id.trim().is_empty())
}

fn has_explicit_smtp_config(object: &serde_json::Map<String, Value>) -> bool {
    object
        .get("smtp_host")
        .and_then(Value::as_str)
        .is_some_and(|host| !host.trim().is_empty())
        && object.get("smtp_port").and_then(Value::as_i64).is_some()
}

pub(crate) fn sanitize_account_config(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter_map(|(key, value)| {
                    if is_secret_config_key(key) {
                        None
                    } else {
                        Some((key.clone(), sanitize_account_config(value)))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(sanitize_account_config).collect()),
        other => other.clone(),
    }
}

pub(crate) fn contains_secret_material(value: &Value) -> bool {
    match value {
        Value::Object(object) => object
            .iter()
            .any(|(key, value)| is_secret_config_key(key) || contains_secret_material(value)),
        Value::Array(items) => items.iter().any(contains_secret_material),
        _ => false,
    }
}

pub(crate) fn is_secret_config_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "password",
        "secret",
        "secret_ref",
        "token",
        "credential",
        "api_key",
        "private_key",
        "client_secret",
        "refresh_token",
        "access_token",
    ]
    .iter()
    .any(|marker| key.contains(marker))
}

pub(crate) fn require_unlocked_host_vault(state: &AppState) -> Result<(), ApiError> {
    match state.vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(ApiError::HostVault(HostVaultError::Locked)),
        VaultMode::Uninitialized => Err(ApiError::HostVault(HostVaultError::Uninitialized)),
    }
}

pub(crate) fn mail_sync_store(state: &AppState) -> Result<MailSyncStore, MailSyncError> {
    let Some(pool) = state.database.pool() else {
        return Err(MailSyncError::InvalidSetting {
            field: "database",
            message: "DATABASE_URL is not configured",
        });
    };

    Ok(crate::app::api_support::stores::domain_stores::app_store::<
        MailSyncStore,
    >(pool.clone()))
}

pub(crate) fn mail_sync_service(
    state: &AppState,
) -> Result<MailBackgroundSyncService, MailSyncError> {
    let Some(pool) = state.database.pool() else {
        return Err(MailSyncError::InvalidSetting {
            field: "database",
            message: "DATABASE_URL is not configured",
        });
    };

    Ok(MailBackgroundSyncService::new(
        pool.clone(),
        state.vault.clone(),
        DEFAULT_MAIL_SYNC_BLOB_ROOT,
        std::sync::Arc::new(
            crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort::new(
                pool.clone(),
                state.vault.clone(),
                std::sync::Arc::new(crate::app::api_support::stores::domain_stores::app_store::<
                    makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore,
                >(pool.clone())),
                crate::workflows::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
            ),
        ),
        std::sync::Arc::new(
            crate::domains::communications::provider_resources::MailProviderResourceStore::new(
                pool.clone(),
            ),
        ),
        std::sync::Arc::new(
            makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        std::sync::Arc::new(
            makosh_communications_postgres::store::CommunicationIngestionStore::new(
                pool.clone(),
            ),
        ),
    ))
}

pub(crate) fn address_book_sync_service(
    state: &AppState,
) -> Result<AddressBookSyncService, AddressBookSyncError> {
    let Some(pool) = state.database.pool() else {
        return Err(AddressBookSyncError::DatabaseNotConfigured);
    };

    Ok(AddressBookSyncService::new(
        pool.clone(),
        std::sync::Arc::new(
            crate::integrations::mail::address_book_sync_provider::LiveAddressBookProviderSyncPort::new(
                pool.clone(),
                state.vault.clone(),
                std::sync::Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                    pool.clone(),
                )),
                crate::workflows::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
            ),
        ),
        std::sync::Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone())),
    ))
}

pub(crate) fn mail_sync_api_error(error: MailSyncError) -> ApiError {
    match error {
        MailSyncError::AccountNotFound => ApiError::NotFound,
        MailSyncError::RunAlreadyActive | MailSyncError::RunNotFound => {
            ApiError::InvalidCommunicationQuery("mail sync run is already active")
        }
        MailSyncError::InvalidSetting {
            field: "database", ..
        } => ApiError::DatabaseNotConfigured,
        MailSyncError::InvalidSetting { .. } => {
            ApiError::InvalidCommunicationQuery("invalid mail sync settings")
        }
        MailSyncError::Sqlx(error) => {
            tracing::error!(error = %error, "mail sync database operation failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
        MailSyncError::CommunicationEvidence(error) => {
            tracing::error!(error = %error, "mail sync communication evidence port failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
        MailSyncError::EmailSyncPlan(_) => {
            ApiError::FailedPrecondition("mail provider configuration is invalid".to_owned())
        }
        MailSyncError::ProviderSync(_) => {
            ApiError::FailedPrecondition("mail provider is temporarily unavailable".to_owned())
        }
        MailSyncError::ProviderAccount(error) => {
            tracing::error!(error = %error, "mail sync provider account port failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
        MailSyncError::EventEnvelope(error) => ApiError::InvalidEnvelope(error),
        MailSyncError::EventStore(error) => ApiError::Store(error),
        MailSyncError::ObservationPort(error) => {
            tracing::error!(error = %error, "mail sync observation store failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
    }
}

pub(crate) fn address_book_sync_api_error(error: AddressBookSyncError) -> ApiError {
    match error {
        AddressBookSyncError::AccountNotFound(_) => ApiError::NotFound,
        AddressBookSyncError::DatabaseNotConfigured => ApiError::DatabaseNotConfigured,
        AddressBookSyncError::Sqlx(error) => {
            tracing::error!(error = %error, "address book sync database operation failed");
            ApiError::InvalidCommunicationQuery("address book sync operation failed")
        }
        AddressBookSyncError::Communication(error) => {
            tracing::error!(error = %error, "address book sync communication store failed");
            ApiError::InvalidCommunicationQuery("address book sync operation failed")
        }
        AddressBookSyncError::ProviderAccount(error) => {
            tracing::error!(error = %error, "address book sync provider account port failed");
            ApiError::InvalidCommunicationQuery("address book sync operation failed")
        }
        AddressBookSyncError::PersonaCommand(error) => {
            tracing::error!(error = %error, "address book sync persona projection failed");
            ApiError::InvalidCommunicationQuery("address book sync operation failed")
        }
        AddressBookSyncError::Provider(error) => {
            tracing::warn!(error = %error, "address book sync provider operation failed");
            ApiError::InvalidCommunicationQuery("address book sync provider operation failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    fn email_account(provider_kind: CommunicationProviderKind, config: Value) -> ProviderAccount {
        ProviderAccount {
            account_id: "account:test".to_owned(),
            provider_kind,
            display_name: "Test account".to_owned(),
            external_account_id: "owner@example.com".to_owned(),
            config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn icloud_without_explicit_smtp_is_send_capable_via_default_smtp() {
        let account = email_account(CommunicationProviderKind::Icloud, json!({}));

        let capabilities = email_account_capabilities(&account);

        assert!(capabilities.smtp);
        assert!(capabilities.send);
    }

    #[test]
    fn icloud_with_non_object_config_is_send_capable_via_default_smtp() {
        let account = email_account(CommunicationProviderKind::Icloud, Value::Null);

        let capabilities = email_account_capabilities(&account);

        assert!(capabilities.smtp);
        assert!(capabilities.send);
    }

    #[test]
    fn imap_without_explicit_smtp_stays_read_only_for_send() {
        let account = email_account(CommunicationProviderKind::Imap, json!({}));

        let capabilities = email_account_capabilities(&account);

        assert!(!capabilities.smtp);
        assert!(!capabilities.send);
        assert!(capabilities.mutate_flags);
        assert!(capabilities.mutate_mailboxes);
        assert!(capabilities.provider_folders);
    }

    #[test]
    fn gmail_modify_scope_enables_provider_mutation_capabilities() {
        let account = email_account(
            CommunicationProviderKind::Gmail,
            json!({
                "requested_scopes": ["https://www.googleapis.com/auth/gmail.modify"],
            }),
        );

        let capabilities = email_account_capabilities(&account);

        assert!(capabilities.mutate_flags);
        assert!(capabilities.mutate_mailboxes);
        assert!(capabilities.provider_folders);
    }

    #[test]
    fn gmail_without_modify_scope_does_not_advertise_provider_mutations() {
        let account = email_account(
            CommunicationProviderKind::Gmail,
            json!({
                "requested_scopes": ["https://www.googleapis.com/auth/gmail.send"],
            }),
        );

        let capabilities = email_account_capabilities(&account);

        assert!(!capabilities.mutate_flags);
        assert!(!capabilities.mutate_mailboxes);
        assert!(!capabilities.provider_folders);
    }
}
