#![forbid(unsafe_code)]

use makosh_runtime_protocol::v1::ContractReferenceV1;

mod validation;

pub use validation::{
    SpeechToTextContractErrorV1, compute_speech_to_text_request_digest_v1,
    seal_speech_to_text_request_v1, validate_speech_to_text_request_v1,
    validate_speech_to_text_result_v1,
};

pub const PACKAGE: &str = "makosh-speech-to-text-api";
pub const SPEECH_TO_TEXT_OWNER_V1: &str = "speech_to_text";
pub const SPEECH_TO_TEXT_MODULE_ID_V1: &str = "makosh-speech-to-text-runtime";
pub const SPEECH_TO_TEXT_CAPABILITY_ID_V1: &str = "speech_to_text.transcribe.v1";
pub const SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1: &str = "speech_to_text.blob.v1";
pub const SPEECH_TO_TEXT_CONTRACT_NAME_V1: &str = "speech_to_text.transcribe";
pub const SPEECH_TO_TEXT_PROVIDER_CONTRACT_NAME_V1: &str = "speech_to_text.provider_transcribe";
pub const SPEECH_TO_TEXT_CONTRACT_MAJOR_V1: u32 = 1;
pub const SPEECH_TO_TEXT_CONTRACT_REVISION_V1: u32 = 1;
pub const SPEECH_TO_TEXT_MAX_AUDIO_BYTES_V1: u64 = 512 * 1024 * 1024;
pub const SPEECH_TO_TEXT_MAX_DURATION_MILLIS_V1: u64 = 4 * 60 * 60 * 1_000;
pub const SPEECH_TO_TEXT_MAX_TRANSCRIPT_BYTES_V1: u32 = 4 * 1024 * 1024;
pub const SPEECH_TO_TEXT_MAX_SEGMENTS_V1: u32 = 100_000;
pub const SPEECH_TO_TEXT_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.speech_to_text.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/speech_to_text_schema.rs"));

pub const SPEECH_TO_TEXT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/speech-to-text-v1.bin"));

#[must_use]
pub fn speech_to_text_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: SPEECH_TO_TEXT_OWNER_V1.to_owned(),
        name: SPEECH_TO_TEXT_CONTRACT_NAME_V1.to_owned(),
        major: SPEECH_TO_TEXT_CONTRACT_MAJOR_V1,
        revision: SPEECH_TO_TEXT_CONTRACT_REVISION_V1,
        schema_sha256: SPEECH_TO_TEXT_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn speech_to_text_provider_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: SPEECH_TO_TEXT_OWNER_V1.to_owned(),
        name: SPEECH_TO_TEXT_PROVIDER_CONTRACT_NAME_V1.to_owned(),
        major: SPEECH_TO_TEXT_CONTRACT_MAJOR_V1,
        revision: SPEECH_TO_TEXT_CONTRACT_REVISION_V1,
        schema_sha256: SPEECH_TO_TEXT_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_is_audio_receipt_based_and_private_content_negative() {
        let source = include_str!("../proto/makosh/speech_to_text/v1/speech_to_text.proto");
        for required in [
            "SpeechAudioSourceReceiptV1",
            "SpeechTranscriptArtifactReceiptV1",
            "consent_receipt_id",
            "request_digest",
            "WAV_PCM_S16LE_MONO_16000_HZ",
        ] {
            assert!(source.contains(required), "missing {required}");
        }
        for forbidden in [
            "transcript_text",
            "segment_text",
            "summary",
            "sender",
            "subject",
            "body_utf8",
            "provider_name",
            "model_name",
            "filesystem_path",
            "map<",
        ] {
            assert!(!source.contains(forbidden), "forbidden field {forbidden}");
        }
        let contract = speech_to_text_contract_reference_v1();
        assert_eq!(contract.owner, SPEECH_TO_TEXT_OWNER_V1);
        assert_eq!(contract.name, SPEECH_TO_TEXT_CONTRACT_NAME_V1);
        assert_eq!(contract.schema_sha256.len(), 32);
        assert_eq!(
            speech_to_text_provider_contract_reference_v1().name,
            "speech_to_text.provider_transcribe"
        );
    }
}
