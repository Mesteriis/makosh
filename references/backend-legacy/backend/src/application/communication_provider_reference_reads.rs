use std::collections::HashMap;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::message_references::TelegramMessageReferenceSummary;
use makosh_communications_api::canonical::CanonicalMessageReadPort;

pub(crate) async fn list_reference_summaries(
    reads: &dyn CanonicalMessageReadPort,
    message_ids: Vec<String>,
) -> Result<HashMap<String, TelegramMessageReferenceSummary>, TelegramError> {
    if message_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut summaries = HashMap::new();
    for row in reads
        .list_message_reference_summaries(&message_ids)
        .await
        .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?
    {
        let message_id = row.message_id;
        summaries.insert(
            message_id.clone(),
            TelegramMessageReferenceSummary {
                message_id,
                provider_message_id: row.provider_message_id,
                provider_chat_id: Some(row.provider_chat_id),
                chat_title: row.chat_title.unwrap_or_default(),
                sender: row.sender.unwrap_or_default(),
                sender_display_name: row.sender_display_name,
                text: row.text.unwrap_or_default(),
                occurred_at: Some(row.occurred_at),
            },
        );
    }
    Ok(summaries)
}
