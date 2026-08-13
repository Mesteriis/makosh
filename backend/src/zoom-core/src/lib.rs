#![forbid(unsafe_code)]
use makosh_communications_call_evidence_ingress::{
    CallDirectionV1, CallEvidenceObservationDraftV1, CallLifecycleStateV1, CallMediaKindV1,
    CallProviderProvenanceV1, CallTerminalDispositionV1,
};
pub const PACKAGE: &str = "makosh-zoom-core";
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZoomCallObservationV1 {
    pub observation_id: String,
    pub logical_owner_id: String,
    pub external_account_id: String,
    pub external_call_id: String,
    pub source_revision: u64,
    pub observed_at_unix_seconds: i64,
    pub started_at_unix_seconds: Option<i64>,
    pub ended_at_unix_seconds: Option<i64>,
    pub participant_display_label: Option<String>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoomCoreErrorV1 {
    InvalidObservation,
}
pub fn sanitize_call_observation_v1(
    value: &ZoomCallObservationV1,
) -> Result<CallEvidenceObservationDraftV1, ZoomCoreErrorV1> {
    let ended = value.ended_at_unix_seconds.is_some();
    let draft = CallEvidenceObservationDraftV1 {
        observation_id: value.observation_id.clone(),
        logical_owner_id: value.logical_owner_id.clone(),
        provider: CallProviderProvenanceV1::Zoom,
        external_account_id: value.external_account_id.clone(),
        external_call_id: value.external_call_id.clone(),
        external_conversation_id: None,
        external_participant_id: None,
        direction: CallDirectionV1::Unknown,
        media_kind: CallMediaKindV1::Meeting,
        state: if ended {
            CallLifecycleStateV1::Ended
        } else {
            CallLifecycleStateV1::Observed
        },
        terminal_disposition: ended.then_some(CallTerminalDispositionV1::Completed),
        source_revision: value.source_revision,
        observed_at_unix_seconds: value.observed_at_unix_seconds,
        started_at_unix_seconds: value.started_at_unix_seconds,
        connected_at_unix_seconds: None,
        ended_at_unix_seconds: value.ended_at_unix_seconds,
        duration_seconds: match (value.started_at_unix_seconds, value.ended_at_unix_seconds) {
            (Some(a), Some(b)) => u64::try_from(b - a).ok(),
            _ => None,
        },
        participant_display_label: value.participant_display_label.clone(),
    };
    draft
        .validate()
        .map_err(|_| ZoomCoreErrorV1::InvalidObservation)?;
    Ok(draft)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sanitizes_to_exact_provider_without_private_locators() {
        let value = ZoomCallObservationV1 {
            observation_id: "obs-1".into(),
            logical_owner_id: "owner-1".into(),
            external_account_id: "account-1".into(),
            external_call_id: "call-1".into(),
            source_revision: 1,
            observed_at_unix_seconds: 20,
            started_at_unix_seconds: Some(10),
            ended_at_unix_seconds: Some(20),
            participant_display_label: None,
        };
        let draft = sanitize_call_observation_v1(&value).unwrap();
        assert_eq!(draft.provider, CallProviderProvenanceV1::Zoom);
        assert!(draft.external_conversation_id.is_none());
        assert!(draft.external_participant_id.is_none());
    }
}
