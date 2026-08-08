use chrono::Utc;
use makosh_communications_api::accounts::ProviderAccount;
use makosh_communications_api::commands::NewCommunicationProviderCommand;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::{Value, json};
use uuid::Uuid;

use super::command_service::CommunicationWorkflowStateTransitionResult;
use super::command_service::{CommunicationCommandService, CommunicationCommandServiceError};
use super::messages::models::ProjectedMessage;
use super::messages::states::WorkflowState;
use super::messages::store::MessageProjectionStore;
use super::outbox::provider_send_store::ProviderSendStore;
use super::spam_reputation::{SenderReputationClassification, SenderReputationStore};
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use makosh_communications_api::email::OutgoingEmail;

const LOCAL_USER_ACTOR_ID: &str = "makosh-local-user";

impl CommunicationCommandService {
    pub(crate) async fn enqueue_archive_provider_command_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        workflow_command_id: &str,
        message: &ProjectedMessage,
        actor_id: &str,
    ) -> Result<(), CommunicationCommandServiceError> {
        if workflow_command_id.trim().is_empty() || actor_id.trim().is_empty() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "workflow_command_id and actor_id must not be empty",
            ));
        }
        let command_id = format!("mail-archive:{}", workflow_command_id.trim());
        Self::enqueue_mail_message_provider_command_in_transaction(
            transaction,
            &command_id,
            message,
            "archive",
            actor_id,
        )
        .await
    }

    pub async fn transition_message_workflow_state(
        &self,
        message_id: &str,
        new_state: WorkflowState,
        actor_id: &str,
    ) -> Result<CommunicationWorkflowStateTransitionResult, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(super::messages::errors::MessageProjectionError::MessageNotFound)?;
        if !WorkflowState::is_valid_transition(&current.workflow_state, &new_state) {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "invalid workflow state transition",
            ));
        }
        let previous_state = current.workflow_state.as_str().to_owned();
        let mut transaction = self.pool.begin().await?;
        let updated = MessageProjectionStore::transition_workflow_state_in_transaction(
            &mut transaction,
            message_id,
            new_state,
        )
        .await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "message_id": message_id,
                    "previous_state": previous_state,
                    "workflow_state": new_state.as_str(),
                    "operation": "message_workflow_state_transition",
                    "actor_id": actor_id,
                }),
                format!("message://{message_id}/workflow-state"),
            )
            .provenance(json!({
                "captured_by": "mail_service.transition_message_workflow_state",
                "operation": "message_workflow_state_transition",
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "workflow state evidence capture",
                source,
            },
        )?;
        link_mail_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "communication_message",
            updated.message_id.clone(),
            "workflow_state_transition",
            json!({ "workflow_state": updated.workflow_state.as_str() }),
            Some(json!({ "previous_state": previous_state })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "workflow state evidence link",
                source,
            },
        )?;
        if let Some(command_kind) = match (current.workflow_state, new_state) {
            (_, WorkflowState::Spam) => Some("mark_spam"),
            (WorkflowState::Spam, WorkflowState::New) => Some("mark_not_spam"),
            (_, WorkflowState::Archived) => Some("archive"),
            _ => None,
        } {
            let command_id = format!("mail-{command_kind}:{}", Uuid::new_v4());
            Self::enqueue_mail_message_provider_command_in_transaction(
                &mut transaction,
                &command_id,
                &updated,
                command_kind,
                actor_id,
            )
            .await?;
        }
        transaction.commit().await?;
        if let Some(classification) = match (current.workflow_state, new_state) {
            (_, WorkflowState::Spam) => Some(SenderReputationClassification::Spam),
            (WorkflowState::Spam, WorkflowState::New) => {
                Some(SenderReputationClassification::NonSpam)
            }
            _ => None,
        } && let Err(error) = SenderReputationStore::new(self.pool.clone())
            .record_analysis(&updated, classification, "manual_workflow_state_transition")
            .await
        {
            tracing::warn!(
                message_id = %updated.message_id,
                error = %error,
                "mail sender reputation feedback could not be recorded"
            );
        }
        Ok(CommunicationWorkflowStateTransitionResult {
            updated,
            previous_state,
        })
    }

    pub async fn record_provider_send_sent(
        &self,
        account: &ProviderAccount,
        email: &OutgoingEmail,
        transport: &str,
        provider_message_id: &str,
    ) -> Result<(), CommunicationCommandServiceError> {
        let operation = match transport.trim() {
            "smtp" => "provider_send_smtp".to_owned(),
            "gmail" => "provider_send_gmail".to_owned(),
            other => format!("provider_send_{other}"),
        };
        let observation = self
            .capture_observation(
                "provider send",
                "COMMUNICATION_MESSAGE",
                json!({
                    "account_id": account.account_id.clone(),
                    "transport": transport,
                    "to_recipient_count": email.to.len(),
                    "cc_recipient_count": email.cc.len(),
                    "bcc_recipient_count": email.bcc.len(),
                    "subject": email.subject.clone(),
                    "has_body_text": !email.body_text.trim().is_empty(),
                    "has_body_html": email.body_html.as_deref().is_some_and(|body| !body.trim().is_empty()),
                    "operation": operation,
                }),
                format!("provider-send://{}/{}", account.account_id, transport.trim()),
                json!({
                    "captured_by": "mail_service.record_provider_send_sent",
                    "operation": operation,
                }),
            )
            .await?;
        ProviderSendStore::new(self.pool.clone())
            .record_sent_with_observation(
                &observation.observation_id,
                provider_message_id,
                transport,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn move_message_to_local_trash(
        &self,
        message_id: &str,
        operation: &'static str,
        reason: &'static str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(super::messages::errors::MessageProjectionError::MessageNotFound)?;
        let mut transaction = self.pool.begin().await?;
        let updated = MessageProjectionStore::move_to_local_trash_in_transaction(
            &mut transaction,
            message_id,
            reason,
        )
        .await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "message_id": message_id,
                    "previous_local_state": current.local_state.as_str(),
                    "local_state": "trash",
                    "operation": operation,
                }),
                format!("message://{message_id}/{}", operation.replace('_', "-")),
            )
            .provenance(json!({
                "captured_by": "mail_service.move_message_to_local_trash",
                "operation": operation,
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "trash state evidence capture",
                source,
            },
        )?;
        link_mail_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "communication_message",
            updated.message_id.clone(),
            "local_state_transition",
            json!({
                "local_state": updated.local_state.as_str(),
                "source": reason,
            }),
            Some(json!({
                "previous_local_state": current.local_state.as_str(),
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "trash state evidence link",
                source,
            },
        )?;
        let command_id = format!("mail-trash:{}", Uuid::new_v4());
        let command = NewCommunicationProviderCommand::new(
            &command_id,
            &updated.account_id,
            "mail",
            "trash",
            &command_id,
            LOCAL_USER_ACTOR_ID,
        )
        .provider_message_id(&updated.provider_record_id)
        .target_ref(json!({ "message_id": updated.message_id }))
        .payload(json!({
            "provider_record_id": updated.provider_record_id,
            "message_metadata": updated.message_metadata,
            "reason": reason,
        }));
        CommunicationProviderCommandStore::enqueue_in_transaction(&mut transaction, &command)
            .await?;
        transaction.commit().await?;
        Ok(updated)
    }

    pub async fn restore_message_from_local_trash(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(super::messages::errors::MessageProjectionError::MessageNotFound)?;
        let observation = self
            .capture_observation(
                "message_restore",
                "COMMUNICATION_MESSAGE",
                json!({
                    "message_id": message_id,
                    "previous_local_state": current.local_state.as_str(),
                    "operation": "message_restore",
                }),
                format!("message://{message_id}/restore"),
                json!({
                    "captured_by": "mail_service.restore_message_from_local_trash",
                    "operation": "message_restore",
                }),
            )
            .await?;
        Ok(store
            .restore_from_local_trash_with_observation(
                message_id,
                Some(&observation.observation_id),
                "local_state_transition",
                Some(json!({
                    "previous_local_state": current.local_state.as_str(),
                })),
            )
            .await?)
    }

    pub async fn mark_message_read_local(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        self.set_message_read_local_with_provider_command(message_id, true, "makosh-local-user")
            .await
    }

    pub async fn set_message_read_local_with_provider_command(
        &self,
        message_id: &str,
        is_read: bool,
        actor_id: &str,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let store = MessageProjectionStore::new(self.pool.clone());
        let current = store
            .message(message_id)
            .await?
            .ok_or(super::messages::errors::MessageProjectionError::MessageNotFound)?;
        if actor_id.trim().is_empty() {
            return Err(CommunicationCommandServiceError::InvalidRequest(
                "actor_id must not be empty",
            ));
        }
        let command_kind = if is_read { "mark_read" } else { "mark_unread" };
        let operation = if is_read {
            "local_mark_read"
        } else {
            "local_mark_unread"
        };
        let mut transaction = self.pool.begin().await?;
        let updated = MessageProjectionStore::set_read_state_in_transaction(
            &mut transaction,
            message_id,
            is_read,
            "local_user",
        )
        .await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "message_id": updated.message_id,
                    "previous_is_read": current.is_read,
                    "is_read": is_read,
                    "operation": operation,
                }),
                format!("message://{message_id}/{operation}"),
            )
            .provenance(json!({
                "captured_by": "mail_service.set_message_read_local_with_provider_command",
                "operation": operation,
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "read state evidence capture",
                source,
            },
        )?;
        link_mail_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "communication_message",
            updated.message_id.clone(),
            "read_state_transition",
            json!({ "is_read": is_read }),
            None,
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "read state evidence link",
                source,
            },
        )?;
        let command_id = format!("mail-read:{}", Uuid::new_v4());
        let command = NewCommunicationProviderCommand::new(
            &command_id,
            &updated.account_id,
            "mail",
            command_kind,
            &command_id,
            actor_id,
        )
        .provider_message_id(&updated.provider_record_id)
        .target_ref(json!({ "message_id": updated.message_id }))
        .payload(json!({
            "desired_is_read": is_read,
            "read_changed_at": updated.read_changed_at.map(|value| value.to_rfc3339()),
            "provider_record_id": updated.provider_record_id,
            "message_metadata": updated.message_metadata,
        }));
        CommunicationProviderCommandStore::enqueue_in_transaction(&mut transaction, &command)
            .await?;
        transaction.commit().await?;
        Ok(updated)
    }

    pub(super) async fn persist_provider_synced_metadata(
        &self,
        current: &ProjectedMessage,
        metadata: &Value,
        command_kind: &str,
        command_payload: Value,
        observation_id: &str,
        link_metadata: Value,
    ) -> Result<ProjectedMessage, CommunicationCommandServiceError> {
        let mut transaction = self.pool.begin().await?;
        let updated = MessageProjectionStore::set_message_metadata_in_transaction(
            &mut transaction,
            &current.message_id,
            metadata,
        )
        .await?;
        link_mail_entity_in_transaction(
            &mut transaction,
            observation_id,
            "communication_message",
            updated.message_id.clone(),
            "message_flag_update",
            json!({}),
            Some(link_metadata),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "message flag evidence link",
                source,
            },
        )?;
        let command_id = format!("mail-command:{}", Uuid::new_v4());
        let mut payload = command_payload;
        payload["provider_record_id"] = json!(updated.provider_record_id);
        payload["message_metadata"] = updated.message_metadata.clone();
        let command = NewCommunicationProviderCommand::new(
            &command_id,
            &updated.account_id,
            "mail",
            command_kind,
            &command_id,
            LOCAL_USER_ACTOR_ID,
        )
        .provider_message_id(&updated.provider_record_id)
        .target_ref(json!({ "message_id": updated.message_id }))
        .payload(payload);
        CommunicationProviderCommandStore::enqueue_in_transaction(&mut transaction, &command)
            .await?;
        transaction.commit().await?;
        Ok(updated)
    }
}
