use makosh_events_api::EventEnvelope;
use makosh_events_postgres::store::EventStore;
use makosh_signal_hub_api::raw_signals::{
    RawSignalCommandPort, RawSignalInput, RawSignalOutcome, RawSignalPortError,
    RawSignalPortFuture, RawSignalRuntimeQueryPort,
};
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;
use sqlx::postgres::PgPool;
use std::sync::Arc;

use super::service::{
    SignalHubSignalService, SignalProcessingOutcome, signal_hub_raw_dispatcher_allows_processing,
};
use super::store::{SignalHubError, SignalHubStore};

/// Provider-facing raw-signal boundary.
///
/// It keeps source policy resolution and canonical signal emission inside Signal
/// Hub while callers retain no access to its PostgreSQL store.
#[derive(Clone)]
pub struct RawSignalProcessingPort {
    signal_store: SignalHubStore,
    raw_signal_store: Arc<RawSignalStore>,
    event_store: EventStore,
}

impl RawSignalProcessingPort {
    pub fn new(pool: PgPool) -> Self {
        Self {
            signal_store: SignalHubStore::new(pool.clone()),
            raw_signal_store: Arc::new(RawSignalStore::new(pool.clone())),
            event_store: EventStore::new(pool),
        }
    }

    pub async fn allows_processing(&self) -> Result<bool, SignalHubError> {
        signal_hub_raw_dispatcher_allows_processing(&self.signal_store).await
    }

    pub async fn process_raw_signal(
        &self,
        raw_event: &EventEnvelope,
    ) -> Result<SignalProcessingOutcome, SignalHubError> {
        SignalHubSignalService::new(self.raw_signal_store.clone(), self.event_store.clone())
            .process_raw_signal(raw_event)
            .await
    }
}

impl RawSignalCommandPort for RawSignalProcessingPort {
    fn process_raw_signal<'a>(
        &'a self,
        input: &'a RawSignalInput,
    ) -> RawSignalPortFuture<'a, RawSignalOutcome> {
        Box::pin(async move {
            RawSignalProcessingPort::process_raw_signal(self, &input.event)
                .await
                .map(raw_signal_outcome)
                .map_err(RawSignalPortError::new)
        })
    }
}

impl RawSignalRuntimeQueryPort for RawSignalProcessingPort {
    fn allows_processing<'a>(&'a self) -> RawSignalPortFuture<'a, bool> {
        Box::pin(async move {
            RawSignalProcessingPort::allows_processing(self)
                .await
                .map_err(RawSignalPortError::new)
        })
    }
}

fn raw_signal_outcome(outcome: SignalProcessingOutcome) -> RawSignalOutcome {
    match outcome {
        SignalProcessingOutcome::Accepted { event_id } => RawSignalOutcome::Accepted { event_id },
        SignalProcessingOutcome::Rejected { reason } => RawSignalOutcome::Rejected { reason },
        SignalProcessingOutcome::Muted { reason } => RawSignalOutcome::Muted { reason },
        SignalProcessingOutcome::Paused { reason } => RawSignalOutcome::Paused { reason },
    }
}
