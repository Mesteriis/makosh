use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use super::models::{PersonaListResponse, PersonaReadModel, persona_read_model_from_api};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::persona_owner_query::PostgresPersonaOwnerQuery;
use makosh_personas_api::{PersonaReadPort, PersonaUpdateCommand, PersonaWritePort};
use makosh_personas_postgres::PersonaPostgresReadQuery;
#[derive(Deserialize)]
pub(crate) struct PersonaListQuery {
    limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct PersonaUpdateRequest {
    identity: Option<PersonaIdentityUpdateRequest>,
    is_self: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct PersonaIdentityUpdateRequest {
    display_name: Option<String>,
}

pub(crate) async fn get_personas(
    State(state): State<AppState>,
    Query(query): Query<PersonaListQuery>,
) -> Result<Json<PersonaListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonaPostgresReadQuery::new(pool);
    let items = store
        .list(query.limit.unwrap_or(50))
        .await?
        .into_iter()
        .map(persona_read_model_from_api)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(PersonaListResponse { items }))
}

pub(crate) async fn get_persona(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaReadModel>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonaPostgresReadQuery::new(pool);
    match store.get(&persona_id).await? {
        Some(persona) => Ok(Json(persona_read_model_from_api(persona)?)),
        None => Err(ApiError::PersonaIdentityNotFound),
    }
}

pub(crate) async fn put_persona(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<PersonaUpdateRequest>,
) -> Result<Json<PersonaReadModel>, ApiError> {
    if req.is_self == Some(false) {
        return Err(ApiError::InvalidPersonaQuery(
            "is_self=false is not supported; set another Persona as owner instead",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let display_name = req
        .identity
        .as_ref()
        .and_then(|identity| identity.display_name.as_deref());
    let store = PostgresPersonaOwnerQuery::new(pool);
    let persona = store
        .update(PersonaUpdateCommand {
            persona_id,
            display_name: display_name.map(ToOwned::to_owned),
            assign_owner: req.is_self == Some(true),
        })
        .await?;
    Ok(Json(persona_read_model_from_api(persona)?))
}
