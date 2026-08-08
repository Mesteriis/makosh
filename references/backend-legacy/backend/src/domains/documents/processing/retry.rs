use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::{Value, json};

use crate::domains::documents::processing::evidence::link_document_processing_entity_in_transaction;
use makosh_events_postgres::store::EventStore;

use super::constants::{
    RETRY_EVENT_ID_PREFIX, RETRY_EVENT_TYPE, RETRY_SOURCE_KIND, RETRY_SOURCE_PROVIDER,
};
use super::errors::DocumentProcessingError;
use super::models::{
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
};
use super::store::DocumentProcessingStore;
use super::validation::validate_non_empty;

impl DocumentProcessingStore {
    pub async fn retry_failed_job(
        &self,
        command: &DocumentProcessingRetryCommand,
    ) -> Result<DocumentProcessingRetryCommandResult, DocumentProcessingError> {
        self.retry_failed_job_with_observation(command, None).await
    }

    pub async fn retry_failed_job_with_observation(
        &self,
        command: &DocumentProcessingRetryCommand,
        observation_id: Option<&str>,
    ) -> Result<DocumentProcessingRetryCommandResult, DocumentProcessingError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let job_id = validate_non_empty("job_id", &command.job_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;
        let event_id = format!("{RETRY_EVENT_ID_PREFIX}{command_id}");

        if let Some(result) = self
            .retry_result_for_existing_event(&event_id, &job_id)
            .await?
        {
            self.link_retry_observation_if_present(observation_id, &job_id, &event_id)
                .await?;
            return Ok(result);
        }

        let mut transaction = self.pool.begin().await?;
        let current_job = self.job_for_update(&mut transaction, &job_id).await?;
        if current_job.status != DocumentProcessingStatus::Failed {
            if let Some(result) = self
                .retry_result_for_existing_event(&event_id, &job_id)
                .await?
            {
                self.link_retry_observation_if_present(observation_id, &job_id, &event_id)
                    .await?;
                return Ok(result);
            }
            return Err(DocumentProcessingError::RetryRequiresFailedJob);
        }

        let event = RetryCommandEvent {
            command_id,
            job_id: job_id.clone(),
            actor_id,
            event_id: event_id.clone(),
            occurred_at: Utc::now(),
        }
        .into_event()?;

        if let Err(error) = EventStore::append_in_transaction(&mut transaction, &event).await {
            if error.is_unique_violation() {
                transaction.rollback().await?;
                return self
                    .retry_result_for_existing_event(&event_id, &job_id)
                    .await?
                    .ok_or(DocumentProcessingError::RetryCommandConflict);
            }

            return Err(DocumentProcessingError::EventStore(error));
        }
        let retried_job = self.requeue_failed_job(&mut transaction, &job_id).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_document_processing_entity_in_transaction(
                &mut transaction,
                observation_id,
                "document_processing_job",
                retried_job.job_id.clone(),
                "retry_command",
                json!({
                    "event_id": event_id,
                }),
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(DocumentProcessingRetryCommandResult {
            job_id: retried_job.job_id,
            status: retried_job.status,
            event_id,
        })
    }

    async fn link_retry_observation_if_present(
        &self,
        observation_id: Option<&str>,
        job_id: &str,
        event_id: &str,
    ) -> Result<(), DocumentProcessingError> {
        let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) else {
            return Ok(());
        };

        let mut transaction = self.pool.begin().await?;
        link_document_processing_entity_in_transaction(
            &mut transaction,
            observation_id,
            "document_processing_job",
            job_id.to_owned(),
            "retry_command",
            json!({
                "event_id": event_id,
            }),
        )
        .await?;
        transaction.commit().await?;

        Ok(())
    }

    async fn retry_result_for_existing_event(
        &self,
        event_id: &str,
        job_id: &str,
    ) -> Result<Option<DocumentProcessingRetryCommandResult>, DocumentProcessingError> {
        let Some(event) = EventStore::new(self.pool.clone())
            .get_by_id(event_id)
            .await?
        else {
            return Ok(None);
        };

        let Some(event_job_id) = event.payload.get("job_id").and_then(Value::as_str) else {
            return Err(DocumentProcessingError::RetryCommandConflict);
        };

        if event.event_type != RETRY_EVENT_TYPE || event_job_id != job_id {
            return Err(DocumentProcessingError::RetryCommandConflict);
        }

        Ok(Some(DocumentProcessingRetryCommandResult {
            job_id: job_id.to_owned(),
            status: DocumentProcessingStatus::Queued,
            event_id: event_id.to_owned(),
        }))
    }
}

#[derive(Debug)]
struct RetryCommandEvent {
    command_id: String,
    job_id: String,
    actor_id: String,
    event_id: String,
    occurred_at: DateTime<Utc>,
}

impl RetryCommandEvent {
    fn into_event(self) -> Result<NewEventEnvelope, DocumentProcessingError> {
        let job_id = self.job_id;
        Ok(NewEventEnvelope::builder(
            self.event_id,
            RETRY_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": RETRY_SOURCE_KIND,
                "provider": RETRY_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "document_processing_job",
                "job_id": job_id.clone(),
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(json!({ "job_id": job_id }))
        .build()?)
    }
}
