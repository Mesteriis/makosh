use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_transfer_v2, request_managed_blob_session_v2,
};
use makosh_communication_delivery_intent_ingress_api::COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_PROOF_BYTES_V1;
use makosh_communication_delivery_intent_persistence::DeliveryIntentIngressBlobReceiptV1;
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::BlobDataOperationV1,
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::admission::COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1;

const MAX_DELIVERY_BODY_BYTES_V1: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentIngressBodyErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub fn read_delivery_intent_ingress_body_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    receipt: &DeliveryIntentIngressBlobReceiptV1,
    command_message_id: &[u8; 16],
    envelope_sha256: &[u8; 32],
) -> Result<
    (Zeroizing<Vec<u8>>, DeliveryIntentIngressBlobReceiptV1),
    DeliveryIntentIngressBodyErrorV1,
> {
    validate_receipt(receipt)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        control_channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &receipt.reference_id,
            declared_size: receipt.declared_bytes,
            receipt_sha256: &receipt.sha256,
            custody_source_proof: &receipt.custody_source_proof,
            evidence_id: command_message_id,
            evidence_envelope_sha256: envelope_sha256,
        },
    )
    .map_err(|_| DeliveryIntentIngressBodyErrorV1::Unavailable)?;
    let reference_id: [u8; 16] = transfer
        .grant
        .target_reference_id
        .as_slice()
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(DeliveryIntentIngressBodyErrorV1::InvalidReceipt)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| DeliveryIntentIngressBodyErrorV1::Unavailable)?;
    let sha256 = receipt.sha256;
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &reference_id,
            declared_size: receipt.declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: None,
        },
    )
    .map_err(|_| DeliveryIntentIngressBodyErrorV1::Unavailable)?;
    let bytes = BlobDataClient::new(session.data_socket_path)
        .and_then(|client| {
            client.read_range(
                session.grant,
                session.channel_binding,
                0,
                receipt.declared_bytes,
            )
        })
        .map_err(|_| DeliveryIntentIngressBodyErrorV1::Unavailable)?;
    if bytes.len() != usize::try_from(receipt.declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(&bytes).as_slice() != sha256
        || std::str::from_utf8(&bytes).is_err()
    {
        return Err(DeliveryIntentIngressBodyErrorV1::InvalidReceipt);
    }
    Ok((
        Zeroizing::new(bytes),
        DeliveryIntentIngressBlobReceiptV1 {
            reference_id,
            declared_bytes: receipt.declared_bytes,
            sha256,
            custody_source_proof: receipt.custody_source_proof.clone(),
        },
    ))
}

fn validate_receipt(
    receipt: &DeliveryIntentIngressBlobReceiptV1,
) -> Result<(), DeliveryIntentIngressBodyErrorV1> {
    if receipt.reference_id.len() != 16
        || receipt.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=MAX_DELIVERY_BODY_BYTES_V1).contains(&receipt.declared_bytes)
        || receipt.sha256.len() != 32
        || receipt.sha256.iter().all(|byte| *byte == 0)
        || receipt.custody_source_proof.is_empty()
        || receipt.custody_source_proof.len()
            > COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_PROOF_BYTES_V1
    {
        return Err(DeliveryIntentIngressBodyErrorV1::InvalidReceipt);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingress_receipt_is_bounded_and_requires_custody_proof() {
        let receipt = DeliveryIntentIngressBlobReceiptV1 {
            reference_id: [1; 16],
            declared_bytes: 42,
            sha256: [2; 32],
            custody_source_proof: vec![3; 64],
        };
        assert_eq!(validate_receipt(&receipt), Ok(()));
        assert_eq!(
            validate_receipt(&DeliveryIntentIngressBlobReceiptV1 {
                custody_source_proof: Vec::new(),
                ..receipt
            }),
            Err(DeliveryIntentIngressBodyErrorV1::InvalidReceipt)
        );
    }
}
