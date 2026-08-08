use std::os::unix::net::UnixStream;

use makosh_communications_export_api::{
    COMMUNICATIONS_EXPORT_REALTIME_EVENT_KIND_V1,
    wire::{
        CommunicationsExportErrorCodeV1, EvidenceExportStatusChangedV1, EvidenceExportStatusV1,
    },
};
use makosh_communications_export_persistence::{
    CommunicationsExportPersistenceErrorV1, CommunicationsExportPersistenceV1,
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

use crate::admission::communications_export_realtime_contract_reference_v1;

const REPLAY_WINDOW_V1: u16 = 1_024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CommunicationsExportClientRealtimePublisherV1 {
    last_sequence: Option<u64>,
}

impl CommunicationsExportClientRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &CommunicationsExportPersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<bool, CommunicationsExportClientRealtimeErrorV1> {
        let transitions = persistence
            .client_realtime_window(logical_owner_id, self.last_sequence, REPLAY_WINDOW_V1)
            .await
            .map_err(|error| {
                diagnostic("persistence", Some(&format!("{error:?}")));
                CommunicationsExportClientRealtimeErrorV1::Persistence(error)
            })?;
        let published = !transitions.is_empty();
        for transition in transitions {
            let occurred_at_unix_millis = u64::try_from(transition.occurred_at_unix_millis)
                .map_err(|_| CommunicationsExportClientRealtimeErrorV1::InvalidTransition)?;
            let payload = EvidenceExportStatusChangedV1 {
                export_id: transition.export_id.to_vec(),
                status: status_wire(transition.state)? as i32,
                requested_items: transition.requested_items,
                completed_items: transition.completed_items,
                artifact_bytes: transition.artifact_bytes,
                occurred_at_unix_millis,
                error: if transition.rejection_code.is_some() {
                    CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodePolicyRejected
                        as i32
                } else {
                    CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeUnspecified as i32
                },
            }
            .encode_to_vec();
            let request = ManagedRuntimeClientRealtimePublishRequestV1 {
                contract: Some(communications_export_realtime_contract_reference_v1()),
                logical_owner_id: logical_owner_id.to_owned(),
                event_id: event_id(transition.export_id, transition.sequence).to_vec(),
                cursor: format!("communications-export/{}", transition.sequence),
                event_kind: COMMUNICATIONS_EXPORT_REALTIME_EVENT_KIND_V1.to_owned(),
                occurred_at_unix_millis,
                causation_id: String::new(),
                correlation_id: String::new(),
                trace_id: String::new(),
                payload,
            };
            validate_managed_client_realtime_publish_request_v1(&request).map_err(|_| {
                diagnostic("request-validation", None);
                CommunicationsExportClientRealtimeErrorV1::InvalidTransition
            })?;
            let cursor = request.cursor.clone();
            let response = channel
                .request_next_with_dispatch(
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::PublishClientRealtime(request)),
                    },
                    dispatcher,
                )
                .map_err(|_| {
                    diagnostic("control-transport", None);
                    CommunicationsExportClientRealtimeErrorV1::Unavailable
                })?;
            if !response.error_code.is_empty() {
                diagnostic("control-rejection", Some(&response.error_code));
                return Err(CommunicationsExportClientRealtimeErrorV1::Unavailable);
            }
            let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
                diagnostic("control-response", None);
                return Err(CommunicationsExportClientRealtimeErrorV1::Unavailable);
            };
            if validate_managed_client_realtime_publish_response_v1(&response).is_err()
                || response.accepted_cursor != cursor
            {
                diagnostic("response-validation", None);
                return Err(CommunicationsExportClientRealtimeErrorV1::Unavailable);
            }
            self.last_sequence = Some(transition.sequence);
        }
        Ok(published)
    }
}

fn diagnostic(stage: &str, detail: Option<&str>) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_none() {
        return;
    }
    if let Some(detail) = detail {
        eprintln!("developer_communications_export_client_realtime stage={stage} detail={detail}");
    } else {
        eprintln!("developer_communications_export_client_realtime stage={stage}");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommunicationsExportClientRealtimeErrorV1 {
    InvalidTransition,
    Persistence(CommunicationsExportPersistenceErrorV1),
    Unavailable,
}

fn status_wire(
    state: u8,
) -> Result<EvidenceExportStatusV1, CommunicationsExportClientRealtimeErrorV1> {
    match state {
        1 => Ok(EvidenceExportStatusV1::EvidenceExportStatusPendingSource),
        2 => Ok(EvidenceExportStatusV1::EvidenceExportStatusMaterializing),
        3 => Ok(EvidenceExportStatusV1::EvidenceExportStatusReady),
        4 => Ok(EvidenceExportStatusV1::EvidenceExportStatusRejected),
        _ => Err(CommunicationsExportClientRealtimeErrorV1::InvalidTransition),
    }
}

fn event_id(export_id: [u8; 16], sequence: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"communications-export-client-realtime-v1");
    digest.update(export_id);
    digest.update(sequence.to_be_bytes());
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use super::event_id;

    #[test]
    fn event_identity_is_stable_and_sequence_specific() {
        assert_eq!(event_id([7; 16], 3), event_id([7; 16], 3));
        assert_ne!(event_id([7; 16], 3), event_id([7; 16], 4));
    }
}
