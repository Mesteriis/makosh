use serde_json::Value;
use sqlx::postgres::PgPool;

use super::errors::MessageProjectionError;
use super::models::NewProjectedMessage;
use super::models::ProjectedMessage;
use super::provider_channel_store::ProviderChannelMessageStore;
use super::states::WorkflowState;
use super::store::MessageProjectionStore;
use crate::platform::communications::errors::ProviderCommunicationMessagePortError;
use makosh_communications_api::projection_state::{
    MessageProjectionState, MessageProjectionStateQueryPort,
};
use makosh_communications_api::projections::{
    MessageProjectionInput, MessageProjectionRead, MessageProjectionWritePort,
};
use makosh_communications_api::provider_messages::ProviderChannelMessage;
use makosh_communications_postgres::projection_state::CommunicationMessageProjectionStateQuery;
use makosh_communications_postgres::projections::CommunicationMessageProjectionStore;

/// Application-facing command/query boundary for message projections.
///
/// Workflows depend on this boundary rather than on SQL storage. PostgreSQL
/// ownership remains encapsulated by `MessageProjectionStore` until this port
/// is moved into the Communications API/PG crate pair.
#[derive(Clone)]
pub struct MessageProjectionPort {
    store: MessageProjectionStore,
    projection_writer: CommunicationMessageProjectionStore,
    projection_state: CommunicationMessageProjectionStateQuery,
}

/// Provider-neutral query port for projected communication messages.
#[derive(Clone)]
pub struct ProviderChannelMessagePort(ProviderChannelMessageStore);

impl ProviderChannelMessagePort {
    pub fn new(pool: PgPool) -> Self {
        Self(ProviderChannelMessageStore::new(pool))
    }

    pub async fn message_by_id(
        &self,
        message_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        self.0.message_by_id(message_id, channel_kinds).await
    }

    pub async fn message_by_provider_record_id(
        &self,
        account_id: &str,
        provider_record_id: &str,
        channel_kinds: &[&str],
    ) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
        self.0
            .message_by_provider_record_id(account_id, provider_record_id, channel_kinds)
            .await
    }
}

impl MessageProjectionPort {
    pub fn new(pool: PgPool) -> Self {
        Self {
            store: MessageProjectionStore::new(pool.clone()),
            projection_writer: CommunicationMessageProjectionStore::new(pool.clone()),
            projection_state: CommunicationMessageProjectionStateQuery::new(pool),
        }
    }

    pub async fn upsert_projection(
        &self,
        input: &MessageProjectionInput,
    ) -> Result<MessageProjectionRead, String> {
        self.projection_writer
            .upsert(input)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn projection_state(
        &self,
        message_id: &str,
    ) -> Result<Option<MessageProjectionState>, String> {
        self.projection_state
            .state(message_id)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn message(
        &self,
        message_id: &str,
    ) -> Result<Option<ProjectedMessage>, MessageProjectionError> {
        self.store.message(message_id).await
    }

    pub async fn upsert_channel_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.upsert_channel_message_via_contract(message).await
    }

    async fn upsert_channel_message_via_contract(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        let input = MessageProjectionInput {
            message_id: message.message_id.clone(),
            raw_record_id: message.raw_record_id.clone(),
            account_id: message.account_id.clone(),
            provider_record_id: message.provider_record_id.clone(),
            subject: message.subject.clone(),
            sender: message.sender.clone(),
            recipients: message.recipients.clone(),
            body_text: message.body_text.clone(),
            occurred_at: message.occurred_at,
            channel_kind: message.channel_kind.clone(),
            conversation_id: message.conversation_id.clone(),
            sender_display_name: message.sender_display_name.clone(),
            delivery_state: message.delivery_state.clone(),
            metadata: message.message_metadata.clone(),
        };
        self.upsert_projection(&input)
            .await
            .map_err(MessageProjectionError::ProjectionWrite)?;
        self.store
            .message(&message.message_id)
            .await?
            .ok_or(MessageProjectionError::MessageNotFound)
    }

    pub async fn set_message_metadata_with_observation(
        &self,
        message_id: &str,
        metadata: &Value,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.store
            .set_message_metadata_with_observation(
                message_id,
                metadata,
                observation_id,
                relationship_kind,
                link_metadata,
            )
            .await
    }

    pub async fn set_ai_analysis(
        &self,
        message_id: &str,
        category: Option<&str>,
        summary: Option<&str>,
        importance_score: Option<i16>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.store
            .set_ai_analysis(message_id, category, summary, importance_score)
            .await
    }

    pub async fn set_message_metadata(
        &self,
        message_id: &str,
        metadata: &Value,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.store.set_message_metadata(message_id, metadata).await
    }

    pub async fn transition_workflow_state(
        &self,
        message_id: &str,
        state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.store
            .transition_workflow_state(message_id, state)
            .await
    }

    pub async fn upsert_email_participant(
        &self,
        message: &ProjectedMessage,
        persona_id: &str,
        email_address: &str,
        display_name: Option<&str>,
        role: &str,
    ) -> Result<bool, MessageProjectionError> {
        self.store
            .upsert_email_participant(message, persona_id, email_address, display_name, role)
            .await
    }
}
