use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};

use crate::platform::secrets::models::{NewSecretReference, SecretKind};

use super::super::errors::EmailAccountSetupError;
use super::super::helpers::{expires_at, oauth_secret_ref};
use super::super::models::{
    EmailAccountSetupResult, GmailOAuthPendingGrant, GmailOAuthTokenBundle,
};
use super::super::vault::SecretWriteContext;
use super::EmailAccountSetupService;
use super::gmail_payloads::{gmail_account_config, gmail_secret_metadata};

impl EmailAccountSetupService {
    pub async fn complete_gmail_oauth(
        &self,
        pending: GmailOAuthPendingGrant,
        authorization_code: &str,
    ) -> Result<EmailAccountSetupResult, EmailAccountSetupError> {
        super::super::validation::validate_non_empty("authorization_code", authorization_code)?;
        let external_account_id = if pending.request.external_account_id.trim().is_empty() {
            pending.account_id.clone()
        } else {
            pending.request.external_account_id.clone()
        };
        let token = self
            .exchange_authorization_code(&pending, authorization_code)
            .await?;
        let refresh_token =
            token
                .refresh_token
                .clone()
                .ok_or(EmailAccountSetupError::MissingProviderField {
                    field: "refresh_token",
                })?;
        let expires_at = expires_at(token.expires_in);
        let provider_account_store = self.provider_account_store()?;
        let account_id = provider_account_store
            .list()
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?
            .into_iter()
            .find(|account| {
                account.provider_kind == CommunicationProviderKind::Gmail
                    && account
                        .external_account_id
                        .trim()
                        .eq_ignore_ascii_case(external_account_id.trim())
            })
            .map(|account| account.account_id)
            .unwrap_or_else(|| pending.account_id.clone());
        let secret_ref = oauth_secret_ref(&account_id);
        let token_bundle = GmailOAuthTokenBundle {
            token_url: pending.request.token_endpoint.clone(),
            client_id: pending.request.client_id.clone(),
            client_secret: pending.request.client_secret.clone(),
            access_token: token.access_token,
            refresh_token,
            expires_at,
            token_type: token.token_type,
            scope: token.scope,
        };
        let secret_store = self.secret_store()?;
        let secret_binding_store = self.provider_secret_binding_store()?;
        let account_config = gmail_account_config(&pending);
        let secret_metadata =
            gmail_secret_metadata(&pending, &account_id, &external_account_id, &account_config);
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    SecretKind::OauthToken,
                    self.vault.store_kind(),
                    format!(
                        "Gmail OAuth credential for {}",
                        pending.request.display_name
                    ),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        self.vault
            .store_secret(
                &secret_ref,
                &serde_json::to_string(&token_bundle)?,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &account_id,
                    purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                    secret_kind: SecretKind::OauthToken,
                    label: "Gmail OAuth credential",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        provider_account_store
            .upsert(
                &NewProviderAccount::new(
                    &account_id,
                    CommunicationProviderKind::Gmail,
                    &pending.request.display_name,
                    &external_account_id,
                )
                .config(account_config),
            )
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;
        secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &account_id,
                ProviderAccountSecretPurpose::OauthToken,
                &secret_ref,
            ))
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;

        Ok(EmailAccountSetupResult {
            account_id,
            secret_ref,
            secret_kind: SecretKind::OauthToken,
            store_kind: self.vault.store_kind(),
        })
    }
}
