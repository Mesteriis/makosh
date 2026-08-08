use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use makosh_events_postgres::store::EventStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

const EVENT_TYPE_RECORDED: &str = "mail.read_receipt.recorded";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationReadReceiptKind {
    Read,
}

impl CommunicationReadReceiptKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
        }
    }
}

impl TryFrom<&str> for CommunicationReadReceiptKind {
    type Error = CommunicationReadReceiptError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "read" => Ok(Self::Read),
            _ => Err(CommunicationReadReceiptError::Invalid("receipt_kind")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationReadReceipt {
    pub receipt_id: String,
    pub account_id: String,
    pub outbox_id: Option<String>,
    pub provider_message_id: String,
    pub recipient: String,
    pub receipt_kind: CommunicationReadReceiptKind,
    pub read_at: DateTime<Utc>,
    pub source_kind: String,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewCommunicationReadReceipt {
    pub receipt_id: Option<String>,
    pub account_id: String,
    pub provider_message_id: String,
    pub recipient: String,
    pub read_at: DateTime<Utc>,
    pub source_kind: Option<String>,
    pub provider_record_id: Option<String>,
    pub raw_record_id: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Clone)]
pub struct CommunicationReadReceiptStore {
    pool: PgPool,
}

impl CommunicationReadReceiptStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        receipt: NewCommunicationReadReceipt,
    ) -> Result<CommunicationReadReceipt, CommunicationReadReceiptError> {
        let normalized = NormalizedCommunicationReadReceipt::from_new(receipt)?;
        let mut transaction = self.pool.begin().await?;
        ensure_canonical_account_in_transaction(
            &mut transaction,
            Some(normalized.account_id.as_str()),
        )
        .await?;
        let outbox_id = correlate_outbox(
            &mut transaction,
            &normalized.account_id,
            &normalized.provider_message_id,
        )
        .await?;
        let receipt = insert_receipt(&mut transaction, &normalized, outbox_id.as_deref()).await?;
        capture_read_receipt_observation(&mut transaction, &receipt).await?;
        let event = read_receipt_event(&receipt)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(receipt)
    }
}

async fn capture_read_receipt_observation(
    transaction: &mut Transaction<'_, Postgres>,
    receipt: &CommunicationReadReceipt,
) -> Result<(), CommunicationReadReceiptError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "COMMUNICATION_READ_RECEIPT",
            ObservationOriginKind::LocalRuntime,
            receipt.read_at,
            json!({
                "receipt_id": receipt.receipt_id,
                "account_id": receipt.account_id,
                "outbox_id": receipt.outbox_id,
                "provider_message_id": receipt.provider_message_id,
                "recipient": receipt.recipient,
                "receipt_kind": receipt.receipt_kind.as_str(),
                "read_at": receipt.read_at,
                "source_kind": receipt.source_kind,
                "provider_record_id": receipt.provider_record_id,
                "raw_record_id": receipt.raw_record_id,
                "operation": "read_receipt_recorded",
            }),
            format!("read-receipt://{}", receipt.receipt_id),
        )
        .provenance(json!({
            "captured_by": "mail.read_receipts",
            "operation": "read_receipt_recorded",
        })),
    )
    .await?;
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "read_receipt",
        receipt.receipt_id.clone(),
        "read_receipt_recorded",
        json!({
            "receipt_kind": receipt.receipt_kind.as_str(),
            "source_kind": receipt.source_kind,
        }),
        None,
    )
    .await?;
    if let Some(outbox_id) = &receipt.outbox_id {
        link_mail_entity_in_transaction(
            transaction,
            &observation.observation_id,
            "outbox_item",
            outbox_id.clone(),
            "read_receipt_observed",
            json!({
                "receipt_kind": receipt.receipt_kind.as_str(),
                "source_kind": receipt.source_kind,
            }),
            None,
        )
        .await?;
    }
    link_mail_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_message",
        receipt.provider_message_id.clone(),
        "read_receipt_observed",
        json!({
            "receipt_kind": receipt.receipt_kind.as_str(),
            "source_kind": receipt.source_kind,
        }),
        None,
    )
    .await?;
    Ok(())
}

#[derive(Debug)]
struct NormalizedCommunicationReadReceipt {
    receipt_id: String,
    account_id: String,
    provider_message_id: String,
    recipient: String,
    read_at: DateTime<Utc>,
    source_kind: String,
    provider_record_id: Option<String>,
    raw_record_id: Option<String>,
    metadata: Value,
}

impl NormalizedCommunicationReadReceipt {
    fn from_new(
        receipt: NewCommunicationReadReceipt,
    ) -> Result<Self, CommunicationReadReceiptError> {
        let account_id = normalize_required("account_id", &receipt.account_id)?;
        let provider_message_id =
            normalize_required("provider_message_id", &receipt.provider_message_id)?;
        let provider_record_id = normalize_optional(receipt.provider_record_id)?;
        let receipt_id = match receipt.receipt_id {
            Some(value) => normalize_required("receipt_id", &value)?,
            None => generate_receipt_id(&account_id, provider_record_id.as_deref()),
        };
        let metadata = receipt.metadata.unwrap_or_else(|| json!({}));
        if !metadata.is_object() {
            return Err(CommunicationReadReceiptError::Invalid("metadata"));
        }

        Ok(Self {
            receipt_id,
            account_id,
            provider_message_id,
            recipient: normalize_required("recipient", &receipt.recipient)?,
            read_at: receipt.read_at,
            source_kind: normalize_optional(receipt.source_kind)?
                .unwrap_or_else(|| "mdn".to_owned()),
            provider_record_id,
            raw_record_id: normalize_optional(receipt.raw_record_id)?,
            metadata,
        })
    }
}

async fn correlate_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
    provider_message_id: &str,
) -> Result<Option<String>, CommunicationReadReceiptError> {
    Ok(sqlx::query_scalar::<_, String>(
        r#"
        SELECT outbox_id
        FROM communication_outbox
        WHERE account_id = $1
          AND provider_message_id = $2
        ORDER BY sent_at DESC NULLS LAST, created_at DESC, outbox_id ASC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .bind(provider_message_id)
    .fetch_optional(&mut **transaction)
    .await?)
}

async fn insert_receipt(
    transaction: &mut Transaction<'_, Postgres>,
    receipt: &NormalizedCommunicationReadReceipt,
    outbox_id: Option<&str>,
) -> Result<CommunicationReadReceipt, CommunicationReadReceiptError> {
    let row = sqlx::query(
        r#"
        INSERT INTO communication_read_receipts (
            receipt_id,
            account_id,
            outbox_id,
            provider_message_id,
            recipient,
            receipt_kind,
            read_at,
            source_kind,
            provider_record_id,
            raw_record_id,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, 'read', $6, $7, $8, $9, $10)
        RETURNING
            receipt_id,
            account_id,
            outbox_id,
            provider_message_id,
            recipient,
            receipt_kind,
            read_at,
            source_kind,
            provider_record_id,
            raw_record_id,
            metadata,
            created_at
        "#,
    )
    .bind(&receipt.receipt_id)
    .bind(&receipt.account_id)
    .bind(outbox_id)
    .bind(&receipt.provider_message_id)
    .bind(&receipt.recipient)
    .bind(receipt.read_at)
    .bind(&receipt.source_kind)
    .bind(receipt.provider_record_id.as_deref())
    .bind(receipt.raw_record_id.as_deref())
    .bind(&receipt.metadata)
    .fetch_one(&mut **transaction)
    .await?;

    row_to_receipt(row)
}

async fn ensure_canonical_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Option<&str>,
) -> Result<(), CommunicationReadReceiptError> {
    let Some(account_id) = account_id else {
        return Ok(());
    };

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id, config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            '{}'::jsonb,
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

fn row_to_receipt(row: PgRow) -> Result<CommunicationReadReceipt, CommunicationReadReceiptError> {
    let receipt_kind: String = row.try_get("receipt_kind")?;
    Ok(CommunicationReadReceipt {
        receipt_id: row.try_get("receipt_id")?,
        account_id: row.try_get("account_id")?,
        outbox_id: row.try_get("outbox_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        recipient: row.try_get("recipient")?,
        receipt_kind: CommunicationReadReceiptKind::try_from(receipt_kind.as_str())?,
        read_at: row.try_get("read_at")?,
        source_kind: row.try_get("source_kind")?,
        provider_record_id: row.try_get("provider_record_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}

fn read_receipt_event(
    receipt: &CommunicationReadReceipt,
) -> Result<NewEventEnvelope, CommunicationReadReceiptError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_read_receipt_event:{}:{}",
            receipt.receipt_id,
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        EVENT_TYPE_RECORDED,
        Utc::now(),
        json!({ "kind": "mail_read_receipt_api" }),
        json!({
            "kind": "mail_read_receipt",
            "id": receipt.receipt_id,
            "account_id": receipt.account_id,
            "outbox_id": receipt.outbox_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(json!({
        "receipt_id": receipt.receipt_id,
        "account_id": receipt.account_id,
        "outbox_id": receipt.outbox_id,
        "provider_message_id": receipt.provider_message_id,
        "receipt_kind": receipt.receipt_kind.as_str(),
        "read_at": receipt.read_at,
        "source_kind": receipt.source_kind,
    }))
    .provenance(json!({
        "source_kind": receipt.source_kind,
        "source_id": receipt.provider_record_id,
    }))
    .correlation_id(
        receipt
            .outbox_id
            .clone()
            .unwrap_or_else(|| receipt.receipt_id.clone()),
    )
    .build()?)
}

fn normalize_required(
    field: &'static str,
    value: &str,
) -> Result<String, CommunicationReadReceiptError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CommunicationReadReceiptError::Invalid(field));
    }
    Ok(value.to_owned())
}

fn normalize_optional(
    value: Option<String>,
) -> Result<Option<String>, CommunicationReadReceiptError> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn generate_receipt_id(account_id: &str, provider_record_id: Option<&str>) -> String {
    match provider_record_id {
        Some(provider_record_id) => format!("mail_read_receipt:{account_id}:{provider_record_id}"),
        None => format!(
            "mail_read_receipt:{account_id}:{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
    }
}

#[derive(Debug, Error)]
pub enum CommunicationReadReceiptError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    EventStore(#[from] makosh_events_postgres::errors::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
    #[error("invalid mail read receipt field: {0}")]
    Invalid(&'static str),
}
