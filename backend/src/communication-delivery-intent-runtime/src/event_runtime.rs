//! Managed Event Hub execution for provider commands and terminal results.

use makosh_communication_delivery_intent_core::CommunicationProviderProvenanceV1;
use makosh_communication_delivery_intent_event_adapters::DeliveryIntentCommandContextV1;
use makosh_communication_delivery_intent_persistence::DeliveryIntentClaimV1;
use makosh_events_jetstream::{RuntimeSubscribePermitV1, receive_runtime_pull_delivery};
use makosh_mail_delivery_intent_contract::{
    mail_delivery_intent_rejected_contract_reference_v1,
    mail_delivery_intent_succeeded_contract_reference_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use makosh_telegram_delivery_intent_contract::{
    telegram_delivery_intent_rejected_contract_reference_v1,
    telegram_delivery_intent_succeeded_contract_reference_v1,
};
use makosh_whatsapp_delivery_intent_contract::{
    whatsapp_delivery_intent_rejected_contract_reference_v1,
    whatsapp_delivery_intent_succeeded_contract_reference_v1,
};
use makosh_zulip_delivery_intent_contract::{
    zulip_delivery_intent_rejected_contract_reference_v1,
    zulip_delivery_intent_succeeded_contract_reference_v1,
};
use sha2::{Digest, Sha256};

use crate::{
    provider_events::DeliveryIntentProviderEventRuntimeErrorV1,
    runtime::{DeliveryIntentManagedRuntimeV1, DeliveryIntentRuntimeErrorV1},
};

const CLAIM_LEASE_SECONDS: i64 = 30;
const COMMAND_DEADLINE_SECONDS: i64 = 300;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProviderTerminalRouteV1 {
    MailSucceeded,
    MailRejected,
    TelegramSucceeded,
    TelegramRejected,
    WhatsAppSucceeded,
    WhatsAppRejected,
    ZulipSucceeded,
    ZulipRejected,
}

#[derive(Clone)]
pub(crate) struct ProviderTerminalSubscriptionV1 {
    route: ProviderTerminalRouteV1,
    permit: RuntimeSubscribePermitV1,
}

pub(crate) fn bind_terminal_subscriptions(
    permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<Vec<ProviderTerminalSubscriptionV1>, DeliveryIntentRuntimeErrorV1> {
    if permits.len() != 8 {
        return Err(DeliveryIntentRuntimeErrorV1::Admission);
    }
    let expected = expected_terminal_routes();
    let mut bound = Vec::with_capacity(expected.len());
    for (route, contract) in expected {
        let mut matching = permits
            .iter()
            .filter(|permit| {
                permit
                    .contract()
                    .is_some_and(|actual| exact_contract(actual, &contract))
            })
            .cloned();
        let permit = matching
            .next()
            .ok_or(DeliveryIntentRuntimeErrorV1::Admission)?;
        if matching.next().is_some() {
            return Err(DeliveryIntentRuntimeErrorV1::Admission);
        }
        bound.push(ProviderTerminalSubscriptionV1 { route, permit });
    }
    Ok(bound)
}

impl DeliveryIntentManagedRuntimeV1 {
    pub async fn process_next_provider_command_v1(
        &self,
        now_unix_seconds: i64,
    ) -> Result<bool, DeliveryIntentRuntimeErrorV1> {
        if now_unix_seconds <= 0 {
            return Err(DeliveryIntentRuntimeErrorV1::Admission);
        }
        let lease_expires_at = now_unix_seconds
            .checked_add(CLAIM_LEASE_SECONDS)
            .ok_or(DeliveryIntentRuntimeErrorV1::Admission)?;
        let worker_id = worker_id(&self.runtime_instance_id, self.runtime_generation);
        let Some(claim) = self
            .persistence()
            .claim_next(
                &self.logical_owner_id,
                &worker_id,
                now_unix_seconds,
                lease_expires_at,
            )
            .await
            .map_err(DeliveryIntentRuntimeErrorV1::Persistence)?
        else {
            return Ok(false);
        };

        let record = if let Some(entry) = self
            .persistence()
            .provider_command_for_claim(&claim)
            .await
            .map_err(DeliveryIntentRuntimeErrorV1::Persistence)?
        {
            entry.record
        } else {
            self.enqueue_claim(&claim, now_unix_seconds).await?;
            self.persistence()
                .provider_command_for_claim(&claim)
                .await
                .map_err(DeliveryIntentRuntimeErrorV1::Persistence)?
                .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)?
                .record
        };

        self.event_connection
            .publish_exact(&self.event_publish_permit, record.exact_bytes())
            .await
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        self.complete_provider_publish(&claim, *record.message_id(), now_unix_seconds)
            .await?;
        Ok(true)
    }

    pub async fn consume_next_terminal_result_v1(
        &mut self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, DeliveryIntentRuntimeErrorV1> {
        if consumed_at_unix_seconds <= 0 || self.terminal_subscriptions.is_empty() {
            return Err(DeliveryIntentRuntimeErrorV1::Admission);
        }
        let index = self.next_terminal_subscription % self.terminal_subscriptions.len();
        self.next_terminal_subscription = (index + 1) % self.terminal_subscriptions.len();
        let subscription = self.terminal_subscriptions[index].clone();
        let delivery = receive_runtime_pull_delivery(&self.event_connection, &subscription.permit)
            .await
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        let exact_bytes = delivery.exact_bytes();
        match subscription.route {
            ProviderTerminalRouteV1::MailSucceeded => {
                self.apply_mail_succeeded_v1(exact_bytes, consumed_at_unix_seconds)
                    .await
            }
            ProviderTerminalRouteV1::MailRejected => {
                self.apply_mail_rejected_v1(exact_bytes, consumed_at_unix_seconds)
                    .await
            }
            ProviderTerminalRouteV1::TelegramSucceeded => {
                self.apply_telegram_succeeded_v1(exact_bytes, consumed_at_unix_seconds)
                    .await
            }
            ProviderTerminalRouteV1::TelegramRejected => {
                self.apply_telegram_rejected_v1(exact_bytes, consumed_at_unix_seconds)
                    .await
            }
            ProviderTerminalRouteV1::WhatsAppSucceeded => {
                self.apply_whatsapp_succeeded_v1(exact_bytes, consumed_at_unix_seconds)
                    .await
            }
            ProviderTerminalRouteV1::WhatsAppRejected => {
                self.apply_whatsapp_rejected_v1(exact_bytes, consumed_at_unix_seconds)
                    .await
            }
            ProviderTerminalRouteV1::ZulipSucceeded => {
                self.apply_zulip_succeeded_v1(exact_bytes, consumed_at_unix_seconds)
                    .await
            }
            ProviderTerminalRouteV1::ZulipRejected => {
                self.apply_zulip_rejected_v1(exact_bytes, consumed_at_unix_seconds)
                    .await
            }
        }
        .map_err(provider_event_error)?;
        delivery
            .acknowledge()
            .await
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    async fn enqueue_claim(
        &self,
        claim: &DeliveryIntentClaimV1,
        now_unix_seconds: i64,
    ) -> Result<(), DeliveryIntentRuntimeErrorV1> {
        let context = DeliveryIntentCommandContextV1 {
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            recorded_at_unix_seconds: now_unix_seconds,
            recorded_at_nanos: 0,
            deadline_unix_seconds: now_unix_seconds
                .checked_add(COMMAND_DEADLINE_SECONDS)
                .ok_or(DeliveryIntentRuntimeErrorV1::Admission)?,
            causation_message_id: claim.intent_id,
        };
        match claim.route.provider {
            CommunicationProviderProvenanceV1::MailImap
            | CommunicationProviderProvenanceV1::MailSmtp
            | CommunicationProviderProvenanceV1::MailGmail => {
                self.enqueue_mail_command_v1(claim, &context, now_unix_seconds)
                    .await
            }
            CommunicationProviderProvenanceV1::Telegram => {
                self.enqueue_telegram_command_v1(claim, &context, now_unix_seconds)
                    .await
            }
            CommunicationProviderProvenanceV1::WhatsAppWeb => {
                self.enqueue_whatsapp_command_v1(claim, &context, now_unix_seconds)
                    .await
            }
            CommunicationProviderProvenanceV1::Zulip => {
                self.enqueue_zulip_command_v1(claim, &context, now_unix_seconds)
                    .await
            }
        }
        .map(|_| ())
        .map_err(provider_event_error)
    }

    async fn complete_provider_publish(
        &self,
        claim: &DeliveryIntentClaimV1,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<(), DeliveryIntentRuntimeErrorV1> {
        match claim.route.provider {
            CommunicationProviderProvenanceV1::MailImap
            | CommunicationProviderProvenanceV1::MailSmtp
            | CommunicationProviderProvenanceV1::MailGmail => {
                self.complete_mail_publish_v1(claim, message_id, published_at_unix_seconds)
                    .await
            }
            CommunicationProviderProvenanceV1::Telegram => {
                self.complete_telegram_publish_v1(claim, message_id, published_at_unix_seconds)
                    .await
            }
            CommunicationProviderProvenanceV1::WhatsAppWeb => {
                self.complete_whatsapp_publish_v1(claim, message_id, published_at_unix_seconds)
                    .await
            }
            CommunicationProviderProvenanceV1::Zulip => {
                self.complete_zulip_publish_v1(claim, message_id, published_at_unix_seconds)
                    .await
            }
        }
        .map(|_| ())
        .map_err(provider_event_error)
    }
}

fn expected_terminal_routes() -> Vec<(ProviderTerminalRouteV1, ContractReferenceV1)> {
    vec![
        (
            ProviderTerminalRouteV1::MailSucceeded,
            mail_delivery_intent_succeeded_contract_reference_v1(),
        ),
        (
            ProviderTerminalRouteV1::MailRejected,
            mail_delivery_intent_rejected_contract_reference_v1(),
        ),
        (
            ProviderTerminalRouteV1::TelegramSucceeded,
            telegram_delivery_intent_succeeded_contract_reference_v1(),
        ),
        (
            ProviderTerminalRouteV1::TelegramRejected,
            telegram_delivery_intent_rejected_contract_reference_v1(),
        ),
        (
            ProviderTerminalRouteV1::WhatsAppSucceeded,
            whatsapp_delivery_intent_succeeded_contract_reference_v1(),
        ),
        (
            ProviderTerminalRouteV1::WhatsAppRejected,
            whatsapp_delivery_intent_rejected_contract_reference_v1(),
        ),
        (
            ProviderTerminalRouteV1::ZulipSucceeded,
            zulip_delivery_intent_succeeded_contract_reference_v1(),
        ),
        (
            ProviderTerminalRouteV1::ZulipRejected,
            zulip_delivery_intent_rejected_contract_reference_v1(),
        ),
    ]
}

fn exact_contract(actual: &ContractReferenceV1, expected: &ContractReferenceV1) -> bool {
    actual.owner == expected.owner
        && actual.name == expected.name
        && actual.major == expected.major
        && actual.revision == expected.revision
        && actual.schema_sha256 == expected.schema_sha256
}

fn provider_event_error(
    error: DeliveryIntentProviderEventRuntimeErrorV1,
) -> DeliveryIntentRuntimeErrorV1 {
    match error {
        DeliveryIntentProviderEventRuntimeErrorV1::Persistence(error) => {
            DeliveryIntentRuntimeErrorV1::Persistence(error)
        }
        DeliveryIntentProviderEventRuntimeErrorV1::WrongProvider
        | DeliveryIntentProviderEventRuntimeErrorV1::Adapter(_) => {
            DeliveryIntentRuntimeErrorV1::EventContract
        }
    }
}

fn worker_id(runtime_instance_id: &str, runtime_generation: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"communication-delivery-intent-provider-worker-v1");
    hasher.update(runtime_instance_id.as_bytes());
    hasher.update(runtime_generation.to_be_bytes());
    let digest = hasher.finalize();
    let mut value = String::from("delivery-intent:");
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(value, "{byte:02x}");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_identity_is_bounded_and_generation_fenced() {
        let first = worker_id("runtime", 7);
        assert!(first.len() <= 128);
        assert_ne!(first, worker_id("runtime", 8));
        assert_ne!(first, worker_id("successor", 7));
    }

    #[test]
    fn terminal_route_inventory_is_exact() {
        let routes = expected_terminal_routes();
        assert_eq!(routes.len(), 8);
        let mut names = routes
            .iter()
            .map(|(_, contract)| format!("{}:{}", contract.owner, contract.name))
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 8);
    }
}
