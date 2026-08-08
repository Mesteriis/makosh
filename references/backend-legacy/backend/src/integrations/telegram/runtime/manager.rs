use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountLookupPort;
use makosh_communications_api::accounts::ProviderSecretBindingLookupPort;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::integrations::telegram::client::store::TelegramStore;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::config::app_config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::secrets::resolver::SecretResolver;

use self::realtime_events::TelegramRuntimeEventBridgeContext;
use self::search::TelegramProviderSearchRequest;
use super::models::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse,
};
use super::state::TelegramRuntimeActorHandle;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::{
    TelegramForwardRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramReplyRequest,
};

mod account;
mod actor_states;
mod chat_event_payloads;
mod chat_events;
pub(crate) mod command_executor;
mod command_executor_dispatch;
mod command_executor_media;
mod lifecycle;
mod media_download;
mod message_events;
mod participant_events;
mod participants;
pub(crate) mod realtime_events;
mod registry;
pub(crate) mod search;
mod send;
mod sync_chats;
mod sync_history;
mod sync_history_tdlib;
mod tdlib_actor;
mod topic_events;
mod topics;

#[derive(Clone, Default)]
pub struct TelegramRuntimeManager {
    actors: Arc<Mutex<HashMap<String, TelegramRuntimeActorHandle>>>,
}

pub(crate) struct TelegramMediaDownloadContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramMemberSyncContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramRuntimeOperationContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramRuntimeOperationDeps<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bridge: Option<TelegramRuntimeEventBridgeContext>,
}

pub(crate) struct TelegramRuntimeStartContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) provider_account_store: &'a dyn ProviderAccountLookupPort,
    pub(crate) provider_secret_binding_store: &'a dyn ProviderSecretBindingLookupPort,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bus: &'a InMemoryEventBus,
}

fn telegram_media_blob_root() -> &'static std::path::Path {
    std::path::Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
}

impl TelegramRuntimeManager {
    pub(crate) async fn sync_chats_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramChatSyncRequest,
    ) -> Result<TelegramChatSyncResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.sync_chats(&operation_context(deps), request).await
    }

    pub(crate) async fn sync_history_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramHistorySyncRequest,
    ) -> Result<TelegramHistorySyncResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.sync_history(&operation_context(deps), request).await
    }

    pub(crate) async fn send_manual_message_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.send_manual_message(&operation_context(deps), request)
            .await
    }

    pub(crate) async fn send_reply_message_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramReplyRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.send_reply_message(&operation_context(deps), request)
            .await
    }

    pub(crate) async fn send_forward_message_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramForwardRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.send_forward_message(&operation_context(deps), request)
            .await
    }

    pub(crate) async fn search_provider_messages_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        request: &TelegramProviderSearchRequest,
    ) -> Result<Vec<String>, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.search_provider_messages(&operation_context(deps), request)
            .await
    }

    pub(crate) async fn sync_forum_topics_with_deps<S>(
        &self,
        deps: TelegramRuntimeOperationDeps<'_, S>,
        telegram_chat_id: &str,
    ) -> Result<usize, TelegramError>
    where
        S: SecretResolver + Sync + ?Sized,
    {
        self.sync_forum_topics(&operation_context(deps), telegram_chat_id)
            .await
    }
}

fn operation_context<S>(
    deps: TelegramRuntimeOperationDeps<'_, S>,
) -> TelegramRuntimeOperationContext<'_, S>
where
    S: SecretResolver + Sync + ?Sized,
{
    TelegramRuntimeOperationContext {
        provider_account_store: deps.provider_account_store,
        provider_secret_binding_store: deps.provider_secret_binding_store,
        telegram_store: deps.telegram_store,
        secret_store: deps.secret_store,
        secret_resolver: deps.secret_resolver,
        config: deps.config,
        event_bridge: deps.event_bridge,
    }
}
