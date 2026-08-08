use crate::errors::CommunicationIngestionError;
use crate::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use crate::store::CommunicationIngestionStore;
use makosh_communications_api::accounts::ProviderAccountUsage;
use makosh_communications_api::accounts::{
    DeletedProviderAccount, NewProviderAccount, NewProviderAccountSecretBinding, ProviderAccount,
    ProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};

use serde_json::Value;

impl CommunicationIngestionStore {
    pub async fn upsert_provider_account(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .upsert(account)
            .await
    }

    pub async fn provider_account(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .get(account_id)
            .await
    }

    pub async fn list_provider_accounts(
        &self,
    ) -> Result<Vec<ProviderAccount>, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .list()
            .await
    }

    pub async fn update_provider_account_config(
        &self,
        account_id: &str,
        config: &Value,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .update_config(account_id, config)
            .await
    }

    pub async fn provider_account_usage(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccountUsage, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .usage(account_id)
            .await
    }

    pub async fn delete_provider_account_metadata(
        &self,
        account_id: &str,
    ) -> Result<DeletedProviderAccount, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .delete_metadata(account_id)
            .await
    }

    pub async fn bind_provider_account_secret(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .bind(binding)
            .await
    }

    pub async fn provider_account_secret_bindings(
        &self,
        account_id: &str,
    ) -> Result<Vec<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .list_for_account(account_id)
            .await
    }

    pub async fn provider_account_secret_binding(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<Option<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .get_for_account(account_id, secret_purpose)
            .await
    }
}
