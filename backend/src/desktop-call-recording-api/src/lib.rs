#![forbid(unsafe_code)]

use makosh_runtime_protocol::v1::ContractReferenceV1;

pub mod host_bridge;

pub const PACKAGE: &str = "makosh-desktop-call-recording-api";
pub const OWNER_ID_V1: &str = "desktop_call_recording";
pub const MODULE_ID_V1: &str = "makosh-desktop-call-recording-runtime";
pub const START_CONTRACT_NAME_V1: &str = "desktop_call_recording.start";
pub const STOP_CONTRACT_NAME_V1: &str = "desktop_call_recording.stop";
pub const GET_CONTRACT_NAME_V1: &str = "desktop_call_recording.get";
pub const HOST_CONTRACT_NAME_V1: &str = "desktop_call_recording.host_bridge";
pub const REALTIME_CONTRACT_NAME_V1: &str = "desktop_call_recording.status_changed";
pub const CONTRACT_MAJOR_V1: u32 = 1;
pub const CONTRACT_REVISION_V1: u32 = 1;
pub const HOST_PROTOCOL_MAJOR_V1: u32 = 1;
pub const HOST_PROTOCOL_REVISION_V1: u32 = 1;
pub const MAX_AUDIO_BYTES_V1: usize = 64 * 1024 * 1024;
pub const MAX_DURATION_MILLIS_V1: u64 = 4 * 60 * 60 * 1_000;
pub const CANONICAL_AUDIO_FORMAT_V1: &str = "wav_pcm_s16le_mono_16000";
pub const CONSENT_PURPOSE_V1: &str = "call_transcription";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.desktop_call_recording.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/desktop_call_recording_schema.rs"
));
pub const DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/desktop-call-recording-v1.bin"));

#[must_use]
pub fn contract_reference_v1(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: CONTRACT_MAJOR_V1,
        revision: CONTRACT_REVISION_V1,
        schema_sha256: DESKTOP_CALL_RECORDING_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_contract_has_no_boolean_consent_or_private_audio_locator() {
        let source = include_str!("../proto/makosh/desktop_call_recording/v1/recording.proto");
        for forbidden in [
            "consent_attested",
            "filesystem_path",
            "audio_input_label",
            "provider_id",
            "blob_reference",
            "custody_proof",
            "map<",
        ] {
            assert!(!source.contains(forbidden), "forbidden {forbidden}");
        }
        assert!(source.contains("canonical_wav_bytes"));
        assert_eq!(
            contract_reference_v1(HOST_CONTRACT_NAME_V1)
                .schema_sha256
                .len(),
            32
        );
        assert_ne!(START_CONTRACT_NAME_V1, STOP_CONTRACT_NAME_V1);
        assert_ne!(STOP_CONTRACT_NAME_V1, GET_CONTRACT_NAME_V1);
    }
}
