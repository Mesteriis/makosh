//! Exact provider-event capability units owned by the delivery-intent workflow.

use makosh_mail_delivery_intent_contract::{
    mail_delivery_intent_execute_publish_request_v1,
    mail_delivery_intent_rejected_consume_request_v1,
    mail_delivery_intent_succeeded_consume_request_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1,
};
use makosh_telegram_delivery_intent_contract::{
    telegram_delivery_intent_execute_publish_request_v1,
    telegram_delivery_intent_rejected_consume_request_v1,
    telegram_delivery_intent_succeeded_consume_request_v1,
};
use makosh_whatsapp_delivery_intent_contract::{
    whatsapp_delivery_intent_execute_publish_request_v1,
    whatsapp_delivery_intent_rejected_consume_request_v1,
    whatsapp_delivery_intent_succeeded_consume_request_v1,
};
use makosh_zulip_delivery_intent_contract::{
    zulip_delivery_intent_execute_publish_request_v1,
    zulip_delivery_intent_rejected_consume_request_v1,
    zulip_delivery_intent_succeeded_consume_request_v1,
};

pub const DELIVERY_INTENT_MAIL_EVENTS_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.mail.events.v1";
pub const DELIVERY_INTENT_TELEGRAM_EVENTS_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.telegram.events.v1";
pub const DELIVERY_INTENT_WHATSAPP_EVENTS_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.whatsapp.events.v1";
pub const DELIVERY_INTENT_ZULIP_EVENTS_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.zulip.events.v1";

#[must_use]
pub fn delivery_intent_mail_events_capability_v1() -> CapabilityDescriptorV1 {
    capability(
        DELIVERY_INTENT_MAIL_EVENTS_CAPABILITY_ID_V1,
        vec![
            mail_delivery_intent_execute_publish_request_v1(),
            mail_delivery_intent_succeeded_consume_request_v1(),
            mail_delivery_intent_rejected_consume_request_v1(),
        ],
    )
}

#[must_use]
pub fn delivery_intent_telegram_events_capability_v1() -> CapabilityDescriptorV1 {
    capability(
        DELIVERY_INTENT_TELEGRAM_EVENTS_CAPABILITY_ID_V1,
        vec![
            telegram_delivery_intent_execute_publish_request_v1(),
            telegram_delivery_intent_succeeded_consume_request_v1(),
            telegram_delivery_intent_rejected_consume_request_v1(),
        ],
    )
}

#[must_use]
pub fn delivery_intent_whatsapp_events_capability_v1() -> CapabilityDescriptorV1 {
    capability(
        DELIVERY_INTENT_WHATSAPP_EVENTS_CAPABILITY_ID_V1,
        vec![
            whatsapp_delivery_intent_execute_publish_request_v1(),
            whatsapp_delivery_intent_succeeded_consume_request_v1(),
            whatsapp_delivery_intent_rejected_consume_request_v1(),
        ],
    )
}

#[must_use]
pub fn delivery_intent_zulip_events_capability_v1() -> CapabilityDescriptorV1 {
    capability(
        DELIVERY_INTENT_ZULIP_EVENTS_CAPABILITY_ID_V1,
        vec![
            zulip_delivery_intent_execute_publish_request_v1(),
            zulip_delivery_intent_succeeded_consume_request_v1(),
            zulip_delivery_intent_rejected_consume_request_v1(),
        ],
    )
}

fn capability(capability_id: &str, requests: Vec<CapabilityRequestV1>) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::{EventRouteDirectionV1, capability_request_v1::Request};

    use super::*;

    #[test]
    fn each_provider_is_an_independent_exact_event_capability() {
        let capabilities = [
            delivery_intent_mail_events_capability_v1(),
            delivery_intent_telegram_events_capability_v1(),
            delivery_intent_whatsapp_events_capability_v1(),
            delivery_intent_zulip_events_capability_v1(),
        ];
        assert_eq!(
            capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                DELIVERY_INTENT_MAIL_EVENTS_CAPABILITY_ID_V1,
                DELIVERY_INTENT_TELEGRAM_EVENTS_CAPABILITY_ID_V1,
                DELIVERY_INTENT_WHATSAPP_EVENTS_CAPABILITY_ID_V1,
                DELIVERY_INTENT_ZULIP_EVENTS_CAPABILITY_ID_V1,
            ]
        );
        for capability in capabilities {
            assert_eq!(capability.requests.len(), 3);
            let directions = capability
                .requests
                .iter()
                .map(|request| match request.request.as_ref() {
                    Some(Request::EventRoute(route)) => route.direction,
                    _ => panic!("provider event request"),
                })
                .collect::<Vec<_>>();
            assert_eq!(
                directions,
                vec![
                    EventRouteDirectionV1::Publish as i32,
                    EventRouteDirectionV1::Consume as i32,
                    EventRouteDirectionV1::Consume as i32,
                ]
            );
        }
    }
}
