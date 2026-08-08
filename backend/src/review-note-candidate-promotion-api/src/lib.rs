#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    ReviewNoteCandidatePromotionEnvelopeBuildErrorV1,
    ReviewNoteCandidatePromotionEnvelopeContextV1,
    build_review_note_candidate_promotion_result_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-review-note-candidate-promotion-api";
pub const REVIEW_NOTE_CANDIDATE_PROMOTION_OWNER_V1: &str = "review";
pub const REVIEW_NOTE_CANDIDATE_PROMOTION_RESULT_CAPABILITY_ID_V1: &str =
    "review.note-candidate.promotion-result.v1";
pub const REVIEW_NOTE_CANDIDATE_PROMOTION_RESULT_CONTRACT_NAME_V1: &str =
    "review_note_candidate_promotion_result";
pub const REVIEW_NOTE_CANDIDATE_PROMOTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const REVIEW_NOTE_CANDIDATE_PROMOTION_CONTRACT_REVISION_V1: u32 = 1;
pub const REVIEW_NOTE_CANDIDATE_PROMOTION_MAX_IN_FLIGHT_V1: u32 = 32;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.review.note_candidate.promotion.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/review_note_candidate_promotion_schema.rs"
));

pub const REVIEW_NOTE_CANDIDATE_PROMOTION_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/review-note-candidate-promotion-v1.bin"
));

#[must_use]
pub fn review_note_candidate_promotion_result_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: REVIEW_NOTE_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
        name: REVIEW_NOTE_CANDIDATE_PROMOTION_RESULT_CONTRACT_NAME_V1.to_owned(),
        major: REVIEW_NOTE_CANDIDATE_PROMOTION_CONTRACT_MAJOR_V1,
        revision: REVIEW_NOTE_CANDIDATE_PROMOTION_CONTRACT_REVISION_V1,
        schema_sha256: REVIEW_NOTE_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1.to_vec(),
    }
}

#[must_use]
pub fn review_note_candidate_promotion_result_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn review_note_candidate_promotion_result_consume_request_v1() -> CapabilityRequestV1 {
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
            contract: Some(review_note_candidate_promotion_result_contract_reference_v1()),
            direction: direction as i32,
            max_in_flight: REVIEW_NOTE_CANDIDATE_PROMOTION_MAX_IN_FLIGHT_V1,
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
            review_note_candidate_promotion_result_consume_request_v1().request
        else {
            panic!("promotion result route");
        };
        assert_eq!(route.envelope_kind, DurableEnvelopeKindV1::Event as i32);
        assert_eq!(route.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            route.contract.expect("contract").owner,
            REVIEW_NOTE_CANDIDATE_PROMOTION_OWNER_V1
        );
    }

    #[test]
    fn wire_contract_excludes_private_candidate_and_provider_data() {
        let source =
            include_str!("../proto/makosh/review/note_candidate/promotion/v1/promotion.proto");
        for forbidden in [
            "title",
            "excerpt",
            "topic_hints",
            "source_basis",
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
