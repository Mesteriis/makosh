use crate::integrations::telegram::client::models::messages::TelegramObservedMessage;
use crate::integrations::telegram::client::store::TelegramStore;
use crate::platform::communications::errors::ProviderCommunicationMessagePortError;
use crate::platform::communications::raw_signals::{
    CommunicationRawSignalSource, build_communication_raw_signal_event,
};
use crate::platform::events::bus::InMemoryEventBus;
use makosh_events_postgres::store::EventStore;

use super::super::errors::TelegramError;

impl TelegramStore {
    pub(in crate::integrations::telegram) async fn publish_observed_message_raw_signal(
        &self,
        observed: &TelegramObservedMessage,
        event_bus: Option<&InMemoryEventBus>,
    ) -> Result<(), TelegramError> {
        let stored_raw = self
            .communication_raw_record_store()
            .record_raw_source(&observed.raw)
            .await?;
        let event = build_communication_raw_signal_event(
            CommunicationRawSignalSource::Telegram,
            &stored_raw,
            None,
        )
        .map_err(ProviderCommunicationMessagePortError::from)?;
        let appended = EventStore::new(self.pool().clone())
            .append_for_dispatch_idempotent(&event)
            .await
            .map_err(ProviderCommunicationMessagePortError::from)?;
        if appended.is_some()
            && let Some(event_bus) = event_bus
        {
            let _ = event_bus.broadcast(event);
        }
        Ok(())
    }
}
