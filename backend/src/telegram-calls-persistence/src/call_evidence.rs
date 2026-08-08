//! Telegram-owned mapping from operational call state to public Communications evidence.

use makosh_communications_call_evidence_ingress::{
    CallDirectionV1, CallEvidenceEnvelopeBuildErrorV1, CallEvidenceEnvelopeContextV1,
    CallEvidenceObservationDraftV1, CallLifecycleStateV1, CallMediaKindV1,
    CallProviderProvenanceV1, CallTerminalDispositionV1,
    build_call_evidence_observed_outbox_record_v1,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_telegram_calls_core::{
    TelegramCallDirection, TelegramCallDiscardReason, TelegramCallSession,
    TelegramProviderCallState,
};

pub(crate) fn call_evidence_record_v1(
    session: &TelegramCallSession,
    logical_owner_id: &str,
    runtime_instance_id: &str,
) -> Result<OutboxRecordV1, CallEvidenceEnvelopeBuildErrorV1> {
    let state = lifecycle_state(session);
    let ended_at = session
        .ended_at_unix_seconds
        .map(i64::try_from)
        .transpose()
        .map_err(|_| CallEvidenceEnvelopeBuildErrorV1::InvalidDraft)?;
    let started_at = i64::try_from(session.created_at_unix_seconds)
        .map_err(|_| CallEvidenceEnvelopeBuildErrorV1::InvalidDraft)?;
    let observed_at = i64::try_from(session.updated_at_unix_seconds)
        .map_err(|_| CallEvidenceEnvelopeBuildErrorV1::InvalidDraft)?;
    let duration_seconds = session
        .ended_at_unix_seconds
        .map(|ended| ended.saturating_sub(session.created_at_unix_seconds));
    build_call_evidence_observed_outbox_record_v1(
        &CallEvidenceObservationDraftV1 {
            observation_id: format!(
                "telegram-call-evidence:{}:{}",
                session.call_session_id, session.revision
            ),
            logical_owner_id: logical_owner_id.to_owned(),
            provider: CallProviderProvenanceV1::Telegram,
            external_account_id: session.account_id.clone(),
            external_call_id: session.call_session_id.clone(),
            external_conversation_id: None,
            external_participant_id: Some(session.provider_user_id.clone()),
            direction: match session.direction {
                TelegramCallDirection::Incoming => CallDirectionV1::Incoming,
                TelegramCallDirection::Outgoing => CallDirectionV1::Outgoing,
            },
            media_kind: CallMediaKindV1::OneToOneAudio,
            state,
            terminal_disposition: terminal_disposition(session),
            source_revision: session.revision,
            observed_at_unix_seconds: observed_at,
            started_at_unix_seconds: Some(started_at),
            connected_at_unix_seconds: None,
            ended_at_unix_seconds: ended_at,
            duration_seconds,
            participant_display_label: None,
        },
        &CallEvidenceEnvelopeContextV1 {
            module_id: "telegram".to_owned(),
            runtime_instance_id: runtime_instance_id.to_owned(),
            runtime_generation: session.runtime_generation,
            recorded_at_unix_seconds: observed_at,
            recorded_at_nanos: 0,
        },
    )
}

const fn lifecycle_state(session: &TelegramCallSession) -> CallLifecycleStateV1 {
    match session.state {
        TelegramProviderCallState::Pending
            if matches!(session.direction, TelegramCallDirection::Incoming) =>
        {
            CallLifecycleStateV1::Ringing
        }
        TelegramProviderCallState::Pending | TelegramProviderCallState::ExchangingKeys => {
            CallLifecycleStateV1::Connecting
        }
        TelegramProviderCallState::MediaReady | TelegramProviderCallState::HangingUp => {
            CallLifecycleStateV1::Active
        }
        TelegramProviderCallState::Discarded | TelegramProviderCallState::Error => {
            CallLifecycleStateV1::Ended
        }
    }
}

const fn terminal_disposition(session: &TelegramCallSession) -> Option<CallTerminalDispositionV1> {
    match session.state {
        TelegramProviderCallState::Error => Some(CallTerminalDispositionV1::Failed),
        TelegramProviderCallState::Discarded => Some(match session.discard_reason {
            Some(TelegramCallDiscardReason::Missed) => CallTerminalDispositionV1::Missed,
            Some(TelegramCallDiscardReason::Declined) => CallTerminalDispositionV1::Declined,
            Some(TelegramCallDiscardReason::Disconnected) => {
                CallTerminalDispositionV1::Disconnected
            }
            Some(TelegramCallDiscardReason::HungUp) => CallTerminalDispositionV1::Completed,
            Some(TelegramCallDiscardReason::Empty) | None => CallTerminalDispositionV1::Canceled,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::{v1::DurableEnvelopeV1, validation::envelope::decode_envelope_v1};
    use prost::Message;

    use super::*;

    fn session(state: TelegramProviderCallState) -> TelegramCallSession {
        TelegramCallSession {
            call_session_id: "call-session-1".to_owned(),
            account_id: "account-1".to_owned(),
            runtime_generation: 3,
            tdlib_call_id: 91,
            provider_call_unique_id: Some(101),
            provider_user_id: "provider-user-1".to_owned(),
            direction: TelegramCallDirection::Incoming,
            state,
            pending_created: true,
            pending_received: true,
            discard_reason: None,
            failure_category: None,
            revision: 1,
            created_at_unix_seconds: 1_700_000_000,
            updated_at_unix_seconds: 1_700_000_001,
            ended_at_unix_seconds: None,
        }
    }

    #[test]
    fn mapper_hashes_provider_locators_before_serialization() {
        let record = call_evidence_record_v1(
            &session(TelegramProviderCallState::Pending),
            "owner-1",
            "tg-1",
        )
        .expect("record");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("valid durable observation");
        let _: DurableEnvelopeV1 =
            DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert!(
            !envelope
                .payload
                .windows(b"provider-user-1".len())
                .any(|bytes| bytes == b"provider-user-1")
        );
        assert!(
            !envelope
                .payload
                .windows(b"account-1".len())
                .any(|bytes| bytes == b"account-1")
        );
    }

    #[test]
    fn terminal_mapping_is_typed_and_closed() {
        let mut ended = session(TelegramProviderCallState::Discarded);
        ended.discard_reason = Some(TelegramCallDiscardReason::Missed);
        ended.ended_at_unix_seconds = Some(1_700_000_005);
        assert_eq!(
            terminal_disposition(&ended),
            Some(CallTerminalDispositionV1::Missed)
        );
        assert_eq!(lifecycle_state(&ended), CallLifecycleStateV1::Ended);
    }
}
