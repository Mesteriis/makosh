//! Replayable client-safe SSE publication for canonical call evidence changes.

use std::os::unix::net::UnixStream;

use makosh_communications_call_evidence_api::{
    CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1, CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1,
    CALL_EVIDENCE_CLIENT_OWNER_V1, CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1,
    CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1, CALL_EVIDENCE_REALTIME_EVENT_KIND_V1,
    wire::CallEvidenceChangedV1,
};
use makosh_communications_call_evidence_persistence::{
    CallEvidencePersistenceErrorV1, CallEvidenceRealtimeRecordV1,
    CommunicationsCallEvidencePersistenceV1,
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

use crate::call_evidence_query_port::{state_value, terminal_disposition_value};

const REPLAY_WINDOW_V1: u32 = 256;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CallEvidenceClientRealtimePublisherV1 {
    last_sequence: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallEvidenceRealtimePumpOutcomeV1 {
    pub(crate) published: bool,
    pub(crate) drained: bool,
}

impl CallEvidenceClientRealtimePublisherV1 {
    pub(crate) async fn publish_pending(
        &mut self,
        persistence: &CommunicationsCallEvidencePersistenceV1,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        logical_owner_id: &str,
    ) -> Result<CallEvidenceRealtimePumpOutcomeV1, CallEvidenceClientRealtimeErrorV1> {
        let mut published = false;
        for _ in 0..4 {
            let records = persistence
                .replay(
                    logical_owner_id,
                    self.last_sequence.unwrap_or_default(),
                    REPLAY_WINDOW_V1,
                )
                .await
                .map_err(CallEvidenceClientRealtimeErrorV1::Persistence)?;
            let page_full = records.len() == REPLAY_WINDOW_V1 as usize;
            if records.is_empty() {
                return Ok(CallEvidenceRealtimePumpOutcomeV1 {
                    published,
                    drained: true,
                });
            }
            published = true;
            for record in records {
                let sequence = record.sequence;
                publish_record(channel, dispatcher, logical_owner_id, &record)?;
                self.last_sequence = Some(sequence);
            }
            if !page_full {
                return Ok(CallEvidenceRealtimePumpOutcomeV1 {
                    published,
                    drained: true,
                });
            }
        }
        Ok(CallEvidenceRealtimePumpOutcomeV1 {
            published,
            drained: false,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallEvidenceClientRealtimeErrorV1 {
    InvalidRecord,
    Persistence(CallEvidencePersistenceErrorV1),
    Unavailable,
}

fn publish_record(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    record: &CallEvidenceRealtimeRecordV1,
) -> Result<(), CallEvidenceClientRealtimeErrorV1> {
    let request = publication(logical_owner_id, record)?;
    validate_managed_client_realtime_publish_request_v1(&request)
        .map_err(|_| CallEvidenceClientRealtimeErrorV1::InvalidRecord)?;
    let expected_cursor = request.cursor.clone();
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::PublishClientRealtime(request)),
            },
            dispatcher,
        )
        .map_err(|_| CallEvidenceClientRealtimeErrorV1::Unavailable)?;
    if !response.error_code.is_empty() {
        return Err(CallEvidenceClientRealtimeErrorV1::Unavailable);
    }
    let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
        return Err(CallEvidenceClientRealtimeErrorV1::Unavailable);
    };
    validate_managed_client_realtime_publish_response_v1(&response)
        .map_err(|_| CallEvidenceClientRealtimeErrorV1::Unavailable)?;
    if response.accepted_cursor != expected_cursor {
        return Err(CallEvidenceClientRealtimeErrorV1::Unavailable);
    }
    Ok(())
}

fn publication(
    logical_owner_id: &str,
    record: &CallEvidenceRealtimeRecordV1,
) -> Result<ManagedRuntimeClientRealtimePublishRequestV1, CallEvidenceClientRealtimeErrorV1> {
    let occurred_at_unix_millis = u64::try_from(record.observed_at_unix_seconds)
        .ok()
        .and_then(|seconds| seconds.checked_mul(1_000))
        .ok_or(CallEvidenceClientRealtimeErrorV1::InvalidRecord)?;
    let payload = CallEvidenceChangedV1 {
        call_evidence_id: record.call_evidence_id.to_vec(),
        canonical_revision: record.canonical_revision,
        state: state_value(record.state),
        terminal_disposition: record
            .terminal_disposition
            .map_or(0, terminal_disposition_value),
        participant_display_label: record.participant_display_label.clone(),
    }
    .encode_to_vec();
    Ok(ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(realtime_contract()),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: event_id(record).to_vec(),
        cursor: format!("communications-call-evidence/{}", record.sequence),
        event_kind: CALL_EVIDENCE_REALTIME_EVENT_KIND_V1.to_owned(),
        occurred_at_unix_millis,
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload,
    })
}

fn realtime_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CALL_EVIDENCE_CLIENT_OWNER_V1.to_owned(),
        name: CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1.to_owned(),
        major: CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1,
        revision: CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

fn event_id(record: &CallEvidenceRealtimeRecordV1) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(record.call_evidence_id);
    hasher.update(record.canonical_revision.to_be_bytes());
    hasher.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has an exact length")
}

#[cfg(test)]
mod tests {
    use makosh_communications_call_evidence_core::{
        CallLifecycleStateV1, CallTerminalDispositionV1,
    };

    use super::*;

    fn record() -> CallEvidenceRealtimeRecordV1 {
        CallEvidenceRealtimeRecordV1 {
            sequence: 9,
            call_evidence_id: [7; 16],
            canonical_revision: 3,
            state: CallLifecycleStateV1::Ended,
            terminal_disposition: Some(CallTerminalDispositionV1::Completed),
            observed_at_unix_seconds: 42,
            participant_display_label: Some("Alice".to_owned()),
        }
    }

    #[test]
    fn publication_contains_only_client_safe_metadata() {
        let request = publication("owner-1", &record()).expect("publication");
        assert_eq!(request.cursor, "communications-call-evidence/9");
        let payload = CallEvidenceChangedV1::decode(request.payload.as_slice()).expect("payload");
        assert_eq!(payload.call_evidence_id, vec![7; 16]);
        assert_eq!(payload.canonical_revision, 3);
        assert_eq!(payload.state, 5);
        assert_eq!(payload.terminal_disposition, 1);
        assert_eq!(payload.participant_display_label.as_deref(), Some("Alice"));
    }

    #[test]
    fn event_identity_is_deterministic_and_revision_bound() {
        assert_eq!(event_id(&record()), event_id(&record()));
        let mut changed = record();
        changed.canonical_revision += 1;
        assert_ne!(event_id(&record()), event_id(&changed));
    }
}
