use super::account_support::*;
use super::*;
use crate::app::signal_hub_support::{
    remove_provider_account_signal_connection, sync_provider_account_signal_connection,
    sync_provider_account_signal_connection_with_status,
};
use crate::domains::communications::sensitive_forwarding::{
    SensitiveForwardingError, SensitiveForwardingPgStore,
};
use crate::platform::secrets::store::SecretReferenceStore;
use crate::workflows::mail_background_sync::models::{
    progress::MailSyncTrigger, settings::MailSyncSettingsUpdate,
};
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use makosh_communications_api::content_egress::AccountContentEgressPermissions;
use makosh_communications_api::sensitive_forwarding::NewSensitiveForwardingPolicy;

pub(crate) async fn get_v1_email_accounts(
    State(state): State<AppState>,
) -> Result<Json<EmailAccountListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let accounts = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationProviderAccountStore,
    >(pool)
    .list()
    .await?
    .into_iter()
    .filter(|account| account.provider_kind.is_email() && !account.is_deleted())
    .map(email_account_view)
    .collect();

    Ok(Json(EmailAccountListResponse { items: accounts }))
}

pub(crate) async fn get_v1_email_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountView>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    Ok(Json(email_account_view(account)))
}

pub(crate) async fn get_v1_email_account_export(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountExportResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let sanitized_account = ProviderAccount {
        config: sanitize_account_config(&account.config),
        ..account
    };

    Ok(Json(EmailAccountExportResponse {
        exported_at: Utc::now(),
        capabilities: email_account_capabilities(&sanitized_account),
        account: sanitized_account,
        sync_settings: settings,
    }))
}

pub(crate) async fn post_v1_email_account_import(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<EmailAccountLogoutResponse>, ApiError> {
    if contains_secret_material(&payload) {
        return Err(ApiError::InvalidCommunicationQuery(
            "account import payload must not contain secrets or secret references",
        ));
    }

    let request: EmailAccountImportRequest = serde_json::from_value(payload)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid account import payload"))?;
    let provider_kind = CommunicationProviderKind::try_from(request.account.provider_kind.as_str())
        .map_err(|_| ApiError::InvalidCommunicationQuery("unsupported email provider kind"))?;
    if !provider_kind.is_email() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider kind is not an email provider",
        ));
    }
    let config = if request.account.config.is_null() {
        json!({})
    } else {
        request.account.config
    };
    if !config.is_object() {
        return Err(ApiError::InvalidCommunicationQuery(
            "account config must be an object",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let account = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationProviderAccountStore,
    >(pool)
    .upsert(
        &makosh_communications_api::accounts::NewProviderAccount::new(
            request.account.account_id,
            provider_kind,
            request.account.display_name,
            request.account.external_account_id,
        )
        .config(config),
    )
    .await?;
    sync_provider_account_signal_connection(&state, &account, None).await?;

    let current_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let settings_update = request.sync_settings.map_or(
        MailSyncSettingsUpdate {
            sync_enabled: current_settings.sync_enabled,
            batch_size: current_settings.batch_size,
            poll_interval_seconds: current_settings.poll_interval_seconds,
            failure_threshold: current_settings.failure_threshold,
        },
        |settings| MailSyncSettingsUpdate {
            sync_enabled: settings
                .sync_enabled
                .unwrap_or(current_settings.sync_enabled),
            batch_size: settings.batch_size.unwrap_or(current_settings.batch_size),
            poll_interval_seconds: settings
                .poll_interval_seconds
                .unwrap_or(current_settings.poll_interval_seconds),
            failure_threshold: settings
                .failure_threshold
                .unwrap_or(current_settings.failure_threshold),
        },
    );
    let sync_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .update_settings(&account.account_id, settings_update)
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(EmailAccountLogoutResponse {
        capabilities: email_account_capabilities(&account),
        account,
        sync_settings,
    }))
}

pub(crate) async fn post_v1_email_account_logout(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountLogoutResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let updated_account = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationProviderAccountStore,
    >(pool)
    .mark_logged_out(&account.account_id)
    .await?
    .ok_or(ApiError::NotFound)?;
    sync_provider_account_signal_connection_with_status(
        &state,
        &updated_account,
        "disconnected",
        None,
    )
    .await?;
    let current_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let sync_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .update_settings(
            &account.account_id,
            MailSyncSettingsUpdate {
                sync_enabled: false,
                batch_size: current_settings.batch_size,
                poll_interval_seconds: current_settings.poll_interval_seconds,
                failure_threshold: current_settings.failure_threshold,
            },
        )
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(EmailAccountLogoutResponse {
        capabilities: email_account_capabilities(&updated_account),
        account: updated_account,
        sync_settings,
    }))
}

pub(crate) async fn delete_v1_email_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountDeleteResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationProviderAccountStore,
    >(pool.clone());
    let usage = store.usage(&account.account_id).await?;
    let binding_store = makosh_communications_postgres::provider_store::
        CommunicationProviderSecretBindingStore::new(pool.clone());
    if binding_store
        .account_has_host_vault_secret_refs(&account.account_id)
        .await?
    {
        require_unlocked_host_vault(&state)?;
    }

    let deleted = if usage.has_retained_evidence() {
        store.delete_access_metadata(&account.account_id).await?
    } else {
        store.delete_metadata(&account.account_id).await?
    };
    remove_provider_account_signal_connection(&state, &account).await?;
    let (vault_deleted_secret_refs, retained_secret_refs) =
        delete_unbound_host_vault_secrets(&state, &pool, &deleted.unbound_secret_refs).await?;

    Ok(Json(EmailAccountDeleteResponse {
        account_id: account.account_id,
        deleted: deleted.account.is_some(),
        unbound_secret_refs: deleted.unbound_secret_refs,
        vault_deleted_secret_refs,
        retained_secret_refs,
    }))
}

async fn delete_unbound_host_vault_secrets(
    state: &AppState,
    pool: &sqlx::postgres::PgPool,
    secret_refs: &[String],
) -> Result<(Vec<String>, Vec<String>), ApiError> {
    let secret_store = SecretReferenceStore::new(pool.clone());
    let binding_store = makosh_communications_postgres::provider_store::
        CommunicationProviderSecretBindingStore::new(pool.clone());
    let mut vault_deleted_secret_refs = Vec::new();
    let mut retained_secret_refs = Vec::new();

    for secret_ref in secret_refs {
        if binding_store.is_secret_ref_bound(secret_ref).await? {
            retained_secret_refs.push(secret_ref.clone());
            continue;
        }

        let Some(reference) = secret_store
            .secret_reference(secret_ref)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = %error,
                    secret_ref = %secret_ref,
                    "failed to load provider account secret reference metadata"
                );
                ApiError::FailedPrecondition(
                    "failed to load provider account secret reference metadata".to_owned(),
                )
            })?
        else {
            retained_secret_refs.push(secret_ref.clone());
            continue;
        };
        if reference.store_kind != SecretStoreKind::HostVault {
            retained_secret_refs.push(secret_ref.clone());
            continue;
        }

        state.vault.delete_secret(secret_ref)?;
        secret_store
            .delete_secret_reference(secret_ref)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = %error,
                    secret_ref = %secret_ref,
                    "failed to delete provider account secret reference metadata"
                );
                ApiError::FailedPrecondition(
                    "failed to delete provider account secret reference metadata".to_owned(),
                )
            })?;
        vault_deleted_secret_refs.push(secret_ref.clone());
    }

    Ok((vault_deleted_secret_refs, retained_secret_refs))
}

pub(crate) async fn get_v1_email_account_sync_status(
    State(state): State<AppState>,
) -> Result<Json<MailSyncStatusListResponse>, ApiError> {
    let statuses = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .sync_statuses()
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(MailSyncStatusListResponse { items: statuses }))
}

pub(crate) async fn get_v1_email_account_sync_settings(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncSettings>, ApiError> {
    Ok(Json(
        mail_sync_store(&state)
            .map_err(mail_sync_api_error)?
            .settings_for_account(&account_id)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn put_v1_email_account_sync_settings(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<MailSyncSettingsPatch>,
) -> Result<Json<MailSyncSettings>, ApiError> {
    let store = mail_sync_store(&state).map_err(mail_sync_api_error)?;
    let current = store
        .settings_for_account(&account_id)
        .await
        .map_err(mail_sync_api_error)?;
    Ok(Json(
        store
            .update_settings(
                &account_id,
                MailSyncSettingsUpdate {
                    sync_enabled: request.sync_enabled.unwrap_or(current.sync_enabled),
                    batch_size: request.batch_size.unwrap_or(current.batch_size),
                    poll_interval_seconds: request
                        .poll_interval_seconds
                        .unwrap_or(current.poll_interval_seconds),
                    failure_threshold: request
                        .failure_threshold
                        .unwrap_or(current.failure_threshold),
                },
            )
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn get_v1_email_account_content_egress_settings(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailContentEgressSettings>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let permissions = AccountContentEgressPermissions::from_account_config(&account.config);
    Ok(Json(MailContentEgressSettings {
        body: permissions.body,
        attachments: permissions.attachments,
        extracted_text: permissions.extracted_text,
    }))
}

pub(crate) async fn put_v1_email_account_content_egress_settings(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<MailContentEgressSettingsPatch>,
) -> Result<Json<MailContentEgressSettings>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let current = AccountContentEgressPermissions::from_account_config(&account.config);
    let next = MailContentEgressSettings {
        body: request.body.unwrap_or(current.body),
        attachments: request.attachments.unwrap_or(current.attachments),
        extracted_text: request.extracted_text.unwrap_or(current.extracted_text),
    };
    let mut config = account.config;
    config["content_egress"] = json!({ "body": next.body, "attachments": next.attachments, "extracted_text": next.extracted_text });
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::stores::domain_stores::app_store::<CommunicationProviderAccountStore>(
        pool,
    )
    .update_config(&account_id, &config)
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(next))
}

pub(crate) async fn get_v1_mail_sensitive_forwarding_policies(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSensitiveForwardingPolicyListResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let policies = sensitive_forwarding_store(&state)?
        .policies_for_source_account(&account.account_id)
        .await
        .map_err(sensitive_forwarding_api_error)?;
    Ok(Json(MailSensitiveForwardingPolicyListResponse {
        items: policies,
    }))
}

pub(crate) async fn post_v1_mail_sensitive_forwarding_policy(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<MailSensitiveForwardingPolicyUpsertRequest>,
) -> Result<Json<MailSensitiveForwardingPolicyListResponse>, ApiError> {
    let source_account = email_account_or_not_found(&state, &account_id).await?;
    let delivery_account = email_account_or_not_found(&state, &request.delivery_account_id).await?;
    let policy_id = request
        .policy_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("mail-sensitive-forwarding:{}", uuid::Uuid::new_v4()));
    let policy = NewSensitiveForwardingPolicy {
        policy_id,
        source_account_id: source_account.account_id.clone(),
        delivery_account_id: delivery_account.account_id,
        name: request.name,
        enabled: request.enabled,
        include_message_body: request.include_message_body,
        include_attachments: request.include_attachments,
        fixed_recipients: request.fixed_recipients,
        minimum_severity: request.minimum_severity,
        subject_template: request.subject_template,
        body_template: request.body_template,
        max_sends_per_hour: request.max_sends_per_hour,
        quiet_hours: request.quiet_hours,
        expires_at: request.expires_at,
    };
    let store = sensitive_forwarding_store(&state)?;
    store
        .upsert_policy(&policy)
        .await
        .map_err(sensitive_forwarding_api_error)?;
    let policies = store
        .policies_for_source_account(&source_account.account_id)
        .await
        .map_err(sensitive_forwarding_api_error)?;
    Ok(Json(MailSensitiveForwardingPolicyListResponse {
        items: policies,
    }))
}

pub(crate) async fn delete_v1_mail_sensitive_forwarding_policy(
    State(state): State<AppState>,
    Path((account_id, policy_id)): Path<(String, String)>,
) -> Result<Json<MailSensitiveForwardingPolicyDeleteResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let deleted = sensitive_forwarding_store(&state)?
        .delete_policy(&account.account_id, &policy_id)
        .await
        .map_err(sensitive_forwarding_api_error)?;
    Ok(Json(MailSensitiveForwardingPolicyDeleteResponse {
        policy_id,
        deleted,
    }))
}

fn sensitive_forwarding_store(state: &AppState) -> Result<SensitiveForwardingPgStore, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(SensitiveForwardingPgStore::new(pool))
}

fn sensitive_forwarding_api_error(error: SensitiveForwardingError) -> ApiError {
    match error {
        SensitiveForwardingError::AccountNotFound => ApiError::NotFound,
        SensitiveForwardingError::Invalid
        | SensitiveForwardingError::PolicyNotFound
        | SensitiveForwardingError::SourceAccountMismatch => {
            ApiError::InvalidCommunicationQuery("invalid sensitive forwarding policy")
        }
        error => {
            tracing::error!(error = %error, "sensitive forwarding policy storage failed");
            ApiError::InvalidCommunicationQuery("sensitive forwarding policy is unavailable")
        }
    }
}

pub(crate) async fn post_v1_email_account_sync_now(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncRunResponse>, ApiError> {
    Ok(Json(
        mail_sync_service(&state)
            .map_err(mail_sync_api_error)?
            .run_account(&account_id, MailSyncTrigger::Manual)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn post_v1_email_account_address_book_sync_now(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<AddressBookSyncRunResponse>, ApiError> {
    let _account = email_account_or_not_found(&state, &account_id).await?;
    let report = address_book_sync_service(&state)
        .map_err(address_book_sync_api_error)?
        .run_account(&account_id, AddressBookSyncTrigger::Manual)
        .await
        .map_err(address_book_sync_api_error)?;

    Ok(Json(report.response()))
}

pub(crate) async fn post_v1_email_account_sync_full_resync(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncRunResponse>, ApiError> {
    Ok(Json(
        mail_sync_service(&state)
            .map_err(mail_sync_api_error)?
            .run_account_full_resync(&account_id)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}
