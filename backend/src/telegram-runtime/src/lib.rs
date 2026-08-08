//! Telegram runtime orchestration. Provider execution stays behind the TDLib port.

#![allow(clippy::collapsible_if)]

pub mod admission;
pub mod automation_client_port;
pub mod bootstrap;
pub mod call_evidence_outbox;
mod calls_backfill;
pub mod calls_client_port;
mod calls_execution;
pub mod client_port;
mod client_realtime;
pub mod client_transport;
pub mod communications_outbox;
mod configuration_client_port;
pub mod delivery_intent_consumer;
pub mod delivery_intent_execution;
pub mod delivery_intent_outbox;
pub mod delivery_intent_result;
pub mod delivery_intent_worker;
mod durable_restore;
pub mod managed_control;
pub mod process;
mod projection_cache;
pub mod settings;
pub mod vault_credentials;

use makosh_blob_client::BlobDataClient;
use makosh_blob_client_contract::BlobReadPort;
use makosh_communications_ingress::{
    BodyAdmissionFailureV1, BodyAvailabilityV1, BodyBlobReceiptV1, CommunicationObservationDraft,
    ObservationEnvelopeContextV1, build_observation_outbox_record_v1, with_admitted_body_blob,
    with_body_admission_failure,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_runtime_protocol::v1::BlobDataSessionGrantV1;
use makosh_telegram_api::{
    TelegramAccount, TelegramAccountSetup, TelegramAccountState, TelegramContractError,
    TelegramDeliveryState, TelegramDownloadFile, TelegramFileSnapshot, TelegramMessageObservation,
    TelegramMessageTombstone, TelegramOperation, TelegramParticipantFilter,
    TelegramParticipantPage, TelegramProviderCommand, TelegramProviderQuery,
    TelegramProviderQueryResponse, TelegramRealtimeFrame, TelegramRuntimeLease,
    TelegramRuntimeLeaseState, TelegramRuntimeReconfiguration,
    TelegramRuntimeReconfigurationRequest, TelegramRuntimeReconfigurationState,
    TelegramRuntimeState, TelegramSendMessage, TelegramTombstoneReason, TelegramTopic,
    provider_command_account_id, provider_command_operation_id, provider_query_account_id,
    validate_provider_command, validate_provider_query, validate_runtime_reconfiguration_request,
    validate_setup,
};
use makosh_telegram_call_media_contract::{TelegramCallReadyMaterialV1, TelegramCallSecretBytesV1};
use makosh_telegram_core::{
    TelegramLifecycle, accept_operation, accept_runtime_reconfiguration,
    complete_runtime_reconfiguration, credential_lease_purposes, event_chat_state,
    event_message_mutation, fail_runtime_reconfiguration, observation_draft,
    operation_awaiting_provider, operation_completed, operation_failed, operation_retry_scheduled,
    project_message, provider_event_draft, qr_login_password_required, qr_login_password_submitted,
    qr_login_preparing, qr_login_qr_issued, qr_login_ready,
};
use makosh_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use makosh_telegram_tdlib::{
    TdJsonLibrary, TdJsonTransport, TdlibAuthorizationDriver, TdlibAuthorizationEvent,
    TdlibAuthorizationParameters, TdlibCallObservation, TdlibError, TdlibProviderUpdate,
    TdlibRequest, TdlibResponse, TdlibTransport, TelegramMediaMaterializer, get_chats_request,
    get_history_request, get_history_request_with_options, parse_file_snapshot,
    parse_provider_events, parse_topic_list,
};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use projection_cache::TelegramRuntimeProjectionCache;

pub const PACKAGE: &str = "makosh-telegram-runtime";

#[derive(Debug)]
pub enum TelegramCallProviderUpdate {
    Observation(TdlibCallObservation),
    Ready {
        observation: TdlibCallObservation,
        material: TelegramCallReadyMaterialV1,
    },
    Signaling {
        account_id: String,
        tdlib_call_id: i32,
        data: TelegramCallSecretBytesV1,
    },
}

#[derive(Debug)]
pub struct TelegramProviderPollBatch {
    pub frames: Vec<TelegramRealtimeFrame>,
    pub call_updates: Vec<TelegramCallProviderUpdate>,
}

#[derive(Debug)]
pub enum TelegramDurableExecutionError {
    Persistence(TelegramDurablePersistenceError),
    Provider,
}

fn event_chat_operational_state(
    event: &makosh_telegram_api::TelegramProviderEvent,
) -> Option<(&str, &str)> {
    match event {
        makosh_telegram_api::TelegramProviderEvent::ChatPositionChanged(position) => {
            Some((&position.account_id, &position.provider_chat_id))
        }
        makosh_telegram_api::TelegramProviderEvent::ChatNotificationChanged {
            account_id,
            provider_chat_id,
            ..
        }
        | makosh_telegram_api::TelegramProviderEvent::ChatMarkedUnreadChanged {
            account_id,
            provider_chat_id,
            ..
        } => Some((account_id, provider_chat_id)),
        _ => None,
    }
}

#[derive(Debug)]
pub enum TelegramDurableLifecycleError {
    Contract(TelegramContractError),
    Persistence(TelegramDurablePersistenceError),
}

fn map_reconfiguration_persistence_error(
    error: TelegramDurablePersistenceError,
) -> TelegramDurableLifecycleError {
    let contract = match error {
        TelegramDurablePersistenceError::RuntimeEpochConflict => {
            Some(TelegramContractError::RuntimeEpochConflict)
        }
        TelegramDurablePersistenceError::ReconfigurationCollision => {
            Some(TelegramContractError::ReconfigurationCollision)
        }
        TelegramDurablePersistenceError::ReconfigurationInProgress => {
            Some(TelegramContractError::ReconfigurationInProgress)
        }
        TelegramDurablePersistenceError::ReconfigurationUnknown => {
            Some(TelegramContractError::ReconfigurationUnknown)
        }
        TelegramDurablePersistenceError::InvalidReconfigurationTransition => {
            Some(TelegramContractError::InvalidTransition)
        }
        _ => None,
    };
    contract.map_or(
        TelegramDurableLifecycleError::Persistence(error),
        TelegramDurableLifecycleError::Contract,
    )
}

#[derive(Debug)]
pub enum TelegramDurableProjectionError {
    Contract(TelegramContractError),
    Persistence(TelegramDurablePersistenceError),
    Provider(TdlibError),
}

static NEXT_MEDIA_FILE_ID: AtomicU64 = AtomicU64::new(1);

fn provider_event_matches_command(
    event: &makosh_telegram_api::TelegramProviderEvent,
    command: &TelegramProviderCommand,
) -> bool {
    match (event, command) {
        (
            makosh_telegram_api::TelegramProviderEvent::MessagePinned {
                account_id,
                provider_chat_id,
                provider_message_id,
                is_pinned,
            },
            TelegramProviderCommand::Pin {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                provider_message_id: command_message_id,
                active,
                ..
            },
        ) => {
            account_id == command_account_id
                && provider_chat_id == command_chat_id
                && provider_message_id == command_message_id
                && is_pinned == active
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ReactionChanged {
                account_id,
                provider_chat_id,
                provider_message_id,
                emoji,
                is_active,
            },
            TelegramProviderCommand::Reaction {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                provider_message_id: command_message_id,
                emoji: command_emoji,
                active,
                ..
            },
        ) => {
            account_id == command_account_id
                && provider_chat_id == command_chat_id
                && provider_message_id == command_message_id
                && emoji.as_deref() == Some(command_emoji.as_str())
                && is_active == active
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ReactionsObserved {
                account_id,
                provider_chat_id,
                provider_message_id,
                reactions,
            },
            TelegramProviderCommand::Reaction {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                provider_message_id: command_message_id,
                emoji: command_emoji,
                active,
                ..
            },
        ) => {
            account_id == command_account_id
                && provider_chat_id == command_chat_id
                && provider_message_id == command_message_id
                && reactions.iter().any(|reaction| {
                    reaction.emoji == command_emoji.as_str() && reaction.is_active == *active
                })
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ChatMarkedUnreadChanged {
                account_id,
                provider_chat_id,
                is_marked_as_unread,
            },
            TelegramProviderCommand::MarkUnread {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                unread,
                ..
            },
        ) => {
            account_id == command_account_id
                && provider_chat_id == command_chat_id
                && is_marked_as_unread == unread
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ChatUnreadChanged {
                account_id,
                provider_chat_id,
                last_read_inbox_message_id,
                ..
            },
            TelegramProviderCommand::MarkUnread {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                unread: false,
                read_through_provider_message_id: Some(command_message_id),
                ..
            },
        ) => {
            account_id == command_account_id
                && provider_chat_id == command_chat_id
                && last_read_inbox_message_id.as_deref() == Some(command_message_id.as_str())
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ChatNotificationChanged {
                account_id,
                provider_chat_id,
                mute_for_seconds,
                ..
            },
            TelegramProviderCommand::Mute {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                muted,
                ..
            },
        ) => {
            account_id == command_account_id
                && provider_chat_id == command_chat_id
                && (*mute_for_seconds > 0) == *muted
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ChatPositionChanged(position),
            TelegramProviderCommand::Archive {
                account_id,
                provider_chat_id,
                archived,
                ..
            },
        ) => {
            position.account_id == *account_id
                && position.provider_chat_id == *provider_chat_id
                && position.list_kind == "archive"
                && (position.order > 0) == *archived
        }
        (
            makosh_telegram_api::TelegramProviderEvent::TopicChanged(topic),
            TelegramProviderCommand::SetTopicClosed {
                account_id,
                provider_chat_id,
                provider_topic_id,
                is_closed,
                ..
            },
        ) => {
            topic.account_id == *account_id
                && topic.provider_chat_id == *provider_chat_id
                && topic.provider_topic_id == *provider_topic_id
                && topic.is_closed == *is_closed
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ChatFoldersChanged {
                account_id,
                folders,
            },
            TelegramProviderCommand::AddChatToFolder {
                account_id: command_account_id,
                provider_chat_id,
                provider_folder_id,
                ..
            },
        ) => {
            account_id == command_account_id
                && folders.iter().any(|folder| {
                    folder.provider_folder_id == *provider_folder_id
                        && (folder
                            .pinned_chat_ids
                            .iter()
                            .any(|id| id == provider_chat_id)
                            || folder
                                .included_chat_ids
                                .iter()
                                .any(|id| id == provider_chat_id))
                })
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ChatFoldersChanged {
                account_id,
                folders,
            },
            TelegramProviderCommand::RemoveChatFromFolder {
                account_id: command_account_id,
                provider_chat_id,
                provider_folder_id,
                ..
            },
        ) => {
            account_id == command_account_id
                && folders.iter().any(|folder| {
                    folder.provider_folder_id == *provider_folder_id
                        && folder
                            .excluded_chat_ids
                            .iter()
                            .any(|id| id == provider_chat_id)
                })
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ParticipantChanged(participant),
            TelegramProviderCommand::Join {
                account_id,
                provider_chat_id,
                ..
            },
        ) => {
            participant.account_id == *account_id
                && participant.provider_chat_id == *provider_chat_id
                && !matches!(participant.status.as_str(), "left" | "banned")
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ParticipantChanged(participant),
            TelegramProviderCommand::Leave {
                account_id,
                provider_chat_id,
                ..
            },
        ) => {
            participant.account_id == *account_id
                && participant.provider_chat_id == *provider_chat_id
                && matches!(participant.status.as_str(), "left" | "banned")
        }
        _ => false,
    }
}

fn provider_event_targets_command(
    event: &makosh_telegram_api::TelegramProviderEvent,
    command: &TelegramProviderCommand,
) -> bool {
    match (event, command) {
        (
            makosh_telegram_api::TelegramProviderEvent::MessagePinned {
                account_id,
                provider_chat_id,
                provider_message_id,
                ..
            },
            TelegramProviderCommand::Pin {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                provider_message_id: command_message_id,
                ..
            },
        )
        | (
            makosh_telegram_api::TelegramProviderEvent::ReactionChanged {
                account_id,
                provider_chat_id,
                provider_message_id,
                ..
            },
            TelegramProviderCommand::Reaction {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                provider_message_id: command_message_id,
                ..
            },
        )
        | (
            makosh_telegram_api::TelegramProviderEvent::ReactionsObserved {
                account_id,
                provider_chat_id,
                provider_message_id,
                ..
            },
            TelegramProviderCommand::Reaction {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                provider_message_id: command_message_id,
                ..
            },
        ) => {
            account_id == command_account_id
                && provider_chat_id == command_chat_id
                && provider_message_id == command_message_id
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ChatMarkedUnreadChanged {
                account_id,
                provider_chat_id,
                ..
            },
            TelegramProviderCommand::MarkUnread {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                ..
            },
        )
        | (
            makosh_telegram_api::TelegramProviderEvent::ChatUnreadChanged {
                account_id,
                provider_chat_id,
                ..
            },
            TelegramProviderCommand::MarkUnread {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                ..
            },
        )
        | (
            makosh_telegram_api::TelegramProviderEvent::ChatNotificationChanged {
                account_id,
                provider_chat_id,
                ..
            },
            TelegramProviderCommand::Mute {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                ..
            },
        )
        | (
            makosh_telegram_api::TelegramProviderEvent::ParticipantChanged(
                makosh_telegram_api::TelegramParticipant {
                    account_id,
                    provider_chat_id,
                    ..
                },
            ),
            TelegramProviderCommand::Join {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                ..
            },
        )
        | (
            makosh_telegram_api::TelegramProviderEvent::ParticipantChanged(
                makosh_telegram_api::TelegramParticipant {
                    account_id,
                    provider_chat_id,
                    ..
                },
            ),
            TelegramProviderCommand::Leave {
                account_id: command_account_id,
                provider_chat_id: command_chat_id,
                ..
            },
        ) => account_id == command_account_id && provider_chat_id == command_chat_id,
        (
            makosh_telegram_api::TelegramProviderEvent::ChatPositionChanged(position),
            TelegramProviderCommand::Archive {
                account_id,
                provider_chat_id,
                ..
            },
        ) => {
            position.account_id == *account_id
                && position.provider_chat_id == *provider_chat_id
                && position.list_kind == "archive"
        }
        (
            makosh_telegram_api::TelegramProviderEvent::TopicChanged(topic),
            TelegramProviderCommand::SetTopicClosed {
                account_id,
                provider_chat_id,
                provider_topic_id,
                ..
            },
        ) => {
            topic.account_id == *account_id
                && topic.provider_chat_id == *provider_chat_id
                && topic.provider_topic_id == *provider_topic_id
        }
        (
            makosh_telegram_api::TelegramProviderEvent::ChatFoldersChanged {
                account_id,
                folders,
            },
            TelegramProviderCommand::AddChatToFolder {
                account_id: command_account_id,
                provider_chat_id,
                provider_folder_id,
                ..
            },
        )
        | (
            makosh_telegram_api::TelegramProviderEvent::ChatFoldersChanged {
                account_id,
                folders,
            },
            TelegramProviderCommand::RemoveChatFromFolder {
                account_id: command_account_id,
                provider_chat_id,
                provider_folder_id,
                ..
            },
        ) => {
            account_id == command_account_id
                && folders.iter().any(|folder| {
                    folder.provider_folder_id == *provider_folder_id
                        && (folder
                            .pinned_chat_ids
                            .iter()
                            .any(|id| id == provider_chat_id)
                            || folder
                                .included_chat_ids
                                .iter()
                                .any(|id| id == provider_chat_id)
                            || folder
                                .excluded_chat_ids
                                .iter()
                                .any(|id| id == provider_chat_id))
                })
        }
        _ => false,
    }
}

pub struct TelegramBlobMaterializationSession {
    pub blob_ref: String,
    pub grant: BlobDataSessionGrantV1,
    pub channel_binding: Vec<u8>,
    pub declared_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_telegram_api::{
        TelegramMessageObservation, TelegramMessageReferences, TelegramProviderEvent,
    };

    struct PollingTransport {
        events: Vec<TelegramProviderEvent>,
    }

    impl TdlibTransport for PollingTransport {
        fn request(&mut self, _request: TdlibRequest) -> Result<TdlibResponse, TdlibError> {
            Err(TdlibError::Transport(
                "request is not used by polling test".to_owned(),
            ))
        }

        fn poll_updates(&mut self) -> Result<Vec<TdlibProviderUpdate>, TdlibError> {
            Ok(std::mem::take(&mut self.events)
                .into_iter()
                .map(|event| TdlibProviderUpdate::Operational(Box::new(event)))
                .collect())
        }
    }

    #[test]
    fn provider_polling_assigns_replay_sequence_and_cursor() {
        let mut runtime = TelegramRuntime::new(PollingTransport {
            events: vec![TelegramProviderEvent::ChatUnreadChanged {
                account_id: "account".to_owned(),
                provider_chat_id: "chat".to_owned(),
                unread_count: Some(3),
                unread_mention_count: Some(1),
                last_read_inbox_message_id: Some("message".to_owned()),
            }],
        });

        let frames = runtime
            .poll_provider_events("account", Some("cursor-1".to_owned()))
            .expect("provider polling");

        assert_eq!(frames.frames.len(), 1);
        assert!(frames.call_updates.is_empty());
        assert_eq!(frames.frames[0].sequence, 1);
        assert_eq!(
            frames.frames[0].provider_cursor.as_deref(),
            Some("cursor-1")
        );
        assert_eq!(runtime.realtime_after("account", 0), frames.frames);
    }

    #[test]
    fn provider_content_edit_advances_current_projection_by_frame_order() {
        let message_id = "telegram:account:chat:message";
        let mut runtime = TelegramRuntime::new(PollingTransport {
            events: vec![
                TelegramProviderEvent::MessageCreated(TelegramMessageObservation {
                    account_id: "account".to_owned(),
                    provider_chat_id: "chat".to_owned(),
                    provider_message_id: "message".to_owned(),
                    provider_topic_id: None,
                    sender_id: "sender".to_owned(),
                    sender_display_name: Some("Sender".to_owned()),
                    is_outgoing: false,
                    text: Some("original body".to_owned()),
                    media: None,
                    references: TelegramMessageReferences::default(),
                    observed_at_unix_seconds: 1_782_504_000,
                }),
                TelegramProviderEvent::MessageEdited {
                    account_id: "account".to_owned(),
                    provider_chat_id: "chat".to_owned(),
                    provider_message_id: "message".to_owned(),
                    text: Some("edited body".to_owned()),
                    observed_at_unix_seconds: 0,
                },
            ],
        });

        runtime
            .poll_provider_events("account", Some("cursor-1".to_owned()))
            .expect("provider polling");

        let message = runtime
            .persistence
            .message(message_id)
            .expect("current message projection");
        assert_eq!(message.text.as_deref(), Some("edited body"));
        assert_eq!(message.observed_at_unix_seconds, 1_782_504_000);
        let versions = runtime
            .persistence
            .message_versions(message_id)
            .expect("message version history");
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version_number, 1);
        assert_eq!(versions[0].body_text.as_deref(), Some("original body"));
        assert_eq!(versions[1].version_number, 2);
        assert_eq!(versions[1].body_text.as_deref(), Some("edited body"));
        assert_eq!(versions[1].observed_at_unix_seconds, 0);
    }

    #[test]
    fn admitted_runtime_lease_is_derived_from_current_runtime_identity() {
        let mut runtime = TelegramRuntime::new(PollingTransport { events: Vec::new() });
        runtime.persistence.put_account(TelegramAccount {
            account_id: "account".to_owned(),
            display_name: "Telegram".to_owned(),
            external_account_id: "telegram:1".to_owned(),
            state: TelegramAccountState::Ready,
            runtime_state: TelegramRuntimeState::Running,
            runtime_epoch: 7,
        });
        runtime.set_admission(Some(TelegramRuntimeAdmission {
            logical_owner_id: "owner".to_owned(),
            logical_human_owner_id: "owner-1".to_owned(),
            configuration_instance_id: "cfg".to_owned(),
            module_registration_id: "registration".to_owned(),
            runtime_instance_id: "admitted-runtime".to_owned(),
            runtime_generation: 4,
            grant_epoch: 5,
            vault_runtime_generation: 6,
            api_hash_revision: 1,
            session_encryption_key_revision: 2,
        }));
        runtime
            .install_admitted_runtime_lease("account")
            .expect("internal admitted lease");
        let lease = runtime
            .persistence
            .runtime_lease("account")
            .expect("runtime lease");
        assert_eq!(lease.topology, "managed_account_v1");
        assert_eq!(lease.holder, "admitted-runtime");
        assert_eq!(lease.epoch, 7);
        assert_eq!(lease.expires_at_unix_seconds, u64::MAX);
    }

    #[test]
    fn provider_text_uses_injected_body_admission_and_records_typed_failure() {
        let observation = TelegramMessageObservation {
            account_id: "account".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: "message".to_owned(),
            provider_topic_id: None,
            sender_id: "sender".to_owned(),
            sender_display_name: None,
            is_outgoing: false,
            text: Some("hello".to_owned()),
            media: None,
            references: TelegramMessageReferences::default(),
            observed_at_unix_seconds: 1,
        };
        let event = TelegramProviderEvent::MessageCreated(observation.clone());
        let draft = makosh_telegram_core::observation_draft(observation).expect("draft");
        let receipt = BodyBlobReceiptV1 {
            blob_ref: "blob-content:test".to_owned(),
            reference_id: [7; 16],
            declared_bytes: 5,
            sha256: [8; 32],
            custody_transfer_source_proof: vec![3; 96],
            media_type: "text/plain".to_owned(),
        };
        let admitted =
            admit_provider_event_body(draft.clone(), &event, &mut |_| Ok(receipt.clone()))
                .expect("admitted body");
        assert_eq!(admitted.body, BodyAvailabilityV1::AdmittedBlob);
        assert_eq!(admitted.body_blob, Some(receipt));

        let failed = admit_provider_event_body(draft, &event, &mut |_| {
            Err(BodyAdmissionFailureV1::PolicyRejected)
        })
        .expect("typed admission failure");
        assert_eq!(failed.body, BodyAvailabilityV1::Unavailable);
        assert_eq!(
            failed.body_admission_failure,
            Some(BodyAdmissionFailureV1::PolicyRejected)
        );
    }

    #[test]
    fn communications_observation_uses_exact_admitted_module_identity() {
        let mut runtime = TelegramRuntime::new(PollingTransport { events: Vec::new() });
        runtime.set_admission(Some(TelegramRuntimeAdmission {
            logical_owner_id: "owner".to_owned(),
            logical_human_owner_id: "owner-1".to_owned(),
            configuration_instance_id: "configuration".to_owned(),
            module_registration_id: "telegram-registration".to_owned(),
            runtime_instance_id: "telegram-runtime-1".to_owned(),
            runtime_generation: 7,
            grant_epoch: 11,
            vault_runtime_generation: 3,
            api_hash_revision: 5,
            session_encryption_key_revision: 6,
        }));
        let draft = makosh_telegram_core::observation_draft(TelegramMessageObservation {
            account_id: "account".to_owned(),
            provider_chat_id: "chat".to_owned(),
            provider_message_id: "message".to_owned(),
            provider_topic_id: None,
            sender_id: "sender".to_owned(),
            sender_display_name: None,
            is_outgoing: false,
            text: None,
            media: None,
            references: TelegramMessageReferences::default(),
            observed_at_unix_seconds: 1,
        })
        .expect("Telegram observation draft");

        let (record, _) = runtime
            .communication_observation_record(&draft)
            .expect("Telegram outbox record");
        let envelope =
            makosh_events_protocol::validation::envelope::decode_envelope_v1(record.exact_bytes())
                .expect("durable observation envelope");
        let source = envelope.source.expect("durable observation source");

        assert_eq!(source.module_id, PACKAGE);
        assert_eq!(source.runtime_instance_id.len(), 16);
        assert_eq!(source.runtime_generation, 7);
    }
}

pub struct TelegramBlobMaterializer<R> {
    reader: R,
    temp_dir: PathBuf,
    sessions: Vec<TelegramBlobMaterializationSession>,
}

impl<R> TelegramBlobMaterializer<R> {
    pub fn new(reader: R, temp_dir: impl Into<PathBuf>) -> Result<Self, TdlibError> {
        let temp_dir = temp_dir.into();
        if !temp_dir.is_absolute() {
            return Err(TdlibError::Protocol(
                "Telegram media temp directory must be absolute".to_owned(),
            ));
        }
        Ok(Self {
            reader,
            temp_dir,
            sessions: Vec::new(),
        })
    }

    pub fn add_session(
        &mut self,
        session: TelegramBlobMaterializationSession,
    ) -> Result<(), TdlibError> {
        if session.blob_ref.trim().is_empty()
            || session.channel_binding.is_empty()
            || session.declared_size == 0
        {
            return Err(TdlibError::Protocol(
                "Telegram Blob materialization session is invalid".to_owned(),
            ));
        }
        self.sessions.push(session);
        Ok(())
    }
}

impl<R: BlobReadPort> TelegramMediaMaterializer for TelegramBlobMaterializer<R> {
    fn materialize(&mut self, blob_ref: &str) -> Result<String, TdlibError> {
        let index = self
            .sessions
            .iter()
            .position(|session| session.blob_ref == blob_ref)
            .ok_or_else(|| {
                TdlibError::Protocol("Telegram Blob session is unavailable".to_owned())
            })?;
        let session = self.sessions.remove(index);
        let bytes = self
            .reader
            .read_range(
                session.grant,
                session.channel_binding,
                0,
                session.declared_size,
            )
            .map_err(|error| {
                TdlibError::Protocol(format!("Telegram Blob read failed: {error:?}"))
            })?;
        if bytes.len() as u64 != session.declared_size {
            return Err(TdlibError::Protocol(
                "Telegram Blob size does not match grant".to_owned(),
            ));
        }
        let file_id = NEXT_MEDIA_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let path = self
            .temp_dir
            .join(format!("makosh-telegram-media-{file_id}"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                TdlibError::Protocol(format!("Telegram media staging failed: {error}"))
            })?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|error| {
            TdlibError::Protocol(format!(
                "Telegram media staging permissions failed: {error}"
            ))
        })?;
        if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
            let _ = fs::remove_file(&path);
            return Err(TdlibError::Protocol(format!(
                "Telegram media staging write failed: {error}"
            )));
        }
        Ok(path.to_string_lossy().into_owned())
    }

    fn release(&mut self, materialized_path: &str) {
        let path = Path::new(materialized_path);
        if path.starts_with(&self.temp_dir) {
            let _ = fs::remove_file(path);
        }
    }
}

pub struct TelegramRuntime<T> {
    transport: T,
    persistence: TelegramRuntimeProjectionCache,
    lifecycle: TelegramLifecycle,
    media_materializer: Option<TelegramBlobMaterializer<BlobDataClient>>,
    call_media:
        Option<Box<dyn makosh_telegram_call_media_contract::TelegramCallSignalingMediaPort>>,
    active_call_media: Option<calls_execution::TelegramActiveCallMediaSession>,
    admission: Option<TelegramRuntimeAdmission>,
    pending_reconfiguration: Option<TelegramRuntimeReconfiguration>,
}

pub struct TelegramRuntimeComposition {
    account_id: String,
    account_setup: Option<TelegramAccountSetup>,
    library: TdJsonLibrary,
    authorization: Option<TdlibAuthorizationDriver>,
    runtime: Option<TelegramRuntime<TdJsonTransport>>,
    call_media:
        Option<Box<dyn makosh_telegram_call_media_contract::TelegramCallSignalingMediaPort>>,
    admission: Option<TelegramRuntimeAdmission>,
    pending_reconfiguration: Option<TelegramRuntimeReconfiguration>,
}

#[derive(Clone)]
pub struct TelegramRuntimeAdmission {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub configuration_instance_id: String,
    pub module_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub vault_runtime_generation: u64,
    pub api_hash_revision: u64,
    pub session_encryption_key_revision: u64,
}

impl TelegramRuntimeComposition {
    pub fn new_configuration_only(
        library: TdJsonLibrary,
        account_id: impl Into<String>,
    ) -> Result<Self, TdlibError> {
        let account_id = account_id.into();
        if account_id.trim().is_empty() {
            return Err(TdlibError::Protocol(
                "Telegram account id is empty".to_owned(),
            ));
        }
        Ok(Self {
            account_id,
            account_setup: None,
            library,
            authorization: None,
            runtime: None,
            call_media: None,
            admission: None,
            pending_reconfiguration: None,
        })
    }

    pub fn new(
        library: TdJsonLibrary,
        account_id: impl Into<String>,
        parameters: TdlibAuthorizationParameters,
    ) -> Result<Self, TdlibError> {
        let account_id: String = account_id.into();
        Self::new_with_account_setup(
            library,
            TelegramAccountSetup {
                account_id: account_id.clone(),
                display_name: account_id.clone(),
                external_account_id: account_id,
                credentials: Vec::new(),
                qr_authorized: false,
            },
            parameters,
        )
    }

    pub fn new_with_account_setup(
        library: TdJsonLibrary,
        account_setup: TelegramAccountSetup,
        parameters: TdlibAuthorizationParameters,
    ) -> Result<Self, TdlibError> {
        let mut composition =
            Self::new_configuration_only(library, account_setup.account_id.clone())?;
        composition.begin_account_authorization(account_setup, parameters)?;
        Ok(composition)
    }

    pub fn begin_account_authorization(
        &mut self,
        account_setup: TelegramAccountSetup,
        parameters: TdlibAuthorizationParameters,
    ) -> Result<(), TdlibError> {
        if self.authorization.is_some() && self.account_setup.as_ref() == Some(&account_setup) {
            return Ok(());
        }
        if account_setup.account_id != self.account_id
            || self.runtime.is_some()
            || self.authorization.is_some()
        {
            return Err(TdlibError::RuntimeUnavailable);
        }
        let client = self.library.create_client()?;
        self.account_setup = Some(account_setup);
        self.authorization = Some(TdlibAuthorizationDriver::new(client, parameters));
        Ok(())
    }

    #[must_use]
    pub fn configured_account_id(&self) -> &str {
        &self.account_id
    }

    pub fn poll_authorization(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<TdlibAuthorizationEvent>, TdlibError> {
        let event = self
            .authorization
            .as_mut()
            .ok_or_else(|| {
                TdlibError::Protocol("Telegram authorization is unavailable".to_owned())
            })?
            .poll(timeout)?;
        if matches!(
            &event,
            Some(TdlibAuthorizationEvent::State(
                makosh_telegram_tdlib::TdlibAuthorizationUpdate::Ready
            ))
        ) {
            let authorization = self.authorization.take().ok_or_else(|| {
                TdlibError::Protocol("Telegram authorization is unavailable".to_owned())
            })?;
            let transport = authorization.into_transport(self.account_id.clone())?;
            let mut runtime = TelegramRuntime::new(transport);
            runtime.set_admission(self.admission.clone());
            if let Some(call_media) = self.call_media.take() {
                runtime.install_call_media_port(call_media);
            }
            runtime
                .provision_account(self.account_setup.clone().ok_or_else(|| {
                    TdlibError::Protocol("Telegram account setup is unavailable".to_owned())
                })?)
                .map_err(|_| {
                    TdlibError::Protocol("Telegram account provisioning failed".to_owned())
                })?;
            self.runtime = Some(runtime);
        }
        Ok(event)
    }

    pub fn submit_password(&self, password: &str) -> Result<(), TdlibError> {
        self.authorization
            .as_ref()
            .ok_or_else(|| {
                TdlibError::Protocol("Telegram authorization is unavailable".to_owned())
            })?
            .submit_password(password)
    }

    pub fn runtime_mut(&mut self) -> Option<&mut TelegramRuntime<TdJsonTransport>> {
        self.runtime.as_mut()
    }

    pub fn set_admission(&mut self, admission: TelegramRuntimeAdmission) {
        self.admission = Some(admission);
    }

    pub fn install_call_media_port(
        &mut self,
        port: Box<dyn makosh_telegram_call_media_contract::TelegramCallSignalingMediaPort>,
    ) {
        self.call_media = Some(port);
    }

    #[must_use]
    pub fn has_pending_authorization(&self) -> bool {
        self.authorization.is_some()
    }

    #[must_use]
    pub fn has_runtime(&self) -> bool {
        self.runtime.is_some()
    }

    pub fn runtime_admission(&self) -> Option<&TelegramRuntimeAdmission> {
        self.runtime
            .as_ref()
            .and_then(|runtime| runtime.admission.as_ref())
    }

    #[must_use]
    pub fn configured_admission(&self) -> Option<&TelegramRuntimeAdmission> {
        self.admission.as_ref()
    }

    pub fn begin_runtime_reconfiguration(
        &mut self,
        reconfiguration: TelegramRuntimeReconfiguration,
        authorization_parameters: TdlibAuthorizationParameters,
    ) -> Result<(), TdlibError> {
        if self.authorization.is_some()
            || self.pending_reconfiguration.is_some()
            || reconfiguration.account_id != self.account_id
            || reconfiguration.state != TelegramRuntimeReconfigurationState::Applying
        {
            return Err(TdlibError::RuntimeUnavailable);
        }
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(TdlibError::RuntimeUnavailable)?;
        let account = runtime
            .account(&self.account_id)
            .ok_or(TdlibError::RuntimeUnavailable)?;
        if account.runtime_epoch != reconfiguration.expected_runtime_epoch {
            return Err(TdlibError::RuntimeUnavailable);
        }
        let client = self.library.create_client()?;
        let call_media = self
            .runtime
            .as_mut()
            .ok_or(TdlibError::RuntimeUnavailable)?
            .take_call_media_for_reconfiguration()
            .map_err(|_| TdlibError::RuntimeUnavailable)?;
        self.runtime = None;
        self.call_media = call_media;
        self.authorization = Some(TdlibAuthorizationDriver::new(
            client,
            authorization_parameters,
        ));
        self.pending_reconfiguration = Some(reconfiguration);
        Ok(())
    }

    #[must_use]
    pub fn pending_runtime_reconfiguration(&self) -> Option<&TelegramRuntimeReconfiguration> {
        self.pending_reconfiguration.as_ref()
    }

    pub fn clear_pending_runtime_reconfiguration(&mut self) {
        self.pending_reconfiguration = None;
    }

    pub fn poll_runtime_events(
        &mut self,
        provider_cursor: Option<String>,
    ) -> Result<TelegramProviderPollBatch, TdlibError> {
        let account_id = self.account_id.clone();
        self.runtime
            .as_mut()
            .ok_or_else(|| TdlibError::Protocol("Telegram runtime is not authorized".to_owned()))?
            .poll_provider_events(&account_id, provider_cursor)
    }
}

impl<T> TelegramRuntime<T>
where
    T: TdlibTransport,
{
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            persistence: TelegramRuntimeProjectionCache::new(),
            lifecycle: TelegramLifecycle,
            media_materializer: None,
            call_media: None,
            active_call_media: None,
            admission: None,
            pending_reconfiguration: None,
        }
    }

    pub fn set_admission(&mut self, admission: Option<TelegramRuntimeAdmission>) {
        self.admission = admission;
    }

    fn install_admitted_runtime_lease(
        &mut self,
        account_id: &str,
    ) -> Result<(), TelegramContractError> {
        let admission = self
            .admission
            .as_ref()
            .ok_or(TelegramContractError::RuntimeBlocked)?;
        let account = self
            .persistence
            .account(account_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        if account.runtime_epoch == 0
            || !matches!(
                account.runtime_state,
                TelegramRuntimeState::Running | TelegramRuntimeState::Degraded
            )
        {
            return Err(TelegramContractError::RuntimeBlocked);
        }
        self.persistence.put_runtime_lease(TelegramRuntimeLease {
            account_id: account_id.to_owned(),
            topology: "managed_account_v1".to_owned(),
            holder: admission.runtime_instance_id.clone(),
            epoch: account.runtime_epoch,
            state: TelegramRuntimeLeaseState::Active,
            expires_at_unix_seconds: u64::MAX,
        });
        Ok(())
    }

    pub async fn runtime_reconfiguration_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        request: &TelegramRuntimeReconfigurationRequest,
    ) -> Result<TelegramRuntimeReconfiguration, TelegramDurableLifecycleError> {
        validate_runtime_reconfiguration_request(request)
            .map_err(TelegramDurableLifecycleError::Contract)?;
        match request {
            TelegramRuntimeReconfigurationRequest::Begin {
                reconfiguration_id,
                account_id,
                expected_runtime_epoch,
            } => {
                if let Some(existing) = durable
                    .runtime_reconfiguration(reconfiguration_id)
                    .await
                    .map_err(TelegramDurableLifecycleError::Persistence)?
                {
                    return if existing.account_id == *account_id
                        && existing.expected_runtime_epoch == *expected_runtime_epoch
                        && existing.target_runtime_epoch
                            == expected_runtime_epoch.checked_add(1).ok_or(
                                TelegramDurableLifecycleError::Contract(
                                    TelegramContractError::RuntimeEpochConflict,
                                ),
                            )?
                    {
                        Ok(existing)
                    } else {
                        Err(TelegramDurableLifecycleError::Contract(
                            TelegramContractError::ReconfigurationCollision,
                        ))
                    };
                }
                let (account, _) = durable
                    .account(account_id)
                    .await
                    .map_err(TelegramDurableLifecycleError::Persistence)?
                    .ok_or(TelegramDurableLifecycleError::Contract(
                        TelegramContractError::AccountUnknown,
                    ))?;
                let proposed = accept_runtime_reconfiguration(
                    &account,
                    reconfiguration_id,
                    *expected_runtime_epoch,
                )
                .map_err(TelegramDurableLifecycleError::Contract)?;
                let accepted = durable
                    .accept_runtime_reconfiguration(&proposed)
                    .await
                    .map_err(map_reconfiguration_persistence_error)?;
                if accepted.state == TelegramRuntimeReconfigurationState::Accepted
                    && self.pending_reconfiguration.is_none()
                {
                    let runtime_account = self.persistence.account(account_id).ok_or(
                        TelegramDurableLifecycleError::Contract(
                            TelegramContractError::RuntimeBlocked,
                        ),
                    )?;
                    if runtime_account.runtime_epoch != accepted.expected_runtime_epoch {
                        return Err(TelegramDurableLifecycleError::Contract(
                            TelegramContractError::RuntimeEpochConflict,
                        ));
                    }
                    self.pending_reconfiguration = Some(accepted.clone());
                }
                Ok(accepted)
            }
            TelegramRuntimeReconfigurationRequest::Status { reconfiguration_id } => durable
                .runtime_reconfiguration(reconfiguration_id)
                .await
                .map_err(TelegramDurableLifecycleError::Persistence)?
                .ok_or(TelegramDurableLifecycleError::Contract(
                    TelegramContractError::ReconfigurationUnknown,
                )),
        }
    }

    pub fn take_pending_runtime_reconfiguration(
        &mut self,
    ) -> Option<TelegramRuntimeReconfiguration> {
        self.pending_reconfiguration.take()
    }

    #[must_use]
    pub fn has_pending_runtime_reconfiguration(&self) -> bool {
        self.pending_reconfiguration.is_some()
    }

    pub async fn complete_pending_runtime_reconfiguration_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
    ) -> Result<Option<TelegramRuntimeReconfiguration>, TelegramDurableLifecycleError> {
        let Some(pending) = durable
            .pending_runtime_reconfiguration(account_id)
            .await
            .map_err(TelegramDurableLifecycleError::Persistence)?
        else {
            return Ok(None);
        };
        let account = self
            .persistence
            .account(account_id)
            .ok_or(TelegramDurableLifecycleError::Contract(
                TelegramContractError::AccountUnknown,
            ))?
            .clone();
        let (account, completed) = complete_runtime_reconfiguration(&account, &pending)
            .map_err(TelegramDurableLifecycleError::Contract)?;
        let completed = durable
            .complete_runtime_reconfiguration(&account, &completed)
            .await
            .map_err(map_reconfiguration_persistence_error)?;
        self.persistence.put_account(account);
        self.install_admitted_runtime_lease(account_id)
            .map_err(TelegramDurableLifecycleError::Contract)?;
        Ok(Some(completed))
    }

    pub async fn fail_runtime_reconfiguration_durable(
        durable: &TelegramDurablePersistence,
        reconfiguration: &TelegramRuntimeReconfiguration,
        sanitized_reason_code: &str,
    ) -> Result<TelegramRuntimeReconfiguration, TelegramDurableLifecycleError> {
        let failed = fail_runtime_reconfiguration(reconfiguration, sanitized_reason_code)
            .map_err(TelegramDurableLifecycleError::Contract)?;
        durable
            .fail_runtime_reconfiguration(&failed)
            .await
            .map_err(map_reconfiguration_persistence_error)
    }

    pub fn authorize_media_session(
        &mut self,
        session: makosh_blob_client::ManagedBlobSessionV1,
        intent: &makosh_telegram_api::TelegramBlobIntentV1,
    ) -> Result<(), TdlibError> {
        if self.media_materializer.is_none() {
            let reader = BlobDataClient::new(session.data_socket_path).map_err(|_| {
                TdlibError::Protocol("Telegram Blob client is unavailable".to_owned())
            })?;
            self.media_materializer =
                Some(TelegramBlobMaterializer::new(reader, std::env::temp_dir())?);
        }
        let materializer = self.media_materializer.as_mut().ok_or_else(|| {
            TdlibError::Protocol("Telegram Blob materializer is unavailable".to_owned())
        })?;
        materializer.add_session(TelegramBlobMaterializationSession {
            blob_ref: intent.blob_ref.clone(),
            grant: session.grant,
            channel_binding: session.channel_binding,
            declared_size: intent.declared_size,
        })
    }

    pub fn provision_account(
        &mut self,
        setup: TelegramAccountSetup,
    ) -> Result<TelegramAccount, TelegramContractError> {
        validate_setup(&setup)?;
        let account = TelegramAccount {
            account_id: setup.account_id,
            display_name: setup.display_name,
            external_account_id: setup.external_account_id,
            state: TelegramAccountState::Provisioning,
            runtime_state: TelegramRuntimeState::Stopped,
            runtime_epoch: 0,
        };
        self.persistence.put_account(account.clone());
        self.persistence
            .put_credentials(&account.account_id, setup.credentials);
        Ok(account)
    }

    pub async fn provision_account_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        setup: TelegramAccountSetup,
    ) -> Result<TelegramAccount, TelegramDurableLifecycleError> {
        let account = self
            .provision_account(setup)
            .map_err(TelegramDurableLifecycleError::Contract)?;
        let credentials = self
            .persistence
            .credentials(&account.account_id)
            .unwrap_or_default()
            .to_vec();
        durable
            .upsert_account(&account, &credentials)
            .await
            .map_err(TelegramDurableLifecycleError::Persistence)?;
        Ok(account)
    }

    pub async fn restore_account_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
    ) -> Result<Option<TelegramAccount>, TelegramDurableLifecycleError> {
        let Some((account, credentials)) = durable
            .account(account_id)
            .await
            .map_err(TelegramDurableLifecycleError::Persistence)?
        else {
            return Ok(None);
        };
        self.persistence.put_credentials(account_id, credentials);
        self.persistence.put_account(account.clone());
        Ok(Some(account))
    }

    pub fn retire_account(
        &mut self,
        account_id: &str,
    ) -> Result<TelegramAccount, TelegramContractError> {
        let account = self
            .persistence
            .account(account_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        let retired = self.lifecycle.retire(account)?;
        self.persistence.revoke_runtime_lease(account_id);
        self.persistence.put_account(retired.clone());
        Ok(retired)
    }

    pub async fn retire_account_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
    ) -> Result<TelegramAccount, TelegramDurableLifecycleError> {
        if self.persistence.account(account_id).is_none() {
            let (account, _credentials) = durable
                .account(account_id)
                .await
                .map_err(TelegramDurableLifecycleError::Persistence)?
                .ok_or(TelegramDurableLifecycleError::Contract(
                    TelegramContractError::AccountUnknown,
                ))?;
            self.persistence.put_account(account);
        }
        let retired = self
            .retire_account(account_id)
            .map_err(TelegramDurableLifecycleError::Contract)?;
        durable
            .upsert_account(&retired, &[])
            .await
            .map_err(TelegramDurableLifecycleError::Persistence)?;
        Ok(retired)
    }

    pub fn credential_lease_requests(
        &self,
        account_id: &str,
        configuration_instance_id: &str,
    ) -> Result<Vec<makosh_telegram_core::VaultPurposeRequestV1>, TelegramContractError> {
        self.persistence
            .account(account_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        let bindings = self
            .persistence
            .credentials(account_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        credential_lease_purposes(configuration_instance_id, bindings)
    }

    pub fn execute(
        &mut self,
        account_id: &str,
        request: TdlibRequest,
    ) -> Result<TdlibResponse, TdlibError> {
        let account = self
            .persistence
            .account(account_id)
            .ok_or_else(|| TdlibError::Protocol("Telegram account is unknown".to_owned()))?;
        if account.runtime_state != TelegramRuntimeState::Running {
            return Err(TdlibError::Protocol(
                "Telegram account runtime is not running".to_owned(),
            ));
        }
        let lease = self.persistence.runtime_lease(account_id).ok_or_else(|| {
            TdlibError::Protocol("Telegram runtime lease is unavailable".to_owned())
        })?;
        if lease.state != TelegramRuntimeLeaseState::Active || lease.epoch != account.runtime_epoch
        {
            return Err(TdlibError::Protocol(
                "Telegram runtime lease is stale".to_owned(),
            ));
        }
        self.transport.request(request)
    }

    pub async fn execute_provider_command_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        command: TelegramProviderCommand,
    ) -> Result<TelegramOperation, TelegramDurableExecutionError>
    where
        T: TdlibTransport,
    {
        validate_provider_command(&command).map_err(|_| TelegramDurableExecutionError::Provider)?;
        let account_id = provider_command_account_id(&command);
        let account = self
            .persistence
            .account(account_id)
            .ok_or(TelegramDurableExecutionError::Provider)?;
        if account.runtime_state != TelegramRuntimeState::Running {
            return Err(TelegramDurableExecutionError::Provider);
        }
        let lease_epoch = self
            .persistence
            .runtime_lease(account_id)
            .filter(|lease| lease.state == TelegramRuntimeLeaseState::Active)
            .ok_or(TelegramDurableExecutionError::Provider)?
            .epoch;
        let operation_id = provider_command_operation_id(&command).to_owned();
        if let Some(operation) = self.persistence.operation(&operation_id).cloned() {
            return Ok(operation);
        }
        let accepted = accept_operation(&command, lease_epoch);
        if !durable
            .insert_operation(&accepted, &command)
            .await
            .map_err(TelegramDurableExecutionError::Persistence)?
        {
            return Ok(accepted);
        }
        self.persistence.put_operation_if_absent(accepted.clone());
        self.persistence.put_command(command);
        Ok(accepted)
    }

    pub fn retry_operation(
        &mut self,
        operation_id: &str,
        now_unix_seconds: u64,
        next_attempt_at_unix_seconds: u64,
    ) -> Result<TelegramOperation, TdlibError> {
        let operation = self
            .persistence
            .operation(operation_id)
            .cloned()
            .ok_or_else(|| TdlibError::Protocol("Telegram operation is unknown".to_owned()))?;
        if !matches!(
            operation.state,
            makosh_telegram_api::TelegramOperationState::Failed
                | makosh_telegram_api::TelegramOperationState::DeadLetter
                | makosh_telegram_api::TelegramOperationState::RetryScheduled
        ) {
            return Err(TdlibError::Protocol(
                "Telegram operation is not retryable".to_owned(),
            ));
        }
        if !self.persistence.schedule_operation_retry(
            operation_id,
            now_unix_seconds,
            next_attempt_at_unix_seconds,
            "manual retry",
        ) {
            return Err(TdlibError::Protocol(
                "Telegram operation retry was not admitted".to_owned(),
            ));
        }
        self.persistence
            .operation(operation_id)
            .cloned()
            .ok_or_else(|| TdlibError::Protocol("Telegram operation disappeared".to_owned()))
    }

    pub async fn retry_operation_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        operation_id: &str,
        now_unix_seconds: u64,
        next_attempt_at_unix_seconds: u64,
    ) -> Result<TelegramOperation, TelegramDurableExecutionError> {
        if self.persistence.operation(operation_id).is_none() {
            if let Some(operation) = durable
                .operation(operation_id)
                .await
                .map_err(TelegramDurableExecutionError::Persistence)?
            {
                self.persistence.put_operation(operation);
            }
        }
        let operation = self
            .retry_operation(operation_id, now_unix_seconds, next_attempt_at_unix_seconds)
            .map_err(|_| TelegramDurableExecutionError::Provider)?;
        durable
            .save_operation(&operation)
            .await
            .map_err(TelegramDurableExecutionError::Persistence)?;
        Ok(operation)
    }

    pub fn execute_provider_query(
        &mut self,
        query: TelegramProviderQuery,
    ) -> Result<TelegramProviderQueryResponse, TdlibError> {
        validate_provider_query(&query)
            .map_err(|_| TdlibError::Protocol("Telegram provider query is invalid".to_owned()))?;
        let account_id = provider_query_account_id(&query).to_owned();
        match query {
            TelegramProviderQuery::LoadChats { limit, .. } => self
                .load_chats(&account_id, limit)
                .map(TelegramProviderQueryResponse::Chats),
            TelegramProviderQuery::Chat {
                provider_chat_id, ..
            } => Ok(TelegramProviderQueryResponse::Chat(
                self.persistence
                    .chat(&account_id, &provider_chat_id)
                    .cloned(),
            )),
            TelegramProviderQuery::ChatAvatar {
                provider_chat_id, ..
            } => Ok(TelegramProviderQueryResponse::ChatAvatar(
                self.persistence
                    .chat_avatar(&account_id, &provider_chat_id)
                    .cloned(),
            )),
            TelegramProviderQuery::LoadHistory {
                provider_chat_id,
                from_message_id,
                mode,
                limit,
                ..
            } => self
                .load_history_with_options(
                    &account_id,
                    &provider_chat_id,
                    from_message_id,
                    mode,
                    limit,
                )
                .map(TelegramProviderQueryResponse::HistoryPage),
            TelegramProviderQuery::CachedChats { limit, .. } => Ok(
                TelegramProviderQueryResponse::Chats(self.cached_chats(&account_id, limit)),
            ),
            TelegramProviderQuery::SearchChats { query, limit, .. } => {
                Ok(TelegramProviderQueryResponse::Chats(
                    self.persistence.search_chats(&account_id, &query, limit),
                ))
            }
            TelegramProviderQuery::CachedMessages {
                provider_chat_id,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::CachedMessages(
                self.cached_messages(&account_id, &provider_chat_id, limit),
            )),
            TelegramProviderQuery::MessageById { message_id, .. } => {
                Ok(TelegramProviderQueryResponse::CachedMessages(
                    self.persistence
                        .message(&message_id)
                        .cloned()
                        .into_iter()
                        .collect(),
                ))
            }
            TelegramProviderQuery::RecentMessages {
                provider_chat_id,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::CachedMessages(
                self.persistence
                    .recent_messages(&account_id, provider_chat_id.as_deref(), limit),
            )),
            TelegramProviderQuery::MessagesByIds { message_ids, .. } => {
                Ok(TelegramProviderQueryResponse::CachedMessages(
                    self.persistence.messages_by_ids(&message_ids),
                ))
            }
            TelegramProviderQuery::MessageVersions { message_id, .. } => {
                Ok(TelegramProviderQueryResponse::MessageVersions(
                    self.persistence
                        .message_versions(&message_id)
                        .map(|versions| versions.to_vec())
                        .unwrap_or_default(),
                ))
            }
            TelegramProviderQuery::MessageTombstones { message_id, .. } => {
                Ok(TelegramProviderQueryResponse::MessageTombstones(
                    self.persistence
                        .message_tombstones(&message_id)
                        .map(|tombstones| tombstones.to_vec())
                        .unwrap_or_default(),
                ))
            }
            TelegramProviderQuery::MessageMutations { message_id, .. } => {
                Ok(TelegramProviderQueryResponse::MessageMutations(
                    self.persistence
                        .message_mutations(&message_id)
                        .map(|mutations| mutations.to_vec())
                        .unwrap_or_default(),
                ))
            }
            TelegramProviderQuery::MessageReferences { message_id, .. } => {
                Ok(TelegramProviderQueryResponse::MessageReferences(
                    self.persistence.message_references(&message_id),
                ))
            }
            TelegramProviderQuery::ReplyChain {
                provider_chat_id,
                provider_message_id,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::ReplyChain(
                self.persistence.reply_chain(
                    &account_id,
                    &provider_chat_id,
                    &provider_message_id,
                    limit,
                ),
            )),
            TelegramProviderQuery::ForwardChain {
                provider_chat_id,
                provider_message_id,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::ForwardChain(
                self.persistence.forward_chain(
                    &account_id,
                    &provider_chat_id,
                    &provider_message_id,
                    limit,
                ),
            )),
            TelegramProviderQuery::Attachment { attachment_id, .. } => {
                Ok(TelegramProviderQueryResponse::Attachment(
                    self.persistence.attachment(&attachment_id).cloned(),
                ))
            }
            TelegramProviderQuery::AttachmentForMessage {
                provider_chat_id,
                provider_message_id,
                ..
            } => Ok(TelegramProviderQueryResponse::Attachment(
                self.persistence.attachment_for_message(
                    &account_id,
                    &provider_chat_id,
                    &provider_message_id,
                ),
            )),
            TelegramProviderQuery::File {
                provider_file_id, ..
            } => Ok(TelegramProviderQueryResponse::File(
                self.persistence
                    .file(&account_id, &provider_file_id)
                    .cloned(),
            )),
            TelegramProviderQuery::ChatState {
                provider_chat_id, ..
            } => Ok(TelegramProviderQueryResponse::ChatState(
                self.persistence
                    .chat_state(&account_id, &provider_chat_id)
                    .cloned(),
            )),
            TelegramProviderQuery::PinnedMessages {
                provider_chat_id,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::CachedMessages(
                self.persistence
                    .pinned_messages(&account_id, &provider_chat_id, limit),
            )),
            TelegramProviderQuery::SearchMessages {
                provider_chat_id,
                query,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::CachedMessages(
                self.persistence.search_messages(
                    &account_id,
                    provider_chat_id.as_deref(),
                    &query,
                    limit,
                ),
            )),
            TelegramProviderQuery::ListParticipants {
                provider_chat_id,
                filter,
                offset,
                limit,
                ..
            } => self
                .list_participants(&account_id, &provider_chat_id, filter, offset, limit)
                .map(TelegramProviderQueryResponse::Participants),
            TelegramProviderQuery::ListTopics {
                provider_chat_id,
                limit,
                ..
            } => self
                .list_topics(&account_id, &provider_chat_id, limit)
                .map(TelegramProviderQueryResponse::Topics),
            TelegramProviderQuery::Topic {
                provider_chat_id,
                provider_topic_id,
                ..
            } => Ok(TelegramProviderQueryResponse::Topic(
                self.persistence
                    .topics_for_chat(&account_id, &provider_chat_id, u32::MAX)
                    .into_iter()
                    .find(|topic| topic.provider_topic_id == provider_topic_id),
            )),
            TelegramProviderQuery::TopicMessageIds {
                provider_chat_id,
                provider_topic_id,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::TopicMessageIds(
                self.persistence.message_ids_for_topic(
                    &account_id,
                    &provider_chat_id,
                    &provider_topic_id,
                    limit,
                ),
            )),
            TelegramProviderQuery::SearchTopics {
                provider_chat_id,
                query,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::Topics(
                self.persistence
                    .search_topics(&account_id, &provider_chat_id, &query, limit),
            )),
            TelegramProviderQuery::Reactions {
                provider_chat_id,
                provider_message_id,
                ..
            } => Ok(TelegramProviderQueryResponse::Reactions(
                self.cached_reactions(&account_id, &provider_chat_id, &provider_message_id),
            )),
            TelegramProviderQuery::ReactionSummary {
                provider_chat_id,
                provider_message_id,
                ..
            } => Ok(TelegramProviderQueryResponse::ReactionSummary(
                self.persistence.reaction_summary(&format!(
                    "telegram:{account_id}:{provider_chat_id}:{provider_message_id}"
                )),
            )),
            TelegramProviderQuery::ChatPositions {
                provider_chat_id, ..
            } => Ok(TelegramProviderQueryResponse::ChatPositions(
                self.persistence
                    .chat_positions(&account_id, &provider_chat_id),
            )),
            TelegramProviderQuery::ChatOperationalState {
                provider_chat_id, ..
            } => Ok(TelegramProviderQueryResponse::ChatOperationalState(
                self.persistence
                    .chat_operational_state(&account_id, &provider_chat_id)
                    .cloned(),
            )),
            TelegramProviderQuery::BasicGroupParticipants {
                provider_chat_id,
                basic_group_id,
                ..
            } => self
                .list_basic_group_participants(&account_id, &provider_chat_id, basic_group_id)
                .map(TelegramProviderQueryResponse::Participants),
            TelegramProviderQuery::ChatFolder {
                provider_folder_id, ..
            } => {
                let cached = self
                    .persistence
                    .chat_folders(&account_id)
                    .into_iter()
                    .filter(|folder| folder.provider_folder_id == provider_folder_id)
                    .collect::<Vec<_>>();
                if !cached.is_empty() {
                    Ok(TelegramProviderQueryResponse::ChatFolders(cached))
                } else {
                    match self.execute(
                        &account_id,
                        TdlibRequest::GetChatFolder {
                            account_id: account_id.clone(),
                            provider_folder_id,
                        },
                    )? {
                        TdlibResponse::ChatFolders(folders) => {
                            self.persistence.put_chat_folders(folders.clone());
                            Ok(TelegramProviderQueryResponse::ChatFolders(folders))
                        }
                        _ => Err(TdlibError::Protocol(
                            "TDLib did not return a chat folder".to_owned(),
                        )),
                    }
                }
            }
            TelegramProviderQuery::ChatFolders {
                provider_folder_ids,
                ..
            } => {
                let mut folders = Vec::new();
                for provider_folder_id in provider_folder_ids {
                    let cached = self
                        .persistence
                        .chat_folders(&account_id)
                        .into_iter()
                        .find(|folder| folder.provider_folder_id == provider_folder_id);
                    if let Some(folder) = cached {
                        folders.push(folder);
                        continue;
                    }
                    match self.execute(
                        &account_id,
                        TdlibRequest::GetChatFolder {
                            account_id: account_id.clone(),
                            provider_folder_id,
                        },
                    )? {
                        TdlibResponse::ChatFolders(mut loaded) => {
                            self.persistence.put_chat_folders(loaded.clone());
                            folders.append(&mut loaded);
                        }
                        _ => {
                            return Err(TdlibError::Protocol(
                                "TDLib did not return chat folders".to_owned(),
                            ));
                        }
                    }
                }
                Ok(TelegramProviderQueryResponse::ChatFolders(folders))
            }
            TelegramProviderQuery::Operations { limit, .. } => {
                let mut operations = self.persistence.operations_for_account(&account_id);
                operations.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
                operations.truncate(limit as usize);
                Ok(TelegramProviderQueryResponse::Operations(operations))
            }
            TelegramProviderQuery::Commands {
                provider_chat_id,
                provider_message_id,
                command_kinds,
                limit,
                ..
            } => Ok(TelegramProviderQueryResponse::Commands(
                self.persistence.command_records_for_account(
                    &account_id,
                    provider_chat_id.as_deref(),
                    provider_message_id.as_deref(),
                    &command_kinds,
                    limit,
                ),
            )),
        }
    }

    fn apply_local_command(&mut self, command: &TelegramProviderCommand) -> bool {
        let TelegramProviderCommand::RestoreVisibility {
            operation_id,
            account_id,
            provider_chat_id,
            provider_message_id,
            ..
        } = command
        else {
            return false;
        };
        let message_id = format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}");
        let _ = self
            .persistence
            .append_message_tombstone(TelegramMessageTombstone {
                tombstone_id: format!("{message_id}:visibility-restored:{operation_id}"),
                message_id,
                account_id: account_id.clone(),
                provider_chat_id: provider_chat_id.clone(),
                provider_message_id: provider_message_id.clone(),
                reason: TelegramTombstoneReason::Unknown,
                observed_at_unix_seconds: 0,
                is_provider_delete: false,
                is_locally_visible: true,
            });
        true
    }

    pub async fn execute_due_durable_operations(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        now_unix_seconds: u64,
        limit: i64,
        worker_id: &str,
        mut issue_media_session: impl FnMut(
            &makosh_telegram_api::TelegramBlobIntentV1,
        ) -> Result<
            makosh_blob_client::ManagedBlobSessionV1,
            TdlibError,
        >,
    ) -> Result<Vec<TelegramOperation>, TelegramDurableExecutionError> {
        let claimed = durable
            .claim_due_operations(account_id, now_unix_seconds, limit, worker_id)
            .await
            .map_err(TelegramDurableExecutionError::Persistence)?;
        let mut completed = Vec::with_capacity(claimed.len());
        for (operation, command) in claimed {
            let current_epoch = self
                .persistence
                .runtime_lease(account_id)
                .filter(|lease| lease.state == TelegramRuntimeLeaseState::Active)
                .map(|lease| lease.epoch);
            let mut next_operation = if current_epoch != Some(operation.lease_epoch) {
                operation_failed(&operation, "Telegram runtime lease is stale")
            } else {
                if self.apply_local_command(&command) {
                    operation_completed(&operation)
                } else {
                    let response = match command.clone() {
                        TelegramProviderCommand::SendMedia(media) => {
                            let materialized_path = issue_media_session(&media.blob)
                                .and_then(|session| {
                                    self.authorize_media_session(session, &media.blob)
                                })
                                .and_then(|()| {
                                    self.media_materializer
                                        .as_mut()
                                        .ok_or_else(|| {
                                            TdlibError::Protocol(
                                                "Telegram Blob materializer is unavailable"
                                                    .to_owned(),
                                            )
                                        })?
                                        .materialize(&media.blob.blob_ref)
                                });
                            match materialized_path {
                                Ok(path) => {
                                    let response = self.transport.request(
                                        TdlibRequest::SendMediaMaterialized {
                                            command: media,
                                            materialized_path: path.clone(),
                                        },
                                    );
                                    if let Some(materializer) = self.media_materializer.as_mut() {
                                        materializer.release(&path);
                                    }
                                    response
                                }
                                Err(_) => Err(TdlibError::Protocol(
                                    "Telegram Blob session is unavailable".to_owned(),
                                )),
                            }
                        }
                        command => self
                            .transport
                            .request(TdlibRequest::ProviderCommand(command)),
                    };
                    match response {
                        Ok(TdlibResponse::Accepted { .. }) => {
                            operation_awaiting_provider(&operation)
                        }
                        Ok(TdlibResponse::History(messages)) => {
                            for message in messages {
                                self.ingest_message(message)
                                    .map_err(|_| TelegramDurableExecutionError::Provider)?;
                            }
                            operation_completed(&operation)
                        }
                        Ok(_) => operation_completed(&operation),
                        Err(_) => operation_retry_scheduled(
                            &operation,
                            now_unix_seconds.saturating_add(1),
                            "Telegram provider execution failed",
                        ),
                    }
                }
            };
            next_operation.provider_observed_at_unix_seconds = (next_operation.state
                == makosh_telegram_api::TelegramOperationState::Completed)
                .then_some(now_unix_seconds);
            durable
                .save_operation(&next_operation)
                .await
                .map_err(TelegramDurableExecutionError::Persistence)?;
            self.persistence.put_operation(next_operation.clone());
            self.persistence.put_command(command);
            completed.push(next_operation);
        }
        Ok(completed)
    }

    pub fn download_file(
        &mut self,
        request: TelegramDownloadFile,
    ) -> Result<TelegramFileSnapshot, TdlibError> {
        let account_id = request.account_id.clone();
        let response = self.execute(&account_id, TdlibRequest::DownloadFile(request))?;
        match response {
            TdlibResponse::File(file) => {
                self.persistence.put_file(file.clone());
                self.persistence
                    .apply_file_to_attachments(&account_id, &file);
                Ok(file)
            }
            _ => Err(TdlibError::Protocol(
                "TDLib did not return a file snapshot".to_owned(),
            )),
        }
    }

    pub async fn download_file_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        request: TelegramDownloadFile,
    ) -> Result<TelegramFileSnapshot, TelegramDurableProjectionError> {
        let file = self
            .download_file(request)
            .map_err(TelegramDurableProjectionError::Provider)?;
        durable
            .upsert_file(&file)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        Ok(file)
    }

    pub fn load_chats(
        &mut self,
        account_id: &str,
        limit: u32,
    ) -> Result<Vec<makosh_telegram_api::TelegramChat>, TdlibError> {
        let response = self.execute(account_id, get_chats_request(account_id, limit)?)?;
        match response {
            TdlibResponse::Chats(chats) => {
                for chat in &chats {
                    self.persistence.put_chat(chat.clone());
                }
                Ok(chats)
            }
            _ => Err(TdlibError::Protocol(
                "TDLib did not return chats".to_owned(),
            )),
        }
    }

    pub async fn load_chats_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        limit: u32,
    ) -> Result<Vec<makosh_telegram_api::TelegramChat>, TelegramDurableProjectionError> {
        let chats = self
            .load_chats(account_id, limit)
            .map_err(TelegramDurableProjectionError::Provider)?;
        for chat in &chats {
            durable
                .upsert_chat(chat)
                .await
                .map_err(TelegramDurableProjectionError::Persistence)?;
        }
        Ok(chats)
    }

    pub fn load_history(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        limit: u32,
    ) -> Result<Vec<TelegramMessageObservation>, TdlibError> {
        let response = self.execute(
            account_id,
            get_history_request(account_id, provider_chat_id, limit)?,
        )?;
        match response {
            TdlibResponse::History(messages) => {
                for message in &messages {
                    self.ingest_message(message.clone()).map_err(|_| {
                        TdlibError::Protocol("Telegram history projection is invalid".to_owned())
                    })?;
                }
                Ok(messages)
            }
            _ => Err(TdlibError::Protocol(
                "TDLib did not return history".to_owned(),
            )),
        }
    }

    pub fn load_history_with_options(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        from_message_id: Option<i64>,
        mode: makosh_telegram_api::TelegramHistorySyncMode,
        limit: u32,
    ) -> Result<makosh_telegram_api::TelegramHistoryPage, TdlibError> {
        let mut cursor = from_message_id;
        let mut observations = Vec::new();
        loop {
            let response = self.execute(
                account_id,
                get_history_request_with_options(
                    account_id,
                    provider_chat_id,
                    cursor,
                    mode,
                    limit,
                )?,
            )?;
            let page = match response {
                TdlibResponse::History(messages) => messages,
                _ => {
                    return Err(TdlibError::Protocol(
                        "TDLib did not return history".to_owned(),
                    ));
                }
            };
            let page_len = page.len();
            let next_cursor = page
                .last()
                .and_then(|message| message.provider_message_id.parse::<i64>().ok());
            for message in &page {
                self.ingest_message(message.clone()).map_err(|_| {
                    TdlibError::Protocol("Telegram history projection is invalid".to_owned())
                })?;
            }
            observations.extend(page);
            if !matches!(mode, makosh_telegram_api::TelegramHistorySyncMode::Full)
                || page_len < limit as usize
                || next_cursor.is_none()
                || next_cursor == cursor
            {
                break;
            }
            cursor = next_cursor;
        }
        let next_from_message_id = observations
            .last()
            .and_then(|message| message.provider_message_id.parse::<i64>().ok());
        let has_more = !matches!(mode, makosh_telegram_api::TelegramHistorySyncMode::Full)
            && observations.len() >= limit as usize
            && next_from_message_id.is_some();
        Ok(makosh_telegram_api::TelegramHistoryPage {
            items: observations,
            next_from_message_id,
            has_more,
        })
    }

    pub async fn load_history_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        provider_chat_id: &str,
        limit: u32,
    ) -> Result<Vec<TelegramMessageObservation>, TelegramDurableProjectionError> {
        let observations = self
            .load_history(account_id, provider_chat_id, limit)
            .map_err(TelegramDurableProjectionError::Provider)?;
        for observation in &observations {
            let projection =
                project_message(observation).map_err(TelegramDurableProjectionError::Contract)?;
            durable
                .upsert_message(&projection)
                .await
                .map_err(TelegramDurableProjectionError::Persistence)?;
        }
        Ok(observations)
    }

    pub async fn restore_projections_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        provider_chat_id: &str,
        chat_limit: i64,
        message_limit: i64,
    ) -> Result<(usize, usize), TelegramDurableProjectionError> {
        let chats = durable
            .list_chats(account_id, chat_limit)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        for chat in &chats {
            self.persistence.put_chat(chat.clone());
        }
        for avatar in durable
            .list_chat_avatars(account_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?
        {
            self.persistence.put_chat_avatar(avatar);
        }
        let folders = durable
            .list_chat_folders(account_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        self.persistence.put_chat_folders(folders);
        let positions = durable
            .list_chat_positions(account_id, provider_chat_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        for position in positions {
            self.persistence.put_chat_position(position);
        }
        if let Some(state) = durable
            .chat_operational_state(account_id, provider_chat_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?
        {
            self.persistence
                .put_chat_operational_state(account_id, provider_chat_id, state);
        }
        let messages = durable
            .list_messages(account_id, provider_chat_id, message_limit)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        for message in &messages {
            self.persistence.put_message(message.clone());
        }
        if let Some(state) = durable
            .chat_state(account_id, provider_chat_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?
        {
            self.persistence
                .apply_chat_state(account_id, provider_chat_id, state);
        }
        Ok((chats.len(), messages.len()))
    }

    pub async fn restore_account_state_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        chat_limit: i64,
    ) -> Result<usize, TelegramDurableProjectionError> {
        self.restore_account_durable(durable, account_id)
            .await
            .map_err(|error| match error {
                TelegramDurableLifecycleError::Contract(error) => {
                    TelegramDurableProjectionError::Contract(error)
                }
                TelegramDurableLifecycleError::Persistence(error) => {
                    TelegramDurableProjectionError::Persistence(error)
                }
            })?
            .ok_or(TelegramDurableProjectionError::Contract(
                TelegramContractError::AccountUnknown,
            ))?;
        let restored = durable_restore::restore_account_projection_cache(
            &mut self.persistence,
            durable,
            account_id,
            chat_limit,
        )
        .await
        .map_err(TelegramDurableProjectionError::Persistence)?;
        let account = self
            .persistence
            .account(account_id)
            .ok_or(TelegramDurableProjectionError::Contract(
                TelegramContractError::AccountUnknown,
            ))?
            .clone();
        if account.state == TelegramAccountState::Retired || account.runtime_epoch == 0 {
            return Err(TelegramDurableProjectionError::Contract(
                TelegramContractError::RuntimeBlocked,
            ));
        }
        if !matches!(
            account.runtime_state,
            TelegramRuntimeState::Running | TelegramRuntimeState::Degraded
        ) {
            let running = self.lifecycle.mark_running(&account);
            let credentials = self
                .persistence
                .credentials(account_id)
                .unwrap_or_default()
                .to_vec();
            durable
                .upsert_account(&running, &credentials)
                .await
                .map_err(TelegramDurableProjectionError::Persistence)?;
            self.persistence.put_account(running);
        }
        self.install_admitted_runtime_lease(account_id)
            .map_err(TelegramDurableProjectionError::Contract)?;
        Ok(restored)
    }

    pub async fn restore_message_lifecycle_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        message_id: &str,
    ) -> Result<(usize, usize, usize), TelegramDurableProjectionError> {
        let versions = durable
            .list_message_versions(message_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        for version in &versions {
            self.persistence.append_message_version(version.clone());
        }
        let tombstones = durable
            .list_tombstones(message_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        for tombstone in &tombstones {
            self.persistence.append_message_tombstone(tombstone.clone());
        }
        let mutations = durable
            .message_mutations(message_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        if let Some((account_id, provider_chat_id, provider_message_id)) = versions
            .first()
            .map(|version| {
                (
                    version.account_id.clone(),
                    version.provider_chat_id.clone(),
                    version.provider_message_id.clone(),
                )
            })
            .or_else(|| {
                tombstones.first().map(|tombstone| {
                    (
                        tombstone.account_id.clone(),
                        tombstone.provider_chat_id.clone(),
                        tombstone.provider_message_id.clone(),
                    )
                })
            })
        {
            for mutation in &mutations {
                self.persistence.apply_message_mutation(
                    &account_id,
                    &provider_chat_id,
                    &provider_message_id,
                    mutation.clone(),
                );
            }
        }
        let reactions = durable
            .reactions(message_id)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        if !reactions.is_empty() {
            self.persistence
                .replace_reactions(message_id, reactions.clone());
        }
        Ok((versions.len(), tombstones.len(), reactions.len()))
    }

    pub fn cached_chats(
        &self,
        account_id: &str,
        limit: u32,
    ) -> Vec<makosh_telegram_api::TelegramChat> {
        self.persistence.chats_for_account(account_id, limit)
    }

    pub fn accounts(&self) -> Vec<TelegramAccount> {
        self.persistence.accounts()
    }

    pub fn account(&self, account_id: &str) -> Option<TelegramAccount> {
        self.persistence.account(account_id).cloned()
    }

    pub fn operations_for_account(
        &self,
        account_id: &str,
    ) -> Vec<makosh_telegram_api::TelegramOperation> {
        self.persistence.operations_for_account(account_id)
    }

    pub fn cached_messages(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        limit: u32,
    ) -> Vec<makosh_telegram_api::TelegramMessageProjection> {
        self.persistence
            .messages_for_chat(account_id, provider_chat_id, limit)
    }

    pub fn cached_reactions(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Vec<makosh_telegram_api::TelegramReactionObservation> {
        self.persistence
            .reactions(&format!(
                "telegram:{account_id}:{provider_chat_id}:{provider_message_id}"
            ))
            .map(|reactions| reactions.to_vec())
            .unwrap_or_default()
    }

    pub fn ingest_file_update(
        &mut self,
        account_id: &str,
        payload: &serde_json::Value,
    ) -> Result<TelegramFileSnapshot, TdlibError> {
        let file = parse_file_snapshot(account_id, payload)?;
        self.persistence.put_file(file.clone());
        Ok(file)
    }

    pub fn list_participants(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        filter: TelegramParticipantFilter,
        offset: u32,
        limit: u32,
    ) -> Result<TelegramParticipantPage, TdlibError> {
        let response = self.execute(
            account_id,
            TdlibRequest::ListParticipants {
                account_id: account_id.to_owned(),
                provider_chat_id: provider_chat_id.to_owned(),
                filter,
                offset,
                limit,
            },
        )?;
        match response {
            TdlibResponse::Participants(page) => {
                self.persistence.put_participants(&page);
                Ok(page)
            }
            _ => Err(TdlibError::Protocol(
                "TDLib did not return participants".to_owned(),
            )),
        }
    }

    pub async fn list_participants_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        provider_chat_id: &str,
        filter: TelegramParticipantFilter,
        offset: u32,
        limit: u32,
    ) -> Result<TelegramParticipantPage, TelegramDurableProjectionError> {
        let page = self
            .list_participants(account_id, provider_chat_id, filter, offset, limit)
            .map_err(TelegramDurableProjectionError::Provider)?;
        durable
            .upsert_participants(&page)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        Ok(page)
    }

    pub async fn restore_participants_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        provider_chat_id: &str,
        filter: TelegramParticipantFilter,
    ) -> Result<Option<TelegramParticipantPage>, TelegramDurableProjectionError> {
        let page = durable
            .participants(account_id, provider_chat_id, filter)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        if let Some(page) = &page {
            self.persistence.put_participants(page);
        }
        Ok(page)
    }

    pub fn list_basic_group_participants(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        basic_group_id: i64,
    ) -> Result<TelegramParticipantPage, TdlibError> {
        let response = self.execute(
            account_id,
            TdlibRequest::ListBasicGroupParticipants {
                account_id: account_id.to_owned(),
                provider_chat_id: provider_chat_id.to_owned(),
                basic_group_id,
            },
        )?;
        match response {
            TdlibResponse::Participants(page) => {
                self.persistence.put_participants(&page);
                Ok(page)
            }
            _ => Err(TdlibError::Protocol(
                "TDLib did not return basic-group participants".to_owned(),
            )),
        }
    }

    pub fn list_all_participants(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        filter: TelegramParticipantFilter,
        limit: u32,
    ) -> Result<TelegramParticipantPage, TdlibError> {
        if limit == 0 {
            return Err(TdlibError::Protocol(
                "participant limit must be positive".to_owned(),
            ));
        }
        let mut offset = 0;
        let mut items = Vec::new();
        while items.len() < limit as usize {
            let page_limit = (limit as usize - items.len()).min(100) as u32;
            let page =
                self.list_participants(account_id, provider_chat_id, filter, offset, page_limit)?;
            let page_len = page.items.len();
            items.extend(page.items);
            if page_len == 0 || page.next_offset.is_none() || page_len < page_limit as usize {
                break;
            }
            offset += page_len as u32;
        }
        Ok(TelegramParticipantPage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            filter,
            next_offset: (items.len() < limit as usize).then_some(offset),
            items,
        })
    }

    pub fn list_topics(
        &mut self,
        account_id: &str,
        provider_chat_id: &str,
        limit: u32,
    ) -> Result<Vec<TelegramTopic>, TdlibError> {
        let response = self.execute(
            account_id,
            TdlibRequest::ListTopics {
                account_id: account_id.to_owned(),
                provider_chat_id: provider_chat_id.to_owned(),
                limit,
            },
        )?;
        match response {
            TdlibResponse::Topics(topics) => {
                for topic in &topics {
                    self.persistence.put_topic(topic.clone());
                }
                Ok(topics)
            }
            _ => Err(TdlibError::Protocol(
                "TDLib did not return topics".to_owned(),
            )),
        }
    }

    pub async fn list_topics_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        provider_chat_id: &str,
        limit: u32,
    ) -> Result<Vec<TelegramTopic>, TelegramDurableProjectionError> {
        let topics = self
            .list_topics(account_id, provider_chat_id, limit)
            .map_err(TelegramDurableProjectionError::Provider)?;
        for topic in &topics {
            durable
                .upsert_topic(topic)
                .await
                .map_err(TelegramDurableProjectionError::Persistence)?;
        }
        Ok(topics)
    }

    pub async fn restore_topics_durable(
        &mut self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        provider_chat_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramTopic>, TelegramDurableProjectionError> {
        let topics = durable
            .list_topics(account_id, provider_chat_id, limit)
            .await
            .map_err(TelegramDurableProjectionError::Persistence)?;
        for topic in &topics {
            self.persistence.put_topic(topic.clone());
        }
        Ok(topics)
    }

    pub fn parse_topics_response(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        payload: &serde_json::Value,
    ) -> Result<Vec<TelegramTopic>, TdlibError> {
        parse_topic_list(account_id, provider_chat_id, payload)
    }

    pub fn ingest_realtime_update(
        &mut self,
        account_id: &str,
        provider_cursor: Option<String>,
        payload: &serde_json::Value,
    ) -> Result<Vec<TelegramRealtimeFrame>, TelegramContractError> {
        let events = parse_provider_events(account_id, payload)
            .map_err(|_| TelegramContractError::InvalidTransition)?;
        let mut frames = Vec::with_capacity(events.len());
        for event in events {
            let frame = TelegramRealtimeFrame {
                account_id: account_id.to_owned(),
                sequence: self.persistence.next_realtime_sequence(account_id),
                provider_cursor: provider_cursor.clone(),
                event,
            };
            self.persistence.append_realtime_frame(frame.clone());
            self.apply_provider_event(frame.event.clone())?;
            frames.push(frame);
        }
        Ok(frames)
    }

    pub fn poll_provider_events(
        &mut self,
        account_id: &str,
        provider_cursor: Option<String>,
    ) -> Result<TelegramProviderPollBatch, TdlibError> {
        let updates = self.transport.poll_updates()?;
        let mut frames = Vec::with_capacity(updates.len());
        let mut call_updates = Vec::new();
        for update in updates {
            match update {
                TdlibProviderUpdate::Operational(event) => {
                    if makosh_telegram_api::provider_event_account_id(&event) != account_id {
                        return Err(TdlibError::Protocol(
                            "Telegram provider event belongs to another account".to_owned(),
                        ));
                    }
                    let frame = TelegramRealtimeFrame {
                        account_id: account_id.to_owned(),
                        sequence: self.persistence.next_realtime_sequence(account_id),
                        provider_cursor: provider_cursor.clone(),
                        event: *event,
                    };
                    self.persistence.append_realtime_frame(frame.clone());
                    self.apply_provider_event(frame.event.clone())
                        .map_err(|_| {
                            TdlibError::Protocol("Telegram provider event is invalid".to_owned())
                        })?;
                    frames.push(frame);
                }
                TdlibProviderUpdate::Call(observation) => {
                    if observation.account_id != account_id {
                        return Err(TdlibError::Protocol(
                            "Telegram call update belongs to another account".to_owned(),
                        ));
                    }
                    call_updates.push(TelegramCallProviderUpdate::Observation(observation));
                }
                TdlibProviderUpdate::CallReady {
                    observation,
                    material,
                } => {
                    if observation.account_id != account_id {
                        return Err(TdlibError::Protocol(
                            "Telegram call update belongs to another account".to_owned(),
                        ));
                    }
                    call_updates.push(TelegramCallProviderUpdate::Ready {
                        observation,
                        material,
                    });
                }
                TdlibProviderUpdate::CallSignaling {
                    account_id: signaling_account_id,
                    tdlib_call_id,
                    data,
                } => {
                    if signaling_account_id != account_id {
                        return Err(TdlibError::Protocol(
                            "Telegram call signaling belongs to another account".to_owned(),
                        ));
                    }
                    call_updates.push(TelegramCallProviderUpdate::Signaling {
                        account_id: signaling_account_id,
                        tdlib_call_id,
                        data,
                    });
                }
            }
        }
        Ok(TelegramProviderPollBatch {
            frames,
            call_updates,
        })
    }

    pub fn realtime_after(&self, account_id: &str, sequence: u64) -> Vec<TelegramRealtimeFrame> {
        self.persistence.realtime_after(account_id, sequence)
    }

    pub fn accept_send(
        &mut self,
        command: TelegramSendMessage,
    ) -> Result<TelegramOperation, TelegramContractError> {
        let account_id = command.account_id.clone();
        self.persistence
            .account(&account_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        let provider_command = TelegramProviderCommand::SendText(command.clone());
        let lease_epoch = self
            .persistence
            .runtime_lease(&command.account_id)
            .map_or(0, |lease| lease.epoch);
        if let Some(existing) = self.persistence.operation(&command.operation_id) {
            return Ok(existing.clone());
        }
        let operation = accept_operation(&provider_command, lease_epoch);
        self.persistence.put_operation_if_absent(operation.clone());
        self.persistence.put_command(provider_command);
        Ok(operation)
    }

    pub fn map_observation(
        &self,
        observation: makosh_telegram_api::TelegramMessageObservation,
    ) -> Result<CommunicationObservationDraft, TelegramContractError> {
        observation_draft(observation)
    }

    pub fn ingest_message(
        &mut self,
        observation: makosh_telegram_api::TelegramMessageObservation,
    ) -> Result<CommunicationObservationDraft, TelegramContractError> {
        let projection = project_message(&observation)?;
        let message_id = projection.message_id.clone();
        if self.persistence.message_versions(&message_id).is_none() {
            self.persistence
                .append_message_version(makosh_telegram_api::TelegramMessageVersion {
                    version_id: format!("{message_id}:version:1"),
                    message_id: message_id.clone(),
                    account_id: projection.account_id.clone(),
                    provider_chat_id: projection.provider_chat_id.clone(),
                    provider_message_id: projection.provider_message_id.clone(),
                    version_number: 1,
                    body_text: projection.text.clone(),
                    observed_at_unix_seconds: projection.observed_at_unix_seconds,
                    source: makosh_telegram_api::TelegramMessageVersionSource::Provider,
                });
        }
        self.persistence.put_message(projection);
        if let Some(media) = &observation.media {
            if let Some(provider_file_id) = &media.provider_file_id {
                self.track_attachment(makosh_telegram_api::TelegramAttachmentProjection {
                    attachment_id: format!(
                        "telegram:{}:{}:{}:{}",
                        observation.account_id,
                        observation.provider_chat_id,
                        observation.provider_message_id,
                        provider_file_id
                    ),
                    account_id: observation.account_id.clone(),
                    provider_chat_id: observation.provider_chat_id.clone(),
                    provider_message_id: observation.provider_message_id.clone(),
                    provider_file_id: provider_file_id.clone(),
                    state: makosh_telegram_api::TelegramAttachmentDownloadState::Pending,
                    size_bytes: None,
                    filename: media.filename.clone(),
                    content_type: media.content_type.clone(),
                    blob_ref: None,
                })?;
            }
        }
        let draft = observation_draft(observation)?;
        Ok(draft)
    }

    pub fn upsert_chat(&mut self, chat: makosh_telegram_api::TelegramChat) {
        self.persistence.put_chat(chat);
    }

    pub async fn persist_provider_frame_durable<F>(
        &mut self,
        durable: &TelegramDurablePersistence,
        frame: &TelegramRealtimeFrame,
        body_admitter: &mut F,
    ) -> Result<(), TelegramDurableProjectionError>
    where
        F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
    {
        match &frame.event {
            makosh_telegram_api::TelegramProviderEvent::ChatFoldersChanged { folders, .. } => {
                durable
                    .upsert_chat_folders(folders)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
            }
            makosh_telegram_api::TelegramProviderEvent::ChatPositionChanged(position) => {
                durable
                    .upsert_chat_position(position)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
            }
            makosh_telegram_api::TelegramProviderEvent::ChatAvatarChanged(avatar) => {
                durable
                    .upsert_chat_avatar(avatar)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
            }
            makosh_telegram_api::TelegramProviderEvent::ParticipantChanged(participant) => {
                durable
                    .upsert_participant(participant)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
            }
            makosh_telegram_api::TelegramProviderEvent::MessageCreated(observation) => {
                let projection = project_message(observation)
                    .map_err(TelegramDurableProjectionError::Contract)?;
                durable
                    .upsert_message(&projection)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
                if let Some(initial_version) = self
                    .persistence
                    .message_versions(&projection.message_id)
                    .and_then(|versions| {
                        versions.iter().find(|version| version.version_number == 1)
                    })
                {
                    durable
                        .upsert_message_version(initial_version)
                        .await
                        .map_err(TelegramDurableProjectionError::Persistence)?;
                }
                if let Some(media) = &observation.media {
                    if let Some(provider_file_id) = &media.provider_file_id {
                        let attachment = makosh_telegram_api::TelegramAttachmentProjection {
                            attachment_id: format!(
                                "telegram:{}:{}:{}:{}",
                                observation.account_id,
                                observation.provider_chat_id,
                                observation.provider_message_id,
                                provider_file_id
                            ),
                            account_id: observation.account_id.clone(),
                            provider_chat_id: observation.provider_chat_id.clone(),
                            provider_message_id: observation.provider_message_id.clone(),
                            provider_file_id: provider_file_id.clone(),
                            state: makosh_telegram_api::TelegramAttachmentDownloadState::Pending,
                            size_bytes: None,
                            filename: media.filename.clone(),
                            content_type: media.content_type.clone(),
                            blob_ref: None,
                        };
                        durable
                            .upsert_attachment(&attachment)
                            .await
                            .map_err(TelegramDurableProjectionError::Persistence)?;
                    }
                }
            }
            makosh_telegram_api::TelegramProviderEvent::FileChanged(file) => {
                durable
                    .upsert_file(file)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
                let attachments = durable
                    .apply_file_to_attachments(&file.account_id, file)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
                for attachment in attachments {
                    if let Some(draft) =
                        makosh_telegram_core::attachment_observation_draft(&attachment)
                            .map_err(TelegramDurableProjectionError::Contract)?
                    {
                        let (record, recorded_at_unix_seconds) = self
                            .communication_observation_record(&draft)
                            .map_err(TelegramDurableProjectionError::Contract)?;
                        durable
                            .enqueue_communications_outbox(&record, recorded_at_unix_seconds)
                            .await
                            .map_err(TelegramDurableProjectionError::Persistence)?;
                    }
                }
            }
            makosh_telegram_api::TelegramProviderEvent::TopicChanged(topic) => {
                durable
                    .upsert_topic(topic)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
            }
            makosh_telegram_api::TelegramProviderEvent::MessageEdited {
                account_id,
                provider_chat_id,
                provider_message_id,
                text,
                observed_at_unix_seconds,
            } => {
                let message_id =
                    format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}");
                if let Some(message) = self.persistence.message(&message_id) {
                    durable
                        .upsert_message(message)
                        .await
                        .map_err(TelegramDurableProjectionError::Persistence)?;
                }
                let version_number = self
                    .persistence
                    .next_message_version_number(&message_id)
                    .saturating_sub(1);
                if version_number > 0 {
                    durable
                        .upsert_message_version(&makosh_telegram_api::TelegramMessageVersion {
                            version_id: format!("{message_id}:version:{version_number}"),
                            message_id,
                            account_id: account_id.clone(),
                            provider_chat_id: provider_chat_id.clone(),
                            provider_message_id: provider_message_id.clone(),
                            version_number,
                            body_text: text.clone(),
                            observed_at_unix_seconds: *observed_at_unix_seconds,
                            source: makosh_telegram_api::TelegramMessageVersionSource::Provider,
                        })
                        .await
                        .map_err(TelegramDurableProjectionError::Persistence)?;
                }
            }
            makosh_telegram_api::TelegramProviderEvent::MessageDeleted {
                account_id,
                provider_chat_id,
                provider_message_id,
                is_permanent,
            } => {
                let message_id =
                    format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}");
                durable
                    .upsert_tombstone(&makosh_telegram_api::TelegramMessageTombstone {
                        tombstone_id: format!("{message_id}:tombstone:provider"),
                        message_id: message_id.clone(),
                        account_id: account_id.clone(),
                        provider_chat_id: provider_chat_id.clone(),
                        provider_message_id: provider_message_id.clone(),
                        reason: makosh_telegram_api::TelegramTombstoneReason::ProviderDeleted,
                        observed_at_unix_seconds: 0,
                        is_provider_delete: *is_permanent,
                        is_locally_visible: false,
                    })
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
            }
            makosh_telegram_api::TelegramProviderEvent::ReactionsObserved {
                account_id,
                provider_chat_id,
                provider_message_id,
                reactions,
            } => {
                durable
                    .replace_reactions(
                        &format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}"),
                        reactions,
                    )
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
            }
            _ => {}
        }
        if let Some((account_id, provider_chat_id, provider_message_id, _)) =
            event_message_mutation(&frame.event)
        {
            let message_id =
                format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}");
            let mutations = self
                .persistence
                .message_mutations(&message_id)
                .unwrap_or_default();
            durable
                .replace_message_mutations(&message_id, mutations)
                .await
                .map_err(TelegramDurableProjectionError::Persistence)?;
        }
        if let Some((account_id, provider_chat_id, state)) = event_chat_state(&frame.event) {
            durable
                .upsert_chat_state(account_id, provider_chat_id, &state)
                .await
                .map_err(TelegramDurableProjectionError::Persistence)?;
        }
        if let Some((account_id, provider_chat_id)) = event_chat_operational_state(&frame.event) {
            if let Some(state) = self
                .persistence
                .chat_operational_state(account_id, provider_chat_id)
                .cloned()
            {
                durable
                    .upsert_chat_operational_state(account_id, provider_chat_id, &state)
                    .await
                    .map_err(TelegramDurableProjectionError::Persistence)?;
            }
        }
        for operation in self.operations_for_account(&frame.account_id) {
            durable
                .save_operation(&operation)
                .await
                .map_err(TelegramDurableProjectionError::Persistence)?;
        }
        if let Some(draft) =
            provider_event_draft(&frame.event).map_err(TelegramDurableProjectionError::Contract)?
        {
            let draft = admit_provider_event_body(draft, &frame.event, body_admitter)?;
            let (record, recorded_at_unix_seconds) = self
                .communication_observation_record(&draft)
                .map_err(TelegramDurableProjectionError::Contract)?;
            durable
                .enqueue_communications_outbox(&record, recorded_at_unix_seconds)
                .await
                .map_err(TelegramDurableProjectionError::Persistence)?;
        }
        Ok(())
    }

    pub fn track_attachment(
        &mut self,
        attachment: makosh_telegram_api::TelegramAttachmentProjection,
    ) -> Result<(), TelegramContractError> {
        if attachment.attachment_id.trim().is_empty()
            || attachment.account_id.trim().is_empty()
            || attachment.provider_chat_id.trim().is_empty()
            || attachment.provider_message_id.trim().is_empty()
            || attachment.provider_file_id.trim().is_empty()
        {
            return Err(TelegramContractError::EmptyField);
        }
        self.persistence.put_attachment(attachment);
        Ok(())
    }

    pub fn apply_provider_event(
        &mut self,
        event: makosh_telegram_api::TelegramProviderEvent,
    ) -> Result<Option<CommunicationObservationDraft>, TelegramContractError> {
        match &event {
            makosh_telegram_api::TelegramProviderEvent::ChatFoldersChanged { folders, .. } => {
                self.persistence.put_chat_folders(folders.clone());
            }
            makosh_telegram_api::TelegramProviderEvent::ChatPositionChanged(position) => {
                self.persistence.put_chat_position(position.clone());
                self.refresh_chat_operational_state(
                    &position.account_id,
                    &position.provider_chat_id,
                );
            }
            makosh_telegram_api::TelegramProviderEvent::ChatNotificationChanged {
                account_id,
                provider_chat_id,
                use_default_mute_for,
                mute_for_seconds,
            } => {
                let mut state = self
                    .persistence
                    .chat_operational_state(account_id, provider_chat_id)
                    .cloned()
                    .unwrap_or_default();
                state.is_muted = !*use_default_mute_for && *mute_for_seconds > 0;
                state.mute_for_seconds = *mute_for_seconds;
                self.persistence
                    .put_chat_operational_state(account_id, provider_chat_id, state);
            }
            makosh_telegram_api::TelegramProviderEvent::ChatAvatarChanged(avatar) => {
                self.persistence.put_chat_avatar(avatar.clone());
            }
            makosh_telegram_api::TelegramProviderEvent::ParticipantChanged(participant) => {
                self.persistence.upsert_participant(participant.clone());
            }
            makosh_telegram_api::TelegramProviderEvent::ChatMarkedUnreadChanged {
                account_id,
                provider_chat_id,
                is_marked_as_unread,
            } => {
                let mut state = self
                    .persistence
                    .chat_operational_state(account_id, provider_chat_id)
                    .cloned()
                    .unwrap_or_default();
                state.is_marked_as_unread = *is_marked_as_unread;
                self.persistence
                    .put_chat_operational_state(account_id, provider_chat_id, state);
            }
            makosh_telegram_api::TelegramProviderEvent::FileChanged(file) => {
                self.persistence.put_file(file.clone());
                self.persistence
                    .apply_file_to_attachments(&file.account_id, file);
            }
            makosh_telegram_api::TelegramProviderEvent::MessageSendFailed {
                account_id,
                provider_chat_id,
                old_provider_message_id,
                ..
            } => {
                self.persistence.reconcile_delivery(
                    account_id,
                    provider_chat_id,
                    old_provider_message_id,
                    None,
                    TelegramDeliveryState::SendFailed,
                );
            }
            makosh_telegram_api::TelegramProviderEvent::MessageSendSucceeded {
                account_id,
                provider_chat_id,
                old_provider_message_id,
                provider_message_id,
            } => {
                self.persistence.reconcile_delivery(
                    account_id,
                    provider_chat_id,
                    old_provider_message_id,
                    Some(provider_message_id),
                    TelegramDeliveryState::Sent,
                );
            }
            makosh_telegram_api::TelegramProviderEvent::ReactionsObserved {
                account_id,
                provider_chat_id,
                provider_message_id,
                reactions,
            } => {
                self.persistence.replace_reactions(
                    &format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}"),
                    reactions.clone(),
                );
            }
            makosh_telegram_api::TelegramProviderEvent::TopicChanged(topic) => {
                self.persistence.put_topic(topic.clone());
            }
            _ => {}
        }
        if let makosh_telegram_api::TelegramProviderEvent::MessageCreated(observation) = &event {
            return self.ingest_message(observation.clone()).map(Some);
        }
        match &event {
            makosh_telegram_api::TelegramProviderEvent::MessageEdited {
                account_id,
                provider_chat_id,
                provider_message_id,
                text,
                observed_at_unix_seconds,
            } => {
                let message_id =
                    format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}");
                let version_number = self.persistence.next_message_version_number(&message_id);
                self.persistence.append_message_version(
                    makosh_telegram_api::TelegramMessageVersion {
                        version_id: format!("{message_id}:version:{version_number}"),
                        message_id: message_id.clone(),
                        account_id: account_id.clone(),
                        provider_chat_id: provider_chat_id.clone(),
                        provider_message_id: provider_message_id.clone(),
                        version_number,
                        body_text: text.clone(),
                        observed_at_unix_seconds: *observed_at_unix_seconds,
                        source: makosh_telegram_api::TelegramMessageVersionSource::Provider,
                    },
                );
                if let Some(text) = text {
                    self.persistence.apply_message_text_edit(&message_id, text);
                }
            }
            makosh_telegram_api::TelegramProviderEvent::MessageDeleted {
                account_id,
                provider_chat_id,
                provider_message_id,
                is_permanent,
            } => {
                let message_id =
                    format!("telegram:{account_id}:{provider_chat_id}:{provider_message_id}");
                self.persistence.append_message_tombstone(
                    makosh_telegram_api::TelegramMessageTombstone {
                        tombstone_id: format!("{message_id}:tombstone:provider"),
                        message_id,
                        account_id: account_id.clone(),
                        provider_chat_id: provider_chat_id.clone(),
                        provider_message_id: provider_message_id.clone(),
                        reason: makosh_telegram_api::TelegramTombstoneReason::ProviderDeleted,
                        observed_at_unix_seconds: 0,
                        is_provider_delete: *is_permanent,
                        is_locally_visible: false,
                    },
                );
            }
            _ => {}
        }
        if let Some((account_id, provider_chat_id, provider_message_id, mutation)) =
            event_message_mutation(&event)
        {
            self.persistence.apply_message_mutation(
                account_id,
                provider_chat_id,
                provider_message_id,
                mutation,
            );
        }
        if let Some((account_id, provider_chat_id, state)) = event_chat_state(&event) {
            self.persistence
                .apply_chat_state(account_id, provider_chat_id, state);
        }
        self.reconcile_provider_operations(&event);
        let draft = provider_event_draft(&event)?;
        Ok(draft)
    }

    fn communication_observation_record(
        &self,
        draft: &CommunicationObservationDraft,
    ) -> Result<(OutboxRecordV1, i64), TelegramContractError> {
        let admission = self
            .admission
            .as_ref()
            .ok_or(TelegramContractError::RuntimeBlocked)?;
        let recorded_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TelegramContractError::RuntimeBlocked)?;
        let recorded_at_unix_seconds = i64::try_from(recorded_at.as_secs())
            .map_err(|_| TelegramContractError::RuntimeBlocked)?;
        let record = build_observation_outbox_record_v1(
            draft,
            &ObservationEnvelopeContextV1 {
                runtime_instance_id: admission.runtime_instance_id.clone(),
                runtime_generation: admission.runtime_generation,
                module_id: PACKAGE.to_owned(),
                recorded_at_unix_seconds,
                recorded_at_nanos: i32::try_from(recorded_at.subsec_nanos())
                    .map_err(|_| TelegramContractError::RuntimeBlocked)?,
            },
        )
        .map_err(|_| TelegramContractError::RuntimeBlocked)?;
        Ok((record, recorded_at_unix_seconds))
    }

    fn refresh_chat_operational_state(&mut self, account_id: &str, provider_chat_id: &str) {
        let prior = self
            .persistence
            .chat_operational_state(account_id, provider_chat_id)
            .cloned()
            .unwrap_or_default();
        let positions = self
            .persistence
            .chat_positions(account_id, provider_chat_id);
        self.persistence.put_chat_operational_state(
            account_id,
            provider_chat_id,
            makosh_telegram_api::TelegramChatOperationalState {
                is_archived: positions
                    .iter()
                    .any(|position| position.list_kind == "archive" && position.order > 0),
                is_pinned: positions.iter().any(|position| position.is_pinned),
                ..prior
            },
        );
    }

    fn reconcile_provider_operations(
        &mut self,
        event: &makosh_telegram_api::TelegramProviderEvent,
    ) {
        let account_id = makosh_telegram_api::provider_event_account_id(event).to_owned();
        let expected_self_member_id = self
            .persistence
            .account(&account_id)
            .map(|account| account.external_account_id.clone());
        let operation_ids = self.persistence.operation_ids_for_account(&account_id);
        for operation_id in operation_ids {
            let Some(command) = self.persistence.command(&operation_id) else {
                continue;
            };
            if let (
                makosh_telegram_api::TelegramProviderEvent::ParticipantChanged(participant),
                TelegramProviderCommand::Join { .. } | TelegramProviderCommand::Leave { .. },
            ) = (event, command)
            {
                let expected = expected_self_member_id
                    .as_deref()
                    .map(|value| value.strip_prefix("telegram:").unwrap_or(value));
                if expected != Some(participant.provider_member_id.as_str()) {
                    continue;
                }
            }
            if !provider_event_targets_command(event, command) {
                continue;
            }
            let Some(operation) = self.persistence.operation(&operation_id) else {
                continue;
            };
            if matches!(
                operation.state,
                makosh_telegram_api::TelegramOperationState::Running
                    | makosh_telegram_api::TelegramOperationState::AwaitingProvider
            ) {
                self.persistence.reconcile_operation(
                    &operation_id,
                    provider_event_matches_command(event, command),
                );
            }
        }
    }

    pub fn start_qr_login(
        &mut self,
        account_id: &str,
        setup_id: &str,
        expires_at_unix_seconds: u64,
    ) -> Result<makosh_telegram_api::TelegramQrLoginSession, TelegramContractError> {
        self.persistence
            .account(account_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        let session = qr_login_preparing(
            setup_id.to_owned(),
            account_id.to_owned(),
            expires_at_unix_seconds,
        );
        self.persistence.put_qr_session(session.clone());
        Ok(session)
    }

    pub fn issue_qr(
        &mut self,
        setup_id: &str,
        qr_link: &str,
    ) -> Result<makosh_telegram_api::TelegramQrLoginSession, TelegramContractError> {
        let session = self
            .persistence
            .qr_session(setup_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        let updated = qr_login_qr_issued(session, qr_link.to_owned())?;
        self.persistence.put_qr_session(updated.clone());
        Ok(updated)
    }

    pub fn mark_qr_password_required(
        &mut self,
        setup_id: &str,
    ) -> Result<makosh_telegram_api::TelegramQrLoginSession, TelegramContractError> {
        let session = self
            .persistence
            .qr_session(setup_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        let updated = qr_login_password_required(session)?;
        self.persistence.put_qr_session(updated.clone());
        Ok(updated)
    }

    pub fn submit_qr_password(
        &mut self,
        setup_id: &str,
    ) -> Result<makosh_telegram_api::TelegramQrLoginSession, TelegramContractError> {
        let session = self
            .persistence
            .qr_session(setup_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        let updated = qr_login_password_submitted(session)?;
        self.persistence.put_qr_session(updated.clone());
        Ok(updated)
    }

    pub fn complete_qr_login(
        &mut self,
        setup_id: &str,
    ) -> Result<makosh_telegram_api::TelegramQrLoginSession, TelegramContractError> {
        let session = self
            .persistence
            .qr_session(setup_id)
            .ok_or(TelegramContractError::AccountUnknown)?;
        let updated = qr_login_ready(session)?;
        self.persistence.put_qr_session(updated.clone());
        Ok(updated)
    }
}

fn admit_provider_event_body<F>(
    draft: CommunicationObservationDraft,
    event: &makosh_telegram_api::TelegramProviderEvent,
    body_admitter: &mut F,
) -> Result<CommunicationObservationDraft, TelegramDurableProjectionError>
where
    F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
{
    let text = match event {
        makosh_telegram_api::TelegramProviderEvent::MessageCreated(observation) => {
            observation.text.as_deref()
        }
        makosh_telegram_api::TelegramProviderEvent::MessageEdited { text, .. } => text.as_deref(),
        _ => None,
    };
    let Some(text) = text else {
        return Ok(draft);
    };
    makosh_telegram_api::validate_text(text).map_err(TelegramDurableProjectionError::Contract)?;
    if draft.body != BodyAvailabilityV1::Unavailable {
        return Err(TelegramDurableProjectionError::Contract(
            TelegramContractError::InvalidTransition,
        ));
    }
    match body_admitter(text.as_bytes()) {
        Ok(receipt) => {
            let mut admitted = draft;
            admitted.body = BodyAvailabilityV1::AdmittedBlob;
            with_admitted_body_blob(admitted, receipt).map_err(|_| {
                TelegramDurableProjectionError::Contract(TelegramContractError::InvalidTransition)
            })
        }
        Err(failure) => with_body_admission_failure(draft, failure).map_err(|_| {
            TelegramDurableProjectionError::Contract(TelegramContractError::InvalidTransition)
        }),
    }
}
