use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountCommandPort;
use makosh_communications_api::accounts::ProviderSecretBindingCommandPort;
use std::sync::Arc;

use crate::platform::secrets::database_vault::DatabaseEncryptedSecretVault;
use crate::vault::HostVault;
use sqlx::postgres::PgPool;

use super::super::helpers::http_client;
use super::super::vault::AccountSecretVault;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub fn new(
        pool: PgPool,
        secret_store: SecretReferenceStore,
        vault: DatabaseEncryptedSecretVault,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    ) -> Self {
        Self {
            pool: Some(pool),
            secret_store: Some(secret_store),
            provider_account_store: Some(provider_account_store),
            provider_secret_binding_store: Some(provider_secret_binding_store),
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_for_vault_only(vault: DatabaseEncryptedSecretVault) -> Self {
        Self {
            pool: None,
            secret_store: None,
            provider_account_store: None,
            provider_secret_binding_store: None,
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_with_host_vault(
        pool: PgPool,
        secret_store: SecretReferenceStore,
        vault: HostVault,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    ) -> Self {
        Self {
            pool: Some(pool),
            secret_store: Some(secret_store),
            provider_account_store: Some(provider_account_store),
            provider_secret_binding_store: Some(provider_secret_binding_store),
            vault: AccountSecretVault::Host(vault),
            http: http_client(),
        }
    }

    pub fn new_with_host_vault_for_token_refresh(
        pool: PgPool,
        secret_store: SecretReferenceStore,
        vault: HostVault,
    ) -> Self {
        Self {
            pool: Some(pool),
            secret_store: Some(secret_store),
            provider_account_store: None,
            provider_secret_binding_store: None,
            vault: AccountSecretVault::Host(vault),
            http: http_client(),
        }
    }
}
