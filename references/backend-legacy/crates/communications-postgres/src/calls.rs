use makosh_communications_api::calls::{
    CanonicalCallReadError, CanonicalCallReadPort, CanonicalCallRecord,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct CanonicalCallReadStore {
    pool: PgPool,
}
impl CanonicalCallReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CanonicalCallReadPort for CanonicalCallReadStore {
    async fn list_whatsapp_calls(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CanonicalCallRecord>, CanonicalCallReadError> {
        let rows = sqlx::query(r#"SELECT call_id, account_id, provider_call_id, provider_chat_id, direction, call_state, started_at, ended_at, metadata FROM telegram_calls WHERE account_id = $1 AND metadata->>'provider' = 'whatsapp_web' AND ($2::text IS NULL OR provider_chat_id = $2) ORDER BY COALESCE(started_at, created_at) DESC, call_id ASC LIMIT $3"#)
            .bind(account_id.trim()).bind(provider_chat_id.map(str::trim).filter(|v| !v.is_empty())).bind(limit.clamp(1, 200)).fetch_all(&self.pool).await.map_err(error)?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalCallRecord {
                    call_id: row.try_get("call_id").map_err(error)?,
                    account_id: row.try_get("account_id").map_err(error)?,
                    provider_call_id: row.try_get("provider_call_id").map_err(error)?,
                    provider_chat_id: row.try_get("provider_chat_id").map_err(error)?,
                    direction: row.try_get("direction").map_err(error)?,
                    call_state: row.try_get("call_state").map_err(error)?,
                    started_at: row.try_get("started_at").map_err(error)?,
                    ended_at: row.try_get("ended_at").map_err(error)?,
                    metadata: row.try_get("metadata").map_err(error)?,
                })
            })
            .collect()
    }
}
fn error(error: sqlx::Error) -> CanonicalCallReadError {
    CanonicalCallReadError(error.to_string())
}
