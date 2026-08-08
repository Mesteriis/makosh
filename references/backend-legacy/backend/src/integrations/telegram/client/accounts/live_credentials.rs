use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use serde_json::{Value, json};

use crate::platform::secrets::models::SecretKind;

use super::super::errors::TelegramError;
use super::super::models::accounts::{TelegramCredentialBinding, TelegramLiveAccountSetupRequest};
use super::super::store::TelegramStore;
use super::super::validation::required_optional_value;
use super::super::vault::{TelegramCredentialWrite, TelegramSecretVault};

impl TelegramStore {
    pub(in crate::integrations::telegram::client::accounts) async fn store_live_account_credentials(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
        account_config: &Value,
    ) -> Result<Vec<TelegramCredentialBinding>, TelegramError> {
        let mut credential_bindings = Vec::new();
        let metadata = telegram_secret_metadata(request, account_config);
        match request.provider_kind {
            CommunicationProviderKind::TelegramUser => {
                if !request.is_qr_authorized_user_account() {
                    credential_bindings.push(
                        self.store_account_credential(
                            secret_store,
                            vault,
                            TelegramCredentialWrite {
                                account_id: &request.account_id,
                                provider_kind: request.provider_kind,
                                secret_purpose: ProviderAccountSecretPurpose::TelegramApiHash,
                                secret_kind: SecretKind::ApiToken,
                                label: "Telegram API hash",
                                value: required_optional_value(
                                    "api_hash",
                                    request.api_hash.as_deref(),
                                )?,
                                metadata: metadata.clone(),
                            },
                        )
                        .await?,
                    );
                }
                if let Some(binding) = self
                    .store_session_key(secret_store, vault, request, &metadata)
                    .await?
                {
                    credential_bindings.push(binding);
                }
            }
            CommunicationProviderKind::TelegramBot => {
                credential_bindings.push(
                    self.store_account_credential(
                        secret_store,
                        vault,
                        TelegramCredentialWrite {
                            account_id: &request.account_id,
                            provider_kind: request.provider_kind,
                            secret_purpose: ProviderAccountSecretPurpose::TelegramBotToken,
                            secret_kind: SecretKind::ApiToken,
                            label: "Telegram bot token",
                            value: required_optional_value(
                                "bot_token",
                                request.bot_token.as_deref(),
                            )?,
                            metadata,
                        },
                    )
                    .await?,
                );
            }
            _ => unreachable!("validated provider kind must be Telegram"),
        }

        Ok(credential_bindings)
    }

    async fn store_session_key(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
        metadata: &Value,
    ) -> Result<Option<TelegramCredentialBinding>, TelegramError> {
        let Some(session_encryption_key) = request
            .session_encryption_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        self.store_account_credential(
            secret_store,
            vault,
            TelegramCredentialWrite {
                account_id: &request.account_id,
                provider_kind: request.provider_kind,
                secret_purpose: ProviderAccountSecretPurpose::TelegramSessionKey,
                secret_kind: SecretKind::Other,
                label: "Telegram session encryption key",
                value: session_encryption_key.to_owned(),
                metadata: metadata.clone(),
            },
        )
        .await
        .map(Some)
    }
}

fn telegram_secret_metadata(
    request: &TelegramLiveAccountSetupRequest,
    account_config: &Value,
) -> Value {
    let connected_services = match request.provider_kind {
        CommunicationProviderKind::TelegramUser => json!(["messages", "contacts"]),
        CommunicationProviderKind::TelegramBot => json!(["messages"]),
        _ => json!([]),
    };

    json!({
        "provider": request.provider_kind.as_str(),
        "account_id": request.account_id,
        "display_name": request.display_name,
        "external_account_id": request.external_account_id,
        "connected_services": connected_services,
        "provider_account_config": account_config
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn telegram_user_secret_metadata_contains_restore_payload() {
        let request = TelegramLiveAccountSetupRequest {
            account_id: "telegram-user-42".to_owned(),
            provider_kind: CommunicationProviderKind::TelegramUser,
            display_name: "Telegram Personal".to_owned(),
            external_account_id: "telegram:42".to_owned(),
            api_id: None,
            api_hash: None,
            bot_token: None,
            session_encryption_key: Some("session-key".to_owned()),
            tdlib_data_path: Some("/tmp/makosh-tdlib/user-42".to_owned()),
            qr_authorized: true,
            transcription_enabled: true,
        };
        let account_config = json!({
            "runtime": "tdlib_qr_authorized",
            "tdlib_data_path": "/tmp/makosh-tdlib/user-42",
            "transcription_enabled": true
        });

        let metadata = telegram_secret_metadata(&request, &account_config);

        assert_eq!(metadata["provider"], json!("telegram_user"));
        assert_eq!(metadata["account_id"], json!("telegram-user-42"));
        assert_eq!(metadata["display_name"], json!("Telegram Personal"));
        assert_eq!(metadata["external_account_id"], json!("telegram:42"));
        assert_eq!(
            metadata["connected_services"],
            json!(["messages", "contacts"])
        );
        assert_eq!(metadata["provider_account_config"], account_config);
    }
}
