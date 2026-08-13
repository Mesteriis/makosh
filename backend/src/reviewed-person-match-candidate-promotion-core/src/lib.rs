#![forbid(unsafe_code)]

use makosh_persons_api::wire::{
    self as persons_wire, ConfirmAttachPersonSourceCommandV1, ConfirmMergePersonsCommandV1,
    ConfirmSplitPersonCommandV1, DecisionProvenanceV1, PersonsCommandV1, ProviderSourceIdentityV1,
    SplitPersonSourceSelectionV1, TimestampV1, persons_command_v1::Command,
};
use makosh_persons_api::{
    PersonsActionDigestSourceV1, PersonsActionDigestSplitSourceV1,
    persons_attach_source_action_digest_v1, persons_confirmed_action_command_id_v1,
    persons_merge_action_digest_v1, persons_split_action_digest_v1,
};
use makosh_review_person_match_candidate_api::wire::{
    PersonMatchCandidateApprovedForPromotionV1, PublicPersonSourceIdentityV1,
    person_match_candidate_approved_action_v1::Action as ReviewAction,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-reviewed-person-match-candidate-promotion-core";

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewedPersonMatchCandidatePromotionPlanV1 {
    pub logical_owner_id: String,
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub decision_id: [u8; 16],
    pub decision_revision: u64,
    pub approved_action_digest: [u8; 32],
    pub persons_command_id: [u8; 16],
    pub persons_command_fingerprint: [u8; 32],
    pub persons_command: PersonsCommandV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedPersonMatchCandidatePromotionCoreErrorV1 {
    InvalidOwner,
    InvalidId,
    InvalidDigest,
    InvalidRevision,
    InvalidTimestamp,
    InvalidAction,
    ActionDigestMismatch,
}

pub fn plan_reviewed_person_match_candidate_promotion_v1(
    approved: &PersonMatchCandidateApprovedForPromotionV1,
) -> Result<
    ReviewedPersonMatchCandidatePromotionPlanV1,
    ReviewedPersonMatchCandidatePromotionCoreErrorV1,
> {
    validate_owner(&approved.logical_owner_id)?;
    let review_id = id16(&approved.review_id)?;
    let candidate_id = id16(&approved.candidate_id)?;
    let decision_id = id16(&approved.decision_id)?;
    let decided_by_owner_device_id = id16(&approved.decided_by_owner_device_id)?;
    let candidate_digest = id32(&approved.candidate_digest)?;
    let approved_digest = id32(&approved.approved_action_digest)?;
    if approved.decision_revision == 0 || approved.decided_at_unix_millis <= 0 {
        return Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidRevision);
    }
    let action = approved
        .approved_action
        .as_ref()
        .and_then(|value| value.action.as_ref())
        .ok_or(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction)?;
    let persons_command_id =
        persons_confirmed_action_command_id_v1(decision_id, approved_digest)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidId)?;
    let decision = DecisionProvenanceV1 {
        decision_id: decision_id.to_vec(),
        review_id: review_id.to_vec(),
        decision_revision: approved.decision_revision,
        decided_by_owner_device_id: decided_by_owner_device_id.to_vec(),
        decided_at: Some(timestamp(approved.decided_at_unix_millis)?),
        approved_action_digest: approved_digest.to_vec(),
    };
    let (actual_digest, command) = match action {
        ReviewAction::Attach(value) => {
            let source = wire_source(
                value
                    .source
                    .as_ref()
                    .ok_or(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction)?,
            )?;
            let from_person_id = id16(&value.from_person_id)?;
            let to_person_id = id16(&value.to_person_id)?;
            require_revision(value.expected_from_person_revision)?;
            require_revision(value.expected_to_person_revision)?;
            require_revision(value.expected_source_revision)?;
            let digest = persons_attach_source_action_digest_v1(
                &approved.logical_owner_id,
                from_person_id,
                value.expected_from_person_revision,
                to_person_id,
                value.expected_to_person_revision,
                digest_source(&source),
                value.expected_source_revision,
            )
            .map_err(|_| ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction)?;
            let command = Command::ConfirmedAttach(ConfirmAttachPersonSourceCommandV1 {
                command_id: persons_command_id.to_vec(),
                from_person_id: from_person_id.to_vec(),
                to_person_id: to_person_id.to_vec(),
                logical_owner_id: approved.logical_owner_id.clone(),
                source: Some(source),
                decision: Some(decision.clone()),
                expected_from_person_revision: value.expected_from_person_revision,
                expected_to_person_revision: value.expected_to_person_revision,
                expected_source_revision: value.expected_source_revision,
            });
            (digest, command)
        }
        ReviewAction::Merge(value) => {
            let source_person_id = id16(&value.source_person_id)?;
            let target_person_id = id16(&value.target_person_id)?;
            require_revision(value.expected_source_person_revision)?;
            require_revision(value.expected_target_person_revision)?;
            let digest = persons_merge_action_digest_v1(
                &approved.logical_owner_id,
                source_person_id,
                value.expected_source_person_revision,
                target_person_id,
                value.expected_target_person_revision,
            )
            .map_err(|_| ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction)?;
            let command = Command::ConfirmedMerge(ConfirmMergePersonsCommandV1 {
                command_id: persons_command_id.to_vec(),
                source_person_id: source_person_id.to_vec(),
                target_person_id: target_person_id.to_vec(),
                logical_owner_id: approved.logical_owner_id.clone(),
                decision: Some(decision.clone()),
                expected_source_person_revision: value.expected_source_person_revision,
                expected_target_person_revision: value.expected_target_person_revision,
            });
            (digest, command)
        }
        ReviewAction::Split(value) => {
            let mut sources = value
                .source_selection
                .iter()
                .map(|selected| {
                    require_revision(selected.expected_source_revision)?;
                    Ok(SplitPersonSourceSelectionV1 {
                        source: Some(wire_source(selected.source.as_ref().ok_or(
                            ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction,
                        )?)?),
                        expected_source_revision: selected.expected_source_revision,
                    })
                })
                .collect::<Result<Vec<_>, ReviewedPersonMatchCandidatePromotionCoreErrorV1>>()?;
            sources.sort_by_key(source_tuple);
            if sources
                .windows(2)
                .any(|pair| source_tuple(&pair[0]) == source_tuple(&pair[1]))
            {
                return Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction);
            }
            let mut facts = value
                .profile_fact_selection
                .iter()
                .map(
                    |value| match persons_wire::SplitProfileFactKindV1::try_from(*value) {
                        Ok(
                            persons_wire::SplitProfileFactKindV1::SplitProfileFactKindDisplayName,
                        ) => Ok(1),
                        Ok(persons_wire::SplitProfileFactKindV1::SplitProfileFactKindGivenName) => {
                            Ok(2)
                        }
                        Ok(
                            persons_wire::SplitProfileFactKindV1::SplitProfileFactKindFamilyName,
                        ) => Ok(3),
                        Ok(persons_wire::SplitProfileFactKindV1::SplitProfileFactKindEmails) => {
                            Ok(4)
                        }
                        Ok(persons_wire::SplitProfileFactKindV1::SplitProfileFactKindPhones) => {
                            Ok(5)
                        }
                        _ => Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction),
                    },
                )
                .collect::<Result<Vec<_>, _>>()?;
            facts.sort();
            if facts.windows(2).any(|pair| pair[0] == pair[1])
                || (sources.is_empty() && facts.is_empty())
            {
                return Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction);
            }
            let merged_person_id = id16(&value.merged_person_id)?;
            let target_person_id = id16(&value.target_person_id)?;
            require_revision(value.expected_merged_person_revision)?;
            require_revision(value.expected_target_person_revision)?;
            let digest_sources = sources
                .iter()
                .map(|value| PersonsActionDigestSplitSourceV1 {
                    source: digest_source(value.source.as_ref().expect("validated")),
                    expected_source_revision: value.expected_source_revision,
                })
                .collect::<Vec<_>>();
            let fact_tags = facts
                .iter()
                .map(|value| u8::try_from(*value).expect("validated"))
                .collect::<Vec<_>>();
            let digest = persons_split_action_digest_v1(
                &approved.logical_owner_id,
                merged_person_id,
                value.expected_merged_person_revision,
                target_person_id,
                value.expected_target_person_revision,
                &digest_sources,
                &fact_tags,
            )
            .map_err(|_| ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidAction)?;
            let command = Command::ConfirmedSplit(ConfirmSplitPersonCommandV1 {
                command_id: persons_command_id.to_vec(),
                merged_person_id: merged_person_id.to_vec(),
                logical_owner_id: approved.logical_owner_id.clone(),
                target_person_id: target_person_id.to_vec(),
                expected_merged_person_revision: value.expected_merged_person_revision,
                expected_target_person_revision: value.expected_target_person_revision,
                source_selection: sources,
                profile_fact_selection: facts,
                decision: Some(decision.clone()),
            });
            (digest, command)
        }
    };
    if actual_digest != approved_digest {
        return Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::ActionDigestMismatch);
    }
    let persons_command = PersonsCommandV1 {
        command: Some(command),
    };
    let persons_command_fingerprint = Sha256::digest(persons_command.encode_to_vec()).into();
    let _candidate_digest = candidate_digest;
    Ok(ReviewedPersonMatchCandidatePromotionPlanV1 {
        logical_owner_id: approved.logical_owner_id.clone(),
        review_id,
        candidate_id,
        decision_id,
        decision_revision: approved.decision_revision,
        approved_action_digest: approved_digest,
        persons_command_id,
        persons_command_fingerprint,
        persons_command,
    })
}

fn wire_source(
    source: &PublicPersonSourceIdentityV1,
) -> Result<ProviderSourceIdentityV1, ReviewedPersonMatchCandidatePromotionCoreErrorV1> {
    Ok(ProviderSourceIdentityV1 {
        integration_public_id: id16(&source.integration_public_id)?.to_vec(),
        account_public_id: id16(&source.account_public_id)?.to_vec(),
        provider_source_contact_public_id: id16(&source.provider_source_contact_public_id)?
            .to_vec(),
    })
}

fn require_revision(value: u64) -> Result<(), ReviewedPersonMatchCandidatePromotionCoreErrorV1> {
    if value == 0 {
        Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidRevision)
    } else {
        Ok(())
    }
}
fn source_tuple(value: &SplitPersonSourceSelectionV1) -> ([u8; 16], [u8; 16], [u8; 16]) {
    let source = value.source.as_ref().expect("validated source");
    (
        source
            .integration_public_id
            .as_slice()
            .try_into()
            .expect("validated"),
        source
            .account_public_id
            .as_slice()
            .try_into()
            .expect("validated"),
        source
            .provider_source_contact_public_id
            .as_slice()
            .try_into()
            .expect("validated"),
    )
}
fn digest_source(source: &ProviderSourceIdentityV1) -> PersonsActionDigestSourceV1 {
    PersonsActionDigestSourceV1 {
        integration_public_id: source
            .integration_public_id
            .as_slice()
            .try_into()
            .expect("validated"),
        account_public_id: source
            .account_public_id
            .as_slice()
            .try_into()
            .expect("validated"),
        provider_source_contact_public_id: source
            .provider_source_contact_public_id
            .as_slice()
            .try_into()
            .expect("validated"),
    }
}

fn timestamp(
    unix_millis: i64,
) -> Result<TimestampV1, ReviewedPersonMatchCandidatePromotionCoreErrorV1> {
    if unix_millis <= 0 {
        return Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidTimestamp);
    }
    Ok(TimestampV1 {
        unix_seconds: unix_millis / 1_000,
        nanos: i32::try_from((unix_millis % 1_000) * 1_000_000)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidTimestamp)?,
    })
}

fn validate_owner(owner: &str) -> Result<(), ReviewedPersonMatchCandidatePromotionCoreErrorV1> {
    if !owner.is_empty()
        && owner.len() <= 128
        && owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Ok(())
    } else {
        Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidOwner)
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewedPersonMatchCandidatePromotionCoreErrorV1> {
    let id: [u8; 16] = value
        .try_into()
        .map_err(|_| ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidId)?;
    if id.iter().all(|byte| *byte == 0) {
        Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidId)
    } else {
        Ok(id)
    }
}

fn id32(value: &[u8]) -> Result<[u8; 32], ReviewedPersonMatchCandidatePromotionCoreErrorV1> {
    let digest: [u8; 32] = value
        .try_into()
        .map_err(|_| ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidDigest)?;
    if digest.iter().all(|byte| *byte == 0) {
        Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::InvalidDigest)
    } else {
        Ok(digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_review_person_match_candidate_api::wire::{
        AttachPersonSourceReviewActionV1, MergePersonsReviewActionV1,
        PersonMatchCandidateApprovedActionV1, person_match_candidate_approved_action_v1,
    };

    fn source(seed: u8) -> PublicPersonSourceIdentityV1 {
        PublicPersonSourceIdentityV1 {
            integration_public_id: [seed; 16].to_vec(),
            account_public_id: [seed + 1; 16].to_vec(),
            provider_source_contact_public_id: [seed + 2; 16].to_vec(),
        }
    }

    fn approved(
        action: ReviewAction,
        digest: [u8; 32],
    ) -> PersonMatchCandidateApprovedForPromotionV1 {
        PersonMatchCandidateApprovedForPromotionV1 {
            review_id: [1; 16].to_vec(),
            candidate_id: [2; 16].to_vec(),
            candidate_digest: [3; 32].to_vec(),
            decision_id: [4; 16].to_vec(),
            decision_revision: 2,
            decided_by_owner_device_id: [5; 16].to_vec(),
            decided_at_unix_millis: 1_001,
            approved_action: Some(PersonMatchCandidateApprovedActionV1 {
                action: Some(action),
            }),
            approved_action_digest: digest.to_vec(),
            logical_owner_id: "owner-a".to_owned(),
        }
    }

    #[test]
    fn attach_and_merge_recompute_exact_persons_action_digest() {
        let attach_digest = persons_attach_source_action_digest_v1(
            "owner-a",
            [10; 16],
            3,
            [11; 16],
            4,
            PersonsActionDigestSourceV1 {
                integration_public_id: [12; 16],
                account_public_id: [13; 16],
                provider_source_contact_public_id: [14; 16],
            },
            5,
        )
        .expect("digest");
        let attach = approved(
            person_match_candidate_approved_action_v1::Action::Attach(
                AttachPersonSourceReviewActionV1 {
                    from_person_id: [10; 16].to_vec(),
                    expected_from_person_revision: 3,
                    to_person_id: [11; 16].to_vec(),
                    expected_to_person_revision: 4,
                    source: Some(source(12)),
                    expected_source_revision: 5,
                },
            ),
            attach_digest,
        );
        let plan = plan_reviewed_person_match_candidate_promotion_v1(&attach).expect("attach plan");
        assert!(matches!(
            plan.persons_command.command,
            Some(Command::ConfirmedAttach(_))
        ));

        let merge_digest =
            persons_merge_action_digest_v1("owner-a", [20; 16], 7, [21; 16], 8).expect("digest");
        let merge = approved(
            person_match_candidate_approved_action_v1::Action::Merge(MergePersonsReviewActionV1 {
                source_person_id: [20; 16].to_vec(),
                expected_source_person_revision: 7,
                target_person_id: [21; 16].to_vec(),
                expected_target_person_revision: 8,
            }),
            merge_digest,
        );
        assert!(matches!(
            plan_reviewed_person_match_candidate_promotion_v1(&merge)
                .expect("merge plan")
                .persons_command
                .command,
            Some(Command::ConfirmedMerge(_))
        ));
    }

    #[test]
    fn mismatched_approved_digest_never_builds_a_persons_command() {
        let action =
            person_match_candidate_approved_action_v1::Action::Merge(MergePersonsReviewActionV1 {
                source_person_id: [20; 16].to_vec(),
                expected_source_person_revision: 7,
                target_person_id: [21; 16].to_vec(),
                expected_target_person_revision: 8,
            });
        assert_eq!(
            plan_reviewed_person_match_candidate_promotion_v1(&approved(action, [9; 32])),
            Err(ReviewedPersonMatchCandidatePromotionCoreErrorV1::ActionDigestMismatch)
        );
    }
}
