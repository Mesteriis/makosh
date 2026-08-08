use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::events::errors::CalendarError;
use makosh_hub_backend::domains::calendar::events::event_store::CalendarEventStore;
use makosh_hub_backend::domains::calendar::events::models::{CalendarEvent, NewCalendarEvent};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct CalendarEventFactory<'a> {
    pool: &'a PgPool,
    title: String,
    start_at: chrono::DateTime<Utc>,
    duration_minutes: i64,
    description: Option<String>,
}

impl<'a> CalendarEventFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        let now = Utc::now();
        Self {
            pool,
            title: format!("test-event-{}", Uuid::new_v4()),
            start_at: now + Duration::hours(1),
            duration_minutes: 60,
            description: Some("Auto-generated test calendar event".into()),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_start(mut self, start: chrono::DateTime<Utc>) -> Self {
        self.start_at = start;
        self
    }

    pub fn with_duration_minutes(mut self, minutes: i64) -> Self {
        self.duration_minutes = minutes;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub async fn create(self) -> Result<CalendarEvent, CalendarError> {
        let store = CalendarEventStore::new(self.pool.clone());
        let end_at = self.start_at + Duration::minutes(self.duration_minutes);
        let new_event = NewCalendarEvent {
            source_event_id: None,
            account_id: None,
            source_id: None,
            title: self.title,
            description: self.description,
            location: None,
            start_at: self.start_at,
            end_at,
            timezone: Some("UTC".into()),
            all_day: Some(false),
            recurrence_rule: None,
            status: Some("confirmed".into()),
            visibility: Some("default".into()),
            event_type: Some("meeting".into()),
            conference_url: None,
            conference_provider: None,
            preparation_reminder_minutes: None,
            travel_buffer_minutes: None,
        };
        store.create(&new_event).await
    }
}
