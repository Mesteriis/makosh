#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    CallEvidenceEnvelopeBuildErrorV1, CallEvidenceEnvelopeContextV1,
    build_call_evidence_observed_outbox_record_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-communications-call-evidence-ingress";
pub const CALL_EVIDENCE_CONTRACT_OWNER_V1: &str = "communications";
pub const CALL_EVIDENCE_OBSERVED_CONTRACT_NAME_V1: &str = "call_evidence_observed";
pub const CALL_EVIDENCE_CONTRACT_MAJOR_V1: u32 = 1;
pub const CALL_EVIDENCE_CONTRACT_REVISION_V1: u32 = 2;
pub const CALL_EVIDENCE_MAX_IN_FLIGHT_V1: u32 = 32;
pub const MAX_CALL_OBSERVATION_ID_BYTES_V1: usize = 256;
pub const MAX_CALL_SOURCE_ID_BYTES_V1: usize = 512;
pub const MAX_CALL_DISPLAY_LABEL_BYTES_V1: usize = 256;
pub const MAX_CALL_DURATION_SECONDS_V1: u64 = 31 * 24 * 60 * 60;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.call_evidence.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communications_call_evidence_ingress_schema.rs"
));

pub const COMMUNICATIONS_CALL_EVIDENCE_INGRESS_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communications-call-evidence-ingress-v1.bin"
));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallProviderProvenanceV1 {
    Telegram,
    WhatsAppWeb,
    Zoom,
    YandexTelemost,
}

impl CallProviderProvenanceV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::WhatsAppWeb => "whatsapp-web",
            Self::Zoom => "zoom",
            Self::YandexTelemost => "yandex-telemost",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallDirectionV1 {
    Incoming,
    Outgoing,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallMediaKindV1 {
    OneToOneAudio,
    Meeting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallLifecycleStateV1 {
    Observed,
    Ringing,
    Connecting,
    Active,
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTerminalDispositionV1 {
    Completed,
    Missed,
    Declined,
    Disconnected,
    Failed,
    Canceled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallEvidenceObservationDraftV1 {
    pub observation_id: String,
    pub logical_owner_id: String,
    pub provider: CallProviderProvenanceV1,
    pub external_account_id: String,
    pub external_call_id: String,
    pub external_conversation_id: Option<String>,
    pub external_participant_id: Option<String>,
    pub direction: CallDirectionV1,
    pub media_kind: CallMediaKindV1,
    pub state: CallLifecycleStateV1,
    pub terminal_disposition: Option<CallTerminalDispositionV1>,
    pub source_revision: u64,
    pub observed_at_unix_seconds: i64,
    pub started_at_unix_seconds: Option<i64>,
    pub connected_at_unix_seconds: Option<i64>,
    pub ended_at_unix_seconds: Option<i64>,
    pub duration_seconds: Option<u64>,
    pub participant_display_label: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidenceDraftErrorV1 {
    InvalidObservationId,
    InvalidOwner,
    InvalidSourceIdentity,
    InvalidRevision,
    InvalidState,
    InvalidTimestamp,
    InvalidDuration,
    InvalidDisplayLabel,
}

impl CallEvidenceObservationDraftV1 {
    pub fn validate(&self) -> Result<(), CallEvidenceDraftErrorV1> {
        validate_identifier(
            &self.observation_id,
            MAX_CALL_OBSERVATION_ID_BYTES_V1,
            CallEvidenceDraftErrorV1::InvalidObservationId,
        )?;
        validate_identifier(
            &self.logical_owner_id,
            MAX_CALL_SOURCE_ID_BYTES_V1,
            CallEvidenceDraftErrorV1::InvalidOwner,
        )?;
        for value in [&self.external_account_id, &self.external_call_id] {
            validate_identifier(
                value,
                MAX_CALL_SOURCE_ID_BYTES_V1,
                CallEvidenceDraftErrorV1::InvalidSourceIdentity,
            )?;
        }
        for value in [
            self.external_conversation_id.as_deref(),
            self.external_participant_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            validate_identifier(
                value,
                MAX_CALL_SOURCE_ID_BYTES_V1,
                CallEvidenceDraftErrorV1::InvalidSourceIdentity,
            )?;
        }
        if self.source_revision == 0 {
            return Err(CallEvidenceDraftErrorV1::InvalidRevision);
        }
        if self.state == CallLifecycleStateV1::Ended {
            if self.terminal_disposition.is_none() || self.ended_at_unix_seconds.is_none() {
                return Err(CallEvidenceDraftErrorV1::InvalidState);
            }
        } else if self.terminal_disposition.is_some() || self.ended_at_unix_seconds.is_some() {
            return Err(CallEvidenceDraftErrorV1::InvalidState);
        }
        validate_timestamp_order(self)?;
        if self
            .duration_seconds
            .is_some_and(|value| value > MAX_CALL_DURATION_SECONDS_V1)
        {
            return Err(CallEvidenceDraftErrorV1::InvalidDuration);
        }
        if let Some(label) = &self.participant_display_label {
            let normalized = label.trim();
            if normalized.is_empty()
                || normalized.len() > MAX_CALL_DISPLAY_LABEL_BYTES_V1
                || normalized.chars().any(char::is_control)
            {
                return Err(CallEvidenceDraftErrorV1::InvalidDisplayLabel);
            }
        }
        Ok(())
    }
}

#[must_use]
pub fn call_evidence_observed_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CALL_EVIDENCE_CONTRACT_OWNER_V1.to_owned(),
        name: CALL_EVIDENCE_OBSERVED_CONTRACT_NAME_V1.to_owned(),
        major: CALL_EVIDENCE_CONTRACT_MAJOR_V1,
        revision: CALL_EVIDENCE_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_CALL_EVIDENCE_INGRESS_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn call_evidence_observed_publish_request_v1() -> CapabilityRequestV1 {
    event_route(EventRouteDirectionV1::Publish)
}

#[must_use]
pub fn call_evidence_observed_consume_request_v1() -> CapabilityRequestV1 {
    event_route(EventRouteDirectionV1::Consume)
}

fn event_route(direction: EventRouteDirectionV1) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Observation as i32,
            contract: Some(call_evidence_observed_contract_reference_v1()),
            direction: direction as i32,
            max_in_flight: CALL_EVIDENCE_MAX_IN_FLIGHT_V1,
            subscription_requirement: if direction == EventRouteDirectionV1::Consume {
                EventSubscriptionRequirementV1::Required as i32
            } else {
                EventSubscriptionRequirementV1::Unspecified as i32
            },
            max_deliver: if direction == EventRouteDirectionV1::Consume {
                10
            } else {
                0
            },
            ack_wait_millis: if direction == EventRouteDirectionV1::Consume {
                30_000
            } else {
                0
            },
        })),
    }
}

fn validate_identifier(
    value: &str,
    max_bytes: usize,
    error: CallEvidenceDraftErrorV1,
) -> Result<(), CallEvidenceDraftErrorV1> {
    if value.trim().is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return Err(error);
    }
    Ok(())
}

fn validate_timestamp_order(
    draft: &CallEvidenceObservationDraftV1,
) -> Result<(), CallEvidenceDraftErrorV1> {
    let timestamps = [
        draft.started_at_unix_seconds,
        draft.connected_at_unix_seconds,
        draft.ended_at_unix_seconds,
    ];
    if timestamps
        .into_iter()
        .flatten()
        .any(|value| !(-62_135_596_800..=253_402_300_799).contains(&value))
        || !(-62_135_596_800..=253_402_300_799).contains(&draft.observed_at_unix_seconds)
    {
        return Err(CallEvidenceDraftErrorV1::InvalidTimestamp);
    }
    if let (Some(started), Some(connected)) = (
        draft.started_at_unix_seconds,
        draft.connected_at_unix_seconds,
    ) && connected < started
    {
        return Err(CallEvidenceDraftErrorV1::InvalidTimestamp);
    }
    if let (Some(connected), Some(ended)) =
        (draft.connected_at_unix_seconds, draft.ended_at_unix_seconds)
        && ended < connected
    {
        return Err(CallEvidenceDraftErrorV1::InvalidTimestamp);
    }
    if let (Some(started), Some(ended)) =
        (draft.started_at_unix_seconds, draft.ended_at_unix_seconds)
        && ended < started
    {
        return Err(CallEvidenceDraftErrorV1::InvalidTimestamp);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> CallEvidenceObservationDraftV1 {
        CallEvidenceObservationDraftV1 {
            observation_id: "telegram-call-1-revision-3".to_owned(),
            logical_owner_id: "owner-1".to_owned(),
            provider: CallProviderProvenanceV1::Telegram,
            external_account_id: "account-1".to_owned(),
            external_call_id: "call-1".to_owned(),
            external_conversation_id: Some("chat-1".to_owned()),
            external_participant_id: Some("user-2".to_owned()),
            direction: CallDirectionV1::Outgoing,
            media_kind: CallMediaKindV1::OneToOneAudio,
            state: CallLifecycleStateV1::Ended,
            terminal_disposition: Some(CallTerminalDispositionV1::Completed),
            source_revision: 3,
            observed_at_unix_seconds: 1_700_000_120,
            started_at_unix_seconds: Some(1_700_000_000),
            connected_at_unix_seconds: Some(1_700_000_010),
            ended_at_unix_seconds: Some(1_700_000_120),
            duration_seconds: Some(110),
            participant_display_label: Some("Example".to_owned()),
        }
    }

    #[test]
    fn exact_routes_are_directional_and_required_only_for_consumer() {
        let Some(Request::EventRoute(publish)) =
            call_evidence_observed_publish_request_v1().request
        else {
            panic!("publish route");
        };
        let Some(Request::EventRoute(consume)) =
            call_evidence_observed_consume_request_v1().request
        else {
            panic!("consume route");
        };
        assert_eq!(publish.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(consume.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            consume.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
        assert_eq!(publish.contract, consume.contract);
    }

    #[test]
    fn draft_requires_terminal_and_timestamp_consistency() {
        assert_eq!(draft().validate(), Ok(()));
        let mut invalid = draft();
        invalid.terminal_disposition = None;
        assert_eq!(
            invalid.validate(),
            Err(CallEvidenceDraftErrorV1::InvalidState)
        );
        let mut invalid = draft();
        invalid.connected_at_unix_seconds = Some(1_700_000_121);
        assert_eq!(
            invalid.validate(),
            Err(CallEvidenceDraftErrorV1::InvalidTimestamp)
        );
    }
}
