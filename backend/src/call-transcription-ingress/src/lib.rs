#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    CallTranscriptionIngressEnvelopeBuildErrorV1, CallTranscriptionIngressEnvelopeContextV1,
    build_recording_ready_outbox_record_v1, build_recording_rejected_outbox_record_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-call-transcription-ingress";
pub const OWNER_ID_V1: &str = "call_transcription";
pub const RECORDING_READY_CONTRACT_NAME_V1: &str = "call_transcription_recording_ready";
pub const RECORDING_REJECTED_CONTRACT_NAME_V1: &str = "call_transcription_recording_rejected";
pub const CONTRACT_MAJOR_V1: u32 = 1;
pub const CONTRACT_REVISION_V1: u32 = 1;
pub const MAX_IN_FLIGHT_V1: u32 = 32;
pub const TARGET_MODULE_ID_V1: &str = "makosh-call-transcription-runtime";
pub const TARGET_BLOB_CAPABILITY_ID_V1: &str = "call_transcription.recording_source.blob.v1";

#[must_use]
pub fn recording_ready_event_id_v1(recording_evidence_id: [u8; 16], revision: u64) -> [u8; 16] {
    derived_id_v1(b"recording-ready", recording_evidence_id, revision)
}

#[must_use]
pub fn recording_rejected_event_id_v1(recording_evidence_id: [u8; 16], revision: u64) -> [u8; 16] {
    derived_id_v1(b"recording-rejected", recording_evidence_id, revision)
}

fn derived_id_v1(label: &[u8], recording_evidence_id: [u8; 16], revision: u64) -> [u8; 16] {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(b"makosh.call-transcription-ingress.v1\0");
    hash.update(label);
    hash.update(recording_evidence_id);
    hash.update(revision.to_be_bytes());
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.call_transcription.ingress.v1.rs"
    ));
}
include!(concat!(
    env!("OUT_DIR"),
    "/call_transcription_ingress_schema.rs"
));
pub const DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/call-transcription-ingress-v1.bin"
));

#[must_use]
pub fn contract_reference_v1(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: CONTRACT_MAJOR_V1,
        revision: CONTRACT_REVISION_V1,
        schema_sha256: CALL_TRANSCRIPTION_INGRESS_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn recording_ready_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        RECORDING_READY_CONTRACT_NAME_V1,
        EventRouteDirectionV1::Publish,
    )
}

#[must_use]
pub fn recording_ready_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        RECORDING_READY_CONTRACT_NAME_V1,
        EventRouteDirectionV1::Consume,
    )
}

#[must_use]
pub fn recording_rejected_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        RECORDING_REJECTED_CONTRACT_NAME_V1,
        EventRouteDirectionV1::Publish,
    )
}

#[must_use]
pub fn recording_rejected_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        RECORDING_REJECTED_CONTRACT_NAME_V1,
        EventRouteDirectionV1::Consume,
    )
}

fn event_route(name: &str, direction: EventRouteDirectionV1) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(contract_reference_v1(name)),
            direction: direction as i32,
            max_in_flight: MAX_IN_FLIGHT_V1,
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

#[cfg(test)]
mod tests {
    #[test]
    fn event_is_target_owned_and_has_no_audio_or_path() {
        let source = include_str!("../proto/makosh/call_transcription/ingress/v1/recording.proto");
        for required in [
            "consent_receipt_id",
            "target_blob_reference_id",
            "custody_transfer_source_proof",
            "logical_owner_id",
        ] {
            assert!(source.contains(required));
        }
        for forbidden in ["audio_bytes", "filesystem_path", "provider_id", "device_id"] {
            assert!(!source.contains(forbidden));
        }
    }
}
