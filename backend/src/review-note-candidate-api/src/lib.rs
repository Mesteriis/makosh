#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    ReviewNoteCandidateEnvelopeBuildErrorV1, ReviewNoteCandidateEnvelopeContextV1,
    build_review_note_candidate_approved_outbox_record_v1,
    build_review_note_candidate_submission_rejected_outbox_record_v1,
    build_review_note_candidate_submitted_outbox_record_v1,
    build_submit_review_note_candidate_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-review-note-candidate-api";
pub const REVIEW_NOTE_CANDIDATE_OWNER_V1: &str = "review";
pub const REVIEW_NOTE_CANDIDATE_MODULE_ID_V1: &str = "makosh-review-note-candidate-runtime";
pub const REVIEW_NOTE_CANDIDATE_CLIENT_CAPABILITY_ID_V1: &str = "review.note-candidate.client.v1";
pub const REVIEW_NOTE_CANDIDATE_SUBMISSION_CAPABILITY_ID_V1: &str =
    "review.note-candidate.submission.v1";
pub const REVIEW_NOTE_CANDIDATE_PROMOTION_CAPABILITY_ID_V1: &str =
    "review.note-candidate.promotion.v1";
pub const REVIEW_NOTE_CANDIDATE_BLOB_CAPABILITY_ID_V1: &str = "review.note-candidate.blob.v1";
pub const REVIEW_NOTE_CANDIDATE_BLOB_TARGET_OWNER_ID_V1: &str = "review";
pub const REVIEW_NOTE_CANDIDATE_BLOB_TARGET_MODULE_ID_V1: &str = REVIEW_NOTE_CANDIDATE_MODULE_ID_V1;
pub const REVIEW_NOTE_CANDIDATE_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    REVIEW_NOTE_CANDIDATE_BLOB_CAPABILITY_ID_V1;
pub const REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_TARGET_OWNER_ID_V1: &str =
    "reviewed_note_candidate_promotion";
pub const REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_TARGET_MODULE_ID_V1: &str =
    "makosh-reviewed-note-candidate-promotion-runtime";
pub const REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    "reviewed-note-candidate-promotion.source.blob.v1";

pub const REVIEW_NOTE_CANDIDATE_SUBMIT_CONTRACT_NAME_V1: &str = "review_note_candidate_submit";
pub const REVIEW_NOTE_CANDIDATE_SUBMITTED_CONTRACT_NAME_V1: &str =
    "review_note_candidate_submitted";
pub const REVIEW_NOTE_CANDIDATE_SUBMISSION_REJECTED_CONTRACT_NAME_V1: &str =
    "review_note_candidate_submission_rejected";
pub const REVIEW_NOTE_CANDIDATE_APPROVED_CONTRACT_NAME_V1: &str =
    "review_note_candidate_approved_for_promotion";
pub const REVIEW_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1: &str =
    "review.note-candidate.status_changed";
pub const REVIEW_NOTE_CANDIDATE_COMMAND_CONTRACT_NAME_V1: &str = "review.note-candidate.command";
pub const REVIEW_NOTE_CANDIDATE_QUERY_CONTRACT_NAME_V1: &str = "review.note-candidate.query";
pub const REVIEW_NOTE_CANDIDATE_LIST_CONTRACT_NAME_V1: &str = "review.note-candidate.list";
pub const REVIEW_NOTE_CANDIDATE_REALTIME_EVENT_KIND_V1: &str =
    "review.note-candidate.status_changed";

pub const REVIEW_NOTE_CANDIDATE_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.review.note_candidate.v1.ReviewNoteCandidateCommandService/Decide";
pub const REVIEW_NOTE_CANDIDATE_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.review.note_candidate.v1.ReviewNoteCandidateQueryService/Get";
pub const REVIEW_NOTE_CANDIDATE_LIST_CONNECT_PATH_V1: &str =
    "/makosh.review.note_candidate.v1.ReviewNoteCandidateQueryService/List";
pub const REVIEW_NOTE_CANDIDATE_CONTRACT_MAJOR_V1: u32 = 1;
pub const REVIEW_NOTE_CANDIDATE_CONTRACT_REVISION_V1: u32 = 1;
pub const REVIEW_NOTE_CANDIDATE_MAX_BLOB_BYTES_V1: u64 = 16 * 1024;
pub const REVIEW_NOTE_CANDIDATE_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const REVIEW_NOTE_CANDIDATE_MAX_IN_FLIGHT_V1: u32 = 32;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.review.note_candidate.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/review_note_candidate_schema.rs"));

pub const REVIEW_NOTE_CANDIDATE_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/review-note-candidate-v1.bin"));

#[must_use]
pub fn review_note_candidate_submit_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(REVIEW_NOTE_CANDIDATE_SUBMIT_CONTRACT_NAME_V1)
}

#[must_use]
pub fn review_note_candidate_submitted_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(REVIEW_NOTE_CANDIDATE_SUBMITTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn review_note_candidate_submission_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(REVIEW_NOTE_CANDIDATE_SUBMISSION_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn review_note_candidate_approved_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(REVIEW_NOTE_CANDIDATE_APPROVED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn review_note_candidate_submit_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        review_note_candidate_submit_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn review_note_candidate_submit_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        review_note_candidate_submit_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn review_note_candidate_submitted_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        review_note_candidate_submitted_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn review_note_candidate_submitted_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        review_note_candidate_submitted_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn review_note_candidate_submission_rejected_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        review_note_candidate_submission_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn review_note_candidate_submission_rejected_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        review_note_candidate_submission_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn review_note_candidate_approved_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Event,
        review_note_candidate_approved_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn review_note_candidate_approved_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Event,
        review_note_candidate_approved_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: REVIEW_NOTE_CANDIDATE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: REVIEW_NOTE_CANDIDATE_CONTRACT_MAJOR_V1,
        revision: REVIEW_NOTE_CANDIDATE_CONTRACT_REVISION_V1,
        schema_sha256: REVIEW_NOTE_CANDIDATE_SCHEMA_SHA256_V1.to_vec(),
    }
}

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
            max_in_flight: REVIEW_NOTE_CANDIDATE_MAX_IN_FLIGHT_V1,
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
    fn note_candidate_is_an_exact_review_capability() {
        assert_eq!(REVIEW_NOTE_CANDIDATE_OWNER_V1, "review");
        assert_ne!(REVIEW_NOTE_CANDIDATE_MODULE_ID_V1, "makosh-review-runtime");
        assert!(REVIEW_NOTE_CANDIDATE_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(REVIEW_NOTE_CANDIDATE_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let Some(Request::EventRoute(route)) =
            review_note_candidate_submit_consume_request_v1().request
        else {
            panic!("submission route");
        };
        assert_eq!(route.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            route.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
    }

    #[test]
    fn durable_and_realtime_contracts_exclude_private_candidate_text() {
        let source = include_str!("../proto/makosh/review/note_candidate/v1/note_candidate.proto");
        assert!(source.contains("ReviewNoteCandidateContentV1"));
        assert!(source.contains("NoteCandidateApprovedForPromotionV1"));
        let durable = source
            .split("message SubmitNoteCandidateForReviewCommandV1")
            .nth(1)
            .expect("durable section")
            .split("message ReviewNoteCandidateContentV1")
            .next()
            .expect("bounded durable section");
        for forbidden in ["string title", "string excerpt", "topic_hints", "map<"] {
            assert!(
                !durable.contains(forbidden),
                "forbidden durable field {forbidden}"
            );
        }
        let realtime = source
            .split("message ReviewNoteCandidateStatusChangedV1")
            .nth(1)
            .expect("realtime section")
            .split("service ReviewNoteCandidateCommandService")
            .next()
            .expect("bounded realtime section");
        for forbidden in ["title", "excerpt", "topic_hint"] {
            assert!(
                !realtime.contains(forbidden),
                "forbidden realtime field {forbidden}"
            );
        }
        for forbidden in [
            "provider_id",
            "account_id",
            "model_id",
            "prompt",
            "google",
            "telegram",
            "ollama",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden owner coupling {forbidden}"
            );
        }
    }
}
