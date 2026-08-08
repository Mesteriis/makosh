use serde_json::Value;

use super::super::models::ZoomAccount;
use super::super::models::ZoomOAuthTokenBundle;
use super::{ZoomError, ZoomStore};
use crate::platform::secrets::models::{NewSecretReference, SecretKind, SecretStoreKind};
use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;
use crate::vault::models::SecretEntryContext;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;

impl ZoomStore {
    pub(super) async fn store_oauth_token_bundle(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account: &ZoomAccount,
        secret_ref: &str,
        token_bundle: &ZoomOAuthTokenBundle,
        metadata: &Value,
    ) -> Result<(), ZoomError> {
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    secret_ref,
                    SecretKind::OauthToken,
                    SecretStoreKind::HostVault,
                    format!("Zoom OAuth credential for {}", account.account_id),
                )
                .metadata(metadata.clone()),
            )
            .await?;
        vault.store_secret(
            secret_ref,
            &serde_json::to_string(token_bundle)?,
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: &account.account_id,
                purpose: ProviderAccountSecretPurpose::ZoomOauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "Zoom OAuth credential",
                metadata,
            },
        )?;
        Ok(())
    }
}
