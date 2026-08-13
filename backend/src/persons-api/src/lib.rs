#![forbid(unsafe_code)]

mod action_digest;
pub use action_digest::{
    PersonsActionDigestErrorV1, PersonsActionDigestSourceV1, PersonsActionDigestSplitSourceV1,
    PersonsIdentityMatchKindV1, persons_attach_source_action_digest_v1,
    persons_confirmed_action_command_id_v1, persons_identity_match_candidate_id_v1,
    persons_merge_action_digest_v1, persons_owner_partition_id_v1, persons_split_action_digest_v1,
};

pub const PACKAGE: &str = "makosh-persons-api";
pub const PERSONS_OWNER_ID_V1: &str = "persons";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const PERSONS_MODULE_ID_V1: &str = "makosh-persons-runtime";
pub const PERSONS_COMMAND_CAPABILITY_ID_V1: &str = "persons.command.v1";
pub const PERSONS_COMMAND_SUCCEEDED_CAPABILITY_ID_V1: &str = "persons.command-succeeded.v1";
pub const PERSONS_COMMAND_REJECTED_CAPABILITY_ID_V1: &str = "persons.command-rejected.v1";
pub const PERSONS_OWNER_EVENT_CAPABILITY_ID_V1: &str = "persons.owner-event.v1";
pub const PERSONS_REVIEW_CANDIDATE_CAPABILITY_ID_V1: &str = "persons.review-candidate.v1";
pub const PERSONS_COMMAND_CONTRACT_NAME_V1: &str = "persons_command";
pub const PERSONS_COMMAND_SUCCEEDED_CONTRACT_NAME_V1: &str = "persons_command_succeeded";
pub const PERSONS_COMMAND_REJECTED_CONTRACT_NAME_V1: &str = "persons_command_rejected";
pub const PERSONS_OWNER_EVENT_CONTRACT_NAME_V1: &str = "persons_owner_event";
pub const PERSONS_REVIEW_CANDIDATE_CONTRACT_NAME_V1: &str = "persons_review_candidate_raised";
pub const PERSONS_CONTRACT_MAJOR_V1: u32 = 1;
pub const PERSONS_CONTRACT_REVISION_V1: u32 = 1;
pub const PERSONS_COMMAND_MAX_IN_FLIGHT_V1: u32 = 16;
pub const PERSONS_CLIENT_CAPABILITY_ID_V1: &str = "persons.client.v1";
pub const PERSONS_CREATE_CONNECT_PATH_V1: &str = "/makosh.persons.v1.PersonsCommandService/Create";
pub const PERSONS_UPDATE_OWNER_PROFILE_CONNECT_PATH_V1: &str =
    "/makosh.persons.v1.PersonsCommandService/UpdateOwnerProfile";
pub const PERSONS_LIST_DIRECTORY_CONNECT_PATH_V1: &str =
    "/makosh.persons.v1.PersonsQueryService/ListDirectory";
pub const PERSONS_GET_PROFILE_CONNECT_PATH_V1: &str =
    "/makosh.persons.v1.PersonsQueryService/GetProfile";
pub const PERSONS_LIST_SOURCE_LINKS_CONNECT_PATH_V1: &str =
    "/makosh.persons.v1.PersonsQueryService/ListSourceLinks";
pub const PERSONS_CLIENT_CREATE_CONTRACT_NAME_V1: &str = "persons_client_create";
pub const PERSONS_CLIENT_UPDATE_PROFILE_CONTRACT_NAME_V1: &str = "persons_client_update_profile";
pub const PERSONS_CLIENT_LIST_DIRECTORY_CONTRACT_NAME_V1: &str = "persons_client_list_directory";
pub const PERSONS_CLIENT_GET_PROFILE_CONTRACT_NAME_V1: &str = "persons_client_get_profile";
pub const PERSONS_CLIENT_LIST_SOURCE_LINKS_CONTRACT_NAME_V1: &str =
    "persons_client_list_source_links";

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.persons.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/persons_schema.rs"));

pub const PERSONS_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/persons-v1.bin"));

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

macro_rules! contract_reference {
    ($name:ident, $contract:ident) => {
        #[must_use]
        pub fn $name() -> ContractReferenceV1 {
            ContractReferenceV1 {
                owner: PERSONS_OWNER_ID_V1.to_owned(),
                name: $contract.to_owned(),
                major: PERSONS_CONTRACT_MAJOR_V1,
                revision: PERSONS_CONTRACT_REVISION_V1,
                schema_sha256: PERSONS_SCHEMA_SHA256_V1.to_vec(),
            }
        }
    };
}

contract_reference!(
    persons_command_contract_reference_v1,
    PERSONS_COMMAND_CONTRACT_NAME_V1
);
contract_reference!(
    persons_client_create_contract_reference_v1,
    PERSONS_CLIENT_CREATE_CONTRACT_NAME_V1
);
contract_reference!(
    persons_client_update_profile_contract_reference_v1,
    PERSONS_CLIENT_UPDATE_PROFILE_CONTRACT_NAME_V1
);
contract_reference!(
    persons_client_list_directory_contract_reference_v1,
    PERSONS_CLIENT_LIST_DIRECTORY_CONTRACT_NAME_V1
);
contract_reference!(
    persons_client_get_profile_contract_reference_v1,
    PERSONS_CLIENT_GET_PROFILE_CONTRACT_NAME_V1
);
contract_reference!(
    persons_client_list_source_links_contract_reference_v1,
    PERSONS_CLIENT_LIST_SOURCE_LINKS_CONTRACT_NAME_V1
);
contract_reference!(
    persons_command_succeeded_contract_reference_v1,
    PERSONS_COMMAND_SUCCEEDED_CONTRACT_NAME_V1
);
contract_reference!(
    persons_command_rejected_contract_reference_v1,
    PERSONS_COMMAND_REJECTED_CONTRACT_NAME_V1
);
contract_reference!(
    persons_owner_event_contract_reference_v1,
    PERSONS_OWNER_EVENT_CONTRACT_NAME_V1
);
contract_reference!(
    persons_review_candidate_contract_reference_v1,
    PERSONS_REVIEW_CANDIDATE_CONTRACT_NAME_V1
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
            max_in_flight: PERSONS_COMMAND_MAX_IN_FLIGHT_V1,
            subscription_requirement: subscription_requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

#[must_use]
pub fn persons_command_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        persons_command_contract_reference_v1(),
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
    persons_command_succeeded_publish_request_v1,
    DurableEnvelopeKindV1::Result,
    persons_command_succeeded_contract_reference_v1
);
publish_request!(
    persons_command_rejected_publish_request_v1,
    DurableEnvelopeKindV1::Result,
    persons_command_rejected_contract_reference_v1
);
publish_request!(
    persons_owner_event_publish_request_v1,
    DurableEnvelopeKindV1::Event,
    persons_owner_event_contract_reference_v1
);
publish_request!(
    persons_review_candidate_publish_request_v1,
    DurableEnvelopeKindV1::Event,
    persons_review_candidate_contract_reference_v1
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_digest_is_exact_and_nonzero() {
        assert_eq!(PERSONS_SCHEMA_SHA256_V1.len(), DIGEST_BYTES_V1);
        assert!(PERSONS_SCHEMA_SHA256_V1.iter().any(|byte| *byte != 0));
        assert!(!PERSONS_DESCRIPTOR_SET_V1.is_empty());
    }

    #[test]
    fn source_identity_and_review_candidate_are_public_and_bounded() {
        let source = include_str!("../proto/makosh/persons/v1/persons.proto");
        let schema_without_comments = source
            .lines()
            .map(|line| line.split("//").next().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");
        for forbidden in [
            "credential",
            "session",
            "token",
            "raw_payload",
            "private_locator",
            "map<",
            "json",
            "error_detail",
            "presentation_label",
        ] {
            assert!(
                !schema_without_comments.to_lowercase().contains(forbidden),
                "{forbidden}"
            );
        }
        let candidate = source
            .split("message PersonReviewCandidateRaisedEventV1")
            .nth(1)
            .and_then(|section| section.split("enum PersonLifecycleV1").next())
            .expect("candidate schema");
        assert!(candidate.contains("IdentityMatchKindV1 match_kind"));
        assert!(!candidate.contains("normalized_email"));
        assert!(!candidate.contains("normalized_phone"));
        assert!(!candidate.contains("merge_authorized"));
    }

    #[test]
    fn contract_exposes_every_required_typed_surface() {
        let source = include_str!("../proto/makosh/persons/v1/persons.proto");
        for message in [
            "ManualCreatePersonCommandV1",
            "UpdatePersonOwnerProfileCommandV1",
            "ObserveProviderSourceContactCommandV1",
            "UpdateProviderSourceContactCommandV1",
            "RemoveProviderSourceContactCommandV1",
            "ConfirmAttachPersonSourceCommandV1",
            "ConfirmDetachPersonSourceCommandV1",
            "ConfirmMergePersonsCommandV1",
            "ConfirmSplitPersonCommandV1",
            "ReadPersonDirectoryRequestV1",
            "ReadPersonProfileRequestV1",
            "ReadPersonSourceLinksRequestV1",
            "PersonCommandSucceededV1",
            "PersonCommandRejectedV1",
            "PersonChangedEventV1",
            "PersonProfileChangedEventV1",
            "PersonSourceLinkChangedEventV1",
            "PersonLineageChangedEventV1",
            "PersonReviewCandidateRaisedEventV1",
        ] {
            assert!(source.contains(&format!("message {message}")), "{message}");
        }
    }

    #[test]
    fn confirmed_actions_bind_exact_snapshots_and_report_each_person_revision() {
        let source = include_str!("../proto/makosh/persons/v1/persons.proto");
        for field in [
            "expected_from_person_revision",
            "expected_to_person_revision",
            "expected_person_revision",
            "expected_source_person_revision",
            "expected_target_person_revision",
            "expected_merged_person_revision",
            "expected_source_revision",
            "approved_action_digest",
        ] {
            assert!(source.contains(field), "missing {field}");
        }
        assert!(source.contains("message SplitPersonSourceSelectionV1"));
        assert!(source.contains("enum SplitProfileFactKindV1"));
        assert!(source.contains("repeated SplitPersonSourceSelectionV1 source_selection"));
        assert!(source.contains("repeated SplitProfileFactKindV1 profile_fact_selection"));
        assert!(source.contains("message PersonRevisionV1"));
        assert!(source.contains("repeated PersonRevisionV1 resulting_person_revisions"));
        assert!(!source.contains("uint64 resulting_person_revision"));
    }

    #[test]
    fn client_services_have_exact_connect_paths() {
        let schema = include_str!("../proto/makosh/persons/v1/persons.proto");
        assert!(schema.contains("service PersonsCommandService"));
        assert!(schema.contains("service PersonsQueryService"));
        for path in [
            PERSONS_CREATE_CONNECT_PATH_V1,
            PERSONS_UPDATE_OWNER_PROFILE_CONNECT_PATH_V1,
            PERSONS_LIST_DIRECTORY_CONNECT_PATH_V1,
            PERSONS_GET_PROFILE_CONNECT_PATH_V1,
            PERSONS_LIST_SOURCE_LINKS_CONNECT_PATH_V1,
        ] {
            assert!(path.starts_with("/makosh.persons.v1."));
        }
    }
}
