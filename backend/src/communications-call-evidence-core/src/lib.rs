#![forbid(unsafe_code)]

use makosh_communications_call_evidence_ingress::{
    MAX_CALL_DISPLAY_LABEL_BYTES_V1, MAX_CALL_DURATION_SECONDS_V1,
    wire::{
        CallDirectionV1 as WireDirection, CallEvidenceObservedV1,
        CallLifecycleStateV1 as WireState, CallMediaKindV1 as WireMediaKind,
        CallProviderProvenanceV1 as WireProvider,
        CallTerminalDispositionV1 as WireTerminalDisposition,
    },
};
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-communications-call-evidence-core";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallProviderProvenanceV1 {
    Telegram,
    WhatsAppWeb,
    Zoom,
    YandexTelemost,
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

impl CallLifecycleStateV1 {
    const fn rank(self) -> u8 {
        match self {
            Self::Observed => 1,
            Self::Ringing => 2,
            Self::Connecting => 3,
            Self::Active => 4,
            Self::Ended => 5,
        }
    }
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
pub struct RecordCallEvidenceV1 {
    pub call_evidence_id: [u8; 16],
    pub logical_owner_id: String,
    pub source_call_cursor_sha256: [u8; 32],
    pub account_cursor_sha256: [u8; 32],
    pub conversation_cursor_sha256: Option<[u8; 32]>,
    pub participant_cursor_sha256: Option<[u8; 32]>,
    pub provider: CallProviderProvenanceV1,
    pub direction: CallDirectionV1,
    pub media_kind: CallMediaKindV1,
    pub state: CallLifecycleStateV1,
    pub terminal_disposition: Option<CallTerminalDispositionV1>,
    pub source_revision: u64,
    pub started_at_unix_seconds: Option<i64>,
    pub connected_at_unix_seconds: Option<i64>,
    pub ended_at_unix_seconds: Option<i64>,
    pub duration_seconds: Option<u64>,
    pub participant_display_label: Option<String>,
    pub payload_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallEvidenceProjectionV1 {
    pub evidence: RecordCallEvidenceV1,
    pub canonical_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidenceApplyOutcomeV1 {
    Applied,
    Duplicate,
    Stale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidenceCoreErrorV1 {
    InvalidPayload,
    IdentityConflict,
    RevisionConflict,
    StateRegression,
    TerminalConflict,
}

pub fn decode_call_evidence_observation_v1(
    wire: &CallEvidenceObservedV1,
    exact_payload: &[u8],
) -> Result<RecordCallEvidenceV1, CallEvidenceCoreErrorV1> {
    let evidence = RecordCallEvidenceV1 {
        call_evidence_id: id16(&wire.call_evidence_id)?,
        logical_owner_id: wire.logical_owner_id.clone(),
        source_call_cursor_sha256: id32(&wire.source_call_cursor_sha256)?,
        account_cursor_sha256: id32(&wire.account_cursor_sha256)?,
        conversation_cursor_sha256: optional_id32(&wire.conversation_cursor_sha256)?,
        participant_cursor_sha256: optional_id32(&wire.participant_cursor_sha256)?,
        provider: provider(wire.provider)?,
        direction: direction(wire.direction)?,
        media_kind: media_kind(wire.media_kind)?,
        state: state(wire.state)?,
        terminal_disposition: terminal_disposition(wire.terminal_disposition)?,
        source_revision: wire.source_revision,
        started_at_unix_seconds: optional_timestamp(wire.started_at.as_ref())?,
        connected_at_unix_seconds: optional_timestamp(wire.connected_at.as_ref())?,
        ended_at_unix_seconds: optional_timestamp(wire.ended_at.as_ref())?,
        duration_seconds: wire.duration_seconds,
        participant_display_label: normalize_display_label(
            wire.participant_display_label.as_deref(),
        )?,
        payload_sha256: Sha256::digest(exact_payload).into(),
    };
    validate_evidence(&evidence)?;
    Ok(evidence)
}

pub fn apply_call_evidence_v1(
    current: Option<&CallEvidenceProjectionV1>,
    incoming: RecordCallEvidenceV1,
) -> Result<(CallEvidenceProjectionV1, CallEvidenceApplyOutcomeV1), CallEvidenceCoreErrorV1> {
    let Some(current) = current else {
        return Ok((
            CallEvidenceProjectionV1 {
                evidence: incoming,
                canonical_revision: 1,
            },
            CallEvidenceApplyOutcomeV1::Applied,
        ));
    };
    validate_stable_identity(&current.evidence, &incoming)?;
    if incoming.source_revision < current.evidence.source_revision {
        return Ok((current.clone(), CallEvidenceApplyOutcomeV1::Stale));
    }
    if incoming.source_revision == current.evidence.source_revision {
        if incoming.payload_sha256 == current.evidence.payload_sha256 {
            return Ok((current.clone(), CallEvidenceApplyOutcomeV1::Duplicate));
        }
        return Err(CallEvidenceCoreErrorV1::RevisionConflict);
    }
    if current.evidence.state == CallLifecycleStateV1::Ended {
        return Err(CallEvidenceCoreErrorV1::TerminalConflict);
    }
    if incoming.state.rank() < current.evidence.state.rank() {
        return Err(CallEvidenceCoreErrorV1::StateRegression);
    }
    Ok((
        CallEvidenceProjectionV1 {
            evidence: incoming,
            canonical_revision: current.canonical_revision + 1,
        },
        CallEvidenceApplyOutcomeV1::Applied,
    ))
}

fn validate_evidence(value: &RecordCallEvidenceV1) -> Result<(), CallEvidenceCoreErrorV1> {
    if value.source_revision == 0
        || !valid_owner(&value.logical_owner_id)
        || value.call_evidence_id.iter().all(|byte| *byte == 0)
        || value
            .source_call_cursor_sha256
            .iter()
            .all(|byte| *byte == 0)
        || value.account_cursor_sha256.iter().all(|byte| *byte == 0)
        || value
            .duration_seconds
            .is_some_and(|duration| duration > MAX_CALL_DURATION_SECONDS_V1)
    {
        return Err(CallEvidenceCoreErrorV1::InvalidPayload);
    }
    if value.state == CallLifecycleStateV1::Ended {
        if value.terminal_disposition.is_none() || value.ended_at_unix_seconds.is_none() {
            return Err(CallEvidenceCoreErrorV1::InvalidPayload);
        }
    } else if value.terminal_disposition.is_some() || value.ended_at_unix_seconds.is_some() {
        return Err(CallEvidenceCoreErrorV1::InvalidPayload);
    }
    if let (Some(started), Some(connected)) = (
        value.started_at_unix_seconds,
        value.connected_at_unix_seconds,
    ) && connected < started
    {
        return Err(CallEvidenceCoreErrorV1::InvalidPayload);
    }
    if let (Some(connected), Some(ended)) =
        (value.connected_at_unix_seconds, value.ended_at_unix_seconds)
        && ended < connected
    {
        return Err(CallEvidenceCoreErrorV1::InvalidPayload);
    }
    if let (Some(started), Some(ended)) =
        (value.started_at_unix_seconds, value.ended_at_unix_seconds)
        && ended < started
    {
        return Err(CallEvidenceCoreErrorV1::InvalidPayload);
    }
    Ok(())
}

fn validate_stable_identity(
    current: &RecordCallEvidenceV1,
    incoming: &RecordCallEvidenceV1,
) -> Result<(), CallEvidenceCoreErrorV1> {
    if current.call_evidence_id != incoming.call_evidence_id
        || current.logical_owner_id != incoming.logical_owner_id
        || current.source_call_cursor_sha256 != incoming.source_call_cursor_sha256
        || current.account_cursor_sha256 != incoming.account_cursor_sha256
        || current.provider != incoming.provider
        || current.direction != incoming.direction
        || current.media_kind != incoming.media_kind
    {
        return Err(CallEvidenceCoreErrorV1::IdentityConflict);
    }
    Ok(())
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CallEvidenceCoreErrorV1> {
    value
        .try_into()
        .map_err(|_| CallEvidenceCoreErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CallEvidenceCoreErrorV1> {
    value
        .try_into()
        .map_err(|_| CallEvidenceCoreErrorV1::InvalidPayload)
}

fn optional_id32(value: &[u8]) -> Result<Option<[u8; 32]>, CallEvidenceCoreErrorV1> {
    if value.is_empty() {
        Ok(None)
    } else {
        id32(value).map(Some)
    }
}

fn provider(value: i32) -> Result<CallProviderProvenanceV1, CallEvidenceCoreErrorV1> {
    match WireProvider::try_from(value) {
        Ok(WireProvider::Telegram) => Ok(CallProviderProvenanceV1::Telegram),
        Ok(WireProvider::WhatsappWeb) => Ok(CallProviderProvenanceV1::WhatsAppWeb),
        Ok(WireProvider::Zoom) => Ok(CallProviderProvenanceV1::Zoom),
        Ok(WireProvider::YandexTelemost) => Ok(CallProviderProvenanceV1::YandexTelemost),
        _ => Err(CallEvidenceCoreErrorV1::InvalidPayload),
    }
}

fn direction(value: i32) -> Result<CallDirectionV1, CallEvidenceCoreErrorV1> {
    match WireDirection::try_from(value) {
        Ok(WireDirection::Incoming) => Ok(CallDirectionV1::Incoming),
        Ok(WireDirection::Outgoing) => Ok(CallDirectionV1::Outgoing),
        Ok(WireDirection::Unknown) => Ok(CallDirectionV1::Unknown),
        _ => Err(CallEvidenceCoreErrorV1::InvalidPayload),
    }
}

fn media_kind(value: i32) -> Result<CallMediaKindV1, CallEvidenceCoreErrorV1> {
    match WireMediaKind::try_from(value) {
        Ok(WireMediaKind::OneToOneAudio) => Ok(CallMediaKindV1::OneToOneAudio),
        Ok(WireMediaKind::Meeting) => Ok(CallMediaKindV1::Meeting),
        _ => Err(CallEvidenceCoreErrorV1::InvalidPayload),
    }
}

fn state(value: i32) -> Result<CallLifecycleStateV1, CallEvidenceCoreErrorV1> {
    match WireState::try_from(value) {
        Ok(WireState::Observed) => Ok(CallLifecycleStateV1::Observed),
        Ok(WireState::Ringing) => Ok(CallLifecycleStateV1::Ringing),
        Ok(WireState::Connecting) => Ok(CallLifecycleStateV1::Connecting),
        Ok(WireState::Active) => Ok(CallLifecycleStateV1::Active),
        Ok(WireState::Ended) => Ok(CallLifecycleStateV1::Ended),
        _ => Err(CallEvidenceCoreErrorV1::InvalidPayload),
    }
}

fn terminal_disposition(
    value: i32,
) -> Result<Option<CallTerminalDispositionV1>, CallEvidenceCoreErrorV1> {
    match WireTerminalDisposition::try_from(value) {
        Ok(WireTerminalDisposition::Unspecified) => Ok(None),
        Ok(WireTerminalDisposition::Completed) => Ok(Some(CallTerminalDispositionV1::Completed)),
        Ok(WireTerminalDisposition::Missed) => Ok(Some(CallTerminalDispositionV1::Missed)),
        Ok(WireTerminalDisposition::Declined) => Ok(Some(CallTerminalDispositionV1::Declined)),
        Ok(WireTerminalDisposition::Disconnected) => {
            Ok(Some(CallTerminalDispositionV1::Disconnected))
        }
        Ok(WireTerminalDisposition::Failed) => Ok(Some(CallTerminalDispositionV1::Failed)),
        Ok(WireTerminalDisposition::Canceled) => Ok(Some(CallTerminalDispositionV1::Canceled)),
        Err(_) => Err(CallEvidenceCoreErrorV1::InvalidPayload),
    }
}

fn optional_timestamp(value: Option<&Timestamp>) -> Result<Option<i64>, CallEvidenceCoreErrorV1> {
    value.map(valid_timestamp).transpose()
}

fn valid_timestamp(value: &Timestamp) -> Result<i64, CallEvidenceCoreErrorV1> {
    if !(-62_135_596_800..=253_402_300_799).contains(&value.seconds)
        || !(0..1_000_000_000).contains(&value.nanos)
    {
        return Err(CallEvidenceCoreErrorV1::InvalidPayload);
    }
    Ok(value.seconds)
}

fn normalize_display_label(value: Option<&str>) -> Result<Option<String>, CallEvidenceCoreErrorV1> {
    value
        .map(str::trim)
        .map(|value| {
            if value.is_empty()
                || value.len() > MAX_CALL_DISPLAY_LABEL_BYTES_V1
                || value.chars().any(char::is_control)
            {
                Err(CallEvidenceCoreErrorV1::InvalidPayload)
            } else {
                Ok(value.to_owned())
            }
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communications_call_evidence_ingress::wire::CallEvidenceObservedV1;

    fn wire(state: WireState, revision: u64) -> CallEvidenceObservedV1 {
        CallEvidenceObservedV1 {
            call_evidence_id: [1; 16].to_vec(),
            logical_owner_id: "owner-1".to_owned(),
            source_call_cursor_sha256: [2; 32].to_vec(),
            account_cursor_sha256: [3; 32].to_vec(),
            conversation_cursor_sha256: [4; 32].to_vec(),
            participant_cursor_sha256: [5; 32].to_vec(),
            provider: WireProvider::Telegram as i32,
            direction: WireDirection::Incoming as i32,
            media_kind: WireMediaKind::OneToOneAudio as i32,
            state: state as i32,
            terminal_disposition: if state == WireState::Ended {
                WireTerminalDisposition::Completed as i32
            } else {
                WireTerminalDisposition::Unspecified as i32
            },
            source_revision: revision,
            started_at: Some(Timestamp {
                seconds: 1_700_000_000,
                nanos: 0,
            }),
            connected_at: (state == WireState::Active || state == WireState::Ended).then_some(
                Timestamp {
                    seconds: 1_700_000_010,
                    nanos: 0,
                },
            ),
            ended_at: (state == WireState::Ended).then_some(Timestamp {
                seconds: 1_700_000_120,
                nanos: 0,
            }),
            duration_seconds: (state == WireState::Ended).then_some(110),
            participant_display_label: Some("Example".to_owned()),
        }
    }

    fn decode(value: &CallEvidenceObservedV1, marker: u8) -> RecordCallEvidenceV1 {
        decode_call_evidence_observation_v1(value, &[marker]).expect("decode")
    }

    #[test]
    fn projection_is_monotonic_duplicate_safe_and_terminal() {
        let initial = decode(&wire(WireState::Ringing, 1), 1);
        let (projection, outcome) = apply_call_evidence_v1(None, initial).expect("initial");
        assert_eq!(outcome, CallEvidenceApplyOutcomeV1::Applied);

        let duplicate = decode(&wire(WireState::Ringing, 1), 1);
        let (projection, outcome) =
            apply_call_evidence_v1(Some(&projection), duplicate).expect("duplicate");
        assert_eq!(outcome, CallEvidenceApplyOutcomeV1::Duplicate);

        let active = decode(&wire(WireState::Active, 2), 2);
        let (projection, outcome) =
            apply_call_evidence_v1(Some(&projection), active).expect("active");
        assert_eq!(outcome, CallEvidenceApplyOutcomeV1::Applied);
        assert_eq!(projection.canonical_revision, 2);

        let terminal = decode(&wire(WireState::Ended, 3), 3);
        let (projection, _) =
            apply_call_evidence_v1(Some(&projection), terminal).expect("terminal");
        let replay = decode(&wire(WireState::Active, 4), 4);
        assert_eq!(
            apply_call_evidence_v1(Some(&projection), replay),
            Err(CallEvidenceCoreErrorV1::TerminalConflict)
        );
    }

    #[test]
    fn conflicting_same_revision_and_identity_change_fail_closed() {
        let initial = decode(&wire(WireState::Ringing, 1), 1);
        let (projection, _) = apply_call_evidence_v1(None, initial).expect("initial");
        let conflict = decode(&wire(WireState::Ringing, 1), 2);
        assert_eq!(
            apply_call_evidence_v1(Some(&projection), conflict),
            Err(CallEvidenceCoreErrorV1::RevisionConflict)
        );

        let mut changed = wire(WireState::Active, 2);
        changed.account_cursor_sha256 = [9; 32].to_vec();
        assert_eq!(
            apply_call_evidence_v1(Some(&projection), decode(&changed, 3)),
            Err(CallEvidenceCoreErrorV1::IdentityConflict)
        );
    }

    #[test]
    fn stale_observation_does_not_reopen_or_mutate_projection() {
        let active = decode(&wire(WireState::Active, 3), 3);
        let (projection, _) = apply_call_evidence_v1(None, active).expect("initial");
        let stale = decode(&wire(WireState::Ringing, 2), 2);
        let (unchanged, outcome) = apply_call_evidence_v1(Some(&projection), stale).expect("stale");
        assert_eq!(outcome, CallEvidenceApplyOutcomeV1::Stale);
        assert_eq!(unchanged, projection);
    }
}
