use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ObservationMetadataV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    CALL_EVIDENCE_CONTRACT_MAJOR_V1, CALL_EVIDENCE_CONTRACT_OWNER_V1,
    CALL_EVIDENCE_CONTRACT_REVISION_V1, CALL_EVIDENCE_OBSERVED_CONTRACT_NAME_V1,
    COMMUNICATIONS_CALL_EVIDENCE_INGRESS_SCHEMA_SHA256, CallDirectionV1,
    CallEvidenceObservationDraftV1, CallLifecycleStateV1, CallMediaKindV1,
    CallProviderProvenanceV1, CallTerminalDispositionV1, wire::CallEvidenceObservedV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallEvidenceEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidenceEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidDraft,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_call_evidence_observed_outbox_record_v1(
    draft: &CallEvidenceObservationDraftV1,
    context: &CallEvidenceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CallEvidenceEnvelopeBuildErrorV1> {
    draft
        .validate()
        .map_err(|_| CallEvidenceEnvelopeBuildErrorV1::InvalidDraft)?;
    validate_context(context)?;

    let call_evidence_id = call_evidence_id(draft);
    let source_call_cursor = source_cursor(
        b"makosh.communications.call-evidence.source-call.v1\0",
        draft,
        &draft.external_call_id,
    );
    let account_cursor = source_cursor(
        b"makosh.communications.call-evidence.account.v1\0",
        draft,
        &draft.external_account_id,
    );
    let conversation_cursor = draft.external_conversation_id.as_ref().map(|value| {
        scoped_cursor(
            b"makosh.communications.call-evidence.conversation.v1\0",
            account_cursor,
            value,
        )
    });
    let participant_cursor = draft.external_participant_id.as_ref().map(|value| {
        scoped_cursor(
            b"makosh.communications.call-evidence.participant.v1\0",
            account_cursor,
            value,
        )
    });
    let payload = CallEvidenceObservedV1 {
        call_evidence_id: call_evidence_id.to_vec(),
        source_call_cursor_sha256: source_call_cursor.to_vec(),
        account_cursor_sha256: account_cursor.to_vec(),
        conversation_cursor_sha256: conversation_cursor.map_or_else(Vec::new, |v| v.to_vec()),
        participant_cursor_sha256: participant_cursor.map_or_else(Vec::new, |v| v.to_vec()),
        provider: provider_value(draft.provider),
        direction: direction_value(draft.direction),
        media_kind: media_kind_value(draft.media_kind),
        state: state_value(draft.state),
        terminal_disposition: draft
            .terminal_disposition
            .map(terminal_disposition_value)
            .unwrap_or_default(),
        source_revision: draft.source_revision,
        started_at: draft.started_at_unix_seconds.map(timestamp),
        connected_at: draft.connected_at_unix_seconds.map(timestamp),
        ended_at: draft.ended_at_unix_seconds.map(timestamp),
        duration_seconds: draft.duration_seconds,
        participant_display_label: draft
            .participant_display_label
            .as_ref()
            .map(|value| value.trim().to_owned()),
        logical_owner_id: draft.logical_owner_id.clone(),
    }
    .encode_to_vec();
    let recorded_at = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let message_id = observation_message_id(draft);
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: CALL_EVIDENCE_CONTRACT_OWNER_V1.to_owned(),
            name: CALL_EVIDENCE_OBSERVED_CONTRACT_NAME_V1.to_owned(),
            major: CALL_EVIDENCE_CONTRACT_MAJOR_V1,
            revision: CALL_EVIDENCE_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATIONS_CALL_EVIDENCE_INGRESS_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(recorded_at),
        partition_key: call_evidence_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: call_evidence_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Observation(ObservationMetadataV1 {
            observation_id: message_id.to_vec(),
            observed_at: Some(recorded_at),
            occurred_at: Some(timestamp(draft.observed_at_unix_seconds)),
            source_cursor_sha256: source_call_cursor.to_vec(),
            source_sequence: Some(draft.source_revision),
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| CallEvidenceEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &CallEvidenceEnvelopeContextV1,
) -> Result<(), CallEvidenceEnvelopeBuildErrorV1> {
    if context.runtime_generation == 0
        || !valid_runtime_identity(&context.module_id)
        || !valid_runtime_identity(&context.runtime_instance_id)
        || !(-62_135_596_800..=253_402_300_799).contains(&context.recorded_at_unix_seconds)
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(CallEvidenceEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn valid_runtime_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn call_evidence_id(draft: &CallEvidenceObservationDraftV1) -> [u8; 16] {
    let digest = source_cursor(
        b"makosh.communications.call-evidence.id.v1\0",
        draft,
        &draft.external_call_id,
    );
    digest[..16].try_into().expect("fixed digest prefix")
}

fn observation_message_id(draft: &CallEvidenceObservationDraftV1) -> [u8; 16] {
    let digest = source_cursor(
        b"makosh.communications.call-evidence.observation.v1\0",
        draft,
        &draft.observation_id,
    );
    digest[..16].try_into().expect("fixed digest prefix")
}

fn source_cursor(domain: &[u8], draft: &CallEvidenceObservationDraftV1, value: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(draft.logical_owner_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(draft.provider.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(draft.external_account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(value.as_bytes());
    hasher.finalize().into()
}

fn scoped_cursor(domain: &[u8], account_cursor: [u8; 32], value: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(account_cursor);
    hasher.update(value.as_bytes());
    hasher.finalize().into()
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.runtime.source-reference.v1\0");
    hasher.update(runtime_instance_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16].try_into().expect("fixed digest prefix")
}

const fn provider_value(value: CallProviderProvenanceV1) -> i32 {
    match value {
        CallProviderProvenanceV1::Telegram => 1,
        CallProviderProvenanceV1::WhatsAppWeb => 2,
        CallProviderProvenanceV1::Zoom => 3,
        CallProviderProvenanceV1::YandexTelemost => 4,
    }
}

const fn direction_value(value: CallDirectionV1) -> i32 {
    match value {
        CallDirectionV1::Incoming => 1,
        CallDirectionV1::Outgoing => 2,
        CallDirectionV1::Unknown => 3,
    }
}

const fn media_kind_value(value: CallMediaKindV1) -> i32 {
    match value {
        CallMediaKindV1::OneToOneAudio => 1,
        CallMediaKindV1::Meeting => 2,
    }
}

const fn state_value(value: CallLifecycleStateV1) -> i32 {
    match value {
        CallLifecycleStateV1::Observed => 1,
        CallLifecycleStateV1::Ringing => 2,
        CallLifecycleStateV1::Connecting => 3,
        CallLifecycleStateV1::Active => 4,
        CallLifecycleStateV1::Ended => 5,
    }
}

const fn terminal_disposition_value(value: CallTerminalDispositionV1) -> i32 {
    match value {
        CallTerminalDispositionV1::Completed => 1,
        CallTerminalDispositionV1::Missed => 2,
        CallTerminalDispositionV1::Declined => 3,
        CallTerminalDispositionV1::Disconnected => 4,
        CallTerminalDispositionV1::Failed => 5,
        CallTerminalDispositionV1::Canceled => 6,
    }
}

const fn timestamp(seconds: i64) -> Timestamp {
    Timestamp { seconds, nanos: 0 }
}

fn outbox_error(_: OutboxRecordError) -> CallEvidenceEnvelopeBuildErrorV1 {
    CallEvidenceEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::validation::envelope::decode_envelope_v1;

    fn draft(observation_id: &str, revision: u64) -> CallEvidenceObservationDraftV1 {
        CallEvidenceObservationDraftV1 {
            observation_id: observation_id.to_owned(),
            logical_owner_id: "owner-1".to_owned(),
            provider: CallProviderProvenanceV1::Telegram,
            external_account_id: "provider-account-secret".to_owned(),
            external_call_id: "provider-call-secret".to_owned(),
            external_conversation_id: Some("provider-chat-secret".to_owned()),
            external_participant_id: Some("provider-user-secret".to_owned()),
            direction: CallDirectionV1::Incoming,
            media_kind: CallMediaKindV1::OneToOneAudio,
            state: CallLifecycleStateV1::Ended,
            terminal_disposition: Some(CallTerminalDispositionV1::Missed),
            source_revision: revision,
            observed_at_unix_seconds: 1_700_000_020,
            started_at_unix_seconds: Some(1_700_000_000),
            connected_at_unix_seconds: None,
            ended_at_unix_seconds: Some(1_700_000_020),
            duration_seconds: Some(0),
            participant_display_label: Some("Example".to_owned()),
        }
    }

    fn context() -> CallEvidenceEnvelopeContextV1 {
        CallEvidenceEnvelopeContextV1 {
            module_id: "makosh-telegram-runtime".to_owned(),
            runtime_instance_id: "telegram-runtime-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_700_000_021,
            recorded_at_nanos: 0,
        }
    }

    #[test]
    fn envelope_is_deterministic_partitioned_and_locator_negative() {
        let first = build_call_evidence_observed_outbox_record_v1(&draft("event-1", 3), &context())
            .expect("first");
        let duplicate =
            build_call_evidence_observed_outbox_record_v1(&draft("event-1", 3), &context())
                .expect("duplicate");
        assert_eq!(first.exact_bytes(), duplicate.exact_bytes());
        let envelope = decode_envelope_v1(first.exact_bytes()).expect("envelope");
        assert_eq!(envelope.partition_key.len(), 16);
        assert_eq!(
            envelope.contract.expect("contract").name,
            CALL_EVIDENCE_OBSERVED_CONTRACT_NAME_V1
        );
        let text = String::from_utf8_lossy(first.exact_bytes());
        for private in [
            "provider-account-secret",
            "provider-call-secret",
            "provider-chat-secret",
            "provider-user-secret",
        ] {
            assert!(!text.contains(private));
        }
    }

    #[test]
    fn observation_identity_changes_but_call_identity_stays_stable() {
        let first = build_call_evidence_observed_outbox_record_v1(&draft("event-1", 3), &context())
            .expect("first");
        let second =
            build_call_evidence_observed_outbox_record_v1(&draft("event-2", 4), &context())
                .expect("second");
        let first = decode_envelope_v1(first.exact_bytes()).expect("first envelope");
        let second = decode_envelope_v1(second.exact_bytes()).expect("second envelope");
        assert_ne!(first.message_id, second.message_id);
        assert_eq!(first.partition_key, second.partition_key);
    }
}
