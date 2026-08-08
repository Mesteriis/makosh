//! Managed Blob adapter for delivery-intent bodies.

use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use makosh_communication_delivery_intent_core::CommunicationProviderProvenanceV1;
use makosh_communication_delivery_intent_persistence::DeliveryIntentBodyBlobReceiptV1;
use makosh_mail_delivery_intent_contract::{
    MAIL_DELIVERY_INTENT_OWNER_ID_V1, MAIL_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
    MAIL_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::BlobDataOperationV1,
};
use makosh_telegram_delivery_intent_contract::{
    TELEGRAM_DELIVERY_INTENT_OWNER_ID_V1, TELEGRAM_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
    TELEGRAM_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
};
use makosh_whatsapp_delivery_intent_contract::{
    WHATSAPP_DELIVERY_INTENT_OWNER_ID_V1, WHATSAPP_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
    WHATSAPP_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
};
use makosh_zulip_delivery_intent_contract::{
    ZULIP_DELIVERY_INTENT_OWNER_ID_V1, ZULIP_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
    ZULIP_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
};
use sha2::{Digest, Sha256};

use crate::{
    admission::COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1,
    coordinator::{DeliveryIntentBodyMaterializerV1, DeliveryIntentCoordinatorErrorV1},
};

pub struct ManagedDeliveryIntentBodyMaterializerV1<'a> {
    pub control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
}

impl DeliveryIntentBodyMaterializerV1 for ManagedDeliveryIntentBodyMaterializerV1<'_> {
    fn materialize_body(
        &mut self,
        logical_owner_id: &str,
        intent_id: [u8; 16],
        provider: CommunicationProviderProvenanceV1,
        body_utf8: &[u8],
    ) -> Result<DeliveryIntentBodyBlobReceiptV1, DeliveryIntentCoordinatorErrorV1> {
        let declared_bytes = u64::try_from(body_utf8.len())
            .ok()
            .filter(|size| *size > 0 && *size <= 64 * 1024)
            .ok_or(DeliveryIntentCoordinatorErrorV1::InvalidInput)?;
        let sha256: [u8; 32] = Sha256::digest(body_utf8).into();
        let reference_id = body_reference_id_v1(logical_owner_id, intent_id, sha256);
        if reference_id.iter().all(|byte| *byte == 0) {
            return Err(DeliveryIntentCoordinatorErrorV1::BlobUnavailable);
        }
        let target = provider_blob_target_v1(provider);
        let session = request_managed_blob_session_v2(
            self.control_channel,
            self.dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size: declared_bytes,
                backup_class: 1,
                receipt_sha256: Some(&sha256),
                custody_target: Some(target),
            },
        )
        .map_err(blob_error)?;
        let custody_transfer_source_proof = session.custody_transfer_source_proof;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.write(session.grant, session.channel_binding, body_utf8.to_vec())
            })
            .map_err(blob_error)?;
        Ok(DeliveryIntentBodyBlobReceiptV1 {
            reference_id,
            declared_bytes,
            sha256,
            custody_transfer_source_proof,
        })
    }
}

fn provider_blob_target_v1(
    provider: CommunicationProviderProvenanceV1,
) -> ManagedBlobCustodyTargetV1<'static> {
    match provider {
        CommunicationProviderProvenanceV1::MailImap
        | CommunicationProviderProvenanceV1::MailSmtp
        | CommunicationProviderProvenanceV1::MailGmail => ManagedBlobCustodyTargetV1 {
            owner_id: MAIL_DELIVERY_INTENT_OWNER_ID_V1,
            module_id: MAIL_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
            capability_id: MAIL_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
        },
        CommunicationProviderProvenanceV1::Telegram => ManagedBlobCustodyTargetV1 {
            owner_id: TELEGRAM_DELIVERY_INTENT_OWNER_ID_V1,
            module_id: TELEGRAM_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
            capability_id: TELEGRAM_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
        },
        CommunicationProviderProvenanceV1::WhatsAppWeb => ManagedBlobCustodyTargetV1 {
            owner_id: WHATSAPP_DELIVERY_INTENT_OWNER_ID_V1,
            module_id: WHATSAPP_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
            capability_id: WHATSAPP_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
        },
        CommunicationProviderProvenanceV1::Zulip => ManagedBlobCustodyTargetV1 {
            owner_id: ZULIP_DELIVERY_INTENT_OWNER_ID_V1,
            module_id: ZULIP_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
            capability_id: ZULIP_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
        },
    }
}

fn body_reference_id_v1(
    logical_owner_id: &str,
    intent_id: [u8; 16],
    body_sha256: [u8; 32],
) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication-delivery-intent.body.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(intent_id);
    digest.update(body_sha256);
    digest.finalize()[..16].try_into().expect("fixed digest")
}

fn blob_error(_: BlobClientError) -> DeliveryIntentCoordinatorErrorV1 {
    DeliveryIntentCoordinatorErrorV1::BlobUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_are_exact_provider_public_contracts() {
        let mail = provider_blob_target_v1(CommunicationProviderProvenanceV1::MailSmtp);
        assert_eq!(mail.owner_id, "mail");
        assert_eq!(mail.module_id, "makosh-mail-runtime");
        assert_eq!(mail.capability_id, "mail.blob.v1");

        let telegram = provider_blob_target_v1(CommunicationProviderProvenanceV1::Telegram);
        assert_eq!(telegram.owner_id, "telegram");
        assert_eq!(telegram.capability_id, "telegram.blob.v1");

        let whatsapp = provider_blob_target_v1(CommunicationProviderProvenanceV1::WhatsAppWeb);
        assert_eq!(whatsapp.owner_id, "whatsapp");
        assert_eq!(whatsapp.capability_id, "whatsapp.blob.v1");

        let zulip = provider_blob_target_v1(CommunicationProviderProvenanceV1::Zulip);
        assert_eq!(zulip.owner_id, "zulip");
        assert_eq!(zulip.capability_id, "zulip.blob.v1");
    }

    #[test]
    fn body_reference_is_owner_intent_and_digest_bound() {
        let first = body_reference_id_v1("owner:a", [1; 16], [2; 32]);
        assert_eq!(first, body_reference_id_v1("owner:a", [1; 16], [2; 32]));
        assert_ne!(first, body_reference_id_v1("owner:b", [1; 16], [2; 32]));
        assert_ne!(first, body_reference_id_v1("owner:a", [1; 16], [3; 32]));
    }
}
