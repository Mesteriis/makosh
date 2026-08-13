#![forbid(unsafe_code)]

mod model;
mod normalization;
mod state;
mod transitions;

pub use model::{
    AttachSourceActionV1, ConfirmedActionOutcomeV1, ConfirmedActionStatusV1, DecisionProvenanceV1,
    DecisionReceiptV1, DetachSourceActionV1, DigestV1, IdentityMatchKindV1, LineageChangeKindV1,
    LineageRecordV1, ManualPersonDraftV1, MergePersonsActionV1, OwnerProfileV1, PersonIdV1,
    PersonLifecycleV1, PersonRevisionV1, PersonV1, PersonsOwnerSnapshotV1,
    PersonsTransitionErrorV1, PublicIdV1, RemovedSourceV1, ReviewCandidateV1, SourceClaimsV1,
    SourceLinkKeyV1, SourceLinkV1, SourceObservationOutcomeV1, SourceObservationV1,
    SourceProvenanceV1, SourceRemovalOutcomeV1, SplitPersonActionV1, SplitProfileFactKindV1,
    SplitSourceSelectionV1, TimestampV1,
};
pub use normalization::{normalize_email_v1, normalize_phone_v1};
pub use state::PersonsStateV1;
pub use transitions::{
    attach_source_action_digest_v1, attach_source_v1, create_manual_person_v1,
    detach_source_action_digest_v1, detach_source_v1, merge_persons_action_digest_v1,
    merge_persons_v1, observe_source_v1, remove_source_v1, split_person_action_digest_v1,
    split_person_v1, update_owner_profile_v1,
};

pub const PACKAGE: &str = "makosh-persons-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_PROFILE_TEXT_CHARS_V1: usize = 240;
pub const MAX_EMAILS_V1: usize = 32;
pub const MAX_PHONES_V1: usize = 32;
pub const MAX_REVIEW_CANDIDATES_PER_COMMAND_V1: usize = 128;

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "owner-1";
    const OTHER_OWNER: &str = "owner-2";

    #[test]
    fn unseen_source_is_deterministic_and_replay_is_idempotent() {
        let mut state = PersonsStateV1::default();
        let first = observe_source_v1(&mut state, source_observation(1, 1, 9)).expect("create");
        let replay = observe_source_v1(&mut state, source_observation(1, 1, 9)).expect("replay");
        assert!(matches!(first, SourceObservationOutcomeV1::Created { .. }));
        assert!(matches!(
            replay,
            SourceObservationOutcomeV1::Unchanged { .. }
        ));
        assert_eq!(state.persons().count(), 1);
    }

    #[test]
    fn source_revision_fences_stale_and_conflicting_replay() {
        let mut state = PersonsStateV1::default();
        observe_source_v1(&mut state, source_observation(2, 4, 7)).expect("create");
        assert_eq!(
            observe_source_v1(&mut state, source_observation(2, 3, 7)),
            Err(PersonsTransitionErrorV1::StaleSourceRevision),
        );
        assert_eq!(
            observe_source_v1(&mut state, source_observation(2, 4, 8)),
            Err(PersonsTransitionErrorV1::SourceRevisionConflict),
        );
    }

    #[test]
    fn matching_identity_raises_review_candidate_without_merge() {
        let mut state = PersonsStateV1::default();
        let first = observe_source_v1(&mut state, source_observation(3, 1, 3)).expect("first");
        let second = observe_source_v1(&mut state, source_observation(4, 1, 4)).expect("second");
        assert_ne!(first.person_id(), second.person_id());
        assert_eq!(second.review_candidates().len(), 2);
        assert_eq!(
            state.person(first.person_id()).expect("first").lifecycle,
            PersonLifecycleV1::Provisional,
        );
        assert_eq!(
            state.person(second.person_id()).expect("second").lifecycle,
            PersonLifecycleV1::Provisional,
        );
    }

    #[test]
    fn candidate_overflow_is_bounded_and_rejected_before_mutation() {
        let mut state = PersonsStateV1::default();
        for seed in 1..=129_u8 {
            let mut observation = source_observation(seed, 1, seed);
            observation.claims.emails = vec!["shared@example.test".to_owned()];
            observation.claims.phones.clear();
            observe_source_v1(&mut state, observation).expect("bounded candidate fixture");
        }
        let before = state.clone();
        let mut overflow = source_observation(200, 1, 200);
        overflow.claims.emails = vec!["shared@example.test".to_owned()];
        overflow.claims.phones.clear();
        assert_eq!(
            observe_source_v1(&mut state, overflow),
            Err(PersonsTransitionErrorV1::ReviewCandidateLimitExceeded)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn provider_ids_are_account_isolated_and_matches_are_owner_isolated() {
        let mut state = PersonsStateV1::default();
        let first =
            observe_source_v1(&mut state, source_observation_for_account(5, 1)).expect("first");
        let second =
            observe_source_v1(&mut state, source_observation_for_account(5, 2)).expect("second");
        assert_ne!(first.person_id(), second.person_id());

        let mut other_owner = source_observation(6, 1, 6);
        other_owner.logical_owner_id = OTHER_OWNER.to_owned();
        let outcome = observe_source_v1(&mut state, other_owner).expect("other owner");
        assert!(outcome.review_candidates().is_empty());
    }

    #[test]
    fn confirmed_attach_detach_merge_and_split_preserve_lineage_and_uniqueness() {
        let mut state = PersonsStateV1::default();
        let first = observe_source_v1(&mut state, source_observation(7, 1, 7)).expect("first");
        let second = observe_source_v1(&mut state, source_observation(8, 1, 8)).expect("second");
        let first_id = first.person_id();
        let second_id = second.person_id();
        let second_key = source_observation(8, 1, 8).key;

        let attach = attach_action(&state, second_id, first_id, second_key);
        apply_attach(&mut state, attach, 1, 1_001);
        let detach = DetachSourceActionV1 {
            logical_owner_id: OWNER.to_owned(),
            person_id: first_id,
            expected_person_revision: revision(&state, first_id),
            source: second_key,
            expected_source_revision: source_revision(&state, first_id, second_key),
            expected_detached_person_revision: revision(&state, second_id),
        };
        apply_detach(&mut state, detach, 2, 1_002);
        let merge = merge_action(&state, second_id, first_id);
        apply_merge(&mut state, merge, 3, 1_003);
        let split = split_action(
            &state,
            second_id,
            first_id,
            vec![SplitSourceSelectionV1 {
                source: second_key,
                expected_source_revision: source_revision(&state, first_id, second_key),
            }],
            Vec::new(),
        );
        apply_split(&mut state, split, 4, 1_004);

        assert_eq!(state.source_owner(second_key), Some(second_id));
        assert_eq!(state.lineage().count(), 2);
        assert_ne!(
            state.person(second_id).expect("split").lifecycle,
            PersonLifecycleV1::Merged,
        );
    }

    #[test]
    fn confirmed_actions_require_nonzero_decision_and_exact_action_digest() {
        let mut state = PersonsStateV1::default();
        let first = observe_source_v1(&mut state, source_observation(9, 1, 9)).expect("first");
        let second = observe_source_v1(&mut state, source_observation(10, 1, 10)).expect("second");
        let action = merge_action(&state, first.person_id(), second.person_id());
        let mut missing = decision(5, DigestV1([0; 32]), 1_005);
        missing.decision_id = PublicIdV1([0; 16]);
        assert_eq!(
            merge_persons_v1(&mut state, action.clone(), missing),
            Err(PersonsTransitionErrorV1::DecisionRequired),
        );
        let wrong = decision(6, DigestV1([77; 32]), 1_006);
        assert_eq!(
            merge_persons_v1(&mut state, action, wrong),
            Err(PersonsTransitionErrorV1::ActionDigestMismatch),
        );
        assert_eq!(state.lineage().count(), 0);
    }

    #[test]
    fn detach_restores_source_person_when_owner_profile_kept_it_active() {
        let mut state = PersonsStateV1::default();
        let source = observe_source_v1(&mut state, source_observation(11, 1, 11)).expect("source");
        let target = observe_source_v1(&mut state, source_observation(12, 1, 12)).expect("target");
        let source_id = source.person_id();
        let source_key = source_observation(11, 1, 11).key;
        update_owner_profile_v1(
            &mut state,
            OWNER,
            source_id,
            1,
            owner_profile(),
            timestamp(500),
        )
        .expect("profile");
        let attach = attach_action(&state, source_id, target.person_id(), source_key);
        apply_attach(&mut state, attach, 7, 1_007);
        let detach = DetachSourceActionV1 {
            logical_owner_id: OWNER.to_owned(),
            person_id: target.person_id(),
            expected_person_revision: revision(&state, target.person_id()),
            source: source_key,
            expected_source_revision: source_revision(&state, target.person_id(), source_key),
            expected_detached_person_revision: revision(&state, source_id),
        };
        apply_detach(&mut state, detach, 8, 1_008);
        assert_eq!(state.source_owner(source_key), Some(source_id));
        assert_eq!(
            state.person(source_id).expect("restored").lifecycle,
            PersonLifecycleV1::Active,
        );
    }

    #[test]
    fn source_removal_archives_only_a_source_only_orphan() {
        let mut state = PersonsStateV1::default();
        let only = observe_source_v1(&mut state, source_observation(13, 1, 13)).expect("only");
        remove_source_v1(
            &mut state,
            OWNER,
            source_observation(13, 1, 13).key,
            provenance_at(2, 14, 600),
        )
        .expect("remove");
        assert_eq!(
            state.person(only.person_id()).expect("orphan").lifecycle,
            PersonLifecycleV1::Archived,
        );

        let retained = observe_source_v1(&mut state, source_observation(14, 1, 14)).expect("kept");
        update_owner_profile_v1(
            &mut state,
            OWNER,
            retained.person_id(),
            1,
            owner_profile(),
            timestamp(500),
        )
        .expect("profile");
        remove_source_v1(
            &mut state,
            OWNER,
            source_observation(14, 1, 14).key,
            provenance_at(2, 15, 600),
        )
        .expect("remove retained");
        assert_eq!(
            state
                .person(retained.person_id())
                .expect("retained")
                .lifecycle,
            PersonLifecycleV1::Active,
        );

        let multi = observe_source_v1(&mut state, source_observation(15, 1, 15)).expect("multi");
        let other = observe_source_v1(&mut state, source_observation(16, 1, 16)).expect("other");
        let other_key = source_observation(16, 1, 16).key;
        let attach = attach_action(&state, other.person_id(), multi.person_id(), other_key);
        apply_attach(&mut state, attach, 9, 1_009);
        remove_source_v1(
            &mut state,
            OWNER,
            source_observation(15, 1, 15).key,
            provenance_at(2, 17, 1_100),
        )
        .expect("remove one of two");
        assert_eq!(
            state.person(multi.person_id()).expect("multi").lifecycle,
            PersonLifecycleV1::Active,
        );
        assert_eq!(state.source_owner(other_key), Some(multi.person_id()));
    }

    #[test]
    fn every_post_creation_mutation_rejects_cross_owner_before_mutation() {
        let mut state = PersonsStateV1::default();
        let first = observe_source_v1(&mut state, source_observation(17, 1, 17)).expect("first");
        let second = observe_source_v1(&mut state, source_observation(18, 1, 18)).expect("second");
        let first_id = first.person_id();
        let second_id = second.person_id();
        let first_key = source_observation(17, 1, 17).key;

        assert_no_mutation(&mut state, |state| {
            update_owner_profile_v1(
                state,
                OTHER_OWNER,
                first_id,
                1,
                owner_profile(),
                timestamp(500),
            )
        });
        assert_no_mutation(&mut state, |state| {
            remove_source_v1(state, OTHER_OWNER, first_key, provenance_at(2, 19, 600)).map(|_| ())
        });

        let wrong_attach = AttachSourceActionV1 {
            logical_owner_id: OTHER_OWNER.to_owned(),
            from_person_id: first_id,
            expected_from_person_revision: 1,
            to_person_id: second_id,
            expected_to_person_revision: 1,
            source: first_key,
            expected_source_revision: 1,
        };
        assert_confirmed_owner_rejection(&mut state, wrong_attach, 20, attach_source_v1);

        let wrong_detach = DetachSourceActionV1 {
            logical_owner_id: OTHER_OWNER.to_owned(),
            person_id: first_id,
            expected_person_revision: 1,
            source: first_key,
            expected_source_revision: 1,
            expected_detached_person_revision: 0,
        };
        let digest = detach_source_action_digest_v1(&wrong_detach).expect("digest");
        let before = state.clone();
        assert_eq!(
            detach_source_v1(&mut state, wrong_detach, decision(21, digest, 1_021)),
            Err(PersonsTransitionErrorV1::OwnerMismatch),
        );
        assert_eq!(state, before);

        let wrong_merge = MergePersonsActionV1 {
            logical_owner_id: OTHER_OWNER.to_owned(),
            source_person_id: first_id,
            expected_source_person_revision: 1,
            target_person_id: second_id,
            expected_target_person_revision: 1,
        };
        let digest = merge_persons_action_digest_v1(&wrong_merge).expect("digest");
        let before = state.clone();
        assert_eq!(
            merge_persons_v1(&mut state, wrong_merge, decision(22, digest, 1_022)),
            Err(PersonsTransitionErrorV1::OwnerMismatch),
        );
        assert_eq!(state, before);

        let merge = merge_action(&state, first_id, second_id);
        apply_merge(&mut state, merge, 23, 1_023);
        let wrong_split = SplitPersonActionV1 {
            logical_owner_id: OTHER_OWNER.to_owned(),
            merged_person_id: first_id,
            expected_merged_person_revision: revision(&state, first_id),
            target_person_id: second_id,
            expected_target_person_revision: revision(&state, second_id),
            source_selection: vec![SplitSourceSelectionV1 {
                source: first_key,
                expected_source_revision: source_revision(&state, second_id, first_key),
            }],
            profile_fact_selection: Vec::new(),
        };
        let digest = split_person_action_digest_v1(&wrong_split).expect("digest");
        let before = state.clone();
        assert_eq!(
            split_person_v1(&mut state, wrong_split, decision(24, digest, 1_024)),
            Err(PersonsTransitionErrorV1::OwnerMismatch),
        );
        assert_eq!(state, before);
    }

    #[test]
    fn confirmed_action_binding_replay_reuse_and_timestamp_are_fail_closed() {
        let mut state = PersonsStateV1::default();
        let first = observe_source_v1(&mut state, source_observation(19, 1, 19)).expect("first");
        let second = observe_source_v1(&mut state, source_observation(20, 1, 20)).expect("second");
        let key = source_observation(19, 1, 19).key;
        let action = attach_action(&state, first.person_id(), second.person_id(), key);
        let digest = attach_source_action_digest_v1(&action).expect("digest");
        let approval = decision(25, digest, 1_025);
        let applied =
            attach_source_v1(&mut state, action.clone(), approval.clone()).expect("apply");
        assert_eq!(applied.status, ConfirmedActionStatusV1::Applied);
        assert_eq!(applied.person_revisions.len(), 2);
        let after_apply = state.clone();
        let replay =
            attach_source_v1(&mut state, action.clone(), approval.clone()).expect("replay");
        assert_eq!(replay.status, ConfirmedActionStatusV1::Replayed);
        assert_eq!(state, after_apply);

        let mut cross_owner_reuse = action.clone();
        cross_owner_reuse.logical_owner_id = OTHER_OWNER.to_owned();
        let cross_owner_digest =
            attach_source_action_digest_v1(&cross_owner_reuse).expect("digest");
        assert_eq!(
            attach_source_v1(
                &mut state,
                cross_owner_reuse,
                decision(25, cross_owner_digest, 1_025),
            ),
            Err(PersonsTransitionErrorV1::OwnerMismatch),
        );
        assert_eq!(state, after_apply);

        let detach = DetachSourceActionV1 {
            logical_owner_id: OWNER.to_owned(),
            person_id: second.person_id(),
            expected_person_revision: revision(&state, second.person_id()),
            source: key,
            expected_source_revision: source_revision(&state, second.person_id(), key),
            expected_detached_person_revision: revision(&state, first.person_id()),
        };
        let detach_digest = detach_source_action_digest_v1(&detach).expect("digest");
        let reused = decision(25, detach_digest, 1_025);
        assert_eq!(
            detach_source_v1(&mut state, detach, reused),
            Err(PersonsTransitionErrorV1::DecisionReuseConflict),
        );
        assert_eq!(state, after_apply);

        let mut tampered = action.clone();
        tampered.expected_to_person_revision += 1;
        assert_eq!(
            attach_source_v1(&mut state, tampered, approval),
            Err(PersonsTransitionErrorV1::ActionDigestMismatch),
        );

        let stale = MergePersonsActionV1 {
            logical_owner_id: OWNER.to_owned(),
            source_person_id: first.person_id(),
            expected_source_person_revision: 1,
            target_person_id: second.person_id(),
            expected_target_person_revision: revision(&state, second.person_id()),
        };
        let stale_digest = merge_persons_action_digest_v1(&stale).expect("digest");
        assert_eq!(
            merge_persons_v1(&mut state, stale, decision(26, stale_digest, 1_026)),
            Err(PersonsTransitionErrorV1::ExpectedRevisionConflict),
        );

        let mut fresh_state = PersonsStateV1::default();
        let left =
            observe_source_v1(&mut fresh_state, source_observation(21, 1, 21)).expect("left");
        let right =
            observe_source_v1(&mut fresh_state, source_observation(22, 1, 22)).expect("right");
        let merge = merge_action(&fresh_state, left.person_id(), right.person_id());
        let merge_digest = merge_persons_action_digest_v1(&merge).expect("digest");
        assert_eq!(
            merge_persons_v1(&mut fresh_state, merge, decision(27, merge_digest, 100)),
            Err(PersonsTransitionErrorV1::DecisionTimestampRegression),
        );
    }

    #[test]
    fn split_selects_subset_and_profile_facts_after_unselected_source_disappears() {
        let mut state = PersonsStateV1::default();
        let source = observe_source_v1(&mut state, source_observation(23, 1, 23)).expect("source");
        let extra = observe_source_v1(&mut state, source_observation(24, 1, 24)).expect("extra");
        let target = observe_source_v1(&mut state, source_observation(25, 1, 25)).expect("target");
        let source_id = source.person_id();
        let target_id = target.person_id();
        let source_key = source_observation(23, 1, 23).key;
        let extra_key = source_observation(24, 1, 24).key;
        update_owner_profile_v1(
            &mut state,
            OWNER,
            source_id,
            1,
            owner_profile(),
            timestamp(500),
        )
        .expect("profile");
        let attach = attach_action(&state, extra.person_id(), source_id, extra_key);
        apply_attach(&mut state, attach, 28, 1_028);
        let merge = merge_action(&state, source_id, target_id);
        apply_merge(&mut state, merge, 29, 1_029);
        remove_source_v1(&mut state, OWNER, extra_key, provenance_at(2, 30, 1_030))
            .expect("remove unselected original source");

        let unavailable = split_action(
            &state,
            source_id,
            target_id,
            vec![SplitSourceSelectionV1 {
                source: extra_key,
                expected_source_revision: 1,
            }],
            vec![SplitProfileFactKindV1::DisplayName],
        );
        let unavailable_digest = split_person_action_digest_v1(&unavailable).expect("digest");
        let before = state.clone();
        assert_eq!(
            split_person_v1(
                &mut state,
                unavailable,
                decision(30, unavailable_digest, 1_030),
            ),
            Err(PersonsTransitionErrorV1::SourceOwnerConflict),
        );
        assert_eq!(state, before);

        let empty = split_action(&state, source_id, target_id, Vec::new(), Vec::new());
        assert_eq!(
            split_person_action_digest_v1(&empty),
            Err(PersonsTransitionErrorV1::EmptySplitSelection),
        );

        let selected = split_action(
            &state,
            source_id,
            target_id,
            vec![SplitSourceSelectionV1 {
                source: source_key,
                expected_source_revision: source_revision(&state, target_id, source_key),
            }],
            vec![SplitProfileFactKindV1::DisplayName],
        );
        apply_split(&mut state, selected, 31, 1_031);
        let restored = state.person(source_id).expect("restored");
        let profile = restored.owner_profile.as_ref().expect("selected profile");
        assert_eq!(profile.display_name.as_deref(), Some("Ada"));
        assert!(profile.given_name.is_none());
        assert!(profile.emails.is_empty());
        assert_eq!(state.source_owner(source_key), Some(source_id));
        assert_eq!(state.lineage().count(), 2);
    }

    #[test]
    fn split_selected_subset_survives_an_unselected_source_move() {
        let mut state = PersonsStateV1::default();
        let source = observe_source_v1(&mut state, source_observation(28, 1, 28)).expect("source");
        let extra = observe_source_v1(&mut state, source_observation(29, 1, 29)).expect("extra");
        let target = observe_source_v1(&mut state, source_observation(30, 1, 30)).expect("target");
        let destination =
            observe_source_v1(&mut state, source_observation(31, 1, 31)).expect("destination");
        let source_key = source_observation(28, 1, 28).key;
        let extra_key = source_observation(29, 1, 29).key;

        let attach = attach_action(&state, extra.person_id(), source.person_id(), extra_key);
        apply_attach(&mut state, attach, 32, 1_032);
        let merge = merge_action(&state, source.person_id(), target.person_id());
        apply_merge(&mut state, merge, 33, 1_033);
        let move_unselected = attach_action(
            &state,
            target.person_id(),
            destination.person_id(),
            extra_key,
        );
        apply_attach(&mut state, move_unselected, 34, 1_034);

        let selected = split_action(
            &state,
            source.person_id(),
            target.person_id(),
            vec![SplitSourceSelectionV1 {
                source: source_key,
                expected_source_revision: source_revision(&state, target.person_id(), source_key),
            }],
            Vec::new(),
        );
        apply_split(&mut state, selected, 35, 1_035);
        assert_eq!(state.source_owner(source_key), Some(source.person_id()));
        assert_eq!(state.source_owner(extra_key), Some(destination.person_id()));
    }

    #[test]
    fn split_digest_is_order_independent_but_selection_exact() {
        let base = SplitPersonActionV1 {
            logical_owner_id: OWNER.to_owned(),
            merged_person_id: PersonIdV1([1; 16]),
            expected_merged_person_revision: 3,
            target_person_id: PersonIdV1([2; 16]),
            expected_target_person_revision: 4,
            source_selection: vec![
                SplitSourceSelectionV1 {
                    source: source_observation(26, 1, 26).key,
                    expected_source_revision: 8,
                },
                SplitSourceSelectionV1 {
                    source: source_observation(27, 1, 27).key,
                    expected_source_revision: 9,
                },
            ],
            profile_fact_selection: vec![
                SplitProfileFactKindV1::Emails,
                SplitProfileFactKindV1::DisplayName,
            ],
        };
        let mut reordered = base.clone();
        reordered.source_selection.reverse();
        reordered.profile_fact_selection.reverse();
        assert_eq!(
            split_person_action_digest_v1(&base),
            split_person_action_digest_v1(&reordered),
        );
        reordered.source_selection[0].expected_source_revision += 1;
        assert_ne!(
            split_person_action_digest_v1(&base),
            split_person_action_digest_v1(&reordered),
        );
    }

    #[test]
    fn owner_snapshot_round_trip_is_validated_and_exact() {
        let mut state = PersonsStateV1::default();
        observe_source_v1(&mut state, source_observation(32, 1, 32)).expect("source");
        let snapshot = state.snapshot_for_owner_v1(OWNER).expect("snapshot");

        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(snapshot).expect("reconstitute"),
            state,
        );
    }

    #[test]
    fn owner_snapshot_rejects_duplicate_source_ownership() {
        let mut state = PersonsStateV1::default();
        observe_source_v1(&mut state, source_observation(33, 1, 33)).expect("first");
        observe_source_v1(&mut state, source_observation(34, 1, 34)).expect("second");
        let mut snapshot = state.snapshot_for_owner_v1(OWNER).expect("snapshot");
        let duplicated = snapshot.persons[0]
            .source_links
            .iter()
            .next()
            .map(|(key, link)| (*key, link.clone()))
            .expect("source");
        snapshot.persons[1]
            .source_links
            .insert(duplicated.0, duplicated.1);

        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(snapshot),
            Err(PersonsTransitionErrorV1::SourceOwnerConflict),
        );
    }

    #[test]
    fn owner_snapshot_rejects_broken_merged_targets_and_lineage_decision_graphs() {
        let mut state = PersonsStateV1::default();
        let source = observe_source_v1(&mut state, source_observation(35, 1, 35)).expect("source");
        let target = observe_source_v1(&mut state, source_observation(36, 1, 36)).expect("target");
        let merge = merge_action(&state, source.person_id(), target.person_id());
        apply_merge(&mut state, merge, 41, 2_041);
        let snapshot = state.snapshot_for_owner_v1(OWNER).expect("valid graph");

        let mut missing_target = snapshot.clone();
        let merged = missing_target
            .persons
            .iter_mut()
            .find(|person| person.person_id == source.person_id())
            .expect("merged Person");
        merged.merged_into = Some(PersonIdV1([199; 16]));
        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(missing_target),
            Err(PersonsTransitionErrorV1::LineageConflict),
        );

        let mut self_target = snapshot.clone();
        let merged = self_target
            .persons
            .iter_mut()
            .find(|person| person.person_id == source.person_id())
            .expect("merged Person");
        merged.merged_into = Some(source.person_id());
        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(self_target),
            Err(PersonsTransitionErrorV1::LineageConflict),
        );

        let mut cycle = snapshot.clone();
        let target_person = cycle
            .persons
            .iter_mut()
            .find(|person| person.person_id == target.person_id())
            .expect("merge target");
        target_person.lifecycle = PersonLifecycleV1::Merged;
        target_person.merged_into = Some(source.person_id());
        target_person.source_links.clear();
        target_person.owner_profile = None;
        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(cycle),
            Err(PersonsTransitionErrorV1::LineageConflict),
        );

        let mut missing_receipt = snapshot.clone();
        missing_receipt.decision_receipts.clear();
        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(missing_receipt),
            Err(PersonsTransitionErrorV1::InvalidSnapshot),
        );

        let mut mismatched_provenance = snapshot.clone();
        mismatched_provenance.lineage[0].decision.review_id = PublicIdV1([198; 16]);
        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(mismatched_provenance),
            Err(PersonsTransitionErrorV1::InvalidSnapshot),
        );

        let mut duplicate_lineage_decision = snapshot.clone();
        duplicate_lineage_decision
            .lineage
            .push(duplicate_lineage_decision.lineage[0].clone());
        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(duplicate_lineage_decision),
            Err(PersonsTransitionErrorV1::InvalidSnapshot),
        );

        let mut incomplete_outcome = snapshot;
        incomplete_outcome.decision_receipts[0]
            .outcome
            .person_revisions
            .pop();
        assert_eq!(
            PersonsStateV1::reconstitute_owner_v1(incomplete_outcome),
            Err(PersonsTransitionErrorV1::InvalidSnapshot),
        );
    }

    fn assert_no_mutation<F>(state: &mut PersonsStateV1, operation: F)
    where
        F: FnOnce(&mut PersonsStateV1) -> Result<(), PersonsTransitionErrorV1>,
    {
        let before = state.clone();
        assert_eq!(
            operation(state),
            Err(PersonsTransitionErrorV1::OwnerMismatch)
        );
        assert_eq!(*state, before);
    }

    fn assert_confirmed_owner_rejection(
        state: &mut PersonsStateV1,
        action: AttachSourceActionV1,
        seed: u8,
        operation: fn(
            &mut PersonsStateV1,
            AttachSourceActionV1,
            DecisionProvenanceV1,
        ) -> Result<ConfirmedActionOutcomeV1, PersonsTransitionErrorV1>,
    ) {
        let digest = attach_source_action_digest_v1(&action).expect("digest");
        let before = state.clone();
        assert_eq!(
            operation(
                state,
                action,
                decision(seed, digest, 1_000 + i64::from(seed))
            ),
            Err(PersonsTransitionErrorV1::OwnerMismatch),
        );
        assert_eq!(*state, before);
    }

    fn attach_action(
        state: &PersonsStateV1,
        from_person_id: PersonIdV1,
        to_person_id: PersonIdV1,
        source: SourceLinkKeyV1,
    ) -> AttachSourceActionV1 {
        AttachSourceActionV1 {
            logical_owner_id: OWNER.to_owned(),
            from_person_id,
            expected_from_person_revision: revision(state, from_person_id),
            to_person_id,
            expected_to_person_revision: revision(state, to_person_id),
            source,
            expected_source_revision: source_revision(state, from_person_id, source),
        }
    }

    fn merge_action(
        state: &PersonsStateV1,
        source_person_id: PersonIdV1,
        target_person_id: PersonIdV1,
    ) -> MergePersonsActionV1 {
        MergePersonsActionV1 {
            logical_owner_id: OWNER.to_owned(),
            source_person_id,
            expected_source_person_revision: revision(state, source_person_id),
            target_person_id,
            expected_target_person_revision: revision(state, target_person_id),
        }
    }

    fn split_action(
        state: &PersonsStateV1,
        merged_person_id: PersonIdV1,
        target_person_id: PersonIdV1,
        source_selection: Vec<SplitSourceSelectionV1>,
        profile_fact_selection: Vec<SplitProfileFactKindV1>,
    ) -> SplitPersonActionV1 {
        SplitPersonActionV1 {
            logical_owner_id: OWNER.to_owned(),
            merged_person_id,
            expected_merged_person_revision: revision(state, merged_person_id),
            target_person_id,
            expected_target_person_revision: revision(state, target_person_id),
            source_selection,
            profile_fact_selection,
        }
    }

    fn apply_attach(
        state: &mut PersonsStateV1,
        action: AttachSourceActionV1,
        seed: u8,
        decided_at: i64,
    ) -> ConfirmedActionOutcomeV1 {
        let digest = attach_source_action_digest_v1(&action).expect("digest");
        attach_source_v1(state, action, decision(seed, digest, decided_at)).expect("attach")
    }

    fn apply_detach(
        state: &mut PersonsStateV1,
        action: DetachSourceActionV1,
        seed: u8,
        decided_at: i64,
    ) -> ConfirmedActionOutcomeV1 {
        let digest = detach_source_action_digest_v1(&action).expect("digest");
        detach_source_v1(state, action, decision(seed, digest, decided_at)).expect("detach")
    }

    fn apply_merge(
        state: &mut PersonsStateV1,
        action: MergePersonsActionV1,
        seed: u8,
        decided_at: i64,
    ) -> ConfirmedActionOutcomeV1 {
        let digest = merge_persons_action_digest_v1(&action).expect("digest");
        merge_persons_v1(state, action, decision(seed, digest, decided_at)).expect("merge")
    }

    fn apply_split(
        state: &mut PersonsStateV1,
        action: SplitPersonActionV1,
        seed: u8,
        decided_at: i64,
    ) -> ConfirmedActionOutcomeV1 {
        let digest = split_person_action_digest_v1(&action).expect("digest");
        split_person_v1(state, action, decision(seed, digest, decided_at)).expect("split")
    }

    fn revision(state: &PersonsStateV1, person_id: PersonIdV1) -> u64 {
        state.person(person_id).expect("person").revision
    }

    fn source_revision(
        state: &PersonsStateV1,
        person_id: PersonIdV1,
        source: SourceLinkKeyV1,
    ) -> u64 {
        state
            .person(person_id)
            .and_then(|person| person.source_links.get(&source))
            .expect("source")
            .provenance
            .revision
    }

    fn source_observation(seed: u8, revision: u64, digest: u8) -> SourceObservationV1 {
        SourceObservationV1 {
            logical_owner_id: OWNER.to_owned(),
            key: SourceLinkKeyV1 {
                integration_public_id: PublicIdV1([1; 16]),
                account_public_id: PublicIdV1([seed; 16]),
                provider_source_contact_public_id: PublicIdV1([seed; 16]),
            },
            claims: SourceClaimsV1 {
                display_name: Some("Ada Lovelace".to_owned()),
                emails: vec![" ADA@Example.test ".to_owned()],
                phones: vec!["+34 (910) 000-000".to_owned()],
            },
            provenance: provenance_at(revision, digest, 100 + revision as i64),
        }
    }

    fn source_observation_for_account(source: u8, account: u8) -> SourceObservationV1 {
        let mut value = source_observation(source, 1, source);
        value.key.account_public_id = PublicIdV1([account; 16]);
        value
    }

    fn provenance_at(revision: u64, digest: u8, observed_at: i64) -> SourceProvenanceV1 {
        SourceProvenanceV1 {
            revision,
            digest: DigestV1([digest; 32]),
            observed_at: timestamp(observed_at),
        }
    }

    fn decision(
        seed: u8,
        approved_action_digest: DigestV1,
        decided_at: i64,
    ) -> DecisionProvenanceV1 {
        DecisionProvenanceV1 {
            decision_id: PublicIdV1([seed; 16]),
            review_id: PublicIdV1([99; 16]),
            revision: u64::from(seed),
            decided_by_owner_device_id: PublicIdV1([88; 16]),
            decided_at: timestamp(decided_at),
            approved_action_digest,
        }
    }

    fn timestamp(unix_seconds: i64) -> TimestampV1 {
        TimestampV1 {
            unix_seconds,
            nanos: 0,
        }
    }

    fn owner_profile() -> OwnerProfileV1 {
        OwnerProfileV1 {
            display_name: Some("Ada".to_owned()),
            given_name: Some("Ada".to_owned()),
            family_name: Some("Lovelace".to_owned()),
            emails: vec!["ada@example.test".to_owned()],
            phones: Vec::new(),
        }
    }
}
