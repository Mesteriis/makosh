use serde_json::{Value, json};

use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::NewProviderAccount;

use super::super::errors::TelegramError;
use super::super::models::accounts::{
    TelegramAccountSetupResponse, TelegramLiveAccountSetupRequest,
};
use super::super::store::TelegramStore;
use super::super::vault::TelegramSecretVault;

impl TelegramStore {
    pub async fn setup_live_blocked_account(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        request.validate()?;
        let provider_kind = request.provider_kind;
        if !provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(
                "provider_kind must be telegram_user or telegram_bot".to_owned(),
            ));
        }

        let runtime = live_runtime(request);
        let account_config = live_account_config(request, runtime);
        let stored_account = self
            .provider_account_store()
            .upsert(
                &NewProviderAccount::new(
                    &request.account_id,
                    provider_kind,
                    &request.display_name,
                    &request.external_account_id,
                )
                .config(account_config.clone()),
            )
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?;

        let credential_bindings = self
            .store_live_account_credentials(secret_store, vault, request, &account_config)
            .await?;

        Ok(TelegramAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: runtime.to_owned(),
            transcription_enabled: request.transcription_enabled,
            credential_bindings,
        })
    }
}

fn live_runtime(request: &TelegramLiveAccountSetupRequest) -> &'static str {
    if request.is_qr_authorized_user_account() {
        "tdlib_qr_authorized"
    } else {
        "live_blocked"
    }
}

fn live_account_config(request: &TelegramLiveAccountSetupRequest, runtime: &str) -> Value {
    let mut config = json!({
        "runtime": runtime,
        "transcription_enabled": request.transcription_enabled,
    });
    if let Some(object) = config.as_object_mut() {
        if let Some(tdlib_data_path) = request
            .tdlib_data_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            object.insert("tdlib_data_path".to_owned(), json!(tdlib_data_path));
        }
        if !request.is_qr_authorized_user_account()
            && let Some(api_id) = request.api_id
        {
            object.insert("api_id".to_owned(), json!(api_id));
        }
    }

    config
}
