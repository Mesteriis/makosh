use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::{ProviderAccount, ProviderSecretBindingLookupPort};
use std::sync::mpsc::Sender;

use chrono::Utc;

use crate::integrations::telegram::client::errors::TelegramError;

use crate::platform::config::app_config::AppConfig;
use crate::platform::secrets::resolver::SecretResolver;

use super::super::actor::session::optional_telegram_session_key;
use super::super::actor::spawn::spawn_tdlib_actor;
use super::super::state::{TelegramRuntimeActorHandle, TelegramRuntimeCommand};
use super::TelegramRuntimeManager;
use super::actor_states::running_actor_state;
use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, spawn_telegram_runtime_event_bridge,
};

impl TelegramRuntimeManager {
    pub(in crate::integrations::telegram::runtime::manager) async fn ensure_tdlib_actor(
        &self,
        provider_secret_binding_store: &dyn ProviderSecretBindingLookupPort,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        account: &ProviderAccount,
        event_bridge: Option<TelegramRuntimeEventBridgeContext>,
    ) -> Result<Sender<TelegramRuntimeCommand>, TelegramError> {
        if let Some(command_tx) = self.actor_command_tx(&account.account_id)? {
            return Ok(command_tx);
        }

        let session_encryption_key = optional_telegram_session_key(
            provider_secret_binding_store,
            secret_store,
            secret_resolver,
            &account.account_id,
        )
        .await?;
        let (runtime_event_tx, runtime_event_rx) = tokio::sync::mpsc::unbounded_channel();
        let runtime_event_tx = event_bridge.as_ref().map(|_| runtime_event_tx);
        let command_tx = spawn_tdlib_actor(
            config.clone(),
            account.clone(),
            session_encryption_key,
            runtime_event_tx,
        )?;
        if let Some(event_bridge) = event_bridge {
            spawn_telegram_runtime_event_bridge(
                event_bridge.telegram_store,
                event_bridge.event_bus,
                account.account_id.clone(),
                runtime_event_rx,
            );
        }
        self.set_actor_handle(
            account.account_id.clone(),
            TelegramRuntimeActorHandle {
                state: running_actor_state(Utc::now()),
                command_tx: Some(command_tx.clone()),
            },
        )?;
        Ok(command_tx)
    }
}
