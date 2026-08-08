use std::{os::unix::net::UnixStream, time::Duration};

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobCustodyTransferRequestV1,
    ManagedBlobSessionRequestV1, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::BlobDataOperationV1,
};
use makosh_whisper_stt_core::{
    WhisperSttBlobReceiptV1, WhisperSttExecutionPlanV1, build_whisper_stt_artifact_v1,
    complete_whisper_stt_result_v1,
};
use makosh_whisper_stt_process::{
    WhisperSttProcessConfigurationV1, WhisperSttProcessErrorV1, execute_whisper_stt_process_v1,
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::{
    WHISPER_STT_BLOB_CAPABILITY_ID_V1,
    worker::{WhisperSttExecutionPortV1, WhisperSttPortErrorV1},
};

pub(crate) struct ManagedWhisperSttExecutionPortV1<'a> {
    pub channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub process: &'a WhisperSttProcessConfigurationV1,
    pub target_owner_id: &'a str,
    pub target_module_id: &'a str,
    pub target_capability_id: &'a str,
}

impl WhisperSttExecutionPortV1 for ManagedWhisperSttExecutionPortV1<'_> {
    fn transcribe(
        &mut self,
        plan: &WhisperSttExecutionPlanV1,
    ) -> Result<makosh_speech_to_text_api::wire::SpeechToTextResultV1, WhisperSttPortErrorV1> {
        let source = plan
            .request
            .source
            .as_ref()
            .ok_or(WhisperSttPortErrorV1::UnsupportedAudio)?;
        let reference_id = id16(&source.reference_id)?;
        let source_sha256 = id32(&source.sha256)?;
        let audio = materialize_and_read_exact(
            self.channel,
            &reference_id,
            source.declared_bytes,
            &source_sha256,
            &source.custody_transfer_source_proof,
            &id16(&plan.request.request_id)?,
            &id32(&plan.request.request_digest)?,
        )?;
        let outcome = execute_whisper_stt_process_v1(self.process, plan, audio.as_slice())
            .map_err(process_error)?;
        let artifact = build_whisper_stt_artifact_v1(plan, outcome)
            .map_err(|_| WhisperSttPortErrorV1::ProviderRejected)?;
        let transcript_reference = transcript_reference_id(plan);
        let proof = write_exact(
            self.channel,
            &transcript_reference,
            artifact.sha256,
            &artifact.encoded_document,
            self.target_owner_id,
            self.target_module_id,
            self.target_capability_id,
        )?;
        complete_whisper_stt_result_v1(
            plan,
            &artifact,
            WhisperSttBlobReceiptV1 {
                reference_id: transcript_reference,
                declared_bytes: artifact.encoded_document.len() as u64,
                sha256: artifact.sha256,
                custody_transfer_source_proof: proof,
            },
        )
        .map_err(|_| WhisperSttPortErrorV1::ProviderRejected)
    }
}

fn materialize_and_read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    source_reference_id: &[u8; 16],
    declared_size: u64,
    sha256: &[u8; 32],
    custody_source_proof: &[u8],
    evidence_id: &[u8; 16],
    evidence_envelope_sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, WhisperSttPortErrorV1> {
    blocking(channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let transfer = request_managed_blob_custody_transfer_v2(
            channel,
            &mut dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: WHISPER_STT_BLOB_CAPABILITY_ID_V1,
                source_reference_id,
                declared_size,
                receipt_sha256: sha256,
                custody_source_proof,
                evidence_id,
                evidence_envelope_sha256,
            },
        )
        .map_err(|_| WhisperSttPortErrorV1::Unavailable)?;
        let reference_id: [u8; 16] = transfer
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| WhisperSttPortErrorV1::Unavailable)?;
        BlobDataClient::new(transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(|_| WhisperSttPortErrorV1::Unavailable)?;
        read_owned_exact(channel, &reference_id, declared_size, sha256)
    })
}

fn write_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    reference_id: &[u8; 16],
    sha256: [u8; 32],
    bytes: &[u8],
    target_owner_id: &str,
    target_module_id: &str,
    target_capability_id: &str,
) -> Result<Vec<u8>, WhisperSttPortErrorV1> {
    blocking(channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: WHISPER_STT_BLOB_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id,
                declared_size: bytes.len() as u64,
                backup_class: 1,
                receipt_sha256: Some(&sha256),
                custody_target: Some(ManagedBlobCustodyTargetV1 {
                    owner_id: target_owner_id,
                    module_id: target_module_id,
                    capability_id: target_capability_id,
                }),
            },
        )
        .map_err(|_| WhisperSttPortErrorV1::Unavailable)?;
        if session.custody_transfer_source_proof.is_empty() {
            return Err(WhisperSttPortErrorV1::Uncertain);
        }
        let write = BlobDataClient::new(session.data_socket_path).and_then(|client| {
            client.write(session.grant, session.channel_binding, bytes.to_vec())
        });
        if write.is_err() {
            let existing = read_owned_exact(channel, reference_id, bytes.len() as u64, &sha256)?;
            if existing.as_slice() != bytes {
                return Err(WhisperSttPortErrorV1::Uncertain);
            }
        }
        Ok(session.custody_transfer_source_proof)
    })
}

fn read_owned_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_size: u64,
    sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, WhisperSttPortErrorV1> {
    let mut dispatcher = RejectManagedControlRequestsV2;
    let session = request_managed_blob_session_v2(
        channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: WHISPER_STT_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target: None,
        },
    )
    .map_err(|_| WhisperSttPortErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(session.grant, session.channel_binding, 0, declared_size)
            })
            .map_err(|_| WhisperSttPortErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_size).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != sha256
    {
        return Err(WhisperSttPortErrorV1::UnsupportedAudio);
    }
    Ok(bytes)
}

fn transcript_reference_id(plan: &WhisperSttExecutionPlanV1) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh-whisper-transcript-v1\0");
    digest.update(&plan.request.request_id);
    digest.update(&plan.request.request_digest);
    let full = digest.finalize();
    let mut value = [0_u8; 16];
    value.copy_from_slice(&full[..16]);
    value
}

fn process_error(error: WhisperSttProcessErrorV1) -> WhisperSttPortErrorV1 {
    match error {
        WhisperSttProcessErrorV1::InvalidAudio => WhisperSttPortErrorV1::UnsupportedAudio,
        WhisperSttProcessErrorV1::ProcessRejected | WhisperSttProcessErrorV1::InvalidOutput => {
            WhisperSttPortErrorV1::ProviderRejected
        }
        WhisperSttProcessErrorV1::InvalidConfiguration
        | WhisperSttProcessErrorV1::WorkUnavailable
        | WhisperSttProcessErrorV1::SpawnFailed
        | WhisperSttProcessErrorV1::TimedOut
        | WhisperSttProcessErrorV1::OutputUnavailable => WhisperSttPortErrorV1::Uncertain,
    }
}

fn blocking<T>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    operation: impl FnOnce(&mut ManagedControlChannelV2<UnixStream>) -> Result<T, WhisperSttPortErrorV1>,
) -> Result<T, WhisperSttPortErrorV1> {
    channel
        .inner_mut()
        .set_nonblocking(false)
        .and_then(|_| {
            channel
                .inner_mut()
                .set_read_timeout(Some(Duration::from_secs(5)))
        })
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| WhisperSttPortErrorV1::Unavailable)?;
    let result = operation(channel);
    let restored = channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true));
    if restored.is_err() {
        return Err(WhisperSttPortErrorV1::Unavailable);
    }
    result
}

fn id16(value: &[u8]) -> Result<[u8; 16], WhisperSttPortErrorV1> {
    value
        .try_into()
        .map_err(|_| WhisperSttPortErrorV1::UnsupportedAudio)
}

fn id32(value: &[u8]) -> Result<[u8; 32], WhisperSttPortErrorV1> {
    value
        .try_into()
        .map_err(|_| WhisperSttPortErrorV1::UnsupportedAudio)
}
