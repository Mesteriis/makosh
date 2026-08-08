use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::application::communication_fixture_error::CommunicationFixtureIngestError;
use crate::application::communication_fixture_ids::stable_whatsapp_id;
use crate::application::communication_fixture_ids::{
    call as whatsapp_call_id, channel as whatsapp_channel_id,
    conversation as whatsapp_conversation_id, fixture_event as whatsapp_fixture_event_id,
    identity as whatsapp_identity_id, message_ref as whatsapp_message_ref_id,
    message_tombstone as whatsapp_message_tombstone_id,
    message_version as whatsapp_message_version_id,
    participant as whatsapp_conversation_participant_id, reaction as whatsapp_reaction_id,
    runtime_event_raw as whatsapp_runtime_event_raw_record_id,
    status_feed_conversation as whatsapp_status_feed_conversation_id,
    status_message as whatsapp_status_message_id,
};
use crate::application::communication_fixture_metadata_merge::{
    annotate_observed_source as annotate_whatsapp_raw_observed_source,
    merge_identity_display_name as merged_identity_display_name_metadata,
    merge_object as merged_object_metadata,
};
use crate::application::communication_fixture_models::{
    AcceptedWhatsappRawRecord, WhatsappAccountProjectionContext, WhatsappParticipantUpsertOutcome,
};
use crate::application::provider_runtime_services::WhatsAppProviderRuntimeRef;
use crate::domains::communications::messages::models::NewProjectedMessage;
use crate::domains::communications::messages::port::{
    MessageProjectionPort, ProviderChannelMessagePort,
};
use crate::domains::communications::messages::provider_observation_projection::{
    project_accepted_signal_if_runtime_allows, whatsapp::project_whatsapp_content_observed,
    whatsapp::project_whatsapp_delivery_state_observed,
};
use crate::domains::communications::ports::CommunicationRawEvidencePort;
use crate::domains::communications::storage::models::{
    NewCommunicationAttachment, NewCommunicationBlob,
};
use crate::domains::communications::storage::port::CommunicationAttachmentPort;
use crate::domains::signal_hub::whatsapp::dispatch_whatsapp_raw_signal;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappWebCallIngestResult, WhatsappWebDialogIngestResult,
    WhatsappWebMediaIngestResult, WhatsappWebMessageDeleteIngestResult,
    WhatsappWebMessageIngestResult, WhatsappWebMessageUpdateIngestResult,
    WhatsappWebParticipantIngestResult, WhatsappWebPresenceIngestResult,
    WhatsappWebReactionIngestResult, WhatsappWebReceiptIngestResult,
    WhatsappWebRuntimeEventIngestResult, WhatsappWebStatusDeleteIngestResult,
    WhatsappWebStatusIngestResult, WhatsappWebStatusViewIngestResult,
};
use crate::platform::calls::models::NewTelegramCall;
use crate::platform::calls::port::CallIntelligencePort;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::whatsapp_event_types;
use crate::workflows::review_inbox::{
    refresh_message_decisions_into_review, refresh_message_knowledge_candidates_into_review,
    refresh_message_people_candidates_into_review, refresh_message_task_candidates_into_review,
};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use makosh_events_postgres::store::EventStore;

#[path = "communication_fixture_observations.rs"]
mod communication_fixture_observations;
#[path = "communication_fixture_persistence.rs"]
mod communication_fixture_persistence;

const AUDIT_ACTOR_ID: &str = "makosh-frontend";
const WHATSAPP_CHANNEL_KINDS: &[&str] = &["whatsapp_web"];

#[derive(Clone)]
pub(crate) struct WhatsappFixtureIngestApplicationService {
    pool: PgPool,
    runtime: WhatsAppProviderRuntimeRef,
    event_store: EventStore,
    event_bus: InMemoryEventBus,
}

impl WhatsappFixtureIngestApplicationService {
    pub(crate) fn new(
        pool: PgPool,
        runtime: WhatsAppProviderRuntimeRef,
        event_store: EventStore,
        event_bus: InMemoryEventBus,
    ) -> Self {
        Self {
            pool,
            runtime,
            event_store,
            event_bus,
        }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(crate) fn event_store(&self) -> &EventStore {
        &self.event_store
    }

    pub(crate) fn event_bus(&self) -> &InMemoryEventBus {
        &self.event_bus
    }

    pub(crate) async fn ingest_message(
        &self,
        request: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_with_reconciliation_source(request, "provider_observed.fixture_message")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_message(
        &self,
        request: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_message",
        )
        .await
    }

    async fn ingest_message_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMessage,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_message(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = CommunicationRawEvidencePort::new(self.pool.clone())
            .record_raw_source(&observed_raw)
            .await?;
        let Some(accepted_event) =
            dispatch_whatsapp_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "whatsapp fixture signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        let Some(projected) =
            project_accepted_signal_if_runtime_allows(self.pool.clone(), &accepted_event).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "whatsapp fixture signal did not produce an accepted projection".to_owned(),
            ));
        };
        self.project_whatsapp_message_refs(
            request,
            &projected.message_id,
            &projected.raw_record_id,
        )
        .await?;
        self.upsert_whatsapp_persona_identity_traces_for_message(
            request,
            &stored_raw.observation_id,
        )
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_message_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;
        let message_ids = vec![projected.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;
        if let Some(projected_message) = MessageProjectionPort::new(self.pool.clone())
            .message(&projected.message_id)
            .await?
        {
            let _ = refresh_message_people_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
            let _ = refresh_message_knowledge_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
        }

        Ok(WhatsappWebMessageIngestResult {
            raw_record_id: projected.raw_record_id,
            message_id: projected.message_id,
        })
    }

    pub(crate) async fn ingest_reaction(
        &self,
        request: &NewWhatsappWebReaction,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        self.ingest_reaction_with_reconciliation_source(
            request,
            "provider_observed.fixture_reaction",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_reaction(
        &self,
        request: &NewWhatsappWebReaction,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        self.ingest_reaction_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_reaction",
        )
        .await
    }

    async fn ingest_reaction_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebReaction,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_reaction(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let message = ProviderChannelMessagePort::new(self.pool.clone())
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp reaction target message `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let reaction_id = whatsapp_reaction_id(
            &request.account_id,
            &request.provider_message_id,
            &request.provider_actor_id,
            &request.reaction,
        );
        sqlx::query(
            r#"
            INSERT INTO communication_message_reactions (
                reaction_id, message_id, account_id, provider_message_id,
                provider_conversation_id, sender_display_name, reaction, is_active,
                observed_at, source_event, provider_actor_id, metadata, provenance, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now())
            ON CONFLICT (reaction_id)
            DO UPDATE SET
                sender_display_name = EXCLUDED.sender_display_name,
                reaction = EXCLUDED.reaction,
                is_active = EXCLUDED.is_active,
                observed_at = EXCLUDED.observed_at,
                source_event = EXCLUDED.source_event,
                provider_actor_id = EXCLUDED.provider_actor_id,
                metadata = EXCLUDED.metadata,
                provenance = EXCLUDED.provenance,
                updated_at = now()
            "#,
        )
        .bind(&reaction_id)
        .bind(&message.message_id)
        .bind(&request.account_id)
        .bind(&request.provider_message_id)
        .bind(&message.conversation_id)
        .bind(&request.sender_display_name)
        .bind(&request.reaction)
        .bind(request.is_active)
        .bind(request.observed_at)
        .bind(&stored_raw.accepted_event_id)
        .bind(&request.provider_actor_id)
        .bind(json!({
            "provider": account_context.provider_kind,
            "provider_chat_id": request.provider_chat_id,
        }))
        .bind(json!({
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .execute(&self.pool)
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_reaction_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebReactionIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
            reaction_id,
        })
    }

    pub(crate) async fn ingest_message_update(
        &self,
        request: &NewWhatsappWebMessageUpdate,
    ) -> Result<WhatsappWebMessageUpdateIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_update_with_reconciliation_source(
            request,
            "provider_observed.fixture_message_update",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_message_update(
        &self,
        request: &NewWhatsappWebMessageUpdate,
    ) -> Result<WhatsappWebMessageUpdateIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_update_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_message_update",
        )
        .await
    }

    async fn ingest_message_update_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMessageUpdate,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMessageUpdateIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_message_update(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let store = ProviderChannelMessagePort::new(self.pool.clone());
        let message = store
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp message update target `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let updated_metadata = merged_object_metadata(
            &message.message_metadata,
            json!({
                "edited": true,
                "provider_chat_id": request.provider_chat_id,
                "accepted_signal_event_id": stored_raw.accepted_event_id,
            }),
        )?;
        let updated_message = project_whatsapp_content_observed(
            self.pool.clone(),
            &message.message_id,
            &request.text,
            &updated_metadata,
            request.observed_at,
        )
        .await?
        .ok_or_else(|| {
            CommunicationFixtureIngestError::SignalControlBlocked(format!(
                "whatsapp message update target `{}` disappeared during projection",
                request.provider_message_id
            ))
        })?;
        let version_id = whatsapp_message_version_id(&stored_raw.accepted_event_id);
        let next_version_number: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(MAX(version_number), 0) + 1
            FROM communication_message_versions
            WHERE message_id = $1
            "#,
        )
        .bind(&message.message_id)
        .fetch_one(&self.pool)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO communication_message_versions (
                version_id, message_id, account_id, provider_message_id,
                provider_conversation_id, version_number, body_text, edited_at,
                source_event, diff_payload, provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (version_id) DO NOTHING
            "#,
        )
        .bind(&version_id)
        .bind(&message.message_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.conversation_id)
        .bind(next_version_number)
        .bind(&request.text)
        .bind(request.observed_at)
        .bind(&stored_raw.accepted_event_id)
        .bind(
            crate::application::communication_fixture_metadata::whatsapp_local_edit_diff(
                Some(message.body_text.as_str()),
                &request.text,
            ),
        )
        .bind(json!({
            "provider": account_context.provider_kind,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .execute(&self.pool)
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_message_update_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebMessageUpdateIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: updated_message.message_id,
            version_id,
        })
    }

    pub(crate) async fn ingest_message_delete(
        &self,
        request: &NewWhatsappWebMessageDelete,
    ) -> Result<WhatsappWebMessageDeleteIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_delete_with_reconciliation_source(
            request,
            "provider_observed.fixture_message_delete",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_message_delete(
        &self,
        request: &NewWhatsappWebMessageDelete,
    ) -> Result<WhatsappWebMessageDeleteIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_delete_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_message_delete",
        )
        .await
    }

    async fn ingest_message_delete_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMessageDelete,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMessageDeleteIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_message_delete(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let message = ProviderChannelMessagePort::new(self.pool.clone())
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp message delete target `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let tombstone_id = whatsapp_message_tombstone_id(&stored_raw.accepted_event_id);
        sqlx::query(
            r#"
            INSERT INTO communication_message_tombstones (
                tombstone_id, message_id, account_id, provider_message_id,
                provider_conversation_id, reason_class, actor_class, observed_at,
                source_event, is_provider_delete, is_local_visible, metadata, provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, TRUE, FALSE, $10, $11)
            ON CONFLICT (tombstone_id) DO NOTHING
            "#,
        )
        .bind(&tombstone_id)
        .bind(&message.message_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.conversation_id)
        .bind(&request.reason_class)
        .bind(&request.actor_class)
        .bind(request.observed_at)
        .bind(&stored_raw.accepted_event_id)
        .bind(json!({
            "provider": account_context.provider_kind,
            "provider_chat_id": request.provider_chat_id,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .bind(json!({
            "provider": account_context.provider_kind,
            "raw_record_id": stored_raw.raw_record_id,
            "accepted_signal_event_id": stored_raw.accepted_event_id,
        }))
        .execute(&self.pool)
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_message_delete_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebMessageDeleteIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
            tombstone_id,
        })
    }

    pub(crate) async fn ingest_receipt(
        &self,
        request: &NewWhatsappWebReceipt,
    ) -> Result<WhatsappWebReceiptIngestResult, CommunicationFixtureIngestError> {
        self.ingest_receipt_with_observed_source(request, "provider_observed.fixture_receipt")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_receipt(
        &self,
        request: &NewWhatsappWebReceipt,
    ) -> Result<WhatsappWebReceiptIngestResult, CommunicationFixtureIngestError> {
        self.ingest_receipt_with_observed_source(
            request,
            "provider_observed.runtime_bridge_receipt",
        )
        .await
    }

    async fn ingest_receipt_with_observed_source(
        &self,
        request: &NewWhatsappWebReceipt,
        observed_source: &str,
    ) -> Result<WhatsappWebReceiptIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_receipt(request).await?;
        let observed_raw = annotate_whatsapp_raw_observed_source(&observed.raw, observed_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let store = ProviderChannelMessagePort::new(self.pool.clone());
        let message = store
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp receipt target `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let updated_message = project_whatsapp_delivery_state_observed(
            self.pool.clone(),
            &message.message_id,
            request.delivery_state.as_message_delivery_state(),
            request.observed_at,
        )
        .await?
        .ok_or_else(|| {
            CommunicationFixtureIngestError::SignalControlBlocked(format!(
                "whatsapp receipt target `{}` disappeared during projection",
                request.provider_message_id
            ))
        })?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_receipt_commands(request)
                .await?,
            "provider_observation_consumer",
        )
        .await?;

        Ok(WhatsappWebReceiptIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: updated_message.message_id,
            delivery_state: request.delivery_state.as_str().to_owned(),
        })
    }

    pub(crate) async fn ingest_media(
        &self,
        request: &NewWhatsappWebMedia,
    ) -> Result<WhatsappWebMediaIngestResult, CommunicationFixtureIngestError> {
        self.ingest_media_with_reconciliation_source(request, "provider_observed.fixture_media")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_media(
        &self,
        request: &NewWhatsappWebMedia,
    ) -> Result<WhatsappWebMediaIngestResult, CommunicationFixtureIngestError> {
        self.ingest_media_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_media",
        )
        .await
    }

    async fn ingest_media_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMedia,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMediaIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_media(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let message = ProviderChannelMessagePort::new(self.pool.clone())
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsapp media target message `{}` is not projected",
                    request.provider_message_id
                ))
            })?;
        let storage = CommunicationAttachmentPort::new(self.pool.clone());
        let storage_kind =
            crate::application::whatsapp_fixture_policy::normalized_media_storage_kind(
                &request.storage_kind,
            );
        let sha256 =
            crate::application::whatsapp_fixture_policy::normalized_media_sha256(&request.sha256);
        let blob = storage
            .upsert_blob(
                &NewCommunicationBlob::new(
                    &storage_kind,
                    &request.storage_path,
                    &sha256,
                    request.size_bytes,
                )
                .content_type(&request.content_type),
            )
            .await?;
        let mut attachment = NewCommunicationAttachment::new(
            &message.message_id,
            &stored_raw.raw_record_id,
            &blob.blob_id,
            &request.provider_attachment_id,
            &request.content_type,
            request.size_bytes,
            &sha256,
        );
        if let Some(filename) = request.filename.as_deref() {
            attachment = attachment.filename(filename);
        }
        let attachment = storage.upsert_attachment(&attachment).await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_media_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;

        Ok(WhatsappWebMediaIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            message_id: message.message_id,
            attachment_id: attachment.attachment_id,
        })
    }
}
