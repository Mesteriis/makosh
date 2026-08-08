use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::messages::reaction_metadata::derive_tdlib_chosen_reaction_emojis;
use crate::integrations::telegram::client::participants::{
    reconcile_participant_commands_from_message_evidence, tdlib_self_membership_lifecycle,
};
use crate::integrations::telegram::client::reactions::reconcile_reaction_commands_from_provider_reactions;
use crate::integrations::telegram::client::store::TelegramStore;
use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountLookupPort;
use makosh_communications_api::accounts::{ProviderAccount, ProviderSecretBindingLookupPort};

use crate::platform::config::app_config::AppConfig;
use crate::platform::secrets::resolver::SecretResolver;

use super::super::actor::support::oldest_tdlib_message_id;
use super::super::commands::request_actor_history;
use super::super::models::{
    TelegramHistorySyncMode, TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};
use super::TelegramRuntimeManager;
use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};

pub(in crate::integrations::telegram::runtime::manager) struct TdlibHistorySyncContext<
    'a,
    S: SecretResolver + Sync + ?Sized,
> {
    pub(in crate::integrations::telegram::runtime::manager) provider_account_store:
        &'a dyn ProviderAccountLookupPort,
    pub(in crate::integrations::telegram::runtime::manager) provider_secret_binding_store:
        &'a dyn ProviderSecretBindingLookupPort,
    pub(in crate::integrations::telegram::runtime::manager) telegram_store: &'a TelegramStore,
    pub(in crate::integrations::telegram::runtime::manager) secret_store: &'a SecretReferenceStore,
    pub(in crate::integrations::telegram::runtime::manager) secret_resolver: &'a S,
    pub(in crate::integrations::telegram::runtime::manager) config: &'a AppConfig,
    pub(in crate::integrations::telegram::runtime::manager) account: &'a ProviderAccount,
    pub(in crate::integrations::telegram::runtime::manager) runtime_kind: String,
    pub(in crate::integrations::telegram::runtime::manager) event_bridge:
        Option<TelegramRuntimeEventBridgeContext>,
}

impl TelegramRuntimeManager {
    pub(in crate::integrations::telegram::runtime::manager) async fn sync_tdlib_history<
        S: SecretResolver + Sync + ?Sized,
    >(
        &self,
        context: TdlibHistorySyncContext<'_, S>,
        request: &TelegramHistorySyncRequest,
    ) -> Result<TelegramHistorySyncResponse, TelegramError> {
        let mode = request.mode();
        if mode == TelegramHistorySyncMode::Full {
            ensure_full_history_sync_allowed(context.telegram_store, context.account, request)
                .await?;
        }
        let command_tx = self
            .ensure_tdlib_actor(
                context.provider_secret_binding_store,
                context.secret_store,
                context.secret_resolver,
                context.config,
                context.account,
                context.event_bridge.clone(),
            )
            .await?;
        let snapshots = request_actor_history(
            command_tx,
            request.provider_chat_id.trim().to_owned(),
            request.from_message_id,
            request.limit.unwrap_or(100) as i32,
            mode,
        )
        .await?;
        let next_from_message_id = oldest_tdlib_message_id(&snapshots);
        let has_more = mode != TelegramHistorySyncMode::Full
            && next_from_message_id.is_some()
            && snapshots.len() >= request.limit.unwrap_or(100) as usize;
        let import_batch_id = format!(
            "telegram-tdlib-history-sync:{}:{}",
            context.account.account_id,
            request.provider_chat_id.trim()
        );
        for snapshot in &snapshots {
            let observed = context
                .telegram_store
                .ingest_tdlib_message_snapshot(
                    &context.account.account_id,
                    snapshot,
                    &import_batch_id,
                )
                .await?;
            context
                .telegram_store
                .publish_observed_message_raw_signal(
                    &observed,
                    context
                        .event_bridge
                        .as_ref()
                        .map(|bridge| &bridge.event_bus),
                )
                .await?;
            if let Some(lifecycle) =
                tdlib_self_membership_lifecycle(&context.account.external_account_id, &snapshot.raw)
            {
                let commands = reconcile_participant_commands_from_message_evidence(
                    context.telegram_store.pool(),
                    &context.account.account_id,
                    &snapshot.provider_chat_id,
                    &snapshot.provider_message_id,
                    snapshot.occurred_at,
                    &lifecycle,
                )
                .await?;
                for command in commands {
                    publish_command_reconciled_events(
                        context.event_bridge.as_ref(),
                        &command,
                        &lifecycle.observed_via,
                    )
                    .await;
                }
            }
            let chosen_reactions = derive_tdlib_chosen_reaction_emojis(&snapshot.raw);
            let commands = reconcile_reaction_commands_from_provider_reactions(
                context.telegram_store.pool(),
                &context.account.account_id,
                &snapshot.provider_chat_id,
                &snapshot.provider_message_id,
                &chosen_reactions,
                snapshot.occurred_at,
                "tdlib.interaction_info.reactions",
            )
            .await?;
            for command in commands {
                publish_command_reconciled_events(
                    context.event_bridge.as_ref(),
                    &command,
                    "tdlib.interaction_info.reactions",
                )
                .await;
            }
        }
        let items = context
            .telegram_store
            .recent_messages(
                Some(&context.account.account_id),
                Some(&request.provider_chat_id),
                request.limit.unwrap_or(100),
            )
            .await?;
        Ok(TelegramHistorySyncResponse {
            account_id: context.account.account_id.clone(),
            provider_chat_id: request.provider_chat_id.trim().to_owned(),
            runtime_kind: context.runtime_kind,
            status: "synced".to_owned(),
            synced_count: snapshots.len(),
            has_more,
            next_from_message_id,
            items,
        })
    }
}

async fn ensure_full_history_sync_allowed(
    telegram_store: &TelegramStore,
    account: &ProviderAccount,
    request: &TelegramHistorySyncRequest,
) -> Result<(), TelegramError> {
    let chat = telegram_store
        .telegram_chat(&account.account_id, &request.provider_chat_id)
        .await?
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram chat `{}` is not synced for account `{}`",
                request.provider_chat_id.trim(),
                account.account_id
            ))
        })?;
    if chat.chat_kind != "private"
        && chat
            .metadata
            .get("full_history_sync_enabled")
            .and_then(serde_json::Value::as_bool)
            != Some(true)
    {
        return Err(TelegramError::InvalidRequest(
            "full Telegram history sync is only allowed for private chats unless the chat full-history setting is enabled"
                .to_owned(),
        ));
    }
    Ok(())
}
