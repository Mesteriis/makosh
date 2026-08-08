use makosh_events_api::{EventEnvelope, NewEventEnvelope, StoredEventEnvelope};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::errors::MessageProjectionError;
use super::models::NewProjectedMessage;
use super::models::ProjectedMessage;
use super::port::MessageProjectionPort;
use super::projection::{project_raw_email_message, project_raw_email_message_from_blob};
use super::provider_channel_store::ProviderChannelMessageStore;
use super::rows::row_to_projected_message;
use super::store::MessageProjectionStore;
use crate::domains::communications::delivery_notifications::consume_accepted_mail_delivery_signal;
use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::communications::errors::ProviderCommunicationMessagePortError;
use makosh_communications_api::evidence::StoredRawCommunicationRecord;
use makosh_communications_api::provider_messages::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage,
    ProviderMessageProjectionObservationContext,
};
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

pub const COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER: &str =
    "communication_provider_observation_projection";

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];
const WHATSAPP_CHANNEL_KINDS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];
const ZULIP_CHANNEL_KIND: &str = "zulip";

pub async fn project_provider_observation_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    if is_supported_mail_delivery_signal_event(&event.event.event_type) {
        consume_accepted_mail_delivery_signal(pool.clone(), &event.event)
            .await
            .map(|_| ())
            .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
        return Ok(());
    }

    if is_base_accepted_signal_event(&event.event.event_type) {
        consume_accepted_signal_event(pool.clone(), &event.event)
            .await
            .map(|_| ())
            .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
        return Ok(());
    }

    if !is_supported_provider_observation_event(&event.event.event_type) {
        return Ok(());
    }

    let updated = telegram::project_telegram_observation(pool.clone(), &event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
    if let Some(message) = updated {
        append_communication_message_updated_event(pool, &event, &message).await?;
    }

    Ok(())
}

pub async fn replay_accepted_signal_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_provider_observation_event(pool, event).await
}

fn is_base_accepted_signal_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.mail.message"
            | "signal.accepted.telegram.message"
            | "signal.accepted.whatsapp.message"
            | "signal.accepted.zulip.message"
            | "signal.accepted.zulip.reaction"
            | "signal.accepted.zulip.message_update"
            | "signal.accepted.zulip.message_delete"
    )
}

fn is_supported_mail_delivery_signal_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.mail.delivery_status" | "signal.accepted.mail.read_receipt"
    )
}

fn is_supported_provider_observation_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.telegram.message.content"
            | "signal.accepted.telegram.message.metadata"
            | "signal.accepted.telegram.message.delivery_state"
            | "signal.accepted.telegram.message.provider_identity"
            | "signal.accepted.telegram.message.pinned_state"
            | "signal.accepted.telegram.attachment.download_state"
    )
}

pub fn supports_communication_projection_signal_event(event_type: &str) -> bool {
    is_base_accepted_signal_event(event_type) || is_supported_provider_observation_event(event_type)
}

pub async fn project_accepted_signal_if_runtime_allows(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    if !accepted_signal_projection_runtime_allows(&pool).await? {
        return Ok(None);
    }

    consume_accepted_signal_event(pool, event).await
}

pub async fn consume_accepted_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    let Some(projection) = project_accepted_signal_event(pool.clone(), event).await? else {
        return Ok(None);
    };

    append_communication_message_projected_event(
        pool,
        event,
        &projection.message,
        projection.message_existed,
    )
    .await?;

    Ok(Some(projection.message))
}

struct AcceptedSignalProjection {
    message: ProjectedMessage,
    message_existed: bool,
}

async fn project_accepted_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type == "signal.accepted.mail.message" {
        return mail::project_message(pool, event).await;
    }
    if event.event_type == "signal.accepted.telegram.message" {
        return telegram::project_telegram_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.whatsapp.message" {
        return whatsapp::project_whatsapp_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.zulip.message" {
        return zulip::project_zulip_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.zulip.reaction" {
        return zulip::project_zulip_reaction_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.zulip.message_update" {
        return zulip::project_zulip_message_update_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.zulip.message_delete" {
        return zulip::project_zulip_message_delete_signal_event(pool, event).await;
    }

    Ok(None)
}

async fn accepted_signal_projection_runtime_allows(
    pool: &PgPool,
) -> Result<bool, CommunicationSignalProjectionError> {
    Ok(crate::platform::events::runtime::runtime_allows_processing(
        pool,
        "system",
        COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        &json!({
            "label": "Communications accepted-signal consumer",
            "scope": "consumer",
        }),
    )
    .await?)
}

fn telegram_projection_context(
    event_kind: &str,
) -> ProviderMessageProjectionObservationContext<'static> {
    ProviderMessageProjectionObservationContext {
        channel_kinds: TELEGRAM_CHANNEL_KINDS,
        relationship_kind: match event_kind {
            "metadata_observed" => "telegram_metadata_observed",
            "delivery_state_observed" => "telegram_delivery_state_observed",
            "provider_identity_observed" => "telegram_provider_identity_observed",
            "content_observed" => "telegram_content_observed",
            "pinned_state_observed" => "telegram_pinned_state_observed",
            "attachment_download_state_observed" => "telegram_attachment_download_state_observed",
            _ => "telegram_provider_observed",
        },
        actor: "domains.communications.messages.communication_provider_observation_projection",
    }
}

fn whatsapp_projection_context(
    relationship_kind: &'static str,
) -> ProviderMessageProjectionObservationContext<'static> {
    ProviderMessageProjectionObservationContext {
        channel_kinds: WHATSAPP_CHANNEL_KINDS,
        relationship_kind,
        actor: "domains.communications.messages.communication_provider_observation_projection",
    }
}

fn zulip_projection_context(
    relationship_kind: &'static str,
) -> ProviderMessageProjectionObservationContext<'static> {
    ProviderMessageProjectionObservationContext {
        channel_kinds: &[ZULIP_CHANNEL_KIND],
        relationship_kind,
        actor: "domains.communications.messages.communication_provider_observation_projection",
    }
}

async fn append_communication_message_updated_event(
    pool: PgPool,
    provider_event: &StoredEventEnvelope,
    message: &ProviderChannelMessage,
) -> Result<(), EventStoreError> {
    let event_id = format!(
        "evt_communication_message_updated_{}",
        provider_event.event.event_id
    );
    let event = NewEventEnvelope::builder(
        event_id,
        "communication.message.updated",
        Utc::now(),
        json!({
            "kind": "communication_projection",
            "consumer": COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        }),
        json!({
            "kind": "communication_message",
            "id": message.message_id,
            "entity_id": message.message_id,
            "message_id": message.message_id,
        }),
    )
    .payload(json!({
        "message_id": message.message_id,
        "raw_record_id": message.raw_record_id,
        "account_id": message.account_id,
        "provider_record_id": message.provider_record_id,
        "channel_kind": message.channel_kind,
        "conversation_id": message.conversation_id,
        "delivery_state": message.delivery_state,
        "message_metadata": message.message_metadata,
        "provider_observation_event_id": provider_event.event.event_id,
        "provider_observation_event_type": provider_event.event.event_type,
    }))
    .provenance(json!({
        "ownership": "communications_projection",
        "source_event_id": provider_event.event.event_id,
    }))
    .causation_id(provider_event.event.event_id.clone())
    .correlation_id(
        provider_event
            .event
            .correlation_id
            .clone()
            .unwrap_or_else(|| provider_event.event.event_id.clone()),
    )
    .build()?;

    EventStore::new(pool).append_idempotent(&event).await?;
    Ok(())
}

async fn append_communication_message_projected_event(
    pool: PgPool,
    accepted_event: &EventEnvelope,
    message: &ProjectedMessage,
    message_existed: bool,
) -> Result<(), EventStoreError> {
    let event_name = if message_existed {
        "communication.message.updated"
    } else {
        "communication.message.recorded"
    };
    let event_suffix = if message_existed {
        "updated"
    } else {
        "recorded"
    };
    let event_id = format!(
        "evt_communication_message_{}_{}",
        event_suffix, accepted_event.event_id
    );
    let event = NewEventEnvelope::builder(
        event_id,
        event_name,
        Utc::now(),
        json!({
            "kind": "communication_projection",
            "consumer": COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        }),
        json!({
            "kind": "communication_message",
            "id": message.message_id,
            "entity_id": message.message_id,
            "message_id": message.message_id,
        }),
    )
    .payload(json!({
        "message_id": message.message_id,
        "raw_record_id": message.raw_record_id,
        "account_id": message.account_id,
        "provider_record_id": message.provider_record_id,
        "channel_kind": message.channel_kind,
        "conversation_id": message.conversation_id,
        "delivery_state": message.delivery_state,
        "accepted_signal_event_id": accepted_event.event_id,
        "accepted_signal_event_type": accepted_event.event_type,
        "projection_kind": event_suffix,
    }))
    .provenance(json!({
        "ownership": "communications_projection",
        "source_event_id": accepted_event.event_id,
    }))
    .causation_id(accepted_event.event_id.clone())
    .correlation_id(
        accepted_event
            .correlation_id
            .clone()
            .unwrap_or_else(|| accepted_event.event_id.clone()),
    )
    .build()?;

    EventStore::new(pool).append_idempotent(&event).await?;
    Ok(())
}

async fn communication_message_exists(
    pool: &PgPool,
    account_id: &str,
    provider_record_id: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM communication_messages
            WHERE account_id = $1
              AND provider_record_id = $2
        )
        "#,
    )
    .bind(account_id.trim())
    .bind(provider_record_id.trim())
    .fetch_one(pool)
    .await
}

async fn raw_record_for_accepted_signal(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<StoredRawCommunicationRecord, CommunicationSignalProjectionError> {
    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    CommunicationIngestionStore::new(pool)
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()).into())
}

async fn zulip_target_message(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
) -> Result<ProviderChannelMessage, CommunicationSignalProjectionError> {
    ensure_canonical_communication_account(&pool, &raw_record.account_id).await?;
    let provider_message_id = required_payload_str(&raw_record.payload, "provider_message_id")?;
    ProviderChannelMessageStore::new(pool)
        .message_by_provider_record_id(
            &raw_record.account_id,
            &provider_message_id,
            &[ZULIP_CHANNEL_KIND],
        )
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip target message `{provider_message_id}` is not projected"
            ))
            .into()
        })
}

async fn ensure_canonical_communication_account(
    pool: &PgPool,
    account_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id,
            config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            jsonb_build_object('source_table', 'communication_provider_accounts'),
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id)
        DO UPDATE SET
            provider_kind = EXCLUDED.provider_kind,
            display_name = EXCLUDED.display_name,
            external_account_id = EXCLUDED.external_account_id,
            config = EXCLUDED.config,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(account_id.trim())
    .execute(pool)
    .await?;
    Ok(())
}

async fn projected_message_by_id(
    pool: &PgPool,
    message_id: &str,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    let row = sqlx::query(
        r#"
        SELECT
            message_id,
            raw_record_id,
            observation_id,
            account_id,
            provider_record_id,
            subject,
            sender,
            recipients,
            body_text,
            occurred_at,
            projected_at,
            channel_kind,
            conversation_id,
            sender_display_name,
            delivery_state,
            message_metadata,
            workflow_state,
            importance_score,
            ai_category,
            ai_summary,
            ai_summary_generated_at,
            (SELECT s.ai_state FROM communication_ai_states s WHERE s.message_id = communication_messages.message_id) AS ai_state,
            local_state,
            local_state_changed_at,
            local_state_reason,
            is_read,
            read_changed_at,
            read_origin
        FROM communication_messages
        WHERE message_id = $1
        "#,
    )
    .bind(message_id.trim())
    .fetch_optional(pool)
    .await?;

    Ok(row.map(row_to_projected_message).transpose()?)
}

#[derive(Debug, Error)]
pub enum CommunicationSignalProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Communication(#[from] makosh_communications_postgres::errors::CommunicationIngestionError),

    #[error(transparent)]
    ProviderCommunication(#[from] ProviderCommunicationMessagePortError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error("accepted signal subject is missing `{0}`")]
    MissingSubjectField(&'static str),
}
mod helpers;
use helpers::*;
mod mail;
mod telegram;
pub(crate) mod whatsapp;
mod zulip;
