use serde_json::{Map, Value, json};

use makosh_communications_api::evidence::NewRawCommunicationRecord;

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::constants::{
    WHATSAPP_WEB_CALL_RECORD_KIND, WHATSAPP_WEB_DIALOG_RECORD_KIND, WHATSAPP_WEB_MEDIA_RECORD_KIND,
    WHATSAPP_WEB_MESSAGE_DELETE_RECORD_KIND, WHATSAPP_WEB_MESSAGE_RECORD_KIND,
    WHATSAPP_WEB_MESSAGE_UPDATE_RECORD_KIND, WHATSAPP_WEB_PARTICIPANT_RECORD_KIND,
    WHATSAPP_WEB_PRESENCE_RECORD_KIND, WHATSAPP_WEB_REACTION_RECORD_KIND,
    WHATSAPP_WEB_RECEIPT_RECORD_KIND, WHATSAPP_WEB_RUNTIME_EVENT_RECORD_KIND,
    WHATSAPP_WEB_STATUS_DELETE_RECORD_KIND, WHATSAPP_WEB_STATUS_RECORD_KIND,
    WHATSAPP_WEB_STATUS_VIEW_RECORD_KIND,
};
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappWebObservedCall, WhatsappWebObservedDialog,
    WhatsappWebObservedMedia, WhatsappWebObservedMessage, WhatsappWebObservedMessageDelete,
    WhatsappWebObservedMessageUpdate, WhatsappWebObservedParticipant, WhatsappWebObservedPresence,
    WhatsappWebObservedReaction, WhatsappWebObservedReceipt, WhatsappWebObservedRuntimeEvent,
    WhatsappWebObservedStatus, WhatsappWebObservedStatusDelete, WhatsappWebObservedStatusView,
};
use makosh_provider_whatsapp::ids::whatsapp_web_raw_record_id;

impl WhatsappWebStore {
    pub async fn ingest_fixture_message(
        &self,
        message: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebObservedMessage, WhatsappWebError> {
        message.validate()?;
        let provider_account = self
            .provider_account_store()
            .get(&message.account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{}` is not configured",
                    message.account_id
                ))
            })?;
        if !provider_account.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "account `{}` is not a WhatsApp Web provider account",
                message.account_id
            )));
        }

        let session = self
            .list_sessions(Some(&message.account_id), 1)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{}` has no session metadata",
                    message.account_id
                ))
            })?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &message.account_id,
            WHATSAPP_WEB_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            WHATSAPP_WEB_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            json!({
                "provider_chat_id": message.provider_chat_id,
                "chat_title": message.chat_title,
                "sender_id": message.sender_id,
                "sender_display_name": message.sender_display_name,
                "text": message.text,
                "reply_to_provider_message_id": message.reply_to_provider_message_id,
                "forward_origin_chat_id": message.forward_origin_chat_id,
                "forward_origin_message_id": message.forward_origin_message_id,
                "forward_origin_sender_id": message.forward_origin_sender_id,
                "forward_origin_sender_name": message.forward_origin_sender_name,
                "forwarded_at": message.forwarded_at,
                "message_metadata": normalized_whatsapp_message_metadata(
                    &message.text,
                    &message.message_metadata,
                ),
                "delivery_state": message.delivery_state.as_str(),
            }),
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": session.companion_runtime,
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        self.update_session_last_sync(&message.account_id, message.occurred_at)
            .await?;

        Ok(WhatsappWebObservedMessage { raw })
    }

    pub async fn ingest_fixture_reaction(
        &self,
        reaction: &NewWhatsappWebReaction,
    ) -> Result<WhatsappWebObservedReaction, WhatsappWebError> {
        reaction.validate()?;
        let context = self.fixture_ingest_context(&reaction.account_id).await?;
        let provider_record_id = reaction.provider_record_id();
        let raw_record_id = whatsapp_web_raw_record_id(
            &reaction.account_id,
            WHATSAPP_WEB_REACTION_RECORD_KIND,
            &provider_record_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &reaction.account_id,
            WHATSAPP_WEB_REACTION_RECORD_KIND,
            &provider_record_id,
            reaction.source_fingerprint(),
            &reaction.import_batch_id,
            json!({
                "provider_chat_id": reaction.provider_chat_id,
                "provider_message_id": reaction.provider_message_id,
                "provider_actor_id": reaction.provider_actor_id,
                "sender_display_name": reaction.sender_display_name,
                "reaction": reaction.reaction,
                "is_active": reaction.is_active,
            }),
        )
        .occurred_at(reaction.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": reaction.account_id,
            "provider_chat_id": reaction.provider_chat_id,
        }));
        self.update_session_last_sync(&reaction.account_id, reaction.observed_at)
            .await?;

        Ok(WhatsappWebObservedReaction { raw })
    }

    pub async fn ingest_fixture_media(
        &self,
        media: &NewWhatsappWebMedia,
    ) -> Result<WhatsappWebObservedMedia, WhatsappWebError> {
        media.validate()?;
        let context = self.fixture_ingest_context(&media.account_id).await?;
        let provider_record_id = media.provider_record_id();
        let raw_record_id = whatsapp_web_raw_record_id(
            &media.account_id,
            WHATSAPP_WEB_MEDIA_RECORD_KIND,
            &provider_record_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &media.account_id,
            WHATSAPP_WEB_MEDIA_RECORD_KIND,
            &provider_record_id,
            media.source_fingerprint(),
            &media.import_batch_id,
            json!({
                "provider_chat_id": media.provider_chat_id,
                "provider_message_id": media.provider_message_id,
                "provider_attachment_id": media.provider_attachment_id,
                "filename": media.filename,
                "content_type": media.content_type,
                "size_bytes": media.size_bytes,
                "sha256": media.sha256,
                "storage_kind": media.storage_kind,
                "storage_path": media.storage_path,
            }),
        )
        .occurred_at(media.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": media.account_id,
            "provider_chat_id": media.provider_chat_id,
        }));
        self.update_session_last_sync(&media.account_id, media.observed_at)
            .await?;

        Ok(WhatsappWebObservedMedia { raw })
    }

    pub async fn ingest_fixture_status(
        &self,
        status: &NewWhatsappWebStatus,
    ) -> Result<WhatsappWebObservedStatus, WhatsappWebError> {
        status.validate()?;
        let context = self.fixture_ingest_context(&status.account_id).await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &status.account_id,
            WHATSAPP_WEB_STATUS_RECORD_KIND,
            &status.provider_status_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &status.account_id,
            WHATSAPP_WEB_STATUS_RECORD_KIND,
            &status.provider_status_id,
            status.source_fingerprint(),
            &status.import_batch_id,
            json!({
                "provider_status_id": status.provider_status_id,
                "sender_id": status.sender_id,
                "sender_display_name": status.sender_display_name,
                "sender_identity_kind": status.sender_identity_kind,
                "sender_address": status.sender_address,
                "sender_push_name": status.sender_push_name,
                "sender_business_profile": status.sender_business_profile,
                "sender_profile_photo_ref": status.sender_profile_photo_ref,
                "text": status.text,
                "delivery_state": "received",
            }),
        )
        .occurred_at(status.occurred_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": status.account_id,
            "provider_status_id": status.provider_status_id,
        }));
        self.update_session_last_sync(&status.account_id, status.occurred_at)
            .await?;

        Ok(WhatsappWebObservedStatus { raw })
    }

    pub async fn ingest_fixture_status_view(
        &self,
        status_view: &NewWhatsappWebStatusView,
    ) -> Result<WhatsappWebObservedStatusView, WhatsappWebError> {
        status_view.validate()?;
        let context = self.fixture_ingest_context(&status_view.account_id).await?;
        let provider_record_id = status_view.provider_record_id();
        let raw_record_id = whatsapp_web_raw_record_id(
            &status_view.account_id,
            WHATSAPP_WEB_STATUS_VIEW_RECORD_KIND,
            &provider_record_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &status_view.account_id,
            WHATSAPP_WEB_STATUS_VIEW_RECORD_KIND,
            &provider_record_id,
            status_view.source_fingerprint(),
            &status_view.import_batch_id,
            json!({
                "provider_status_id": status_view.provider_status_id,
                "viewer_id": status_view.viewer_id,
                "viewer_display_name": status_view.viewer_display_name,
            }),
        )
        .occurred_at(status_view.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": status_view.account_id,
            "provider_status_id": status_view.provider_status_id,
        }));
        self.update_session_last_sync(&status_view.account_id, status_view.observed_at)
            .await?;

        Ok(WhatsappWebObservedStatusView { raw })
    }

    pub async fn ingest_fixture_status_delete(
        &self,
        status_delete: &NewWhatsappWebStatusDelete,
    ) -> Result<WhatsappWebObservedStatusDelete, WhatsappWebError> {
        status_delete.validate()?;
        let context = self
            .fixture_ingest_context(&status_delete.account_id)
            .await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &status_delete.account_id,
            WHATSAPP_WEB_STATUS_DELETE_RECORD_KIND,
            &status_delete.provider_status_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &status_delete.account_id,
            WHATSAPP_WEB_STATUS_DELETE_RECORD_KIND,
            &status_delete.provider_status_id,
            status_delete.source_fingerprint(),
            &status_delete.import_batch_id,
            json!({
                "provider_status_id": status_delete.provider_status_id,
                "actor_class": status_delete.actor_class,
                "reason_class": status_delete.reason_class,
            }),
        )
        .occurred_at(status_delete.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": status_delete.account_id,
            "provider_status_id": status_delete.provider_status_id,
        }));
        self.update_session_last_sync(&status_delete.account_id, status_delete.observed_at)
            .await?;

        Ok(WhatsappWebObservedStatusDelete { raw })
    }

    pub async fn ingest_fixture_presence(
        &self,
        presence: &NewWhatsappWebPresence,
    ) -> Result<WhatsappWebObservedPresence, WhatsappWebError> {
        presence.validate()?;
        let context = self.fixture_ingest_context(&presence.account_id).await?;
        let provider_record_id = presence.provider_record_id();
        let raw_record_id = whatsapp_web_raw_record_id(
            &presence.account_id,
            WHATSAPP_WEB_PRESENCE_RECORD_KIND,
            &provider_record_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &presence.account_id,
            WHATSAPP_WEB_PRESENCE_RECORD_KIND,
            &provider_record_id,
            presence.source_fingerprint(),
            &presence.import_batch_id,
            json!({
                "provider_chat_id": presence.provider_chat_id,
                "provider_identity_id": presence.provider_identity_id,
                "identity_kind": presence.identity_kind,
                "display_name": presence.display_name,
                "presence_state": presence.presence_state,
                "last_seen_at": presence.last_seen_at,
            }),
        )
        .occurred_at(presence.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": presence.account_id,
            "provider_chat_id": presence.provider_chat_id,
        }));
        self.update_session_last_sync(&presence.account_id, presence.observed_at)
            .await?;

        Ok(WhatsappWebObservedPresence { raw })
    }

    pub async fn ingest_fixture_dialog(
        &self,
        dialog: &NewWhatsappWebDialog,
    ) -> Result<WhatsappWebObservedDialog, WhatsappWebError> {
        dialog.validate()?;
        let context = self.fixture_ingest_context(&dialog.account_id).await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &dialog.account_id,
            WHATSAPP_WEB_DIALOG_RECORD_KIND,
            &dialog.provider_chat_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &dialog.account_id,
            WHATSAPP_WEB_DIALOG_RECORD_KIND,
            &dialog.provider_chat_id,
            dialog.source_fingerprint(),
            &dialog.import_batch_id,
            json!({
                "provider_chat_id": dialog.provider_chat_id,
                "chat_title": dialog.chat_title,
                "chat_kind": dialog.chat_kind,
                "is_archived": dialog.is_archived,
                "is_pinned": dialog.is_pinned,
                "is_muted": dialog.is_muted,
                "is_unread": dialog.is_unread,
                "community_parent_chat_id": dialog.community_parent_chat_id,
                "community_parent_title": dialog.community_parent_title,
                "invite_link": dialog.invite_link,
                "is_community_root": dialog.is_community_root,
                "is_broadcast": dialog.is_broadcast,
                "is_newsletter": dialog.is_newsletter,
                "unread_count": dialog.unread_count,
                "participant_count": dialog.participant_count,
                "avatar_metadata": dialog.avatar_metadata,
                "provider_labels": dialog.provider_labels,
            }),
        )
        .occurred_at(dialog.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": dialog.account_id,
            "provider_chat_id": dialog.provider_chat_id,
        }));
        self.update_session_last_sync(&dialog.account_id, dialog.observed_at)
            .await?;

        Ok(WhatsappWebObservedDialog { raw })
    }

    pub async fn ingest_fixture_participant(
        &self,
        participant: &NewWhatsappWebParticipant,
    ) -> Result<WhatsappWebObservedParticipant, WhatsappWebError> {
        participant.validate()?;
        let context = self.fixture_ingest_context(&participant.account_id).await?;
        let provider_record_id = participant.provider_record_id();
        let provider_member_id = participant.effective_provider_member_id();
        let raw_record_id = whatsapp_web_raw_record_id(
            &participant.account_id,
            WHATSAPP_WEB_PARTICIPANT_RECORD_KIND,
            &provider_record_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &participant.account_id,
            WHATSAPP_WEB_PARTICIPANT_RECORD_KIND,
            &provider_record_id,
            participant.source_fingerprint(),
            &participant.import_batch_id,
            json!({
                "provider_chat_id": participant.provider_chat_id,
                "chat_title": participant.chat_title,
                "chat_kind": participant.chat_kind,
                "provider_member_id": provider_member_id,
                "provider_identity_id": participant.provider_identity_id,
                "identity_kind": participant.identity_kind,
                "display_name": participant.display_name,
                "push_name": participant.push_name,
                "address": participant.address,
                "business_profile": participant.business_profile,
                "profile_photo_ref": participant.profile_photo_ref,
                "role": participant.role,
                "status": participant.status,
                "is_self": participant.is_self,
                "is_admin": participant.is_admin,
                "is_owner": participant.is_owner,
            }),
        )
        .occurred_at(participant.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": participant.account_id,
            "provider_chat_id": participant.provider_chat_id,
        }));
        self.update_session_last_sync(&participant.account_id, participant.observed_at)
            .await?;

        Ok(WhatsappWebObservedParticipant { raw })
    }

    pub async fn ingest_fixture_message_update(
        &self,
        update: &NewWhatsappWebMessageUpdate,
    ) -> Result<WhatsappWebObservedMessageUpdate, WhatsappWebError> {
        update.validate()?;
        let context = self.fixture_ingest_context(&update.account_id).await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &update.account_id,
            WHATSAPP_WEB_MESSAGE_UPDATE_RECORD_KIND,
            &update.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &update.account_id,
            WHATSAPP_WEB_MESSAGE_UPDATE_RECORD_KIND,
            &update.provider_message_id,
            update.source_fingerprint(),
            &update.import_batch_id,
            json!({
                "provider_chat_id": update.provider_chat_id,
                "provider_message_id": update.provider_message_id,
                "text": update.text,
            }),
        )
        .occurred_at(update.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": update.account_id,
            "provider_chat_id": update.provider_chat_id,
        }));
        self.update_session_last_sync(&update.account_id, update.observed_at)
            .await?;

        Ok(WhatsappWebObservedMessageUpdate { raw })
    }

    pub async fn ingest_fixture_message_delete(
        &self,
        delete: &NewWhatsappWebMessageDelete,
    ) -> Result<WhatsappWebObservedMessageDelete, WhatsappWebError> {
        delete.validate()?;
        let context = self.fixture_ingest_context(&delete.account_id).await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &delete.account_id,
            WHATSAPP_WEB_MESSAGE_DELETE_RECORD_KIND,
            &delete.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &delete.account_id,
            WHATSAPP_WEB_MESSAGE_DELETE_RECORD_KIND,
            &delete.provider_message_id,
            delete.source_fingerprint(),
            &delete.import_batch_id,
            json!({
                "provider_chat_id": delete.provider_chat_id,
                "provider_message_id": delete.provider_message_id,
                "reason_class": delete.reason_class,
                "actor_class": delete.actor_class,
            }),
        )
        .occurred_at(delete.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": delete.account_id,
            "provider_chat_id": delete.provider_chat_id,
        }));
        self.update_session_last_sync(&delete.account_id, delete.observed_at)
            .await?;

        Ok(WhatsappWebObservedMessageDelete { raw })
    }

    pub async fn ingest_fixture_receipt(
        &self,
        receipt: &NewWhatsappWebReceipt,
    ) -> Result<WhatsappWebObservedReceipt, WhatsappWebError> {
        receipt.validate()?;
        let context = self.fixture_ingest_context(&receipt.account_id).await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &receipt.account_id,
            WHATSAPP_WEB_RECEIPT_RECORD_KIND,
            &receipt.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &receipt.account_id,
            WHATSAPP_WEB_RECEIPT_RECORD_KIND,
            &receipt.provider_message_id,
            receipt.source_fingerprint(),
            &receipt.import_batch_id,
            json!({
                "provider_chat_id": receipt.provider_chat_id,
                "provider_message_id": receipt.provider_message_id,
                "delivery_state": receipt.delivery_state.as_str(),
            }),
        )
        .occurred_at(receipt.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": receipt.account_id,
            "provider_chat_id": receipt.provider_chat_id,
        }));
        self.update_session_last_sync(&receipt.account_id, receipt.observed_at)
            .await?;

        Ok(WhatsappWebObservedReceipt { raw })
    }

    pub async fn ingest_fixture_call(
        &self,
        call: &NewWhatsappWebCall,
    ) -> Result<WhatsappWebObservedCall, WhatsappWebError> {
        call.validate()?;
        let context = self.fixture_ingest_context(&call.account_id).await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &call.account_id,
            WHATSAPP_WEB_CALL_RECORD_KIND,
            &call.provider_call_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &call.account_id,
            WHATSAPP_WEB_CALL_RECORD_KIND,
            &call.provider_call_id,
            call.source_fingerprint(),
            &call.import_batch_id,
            json!({
                "provider_call_id": call.provider_call_id,
                "provider_chat_id": call.provider_chat_id,
                "direction": call.direction,
                "call_state": call.call_state,
                "started_at": call.started_at,
                "ended_at": call.ended_at,
                "metadata": call.metadata,
            }),
        )
        .occurred_at(call.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": call.account_id,
            "provider_chat_id": call.provider_chat_id,
        }));
        self.update_session_last_sync(&call.account_id, call.observed_at)
            .await?;

        Ok(WhatsappWebObservedCall { raw })
    }

    pub async fn ingest_fixture_runtime_event(
        &self,
        runtime_event: &NewWhatsappWebRuntimeEvent,
    ) -> Result<WhatsappWebObservedRuntimeEvent, WhatsappWebError> {
        runtime_event.validate()?;
        let context = self
            .fixture_ingest_context(&runtime_event.account_id)
            .await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &runtime_event.account_id,
            WHATSAPP_WEB_RUNTIME_EVENT_RECORD_KIND,
            &runtime_event.provider_event_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &runtime_event.account_id,
            WHATSAPP_WEB_RUNTIME_EVENT_RECORD_KIND,
            &runtime_event.provider_event_id,
            runtime_event.source_fingerprint(),
            &runtime_event.import_batch_id,
            json!({
                "provider_event_id": runtime_event.provider_event_id,
                "runtime_event_kind": runtime_event.runtime_event_kind,
                "runtime_status": runtime_event.effective_runtime_status(),
                "lifecycle_state": runtime_event.effective_lifecycle_state(),
                "severity": runtime_event.effective_severity(),
                "metadata": redact_secret_material(runtime_event.metadata.clone()),
            }),
        )
        .occurred_at(runtime_event.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": runtime_event.account_id,
        }));
        self.update_session_last_sync(&runtime_event.account_id, runtime_event.observed_at)
            .await?;

        Ok(WhatsappWebObservedRuntimeEvent { raw })
    }

    async fn fixture_ingest_context(
        &self,
        account_id: &str,
    ) -> Result<FixtureIngestContext, WhatsappWebError> {
        let provider_account = self
            .provider_account_store()
            .get(account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{account_id}` is not configured"
                ))
            })?;
        if !provider_account.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "account `{account_id}` is not a WhatsApp Web provider account"
            )));
        }
        let session = self
            .list_sessions(Some(account_id), 1)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{account_id}` has no session metadata"
                ))
            })?;
        Ok(FixtureIngestContext {
            provider_kind: provider_account.provider_kind.as_str().to_owned(),
            runtime: session.companion_runtime,
        })
    }
}

struct FixtureIngestContext {
    provider_kind: String,
    runtime: String,
}

fn normalized_whatsapp_message_metadata(text: &str, metadata: &Value) -> Value {
    let mut normalized = metadata.as_object().cloned().unwrap_or_default();

    let mention_usernames = mention_usernames_from_metadata(metadata);
    if !mention_usernames.is_empty() {
        normalized.insert(
            "mention_count".to_owned(),
            json!(i64::try_from(mention_usernames.len()).unwrap_or(0)),
        );
        normalized.insert("mention_usernames".to_owned(), json!(mention_usernames));
    }

    if let Some(poll) = metadata.get("poll") {
        normalized.insert("whatsapp_poll".to_owned(), poll.clone());
    }
    if let Some(location) = metadata.get("location") {
        normalized.insert("whatsapp_location".to_owned(), location.clone());
    }
    if let Some(contact_card) = metadata.get("contact_card") {
        normalized.insert("whatsapp_contact_card".to_owned(), contact_card.clone());
    }
    if let Some(join_leave) = metadata.get("join_leave") {
        normalized.insert("whatsapp_join_leave".to_owned(), join_leave.clone());
    }
    if let Some(sticker) = metadata.get("sticker") {
        normalized.insert("whatsapp_sticker".to_owned(), sticker.clone());
    }
    if let Some(system_message) = metadata.get("system_message") {
        normalized.insert("whatsapp_system_message".to_owned(), system_message.clone());
    }
    if let Some(ephemeral) = metadata.get("ephemeral") {
        normalized.insert("whatsapp_ephemeral".to_owned(), ephemeral.clone());
    }
    if let Some(view_once) = metadata.get("view_once") {
        normalized.insert("whatsapp_view_once".to_owned(), view_once.clone());
    }
    if let Some(link_preview) = derive_whatsapp_link_preview(metadata) {
        normalized.insert("whatsapp_link_preview".to_owned(), link_preview);
    }
    if text.trim().is_empty() {
        normalized.insert("body_text_empty".to_owned(), Value::Bool(true));
    }

    Value::Object(normalized)
}

fn redact_secret_material(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    if is_secret_like_key(&key) {
                        (key, Value::String("[redacted]".to_owned()))
                    } else {
                        (key, redact_secret_material(value))
                    }
                })
                .collect::<Map<String, Value>>(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_material).collect())
        }
        other => other,
    }
}

fn is_secret_like_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "access_token"
            | "refresh_token"
            | "session_key"
            | "session_material"
            | "authorization"
            | "cookie"
            | "token"
            | "secret"
            | "secret_key"
            | "password"
    )
}

fn mention_usernames_from_metadata(metadata: &Value) -> Vec<String> {
    metadata
        .get("mentions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|mention| {
            mention
                .get("username")
                .and_then(Value::as_str)
                .or_else(|| mention.get("display_name").and_then(Value::as_str))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn derive_whatsapp_link_preview(metadata: &Value) -> Option<Value> {
    if let Some(link_preview) = metadata.get("link_preview") {
        return Some(link_preview.clone());
    }

    let first_link = metadata
        .get("links")
        .and_then(Value::as_array)?
        .first()?
        .clone();
    let mut preview = Map::new();
    preview.insert(
        "url".to_owned(),
        first_link.get("url").cloned().unwrap_or(Value::Null),
    );
    if let Some(title) = first_link.get("title").cloned() {
        preview.insert("title".to_owned(), title);
    }
    if let Some(description) = first_link.get("description").cloned() {
        preview.insert("description".to_owned(), description);
    }
    Some(Value::Object(preview))
}
