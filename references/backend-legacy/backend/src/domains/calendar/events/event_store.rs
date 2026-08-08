use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::CalendarError;
use super::models::{CalendarEvent, CalendarEventUpdate, NewCalendarEvent};
use super::queries::CalendarEventListQuery;
use super::rows::row_to_event;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use makosh_observations_postgres::store::ObservationStore;

const CALENDAR_EVENT_COLUMNS: &str = "event_id, observation_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at";

#[derive(Clone)]
pub struct CalendarEventStore {
    pool: PgPool,
}

impl CalendarEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: &NewCalendarEvent) -> Result<CalendarEvent, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let event = Self::create_in_transaction(&mut transaction, req).await?;
        transaction.commit().await?;
        Ok(event)
    }

    pub async fn create_manual(
        &self,
        req: &NewCalendarEvent,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let event =
            Self::create_manual_in_transaction(&mut transaction, req, "calendar_api.create")
                .await?;
        transaction.commit().await?;
        Ok(event)
    }

    pub async fn create_file_import(
        &self,
        req: &NewCalendarEvent,
        source_ref: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let event =
            Self::create_file_import_in_transaction(&mut transaction, req, source_ref).await?;
        transaction.commit().await?;
        Ok(event)
    }

    pub(crate) async fn create_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        req: &NewCalendarEvent,
    ) -> Result<CalendarEvent, CalendarError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let event_id = format!("evt:v1:{ts:x}");
        let observation = NewObservation::new(
            "CALENDAR_EVENT",
            ObservationOriginKind::LocalRuntime,
            req.start_at,
            json!({
                "event_id": event_id,
                "source_event_id": req.source_event_id,
                "account_id": req.account_id,
                "source_id": req.source_id,
                "title": req.title,
                "description": req.description,
                "location": req.location,
                "start_at": req.start_at,
                "end_at": req.end_at,
                "timezone": req.timezone,
                "all_day": req.all_day.unwrap_or(false),
                "recurrence_rule": req.recurrence_rule,
                "status": req.status.clone().unwrap_or_else(|| "scheduled".to_owned()),
                "visibility": req.visibility.clone().unwrap_or_else(|| "private".to_owned()),
                "event_type": req.event_type,
                "conference_url": req.conference_url,
                "conference_provider": req.conference_provider,
                "preparation_reminder_minutes": req.preparation_reminder_minutes,
                "travel_buffer_minutes": req.travel_buffer_minutes,
            }),
            format!("calendar_event://{event_id}"),
        )
        .provenance(json!({
            "ingested_by": "calendar_events_domain",
        }));
        let observation =
            ObservationStore::capture_in_transaction(transaction, &observation).await?;
        let row = sqlx::query(
            &format!(
                "INSERT INTO calendar_events (event_id, observation_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20) RETURNING {CALENDAR_EVENT_COLUMNS}"
            )
        ).bind(&event_id).bind(&observation.observation_id).bind(req.source_event_id.as_deref()).bind(req.account_id.as_deref()).bind(req.source_id.as_deref()).bind(&req.title).bind(req.description.as_deref()).bind(req.location.as_deref()).bind(req.start_at).bind(req.end_at).bind(req.timezone.as_deref()).bind(req.all_day.unwrap_or(false)).bind(req.recurrence_rule.as_deref()).bind(req.status.as_deref().unwrap_or("scheduled")).bind(req.visibility.as_deref().unwrap_or("private")).bind(req.event_type.as_deref()).bind(req.conference_url.as_deref()).bind(req.conference_provider.as_deref()).bind(req.preparation_reminder_minutes).bind(req.travel_buffer_minutes).fetch_one(&mut **transaction).await?;
        link_calendar_event_from_observation_in_transaction(
            transaction,
            &observation.observation_id,
            &event_id,
            None,
            json!({ "action": "create" }),
        )
        .await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub(crate) async fn create_manual_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        req: &NewCalendarEvent,
        actor: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        Self::create_manual_with_observation_in_transaction(transaction, req, actor, None, None)
            .await
    }

    pub(crate) async fn create_manual_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        req: &NewCalendarEvent,
        actor: &str,
        source_observation_id: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<CalendarEvent, CalendarError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let event_id = format!("evt:v1:{ts:x}");
        let status = req.status.clone().unwrap_or_else(|| "scheduled".to_owned());
        let visibility = req
            .visibility
            .clone()
            .unwrap_or_else(|| "private".to_owned());
        let observation = ObservationStore::capture_in_transaction(
            transaction,
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::Manual,
                req.start_at,
                event_payload(
                    &event_id,
                    req.source_event_id.as_deref(),
                    req.account_id.as_deref(),
                    req.source_id.as_deref(),
                    &req.title,
                    req.description.as_deref(),
                    req.location.as_deref(),
                    req.start_at,
                    req.end_at,
                    req.timezone.as_deref(),
                    req.all_day.unwrap_or(false),
                    req.recurrence_rule.as_deref(),
                    &status,
                    &visibility,
                    req.event_type.as_deref(),
                    req.conference_url.as_deref(),
                    req.conference_provider.as_deref(),
                    req.preparation_reminder_minutes,
                    req.travel_buffer_minutes,
                    "create",
                ),
                format!("calendar_event://{event_id}"),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "create",
            })),
        )
        .await?;
        let row = sqlx::query(
            &format!(
                "INSERT INTO calendar_events (event_id, observation_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20) RETURNING {CALENDAR_EVENT_COLUMNS}"
            )
        ).bind(&event_id).bind(&observation.observation_id).bind(req.source_event_id.as_deref()).bind(req.account_id.as_deref()).bind(req.source_id.as_deref()).bind(&req.title).bind(req.description.as_deref()).bind(req.location.as_deref()).bind(req.start_at).bind(req.end_at).bind(req.timezone.as_deref()).bind(req.all_day.unwrap_or(false)).bind(req.recurrence_rule.as_deref()).bind(&status).bind(&visibility).bind(req.event_type.as_deref()).bind(req.conference_url.as_deref()).bind(req.conference_provider.as_deref()).bind(req.preparation_reminder_minutes).bind(req.travel_buffer_minutes).fetch_one(&mut **transaction).await?;
        link_calendar_event_from_observation_in_transaction(
            transaction,
            &observation.observation_id,
            &event_id,
            None,
            json!({ "action": "create" }),
        )
        .await?;
        let event = row_to_event(row).map_err(CalendarError::from)?;
        if let Some(source_observation_id) =
            source_observation_id.filter(|value| !value.trim().is_empty())
        {
            link_calendar_event_from_observation_in_transaction(
                transaction,
                source_observation_id,
                &event.event_id,
                Some("workflow_action_projection"),
                merge_json_objects(
                    json!({
                        "action": "create",
                        "source_event_id": event.source_event_id,
                    }),
                    metadata,
                ),
            )
            .await?;
        }
        Ok(event)
    }

    pub(crate) async fn create_file_import_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        req: &NewCalendarEvent,
        source_ref: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let event_id = format!("evt:v1:{ts:x}");
        let status = req.status.clone().unwrap_or_else(|| "scheduled".to_owned());
        let visibility = req
            .visibility
            .clone()
            .unwrap_or_else(|| "private".to_owned());
        let observation = ObservationStore::capture_in_transaction(
            transaction,
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::FileImport,
                req.start_at,
                event_payload(
                    &event_id,
                    req.source_event_id.as_deref(),
                    req.account_id.as_deref(),
                    req.source_id.as_deref(),
                    &req.title,
                    req.description.as_deref(),
                    req.location.as_deref(),
                    req.start_at,
                    req.end_at,
                    req.timezone.as_deref(),
                    req.all_day.unwrap_or(false),
                    req.recurrence_rule.as_deref(),
                    &status,
                    &visibility,
                    req.event_type.as_deref(),
                    req.conference_url.as_deref(),
                    req.conference_provider.as_deref(),
                    req.preparation_reminder_minutes,
                    req.travel_buffer_minutes,
                    "import",
                ),
                source_ref.to_owned(),
            )
            .provenance(json!({
                "captured_by": "calendar_api.post_calendar_import",
                "action": "import",
            })),
        )
        .await?;
        let row = sqlx::query(
            &format!(
                "INSERT INTO calendar_events (event_id, observation_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20) RETURNING {CALENDAR_EVENT_COLUMNS}"
            )
        ).bind(&event_id).bind(&observation.observation_id).bind(req.source_event_id.as_deref()).bind(req.account_id.as_deref()).bind(req.source_id.as_deref()).bind(&req.title).bind(req.description.as_deref()).bind(req.location.as_deref()).bind(req.start_at).bind(req.end_at).bind(req.timezone.as_deref()).bind(req.all_day.unwrap_or(false)).bind(req.recurrence_rule.as_deref()).bind(&status).bind(&visibility).bind(req.event_type.as_deref()).bind(req.conference_url.as_deref()).bind(req.conference_provider.as_deref()).bind(req.preparation_reminder_minutes).bind(req.travel_buffer_minutes).fetch_one(&mut **transaction).await?;
        link_calendar_event_from_observation_in_transaction(
            transaction,
            &observation.observation_id,
            &event_id,
            None,
            json!({ "action": "import" }),
        )
        .await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<CalendarEvent>, CalendarError> {
        let row = sqlx::query(&format!(
            "SELECT {CALENDAR_EVENT_COLUMNS} FROM calendar_events WHERE event_id=$1"
        ))
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_event)
            .transpose()
            .map_err(CalendarError::from)
    }

    pub async fn list(
        &self,
        query: &CalendarEventListQuery,
    ) -> Result<Vec<CalendarEvent>, CalendarError> {
        let limit = query.limit.unwrap_or(100).clamp(1, 500);
        let rows = sqlx::query(
            &format!("SELECT {CALENDAR_EVENT_COLUMNS} FROM calendar_events WHERE ($1::text IS NULL OR account_id=$1) AND ($2::text IS NULL OR source_id=$2) AND ($3::timestamptz IS NULL OR end_at>=$3) AND ($4::timestamptz IS NULL OR start_at<=$4) AND ($5::text IS NULL OR status=$5) AND ($6::text IS NULL OR event_type=$6) ORDER BY start_at ASC LIMIT $7")
        ).bind(query.account_id.as_deref()).bind(query.source_id.as_deref()).bind(query.from).bind(query.to).bind(query.status.as_deref()).bind(query.event_type.as_deref()).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_event)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn find_zoom_conference_match(
        &self,
        join_url: Option<&str>,
        meeting_id: &str,
        started_at: Option<DateTime<Utc>>,
        ended_at: Option<DateTime<Utc>>,
    ) -> Result<Option<CalendarEvent>, CalendarError> {
        self.find_provider_conference_match("zoom", join_url, meeting_id, started_at, ended_at)
            .await
    }

    pub async fn find_yandex_telemost_conference_match(
        &self,
        join_url: Option<&str>,
        conference_id: &str,
    ) -> Result<Option<CalendarEvent>, CalendarError> {
        self.find_provider_conference_match("yandex_telemost", join_url, conference_id, None, None)
            .await
    }

    async fn find_provider_conference_match(
        &self,
        conference_provider: &str,
        join_url: Option<&str>,
        conference_id: &str,
        started_at: Option<DateTime<Utc>>,
        ended_at: Option<DateTime<Utc>>,
    ) -> Result<Option<CalendarEvent>, CalendarError> {
        let conference_id = conference_id.trim();
        if conference_id.is_empty() {
            return Ok(None);
        }
        let join_url = join_url
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        let like_pattern = format!("%{conference_id}%");
        let window_start = started_at.or(ended_at);
        let window_end = ended_at.or(started_at);
        let rows = sqlx::query(&format!(
            "SELECT {CALENDAR_EVENT_COLUMNS}
                 FROM calendar_events
                 WHERE conference_provider = $1
                   AND (
                        ($2::text IS NOT NULL AND conference_url = $2)
                        OR conference_url ILIKE $3
                   )
                   AND ($4::timestamptz IS NULL OR end_at >= $4)
                   AND ($5::timestamptz IS NULL OR start_at <= $5)
                 ORDER BY
                   CASE WHEN $2::text IS NOT NULL AND conference_url = $2 THEN 0 ELSE 1 END,
                   start_at ASC
                 LIMIT 1"
        ))
        .bind(conference_provider)
        .bind(join_url.as_deref())
        .bind(&like_pattern)
        .bind(window_start)
        .bind(window_end)
        .fetch_optional(&self.pool)
        .await?;

        rows.map(row_to_event)
            .transpose()
            .map_err(CalendarError::from)
    }

    pub async fn update(
        &self,
        event_id: &str,
        update: &CalendarEventUpdate,
    ) -> Result<CalendarEvent, CalendarError> {
        let row = sqlx::query(
            &format!("UPDATE calendar_events SET title=COALESCE($2,title), description=COALESCE($3,description), location=COALESCE($4,location), start_at=COALESCE($5,start_at), end_at=COALESCE($6,end_at), timezone=COALESCE($7,timezone), all_day=COALESCE($8,all_day), recurrence_rule=COALESCE($9,recurrence_rule), status=COALESCE($10,status), visibility=COALESCE($11,visibility), event_type=COALESCE($12,event_type), importance_score=COALESCE($13,importance_score), readiness_score=COALESCE($14,readiness_score), conference_url=COALESCE($15,conference_url), conference_provider=COALESCE($16,conference_provider), preparation_reminder_minutes=COALESCE($17,preparation_reminder_minutes), travel_buffer_minutes=COALESCE($18,travel_buffer_minutes), updated_at=now() WHERE event_id=$1 RETURNING {CALENDAR_EVENT_COLUMNS}")
        ).bind(event_id).bind(update.title.as_deref()).bind(update.description.as_deref()).bind(update.location.as_deref()).bind(update.start_at).bind(update.end_at).bind(update.timezone.as_deref()).bind(update.all_day).bind(update.recurrence_rule.as_deref()).bind(update.status.as_deref()).bind(update.visibility.as_deref()).bind(update.event_type.as_deref()).bind(update.importance_score).bind(update.readiness_score).bind(update.conference_url.as_deref()).bind(update.conference_provider.as_deref()).bind(update.preparation_reminder_minutes).bind(update.travel_buffer_minutes).fetch_one(&self.pool).await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub async fn update_manual(
        &self,
        event_id: &str,
        update: &CalendarEventUpdate,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let current = Self::get_in_transaction(&mut transaction, event_id)
            .await?
            .ok_or(CalendarError::NotFound)?;
        let merged = merge_event_update(&current, update);
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::Manual,
                merged.start_at,
                event_payload(
                    &current.event_id,
                    current.source_event_id.as_deref(),
                    current.account_id.as_deref(),
                    current.source_id.as_deref(),
                    &merged.title,
                    merged.description.as_deref(),
                    merged.location.as_deref(),
                    merged.start_at,
                    merged.end_at,
                    merged.timezone.as_deref(),
                    merged.all_day,
                    merged.recurrence_rule.as_deref(),
                    &merged.status,
                    &merged.visibility,
                    merged.event_type.as_deref(),
                    merged.conference_url.as_deref(),
                    merged.conference_provider.as_deref(),
                    merged.preparation_reminder_minutes,
                    merged.travel_buffer_minutes,
                    "update",
                ),
                format!("calendar_event://{}", current.event_id),
            )
            .provenance(json!({
                "captured_by": "calendar_api.put_calendar_event",
                "action": "update",
                "update": update,
            })),
        )
        .await?;
        let row = sqlx::query(
            &format!("UPDATE calendar_events SET observation_id=$2, title=COALESCE($3,title), description=COALESCE($4,description), location=COALESCE($5,location), start_at=COALESCE($6,start_at), end_at=COALESCE($7,end_at), timezone=COALESCE($8,timezone), all_day=COALESCE($9,all_day), recurrence_rule=COALESCE($10,recurrence_rule), status=COALESCE($11,status), visibility=COALESCE($12,visibility), event_type=COALESCE($13,event_type), importance_score=COALESCE($14,importance_score), readiness_score=COALESCE($15,readiness_score), conference_url=COALESCE($16,conference_url), conference_provider=COALESCE($17,conference_provider), preparation_reminder_minutes=COALESCE($18,preparation_reminder_minutes), travel_buffer_minutes=COALESCE($19,travel_buffer_minutes), updated_at=now() WHERE event_id=$1 RETURNING {CALENDAR_EVENT_COLUMNS}")
        ).bind(event_id).bind(&observation.observation_id).bind(update.title.as_deref()).bind(update.description.as_deref()).bind(update.location.as_deref()).bind(update.start_at).bind(update.end_at).bind(update.timezone.as_deref()).bind(update.all_day).bind(update.recurrence_rule.as_deref()).bind(update.status.as_deref()).bind(update.visibility.as_deref()).bind(update.event_type.as_deref()).bind(update.importance_score).bind(update.readiness_score).bind(update.conference_url.as_deref()).bind(update.conference_provider.as_deref()).bind(update.preparation_reminder_minutes).bind(update.travel_buffer_minutes).fetch_one(&mut *transaction).await?;
        link_calendar_event_from_observation_in_transaction(
            &mut transaction,
            &observation.observation_id,
            event_id,
            None,
            json!({ "action": "update" }),
        )
        .await?;
        transaction.commit().await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub async fn update_runtime(
        &self,
        event_id: &str,
        update: &CalendarEventUpdate,
        actor: &str,
        action: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let current = Self::get_in_transaction(&mut transaction, event_id)
            .await?
            .ok_or(CalendarError::NotFound)?;
        let merged = merge_event_update(&current, update);
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::LocalRuntime,
                merged.start_at,
                event_payload(
                    &current.event_id,
                    current.source_event_id.as_deref(),
                    current.account_id.as_deref(),
                    current.source_id.as_deref(),
                    &merged.title,
                    merged.description.as_deref(),
                    merged.location.as_deref(),
                    merged.start_at,
                    merged.end_at,
                    merged.timezone.as_deref(),
                    merged.all_day,
                    merged.recurrence_rule.as_deref(),
                    &merged.status,
                    &merged.visibility,
                    merged.event_type.as_deref(),
                    merged.conference_url.as_deref(),
                    merged.conference_provider.as_deref(),
                    merged.preparation_reminder_minutes,
                    merged.travel_buffer_minutes,
                    action,
                ),
                format!("calendar_event://{event_id}/{action}"),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": action,
                "update": update,
            })),
        )
        .await?;
        let row = sqlx::query(
            &format!("UPDATE calendar_events SET observation_id=$2, title=COALESCE($3,title), description=COALESCE($4,description), location=COALESCE($5,location), start_at=COALESCE($6,start_at), end_at=COALESCE($7,end_at), timezone=COALESCE($8,timezone), all_day=COALESCE($9,all_day), recurrence_rule=COALESCE($10,recurrence_rule), status=COALESCE($11,status), visibility=COALESCE($12,visibility), event_type=COALESCE($13,event_type), importance_score=COALESCE($14,importance_score), readiness_score=COALESCE($15,readiness_score), conference_url=COALESCE($16,conference_url), conference_provider=COALESCE($17,conference_provider), preparation_reminder_minutes=COALESCE($18,preparation_reminder_minutes), travel_buffer_minutes=COALESCE($19,travel_buffer_minutes), updated_at=now() WHERE event_id=$1 RETURNING {CALENDAR_EVENT_COLUMNS}")
        ).bind(event_id).bind(&observation.observation_id).bind(update.title.as_deref()).bind(update.description.as_deref()).bind(update.location.as_deref()).bind(update.start_at).bind(update.end_at).bind(update.timezone.as_deref()).bind(update.all_day).bind(update.recurrence_rule.as_deref()).bind(update.status.as_deref()).bind(update.visibility.as_deref()).bind(update.event_type.as_deref()).bind(update.importance_score).bind(update.readiness_score).bind(update.conference_url.as_deref()).bind(update.conference_provider.as_deref()).bind(update.preparation_reminder_minutes).bind(update.travel_buffer_minutes).fetch_one(&mut *transaction).await?;
        link_calendar_event_from_observation_in_transaction(
            &mut transaction,
            &observation.observation_id,
            event_id,
            Some("runtime_update"),
            json!({ "action": action }),
        )
        .await?;
        transaction.commit().await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub async fn delete(&self, event_id: &str) -> Result<(), CalendarError> {
        sqlx::query("DELETE FROM calendar_events WHERE event_id=$1")
            .bind(event_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_manual(&self, event_id: &str) -> Result<(), CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let current = Self::get_in_transaction(&mut transaction, event_id)
            .await?
            .ok_or(CalendarError::NotFound)?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "CALENDAR_EVENT_DELETED",
                ObservationOriginKind::Manual,
                Utc::now(),
                event_payload(
                    &current.event_id,
                    current.source_event_id.as_deref(),
                    current.account_id.as_deref(),
                    current.source_id.as_deref(),
                    &current.title,
                    current.description.as_deref(),
                    current.location.as_deref(),
                    current.start_at,
                    current.end_at,
                    current.timezone.as_deref(),
                    current.all_day,
                    current.recurrence_rule.as_deref(),
                    &current.status,
                    &current.visibility,
                    current.event_type.as_deref(),
                    current.conference_url.as_deref(),
                    current.conference_provider.as_deref(),
                    current.preparation_reminder_minutes,
                    current.travel_buffer_minutes,
                    "delete",
                ),
                format!("calendar_event://{event_id}/deleted"),
            )
            .provenance(json!({
                "captured_by": "calendar_api.delete_calendar_event",
                "action": "delete",
            })),
        )
        .await?;
        link_calendar_event_from_observation_in_transaction(
            &mut transaction,
            &observation.observation_id,
            event_id,
            None,
            json!({ "action": "delete" }),
        )
        .await?;
        sqlx::query("DELETE FROM calendar_events WHERE event_id=$1")
            .bind(event_id)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn set_status(&self, event_id: &str, status: &str) -> Result<(), CalendarError> {
        sqlx::query(
            "UPDATE calendar_events SET status = $2, updated_at = now() WHERE event_id = $1",
        )
        .bind(event_id)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_status_manual(
        &self,
        event_id: &str,
        status: &str,
        actor: &str,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let current = Self::get_in_transaction(&mut transaction, event_id)
            .await?
            .ok_or(CalendarError::NotFound)?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::Manual,
                current.start_at,
                event_payload(
                    &current.event_id,
                    current.source_event_id.as_deref(),
                    current.account_id.as_deref(),
                    current.source_id.as_deref(),
                    &current.title,
                    current.description.as_deref(),
                    current.location.as_deref(),
                    current.start_at,
                    current.end_at,
                    current.timezone.as_deref(),
                    current.all_day,
                    current.recurrence_rule.as_deref(),
                    status,
                    &current.visibility,
                    current.event_type.as_deref(),
                    current.conference_url.as_deref(),
                    current.conference_provider.as_deref(),
                    current.preparation_reminder_minutes,
                    current.travel_buffer_minutes,
                    "status_change",
                ),
                format!("calendar_event://{event_id}/status/{status}"),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "status_change",
                "status": status,
            })),
        )
        .await?;
        let row = sqlx::query(
            &format!("UPDATE calendar_events SET observation_id = $2, status = $3, updated_at = now() WHERE event_id = $1 RETURNING {CALENDAR_EVENT_COLUMNS}"),
        )
        .bind(event_id)
        .bind(&observation.observation_id)
        .bind(status)
        .fetch_one(&mut *transaction)
        .await?;
        link_calendar_event_from_observation_in_transaction(
            &mut transaction,
            &observation.observation_id,
            event_id,
            None,
            json!({
                "action": "status_change",
                "status": status,
            }),
        )
        .await?;
        transaction.commit().await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub async fn reschedule(
        &self,
        event_id: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
    ) -> Result<CalendarEvent, CalendarError> {
        let row = sqlx::query(&format!("UPDATE calendar_events SET start_at=$2, end_at=$3, status='rescheduled', updated_at=now() WHERE event_id=$1 RETURNING {CALENDAR_EVENT_COLUMNS}"))
            .bind(event_id).bind(start_at).bind(end_at).fetch_one(&self.pool).await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub async fn reschedule_manual(
        &self,
        event_id: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
    ) -> Result<CalendarEvent, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let current = Self::get_in_transaction(&mut transaction, event_id)
            .await?
            .ok_or(CalendarError::NotFound)?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::Manual,
                start_at,
                event_payload(
                    &current.event_id,
                    current.source_event_id.as_deref(),
                    current.account_id.as_deref(),
                    current.source_id.as_deref(),
                    &current.title,
                    current.description.as_deref(),
                    current.location.as_deref(),
                    start_at,
                    end_at,
                    current.timezone.as_deref(),
                    current.all_day,
                    current.recurrence_rule.as_deref(),
                    "rescheduled",
                    &current.visibility,
                    current.event_type.as_deref(),
                    current.conference_url.as_deref(),
                    current.conference_provider.as_deref(),
                    current.preparation_reminder_minutes,
                    current.travel_buffer_minutes,
                    "reschedule",
                ),
                format!("calendar_event://{event_id}/reschedule"),
            )
            .provenance(json!({
                "captured_by": "calendar_api.post_calendar_event_reschedule",
                "action": "reschedule",
            })),
        )
        .await?;
        let row = sqlx::query(&format!("UPDATE calendar_events SET observation_id=$2, start_at=$3, end_at=$4, status='rescheduled', updated_at=now() WHERE event_id=$1 RETURNING {CALENDAR_EVENT_COLUMNS}"))
            .bind(event_id).bind(&observation.observation_id).bind(start_at).bind(end_at).fetch_one(&mut *transaction).await?;
        link_calendar_event_from_observation_in_transaction(
            &mut transaction,
            &observation.observation_id,
            event_id,
            None,
            json!({ "action": "reschedule" }),
        )
        .await?;
        transaction.commit().await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    async fn get_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
    ) -> Result<Option<CalendarEvent>, CalendarError> {
        let row = sqlx::query(&format!(
            "SELECT {CALENDAR_EVENT_COLUMNS} FROM calendar_events WHERE event_id=$1"
        ))
        .bind(event_id)
        .fetch_optional(&mut **transaction)
        .await?;
        row.map(row_to_event)
            .transpose()
            .map_err(CalendarError::from)
    }
}

#[allow(clippy::too_many_arguments)]
fn event_payload(
    event_id: &str,
    source_event_id: Option<&str>,
    account_id: Option<&str>,
    source_id: Option<&str>,
    title: &str,
    description: Option<&str>,
    location: Option<&str>,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    timezone: Option<&str>,
    all_day: bool,
    recurrence_rule: Option<&str>,
    status: &str,
    visibility: &str,
    event_type: Option<&str>,
    conference_url: Option<&str>,
    conference_provider: Option<&str>,
    preparation_reminder_minutes: Option<i32>,
    travel_buffer_minutes: Option<i32>,
    action: &str,
) -> serde_json::Value {
    json!({
        "event_id": event_id,
        "source_event_id": source_event_id,
        "account_id": account_id,
        "source_id": source_id,
        "title": title,
        "description": description,
        "location": location,
        "start_at": start_at,
        "end_at": end_at,
        "timezone": timezone,
        "all_day": all_day,
        "recurrence_rule": recurrence_rule,
        "status": status,
        "visibility": visibility,
        "event_type": event_type,
        "conference_url": conference_url,
        "conference_provider": conference_provider,
        "preparation_reminder_minutes": preparation_reminder_minutes,
        "travel_buffer_minutes": travel_buffer_minutes,
        "action": action,
    })
}

fn merge_event_update(current: &CalendarEvent, update: &CalendarEventUpdate) -> CalendarEvent {
    CalendarEvent {
        event_id: current.event_id.clone(),
        observation_id: current.observation_id.clone(),
        source_event_id: current.source_event_id.clone(),
        account_id: current.account_id.clone(),
        source_id: current.source_id.clone(),
        title: update
            .title
            .clone()
            .unwrap_or_else(|| current.title.clone()),
        description: update
            .description
            .clone()
            .or_else(|| current.description.clone()),
        location: update.location.clone().or_else(|| current.location.clone()),
        start_at: update.start_at.unwrap_or(current.start_at),
        end_at: update.end_at.unwrap_or(current.end_at),
        timezone: update.timezone.clone().or_else(|| current.timezone.clone()),
        all_day: update.all_day.unwrap_or(current.all_day),
        recurrence_rule: update
            .recurrence_rule
            .clone()
            .or_else(|| current.recurrence_rule.clone()),
        status: update
            .status
            .clone()
            .unwrap_or_else(|| current.status.clone()),
        visibility: update
            .visibility
            .clone()
            .unwrap_or_else(|| current.visibility.clone()),
        event_type: update
            .event_type
            .clone()
            .or_else(|| current.event_type.clone()),
        importance_score: update.importance_score.or(current.importance_score),
        readiness_score: update.readiness_score.or(current.readiness_score),
        sync_status: current.sync_status.clone(),
        conference_url: update
            .conference_url
            .clone()
            .or_else(|| current.conference_url.clone()),
        conference_provider: update
            .conference_provider
            .clone()
            .or_else(|| current.conference_provider.clone()),
        preparation_reminder_minutes: update
            .preparation_reminder_minutes
            .or(current.preparation_reminder_minutes),
        travel_buffer_minutes: update
            .travel_buffer_minutes
            .or(current.travel_buffer_minutes),
        created_at: current.created_at,
        updated_at: current.updated_at,
    }
}

async fn link_calendar_event_from_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    event_id: &str,
    relationship_kind: Option<&str>,
    metadata: serde_json::Value,
) -> Result<(), makosh_observations_postgres::errors::ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "calendar",
        "event",
        event_id.to_owned(),
        relationship_kind,
        None,
        Some(metadata),
    )
    .await
}

fn merge_json_objects(
    base: serde_json::Value,
    extra: Option<serde_json::Value>,
) -> serde_json::Value {
    match extra {
        Some(extra) if base.is_object() && extra.is_object() => {
            let mut merged = base;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => base,
    }
}
