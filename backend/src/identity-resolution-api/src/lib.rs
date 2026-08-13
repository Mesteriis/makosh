#![forbid(unsafe_code)]

mod envelope;
pub use envelope::{
    IdentityResolutionEnvelopeBuildErrorV1, IdentityResolutionEnvelopeContextV1,
    build_identity_resolution_person_match_candidate_outbox_record_v1,
};

pub const PACKAGE: &str = "makosh-identity-resolution-api";
pub const IDENTITY_RESOLUTION_OWNER_ID_V1: &str = "identity_resolution";
pub const IDENTITY_RESOLUTION_MODULE_ID_V1: &str = "makosh-identity-resolution-runtime";
pub const IDENTITY_RESOLUTION_PERSONS_EVIDENCE_CAPABILITY_ID_V1: &str =
    "identity_resolution.persons-evidence.consumer.v1";
pub const IDENTITY_RESOLUTION_REVIEW_CANDIDATE_CAPABILITY_ID_V1: &str =
    "identity_resolution.review-candidate.publisher.v1";
pub const IDENTITY_RESOLUTION_STORAGE_CAPABILITY_ID_V1: &str = "identity_resolution.storage.v1";
pub const IDENTITY_RESOLUTION_PERSON_MATCH_CANDIDATE_CONTRACT_NAME_V1: &str =
    "identity_resolution_person_match_candidate_proposed";
pub const IDENTITY_RESOLUTION_CONTRACT_MAJOR_V1: u32 = 1;
pub const IDENTITY_RESOLUTION_CONTRACT_REVISION_V1: u32 = 1;

#[must_use]
pub fn identity_resolution_proposal_event_id_v1(
    evidence_event_id: [u8; 16],
    candidate_id: [u8; 16],
) -> [u8; 16] {
    use sha2::{Digest, Sha256};

    let mut hash = Sha256::new();
    for value in [
        b"makosh.identity-resolution.proposal.v1".as_slice(),
        evidence_event_id.as_slice(),
        candidate_id.as_slice(),
    ] {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value);
    }
    hash.finalize()[..16].try_into().expect("SHA-256 prefix")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityResolutionPartitionErrorV1 {
    InvalidOwner,
}

pub fn identity_resolution_owner_partition_id_v1(
    owner: &str,
) -> Result<[u8; 16], IdentityResolutionPartitionErrorV1> {
    use sha2::{Digest, Sha256};
    if owner.is_empty()
        || owner.len() > 128
        || !owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        return Err(IdentityResolutionPartitionErrorV1::InvalidOwner);
    }
    let mut hash = Sha256::new();
    for value in [
        b"identity-resolution-owner.v1".as_slice(),
        owner.as_bytes(),
        b"".as_slice(),
    ] {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value);
    }
    Ok(hash.finalize()[..16].try_into().expect("SHA-256 prefix"))
}

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.identity_resolution.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/identity_resolution_schema.rs"));

pub const IDENTITY_RESOLUTION_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/identity-resolution-v1.bin"));

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

#[must_use]
pub fn identity_resolution_person_match_candidate_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: IDENTITY_RESOLUTION_OWNER_ID_V1.to_owned(),
        name: IDENTITY_RESOLUTION_PERSON_MATCH_CANDIDATE_CONTRACT_NAME_V1.to_owned(),
        major: IDENTITY_RESOLUTION_CONTRACT_MAJOR_V1,
        revision: IDENTITY_RESOLUTION_CONTRACT_REVISION_V1,
        schema_sha256: IDENTITY_RESOLUTION_SCHEMA_SHA256_V1.to_vec(),
    }
}

#[must_use]
pub fn identity_resolution_person_match_candidate_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(identity_resolution_person_match_candidate_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
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
    fn proposal_contract_is_private_free_and_exact() {
        assert!(
            IDENTITY_RESOLUTION_SCHEMA_SHA256_V1
                .iter()
                .any(|value| *value != 0)
        );
        let schema =
            include_str!("../proto/makosh/identity_resolution/v1/identity_resolution.proto");
        for forbidden in [
            "credential",
            "provider_payload",
            "private_locator",
            "confidence",
            "risk_score",
            "map<",
            "json",
        ] {
            assert!(
                !schema.to_ascii_lowercase().contains(forbidden),
                "{forbidden}"
            );
        }
    }
}
