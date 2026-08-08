//! Plaintext-to-Blob persistence boundary owned by the workflow runtime.

use makosh_communication_delivery_intent_core::{
    CommunicationProviderProvenanceV1, PlannedDeliveryIntentV1,
};
use makosh_communication_delivery_intent_persistence::{
    CreateDeliveryIntentV1, DeliveryIntentBodyBlobReceiptV1,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentCoordinatorErrorV1 {
    InvalidInput,
    BlobUnavailable,
}

pub trait DeliveryIntentBodyMaterializerV1 {
    fn materialize_body(
        &mut self,
        logical_owner_id: &str,
        intent_id: [u8; 16],
        provider: CommunicationProviderProvenanceV1,
        body_utf8: &[u8],
    ) -> Result<DeliveryIntentBodyBlobReceiptV1, DeliveryIntentCoordinatorErrorV1>;
}

pub fn prepare_create_delivery_intent_v1<M: DeliveryIntentBodyMaterializerV1>(
    logical_owner_id: String,
    planned: PlannedDeliveryIntentV1,
    created_at_unix_seconds: i64,
    materializer: &mut M,
) -> Result<CreateDeliveryIntentV1, DeliveryIntentCoordinatorErrorV1> {
    if !valid_logical_owner_id(&logical_owner_id) || created_at_unix_seconds <= 0 {
        return Err(DeliveryIntentCoordinatorErrorV1::InvalidInput);
    }
    let body_receipt = materializer.materialize_body(
        &logical_owner_id,
        planned.intent_id,
        planned.route.provider,
        planned.body.as_bytes(),
    )?;
    let request_fingerprint = request_fingerprint_v1(&logical_owner_id, &planned, &body_receipt);
    Ok(CreateDeliveryIntentV1 {
        logical_owner_id,
        intent_id: planned.intent_id,
        canonical_conversation_id: planned.canonical_conversation_id,
        canonical_reply_message_id: planned.canonical_reply_to_message_id,
        route: planned.route,
        body_receipt,
        request_fingerprint,
        created_at_unix_seconds,
    })
}

fn request_fingerprint_v1(
    logical_owner_id: &str,
    planned: &PlannedDeliveryIntentV1,
    body: &DeliveryIntentBodyBlobReceiptV1,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication-delivery-intent.request.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(planned.intent_id);
    digest.update(planned.canonical_conversation_id.bytes());
    if let Some(reply_id) = planned.canonical_reply_to_message_id {
        digest.update([1]);
        digest.update(reply_id.bytes());
    } else {
        digest.update([0]);
    }
    digest.update([provider_code(planned.route.provider)]);
    digest.update(planned.route.account_cursor.bytes());
    digest.update(planned.route.conversation_cursor.bytes());
    if let Some(cursor) = planned.route.reply_to_source_cursor {
        digest.update([1]);
        digest.update(cursor.bytes());
    } else {
        digest.update([0]);
    }
    digest.update(body.reference_id);
    digest.update(body.declared_bytes.to_be_bytes());
    digest.update(body.sha256);
    digest.finalize().into()
}

const fn provider_code(provider: CommunicationProviderProvenanceV1) -> u8 {
    match provider {
        CommunicationProviderProvenanceV1::MailImap => 1,
        CommunicationProviderProvenanceV1::Telegram => 2,
        CommunicationProviderProvenanceV1::WhatsAppWeb => 3,
        CommunicationProviderProvenanceV1::MailSmtp => 4,
        CommunicationProviderProvenanceV1::Zulip => 5,
        CommunicationProviderProvenanceV1::MailGmail => 6,
    }
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use makosh_communication_delivery_intent_core::{
        CommunicationConversationIdV1, CommunicationDeliveryRouteV1, CommunicationSourceCursorV1,
        ValidatedDeliveryBodyV1,
    };

    use super::*;

    struct RecordingMaterializer {
        observed_body: Vec<u8>,
    }

    impl DeliveryIntentBodyMaterializerV1 for RecordingMaterializer {
        fn materialize_body(
            &mut self,
            _logical_owner_id: &str,
            _intent_id: [u8; 16],
            provider: CommunicationProviderProvenanceV1,
            body_utf8: &[u8],
        ) -> Result<DeliveryIntentBodyBlobReceiptV1, DeliveryIntentCoordinatorErrorV1> {
            assert_eq!(provider, CommunicationProviderProvenanceV1::Telegram);
            self.observed_body = body_utf8.to_vec();
            Ok(DeliveryIntentBodyBlobReceiptV1 {
                reference_id: [9; 16],
                declared_bytes: body_utf8.len() as u64,
                sha256: Sha256::digest(body_utf8).into(),
                custody_transfer_source_proof: vec![8; 48],
            })
        }
    }

    fn planned() -> PlannedDeliveryIntentV1 {
        PlannedDeliveryIntentV1 {
            intent_id: [1; 16],
            canonical_conversation_id: CommunicationConversationIdV1::new([2; 16]),
            canonical_reply_to_message_id: None,
            route: CommunicationDeliveryRouteV1 {
                provider: CommunicationProviderProvenanceV1::Telegram,
                account_cursor: CommunicationSourceCursorV1::new([3; 32]),
                conversation_cursor: CommunicationSourceCursorV1::new([4; 32]),
                reply_to_source_cursor: None,
            },
            body: ValidatedDeliveryBodyV1::try_from(b"private body".to_vec()).expect("body"),
        }
    }

    #[test]
    fn plaintext_is_materialized_and_only_blob_receipt_reaches_persistence() {
        let mut materializer = RecordingMaterializer {
            observed_body: Vec::new(),
        };
        let command = prepare_create_delivery_intent_v1(
            "owner:test".to_owned(),
            planned(),
            10,
            &mut materializer,
        )
        .expect("materialized command");
        assert_eq!(materializer.observed_body, b"private body");
        assert_eq!(command.body_receipt.reference_id, [9; 16]);
        assert_eq!(command.intent_id, [1; 16]);
        assert!(command.request_fingerprint.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn invalid_owner_or_time_never_reaches_blob() {
        let mut materializer = RecordingMaterializer {
            observed_body: Vec::new(),
        };
        assert!(matches!(
            prepare_create_delivery_intent_v1(
                "owner invalid".to_owned(),
                planned(),
                10,
                &mut materializer
            ),
            Err(DeliveryIntentCoordinatorErrorV1::InvalidInput)
        ));
        assert!(materializer.observed_body.is_empty());
    }
}
