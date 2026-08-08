use crate::application::communication_fixture_error::CommunicationFixtureIngestError;
use crate::application::communication_fixture_event::build as build_event;
use crate::domains::communications::messages::provider_observation_projection::project_accepted_signal_if_runtime_allows;
use crate::domains::communications::ports::CommunicationRawEvidencePort;
use crate::domains::signal_hub::telegram::dispatch_telegram_raw_signal;
use crate::integrations::telegram::client::fixture_port::TelegramFixturePort;
use crate::integrations::telegram::client::models::messages::{
    NewTelegramMessage, TelegramMessageIngestResult,
};
use crate::platform::events::bus::{InMemoryEventBus, telegram_event_types};
use crate::workflows::review_inbox::{
    refresh_message_decisions_into_review, refresh_message_task_candidates_into_review,
};
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::store::EventStore;
use serde_json::json;
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub(crate) struct TelegramFixtureIngestApplicationService {
    pool: PgPool,
    store: TelegramFixturePort,
    event_store: EventStore,
    event_bus: InMemoryEventBus,
}

impl TelegramFixtureIngestApplicationService {
    pub(crate) fn new(
        pool: PgPool,
        store: TelegramFixturePort,
        event_store: EventStore,
        event_bus: InMemoryEventBus,
    ) -> Self {
        Self {
            pool,
            store,
            event_store,
            event_bus,
        }
    }

    pub(crate) async fn ingest_message(
        &self,
        request: &NewTelegramMessage,
    ) -> Result<TelegramMessageIngestResult, CommunicationFixtureIngestError> {
        let observed = self.store.ingest_fixture_message(request).await?;
        let stored_raw = CommunicationRawEvidencePort::new(self.pool.clone())
            .record_raw_source(&observed.raw)
            .await?;
        let Some(accepted_event) =
            dispatch_telegram_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "telegram fixture signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        let Some(projected) =
            project_accepted_signal_if_runtime_allows(self.pool.clone(), &accepted_event).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "telegram fixture signal did not produce an accepted projection".to_owned(),
            ));
        };
        let response = TelegramMessageIngestResult {
            raw_record_id: projected.raw_record_id,
            message_id: projected.message_id,
        };
        self.store
            .recompute_chat_unread_count(&observed.telegram_chat_id)
            .await?;
        let message_ids = vec![response.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;
        let event = build_event(
            telegram_event_types::MESSAGE_CREATED,
            &request.account_id,
            &response.message_id,
            self.store
                .snapshot_payload(
                    &response.message_id,
                    json!({
                        "provider_chat_id": &request.provider_chat_id,
                        "provider_message_id": &request.provider_message_id,
                        "delivery_state": request.delivery_state.as_str(),
                        "runtime_kind": "fixture",
                    }),
                )
                .await?,
        );
        self.publish_event(event).await?;
        Ok(response)
    }

    async fn publish_event(
        &self,
        event: NewEventEnvelope,
    ) -> Result<(), CommunicationFixtureIngestError> {
        if let Err(error) = self.event_store.append(&event).await {
            tracing::warn!(error = %error, "failed to append fixture ingest event");
            return Err(error.into());
        }
        let _ = self.event_bus.broadcast(event);
        Ok(())
    }
}
