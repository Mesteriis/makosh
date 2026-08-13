#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-review-person-match-candidate-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PublicPersonSourceIdentityV1 {
    pub integration_public_id: [u8; 16],
    pub account_public_id: [u8; 16],
    pub provider_source_contact_public_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonMatchKindV1 {
    NormalizedEmail,
    NormalizedPhone,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonMatchCandidateEvidenceV1 {
    pub evidence_event_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub logical_owner_id: String,
    pub first_person_id: [u8; 16],
    pub second_person_id: [u8; 16],
    pub first_source: PublicPersonSourceIdentityV1,
    pub second_source: PublicPersonSourceIdentityV1,
    pub match_kind: PersonMatchKindV1,
    pub observed_at_unix_millis: i64,
    pub resulting_owner_revision: u64,
    pub candidate_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SplitProfileFactKindV1 {
    DisplayName,
    GivenName,
    FamilyName,
    Emails,
    Phones,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SplitSourceSelectionV1 {
    pub source: PublicPersonSourceIdentityV1,
    pub expected_source_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersonMatchCandidateApprovedActionV1 {
    Attach {
        from_person_id: [u8; 16],
        expected_from_person_revision: u64,
        to_person_id: [u8; 16],
        expected_to_person_revision: u64,
        source: PublicPersonSourceIdentityV1,
        expected_source_revision: u64,
    },
    Merge {
        source_person_id: [u8; 16],
        expected_source_person_revision: u64,
        target_person_id: [u8; 16],
        expected_target_person_revision: u64,
    },
    Split {
        merged_person_id: [u8; 16],
        expected_merged_person_revision: u64,
        target_person_id: [u8; 16],
        expected_target_person_revision: u64,
        source_selection: Vec<SplitSourceSelectionV1>,
        profile_fact_selection: Vec<SplitProfileFactKindV1>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonMatchCandidateStateV1 {
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonMatchCandidatePromotionStatusV1 {
    NotRequested,
    Pending,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonMatchCandidateReviewV1 {
    pub review_id: [u8; 16],
    pub evidence: PersonMatchCandidateEvidenceV1,
    pub state: PersonMatchCandidateStateV1,
    pub promotion_status: PersonMatchCandidatePromotionStatusV1,
    pub review_revision: u64,
    pub decision_id: Option<[u8; 16]>,
    pub decided_by_owner_device_id: Option<[u8; 16]>,
    pub decided_at_unix_millis: Option<i64>,
    pub approved_action: Option<PersonMatchCandidateApprovedActionV1>,
    pub approved_action_digest: Option<[u8; 32]>,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersonMatchCandidateDecisionV1 {
    Approve {
        action: PersonMatchCandidateApprovedActionV1,
        approved_action_digest: [u8; 32],
    },
    Reject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecidePersonMatchCandidateV1 {
    pub decision_id: [u8; 16],
    pub expected_review_revision: u64,
    pub decision: PersonMatchCandidateDecisionV1,
    pub decided_by_owner_device_id: [u8; 16],
    pub decided_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonMatchCandidateCoreErrorV1 {
    InvalidOwner,
    InvalidId,
    InvalidDigest,
    InvalidRevision,
    InvalidTimestamp,
    InvalidEvidence,
    InvalidAction,
    RevisionConflict,
    TerminalDecision,
    InvalidPromotion,
}

pub fn person_match_candidate_evidence_digest_v1(
    evidence: &PersonMatchCandidateEvidenceV1,
) -> Result<[u8; 32], PersonMatchCandidateCoreErrorV1> {
    validate_owner(&evidence.logical_owner_id)?;
    for id in [
        evidence.evidence_event_id,
        evidence.candidate_id,
        evidence.first_person_id,
        evidence.second_person_id,
    ] {
        require_id(id)?;
    }
    if evidence.first_person_id == evidence.second_person_id
        || evidence.observed_at_unix_millis <= 0
        || evidence.resulting_owner_revision == 0
    {
        return Err(PersonMatchCandidateCoreErrorV1::InvalidEvidence);
    }
    validate_source(evidence.first_source)?;
    validate_source(evidence.second_source)?;
    let mut hash = Sha256::new();
    hash.update(b"makosh.review.person-match-candidate.evidence.v1");
    update_bytes(&mut hash, evidence.logical_owner_id.as_bytes());
    for id in [
        evidence.evidence_event_id,
        evidence.candidate_id,
        evidence.first_person_id,
        evidence.second_person_id,
    ] {
        hash.update(id);
    }
    update_source(&mut hash, evidence.first_source);
    update_source(&mut hash, evidence.second_source);
    hash.update([match evidence.match_kind {
        PersonMatchKindV1::NormalizedEmail => 1,
        PersonMatchKindV1::NormalizedPhone => 2,
    }]);
    hash.update(evidence.observed_at_unix_millis.to_be_bytes());
    hash.update(evidence.resulting_owner_revision.to_be_bytes());
    Ok(hash.finalize().into())
}

pub fn create_person_match_candidate_review_v1(
    evidence: PersonMatchCandidateEvidenceV1,
) -> Result<PersonMatchCandidateReviewV1, PersonMatchCandidateCoreErrorV1> {
    let digest = person_match_candidate_evidence_digest_v1(&evidence)?;
    if evidence.candidate_digest != digest {
        return Err(PersonMatchCandidateCoreErrorV1::InvalidDigest);
    }
    let review_id = derive_id(
        b"makosh.review.person-match-candidate.review.v1",
        evidence.logical_owner_id.as_bytes(),
        &evidence.candidate_id,
    );
    Ok(PersonMatchCandidateReviewV1 {
        review_id,
        updated_at_unix_millis: evidence.observed_at_unix_millis,
        evidence,
        state: PersonMatchCandidateStateV1::Pending,
        promotion_status: PersonMatchCandidatePromotionStatusV1::NotRequested,
        review_revision: 1,
        decision_id: None,
        decided_by_owner_device_id: None,
        decided_at_unix_millis: None,
        approved_action: None,
        approved_action_digest: None,
    })
}

pub fn decide_person_match_candidate_v1(
    current: &PersonMatchCandidateReviewV1,
    input: DecidePersonMatchCandidateV1,
) -> Result<PersonMatchCandidateReviewV1, PersonMatchCandidateCoreErrorV1> {
    validate_review(current)?;
    if current.state != PersonMatchCandidateStateV1::Pending {
        return Err(PersonMatchCandidateCoreErrorV1::TerminalDecision);
    }
    if input.expected_review_revision != current.review_revision {
        return Err(PersonMatchCandidateCoreErrorV1::RevisionConflict);
    }
    require_id(input.decision_id)?;
    require_id(input.decided_by_owner_device_id)?;
    if input.decided_at_unix_millis < current.updated_at_unix_millis {
        return Err(PersonMatchCandidateCoreErrorV1::InvalidTimestamp);
    }
    let mut next = current.clone();
    next.review_revision = next
        .review_revision
        .checked_add(1)
        .ok_or(PersonMatchCandidateCoreErrorV1::InvalidRevision)?;
    next.decision_id = Some(input.decision_id);
    next.decided_by_owner_device_id = Some(input.decided_by_owner_device_id);
    next.decided_at_unix_millis = Some(input.decided_at_unix_millis);
    next.updated_at_unix_millis = input.decided_at_unix_millis;
    match input.decision {
        PersonMatchCandidateDecisionV1::Approve {
            action,
            approved_action_digest,
        } => {
            validate_action(&action)?;
            require_digest(approved_action_digest)?;
            next.state = PersonMatchCandidateStateV1::Approved;
            next.promotion_status = PersonMatchCandidatePromotionStatusV1::Pending;
            next.approved_action = Some(action);
            next.approved_action_digest = Some(approved_action_digest);
        }
        PersonMatchCandidateDecisionV1::Reject => {
            next.state = PersonMatchCandidateStateV1::Rejected;
        }
    }
    validate_review(&next)?;
    Ok(next)
}

pub fn record_person_match_candidate_promotion_v1(
    current: &PersonMatchCandidateReviewV1,
    succeeded: bool,
    occurred_at_unix_millis: i64,
) -> Result<PersonMatchCandidateReviewV1, PersonMatchCandidateCoreErrorV1> {
    validate_review(current)?;
    if current.state != PersonMatchCandidateStateV1::Approved
        || current.promotion_status != PersonMatchCandidatePromotionStatusV1::Pending
        || occurred_at_unix_millis < current.updated_at_unix_millis
    {
        return Err(PersonMatchCandidateCoreErrorV1::InvalidPromotion);
    }
    let mut next = current.clone();
    next.review_revision = next
        .review_revision
        .checked_add(1)
        .ok_or(PersonMatchCandidateCoreErrorV1::InvalidRevision)?;
    next.promotion_status = if succeeded {
        PersonMatchCandidatePromotionStatusV1::Succeeded
    } else {
        PersonMatchCandidatePromotionStatusV1::Failed
    };
    next.updated_at_unix_millis = occurred_at_unix_millis;
    validate_review(&next)?;
    Ok(next)
}

pub fn validate_review(
    review: &PersonMatchCandidateReviewV1,
) -> Result<(), PersonMatchCandidateCoreErrorV1> {
    let digest = person_match_candidate_evidence_digest_v1(&review.evidence)?;
    if review.evidence.candidate_digest != digest
        || review.review_id
            != derive_id(
                b"makosh.review.person-match-candidate.review.v1",
                review.evidence.logical_owner_id.as_bytes(),
                &review.evidence.candidate_id,
            )
        || review.review_revision == 0
        || review.updated_at_unix_millis <= 0
    {
        return Err(PersonMatchCandidateCoreErrorV1::InvalidEvidence);
    }
    let has_decision = review.decision_id.is_some()
        && review.decided_by_owner_device_id.is_some()
        && review.decided_at_unix_millis.is_some();
    match review.state {
        PersonMatchCandidateStateV1::Pending => {
            if has_decision
                || review.promotion_status != PersonMatchCandidatePromotionStatusV1::NotRequested
                || review.approved_action.is_some()
                || review.approved_action_digest.is_some()
            {
                return Err(PersonMatchCandidateCoreErrorV1::InvalidEvidence);
            }
        }
        PersonMatchCandidateStateV1::Rejected => {
            if !has_decision
                || review.promotion_status != PersonMatchCandidatePromotionStatusV1::NotRequested
                || review.approved_action.is_some()
                || review.approved_action_digest.is_some()
            {
                return Err(PersonMatchCandidateCoreErrorV1::InvalidEvidence);
            }
        }
        PersonMatchCandidateStateV1::Approved => {
            if !has_decision
                || review.promotion_status == PersonMatchCandidatePromotionStatusV1::NotRequested
                || review
                    .approved_action
                    .as_ref()
                    .map(validate_action)
                    .transpose()?
                    .is_none()
                || !review.approved_action_digest.is_some_and(valid_digest)
            {
                return Err(PersonMatchCandidateCoreErrorV1::InvalidEvidence);
            }
        }
    }
    Ok(())
}

fn validate_action(
    action: &PersonMatchCandidateApprovedActionV1,
) -> Result<(), PersonMatchCandidateCoreErrorV1> {
    match action {
        PersonMatchCandidateApprovedActionV1::Attach {
            from_person_id,
            expected_from_person_revision,
            to_person_id,
            expected_to_person_revision,
            source,
            expected_source_revision,
        } => {
            require_id(*from_person_id)?;
            require_id(*to_person_id)?;
            validate_source(*source)?;
            if from_person_id == to_person_id
                || *expected_from_person_revision == 0
                || *expected_to_person_revision == 0
                || *expected_source_revision == 0
            {
                return Err(PersonMatchCandidateCoreErrorV1::InvalidAction);
            }
        }
        PersonMatchCandidateApprovedActionV1::Merge {
            source_person_id,
            expected_source_person_revision,
            target_person_id,
            expected_target_person_revision,
        } => {
            require_id(*source_person_id)?;
            require_id(*target_person_id)?;
            if source_person_id == target_person_id
                || *expected_source_person_revision == 0
                || *expected_target_person_revision == 0
            {
                return Err(PersonMatchCandidateCoreErrorV1::InvalidAction);
            }
        }
        PersonMatchCandidateApprovedActionV1::Split {
            merged_person_id,
            expected_merged_person_revision,
            target_person_id,
            expected_target_person_revision,
            source_selection,
            profile_fact_selection,
        } => {
            require_id(*merged_person_id)?;
            require_id(*target_person_id)?;
            if merged_person_id == target_person_id
                || *expected_merged_person_revision == 0
                || *expected_target_person_revision == 0
                || (source_selection.is_empty() && profile_fact_selection.is_empty())
                || source_selection.windows(2).any(|pair| pair[0] >= pair[1])
                || profile_fact_selection
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])
            {
                return Err(PersonMatchCandidateCoreErrorV1::InvalidAction);
            }
            for selected in source_selection {
                validate_source(selected.source)?;
                if selected.expected_source_revision == 0 {
                    return Err(PersonMatchCandidateCoreErrorV1::InvalidAction);
                }
            }
        }
    }
    Ok(())
}

fn validate_source(
    source: PublicPersonSourceIdentityV1,
) -> Result<(), PersonMatchCandidateCoreErrorV1> {
    for id in [
        source.integration_public_id,
        source.account_public_id,
        source.provider_source_contact_public_id,
    ] {
        require_id(id)?;
    }
    Ok(())
}

fn validate_owner(owner: &str) -> Result<(), PersonMatchCandidateCoreErrorV1> {
    if !owner.is_empty()
        && owner.len() <= 128
        && owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Ok(())
    } else {
        Err(PersonMatchCandidateCoreErrorV1::InvalidOwner)
    }
}

fn require_id(id: [u8; 16]) -> Result<(), PersonMatchCandidateCoreErrorV1> {
    if id.iter().any(|byte| *byte != 0) {
        Ok(())
    } else {
        Err(PersonMatchCandidateCoreErrorV1::InvalidId)
    }
}

fn require_digest(digest: [u8; 32]) -> Result<(), PersonMatchCandidateCoreErrorV1> {
    if valid_digest(digest) {
        Ok(())
    } else {
        Err(PersonMatchCandidateCoreErrorV1::InvalidDigest)
    }
}

fn valid_digest(digest: [u8; 32]) -> bool {
    digest.iter().any(|byte| *byte != 0)
}

fn update_source(hash: &mut Sha256, source: PublicPersonSourceIdentityV1) {
    hash.update(source.integration_public_id);
    hash.update(source.account_public_id);
    hash.update(source.provider_source_contact_public_id);
}

fn update_bytes(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}

fn derive_id(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    update_bytes(&mut hash, label);
    update_bytes(&mut hash, first);
    update_bytes(&mut hash, second);
    hash.finalize()[..16].try_into().expect("SHA-256 prefix")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(seed: u8) -> PublicPersonSourceIdentityV1 {
        PublicPersonSourceIdentityV1 {
            integration_public_id: [seed; 16],
            account_public_id: [seed + 1; 16],
            provider_source_contact_public_id: [seed + 2; 16],
        }
    }

    fn evidence() -> PersonMatchCandidateEvidenceV1 {
        let mut value = PersonMatchCandidateEvidenceV1 {
            evidence_event_id: [1; 16],
            candidate_id: [2; 16],
            logical_owner_id: "owner-a".to_owned(),
            first_person_id: [3; 16],
            second_person_id: [4; 16],
            first_source: source(5),
            second_source: source(8),
            match_kind: PersonMatchKindV1::NormalizedEmail,
            observed_at_unix_millis: 1_000,
            resulting_owner_revision: 7,
            candidate_digest: [0; 32],
        };
        value.candidate_digest = person_match_candidate_evidence_digest_v1(&value).expect("digest");
        value
    }

    fn attach() -> PersonMatchCandidateApprovedActionV1 {
        PersonMatchCandidateApprovedActionV1::Attach {
            from_person_id: [3; 16],
            expected_from_person_revision: 4,
            to_person_id: [4; 16],
            expected_to_person_revision: 5,
            source: source(5),
            expected_source_revision: 6,
        }
    }

    #[test]
    fn review_aggregate_id_is_owner_candidate_stable_across_evidence_updates() {
        let first = evidence();
        let mut updated = first.clone();
        updated.evidence_event_id = [9; 16];
        std::mem::swap(&mut updated.first_person_id, &mut updated.second_person_id);
        std::mem::swap(&mut updated.first_source, &mut updated.second_source);
        updated.observed_at_unix_millis += 1;
        updated.resulting_owner_revision += 1;
        updated.candidate_digest =
            person_match_candidate_evidence_digest_v1(&updated).expect("digest");
        assert_ne!(first.candidate_digest, updated.candidate_digest);
        assert_eq!(
            create_person_match_candidate_review_v1(first)
                .expect("first")
                .review_id,
            create_person_match_candidate_review_v1(updated)
                .expect("updated")
                .review_id,
        );
    }

    #[test]
    fn public_evidence_digest_binds_every_id_revision_and_match_kind() {
        let value = evidence();
        let digest = value.candidate_digest;
        for changed in [
            {
                let mut changed = value.clone();
                changed.second_person_id = [9; 16];
                changed
            },
            {
                let mut changed = value.clone();
                changed.resulting_owner_revision += 1;
                changed
            },
            {
                let mut changed = value.clone();
                changed.match_kind = PersonMatchKindV1::NormalizedPhone;
                changed
            },
        ] {
            assert_ne!(
                person_match_candidate_evidence_digest_v1(&changed).expect("digest"),
                digest
            );
        }
    }

    #[test]
    fn approval_is_terminal_and_rejection_never_requests_promotion() {
        let pending = create_person_match_candidate_review_v1(evidence()).expect("pending");
        let approved = decide_person_match_candidate_v1(
            &pending,
            DecidePersonMatchCandidateV1 {
                decision_id: [20; 16],
                expected_review_revision: 1,
                decision: PersonMatchCandidateDecisionV1::Approve {
                    action: attach(),
                    approved_action_digest: [21; 32],
                },
                decided_by_owner_device_id: [22; 16],
                decided_at_unix_millis: 1_001,
            },
        )
        .expect("approve");
        assert_eq!(approved.state, PersonMatchCandidateStateV1::Approved);
        assert_eq!(
            approved.promotion_status,
            PersonMatchCandidatePromotionStatusV1::Pending
        );
        assert_eq!(
            decide_person_match_candidate_v1(
                &approved,
                DecidePersonMatchCandidateV1 {
                    decision_id: [23; 16],
                    expected_review_revision: 2,
                    decision: PersonMatchCandidateDecisionV1::Reject,
                    decided_by_owner_device_id: [22; 16],
                    decided_at_unix_millis: 1_002,
                },
            ),
            Err(PersonMatchCandidateCoreErrorV1::TerminalDecision)
        );

        let rejected = decide_person_match_candidate_v1(
            &pending,
            DecidePersonMatchCandidateV1 {
                decision_id: [24; 16],
                expected_review_revision: 1,
                decision: PersonMatchCandidateDecisionV1::Reject,
                decided_by_owner_device_id: [22; 16],
                decided_at_unix_millis: 1_001,
            },
        )
        .expect("reject");
        assert_eq!(
            rejected.promotion_status,
            PersonMatchCandidatePromotionStatusV1::NotRequested
        );
        assert!(rejected.approved_action.is_none());
    }

    #[test]
    fn approve_requires_nonzero_digest_and_exact_revision() {
        let pending = create_person_match_candidate_review_v1(evidence()).expect("pending");
        let invalid = DecidePersonMatchCandidateV1 {
            decision_id: [20; 16],
            expected_review_revision: 2,
            decision: PersonMatchCandidateDecisionV1::Approve {
                action: attach(),
                approved_action_digest: [0; 32],
            },
            decided_by_owner_device_id: [22; 16],
            decided_at_unix_millis: 1_001,
        };
        assert_eq!(
            decide_person_match_candidate_v1(&pending, invalid),
            Err(PersonMatchCandidateCoreErrorV1::RevisionConflict)
        );
    }
}
