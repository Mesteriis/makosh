use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};

use crate::platform::communications::errors::ProviderCommunicationMessagePortError;
use crate::platform::communications::{
    ProviderChannelMessageCommandPort, ProviderChannelMessageLookupPort,
};
use makosh_communications_api::provider_messages::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage, ProviderHeuristicMember,
    ProviderMessageAttachmentAnchor, ProviderMessageProjectionObservationContext,
    ProviderMessageReferenceSummary,
};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use makosh_observations_postgres::store::ObservationStore;

#[derive(Clone)]
pub struct ProviderChannelMessageStore {
    pool: PgPool,
}

async fn capture_projection_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    message: &ProviderChannelMessage,
    observed_at: DateTime<Utc>,
    relationship_kind: &str,
    payload: Value,
    actor: &str,
) -> Result<(), ProviderCommunicationMessagePortError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "COMMUNICATION_MESSAGE",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            payload,
            format!("message://{}/{}", message.message_id, relationship_kind),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": provider_from_channel_kind(&message.channel_kind),
            "account_id": message.account_id,
            "provider_message_id": message.provider_record_id,
            "provider_chat_id": message.conversation_id,
        })),
    )
    .await?;
    link_domain_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "communications",
        "communication_message",
        message.message_id.clone(),
        Some(relationship_kind),
        None,
        Some(json!({
            "account_id": message.account_id,
            "provider_message_id": message.provider_record_id,
            "provider_chat_id": message.conversation_id,
        })),
    )
    .await?;
    Ok(())
}

fn row_to_provider_channel_message(
    row: PgRow,
) -> Result<ProviderChannelMessage, ProviderCommunicationMessagePortError> {
    Ok(ProviderChannelMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_record_id: row.try_get("provider_record_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        body_text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        conversation_id: row
            .try_get::<Option<String>, _>("conversation_id")?
            .unwrap_or_default(),
        sender_display_name: row.try_get("sender_display_name")?,
        delivery_state: row.try_get("delivery_state")?,
        message_metadata: row.try_get("message_metadata")?,
    })
}

fn provider_from_channel_kind(channel_kind: &str) -> &'static str {
    match channel_kind {
        "telegram_user" | "telegram_bot" => "telegram",
        "whatsapp_web" => "whatsapp_web",
        "whatsapp_business_cloud" => "whatsapp_business_cloud",
        _ => "provider",
    }
}
mod command;
mod lookup;
#[path = "provider_channel_store/runtime.rs"]
mod runtime;
