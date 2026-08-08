#![forbid(unsafe_code)]

use makosh_runtime_protocol::v1::ContractReferenceV1;
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-call-transcription-api";
pub const OWNER_ID_V1: &str = "call_transcription";
pub const MODULE_ID_V1: &str = "makosh-call-transcription-runtime";
pub const CAPABILITY_ID_V1: &str = "call_transcription.v1";
pub const START_CONTRACT_NAME_V1: &str = "call_transcription.start";
pub const GET_CONTRACT_NAME_V1: &str = "call_transcription.get";
pub const TICKET_CONTRACT_NAME_V1: &str = "call_transcription.transcript_ticket";
pub const READ_CONTRACT_NAME_V1: &str = "call_transcription.read_transcript";
pub const REALTIME_CONTRACT_NAME_V1: &str = "call_transcription.status_changed";
pub const REALTIME_EVENT_KIND_V1: &str = "call_transcription.status_changed";
pub const START_CONNECT_PATH_V1: &str =
    "/makosh.call_transcription.v1.CallTranscriptionCommandService/Start";
pub const GET_CONNECT_PATH_V1: &str =
    "/makosh.call_transcription.v1.CallTranscriptionQueryService/Get";
pub const TICKET_CONNECT_PATH_V1: &str =
    "/makosh.call_transcription.v1.CallTranscriptTicketService/IssueRead";
pub const TRANSCRIPT_BLOB_PATH_V1: &str = "/api/blobs/call-transcription/v1/transcript";
pub const CONTRACT_MAJOR_V1: u32 = 1;
pub const CONTRACT_REVISION_V1: u32 = 1;
pub const MAX_TRANSCRIPT_BYTES_V1: u64 = 4 * 1024 * 1024;
pub const MAX_SEGMENTS_V1: u32 = 100_000;
pub const READ_TICKET_BYTES_V1: usize = 32;
pub const READ_TICKET_TTL_SECONDS_V1: i64 = 30;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.call_transcription.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/call_transcription_schema.rs"));
pub const DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/call-transcription-v1.bin"));

#[must_use]
pub fn contract_reference_v1(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: CONTRACT_MAJOR_V1,
        revision: CONTRACT_REVISION_V1,
        schema_sha256: CALL_TRANSCRIPTION_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn run_id_v1(operation_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.call-transcription.run.v1\0");
    digest.update(operation_id);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_contract_is_metadata_only_and_not_a_summary_facade() {
        let source = include_str!("../proto/makosh/call_transcription/v1/transcription.proto");
        for required in [
            "recording_evidence_id",
            "consent_receipt_id",
            "IssueCallTranscriptReadRequestV1",
            "ReadCallTranscriptRequestV1",
            "transcript_sha256",
            "segment_count",
        ] {
            assert!(source.contains(required), "missing {required}");
        }
        for forbidden in [
            "transcript_text",
            "segment_text",
            "summary",
            "source_message_id",
            "provider_id",
            "model_id",
            "blob_reference",
            "custody_proof",
            "filesystem_path",
            "map<",
        ] {
            assert!(!source.contains(forbidden), "forbidden {forbidden}");
        }
        assert_eq!(
            contract_reference_v1(START_CONTRACT_NAME_V1).owner,
            OWNER_ID_V1
        );
        assert_ne!(run_id_v1([1; 16]), run_id_v1([2; 16]));
    }
}
