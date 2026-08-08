//! Client-safe mapping of durable delivery-intent state and rejection.

use makosh_communication_delivery_intent_api::wire::{
    DeliveryIntentErrorCodeV1, DeliveryIntentStatusV1,
};
use makosh_communication_delivery_intent_persistence::DeliveryIntentStateV1;

pub(crate) const fn status_value(state: DeliveryIntentStateV1) -> i32 {
    match state {
        DeliveryIntentStateV1::Accepted => {
            DeliveryIntentStatusV1::DeliveryIntentStatusAccepted as i32
        }
        DeliveryIntentStateV1::ResolvingRoute => {
            DeliveryIntentStatusV1::DeliveryIntentStatusResolvingRoute as i32
        }
        DeliveryIntentStateV1::SubmittedToProvider => {
            DeliveryIntentStatusV1::DeliveryIntentStatusSubmittedToProvider as i32
        }
        DeliveryIntentStateV1::ProviderConfirmed => {
            DeliveryIntentStatusV1::DeliveryIntentStatusProviderConfirmed as i32
        }
        DeliveryIntentStateV1::Rejected => {
            DeliveryIntentStatusV1::DeliveryIntentStatusRejected as i32
        }
    }
}

pub(crate) const fn rejection_value(rejection_code: Option<u16>) -> i32 {
    match rejection_code {
        Some(_) => DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeProviderRejected as i32,
        None => DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnspecified as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_preserves_every_durable_state() {
        assert_eq!(status_value(DeliveryIntentStateV1::Accepted), 1);
        assert_eq!(status_value(DeliveryIntentStateV1::ResolvingRoute), 2);
        assert_eq!(status_value(DeliveryIntentStateV1::SubmittedToProvider), 3);
        assert_eq!(status_value(DeliveryIntentStateV1::ProviderConfirmed), 4);
        assert_eq!(status_value(DeliveryIntentStateV1::Rejected), 5);
    }

    #[test]
    fn rejection_is_client_safe() {
        assert_eq!(rejection_value(None), 0);
        assert_eq!(rejection_value(Some(731)), 5);
    }
}
