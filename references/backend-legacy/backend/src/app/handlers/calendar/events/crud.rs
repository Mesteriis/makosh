use super::super::*;
use makosh_calendar_api::{CalendarEventListQuery, CalendarEventReadPort};
use makosh_calendar_postgres::CalendarPostgresReadQuery;

#[derive(Serialize)]
pub(crate) struct CalendarEventsResponse {
    items: Vec<makosh_calendar_api::CalendarEventRead>,
}

#[derive(Deserialize)]
pub(crate) struct CalendarEventQuery {
    account_id: Option<String>,
    source_id: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    status: Option<String>,
    event_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_calendar_events(
    State(state): State<AppState>,
    Query(query): Query<CalendarEventQuery>,
) -> Result<Json<CalendarEventsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let list_query = CalendarEventListQuery {
        account_id: query.account_id,
        source_id: query.source_id,
        from: query.from,
        to: query.to,
        status: query.status,
        event_type: query.event_type,
        limit: query.limit,
    };
    let items = CalendarEventReadPort::list(&CalendarPostgresReadQuery::new(pool), list_query)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
    Ok(Json(CalendarEventsResponse { items }))
}

pub(crate) async fn post_calendar_event(
    State(state): State<AppState>,
    Json(req): Json<NewCalendarEvent>,
) -> Result<Json<crate::domains::calendar::events::models::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event =
        crate::app::api_support::stores::domain_stores::app_store::<CalendarEventStore>(pool)
            .create_manual(&req)
            .await?;
    Ok(Json(event))
}

pub(crate) async fn get_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<crate::domains::calendar::events::models::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::stores::domain_stores::app_store::<CalendarEventStore>(pool)
        .get(&event_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(update): Json<CalendarEventUpdate>,
) -> Result<Json<crate::domains::calendar::events::models::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event =
        crate::app::api_support::stores::domain_stores::app_store::<CalendarEventStore>(pool)
            .update_manual(&event_id, &update)
            .await?;
    Ok(Json(event))
}

pub(crate) async fn delete_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::stores::domain_stores::app_store::<CalendarEventStore>(pool)
        .delete_manual(&event_id)
        .await?;
    Ok(Json(json!({"deleted": true})))
}
