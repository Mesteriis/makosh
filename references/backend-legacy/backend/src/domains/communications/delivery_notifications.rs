use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::messages::provider_observation_projection::COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER;
use super::outbox::CommunicationOutboxStore;
use super::outbox::delivery_status::{
    NewOutboxDeliveryStatus, OutboxDeliveryStatus, OutboxDeliveryStatusRecord,
};
use super::read_receipts::{
    CommunicationReadReceipt, CommunicationReadReceiptStore, NewCommunicationReadReceipt,
};

const MAX_NOTIFICATION_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Deserialize)]
pub struct NewCommunicationDeliveryNotification {
    pub account_id: String,
    pub raw_message: String,
    pub received_at: Option<DateTime<Utc>>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderDeliveryEventKind {
    Delivered,
    Delayed,
    Failed,
    Read,
}

impl ProviderDeliveryEventKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Delayed => "delayed",
            Self::Failed => "failed",
            Self::Read => "read",
        }
    }

    fn delivery_status(self) -> Option<OutboxDeliveryStatus> {
        match self {
            Self::Delivered => Some(OutboxDeliveryStatus::Delivered),
            Self::Delayed => Some(OutboxDeliveryStatus::Delayed),
            Self::Failed => Some(OutboxDeliveryStatus::Failed),
            Self::Read => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewProviderDeliveryEvent {
    pub account_id: String,
    pub provider_message_id: String,
    pub event_kind: ProviderDeliveryEventKind,
    pub recipient: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub source_kind: Option<String>,
    pub smtp_status: Option<String>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationDeliveryNotificationKind {
    DeliveryStatus,
    ReadReceipt,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationDeliveryNotificationRecord {
    pub notification_kind: CommunicationDeliveryNotificationKind,
    pub account_id: String,
    pub outbox_id: Option<String>,
    pub provider_message_id: String,
    pub delivery_status: Option<OutboxDeliveryStatus>,
    pub smtp_status: Option<String>,
    pub source_kind: String,
    pub read_receipt: Option<CommunicationReadReceipt>,
}

#[derive(Clone)]
pub struct CommunicationDeliveryNotificationStore {
    pool: PgPool,
}

impl CommunicationDeliveryNotificationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        notification: NewCommunicationDeliveryNotification,
    ) -> Result<CommunicationDeliveryNotificationRecord, CommunicationDeliveryNotificationError>
    {
        let account_id = normalize_required("account_id", &notification.account_id)?;
        let received_at = notification.received_at.unwrap_or_else(Utc::now);
        let provider_record_id = normalize_optional(notification.provider_record_id);
        let raw_record_id = normalize_optional(notification.raw_record_id);
        let parsed = parse_delivery_notification(&notification.raw_message)?;

        match parsed {
            ParsedDeliveryNotification::DeliveryStatus {
                provider_message_id,
                delivery_status,
                smtp_status,
            } => {
                let record = CommunicationOutboxStore::new(self.pool.clone())
                    .record_delivery_status(&NewOutboxDeliveryStatus {
                        account_id,
                        provider_message_id,
                        delivery_status,
                        smtp_status,
                        source_kind: "dsn".to_owned(),
                        provider_record_id,
                        raw_record_id,
                        recorded_at: received_at,
                    })
                    .await?;
                Ok(delivery_status_response(record))
            }
            ParsedDeliveryNotification::ReadReceipt {
                provider_message_id,
                recipient,
                reporting_ua,
            } => {
                let receipt = CommunicationReadReceiptStore::new(self.pool.clone())
                    .record(NewCommunicationReadReceipt {
                        receipt_id: None,
                        account_id,
                        provider_message_id,
                        recipient,
                        read_at: received_at,
                        source_kind: Some("mdn".to_owned()),
                        provider_record_id,
                        raw_record_id,
                        metadata: Some(json!({ "reporting_ua": reporting_ua })),
                    })
                    .await?;
                Ok(read_receipt_response(receipt))
            }
        }
    }

    pub async fn record_provider_event(
        &self,
        event: NewProviderDeliveryEvent,
    ) -> Result<CommunicationDeliveryNotificationRecord, CommunicationDeliveryNotificationError>
    {
        let account_id = normalize_required("account_id", &event.account_id)?;
        let provider_message_id =
            normalize_required("provider_message_id", &event.provider_message_id)?;
        let occurred_at = event.occurred_at.unwrap_or_else(Utc::now);
        let source_kind =
            normalize_optional(event.source_kind).unwrap_or_else(|| "provider_event".to_owned());
        let provider_record_id = normalize_optional(event.provider_record_id);
        let raw_record_id = normalize_optional(event.raw_record_id);

        if let Some(delivery_status) = event.event_kind.delivery_status() {
            let record = CommunicationOutboxStore::new(self.pool.clone())
                .record_delivery_status(&NewOutboxDeliveryStatus {
                    account_id,
                    provider_message_id,
                    delivery_status,
                    smtp_status: normalize_optional(event.smtp_status),
                    source_kind,
                    provider_record_id,
                    raw_record_id,
                    recorded_at: occurred_at,
                })
                .await?;
            return Ok(delivery_status_response(record));
        }

        let recipient = normalize_optional(event.recipient)
            .ok_or(CommunicationDeliveryNotificationError::Invalid("recipient"))?;
        let mut metadata = json!({ "provider_event_kind": event.event_kind.as_str() });
        if let Some(extra) = event.metadata
            && let (Some(target), Some(extra_map)) = (metadata.as_object_mut(), extra.as_object())
        {
            for (key, value) in extra_map {
                target.insert(key.clone(), value.clone());
            }
        }
        let receipt = CommunicationReadReceiptStore::new(self.pool.clone())
            .record(NewCommunicationReadReceipt {
                receipt_id: None,
                account_id,
                provider_message_id,
                recipient,
                read_at: occurred_at,
                source_kind: Some(source_kind),
                provider_record_id,
                raw_record_id,
                metadata: Some(metadata),
            })
            .await?;

        Ok(read_receipt_response(receipt))
    }
}

pub fn provider_event_from_delivery_notification(
    notification: &NewCommunicationDeliveryNotification,
) -> Result<NewProviderDeliveryEvent, CommunicationDeliveryNotificationError> {
    let account_id = normalize_required("account_id", &notification.account_id)?;
    let occurred_at = notification.received_at.unwrap_or_else(Utc::now);
    let provider_record_id = normalize_optional(notification.provider_record_id.clone());
    let raw_record_id = normalize_optional(notification.raw_record_id.clone());
    let parsed = parse_delivery_notification(&notification.raw_message)?;

    match parsed {
        ParsedDeliveryNotification::DeliveryStatus {
            provider_message_id,
            delivery_status,
            smtp_status,
        } => Ok(NewProviderDeliveryEvent {
            account_id,
            provider_message_id,
            event_kind: delivery_status_provider_event_kind(delivery_status),
            recipient: None,
            occurred_at: Some(occurred_at),
            source_kind: Some("dsn".to_owned()),
            smtp_status,
            provider_record_id,
            raw_record_id,
            metadata: None,
        }),
        ParsedDeliveryNotification::ReadReceipt {
            provider_message_id,
            recipient,
            reporting_ua,
        } => Ok(NewProviderDeliveryEvent {
            account_id,
            provider_message_id,
            event_kind: ProviderDeliveryEventKind::Read,
            recipient: Some(recipient),
            occurred_at: Some(occurred_at),
            source_kind: Some("mdn".to_owned()),
            smtp_status: None,
            provider_record_id,
            raw_record_id,
            metadata: Some(json!({ "reporting_ua": reporting_ua })),
        }),
    }
}

pub async fn project_accepted_mail_delivery_signal_if_runtime_allows(
    pool: PgPool,
    event: &makosh_events_api::EventEnvelope,
) -> Result<Option<CommunicationDeliveryNotificationRecord>, CommunicationDeliveryNotificationError>
{
    if !crate::platform::events::runtime::runtime_allows_processing(
        &pool,
        "system",
        COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        &json!({
            "label": "Communications accepted-signal consumer",
            "scope": "consumer",
        }),
    )
    .await?
    {
        return Ok(None);
    }

    consume_accepted_mail_delivery_signal(pool, event).await
}

pub async fn consume_accepted_mail_delivery_signal(
    pool: PgPool,
    event: &makosh_events_api::EventEnvelope,
) -> Result<Option<CommunicationDeliveryNotificationRecord>, CommunicationDeliveryNotificationError>
{
    let Some(provider_event) = provider_event_from_accepted_signal(event)? else {
        return Ok(None);
    };

    CommunicationDeliveryNotificationStore::new(pool)
        .record_provider_event(provider_event)
        .await
        .map(Some)
}

fn delivery_status_response(
    record: OutboxDeliveryStatusRecord,
) -> CommunicationDeliveryNotificationRecord {
    CommunicationDeliveryNotificationRecord {
        notification_kind: CommunicationDeliveryNotificationKind::DeliveryStatus,
        account_id: record.account_id,
        outbox_id: record.outbox_id,
        provider_message_id: record.provider_message_id,
        delivery_status: Some(record.delivery_status),
        smtp_status: record.smtp_status,
        source_kind: record.source_kind,
        read_receipt: None,
    }
}

fn read_receipt_response(
    receipt: CommunicationReadReceipt,
) -> CommunicationDeliveryNotificationRecord {
    CommunicationDeliveryNotificationRecord {
        notification_kind: CommunicationDeliveryNotificationKind::ReadReceipt,
        account_id: receipt.account_id.clone(),
        outbox_id: receipt.outbox_id.clone(),
        provider_message_id: receipt.provider_message_id.clone(),
        delivery_status: None,
        smtp_status: None,
        source_kind: receipt.source_kind.clone(),
        read_receipt: Some(receipt),
    }
}

fn provider_event_from_accepted_signal(
    event: &makosh_events_api::EventEnvelope,
) -> Result<Option<NewProviderDeliveryEvent>, CommunicationDeliveryNotificationError> {
    match event.event_type.as_str() {
        "signal.accepted.mail.delivery_status" => Ok(Some(NewProviderDeliveryEvent {
            account_id: required_payload_str(&event.payload, "account_id")?,
            provider_message_id: required_payload_str(&event.payload, "provider_message_id")?,
            event_kind: provider_event_kind_from_str(
                required_payload_str(&event.payload, "event_kind")?.as_str(),
            )?,
            recipient: None,
            occurred_at: Some(payload_occurred_at(&event.payload).unwrap_or(event.occurred_at)),
            source_kind: normalize_optional(
                event.payload.get("source_kind").and_then(|v| v.as_str()),
            ),
            smtp_status: normalize_optional(
                event.payload.get("smtp_status").and_then(|v| v.as_str()),
            ),
            provider_record_id: normalize_optional(
                event
                    .payload
                    .get("provider_record_id")
                    .and_then(|v| v.as_str()),
            ),
            raw_record_id: normalize_optional(
                event.payload.get("raw_record_id").and_then(|v| v.as_str()),
            ),
            metadata: None,
        })),
        "signal.accepted.mail.read_receipt" => Ok(Some(NewProviderDeliveryEvent {
            account_id: required_payload_str(&event.payload, "account_id")?,
            provider_message_id: required_payload_str(&event.payload, "provider_message_id")?,
            event_kind: ProviderDeliveryEventKind::Read,
            recipient: Some(required_payload_str(&event.payload, "recipient")?),
            occurred_at: Some(payload_occurred_at(&event.payload).unwrap_or(event.occurred_at)),
            source_kind: normalize_optional(
                event.payload.get("source_kind").and_then(|v| v.as_str()),
            ),
            smtp_status: None,
            provider_record_id: normalize_optional(
                event
                    .payload
                    .get("provider_record_id")
                    .and_then(|v| v.as_str()),
            ),
            raw_record_id: normalize_optional(
                event.payload.get("raw_record_id").and_then(|v| v.as_str()),
            ),
            metadata: Some(json!({
                "reporting_ua": normalize_optional(
                    event.payload.get("reporting_ua").and_then(|v| v.as_str())
                ),
            })),
        })),
        _ => Ok(None),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ParsedDeliveryNotification {
    DeliveryStatus {
        provider_message_id: String,
        delivery_status: OutboxDeliveryStatus,
        smtp_status: Option<String>,
    },
    ReadReceipt {
        provider_message_id: String,
        recipient: String,
        reporting_ua: Option<String>,
    },
}

fn parse_delivery_notification(
    raw_message: &str,
) -> Result<ParsedDeliveryNotification, CommunicationDeliveryNotificationError> {
    if raw_message.trim().is_empty() {
        return Err(CommunicationDeliveryNotificationError::Invalid(
            "raw_message",
        ));
    }
    if raw_message.len() > MAX_NOTIFICATION_BYTES {
        return Err(CommunicationDeliveryNotificationError::Invalid(
            "raw_message",
        ));
    }

    let fields = unfolded_fields(raw_message);
    if let Some(disposition) = first_field(&fields, "disposition")
        && disposition.to_ascii_lowercase().contains("displayed")
    {
        return Ok(ParsedDeliveryNotification::ReadReceipt {
            provider_message_id: original_message_id(&fields)?,
            recipient: recipient_from_fields(&fields)?,
            reporting_ua: first_field(&fields, "reporting-ua"),
        });
    }

    let action = first_field(&fields, "action")
        .ok_or(CommunicationDeliveryNotificationError::Invalid("action"))?;
    Ok(ParsedDeliveryNotification::DeliveryStatus {
        provider_message_id: original_message_id(&fields)?,
        delivery_status: delivery_status_from_action(&action)?,
        smtp_status: first_field(&fields, "status")
            .and_then(|value| normalize_optional(Some(value))),
    })
}

fn delivery_status_provider_event_kind(
    delivery_status: OutboxDeliveryStatus,
) -> ProviderDeliveryEventKind {
    match delivery_status {
        OutboxDeliveryStatus::Delivered => ProviderDeliveryEventKind::Delivered,
        OutboxDeliveryStatus::Delayed => ProviderDeliveryEventKind::Delayed,
        OutboxDeliveryStatus::Failed => ProviderDeliveryEventKind::Failed,
    }
}

fn provider_event_kind_from_str(
    value: &str,
) -> Result<ProviderDeliveryEventKind, CommunicationDeliveryNotificationError> {
    match value.trim() {
        "delivered" => Ok(ProviderDeliveryEventKind::Delivered),
        "delayed" => Ok(ProviderDeliveryEventKind::Delayed),
        "failed" => Ok(ProviderDeliveryEventKind::Failed),
        "read" => Ok(ProviderDeliveryEventKind::Read),
        _ => Err(CommunicationDeliveryNotificationError::Invalid(
            "event_kind",
        )),
    }
}

fn unfolded_fields(raw_message: &str) -> Vec<(String, String)> {
    let normalized = raw_message.replace("\r\n", "\n");
    let mut fields: Vec<(String, String)> = Vec::new();

    for line in normalized.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some((_, value)) = fields.last_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if !name.is_empty() {
            fields.push((name, value.trim().to_owned()));
        }
    }

    fields
}

fn original_message_id(
    fields: &[(String, String)],
) -> Result<String, CommunicationDeliveryNotificationError> {
    first_field(fields, "original-message-id")
        .or_else(|| first_field(fields, "message-id"))
        .and_then(|value| normalize_message_id(&value))
        .ok_or(CommunicationDeliveryNotificationError::Invalid(
            "original-message-id",
        ))
}

fn recipient_from_fields(
    fields: &[(String, String)],
) -> Result<String, CommunicationDeliveryNotificationError> {
    first_field(fields, "final-recipient")
        .or_else(|| first_field(fields, "original-recipient"))
        .and_then(|value| recipient_from_report_field(&value))
        .ok_or(CommunicationDeliveryNotificationError::Invalid(
            "final-recipient",
        ))
}

fn delivery_status_from_action(
    action: &str,
) -> Result<OutboxDeliveryStatus, CommunicationDeliveryNotificationError> {
    match action.trim().to_ascii_lowercase().as_str() {
        "delivered" | "relayed" | "expanded" => Ok(OutboxDeliveryStatus::Delivered),
        "delayed" => Ok(OutboxDeliveryStatus::Delayed),
        "failed" => Ok(OutboxDeliveryStatus::Failed),
        _ => Err(CommunicationDeliveryNotificationError::Invalid("action")),
    }
}

fn first_field(fields: &[(String, String)], name: &str) -> Option<String> {
    fields
        .iter()
        .find(|(field_name, _)| field_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_message_id(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value
        .strip_prefix('<')
        .and_then(|stripped| stripped.strip_suffix('>'))
        .unwrap_or(value)
        .trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn recipient_from_report_field(value: &str) -> Option<String> {
    let recipient = value
        .split_once(';')
        .map(|(_, recipient)| recipient)
        .unwrap_or(value)
        .trim();
    normalize_optional(Some(recipient))
}

fn normalize_required(
    field_name: &'static str,
    value: &str,
) -> Result<String, CommunicationDeliveryNotificationError> {
    normalize_optional(Some(value))
        .ok_or(CommunicationDeliveryNotificationError::Invalid(field_name))
}

fn normalize_optional(value: Option<impl AsRef<str>>) -> Option<String> {
    value
        .map(|value| value.as_ref().trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn required_payload_str(
    payload: &serde_json::Value,
    field_name: &'static str,
) -> Result<String, CommunicationDeliveryNotificationError> {
    payload
        .get(field_name)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(CommunicationDeliveryNotificationError::Invalid(field_name))
}

fn payload_occurred_at(payload: &serde_json::Value) -> Option<DateTime<Utc>> {
    payload
        .get("occurred_at")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
}

#[derive(Debug, Error)]
pub enum CommunicationDeliveryNotificationError {
    #[error(transparent)]
    Outbox(#[from] super::outbox::CommunicationOutboxError),
    #[error(transparent)]
    ReadReceipt(#[from] super::read_receipts::CommunicationReadReceiptError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    SignalControlBlocked(String),
    #[error("invalid mail delivery notification field: {0}")]
    Invalid(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dsn_delivery_status() {
        let parsed = parse_delivery_notification(
            "Original-Message-ID: <provider-1>\r\nFinal-Recipient: rfc822; user@example.com\r\nAction: failed\r\nStatus: 5.1.1\r\n",
        )
        .expect("parse dsn");

        assert_eq!(
            parsed,
            ParsedDeliveryNotification::DeliveryStatus {
                provider_message_id: "provider-1".to_owned(),
                delivery_status: OutboxDeliveryStatus::Failed,
                smtp_status: Some("5.1.1".to_owned())
            }
        );
    }

    #[test]
    fn parse_mdn_read_receipt() {
        let parsed = parse_delivery_notification(
            "Original-Message-ID: <provider-2>\r\nFinal-Recipient: rfc822; reader@example.com\r\nDisposition: automatic-action/MDN-sent-automatically; displayed\r\nReporting-UA: fixture\r\n",
        )
        .expect("parse mdn");

        assert_eq!(
            parsed,
            ParsedDeliveryNotification::ReadReceipt {
                provider_message_id: "provider-2".to_owned(),
                recipient: "reader@example.com".to_owned(),
                reporting_ua: Some("fixture".to_owned())
            }
        );
    }
}
