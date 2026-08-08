use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyDelegationRequestV1,
    ManagedBlobCustodyTargetV1, ManagedBlobCustodyTransferRequestV1,
    ManagedBlobResolvedProviderCustodyDelegationRequestV1,
    request_managed_blob_custody_delegation_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_resolved_provider_custody_delegation_v1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeModuleRequestRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1, validate_module_request_request_v1,
        validate_module_request_response_v1,
    },
};
use makosh_speech_to_text_api::{
    seal_speech_to_text_request_v1, speech_to_text_provider_contract_reference_v1,
    validate_speech_to_text_request_v1, validate_speech_to_text_result_v1,
    wire::{SpeechToTextRequestV1, SpeechToTextResultV1, SpeechToTextTerminalStatusV1},
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1, SpeechToTextExecutionPortsV1,
    SpeechToTextResponseBlobTargetV1, worker::SpeechToTextPortErrorV1,
};

pub struct ManagedSpeechToTextExecutionPortsV1<'a> {
    pub control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
}

struct CustodyMaterializationV1<'a> {
    source_reference_id: &'a [u8; 16],
    declared_size: u64,
    receipt_sha256: &'a [u8; 32],
    custody_source_proof: &'a [u8],
    evidence_id: &'a [u8; 16],
    evidence_envelope_sha256: &'a [u8; 32],
}

impl ManagedSpeechToTextExecutionPortsV1<'_> {
    fn materialize_custody(
        &mut self,
        request: CustodyMaterializationV1<'_>,
    ) -> Result<[u8; 16], SpeechToTextPortErrorV1> {
        let transfer = request_managed_blob_custody_transfer_v2(
            self.control_channel,
            self.dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1,
                source_reference_id: request.source_reference_id,
                declared_size: request.declared_size,
                receipt_sha256: request.receipt_sha256,
                custody_source_proof: request.custody_source_proof,
                evidence_id: request.evidence_id,
                evidence_envelope_sha256: request.evidence_envelope_sha256,
            },
        )
        .map_err(blob_error)?;
        let target_reference_id = id16(&transfer.grant.target_reference_id)?;
        BlobDataClient::new(transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(blob_error)?;
        Ok(target_reference_id)
    }
}

impl SpeechToTextExecutionPortsV1 for ManagedSpeechToTextExecutionPortsV1<'_> {
    fn transcribe(
        &mut self,
        mut request: SpeechToTextRequestV1,
        response_target: &SpeechToTextResponseBlobTargetV1,
    ) -> Result<SpeechToTextResultV1, SpeechToTextPortErrorV1> {
        validate_speech_to_text_request_v1(&request)
            .map_err(|_| SpeechToTextPortErrorV1::Rejected)?;
        let caller_request = request.clone();
        let request_id = id16(&request.request_id)?;
        let request_digest = id32(&request.request_digest)?;
        let source = request
            .source
            .as_mut()
            .ok_or(SpeechToTextPortErrorV1::Rejected)?;
        let source_reference_id = id16(&source.reference_id)?;
        let source_sha256 = id32(&source.sha256)?;
        let predecessor_proof = source.custody_transfer_source_proof.clone();
        let engine_reference_id = self.materialize_custody(CustodyMaterializationV1 {
            source_reference_id: &source_reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source_sha256,
            custody_source_proof: &predecessor_proof,
            evidence_id: &request_id,
            evidence_envelope_sha256: &request_digest,
        })?;
        let provider_contract = speech_to_text_provider_contract_reference_v1();
        let audio_delegation_id =
            operation_id(b"speech-audio-provider", &request_id, &request_digest);
        let audio_delegation = request_managed_blob_resolved_provider_custody_delegation_v1(
            self.control_channel,
            self.dispatcher,
            ManagedBlobResolvedProviderCustodyDelegationRequestV1 {
                request_id: &audio_delegation_id,
                capability_id: SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1,
                current_reference_id: &engine_reference_id,
                predecessor_custody_source_proof: &predecessor_proof,
                predecessor_evidence_id: &request_id,
                predecessor_evidence_envelope_sha256: &request_digest,
                target_request_contract: &provider_contract,
            },
        )
        .map_err(blob_error)?;
        source.reference_id = engine_reference_id.to_vec();
        source.custody_transfer_source_proof = audio_delegation.custody_transfer_source_proof;
        request = seal_speech_to_text_request_v1(request)
            .map_err(|_| SpeechToTextPortErrorV1::Rejected)?;
        let provider_request_digest = id32(&request.request_digest)?;

        let routed = ManagedRuntimeModuleRequestRequestV1 {
            request_id: request_id.to_vec(),
            contract: Some(provider_contract),
            request_payload: request.encode_to_vec(),
            deadline_millis: MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1,
            response_blob_capability_id: SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1.to_owned(),
        };
        validate_module_request_request_v1(&routed)
            .map_err(|_| SpeechToTextPortErrorV1::Rejected)?;
        let response = self
            .control_channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteModuleRequest(routed)),
                },
                self.dispatcher,
            )
            .map_err(|_| SpeechToTextPortErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(SpeechToTextPortErrorV1::Unavailable);
        }
        let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
            return Err(SpeechToTextPortErrorV1::Unavailable);
        };
        validate_module_request_response_v1(&response)
            .map_err(|_| SpeechToTextPortErrorV1::Unavailable)?;
        if response.request_id != request_id {
            return Err(SpeechToTextPortErrorV1::Unavailable);
        }
        match response.error_code.as_str() {
            "" => {}
            "REJECTED" => return Err(SpeechToTextPortErrorV1::Rejected),
            "UNAVAILABLE" => return Err(SpeechToTextPortErrorV1::Unavailable),
            _ => return Err(SpeechToTextPortErrorV1::Unavailable),
        }
        let mut result = SpeechToTextResultV1::decode(response.response_payload.as_slice())
            .map_err(|_| SpeechToTextPortErrorV1::Rejected)?;
        validate_speech_to_text_result_v1(&request, &result)
            .map_err(|_| SpeechToTextPortErrorV1::Rejected)?;
        if result.terminal_status == SpeechToTextTerminalStatusV1::Rejected as i32 {
            result.request_digest = caller_request.request_digest.clone();
            validate_speech_to_text_result_v1(&caller_request, &result)
                .map_err(|_| SpeechToTextPortErrorV1::Rejected)?;
            return Ok(result);
        }

        let transcript = result
            .transcript
            .as_mut()
            .ok_or(SpeechToTextPortErrorV1::Rejected)?;
        let provider_transcript_reference_id = id16(&transcript.reference_id)?;
        let transcript_sha256 = id32(&transcript.sha256)?;
        let predecessor_proof = transcript.custody_transfer_source_proof.clone();
        let engine_transcript_reference_id =
            self.materialize_custody(CustodyMaterializationV1 {
                source_reference_id: &provider_transcript_reference_id,
                declared_size: transcript.declared_bytes,
                receipt_sha256: &transcript_sha256,
                custody_source_proof: &predecessor_proof,
                evidence_id: &request_id,
                evidence_envelope_sha256: &provider_request_digest,
            })?;
        let transcript_delegation_id =
            operation_id(b"speech-transcript-caller", &request_id, &request_digest);
        let transcript_delegation = request_managed_blob_custody_delegation_v2(
            self.control_channel,
            self.dispatcher,
            ManagedBlobCustodyDelegationRequestV1 {
                request_id: &transcript_delegation_id,
                capability_id: SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1,
                current_reference_id: &engine_transcript_reference_id,
                predecessor_custody_source_proof: &predecessor_proof,
                predecessor_evidence_id: &request_id,
                predecessor_evidence_envelope_sha256: &provider_request_digest,
                target: ManagedBlobCustodyTargetV1 {
                    owner_id: &response_target.owner_id,
                    module_id: &response_target.module_id,
                    capability_id: &response_target.capability_id,
                },
            },
        )
        .map_err(blob_error)?;
        transcript.reference_id = engine_transcript_reference_id.to_vec();
        transcript.custody_transfer_source_proof =
            transcript_delegation.custody_transfer_source_proof;
        result.request_digest = caller_request.request_digest.clone();
        validate_speech_to_text_result_v1(&caller_request, &result)
            .map_err(|_| SpeechToTextPortErrorV1::Rejected)?;
        Ok(result)
    }
}

fn operation_id(domain: &[u8], request_id: &[u8; 16], request_digest: &[u8; 32]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.speech-to-text.operation.v1\0");
    hasher.update(domain);
    hasher.update(request_id);
    hasher.update(request_digest);
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16].try_into().expect("fixed SHA-256 prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], SpeechToTextPortErrorV1> {
    value
        .try_into()
        .map_err(|_| SpeechToTextPortErrorV1::Rejected)
}

fn id32(value: &[u8]) -> Result<[u8; 32], SpeechToTextPortErrorV1> {
    value
        .try_into()
        .map_err(|_| SpeechToTextPortErrorV1::Rejected)
}

fn blob_error(error: BlobClientError) -> SpeechToTextPortErrorV1 {
    match error {
        BlobClientError::Unavailable => SpeechToTextPortErrorV1::Unavailable,
        BlobClientError::Rejected(_) => SpeechToTextPortErrorV1::Rejected,
        _ => SpeechToTextPortErrorV1::Rejected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custody_operation_ids_are_purpose_separated_and_repeatable() {
        let request_id = [1; 16];
        let digest = [2; 32];
        assert_eq!(
            operation_id(b"speech-audio-provider", &request_id, &digest),
            operation_id(b"speech-audio-provider", &request_id, &digest)
        );
        assert_ne!(
            operation_id(b"speech-audio-provider", &request_id, &digest),
            operation_id(b"speech-transcript-caller", &request_id, &digest)
        );
    }
}
