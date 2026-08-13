#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    ReviewPersonMatchCandidateEnvelopeBuildErrorV1, ReviewPersonMatchCandidateEnvelopeContextV1,
    build_review_person_match_candidate_approved_outbox_record_v1,
    build_review_person_match_candidate_submission_rejected_outbox_record_v1,
    build_review_person_match_candidate_submitted_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-review-person-match-candidate-api";
pub const REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1: &str = "review";
pub const REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1: &str =
    "makosh-review-person-match-candidate-runtime";
pub const REVIEW_PERSON_MATCH_CANDIDATE_CONTRACT_MAJOR_V1: u32 = 1;
pub const REVIEW_PERSON_MATCH_CANDIDATE_CONTRACT_REVISION_V1: u32 = 1;
pub const REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CAPABILITY_ID_V1: &str =
    "review.person-match-candidate.decision.v1";
pub const REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CONTRACT_NAME_V1: &str =
    "review_person_match_candidate_decision";
pub const REVIEW_PERSON_MATCH_CANDIDATE_SUBMITTED_CONTRACT_NAME_V1: &str =
    "review_person_match_candidate_submitted";
pub const REVIEW_PERSON_MATCH_CANDIDATE_SUBMISSION_REJECTED_CONTRACT_NAME_V1: &str =
    "review_person_match_candidate_submission_rejected";
pub const REVIEW_PERSON_MATCH_CANDIDATE_APPROVED_CONTRACT_NAME_V1: &str =
    "review_person_match_candidate_approved_for_promotion";
pub const REVIEW_PERSON_MATCH_CANDIDATE_MAX_IN_FLIGHT_V1: u32 = 32;
pub const REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_CAPABILITY_ID_V1: &str =
    "review.person-match-candidate.client.v1";
pub const REVIEW_PERSON_MATCH_CANDIDATE_DECIDE_CONNECT_PATH_V1: &str =
    "/makosh.review.person_match_candidate.v1.ReviewPersonMatchCandidateCommandService/Decide";
pub const REVIEW_PERSON_MATCH_CANDIDATE_GET_CONNECT_PATH_V1: &str =
    "/makosh.review.person_match_candidate.v1.ReviewPersonMatchCandidateQueryService/Get";
pub const REVIEW_PERSON_MATCH_CANDIDATE_LIST_CONNECT_PATH_V1: &str =
    "/makosh.review.person_match_candidate.v1.ReviewPersonMatchCandidateQueryService/List";
pub const REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_DECIDE_CONTRACT_NAME_V1: &str =
    "review_person_match_candidate_client_decide";
pub const REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_GET_CONTRACT_NAME_V1: &str =
    "review_person_match_candidate_client_get";
pub const REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_LIST_CONTRACT_NAME_V1: &str =
    "review_person_match_candidate_client_list";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.review.person_match_candidate.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/review_person_match_candidate_schema.rs"
));

pub const REVIEW_PERSON_MATCH_CANDIDATE_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/review-person-match-candidate-v1.bin"
));

#[must_use]
pub fn review_person_match_candidate_contract_reference_v1(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: REVIEW_PERSON_MATCH_CANDIDATE_CONTRACT_MAJOR_V1,
        revision: REVIEW_PERSON_MATCH_CANDIDATE_CONTRACT_REVISION_V1,
        schema_sha256: REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_SHA256_V1.to_vec(),
    }
}

macro_rules! contract_reference {
    ($name:ident, $contract:ident) => {
        #[must_use]
        pub fn $name() -> ContractReferenceV1 {
            review_person_match_candidate_contract_reference_v1($contract)
        }
    };
}

contract_reference!(
    review_person_match_candidate_decision_contract_reference_v1,
    REVIEW_PERSON_MATCH_CANDIDATE_DECISION_CONTRACT_NAME_V1
);
contract_reference!(
    review_person_match_candidate_client_decide_contract_reference_v1,
    REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_DECIDE_CONTRACT_NAME_V1
);
contract_reference!(
    review_person_match_candidate_client_get_contract_reference_v1,
    REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_GET_CONTRACT_NAME_V1
);
contract_reference!(
    review_person_match_candidate_client_list_contract_reference_v1,
    REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_LIST_CONTRACT_NAME_V1
);
contract_reference!(
    review_person_match_candidate_submitted_contract_reference_v1,
    REVIEW_PERSON_MATCH_CANDIDATE_SUBMITTED_CONTRACT_NAME_V1
);
contract_reference!(
    review_person_match_candidate_submission_rejected_contract_reference_v1,
    REVIEW_PERSON_MATCH_CANDIDATE_SUBMISSION_REJECTED_CONTRACT_NAME_V1
);
contract_reference!(
    review_person_match_candidate_approved_contract_reference_v1,
    REVIEW_PERSON_MATCH_CANDIDATE_APPROVED_CONTRACT_NAME_V1
);

#[must_use]
pub fn review_person_match_candidate_decision_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        review_person_match_candidate_decision_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn review_person_match_candidate_approved_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Event,
        review_person_match_candidate_approved_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

macro_rules! publish_request {
    ($name:ident, $kind:expr, $contract:ident) => {
        #[must_use]
        pub fn $name() -> CapabilityRequestV1 {
            event_route(
                $kind,
                $contract(),
                EventRouteDirectionV1::Publish,
                EventSubscriptionRequirementV1::Unspecified,
            )
        }
    };
}

publish_request!(
    review_person_match_candidate_submitted_publish_request_v1,
    DurableEnvelopeKindV1::Event,
    review_person_match_candidate_submitted_contract_reference_v1
);
publish_request!(
    review_person_match_candidate_submission_rejected_publish_request_v1,
    DurableEnvelopeKindV1::Event,
    review_person_match_candidate_submission_rejected_contract_reference_v1
);
publish_request!(
    review_person_match_candidate_approved_publish_request_v1,
    DurableEnvelopeKindV1::Event,
    review_person_match_candidate_approved_contract_reference_v1
);

fn event_route(
    envelope_kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    subscription_requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: envelope_kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: REVIEW_PERSON_MATCH_CANDIDATE_MAX_IN_FLIGHT_V1,
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
    fn schema_is_review_owned_and_contains_only_public_evidence() {
        assert_eq!(REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1, "review");
        assert!(!REVIEW_PERSON_MATCH_CANDIDATE_DESCRIPTOR_SET_V1.is_empty());
        let schema = include_str!(
            "../proto/makosh/review/person_match_candidate/v1/person_match_candidate.proto"
        );
        for required in [
            "PersonMatchCandidateApprovedForPromotionV1",
            "PERSON_MATCH_CANDIDATE_DECISION_APPROVE",
            "PERSON_MATCH_CANDIDATE_DECISION_REJECT",
        ] {
            assert!(schema.contains(required), "missing {required}");
        }
        for forbidden in [
            "normalized_email",
            "normalized_phone",
            "provider_entry_id",
            "provider_etag",
            "continuation_cursor",
            "credential",
            "private_locator",
            "raw_payload",
        ] {
            assert!(!schema.contains(forbidden), "private field {forbidden}");
        }
    }

    #[test]
    fn exact_decision_and_approval_routes_are_bounded() {
        let Some(Request::EventRoute(decision)) =
            review_person_match_candidate_decision_consume_request_v1().request
        else {
            panic!("decision event route")
        };
        assert_eq!(decision.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(decision.max_in_flight, 32);
        let Some(Request::EventRoute(approval)) =
            review_person_match_candidate_approved_publish_request_v1().request
        else {
            panic!("approval event route")
        };
        assert_eq!(approval.direction, EventRouteDirectionV1::Publish as i32);
    }
}
