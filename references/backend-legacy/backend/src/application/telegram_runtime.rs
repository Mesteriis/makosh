use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::chats::TelegramChatMember;
use crate::integrations::telegram::client::models::messages::{
    TelegramForwardRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramReplyRequest,
};
use crate::integrations::telegram::client::store::TelegramStore;
use crate::integrations::telegram::runtime::manager::realtime_events::TelegramRuntimeEventBridgeContext;
use crate::integrations::telegram::runtime::manager::search::TelegramProviderSearchRequest;
use crate::integrations::telegram::runtime::manager::{
    TelegramMediaDownloadContext, TelegramMemberSyncContext, TelegramRuntimeManager,
    TelegramRuntimeOperationDeps, TelegramRuntimeStartContext,
};
use crate::integrations::telegram::runtime::models::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse, TelegramMediaDownloadRequest, TelegramMediaDownloadResponse,
    TelegramRuntimeRestartRequest, TelegramRuntimeStartRequest, TelegramRuntimeStatus,
    TelegramRuntimeStopRequest,
};
use crate::platform::config::app_config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};

pub(crate) struct TelegramRuntimeUseCaseContext<'a> {
    pub(crate) provider_account_store: CommunicationProviderAccountStore,
    pub(crate) provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    pub(crate) telegram_store: TelegramStore,
    pub(crate) secret_store: SecretReferenceStore,
    pub(crate) secret_resolver: &'a HostVault,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bus: &'a InMemoryEventBus,
    pub(crate) runtime: &'a TelegramRuntimeManager,
}

pub(crate) struct TelegramRuntimeUseCaseStores {
    pub(crate) provider_account_store: CommunicationProviderAccountStore,
    pub(crate) provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    pub(crate) telegram_store: TelegramStore,
    pub(crate) secret_store: SecretReferenceStore,
}

pub(crate) struct TelegramRuntimeUseCaseRuntime<'a> {
    pub(crate) secret_resolver: &'a HostVault,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bus: &'a InMemoryEventBus,
    pub(crate) runtime: &'a TelegramRuntimeManager,
}

impl<'a> TelegramRuntimeUseCaseContext<'a> {
    pub(crate) fn new(
        stores: TelegramRuntimeUseCaseStores,
        runtime: TelegramRuntimeUseCaseRuntime<'a>,
    ) -> Self {
        Self {
            provider_account_store: stores.provider_account_store,
            provider_secret_binding_store: stores.provider_secret_binding_store,
            telegram_store: stores.telegram_store,
            secret_store: stores.secret_store,
            secret_resolver: runtime.secret_resolver,
            config: runtime.config,
            event_bus: runtime.event_bus,
            runtime: runtime.runtime,
        }
    }

    fn event_bridge_context(&self) -> TelegramRuntimeEventBridgeContext {
        TelegramRuntimeEventBridgeContext::new(
            Some(self.telegram_store.clone()),
            self.event_bus.clone(),
        )
    }

    fn operation_deps(&self) -> TelegramRuntimeOperationDeps<'_, HostVault> {
        TelegramRuntimeOperationDeps {
            provider_account_store: &self.provider_account_store,
            provider_secret_binding_store: &self.provider_secret_binding_store,
            telegram_store: &self.telegram_store,
            secret_store: &self.secret_store,
            secret_resolver: self.secret_resolver,
            config: self.config,
            event_bridge: Some(self.event_bridge_context()),
        }
    }
}

pub(crate) async fn runtime_status(
    context: &TelegramRuntimeUseCaseContext<'_>,
    account_id: &str,
) -> Result<TelegramRuntimeStatus, TelegramError> {
    context
        .runtime
        .status_for_account(&context.provider_account_store, context.config, account_id)
        .await
}

pub(crate) async fn start_runtime(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramRuntimeStartRequest,
) -> Result<TelegramRuntimeStatus, TelegramError> {
    let start_context = TelegramRuntimeStartContext {
        provider_account_store: &context.provider_account_store,
        provider_secret_binding_store: &context.provider_secret_binding_store,
        telegram_store: &context.telegram_store,
        secret_store: &context.secret_store,
        secret_resolver: context.secret_resolver,
        config: context.config,
        event_bus: context.event_bus,
    };
    context.runtime.start_account(&start_context, request).await
}

pub(crate) async fn stop_runtime(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramRuntimeStopRequest,
) -> Result<TelegramRuntimeStatus, TelegramError> {
    context
        .runtime
        .stop_account_runtime(&context.provider_account_store, context.config, request)
        .await
}

pub(crate) async fn restart_runtime(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramRuntimeRestartRequest,
) -> Result<TelegramRuntimeStatus, TelegramError> {
    let start_context = TelegramRuntimeStartContext {
        provider_account_store: &context.provider_account_store,
        provider_secret_binding_store: &context.provider_secret_binding_store,
        telegram_store: &context.telegram_store,
        secret_store: &context.secret_store,
        secret_resolver: context.secret_resolver,
        config: context.config,
        event_bus: context.event_bus,
    };
    context
        .runtime
        .restart_account_runtime(&start_context, request)
        .await
}

pub(crate) async fn sync_chat_members(
    context: &TelegramRuntimeUseCaseContext<'_>,
    telegram_chat_id: &str,
) -> Result<Vec<TelegramChatMember>, TelegramError> {
    context
        .runtime
        .sync_chat_members(
            TelegramMemberSyncContext {
                provider_account_store: &context.provider_account_store,
                provider_secret_binding_store: &context.provider_secret_binding_store,
                telegram_store: &context.telegram_store,
                secret_store: &context.secret_store,
                secret_resolver: context.secret_resolver,
                config: context.config,
                event_bridge: Some(context.event_bridge_context()),
            },
            telegram_chat_id,
        )
        .await
}

pub(crate) async fn sync_chats(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramChatSyncRequest,
) -> Result<TelegramChatSyncResponse, TelegramError> {
    context
        .runtime
        .sync_chats_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn sync_history(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramHistorySyncRequest,
) -> Result<TelegramHistorySyncResponse, TelegramError> {
    context
        .runtime
        .sync_history_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn send_manual_message(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramManualSendRequest,
) -> Result<TelegramManualSendResponse, TelegramError> {
    context
        .runtime
        .send_manual_message_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn send_reply_message(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramReplyRequest,
) -> Result<TelegramManualSendResponse, TelegramError> {
    context
        .runtime
        .send_reply_message_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn send_forward_message(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramForwardRequest,
) -> Result<TelegramManualSendResponse, TelegramError> {
    context
        .runtime
        .send_forward_message_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn refresh_provider_search(
    context: &TelegramRuntimeUseCaseContext<'_>,
    account_id: String,
    provider_chat_id: Option<String>,
    query: String,
    limit: i32,
) -> Result<(), TelegramError> {
    context
        .runtime
        .search_provider_messages_with_deps(
            context.operation_deps(),
            &TelegramProviderSearchRequest {
                account_id,
                provider_chat_id,
                query,
                limit,
            },
        )
        .await
        .map(|_| ())
}

pub(crate) async fn refresh_forum_topics(
    context: &TelegramRuntimeUseCaseContext<'_>,
    telegram_chat_id: &str,
) -> Result<(), TelegramError> {
    context
        .runtime
        .sync_forum_topics_with_deps(context.operation_deps(), telegram_chat_id)
        .await
        .map(|_| ())
}

pub(crate) async fn download_media(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramMediaDownloadRequest,
) -> Result<TelegramMediaDownloadResponse, TelegramError> {
    context
        .runtime
        .download_media(
            TelegramMediaDownloadContext {
                provider_account_store: &context.provider_account_store,
                provider_secret_binding_store: &context.provider_secret_binding_store,
                telegram_store: &context.telegram_store,
                secret_store: &context.secret_store,
                secret_resolver: context.secret_resolver,
                config: context.config,
                event_bridge: Some(context.event_bridge_context()),
            },
            request,
        )
        .await
}
