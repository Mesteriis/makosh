use makosh_communication_delivery_intent_persistence::{
    CommunicationDeliveryIntentPersistenceV1, DeliveryIntentPersistenceErrorV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

use crate::runtime::{DeliveryIntentManagedRuntimeV1, DeliveryIntentRuntimeErrorV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentIngressResultRelayErrorV1 {
    InvalidTimestamp,
    Persistence(DeliveryIntentPersistenceErrorV1),
    EventUnavailable,
}

pub async fn relay_ingress_result_once_v1(
    persistence: &CommunicationDeliveryIntentPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<bool, DeliveryIntentIngressResultRelayErrorV1> {
    if published_at_unix_seconds <= 0 {
        return Err(DeliveryIntentIngressResultRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .pending_ingress_results(1)
        .await
        .map_err(DeliveryIntentIngressResultRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, record.exact_bytes())
        .await
        .map_err(|_| DeliveryIntentIngressResultRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_ingress_result_published(*record.message_id(), published_at_unix_seconds)
        .await
        .map_err(DeliveryIntentIngressResultRelayErrorV1::Persistence)?;
    Ok(true)
}

impl DeliveryIntentManagedRuntimeV1 {
    pub async fn relay_ingress_result_once_v1(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<bool, DeliveryIntentRuntimeErrorV1> {
        relay_ingress_result_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.event_publish_permit,
            published_at_unix_seconds,
        )
        .await
        .map_err(|error| match error {
            DeliveryIntentIngressResultRelayErrorV1::InvalidTimestamp => {
                DeliveryIntentRuntimeErrorV1::InvalidRequest
            }
            DeliveryIntentIngressResultRelayErrorV1::Persistence(error) => {
                DeliveryIntentRuntimeErrorV1::Persistence(error)
            }
            DeliveryIntentIngressResultRelayErrorV1::EventUnavailable => {
                DeliveryIntentRuntimeErrorV1::Unavailable
            }
        })
    }
}
