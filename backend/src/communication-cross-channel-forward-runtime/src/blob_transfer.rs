use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobCustodyTransferRequestV1,
    ManagedBlobSessionRequestV1, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_communication_cross_channel_forward_persistence::{
    CrossChannelForwardBlobReceiptV1, CrossChannelForwardPreparedEventV1,
};
use makosh_communication_delivery_intent_ingress_api::{
    COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_CAPABILITY_ID_V1,
    COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1,
    COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_OWNER_ID_V1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::BlobDataOperationV1,
};
use sha2::{Digest, Sha256};

use crate::COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1;

const MAX_BODY_BYTES_V1: u64 = 64 * 1024;
const MAX_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardBlobTransferErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub struct CrossChannelForwardBlobMaterializationV1 {
    pub source_body: CrossChannelForwardBlobReceiptV1,
    pub delivery_body: CrossChannelForwardBlobReceiptV1,
}

pub trait CrossChannelForwardBlobPortV1 {
    fn transfer_to_delivery_intent(
        &mut self,
        prepared: &CrossChannelForwardPreparedEventV1,
    ) -> Result<CrossChannelForwardBlobMaterializationV1, CrossChannelForwardBlobTransferErrorV1>;
}

pub struct ManagedCrossChannelForwardBlobPortV1<'a> {
    pub control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
}

impl CrossChannelForwardBlobPortV1 for ManagedCrossChannelForwardBlobPortV1<'_> {
    fn transfer_to_delivery_intent(
        &mut self,
        prepared: &CrossChannelForwardPreparedEventV1,
    ) -> Result<CrossChannelForwardBlobMaterializationV1, CrossChannelForwardBlobTransferErrorV1>
    {
        validate_source_receipt(&prepared.source_body)?;
        let transfer = request_managed_blob_custody_transfer_v2(
            self.control_channel,
            self.dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1,
                source_reference_id: &prepared.source_body.reference_id,
                declared_size: prepared.source_body.declared_bytes,
                receipt_sha256: &prepared.source_body.sha256,
                custody_source_proof: &prepared.source_body.custody_transfer_source_proof,
                evidence_id: &prepared.result_message_id,
                evidence_envelope_sha256: &prepared.envelope_sha256,
            },
        )
        .map_err(|_| CrossChannelForwardBlobTransferErrorV1::Unavailable)?;
        let source_reference_id: [u8; 16] = transfer
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .ok()
            .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
            .ok_or(CrossChannelForwardBlobTransferErrorV1::InvalidReceipt)?;
        BlobDataClient::new(&transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(|_| CrossChannelForwardBlobTransferErrorV1::Unavailable)?;
        let read = request_managed_blob_session_v2(
            self.control_channel,
            self.dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &source_reference_id,
                declared_size: prepared.source_body.declared_bytes,
                backup_class: 1,
                receipt_sha256: Some(&prepared.source_body.sha256),
                custody_target: None,
            },
        )
        .map_err(|_| CrossChannelForwardBlobTransferErrorV1::Unavailable)?;
        let bytes = BlobDataClient::new(read.data_socket_path)
            .and_then(|client| {
                client.read_range(
                    read.grant,
                    read.channel_binding,
                    0,
                    prepared.source_body.declared_bytes,
                )
            })
            .map_err(|_| CrossChannelForwardBlobTransferErrorV1::Unavailable)?;
        if bytes.len() != usize::try_from(prepared.source_body.declared_bytes).unwrap_or(usize::MAX)
            || Sha256::digest(&bytes).as_slice() != prepared.source_body.sha256
            || std::str::from_utf8(&bytes).is_err()
        {
            return Err(CrossChannelForwardBlobTransferErrorV1::InvalidReceipt);
        }

        let reference_id = delivery_reference_id(prepared);
        let write = request_managed_blob_session_v2(
            self.control_channel,
            self.dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size: prepared.source_body.declared_bytes,
                backup_class: 1,
                receipt_sha256: Some(&prepared.source_body.sha256),
                custody_target: Some(ManagedBlobCustodyTargetV1 {
                    owner_id: COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_OWNER_ID_V1,
                    module_id: COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1,
                    capability_id: COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_CAPABILITY_ID_V1,
                }),
            },
        )
        .map_err(|_| CrossChannelForwardBlobTransferErrorV1::Unavailable)?;
        let proof = write.custody_transfer_source_proof;
        if proof.is_empty() || proof.len() > MAX_PROOF_BYTES_V1 {
            return Err(CrossChannelForwardBlobTransferErrorV1::InvalidReceipt);
        }
        BlobDataClient::new(write.data_socket_path)
            .and_then(|client| client.write(write.grant, write.channel_binding, bytes))
            .map_err(|_| CrossChannelForwardBlobTransferErrorV1::Unavailable)?;
        Ok(CrossChannelForwardBlobMaterializationV1 {
            source_body: CrossChannelForwardBlobReceiptV1 {
                reference_id: source_reference_id,
                declared_bytes: prepared.source_body.declared_bytes,
                sha256: prepared.source_body.sha256,
                custody_transfer_source_proof: prepared
                    .source_body
                    .custody_transfer_source_proof
                    .clone(),
            },
            delivery_body: CrossChannelForwardBlobReceiptV1 {
                reference_id,
                declared_bytes: prepared.source_body.declared_bytes,
                sha256: prepared.source_body.sha256,
                custody_transfer_source_proof: proof,
            },
        })
    }
}

fn validate_source_receipt(
    receipt: &CrossChannelForwardBlobReceiptV1,
) -> Result<(), CrossChannelForwardBlobTransferErrorV1> {
    if receipt.reference_id.iter().all(|byte| *byte == 0)
        || receipt.declared_bytes == 0
        || receipt.declared_bytes > MAX_BODY_BYTES_V1
        || receipt.sha256.iter().all(|byte| *byte == 0)
        || receipt.custody_transfer_source_proof.is_empty()
        || receipt.custody_transfer_source_proof.len() > MAX_PROOF_BYTES_V1
    {
        return Err(CrossChannelForwardBlobTransferErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn delivery_reference_id(prepared: &CrossChannelForwardPreparedEventV1) -> [u8; 16] {
    let digest = Sha256::digest(
        [
            b"cross-channel-delivery-intent-body-v1".as_slice(),
            &prepared.forward_id,
            &prepared.source_body.reference_id,
            &prepared.source_body.sha256,
        ]
        .concat(),
    );
    digest[..16].try_into().expect("SHA-256 prefix is 16 bytes")
}

#[cfg(test)]
mod tests {
    use super::delivery_reference_id;
    use makosh_communication_cross_channel_forward_persistence::{
        CrossChannelForwardBlobReceiptV1, CrossChannelForwardPreparedEventV1,
    };

    fn prepared() -> CrossChannelForwardPreparedEventV1 {
        CrossChannelForwardPreparedEventV1 {
            result_message_id: [1; 16],
            envelope_sha256: [2; 32],
            logical_owner_id: "owner-1".to_owned(),
            forward_id: [3; 16],
            source_message_id: [4; 16],
            target_conversation_id: [5; 16],
            source_evidence_id: [6; 16],
            source_evidence_revision: 1,
            source_body: CrossChannelForwardBlobReceiptV1 {
                reference_id: [7; 16],
                declared_bytes: 5,
                sha256: [8; 32],
                custody_transfer_source_proof: vec![9; 48],
            },
        }
    }

    #[test]
    fn delivery_reference_is_deterministic_and_source_bound() {
        let value = prepared();
        assert_eq!(delivery_reference_id(&value), delivery_reference_id(&value));
        let mut changed = value;
        changed.source_body.sha256 = [10; 32];
        assert_ne!(
            delivery_reference_id(&changed),
            delivery_reference_id(&prepared())
        );
    }
}
