use super::*;

use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;

#[derive(Serialize)]
pub(crate) struct ProviderCommandRetryResponse {
    command_id: String,
    status: String,
    retry_count: i32,
    max_retries: i32,
    reconciliation_status: String,
    next_attempt_at: Option<DateTime<Utc>>,
}

/// Requeues only failed/dead-letter Mail commands; local message truth is unchanged.
pub(crate) async fn post_v1_provider_command_retry(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
) -> Result<Json<ProviderCommandRetryResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let command = CommunicationProviderCommandStore::new(pool)
        .manual_retry(&command_id, "mail", Utc::now())
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(ProviderCommandRetryResponse {
        command_id: command.command_id,
        status: command.status,
        retry_count: command.retry_count,
        max_retries: command.max_retries,
        reconciliation_status: command.reconciliation_status,
        next_attempt_at: command.next_attempt_at,
    }))
}
