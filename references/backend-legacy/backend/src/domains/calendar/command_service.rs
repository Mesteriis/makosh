use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::events::account_store::CalendarAccountStore;
use crate::domains::calendar::events::source_store::CalendarSourceStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::core::agendas::{EventAgenda, EventAgendaStore};
use super::core::checklists::{EventChecklist, EventChecklistStore};
use super::core::errors::CalendarCoreError;
use super::core::participants::{EventParticipant, EventParticipantStore};
use super::core::relations::{EventRelation, EventRelationStore};
use super::events::errors::CalendarError;
use super::events::models::{CalendarAccount, CalendarAccountUpdate, CalendarSource};
use super::meetings::errors::MeetingsError;
use super::meetings::models::{EventRecording, MeetingNote, MeetingOutcome};
use super::meetings::notes::MeetingNoteStore;
use super::meetings::outcomes::MeetingOutcomeStore;
use super::meetings::recordings::EventRecordingStore;
use super::reminders::{CalendarReminder, CalendarReminderStore, ReminderError};
use super::rules::{CalendarRule, CalendarRuleError, CalendarRuleStore, RuleUpdate};
use super::scheduling::{
    DeadlineEvent, DeadlineStore, FocusBlock, FocusBlockStore, SchedulingError,
};

#[derive(Clone)]
pub struct CalendarCommandService {
    pool: PgPool,
}

impl CalendarCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_calendar_account_manual(
        &self,
        provider: &str,
        account_name: &str,
        email: Option<&str>,
    ) -> Result<CalendarAccount, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_ACCOUNT_MUTATION",
                json!({
                    "provider": provider,
                    "account_name": account_name,
                    "email": email,
                    "action": "create_calendar_account",
                }),
                "calendar-account://create".to_owned(),
                json!({
                    "captured_by": "calendar_service.create_calendar_account_manual",
                    "operation": "create_calendar_account_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(CalendarAccountStore::new(self.pool.clone())
            .create_with_observation(
                provider,
                account_name,
                email,
                Some(&observation.observation_id),
                "create",
                None,
            )
            .await?)
    }

    pub async fn update_calendar_account_manual(
        &self,
        account_id: &str,
        update: &CalendarAccountUpdate,
    ) -> Result<CalendarAccount, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_ACCOUNT_MUTATION",
                json!({
                    "account_id": account_id,
                    "update": serde_json::to_value(update).unwrap_or(Value::Null),
                    "action": "update_calendar_account",
                }),
                format!("calendar-account://{account_id}/update"),
                json!({
                    "captured_by": "calendar_service.update_calendar_account_manual",
                    "operation": "update_calendar_account_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(CalendarAccountStore::new(self.pool.clone())
            .update_with_observation(
                account_id,
                update,
                Some(&observation.observation_id),
                "update",
                None,
            )
            .await?)
    }

    pub async fn delete_calendar_account_manual(
        &self,
        account_id: &str,
    ) -> Result<(), CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_ACCOUNT_MUTATION",
                json!({
                    "account_id": account_id,
                    "action": "delete_calendar_account",
                }),
                format!("calendar-account://{account_id}/delete"),
                json!({
                    "captured_by": "calendar_service.delete_calendar_account_manual",
                    "operation": "delete_calendar_account_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        CalendarAccountStore::new(self.pool.clone())
            .delete_with_observation(
                account_id,
                Some(&observation.observation_id),
                "delete",
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn create_calendar_source_manual(
        &self,
        account_id: &str,
        name: &str,
        provider_calendar_id: Option<&str>,
        color: Option<&str>,
        timezone: Option<&str>,
    ) -> Result<CalendarSource, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_EVENT",
                json!({
                    "account_id": account_id,
                    "name": name,
                    "provider_calendar_id": provider_calendar_id,
                    "color": color,
                    "timezone": timezone,
                    "action": "create_calendar_source",
                }),
                format!("calendar-source://{account_id}/create"),
                json!({
                    "captured_by": "calendar_service.create_calendar_source_manual",
                    "operation": "create_calendar_source_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(CalendarSourceStore::new(self.pool.clone())
            .create_with_observation(
                account_id,
                name,
                provider_calendar_id,
                color,
                timezone,
                Some(&observation.observation_id),
                "create",
                None,
            )
            .await?)
    }

    pub async fn set_event_agenda_manual(
        &self,
        event_id: &str,
        items: Value,
        requested_source: &str,
    ) -> Result<EventAgenda, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "EVENT_AGENDA",
                json!({
                    "event_id": event_id,
                    "items": items.clone(),
                    "source": requested_source,
                }),
                format!("calendar-event://{event_id}/agenda"),
                json!({
                    "captured_by": "calendar_service.set_event_agenda_manual",
                    "operation": "set_event_agenda_manual",
                    "requested_source": requested_source,
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(EventAgendaStore::new(self.pool.clone())
            .set_with_observation(
                event_id,
                items,
                &format!("observation:{}", observation.observation_id),
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn set_event_checklist_manual(
        &self,
        event_id: &str,
        items: Value,
        requested_source: &str,
    ) -> Result<EventChecklist, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "EVENT_CHECKLIST",
                json!({
                    "event_id": event_id,
                    "items": items.clone(),
                    "source": requested_source,
                }),
                format!("calendar-event://{event_id}/checklist"),
                json!({
                    "captured_by": "calendar_service.set_event_checklist_manual",
                    "operation": "set_event_checklist_manual",
                    "requested_source": requested_source,
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(EventChecklistStore::new(self.pool.clone())
            .set_with_observation(
                event_id,
                items,
                &format!("observation:{}", observation.observation_id),
                Some(&observation.observation_id),
            )
            .await?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_event_participant_manual(
        &self,
        event_id: &str,
        email: &str,
        display_name: Option<&str>,
        role: Option<&str>,
        persona_id: Option<&str>,
        organization_id: Option<&str>,
    ) -> Result<EventParticipant, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_EVENT",
                json!({
                    "event_id": event_id,
                    "email": email,
                    "display_name": display_name,
                    "role": role,
                    "persona_id": persona_id,
                    "organization_id": organization_id,
                    "action": "add_participant",
                }),
                format!("calendar-event://{event_id}/participants"),
                json!({
                    "captured_by": "calendar_service.add_event_participant_manual",
                    "operation": "add_event_participant_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(EventParticipantStore::new(self.pool.clone())
            .add_with_observation(
                event_id,
                email,
                display_name,
                role,
                persona_id,
                organization_id,
                &format!("observation:{}", observation.observation_id),
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn link_event_relation_manual(
        &self,
        event_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
    ) -> Result<EventRelation, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_EVENT",
                json!({
                    "event_id": event_id,
                    "entity_type": entity_type,
                    "entity_id": entity_id,
                    "relation_type": relation_type,
                    "action": "link_relation",
                }),
                format!("calendar-event://{event_id}/relations"),
                json!({
                    "captured_by": "calendar_service.link_event_relation_manual",
                    "operation": "link_event_relation_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(EventRelationStore::new(self.pool.clone())
            .link_with_observation(
                event_id,
                entity_type,
                entity_id,
                relation_type,
                &format!("observation:{}", observation.observation_id),
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn create_meeting_note_manual(
        &self,
        event_id: &str,
        content: &str,
        format: Option<&str>,
        requested_source: &str,
    ) -> Result<MeetingNote, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "MEETING_NOTE",
                json!({
                    "event_id": event_id,
                    "content": content,
                    "format": format,
                    "source": requested_source,
                }),
                format!("calendar-event://{event_id}/meeting-note"),
                json!({
                    "captured_by": "calendar_service.create_meeting_note_manual",
                    "operation": "create_meeting_note_manual",
                    "requested_source": requested_source,
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(MeetingNoteStore::new(self.pool.clone())
            .create_with_observation(
                event_id,
                content,
                format,
                Some(&format!("observation:{}", observation.observation_id)),
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn add_meeting_outcome_manual(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_person_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<MeetingOutcome, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "MEETING",
                json!({
                    "event_id": event_id,
                    "outcome_type": outcome_type,
                    "title": title,
                    "description": description,
                    "owner_person_id": owner_person_id,
                    "due_date": due_date,
                }),
                format!("calendar-event://{event_id}/meeting-outcome"),
                json!({
                    "captured_by": "calendar_service.add_meeting_outcome_manual",
                    "operation": "add_meeting_outcome_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(MeetingOutcomeStore::new(self.pool.clone())
            .add_with_observation(
                event_id,
                outcome_type,
                title,
                description,
                owner_person_id,
                due_date,
                Some(&format!("observation:{}", observation.observation_id)),
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn add_event_recording_manual(
        &self,
        event_id: &str,
        file_path: Option<&str>,
        requested_source: &str,
        duration_seconds: Option<i32>,
    ) -> Result<EventRecording, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "MEETING_RECORDING",
                json!({
                    "event_id": event_id,
                    "file_path": file_path,
                    "duration_seconds": duration_seconds,
                    "source": requested_source,
                }),
                format!("calendar-event://{event_id}/recording"),
                json!({
                    "captured_by": "calendar_service.add_event_recording_manual",
                    "operation": "add_event_recording_manual",
                    "requested_source": requested_source,
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(EventRecordingStore::new(self.pool.clone())
            .add_with_observation(
                event_id,
                file_path,
                Some(&format!("observation:{}", observation.observation_id)),
                duration_seconds,
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn create_event_reminder_manual(
        &self,
        event_id: &str,
        reminder_type: &str,
        minutes_before: Option<i32>,
        message: Option<&str>,
    ) -> Result<CalendarReminder, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_EVENT",
                json!({
                    "event_id": event_id,
                    "reminder_type": reminder_type,
                    "minutes_before": minutes_before,
                    "message": message,
                    "action": "create_reminder",
                }),
                format!("calendar-event://{event_id}/reminders"),
                json!({
                    "captured_by": "calendar_service.create_event_reminder_manual",
                    "operation": "create_event_reminder_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(CalendarReminderStore::new(self.pool.clone())
            .create_with_observation(
                event_id,
                reminder_type,
                minutes_before,
                message,
                &format!("observation:{}", observation.observation_id),
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn toggle_event_reminder_manual(
        &self,
        event_id: &str,
        reminder_id: &str,
        active: bool,
    ) -> Result<(), CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_EVENT",
                json!({
                    "event_id": event_id,
                    "reminder_id": reminder_id,
                    "active": active,
                    "action": "toggle_reminder",
                }),
                format!("calendar-event://{event_id}/reminders/{reminder_id}/toggle"),
                json!({
                    "captured_by": "calendar_service.toggle_event_reminder_manual",
                    "operation": "toggle_event_reminder_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        CalendarReminderStore::new(self.pool.clone())
            .set_active_with_observation(
                reminder_id,
                active,
                &format!("observation:{}", observation.observation_id),
                Some(&observation.observation_id),
                Some(json!({
                    "event_id": event_id,
                    "active": active,
                    "action": "toggle",
                })),
            )
            .await?;
        Ok(())
    }

    pub async fn create_calendar_rule_manual(
        &self,
        name: &str,
        description: Option<&str>,
        dsl: Value,
        approval_mode: Option<&str>,
    ) -> Result<CalendarRule, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_RULE",
                json!({
                    "name": name,
                    "description": description,
                    "dsl": dsl.clone(),
                    "approval_mode": approval_mode,
                    "action": "create_calendar_rule",
                }),
                "calendar-rule://create".to_owned(),
                json!({
                    "captured_by": "calendar_service.create_calendar_rule_manual",
                    "operation": "create_calendar_rule_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(CalendarRuleStore::new(self.pool.clone())
            .create_with_observation(
                name,
                description,
                dsl,
                approval_mode,
                Some(&observation.observation_id),
                "create",
                None,
            )
            .await?)
    }

    pub async fn update_calendar_rule_manual(
        &self,
        rule_id: &str,
        update: &RuleUpdate,
    ) -> Result<CalendarRule, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_RULE",
                json!({
                    "rule_id": rule_id,
                    "update": serde_json::to_value(update).unwrap_or(Value::Null),
                    "action": "update_calendar_rule",
                }),
                format!("calendar-rule://{rule_id}/update"),
                json!({
                    "captured_by": "calendar_service.update_calendar_rule_manual",
                    "operation": "update_calendar_rule_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(CalendarRuleStore::new(self.pool.clone())
            .update_with_observation(
                rule_id,
                update,
                Some(&observation.observation_id),
                "update",
                None,
            )
            .await?)
    }

    pub async fn delete_calendar_rule_manual(
        &self,
        rule_id: &str,
    ) -> Result<(), CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_RULE",
                json!({
                    "rule_id": rule_id,
                    "action": "delete_calendar_rule",
                }),
                format!("calendar-rule://{rule_id}/delete"),
                json!({
                    "captured_by": "calendar_service.delete_calendar_rule_manual",
                    "operation": "delete_calendar_rule_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        CalendarRuleStore::new(self.pool.clone())
            .delete_with_observation(rule_id, Some(&observation.observation_id), "delete", None)
            .await?;
        Ok(())
    }

    pub async fn create_deadline_manual(
        &self,
        title: &str,
        due_at: DateTime<Utc>,
        severity: Option<&str>,
        source_entity_type: Option<&str>,
        source_entity_id: Option<&str>,
    ) -> Result<DeadlineEvent, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_EVENT",
                json!({
                    "title": title,
                    "due_at": due_at,
                    "severity": severity,
                    "source_entity_type": source_entity_type,
                    "source_entity_id": source_entity_id,
                    "action": "create_deadline",
                }),
                "calendar-scheduling://deadlines/create".to_owned(),
                json!({
                    "captured_by": "calendar_service.create_deadline_manual",
                    "operation": "create_deadline_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(DeadlineStore::new(self.pool.clone())
            .create_with_observation(
                title,
                due_at,
                severity,
                source_entity_type,
                source_entity_id,
                Some(&observation.observation_id),
                "create",
                None,
            )
            .await?)
    }

    pub async fn create_focus_block_manual(
        &self,
        title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        purpose: Option<&str>,
        linked_project_id: Option<&str>,
        protection_level: Option<&str>,
    ) -> Result<FocusBlock, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_EVENT",
                json!({
                    "title": title,
                    "start_at": start_at,
                    "end_at": end_at,
                    "purpose": purpose,
                    "linked_project_id": linked_project_id,
                    "protection_level": protection_level,
                    "action": "create_focus_block",
                }),
                "calendar-scheduling://focus-blocks/create".to_owned(),
                json!({
                    "captured_by": "calendar_service.create_focus_block_manual",
                    "operation": "create_focus_block_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(FocusBlockStore::new(self.pool.clone())
            .create_with_observation(
                title,
                start_at,
                end_at,
                purpose,
                linked_project_id,
                protection_level,
                Some(&observation.observation_id),
                "create",
                None,
            )
            .await?)
    }

    pub async fn trigger_calendar_sync_manual(
        &self,
        account_id: &str,
    ) -> Result<CalendarAccount, CalendarCommandServiceError> {
        let observation = self
            .capture_manual(
                "CALENDAR_ACCOUNT_MUTATION",
                json!({
                    "account_id": account_id,
                    "sync_status": "syncing",
                    "action": "trigger_calendar_sync",
                }),
                format!("calendar-account://{account_id}/sync"),
                json!({
                    "captured_by": "calendar_service.trigger_calendar_sync_manual",
                    "operation": "trigger_calendar_sync_manual",
                }),
            )
            .await
            .map_err(CalendarCommandServiceError::ObservationCapture)?;

        Ok(CalendarAccountStore::new(self.pool.clone())
            .update_with_observation(
                account_id,
                &CalendarAccountUpdate {
                    sync_status: Some("syncing".into()),
                    ..Default::default()
                },
                Some(&observation.observation_id),
                "sync_trigger",
                Some(json!({
                    "account_id": account_id,
                    "sync_status": "syncing",
                })),
            )
            .await?)
    }

    async fn capture_manual(
        &self,
        kind: &str,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<makosh_observations_api::models::Observation, ObservationStoreError> {
        ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    kind,
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await
    }
}

#[derive(Debug, Error)]
pub enum CalendarCommandServiceError {
    #[error("calendar observation capture failed")]
    ObservationCapture(#[source] ObservationStoreError),

    #[error(transparent)]
    Calendar(#[from] CalendarError),

    #[error(transparent)]
    CalendarCore(#[from] CalendarCoreError),

    #[error(transparent)]
    Meetings(#[from] MeetingsError),

    #[error(transparent)]
    Reminder(#[from] ReminderError),

    #[error(transparent)]
    CalendarRule(#[from] CalendarRuleError),

    #[error(transparent)]
    Scheduling(#[from] SchedulingError),
}
