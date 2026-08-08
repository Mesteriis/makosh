use makosh_events_api::NewEventEnvelope;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

use makosh_communications_api::commands::NewCommunicationProviderCommand;
use makosh_events_postgres::store::EventStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

use super::evidence::link_mail_entity_in_transaction;
use makosh_communications_postgres::provider_commands::{
    CommunicationProviderCommandError, CommunicationProviderCommandStore,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BulkMessageAction {
    MarkRead,
    MarkUnread,
    Archive,
    Trash,
    Restore,
    Pin,
    Unpin,
    Important,
    NotImportant,
    Star,
    Unstar,
    AddLabel(String),
    RemoveLabel(String),
    Snooze(DateTime<Utc>),
}

impl BulkMessageAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MarkRead => "mark_read",
            Self::MarkUnread => "mark_unread",
            Self::Archive => "archive",
            Self::Trash => "trash",
            Self::Restore => "restore",
            Self::Pin => "pin",
            Self::Unpin => "unpin",
            Self::Important => "important",
            Self::NotImportant => "not_important",
            Self::Star => "star",
            Self::Unstar => "unstar",
            Self::AddLabel(_) => "add_label",
            Self::RemoveLabel(_) => "remove_label",
            Self::Snooze(_) => "snooze",
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            Self::MarkRead | Self::MarkUnread => "communication.message.read_state_changed.v1",
            Self::Archive => "mail.message.archived",
            Self::Trash => "mail.message.deleted",
            Self::Restore => "mail.message.restored",
            Self::Pin => "mail.message.pinned",
            Self::Unpin => "mail.message.unpinned",
            Self::Important => "mail.message.important",
            Self::NotImportant => "mail.message.not_important",
            Self::Star => "mail.message.starred",
            Self::Unstar => "mail.message.unstarred",
            Self::AddLabel(_) => "mail.message.labeled",
            Self::RemoveLabel(_) => "mail.message.unlabeled",
            Self::Snooze(_) => "mail.message.snoozed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BulkMessageActionOutcome {
    pub action: String,
    pub requested_count: usize,
    pub matched_count: usize,
    pub updated_count: usize,
    pub not_found: Vec<String>,
}

pub struct BulkMessageActionStore {
    pool: PgPool,
}

impl BulkMessageActionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn apply(
        &self,
        message_ids: Vec<String>,
        action: BulkMessageAction,
    ) -> Result<BulkMessageActionOutcome, BulkMessageActionError> {
        let message_ids = normalize_message_ids(message_ids)?;
        let mut transaction = self.pool.begin().await?;
        let updated_ids = match &action {
            BulkMessageAction::MarkRead => {
                self.update_read_state(&mut transaction, &message_ids, true)
                    .await?
            }
            BulkMessageAction::MarkUnread => {
                self.update_read_state(&mut transaction, &message_ids, false)
                    .await?
            }
            BulkMessageAction::Archive => {
                self.update_workflow_state(&mut transaction, &message_ids, "archived")
                    .await?
            }
            BulkMessageAction::Trash => self.move_to_trash(&mut transaction, &message_ids).await?,
            BulkMessageAction::Restore => {
                self.restore_from_trash(&mut transaction, &message_ids)
                    .await?
            }
            BulkMessageAction::Pin => {
                self.set_metadata_bool(&mut transaction, &message_ids, "pinned", true)
                    .await?
            }
            BulkMessageAction::Unpin => {
                self.set_metadata_bool(&mut transaction, &message_ids, "pinned", false)
                    .await?
            }
            BulkMessageAction::Important => {
                self.set_provider_flag_bool(&mut transaction, &message_ids, "important", true)
                    .await?
            }
            BulkMessageAction::NotImportant => {
                self.set_provider_flag_bool(&mut transaction, &message_ids, "important", false)
                    .await?
            }
            BulkMessageAction::Star => {
                self.set_provider_flag_bool(&mut transaction, &message_ids, "starred", true)
                    .await?
            }
            BulkMessageAction::Unstar => {
                self.set_provider_flag_bool(&mut transaction, &message_ids, "starred", false)
                    .await?
            }
            BulkMessageAction::AddLabel(label) => {
                self.add_label(&mut transaction, &message_ids, label)
                    .await?
            }
            BulkMessageAction::RemoveLabel(label) => {
                self.remove_label(&mut transaction, &message_ids, label)
                    .await?
            }
            BulkMessageAction::Snooze(until) => {
                self.snooze(&mut transaction, &message_ids, until).await?
            }
        };
        let outcome = outcome(action.as_str(), &message_ids, updated_ids.clone());

        if !updated_ids.is_empty() {
            if provider_command_kind(&action).is_some() {
                self.enqueue_provider_commands(&mut transaction, &updated_ids, &action)
                    .await?;
            }
            self.capture_observation_trail(&mut transaction, &action, &outcome, &updated_ids)
                .await?;
            let event = bulk_message_action_event(&action, &outcome, &updated_ids)?;
            EventStore::append_in_transaction(&mut transaction, &event).await?;
        }
        transaction.commit().await?;

        Ok(outcome)
    }

    async fn update_workflow_state(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        workflow_state: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET workflow_state = $2, projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
            "#,
        )
        .bind(message_ids)
        .bind(workflow_state)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn update_read_state(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        is_read: bool,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET is_read = $2,
                read_changed_at = now(),
                read_origin = 'local_user',
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
            "#,
        )
        .bind(message_ids)
        .bind(is_read)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn enqueue_provider_commands(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        action: &BulkMessageAction,
    ) -> Result<(), BulkMessageActionError> {
        let rows = sqlx::query(
            r#"
            SELECT message_id, account_id, provider_record_id, message_metadata
            FROM communication_messages
            WHERE message_id = ANY($1)
            "#,
        )
        .bind(message_ids)
        .fetch_all(&mut **transaction)
        .await?;
        let command_kind = provider_command_kind(action).ok_or(BulkMessageActionError::Invalid(
            "action does not have a provider command",
        ))?;
        for row in rows {
            let message_id: String = row.try_get("message_id")?;
            let account_id: String = row.try_get("account_id")?;
            let provider_record_id: String = row.try_get("provider_record_id")?;
            let message_metadata: Value = row.try_get("message_metadata")?;
            let command_id = format!("mail-command:{}", Uuid::new_v4());
            let command = NewCommunicationProviderCommand::new(
                &command_id,
                account_id,
                "mail",
                command_kind,
                &command_id,
                "makosh-local-user",
            )
            .provider_message_id(&provider_record_id)
            .target_ref(json!({ "message_id": message_id }))
            .payload(provider_command_payload(
                action,
                &provider_record_id,
                &message_metadata,
            ));
            CommunicationProviderCommandStore::enqueue_in_transaction(transaction, &command)
                .await?;
        }
        Ok(())
    }

    async fn move_to_trash(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET local_state = 'trash',
                local_state_changed_at = now(),
                local_state_reason = 'bulk_action',
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
        "#,
        )
        .bind(message_ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn restore_from_trash(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET local_state = 'active',
                local_state_changed_at = now(),
                local_state_reason = NULL,
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
        "#,
        )
        .bind(message_ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn set_metadata_bool(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        key: &str,
        value: bool,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        let path = vec![key.to_owned()];
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    COALESCE(message_metadata, '{}'::jsonb),
                    $2,
                    to_jsonb($3::boolean),
                    true
                ),
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
            "#,
        )
        .bind(message_ids)
        .bind(path)
        .bind(value)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn set_provider_flag_bool(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        key: &str,
        value: bool,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        let value_path = vec![key.to_owned()];
        let origin_path = vec![format!("{key}_origin")];
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    jsonb_set(
                        COALESCE(message_metadata, '{}'::jsonb),
                        $2,
                        to_jsonb($3::boolean),
                        true
                    ),
                    $4,
                    '"local_user"'::jsonb,
                    true
                ),
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
            "#,
        )
        .bind(message_ids)
        .bind(value_path)
        .bind(value)
        .bind(origin_path)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn add_label(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        label: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        validate_label(label)?;
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages AS m
            SET message_metadata = jsonb_set(
                    COALESCE(m.message_metadata, '{}'::jsonb),
                    '{labels}',
                    (
                        SELECT COALESCE(jsonb_agg(label_value ORDER BY label_value), '[]'::jsonb)
                        FROM (
                            SELECT DISTINCT label_value
                            FROM (
                                SELECT jsonb_array_elements_text(
                                    CASE
                                        WHEN jsonb_typeof(COALESCE(m.message_metadata, '{}'::jsonb)->'labels') = 'array'
                                        THEN m.message_metadata->'labels'
                                        ELSE '[]'::jsonb
                                    END
                                ) AS label_value
                                UNION ALL
                                SELECT $2::text AS label_value
                            ) labels
                            WHERE trim(label_value) <> ''
                        ) distinct_labels
                    ),
                    true
                ),
                projected_at = now()
            WHERE m.message_id = ANY($1)
            RETURNING m.message_id
        "#,
        )
        .bind(message_ids)
        .bind(label.trim())
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn remove_label(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        label: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        validate_label(label)?;
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages AS m
            SET message_metadata = jsonb_set(
                    COALESCE(m.message_metadata, '{}'::jsonb),
                    '{labels}',
                    (
                        SELECT COALESCE(jsonb_agg(label_value ORDER BY label_value), '[]'::jsonb)
                        FROM (
                            SELECT DISTINCT label_value
                            FROM jsonb_array_elements_text(
                                CASE
                                    WHEN jsonb_typeof(COALESCE(m.message_metadata, '{}'::jsonb)->'labels') = 'array'
                                    THEN m.message_metadata->'labels'
                                    ELSE '[]'::jsonb
                                END
                            ) AS label_value
                            WHERE label_value <> $2::text
                              AND trim(label_value) <> ''
                        ) remaining_labels
                    ),
                    true
                ),
                projected_at = now()
            WHERE m.message_id = ANY($1)
            RETURNING m.message_id
        "#,
        )
        .bind(message_ids)
        .bind(label.trim())
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn snooze(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        until: &DateTime<Utc>,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    COALESCE(message_metadata, '{}'::jsonb),
                    '{snooze_until}',
                    to_jsonb($2::text),
                    true
                ),
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
        "#,
        )
        .bind(message_ids)
        .bind(until.to_rfc3339())
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn capture_observation_trail(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        action: &BulkMessageAction,
        outcome: &BulkMessageActionOutcome,
        updated_ids: &[String],
    ) -> Result<(), BulkMessageActionError> {
        let recorded_at = Utc::now();
        for message_id in updated_ids {
            let observation = ObservationStore::capture_in_transaction(
                transaction,
                &NewObservation::new(
                    "COMMUNICATION_MESSAGE",
                    ObservationOriginKind::Manual,
                    recorded_at,
                    bulk_action_observation_payload(action, outcome, message_id),
                    format!(
                        "message://{message_id}/bulk/{}/{}",
                        action.as_str(),
                        system_time_nanos()
                    ),
                )
                .provenance(json!({
                    "captured_by": "mail.bulk_actions",
                    "operation": action.as_str(),
                    "event_type": action.event_type(),
                })),
            )
            .await?;

            link_mail_entity_in_transaction(
                transaction,
                &observation.observation_id,
                "communication_message",
                message_id.to_owned(),
                bulk_action_relationship_kind(action),
                bulk_action_link_metadata(action),
                None,
            )
            .await?;
        }

        Ok(())
    }
}

fn bulk_action_relationship_kind(action: &BulkMessageAction) -> &'static str {
    match action {
        BulkMessageAction::MarkRead | BulkMessageAction::MarkUnread => "read_state_transition",
        BulkMessageAction::Archive => "workflow_state_transition",
        BulkMessageAction::Trash | BulkMessageAction::Restore => "local_state_transition",
        BulkMessageAction::Pin
        | BulkMessageAction::Unpin
        | BulkMessageAction::Important
        | BulkMessageAction::NotImportant
        | BulkMessageAction::Star
        | BulkMessageAction::Unstar
        | BulkMessageAction::AddLabel(_)
        | BulkMessageAction::RemoveLabel(_)
        | BulkMessageAction::Snooze(_) => "message_flag_update",
    }
}

fn provider_command_kind(action: &BulkMessageAction) -> Option<&'static str> {
    match action {
        BulkMessageAction::MarkRead => Some("mark_read"),
        BulkMessageAction::MarkUnread => Some("mark_unread"),
        BulkMessageAction::Archive => Some("archive"),
        BulkMessageAction::Trash => Some("trash"),
        BulkMessageAction::Important => Some("important"),
        BulkMessageAction::NotImportant => Some("not_important"),
        BulkMessageAction::Star => Some("star"),
        BulkMessageAction::Unstar => Some("unstar"),
        BulkMessageAction::AddLabel(_) => Some("add_label"),
        BulkMessageAction::RemoveLabel(_) => Some("remove_label"),
        _ => None,
    }
}

fn provider_command_payload(
    action: &BulkMessageAction,
    provider_record_id: &str,
    message_metadata: &Value,
) -> Value {
    let mut payload = json!({
        "provider_record_id": provider_record_id,
        "message_metadata": message_metadata,
    });
    if let Some(is_read) = match action {
        BulkMessageAction::MarkRead => Some(true),
        BulkMessageAction::MarkUnread => Some(false),
        _ => None,
    } {
        payload["desired_is_read"] = json!(is_read);
    }
    match action {
        BulkMessageAction::AddLabel(label) | BulkMessageAction::RemoveLabel(label) => {
            payload["label"] = json!(label.trim());
        }
        _ => {}
    }
    payload
}

fn bulk_action_link_metadata(action: &BulkMessageAction) -> Value {
    match action {
        BulkMessageAction::MarkRead => json!({ "is_read": true }),
        BulkMessageAction::MarkUnread => json!({ "is_read": false }),
        BulkMessageAction::Archive => json!({ "workflow_state": "archived" }),
        BulkMessageAction::Trash => json!({ "local_state": "trash" }),
        BulkMessageAction::Restore => json!({ "local_state": "active" }),
        BulkMessageAction::Pin => json!({ "pinned": true }),
        BulkMessageAction::Unpin => json!({ "pinned": false }),
        BulkMessageAction::Important => json!({ "important": true }),
        BulkMessageAction::NotImportant => json!({ "important": false }),
        BulkMessageAction::Star => json!({ "starred": true }),
        BulkMessageAction::Unstar => json!({ "starred": false }),
        BulkMessageAction::AddLabel(label) => json!({ "label": label.trim(), "action": "add" }),
        BulkMessageAction::RemoveLabel(label) => {
            json!({ "label": label.trim(), "action": "remove" })
        }
        BulkMessageAction::Snooze(until) => json!({ "snooze_until": until.to_rfc3339() }),
    }
}

fn bulk_action_observation_payload(
    action: &BulkMessageAction,
    outcome: &BulkMessageActionOutcome,
    message_id: &str,
) -> Value {
    json!({
        "message_id": message_id,
        "operation": format!("bulk_{}", action.as_str()),
        "action": action.as_str(),
        "action_parameters": action_parameters(action),
        "requested_count": outcome.requested_count,
        "matched_count": outcome.matched_count,
        "updated_count": outcome.updated_count,
        "not_found": outcome.not_found,
    })
}

fn bulk_message_action_event(
    action: &BulkMessageAction,
    outcome: &BulkMessageActionOutcome,
    updated_ids: &[String],
) -> Result<NewEventEnvelope, BulkMessageActionError> {
    let event_id = format!(
        "mail_message_action_event:{}:{:x}",
        outcome.action,
        system_time_nanos()
    );

    Ok(NewEventEnvelope::builder(
        event_id.clone(),
        action.event_type(),
        Utc::now(),
        json!({ "kind": "mail_bulk_action_api" }),
        json!({
            "kind": "mail_message_bulk_action",
            "id": event_id,
            "message_ids": updated_ids,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(json!({
        "action": outcome.action,
        "action_parameters": action_parameters(action),
        "requested_count": outcome.requested_count,
        "matched_count": outcome.matched_count,
        "updated_count": outcome.updated_count,
        "message_ids": updated_ids,
        "not_found": outcome.not_found,
    }))
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": outcome.action,
    }))
    .correlation_id(outcome.action.clone())
    .build()?)
}

fn action_parameters(action: &BulkMessageAction) -> Value {
    match action {
        BulkMessageAction::AddLabel(label) => json!({ "label": label.trim() }),
        BulkMessageAction::RemoveLabel(label) => json!({ "label": label.trim() }),
        BulkMessageAction::Snooze(until) => json!({ "snooze_until": until.to_rfc3339() }),
        _ => json!({}),
    }
}

fn normalize_message_ids(message_ids: Vec<String>) -> Result<Vec<String>, BulkMessageActionError> {
    if message_ids.is_empty() {
        return Err(BulkMessageActionError::Invalid(
            "message_ids must not be empty",
        ));
    }
    if message_ids.len() > 500 {
        return Err(BulkMessageActionError::Invalid(
            "message_ids must contain at most 500 items",
        ));
    }

    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for message_id in message_ids {
        let message_id = message_id.trim();
        if message_id.is_empty() {
            return Err(BulkMessageActionError::Invalid(
                "message_ids must not contain empty values",
            ));
        }
        if seen.insert(message_id.to_owned()) {
            normalized.push(message_id.to_owned());
        }
    }

    Ok(normalized)
}

fn validate_label(label: &str) -> Result<(), BulkMessageActionError> {
    if label.trim().is_empty() {
        return Err(BulkMessageActionError::Invalid("label must not be empty"));
    }
    Ok(())
}

fn outcome(
    action: &str,
    requested_ids: &[String],
    updated_ids: Vec<String>,
) -> BulkMessageActionOutcome {
    let updated = updated_ids.into_iter().collect::<HashSet<_>>();
    let not_found = requested_ids
        .iter()
        .filter(|message_id| !updated.contains(*message_id))
        .cloned()
        .collect::<Vec<_>>();
    let updated_count = updated.len();

    BulkMessageActionOutcome {
        action: action.to_owned(),
        requested_count: requested_ids.len(),
        matched_count: updated_count,
        updated_count,
        not_found,
    }
}

#[derive(Debug, Error)]
pub enum BulkMessageActionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ObservationStore(#[from] makosh_observations_postgres::errors::ObservationStoreError),
    #[error(transparent)]
    EventStore(#[from] makosh_events_postgres::errors::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),
    #[error(transparent)]
    ProviderCommand(#[from] CommunicationProviderCommandError),
    #[error("invalid bulk message action request: {0}")]
    Invalid(&'static str),
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
