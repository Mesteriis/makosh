use chrono::Utc;

use super::super::errors::TelegramError;
use super::super::identifiers::{telegram_account_runtime, telegram_text_preview_hash};
use super::super::models::chats::TelegramChatKind;
use super::super::models::messages::{
    NewTelegramMessage, TelegramDeliveryState, TelegramManualSendRequest,
    TelegramManualSendResponse,
};
use super::super::store::TelegramStore;

impl TelegramStore {
    pub async fn manual_send_message(
        &self,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError> {
        request.validate()?;
        let provider_account = self.telegram_provider_account(&request.account_id).await?;
        let runtime_kind = telegram_account_runtime(&provider_account);
        if runtime_kind != "fixture" {
            return Err(TelegramError::InvalidRequest(
                "manual live Telegram sends require an enabled TDLib actor".to_owned(),
            ));
        }

        let chat = self
            .telegram_chat(&request.account_id, &request.provider_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{}` is not synced for account `{}`",
                    request.provider_chat_id, request.account_id
                ))
            })?;
        let provider_message_id = format!("manual:{}", request.command_id.trim());
        let rendered_preview_hash = telegram_text_preview_hash(&request.text);
        let message = NewTelegramMessage {
            account_id: request.account_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id.trim().to_owned(),
            provider_message_id,
            chat_kind: TelegramChatKind::try_from(chat.chat_kind.as_str())?,
            chat_title: chat.title,
            sender_id: "makosh".to_owned(),
            sender_display_name: "Макошь".to_owned(),
            text: request.text.trim().to_owned(),
            import_batch_id: format!("telegram-manual-send:{}", request.command_id.trim()),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Sent,
        };
        let result = self.ingest_fixture_message(&message).await?;

        Ok(TelegramManualSendResponse {
            raw: Some(result.raw),
            raw_record_id: result.raw_record_id,
            message_id: result.message_id,
            account_id: request.account_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id.trim().to_owned(),
            delivery_state: TelegramDeliveryState::Sent.as_str().to_owned(),
            status: "sent".to_owned(),
            runtime_kind,
            rendered_preview_hash,
        })
    }
}
