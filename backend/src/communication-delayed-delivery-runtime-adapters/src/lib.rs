#![forbid(unsafe_code)]

use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyReleaseRequestV1,
    ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_session_v2,
};
use makosh_communication_delayed_delivery_api::{
    COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1, COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
};
use makosh_communication_delayed_delivery_execution::{
    BodyCleanupErrorV1, BodyCleanupPortV1, BodyCleanupReasonV1, BodyReadErrorV1, BodyReadPortV1,
    DelayedDeliveryBodyCleanupJobV1, DelayedDeliveryExecutionClaimV1, DeliveryIntentRequestErrorV1,
    DeliveryIntentRequestPortV1,
};
use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1, COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
    COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        BlobCustodyReleaseReasonV1, BlobDataOperationV1, ContractReferenceV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeModuleRequestRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        validate_module_request_request_v1, validate_module_request_response_v1,
    },
};
use sha2::{Digest, Sha256};

const REQUIRED_BACKUP_CLASS_V1: u32 = 1;
const DELIVERY_REQUEST_DEADLINE_MILLIS_V1: u32 = 30_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedDelayedDeliveryRuntimePortErrorV1 {
    InvalidCapability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryCustodyErrorV1 {
    InvalidInput,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryBodyCustodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

pub struct ManagedDelayedDeliveryRuntimePortV1<'a> {
    channel: &'a mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    blob_capability_id: &'a str,
}

impl<'a> ManagedDelayedDeliveryRuntimePortV1<'a> {
    pub fn new(
        channel: &'a mut ManagedControlChannelV2<UnixStream>,
        dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        blob_capability_id: &'a str,
    ) -> Result<Self, ManagedDelayedDeliveryRuntimePortErrorV1> {
        if blob_capability_id.trim().is_empty()
            || blob_capability_id.len() > 128
            || !blob_capability_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        {
            return Err(ManagedDelayedDeliveryRuntimePortErrorV1::InvalidCapability);
        }
        Ok(Self {
            channel,
            dispatcher,
            blob_capability_id,
        })
    }

    pub fn materialize_body(
        &mut self,
        logical_owner_id: &str,
        delayed_operation_id: [u8; 16],
        body_utf8: &[u8],
    ) -> Result<DelayedDeliveryBodyCustodyReceiptV1, DelayedDeliveryCustodyErrorV1> {
        let declared_bytes = u64::try_from(body_utf8.len())
            .ok()
            .filter(|size| *size > 0 && *size <= 64 * 1024)
            .ok_or(DelayedDeliveryCustodyErrorV1::InvalidInput)?;
        if logical_owner_id.is_empty()
            || logical_owner_id.len() > 128
            || delayed_operation_id.iter().all(|byte| *byte == 0)
        {
            return Err(DelayedDeliveryCustodyErrorV1::InvalidInput);
        }
        let sha256: [u8; 32] = Sha256::digest(body_utf8).into();
        let reference_id = body_reference_id(logical_owner_id, delayed_operation_id, sha256);
        let session = request_managed_blob_session_v2(
            self.channel,
            self.dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: self.blob_capability_id,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size: declared_bytes,
                backup_class: REQUIRED_BACKUP_CLASS_V1,
                receipt_sha256: Some(&sha256),
                custody_target: Some(ManagedBlobCustodyTargetV1 {
                    owner_id: COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
                    module_id: COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1,
                    capability_id: self.blob_capability_id,
                }),
            },
        )
        .map_err(|_| DelayedDeliveryCustodyErrorV1::Unavailable)?;
        let custody_proof = session.custody_transfer_source_proof;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.write(session.grant, session.channel_binding, body_utf8.to_vec())
            })
            .map_err(|_| DelayedDeliveryCustodyErrorV1::Unavailable)?;
        Ok(DelayedDeliveryBodyCustodyReceiptV1 {
            reference_id,
            declared_bytes,
            sha256,
            custody_proof,
        })
    }
}

fn body_reference_id(
    logical_owner_id: &str,
    delayed_operation_id: [u8; 16],
    body_sha256: [u8; 32],
) -> [u8; 16] {
    let digest = Sha256::new()
        .chain_update(b"makosh.communication-delayed-delivery.body.v1\0")
        .chain_update((logical_owner_id.len() as u64).to_be_bytes())
        .chain_update(logical_owner_id.as_bytes())
        .chain_update(delayed_operation_id)
        .chain_update(body_sha256)
        .finalize();
    digest[..16].try_into().expect("SHA-256 prefix is exact")
}

impl BodyReadPortV1 for ManagedDelayedDeliveryRuntimePortV1<'_> {
    async fn read_once(
        &mut self,
        claim: &DelayedDeliveryExecutionClaimV1,
    ) -> Result<Vec<u8>, BodyReadErrorV1> {
        let session = request_managed_blob_session_v2(
            self.channel,
            self.dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: self.blob_capability_id,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &claim.body_receipt.reference_id,
                declared_size: claim.body_receipt.declared_bytes,
                backup_class: REQUIRED_BACKUP_CLASS_V1,
                receipt_sha256: Some(&claim.body_receipt.sha256),
                custody_target: None,
            },
        )
        .map_err(body_read_error)?;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(
                    session.grant,
                    session.channel_binding,
                    0,
                    claim.body_receipt.declared_bytes,
                )
            })
            .map_err(body_read_error)
    }
}

impl DeliveryIntentRequestPortV1 for ManagedDelayedDeliveryRuntimePortV1<'_> {
    async fn request(
        &mut self,
        request_id: [u8; 16],
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, DeliveryIntentRequestErrorV1> {
        let request = ManagedRuntimeModuleRequestRequestV1 {
            request_id: request_id.to_vec(),
            contract: Some(delivery_intent_command_contract_v1()),
            request_payload: payload,
            deadline_millis: DELIVERY_REQUEST_DEADLINE_MILLIS_V1,
            response_blob_capability_id: String::new(),
        };
        validate_module_request_request_v1(&request)
            .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
        let response = self
            .channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteModuleRequest(request)),
                },
                self.dispatcher,
            )
            .map_err(|_| DeliveryIntentRequestErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(DeliveryIntentRequestErrorV1::Unavailable);
        }
        let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
            return Err(DeliveryIntentRequestErrorV1::Protocol);
        };
        validate_module_request_response_v1(&response)
            .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
        if response.request_id != request_id {
            return Err(DeliveryIntentRequestErrorV1::Protocol);
        }
        if !response.error_code.is_empty() {
            return Err(DeliveryIntentRequestErrorV1::Unavailable);
        }
        Ok(response.response_payload)
    }
}

impl BodyCleanupPortV1 for ManagedDelayedDeliveryRuntimePortV1<'_> {
    async fn request_cleanup(
        &mut self,
        job: &DelayedDeliveryBodyCleanupJobV1,
    ) -> Result<(), BodyCleanupErrorV1> {
        let reason = match job.reason {
            BodyCleanupReasonV1::DeliveryAccepted => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
            }
            BodyCleanupReasonV1::DeliveryRejected => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
            }
            BodyCleanupReasonV1::DeliveryCancelled => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalCancelledV1
            }
        };
        request_managed_blob_custody_release_v2(
            self.channel,
            self.dispatcher,
            ManagedBlobCustodyReleaseRequestV1 {
                operation_id: &job.delayed_operation_id,
                capability_id: self.blob_capability_id,
                reference_id: &job.body_receipt.reference_id,
                declared_size: job.body_receipt.declared_bytes,
                receipt_sha256: &job.body_receipt.sha256,
                custody_source_proof: &job.body_receipt.custody_proof,
                reason,
            },
        )
        .map(|_| ())
        .map_err(|_| BodyCleanupErrorV1::Unavailable)
    }
}

fn delivery_intent_command_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
        name: COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1.to_owned(),
        major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
    }
}

fn body_read_error(error: BlobClientError) -> BodyReadErrorV1 {
    match error {
        BlobClientError::InvalidSessionRequest
        | BlobClientError::InvalidCustodyDelegationRequest
        | BlobClientError::InvalidCustodyReleaseRequest
        | BlobClientError::InvalidSocketPath
        | BlobClientError::InvalidTimeout => BodyReadErrorV1::InvalidReceipt,
        BlobClientError::Rejected(_)
        | BlobClientError::InvalidFrame
        | BlobClientError::InvalidResponse
        | BlobClientError::FrameTooLarge => BodyReadErrorV1::Denied,
        BlobClientError::Connect(_) | BlobClientError::Io(_) | BlobClientError::Unavailable => {
            BodyReadErrorV1::Unavailable
        }
    }
}

pub const PACKAGE: &str = "makosh-communication-delayed-delivery-runtime-adapters";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_contract_is_exact_and_provider_neutral() {
        let contract = delivery_intent_command_contract_v1();
        assert_eq!(contract.owner, "communication_delivery_intent");
        assert_eq!(contract.name, "communication.delivery_intent.command");
        assert_eq!(contract.schema_sha256.len(), 32);
    }

    #[test]
    fn invalid_blob_inputs_never_become_retryable_transport_failures() {
        assert_eq!(
            body_read_error(BlobClientError::InvalidSessionRequest),
            BodyReadErrorV1::InvalidReceipt
        );
        assert_eq!(
            body_read_error(BlobClientError::Unavailable),
            BodyReadErrorV1::Unavailable
        );
    }

    #[test]
    fn custody_reference_binds_owner_operation_and_body_digest() {
        let digest = [3; 32];
        let reference = body_reference_id("owner-1", [1; 16], digest);
        assert_eq!(reference, body_reference_id("owner-1", [1; 16], digest));
        assert_ne!(reference, body_reference_id("owner-2", [1; 16], digest));
        assert_ne!(reference, body_reference_id("owner-1", [2; 16], digest));
        assert_ne!(reference, body_reference_id("owner-1", [1; 16], [4; 32]));
    }
}
