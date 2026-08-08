use super::super::models::ZoomAccount;
use super::super::models::ZoomOAuthTokenBundle;
use super::{ZoomError, ZoomStore};
use crate::platform::secrets::models::{SecretKind, SecretStoreKind};
use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_provider_zoom::protocol::ZoomAuthShape;

impl ZoomStore {
    pub(super) async fn load_token_bundle(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
    ) -> Result<ZoomOAuthTokenBundle, ZoomError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::ZoomOauthToken,
            )
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom account `{}` has no zoom_oauth_token secret binding",
                    account.account_id
                ))
            })?;
        let reference = secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom token secret reference `{}` was not found",
                    binding.secret_ref
                ))
            })?;
        if reference.secret_kind != SecretKind::OauthToken
            || reference.store_kind != SecretStoreKind::HostVault
        {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom token secret reference `{}` must be an oauth_token in host_vault",
                reference.secret_ref
            )));
        }
        Ok(serde_json::from_str(
            &vault.read_secret(&reference.secret_ref)?,
        )?)
    }

    pub(super) async fn ensure_webhook_secret_available(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
    ) -> Result<(), ZoomError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::ZoomWebhookSecret,
            )
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom account `{}` has no zoom_webhook_secret binding",
                    account.account_id
                ))
            })?;
        let reference = secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!(
                    "Zoom webhook secret reference `{}` was not found",
                    binding.secret_ref
                ))
            })?;
        if reference.secret_kind != SecretKind::ApiToken
            || reference.store_kind != SecretStoreKind::HostVault
        {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom webhook secret reference `{}` must be an api_token in host_vault",
                reference.secret_ref
            )));
        }
        let _ = vault.read_secret(&reference.secret_ref)?;
        Ok(())
    }

    pub(super) async fn load_webhook_management_access_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
    ) -> Result<String, ZoomError> {
        let bundle = self.load_token_bundle(secret_store, vault, account).await?;
        let client_secret_ref = bundle.client_secret_ref.clone().ok_or_else(|| {
            ZoomError::InvalidRequest(format!(
                "Zoom account `{}` has no client secret reference in token bundle",
                account.account_id
            ))
        })?;
        let client_secret = vault.read_secret(&client_secret_ref)?;
        let token = if account.auth_shape == ZoomAuthShape::ServerToServer.as_str() {
            let account_id = bundle
                .zoom_account_id
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| account.external_account_id.trim().to_owned());
            self.exchange_server_to_server_token(
                bundle.token_url.clone(),
                &bundle.client_id,
                &client_secret,
                &account_id,
            )
            .await?
        } else if account.auth_shape == ZoomAuthShape::OAuthUser.as_str() {
            self.exchange_client_credentials_token(
                bundle.token_url.clone(),
                &bundle.client_id,
                &client_secret,
            )
            .await?
        } else {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` is not a live account",
                account.account_id
            )));
        };
        let access_token = token.access_token.trim().to_owned();
        if access_token.is_empty() {
            return Err(ZoomError::InvalidRequest(format!(
                "Zoom account `{}` returned an empty webhook-management access token",
                account.account_id
            )));
        }
        Ok(access_token)
    }
}
