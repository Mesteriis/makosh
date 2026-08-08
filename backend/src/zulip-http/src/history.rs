use makosh_zulip_api::ZulipHistoryPageV1;
use serde_json::Value;

use crate::{
    ZulipHttpConfigV1,
    command::request_for_message_history,
    event_queue::message_snapshot,
    wire::{ZulipHttpErrorV1, execute_value},
};

pub async fn fetch_page(
    config: &ZulipHttpConfigV1,
    before_provider_message_id: Option<&str>,
    limit: u32,
) -> Result<ZulipHistoryPageV1, ZulipHttpErrorV1> {
    let (_, value) = execute_value(
        config,
        request_for_message_history(config, before_provider_message_id, limit)?,
    )
    .await?;
    let messages = value
        .get("messages")
        .and_then(Value::as_array)
        .ok_or(ZulipHttpErrorV1::Protocol)?
        .iter()
        .map(|message| message_snapshot(config, message))
        .collect::<Result<Vec<_>, _>>()?;
    let oldest_provider_message_id = messages
        .iter()
        .filter_map(|message| {
            message
                .provider_message_id
                .parse::<i64>()
                .ok()
                .map(|id| (id, message.provider_message_id.clone()))
        })
        .min_by_key(|(id, _)| *id)
        .map(|(_, id)| id);
    Ok(ZulipHistoryPageV1 {
        messages,
        oldest_provider_message_id,
        found_oldest: value
            .get("found_oldest")
            .and_then(Value::as_bool)
            .ok_or(ZulipHttpErrorV1::Protocol)?,
    })
}

#[cfg(test)]
mod tests {
    use makosh_zulip_api::ZulipAccountV1;
    use serde_json::json;

    use super::*;

    #[test]
    fn maps_history_message_as_raw_markdown_snapshot() {
        let config = ZulipHttpConfigV1::new(
            ZulipAccountV1 {
                account_id: "account".into(),
                realm_url: "https://zulip.test/".into(),
                account_email: "account@zulip.test".into(),
            },
            "secret".into(),
        )
        .expect("config");
        let message = message_snapshot(
            &config,
            &json!({
                "id": 7,
                "stream_id": 2,
                "display_recipient": "engineering",
                "subject": "decisions",
                "sender_id": 3,
                "sender_email": "person@zulip.test",
                "content": "**raw markdown**",
                "timestamp": 11,
                "reactions": [{
                    "user_id": 4,
                    "emoji_name": "thumbs_up",
                    "emoji_code": "1f44d",
                    "reaction_type": "unicode_emoji"
                }]
            }),
        )
        .expect("message");
        assert_eq!(message.provider_conversation_id, "stream:2:decisions");
        assert_eq!(message.stream_name.as_deref(), Some("engineering"));
        assert_eq!(message.content.as_deref(), Some("**raw markdown**"));
        assert_eq!(message.reactions.len(), 1);
    }
}
