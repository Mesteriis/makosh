//! Runtime-local transient Telegram projection cache.

use std::collections::{HashMap, HashSet};

use makosh_telegram_api::{
    TelegramAccount, TelegramAccountId, TelegramAttachmentProjection, TelegramChat,
    TelegramChatAvatar, TelegramChatFolder, TelegramChatOperationalState, TelegramChatPosition,
    TelegramChatStateProjection, TelegramCommandRecord, TelegramCredentialBinding,
    TelegramDeliveryState, TelegramFileSnapshot, TelegramMessageMutation,
    TelegramMessageProjection, TelegramMessageReferences, TelegramMessageTombstone,
    TelegramMessageVersion, TelegramOperation, TelegramOperationId, TelegramOperationState,
    TelegramParticipant, TelegramProviderCommand, TelegramQrLoginSession,
    TelegramReactionObservation, TelegramReactionSummary, TelegramRealtimeFrame,
    TelegramReconciliationState, TelegramRuntimeLease, TelegramRuntimeLeaseState, TelegramSetupId,
    TelegramTopic, provider_command_chat_id, provider_command_kind, provider_command_message_id,
};

#[derive(Default)]
pub(crate) struct TelegramRuntimeProjectionCache {
    accounts: HashMap<TelegramAccountId, TelegramAccount>,
    chats: HashMap<String, TelegramChat>,
    chat_avatars: HashMap<String, TelegramChatAvatar>,
    chat_folders: HashMap<String, TelegramChatFolder>,
    chat_positions: HashMap<String, TelegramChatPosition>,
    chat_operational_states: HashMap<String, TelegramChatOperationalState>,
    topics: HashMap<String, TelegramTopic>,
    messages: HashMap<String, TelegramMessageProjection>,
    message_mutations: HashMap<String, Vec<TelegramMessageMutation>>,
    chat_states: HashMap<String, TelegramChatStateProjection>,
    credentials: HashMap<TelegramAccountId, Vec<TelegramCredentialBinding>>,
    operations: HashMap<TelegramOperationId, TelegramOperation>,
    commands: HashMap<TelegramOperationId, TelegramProviderCommand>,
    qr_sessions: HashMap<TelegramSetupId, TelegramQrLoginSession>,
    files: HashMap<String, TelegramFileSnapshot>,
    participants: HashMap<String, Vec<TelegramParticipant>>,
    realtime_frames: HashMap<TelegramAccountId, Vec<TelegramRealtimeFrame>>,
    runtime_leases: HashMap<TelegramAccountId, TelegramRuntimeLease>,
    message_versions: HashMap<String, Vec<TelegramMessageVersion>>,
    message_tombstones: HashMap<String, Vec<TelegramMessageTombstone>>,
    attachments: HashMap<String, TelegramAttachmentProjection>,
    reactions: HashMap<String, Vec<TelegramReactionObservation>>,
}

impl TelegramRuntimeProjectionCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put_account(&mut self, account: TelegramAccount) {
        self.accounts.insert(account.account_id.clone(), account);
    }

    pub fn account(&self, account_id: &str) -> Option<&TelegramAccount> {
        self.accounts.get(account_id)
    }

    pub fn accounts(&self) -> Vec<TelegramAccount> {
        let mut accounts = self.accounts.values().cloned().collect::<Vec<_>>();
        accounts.sort_by(|left, right| left.account_id.cmp(&right.account_id));
        accounts
    }

    pub fn put_credentials(&mut self, account_id: &str, bindings: Vec<TelegramCredentialBinding>) {
        self.credentials.insert(account_id.to_owned(), bindings);
    }

    pub fn put_chat(&mut self, chat: TelegramChat) {
        let key = format!("{}:{}", chat.account_id, chat.provider_chat_id);
        self.chats.insert(key, chat);
    }

    pub fn chat(&self, account_id: &str, provider_chat_id: &str) -> Option<&TelegramChat> {
        self.chats.get(&format!("{account_id}:{provider_chat_id}"))
    }

    pub fn put_chat_avatar(&mut self, avatar: TelegramChatAvatar) {
        self.chat_avatars.insert(
            format!("{}:{}", avatar.account_id, avatar.provider_chat_id),
            avatar,
        );
    }

    pub fn chat_avatar(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Option<&TelegramChatAvatar> {
        self.chat_avatars
            .get(&format!("{account_id}:{provider_chat_id}"))
    }

    pub fn put_chat_folders(&mut self, folders: Vec<TelegramChatFolder>) {
        for folder in folders {
            self.chat_folders.insert(
                format!("{}:{}", folder.account_id, folder.provider_folder_id),
                folder,
            );
        }
    }

    pub fn chat_folders(&self, account_id: &str) -> Vec<TelegramChatFolder> {
        let mut folders = self
            .chat_folders
            .values()
            .filter(|folder| folder.account_id == account_id)
            .cloned()
            .collect::<Vec<_>>();
        folders.sort_by_key(|folder| folder.provider_folder_id);
        folders
    }

    pub fn put_chat_position(&mut self, position: TelegramChatPosition) {
        let key = format!(
            "{}:{}:{}:{}",
            position.account_id,
            position.provider_chat_id,
            position.list_kind,
            position.provider_folder_id.unwrap_or_default()
        );
        if position.order <= 0 {
            self.chat_positions.remove(&key);
        } else {
            self.chat_positions.insert(key, position);
        }
    }

    pub fn chat_positions(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Vec<TelegramChatPosition> {
        self.chat_positions
            .values()
            .filter(|position| {
                position.account_id == account_id && position.provider_chat_id == provider_chat_id
            })
            .cloned()
            .collect()
    }

    pub fn put_chat_operational_state(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        state: TelegramChatOperationalState,
    ) {
        self.chat_operational_states
            .insert(format!("{account_id}:{provider_chat_id}"), state);
    }

    pub fn chat_operational_state(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Option<&TelegramChatOperationalState> {
        self.chat_operational_states
            .get(&format!("{account_id}:{provider_chat_id}"))
    }

    pub fn chats_for_account(&self, account_id: &str, limit: u32) -> Vec<TelegramChat> {
        let mut chats = self
            .chats
            .values()
            .filter(|chat| chat.account_id == account_id)
            .cloned()
            .collect::<Vec<_>>();
        chats.sort_by(|left, right| left.provider_chat_id.cmp(&right.provider_chat_id));
        chats.truncate(limit as usize);
        chats
    }

    pub fn put_message(&mut self, message: TelegramMessageProjection) {
        self.messages.insert(message.message_id.clone(), message);
    }

    pub fn update_sender_display_name(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        provider_sender_id: &str,
        display_name: &str,
    ) -> usize {
        let mut updated = 0;
        for message in self.messages.values_mut().filter(|message| {
            message.account_id == account_id
                && message.provider_chat_id == provider_chat_id
                && message.sender_id == provider_sender_id
                && message.sender_display_name.as_deref().is_none_or(|name| {
                    matches!(
                        name.trim(),
                        "" | "Telegram user" | "Telegram chat" | "Telegram participant"
                    )
                })
        }) {
            message.sender_display_name = Some(display_name.to_owned());
            updated += 1;
        }
        updated
    }

    pub fn apply_message_text_edit(&mut self, message_id: &str, text: &str) -> bool {
        let Some(message) = self.messages.get_mut(message_id) else {
            return false;
        };
        message.text = Some(text.to_owned());
        true
    }

    pub fn put_topic(&mut self, topic: TelegramTopic) {
        self.topics.insert(
            format!(
                "{}:{}:{}",
                topic.account_id, topic.provider_chat_id, topic.provider_topic_id
            ),
            topic,
        );
    }

    pub fn topics_for_chat(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        limit: u32,
    ) -> Vec<TelegramTopic> {
        let mut topics = self
            .topics
            .values()
            .filter(|topic| {
                topic.account_id == account_id && topic.provider_chat_id == provider_chat_id
            })
            .cloned()
            .collect::<Vec<_>>();
        topics.sort_by(|left, right| {
            right
                .is_pinned
                .cmp(&left.is_pinned)
                .then_with(|| {
                    right
                        .last_message_at_unix_seconds
                        .cmp(&left.last_message_at_unix_seconds)
                })
                .then_with(|| left.provider_topic_id.cmp(&right.provider_topic_id))
        });
        topics.truncate(limit as usize);
        topics
    }

    pub fn search_topics(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        query: &str,
        limit: u32,
    ) -> Vec<TelegramTopic> {
        let query = query.to_lowercase();
        self.topics_for_chat(account_id, provider_chat_id, u32::MAX)
            .into_iter()
            .filter(|topic| topic.title.to_lowercase().contains(&query))
            .take(limit as usize)
            .collect()
    }

    pub fn message(&self, message_id: &str) -> Option<&TelegramMessageProjection> {
        self.messages.get(message_id)
    }

    pub fn message_references(&self, message_id: &str) -> Option<TelegramMessageReferences> {
        self.messages
            .get(message_id)
            .map(|message| message.references.clone())
    }

    pub fn reply_chain(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
        limit: u32,
    ) -> Vec<TelegramMessageProjection> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut current = self
            .messages
            .values()
            .find(|message| {
                message.account_id == account_id
                    && message.provider_chat_id == provider_chat_id
                    && message.provider_message_id == provider_message_id
            })
            .cloned();
        while let Some(message) = current {
            let key = (
                message.provider_chat_id.clone(),
                message.provider_message_id.clone(),
            );
            if !visited.insert(key) || chain.len() >= 128 {
                break;
            }
            let next = message.references.reply_to.clone();
            chain.push(message);
            if chain.len() >= limit as usize {
                break;
            }
            current = next.and_then(|reference| {
                self.messages
                    .values()
                    .find(|candidate| {
                        candidate.account_id == account_id
                            && candidate.provider_chat_id == reference.provider_chat_id
                            && candidate.provider_message_id == reference.provider_message_id
                    })
                    .cloned()
            });
        }
        chain
    }

    pub fn forward_chain(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
        limit: u32,
    ) -> Vec<TelegramMessageProjection> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut current = self
            .messages
            .values()
            .find(|message| {
                message.account_id == account_id
                    && message.provider_chat_id == provider_chat_id
                    && message.provider_message_id == provider_message_id
            })
            .cloned();
        while let Some(message) = current {
            let key = (
                message.provider_chat_id.clone(),
                message.provider_message_id.clone(),
            );
            if !visited.insert(key) || chain.len() >= 128 {
                break;
            }
            let next = message
                .references
                .forward_origin
                .as_ref()
                .and_then(|origin| {
                    Some((
                        origin.provider_chat_id.as_ref()?.clone(),
                        origin.provider_message_id.as_ref()?.clone(),
                    ))
                });
            chain.push(message);
            if chain.len() >= limit as usize {
                break;
            }
            current = next.and_then(|(chat_id, message_id)| {
                self.messages
                    .values()
                    .find(|candidate| {
                        candidate.account_id == account_id
                            && candidate.provider_chat_id == chat_id
                            && candidate.provider_message_id == message_id
                    })
                    .cloned()
            });
        }
        chain
    }

    pub fn messages_by_ids(&self, message_ids: &[String]) -> Vec<TelegramMessageProjection> {
        message_ids
            .iter()
            .filter_map(|message_id| self.messages.get(message_id).cloned())
            .collect()
    }

    pub fn recent_messages(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        limit: u32,
    ) -> Vec<TelegramMessageProjection> {
        let mut messages = self
            .messages
            .values()
            .filter(|message| {
                message.account_id == account_id
                    && provider_chat_id.is_none_or(|chat_id| message.provider_chat_id == chat_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        messages.sort_by(|left, right| {
            right
                .observed_at_unix_seconds
                .cmp(&left.observed_at_unix_seconds)
                .then_with(|| right.provider_message_id.cmp(&left.provider_message_id))
        });
        messages.truncate(limit as usize);
        messages
    }

    pub fn pinned_messages(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        limit: u32,
    ) -> Vec<TelegramMessageProjection> {
        self.recent_messages(account_id, Some(provider_chat_id), u32::MAX)
            .into_iter()
            .filter(|message| {
                self.message_mutations
                    .get(&message.message_id)
                    .and_then(|mutations| mutations.last())
                    .is_some_and(|mutation| {
                        matches!(mutation, TelegramMessageMutation::Pin { is_pinned: true })
                    })
            })
            .take(limit as usize)
            .collect()
    }

    pub fn messages_for_chat(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        limit: u32,
    ) -> Vec<TelegramMessageProjection> {
        let mut messages = self
            .messages
            .values()
            .filter(|message| {
                message.account_id == account_id && message.provider_chat_id == provider_chat_id
            })
            .cloned()
            .collect::<Vec<_>>();
        messages.sort_by(|left, right| {
            right
                .observed_at_unix_seconds
                .cmp(&left.observed_at_unix_seconds)
                .then_with(|| right.provider_message_id.cmp(&left.provider_message_id))
        });
        messages.truncate(limit as usize);
        messages
    }

    pub fn message_ids_for_topic(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_topic_id: &str,
        limit: u32,
    ) -> Vec<String> {
        let mut message_ids = self
            .messages
            .values()
            .filter(|message| {
                message.account_id == account_id
                    && message.provider_chat_id == provider_chat_id
                    && message.provider_topic_id.as_deref() == Some(provider_topic_id)
            })
            .map(|message| message.provider_message_id.clone())
            .collect::<Vec<_>>();
        message_ids.sort();
        message_ids.truncate(limit as usize);
        message_ids
    }

    pub fn search_messages(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        query: &str,
        limit: u32,
    ) -> Vec<TelegramMessageProjection> {
        let query = query.to_lowercase();
        let mut messages = self
            .messages
            .values()
            .filter(|message| {
                message.account_id == account_id
                    && provider_chat_id.is_none_or(|chat_id| message.provider_chat_id == chat_id)
                    && message
                        .text
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>();
        messages.sort_by(|left, right| {
            right
                .observed_at_unix_seconds
                .cmp(&left.observed_at_unix_seconds)
                .then_with(|| right.provider_message_id.cmp(&left.provider_message_id))
        });
        messages.truncate(limit as usize);
        messages
    }

    pub fn search_chats(&self, account_id: &str, query: &str, limit: u32) -> Vec<TelegramChat> {
        let query = query.to_lowercase();
        let mut chats = self
            .chats
            .values()
            .filter(|chat| {
                chat.account_id == account_id
                    && (chat.title.to_lowercase().contains(&query)
                        || chat
                            .username
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&query))
            })
            .cloned()
            .collect::<Vec<_>>();
        chats.sort_by(|left, right| left.provider_chat_id.cmp(&right.provider_chat_id));
        chats.truncate(limit as usize);
        chats
    }

    pub fn reconcile_delivery(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        old_provider_message_id: &str,
        provider_message_id: Option<&str>,
        state: TelegramDeliveryState,
    ) -> bool {
        let Some((key, _)) = self.messages.iter().find(|(_, message)| {
            message.account_id == account_id
                && message.provider_chat_id == provider_chat_id
                && message.provider_message_id == old_provider_message_id
        }) else {
            return false;
        };
        let key = key.clone();
        let Some(message) = self.messages.get_mut(&key) else {
            return false;
        };
        if let Some(provider_message_id) = provider_message_id {
            message.provider_message_id = provider_message_id.to_owned();
        }
        message.delivery_state = state;
        true
    }

    pub fn apply_message_mutation(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
        mutation: TelegramMessageMutation,
    ) {
        let key = format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}");
        self.message_mutations
            .entry(key)
            .or_default()
            .push(mutation);
    }

    pub fn message_mutations(&self, message_id: &str) -> Option<&[TelegramMessageMutation]> {
        self.message_mutations.get(message_id).map(Vec::as_slice)
    }

    pub fn append_message_version(&mut self, version: TelegramMessageVersion) -> bool {
        let versions = self
            .message_versions
            .entry(version.message_id.clone())
            .or_default();
        if versions.iter().any(|existing| {
            existing.version_number == version.version_number
                && existing.body_text == version.body_text
                && existing.observed_at_unix_seconds == version.observed_at_unix_seconds
        }) {
            return false;
        }
        versions.push(version);
        true
    }

    pub fn next_message_version_number(&self, message_id: &str) -> u32 {
        self.message_versions
            .get(message_id)
            .and_then(|versions| versions.iter().map(|version| version.version_number).max())
            .unwrap_or(0)
            .saturating_add(1)
    }

    pub fn message_versions(&self, message_id: &str) -> Option<&[TelegramMessageVersion]> {
        self.message_versions.get(message_id).map(Vec::as_slice)
    }

    pub fn append_message_tombstone(&mut self, tombstone: TelegramMessageTombstone) -> bool {
        let tombstones = self
            .message_tombstones
            .entry(tombstone.message_id.clone())
            .or_default();
        if tombstones.iter().any(|existing| {
            existing.reason == tombstone.reason
                && existing.is_provider_delete == tombstone.is_provider_delete
                && !existing.is_locally_visible
        }) {
            return false;
        }
        tombstones.push(tombstone);
        true
    }

    pub fn message_tombstones(&self, message_id: &str) -> Option<&[TelegramMessageTombstone]> {
        self.message_tombstones.get(message_id).map(Vec::as_slice)
    }

    pub fn put_attachment(&mut self, attachment: TelegramAttachmentProjection) {
        self.attachments
            .insert(attachment.attachment_id.clone(), attachment);
    }

    pub fn attachment(&self, attachment_id: &str) -> Option<&TelegramAttachmentProjection> {
        self.attachments.get(attachment_id)
    }

    pub fn attachment_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Option<TelegramAttachmentProjection> {
        self.attachments
            .values()
            .find(|attachment| {
                attachment.account_id == account_id
                    && attachment.provider_chat_id == provider_chat_id
                    && attachment.provider_message_id == provider_message_id
            })
            .cloned()
    }

    pub fn replace_reactions(
        &mut self,
        message_id: &str,
        reactions: Vec<TelegramReactionObservation>,
    ) {
        self.reactions.insert(message_id.to_owned(), reactions);
    }

    pub fn reactions(&self, message_id: &str) -> Option<&[TelegramReactionObservation]> {
        self.reactions.get(message_id).map(Vec::as_slice)
    }

    pub fn reaction_summary(&self, message_id: &str) -> Vec<TelegramReactionSummary> {
        let mut summary = HashMap::<String, TelegramReactionSummary>::new();
        for reaction in self.reactions(message_id).unwrap_or_default() {
            let entry = summary
                .entry(reaction.emoji.clone())
                .or_insert(TelegramReactionSummary {
                    emoji: reaction.emoji.clone(),
                    count: 0,
                    is_active: false,
                });
            if reaction.is_active {
                entry.count = entry.count.saturating_add(1);
            }
            entry.is_active |= reaction.is_active && reaction.is_outgoing;
        }
        let mut values = summary.into_values().collect::<Vec<_>>();
        values.sort_by(|left, right| left.emoji.cmp(&right.emoji));
        values
    }

    pub fn apply_file_to_attachments(
        &mut self,
        account_id: &str,
        file: &TelegramFileSnapshot,
    ) -> usize {
        let state = if file.is_downloaded {
            makosh_telegram_api::TelegramAttachmentDownloadState::Downloaded
        } else if file.is_downloading {
            makosh_telegram_api::TelegramAttachmentDownloadState::Downloading
        } else {
            makosh_telegram_api::TelegramAttachmentDownloadState::Pending
        };
        let mut updated = 0;
        for attachment in self.attachments.values_mut().filter(|attachment| {
            attachment.account_id == account_id
                && attachment.provider_file_id == file.provider_file_id
        }) {
            attachment.state = state;
            attachment.size_bytes = file.size_bytes.or(file.downloaded_size_bytes);
            attachment.blob_ref = file.blob_reference_id.as_ref().and_then(|reference_id| {
                (reference_id.len() == 16).then(|| {
                    format!(
                        "blob-content:{}",
                        reference_id
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>()
                    )
                })
            });
            updated += 1;
        }
        updated
    }

    pub fn apply_chat_state(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        state: TelegramChatStateProjection,
    ) {
        self.chat_states
            .insert(format!("{account_id}:{provider_chat_id}"), state);
    }

    pub fn chat_state(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Option<&TelegramChatStateProjection> {
        self.chat_states
            .get(&format!("{account_id}:{provider_chat_id}"))
    }

    pub fn credentials(&self, account_id: &str) -> Option<&[TelegramCredentialBinding]> {
        self.credentials.get(account_id).map(Vec::as_slice)
    }

    pub fn put_runtime_lease(&mut self, lease: TelegramRuntimeLease) {
        self.runtime_leases.insert(lease.account_id.clone(), lease);
    }

    pub fn runtime_lease(&self, account_id: &str) -> Option<&TelegramRuntimeLease> {
        self.runtime_leases.get(account_id)
    }

    pub fn revoke_runtime_lease(&mut self, account_id: &str) -> bool {
        let Some(lease) = self.runtime_leases.get_mut(account_id) else {
            return false;
        };
        lease.state = TelegramRuntimeLeaseState::Revoked;
        true
    }

    pub fn put_operation(&mut self, operation: TelegramOperation) {
        self.operations
            .insert(operation.operation_id.clone(), operation);
    }

    pub fn put_operation_if_absent(&mut self, operation: TelegramOperation) -> bool {
        if self.operations.contains_key(&operation.operation_id) {
            return false;
        }
        self.put_operation(operation);
        true
    }

    pub fn put_command(&mut self, command: TelegramProviderCommand) {
        let operation_id = makosh_telegram_api::provider_command_operation_id(&command).to_owned();
        self.commands.insert(operation_id, command);
    }

    pub fn command(&self, operation_id: &str) -> Option<&TelegramProviderCommand> {
        self.commands.get(operation_id)
    }

    pub fn operation(&self, operation_id: &str) -> Option<&TelegramOperation> {
        self.operations.get(operation_id)
    }

    pub fn operation_ids_for_account(&self, account_id: &str) -> Vec<String> {
        self.operations
            .values()
            .filter(|operation| operation.account_id == account_id)
            .map(|operation| operation.operation_id.clone())
            .collect()
    }

    pub fn operations_for_account(&self, account_id: &str) -> Vec<TelegramOperation> {
        self.operations
            .values()
            .filter(|operation| operation.account_id == account_id)
            .cloned()
            .collect()
    }

    pub fn command_records_for_account(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        provider_message_id: Option<&str>,
        command_kinds: &[String],
        limit: u32,
    ) -> Vec<TelegramCommandRecord> {
        let mut records = self
            .operations
            .values()
            .filter_map(|operation| {
                if operation.account_id != account_id {
                    return None;
                }
                let command = self.commands.get(&operation.operation_id)?;
                if provider_chat_id
                    .is_some_and(|value| provider_command_chat_id(command) != Some(value))
                    || provider_message_id
                        .is_some_and(|value| provider_command_message_id(command) != Some(value))
                    || (!command_kinds.is_empty()
                        && !command_kinds
                            .iter()
                            .any(|kind| provider_command_kind(command).as_str() == kind))
                {
                    return None;
                }
                Some(TelegramCommandRecord {
                    operation: operation.clone(),
                    command: command.clone(),
                })
            })
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.operation
                .operation_id
                .cmp(&right.operation.operation_id)
        });
        records.truncate(limit as usize);
        records
    }

    pub fn schedule_operation_retry(
        &mut self,
        operation_id: &str,
        now_unix_seconds: u64,
        next_attempt_at_unix_seconds: u64,
        error: impl Into<String>,
    ) -> bool {
        let Some(operation) = self.operations.get_mut(operation_id) else {
            return false;
        };
        if operation.retry_count >= operation.max_retries {
            operation.state = TelegramOperationState::DeadLetter;
            operation.next_attempt_at_unix_seconds = None;
        } else {
            operation.state = TelegramOperationState::RetryScheduled;
            operation.next_attempt_at_unix_seconds = Some(next_attempt_at_unix_seconds);
        }
        operation.locked_at_unix_seconds = None;
        operation.locked_by = None;
        operation.last_error = Some(error.into());
        operation.reconciled_at_unix_seconds = Some(now_unix_seconds);
        true
    }

    pub fn reconcile_operation(&mut self, operation_id: &str, observed: bool) -> bool {
        let Some(operation) = self.operations.get_mut(operation_id) else {
            return false;
        };
        operation.reconciliation = if observed {
            TelegramReconciliationState::Observed
        } else {
            TelegramReconciliationState::Mismatch
        };
        operation.state = if observed {
            TelegramOperationState::Completed
        } else {
            TelegramOperationState::Failed
        };
        operation.locked_at_unix_seconds = None;
        operation.locked_by = None;
        operation.next_attempt_at_unix_seconds = None;
        true
    }

    pub fn put_qr_session(&mut self, session: TelegramQrLoginSession) {
        self.qr_sessions.insert(session.setup_id.clone(), session);
    }

    pub fn qr_session(&self, setup_id: &str) -> Option<&TelegramQrLoginSession> {
        self.qr_sessions.get(setup_id)
    }

    pub fn put_file(&mut self, file: TelegramFileSnapshot) {
        self.files.insert(
            format!("{}:{}", file.account_id, file.provider_file_id),
            file,
        );
    }

    pub fn file(&self, account_id: &str, provider_file_id: &str) -> Option<&TelegramFileSnapshot> {
        self.files.get(&format!("{account_id}:{provider_file_id}"))
    }

    pub fn put_participants(&mut self, page: &makosh_telegram_api::TelegramParticipantPage) {
        self.participants.insert(
            format!(
                "{}:{}:{:?}",
                page.account_id, page.provider_chat_id, page.filter
            ),
            page.items.clone(),
        );
    }

    pub fn upsert_participant(&mut self, participant: TelegramParticipant) {
        let key = format!(
            "{}:{}:{:?}",
            participant.account_id,
            participant.provider_chat_id,
            makosh_telegram_api::TelegramParticipantFilter::Recent
        );
        let items = self.participants.entry(key).or_default();
        if let Some(existing) = items
            .iter_mut()
            .find(|existing| existing.provider_member_id == participant.provider_member_id)
        {
            *existing = participant;
        } else {
            items.push(participant);
        }
    }

    pub fn append_realtime_frame(&mut self, frame: TelegramRealtimeFrame) {
        self.realtime_frames
            .entry(frame.account_id.clone())
            .or_default()
            .push(frame);
    }

    pub fn next_realtime_sequence(&self, account_id: &str) -> u64 {
        self.realtime_frames
            .get(account_id)
            .and_then(|frames| frames.last())
            .map_or(1, |frame| frame.sequence.saturating_add(1))
    }

    pub fn realtime_after(&self, account_id: &str, sequence: u64) -> Vec<TelegramRealtimeFrame> {
        self.realtime_frames
            .get(account_id)
            .into_iter()
            .flat_map(|frames| frames.iter())
            .filter(|frame| frame.sequence > sequence)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn realtime_frames_are_replayable_after_provider_sequence() {
        let mut persistence = TelegramRuntimeProjectionCache::new();
        let event = makosh_telegram_api::TelegramProviderEvent::ChatMarkedUnreadChanged {
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            is_marked_as_unread: true,
        };
        persistence.append_realtime_frame(TelegramRealtimeFrame {
            account_id: "account".to_owned(),
            sequence: 1,
            provider_cursor: Some("cursor-1".to_owned()),
            event: event.clone(),
        });
        persistence.append_realtime_frame(TelegramRealtimeFrame {
            account_id: "account".to_owned(),
            sequence: 2,
            provider_cursor: Some("cursor-2".to_owned()),
            event,
        });
        let replay = persistence.realtime_after("account", 1);
        assert_eq!(replay.len(), 1);
        assert_eq!(replay[0].sequence, 2);
        assert_eq!(replay[0].provider_cursor.as_deref(), Some("cursor-2"));
    }

    #[test]
    fn zero_order_chat_position_removes_provider_membership() {
        let mut persistence = TelegramRuntimeProjectionCache::new();
        let mut position = TelegramChatPosition {
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            list_kind: "folder".to_owned(),
            provider_folder_id: Some(7),
            order: 10,
            is_pinned: false,
        };
        persistence.put_chat_position(position.clone());
        assert_eq!(persistence.chat_positions("account", "100").len(), 1);

        position.order = 0;
        persistence.put_chat_position(position);
        assert!(persistence.chat_positions("account", "100").is_empty());
    }

    #[test]
    fn file_progress_updates_matching_attachment_state() {
        let mut persistence = TelegramRuntimeProjectionCache::new();
        persistence.put_attachment(TelegramAttachmentProjection {
            attachment_id: "attachment-1".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            provider_message_id: "200".to_owned(),
            provider_file_id: "300".to_owned(),
            state: makosh_telegram_api::TelegramAttachmentDownloadState::Pending,
            size_bytes: None,
            filename: Some("report.pdf".to_owned()),
            content_type: None,
            blob_ref: None,
        });
        let updated = persistence.apply_file_to_attachments(
            "account",
            &TelegramFileSnapshot {
                account_id: "account".to_owned(),
                provider_file_id: "300".to_owned(),
                provider_unique_id: None,
                media_kind: Some(makosh_telegram_api::TelegramMediaKind::Document),
                size_bytes: Some(42),
                expected_size_bytes: Some(42),
                downloaded_size_bytes: Some(42),
                is_downloading: false,
                is_downloaded: true,
                blob_reference_id: None,
                blob_plaintext_sha256: None,
                blob_backup_class: None,
            },
        );
        assert_eq!(updated, 1);
        assert_eq!(
            persistence
                .attachment("attachment-1")
                .map(|attachment| attachment.state),
            Some(makosh_telegram_api::TelegramAttachmentDownloadState::Downloaded)
        );
        assert_eq!(
            persistence
                .attachment("attachment-1")
                .and_then(|attachment| attachment.size_bytes),
            Some(42)
        );
    }

    #[test]
    fn reactions_are_replaced_as_provider_message_projection() {
        let mut persistence = TelegramRuntimeProjectionCache::new();
        persistence.replace_reactions(
            "telegram:account:chat:message",
            vec![makosh_telegram_api::TelegramReactionObservation {
                sender_id: "user-1".to_owned(),
                emoji: ":thumbsup:".to_owned(),
                is_outgoing: false,
                is_active: true,
            }],
        );

        let reactions = persistence
            .reactions("telegram:account:chat:message")
            .expect("provider reaction projection");
        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions[0].sender_id, "user-1");
        assert_eq!(reactions[0].emoji, ":thumbsup:");
    }

    #[test]
    fn resolved_sender_name_updates_only_matching_generic_cached_messages() {
        let mut persistence = TelegramRuntimeProjectionCache::new();
        for (message_id, chat_id, sender_id, sender_name) in [
            ("one", "chat", "42", Some("Telegram user")),
            ("two", "chat", "42", None),
            ("three", "other-chat", "42", Some("Telegram user")),
            ("four", "chat", "7", Some("Known sender")),
        ] {
            persistence.put_message(TelegramMessageProjection {
                message_id: message_id.to_owned(),
                account_id: "account".to_owned(),
                provider_chat_id: chat_id.to_owned(),
                provider_message_id: message_id.to_owned(),
                provider_topic_id: None,
                sender_id: sender_id.to_owned(),
                sender_display_name: sender_name.map(ToOwned::to_owned),
                sender_source_identity: None,
                text: None,
                media: None,
                references: TelegramMessageReferences::default(),
                observed_at_unix_seconds: 0,
                delivery_state: TelegramDeliveryState::Received,
            });
        }

        assert_eq!(
            persistence.update_sender_display_name("account", "chat", "42", "Resolved sender"),
            2
        );
        assert_eq!(
            persistence
                .message("one")
                .and_then(|message| message.sender_display_name.as_deref()),
            Some("Resolved sender")
        );
        assert_eq!(
            persistence
                .message("three")
                .and_then(|message| message.sender_display_name.as_deref()),
            Some("Telegram user")
        );
        assert_eq!(
            persistence
                .message("four")
                .and_then(|message| message.sender_display_name.as_deref()),
            Some("Known sender")
        );
    }
}
