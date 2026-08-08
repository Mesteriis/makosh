use super::contracts::WhatsAppProviderRuntime;
use super::{
    WhatsappWebError, WhatsappWebStore, account_provider_shape,
    provider_shape_restorable_secret_purpose,
};
use crate::platform::secrets::models::SecretStoreKind;
use crate::platform::secrets::resolver::SecretResolver;
use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;
use makosh_communications_api::accounts::ProviderAccountSecretBinding;

impl WhatsappWebStore {
    pub(super) async fn optional_restored_session_secret_ref(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<Option<String>, WhatsappWebError> {
        let account = self.whatsapp_account(account_id).await?;
        let purpose = provider_shape_restorable_secret_purpose(account_provider_shape(
            &account,
            self.provider_shape(),
        ));
        let binding = self
            .provider_secret_binding_store()
            .get_for_account(account_id, purpose)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
        let Some(binding) = binding else {
            return Ok(None);
        };
        let reference = secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp session secret reference metadata not found: {}",
                    binding.secret_ref
                ))
            })?;
        if !binding
            .secret_purpose
            .accepts_secret_kind(reference.secret_kind)
        {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "secret_kind `{}` is incompatible with {}",
                reference.secret_kind.as_str(),
                binding.secret_purpose.as_str()
            )));
        }
        if vault
            .resolve(&reference)
            .await?
            .expose_for_runtime()
            .trim()
            .is_empty()
        {
            return Err(WhatsappWebError::InvalidRequest(
                "WhatsApp session material must not be empty".to_owned(),
            ));
        }
        Ok(Some(reference.secret_ref))
    }

    pub(super) async fn clear_session_secret_material(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<Vec<String>, WhatsappWebError> {
        let account = self.whatsapp_account(account_id).await?;
        let purpose = provider_shape_restorable_secret_purpose(account_provider_shape(
            &account,
            self.provider_shape(),
        ));
        let binding = self
            .provider_secret_binding_store()
            .get_for_account(account_id, purpose)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
        self.clear_secret_material_for_bindings(secret_store, vault, vec![binding], account_id)
            .await
    }

    pub(super) async fn clear_account_secret_material(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        account_id: &str,
    ) -> Result<Vec<String>, WhatsappWebError> {
        let bindings = self
            .provider_secret_binding_store()
            .list_for_account(account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .into_iter()
            .map(Some)
            .collect();
        self.clear_secret_material_for_bindings(secret_store, vault, bindings, account_id)
            .await
    }

    async fn clear_secret_material_for_bindings(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        bindings: Vec<Option<ProviderAccountSecretBinding>>,
        account_id: &str,
    ) -> Result<Vec<String>, WhatsappWebError> {
        let mut removed = Vec::new();
        for binding in bindings.into_iter().flatten() {
            let reference = secret_store.secret_reference(&binding.secret_ref).await?;
            self.provider_secret_binding_store()
                .unbind_for_account(account_id, binding.secret_purpose)
                .await
                .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;
            secret_store
                .delete_secret_reference(&binding.secret_ref)
                .await?;
            if matches!(
                reference.as_ref().map(|entry| entry.store_kind),
                Some(SecretStoreKind::HostVault) | None
            ) {
                vault.delete_secret(&binding.secret_ref)?;
            }
            removed.push(binding.secret_ref);
        }
        Ok(removed)
    }
}
