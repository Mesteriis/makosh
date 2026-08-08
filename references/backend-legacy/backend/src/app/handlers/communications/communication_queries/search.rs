use super::super::*;

#[derive(Deserialize)]
pub(crate) struct EmailSearchQuery {
    pub(super) q: String,
    pub(super) limit: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct CommunicationSearchResponse {
    pub(super) results: Vec<SearchResultResponse>,
}

#[derive(Serialize)]
pub(crate) struct SearchResultResponse {
    pub(super) object_id: String,
    pub(super) object_kind: String,
    pub(super) title: String,
}

pub(crate) async fn get_v1_email_search(
    State(state): State<AppState>,
    Query(query): Query<EmailSearchQuery>,
) -> Result<Json<CommunicationSearchResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "search query is required",
        ));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<MessageProjectionStore>(
        pool.clone(),
    );

    let search_path: Option<String> = std::env::var("MAKOSH_SEARCH_INDEX_PATH").ok();
    if let Some(path) = search_path {
        let index = crate::engines::search::engine::SearchIndex::open_or_create(
            std::path::Path::new(&path),
        )?;
        let _ = crate::domains::communications::search::index_messages(&index, &store, 100).await;
        let results = crate::domains::communications::search::search_emails(
            &index,
            &query.q,
            query.limit.unwrap_or(20),
        )?;
        let items: Vec<SearchResultResponse> = results
            .into_iter()
            .map(|result| SearchResultResponse {
                object_id: result.object_id,
                object_kind: result.object_kind,
                title: result.title,
            })
            .collect();
        return Ok(Json(CommunicationSearchResponse { results: items }));
    }

    Ok(Json(CommunicationSearchResponse {
        results: Vec::new(),
    }))
}
