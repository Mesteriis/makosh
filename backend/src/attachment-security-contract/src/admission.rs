//! Exact Attachment Security scan-candidate contract reference.

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

use crate::ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256;

pub const ATTACHMENT_SECURITY_CONTRACT_OWNER: &str = "attachment_security";
pub const ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME: &str =
    "attachment_security_scan_candidate_observed";
pub const ATTACHMENT_SECURITY_CONTRACT_MAJOR: u32 = 1;
pub const ATTACHMENT_SECURITY_CONTRACT_REVISION: u32 = 2;
pub const ATTACHMENT_SECURITY_MAX_IN_FLIGHT: u32 = 32;
pub const ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_OWNER_ID: &str =
    ATTACHMENT_SECURITY_CONTRACT_OWNER;
pub const ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID: &str =
    "makosh-attachment-security-runtime";
pub const ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID: &str =
    "attachment_security.blob.v1";

#[must_use]
pub fn attachment_security_scan_candidate_observed_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ATTACHMENT_SECURITY_CONTRACT_OWNER.to_owned(),
        name: ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME.to_owned(),
        major: ATTACHMENT_SECURITY_CONTRACT_MAJOR,
        revision: ATTACHMENT_SECURITY_CONTRACT_REVISION,
        schema_sha256: ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn attachment_security_scan_candidate_observed_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Observation as i32,
            contract: Some(attachment_security_scan_candidate_observed_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
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
        let request = attachment_security_scan_candidate_observed_publish_request_v1();
        let Some(Request::EventRoute(route)) = request.request else {
            panic!("event route");
        };

        assert_eq!(
            route.envelope_kind,
            DurableEnvelopeKindV1::Observation as i32
        );
        assert_eq!(route.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(
            route.contract.expect("contract").schema_sha256,
            ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256,
        );
    }
}
