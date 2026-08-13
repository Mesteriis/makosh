#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    ReviewObligationCandidatePromotionEnvelopeBuildErrorV1,
    ReviewObligationCandidatePromotionEnvelopeContextV1,
    build_review_obligation_candidate_promotion_result_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-review-obligation-candidate-promotion-api";
pub const REVIEW_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1: &str = "review";
pub const REVIEW_OBLIGATION_CANDIDATE_PROMOTION_RESULT_CAPABILITY_ID_V1: &str =
    "review.obligation-candidate.promotion-result.v1";
pub const REVIEW_OBLIGATION_CANDIDATE_PROMOTION_RESULT_CONTRACT_NAME_V1: &str =
    "review_obligation_candidate_promotion_result";
pub const REVIEW_OBLIGATION_CANDIDATE_PROMOTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const REVIEW_OBLIGATION_CANDIDATE_PROMOTION_CONTRACT_REVISION_V1: u32 = 1;
pub const REVIEW_OBLIGATION_CANDIDATE_PROMOTION_MAX_IN_FLIGHT_V1: u32 = 32;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.review.obligation_candidate.promotion.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/review_obligation_candidate_promotion_schema.rs"
));

pub const REVIEW_OBLIGATION_CANDIDATE_PROMOTION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/review-obligation-candidate-promotion-v1.bin"
));

#[must_use]
pub fn review_obligation_candidate_promotion_result_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: REVIEW_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
        name: REVIEW_OBLIGATION_CANDIDATE_PROMOTION_RESULT_CONTRACT_NAME_V1.to_owned(),
        major: REVIEW_OBLIGATION_CANDIDATE_PROMOTION_CONTRACT_MAJOR_V1,
        revision: REVIEW_OBLIGATION_CANDIDATE_PROMOTION_CONTRACT_REVISION_V1,
        schema_sha256: REVIEW_OBLIGATION_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1.to_vec(),
    }
}

#[must_use]
pub fn review_obligation_candidate_promotion_result_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn review_obligation_candidate_promotion_result_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn event_route(
    direction: EventRouteDirectionV1,
    subscription_requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(review_obligation_candidate_promotion_result_contract_reference_v1()),
            direction: direction as i32,
            max_in_flight: REVIEW_OBLIGATION_CANDIDATE_PROMOTION_MAX_IN_FLIGHT_V1,
            subscription_requirement: subscription_requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotion_result_is_review_owned_and_event_only() {
        let Some(Request::EventRoute(route)) =
            review_obligation_candidate_promotion_result_consume_request_v1().request
        else {
            panic!("promotion result route");
        };
        assert_eq!(route.envelope_kind, DurableEnvelopeKindV1::Event as i32);
        assert_eq!(route.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            route.contract.expect("contract").owner,
            REVIEW_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1
        );
    }

    #[test]
    fn wire_contract_excludes_private_candidate_and_provider_data() {
        let source = include_str!(
            "../proto/makosh/review/obligation_candidate/promotion/v1/promotion.proto"
        );
        for forbidden in [
            "statement",
            "due_text",
            "assignee_label",
            "source_body",
            "blob",
            "provider_id",
            "account_id",
            "map<",
        ] {
            assert!(!source.contains(forbidden), "forbidden field {forbidden}");
        }
    }
}
