use makosh_communications_api::accounts::ProviderAccountMutationOrigin;
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};

use crate::platform::audit::models::NewApiAuditRecord;

const GOOGLE_CONTACTS_WRITE_SCOPE: &str = "https://www.googleapis.com/auth/contacts";

use crate::platform::settings::models::ApplicationSetting;
use crate::workflows::mail_background_sync::store::MailSyncStore;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;

use crate::app::api_support::{
    platform_dtos::*,
    stores::{domain_stores::*, settings_vault::*},
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;

pub(crate) async fn get_application_settings(
    State(state): State<AppState>,
) -> Result<Json<ApplicationSettingsResponse>, ApiError> {
    let items = settings_store(&state)?.list_public_settings().await?;

    Ok(Json(ApplicationSettingsResponse { items }))
}

pub(crate) async fn get_application_settings_accounts(
    State(state): State<AppState>,
) -> Result<Json<ApplicationAccountsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let accounts = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationProviderAccountStore,
    >(pool.clone())
    .list()
    .await?
    .into_iter()
    .filter(|account| !account.is_deleted())
    .collect::<Vec<_>>();
    let mail_sync_error_codes = match crate::app::api_support::stores::domain_stores::app_store::<
        MailSyncStore,
    >(pool.clone())
    .sync_statuses()
    .await
    {
        Ok(statuses) => statuses
            .into_iter()
            .map(|status| (status.account_id, status.last_error_code))
            .collect::<HashMap<_, _>>(),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "failed to inspect mail sync statuses for provider account credential state"
            );
            HashMap::new()
        }
    };
    let mut items = Vec::with_capacity(accounts.len());
    for account in accounts {
        let credential_state = application_account_credential_state_from_sync_failure(
            account.provider_kind,
            mail_sync_error_codes
                .get(&account.account_id)
                .and_then(|error_code| error_code.as_deref()),
        );
        items.push(ApplicationAccountView {
            account,
            credential_state,
        });
    }

    Ok(Json(ApplicationAccountsResponse { items }))
}

fn application_account_credential_state_from_sync_failure(
    provider_kind: CommunicationProviderKind,
    last_error_code: Option<&str>,
) -> ApplicationAccountCredentialState {
    if provider_kind != CommunicationProviderKind::Gmail {
        return ApplicationAccountCredentialState::not_applicable();
    }

    if last_error_code == Some("oauth_refresh_failed") {
        ApplicationAccountCredentialState::expired()
    } else {
        ApplicationAccountCredentialState::valid()
    }
}

pub(crate) async fn patch_application_settings_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<ApplicationAccountUpdateRequest>,
) -> Result<Json<ProviderAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationProviderAccountStore,
    >(pool);
    let mut account = if let Some(display_name) = request
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        store
            .update_display_name(&account_id, display_name)
            .await?
            .ok_or(ApiError::NotFound)?
    } else {
        store.get(&account_id).await?.ok_or(ApiError::NotFound)?
    };

    if request.address_book_sync_enabled.is_some()
        || request.address_book_sync_direction.is_some()
        || request.address_book_remote_write_enabled.is_some()
    {
        let config = address_book_sync_config(
            &account,
            request.address_book_sync_enabled,
            request.address_book_sync_direction.as_deref(),
            request.address_book_remote_write_enabled,
        )?;
        account = store
            .update_config_with_origin(
                &account_id,
                &config,
                ProviderAccountMutationOrigin::LocalRuntime,
                "settings.provider_accounts.update_address_book_sync",
                "update_address_book_sync",
            )
            .await?
            .ok_or(ApiError::NotFound)?;
    } else if request
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "account update field is required",
        ));
    }

    Ok(Json(account))
}

fn address_book_sync_config(
    account: &ProviderAccount,
    enabled: Option<bool>,
    direction: Option<&str>,
    remote_write_enabled: Option<bool>,
) -> Result<Value, ApiError> {
    if !provider_account_supports_contacts(account) {
        return Err(ApiError::InvalidCommunicationQuery(
            "account contacts service is not available",
        ));
    }
    if account_config_string(account, "address_book_sync_unsupported_reason").is_some() {
        return Err(ApiError::InvalidCommunicationQuery(
            "account contacts service is disabled by provider adapter",
        ));
    }

    let mut config = account.config.clone();
    let Some(config_object) = config.as_object_mut() else {
        return Err(ApiError::InvalidCommunicationQuery(
            "account config must be an object",
        ));
    };
    if let Some(enabled) = enabled {
        config_object.insert("address_book_sync_enabled".to_owned(), json!(enabled));
    }
    if let Some(direction) = direction {
        let direction = direction.trim();
        if direction != "read_only" && direction != "bidirectional" {
            return Err(ApiError::InvalidCommunicationQuery(
                "address book sync direction must be read_only or bidirectional",
            ));
        }
        config_object.insert("address_book_sync_direction".to_owned(), json!(direction));
        if direction != "bidirectional" {
            config_object.insert("address_book_remote_write_enabled".to_owned(), json!(false));
        }
    }
    if let Some(remote_write_enabled) = remote_write_enabled {
        if remote_write_enabled {
            let direction_allows_write = direction == Some("bidirectional")
                || account_config_string(account, "address_book_sync_direction")
                    .is_some_and(|value| value == "bidirectional");
            if !direction_allows_write {
                return Err(ApiError::InvalidCommunicationQuery(
                    "address book remote write requires bidirectional sync",
                ));
            }
            if account.provider_kind != CommunicationProviderKind::Gmail {
                return Err(ApiError::InvalidCommunicationQuery(
                    "address book remote write is only supported for Gmail accounts",
                ));
            }
            if !provider_account_requested_scope(account, GOOGLE_CONTACTS_WRITE_SCOPE) {
                return Err(ApiError::InvalidCommunicationQuery(
                    "address book remote write requires Google Contacts write scope",
                ));
            }
        }
        config_object.insert(
            "address_book_remote_write_enabled".to_owned(),
            json!(remote_write_enabled),
        );
    }
    Ok(config)
}

fn provider_account_supports_contacts(account: &ProviderAccount) -> bool {
    account
        .config
        .get("connected_services")
        .and_then(Value::as_array)
        .is_some_and(|services| {
            services
                .iter()
                .any(|service| service.as_str() == Some("contacts"))
        })
}

fn account_config_string<'a>(account: &'a ProviderAccount, key: &str) -> Option<&'a str> {
    account.config.get(key).and_then(Value::as_str)
}

fn provider_account_requested_scope(account: &ProviderAccount, scope: &str) -> bool {
    account
        .config
        .get("requested_scopes")
        .and_then(Value::as_array)
        .is_some_and(|scopes| {
            scopes
                .iter()
                .filter_map(Value::as_str)
                .any(|value| value.trim() == scope)
        })
}

pub(crate) async fn put_application_setting(
    State(state): State<AppState>,
    Path(setting_key): Path<String>,
    Json(request): Json<ApplicationSettingUpdateRequest>,
) -> Result<Json<ApplicationSetting>, ApiError> {
    let actor_id = "makosh-frontend".to_string();

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::application_setting_set(
            &actor_id,
            &setting_key,
        ))
        .await?;
    let setting = settings_store(&state)?
        .update_setting_value(&setting_key, &request.value, &actor_id)
        .await?;

    Ok(Json(setting))
}

#[cfg(test)]
mod account_credential_state_tests {
    use super::*;

    #[test]
    fn gmail_oauth_credential_state_requests_reauth_after_refresh_failure() {
        let state = application_account_credential_state_from_sync_failure(
            CommunicationProviderKind::Gmail,
            Some("oauth_refresh_failed"),
        );

        assert_eq!(state, ApplicationAccountCredentialState::expired());
    }

    #[test]
    fn gmail_oauth_credential_state_does_not_request_reauth_without_refresh_failure() {
        let state = application_account_credential_state_from_sync_failure(
            CommunicationProviderKind::Gmail,
            Some("provider_network_error"),
        );

        assert_eq!(state, ApplicationAccountCredentialState::valid());
    }

    #[test]
    fn non_gmail_provider_credential_state_is_not_applicable() {
        let state = application_account_credential_state_from_sync_failure(
            CommunicationProviderKind::Icloud,
            Some("oauth_refresh_failed"),
        );

        assert_eq!(state, ApplicationAccountCredentialState::not_applicable());
    }
}
