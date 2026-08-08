use axum::http::HeaderMap;

use crate::ai::control_center::store::AiControlCenterStore;
use crate::app::error::types::ApiError;
use crate::app::state::AppState;

pub(super) fn ai_control_center_store(state: &AppState) -> Result<AiControlCenterStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(AiControlCenterStore::new(pool.clone()))
}

pub(super) fn request_actor_id(headers: &HeaderMap) -> String {
    headers
        .get("x-makosh-actor-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("makosh-frontend")
        .to_owned()
}
