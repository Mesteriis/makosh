use makosh_communication_delivery_intent_core::CommunicationProviderProvenanceV1;
use makosh_communication_delivery_intent_event_adapters::{
    DecodedDeliveryIntentTerminalV1, DeliveryIntentBodySourceV1, DeliveryIntentCommandContextV1,
    DeliveryIntentEventAdapterErrorV1, DeliveryIntentTerminalOutcomeV1, mail, telegram, whatsapp,
    zulip,
};
use makosh_communication_delivery_intent_persistence::{
    ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentClaimV1, DeliveryIntentPersistenceErrorV1,
    DeliveryIntentStatusRecordV1, EnqueueProviderCommandOutcomeV1, TerminalDeliveryResultV1,
    TerminalDeliveryResultValueV1,
};

use crate::runtime::DeliveryIntentManagedRuntimeV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentProviderEventRuntimeErrorV1 {
    WrongProvider,
    Adapter(DeliveryIntentEventAdapterErrorV1),
    Persistence(DeliveryIntentPersistenceErrorV1),
}

impl DeliveryIntentManagedRuntimeV1 {
    pub async fn enqueue_mail_command_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        context: &DeliveryIntentCommandContextV1,
        now_unix_seconds: i64,
    ) -> Result<EnqueueProviderCommandOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        if !matches!(
            claim.route.provider,
            CommunicationProviderProvenanceV1::MailImap
                | CommunicationProviderProvenanceV1::MailSmtp
                | CommunicationProviderProvenanceV1::MailGmail
        ) {
            return Err(DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider);
        }
        let record = mail::build_execute_outbox_v1(
            claim.intent_id,
            &claim.logical_owner_id,
            claim.route.account_cursor.bytes(),
            claim.route.conversation_cursor.bytes(),
            claim
                .route
                .reply_to_source_cursor
                .map(|cursor| cursor.bytes()),
            &body_source_v1(claim),
            context,
        )
        .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?;
        self.persistence()
            .enqueue_provider_command(claim, claim.route.provider, &record, now_unix_seconds)
            .await
            .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Persistence)
    }

    pub async fn enqueue_telegram_command_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        context: &DeliveryIntentCommandContextV1,
        now_unix_seconds: i64,
    ) -> Result<EnqueueProviderCommandOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        if claim.route.provider != CommunicationProviderProvenanceV1::Telegram {
            return Err(DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider);
        }
        let record = telegram::build_execute_outbox_v1(
            claim.intent_id,
            &claim.logical_owner_id,
            claim.route.account_cursor.bytes(),
            claim.route.conversation_cursor.bytes(),
            claim
                .route
                .reply_to_source_cursor
                .map(|cursor| cursor.bytes()),
            &body_source_v1(claim),
            context,
        )
        .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?;
        self.persistence()
            .enqueue_provider_command(claim, claim.route.provider, &record, now_unix_seconds)
            .await
            .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Persistence)
    }

    pub async fn enqueue_whatsapp_command_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        context: &DeliveryIntentCommandContextV1,
        now_unix_seconds: i64,
    ) -> Result<EnqueueProviderCommandOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        if claim.route.provider != CommunicationProviderProvenanceV1::WhatsAppWeb {
            return Err(DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider);
        }
        let record = whatsapp::build_execute_outbox_v1(
            claim.intent_id,
            &claim.logical_owner_id,
            claim.route.account_cursor.bytes(),
            claim.route.conversation_cursor.bytes(),
            claim
                .route
                .reply_to_source_cursor
                .map(|cursor| cursor.bytes()),
            &body_source_v1(claim),
            context,
        )
        .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?;
        self.persistence()
            .enqueue_provider_command(claim, claim.route.provider, &record, now_unix_seconds)
            .await
            .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Persistence)
    }

    pub async fn enqueue_zulip_command_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        context: &DeliveryIntentCommandContextV1,
        now_unix_seconds: i64,
    ) -> Result<EnqueueProviderCommandOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        if claim.route.provider != CommunicationProviderProvenanceV1::Zulip {
            return Err(DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider);
        }
        let record = zulip::build_execute_outbox_v1(
            claim.intent_id,
            &claim.logical_owner_id,
            claim.route.account_cursor.bytes(),
            claim.route.conversation_cursor.bytes(),
            claim
                .route
                .reply_to_source_cursor
                .map(|cursor| cursor.bytes()),
            &body_source_v1(claim),
            context,
        )
        .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?;
        self.persistence()
            .enqueue_provider_command(claim, claim.route.provider, &record, now_unix_seconds)
            .await
            .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Persistence)
    }

    pub async fn complete_mail_publish_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        if !matches!(
            claim.route.provider,
            CommunicationProviderProvenanceV1::MailImap
                | CommunicationProviderProvenanceV1::MailSmtp
                | CommunicationProviderProvenanceV1::MailGmail
        ) {
            return Err(DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider);
        }
        self.complete_publish_v1(claim, message_id, published_at_unix_seconds)
            .await
    }

    pub async fn complete_telegram_publish_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        if claim.route.provider != CommunicationProviderProvenanceV1::Telegram {
            return Err(DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider);
        }
        self.complete_publish_v1(claim, message_id, published_at_unix_seconds)
            .await
    }

    pub async fn complete_whatsapp_publish_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        if claim.route.provider != CommunicationProviderProvenanceV1::WhatsAppWeb {
            return Err(DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider);
        }
        self.complete_publish_v1(claim, message_id, published_at_unix_seconds)
            .await
    }

    pub async fn complete_zulip_publish_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        if claim.route.provider != CommunicationProviderProvenanceV1::Zulip {
            return Err(DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider);
        }
        self.complete_publish_v1(claim, message_id, published_at_unix_seconds)
            .await
    }

    pub async fn apply_mail_succeeded_v1(
        &self,
        exact_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        self.apply_decoded_terminal_v1(
            mail::decode_succeeded_v1(exact_bytes)
                .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?,
            consumed_at_unix_seconds,
        )
        .await
    }

    pub async fn apply_mail_rejected_v1(
        &self,
        exact_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        self.apply_decoded_terminal_v1(
            mail::decode_rejected_v1(exact_bytes)
                .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?,
            consumed_at_unix_seconds,
        )
        .await
    }

    pub async fn apply_telegram_succeeded_v1(
        &self,
        exact_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        self.apply_decoded_terminal_v1(
            telegram::decode_succeeded_v1(exact_bytes)
                .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?,
            consumed_at_unix_seconds,
        )
        .await
    }

    pub async fn apply_telegram_rejected_v1(
        &self,
        exact_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        self.apply_decoded_terminal_v1(
            telegram::decode_rejected_v1(exact_bytes)
                .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?,
            consumed_at_unix_seconds,
        )
        .await
    }

    pub async fn apply_whatsapp_succeeded_v1(
        &self,
        exact_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        self.apply_decoded_terminal_v1(
            whatsapp::decode_succeeded_v1(exact_bytes)
                .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?,
            consumed_at_unix_seconds,
        )
        .await
    }

    pub async fn apply_whatsapp_rejected_v1(
        &self,
        exact_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        self.apply_decoded_terminal_v1(
            whatsapp::decode_rejected_v1(exact_bytes)
                .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?,
            consumed_at_unix_seconds,
        )
        .await
    }

    pub async fn apply_zulip_succeeded_v1(
        &self,
        exact_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        self.apply_decoded_terminal_v1(
            zulip::decode_succeeded_v1(exact_bytes)
                .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?,
            consumed_at_unix_seconds,
        )
        .await
    }

    pub async fn apply_zulip_rejected_v1(
        &self,
        exact_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        self.apply_decoded_terminal_v1(
            zulip::decode_rejected_v1(exact_bytes)
                .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Adapter)?,
            consumed_at_unix_seconds,
        )
        .await
    }

    async fn apply_decoded_terminal_v1(
        &self,
        decoded: DecodedDeliveryIntentTerminalV1,
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentProviderEventRuntimeErrorV1>
    {
        let value = match decoded.outcome {
            DeliveryIntentTerminalOutcomeV1::Succeeded {
                provider_operation_id,
            } => TerminalDeliveryResultValueV1::Succeeded {
                provider_operation_id,
            },
            DeliveryIntentTerminalOutcomeV1::Rejected { rejection_code } => {
                TerminalDeliveryResultValueV1::Rejected { rejection_code }
            }
        };
        self.persistence()
            .apply_terminal_result(
                &TerminalDeliveryResultV1 {
                    envelope_message_id: decoded.envelope_message_id,
                    envelope_sha256: decoded.envelope_sha256,
                    command_message_id: decoded.command_message_id,
                    logical_owner_id: decoded.logical_owner_id,
                    intent_id: decoded.intent_id,
                    value,
                },
                consumed_at_unix_seconds,
            )
            .await
            .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Persistence)
    }

    async fn complete_publish_v1(
        &self,
        claim: &DeliveryIntentClaimV1,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentProviderEventRuntimeErrorV1> {
        self.persistence()
            .mark_provider_command_published(claim, message_id, published_at_unix_seconds)
            .await
            .map_err(DeliveryIntentProviderEventRuntimeErrorV1::Persistence)
    }
}

fn body_source_v1(claim: &DeliveryIntentClaimV1) -> DeliveryIntentBodySourceV1 {
    DeliveryIntentBodySourceV1 {
        reference_id: claim.body_receipt.reference_id,
        declared_bytes: claim.body_receipt.declared_bytes,
        sha256: claim.body_receipt.sha256,
        custody_transfer_source_proof: claim.body_receipt.custody_transfer_source_proof.clone(),
    }
}
