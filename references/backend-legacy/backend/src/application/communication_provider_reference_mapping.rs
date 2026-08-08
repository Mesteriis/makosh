use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::message_references::{
    TelegramForwardRef, TelegramReplyRef,
};
use makosh_communications_api::canonical::{
    CanonicalForwardReferenceRecord, CanonicalReplyReferenceRecord,
};

pub(crate) fn map_reply_reference(
    rows: Vec<CanonicalReplyReferenceRecord>,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    Ok(rows
        .into_iter()
        .map(|row| TelegramReplyRef {
            reply_ref_id: row.reply_ref_id,
            source_message_id: row.source_message_id,
            target_message_id: row.target_message_id,
            account_id: row.account_id,
            provider_chat_id: row.provider_chat_id,
            source_provider_id: row.source_provider_id,
            target_provider_id: row.target_provider_id,
            reply_depth: row.reply_depth,
            is_topic_reply: row.is_topic_reply,
            topic_id: row.topic_id,
            source_message_summary: None,
            target_message_summary: None,
            metadata: row.metadata,
            provenance: row.provenance,
            created_at: row.created_at,
        })
        .collect())
}

pub(crate) fn map_forward_reference(
    rows: Vec<CanonicalForwardReferenceRecord>,
) -> Result<Vec<TelegramForwardRef>, TelegramError> {
    Ok(rows
        .into_iter()
        .map(|row| {
            let mut metadata = row.metadata;
            if let Some(target_message_id) = row.target_message_id {
                metadata["target_message_id"] = serde_json::json!(target_message_id);
            }
            TelegramForwardRef {
                forward_ref_id: row.forward_ref_id,
                source_message_id: row.source_message_id,
                account_id: row.account_id,
                provider_chat_id: row.provider_chat_id,
                source_provider_id: row.source_provider_id,
                forward_origin_chat_id: row.forward_origin_chat_id,
                forward_origin_message_id: row.forward_origin_message_id,
                forward_origin_sender_id: row.forward_origin_sender_id,
                forward_origin_sender_name: row.forward_origin_sender_name,
                forward_date: row.forward_date,
                forward_depth: row.forward_depth,
                source_message_summary: None,
                metadata,
                provenance: row.provenance,
                created_at: row.created_at,
            }
        })
        .collect())
}
