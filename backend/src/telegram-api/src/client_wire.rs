//! Generated Telegram authorization payload adapters.

use prost::Message;

use crate::{
    TelegramAccount, TelegramAccountSetup, TelegramAttachmentDownloadState,
    TelegramAttachmentProjection, TelegramAuthorizationStatus, TelegramBlobIntentV1, TelegramChat,
    TelegramChatAvatar, TelegramChatFolder, TelegramChatKind, TelegramChatOperationalState,
    TelegramChatPosition, TelegramChatStateProjection, TelegramClientResponse,
    TelegramCommandRecord, TelegramCredentialBinding, TelegramCredentialPurpose,
    TelegramDownloadFile, TelegramFileSnapshot, TelegramHistoryPage, TelegramHistorySyncMode,
    TelegramMediaKind, TelegramMessageMedia, TelegramMessageMutation, TelegramMessageObservation,
    TelegramMessageProjection, TelegramMessageReferences, TelegramMessageTombstone,
    TelegramMessageVersion, TelegramMessageVersionSource, TelegramOperation, TelegramParticipant,
    TelegramParticipantFilter, TelegramParticipantPage, TelegramPersonSourceIdentity,
    TelegramProviderCommand, TelegramProviderEvent, TelegramProviderQuery,
    TelegramProviderQueryResponse, TelegramReactionObservation, TelegramReactionSummary,
    TelegramRealtimeFrame, TelegramRealtimeReplayPage, TelegramRuntimeReconfiguration,
    TelegramRuntimeReconfigurationRequest, TelegramRuntimeReconfigurationState, TelegramSendMedia,
    TelegramSendMessage, TelegramTombstoneReason, TelegramTopic, TelegramTypingState,
    wire::{
        self, telegram_authorization_request_v1::Request,
        telegram_authorization_response_v1::Response,
    },
};

fn media_kind_name(kind: TelegramMediaKind) -> &'static str {
    match kind {
        TelegramMediaKind::Photo => "photo",
        TelegramMediaKind::Video => "video",
        TelegramMediaKind::Audio => "audio",
        TelegramMediaKind::Document => "document",
        TelegramMediaKind::Animation => "animation",
        TelegramMediaKind::VoiceNote => "voice_note",
    }
}

fn parse_media_kind(value: &str) -> Result<TelegramMediaKind, TelegramAuthorizationWireError> {
    match value {
        "photo" => Ok(TelegramMediaKind::Photo),
        "video" => Ok(TelegramMediaKind::Video),
        "audio" => Ok(TelegramMediaKind::Audio),
        "document" => Ok(TelegramMediaKind::Document),
        "animation" => Ok(TelegramMediaKind::Animation),
        "voice_note" => Ok(TelegramMediaKind::VoiceNote),
        _ => Err(TelegramAuthorizationWireError::InvalidPayload),
    }
}

fn participant_filter_name(filter: TelegramParticipantFilter) -> &'static str {
    match filter {
        TelegramParticipantFilter::Recent => "recent",
        TelegramParticipantFilter::Administrators => "administrators",
    }
}

fn parse_participant_filter(
    value: &str,
) -> Result<TelegramParticipantFilter, TelegramAuthorizationWireError> {
    match value {
        "recent" => Ok(TelegramParticipantFilter::Recent),
        "administrators" => Ok(TelegramParticipantFilter::Administrators),
        _ => Err(TelegramAuthorizationWireError::InvalidPayload),
    }
}

pub fn encode_command(command: &TelegramProviderCommand) -> Vec<u8> {
    use wire::telegram_provider_command_v1::Command;
    let command = match command {
        TelegramProviderCommand::SendText(value) => Command::SendText(wire::SendTextCommand {
            operation_id: value.operation_id.clone(),
            account_id: value.account_id.clone(),
            provider_chat_id: value.provider_chat_id.clone(),
            text: value.text.clone(),
        }),
        TelegramProviderCommand::SendMedia(value) => Command::SendMedia(wire::SendMediaCommand {
            operation_id: value.operation_id.clone(),
            account_id: value.account_id.clone(),
            provider_chat_id: value.provider_chat_id.clone(),
            media_kind: media_kind_name(value.media_kind).to_owned(),
            blob: Some(blob_message(&value.blob)),
            caption: value.caption.clone(),
            filename: value.filename.clone(),
        }),
        TelegramProviderCommand::DownloadFile(value) => {
            Command::DownloadFile(wire::DownloadFileCommand {
                operation_id: value.operation_id.clone(),
                account_id: value.account_id.clone(),
                provider_file_id: value.provider_file_id.clone(),
                priority: value.priority,
            })
        }
        TelegramProviderCommand::Reply {
            operation_id,
            account_id,
            provider_chat_id,
            reply_to_provider_message_id,
            text,
        } => Command::Reply(wire::ReplyCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            reply_to_provider_message_id: reply_to_provider_message_id.clone(),
            text: text.clone(),
        }),
        TelegramProviderCommand::Forward {
            operation_id,
            account_id,
            provider_chat_id,
            from_provider_chat_id,
            from_provider_message_id,
        } => Command::Forward(wire::ForwardCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            from_provider_chat_id: from_provider_chat_id.clone(),
            from_provider_message_id: from_provider_message_id.clone(),
        }),
        TelegramProviderCommand::Edit {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            text,
        } => Command::Edit(wire::EditCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            text: text.clone(),
        }),
        TelegramProviderCommand::Delete {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            revoke,
        } => Command::Delete(wire::DeleteCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            revoke: *revoke,
        }),
        TelegramProviderCommand::RestoreVisibility {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            reason,
        } => Command::RestoreVisibility(wire::RestoreVisibilityCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            reason: reason.clone(),
        }),
        TelegramProviderCommand::Reaction {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            emoji,
            active,
        } => Command::Reaction(wire::ReactionCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            emoji: emoji.clone(),
            active: *active,
        }),
        TelegramProviderCommand::Pin {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            active,
        } => Command::Pin(wire::PinCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            active: *active,
        }),
        TelegramProviderCommand::MarkUnread {
            operation_id,
            account_id,
            provider_chat_id,
            unread,
            read_through_provider_message_id,
        } => Command::MarkUnread(wire::MarkUnreadCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            unread: *unread,
            read_through_provider_message_id: read_through_provider_message_id.clone(),
        }),
        TelegramProviderCommand::Archive {
            operation_id,
            account_id,
            provider_chat_id,
            archived,
        } => Command::Archive(wire::ArchiveCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            archived: *archived,
        }),
        TelegramProviderCommand::Mute {
            operation_id,
            account_id,
            provider_chat_id,
            muted,
        } => Command::Mute(wire::MuteCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            muted: *muted,
        }),
        TelegramProviderCommand::Join {
            operation_id,
            account_id,
            provider_chat_id,
        } => Command::Join(wire::JoinCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
        }),
        TelegramProviderCommand::Leave {
            operation_id,
            account_id,
            provider_chat_id,
        } => Command::Leave(wire::LeaveCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
        }),
        TelegramProviderCommand::AddChatToFolder {
            operation_id,
            account_id,
            provider_chat_id,
            provider_folder_id,
        } => Command::AddChatToFolder(wire::FolderCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_folder_id: *provider_folder_id,
        }),
        TelegramProviderCommand::RemoveChatFromFolder {
            operation_id,
            account_id,
            provider_chat_id,
            provider_folder_id,
        } => Command::RemoveChatFromFolder(wire::FolderCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_folder_id: *provider_folder_id,
        }),
        TelegramProviderCommand::ReassignChatFolders {
            operation_id,
            account_id,
            provider_chat_id,
            target_provider_folder_ids,
        } => Command::ReassignChatFolders(wire::ReassignChatFoldersCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            target_provider_folder_ids: target_provider_folder_ids.clone(),
        }),
        TelegramProviderCommand::SearchMessages {
            operation_id,
            account_id,
            provider_chat_id,
            query,
            limit,
        } => Command::SearchMessages(wire::SearchMessagesCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            query: query.clone(),
            limit: *limit,
        }),
        TelegramProviderCommand::ListParticipants {
            operation_id,
            account_id,
            provider_chat_id,
            filter,
            offset,
            limit,
        } => Command::ListParticipants(wire::ListParticipantsCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            filter: participant_filter_name(*filter).to_owned(),
            offset: *offset,
            limit: *limit,
        }),
        TelegramProviderCommand::ListTopics {
            operation_id,
            account_id,
            provider_chat_id,
            limit,
        } => Command::ListTopics(wire::ListTopicsCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            limit: *limit,
        }),
        TelegramProviderCommand::CreateTopic {
            operation_id,
            account_id,
            provider_chat_id,
            title,
        } => Command::CreateTopic(wire::CreateTopicCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            title: title.clone(),
        }),
        TelegramProviderCommand::SetTopicClosed {
            operation_id,
            account_id,
            provider_chat_id,
            provider_topic_id,
            is_closed,
        } => Command::SetTopicClosed(wire::SetTopicClosedCommand {
            operation_id: operation_id.clone(),
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_topic_id: provider_topic_id.clone(),
            is_closed: *is_closed,
        }),
    };
    wire::TelegramProviderCommandV1 {
        command: Some(command),
    }
    .encode_to_vec()
}

pub fn decode_command(
    bytes: &[u8],
) -> Result<TelegramProviderCommand, TelegramAuthorizationWireError> {
    use wire::telegram_provider_command_v1::Command;
    let message = wire::TelegramProviderCommandV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    match message
        .command
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
    {
        Command::SendText(value) => Ok(TelegramProviderCommand::SendText(TelegramSendMessage {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            text: value.text,
        })),
        Command::SendMedia(value) => Ok(TelegramProviderCommand::SendMedia(TelegramSendMedia {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            media_kind: parse_media_kind(&value.media_kind)?,
            blob: decode_blob(value.blob)?,
            caption: value.caption,
            filename: value.filename,
        })),
        Command::DownloadFile(value) => Ok(TelegramProviderCommand::DownloadFile(
            TelegramDownloadFile {
                operation_id: value.operation_id,
                account_id: value.account_id,
                provider_file_id: value.provider_file_id,
                priority: value.priority,
            },
        )),
        Command::Reply(value) => Ok(TelegramProviderCommand::Reply {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            reply_to_provider_message_id: value.reply_to_provider_message_id,
            text: value.text,
        }),
        Command::Forward(value) => Ok(TelegramProviderCommand::Forward {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            from_provider_chat_id: value.from_provider_chat_id,
            from_provider_message_id: value.from_provider_message_id,
        }),
        Command::Edit(value) => Ok(TelegramProviderCommand::Edit {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            text: value.text,
        }),
        Command::Delete(value) => Ok(TelegramProviderCommand::Delete {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            revoke: value.revoke,
        }),
        Command::RestoreVisibility(value) => Ok(TelegramProviderCommand::RestoreVisibility {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            reason: value.reason,
        }),
        Command::Reaction(value) => Ok(TelegramProviderCommand::Reaction {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            emoji: value.emoji,
            active: value.active,
        }),
        Command::Pin(value) => Ok(TelegramProviderCommand::Pin {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            active: value.active,
        }),
        Command::MarkUnread(value) => Ok(TelegramProviderCommand::MarkUnread {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            unread: value.unread,
            read_through_provider_message_id: value.read_through_provider_message_id,
        }),
        Command::Archive(value) => Ok(TelegramProviderCommand::Archive {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            archived: value.archived,
        }),
        Command::Mute(value) => Ok(TelegramProviderCommand::Mute {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            muted: value.muted,
        }),
        Command::Join(value) => Ok(TelegramProviderCommand::Join {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
        }),
        Command::Leave(value) => Ok(TelegramProviderCommand::Leave {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
        }),
        Command::AddChatToFolder(value) => Ok(TelegramProviderCommand::AddChatToFolder {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_folder_id: value.provider_folder_id,
        }),
        Command::RemoveChatFromFolder(value) => Ok(TelegramProviderCommand::RemoveChatFromFolder {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_folder_id: value.provider_folder_id,
        }),
        Command::ReassignChatFolders(value) => Ok(TelegramProviderCommand::ReassignChatFolders {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            target_provider_folder_ids: value.target_provider_folder_ids,
        }),
        Command::SearchMessages(value) => Ok(TelegramProviderCommand::SearchMessages {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            query: value.query,
            limit: value.limit,
        }),
        Command::ListParticipants(value) => Ok(TelegramProviderCommand::ListParticipants {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            filter: parse_participant_filter(&value.filter)?,
            offset: value.offset,
            limit: value.limit,
        }),
        Command::ListTopics(value) => Ok(TelegramProviderCommand::ListTopics {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            limit: value.limit,
        }),
        Command::CreateTopic(value) => Ok(TelegramProviderCommand::CreateTopic {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            title: value.title,
        }),
        Command::SetTopicClosed(value) => Ok(TelegramProviderCommand::SetTopicClosed {
            operation_id: value.operation_id,
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_topic_id: value.provider_topic_id,
            is_closed: value.is_closed,
        }),
    }
}

fn blob_message(value: &TelegramBlobIntentV1) -> wire::TelegramBlobIntentV1 {
    wire::TelegramBlobIntentV1 {
        blob_ref: value.blob_ref.clone(),
        reference_id: value.reference_id.clone(),
        declared_size: value.declared_size,
        backup_class: value.backup_class,
    }
}

fn decode_blob(
    value: Option<wire::TelegramBlobIntentV1>,
) -> Result<TelegramBlobIntentV1, TelegramAuthorizationWireError> {
    let value = value.ok_or(TelegramAuthorizationWireError::MissingVariant)?;
    (value.reference_id.len() == 16
        && value.reference_id.iter().any(|byte| *byte != 0)
        && value.declared_size > 0
        && (1..=3).contains(&value.backup_class)
        && !value.blob_ref.is_empty())
    .then_some(TelegramBlobIntentV1 {
        blob_ref: value.blob_ref,
        reference_id: value.reference_id,
        declared_size: value.declared_size,
        backup_class: value.backup_class,
    })
    .ok_or(TelegramAuthorizationWireError::InvalidPayload)
}

pub fn encode_query(query: &TelegramProviderQuery) -> Vec<u8> {
    use wire::telegram_provider_query_v1::Query;
    let query = match query {
        TelegramProviderQuery::LoadChats { account_id, limit } => {
            Query::LoadChats(wire::AccountLimitQuery {
                account_id: account_id.clone(),
                limit: *limit,
            })
        }
        TelegramProviderQuery::Chat {
            account_id,
            provider_chat_id,
        } => Query::Chat(wire::ChatIdQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
        }),
        TelegramProviderQuery::ChatAvatar {
            account_id,
            provider_chat_id,
        } => Query::ChatAvatar(wire::ChatIdQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
        }),
        TelegramProviderQuery::LoadHistory {
            account_id,
            provider_chat_id,
            from_message_id,
            mode,
            limit,
        } => Query::LoadHistory(wire::HistoryQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            from_message_id: *from_message_id,
            mode: match mode {
                TelegramHistorySyncMode::Latest => "latest",
                TelegramHistorySyncMode::Older => "older",
                TelegramHistorySyncMode::Full => "full",
            }
            .to_owned(),
            limit: *limit,
        }),
        TelegramProviderQuery::CachedChats { account_id, limit } => {
            Query::CachedChats(wire::AccountLimitQuery {
                account_id: account_id.clone(),
                limit: *limit,
            })
        }
        TelegramProviderQuery::SearchChats {
            account_id,
            query,
            limit,
        } => Query::SearchChats(wire::SearchChatsQuery {
            account_id: account_id.clone(),
            query: query.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::CachedMessages {
            account_id,
            provider_chat_id,
            limit,
        } => Query::CachedMessages(wire::ChatLimitQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::MessageById {
            account_id,
            message_id,
        } => Query::MessageById(wire::MessageIdQuery {
            account_id: account_id.clone(),
            message_id: message_id.clone(),
        }),
        TelegramProviderQuery::RecentMessages {
            account_id,
            provider_chat_id,
            limit,
        } => Query::RecentMessages(wire::RecentMessagesQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::MessagesByIds {
            account_id,
            message_ids,
        } => Query::MessagesByIds(wire::MessageIdsQuery {
            account_id: account_id.clone(),
            message_id: message_ids.clone(),
        }),
        TelegramProviderQuery::MessageVersions {
            account_id,
            message_id,
        } => Query::MessageVersions(wire::MessageIdQuery {
            account_id: account_id.clone(),
            message_id: message_id.clone(),
        }),
        TelegramProviderQuery::MessageTombstones {
            account_id,
            message_id,
        } => Query::MessageTombstones(wire::MessageIdQuery {
            account_id: account_id.clone(),
            message_id: message_id.clone(),
        }),
        TelegramProviderQuery::MessageMutations {
            account_id,
            message_id,
        } => Query::MessageMutations(wire::MessageIdQuery {
            account_id: account_id.clone(),
            message_id: message_id.clone(),
        }),
        TelegramProviderQuery::MessageReferences {
            account_id,
            message_id,
        } => Query::MessageReferences(wire::MessageIdQuery {
            account_id: account_id.clone(),
            message_id: message_id.clone(),
        }),
        TelegramProviderQuery::ReplyChain {
            account_id,
            provider_chat_id,
            provider_message_id,
            limit,
        } => Query::ReplyChain(wire::MessageChainQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::ForwardChain {
            account_id,
            provider_chat_id,
            provider_message_id,
            limit,
        } => Query::ForwardChain(wire::MessageChainQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::Attachment {
            account_id,
            attachment_id,
        } => Query::Attachment(wire::AttachmentIdQuery {
            account_id: account_id.clone(),
            attachment_id: attachment_id.clone(),
        }),
        TelegramProviderQuery::AttachmentForMessage {
            account_id,
            provider_chat_id,
            provider_message_id,
        } => Query::AttachmentForMessage(wire::MessageTargetQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
        }),
        TelegramProviderQuery::File {
            account_id,
            provider_file_id,
        } => Query::File(wire::FileIdQuery {
            account_id: account_id.clone(),
            provider_file_id: provider_file_id.clone(),
        }),
        TelegramProviderQuery::ChatState {
            account_id,
            provider_chat_id,
        } => Query::ChatState(wire::ChatIdQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
        }),
        TelegramProviderQuery::ChatPositions {
            account_id,
            provider_chat_id,
        } => Query::ChatPositions(wire::ChatIdQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
        }),
        TelegramProviderQuery::ChatOperationalState {
            account_id,
            provider_chat_id,
        } => Query::ChatOperationalState(wire::ChatIdQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
        }),
        TelegramProviderQuery::PinnedMessages {
            account_id,
            provider_chat_id,
            limit,
        } => Query::PinnedMessages(wire::ChatLimitQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::SearchMessages {
            account_id,
            provider_chat_id,
            query,
            limit,
        } => Query::SearchMessages(wire::SearchMessagesQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            query: query.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::ListParticipants {
            account_id,
            provider_chat_id,
            filter,
            offset,
            limit,
        } => Query::ListParticipants(wire::ListParticipantsQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            filter: participant_filter_name(*filter).to_owned(),
            offset: *offset,
            limit: *limit,
        }),
        TelegramProviderQuery::BasicGroupParticipants {
            account_id,
            provider_chat_id,
            basic_group_id,
        } => Query::BasicGroupParticipants(wire::BasicGroupParticipantsQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            basic_group_id: *basic_group_id,
        }),
        TelegramProviderQuery::ListTopics {
            account_id,
            provider_chat_id,
            limit,
        } => Query::ListTopics(wire::ChatLimitQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::Topic {
            account_id,
            provider_chat_id,
            provider_topic_id,
        } => Query::Topic(wire::TopicQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_topic_id: provider_topic_id.clone(),
        }),
        TelegramProviderQuery::TopicMessageIds {
            account_id,
            provider_chat_id,
            provider_topic_id,
            limit,
        } => Query::TopicMessageIds(wire::TopicMessageIdsQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_topic_id: provider_topic_id.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::SearchTopics {
            account_id,
            provider_chat_id,
            query,
            limit,
        } => Query::SearchTopics(wire::SearchTopicsQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            query: query.clone(),
            limit: *limit,
        }),
        TelegramProviderQuery::Reactions {
            account_id,
            provider_chat_id,
            provider_message_id,
        } => Query::Reactions(wire::MessageTargetQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
        }),
        TelegramProviderQuery::ReactionSummary {
            account_id,
            provider_chat_id,
            provider_message_id,
        } => Query::ReactionSummary(wire::MessageTargetQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
        }),
        TelegramProviderQuery::ChatFolder {
            account_id,
            provider_folder_id,
        } => Query::ChatFolder(wire::FolderIdQuery {
            account_id: account_id.clone(),
            provider_folder_id: *provider_folder_id,
        }),
        TelegramProviderQuery::ChatFolders {
            account_id,
            provider_folder_ids,
        } => Query::ChatFolders(wire::FolderIdsQuery {
            account_id: account_id.clone(),
            provider_folder_id: provider_folder_ids.clone(),
        }),
        TelegramProviderQuery::Operations { account_id, limit } => {
            Query::Operations(wire::AccountLimitQuery {
                account_id: account_id.clone(),
                limit: *limit,
            })
        }
        TelegramProviderQuery::Commands {
            account_id,
            provider_chat_id,
            provider_message_id,
            command_kinds,
            limit,
        } => Query::Commands(wire::CommandsQuery {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            command_kind: command_kinds.clone(),
            limit: *limit,
        }),
    };
    wire::TelegramProviderQueryV1 { query: Some(query) }.encode_to_vec()
}

pub fn decode_query(bytes: &[u8]) -> Result<TelegramProviderQuery, TelegramAuthorizationWireError> {
    use wire::telegram_provider_query_v1::Query;
    let message = wire::TelegramProviderQueryV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    match message
        .query
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
    {
        Query::LoadChats(v) => Ok(TelegramProviderQuery::LoadChats {
            account_id: v.account_id,
            limit: v.limit,
        }),
        Query::Chat(v) => Ok(TelegramProviderQuery::Chat {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
        }),
        Query::ChatAvatar(v) => Ok(TelegramProviderQuery::ChatAvatar {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
        }),
        Query::LoadHistory(v) => Ok(TelegramProviderQuery::LoadHistory {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            from_message_id: v.from_message_id,
            mode: match v.mode.as_str() {
                "latest" => TelegramHistorySyncMode::Latest,
                "older" => TelegramHistorySyncMode::Older,
                "full" => TelegramHistorySyncMode::Full,
                _ => return Err(TelegramAuthorizationWireError::InvalidPayload),
            },
            limit: v.limit,
        }),
        Query::CachedChats(v) => Ok(TelegramProviderQuery::CachedChats {
            account_id: v.account_id,
            limit: v.limit,
        }),
        Query::SearchChats(v) => Ok(TelegramProviderQuery::SearchChats {
            account_id: v.account_id,
            query: v.query,
            limit: v.limit,
        }),
        Query::CachedMessages(v) => Ok(TelegramProviderQuery::CachedMessages {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            limit: v.limit,
        }),
        Query::MessageById(v) => Ok(TelegramProviderQuery::MessageById {
            account_id: v.account_id,
            message_id: v.message_id,
        }),
        Query::RecentMessages(v) => Ok(TelegramProviderQuery::RecentMessages {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            limit: v.limit,
        }),
        Query::MessagesByIds(v) => Ok(TelegramProviderQuery::MessagesByIds {
            account_id: v.account_id,
            message_ids: v.message_id,
        }),
        Query::MessageVersions(v) => Ok(TelegramProviderQuery::MessageVersions {
            account_id: v.account_id,
            message_id: v.message_id,
        }),
        Query::MessageTombstones(v) => Ok(TelegramProviderQuery::MessageTombstones {
            account_id: v.account_id,
            message_id: v.message_id,
        }),
        Query::MessageMutations(v) => Ok(TelegramProviderQuery::MessageMutations {
            account_id: v.account_id,
            message_id: v.message_id,
        }),
        Query::MessageReferences(v) => Ok(TelegramProviderQuery::MessageReferences {
            account_id: v.account_id,
            message_id: v.message_id,
        }),
        Query::ReplyChain(v) => Ok(TelegramProviderQuery::ReplyChain {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            provider_message_id: v.provider_message_id,
            limit: v.limit,
        }),
        Query::ForwardChain(v) => Ok(TelegramProviderQuery::ForwardChain {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            provider_message_id: v.provider_message_id,
            limit: v.limit,
        }),
        Query::Attachment(v) => Ok(TelegramProviderQuery::Attachment {
            account_id: v.account_id,
            attachment_id: v.attachment_id,
        }),
        Query::AttachmentForMessage(v) => Ok(TelegramProviderQuery::AttachmentForMessage {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            provider_message_id: v.provider_message_id,
        }),
        Query::File(v) => Ok(TelegramProviderQuery::File {
            account_id: v.account_id,
            provider_file_id: v.provider_file_id,
        }),
        Query::ChatState(v) => Ok(TelegramProviderQuery::ChatState {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
        }),
        Query::ChatPositions(v) => Ok(TelegramProviderQuery::ChatPositions {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
        }),
        Query::ChatOperationalState(v) => Ok(TelegramProviderQuery::ChatOperationalState {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
        }),
        Query::PinnedMessages(v) => Ok(TelegramProviderQuery::PinnedMessages {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            limit: v.limit,
        }),
        Query::SearchMessages(v) => Ok(TelegramProviderQuery::SearchMessages {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            query: v.query,
            limit: v.limit,
        }),
        Query::ListParticipants(v) => Ok(TelegramProviderQuery::ListParticipants {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            filter: parse_participant_filter(&v.filter)?,
            offset: v.offset,
            limit: v.limit,
        }),
        Query::BasicGroupParticipants(v) => Ok(TelegramProviderQuery::BasicGroupParticipants {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            basic_group_id: v.basic_group_id,
        }),
        Query::ListTopics(v) => Ok(TelegramProviderQuery::ListTopics {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            limit: v.limit,
        }),
        Query::Topic(v) => Ok(TelegramProviderQuery::Topic {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            provider_topic_id: v.provider_topic_id,
        }),
        Query::TopicMessageIds(v) => Ok(TelegramProviderQuery::TopicMessageIds {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            provider_topic_id: v.provider_topic_id,
            limit: v.limit,
        }),
        Query::SearchTopics(v) => Ok(TelegramProviderQuery::SearchTopics {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            query: v.query,
            limit: v.limit,
        }),
        Query::Reactions(v) => Ok(TelegramProviderQuery::Reactions {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            provider_message_id: v.provider_message_id,
        }),
        Query::ReactionSummary(v) => Ok(TelegramProviderQuery::ReactionSummary {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            provider_message_id: v.provider_message_id,
        }),
        Query::ChatFolder(v) => Ok(TelegramProviderQuery::ChatFolder {
            account_id: v.account_id,
            provider_folder_id: v.provider_folder_id,
        }),
        Query::ChatFolders(v) => Ok(TelegramProviderQuery::ChatFolders {
            account_id: v.account_id,
            provider_folder_ids: v.provider_folder_id,
        }),
        Query::Operations(v) => Ok(TelegramProviderQuery::Operations {
            account_id: v.account_id,
            limit: v.limit,
        }),
        Query::Commands(v) => Ok(TelegramProviderQuery::Commands {
            account_id: v.account_id,
            provider_chat_id: v.provider_chat_id,
            provider_message_id: v.provider_message_id,
            command_kinds: v.command_kind,
            limit: v.limit,
        }),
    }
}

fn chat_kind_name(kind: TelegramChatKind) -> &'static str {
    match kind {
        TelegramChatKind::Private => "private",
        TelegramChatKind::Group => "group",
        TelegramChatKind::Channel => "channel",
        TelegramChatKind::Bot => "bot",
    }
}

fn parse_chat_kind(value: &str) -> Result<TelegramChatKind, TelegramAuthorizationWireError> {
    match value {
        "private" => Ok(TelegramChatKind::Private),
        "group" => Ok(TelegramChatKind::Group),
        "channel" => Ok(TelegramChatKind::Channel),
        "bot" => Ok(TelegramChatKind::Bot),
        _ => Err(TelegramAuthorizationWireError::InvalidPayload),
    }
}

fn message_media_to_wire(value: &TelegramMessageMedia) -> wire::TelegramMessageMediaProjection {
    wire::TelegramMessageMediaProjection {
        kind: media_kind_name(value.kind).to_owned(),
        provider_file_id: value.provider_file_id.clone(),
        caption: value.caption.clone(),
        filename: value.filename.clone(),
        content_type: value.content_type.clone(),
        preview_provider_file_id: value.preview_provider_file_id.clone(),
        preview_content_type: value.preview_content_type.clone(),
        preview_inline_data: value.preview_inline_data.clone(),
        preview_metadata_loaded: value.preview_metadata_loaded,
    }
}

fn message_references_to_wire(
    value: &TelegramMessageReferences,
) -> wire::TelegramMessageReferencesProjection {
    wire::TelegramMessageReferencesProjection {
        reply_to: value
            .reply_to
            .as_ref()
            .map(|reply| wire::TelegramReplyReferenceProjection {
                provider_chat_id: reply.provider_chat_id.clone(),
                provider_message_id: reply.provider_message_id.clone(),
            }),
        forward_origin: value.forward_origin.as_ref().map(|origin| {
            wire::TelegramForwardOriginProjection {
                provider_chat_id: origin.provider_chat_id.clone(),
                provider_message_id: origin.provider_message_id.clone(),
                provider_sender_id: origin.provider_sender_id.clone(),
                sender_name: origin.sender_name.clone(),
                observed_at_unix_seconds: origin.observed_at_unix_seconds,
            }
        }),
    }
}

fn person_source_identity_to_wire(
    value: &TelegramPersonSourceIdentity,
) -> wire::TelegramPersonSourceIdentityProjection {
    wire::TelegramPersonSourceIdentityProjection {
        integration_public_id: value.integration_public_id.to_vec(),
        account_public_id: value.account_public_id.to_vec(),
        provider_source_contact_public_id: value.provider_source_contact_public_id.to_vec(),
    }
}

fn parse_person_source_identity(
    value: wire::TelegramPersonSourceIdentityProjection,
) -> Result<TelegramPersonSourceIdentity, TelegramAuthorizationWireError> {
    Ok(TelegramPersonSourceIdentity {
        integration_public_id: value
            .integration_public_id
            .try_into()
            .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?,
        account_public_id: value
            .account_public_id
            .try_into()
            .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?,
        provider_source_contact_public_id: value
            .provider_source_contact_public_id
            .try_into()
            .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?,
    })
}

fn message_observation_to_wire(
    value: &TelegramMessageObservation,
) -> wire::TelegramMessageObservationProjection {
    wire::TelegramMessageObservationProjection {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_message_id: value.provider_message_id.clone(),
        provider_topic_id: value.provider_topic_id.clone(),
        sender_id: value.sender_id.clone(),
        sender_display_name: value.sender_display_name.clone(),
        sender_source_identity: value
            .sender_source_identity
            .as_ref()
            .map(person_source_identity_to_wire),
        is_outgoing: value.is_outgoing,
        text: value.text.clone(),
        media: value.media.as_ref().map(message_media_to_wire),
        references: Some(message_references_to_wire(&value.references)),
        observed_at_unix_seconds: value.observed_at_unix_seconds,
    }
}

fn parse_message_observation(
    value: wire::TelegramMessageObservationProjection,
) -> Result<TelegramMessageObservation, TelegramAuthorizationWireError> {
    let references = value
        .references
        .ok_or(TelegramAuthorizationWireError::InvalidPayload)?;
    let reply_to = references
        .reply_to
        .map(|reply| crate::TelegramReplyReference {
            provider_chat_id: reply.provider_chat_id,
            provider_message_id: reply.provider_message_id,
        });
    let forward_origin = references
        .forward_origin
        .map(|origin| crate::TelegramForwardOrigin {
            provider_chat_id: origin.provider_chat_id,
            provider_message_id: origin.provider_message_id,
            provider_sender_id: origin.provider_sender_id,
            sender_name: origin.sender_name,
            observed_at_unix_seconds: origin.observed_at_unix_seconds,
        });
    let media = value
        .media
        .map(|media| {
            Ok(TelegramMessageMedia {
                kind: parse_media_kind(&media.kind)?,
                provider_file_id: media.provider_file_id,
                caption: media.caption,
                filename: media.filename,
                content_type: media.content_type,
                preview_provider_file_id: media.preview_provider_file_id,
                preview_content_type: media.preview_content_type,
                preview_inline_data: media.preview_inline_data,
                preview_metadata_loaded: media.preview_metadata_loaded,
            })
        })
        .transpose()?;
    Ok(TelegramMessageObservation {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        provider_message_id: value.provider_message_id,
        provider_topic_id: value.provider_topic_id,
        sender_id: value.sender_id,
        sender_display_name: value.sender_display_name,
        sender_source_identity: value
            .sender_source_identity
            .map(parse_person_source_identity)
            .transpose()?,
        is_outgoing: value.is_outgoing,
        text: value.text,
        media,
        references: TelegramMessageReferences {
            reply_to,
            forward_origin,
        },
        observed_at_unix_seconds: value.observed_at_unix_seconds,
    })
}

fn chat_to_wire(value: &TelegramChat) -> wire::TelegramChatProjection {
    wire::TelegramChatProjection {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        kind: chat_kind_name(value.kind).to_owned(),
        title: value.title.clone(),
        username: value.username.clone(),
        avatar_provider_file_id: value.avatar_provider_file_id.clone(),
        avatar_provider_unique_id: value.avatar_provider_unique_id.clone(),
    }
}

fn parse_chat(
    value: wire::TelegramChatProjection,
) -> Result<TelegramChat, TelegramAuthorizationWireError> {
    Ok(TelegramChat {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        kind: parse_chat_kind(&value.kind)?,
        title: value.title,
        username: value.username,
        avatar_provider_file_id: value.avatar_provider_file_id,
        avatar_provider_unique_id: value.avatar_provider_unique_id,
    })
}

fn avatar_to_wire(value: &TelegramChatAvatar) -> wire::TelegramChatAvatarProjection {
    wire::TelegramChatAvatarProjection {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_file_id: value.provider_file_id.clone(),
        provider_unique_id: value.provider_unique_id.clone(),
    }
}

fn parse_avatar(value: wire::TelegramChatAvatarProjection) -> TelegramChatAvatar {
    TelegramChatAvatar {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        provider_file_id: value.provider_file_id,
        provider_unique_id: value.provider_unique_id,
    }
}

fn chat_state_to_wire(
    value: &crate::TelegramChatStateProjection,
) -> wire::TelegramChatStateProjection {
    wire::TelegramChatStateProjection {
        unread_count: value.unread_count,
        unread_mention_count: value.unread_mention_count,
        last_read_inbox_message_id: value.last_read_inbox_message_id.clone(),
        is_marked_as_unread: value.is_marked_as_unread,
    }
}

fn parse_chat_state(value: wire::TelegramChatStateProjection) -> TelegramChatStateProjection {
    TelegramChatStateProjection {
        unread_count: value.unread_count,
        unread_mention_count: value.unread_mention_count,
        last_read_inbox_message_id: value.last_read_inbox_message_id,
        is_marked_as_unread: value.is_marked_as_unread,
    }
}

fn chat_position_to_wire(value: &TelegramChatPosition) -> wire::TelegramChatPositionProjection {
    wire::TelegramChatPositionProjection {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        list_kind: value.list_kind.clone(),
        provider_folder_id: value.provider_folder_id,
        order: value.order,
        is_pinned: value.is_pinned,
    }
}

fn parse_chat_position(value: wire::TelegramChatPositionProjection) -> TelegramChatPosition {
    TelegramChatPosition {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        list_kind: value.list_kind,
        provider_folder_id: value.provider_folder_id,
        order: value.order,
        is_pinned: value.is_pinned,
    }
}

fn chat_operational_state_to_wire(
    value: &TelegramChatOperationalState,
) -> wire::TelegramChatOperationalStateProjection {
    wire::TelegramChatOperationalStateProjection {
        is_archived: value.is_archived,
        is_pinned: value.is_pinned,
        is_muted: value.is_muted,
        mute_for_seconds: value.mute_for_seconds,
        is_marked_as_unread: value.is_marked_as_unread,
    }
}

fn parse_chat_operational_state(
    value: wire::TelegramChatOperationalStateProjection,
) -> TelegramChatOperationalState {
    TelegramChatOperationalState {
        is_archived: value.is_archived,
        is_pinned: value.is_pinned,
        is_muted: value.is_muted,
        mute_for_seconds: value.mute_for_seconds,
        is_marked_as_unread: value.is_marked_as_unread,
    }
}

fn chat_folder_to_wire(value: &TelegramChatFolder) -> wire::TelegramChatFolderProjection {
    wire::TelegramChatFolderProjection {
        account_id: value.account_id.clone(),
        provider_folder_id: value.provider_folder_id,
        title: value.title.clone(),
        icon_name: value.icon_name.clone(),
        color_id: value.color_id,
        pinned_chat_id: value.pinned_chat_ids.clone(),
        included_chat_id: value.included_chat_ids.clone(),
        excluded_chat_id: value.excluded_chat_ids.clone(),
    }
}

fn parse_chat_folder(value: wire::TelegramChatFolderProjection) -> TelegramChatFolder {
    TelegramChatFolder {
        account_id: value.account_id,
        provider_folder_id: value.provider_folder_id,
        title: value.title,
        icon_name: value.icon_name,
        color_id: value.color_id,
        pinned_chat_ids: value.pinned_chat_id,
        included_chat_ids: value.included_chat_id,
        excluded_chat_ids: value.excluded_chat_id,
    }
}

fn delivery_state_name(value: crate::TelegramDeliveryState) -> &'static str {
    match value {
        crate::TelegramDeliveryState::Received => "received",
        crate::TelegramDeliveryState::Queued => "queued",
        crate::TelegramDeliveryState::Sent => "sent",
        crate::TelegramDeliveryState::SendFailed => "send_failed",
    }
}

fn parse_delivery_state(
    value: &str,
) -> Result<crate::TelegramDeliveryState, TelegramAuthorizationWireError> {
    match value {
        "received" => Ok(crate::TelegramDeliveryState::Received),
        "queued" => Ok(crate::TelegramDeliveryState::Queued),
        "sent" => Ok(crate::TelegramDeliveryState::Sent),
        "send_failed" => Ok(crate::TelegramDeliveryState::SendFailed),
        _ => Err(TelegramAuthorizationWireError::InvalidPayload),
    }
}

fn message_projection_to_wire(
    value: &TelegramMessageProjection,
) -> wire::TelegramMessageProjection {
    wire::TelegramMessageProjection {
        message_id: value.message_id.clone(),
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_message_id: value.provider_message_id.clone(),
        provider_topic_id: value.provider_topic_id.clone(),
        sender_id: value.sender_id.clone(),
        sender_display_name: value.sender_display_name.clone(),
        sender_source_identity: value
            .sender_source_identity
            .as_ref()
            .map(person_source_identity_to_wire),
        text: value.text.clone(),
        media: value.media.as_ref().map(message_media_to_wire),
        references: Some(message_references_to_wire(&value.references)),
        observed_at_unix_seconds: value.observed_at_unix_seconds,
        delivery_state: delivery_state_name(value.delivery_state).to_owned(),
    }
}

fn parse_message_projection(
    value: wire::TelegramMessageProjection,
) -> Result<TelegramMessageProjection, TelegramAuthorizationWireError> {
    let references = value
        .references
        .ok_or(TelegramAuthorizationWireError::InvalidPayload)?;
    let media = value
        .media
        .map(|media| {
            Ok(TelegramMessageMedia {
                kind: parse_media_kind(&media.kind)?,
                provider_file_id: media.provider_file_id,
                caption: media.caption,
                filename: media.filename,
                content_type: media.content_type,
                preview_provider_file_id: media.preview_provider_file_id,
                preview_content_type: media.preview_content_type,
                preview_inline_data: media.preview_inline_data,
                preview_metadata_loaded: media.preview_metadata_loaded,
            })
        })
        .transpose()?;
    Ok(TelegramMessageProjection {
        message_id: value.message_id,
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        provider_message_id: value.provider_message_id,
        provider_topic_id: value.provider_topic_id,
        sender_id: value.sender_id,
        sender_display_name: value.sender_display_name,
        sender_source_identity: value
            .sender_source_identity
            .map(parse_person_source_identity)
            .transpose()?,
        text: value.text,
        media,
        references: TelegramMessageReferences {
            reply_to: references
                .reply_to
                .map(|reply| crate::TelegramReplyReference {
                    provider_chat_id: reply.provider_chat_id,
                    provider_message_id: reply.provider_message_id,
                }),
            forward_origin: references
                .forward_origin
                .map(|origin| crate::TelegramForwardOrigin {
                    provider_chat_id: origin.provider_chat_id,
                    provider_message_id: origin.provider_message_id,
                    provider_sender_id: origin.provider_sender_id,
                    sender_name: origin.sender_name,
                    observed_at_unix_seconds: origin.observed_at_unix_seconds,
                }),
        },
        observed_at_unix_seconds: value.observed_at_unix_seconds,
        delivery_state: parse_delivery_state(&value.delivery_state)?,
    })
}

fn tombstone_reason_name(value: TelegramTombstoneReason) -> &'static str {
    match value {
        TelegramTombstoneReason::ProviderDeleted => "provider_deleted",
        TelegramTombstoneReason::OwnerDeleted => "owner_deleted",
        TelegramTombstoneReason::Unknown => "unknown",
    }
}

fn parse_tombstone_reason(
    value: &str,
) -> Result<TelegramTombstoneReason, TelegramAuthorizationWireError> {
    match value {
        "provider_deleted" => Ok(TelegramTombstoneReason::ProviderDeleted),
        "owner_deleted" => Ok(TelegramTombstoneReason::OwnerDeleted),
        "unknown" => Ok(TelegramTombstoneReason::Unknown),
        _ => Err(TelegramAuthorizationWireError::InvalidPayload),
    }
}

fn version_source_name(value: TelegramMessageVersionSource) -> &'static str {
    match value {
        TelegramMessageVersionSource::Provider => "provider",
        TelegramMessageVersionSource::Owner => "owner",
    }
}

fn parse_version_source(
    value: &str,
) -> Result<TelegramMessageVersionSource, TelegramAuthorizationWireError> {
    match value {
        "provider" => Ok(TelegramMessageVersionSource::Provider),
        "owner" => Ok(TelegramMessageVersionSource::Owner),
        _ => Err(TelegramAuthorizationWireError::InvalidPayload),
    }
}

fn attachment_state_name(value: TelegramAttachmentDownloadState) -> &'static str {
    match value {
        TelegramAttachmentDownloadState::Pending => "pending",
        TelegramAttachmentDownloadState::Downloading => "downloading",
        TelegramAttachmentDownloadState::Downloaded => "downloaded",
        TelegramAttachmentDownloadState::Failed => "failed",
    }
}

fn parse_attachment_state(
    value: &str,
) -> Result<TelegramAttachmentDownloadState, TelegramAuthorizationWireError> {
    match value {
        "pending" => Ok(TelegramAttachmentDownloadState::Pending),
        "downloading" => Ok(TelegramAttachmentDownloadState::Downloading),
        "downloaded" => Ok(TelegramAttachmentDownloadState::Downloaded),
        "failed" => Ok(TelegramAttachmentDownloadState::Failed),
        _ => Err(TelegramAuthorizationWireError::InvalidPayload),
    }
}

fn file_to_wire(value: &TelegramFileSnapshot) -> wire::TelegramFileSnapshotProjection {
    wire::TelegramFileSnapshotProjection {
        account_id: value.account_id.clone(),
        provider_file_id: value.provider_file_id.clone(),
        provider_unique_id: value.provider_unique_id.clone(),
        media_kind: value
            .media_kind
            .map(|kind| media_kind_name(kind).to_owned()),
        size_bytes: value.size_bytes,
        expected_size_bytes: value.expected_size_bytes,
        downloaded_size_bytes: value.downloaded_size_bytes,
        is_downloading: value.is_downloading,
        is_downloaded: value.is_downloaded,
        blob_reference_id: value.blob_reference_id.clone(),
        blob_plaintext_sha256: value.blob_plaintext_sha256.clone(),
        blob_backup_class: value.blob_backup_class,
    }
}

fn parse_file(
    value: wire::TelegramFileSnapshotProjection,
) -> Result<TelegramFileSnapshot, TelegramAuthorizationWireError> {
    Ok(TelegramFileSnapshot {
        account_id: value.account_id,
        provider_file_id: value.provider_file_id,
        provider_unique_id: value.provider_unique_id,
        media_kind: value
            .media_kind
            .as_deref()
            .map(parse_media_kind)
            .transpose()?,
        size_bytes: value.size_bytes,
        expected_size_bytes: value.expected_size_bytes,
        downloaded_size_bytes: value.downloaded_size_bytes,
        is_downloading: value.is_downloading,
        is_downloaded: value.is_downloaded,
        blob_reference_id: value.blob_reference_id,
        blob_plaintext_sha256: value.blob_plaintext_sha256,
        blob_backup_class: value.blob_backup_class,
    })
}

fn participant_to_wire(value: &TelegramParticipant) -> wire::TelegramParticipantProjection {
    wire::TelegramParticipantProjection {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_member_id: value.provider_member_id.clone(),
        display_name: value.display_name.clone(),
        username: value.username.clone(),
        role: value.role.clone(),
        status: value.status.clone(),
        is_admin: value.is_admin,
        is_owner: value.is_owner,
        permission: value.permissions.clone(),
    }
}

fn parse_participant(value: wire::TelegramParticipantProjection) -> TelegramParticipant {
    TelegramParticipant {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        provider_member_id: value.provider_member_id,
        display_name: value.display_name,
        username: value.username,
        role: value.role,
        status: value.status,
        is_admin: value.is_admin,
        is_owner: value.is_owner,
        permissions: value.permission,
    }
}

fn topic_to_wire(value: &TelegramTopic) -> wire::TelegramTopicProjection {
    wire::TelegramTopicProjection {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_topic_id: value.provider_topic_id.clone(),
        title: value.title.clone(),
        icon_emoji: value.icon_emoji.clone(),
        is_pinned: value.is_pinned,
        is_closed: value.is_closed,
        unread_count: value.unread_count,
        last_message_at_unix_seconds: value.last_message_at_unix_seconds,
    }
}

fn parse_topic(value: wire::TelegramTopicProjection) -> TelegramTopic {
    TelegramTopic {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        provider_topic_id: value.provider_topic_id,
        title: value.title,
        icon_emoji: value.icon_emoji,
        is_pinned: value.is_pinned,
        is_closed: value.is_closed,
        unread_count: value.unread_count,
        last_message_at_unix_seconds: value.last_message_at_unix_seconds,
    }
}

fn typing_to_wire(value: &TelegramTypingState) -> wire::TelegramTypingStateProjection {
    wire::TelegramTypingStateProjection {
        account_id: value.account_id.clone(),
        provider_chat_id: value.provider_chat_id.clone(),
        provider_thread_id: value.provider_thread_id.clone(),
        sender_id: value.sender_id.clone(),
        action: value.action.clone(),
        is_active: value.is_active,
    }
}

fn parse_typing(value: wire::TelegramTypingStateProjection) -> TelegramTypingState {
    TelegramTypingState {
        account_id: value.account_id,
        provider_chat_id: value.provider_chat_id,
        provider_thread_id: value.provider_thread_id,
        sender_id: value.sender_id,
        action: value.action,
        is_active: value.is_active,
    }
}

fn event_to_wire(value: &TelegramProviderEvent) -> wire::TelegramProviderEventProjection {
    use wire::telegram_provider_event_projection::Event;
    let event = match value {
        TelegramProviderEvent::MessageCreated(value) => {
            Event::MessageCreated(message_observation_to_wire(value))
        }
        TelegramProviderEvent::MessageEdited {
            account_id,
            provider_chat_id,
            provider_message_id,
            text,
            observed_at_unix_seconds,
        } => Event::MessageEdited(wire::MessageEditedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            text: text.clone(),
            observed_at_unix_seconds: *observed_at_unix_seconds,
        }),
        TelegramProviderEvent::MessageDeleted {
            account_id,
            provider_chat_id,
            provider_message_id,
            is_permanent,
        } => Event::MessageDeleted(wire::MessageDeletedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            is_permanent: *is_permanent,
        }),
        TelegramProviderEvent::MessageSendFailed {
            account_id,
            provider_chat_id,
            old_provider_message_id,
            error_code,
        } => Event::MessageSendFailed(wire::MessageSendFailedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            old_provider_message_id: old_provider_message_id.clone(),
            error_code: *error_code,
        }),
        TelegramProviderEvent::MessageSendSucceeded {
            account_id,
            provider_chat_id,
            old_provider_message_id,
            provider_message_id,
        } => Event::MessageSendSucceeded(wire::MessageSendSucceededEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            old_provider_message_id: old_provider_message_id.clone(),
            provider_message_id: provider_message_id.clone(),
        }),
        TelegramProviderEvent::MessagePinned {
            account_id,
            provider_chat_id,
            provider_message_id,
            is_pinned,
        } => Event::MessagePinned(wire::MessagePinnedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            is_pinned: *is_pinned,
        }),
        TelegramProviderEvent::ReactionChanged {
            account_id,
            provider_chat_id,
            provider_message_id,
            emoji,
            is_active,
        } => Event::ReactionChanged(wire::ReactionChangedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            emoji: emoji.clone(),
            is_active: *is_active,
        }),
        TelegramProviderEvent::ReactionsObserved {
            account_id,
            provider_chat_id,
            provider_message_id,
            reactions,
        } => Event::ReactionsObserved(wire::ReactionsObservedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            reaction: reactions
                .iter()
                .map(|value| wire::TelegramReactionObservationProjection {
                    sender_id: value.sender_id.clone(),
                    emoji: value.emoji.clone(),
                    is_outgoing: value.is_outgoing,
                    is_active: value.is_active,
                })
                .collect(),
        }),
        TelegramProviderEvent::ChatUnreadChanged {
            account_id,
            provider_chat_id,
            unread_count,
            unread_mention_count,
            last_read_inbox_message_id,
        } => Event::ChatUnreadChanged(wire::ChatUnreadChangedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            unread_count: *unread_count,
            unread_mention_count: *unread_mention_count,
            last_read_inbox_message_id: last_read_inbox_message_id.clone(),
        }),
        TelegramProviderEvent::ChatMarkedUnreadChanged {
            account_id,
            provider_chat_id,
            is_marked_as_unread,
        } => Event::ChatMarkedUnreadChanged(wire::ChatMarkedUnreadChangedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            is_marked_as_unread: *is_marked_as_unread,
        }),
        TelegramProviderEvent::TypingChanged(value) => Event::TypingChanged(typing_to_wire(value)),
        TelegramProviderEvent::TopicChanged(value) => Event::TopicChanged(topic_to_wire(value)),
        TelegramProviderEvent::ChatPositionChanged(value) => {
            Event::ChatPositionChanged(chat_position_to_wire(value))
        }
        TelegramProviderEvent::ChatFoldersChanged {
            account_id,
            folders,
        } => Event::ChatFoldersChanged(wire::ChatFoldersChangedEvent {
            account_id: account_id.clone(),
            folder: folders.iter().map(chat_folder_to_wire).collect(),
        }),
        TelegramProviderEvent::ChatNotificationChanged {
            account_id,
            provider_chat_id,
            use_default_mute_for,
            mute_for_seconds,
        } => Event::ChatNotificationChanged(wire::ChatNotificationChangedEvent {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            use_default_mute_for: *use_default_mute_for,
            mute_for_seconds: *mute_for_seconds,
        }),
        TelegramProviderEvent::ChatAvatarChanged(value) => {
            Event::ChatAvatarChanged(avatar_to_wire(value))
        }
        TelegramProviderEvent::ParticipantChanged(value) => {
            Event::ParticipantChanged(participant_to_wire(value))
        }
        TelegramProviderEvent::FileChanged(value) => Event::FileChanged(file_to_wire(value)),
    };
    wire::TelegramProviderEventProjection { event: Some(event) }
}

fn parse_event(
    value: wire::TelegramProviderEventProjection,
) -> Result<TelegramProviderEvent, TelegramAuthorizationWireError> {
    use wire::telegram_provider_event_projection::Event;
    match value
        .event
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
    {
        Event::MessageCreated(value) => Ok(TelegramProviderEvent::MessageCreated(
            parse_message_observation(value)?,
        )),
        Event::MessageEdited(value) => Ok(TelegramProviderEvent::MessageEdited {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            text: value.text,
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }),
        Event::MessageDeleted(value) => Ok(TelegramProviderEvent::MessageDeleted {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            is_permanent: value.is_permanent,
        }),
        Event::MessageSendFailed(value) => Ok(TelegramProviderEvent::MessageSendFailed {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            old_provider_message_id: value.old_provider_message_id,
            error_code: value.error_code,
        }),
        Event::MessageSendSucceeded(value) => Ok(TelegramProviderEvent::MessageSendSucceeded {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            old_provider_message_id: value.old_provider_message_id,
            provider_message_id: value.provider_message_id,
        }),
        Event::MessagePinned(value) => Ok(TelegramProviderEvent::MessagePinned {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            is_pinned: value.is_pinned,
        }),
        Event::ReactionChanged(value) => Ok(TelegramProviderEvent::ReactionChanged {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            emoji: value.emoji,
            is_active: value.is_active,
        }),
        Event::ReactionsObserved(value) => Ok(TelegramProviderEvent::ReactionsObserved {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            provider_message_id: value.provider_message_id,
            reactions: value
                .reaction
                .into_iter()
                .map(|value| TelegramReactionObservation {
                    sender_id: value.sender_id,
                    emoji: value.emoji,
                    is_outgoing: value.is_outgoing,
                    is_active: value.is_active,
                })
                .collect(),
        }),
        Event::ChatUnreadChanged(value) => Ok(TelegramProviderEvent::ChatUnreadChanged {
            account_id: value.account_id,
            provider_chat_id: value.provider_chat_id,
            unread_count: value.unread_count,
            unread_mention_count: value.unread_mention_count,
            last_read_inbox_message_id: value.last_read_inbox_message_id,
        }),
        Event::ChatMarkedUnreadChanged(value) => {
            Ok(TelegramProviderEvent::ChatMarkedUnreadChanged {
                account_id: value.account_id,
                provider_chat_id: value.provider_chat_id,
                is_marked_as_unread: value.is_marked_as_unread,
            })
        }
        Event::TypingChanged(value) => {
            Ok(TelegramProviderEvent::TypingChanged(parse_typing(value)))
        }
        Event::TopicChanged(value) => Ok(TelegramProviderEvent::TopicChanged(parse_topic(value))),
        Event::ChatPositionChanged(value) => Ok(TelegramProviderEvent::ChatPositionChanged(
            parse_chat_position(value),
        )),
        Event::ChatFoldersChanged(value) => Ok(TelegramProviderEvent::ChatFoldersChanged {
            account_id: value.account_id,
            folders: value.folder.into_iter().map(parse_chat_folder).collect(),
        }),
        Event::ChatNotificationChanged(value) => {
            Ok(TelegramProviderEvent::ChatNotificationChanged {
                account_id: value.account_id,
                provider_chat_id: value.provider_chat_id,
                use_default_mute_for: value.use_default_mute_for,
                mute_for_seconds: value.mute_for_seconds,
            })
        }
        Event::ChatAvatarChanged(value) => Ok(TelegramProviderEvent::ChatAvatarChanged(
            parse_avatar(value),
        )),
        Event::ParticipantChanged(value) => Ok(TelegramProviderEvent::ParticipantChanged(
            parse_participant(value),
        )),
        Event::FileChanged(value) => Ok(TelegramProviderEvent::FileChanged(parse_file(value)?)),
    }
}

pub fn encode_query_response(response: &TelegramProviderQueryResponse) -> Option<Vec<u8>> {
    use wire::telegram_provider_query_response_v1::Response;
    let response = match response {
        TelegramProviderQueryResponse::Chats(values) => Response::Chats(wire::ChatListResponse {
            chat: values.iter().map(chat_to_wire).collect(),
        }),
        TelegramProviderQueryResponse::Chat(value) => Response::Chat(wire::ChatResponse {
            chat: value.as_ref().map(chat_to_wire),
        }),
        TelegramProviderQueryResponse::ChatAvatar(value) => {
            Response::ChatAvatar(wire::ChatAvatarResponse {
                avatar: value.as_ref().map(avatar_to_wire),
            })
        }
        TelegramProviderQueryResponse::History(values) => {
            Response::History(wire::MessageObservationListResponse {
                item: values.iter().map(message_observation_to_wire).collect(),
            })
        }
        TelegramProviderQueryResponse::HistoryPage(value) => {
            Response::HistoryPage(wire::HistoryPageResponse {
                page: Some(wire::TelegramHistoryPageProjection {
                    item: value
                        .items
                        .iter()
                        .map(message_observation_to_wire)
                        .collect(),
                    next_from_message_id: value.next_from_message_id,
                    has_more: value.has_more,
                }),
            })
        }
        TelegramProviderQueryResponse::ChatState(value) => {
            Response::ChatState(wire::ChatStateResponse {
                state: value.as_ref().map(chat_state_to_wire),
            })
        }
        TelegramProviderQueryResponse::ChatPositions(values) => {
            Response::ChatPositions(wire::ChatPositionsResponse {
                position: values.iter().map(chat_position_to_wire).collect(),
            })
        }
        TelegramProviderQueryResponse::ChatOperationalState(value) => {
            Response::ChatOperationalState(wire::ChatOperationalStateResponse {
                state: value.as_ref().map(chat_operational_state_to_wire),
            })
        }
        TelegramProviderQueryResponse::TopicMessageIds(values) => {
            Response::TopicMessageIds(wire::TopicMessageIdsResponse {
                provider_message_id: values.clone(),
            })
        }
        TelegramProviderQueryResponse::Reactions(values) => {
            Response::Reactions(wire::ReactionListResponse {
                reaction: values
                    .iter()
                    .map(|value| wire::TelegramReactionObservationProjection {
                        sender_id: value.sender_id.clone(),
                        emoji: value.emoji.clone(),
                        is_outgoing: value.is_outgoing,
                        is_active: value.is_active,
                    })
                    .collect(),
            })
        }
        TelegramProviderQueryResponse::ReactionSummary(values) => {
            Response::ReactionSummary(wire::ReactionSummaryResponse {
                summary: values
                    .iter()
                    .map(|value| wire::TelegramReactionSummaryProjection {
                        emoji: value.emoji.clone(),
                        count: value.count,
                        is_active: value.is_active,
                    })
                    .collect(),
            })
        }
        TelegramProviderQueryResponse::ChatFolders(values) => {
            Response::ChatFolders(wire::ChatFolderListResponse {
                folder: values.iter().map(chat_folder_to_wire).collect(),
            })
        }
        TelegramProviderQueryResponse::CachedMessages(values) => {
            Response::CachedMessages(wire::MessageProjectionListResponse {
                item: values.iter().map(message_projection_to_wire).collect(),
            })
        }
        TelegramProviderQueryResponse::ReplyChain(values) => {
            Response::ReplyChain(wire::MessageProjectionListResponse {
                item: values.iter().map(message_projection_to_wire).collect(),
            })
        }
        TelegramProviderQueryResponse::ForwardChain(values) => {
            Response::ForwardChain(wire::MessageProjectionListResponse {
                item: values.iter().map(message_projection_to_wire).collect(),
            })
        }
        TelegramProviderQueryResponse::MessageVersions(values) => {
            Response::MessageVersions(wire::MessageVersionListResponse {
                item: values
                    .iter()
                    .map(|value| wire::TelegramMessageVersionProjection {
                        version_id: value.version_id.clone(),
                        message_id: value.message_id.clone(),
                        account_id: value.account_id.clone(),
                        provider_chat_id: value.provider_chat_id.clone(),
                        provider_message_id: value.provider_message_id.clone(),
                        version_number: value.version_number,
                        body_text: value.body_text.clone(),
                        observed_at_unix_seconds: value.observed_at_unix_seconds,
                        source: version_source_name(value.source).to_owned(),
                    })
                    .collect(),
            })
        }
        TelegramProviderQueryResponse::MessageTombstones(values) => {
            Response::MessageTombstones(wire::MessageTombstoneListResponse {
                item: values
                    .iter()
                    .map(|value| wire::TelegramMessageTombstoneProjection {
                        tombstone_id: value.tombstone_id.clone(),
                        message_id: value.message_id.clone(),
                        account_id: value.account_id.clone(),
                        provider_chat_id: value.provider_chat_id.clone(),
                        provider_message_id: value.provider_message_id.clone(),
                        reason: tombstone_reason_name(value.reason).to_owned(),
                        observed_at_unix_seconds: value.observed_at_unix_seconds,
                        is_provider_delete: value.is_provider_delete,
                        is_locally_visible: value.is_locally_visible,
                    })
                    .collect(),
            })
        }
        TelegramProviderQueryResponse::MessageMutations(values) => {
            Response::MessageMutations(wire::MessageMutationListResponse {
                item: values
                    .iter()
                    .map(|value| wire::TelegramMessageMutationProjection {
                        mutation: Some(match value {
                            TelegramMessageMutation::Edit {
                                text,
                                observed_at_unix_seconds,
                            } => wire::telegram_message_mutation_projection::Mutation::Edit(
                                wire::EditMutation {
                                    text: text.clone(),
                                    observed_at_unix_seconds: *observed_at_unix_seconds,
                                },
                            ),
                            TelegramMessageMutation::Delete { is_permanent } => {
                                wire::telegram_message_mutation_projection::Mutation::Delete(
                                    wire::DeleteMutation {
                                        is_permanent: *is_permanent,
                                    },
                                )
                            }
                            TelegramMessageMutation::Pin { is_pinned } => {
                                wire::telegram_message_mutation_projection::Mutation::Pin(
                                    wire::PinMutation {
                                        is_pinned: *is_pinned,
                                    },
                                )
                            }
                            TelegramMessageMutation::Reaction { emoji, is_active } => {
                                wire::telegram_message_mutation_projection::Mutation::Reaction(
                                    wire::ReactionMutation {
                                        emoji: emoji.clone(),
                                        is_active: *is_active,
                                    },
                                )
                            }
                        }),
                    })
                    .collect(),
            })
        }
        TelegramProviderQueryResponse::MessageReferences(value) => {
            Response::MessageReferences(wire::MessageReferencesResponse {
                references: value.as_ref().map(message_references_to_wire),
            })
        }
        TelegramProviderQueryResponse::Attachment(value) => {
            Response::Attachment(wire::AttachmentResponse {
                attachment: value
                    .as_ref()
                    .map(|value| wire::TelegramAttachmentProjection {
                        attachment_id: value.attachment_id.clone(),
                        account_id: value.account_id.clone(),
                        provider_chat_id: value.provider_chat_id.clone(),
                        provider_message_id: value.provider_message_id.clone(),
                        provider_file_id: value.provider_file_id.clone(),
                        state: attachment_state_name(value.state).to_owned(),
                        size_bytes: value.size_bytes,
                        filename: value.filename.clone(),
                        content_type: value.content_type.clone(),
                        blob_ref: value.blob_ref.clone(),
                    }),
            })
        }
        TelegramProviderQueryResponse::File(value) => Response::File(wire::FileResponse {
            file: value.as_ref().map(file_to_wire),
        }),
        TelegramProviderQueryResponse::Participants(value) => {
            Response::Participants(wire::ParticipantPageResponse {
                account_id: value.account_id.clone(),
                provider_chat_id: value.provider_chat_id.clone(),
                filter: participant_filter_name(value.filter).to_owned(),
                item: value.items.iter().map(participant_to_wire).collect(),
                next_offset: value.next_offset,
            })
        }
        TelegramProviderQueryResponse::Topics(values) => {
            Response::Topics(wire::TopicListResponse {
                topic: values.iter().map(topic_to_wire).collect(),
            })
        }
        TelegramProviderQueryResponse::Topic(value) => Response::Topic(wire::TopicResponse {
            topic: value.as_ref().map(topic_to_wire),
        }),
        TelegramProviderQueryResponse::Operations(values) => {
            Response::Operations(wire::OperationListResponse {
                operation: values.iter().map(operation_to_wire).collect(),
            })
        }
        TelegramProviderQueryResponse::Commands(values) => {
            Response::Commands(wire::CommandRecordListResponse {
                record: values
                    .iter()
                    .map(|value| {
                        let command = wire::TelegramProviderCommandV1::decode(
                            encode_command(&value.command).as_slice(),
                        )
                        .ok()?;
                        Some(wire::TelegramCommandRecordProjection {
                            operation: Some(operation_to_wire(&value.operation)),
                            command: Some(command),
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
            })
        }
    };
    Some(
        wire::TelegramProviderQueryResponseV1 {
            response: Some(response),
        }
        .encode_to_vec(),
    )
}

pub fn decode_query_response(
    bytes: &[u8],
) -> Result<TelegramProviderQueryResponse, TelegramAuthorizationWireError> {
    use wire::telegram_provider_query_response_v1::Response;
    let message = wire::TelegramProviderQueryResponseV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    match message
        .response
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
    {
        Response::Chats(value) => Ok(TelegramProviderQueryResponse::Chats(
            value
                .chat
                .into_iter()
                .map(parse_chat)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::Chat(value) => Ok(TelegramProviderQueryResponse::Chat(
            value.chat.map(parse_chat).transpose()?,
        )),
        Response::ChatAvatar(value) => Ok(TelegramProviderQueryResponse::ChatAvatar(
            value.avatar.map(parse_avatar),
        )),
        Response::History(value) => Ok(TelegramProviderQueryResponse::History(
            value
                .item
                .into_iter()
                .map(parse_message_observation)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::HistoryPage(value) => {
            let page = value
                .page
                .ok_or(TelegramAuthorizationWireError::InvalidPayload)?;
            Ok(TelegramProviderQueryResponse::HistoryPage(
                TelegramHistoryPage {
                    items: page
                        .item
                        .into_iter()
                        .map(parse_message_observation)
                        .collect::<Result<Vec<_>, _>>()?,
                    next_from_message_id: page.next_from_message_id,
                    has_more: page.has_more,
                },
            ))
        }
        Response::ChatState(value) => Ok(TelegramProviderQueryResponse::ChatState(
            value.state.map(parse_chat_state),
        )),
        Response::ChatPositions(value) => Ok(TelegramProviderQueryResponse::ChatPositions(
            value
                .position
                .into_iter()
                .map(parse_chat_position)
                .collect(),
        )),
        Response::ChatOperationalState(value) => {
            Ok(TelegramProviderQueryResponse::ChatOperationalState(
                value.state.map(parse_chat_operational_state),
            ))
        }
        Response::TopicMessageIds(value) => Ok(TelegramProviderQueryResponse::TopicMessageIds(
            value.provider_message_id,
        )),
        Response::Reactions(value) => Ok(TelegramProviderQueryResponse::Reactions(
            value
                .reaction
                .into_iter()
                .map(|value| TelegramReactionObservation {
                    sender_id: value.sender_id,
                    emoji: value.emoji,
                    is_outgoing: value.is_outgoing,
                    is_active: value.is_active,
                })
                .collect(),
        )),
        Response::ReactionSummary(value) => Ok(TelegramProviderQueryResponse::ReactionSummary(
            value
                .summary
                .into_iter()
                .map(|value| TelegramReactionSummary {
                    emoji: value.emoji,
                    count: value.count,
                    is_active: value.is_active,
                })
                .collect(),
        )),
        Response::ChatFolders(value) => Ok(TelegramProviderQueryResponse::ChatFolders(
            value.folder.into_iter().map(parse_chat_folder).collect(),
        )),
        Response::CachedMessages(value) => Ok(TelegramProviderQueryResponse::CachedMessages(
            value
                .item
                .into_iter()
                .map(parse_message_projection)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::ReplyChain(value) => Ok(TelegramProviderQueryResponse::ReplyChain(
            value
                .item
                .into_iter()
                .map(parse_message_projection)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::ForwardChain(value) => Ok(TelegramProviderQueryResponse::ForwardChain(
            value
                .item
                .into_iter()
                .map(parse_message_projection)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::MessageVersions(value) => Ok(TelegramProviderQueryResponse::MessageVersions(
            value
                .item
                .into_iter()
                .map(|value| {
                    Ok(TelegramMessageVersion {
                        version_id: value.version_id,
                        message_id: value.message_id,
                        account_id: value.account_id,
                        provider_chat_id: value.provider_chat_id,
                        provider_message_id: value.provider_message_id,
                        version_number: value.version_number,
                        body_text: value.body_text,
                        observed_at_unix_seconds: value.observed_at_unix_seconds,
                        source: parse_version_source(&value.source)?,
                    })
                })
                .collect::<Result<Vec<_>, TelegramAuthorizationWireError>>()?,
        )),
        Response::MessageTombstones(value) => Ok(TelegramProviderQueryResponse::MessageTombstones(
            value
                .item
                .into_iter()
                .map(|value| {
                    Ok(TelegramMessageTombstone {
                        tombstone_id: value.tombstone_id,
                        message_id: value.message_id,
                        account_id: value.account_id,
                        provider_chat_id: value.provider_chat_id,
                        provider_message_id: value.provider_message_id,
                        reason: parse_tombstone_reason(&value.reason)?,
                        observed_at_unix_seconds: value.observed_at_unix_seconds,
                        is_provider_delete: value.is_provider_delete,
                        is_locally_visible: value.is_locally_visible,
                    })
                })
                .collect::<Result<Vec<_>, TelegramAuthorizationWireError>>()?,
        )),
        Response::MessageMutations(value) => Ok(TelegramProviderQueryResponse::MessageMutations(
            value
                .item
                .into_iter()
                .map(|value| {
                    match value
                        .mutation
                        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
                    {
                        wire::telegram_message_mutation_projection::Mutation::Edit(value) => {
                            Ok(TelegramMessageMutation::Edit {
                                text: value.text,
                                observed_at_unix_seconds: value.observed_at_unix_seconds,
                            })
                        }
                        wire::telegram_message_mutation_projection::Mutation::Delete(value) => {
                            Ok(TelegramMessageMutation::Delete {
                                is_permanent: value.is_permanent,
                            })
                        }
                        wire::telegram_message_mutation_projection::Mutation::Pin(value) => {
                            Ok(TelegramMessageMutation::Pin {
                                is_pinned: value.is_pinned,
                            })
                        }
                        wire::telegram_message_mutation_projection::Mutation::Reaction(value) => {
                            Ok(TelegramMessageMutation::Reaction {
                                emoji: value.emoji,
                                is_active: value.is_active,
                            })
                        }
                    }
                })
                .collect::<Result<Vec<_>, TelegramAuthorizationWireError>>()?,
        )),
        Response::MessageReferences(value) => Ok(TelegramProviderQueryResponse::MessageReferences(
            value.references.map(|value| {
                let reply_to = value.reply_to.map(|reply| crate::TelegramReplyReference {
                    provider_chat_id: reply.provider_chat_id,
                    provider_message_id: reply.provider_message_id,
                });
                let forward_origin =
                    value
                        .forward_origin
                        .map(|origin| crate::TelegramForwardOrigin {
                            provider_chat_id: origin.provider_chat_id,
                            provider_message_id: origin.provider_message_id,
                            provider_sender_id: origin.provider_sender_id,
                            sender_name: origin.sender_name,
                            observed_at_unix_seconds: origin.observed_at_unix_seconds,
                        });
                crate::TelegramMessageReferences {
                    reply_to,
                    forward_origin,
                }
            }),
        )),
        Response::Attachment(value) => Ok(TelegramProviderQueryResponse::Attachment(
            value
                .attachment
                .map(|value| {
                    Ok(TelegramAttachmentProjection {
                        attachment_id: value.attachment_id,
                        account_id: value.account_id,
                        provider_chat_id: value.provider_chat_id,
                        provider_message_id: value.provider_message_id,
                        provider_file_id: value.provider_file_id,
                        state: parse_attachment_state(&value.state)?,
                        size_bytes: value.size_bytes,
                        filename: value.filename,
                        content_type: value.content_type,
                        blob_ref: value.blob_ref,
                    })
                })
                .transpose()?,
        )),
        Response::File(value) => Ok(TelegramProviderQueryResponse::File(
            value.file.map(parse_file).transpose()?,
        )),
        Response::Participants(value) => Ok(TelegramProviderQueryResponse::Participants(
            TelegramParticipantPage {
                account_id: value.account_id,
                provider_chat_id: value.provider_chat_id,
                filter: parse_participant_filter(&value.filter)?,
                items: value.item.into_iter().map(parse_participant).collect(),
                next_offset: value.next_offset,
            },
        )),
        Response::Topics(value) => Ok(TelegramProviderQueryResponse::Topics(
            value.topic.into_iter().map(parse_topic).collect(),
        )),
        Response::Topic(value) => Ok(TelegramProviderQueryResponse::Topic(
            value.topic.map(parse_topic),
        )),
        Response::Operations(value) => Ok(TelegramProviderQueryResponse::Operations(
            value
                .operation
                .into_iter()
                .map(parse_operation)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::Commands(value) => Ok(TelegramProviderQueryResponse::Commands(
            value
                .record
                .into_iter()
                .map(|value| {
                    let operation = parse_operation(
                        value
                            .operation
                            .ok_or(TelegramAuthorizationWireError::InvalidPayload)?,
                    )?;
                    let command = value
                        .command
                        .ok_or(TelegramAuthorizationWireError::InvalidPayload)?;
                    let command = decode_command(&command.encode_to_vec())?;
                    Ok(TelegramCommandRecord { operation, command })
                })
                .collect::<Result<Vec<_>, TelegramAuthorizationWireError>>()?,
        )),
        Response::Realtime(_) => Err(TelegramAuthorizationWireError::InvalidPayload),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TelegramLifecycleRequest {
    Provision(TelegramAccountSetup),
    Retry {
        operation_id: String,
        now_unix_seconds: u64,
        next_attempt_at_unix_seconds: u64,
    },
    ListAccounts,
    GetAccount {
        account_id: String,
    },
    RetireAccount {
        account_id: String,
    },
    Replay {
        account_id: String,
        after_sequence: u64,
        limit: u32,
    },
}

pub fn encode_realtime_request(account_id: &str, after_sequence: u64, limit: u32) -> Vec<u8> {
    wire::TelegramRealtimeReplayRequestV1 {
        account_id: account_id.to_owned(),
        after_sequence,
        limit,
    }
    .encode_to_vec()
}

pub fn decode_realtime_request(
    bytes: &[u8],
) -> Result<(String, u64, u32), TelegramAuthorizationWireError> {
    let request = wire::TelegramRealtimeReplayRequestV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    Ok((request.account_id, request.after_sequence, request.limit))
}

pub fn encode_realtime_response(page: &TelegramRealtimeReplayPage) -> Vec<u8> {
    wire::TelegramRealtimeReplayResponseV1 {
        frame: page
            .frames
            .iter()
            .map(|value| wire::TelegramRealtimeFrameProjection {
                account_id: value.account_id.clone(),
                sequence: value.sequence,
                provider_cursor: value.provider_cursor.clone(),
                event: Some(event_to_wire(&value.event)),
            })
            .collect(),
        earliest_available_sequence: page.earliest_available_sequence,
        latest_sequence: page.latest_sequence,
        next_after_sequence: page.next_after_sequence,
        reset_required: page.reset_required,
        contract_major: 1,
    }
    .encode_to_vec()
}

pub fn decode_realtime_response(
    bytes: &[u8],
) -> Result<TelegramRealtimeReplayPage, TelegramAuthorizationWireError> {
    let response = wire::TelegramRealtimeReplayResponseV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    if response.contract_major != 1 {
        return Err(TelegramAuthorizationWireError::InvalidPayload);
    }
    let frames = response
        .frame
        .into_iter()
        .map(|value| {
            Ok(TelegramRealtimeFrame {
                account_id: value.account_id,
                sequence: value.sequence,
                provider_cursor: value.provider_cursor,
                event: parse_event(
                    value
                        .event
                        .ok_or(TelegramAuthorizationWireError::InvalidPayload)?,
                )?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TelegramRealtimeReplayPage {
        frames,
        earliest_available_sequence: response.earliest_available_sequence,
        latest_sequence: response.latest_sequence,
        next_after_sequence: response.next_after_sequence,
        reset_required: response.reset_required,
    })
}

pub fn encode_reconfiguration_request(request: &TelegramRuntimeReconfigurationRequest) -> Vec<u8> {
    use wire::telegram_reconfiguration_request_v1::Request;
    let request = match request {
        TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id,
            account_id,
            expected_runtime_epoch,
        } => Request::Begin(wire::BeginTelegramReconfigurationRequest {
            reconfiguration_id: reconfiguration_id.clone(),
            account_id: account_id.clone(),
            expected_runtime_epoch: *expected_runtime_epoch,
        }),
        TelegramRuntimeReconfigurationRequest::Status { reconfiguration_id } => {
            Request::Status(wire::TelegramReconfigurationStatusRequest {
                reconfiguration_id: reconfiguration_id.clone(),
            })
        }
    };
    wire::TelegramReconfigurationRequestV1 {
        request: Some(request),
    }
    .encode_to_vec()
}

pub fn decode_reconfiguration_request(
    bytes: &[u8],
) -> Result<TelegramRuntimeReconfigurationRequest, TelegramAuthorizationWireError> {
    use wire::telegram_reconfiguration_request_v1::Request;
    let request = wire::TelegramReconfigurationRequestV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?
        .request
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?;
    Ok(match request {
        Request::Begin(value) => TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id: value.reconfiguration_id,
            account_id: value.account_id,
            expected_runtime_epoch: value.expected_runtime_epoch,
        },
        Request::Status(value) => TelegramRuntimeReconfigurationRequest::Status {
            reconfiguration_id: value.reconfiguration_id,
        },
    })
}

pub fn encode_reconfiguration_response(
    reconfiguration: &TelegramRuntimeReconfiguration,
) -> Vec<u8> {
    wire::TelegramReconfigurationResponseV1 {
        reconfiguration_id: reconfiguration.reconfiguration_id.clone(),
        account_id: reconfiguration.account_id.clone(),
        expected_runtime_epoch: reconfiguration.expected_runtime_epoch,
        target_runtime_epoch: reconfiguration.target_runtime_epoch,
        state: match reconfiguration.state {
            TelegramRuntimeReconfigurationState::Accepted => "accepted",
            TelegramRuntimeReconfigurationState::Applying => "applying",
            TelegramRuntimeReconfigurationState::Completed => "completed",
            TelegramRuntimeReconfigurationState::Failed => "failed",
        }
        .to_owned(),
        sanitized_reason_code: reconfiguration.sanitized_reason_code.clone(),
        contract_major: 1,
    }
    .encode_to_vec()
}

pub fn decode_reconfiguration_response(
    bytes: &[u8],
) -> Result<TelegramRuntimeReconfiguration, TelegramAuthorizationWireError> {
    let response = wire::TelegramReconfigurationResponseV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    if response.contract_major != 1 {
        return Err(TelegramAuthorizationWireError::InvalidPayload);
    }
    let state = match response.state.as_str() {
        "accepted" => TelegramRuntimeReconfigurationState::Accepted,
        "applying" => TelegramRuntimeReconfigurationState::Applying,
        "completed" => TelegramRuntimeReconfigurationState::Completed,
        "failed" => TelegramRuntimeReconfigurationState::Failed,
        _ => return Err(TelegramAuthorizationWireError::InvalidPayload),
    };
    Ok(TelegramRuntimeReconfiguration {
        reconfiguration_id: response.reconfiguration_id,
        account_id: response.account_id,
        expected_runtime_epoch: response.expected_runtime_epoch,
        target_runtime_epoch: response.target_runtime_epoch,
        state,
        sanitized_reason_code: response.sanitized_reason_code,
    })
}

pub fn encode_lifecycle_request(request: &TelegramLifecycleRequest) -> Vec<u8> {
    use wire::telegram_lifecycle_request_v1::Request;
    let request = match request {
        TelegramLifecycleRequest::Provision(setup) => {
            Request::Provision(wire::ProvisionAccountRequest {
                account_id: setup.account_id.clone(),
                display_name: setup.display_name.clone(),
                external_account_id: setup.external_account_id.clone(),
                credential: setup
                    .credentials
                    .iter()
                    .map(|binding| wire::CredentialBinding {
                        purpose: binding.purpose.as_str().to_owned(),
                        revision: binding.revision,
                    })
                    .collect(),
                qr_authorized: setup.qr_authorized,
            })
        }
        TelegramLifecycleRequest::Retry {
            operation_id,
            now_unix_seconds,
            next_attempt_at_unix_seconds,
        } => Request::Retry(wire::RetryCommandRequest {
            operation_id: operation_id.clone(),
            now_unix_seconds: *now_unix_seconds,
            next_attempt_at_unix_seconds: *next_attempt_at_unix_seconds,
        }),
        TelegramLifecycleRequest::ListAccounts => {
            Request::ListAccounts(wire::ListAccountsRequest {})
        }
        TelegramLifecycleRequest::GetAccount { account_id } => {
            Request::GetAccount(wire::AccountIdRequest {
                account_id: account_id.clone(),
            })
        }
        TelegramLifecycleRequest::RetireAccount { account_id } => {
            Request::RetireAccount(wire::AccountIdRequest {
                account_id: account_id.clone(),
            })
        }
        TelegramLifecycleRequest::Replay {
            account_id,
            after_sequence,
            limit,
        } => Request::Replay(wire::ReplayRequest {
            account_id: account_id.clone(),
            after_sequence: *after_sequence,
            limit: *limit,
        }),
    };
    wire::TelegramLifecycleRequestV1 {
        request: Some(request),
    }
    .encode_to_vec()
}

pub fn decode_lifecycle_request(
    bytes: &[u8],
) -> Result<TelegramLifecycleRequest, TelegramAuthorizationWireError> {
    use wire::telegram_lifecycle_request_v1::Request;
    let message = wire::TelegramLifecycleRequestV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    match message
        .request
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
    {
        Request::Provision(value) => {
            let credentials = value
                .credential
                .into_iter()
                .map(|binding| {
                    let purpose = match binding.purpose.as_str() {
                        "telegram_api_hash" => TelegramCredentialPurpose::ApiHash,
                        "telegram_session_encryption_key" => {
                            TelegramCredentialPurpose::SessionEncryptionKey
                        }
                        _ => return Err(TelegramAuthorizationWireError::InvalidPayload),
                    };
                    Ok(TelegramCredentialBinding {
                        purpose,
                        revision: binding.revision,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let setup = TelegramAccountSetup {
                account_id: value.account_id,
                display_name: value.display_name,
                external_account_id: value.external_account_id,
                credentials,
                qr_authorized: value.qr_authorized,
            };
            crate::validate_setup(&setup)
                .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
            Ok(TelegramLifecycleRequest::Provision(setup))
        }
        Request::Retry(value) => Ok(TelegramLifecycleRequest::Retry {
            operation_id: value.operation_id,
            now_unix_seconds: value.now_unix_seconds,
            next_attempt_at_unix_seconds: value.next_attempt_at_unix_seconds,
        }),
        Request::ListAccounts(_) => Ok(TelegramLifecycleRequest::ListAccounts),
        Request::GetAccount(value) => Ok(TelegramLifecycleRequest::GetAccount {
            account_id: value.account_id,
        }),
        Request::RetireAccount(value) => Ok(TelegramLifecycleRequest::RetireAccount {
            account_id: value.account_id,
        }),
        Request::Replay(value) => Ok(TelegramLifecycleRequest::Replay {
            account_id: value.account_id,
            after_sequence: value.after_sequence,
            limit: value.limit,
        }),
    }
}

fn account_to_wire(account: &TelegramAccount) -> wire::TelegramAccountResponse {
    wire::TelegramAccountResponse {
        account_id: account.account_id.clone(),
        display_name: account.display_name.clone(),
        external_account_id: account.external_account_id.clone(),
        state: match account.state {
            crate::TelegramAccountState::Provisioning => "provisioning",
            crate::TelegramAccountState::Ready => "ready",
            crate::TelegramAccountState::Degraded => "degraded",
            crate::TelegramAccountState::Retired => "retired",
        }
        .to_owned(),
        runtime_state: match account.runtime_state {
            crate::TelegramRuntimeState::Stopped => "stopped",
            crate::TelegramRuntimeState::Starting => "starting",
            crate::TelegramRuntimeState::Running => "running",
            crate::TelegramRuntimeState::Degraded => "degraded",
            crate::TelegramRuntimeState::Blocked => "blocked",
        }
        .to_owned(),
        runtime_epoch: account.runtime_epoch,
    }
}

fn operation_to_wire(operation: &TelegramOperation) -> wire::TelegramOperationResponse {
    wire::TelegramOperationResponse {
        operation_id: operation.operation_id.clone(),
        account_id: operation.account_id.clone(),
        command_kind: operation.command_kind.as_str().to_owned(),
        idempotency_key: operation.idempotency_key.clone(),
        state: match operation.state {
            crate::TelegramOperationState::Accepted => "accepted",
            crate::TelegramOperationState::Running => "running",
            crate::TelegramOperationState::AwaitingProvider => "awaiting_provider",
            crate::TelegramOperationState::Completed => "completed",
            crate::TelegramOperationState::Failed => "failed",
            crate::TelegramOperationState::RetryScheduled => "retry_scheduled",
            crate::TelegramOperationState::DeadLetter => "dead_letter",
        }
        .to_owned(),
        retry_count: operation.retry_count,
        max_retries: operation.max_retries,
        lease_epoch: operation.lease_epoch,
        reconciliation: match operation.reconciliation {
            crate::TelegramReconciliationState::NotObserved => "not_observed",
            crate::TelegramReconciliationState::AwaitingProvider => "awaiting_provider",
            crate::TelegramReconciliationState::Observed => "observed",
            crate::TelegramReconciliationState::Mismatch => "mismatch",
        }
        .to_owned(),
        last_error: operation.last_error.clone(),
        next_attempt_at_unix_seconds: operation.next_attempt_at_unix_seconds,
        locked_at_unix_seconds: operation.locked_at_unix_seconds,
        locked_by: operation.locked_by.clone(),
        provider_observed_at_unix_seconds: operation.provider_observed_at_unix_seconds,
        reconciled_at_unix_seconds: operation.reconciled_at_unix_seconds,
    }
}

pub fn encode_lifecycle_response(response: &TelegramClientResponse) -> Option<Vec<u8>> {
    use wire::telegram_lifecycle_response_v1::Response;
    let response = match response {
        TelegramClientResponse::Account(account) => Response::Account(account_to_wire(account)),
        TelegramClientResponse::Accounts(accounts) => {
            Response::Accounts(wire::TelegramAccountList {
                account: accounts.iter().map(account_to_wire).collect(),
            })
        }
        TelegramClientResponse::Accepted { operation_id } => {
            Response::Accepted(wire::AcceptedResponse {
                operation_id: operation_id.clone(),
            })
        }
        TelegramClientResponse::Operation(operation) => {
            Response::Operation(operation_to_wire(operation))
        }
        _ => return None,
    };
    Some(
        wire::TelegramLifecycleResponseV1 {
            response: Some(response),
        }
        .encode_to_vec(),
    )
}

pub fn encode_command_response(operation: &TelegramOperation) -> Vec<u8> {
    operation_to_wire(operation).encode_to_vec()
}

pub fn decode_command_response(
    bytes: &[u8],
) -> Result<TelegramOperation, TelegramAuthorizationWireError> {
    let response = wire::TelegramOperationResponse::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    parse_operation(response)
}

fn parse_account(
    value: wire::TelegramAccountResponse,
) -> Result<TelegramAccount, TelegramAuthorizationWireError> {
    let state = match value.state.as_str() {
        "provisioning" => crate::TelegramAccountState::Provisioning,
        "ready" => crate::TelegramAccountState::Ready,
        "degraded" => crate::TelegramAccountState::Degraded,
        "retired" => crate::TelegramAccountState::Retired,
        _ => return Err(TelegramAuthorizationWireError::InvalidPayload),
    };
    let runtime_state = match value.runtime_state.as_str() {
        "stopped" => crate::TelegramRuntimeState::Stopped,
        "starting" => crate::TelegramRuntimeState::Starting,
        "running" => crate::TelegramRuntimeState::Running,
        "degraded" => crate::TelegramRuntimeState::Degraded,
        "blocked" => crate::TelegramRuntimeState::Blocked,
        _ => return Err(TelegramAuthorizationWireError::InvalidPayload),
    };
    Ok(TelegramAccount {
        account_id: value.account_id,
        display_name: value.display_name,
        external_account_id: value.external_account_id,
        state,
        runtime_state,
        runtime_epoch: value.runtime_epoch,
    })
}

fn parse_operation(
    value: wire::TelegramOperationResponse,
) -> Result<TelegramOperation, TelegramAuthorizationWireError> {
    let command_kind = match value.command_kind.as_str() {
        "send_text" => crate::TelegramCommandKind::SendText,
        "send_media" => crate::TelegramCommandKind::SendMedia,
        "download_file" => crate::TelegramCommandKind::DownloadFile,
        "reply" => crate::TelegramCommandKind::Reply,
        "forward" => crate::TelegramCommandKind::Forward,
        "edit" => crate::TelegramCommandKind::Edit,
        "delete" => crate::TelegramCommandKind::Delete,
        "restore_visibility" => crate::TelegramCommandKind::RestoreVisibility,
        "reaction" => crate::TelegramCommandKind::Reaction,
        "pin" => crate::TelegramCommandKind::Pin,
        "mark_unread" => crate::TelegramCommandKind::MarkUnread,
        "archive" => crate::TelegramCommandKind::Archive,
        "mute" => crate::TelegramCommandKind::Mute,
        "join" => crate::TelegramCommandKind::Join,
        "leave" => crate::TelegramCommandKind::Leave,
        "folder_add" => crate::TelegramCommandKind::AddChatToFolder,
        "folder_remove" => crate::TelegramCommandKind::RemoveChatFromFolder,
        "folder_reassign" => crate::TelegramCommandKind::ReassignChatFolders,
        "search_messages" => crate::TelegramCommandKind::SearchMessages,
        "list_participants" => crate::TelegramCommandKind::ListParticipants,
        "list_topics" => crate::TelegramCommandKind::ListTopics,
        "create_topic" => crate::TelegramCommandKind::CreateTopic,
        "set_topic_closed" => crate::TelegramCommandKind::SetTopicClosed,
        _ => return Err(TelegramAuthorizationWireError::InvalidPayload),
    };
    let state = match value.state.as_str() {
        "accepted" => crate::TelegramOperationState::Accepted,
        "running" => crate::TelegramOperationState::Running,
        "awaiting_provider" => crate::TelegramOperationState::AwaitingProvider,
        "completed" => crate::TelegramOperationState::Completed,
        "failed" => crate::TelegramOperationState::Failed,
        "retry_scheduled" => crate::TelegramOperationState::RetryScheduled,
        "dead_letter" => crate::TelegramOperationState::DeadLetter,
        _ => return Err(TelegramAuthorizationWireError::InvalidPayload),
    };
    let reconciliation = match value.reconciliation.as_str() {
        "not_observed" => crate::TelegramReconciliationState::NotObserved,
        "awaiting_provider" => crate::TelegramReconciliationState::AwaitingProvider,
        "observed" => crate::TelegramReconciliationState::Observed,
        "mismatch" => crate::TelegramReconciliationState::Mismatch,
        _ => return Err(TelegramAuthorizationWireError::InvalidPayload),
    };
    Ok(TelegramOperation {
        operation_id: value.operation_id,
        account_id: value.account_id,
        command_kind,
        idempotency_key: value.idempotency_key,
        state,
        retry_count: value.retry_count,
        max_retries: value.max_retries,
        lease_epoch: value.lease_epoch,
        reconciliation,
        last_error: value.last_error,
        next_attempt_at_unix_seconds: value.next_attempt_at_unix_seconds,
        locked_at_unix_seconds: value.locked_at_unix_seconds,
        locked_by: value.locked_by,
        provider_observed_at_unix_seconds: value.provider_observed_at_unix_seconds,
        reconciled_at_unix_seconds: value.reconciled_at_unix_seconds,
    })
}

pub fn decode_lifecycle_response(
    bytes: &[u8],
) -> Result<TelegramClientResponse, TelegramAuthorizationWireError> {
    use wire::telegram_lifecycle_response_v1::Response;
    let message = wire::TelegramLifecycleResponseV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    match message
        .response
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
    {
        Response::Account(value) => Ok(TelegramClientResponse::Account(parse_account(value)?)),
        Response::Accounts(value) => Ok(TelegramClientResponse::Accounts(
            value
                .account
                .into_iter()
                .map(parse_account)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Response::Accepted(value) => Ok(TelegramClientResponse::Accepted {
            operation_id: value.operation_id,
        }),
        Response::Operation(value) => {
            Ok(TelegramClientResponse::Operation(parse_operation(value)?))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TelegramAuthorizationRequest {
    Status,
    SubmitPassword(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TelegramAuthorizationResponse {
    Status(TelegramAuthorizationStatus),
    PasswordAccepted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramAuthorizationWireError {
    InvalidPayload,
    MissingVariant,
}

pub fn encode_request(request: &TelegramAuthorizationRequest) -> Vec<u8> {
    let request = match request {
        TelegramAuthorizationRequest::Status => {
            Request::AuthorizationStatus(wire::AuthorizationStatusRequest {})
        }
        TelegramAuthorizationRequest::SubmitPassword(password) => {
            Request::SubmitPassword(wire::SubmitAuthorizationPasswordRequest {
                password: password.clone(),
            })
        }
    };
    wire::TelegramAuthorizationRequestV1 {
        request: Some(request),
    }
    .encode_to_vec()
}

pub fn decode_request(
    bytes: &[u8],
) -> Result<TelegramAuthorizationRequest, TelegramAuthorizationWireError> {
    let message = wire::TelegramAuthorizationRequestV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    match message
        .request
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
    {
        Request::AuthorizationStatus(_) => Ok(TelegramAuthorizationRequest::Status),
        Request::SubmitPassword(value) => {
            Ok(TelegramAuthorizationRequest::SubmitPassword(value.password))
        }
    }
}

pub fn encode_response(response: &TelegramAuthorizationResponse) -> Vec<u8> {
    let response = match response {
        TelegramAuthorizationResponse::Status(status) => {
            Response::AuthorizationStatus(wire::AuthorizationStatusResponse {
                state: status.state.clone(),
                qr_link: status.qr_link.clone(),
                password_hint: status.password_hint.clone(),
            })
        }
        TelegramAuthorizationResponse::PasswordAccepted => {
            Response::PasswordAccepted(wire::AuthorizationPasswordAcceptedResponse {})
        }
    };
    wire::TelegramAuthorizationResponseV1 {
        response: Some(response),
    }
    .encode_to_vec()
}

pub fn decode_response(
    bytes: &[u8],
) -> Result<TelegramAuthorizationResponse, TelegramAuthorizationWireError> {
    let message = wire::TelegramAuthorizationResponseV1::decode(bytes)
        .map_err(|_| TelegramAuthorizationWireError::InvalidPayload)?;
    match message
        .response
        .ok_or(TelegramAuthorizationWireError::MissingVariant)?
    {
        Response::AuthorizationStatus(value) => Ok(TelegramAuthorizationResponse::Status(
            TelegramAuthorizationStatus {
                state: value.state,
                qr_link: value.qr_link,
                password_hint: value.password_hint,
            },
        )),
        Response::PasswordAccepted(_) => Ok(TelegramAuthorizationResponse::PasswordAccepted),
    }
}
