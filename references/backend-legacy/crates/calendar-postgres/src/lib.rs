use makosh_calendar_api::{
    CalendarEventListFuture, CalendarEventListQuery, CalendarEventRead, CalendarEventReadPort,
    CalendarQueryError,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct CalendarPostgresReadQuery {
    pool: PgPool,
}
impl CalendarPostgresReadQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl CalendarEventReadPort for CalendarPostgresReadQuery {
    fn list<'a>(&'a self, query: CalendarEventListQuery) -> CalendarEventListFuture<'a> {
        Box::pin(async move {
            let limit = query.limit.unwrap_or(100).clamp(1, 500);
            let rows = sqlx::query(CALENDAR_SQL)
                .bind(query.account_id.as_deref())
                .bind(query.source_id.as_deref())
                .bind(query.from)
                .bind(query.to)
                .bind(query.status.as_deref())
                .bind(query.event_type.as_deref())
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(error)?;
            rows.into_iter().map(map_row).collect()
        })
    }
}

const CALENDAR_SQL: &str = "SELECT event_id, observation_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at FROM calendar_events WHERE ($1::text IS NULL OR account_id=$1) AND ($2::text IS NULL OR source_id=$2) AND ($3::timestamptz IS NULL OR end_at>=$3) AND ($4::timestamptz IS NULL OR start_at<=$4) AND ($5::text IS NULL OR status=$5) AND ($6::text IS NULL OR event_type=$6) ORDER BY start_at ASC LIMIT $7";
fn error(e: sqlx::Error) -> CalendarQueryError {
    CalendarQueryError(e.to_string())
}
fn map_row(row: sqlx::postgres::PgRow) -> Result<CalendarEventRead, CalendarQueryError> {
    macro_rules! get {
        ($name:literal) => {
            row.try_get($name).map_err(error)?
        };
    }
    Ok(CalendarEventRead {
        event_id: get!("event_id"),
        observation_id: get!("observation_id"),
        source_event_id: get!("source_event_id"),
        account_id: get!("account_id"),
        source_id: get!("source_id"),
        title: get!("title"),
        description: get!("description"),
        location: get!("location"),
        start_at: get!("start_at"),
        end_at: get!("end_at"),
        timezone: get!("timezone"),
        all_day: get!("all_day"),
        recurrence_rule: get!("recurrence_rule"),
        status: get!("status"),
        visibility: get!("visibility"),
        event_type: get!("event_type"),
        importance_score: get!("importance_score"),
        readiness_score: get!("readiness_score"),
        sync_status: get!("sync_status"),
        conference_url: get!("conference_url"),
        conference_provider: get!("conference_provider"),
        preparation_reminder_minutes: get!("preparation_reminder_minutes"),
        travel_buffer_minutes: get!("travel_buffer_minutes"),
        created_at: get!("created_at"),
        updated_at: get!("updated_at"),
    })
}
