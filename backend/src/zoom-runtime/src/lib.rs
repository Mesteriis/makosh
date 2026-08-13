#![forbid(unsafe_code)]
mod admission;
pub use admission::zoom_module_descriptor_v1;
use makosh_communications_call_evidence_ingress::{
    CallEvidenceEnvelopeBuildErrorV1, CallEvidenceEnvelopeContextV1,
    build_call_evidence_observed_outbox_record_v1,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_zoom_core::{ZoomCallObservationV1, ZoomCoreErrorV1, sanitize_call_observation_v1};
pub const PACKAGE: &str = "makosh-zoom-runtime";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoomRuntimeErrorV1 {
    InvalidObservation,
    InvalidContext,
}
pub fn build_sanitized_call_evidence_v1(
    value: &ZoomCallObservationV1,
    context: &CallEvidenceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ZoomRuntimeErrorV1> {
    let draft = sanitize_call_observation_v1(value)
        .map_err(|ZoomCoreErrorV1::InvalidObservation| ZoomRuntimeErrorV1::InvalidObservation)?;
    build_call_evidence_observed_outbox_record_v1(&draft, context)
        .map_err(|_: CallEvidenceEnvelopeBuildErrorV1| ZoomRuntimeErrorV1::InvalidContext)
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communications_call_evidence_ingress::CallEvidenceEnvelopeContextV1;
    #[test]
    fn builder_emits_sanitized_canonical_evidence() {
        let v = ZoomCallObservationV1 {
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
        let c = CallEvidenceEnvelopeContextV1 {
            module_id: "makosh-zoom-runtime".into(),
            runtime_instance_id: "runtime-1".into(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 20,
            recorded_at_nanos: 0,
        };
        assert!(build_sanitized_call_evidence_v1(&v, &c).is_ok());
    }
}
