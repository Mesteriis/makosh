use chrono::{DateTime, Utc};
use makosh_communications_api::projection_state::{
    MessageProjectionState, MessageProjectionStateError, MessageProjectionStateFuture,
    MessageProjectionStateQueryPort,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct CommunicationMessageProjectionStateQuery {
    pool: PgPool,
}

impl CommunicationMessageProjectionStateQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl MessageProjectionStateQueryPort for CommunicationMessageProjectionStateQuery {
    fn state<'a>(&'a self, message_id: &'a str) -> MessageProjectionStateFuture<'a> {
        Box::pin(async move {
            let row = sqlx::query("SELECT message_id,workflow_state,local_state,local_state_changed_at,local_state_reason,importance_score,ai_category,ai_summary,ai_summary_generated_at,is_read,read_changed_at,read_origin FROM communication_messages WHERE message_id=$1")
                .bind(message_id).fetch_optional(&self.pool).await.map_err(|e| MessageProjectionStateError(e.to_string()))?;
            row.map(|row| {
                Ok(MessageProjectionState {
                    message_id: row.try_get("message_id").map_err(error)?,
                    workflow_state: row.try_get("workflow_state").map_err(error)?,
                    local_state: row.try_get("local_state").map_err(error)?,
                    local_state_changed_at: row
                        .try_get::<Option<DateTime<Utc>>, _>("local_state_changed_at")
                        .map_err(error)?,
                    local_state_reason: row.try_get("local_state_reason").map_err(error)?,
                    importance_score: row.try_get("importance_score").map_err(error)?,
                    ai_category: row.try_get("ai_category").map_err(error)?,
                    ai_summary: row.try_get("ai_summary").map_err(error)?,
                    ai_summary_generated_at: row
                        .try_get::<Option<DateTime<Utc>>, _>("ai_summary_generated_at")
                        .map_err(error)?,
                    is_read: row.try_get("is_read").map_err(error)?,
                    read_changed_at: row
                        .try_get::<Option<DateTime<Utc>>, _>("read_changed_at")
                        .map_err(error)?,
                    read_origin: row.try_get("read_origin").map_err(error)?,
                })
            })
            .transpose()
        })
    }
}

fn error(error: sqlx::Error) -> MessageProjectionStateError {
    MessageProjectionStateError(error.to_string())
}
