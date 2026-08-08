use std::os::unix::net::UnixStream;

use makosh_mail_contacts_sync_api::{
    MAIL_CONTACTS_SYNC_REALTIME_EVENT_KIND_V1, mail_contacts_sync_realtime_contract_v1,
    wire::{
        MailContactsSyncErrorCodeV1 as WireError, MailContactsSyncStateV1 as WireState,
        MailContactsSyncStatusChangedV1,
    },
};
use makosh_mail_contacts_sync_core::{MailContactsSyncRejectCodeV1, MailContactsSyncStateV1};
use makosh_mail_contacts_sync_persistence::{
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    MailContactsSyncRealtimeTransitionV1,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MailContactsSyncRealtimePublisherV1 {
    last_sequence: Option<u64>,
}

impl MailContactsSyncRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &MailContactsSyncPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, MailContactsSyncRealtimeErrorV1> {
        let transitions = persistence
            .client_realtime_window(logical_owner_id, self.last_sequence, 256)
            .await
            .map_err(MailContactsSyncRealtimeErrorV1::Persistence)?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let request = realtime_request(logical_owner_id, &transition);
            if validate_managed_client_realtime_publish_request_v1(&request).is_err() {
                return Err(MailContactsSyncRealtimeErrorV1::InvalidTransition);
            }
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| MailContactsSyncRealtimeErrorV1::Unavailable)?;
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                return Err(MailContactsSyncRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                return Err(MailContactsSyncRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = Some(transition.sequence);
        }
        Ok(published)
    }
}

fn realtime_request(
    logical_owner_id: &str,
    transition: &MailContactsSyncRealtimeTransitionV1,
) -> ManagedRuntimeClientRealtimePublishRequestV1 {
    ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(mail_contacts_sync_realtime_contract_v1()),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: event_id(transition.run_id, transition.state_revision).to_vec(),
        cursor: format!("mail-contacts-sync/{}", transition.sequence),
        event_kind: MAIL_CONTACTS_SYNC_REALTIME_EVENT_KIND_V1.to_owned(),
        occurred_at_unix_millis: u64::try_from(transition.occurred_at_unix_millis)
            .unwrap_or_default(),
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload: MailContactsSyncStatusChangedV1 {
            run_id: transition.run_id.to_vec(),
            state: wire_state(transition.state) as i32,
            state_revision: transition.state_revision,
            occurred_at_unix_millis: u64::try_from(transition.occurred_at_unix_millis)
                .unwrap_or_default(),
            error: transition.rejection.map_or(
                WireError::MailContactsSyncErrorCodeUnspecified,
                wire_rejection,
            ) as i32,
        }
        .encode_to_vec(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MailContactsSyncRealtimeErrorV1 {
    InvalidTransition,
    Persistence(MailContactsSyncPersistenceErrorV1),
    Unavailable,
}

const fn wire_state(value: MailContactsSyncStateV1) -> WireState {
    match value {
        MailContactsSyncStateV1::Accepted => WireState::MailContactsSyncStateAccepted,
        MailContactsSyncStateV1::FetchingProviderPage => {
            WireState::MailContactsSyncStateFetchingProviderPage
        }
        MailContactsSyncStateV1::ApplyingContacts => {
            WireState::MailContactsSyncStateApplyingContacts
        }
        MailContactsSyncStateV1::WritingProvider => WireState::MailContactsSyncStateWritingProvider,
        MailContactsSyncStateV1::ReconcilingOutcome => {
            WireState::MailContactsSyncStateReconcilingOutcome
        }
        MailContactsSyncStateV1::Completed => WireState::MailContactsSyncStateCompleted,
        MailContactsSyncStateV1::Rejected => WireState::MailContactsSyncStateRejected,
    }
}

const fn wire_rejection(value: MailContactsSyncRejectCodeV1) -> WireError {
    match value {
        MailContactsSyncRejectCodeV1::InvalidRequest => {
            WireError::MailContactsSyncErrorCodeInvalidRequest
        }
        MailContactsSyncRejectCodeV1::AccountUnavailable => {
            WireError::MailContactsSyncErrorCodeAccountUnavailable
        }
        MailContactsSyncRejectCodeV1::ProviderUnavailable => {
            WireError::MailContactsSyncErrorCodeProviderUnavailable
        }
        MailContactsSyncRejectCodeV1::ContactsRejected => {
            WireError::MailContactsSyncErrorCodeContactsRejected
        }
        MailContactsSyncRejectCodeV1::RemoteWriteBlocked => {
            WireError::MailContactsSyncErrorCodeRemoteWriteBlocked
        }
        MailContactsSyncRejectCodeV1::EtagConflict => {
            WireError::MailContactsSyncErrorCodeEtagConflict
        }
        MailContactsSyncRejectCodeV1::OutcomeUnknown => {
            WireError::MailContactsSyncErrorCodeOutcomeUnknown
        }
        MailContactsSyncRejectCodeV1::Policy => WireError::MailContactsSyncErrorCodePolicy,
    }
}

fn event_id(run_id: [u8; 16], state_revision: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"mail-contacts-sync/client-realtime/v1");
    digest.update(run_id);
    digest.update(state_revision.to_be_bytes());
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::client_realtime::validate_managed_client_realtime_publish_request_v1;

    use super::*;

    #[test]
    fn persisted_transition_maps_to_valid_shared_sse_frame() {
        let request = realtime_request(
            "owner-1",
            &MailContactsSyncRealtimeTransitionV1 {
                sequence: 1,
                run_id: [1; 16],
                state: MailContactsSyncStateV1::FetchingProviderPage,
                state_revision: 2,
                rejection: None,
                occurred_at_unix_millis: 1_800_000_000_000,
            },
        );
        assert_eq!(
            validate_managed_client_realtime_publish_request_v1(&request),
            Ok(())
        );
        assert_eq!(request.cursor, "mail-contacts-sync/1");
    }
}
