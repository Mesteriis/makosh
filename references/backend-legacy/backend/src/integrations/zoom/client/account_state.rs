use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::super::models::{ZoomAccount, ZoomTranscriptObservationRequest};
use super::auth::validate_account_id;
use super::{ZoomError, ZoomStore};
use crate::platform::calls::models::{CallDirection, CallState, NewProviderCall};
use makosh_provider_zoom::protocol::ZOOM_PROVIDER_KIND_STR;

impl ZoomStore {
    pub(super) async fn zoom_account(&self, account_id: &str) -> Result<ZoomAccount, ZoomError> {
        let account_id = validate_account_id(account_id)?;
        let account = self
            .provider_account_store
            .get(&account_id)
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!("Zoom account `{account_id}` was not found"))
            })?;
        if !account.provider_kind.is_zoom() {
            return Err(ZoomError::InvalidRequest(format!(
                "account `{account_id}` is not a Zoom provider account"
            )));
        }
        Ok(account.into())
    }

    pub(super) async fn ensure_zoom_account(&self, account_id: &str) -> Result<(), ZoomError> {
        self.zoom_account(account_id).await.map(|_| ())
    }

    pub(super) async fn update_account_config(
        &self,
        account_id: &str,
        config: Value,
    ) -> Result<ZoomAccount, ZoomError> {
        let account = self
            .provider_account_store
            .update_config(account_id, &config)
            .await?
            .ok_or_else(|| {
                ZoomError::InvalidRequest(format!("Zoom account `{account_id}` was not found"))
            })?;
        Ok(account.into())
    }

    pub(super) async fn ensure_placeholder_call(
        &self,
        request: &ZoomTranscriptObservationRequest,
        call_id: String,
        observed_at: DateTime<Utc>,
    ) -> Result<(), ZoomError> {
        let call_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM telegram_calls WHERE account_id = $1 AND provider_call_id = $2)")
            .bind(request.account_id.trim()).bind(request.meeting_id.trim()).fetch_one(&self.pool).await?;
        if call_exists {
            return Ok(());
        }
        let call = NewProviderCall {
            call_id,
            account_id: request.account_id.trim().to_owned(),
            provider_call_id: request.meeting_id.trim().to_owned(),
            provider_chat_id: request.provider_chat_id(),
            direction: CallDirection::Outgoing,
            call_state: CallState::Ended,
            started_at: None,
            ended_at: None,
            transcription_policy_id: None,
            metadata: json!({"provider":"zoom", "provider_kind": ZOOM_PROVIDER_KIND_STR, "meeting_id": &request.meeting_id, "meeting_uuid": &request.meeting_uuid, "observed_at": observed_at, "placeholder": true}),
        };
        self.call_store
            .upsert_call(&call)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}
