//! Client for the owner-private Blob data socket.
//!
//! This package transports already-authorized session grants. It does not
//! issue grants, access the Vault, expose filesystem paths, or interpret
//! provider/domain payloads.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use makosh_blob_client_contract::{BlobReadError, BlobReadPort};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_runtime_protocol::v1::{
    BlobCustodyReleaseOutcomeV1, BlobCustodyReleaseReasonV1, BlobCustodySourceProofKindV1,
    BlobCustodySourceProofV1, BlobCustodyTransferGrantV1, BlobDataCustodyTransferRequestV1,
    BlobDataOperationV1, BlobDataReadRangeRequestV1, BlobDataRequestV1, BlobDataResponseV1,
    BlobDataSessionGrantV1, BlobDataWriteRequestV1, ContractReferenceV1,
    ManagedRuntimeBlobCustodyDelegationRequestV1, ManagedRuntimeBlobCustodyReleaseRequestV1,
    ManagedRuntimeBlobSessionRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, blob_data_request_v1::Operation,
    managed_runtime_control_request_v1::Operation as ControlOperation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-blob-client";
const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024 + 32 * 1024;
const CONTROL_FRAME_BYTES: usize = 512 * 1024;

pub struct ManagedBlobSessionV1 {
    pub data_socket_path: PathBuf,
    pub grant: BlobDataSessionGrantV1,
    pub channel_binding: Vec<u8>,
    pub custody_transfer_source_proof: Vec<u8>,
}

/// Kernel-authorized internal rewrap request. It transports no source bytes,
/// key material, provider identity, or target storage path.
pub struct ManagedBlobCustodyTransferV1 {
    pub data_socket_path: PathBuf,
    pub grant: BlobCustodyTransferGrantV1,
    pub channel_binding: Vec<u8>,
}

/// Typed evidence-bound request for an internal rewrap session.
pub struct ManagedBlobCustodyTransferRequestV1<'a> {
    pub capability_id: &'a str,
    pub source_reference_id: &'a [u8; 16],
    pub declared_size: u64,
    pub receipt_sha256: &'a [u8; 32],
    pub custody_source_proof: &'a [u8],
    pub evidence_id: &'a [u8; 16],
    pub evidence_envelope_sha256: &'a [u8; 32],
}

pub struct ManagedBlobCustodyReleaseRequestV1<'a> {
    pub operation_id: &'a [u8; 16],
    pub capability_id: &'a str,
    pub reference_id: &'a [u8; 16],
    pub declared_size: u64,
    pub receipt_sha256: &'a [u8; 32],
    pub custody_source_proof: &'a [u8],
    pub reason: BlobCustodyReleaseReasonV1,
}

pub struct ManagedBlobCustodyDelegationRequestV1<'a> {
    pub request_id: &'a [u8; 16],
    pub capability_id: &'a str,
    pub current_reference_id: &'a [u8; 16],
    pub predecessor_custody_source_proof: &'a [u8],
    pub predecessor_evidence_id: &'a [u8; 16],
    pub predecessor_evidence_envelope_sha256: &'a [u8; 32],
    pub target: ManagedBlobCustodyTargetV1<'a>,
}

pub struct ManagedBlobResolvedProviderCustodyDelegationRequestV1<'a> {
    pub request_id: &'a [u8; 16],
    pub capability_id: &'a str,
    pub current_reference_id: &'a [u8; 16],
    pub predecessor_custody_source_proof: &'a [u8],
    pub predecessor_evidence_id: &'a [u8; 16],
    pub predecessor_evidence_envelope_sha256: &'a [u8; 32],
    pub target_request_contract: &'a ContractReferenceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedBlobCustodyDelegationV1 {
    pub request_id: [u8; 16],
    pub custody_transfer_source_proof: Vec<u8>,
    pub resolved_target_owner_id: String,
    pub resolved_target_module_id: String,
    pub resolved_target_capability_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagedBlobCustodyReleaseV1 {
    pub operation_id: [u8; 16],
    pub outcome: BlobCustodyReleaseOutcomeV1,
    pub delete_not_before_unix_ms: u64,
}

/// One exact Blob data-session intent, independent of the control transport.
pub struct ManagedBlobSessionRequestV1<'a> {
    pub capability_id: &'a str,
    pub operation: BlobDataOperationV1,
    pub reference_id: &'a [u8],
    pub declared_size: u64,
    pub backup_class: u32,
    pub receipt_sha256: Option<&'a [u8; 32]>,
    pub custody_target: Option<ManagedBlobCustodyTargetV1<'a>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagedBlobCustodyTargetV1<'a> {
    pub owner_id: &'a str,
    pub module_id: &'a str,
    pub capability_id: &'a str,
}

pub fn request_managed_blob_custody_transfer(
    channel: &mut UnixStream,
    request: ManagedBlobCustodyTransferRequestV1<'_>,
) -> Result<ManagedBlobCustodyTransferV1, BlobClientError> {
    if request.capability_id.is_empty()
        || request.capability_id.len() > 128
        || request.source_reference_id.iter().all(|byte| *byte == 0)
        || request.declared_size == 0
        || request.receipt_sha256.iter().all(|byte| *byte == 0)
        || request.custody_source_proof.is_empty()
        || request.custody_source_proof.len() > 2_048
        || request.evidence_id.iter().all(|byte| *byte == 0)
        || request
            .evidence_envelope_sha256
            .iter()
            .all(|byte| *byte == 0)
    {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    let mut request_id = [0_u8; 16];
    let mut channel_binding = vec![0_u8; 32];
    getrandom::fill(&mut request_id).map_err(|_| BlobClientError::Unavailable)?;
    getrandom::fill(&mut channel_binding).map_err(|_| BlobClientError::Unavailable)?;
    if request_id.iter().all(|byte| *byte == 0) || channel_binding.iter().all(|byte| *byte == 0) {
        return Err(BlobClientError::Unavailable);
    }
    let control_request = ManagedRuntimeControlRequestV1 {
        operation: Some(ControlOperation::IssueBlobSession(
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: request_id.to_vec(),
                capability_id: request.capability_id.to_owned(),
                operation: BlobDataOperationV1::BlobDataOperationCustodyTransferV1 as u32,
                channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
                reference_id: request.source_reference_id.to_vec(),
                declared_size: request.declared_size,
                backup_class: 1,
                ttl_seconds: 30,
                receipt_sha256: request.receipt_sha256.to_vec(),
                custody_source_proof: request.custody_source_proof.to_vec(),
                evidence_id: request.evidence_id.to_vec(),
                evidence_envelope_sha256: request.evidence_envelope_sha256.to_vec(),
                custody_target_owner_id: String::new(),
                custody_target_module_id: String::new(),
                custody_target_capability_id: String::new(),
            },
        )),
    };
    let bytes = control_request.encode_to_vec();
    if bytes.len() > CONTROL_FRAME_BYTES {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    channel
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| channel.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|error| BlobClientError::Io(error.to_string()))?;
    write_frame(channel, &bytes)?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(channel)?.as_slice())
        .map_err(|_| BlobClientError::InvalidResponse)?;
    channel
        .set_read_timeout(None)
        .and_then(|_| channel.set_write_timeout(None))
        .map_err(|error| BlobClientError::Io(error.to_string()))?;
    let delivery = match response.result {
        Some(ControlResult::BlobSessionDelivery(delivery)) if response.error_code.is_empty() => {
            delivery
        }
        _ => {
            return Err(managed_blob_session_error(
                &response.error_code,
                "managed_blob_custody_transfer_denied",
            ));
        }
    };
    let grant = delivery
        .custody_transfer_grant
        .ok_or(BlobClientError::InvalidResponse)?;
    if delivery.grant.is_some()
        || !delivery.custody_transfer_source_proof.is_empty()
        || !Path::new(&delivery.data_socket_path).is_absolute()
        || grant.evidence_id.as_slice() != request.evidence_id.as_slice()
        || grant.evidence_envelope_sha256.as_slice() != request.evidence_envelope_sha256.as_slice()
        || grant.channel_binding_sha256 != Sha256::digest(&channel_binding).as_slice()
        || grant.target_reference_id.len() != 16
        || grant.target_reference_id.iter().all(|byte| *byte == 0)
    {
        return Err(BlobClientError::InvalidResponse);
    }
    Ok(ManagedBlobCustodyTransferV1 {
        data_socket_path: PathBuf::from(delivery.data_socket_path),
        grant,
        channel_binding,
    })
}

pub fn request_managed_blob_custody_release_v2(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: ManagedBlobCustodyReleaseRequestV1<'_>,
) -> Result<ManagedBlobCustodyReleaseV1, BlobClientError> {
    if request.operation_id.iter().all(|byte| *byte == 0)
        || !valid_token(request.capability_id)
        || request.reference_id.iter().all(|byte| *byte == 0)
        || request.declared_size == 0
        || request.receipt_sha256.iter().all(|byte| *byte == 0)
        || request.custody_source_proof.is_empty()
        || request.custody_source_proof.len() > 2_048
        || !matches!(
            request.reason,
            BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
                | BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
                | BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalCancelledV1
        )
    {
        return Err(BlobClientError::InvalidCustodyReleaseRequest);
    }
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(ControlOperation::ReleaseBlobCustody(
                    ManagedRuntimeBlobCustodyReleaseRequestV1 {
                        operation_id: request.operation_id.to_vec(),
                        capability_id: request.capability_id.to_owned(),
                        reference_id: request.reference_id.to_vec(),
                        declared_size: request.declared_size,
                        receipt_sha256: request.receipt_sha256.to_vec(),
                        custody_source_proof: request.custody_source_proof.to_vec(),
                        reason: request.reason as i32,
                    },
                )),
            },
            dispatcher,
        )
        .map_err(|_| BlobClientError::Unavailable)?;
    decode_custody_release_response(response, request.operation_id)
}

fn decode_custody_release_response(
    response: ManagedRuntimeControlResponseV1,
    expected_operation_id: &[u8; 16],
) -> Result<ManagedBlobCustodyReleaseV1, BlobClientError> {
    let delivery = match response.result {
        Some(ControlResult::BlobCustodyRelease(delivery)) if response.error_code.is_empty() => {
            delivery
        }
        _ if response.error_code == "managed_blob_custody_release_unavailable" => {
            return Err(BlobClientError::Unavailable);
        }
        _ => {
            return Err(BlobClientError::Rejected(
                "managed_blob_custody_release_denied".to_owned(),
            ));
        }
    };
    let operation_id = delivery
        .operation_id
        .as_slice()
        .try_into()
        .map_err(|_| BlobClientError::InvalidResponse)?;
    let outcome = BlobCustodyReleaseOutcomeV1::try_from(delivery.outcome)
        .map_err(|_| BlobClientError::InvalidResponse)?;
    if operation_id != *expected_operation_id
        || delivery.delete_not_before_unix_ms == 0
        || !matches!(
            outcome,
            BlobCustodyReleaseOutcomeV1::BlobCustodyReleaseOutcomeAcceptedV1
                | BlobCustodyReleaseOutcomeV1::BlobCustodyReleaseOutcomeExistingV1
                | BlobCustodyReleaseOutcomeV1::BlobCustodyReleaseOutcomeAlreadyReleasedV1
        )
    {
        return Err(BlobClientError::InvalidResponse);
    }
    Ok(ManagedBlobCustodyReleaseV1 {
        operation_id,
        outcome,
        delete_not_before_unix_ms: delivery.delete_not_before_unix_ms,
    })
}

pub fn request_managed_blob_custody_delegation_v2(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: ManagedBlobCustodyDelegationRequestV1<'_>,
) -> Result<ManagedBlobCustodyDelegationV1, BlobClientError> {
    if request.request_id.iter().all(|byte| *byte == 0)
        || !valid_token(request.capability_id)
        || request.current_reference_id.iter().all(|byte| *byte == 0)
        || request.predecessor_custody_source_proof.is_empty()
        || request.predecessor_custody_source_proof.len() > 2_048
        || request
            .predecessor_evidence_id
            .iter()
            .all(|byte| *byte == 0)
        || request
            .predecessor_evidence_envelope_sha256
            .iter()
            .all(|byte| *byte == 0)
        || !valid_target_token(request.target.owner_id)
        || !valid_target_token(request.target.module_id)
        || !valid_target_token(request.target.capability_id)
    {
        return Err(BlobClientError::InvalidCustodyDelegationRequest);
    }
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(ControlOperation::DelegateBlobCustody(
                    ManagedRuntimeBlobCustodyDelegationRequestV1 {
                        request_id: request.request_id.to_vec(),
                        capability_id: request.capability_id.to_owned(),
                        current_reference_id: request.current_reference_id.to_vec(),
                        predecessor_custody_source_proof: request
                            .predecessor_custody_source_proof
                            .to_vec(),
                        predecessor_evidence_id: request.predecessor_evidence_id.to_vec(),
                        predecessor_evidence_envelope_sha256: request
                            .predecessor_evidence_envelope_sha256
                            .to_vec(),
                        target_owner_id: request.target.owner_id.to_owned(),
                        target_module_id: request.target.module_id.to_owned(),
                        target_capability_id: request.target.capability_id.to_owned(),
                        target_request_contract: None,
                    },
                )),
            },
            dispatcher,
        )
        .map_err(|_| BlobClientError::Unavailable)?;
    decode_custody_delegation_response(response, &request)
}

pub fn request_managed_blob_resolved_provider_custody_delegation_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: ManagedBlobResolvedProviderCustodyDelegationRequestV1<'_>,
) -> Result<ManagedBlobCustodyDelegationV1, BlobClientError> {
    if request.request_id.iter().all(|byte| *byte == 0)
        || !valid_token(request.capability_id)
        || request.current_reference_id.iter().all(|byte| *byte == 0)
        || request.predecessor_custody_source_proof.is_empty()
        || request.predecessor_custody_source_proof.len() > 2_048
        || request
            .predecessor_evidence_id
            .iter()
            .all(|byte| *byte == 0)
        || request
            .predecessor_evidence_envelope_sha256
            .iter()
            .all(|byte| *byte == 0)
        || !valid_contract_reference(request.target_request_contract)
    {
        return Err(BlobClientError::InvalidCustodyDelegationRequest);
    }
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(ControlOperation::DelegateBlobCustody(
                    ManagedRuntimeBlobCustodyDelegationRequestV1 {
                        request_id: request.request_id.to_vec(),
                        capability_id: request.capability_id.to_owned(),
                        current_reference_id: request.current_reference_id.to_vec(),
                        predecessor_custody_source_proof: request
                            .predecessor_custody_source_proof
                            .to_vec(),
                        predecessor_evidence_id: request.predecessor_evidence_id.to_vec(),
                        predecessor_evidence_envelope_sha256: request
                            .predecessor_evidence_envelope_sha256
                            .to_vec(),
                        target_owner_id: String::new(),
                        target_module_id: String::new(),
                        target_capability_id: String::new(),
                        target_request_contract: Some(request.target_request_contract.clone()),
                    },
                )),
            },
            dispatcher,
        )
        .map_err(|_| BlobClientError::Unavailable)?;
    decode_resolved_custody_delegation_response(response, &request)
}

fn decode_custody_delegation_response(
    response: ManagedRuntimeControlResponseV1,
    request: &ManagedBlobCustodyDelegationRequestV1<'_>,
) -> Result<ManagedBlobCustodyDelegationV1, BlobClientError> {
    let delivery = delegation_delivery(response)?;
    let decoded = decode_delegation_proof(
        &delivery,
        request.request_id,
        request.current_reference_id,
        request.predecessor_custody_source_proof,
    )?;
    if decoded.resolved_target_owner_id != request.target.owner_id
        || decoded.resolved_target_module_id != request.target.module_id
        || decoded.resolved_target_capability_id != request.target.capability_id
    {
        return Err(BlobClientError::InvalidResponse);
    }
    Ok(decoded)
}

fn decode_resolved_custody_delegation_response(
    response: ManagedRuntimeControlResponseV1,
    request: &ManagedBlobResolvedProviderCustodyDelegationRequestV1<'_>,
) -> Result<ManagedBlobCustodyDelegationV1, BlobClientError> {
    let delivery = delegation_delivery(response)?;
    decode_delegation_proof(
        &delivery,
        request.request_id,
        request.current_reference_id,
        request.predecessor_custody_source_proof,
    )
}

fn delegation_delivery(
    response: ManagedRuntimeControlResponseV1,
) -> Result<
    makosh_runtime_protocol::v1::ManagedRuntimeBlobCustodyDelegationDeliveryV1,
    BlobClientError,
> {
    match response.result {
        Some(ControlResult::BlobCustodyDelegation(delivery)) if response.error_code.is_empty() => {
            Ok(delivery)
        }
        _ if response.error_code == "managed_blob_custody_delegation_unavailable" => {
            Err(BlobClientError::Unavailable)
        }
        _ => Err(BlobClientError::Rejected(
            "managed_blob_custody_delegation_denied".to_owned(),
        )),
    }
}

fn decode_delegation_proof(
    delivery: &makosh_runtime_protocol::v1::ManagedRuntimeBlobCustodyDelegationDeliveryV1,
    expected_request_id: &[u8; 16],
    expected_reference_id: &[u8; 16],
    predecessor_proof: &[u8],
) -> Result<ManagedBlobCustodyDelegationV1, BlobClientError> {
    let request_id: [u8; 16] = delivery
        .request_id
        .as_slice()
        .try_into()
        .map_err(|_| BlobClientError::InvalidResponse)?;
    let proof = BlobCustodySourceProofV1::decode(delivery.custody_transfer_source_proof.as_slice())
        .map_err(|_| BlobClientError::InvalidResponse)?;
    if request_id != *expected_request_id
        || proof.proof_kind
            != BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1
                as i32
        || proof.delegation_id != expected_request_id
        || proof.reference_id != expected_reference_id
        || proof.predecessor_proof_sha256
            != Sha256::digest(predecessor_proof).as_slice()
        || !valid_target_token(&delivery.resolved_target_owner_id)
        || !valid_target_token(&delivery.resolved_target_module_id)
        || !valid_target_token(&delivery.resolved_target_capability_id)
        || proof.target_owner_id != delivery.resolved_target_owner_id
        || proof.target_module_id != delivery.resolved_target_module_id
        || proof.target_capability_id != delivery.resolved_target_capability_id
    {
        return Err(BlobClientError::InvalidResponse);
    }
    Ok(ManagedBlobCustodyDelegationV1 {
        request_id,
        custody_transfer_source_proof: delivery.custody_transfer_source_proof.clone(),
        resolved_target_owner_id: delivery.resolved_target_owner_id.clone(),
        resolved_target_module_id: delivery.resolved_target_module_id.clone(),
        resolved_target_capability_id: delivery.resolved_target_capability_id.clone(),
    })
}

fn valid_contract_reference(value: &ContractReferenceV1) -> bool {
    valid_target_token(&value.owner)
        && valid_target_token(&value.name)
        && value.major > 0
        && value.revision > 0
        && value.schema_sha256.len() == 32
        && value.schema_sha256.iter().any(|byte| *byte != 0)
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

/// Correlated V2 transport for the exact custody-transfer session operation.
///
/// The request stays evidence-bound and carries no source payload. The
/// caller owns the one inherited control channel; this client only issues the
/// typed Blob grant request over that channel.
pub fn request_managed_blob_custody_transfer_v2(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: ManagedBlobCustodyTransferRequestV1<'_>,
) -> Result<ManagedBlobCustodyTransferV1, BlobClientError> {
    if request.capability_id.is_empty()
        || request.capability_id.len() > 128
        || request.source_reference_id.iter().all(|byte| *byte == 0)
        || request.declared_size == 0
        || request.receipt_sha256.iter().all(|byte| *byte == 0)
        || request.custody_source_proof.is_empty()
        || request.custody_source_proof.len() > 2_048
        || request.evidence_id.iter().all(|byte| *byte == 0)
        || request
            .evidence_envelope_sha256
            .iter()
            .all(|byte| *byte == 0)
    {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    let mut request_id = [0_u8; 16];
    let mut channel_binding = vec![0_u8; 32];
    getrandom::fill(&mut request_id).map_err(|_| BlobClientError::Unavailable)?;
    getrandom::fill(&mut channel_binding).map_err(|_| BlobClientError::Unavailable)?;
    if request_id.iter().all(|byte| *byte == 0) || channel_binding.iter().all(|byte| *byte == 0) {
        return Err(BlobClientError::Unavailable);
    }
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(ControlOperation::IssueBlobSession(
                    ManagedRuntimeBlobSessionRequestV1 {
                        request_id: request_id.to_vec(),
                        capability_id: request.capability_id.to_owned(),
                        operation: BlobDataOperationV1::BlobDataOperationCustodyTransferV1 as u32,
                        channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
                        reference_id: request.source_reference_id.to_vec(),
                        declared_size: request.declared_size,
                        backup_class: 1,
                        ttl_seconds: 30,
                        receipt_sha256: request.receipt_sha256.to_vec(),
                        custody_source_proof: request.custody_source_proof.to_vec(),
                        evidence_id: request.evidence_id.to_vec(),
                        evidence_envelope_sha256: request.evidence_envelope_sha256.to_vec(),
                        custody_target_owner_id: String::new(),
                        custody_target_module_id: String::new(),
                        custody_target_capability_id: String::new(),
                    },
                )),
            },
            dispatcher,
        )
        .map_err(|_| BlobClientError::Unavailable)?;
    let delivery = match response.result {
        Some(ControlResult::BlobSessionDelivery(delivery)) if response.error_code.is_empty() => {
            delivery
        }
        _ => {
            return Err(managed_blob_session_error(
                &response.error_code,
                "managed_blob_custody_transfer_denied",
            ));
        }
    };
    let grant = delivery
        .custody_transfer_grant
        .ok_or(BlobClientError::InvalidResponse)?;
    if delivery.grant.is_some()
        || !delivery.custody_transfer_source_proof.is_empty()
        || !Path::new(&delivery.data_socket_path).is_absolute()
        || grant.evidence_id.as_slice() != request.evidence_id.as_slice()
        || grant.evidence_envelope_sha256.as_slice() != request.evidence_envelope_sha256.as_slice()
        || grant.channel_binding_sha256 != Sha256::digest(&channel_binding).as_slice()
        || grant.target_reference_id.len() != 16
        || grant.target_reference_id.iter().all(|byte| *byte == 0)
    {
        return Err(BlobClientError::InvalidResponse);
    }
    Ok(ManagedBlobCustodyTransferV1 {
        data_socket_path: PathBuf::from(delivery.data_socket_path),
        grant,
        channel_binding,
    })
}

pub fn request_managed_blob_session(
    channel: &mut UnixStream,
    capability_id: &str,
    operation: BlobDataOperationV1,
    reference_id: &[u8],
    declared_size: u64,
    backup_class: u32,
    receipt_sha256: Option<&[u8; 32]>,
) -> Result<ManagedBlobSessionV1, BlobClientError> {
    if capability_id.is_empty()
        || capability_id.len() > 128
        || reference_id.len() != 16
        || reference_id.iter().all(|byte| *byte == 0)
        || declared_size == 0
        || !(1..=3).contains(&backup_class)
        || !valid_receipt_binding(operation, receipt_sha256)
    {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    let mut request_id = [0_u8; 16];
    let mut channel_binding = vec![0_u8; 32];
    getrandom::fill(&mut request_id).map_err(|_| BlobClientError::Unavailable)?;
    getrandom::fill(&mut channel_binding).map_err(|_| BlobClientError::Unavailable)?;
    if request_id.iter().all(|byte| *byte == 0) || channel_binding.iter().all(|byte| *byte == 0) {
        return Err(BlobClientError::Unavailable);
    }
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(ControlOperation::IssueBlobSession(
            ManagedRuntimeBlobSessionRequestV1 {
                request_id: request_id.to_vec(),
                capability_id: capability_id.to_owned(),
                operation: operation as u32,
                channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
                reference_id: reference_id.to_vec(),
                declared_size,
                backup_class,
                ttl_seconds: 30,
                receipt_sha256: receipt_sha256.map_or_else(Vec::new, |digest| digest.to_vec()),
                custody_source_proof: Vec::new(),
                evidence_id: Vec::new(),
                evidence_envelope_sha256: Vec::new(),
                custody_target_owner_id: String::new(),
                custody_target_module_id: String::new(),
                custody_target_capability_id: String::new(),
            },
        )),
    };
    let bytes = request.encode_to_vec();
    if bytes.len() > CONTROL_FRAME_BYTES {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    channel
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| channel.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|error| BlobClientError::Io(error.to_string()))?;
    write_frame(channel, &bytes)?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(channel)?.as_slice())
        .map_err(|_| BlobClientError::InvalidResponse)?;
    channel
        .set_read_timeout(None)
        .and_then(|_| channel.set_write_timeout(None))
        .map_err(|error| BlobClientError::Io(error.to_string()))?;
    let delivery = match response.result {
        Some(ControlResult::BlobSessionDelivery(delivery)) if response.error_code.is_empty() => {
            delivery
        }
        _ => {
            return Err(managed_blob_session_error(
                &response.error_code,
                "managed_blob_session_denied",
            ));
        }
    };
    let grant = delivery.grant.ok_or(BlobClientError::InvalidResponse)?;
    if !Path::new(&delivery.data_socket_path).is_absolute()
        || grant.reference_id != reference_id
        || grant.declared_size != declared_size
        || grant.operation != operation as i32
        || grant.channel_binding_sha256 != Sha256::digest(&channel_binding).as_slice()
        || !exact_receipt_binding(&grant.expected_plaintext_sha256, receipt_sha256)
        || (receipt_sha256.is_some()
            && operation == BlobDataOperationV1::BlobDataOperationWriteV1
            && delivery.custody_transfer_source_proof.is_empty())
    {
        return Err(BlobClientError::InvalidResponse);
    }
    Ok(ManagedBlobSessionV1 {
        data_socket_path: PathBuf::from(delivery.data_socket_path),
        grant,
        channel_binding,
        custody_transfer_source_proof: delivery.custody_transfer_source_proof,
    })
}

/// Correlated V2 transport for the existing exact Blob-session operation.
pub fn request_managed_blob_session_v2(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: ManagedBlobSessionRequestV1<'_>,
) -> Result<ManagedBlobSessionV1, BlobClientError> {
    if request.capability_id.is_empty()
        || request.capability_id.len() > 128
        || request.reference_id.len() != 16
        || request.reference_id.iter().all(|byte| *byte == 0)
        || request.declared_size == 0
        || !(1..=3).contains(&request.backup_class)
        || !valid_receipt_binding(request.operation, request.receipt_sha256)
        || !valid_custody_target(
            request.operation,
            request.receipt_sha256,
            request.custody_target,
        )
    {
        return Err(BlobClientError::InvalidSessionRequest);
    }
    let mut request_id = [0_u8; 16];
    let mut channel_binding = vec![0_u8; 32];
    getrandom::fill(&mut request_id).map_err(|_| BlobClientError::Unavailable)?;
    getrandom::fill(&mut channel_binding).map_err(|_| BlobClientError::Unavailable)?;
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(ControlOperation::IssueBlobSession(
                    ManagedRuntimeBlobSessionRequestV1 {
                        request_id: request_id.to_vec(),
                        capability_id: request.capability_id.to_owned(),
                        operation: request.operation as u32,
                        channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
                        reference_id: request.reference_id.to_vec(),
                        declared_size: request.declared_size,
                        backup_class: request.backup_class,
                        ttl_seconds: 30,
                        receipt_sha256: request
                            .receipt_sha256
                            .map_or_else(Vec::new, |digest| digest.to_vec()),
                        custody_source_proof: Vec::new(),
                        evidence_id: Vec::new(),
                        evidence_envelope_sha256: Vec::new(),
                        custody_target_owner_id: request
                            .custody_target
                            .map_or_else(String::new, |target| target.owner_id.to_owned()),
                        custody_target_module_id: request
                            .custody_target
                            .map_or_else(String::new, |target| target.module_id.to_owned()),
                        custody_target_capability_id: request
                            .custody_target
                            .map_or_else(String::new, |target| target.capability_id.to_owned()),
                    },
                )),
            },
            dispatcher,
        )
        .map_err(|_| BlobClientError::Unavailable)?;
    let delivery = match response.result {
        Some(ControlResult::BlobSessionDelivery(delivery)) if response.error_code.is_empty() => {
            delivery
        }
        _ => {
            return Err(managed_blob_session_error(
                &response.error_code,
                "managed_blob_session_denied",
            ));
        }
    };
    let grant = delivery.grant.ok_or(BlobClientError::InvalidResponse)?;
    if !Path::new(&delivery.data_socket_path).is_absolute()
        || grant.reference_id != request.reference_id
        || grant.declared_size != request.declared_size
        || grant.operation != request.operation as i32
        || grant.channel_binding_sha256 != Sha256::digest(&channel_binding).as_slice()
        || !exact_receipt_binding(&grant.expected_plaintext_sha256, request.receipt_sha256)
        || (request.receipt_sha256.is_some()
            && request.operation == BlobDataOperationV1::BlobDataOperationWriteV1
            && delivery.custody_transfer_source_proof.is_empty())
    {
        return Err(BlobClientError::InvalidResponse);
    }
    if let Some(target) = request.custody_target {
        let proof =
            BlobCustodySourceProofV1::decode(delivery.custody_transfer_source_proof.as_slice())
                .map_err(|_| BlobClientError::InvalidResponse)?;
        if proof.target_owner_id != target.owner_id
            || proof.target_module_id != target.module_id
            || proof.target_capability_id != target.capability_id
        {
            return Err(BlobClientError::InvalidResponse);
        }
    }
    Ok(ManagedBlobSessionV1 {
        data_socket_path: PathBuf::from(delivery.data_socket_path),
        grant,
        channel_binding,
        custody_transfer_source_proof: delivery.custody_transfer_source_proof,
    })
}

fn valid_custody_target(
    operation: BlobDataOperationV1,
    receipt_sha256: Option<&[u8; 32]>,
    target: Option<ManagedBlobCustodyTargetV1<'_>>,
) -> bool {
    let Some(target) = target else {
        return true;
    };
    operation == BlobDataOperationV1::BlobDataOperationWriteV1
        && receipt_sha256.is_some()
        && valid_target_token(target.owner_id)
        && valid_target_token(target.module_id)
        && valid_target_token(target.capability_id)
}

fn valid_target_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn valid_receipt_binding(
    operation: BlobDataOperationV1,
    receipt_sha256: Option<&[u8; 32]>,
) -> bool {
    receipt_sha256.is_none()
        || matches!(
            operation,
            BlobDataOperationV1::BlobDataOperationWriteV1
                | BlobDataOperationV1::BlobDataOperationReadRangeV1
        )
}

fn exact_receipt_binding(grant: &[u8], expected: Option<&[u8; 32]>) -> bool {
    expected.map_or_else(|| grant.is_empty(), |expected| grant == expected)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobDataClient {
    socket_path: PathBuf,
    timeout: Duration,
}

const DEFAULT_BLOB_DATA_TIMEOUT: Duration = Duration::from_secs(30);

impl BlobDataClient {
    pub fn new(socket_path: impl Into<PathBuf>) -> Result<Self, BlobClientError> {
        let socket_path = socket_path.into();
        if !socket_path.is_absolute() || socket_path.as_os_str().is_empty() {
            return Err(BlobClientError::InvalidSocketPath);
        }
        Ok(Self {
            socket_path,
            timeout: DEFAULT_BLOB_DATA_TIMEOUT,
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, BlobClientError> {
        if timeout.is_zero() {
            return Err(BlobClientError::InvalidTimeout);
        }
        self.timeout = timeout;
        Ok(self)
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn write(
        &self,
        grant: BlobDataSessionGrantV1,
        channel_binding: Vec<u8>,
        plaintext: Vec<u8>,
    ) -> Result<(), BlobClientError> {
        let response = self.request(BlobDataRequestV1 {
            grant: Some(grant),
            channel_binding,
            operation: Some(Operation::Write(BlobDataWriteRequestV1 { plaintext })),
        })?;
        if response.accepted {
            return Ok(());
        }
        Err(BlobClientError::Rejected(response.error_code))
    }

    pub fn read_range(
        &self,
        grant: BlobDataSessionGrantV1,
        channel_binding: Vec<u8>,
        start: u64,
        end_exclusive: u64,
    ) -> Result<Vec<u8>, BlobClientError> {
        let response = self.request(BlobDataRequestV1 {
            grant: Some(grant),
            channel_binding,
            operation: Some(Operation::ReadRange(BlobDataReadRangeRequestV1 {
                start,
                end_exclusive,
            })),
        })?;
        if response.accepted {
            return Ok(response.plaintext);
        }
        Err(BlobClientError::Rejected(response.error_code))
    }

    pub fn custody_transfer(
        &self,
        grant: BlobCustodyTransferGrantV1,
        channel_binding: Vec<u8>,
    ) -> Result<(), BlobClientError> {
        let response = self.request(BlobDataRequestV1 {
            grant: None,
            channel_binding: Vec::new(),
            operation: Some(Operation::CustodyTransfer(
                BlobDataCustodyTransferRequestV1 {
                    grant: Some(grant),
                    channel_binding,
                },
            )),
        })?;
        if response.accepted {
            return Ok(());
        }
        Err(BlobClientError::Rejected(response.error_code))
    }

    fn request(&self, request: BlobDataRequestV1) -> Result<BlobDataResponseV1, BlobClientError> {
        let bytes = request.encode_to_vec();
        if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
            return Err(BlobClientError::FrameTooLarge);
        }
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|error| BlobClientError::Connect(error.to_string()))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .and_then(|_| stream.set_write_timeout(Some(self.timeout)))
            .map_err(|error| BlobClientError::Io(error.to_string()))?;
        write_frame(&mut stream, &bytes)?;
        let response = read_frame(&mut stream)?;
        BlobDataResponseV1::decode(response.as_slice())
            .map_err(|_| BlobClientError::InvalidResponse)
    }
}

impl BlobReadPort for BlobDataClient {
    fn read_range(
        &mut self,
        grant: BlobDataSessionGrantV1,
        channel_binding: Vec<u8>,
        start: u64,
        end_exclusive: u64,
    ) -> Result<Vec<u8>, BlobReadError> {
        BlobDataClient::read_range(self, grant, channel_binding, start, end_exclusive).map_err(
            |error| match error {
                BlobClientError::Rejected(_) => BlobReadError::Rejected,
                BlobClientError::InvalidResponse => BlobReadError::InvalidResponse,
                _ => BlobReadError::Unavailable,
            },
        )
    }
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), BlobClientError> {
    let mut length = u32::try_from(bytes.len()).map_err(|_| BlobClientError::FrameTooLarge)?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    stream
        .write_all(&prefix)
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| BlobClientError::Io(error.to_string()))
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, BlobClientError> {
    let mut length = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| BlobClientError::Io(error.to_string()))?;
        length |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let length = usize::try_from(length).map_err(|_| BlobClientError::FrameTooLarge)?;
            if length == 0 || length > MAX_FRAME_BYTES {
                return Err(BlobClientError::FrameTooLarge);
            }
            let mut bytes = vec![0; length];
            stream
                .read_exact(&mut bytes)
                .map_err(|error| BlobClientError::Io(error.to_string()))?;
            return Ok(bytes);
        }
    }
    Err(BlobClientError::InvalidFrame)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlobClientError {
    InvalidSocketPath,
    InvalidTimeout,
    Connect(String),
    Io(String),
    FrameTooLarge,
    InvalidFrame,
    InvalidResponse,
    Rejected(String),
    InvalidSessionRequest,
    InvalidCustodyDelegationRequest,
    InvalidCustodyReleaseRequest,
    Unavailable,
}

fn managed_blob_session_error(error_code: &str, denied_code: &str) -> BlobClientError {
    if error_code == "managed_blob_session_unavailable" {
        BlobClientError::Unavailable
    } else {
        BlobClientError::Rejected(denied_code.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_path_must_be_absolute() {
        assert_eq!(
            BlobDataClient::new("relative.sock"),
            Err(BlobClientError::InvalidSocketPath)
        );
    }

    #[test]
    fn timeout_must_be_positive() {
        assert_eq!(
            BlobDataClient::new("/tmp/blob.sock")
                .expect("valid path")
                .with_timeout(Duration::ZERO),
            Err(BlobClientError::InvalidTimeout)
        );
    }

    #[test]
    fn default_timeout_covers_the_admitted_large_local_frame() {
        assert_eq!(
            BlobDataClient::new("/tmp/blob.sock")
                .expect("valid path")
                .timeout,
            Duration::from_secs(30)
        );
    }

    #[test]
    fn managed_session_unavailability_remains_retryable() {
        assert_eq!(
            managed_blob_session_error(
                "managed_blob_session_unavailable",
                "managed_blob_custody_transfer_denied",
            ),
            BlobClientError::Unavailable,
        );
        assert_eq!(
            managed_blob_session_error(
                "managed_blob_session_denied",
                "managed_blob_session_denied"
            ),
            BlobClientError::Rejected("managed_blob_session_denied".to_owned()),
        );
    }

    #[test]
    fn custody_release_response_is_exact_and_typed() {
        let response = ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::BlobCustodyRelease(
                makosh_runtime_protocol::v1::ManagedRuntimeBlobCustodyReleaseDeliveryV1 {
                    operation_id: vec![1; 16],
                    outcome: BlobCustodyReleaseOutcomeV1::BlobCustodyReleaseOutcomeExistingV1
                        as i32,
                    delete_not_before_unix_ms: 42,
                },
            )),
            error_code: String::new(),
        };
        assert_eq!(
            decode_custody_release_response(response.clone(), &[1; 16]),
            Ok(ManagedBlobCustodyReleaseV1 {
                operation_id: [1; 16],
                outcome: BlobCustodyReleaseOutcomeV1::BlobCustodyReleaseOutcomeExistingV1,
                delete_not_before_unix_ms: 42,
            })
        );
        assert_eq!(
            decode_custody_release_response(response, &[2; 16]),
            Err(BlobClientError::InvalidResponse)
        );
    }

    #[test]
    fn custody_delegation_response_is_lineage_and_target_bound() {
        let request_id = [1; 16];
        let reference_id = [2; 16];
        let predecessor = [3; 64];
        let predecessor_evidence_id = [4; 16];
        let predecessor_envelope_sha256 = [5; 32];
        let target = ManagedBlobCustodyTargetV1 {
            owner_id: "attachment_archive_inspection",
            module_id: "makosh-attachment-archive-inspection-runtime",
            capability_id: "attachment_archive_inspection.blob.v1",
        };
        let request = ManagedBlobCustodyDelegationRequestV1 {
            request_id: &request_id,
            capability_id: "attachment_security.blob.v1",
            current_reference_id: &reference_id,
            predecessor_custody_source_proof: &predecessor,
            predecessor_evidence_id: &predecessor_evidence_id,
            predecessor_evidence_envelope_sha256: &predecessor_envelope_sha256,
            target,
        };
        let proof = BlobCustodySourceProofV1 {
            proof_kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1 as i32,
            delegation_id: request_id.to_vec(),
            predecessor_proof_sha256: Sha256::digest(predecessor).to_vec(),
            reference_id: reference_id.to_vec(),
            target_owner_id: target.owner_id.to_owned(),
            target_module_id: target.module_id.to_owned(),
            target_capability_id: target.capability_id.to_owned(),
            ..Default::default()
        };
        let response = ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::BlobCustodyDelegation(
                makosh_runtime_protocol::v1::ManagedRuntimeBlobCustodyDelegationDeliveryV1 {
                    request_id: request_id.to_vec(),
                    custody_transfer_source_proof: proof.encode_to_vec(),
                    resolved_target_owner_id: target.owner_id.to_owned(),
                    resolved_target_module_id: target.module_id.to_owned(),
                    resolved_target_capability_id: target.capability_id.to_owned(),
                },
            )),
            error_code: String::new(),
        };
        let decoded =
            decode_custody_delegation_response(response.clone(), &request).expect("delegation");
        assert_eq!(decoded.request_id, request_id);

        let wrong_target = ManagedBlobCustodyDelegationRequestV1 {
            target: ManagedBlobCustodyTargetV1 {
                owner_id: "communications",
                module_id: "makosh-communications-runtime",
                capability_id: "communications.blob.v1",
            },
            ..request
        };
        assert_eq!(
            decode_custody_delegation_response(response, &wrong_target),
            Err(BlobClientError::InvalidResponse)
        );
    }

    #[test]
    fn resolved_provider_delegation_trusts_only_kernel_returned_exact_target() {
        let request_id = [1; 16];
        let reference_id = [2; 16];
        let predecessor = [3; 64];
        let evidence_id = [4; 16];
        let evidence_sha256 = [5; 32];
        let contract = ContractReferenceV1 {
            owner: "speech_to_text".to_owned(),
            name: "speech_to_text_provider_transcribe".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![6; 32],
        };
        let request = ManagedBlobResolvedProviderCustodyDelegationRequestV1 {
            request_id: &request_id,
            capability_id: "speech_to_text.blob.v1",
            current_reference_id: &reference_id,
            predecessor_custody_source_proof: &predecessor,
            predecessor_evidence_id: &evidence_id,
            predecessor_evidence_envelope_sha256: &evidence_sha256,
            target_request_contract: &contract,
        };
        let proof = BlobCustodySourceProofV1 {
            proof_kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1 as i32,
            delegation_id: request_id.to_vec(),
            predecessor_proof_sha256: Sha256::digest(predecessor).to_vec(),
            reference_id: reference_id.to_vec(),
            target_owner_id: "whisper_stt".to_owned(),
            target_module_id: "makosh-whisper-stt-runtime".to_owned(),
            target_capability_id: "speech_to_text.provider.v1".to_owned(),
            ..Default::default()
        };
        let response = ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::BlobCustodyDelegation(
                makosh_runtime_protocol::v1::ManagedRuntimeBlobCustodyDelegationDeliveryV1 {
                    request_id: request_id.to_vec(),
                    custody_transfer_source_proof: proof.encode_to_vec(),
                    resolved_target_owner_id: "whisper_stt".to_owned(),
                    resolved_target_module_id: "makosh-whisper-stt-runtime".to_owned(),
                    resolved_target_capability_id: "speech_to_text.provider.v1".to_owned(),
                },
            )),
            error_code: String::new(),
        };
        let decoded = decode_resolved_custody_delegation_response(response, &request)
            .expect("resolved delegation");
        assert_eq!(decoded.resolved_target_owner_id, "whisper_stt");
        assert_eq!(
            decoded.resolved_target_module_id,
            "makosh-whisper-stt-runtime"
        );
    }

    #[test]
    fn receipt_binding_is_allowed_only_for_exact_write_or_read_sessions() {
        let receipt = [7; 32];
        assert!(valid_receipt_binding(
            BlobDataOperationV1::BlobDataOperationWriteV1,
            Some(&receipt)
        ));
        assert!(valid_receipt_binding(
            BlobDataOperationV1::BlobDataOperationReadRangeV1,
            Some(&receipt)
        ));
        assert!(!valid_receipt_binding(
            BlobDataOperationV1::BlobDataOperationCustodyTransferV1,
            Some(&receipt)
        ));
        assert!(exact_receipt_binding(&receipt, Some(&receipt)));
        assert!(!exact_receipt_binding(&[8; 32], Some(&receipt)));
        assert!(exact_receipt_binding(&[], None));
    }

    #[test]
    fn custody_target_is_allowed_only_on_receipt_bound_writes() {
        let receipt = [7; 32];
        let target = ManagedBlobCustodyTargetV1 {
            owner_id: "attachment_security",
            module_id: "makosh-attachment-security-runtime",
            capability_id: "attachment_security.blob.v1",
        };
        assert!(valid_custody_target(
            BlobDataOperationV1::BlobDataOperationWriteV1,
            Some(&receipt),
            Some(target),
        ));
        assert!(!valid_custody_target(
            BlobDataOperationV1::BlobDataOperationReadRangeV1,
            Some(&receipt),
            Some(target),
        ));
        assert!(!valid_custody_target(
            BlobDataOperationV1::BlobDataOperationWriteV1,
            None,
            Some(target),
        ));
        assert!(!valid_target_token("Attachment Security"));
    }
}
