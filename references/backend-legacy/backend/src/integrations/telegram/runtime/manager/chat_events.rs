use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::chat_state::{
    TelegramProviderChatPositionUpdate, reconcile_archive_commands_from_provider_state,
    reconcile_folder_add_commands_from_provider_state,
    reconcile_folder_remove_commands_from_provider_state,
    reconcile_mark_read_commands_from_provider_state,
    reconcile_marked_as_unread_commands_from_provider_state,
    reconcile_mute_commands_from_provider_state, reconcile_pin_commands_from_provider_state,
};
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::chats::{TelegramChat, TelegramChatGroupFilter};
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::store::TelegramStore;
use crate::integrations::telegram::tdjson::parsing::events::{
    TelegramTdlibChatMarkedAsUnreadSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatRemovedFromListSnapshot,
    TelegramTdlibChatUnreadSnapshot,
};
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatFolderSnapshot;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;
use makosh_events_postgres::store::EventStore;

use super::chat_event_payloads::{
    chat_archived_updated_event, chat_folder_labels_updated_event,
    chat_marked_as_unread_updated_event, chat_notification_settings_chat_updated_event,
    chat_notification_settings_updated_event, chat_pinned_updated_event,
    chat_position_updated_event, chat_unread_updated_event,
};
use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};

pub(super) async fn publish_chat_unread_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatUnreadSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let (chat, reconciled) = match apply_chat_unread_update(store, account_id, snapshot).await {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project unread update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = chat_unread_updated_event(account_id, &chat, snapshot, Utc::now()) else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append unread chat event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_marked_as_unread_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatMarkedAsUnreadSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let (chat, reconciled) = match apply_chat_marked_as_unread_update(store, account_id, snapshot)
        .await
    {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project marked-as-unread update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = chat_marked_as_unread_updated_event(account_id, &chat, snapshot, Utc::now())
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append marked-as-unread chat event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_notification_settings_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let (chat, reconciled) = match apply_chat_notification_settings_update(
        store, account_id, snapshot,
    )
    .await
    {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project notification settings update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(chat_updated_event) =
        chat_notification_settings_chat_updated_event(account_id, &chat, snapshot, Utc::now())
    else {
        return;
    };
    let Ok(event) =
        chat_notification_settings_updated_event(account_id, &chat, snapshot, Utc::now())
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&chat_updated_event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append notification chat-updated event");
    }
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append notification settings event");
    }

    let _ = event_bus.broadcast(chat_updated_event);
    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_position_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatPositionSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let (chat, reconciled) = match apply_chat_position_update(store, account_id, snapshot).await {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project chat position update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let occurred_at = Utc::now();
    let Ok(chat_updated_event) =
        chat_position_updated_event(account_id, &chat, snapshot, occurred_at)
    else {
        return;
    };
    let pin_event = if matches!(snapshot.list_kind.as_str(), "main" | "archive") {
        match chat_pinned_updated_event(account_id, &chat, snapshot, occurred_at) {
            Ok(event) => Some(event),
            Err(_) => return,
        }
    } else {
        None
    };
    let Ok(archive_event) = chat_archived_updated_event(account_id, &chat, snapshot, occurred_at)
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&chat_updated_event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append position chat-updated event");
    }
    if let Some(event) = &pin_event
        && let Err(error) = event_store.append(event).await
    {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append pinned chat event");
    }
    if let Err(error) = event_store.append(&archive_event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append archived chat event");
    }

    let _ = event_bus.broadcast(chat_updated_event);
    if let Some(event) = pin_event {
        let _ = event_bus.broadcast(event);
    }
    let _ = event_bus.broadcast(archive_event);

    if snapshot.list_kind == "folder"
        && let Ok(items) = store.list_chat_group_filters(Some(account_id)).await
    {
        publish_chat_group_filters_event(pool, event_bus, account_id, &items).await;
    }
}

pub(super) async fn publish_chat_removed_from_list_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatRemovedFromListSnapshot,
) {
    let removal_snapshot = TelegramTdlibChatPositionSnapshot {
        provider_chat_id: snapshot.provider_chat_id.clone(),
        list_kind: snapshot.list_kind.clone(),
        provider_folder_id: snapshot.provider_folder_id,
        order: 0,
        is_pinned: false,
        source_event: snapshot.source_event.clone(),
    };

    publish_chat_position_event(telegram_store, event_bus, account_id, &removal_snapshot).await;
}

pub(super) async fn publish_chat_folders_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    folders: &[TelegramTdlibChatFolderSnapshot],
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let update = match apply_chat_folder_update(store, account_id, folders).await {
        Ok(items) => items,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project chat folder update");
            return;
        }
    };

    for chat in &update.chats {
        let Ok(event) = chat_folder_labels_updated_event(account_id, chat, Utc::now()) else {
            continue;
        };
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append folder-label chat event");
        }
        let _ = event_bus.broadcast(event);
    }

    publish_chat_group_filters_event(pool, event_bus, account_id, &update.filters).await;
}

async fn publish_chat_group_filters_event(
    pool: &PgPool,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    items: &[TelegramChatGroupFilter],
) {
    let now = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_telegram_folders_updated_{}_{}",
            account_id,
            now.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::FOLDERS_UPDATED.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": account_id, "kind": "telegram_account"}),
    )
    .payload(json!({
        "account_id": account_id,
        "items": items,
    }))
    .build();

    let Ok(event) = event else {
        return;
    };
    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append folders event");
    }
    let _ = event_bus.broadcast(event);
}

async fn apply_chat_unread_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatUnreadSnapshot,
) -> Result<Option<(TelegramChat, Vec<TelegramProviderWriteCommand>)>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    store
        .apply_provider_unread_counts(
            &chat.telegram_chat_id,
            snapshot.unread_count,
            snapshot.unread_mention_count,
            snapshot.last_read_inbox_message_id.as_deref(),
            &snapshot.source_event,
        )
        .await?;

    let reconciled =
        if let Some(last_read_inbox_message_id) = snapshot.last_read_inbox_message_id.as_deref() {
            let observed_at = provider_observed_at(store.pool()).await?;
            reconcile_mark_read_commands_from_provider_state(
                store.pool(),
                account_id,
                &snapshot.provider_chat_id,
                last_read_inbox_message_id,
                observed_at,
                &format!("tdlib.{}", snapshot.source_event),
            )
            .await?
        } else {
            Vec::new()
        };

    let Some(chat) = store.telegram_chat_by_id(&chat.telegram_chat_id).await? else {
        return Ok(None);
    };

    Ok(Some((chat, reconciled)))
}

async fn apply_chat_marked_as_unread_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatMarkedAsUnreadSnapshot,
) -> Result<Option<(TelegramChat, Vec<TelegramProviderWriteCommand>)>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    store
        .apply_provider_marked_as_unread(
            &chat.telegram_chat_id,
            snapshot.is_marked_as_unread,
            &snapshot.source_event,
        )
        .await?;
    let observed_at = provider_observed_at(store.pool()).await?;
    let reconciled = reconcile_marked_as_unread_commands_from_provider_state(
        store.pool(),
        account_id,
        &snapshot.provider_chat_id,
        snapshot.is_marked_as_unread,
        observed_at,
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await?;
    let Some(chat) = store.telegram_chat_by_id(&chat.telegram_chat_id).await? else {
        return Ok(None);
    };

    Ok(Some((chat, reconciled)))
}

async fn apply_chat_notification_settings_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
) -> Result<Option<(TelegramChat, Vec<TelegramProviderWriteCommand>)>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    store
        .apply_provider_notification_settings(
            &chat.telegram_chat_id,
            snapshot.use_default_mute_for,
            snapshot.mute_for,
            &snapshot.source_event,
        )
        .await?;
    let observed_at = provider_observed_at(store.pool()).await?;
    let reconciled = reconcile_mute_commands_from_provider_state(
        store.pool(),
        account_id,
        &snapshot.provider_chat_id,
        snapshot.use_default_mute_for,
        snapshot.mute_for,
        observed_at,
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await?;
    let Some(chat) = store.telegram_chat_by_id(&chat.telegram_chat_id).await? else {
        return Ok(None);
    };

    Ok(Some((chat, reconciled)))
}

async fn apply_chat_position_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatPositionSnapshot,
) -> Result<Option<(TelegramChat, Vec<TelegramProviderWriteCommand>)>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    let position = TelegramProviderChatPositionUpdate {
        list_kind: snapshot.list_kind.clone(),
        provider_folder_id: snapshot.provider_folder_id,
        order: snapshot.order,
        is_pinned: snapshot.is_pinned,
        source_event: snapshot.source_event.clone(),
    };
    store
        .apply_provider_chat_position(&chat.telegram_chat_id, &position)
        .await?;

    let observed_at = provider_observed_at(store.pool()).await?;
    let observed_via = format!("tdlib.{}", snapshot.source_event);
    let mut reconciled = Vec::new();
    match (snapshot.list_kind.as_str(), snapshot.order > 0) {
        ("archive", true) => {
            reconciled.extend(
                reconcile_archive_commands_from_provider_state(
                    store.pool(),
                    account_id,
                    &snapshot.provider_chat_id,
                    true,
                    observed_at,
                    &observed_via,
                )
                .await?,
            );
            reconciled.extend(
                reconcile_pin_commands_from_provider_state(
                    store.pool(),
                    account_id,
                    &snapshot.provider_chat_id,
                    snapshot.is_pinned,
                    observed_at,
                    &observed_via,
                )
                .await?,
            );
        }
        ("main", true) => {
            reconciled.extend(
                reconcile_archive_commands_from_provider_state(
                    store.pool(),
                    account_id,
                    &snapshot.provider_chat_id,
                    false,
                    observed_at,
                    &observed_via,
                )
                .await?,
            );
            reconciled.extend(
                reconcile_pin_commands_from_provider_state(
                    store.pool(),
                    account_id,
                    &snapshot.provider_chat_id,
                    snapshot.is_pinned,
                    observed_at,
                    &observed_via,
                )
                .await?,
            );
        }
        ("folder", true) => {
            if let Some(provider_folder_id) = snapshot.provider_folder_id {
                reconciled.extend(
                    reconcile_folder_add_commands_from_provider_state(
                        store.pool(),
                        account_id,
                        &snapshot.provider_chat_id,
                        provider_folder_id,
                        observed_at,
                        &observed_via,
                    )
                    .await?,
                );
            }
        }
        ("folder", false) => {
            if let Some(provider_folder_id) = snapshot.provider_folder_id {
                reconciled.extend(
                    reconcile_folder_remove_commands_from_provider_state(
                        store.pool(),
                        account_id,
                        &snapshot.provider_chat_id,
                        provider_folder_id,
                        observed_at,
                        &observed_via,
                    )
                    .await?,
                );
            }
        }
        _ => {}
    }
    let Some(chat) = store.telegram_chat_by_id(&chat.telegram_chat_id).await? else {
        return Ok(None);
    };
    Ok(Some((chat, reconciled)))
}

async fn provider_observed_at(pool: &PgPool) -> Result<chrono::DateTime<Utc>, TelegramError> {
    sqlx::query_scalar("SELECT now()")
        .fetch_one(pool)
        .await
        .map_err(TelegramError::from)
}

async fn apply_chat_folder_update(
    store: &TelegramStore,
    account_id: &str,
    folders: &[TelegramTdlibChatFolderSnapshot],
) -> Result<ChatFolderUpdateProjection, TelegramError> {
    let chats = store
        .apply_provider_chat_folders(account_id, folders)
        .await?;
    let filters = store.list_chat_group_filters(Some(account_id)).await?;
    Ok(ChatFolderUpdateProjection { chats, filters })
}

struct ChatFolderUpdateProjection {
    chats: Vec<TelegramChat>,
    filters: Vec<TelegramChatGroupFilter>,
}

#[cfg(test)]
mod tests;
