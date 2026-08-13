#![forbid(unsafe_code)]

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

mod envelope;
pub use envelope::{
    ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1,
    ReviewPersonMatchCandidatePromotionEnvelopeContextV1,
    ReviewPersonMatchCandidatePromotionResultShapeV1,
    build_review_person_match_candidate_promotion_result_outbox_record_v1,
    review_person_match_candidate_promotion_result_id_v1,
};

pub const PACKAGE: &str = "makosh-review-person-match-candidate-promotion-api";
pub const REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1: &str = "review";
pub const REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_CONTRACT_NAME_V1: &str =
    "review_person_match_candidate_promotion_result";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.review.person_match_candidate.promotion.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/review_person_match_candidate_promotion_schema.rs"
));

pub const REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(
        env!("OUT_DIR"),
        "/review-person-match-candidate-promotion-v1.bin"
    ));

#[must_use]
pub fn review_person_match_candidate_promotion_result_contract_reference_v1() -> ContractReferenceV1
{
    ContractReferenceV1 {
        owner: REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
        name: REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_CONTRACT_NAME_V1.to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1.to_vec(),
    }
}

#[must_use]
pub fn review_person_match_candidate_promotion_result_consume_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Result as i32,
            contract: Some(review_person_match_candidate_promotion_result_contract_reference_v1()),
            direction: EventRouteDirectionV1::Consume as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
            max_deliver: 10,
            ack_wait_millis: 30_000,
        })),
    }
}

#[must_use]
pub fn review_person_match_candidate_promotion_result_publish_request_v1() -> CapabilityRequestV1 {
    let mut request = review_person_match_candidate_promotion_result_consume_request_v1();
    if let Some(Request::EventRoute(route)) = request.request.as_mut() {
        route.direction = EventRouteDirectionV1::Publish as i32;
        route.subscription_requirement = EventSubscriptionRequirementV1::Unspecified as i32;
        route.max_deliver = 0;
        route.ack_wait_millis = 0;
    }
    request
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;
    use prost::Message;

    #[test]
    fn promotion_result_is_bounded_and_review_owned() {
        let reference = review_person_match_candidate_promotion_result_contract_reference_v1();
        assert_eq!(reference.owner, "review");
        assert!(!REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_DESCRIPTOR_SET_V1.is_empty());
        let schema = include_str!(
            "../proto/makosh/review/person_match_candidate/promotion/v1/promotion.proto"
        );
        assert!(schema.contains("ReviewPersonMatchCandidatePromotionResultV1"));
        for forbidden in [
            "normalized_email",
            "normalized_phone",
            "provider_entry_id",
            "provider_etag",
            "continuation_cursor",
        ] {
            assert!(!schema.contains(forbidden));
        }
    }

    #[test]
    fn action_digest_mismatch_is_a_typed_terminal_without_persons_command() {
        let result_id = review_person_match_candidate_promotion_result_id_v1(
            [4; 16],
            [4; 16],
            ReviewPersonMatchCandidatePromotionResultShapeV1::ActionDigestMismatch,
        )
        .expect("canonical local failure result ID");
        let payload = wire::ReviewPersonMatchCandidatePromotionResultV1 {
            result_id: result_id.to_vec(),
            review_id: vec![2; 16],
            candidate_id: vec![3; 16],
            decision_id: vec![4; 16],
            expected_review_revision: 2,
            outcome: wire::ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeFailed as i32,
            persons_command_id: None,
            failure_code: wire::ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeActionDigestMismatch as i32,
            logical_owner_id: "owner-a".to_owned(),
            completed_at_unix_millis: 1_000,
        };
        let record = build_review_person_match_candidate_promotion_result_outbox_record_v1(
            [4; 16],
            payload.clone(),
            &ReviewPersonMatchCandidatePromotionEnvelopeContextV1 {
                module_id: "makosh-reviewed-person-match-candidate-promotion-runtime".to_owned(),
                runtime_instance_id: "runtime-a".to_owned(),
                runtime_generation: 2,
                recorded_at_unix_millis: 1_000,
            },
        )
        .expect("typed local terminal");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert_eq!(envelope.causation_message_id, vec![4; 16]);
        assert_eq!(
            wire::ReviewPersonMatchCandidatePromotionResultV1::decode(envelope.payload.as_slice())
                .expect("payload"),
            payload
        );
    }

    #[test]
    fn canonical_result_id_binds_causation_decision_and_shape() {
        let persons = review_person_match_candidate_promotion_result_id_v1(
            [1; 16],
            [2; 16],
            ReviewPersonMatchCandidatePromotionResultShapeV1::PersonsTerminal,
        )
        .expect("Persons terminal result ID");
        let local = review_person_match_candidate_promotion_result_id_v1(
            [1; 16],
            [2; 16],
            ReviewPersonMatchCandidatePromotionResultShapeV1::ActionDigestMismatch,
        )
        .expect("local result ID");
        assert_ne!(persons, local);
        assert_ne!(
            persons,
            review_person_match_candidate_promotion_result_id_v1(
                [3; 16],
                [2; 16],
                ReviewPersonMatchCandidatePromotionResultShapeV1::PersonsTerminal,
            )
            .expect("changed causation")
        );
        assert_ne!(
            persons,
            review_person_match_candidate_promotion_result_id_v1(
                [1; 16],
                [3; 16],
                ReviewPersonMatchCandidatePromotionResultShapeV1::PersonsTerminal,
            )
            .expect("changed decision")
        );
    }
}
