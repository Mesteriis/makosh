use serde_json::Value;
use sqlx::PgPool;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::TelegramManualSendRequest;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::tdjson::{
    snapshots::TelegramTdlibMessageSnapshot, snapshots::TelegramTdlibTopicSnapshot,
};
use makosh_provider_telegram::tdlib::types::TdlibMediaKind;

use super::super::commands::{
    request_actor_add_chat_to_folder, request_actor_create_forum_topic,
    request_actor_delete_message, request_actor_edit_message, request_actor_forward,
    request_actor_join_chat, request_actor_leave_chat, request_actor_pin_message,
    request_actor_remove_chat_from_folder, request_actor_reply, request_actor_send,
    request_actor_send_media, request_actor_set_reaction, request_actor_toggle_chat_archive,
    request_actor_toggle_chat_mute, request_actor_toggle_chat_unread,
    request_actor_toggle_forum_topic_closed,
};
use super::super::models::TelegramMediaSendRequest;
use super::super::state::TelegramRuntimeCommand;

pub(super) enum DispatchOutcome {
    AwaitingProvider,
    ObservedMessage(TelegramTdlibMessageSnapshot),
    ObservedTopic(TelegramTdlibTopicSnapshot),
}

pub(super) async fn dispatch_command(
    _pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    command_tx: std::sync::mpsc::Sender<TelegramRuntimeCommand>,
) -> Result<DispatchOutcome, TelegramError> {
    match command.command_kind.as_str() {
        "send_text" => {
            let snapshot = request_actor_send(
                command_tx,
                TelegramManualSendRequest {
                    command_id: command.command_id.clone(),
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    text: payload_string(command, "text")?,
                },
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "send_media" => {
            let request = TelegramMediaSendRequest {
                command_id: command.command_id.clone(),
                provider_chat_id: command.provider_chat_id.clone(),
                media_type: TdlibMediaKind::try_from(
                    payload_string(command, "media_type")?.as_str(),
                )
                .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?,
                local_path: payload_string(command, "local_path")?,
                caption: payload_optional_string(command, "caption"),
                filename: payload_optional_string(command, "filename"),
            };
            let snapshot = request_actor_send_media(command_tx, request).await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "reply" => {
            let snapshot = request_actor_reply(
                command_tx,
                command.provider_chat_id.clone(),
                payload_string(command, "reply_to_provider_message_id")?,
                payload_string(command, "text")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "forward" => {
            let snapshot = request_actor_forward(
                command_tx,
                command.provider_chat_id.clone(),
                payload_string(command, "from_provider_chat_id")?,
                payload_string(command, "from_provider_message_id")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "edit" => {
            request_actor_edit_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, "edit")?,
                payload_string(command, "new_text")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "delete" => {
            request_actor_delete_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, "delete")?,
                command
                    .payload
                    .get("is_provider_delete")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "react" | "unreact" => {
            let is_active = command.command_kind == "react";
            request_actor_set_reaction(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, &command.command_kind)?,
                payload_string(command, "reaction_emoji")?,
                is_active,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "pin" | "unpin" => {
            let pin = command.command_kind == "pin";
            request_actor_pin_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, &command.command_kind)?,
                pin,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "mark_read" | "mark_unread" => {
            request_actor_toggle_chat_unread(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "mark_unread",
                command.provider_message_id.clone(),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "archive" | "unarchive" => {
            request_actor_toggle_chat_archive(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "archive",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "mute" | "unmute" => {
            request_actor_toggle_chat_mute(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "mute",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "folder_add" => {
            request_actor_add_chat_to_folder(
                command_tx,
                command.provider_chat_id.clone(),
                payload_i64(command, "provider_folder_id")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "folder_remove" => {
            request_actor_remove_chat_from_folder(
                command_tx,
                command.provider_chat_id.clone(),
                payload_i64(command, "provider_folder_id")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "join" => {
            request_actor_join_chat(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "leave" => {
            request_actor_leave_chat(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "topic_create" => {
            let snapshot = request_actor_create_forum_topic(
                command_tx,
                command.provider_chat_id.clone(),
                payload_string(command, "title")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::ObservedTopic(snapshot))
        }
        "topic_close" | "topic_reopen" => {
            request_actor_toggle_forum_topic_closed(
                command_tx,
                command.provider_chat_id.clone(),
                payload_i64(command, "provider_topic_id")?,
                command.command_kind == "topic_close",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        other => Err(TelegramError::InvalidRequest(format!(
            "command executor: unsupported command kind `{other}`"
        ))),
    }
}

fn payload_string(
    command: &TelegramProviderWriteCommand,
    key: &str,
) -> Result<String, TelegramError> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{} command missing `{key}`",
                command.command_kind
            ))
        })
}

fn payload_optional_string(command: &TelegramProviderWriteCommand, key: &str) -> Option<String> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn payload_i64(command: &TelegramProviderWriteCommand, key: &str) -> Result<i64, TelegramError> {
    command
        .payload
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{} command missing numeric `{key}`",
                command.command_kind
            ))
        })
}

fn provider_message_id(
    command: &TelegramProviderWriteCommand,
    operation: &str,
) -> Result<String, TelegramError> {
    command
        .provider_message_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{operation} command missing provider_message_id"
            ))
        })
}
