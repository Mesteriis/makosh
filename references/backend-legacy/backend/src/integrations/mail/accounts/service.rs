use makosh_communications_api::accounts::ProviderAccountCommandPort;
use makosh_communications_api::accounts::ProviderSecretBindingCommandPort;
mod constructors;
mod gmail_complete;
mod gmail_payloads;
mod gmail_refresh;
mod gmail_start;
mod imap;
mod imap_payloads;
mod stores;
mod token_http;

use reqwest::Client;
use sqlx::postgres::PgPool;
use std::sync::Arc;

use crate::platform::secrets::store::SecretReferenceStore;

use super::vault::AccountSecretVault;

#[derive(Clone)]
pub struct EmailAccountSetupService {
    pub(in crate::integrations::mail::accounts::service) pool: Option<PgPool>,
    pub(in crate::integrations::mail::accounts::service) secret_store: Option<SecretReferenceStore>,
    pub(in crate::integrations::mail::accounts::service) provider_account_store:
        Option<Arc<dyn ProviderAccountCommandPort>>,
    pub(in crate::integrations::mail::accounts::service) provider_secret_binding_store:
        Option<Arc<dyn ProviderSecretBindingCommandPort>>,
    pub(in crate::integrations::mail::accounts::service) vault: AccountSecretVault,
    pub(in crate::integrations::mail::accounts::service) http: Client,
}
