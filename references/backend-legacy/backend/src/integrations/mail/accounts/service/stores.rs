use makosh_communications_api::accounts::ProviderAccountCommandPort;
use makosh_communications_api::accounts::ProviderSecretBindingCommandPort;
use std::sync::Arc;

use crate::platform::secrets::store::SecretReferenceStore;
use sqlx::postgres::PgPool;

use super::super::errors::EmailAccountSetupError;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub(in crate::integrations::mail::accounts::service) fn pool(
        &self,
    ) -> Result<&PgPool, EmailAccountSetupError> {
        self.pool
            .as_ref()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }

    pub(in crate::integrations::mail::accounts::service) fn secret_store(
        &self,
    ) -> Result<&SecretReferenceStore, EmailAccountSetupError> {
        self.secret_store
            .as_ref()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }

    pub(in crate::integrations::mail::accounts::service) fn provider_account_store(
        &self,
    ) -> Result<Arc<dyn ProviderAccountCommandPort>, EmailAccountSetupError> {
        self.provider_account_store
            .clone()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }

    pub(in crate::integrations::mail::accounts::service) fn provider_secret_binding_store(
        &self,
    ) -> Result<Arc<dyn ProviderSecretBindingCommandPort>, EmailAccountSetupError> {
        self.provider_secret_binding_store
            .clone()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }
}
