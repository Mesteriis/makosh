use sqlx::Row;
use sqlx::postgres::PgRow;

use makosh_communications_api::provider_messages::ProviderChannelMessage;

use super::errors::TelegramError;
use super::models::chats::TelegramChat;
use super::models::messages::TelegramMessage;
use super::models::messages::{
    TelegramMessageTombstone, TelegramMessageVersion, TelegramProviderWriteCommand,
};

pub(super) fn row_to_telegram_chat(row: PgRow) -> Result<TelegramChat, TelegramError> {
    Ok(TelegramChat {
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        chat_kind: row.try_get("chat_kind")?,
        title: row.try_get("title")?,
        username: row.try_get("username")?,
        sync_state: row.try_get("sync_state")?,
        last_message_at: row.try_get("last_message_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_telegram_message(row: PgRow) -> Result<TelegramMessage, TelegramError> {
    Ok(TelegramMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_record_id")?,
        provider_chat_id: row.try_get("conversation_id")?,
        chat_title: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        sender_display_name: row.try_get("sender_display_name")?,
        text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        delivery_state: row.try_get("delivery_state")?,
        metadata: row.try_get("message_metadata")?,
    })
}

pub(in crate::integrations::telegram) fn provider_channel_message_to_telegram_message(
    message: ProviderChannelMessage,
) -> TelegramMessage {
    TelegramMessage {
        message_id: message.message_id,
        raw_record_id: message.raw_record_id,
        account_id: message.account_id,
        provider_message_id: message.provider_record_id,
        provider_chat_id: Some(message.conversation_id),
        chat_title: message.subject,
        sender: message.sender,
        sender_display_name: message.sender_display_name,
        text: message.body_text,
        occurred_at: message.occurred_at,
        projected_at: message.projected_at,
        channel_kind: message.channel_kind,
        delivery_state: message.delivery_state,
        metadata: message.message_metadata,
    }
}

pub(super) fn row_to_telegram_message_version(
    row: PgRow,
) -> Result<TelegramMessageVersion, TelegramError> {
    Ok(TelegramMessageVersion {
        version_id: row.try_get("version_id")?,
        message_id: row.try_get("message_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        version_number: row.try_get("version_number")?,
        body_text: row.try_get("body_text")?,
        edit_timestamp: row.try_get("edit_timestamp")?,
        source_event: row.try_get("source_event")?,
        raw_diff_payload: row.try_get("raw_diff_payload")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(super) fn row_to_telegram_message_tombstone(
    row: PgRow,
) -> Result<TelegramMessageTombstone, TelegramError> {
    Ok(TelegramMessageTombstone {
        tombstone_id: row.try_get("tombstone_id")?,
        message_id: row.try_get("message_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        reason_class: row.try_get("reason_class")?,
        actor_class: row.try_get("actor_class")?,
        observed_at: row.try_get("observed_at")?,
        source_event: row.try_get("source_event")?,
        is_provider_delete: row.try_get("is_provider_delete")?,
        is_local_visible: row.try_get("is_local_visible")?,
        metadata: row.try_get("metadata")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(crate) fn row_to_telegram_provider_write_command(
    row: PgRow,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    Ok(TelegramProviderWriteCommand {
        command_id: row.try_get("command_id")?,
        account_id: row.try_get("account_id")?,
        command_kind: row.try_get("command_kind")?,
        idempotency_key: row.try_get("idempotency_key")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        target_ref: row.try_get("target_ref")?,
        payload: row.try_get("payload")?,
        capability_state: row.try_get("capability_state")?,
        action_class: row.try_get("action_class")?,
        confirmation_decision: row.try_get("confirmation_decision")?,
        status: row.try_get("status")?,
        retry_count: row.try_get("retry_count")?,
        max_retries: row.try_get("max_retries")?,
        lease_epoch: row.try_get("lease_epoch")?,
        last_error: row.try_get("last_error")?,
        result_payload: row.try_get("result_payload")?,
        audit_metadata: row.try_get("audit_metadata")?,
        actor_id: row.try_get("actor_id")?,
        happened_at: row.try_get("happened_at")?,
        next_attempt_at: row.try_get("next_attempt_at")?,
        last_attempt_at: row.try_get("last_attempt_at")?,
        locked_at: row.try_get("locked_at")?,
        locked_by: row.try_get("locked_by")?,
        provider_observed_at: row.try_get("provider_observed_at")?,
        provider_state: row.try_get("provider_state")?,
        reconciliation_status: row.try_get("reconciliation_status")?,
        reconciled_at: row.try_get("reconciled_at")?,
        dead_lettered_at: row.try_get("dead_lettered_at")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

// --- Reaction rows ---

use super::models::messages::TelegramReaction;

pub(super) fn row_to_telegram_reaction(row: PgRow) -> Result<TelegramReaction, TelegramError> {
    Ok(TelegramReaction {
        reaction_id: row.try_get("reaction_id")?,
        message_id: row.try_get("message_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        sender_id: row.try_get("sender_id")?,
        sender_display_name: row.try_get("sender_display_name")?,
        reaction_emoji: row.try_get("reaction_emoji")?,
        is_active: row.try_get("is_active")?,
        observed_at: row.try_get("observed_at")?,
        source_event: row.try_get("source_event")?,
        provider_actor_id: row.try_get("provider_actor_id")?,
        metadata: row.try_get("metadata")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

// --- Reply/Forward rows ---

use super::models::message_references::{TelegramForwardRef, TelegramReplyRef};

pub(super) fn row_to_telegram_reply_ref(row: PgRow) -> Result<TelegramReplyRef, TelegramError> {
    Ok(TelegramReplyRef {
        reply_ref_id: row.try_get("reply_ref_id")?,
        source_message_id: row.try_get("source_message_id")?,
        target_message_id: row.try_get("target_message_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        source_provider_id: row.try_get("source_provider_id")?,
        target_provider_id: row.try_get("target_provider_id")?,
        reply_depth: row.try_get("reply_depth")?,
        is_topic_reply: row.try_get("is_topic_reply")?,
        topic_id: row.try_get("topic_id")?,
        source_message_summary: None,
        target_message_summary: None,
        metadata: row.try_get("metadata")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(super) fn row_to_telegram_forward_ref(row: PgRow) -> Result<TelegramForwardRef, TelegramError> {
    Ok(TelegramForwardRef {
        forward_ref_id: row.try_get("forward_ref_id")?,
        source_message_id: row.try_get("source_message_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        source_provider_id: row.try_get("source_provider_id")?,
        forward_origin_chat_id: row.try_get("forward_origin_chat_id")?,
        forward_origin_message_id: row.try_get("forward_origin_message_id")?,
        forward_origin_sender_id: row.try_get("forward_origin_sender_id")?,
        forward_origin_sender_name: row.try_get("forward_origin_sender_name")?,
        forward_date: row.try_get("forward_date")?,
        forward_depth: row.try_get("forward_depth")?,
        source_message_summary: None,
        metadata: row.try_get("metadata")?,
        provenance: row.try_get("provenance")?,
        created_at: row.try_get("created_at")?,
    })
}
