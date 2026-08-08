use axum::Json;
use axum::extract::{RawQuery, State};

use crate::app::api_support::query_parsing::graph::*;
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use makosh_graph_api::{
    GraphNeighborhoodQueryPort, GraphNodeReadPort, GraphNodeSearchPort, GraphSummaryQueryPort,
};
use makosh_graph_postgres::GraphPostgresSummaryQuery;

pub(crate) async fn get_graph_summary(
    State(state): State<AppState>,
) -> Result<Json<makosh_graph_api::GraphSummary>, ApiError> {
    let query = GraphPostgresSummaryQuery::new(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    );
    Ok(Json(query.summary().await.map_err(|error| {
        ApiError::FailedPrecondition(error.to_string())
    })?))
}

pub(crate) async fn get_graph_nodes(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<makosh_graph_api::GraphNodeRead>>, ApiError> {
    let query = parse_graph_nodes_query(raw_query.as_deref())?;
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    let query = GraphPostgresSummaryQuery::new(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    );
    Ok(Json(query.list_nodes(limit).await.map_err(|error| {
        ApiError::FailedPrecondition(error.to_string())
    })?))
}

pub(crate) async fn get_graph_neighborhood(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<makosh_graph_api::GraphNeighborhoodRead>, ApiError> {
    let query = parse_graph_neighborhood_query(raw_query.as_deref())?;
    if query.depth.unwrap_or(1) != 1 {
        return Err(ApiError::InvalidGraphQuery("depth supports only 1"));
    }
    let Some(node_id) = query
        .node_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return Err(ApiError::GraphNotFound);
    };
    let query = GraphPostgresSummaryQuery::new(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    );
    let Some(neighborhood) = query
        .neighborhood(node_id)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?
    else {
        return Err(ApiError::GraphNotFound);
    };
    Ok(Json(neighborhood))
}

pub(crate) async fn get_graph_search(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<makosh_graph_api::GraphNodeRead>>, ApiError> {
    let query = parse_graph_search_query(raw_query.as_deref())?;
    let search = query.q.as_deref().unwrap_or_default().trim();
    if search.is_empty() {
        return Err(ApiError::InvalidGraphQuery("q must not be empty"));
    }
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    let query = GraphPostgresSummaryQuery::new(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    );
    Ok(Json(query.search_nodes(search, limit).await.map_err(
        |error| ApiError::FailedPrecondition(error.to_string()),
    )?))
}
