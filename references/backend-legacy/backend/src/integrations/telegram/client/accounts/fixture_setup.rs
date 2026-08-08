use serde_json::json;

use makosh_communications_api::accounts::NewProviderAccount;

use super::super::errors::TelegramError;
use super::super::models::accounts::{TelegramAccountSetupRequest, TelegramAccountSetupResponse};
use super::super::store::TelegramStore;

impl TelegramStore {
    pub async fn setup_fixture_account(
        &self,
        request: &TelegramAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        request.validate()?;
        let provider_kind = request.provider_kind;
        if !provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(
                "provider_kind must be telegram_user or telegram_bot".to_owned(),
            ));
        }

        let account = NewProviderAccount::new(
            &request.account_id,
            provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "tdlib_data_path": request.tdlib_data_path,
            "transcription_enabled": request.transcription_enabled,
        }));
        let stored_account = self
            .provider_account_store()
            .upsert(&account)
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?;

        Ok(TelegramAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            transcription_enabled: request.transcription_enabled,
            credential_bindings: vec![],
        })
    }
}
