//! Telegram integration policy and provider-neutral orchestration.

use makosh_communications_ingress::{
    AttachmentDescriptorV1, AttachmentDispositionV1, BodyAvailabilityV1, CommunicationDirectionV1,
    CommunicationEvidenceKindV1, ProviderProvenanceV1, SourceEnvelope, SourceScopeEnvelope,
    new_scoped_communication_observation_draft, with_attachment_descriptor,
    with_participant_display_label,
};
use makosh_telegram_api::{
    TelegramAccount, TelegramAccountState, TelegramAttachmentProjection,
    TelegramChatStateProjection, TelegramContractError, TelegramCredentialBinding,
    TelegramDeliveryState, TelegramMessageMutation, TelegramMessageObservation,
    TelegramMessageProjection, TelegramOperation, TelegramOperationState, TelegramProviderCommand,
    TelegramProviderEvent, TelegramQrLoginSession, TelegramQrLoginState,
    TelegramReconciliationState, TelegramRuntimeReconfiguration,
    TelegramRuntimeReconfigurationState, TelegramRuntimeState, provider_command_account_id,
    provider_command_kind, provider_command_operation_id, validate_text,
};
use makosh_vault_protocol::{DEFAULT_LEASE_TTL_SECONDS, SecretClassV1, VaultActionV1};
use sha2::{Digest, Sha256};

pub use makosh_vault_protocol::VaultPurposeRequestV1;
pub use makosh_vault_protocol::{CredentialLeaseV1 as TelegramCredentialLeaseV1, LeaseAudienceV1};

pub const PACKAGE: &str = "makosh-telegram-core";

pub const TELEGRAM_API_HASH_PURPOSE_ID: &str = "telegram_api_hash";
pub const TELEGRAM_SESSION_STORE_KEY_PURPOSE_ID: &str = "telegram_session_store_key";

pub fn credential_lease_purpose(
    configuration_instance_id: &str,
    binding: &TelegramCredentialBinding,
) -> Result<VaultPurposeRequestV1, TelegramContractError> {
    credential_lease_purpose_for_purpose(configuration_instance_id, binding.purpose)
}

pub fn credential_lease_purpose_for_purpose(
    configuration_instance_id: &str,
    purpose: makosh_telegram_api::TelegramCredentialPurpose,
) -> Result<VaultPurposeRequestV1, TelegramContractError> {
    let (purpose_id, secret_class) = match purpose {
        makosh_telegram_api::TelegramCredentialPurpose::ApiHash => (
            TELEGRAM_API_HASH_PURPOSE_ID,
            SecretClassV1::ProviderCredential,
        ),
        makosh_telegram_api::TelegramCredentialPurpose::SessionEncryptionKey => (
            TELEGRAM_SESSION_STORE_KEY_PURPOSE_ID,
            SecretClassV1::SessionStoreKey,
        ),
    };
    VaultPurposeRequestV1::new(
        purpose_id.to_owned(),
        configuration_instance_id.to_owned(),
        vec![secret_class],
        vec![VaultActionV1::Resolve],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| TelegramContractError::InvalidTransition)
}

pub fn credential_lease_purposes(
    configuration_instance_id: &str,
    bindings: &[TelegramCredentialBinding],
) -> Result<Vec<VaultPurposeRequestV1>, TelegramContractError> {
    bindings
        .iter()
        .map(|binding| credential_lease_purpose(configuration_instance_id, binding))
        .collect()
}

pub fn qr_login_preparing(
    setup_id: impl Into<String>,
    account_id: impl Into<String>,
    expires_at_unix_seconds: u64,
) -> TelegramQrLoginSession {
    TelegramQrLoginSession {
        setup_id: setup_id.into(),
        account_id: account_id.into(),
        state: TelegramQrLoginState::Preparing,
        qr_link: None,
        password_attempts: 0,
        expires_at_unix_seconds,
    }
}

pub fn qr_login_qr_issued(
    session: &TelegramQrLoginSession,
    qr_link: impl Into<String>,
) -> Result<TelegramQrLoginSession, TelegramContractError> {
    if session.state != TelegramQrLoginState::Preparing {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramQrLoginSession {
        state: TelegramQrLoginState::WaitingQrScan,
        qr_link: Some(qr_link.into()),
        ..session.clone()
    })
}

pub fn qr_login_password_required(
    session: &TelegramQrLoginSession,
) -> Result<TelegramQrLoginSession, TelegramContractError> {
    if session.state != TelegramQrLoginState::WaitingQrScan {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramQrLoginSession {
        state: TelegramQrLoginState::WaitingPassword,
        ..session.clone()
    })
}

pub fn qr_login_password_submitted(
    session: &TelegramQrLoginSession,
) -> Result<TelegramQrLoginSession, TelegramContractError> {
    if session.state != TelegramQrLoginState::WaitingPassword || session.password_attempts >= 5 {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramQrLoginSession {
        password_attempts: session.password_attempts + 1,
        ..session.clone()
    })
}

pub fn qr_login_ready(
    session: &TelegramQrLoginSession,
) -> Result<TelegramQrLoginSession, TelegramContractError> {
    if !matches!(
        session.state,
        TelegramQrLoginState::WaitingQrScan | TelegramQrLoginState::WaitingPassword
    ) {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramQrLoginSession {
        state: TelegramQrLoginState::Ready,
        ..session.clone()
    })
}

pub use makosh_communications_ingress::CommunicationObservationDraft;

#[derive(Default)]
pub struct TelegramLifecycle;

pub fn accept_runtime_reconfiguration(
    account: &TelegramAccount,
    reconfiguration_id: &str,
    expected_runtime_epoch: u64,
) -> Result<TelegramRuntimeReconfiguration, TelegramContractError> {
    if reconfiguration_id.trim().is_empty() {
        return Err(TelegramContractError::EmptyField);
    }
    if account.runtime_epoch != expected_runtime_epoch
        || expected_runtime_epoch == 0
        || !matches!(
            account.runtime_state,
            TelegramRuntimeState::Running | TelegramRuntimeState::Degraded
        )
    {
        return Err(TelegramContractError::RuntimeEpochConflict);
    }
    let target_runtime_epoch = expected_runtime_epoch
        .checked_add(1)
        .ok_or(TelegramContractError::RuntimeEpochConflict)?;
    Ok(TelegramRuntimeReconfiguration {
        reconfiguration_id: reconfiguration_id.to_owned(),
        account_id: account.account_id.clone(),
        expected_runtime_epoch,
        target_runtime_epoch,
        state: TelegramRuntimeReconfigurationState::Accepted,
        sanitized_reason_code: None,
    })
}

pub fn apply_runtime_reconfiguration(
    reconfiguration: &TelegramRuntimeReconfiguration,
) -> Result<TelegramRuntimeReconfiguration, TelegramContractError> {
    if reconfiguration.state != TelegramRuntimeReconfigurationState::Accepted {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramRuntimeReconfiguration {
        state: TelegramRuntimeReconfigurationState::Applying,
        ..reconfiguration.clone()
    })
}

pub fn complete_runtime_reconfiguration(
    account: &TelegramAccount,
    reconfiguration: &TelegramRuntimeReconfiguration,
) -> Result<(TelegramAccount, TelegramRuntimeReconfiguration), TelegramContractError> {
    if !matches!(
        reconfiguration.state,
        TelegramRuntimeReconfigurationState::Accepted
            | TelegramRuntimeReconfigurationState::Applying
    ) || account.account_id != reconfiguration.account_id
        || account.runtime_epoch != reconfiguration.expected_runtime_epoch
    {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok((
        TelegramAccount {
            runtime_state: TelegramRuntimeState::Running,
            runtime_epoch: reconfiguration.target_runtime_epoch,
            ..account.clone()
        },
        TelegramRuntimeReconfiguration {
            state: TelegramRuntimeReconfigurationState::Completed,
            sanitized_reason_code: None,
            ..reconfiguration.clone()
        },
    ))
}

pub fn fail_runtime_reconfiguration(
    reconfiguration: &TelegramRuntimeReconfiguration,
    sanitized_reason_code: &str,
) -> Result<TelegramRuntimeReconfiguration, TelegramContractError> {
    if reconfiguration.state.is_terminal()
        || sanitized_reason_code.trim().is_empty()
        || sanitized_reason_code.len() > 64
        || !sanitized_reason_code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
    {
        return Err(TelegramContractError::InvalidTransition);
    }
    Ok(TelegramRuntimeReconfiguration {
        state: TelegramRuntimeReconfigurationState::Failed,
        sanitized_reason_code: Some(sanitized_reason_code.to_owned()),
        ..reconfiguration.clone()
    })
}

impl TelegramLifecycle {
    pub fn mark_running(&self, account: &TelegramAccount) -> TelegramAccount {
        TelegramAccount {
            state: TelegramAccountState::Ready,
            runtime_state: TelegramRuntimeState::Running,
            runtime_epoch: account.runtime_epoch.max(1),
            ..account.clone()
        }
    }

    pub fn retire(
        &self,
        account: &TelegramAccount,
    ) -> Result<TelegramAccount, TelegramContractError> {
        if account.state == TelegramAccountState::Retired {
            return Err(TelegramContractError::AccountRetired);
        }
        Ok(TelegramAccount {
            state: TelegramAccountState::Retired,
            runtime_state: TelegramRuntimeState::Stopped,
            ..account.clone()
        })
    }
}

pub fn accept_operation(command: &TelegramProviderCommand, lease_epoch: u64) -> TelegramOperation {
    let operation_id = provider_command_operation_id(command).to_owned();
    let command_kind = provider_command_kind(command);
    TelegramOperation {
        operation_id: operation_id.clone(),
        account_id: provider_command_account_id(command).to_owned(),
        command_kind,
        idempotency_key: format!("telegram:{}:{}", command_kind.as_str(), operation_id),
        state: TelegramOperationState::Accepted,
        retry_count: 0,
        max_retries: 3,
        lease_epoch,
        reconciliation: TelegramReconciliationState::NotObserved,
        last_error: None,
        next_attempt_at_unix_seconds: None,
        locked_at_unix_seconds: None,
        locked_by: None,
        provider_observed_at_unix_seconds: None,
        reconciled_at_unix_seconds: None,
    }
}

pub fn operation_running(operation: &TelegramOperation) -> TelegramOperation {
    TelegramOperation {
        state: TelegramOperationState::Running,
        ..operation.clone()
    }
}

pub fn operation_awaiting_provider(operation: &TelegramOperation) -> TelegramOperation {
    TelegramOperation {
        state: TelegramOperationState::AwaitingProvider,
        reconciliation: TelegramReconciliationState::AwaitingProvider,
        ..operation.clone()
    }
}

pub fn operation_completed(operation: &TelegramOperation) -> TelegramOperation {
    TelegramOperation {
        state: TelegramOperationState::Completed,
        reconciliation: TelegramReconciliationState::Observed,
        ..operation.clone()
    }
}

pub fn operation_failed(
    operation: &TelegramOperation,
    error: impl Into<String>,
) -> TelegramOperation {
    TelegramOperation {
        state: TelegramOperationState::Failed,
        last_error: Some(error.into()),
        ..operation.clone()
    }
}

pub fn operation_retry_scheduled(
    operation: &TelegramOperation,
    next_attempt_at_unix_seconds: u64,
    error: impl Into<String>,
) -> TelegramOperation {
    let state = if operation.retry_count >= operation.max_retries {
        TelegramOperationState::DeadLetter
    } else {
        TelegramOperationState::RetryScheduled
    };
    TelegramOperation {
        state,
        next_attempt_at_unix_seconds: (state == TelegramOperationState::RetryScheduled)
            .then_some(next_attempt_at_unix_seconds),
        locked_at_unix_seconds: None,
        locked_by: None,
        last_error: Some(error.into()),
        ..operation.clone()
    }
}

pub fn observation_draft(
    observation: TelegramMessageObservation,
) -> Result<CommunicationObservationDraft, TelegramContractError> {
    let source_id = format!(
        "telegram:{}:{}:{}",
        observation.account_id, observation.provider_chat_id, observation.provider_message_id
    );
    let text_preview = observation.text.and_then(|text| {
        let trimmed = text.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    });
    if let Some(text) = &text_preview {
        validate_text(text)?;
    }
    let draft = new_scoped_communication_observation_draft(
        source_id.clone(),
        SourceEnvelope {
            provider: ProviderProvenanceV1::Telegram,
            external_record_id: source_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: observation.account_id.to_string(),
                external_conversation_id: Some(observation.provider_chat_id.clone()),
                external_participant_id: Some(observation.sender_id.clone()),
                external_media_id: observation
                    .media
                    .as_ref()
                    .and_then(|media| media.provider_file_id.clone()),
                external_reply_to_record_id: observation.references.reply_to.as_ref().map(
                    |reference| {
                        telegram_record_id(
                            &observation.account_id,
                            &reference.provider_chat_id,
                            &reference.provider_message_id,
                        )
                    },
                ),
                external_forward_origin_record_id: observation
                    .references
                    .forward_origin
                    .as_ref()
                    .and_then(|origin| {
                        match (&origin.provider_chat_id, &origin.provider_message_id) {
                            (Some(chat_id), Some(message_id)) => Some(telegram_record_id(
                                &observation.account_id,
                                chat_id,
                                message_id,
                            )),
                            _ => None,
                        }
                    }),
            }),
        },
        CommunicationEvidenceKindV1::ChatMessage,
        if text_preview.is_some() {
            BodyAvailabilityV1::Unavailable
        } else {
            BodyAvailabilityV1::MetadataOnly
        },
        if observation.is_outgoing {
            CommunicationDirectionV1::Outgoing
        } else {
            CommunicationDirectionV1::Incoming
        },
        Some(observation.observed_at_unix_seconds),
    )
    .map_err(|_| TelegramContractError::InvalidTransition)?;
    match with_participant_display_label(draft.clone(), observation.sender_display_name) {
        Ok(draft) => Ok(draft),
        Err(_) => Ok(draft),
    }
}

pub fn attachment_observation_draft(
    attachment: &TelegramAttachmentProjection,
) -> Result<Option<CommunicationObservationDraft>, TelegramContractError> {
    let (Some(media_type), Some(declared_bytes)) =
        (&attachment.content_type, attachment.size_bytes)
    else {
        return Ok(None);
    };
    let source_id = format!(
        "telegram:{}:{}:{}:{}",
        attachment.account_id,
        attachment.provider_chat_id,
        attachment.provider_message_id,
        attachment.provider_file_id,
    );
    let draft = new_scoped_communication_observation_draft(
        &source_id,
        SourceEnvelope {
            provider: ProviderProvenanceV1::Telegram,
            external_record_id: source_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: attachment.account_id.clone(),
                external_conversation_id: Some(attachment.provider_chat_id.clone()),
                external_participant_id: None,
                external_media_id: Some(attachment.provider_file_id.clone()),
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::MediaChanged,
        BodyAvailabilityV1::MetadataOnly,
        CommunicationDirectionV1::Unknown,
        None,
    )
    .map_err(|_| TelegramContractError::InvalidTransition)?;
    match with_attachment_descriptor(
        draft.clone(),
        AttachmentDescriptorV1 {
            filename: attachment.filename.clone(),
            media_type: media_type.clone(),
            declared_bytes,
            sha256: None,
            disposition: AttachmentDispositionV1::Unknown,
        },
    ) {
        Ok(draft) => Ok(Some(draft)),
        Err(_) => Ok(Some(draft)),
    }
}

pub fn project_message(
    observation: &TelegramMessageObservation,
) -> Result<TelegramMessageProjection, TelegramContractError> {
    if observation.account_id.trim().is_empty()
        || observation.provider_chat_id.trim().is_empty()
        || observation.provider_message_id.trim().is_empty()
        || observation.sender_id.trim().is_empty()
    {
        return Err(TelegramContractError::EmptyField);
    }
    Ok(TelegramMessageProjection {
        message_id: format!(
            "telegram:{}:{}:{}",
            observation.account_id, observation.provider_chat_id, observation.provider_message_id
        ),
        account_id: observation.account_id.clone(),
        provider_chat_id: observation.provider_chat_id.clone(),
        provider_message_id: observation.provider_message_id.clone(),
        provider_topic_id: observation.provider_topic_id.clone(),
        sender_id: observation.sender_id.clone(),
        sender_display_name: observation.sender_display_name.clone(),
        sender_source_identity: observation.sender_source_identity,
        text: observation.text.clone(),
        media: observation.media.clone(),
        references: observation.references.clone(),
        observed_at_unix_seconds: observation.observed_at_unix_seconds,
        delivery_state: TelegramDeliveryState::Received,
    })
}

pub fn event_message_mutation(
    event: &TelegramProviderEvent,
) -> Option<(&str, &str, &str, TelegramMessageMutation)> {
    match event {
        TelegramProviderEvent::MessageEdited {
            account_id,
            provider_chat_id,
            provider_message_id,
            text,
            observed_at_unix_seconds,
        } => Some((
            account_id,
            provider_chat_id,
            provider_message_id,
            TelegramMessageMutation::Edit {
                text: text.clone(),
                observed_at_unix_seconds: *observed_at_unix_seconds,
            },
        )),
        TelegramProviderEvent::MessageDeleted {
            account_id,
            provider_chat_id,
            provider_message_id,
            is_permanent,
        } => Some((
            account_id,
            provider_chat_id,
            provider_message_id,
            TelegramMessageMutation::Delete {
                is_permanent: *is_permanent,
            },
        )),
        TelegramProviderEvent::MessagePinned {
            account_id,
            provider_chat_id,
            provider_message_id,
            is_pinned,
        } => Some((
            account_id,
            provider_chat_id,
            provider_message_id,
            TelegramMessageMutation::Pin {
                is_pinned: *is_pinned,
            },
        )),
        TelegramProviderEvent::ReactionChanged {
            account_id,
            provider_chat_id,
            provider_message_id,
            emoji,
            is_active,
        } => Some((
            account_id,
            provider_chat_id,
            provider_message_id,
            TelegramMessageMutation::Reaction {
                emoji: emoji.clone(),
                is_active: *is_active,
            },
        )),
        _ => None,
    }
}

pub fn event_chat_state(
    event: &TelegramProviderEvent,
) -> Option<(&str, &str, TelegramChatStateProjection)> {
    match event {
        TelegramProviderEvent::ChatUnreadChanged {
            account_id,
            provider_chat_id,
            unread_count,
            unread_mention_count,
            last_read_inbox_message_id,
        } => Some((
            account_id,
            provider_chat_id,
            TelegramChatStateProjection {
                unread_count: *unread_count,
                unread_mention_count: *unread_mention_count,
                last_read_inbox_message_id: last_read_inbox_message_id.clone(),
                is_marked_as_unread: false,
            },
        )),
        TelegramProviderEvent::ChatMarkedUnreadChanged {
            account_id,
            provider_chat_id,
            is_marked_as_unread,
        } => Some((
            account_id,
            provider_chat_id,
            TelegramChatStateProjection {
                is_marked_as_unread: *is_marked_as_unread,
                ..TelegramChatStateProjection::default()
            },
        )),
        _ => None,
    }
}

pub fn provider_event_draft(
    event: &TelegramProviderEvent,
) -> Result<Option<CommunicationObservationDraft>, TelegramContractError> {
    let event_identity = provider_event_identity(event)?;
    match event {
        TelegramProviderEvent::MessageCreated(observation) => {
            observation_draft(observation.clone()).map(Some)
        }
        TelegramProviderEvent::MessageEdited { text, .. } => event_draft(
            &event_identity,
            event,
            text.clone(),
            CommunicationEvidenceKindV1::MessageEdited,
        ),
        TelegramProviderEvent::MessageDeleted { .. } => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::MessageDeleted,
        ),
        TelegramProviderEvent::MessagePinned { .. } => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ConversationStateChanged,
        ),
        TelegramProviderEvent::ReactionChanged { .. } => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ReactionChanged,
        ),
        TelegramProviderEvent::ReactionsObserved { .. } => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ReactionChanged,
        ),
        TelegramProviderEvent::MessageSendFailed { .. }
        | TelegramProviderEvent::MessageSendSucceeded { .. } => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::DeliveryStateChanged,
        ),
        TelegramProviderEvent::ChatUnreadChanged { .. }
        | TelegramProviderEvent::ChatMarkedUnreadChanged { .. } => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ConversationStateChanged,
        ),
        TelegramProviderEvent::TypingChanged(_) => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::TypingChanged,
        ),
        TelegramProviderEvent::TopicChanged(topic) => event_draft(
            &event_identity,
            event,
            Some(topic.title.clone()),
            CommunicationEvidenceKindV1::TopicChanged,
        ),
        TelegramProviderEvent::ChatPositionChanged(_) => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ConversationStateChanged,
        ),
        TelegramProviderEvent::ChatFoldersChanged { .. } => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ConversationStateChanged,
        ),
        TelegramProviderEvent::ChatNotificationChanged { .. } => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ConversationStateChanged,
        ),
        TelegramProviderEvent::ChatAvatarChanged(_) => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ConversationStateChanged,
        ),
        TelegramProviderEvent::ParticipantChanged(_) => event_draft(
            &event_identity,
            event,
            None,
            CommunicationEvidenceKindV1::ParticipantChanged,
        ),
        TelegramProviderEvent::FileChanged(_) => Ok(None),
    }
}

fn provider_event_identity(event: &TelegramProviderEvent) -> Result<String, TelegramContractError> {
    let canonical_event =
        serde_json::to_vec(event).map_err(|_| TelegramContractError::InvalidTransition)?;
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.telegram.provider-event.v1\0");
    hasher.update(canonical_event);
    Ok(format!("telegram:event:{}", hex_digest(&hasher.finalize())))
}

fn hex_digest(value: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

#[cfg(test)]
mod provider_event_identity_tests {
    use super::*;

    #[test]
    fn provider_event_identity_deduplicates_replay_and_preserves_distinct_edits() {
        let replayed_event = TelegramProviderEvent::MessageEdited {
            account_id: "account".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: "message".to_owned(),
            text: Some("first body".to_owned()),
            observed_at_unix_seconds: 1_782_504_000,
        };
        let later_edit = TelegramProviderEvent::MessageEdited {
            account_id: "account".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: "message".to_owned(),
            text: Some("second body".to_owned()),
            observed_at_unix_seconds: 1_782_504_000,
        };

        assert_eq!(
            provider_event_identity(&replayed_event).expect("valid replay identity"),
            provider_event_identity(&replayed_event).expect("valid replay identity"),
        );
        assert_ne!(
            provider_event_identity(&replayed_event).expect("valid replay identity"),
            provider_event_identity(&later_edit).expect("valid later edit identity"),
        );
    }

    #[test]
    fn message_mutation_evidence_keeps_account_and_conversation_scope() {
        let event = TelegramProviderEvent::MessageEdited {
            account_id: "account".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: "message".to_owned(),
            text: None,
            observed_at_unix_seconds: 1_782_504_000,
        };

        let draft = provider_event_draft(&event)
            .expect("valid provider event")
            .expect("neutral evidence draft");
        assert_eq!(
            draft.source.external_record_id,
            "telegram:account:chat:message"
        );
        let scope = draft.source.scope.expect("provider event scope");
        assert_eq!(scope.external_account_id, "account");
        assert_eq!(scope.external_conversation_id.as_deref(), Some("chat"));
    }
}

fn event_draft(
    source_id: &str,
    event: &TelegramProviderEvent,
    text: Option<String>,
    kind: CommunicationEvidenceKindV1,
) -> Result<Option<CommunicationObservationDraft>, TelegramContractError> {
    if let Some(value) = &text {
        validate_text(value)?;
    }
    let body = if text.is_some() {
        BodyAvailabilityV1::Unavailable
    } else {
        BodyAvailabilityV1::MetadataOnly
    };
    new_scoped_communication_observation_draft(
        source_id,
        SourceEnvelope {
            provider: ProviderProvenanceV1::Telegram,
            external_record_id: provider_event_message_record_id(event)
                .unwrap_or_else(|| source_id.to_owned()),
            scope: Some(provider_event_scope(event)),
        },
        kind,
        body,
        CommunicationDirectionV1::Unknown,
        None,
    )
    .map(Some)
    .map_err(|_| TelegramContractError::InvalidTransition)
}

fn provider_event_message_record_id(event: &TelegramProviderEvent) -> Option<String> {
    let (account_id, provider_chat_id, provider_message_id) = match event {
        TelegramProviderEvent::MessageEdited {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        }
        | TelegramProviderEvent::MessageDeleted {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        }
        | TelegramProviderEvent::MessagePinned {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        }
        | TelegramProviderEvent::ReactionChanged {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        }
        | TelegramProviderEvent::ReactionsObserved {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        }
        | TelegramProviderEvent::MessageSendSucceeded {
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        } => (account_id, provider_chat_id, provider_message_id),
        TelegramProviderEvent::MessageSendFailed {
            account_id,
            provider_chat_id,
            old_provider_message_id,
            ..
        } => (account_id, provider_chat_id, old_provider_message_id),
        _ => return None,
    };
    Some(telegram_record_id(
        account_id,
        provider_chat_id,
        provider_message_id,
    ))
}

fn provider_event_scope(event: &TelegramProviderEvent) -> SourceScopeEnvelope {
    let account_id = makosh_telegram_api::provider_event_account_id(event).to_owned();
    let (external_conversation_id, external_participant_id) = match event {
        TelegramProviderEvent::MessageCreated(observation) => (
            Some(observation.provider_chat_id.clone()),
            Some(observation.sender_id.clone()),
        ),
        TelegramProviderEvent::MessageEdited {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::MessageDeleted {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::MessageSendFailed {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::MessageSendSucceeded {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::MessagePinned {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::ReactionChanged {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::ReactionsObserved {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::ChatUnreadChanged {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::ChatMarkedUnreadChanged {
            provider_chat_id, ..
        }
        | TelegramProviderEvent::ChatNotificationChanged {
            provider_chat_id, ..
        } => (Some(provider_chat_id.clone()), None),
        TelegramProviderEvent::TypingChanged(state) => (
            Some(state.provider_chat_id.clone()),
            Some(state.sender_id.clone()),
        ),
        TelegramProviderEvent::TopicChanged(topic) => (Some(topic.provider_chat_id.clone()), None),
        TelegramProviderEvent::ChatPositionChanged(position) => {
            (Some(position.provider_chat_id.clone()), None)
        }
        TelegramProviderEvent::ChatAvatarChanged(avatar) => {
            (Some(avatar.provider_chat_id.clone()), None)
        }
        TelegramProviderEvent::ParticipantChanged(participant) => (
            Some(participant.provider_chat_id.clone()),
            Some(participant.provider_member_id.clone()),
        ),
        TelegramProviderEvent::ChatFoldersChanged { .. }
        | TelegramProviderEvent::FileChanged(_) => (None, None),
    };
    SourceScopeEnvelope {
        external_account_id: account_id,
        external_conversation_id,
        external_participant_id,
        external_media_id: None,
        external_reply_to_record_id: None,
        external_forward_origin_record_id: None,
    }
}

#[cfg(test)]
mod attachment_observation_tests {
    use super::*;

    fn attachment() -> TelegramAttachmentProjection {
        TelegramAttachmentProjection {
            attachment_id: "telegram:account:chat:message:file".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: "message".to_owned(),
            provider_file_id: "file".to_owned(),
            state: makosh_telegram_api::TelegramAttachmentDownloadState::Pending,
            size_bytes: Some(5),
            filename: Some("report.pdf".to_owned()),
            content_type: Some("application/pdf".to_owned()),
            blob_ref: None,
        }
    }

    #[test]
    fn attachment_metadata_becomes_scoped_media_observation() {
        let draft = attachment_observation_draft(&attachment())
            .expect("valid attachment")
            .expect("complete metadata must publish");

        assert_eq!(draft.kind, CommunicationEvidenceKindV1::MediaChanged);
        assert_eq!(
            draft
                .source
                .scope
                .as_ref()
                .and_then(|scope| scope.external_media_id.as_deref()),
            Some("file")
        );
        assert_eq!(
            draft
                .attachment_descriptor
                .as_ref()
                .map(|value| value.media_type.as_str()),
            Some("application/pdf")
        );
    }

    #[test]
    fn incomplete_attachment_metadata_is_not_published() {
        let mut value = attachment();
        value.content_type = None;
        assert!(
            attachment_observation_draft(&value)
                .expect("incomplete attachment is not an error")
                .is_none()
        );
    }

    #[test]
    fn provider_filename_outside_neutral_contract_does_not_reject_media_observation() {
        let mut value = attachment();
        value.filename = Some("изображение.jpg".to_owned());

        let draft = attachment_observation_draft(&value)
            .expect("provider filename must not stop projection")
            .expect("complete media metadata must remain observable");

        assert!(draft.attachment_descriptor.is_none());
    }
}

pub fn source_envelope(observation: &TelegramMessageObservation) -> SourceEnvelope {
    SourceEnvelope {
        provider: ProviderProvenanceV1::Telegram,
        external_record_id: format!(
            "telegram:{}:{}:{}",
            observation.account_id, observation.provider_chat_id, observation.provider_message_id
        ),
        scope: Some(SourceScopeEnvelope {
            external_account_id: observation.account_id.to_string(),
            external_conversation_id: Some(observation.provider_chat_id.clone()),
            external_participant_id: Some(observation.sender_id.clone()),
            external_media_id: observation
                .media
                .as_ref()
                .and_then(|media| media.provider_file_id.clone()),
            external_reply_to_record_id: observation.references.reply_to.as_ref().map(
                |reference| {
                    telegram_record_id(
                        &observation.account_id,
                        &reference.provider_chat_id,
                        &reference.provider_message_id,
                    )
                },
            ),
            external_forward_origin_record_id: observation
                .references
                .forward_origin
                .as_ref()
                .and_then(
                    |origin| match (&origin.provider_chat_id, &origin.provider_message_id) {
                        (Some(chat_id), Some(message_id)) => Some(telegram_record_id(
                            &observation.account_id,
                            chat_id,
                            message_id,
                        )),
                        _ => None,
                    },
                ),
        }),
    }
}

fn telegram_record_id(account_id: &str, chat_id: &str, message_id: &str) -> String {
    format!("telegram:{account_id}:{chat_id}:{message_id}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_telegram_api::{TelegramAccountSetup, TelegramCredentialPurpose};
    use makosh_vault_protocol::{SecretClassV1, VaultActionV1};

    fn account_setup() -> TelegramAccountSetup {
        TelegramAccountSetup {
            account_id: "telegram-account".to_owned(),
            display_name: "Personal Telegram".to_owned(),
            external_account_id: "telegram:42".to_owned(),
            credentials: vec![
                TelegramCredentialBinding {
                    purpose: TelegramCredentialPurpose::ApiHash,
                    revision: 1,
                },
                TelegramCredentialBinding {
                    purpose: TelegramCredentialPurpose::SessionEncryptionKey,
                    revision: 1,
                },
            ],
            qr_authorized: false,
        }
    }

    #[test]
    fn credential_purpose_is_scoped_to_provider_credential_and_resolve() {
        let binding = &account_setup().credentials[0];
        let purpose =
            credential_lease_purpose("cfg-1", binding).expect("valid Telegram Vault purpose");
        assert_eq!(purpose.purpose_id(), TELEGRAM_API_HASH_PURPOSE_ID);
        assert_eq!(
            purpose.allowed_secret_classes(),
            &[SecretClassV1::ProviderCredential]
        );
        assert_eq!(purpose.actions(), &[VaultActionV1::Resolve]);
    }

    #[test]
    fn qr_password_attempts_stop_after_five() {
        let mut session = qr_login_preparing("setup-1", "telegram-account", 100);
        session = qr_login_qr_issued(&session, "tg://login?token=x").expect("QR issued");
        session = qr_login_password_required(&session).expect("password requested");
        for _ in 0..5 {
            session = qr_login_password_submitted(&session).expect("attempt accepted");
        }
        assert_eq!(session.password_attempts, 5);
        assert_eq!(
            qr_login_password_submitted(&session),
            Err(TelegramContractError::InvalidTransition)
        );
    }

    #[test]
    fn projection_and_evidence_use_provider_source_without_business_entity() {
        let observation = TelegramMessageObservation {
            sender_source_identity: None,
            account_id: "telegram-account".to_owned(),
            provider_chat_id: "100".to_owned(),
            provider_message_id: "200".to_owned(),
            provider_topic_id: None,
            sender_id: "42".to_owned(),
            sender_display_name: Some("Owner".to_owned()),
            is_outgoing: true,
            text: Some("hello".to_owned()),
            media: None,
            references: makosh_telegram_api::TelegramMessageReferences::default(),
            observed_at_unix_seconds: 10,
        };
        let projection = project_message(&observation).expect("projection");
        let draft = observation_draft(observation).expect("neutral evidence draft");
        assert_eq!(projection.message_id, "telegram:telegram-account:100:200");
        assert_eq!(draft.source.provider, ProviderProvenanceV1::Telegram);
        assert_eq!(draft.direction, CommunicationDirectionV1::Outgoing);
        assert_eq!(
            draft.source.external_record_id,
            "telegram:telegram-account:100:200"
        );
    }

    #[test]
    fn provider_display_label_outside_neutral_contract_does_not_reject_message_observation() {
        let observation = TelegramMessageObservation {
            sender_source_identity: None,
            account_id: "telegram-account".to_owned(),
            provider_chat_id: "100".to_owned(),
            provider_message_id: "200".to_owned(),
            provider_topic_id: None,
            sender_id: "42".to_owned(),
            sender_display_name: Some("x".repeat(257)),
            is_outgoing: false,
            text: Some("hello".to_owned()),
            media: None,
            references: makosh_telegram_api::TelegramMessageReferences::default(),
            observed_at_unix_seconds: 10,
        };

        let draft = observation_draft(observation)
            .expect("provider display label must not stop projection");

        assert!(draft.participant_display_label.is_none());
    }

    #[test]
    fn runtime_reconfiguration_fences_epoch_before_any_transition() {
        let account = TelegramAccount {
            account_id: "telegram-account".to_owned(),
            display_name: "Personal Telegram".to_owned(),
            external_account_id: "telegram:42".to_owned(),
            state: TelegramAccountState::Ready,
            runtime_state: TelegramRuntimeState::Running,
            runtime_epoch: 4,
        };
        assert_eq!(
            accept_runtime_reconfiguration(&account, "reconfiguration-1", 3),
            Err(TelegramContractError::RuntimeEpochConflict)
        );
        assert_eq!(account.runtime_epoch, 4);
        assert_eq!(account.runtime_state, TelegramRuntimeState::Running);
    }

    #[test]
    fn first_runtime_activation_initializes_the_account_epoch() {
        let account = TelegramAccount {
            account_id: "telegram-account".to_owned(),
            display_name: "Personal Telegram".to_owned(),
            external_account_id: String::new(),
            state: TelegramAccountState::Provisioning,
            runtime_state: TelegramRuntimeState::Stopped,
            runtime_epoch: 0,
        };

        let running = TelegramLifecycle.mark_running(&account);

        assert_eq!(running.state, TelegramAccountState::Ready);
        assert_eq!(running.runtime_state, TelegramRuntimeState::Running);
        assert_eq!(running.runtime_epoch, 1);
    }

    #[test]
    fn runtime_reconfiguration_applies_one_target_epoch_only_at_completion() {
        let account = TelegramAccount {
            account_id: "telegram-account".to_owned(),
            display_name: "Personal Telegram".to_owned(),
            external_account_id: "telegram:42".to_owned(),
            state: TelegramAccountState::Ready,
            runtime_state: TelegramRuntimeState::Running,
            runtime_epoch: 4,
        };
        let accepted = accept_runtime_reconfiguration(&account, "reconfiguration-1", 4)
            .expect("accepted reconfiguration");
        assert_eq!(accepted.target_runtime_epoch, 5);
        assert_eq!(
            accepted.state,
            TelegramRuntimeReconfigurationState::Accepted
        );
        let applying = apply_runtime_reconfiguration(&accepted).expect("applying reconfiguration");
        assert_eq!(account.runtime_epoch, 4);
        let (completed_account, completed) =
            complete_runtime_reconfiguration(&account, &applying).expect("completed");
        assert_eq!(completed_account.runtime_epoch, 5);
        assert_eq!(
            completed.state,
            TelegramRuntimeReconfigurationState::Completed
        );
        assert_eq!(
            complete_runtime_reconfiguration(&completed_account, &applying),
            Err(TelegramContractError::InvalidTransition)
        );
    }
}
