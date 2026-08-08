use super::super::*;
use makosh_communications_api::commands::CommunicationProviderCommandDiagnostics;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;

#[derive(Deserialize)]
pub(crate) struct ProviderCommandDiagnosticsQuery {
    account_id: Option<String>,
    status: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_v1_provider_command_diagnostics(
    State(state): State<AppState>,
    Query(query): Query<ProviderCommandDiagnosticsQuery>,
) -> Result<Json<CommunicationProviderCommandDiagnostics>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let diagnostics = CommunicationProviderCommandStore::new(pool)
        .diagnostics(
            query.account_id.as_deref(),
            query.status.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;
    Ok(Json(diagnostics))
}
