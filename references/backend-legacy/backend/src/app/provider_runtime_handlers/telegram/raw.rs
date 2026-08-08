use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::Value;

use crate::app::api_support::stores::domain_stores::communication_ingestion_store;
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore;
use crate::integrations::telegram::client::errors::TelegramError;
use makosh_communications_api::evidence::StoredRawCommunicationRecord;

const COMMUNICATION_RAW_EVIDENCE_CHANNEL_KINDS: &[&str] =
    &["telegram_user", "telegram_bot", "whatsapp_web"];

#[derive(Serialize)]
pub(crate) struct TelegramRawMessageResponse {
    pub(crate) raw_record: TelegramRawMessageRecord,
}

#[derive(Serialize)]
pub(crate) struct TelegramRawMessageRecord {
    pub(crate) raw_record_id: String,
    pub(crate) account_id: String,
    pub(crate) record_kind: String,
    pub(crate) provider_record_id: String,
    pub(crate) source_fingerprint: String,
    pub(crate) import_batch_id: String,
    pub(crate) occurred_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) captured_at: chrono::DateTime<chrono::Utc>,
    pub(crate) payload: Value,
    pub(crate) provenance: Value,
}

impl From<StoredRawCommunicationRecord> for TelegramRawMessageRecord {
    fn from(record: StoredRawCommunicationRecord) -> Self {
        Self {
            raw_record_id: record.raw_record_id,
            account_id: record.account_id,
            record_kind: record.record_kind,
            provider_record_id: record.provider_record_id,
            source_fingerprint: record.source_fingerprint,
            import_batch_id: record.import_batch_id,
            occurred_at: record.occurred_at,
            captured_at: record.captured_at,
            payload: redact_secret_material(record.payload),
            provenance: redact_secret_material(record.provenance),
        }
    }
}

/// GET /api/v1/communications/messages/{message_id}/raw-evidence
pub(crate) async fn get_telegram_message_raw(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramRawMessageResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let Some(message) = ProviderChannelMessageStore::new(pool)
        .message_by_id(&message_id, COMMUNICATION_RAW_EVIDENCE_CHANNEL_KINDS)
        .await
        .map_err(TelegramError::from)?
    else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let Some(raw_record) = communication_ingestion_store(&state)?
        .raw_record(&message.raw_record_id)
        .await?
    else {
        return Err(ApiError::CommunicationMessageNotFound);
    };

    Ok(Json(TelegramRawMessageResponse {
        raw_record: raw_record.into(),
    }))
}

fn redact_secret_material(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    if is_secret_key(&key) {
                        (key, Value::String("[redacted]".to_owned()))
                    } else {
                        (key, redact_secret_material(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_material).collect())
        }
        other => other,
    }
}

fn is_secret_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "access_token"
            | "api_hash"
            | "authorization"
            | "auth"
            | "bot_token"
            | "client_secret"
            | "cookie"
            | "cookies"
            | "credential"
            | "credentials"
            | "password"
            | "private_key"
            | "proxy_password"
            | "refresh_token"
            | "secret"
            | "session"
            | "session_blob"
            | "session_cookie"
            | "session_encryption_key"
            | "session_key"
            | "token"
    ) || normalized.ends_with("_token")
        || normalized.ends_with("_cookie")
        || normalized.ends_with("_credentials")
        || normalized.ends_with("_private_key")
}
