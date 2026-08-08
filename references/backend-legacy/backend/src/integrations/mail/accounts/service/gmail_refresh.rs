use chrono::{TimeDelta, Utc};
use serde_json::json;

use crate::platform::secrets::models::{ResolvedSecret, SecretKind};
use crate::platform::secrets::resolver::SecretResolver;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;

use super::super::errors::EmailAccountSetupError;
use super::super::helpers::expires_at;
use super::super::models::GmailOAuthTokenBundle;
use super::super::validation::validate_non_empty;
use super::super::vault::SecretWriteContext;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub async fn refresh_gmail_access_token(
        &self,
        secret_ref: &str,
    ) -> Result<ResolvedSecret, EmailAccountSetupError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let reference = self
            .vault
            .secret_reference(secret_ref, SecretKind::OauthToken);
        let resolved = self.vault.resolve(&reference).await?;
        let mut bundle: GmailOAuthTokenBundle =
            serde_json::from_str(resolved.expose_for_runtime())?;

        if bundle.expires_at > Utc::now() + TimeDelta::seconds(60) {
            return ResolvedSecret::new(bundle.access_token)
                .map_err(EmailAccountSetupError::Secret);
        }

        let refreshed = self.refresh_token(&bundle).await?;
        bundle.access_token = refreshed.access_token;
        if let Some(refresh_token) = refreshed.refresh_token {
            bundle.refresh_token = refresh_token;
        }
        bundle.expires_at = expires_at(refreshed.expires_in);
        bundle.token_type = refreshed.token_type;
        if refreshed.scope.is_some() {
            bundle.scope = refreshed.scope;
        }

        self.vault
            .store_secret(
                secret_ref,
                &serde_json::to_string(&bundle)?,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: secret_ref,
                    purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                    secret_kind: SecretKind::OauthToken,
                    label: "OAuth credential",
                    metadata: &json!({}),
                },
            )
            .await?;
        ResolvedSecret::new(bundle.access_token).map_err(EmailAccountSetupError::Secret)
    }
}
