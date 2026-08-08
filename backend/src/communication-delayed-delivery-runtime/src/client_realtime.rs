use std::os::unix::net::UnixStream;

use makosh_communication_delayed_delivery_api::{
    COMMUNICATION_DELAYED_DELIVERY_REALTIME_EVENT_KIND_V1,
    wire::{DelayedDeliveryStateV1 as WireState, DelayedDeliveryStatusChangedV1},
};
use makosh_communication_delayed_delivery_core::DelayedDeliveryStateV1;
use makosh_communication_delayed_delivery_persistence::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryClientRealtimeTransitionV1,
    DelayedDeliveryPersistenceErrorV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ManagedRuntimeClientRealtimePublishRequestV1, ManagedRuntimeControlRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::client_realtime::{
        validate_managed_client_realtime_publish_request_v1,
        validate_managed_client_realtime_publish_response_v1,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::delayed_delivery_realtime_contract_v1;

const REPLAY_WINDOW_V1: u16 = 256;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DelayedDeliveryClientRealtimePublisherV1 {
    last_sequence: Option<u64>,
}

impl DelayedDeliveryClientRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &CommunicationDelayedDeliveryPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, DelayedDeliveryClientRealtimeErrorV1> {
        let transitions = persistence
            .client_realtime_window(logical_owner_id, self.last_sequence, REPLAY_WINDOW_V1)
            .await
            .map_err(DelayedDeliveryClientRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();

        for transition in transitions {
            let request = client_realtime_request(logical_owner_id, &transition);
            if validate_managed_client_realtime_publish_request_v1(&request).is_err() {
                eprintln!(
                    "developer_delayed_delivery_realtime_rejected=invalid_transition \
                     contract_schema_bytes={} logical_owner_empty={} event_id_bytes={} \
                     cursor_bytes={} event_kind_bytes={} occurred_at_zero={} payload_bytes={}",
                    request
                        .contract
                        .as_ref()
                        .map_or(0, |contract| contract.schema_sha256.len()),
                    request.logical_owner_id.is_empty(),
                    request.event_id.len(),
                    request.cursor.len(),
                    request.event_kind.len(),
                    request.occurred_at_unix_millis == 0,
                    request.payload.len(),
                );
                return Err(DelayedDeliveryClientRealtimeErrorV1::InvalidTransition);
            }
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| DelayedDeliveryClientRealtimeErrorV1::Unavailable)?;
            if !response.error_code.is_empty() {
                return Err(DelayedDeliveryClientRealtimeErrorV1::Unavailable);
            }
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                return Err(DelayedDeliveryClientRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                return Err(DelayedDeliveryClientRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = Some(transition.sequence);
        }

        Ok(published)
    }
}

fn client_realtime_request(
    logical_owner_id: &str,
    transition: &DelayedDeliveryClientRealtimeTransitionV1,
) -> ManagedRuntimeClientRealtimePublishRequestV1 {
    let payload = DelayedDeliveryStatusChangedV1 {
        delayed_operation_id: transition.delayed_operation_id.to_vec(),
        state: wire_state(transition.state) as i32,
        state_revision: transition.state_revision,
        occurred_at_unix_millis: transition.occurred_at_unix_millis,
    }
    .encode_to_vec();
    ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(delayed_delivery_realtime_contract_v1()),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: event_id(transition.delayed_operation_id, transition.state_revision).to_vec(),
        cursor: format!("communication-delayed-delivery/{}", transition.sequence),
        event_kind: COMMUNICATION_DELAYED_DELIVERY_REALTIME_EVENT_KIND_V1.to_owned(),
        occurred_at_unix_millis: transition.occurred_at_unix_millis,
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DelayedDeliveryClientRealtimeErrorV1 {
    InvalidTransition,
    Persistence(DelayedDeliveryPersistenceErrorV1),
    Unavailable,
}

const fn wire_state(state: DelayedDeliveryStateV1) -> WireState {
    match state {
        DelayedDeliveryStateV1::Accepted => WireState::DelayedDeliveryStateAccepted,
        DelayedDeliveryStateV1::SchedulePending => WireState::DelayedDeliveryStateSchedulePending,
        DelayedDeliveryStateV1::Scheduled => WireState::DelayedDeliveryStateScheduled,
        DelayedDeliveryStateV1::Due => WireState::DelayedDeliveryStateDue,
        DelayedDeliveryStateV1::Dispatching => WireState::DelayedDeliveryStateDispatching,
        DelayedDeliveryStateV1::DeliveryAccepted => WireState::DelayedDeliveryStateDeliveryAccepted,
        DelayedDeliveryStateV1::CancelRequested => WireState::DelayedDeliveryStateCancelRequested,
        DelayedDeliveryStateV1::Cancelled => WireState::DelayedDeliveryStateCancelled,
        DelayedDeliveryStateV1::Failed => WireState::DelayedDeliveryStateFailed,
    }
}

fn event_id(delayed_operation_id: [u8; 16], state_revision: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"communication-delayed-delivery/client-realtime/v1");
    digest.update(delayed_operation_id);
    digest.update(state_revision.to_be_bytes());
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use super::{client_realtime_request, event_id, wire_state};
    use makosh_communication_delayed_delivery_api::wire::DelayedDeliveryStateV1 as WireState;
    use makosh_communication_delayed_delivery_core::DelayedDeliveryStateV1;
    use makosh_communication_delayed_delivery_persistence::DelayedDeliveryClientRealtimeTransitionV1;
    use makosh_runtime_protocol::validation::client_realtime::validate_managed_client_realtime_publish_request_v1;

    #[test]
    fn event_identity_binds_operation_and_revision() {
        let operation_id = [7_u8; 16];

        assert_eq!(event_id(operation_id, 3), event_id(operation_id, 3));
        assert_ne!(event_id(operation_id, 3), event_id(operation_id, 4));
        assert_ne!(event_id(operation_id, 3), event_id([8_u8; 16], 3));
    }

    #[test]
    fn terminal_state_is_exposed_on_client_realtime_contract() {
        assert_eq!(
            wire_state(DelayedDeliveryStateV1::DeliveryAccepted),
            WireState::DelayedDeliveryStateDeliveryAccepted
        );
    }

    #[test]
    fn persisted_transition_maps_to_a_valid_managed_realtime_request() {
        let request = client_realtime_request(
            "development-owner",
            &DelayedDeliveryClientRealtimeTransitionV1 {
                sequence: 1,
                delayed_operation_id: [0x8d; 16],
                state: DelayedDeliveryStateV1::SchedulePending,
                state_revision: 1,
                occurred_at_unix_millis: 1_785_388_099_075,
            },
        );

        assert_eq!(
            validate_managed_client_realtime_publish_request_v1(&request),
            Ok(())
        );
    }
}
