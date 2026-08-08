use chrono::{DateTime, Utc};
use makosh_communications_api::projections::{
    MessageProjectionInput, MessageProjectionRead, MessageProjectionWriteError,
    MessageProjectionWriteFuture, MessageProjectionWritePort,
};
use serde_json::Value;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct CommunicationMessageProjectionStore {
    pool: PgPool,
}

impl CommunicationMessageProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl MessageProjectionWritePort for CommunicationMessageProjectionStore {
    fn upsert<'a>(&'a self, input: &'a MessageProjectionInput) -> MessageProjectionWriteFuture<'a> {
        Box::pin(async move {
            if input.message_id.trim().is_empty()
                || input.raw_record_id.trim().is_empty()
                || input.account_id.trim().is_empty()
                || input.provider_record_id.trim().is_empty()
            {
                return Err(MessageProjectionWriteError::InvalidInput(
                    "message, raw record, account and provider ids are required",
                ));
            }
            let row = sqlx::query("INSERT INTO communication_messages (message_id,raw_record_id,observation_id,account_id,provider_record_id,subject,sender,recipients,body_text,occurred_at,channel_kind,conversation_id,sender_display_name,delivery_state,message_metadata,is_read,read_changed_at,read_origin) SELECT $1,raw_record_id,observation_id,account_id,provider_record_id,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,CASE WHEN COALESCE($14->'label_ids', '[]'::jsonb) ? 'UNREAD' THEN false WHEN jsonb_typeof($14->'label_ids') = 'array' THEN true ELSE COALESCE(($14->>'is_read')::boolean, false) END,now(),'provider_observed' FROM communication_raw_records WHERE raw_record_id=$2 AND account_id=$3 AND provider_record_id=$4 ON CONFLICT (account_id,provider_record_id) DO UPDATE SET message_id=EXCLUDED.message_id,raw_record_id=EXCLUDED.raw_record_id,observation_id=EXCLUDED.observation_id,subject=EXCLUDED.subject,sender=EXCLUDED.sender,recipients=EXCLUDED.recipients,body_text=EXCLUDED.body_text,occurred_at=EXCLUDED.occurred_at,channel_kind=EXCLUDED.channel_kind,conversation_id=EXCLUDED.conversation_id,sender_display_name=EXCLUDED.sender_display_name,delivery_state=EXCLUDED.delivery_state,message_metadata=EXCLUDED.message_metadata,is_read=CASE WHEN communication_messages.read_origin='local_user' THEN communication_messages.is_read ELSE EXCLUDED.is_read END,read_changed_at=CASE WHEN communication_messages.read_origin='local_user' THEN communication_messages.read_changed_at ELSE EXCLUDED.read_changed_at END,read_origin=CASE WHEN communication_messages.read_origin='local_user' THEN communication_messages.read_origin ELSE EXCLUDED.read_origin END,projected_at=now() RETURNING message_id,raw_record_id,observation_id,account_id,provider_record_id,subject,sender,recipients,body_text,occurred_at,projected_at,channel_kind,conversation_id,sender_display_name,delivery_state,message_metadata")
                .bind(&input.message_id).bind(&input.raw_record_id).bind(&input.account_id).bind(&input.provider_record_id)
                .bind(&input.subject).bind(&input.sender).bind(&input.recipients).bind(&input.body_text).bind(input.occurred_at)
                .bind(&input.channel_kind).bind(&input.conversation_id).bind(&input.sender_display_name).bind(&input.delivery_state).bind(&input.metadata)
                .fetch_optional(&self.pool).await.map_err(|e| MessageProjectionWriteError::Failed(e.to_string()))?
                .ok_or_else(|| MessageProjectionWriteError::Failed("raw communication record was not found".to_owned()))?;
            Ok(MessageProjectionRead {
                message_id: row.try_get("message_id").map_err(fail)?,
                raw_record_id: row.try_get("raw_record_id").map_err(fail)?,
                observation_id: row.try_get("observation_id").map_err(fail)?,
                account_id: row.try_get("account_id").map_err(fail)?,
                provider_record_id: row.try_get("provider_record_id").map_err(fail)?,
                subject: row.try_get("subject").map_err(fail)?,
                sender: row.try_get("sender").map_err(fail)?,
                recipients: row.try_get("recipients").map_err(fail)?,
                body_text: row.try_get("body_text").map_err(fail)?,
                occurred_at: row
                    .try_get::<Option<DateTime<Utc>>, _>("occurred_at")
                    .map_err(fail)?,
                projected_at: row.try_get("projected_at").map_err(fail)?,
                channel_kind: row.try_get("channel_kind").map_err(fail)?,
                conversation_id: row.try_get("conversation_id").map_err(fail)?,
                sender_display_name: row.try_get("sender_display_name").map_err(fail)?,
                delivery_state: row.try_get("delivery_state").map_err(fail)?,
                metadata: row.try_get::<Value, _>("message_metadata").map_err(fail)?,
            })
        })
    }
}

fn fail(error: sqlx::Error) -> MessageProjectionWriteError {
    MessageProjectionWriteError::Failed(error.to_string())
}
