use std::os::unix::net::UnixStream;

use makosh_communication_bulk_action_api::{
    COMMUNICATION_BULK_ACTION_REALTIME_EVENT_KIND_V1,
    wire::{BulkDeliveryBatchStateV1 as WireBatchState, BulkDeliveryStatusChangedV1},
};
use makosh_communication_bulk_action_persistence::{
    BulkDeliveryBatchStateV1, BulkDeliveryPersistenceErrorV1, CommunicationBulkActionPersistenceV1,
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

use crate::contracts::bulk_realtime_contract_v1;

const REPLAY_WINDOW_V1: u16 = 1_024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BulkDeliveryClientRealtimePublisherV1 {
    last_sequence: Option<u64>,
}

impl BulkDeliveryClientRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &CommunicationBulkActionPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, BulkDeliveryClientRealtimeErrorV1> {
        let transitions = persistence
            .client_realtime_window(logical_owner_id, self.last_sequence, REPLAY_WINDOW_V1)
            .await
            .map_err(BulkDeliveryClientRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let occurred_at_unix_millis = u64::try_from(transition.occurred_at_unix_seconds)
                .ok()
                .and_then(|seconds| seconds.checked_mul(1_000))
                .ok_or(BulkDeliveryClientRealtimeErrorV1::InvalidTransition)?;
            let payload = BulkDeliveryStatusChangedV1 {
                batch_id: transition.batch_id.to_vec(),
                state: wire_state(transition.state) as i32,
                state_revision: transition.state_revision,
                occurred_at_unix_millis,
            }
            .encode_to_vec();
            let request = ManagedRuntimeClientRealtimePublishRequestV1 {
                contract: Some(bulk_realtime_contract_v1()),
                logical_owner_id: logical_owner_id.to_owned(),
                event_id: event_id(transition.batch_id, transition.state_revision).to_vec(),
                cursor: format!("communication-bulk-action/{}", transition.sequence),
                event_kind: COMMUNICATION_BULK_ACTION_REALTIME_EVENT_KIND_V1.to_owned(),
                occurred_at_unix_millis,
                causation_id: String::new(),
                correlation_id: String::new(),
                trace_id: String::new(),
                payload,
            };
            validate_managed_client_realtime_publish_request_v1(&request)
                .map_err(|_| BulkDeliveryClientRealtimeErrorV1::InvalidTransition)?;
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| BulkDeliveryClientRealtimeErrorV1::Unavailable)?;
            if !response.error_code.is_empty() {
                return Err(BulkDeliveryClientRealtimeErrorV1::Unavailable);
            }
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                return Err(BulkDeliveryClientRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                return Err(BulkDeliveryClientRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = Some(transition.sequence);
        }
        Ok(published)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BulkDeliveryClientRealtimeErrorV1 {
    InvalidTransition,
    Persistence(BulkDeliveryPersistenceErrorV1),
    Unavailable,
}

const fn wire_state(state: BulkDeliveryBatchStateV1) -> WireBatchState {
    match state {
        BulkDeliveryBatchStateV1::Accepted => WireBatchState::BulkDeliveryBatchStateAccepted,
        BulkDeliveryBatchStateV1::Completed => WireBatchState::BulkDeliveryBatchStateCompleted,
        BulkDeliveryBatchStateV1::CompletedWithErrors => {
            WireBatchState::BulkDeliveryBatchStateCompletedWithErrors
        }
        BulkDeliveryBatchStateV1::Rejected => WireBatchState::BulkDeliveryBatchStateRejected,
    }
}

fn event_id(batch_id: [u8; 16], state_revision: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(batch_id);
    digest.update(state_revision.to_be_bytes());
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}
