use serde_json::Value;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibTopicSnapshot;

use super::topics::parse_forum_topic_info;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatUnreadSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) unread_count: Option<i64>,
    pub(crate) unread_mention_count: Option<i64>,
    pub(crate) last_read_inbox_message_id: Option<String>,
    pub(crate) source_event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatMarkedAsUnreadSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) is_marked_as_unread: bool,
    pub(crate) source_event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatNotificationSettingsSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) use_default_mute_for: bool,
    pub(crate) mute_for: i64,
    pub(crate) source_event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatPositionSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) list_kind: String,
    pub(crate) provider_folder_id: Option<i64>,
    pub(crate) order: i64,
    pub(crate) is_pinned: bool,
    pub(crate) source_event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatRemovedFromListSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) list_kind: String,
    pub(crate) provider_folder_id: Option<i64>,
    pub(crate) source_event: String,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibTopicUpdateSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) topic: TelegramTdlibTopicSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibTypingSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_thread_id: Option<String>,
    pub(crate) sender_id: String,
    pub(crate) action: String,
    pub(crate) is_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatFoldersUpdateSnapshot {
    pub(crate) folders:
        Vec<crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatFolderSnapshot>,
    pub(crate) source_event: String,
}

pub(crate) fn authorization_state(event: &Value) -> Option<&Value> {
    match event.get("@type").and_then(Value::as_str) {
        Some("updateAuthorizationState") => event.get("authorization_state"),
        Some(value) if value.starts_with("authorizationState") => Some(event),
        _ => None,
    }
}

pub(crate) fn is_tdlib_parameters_not_specified_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event.get("message").and_then(Value::as_str) == Some("Parameters aren't specified")
}

pub(crate) fn is_tdlib_database_encryption_key_needed_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| {
                message.contains("Database encryption key is needed")
                    && message.contains("checkDatabaseEncryptionKey")
            })
}

pub(crate) fn tdlib_error_message(event: &Value) -> Option<String> {
    if event.get("@type").and_then(Value::as_str) != Some("error") {
        return None;
    }

    let code = event
        .get("code")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned());
    let message = event
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("TDLib returned an error");

    Some(format!("TDLib error {code}: {message}"))
}

pub(crate) fn parse_tdlib_typing_snapshot(event: &Value) -> Option<TelegramTdlibTypingSnapshot> {
    if event.get("@type").and_then(Value::as_str) != Some("updateUserChatAction") {
        return None;
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id")?;
    let provider_thread_id = tdlib_event_id(event, "message_thread_id");
    let sender_id = tdlib_sender_id(event.get("sender_id")?)?;
    let action = event
        .get("action")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)?;
    let is_active = action != "chatActionCancel";

    Some(TelegramTdlibTypingSnapshot {
        provider_chat_id,
        provider_thread_id,
        sender_id,
        action,
        is_active,
    })
}

pub(crate) fn parse_tdlib_topic_update_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibTopicUpdateSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateForumTopicInfo") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateForumTopicInfo missing `chat_id`".to_owned())
    })?;
    let info = event.get("info").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateForumTopicInfo missing `info` field".to_owned())
    })?;
    let topic = parse_forum_topic_info(info)?;

    Ok(Some(TelegramTdlibTopicUpdateSnapshot {
        provider_chat_id,
        topic,
    }))
}

pub(crate) fn parse_tdlib_chat_unread_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatUnreadSnapshot>, TelegramError> {
    match event.get("@type").and_then(Value::as_str) {
        Some("updateChatReadInbox") => {
            let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
                TelegramError::TdlibRuntime("updateChatReadInbox missing `chat_id`".to_owned())
            })?;
            let unread_count = event
                .get("unread_count")
                .and_then(Value::as_i64)
                .ok_or_else(|| {
                    TelegramError::TdlibRuntime(
                        "updateChatReadInbox missing `unread_count`".to_owned(),
                    )
                })?;
            Ok(Some(TelegramTdlibChatUnreadSnapshot {
                provider_chat_id,
                unread_count: Some(unread_count),
                unread_mention_count: None,
                last_read_inbox_message_id: tdlib_event_id(event, "last_read_inbox_message_id"),
                source_event: "updateChatReadInbox".to_owned(),
            }))
        }
        Some("updateChatUnreadMentionCount") => {
            let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
                TelegramError::TdlibRuntime(
                    "updateChatUnreadMentionCount missing `chat_id`".to_owned(),
                )
            })?;
            let unread_mention_count = event
                .get("unread_mention_count")
                .and_then(Value::as_i64)
                .ok_or_else(|| {
                    TelegramError::TdlibRuntime(
                        "updateChatUnreadMentionCount missing `unread_mention_count`".to_owned(),
                    )
                })?;
            Ok(Some(TelegramTdlibChatUnreadSnapshot {
                provider_chat_id,
                unread_count: None,
                unread_mention_count: Some(unread_mention_count),
                last_read_inbox_message_id: None,
                source_event: "updateChatUnreadMentionCount".to_owned(),
            }))
        }
        _ => Ok(None),
    }
}

pub(crate) fn parse_tdlib_chat_marked_as_unread_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatMarkedAsUnreadSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateChatIsMarkedAsUnread") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatIsMarkedAsUnread missing `chat_id`".to_owned())
    })?;
    let is_marked_as_unread = event
        .get("is_marked_as_unread")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "updateChatIsMarkedAsUnread missing `is_marked_as_unread`".to_owned(),
            )
        })?;

    Ok(Some(TelegramTdlibChatMarkedAsUnreadSnapshot {
        provider_chat_id,
        is_marked_as_unread,
        source_event: "updateChatIsMarkedAsUnread".to_owned(),
    }))
}

pub(crate) fn parse_tdlib_chat_notification_settings_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatNotificationSettingsSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateChatNotificationSettings") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatNotificationSettings missing `chat_id`".to_owned())
    })?;
    let notification_settings = event.get("notification_settings").ok_or_else(|| {
        TelegramError::TdlibRuntime(
            "updateChatNotificationSettings missing `notification_settings`".to_owned(),
        )
    })?;
    let use_default_mute_for = notification_settings
        .get("use_default_mute_for")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "updateChatNotificationSettings missing `use_default_mute_for`".to_owned(),
            )
        })?;
    let mute_for = notification_settings
        .get("mute_for")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "updateChatNotificationSettings missing `mute_for`".to_owned(),
            )
        })?;

    Ok(Some(TelegramTdlibChatNotificationSettingsSnapshot {
        provider_chat_id,
        use_default_mute_for,
        mute_for,
        source_event: "updateChatNotificationSettings".to_owned(),
    }))
}

pub(crate) fn parse_tdlib_chat_position_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatPositionSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateChatPosition") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatPosition missing `chat_id`".to_owned())
    })?;
    let position = event.get("position").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatPosition missing `position`".to_owned())
    })?;
    let list = position.get("list").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatPosition missing `position.list`".to_owned())
    })?;
    let list_type = list.get("@type").and_then(Value::as_str).ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatPosition missing `position.list.@type`".to_owned())
    })?;
    let (list_kind, provider_folder_id) = match list_type {
        "chatListMain" => ("main".to_owned(), None),
        "chatListArchive" => ("archive".to_owned(), None),
        "chatListFolder" => (
            "folder".to_owned(),
            list.get("chat_folder_id").and_then(Value::as_i64),
        ),
        // TDLib may add client-only list types. They do not map to Макошь folder
        // semantics, so ignore the update instead of terminating the actor.
        _ => return Ok(None),
    };
    let Some(order) = position.get("order").and_then(Value::as_i64) else {
        return Ok(None);
    };
    let Some(is_pinned) = position.get("is_pinned").and_then(Value::as_bool) else {
        return Ok(None);
    };

    Ok(Some(TelegramTdlibChatPositionSnapshot {
        provider_chat_id,
        list_kind,
        provider_folder_id,
        order,
        is_pinned,
        source_event: "updateChatPosition".to_owned(),
    }))
}

pub(crate) fn parse_tdlib_chat_folder_snapshot(
    event: &Value,
) -> Result<
    Option<crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatFolderSnapshot>,
    TelegramError,
> {
    if event.get("@type").and_then(Value::as_str) != Some("chatFolder") {
        return Ok(None);
    }

    let provider_folder_id = event
        .get("id")
        .and_then(Value::as_i64)
        .ok_or_else(|| TelegramError::TdlibRuntime("chatFolder missing `id`".to_owned()))?;
    let title = tdlib_chat_folder_name(event.get("name"))
        .ok_or_else(|| TelegramError::TdlibRuntime("chatFolder missing `name.text`".to_owned()))?;

    Ok(Some(
        crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatFolderSnapshot {
            provider_folder_id,
            title,
            icon_name: tdlib_chat_folder_icon_name(event.get("icon")),
            color_id: event.get("color_id").and_then(Value::as_i64),
            raw: event.clone(),
        },
    ))
}

pub(crate) fn parse_tdlib_chat_removed_from_list_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatRemovedFromListSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateChatRemovedFromList") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_event_id(event, "chat_id").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatRemovedFromList missing `chat_id`".to_owned())
    })?;
    let list = event.get("chat_list").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateChatRemovedFromList missing `chat_list`".to_owned())
    })?;
    let list_type = list.get("@type").and_then(Value::as_str).ok_or_else(|| {
        TelegramError::TdlibRuntime(
            "updateChatRemovedFromList missing `chat_list.@type`".to_owned(),
        )
    })?;
    let (list_kind, provider_folder_id) = match list_type {
        "chatListMain" => ("main".to_owned(), None),
        "chatListArchive" => ("archive".to_owned(), None),
        "chatListFolder" => (
            "folder".to_owned(),
            list.get("chat_folder_id").and_then(Value::as_i64),
        ),
        _ => return Ok(None),
    };

    Ok(Some(TelegramTdlibChatRemovedFromListSnapshot {
        provider_chat_id,
        list_kind,
        provider_folder_id,
        source_event: "updateChatRemovedFromList".to_owned(),
        raw: event.clone(),
    }))
}

pub(crate) fn parse_tdlib_chat_folders_update_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibChatFoldersUpdateSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateChatFolders") {
        return Ok(None);
    }

    let folders = event
        .get("chat_folders")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("updateChatFolders missing `chat_folders`".to_owned())
        })?;
    let mut snapshots = Vec::with_capacity(folders.len());
    for folder in folders {
        let provider_folder_id = folder.get("id").and_then(Value::as_i64).ok_or_else(|| {
            TelegramError::TdlibRuntime("updateChatFolders folder missing `id`".to_owned())
        })?;
        let title = tdlib_chat_folder_name(folder.get("name")).ok_or_else(|| {
            TelegramError::TdlibRuntime("updateChatFolders folder missing `name.text`".to_owned())
        })?;
        snapshots.push(
            crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatFolderSnapshot {
                provider_folder_id,
                title,
                icon_name: tdlib_chat_folder_icon_name(folder.get("icon")),
                color_id: folder.get("color_id").and_then(Value::as_i64),
                raw: folder.clone(),
            },
        );
    }

    Ok(Some(TelegramTdlibChatFoldersUpdateSnapshot {
        folders: snapshots,
        source_event: "updateChatFolders".to_owned(),
    }))
}

fn tdlib_event_id(event: &Value, key: &str) -> Option<String> {
    event
        .get(key)
        .and_then(|value| {
            value
                .as_i64()
                .map(|number| number.to_string())
                .or_else(|| value.as_str().map(ToOwned::to_owned))
        })
        .filter(|value| value != "0")
}

fn tdlib_sender_id(sender: &Value) -> Option<String> {
    match sender.get("@type").and_then(Value::as_str)? {
        "messageSenderUser" => tdlib_event_id(sender, "user_id").map(|id| format!("user:{id}")),
        "messageSenderChat" => tdlib_event_id(sender, "chat_id").map(|id| format!("chat:{id}")),
        _ => None,
    }
}

fn tdlib_chat_folder_name(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|value| value.get("text"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            value
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
}

fn tdlib_chat_folder_icon_name(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
