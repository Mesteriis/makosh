//! Bounded reconstruction of Telegram-owned runtime projections.

use std::collections::BTreeMap;

use makosh_telegram_api::TelegramMessageProjection;
use makosh_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};

use crate::projection_cache::TelegramRuntimeProjectionCache;

pub(crate) async fn restore_account_projection_cache(
    cache: &mut TelegramRuntimeProjectionCache,
    durable: &TelegramDurablePersistence,
    account_id: &str,
    projection_limit: i64,
) -> Result<usize, TelegramDurablePersistenceError> {
    let chat_count =
        restore_conversation_projections(cache, durable, account_id, projection_limit).await?;
    let messages =
        restore_message_projections(cache, durable, account_id, projection_limit).await?;
    restore_message_lifecycle(cache, durable, account_id, projection_limit, &messages).await?;
    restore_media_projections(cache, durable, account_id, projection_limit).await?;
    restore_group_projections(cache, durable, account_id, projection_limit).await?;
    Ok(chat_count)
}

async fn restore_conversation_projections(
    cache: &mut TelegramRuntimeProjectionCache,
    durable: &TelegramDurablePersistence,
    account_id: &str,
    projection_limit: i64,
) -> Result<usize, TelegramDurablePersistenceError> {
    let chats = durable.list_chats(account_id, projection_limit).await?;
    ensure_complete(chats.len(), projection_limit)?;
    for chat in &chats {
        cache.put_chat(chat.clone());
    }
    for avatar in durable.list_chat_avatars(account_id).await? {
        cache.put_chat_avatar(avatar);
    }
    cache.put_chat_folders(durable.list_chat_folders(account_id).await?);
    for position in durable.list_chat_positions_for_account(account_id).await? {
        cache.put_chat_position(position);
    }
    for (provider_chat_id, state) in durable.list_chat_operational_states(account_id).await? {
        cache.put_chat_operational_state(account_id, &provider_chat_id, state);
    }
    Ok(chats.len())
}

async fn restore_message_projections(
    cache: &mut TelegramRuntimeProjectionCache,
    durable: &TelegramDurablePersistence,
    account_id: &str,
    projection_limit: i64,
) -> Result<Vec<TelegramMessageProjection>, TelegramDurablePersistenceError> {
    let messages = durable
        .list_messages_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(messages.len(), projection_limit)?;
    for message in &messages {
        cache.put_message(message.clone());
    }
    Ok(messages)
}

async fn restore_message_lifecycle(
    cache: &mut TelegramRuntimeProjectionCache,
    durable: &TelegramDurablePersistence,
    account_id: &str,
    projection_limit: i64,
    messages: &[TelegramMessageProjection],
) -> Result<(), TelegramDurablePersistenceError> {
    let identities = messages
        .iter()
        .map(|message| {
            (
                message.message_id.clone(),
                (
                    message.account_id.clone(),
                    message.provider_chat_id.clone(),
                    message.provider_message_id.clone(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let versions = durable
        .list_message_versions_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(versions.len(), projection_limit)?;
    for version in versions {
        cache.append_message_version(version);
    }

    let tombstones = durable
        .list_tombstones_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(tombstones.len(), projection_limit)?;
    for tombstone in tombstones {
        cache.append_message_tombstone(tombstone);
    }

    let mutations = durable
        .list_message_mutations_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(mutations.len(), projection_limit)?;
    for (message_id, mutations) in mutations {
        let (message_account_id, provider_chat_id, provider_message_id) = identities
            .get(&message_id)
            .ok_or(TelegramDurablePersistenceError::InvalidRow)?;
        for mutation in mutations {
            cache.apply_message_mutation(
                message_account_id,
                provider_chat_id,
                provider_message_id,
                mutation,
            );
        }
    }

    let reactions = durable
        .list_reactions_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(reactions.len(), projection_limit)?;
    for (message_id, reactions) in reactions {
        cache.replace_reactions(&message_id, reactions);
    }
    Ok(())
}

async fn restore_media_projections(
    cache: &mut TelegramRuntimeProjectionCache,
    durable: &TelegramDurablePersistence,
    account_id: &str,
    projection_limit: i64,
) -> Result<(), TelegramDurablePersistenceError> {
    let files = durable
        .list_files_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(files.len(), projection_limit)?;
    for file in files {
        cache.put_file(file);
    }

    let attachments = durable
        .list_attachments_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(attachments.len(), projection_limit)?;
    for attachment in attachments {
        cache.put_attachment(attachment);
    }
    Ok(())
}

async fn restore_group_projections(
    cache: &mut TelegramRuntimeProjectionCache,
    durable: &TelegramDurablePersistence,
    account_id: &str,
    projection_limit: i64,
) -> Result<(), TelegramDurablePersistenceError> {
    let topics = durable
        .list_topics_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(topics.len(), projection_limit)?;
    for topic in topics {
        cache.put_topic(topic);
    }

    let participant_pages = durable
        .list_participant_pages_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(participant_pages.len(), projection_limit)?;
    for page in participant_pages {
        cache.put_participants(&page);
    }

    let chat_states = durable
        .list_chat_states_for_account(account_id, projection_limit)
        .await?;
    ensure_complete(chat_states.len(), projection_limit)?;
    for (provider_chat_id, state) in chat_states {
        cache.apply_chat_state(account_id, &provider_chat_id, state);
    }
    Ok(())
}

fn ensure_complete(
    projection_count: usize,
    projection_limit: i64,
) -> Result<(), TelegramDurablePersistenceError> {
    let projection_limit = usize::try_from(projection_limit)
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
    if projection_count > projection_limit {
        return Err(TelegramDurablePersistenceError::ProjectionLimitExceeded);
    }
    Ok(())
}
