#![forbid(unsafe_code)]

use makosh_identity_resolution_api::{
    identity_resolution_proposal_event_id_v1,
    wire::{
        IdentityMatchKindV1 as WireMatchKind, PersonLinkMergeCandidateProposedEventV1,
        PublicPersonSourceIdentityV1,
    },
};
use makosh_persons_api::{
    PersonsActionDigestSourceV1, PersonsIdentityMatchKindV1, persons_identity_match_candidate_id_v1,
};

pub const PACKAGE: &str = "makosh-identity-resolution-core";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityResolutionMatchKindV1 {
    NormalizedEmail,
    NormalizedPhone,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IdentityResolutionSourceV1 {
    pub integration_public_id: [u8; 16],
    pub account_public_id: [u8; 16],
    pub provider_source_contact_public_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityMatchEvidenceV1 {
    pub evidence_event_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub logical_owner_id: String,
    pub first_person_id: [u8; 16],
    pub second_person_id: [u8; 16],
    pub first_source: IdentityResolutionSourceV1,
    pub second_source: IdentityResolutionSourceV1,
    pub match_kind: IdentityResolutionMatchKindV1,
    pub observed_at_unix_millis: i64,
    pub resulting_owner_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityResolutionCoreErrorV1 {
    InvalidIdentity,
    InvalidOwner,
    InvalidTime,
    InvalidRevision,
    CandidateMismatch,
}

pub fn propose_person_link_merge_candidate_v1(
    evidence: &IdentityMatchEvidenceV1,
) -> Result<PersonLinkMergeCandidateProposedEventV1, IdentityResolutionCoreErrorV1> {
    validate_identity_match_evidence_v1(evidence)?;
    Ok(PersonLinkMergeCandidateProposedEventV1 {
        event_id: identity_resolution_proposal_event_id_v1(
            evidence.evidence_event_id,
            evidence.candidate_id,
        )
        .to_vec(),
        evidence_event_id: evidence.evidence_event_id.to_vec(),
        candidate_id: evidence.candidate_id.to_vec(),
        logical_owner_id: evidence.logical_owner_id.clone(),
        first_person_id: evidence.first_person_id.to_vec(),
        second_person_id: evidence.second_person_id.to_vec(),
        first_source: Some(wire_source(evidence.first_source)),
        second_source: Some(wire_source(evidence.second_source)),
        match_kind: match evidence.match_kind {
            IdentityResolutionMatchKindV1::NormalizedEmail => {
                WireMatchKind::IdentityMatchKindNormalizedEmail
            }
            IdentityResolutionMatchKindV1::NormalizedPhone => {
                WireMatchKind::IdentityMatchKindNormalizedPhone
            }
        } as i32,
        observed_at_unix_millis: evidence.observed_at_unix_millis,
        resulting_owner_revision: evidence.resulting_owner_revision,
    })
}

pub fn validate_identity_match_evidence_v1(
    evidence: &IdentityMatchEvidenceV1,
) -> Result<(), IdentityResolutionCoreErrorV1> {
    for id in [
        evidence.evidence_event_id,
        evidence.candidate_id,
        evidence.first_person_id,
        evidence.second_person_id,
        evidence.first_source.integration_public_id,
        evidence.first_source.account_public_id,
        evidence.first_source.provider_source_contact_public_id,
        evidence.second_source.integration_public_id,
        evidence.second_source.account_public_id,
        evidence.second_source.provider_source_contact_public_id,
    ] {
        if id.iter().all(|value| *value == 0) {
            return Err(IdentityResolutionCoreErrorV1::InvalidIdentity);
        }
    }
    if evidence.first_person_id == evidence.second_person_id {
        return Err(IdentityResolutionCoreErrorV1::InvalidIdentity);
    }
    if evidence.logical_owner_id.is_empty()
        || evidence.logical_owner_id.len() > 128
        || !evidence.logical_owner_id.bytes().all(|value| {
            value.is_ascii_lowercase()
                || value.is_ascii_digit()
                || matches!(value, b'.' | b'_' | b'-')
        })
    {
        return Err(IdentityResolutionCoreErrorV1::InvalidOwner);
    }
    if evidence.observed_at_unix_millis <= 0 {
        return Err(IdentityResolutionCoreErrorV1::InvalidTime);
    }
    if evidence.resulting_owner_revision == 0 {
        return Err(IdentityResolutionCoreErrorV1::InvalidRevision);
    }
    let expected = persons_identity_match_candidate_id_v1(
        &evidence.logical_owner_id,
        api_source(evidence.first_source),
        api_source(evidence.second_source),
        match evidence.match_kind {
            IdentityResolutionMatchKindV1::NormalizedEmail => {
                PersonsIdentityMatchKindV1::NormalizedEmail
            }
            IdentityResolutionMatchKindV1::NormalizedPhone => {
                PersonsIdentityMatchKindV1::NormalizedPhone
            }
        },
    )
    .map_err(|_| IdentityResolutionCoreErrorV1::CandidateMismatch)?;
    if expected != evidence.candidate_id {
        return Err(IdentityResolutionCoreErrorV1::CandidateMismatch);
    }
    Ok(())
}

const fn api_source(value: IdentityResolutionSourceV1) -> PersonsActionDigestSourceV1 {
    PersonsActionDigestSourceV1 {
        integration_public_id: value.integration_public_id,
        account_public_id: value.account_public_id,
        provider_source_contact_public_id: value.provider_source_contact_public_id,
    }
}

fn wire_source(value: IdentityResolutionSourceV1) -> PublicPersonSourceIdentityV1 {
    PublicPersonSourceIdentityV1 {
        integration_public_id: value.integration_public_id.to_vec(),
        account_public_id: value.account_public_id.to_vec(),
        provider_source_contact_public_id: value.provider_source_contact_public_id.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(marker: u8) -> IdentityResolutionSourceV1 {
        IdentityResolutionSourceV1 {
            integration_public_id: [marker; 16],
            account_public_id: [marker + 1; 16],
            provider_source_contact_public_id: [marker + 2; 16],
        }
    }

    fn evidence() -> IdentityMatchEvidenceV1 {
        let first = source(4);
        let second = source(8);
        let candidate_id = persons_identity_match_candidate_id_v1(
            "owner-1",
            api_source(first),
            api_source(second),
            PersonsIdentityMatchKindV1::NormalizedEmail,
        )
        .expect("candidate");
        IdentityMatchEvidenceV1 {
            evidence_event_id: [1; 16],
            candidate_id,
            logical_owner_id: "owner-1".to_owned(),
            first_person_id: [2; 16],
            second_person_id: [3; 16],
            first_source: first,
            second_source: second,
            match_kind: IdentityResolutionMatchKindV1::NormalizedEmail,
            observed_at_unix_millis: 1_800_000_000_000,
            resulting_owner_revision: 7,
        }
    }

    #[test]
    fn proposal_is_deterministic_and_private_free() {
        assert_eq!(
            propose_person_link_merge_candidate_v1(&evidence()).expect("first"),
            propose_person_link_merge_candidate_v1(&evidence()).expect("second"),
        );
    }

    #[test]
    fn candidate_digest_mismatch_is_rejected() {
        let mut value = evidence();
        value.candidate_id[0] ^= 1;
        assert_eq!(
            validate_identity_match_evidence_v1(&value),
            Err(IdentityResolutionCoreErrorV1::CandidateMismatch)
        );
    }

    #[test]
    fn source_order_does_not_change_canonical_candidate() {
        let value = evidence();
        assert_eq!(
            persons_identity_match_candidate_id_v1(
                &value.logical_owner_id,
                api_source(value.first_source),
                api_source(value.second_source),
                PersonsIdentityMatchKindV1::NormalizedEmail
            ),
            persons_identity_match_candidate_id_v1(
                &value.logical_owner_id,
                api_source(value.second_source),
                api_source(value.first_source),
                PersonsIdentityMatchKindV1::NormalizedEmail
            ),
        );
    }
}
