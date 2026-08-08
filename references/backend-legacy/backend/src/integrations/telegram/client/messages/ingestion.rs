use serde_json::{Value, json};

use makosh_communications_api::evidence::NewRawCommunicationRecord;

use super::super::TELEGRAM_MESSAGE_RECORD_KIND;
use super::super::errors::TelegramError;
use super::super::identifiers::{stable_hash, telegram_raw_record_id};
use super::super::models::chats::{NewTelegramChat, TelegramSyncState};
use super::super::models::messages::{NewTelegramMessage, TelegramObservedMessage};
use super::super::store::TelegramStore;
use super::message_metadata::{
    derive_mention_metadata, derive_tdlib_attachment_metadata, derive_tdlib_media_album_metadata,
    derive_tdlib_structured_evidence, telegram_public_message_link,
};
use super::reaction_metadata::{
    derive_tdlib_chosen_reaction_emojis, derive_tdlib_provider_reactions,
    derive_tdlib_reaction_summary_metadata,
};

impl TelegramStore {
    pub async fn ingest_fixture_message(
        &self,
        message: &NewTelegramMessage,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        self.observe_message_with_runtime(message, "fixture", None)
            .await
    }

    pub(in crate::integrations::telegram::client::messages) async fn ingest_message_with_runtime(
        &self,
        message: &NewTelegramMessage,
        runtime_kind: &str,
        tdlib_raw: Option<Value>,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        self.observe_message_with_runtime(message, runtime_kind, tdlib_raw)
            .await
    }

    pub(in crate::integrations::telegram::client::messages) async fn observe_message_with_runtime(
        &self,
        message: &NewTelegramMessage,
        runtime_kind: &str,
        tdlib_raw: Option<Value>,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        message.validate_for_runtime(runtime_kind)?;
        let provider_account = self.telegram_provider_account(&message.account_id).await?;

        let chat = NewTelegramChat {
            account_id: message.account_id.clone(),
            provider_chat_id: message.provider_chat_id.clone(),
            chat_kind: message.chat_kind,
            title: message.chat_title.clone(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: Some(message.occurred_at),
            metadata: json!({"runtime": runtime_kind}),
        };
        let chat = self.upsert_chat(&chat).await?;

        let mention_metadata = derive_mention_metadata(&message.text, tdlib_raw.as_ref());
        let public_message_link =
            telegram_public_message_link(chat.username.as_deref(), &message.provider_message_id);
        let tdlib_media_album = tdlib_raw
            .as_ref()
            .and_then(|raw| derive_tdlib_media_album_metadata(raw, &message.provider_chat_id));
        let tdlib_attachments = tdlib_raw
            .as_ref()
            .map(derive_tdlib_attachment_metadata)
            .unwrap_or_default();
        let tdlib_structured_evidence = tdlib_raw
            .as_ref()
            .map(derive_tdlib_structured_evidence)
            .unwrap_or_default();
        let tdlib_reaction_summary = tdlib_raw
            .as_ref()
            .and_then(derive_tdlib_reaction_summary_metadata);
        let tdlib_provider_reactions = tdlib_raw
            .as_ref()
            .map(derive_tdlib_provider_reactions)
            .unwrap_or_default();
        let tdlib_chosen_reactions = tdlib_raw
            .as_ref()
            .map(derive_tdlib_chosen_reaction_emojis)
            .unwrap_or_default();
        let mut payload = json!({
            "provider_chat_id": message.provider_chat_id,
            "chat_title": message.chat_title,
            "chat_kind": message.chat_kind.as_str(),
            "sender_id": message.sender_id,
            "sender_display_name": message.sender_display_name,
            "text": message.text,
            "delivery_state": message.delivery_state.as_str(),
            "mention_count": mention_metadata.count,
            "mentions": mention_metadata.mentions,
            "mentions_detected_by": mention_metadata.detected_by,
        });
        if let Some(payload) = payload.as_object_mut() {
            if let Some(link) = public_message_link {
                payload.insert("message_link".to_owned(), Value::String(link));
                payload.insert(
                    "message_link_kind".to_owned(),
                    Value::String("public_t_me".to_owned()),
                );
            }
            if let Some((album_id, album_key)) = tdlib_media_album {
                payload.insert("media_album_id".to_owned(), Value::String(album_id));
                payload.insert("media_album_key".to_owned(), Value::String(album_key));
            }
            if !tdlib_attachments.is_empty() {
                payload.insert("attachments".to_owned(), Value::Array(tdlib_attachments));
            }
            if let Some(reaction_summary) = tdlib_reaction_summary {
                payload.insert("reaction_summary".to_owned(), reaction_summary);
            }
            for (key, value) in tdlib_structured_evidence {
                payload.insert(key, value);
            }
            if let Some(tdlib_raw) = tdlib_raw {
                payload.insert("tdlib_raw".to_owned(), tdlib_raw);
            }
        }
        let raw_record_id = telegram_raw_record_id(
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            payload,
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "telegram",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": runtime_kind,
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        let _ = (
            provider_account.external_account_id,
            tdlib_provider_reactions,
            tdlib_chosen_reactions,
        );

        let message_id = format!(
            "message:v4:telegram:{}",
            stable_hash(
                [
                    message.account_id.as_str(),
                    message.provider_message_id.as_str()
                ]
                .join("\0")
                .as_bytes()
            )
        );

        Ok(TelegramObservedMessage {
            raw_record_id,
            message_id,
            raw,
            telegram_chat_id: chat.telegram_chat_id,
        })
    }
}
