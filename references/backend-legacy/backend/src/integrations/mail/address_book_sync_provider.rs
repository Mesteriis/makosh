use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::{
    CommunicationProviderKind, ProviderSecretBindingLookupPort,
};
use makosh_communications_api::address_book::{
    AddressBookProviderBatch, AddressBookProviderEntry, AddressBookProviderFetchRequest,
    AddressBookProviderSyncError, AddressBookProviderSyncPort, AddressBookProviderUpsertRequest,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use sqlx::postgres::PgPool;

use crate::integrations::mail::accounts::service::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::{
    gmail_api::GmailApiClient, options::GmailContactFetchOptions,
};
use crate::integrations::mail::icloud_carddav::IcloudCardDavClient;
use crate::integrations::mail::sync_provider::read_provider_secret;
use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;

#[derive(Clone)]
pub struct LiveAddressBookProviderSyncPort {
    pool: PgPool,
    vault: HostVault,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingLookupPort>,
    google_api_base_url: String,
}

impl LiveAddressBookProviderSyncPort {
    pub fn new(
        pool: PgPool,
        vault: HostVault,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingLookupPort>,
        google_api_base_url: impl Into<String>,
    ) -> Self {
        Self {
            pool,
            vault,
            provider_secret_binding_store,
            google_api_base_url: google_api_base_url.into(),
        }
    }

    async fn gmail_access_token(
        &self,
        account_id: &str,
    ) -> Result<crate::platform::secrets::models::ResolvedSecret, AddressBookProviderSyncError>
    {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(account_id, ProviderAccountSecretPurpose::OauthToken)
            .await
            .map_err(|error| AddressBookProviderSyncError::Credential(error.to_string()))?
            .ok_or_else(|| {
                AddressBookProviderSyncError::Credential("missing credential".to_owned())
            })?;
        EmailAccountSetupService::new_with_host_vault_for_token_refresh(
            self.pool.clone(),
            SecretReferenceStore::new(self.pool.clone()),
            self.vault.clone(),
        )
        .refresh_gmail_access_token(&binding.secret_ref)
        .await
        .map_err(|error| AddressBookProviderSyncError::Credential(error.to_string()))
    }
}

impl AddressBookProviderSyncPort for LiveAddressBookProviderSyncPort {
    fn fetch_entries<'a>(
        &'a self,
        request: AddressBookProviderFetchRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AddressBookProviderBatch, AddressBookProviderSyncError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            if request.provider_kind != CommunicationProviderKind::Gmail {
                if request.provider_kind != CommunicationProviderKind::Icloud {
                    return Err(AddressBookProviderSyncError::UnsupportedProvider(
                        request.provider_kind.as_str().to_owned(),
                    ));
                }

                let secret_store = SecretReferenceStore::new(self.pool.clone());
                let password = read_provider_secret(
                    self.provider_secret_binding_store.as_ref(),
                    &secret_store,
                    &self.vault,
                    &request.account_id,
                    ProviderAccountSecretPurpose::ImapPassword,
                )
                .await
                .map_err(|error| AddressBookProviderSyncError::Credential(error.to_string()))?;
                let client = IcloudCardDavClient::from_config(&request.provider_config, &password)
                    .map_err(|error| {
                        AddressBookProviderSyncError::ProviderNetwork(error.to_string())
                    })?;
                return client.fetch_entries().await.map_err(|error| {
                    AddressBookProviderSyncError::ProviderNetwork(error.to_string())
                });
            }

            let access_token = self.gmail_access_token(&request.account_id).await?;
            let mut options = GmailContactFetchOptions::new(request.page_size);
            if let Some(page_token) = request.page_token {
                options = options.page_token(page_token);
            }
            GmailApiClient::new(&self.google_api_base_url)
                .fetch_entries(&access_token, &options)
                .await
                .map_err(|error| AddressBookProviderSyncError::ProviderNetwork(error.to_string()))
        })
    }

    fn upsert_entry<'a>(
        &'a self,
        request: AddressBookProviderUpsertRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AddressBookProviderEntry, AddressBookProviderSyncError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            if request.provider_kind != CommunicationProviderKind::Gmail {
                return Err(AddressBookProviderSyncError::UnsupportedProvider(
                    request.provider_kind.as_str().to_owned(),
                ));
            }
            if !request.remote_write_allowed {
                return Err(AddressBookProviderSyncError::RemoteWriteBlocked(
                    "contacts_write_scope_not_granted",
                ));
            }

            let access_token = self.gmail_access_token(&request.account_id).await?;
            GmailApiClient::new(&self.google_api_base_url)
                .upsert_entry(&access_token, &request)
                .await
                .map_err(|error| AddressBookProviderSyncError::ProviderNetwork(error.to_string()))
        })
    }
}
