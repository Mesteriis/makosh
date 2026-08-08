use std::future::Future;
use std::pin::Pin;

use sqlx::postgres::PgPool;

use crate::integrations::mail::accounts::service::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::gmail_api::GmailApiClient;
use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;
use makosh_communications_api::email::{
    EmailSendError, GmailOutboxSendRequest, GmailOutboxTransport, SendResult,
};

#[derive(Clone)]
pub struct LiveGmailOutboxTransport {
    pool: PgPool,
    secret_store: SecretReferenceStore,
    vault: HostVault,
}

impl LiveGmailOutboxTransport {
    pub fn new(pool: PgPool, vault: HostVault) -> Self {
        Self {
            pool: pool.clone(),
            secret_store: SecretReferenceStore::new(pool),
            vault,
        }
    }
}

impl GmailOutboxTransport for LiveGmailOutboxTransport {
    fn send<'a>(
        &'a self,
        request: GmailOutboxSendRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>> {
        Box::pin(async move {
            let account_setup = EmailAccountSetupService::new_with_host_vault_for_token_refresh(
                self.pool.clone(),
                self.secret_store.clone(),
                self.vault.clone(),
            );
            let access_token = account_setup
                .refresh_gmail_access_token(request.oauth_secret_ref)
                .await
                .map_err(|error| EmailSendError::Provider(error.to_string()))?;

            GmailApiClient::new(request.api_base_url)
                .user_id("me")
                .send_message(&access_token, request.email)
                .await
                .map_err(|error| EmailSendError::Provider(error.to_string()))
        })
    }
}

pub(crate) fn gmail_outbox_transport(pool: PgPool, vault: HostVault) -> impl GmailOutboxTransport {
    LiveGmailOutboxTransport::new(pool, vault)
}
