//! Owner-local durable replay adapter for client-safe delivery-intent changes.

use std::os::unix::net::UnixStream;

use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1, COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
    COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_REALTIME_EVENT_KIND_V1,
    COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256, wire::DeliveryIntentStatusChangedV1,
};
use makosh_communication_delivery_intent_persistence::{
    CommunicationDeliveryIntentPersistenceV1, DeliveryIntentClientRealtimeTransitionV1,
    DeliveryIntentPersistenceErrorV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ContractReferenceV1, ManagedRuntimeClientRealtimePublishRequestV1,
        ManagedRuntimeControlRequestV1, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::client_realtime::{
        validate_managed_client_realtime_publish_request_v1,
        validate_managed_client_realtime_publish_response_v1,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::client_status::{rejection_value, status_value};

const REPLAY_WINDOW: u16 = 1_024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DeliveryIntentClientRealtimePublisherV1 {
    last_sequence: Option<u64>,
}

impl DeliveryIntentClientRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &CommunicationDeliveryIntentPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, DeliveryIntentClientRealtimeErrorV1> {
        let transitions = persistence
            .client_realtime_window(logical_owner_id, self.last_sequence, REPLAY_WINDOW)
            .await
            .map_err(DeliveryIntentClientRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let sequence = transition.sequence;
            publish_transition(channel, dispatcher, logical_owner_id, transition)?;
            self.last_sequence = Some(sequence);
        }
        Ok(published)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeliveryIntentClientRealtimeErrorV1 {
    InvalidTransition,
    Persistence(DeliveryIntentPersistenceErrorV1),
    Unavailable,
}

fn publish_transition(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    transition: DeliveryIntentClientRealtimeTransitionV1,
) -> Result<(), DeliveryIntentClientRealtimeErrorV1> {
    let request = publication(logical_owner_id, &transition)?;
    validate_managed_client_realtime_publish_request_v1(&request)
        .map_err(|_| DeliveryIntentClientRealtimeErrorV1::InvalidTransition)?;
    let expected_cursor = request.cursor.clone();
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::PublishClientRealtime(request)),
            },
            dispatcher,
        )
        .map_err(|_| DeliveryIntentClientRealtimeErrorV1::Unavailable)?;
    if !response.error_code.is_empty() {
        return Err(DeliveryIntentClientRealtimeErrorV1::Unavailable);
    }
    let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
        return Err(DeliveryIntentClientRealtimeErrorV1::Unavailable);
    };
    validate_managed_client_realtime_publish_response_v1(&response)
        .map_err(|_| DeliveryIntentClientRealtimeErrorV1::Unavailable)?;
    if response.accepted_cursor != expected_cursor {
        return Err(DeliveryIntentClientRealtimeErrorV1::Unavailable);
    }
    Ok(())
}

fn publication(
    logical_owner_id: &str,
    transition: &DeliveryIntentClientRealtimeTransitionV1,
) -> Result<ManagedRuntimeClientRealtimePublishRequestV1, DeliveryIntentClientRealtimeErrorV1> {
    let occurred_at_unix_millis = u64::try_from(transition.occurred_at_unix_seconds)
        .ok()
        .and_then(|seconds| seconds.checked_mul(1_000))
        .ok_or(DeliveryIntentClientRealtimeErrorV1::InvalidTransition)?;
    let payload = DeliveryIntentStatusChangedV1 {
        intent_id: transition.intent_id.to_vec(),
        status: status_value(transition.state),
        state_revision: transition.state_revision,
        occurred_at_unix_millis,
        rejection: rejection_value(transition.rejection_code),
    }
    .encode_to_vec();
    Ok(ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(realtime_contract()),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: event_id(transition).to_vec(),
        cursor: format!("communication-delivery-intent/{}", transition.sequence),
        event_kind: COMMUNICATION_DELIVERY_INTENT_REALTIME_EVENT_KIND_V1.to_owned(),
        occurred_at_unix_millis,
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload,
    })
}

fn realtime_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
        name: COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1.to_owned(),
        major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
    }
}

fn event_id(transition: &DeliveryIntentClientRealtimeTransitionV1) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(transition.intent_id);
    hasher.update(transition.state_revision.to_be_bytes());
    hasher.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has an exact length")
}

#[cfg(test)]
mod tests {
    use makosh_communication_delivery_intent_api::wire::{
        DeliveryIntentErrorCodeV1, DeliveryIntentStatusChangedV1,
    };
    use makosh_communication_delivery_intent_persistence::DeliveryIntentStateV1;

    use super::*;

    fn transition() -> DeliveryIntentClientRealtimeTransitionV1 {
        DeliveryIntentClientRealtimeTransitionV1 {
            sequence: 42,
            intent_id: [7; 16],
            state: DeliveryIntentStateV1::Rejected,
            state_revision: 5,
            rejection_code: Some(731),
            occurred_at_unix_seconds: 9,
        }
    }

    #[test]
    fn publication_contains_only_client_safe_status_data() {
        let request = publication("owner-1", &transition()).expect("publication");
        assert_eq!(request.cursor, "communication-delivery-intent/42");
        assert_eq!(request.event_id.len(), 16);
        let payload =
            DeliveryIntentStatusChangedV1::decode(request.payload.as_slice()).expect("payload");
        assert_eq!(payload.intent_id, vec![7; 16]);
        assert_eq!(payload.status, 5);
        assert_eq!(payload.state_revision, 5);
        assert_eq!(payload.occurred_at_unix_millis, 9_000);
        assert_eq!(
            payload.rejection,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeProviderRejected as i32
        );
    }

    #[test]
    fn event_identity_is_stable_for_intent_revision() {
        assert_eq!(event_id(&transition()), event_id(&transition()));
        let mut changed = transition();
        changed.state_revision += 1;
        assert_ne!(event_id(&transition()), event_id(&changed));
    }
}
