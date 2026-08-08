//! Exact attachment event and observation contract references.

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

use crate::{
    COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_SCHEMA_SHA256,
    COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256,
    COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256,
    COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256,
};

pub const COMMUNICATION_ATTACHMENT_CONTRACT_OWNER: &str = "communications";
pub const COMMUNICATION_ATTACHMENT_CONTRACT_MAJOR: u32 = 1;
pub const COMMUNICATION_ATTACHMENT_CONTRACT_REVISION: u32 = 1;
pub const COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT: u32 = 64;
pub const COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVED_CONTRACT_NAME: &str =
    "communication_attachment_blob_admission_observed";
pub const COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVED_CONTRACT_NAME: &str =
    "communication_attachment_safety_verdict_observed";
pub const COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_CONTRACT_NAME: &str =
    "communication_attachment_anchor_recorded";
pub const COMMUNICATION_ATTACHMENT_SAFETY_STATE_CHANGED_CONTRACT_NAME: &str =
    "communication_attachment_safety_state_changed";

#[must_use]
pub fn communication_attachment_blob_admission_observed_contract_reference_v1()
-> ContractReferenceV1 {
    contract(
        COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVED_CONTRACT_NAME,
        COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256,
    )
}

#[must_use]
pub fn communication_attachment_safety_verdict_observed_contract_reference_v1()
-> ContractReferenceV1 {
    contract(
        COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVED_CONTRACT_NAME,
        COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256,
    )
}

#[must_use]
pub fn communication_attachment_anchor_recorded_contract_reference_v1() -> ContractReferenceV1 {
    contract(
        COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_CONTRACT_NAME,
        COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_SCHEMA_SHA256,
    )
}

#[must_use]
pub fn communication_attachment_safety_state_changed_contract_reference_v1() -> ContractReferenceV1
{
    contract(
        COMMUNICATION_ATTACHMENT_SAFETY_STATE_CHANGED_CONTRACT_NAME,
        COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256,
    )
}

#[must_use]
pub fn communication_attachment_blob_admission_observed_publish_request_v1() -> CapabilityRequestV1
{
    publish_observation(communication_attachment_blob_admission_observed_contract_reference_v1())
}

#[must_use]
pub fn communication_attachment_safety_verdict_observed_publish_request_v1() -> CapabilityRequestV1
{
    publish_observation(communication_attachment_safety_verdict_observed_contract_reference_v1())
}

fn contract(name: &str, schema_sha256: [u8; 32]) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_ATTACHMENT_CONTRACT_OWNER.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_ATTACHMENT_CONTRACT_MAJOR,
        revision: COMMUNICATION_ATTACHMENT_CONTRACT_REVISION,
        schema_sha256: schema_sha256.to_vec(),
    }
}

fn publish_observation(contract: ContractReferenceV1) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Observation as i32,
            contract: Some(contract),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
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
    fn attachment_contracts_are_distinct_and_schema_bound() {
        let blob = communication_attachment_blob_admission_observed_contract_reference_v1();
        let safety = communication_attachment_safety_verdict_observed_contract_reference_v1();
        let anchor = communication_attachment_anchor_recorded_contract_reference_v1();
        let lifecycle = communication_attachment_safety_state_changed_contract_reference_v1();

        assert_eq!(blob.owner, COMMUNICATION_ATTACHMENT_CONTRACT_OWNER);
        assert_eq!(safety.owner, COMMUNICATION_ATTACHMENT_CONTRACT_OWNER);
        assert_ne!(blob.name, safety.name);
        assert_ne!(anchor.name, lifecycle.name);
        assert_eq!(
            blob.schema_sha256,
            COMMUNICATION_ATTACHMENT_BLOB_ADMISSION_OBSERVATION_SCHEMA_SHA256,
        );
        assert_eq!(
            safety.schema_sha256,
            COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256,
        );
        assert_eq!(
            anchor.schema_sha256,
            COMMUNICATION_ATTACHMENT_ANCHOR_RECORDED_SCHEMA_SHA256,
        );
        assert_eq!(
            lifecycle.schema_sha256,
            COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256,
        );
    }

    #[test]
    fn producer_routes_are_publish_only_observations() {
        for request in [
            communication_attachment_blob_admission_observed_publish_request_v1(),
            communication_attachment_safety_verdict_observed_publish_request_v1(),
        ] {
            let Some(Request::EventRoute(route)) = request.request else {
                panic!("event route");
            };
            assert_eq!(
                route.envelope_kind,
                DurableEnvelopeKindV1::Observation as i32
            );
            assert_eq!(route.direction, EventRouteDirectionV1::Publish as i32);
        }
    }
}
