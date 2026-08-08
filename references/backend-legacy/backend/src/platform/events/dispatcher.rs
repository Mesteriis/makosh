use chrono::{DateTime, Duration, Utc};

use crate::platform::events::bus::InMemoryEventBus;
use makosh_events_api::DispatchableEventOutboxItem;
use makosh_events_nats::jetstream::{NatsJetStreamEventBus, NatsJetStreamEventBusError};
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

const DEFAULT_DISPATCH_BATCH_SIZE: u32 = 100;
const DEFAULT_STALE_DISPATCH_AFTER_SECONDS: i64 = 60;

#[derive(Clone)]
pub struct EventOutboxDispatcher {
    store: EventStore,
    bus: NatsJetStreamEventBus,
    realtime_bus: Option<InMemoryEventBus>,
    batch_size: u32,
    stale_dispatch_after: Duration,
}

impl EventOutboxDispatcher {
    pub fn new(store: EventStore, bus: NatsJetStreamEventBus) -> Self {
        Self {
            store,
            bus,
            realtime_bus: None,
            batch_size: DEFAULT_DISPATCH_BATCH_SIZE,
            stale_dispatch_after: Duration::seconds(DEFAULT_STALE_DISPATCH_AFTER_SECONDS),
        }
    }

    pub fn with_realtime_bus(mut self, realtime_bus: InMemoryEventBus) -> Self {
        self.realtime_bus = Some(realtime_bus);
        self
    }

    pub fn with_batch_size(mut self, batch_size: u32) -> Self {
        self.batch_size = batch_size.clamp(1, 1000);
        self
    }

    pub fn with_stale_dispatch_after(mut self, stale_dispatch_after: Duration) -> Self {
        self.stale_dispatch_after = stale_dispatch_after;
        self
    }

    pub async fn dispatch_pending_once(&self) -> Result<EventDispatchReport, EventDispatcherError> {
        let recovered = self
            .store
            .recover_stale_outbox_items(self.stale_dispatch_after)
            .await?;
        let items = self
            .store
            .claim_pending_outbox_batch(self.batch_size)
            .await?;

        let mut report = EventDispatchReport {
            recovered,
            claimed: u32::try_from(items.len()).unwrap_or(u32::MAX),
            ..EventDispatchReport::default()
        };

        for item in items {
            if let Err(error) = self.dispatch_item(&item).await {
                report.retried += 1;
                tracing::warn!(
                    event_id = %item.event_id,
                    subject = %item.subject,
                    error = %error,
                    "event outbox dispatch failed"
                );
            } else {
                report.published += 1;
            }
        }

        Ok(report)
    }

    async fn dispatch_item(
        &self,
        item: &DispatchableEventOutboxItem,
    ) -> Result<(), EventDispatcherError> {
        match self.bus.publish(&item.event).await {
            Ok(()) => {
                self.store.mark_outbox_published(&item.event_id).await?;
                if let Some(realtime_bus) = &self.realtime_bus {
                    let _ = realtime_bus.broadcast_stored(&item.event);
                }
                Ok(())
            }
            Err(error) => {
                let next_attempt_at = next_attempt_at(item.attempts);
                self.store
                    .mark_outbox_retry(&item.event_id, &error.to_string(), next_attempt_at)
                    .await?;
                Err(error.into())
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct EventDispatchReport {
    pub recovered: u32,
    pub claimed: u32,
    pub published: u32,
    pub retried: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum EventDispatcherError {
    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Nats(#[from] NatsJetStreamEventBusError),
}

fn next_attempt_at(attempts: i32) -> DateTime<Utc> {
    let exponent = u32::try_from(attempts.saturating_sub(1))
        .unwrap_or(0)
        .min(6);
    let delay_seconds = (5_i64 * 2_i64.pow(exponent)).min(300);
    Utc::now() + Duration::seconds(delay_seconds)
}
