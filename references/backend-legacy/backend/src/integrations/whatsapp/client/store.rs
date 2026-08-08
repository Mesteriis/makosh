use makosh_communications_api::accounts::ProviderAccountCommandPort;
use makosh_communications_api::accounts::ProviderSecretBindingCommandPort;
use makosh_communications_api::commands::ProviderCommandMirrorPort;
mod accounts;
mod evidence;
mod ingestion;
mod intelligence;
mod queries;
mod sessions;

use std::sync::Arc;

use sqlx::postgres::PgPool;

use crate::platform::communications::ProviderChannelMessageLookupPort;

#[derive(Clone)]
pub struct WhatsappWebStore {
    pub(in crate::integrations::whatsapp::client::store) pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    provider_command_mirror: Arc<dyn ProviderCommandMirrorPort>,
}

impl WhatsappWebStore {
    pub fn new(
        pool: PgPool,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
        provider_command_mirror: Arc<dyn ProviderCommandMirrorPort>,
    ) -> Self {
        Self {
            pool,
            provider_account_store,
            provider_secret_binding_store,
            provider_channel_message_store,
            provider_command_mirror,
        }
    }

    pub(in crate::integrations::whatsapp) fn provider_account_store(
        &self,
    ) -> &dyn ProviderAccountCommandPort {
        self.provider_account_store.as_ref()
    }

    pub(in crate::integrations::whatsapp) fn provider_secret_binding_store(
        &self,
    ) -> &dyn ProviderSecretBindingCommandPort {
        self.provider_secret_binding_store.as_ref()
    }

    pub(in crate::integrations::whatsapp) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(in crate::integrations::whatsapp::client) fn provider_channel_message_store(
        &self,
    ) -> &dyn ProviderChannelMessageLookupPort {
        self.provider_channel_message_store.as_ref()
    }

    pub(in crate::integrations::whatsapp) fn provider_command_mirror(
        &self,
    ) -> &dyn ProviderCommandMirrorPort {
        self.provider_command_mirror.as_ref()
    }
}
