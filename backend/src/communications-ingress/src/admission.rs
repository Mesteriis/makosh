//! Descriptor metadata for the exact provider-neutral observation ingress.

use hermes_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

use crate::COMMUNICATION_OBSERVATION_SCHEMA_SHA256;

pub const COMMUNICATION_OBSERVED_CONTRACT_OWNER: &str = "communications";
pub const COMMUNICATION_OBSERVED_CONTRACT_NAME: &str = "communication_observed";
pub const COMMUNICATION_OBSERVED_CONTRACT_MAJOR: u32 = 1;
pub const COMMUNICATION_OBSERVED_CONTRACT_REVISION: u32 = 3;
pub const COMMUNICATION_OBSERVED_PRIOR_CONTRACT_REVISION: u32 = 2;
pub const COMMUNICATION_OBSERVED_PRIOR_SCHEMA_SHA256: [u8; 32] = [
    103, 206, 14, 246, 59, 211, 161, 181, 56, 225, 135, 41, 245, 175, 149, 98, 200, 113, 74, 237,
    210, 242, 94, 254, 110, 38, 18, 134, 37, 212, 177, 154,
];
pub const COMMUNICATION_OBSERVED_MAX_IN_FLIGHT: u32 = 64;

#[must_use]
pub fn communication_observed_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_OBSERVED_CONTRACT_OWNER.to_owned(),
        name: COMMUNICATION_OBSERVED_CONTRACT_NAME.to_owned(),
        major: COMMUNICATION_OBSERVED_CONTRACT_MAJOR,
        revision: COMMUNICATION_OBSERVED_CONTRACT_REVISION,
        schema_sha256: COMMUNICATION_OBSERVATION_SCHEMA_SHA256.to_vec(),
    }
}

/// Exact predecessor retained only so durable revision-2 backlog can be
/// consumed after the additive media-type evolution.
#[must_use]
pub fn communication_observed_prior_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_OBSERVED_CONTRACT_OWNER.to_owned(),
        name: COMMUNICATION_OBSERVED_CONTRACT_NAME.to_owned(),
        major: COMMUNICATION_OBSERVED_CONTRACT_MAJOR,
        revision: COMMUNICATION_OBSERVED_PRIOR_CONTRACT_REVISION,
        schema_sha256: COMMUNICATION_OBSERVED_PRIOR_SCHEMA_SHA256.to_vec(),
    }
}

/// Exact route request used by integration descriptors. It requests publish
/// authority only; it does not expose a Communications runtime or store.
#[must_use]
pub fn communication_observed_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Observation as i32,
            contract: Some(communication_observed_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_route_is_publish_only_and_schema_bound() {
        let request = communication_observed_publish_request_v1();
        let Some(Request::EventRoute(route)) = request.request else {
            panic!("event route");
        };

        assert_eq!(route.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(
            route.contract.expect("contract").schema_sha256,
            COMMUNICATION_OBSERVATION_SCHEMA_SHA256,
        );
    }

    #[test]
    fn prior_revision_is_exactly_pinned_for_durable_backlog_only() {
        let prior = communication_observed_prior_contract_reference_v1();
        let current = communication_observed_contract_reference_v1();

        assert_eq!(prior.revision, 2);
        assert_eq!(
            prior.schema_sha256,
            COMMUNICATION_OBSERVED_PRIOR_SCHEMA_SHA256
        );
        assert_ne!(prior.schema_sha256, current.schema_sha256);
    }
}
