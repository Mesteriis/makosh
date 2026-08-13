use makosh_persons_core::{
    DecisionProvenanceV1, DigestV1, ManualPersonDraftV1, MergePersonsActionV1, OwnerProfileV1,
    PersonIdV1, PublicIdV1, SourceClaimsV1, SourceLinkKeyV1, SourceObservationV1,
    SourceProvenanceV1, SplitPersonActionV1, SplitSourceSelectionV1, TimestampV1,
    create_manual_person_v1, merge_persons_action_digest_v1, merge_persons_v1, observe_source_v1,
    remove_source_v1, split_person_action_digest_v1, split_person_v1, update_owner_profile_v1,
};
use makosh_persons_persistence::{
    ApplyPersonsCommandV1, PersonsCommandCommitV1, PersonsEnvelopeRecordV1,
    PersonsPersistenceConformanceV1, PersonsPersistenceErrorV1,
};
use sha2::{Digest, Sha256};

const OWNER_A: &str = "owner-a";
const OWNER_B: &str = "owner-b";

#[tokio::test]
#[ignore = "requires MAKOSH_PERSONS_POSTGRES_URL disposable PostgreSQL"]
async fn postgres_upgrade_restart_replay_history_and_rollback_are_exact() {
    let url = database_url();
    let persistence = PersonsPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    PersonsPersistenceConformanceV1::install_initial_schema(&persistence)
        .await
        .expect("initial schema");
    PersonsPersistenceConformanceV1::seed_initial_source_fixture(&persistence, OWNER_A)
        .await
        .expect("initial fixture");
    PersonsPersistenceConformanceV1::upgrade_to_durable_v2(&persistence)
        .await
        .expect("V2 durable upgrade");
    let legacy = envelope(240, b"legacy-v2-pending");
    PersonsPersistenceConformanceV1::seed_legacy_v2_pending_outbox(
        &persistence,
        OWNER_A,
        &legacy,
        1_800_000_000_500_i64,
    )
    .await
    .expect("legacy V2 pending row");
    PersonsPersistenceConformanceV1::upgrade_outbox_order_v3(&persistence)
        .await
        .expect("forward upgrade");
    let legacy_after_upgrade = persistence
        .load_pending_outbox(OWNER_A)
        .await
        .expect("legacy outbox after V3");
    assert_eq!(legacy_after_upgrade.len(), 1);
    assert_eq!(legacy_after_upgrade[0].record, legacy);
    assert_eq!(legacy_after_upgrade[0].resulting_owner_revision, 0);
    assert_eq!(legacy_after_upgrade[0].outbox_ordinal, 0);
    assert_eq!(legacy_after_upgrade[0].semantic_order_key, [0]);
    assert_eq!(
        persistence
            .load_owner(OWNER_A)
            .await
            .expect("upgrade load")
            .state
            .persons()
            .count(),
        1
    );

    PersonsPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("fresh schema");
    let created = persistence
        .apply_command_once(&command(1, OWNER_A, 0), |state| {
            create_manual_person_v1(
                state,
                ManualPersonDraftV1 {
                    person_id: PersonIdV1([31; 16]),
                    logical_owner_id: OWNER_A.to_owned(),
                    owner_profile: profile("Ada"),
                    created_at: time(1_800_000_001),
                },
            )
            .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(commit(101, 1))
        })
        .await
        .expect("create manual Person");
    assert!(!created.replayed);

    persistence
        .apply_command_once(&command(2, OWNER_A, 1), |state| {
            update_owner_profile_v1(
                state,
                OWNER_A,
                PersonIdV1([31; 16]),
                1,
                profile("Augusta Ada"),
                time(1_800_000_002),
            )
            .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(commit(102, 2))
        })
        .await
        .expect("profile revision two");
    persistence
        .apply_command_once(&command(3, OWNER_A, 2), |state| {
            update_owner_profile_v1(
                state,
                OWNER_A,
                PersonIdV1([31; 16]),
                2,
                profile("Augusta Ada"),
                time(1_800_000_003),
            )
            .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(commit(103, 3))
        })
        .await
        .expect("same profile content");

    let durable_before_replays =
        PersonsPersistenceConformanceV1::durable_command_outbox_counts(&persistence, OWNER_A)
            .await
            .expect("durable counts before replays");
    let replay = persistence
        .apply_command_once(&command(2, OWNER_A, 1), |_| {
            panic!("exact replay must not invoke closure")
        })
        .await
        .expect("exact replay");
    assert!(replay.replayed);
    let mut runtime_derived_replay = command(2, OWNER_A, 3);
    runtime_derived_replay.expected_aggregate_revision = persistence
        .load_owner(OWNER_A)
        .await
        .expect("runtime current aggregate")
        .aggregate_revision;
    let runtime_derived_replay = persistence
        .apply_command_once(&runtime_derived_replay, |_| {
            panic!("completed runtime-derived replay must not invoke closure")
        })
        .await
        .expect("completed replay bypasses runtime-derived aggregate fence");
    assert!(runtime_derived_replay.replayed);
    assert_eq!(
        runtime_derived_replay.terminal_result,
        replay.terminal_result
    );
    assert_eq!(
        PersonsPersistenceConformanceV1::durable_command_outbox_counts(&persistence, OWNER_A)
            .await
            .expect("durable counts after exact replays"),
        durable_before_replays,
        "exact completed replay creates no inbox or outbox duplicates",
    );
    let mut altered_envelope = command(2, OWNER_A, 1);
    altered_envelope.command_envelope_sha256 = [200; 32];
    assert_eq!(
        persistence
            .apply_command_once(&altered_envelope, |_| panic!("altered envelope"))
            .await,
        Err(PersonsPersistenceErrorV1::CommandConflict),
    );
    let mut altered_command_id = command(2, OWNER_A, 1);
    altered_command_id.command_id = [199; 16];
    assert_eq!(
        persistence
            .apply_command_once(&altered_command_id, |_| panic!("altered command id"))
            .await,
        Err(PersonsPersistenceErrorV1::CommandConflict),
    );
    let mut conflicting_replay = command(2, OWNER_A, 1);
    conflicting_replay.command_fingerprint = [201; 32];
    assert_eq!(
        persistence
            .apply_command_once(&conflicting_replay, |_| panic!("conflict"))
            .await,
        Err(PersonsPersistenceErrorV1::CommandConflict),
    );
    let mut reused_command_id = command(9, OWNER_A, 3);
    reused_command_id.command_id = command(2, OWNER_A, 1).command_id;
    assert_eq!(
        persistence
            .apply_command_once(&reused_command_id, |_| panic!("global command identity"))
            .await,
        Err(PersonsPersistenceErrorV1::CommandConflict),
    );
    assert_eq!(
        PersonsPersistenceConformanceV1::durable_command_outbox_counts(&persistence, OWNER_A)
            .await
            .expect("durable counts after identity conflicts"),
        durable_before_replays,
        "altered identity and globally reused command ID persist nothing",
    );

    let before_failure = persistence
        .load_owner(OWNER_A)
        .await
        .expect("before failure");
    let durable_before_failure =
        PersonsPersistenceConformanceV1::durable_command_outbox_counts(&persistence, OWNER_A)
            .await
            .expect("durable counts before failures");
    assert_eq!(
        persistence
            .apply_command_once(&command(4, OWNER_A, 3), |state| {
                update_owner_profile_v1(
                    state,
                    OWNER_A,
                    PersonIdV1([31; 16]),
                    3,
                    profile("must rollback"),
                    time(1_800_000_004),
                )
                .expect("valid in-memory mutation");
                Err(PersonsPersistenceErrorV1::MutationRejected)
            })
            .await,
        Err(PersonsPersistenceErrorV1::MutationRejected),
    );
    assert_eq!(
        persistence
            .load_owner(OWNER_A)
            .await
            .expect("after failure"),
        before_failure
    );
    assert_eq!(
        persistence
            .apply_command_once(&command(5, OWNER_A, 3), |state| {
                update_owner_profile_v1(
                    state,
                    OWNER_A,
                    PersonIdV1([31; 16]),
                    3,
                    profile("bad terminal hash"),
                    time(1_800_000_005),
                )
                .expect("in-memory mutation");
                let mut invalid = commit(105, 5);
                invalid.terminal_result.envelope_sha256 = [7; 32];
                Ok(invalid)
            })
            .await,
        Err(PersonsPersistenceErrorV1::HashMismatch),
    );
    assert_eq!(
        persistence
            .apply_command_once(&command(6, OWNER_A, 3), |state| {
                update_owner_profile_v1(
                    state,
                    OWNER_A,
                    PersonIdV1([31; 16]),
                    3,
                    profile("bad event hash"),
                    time(1_800_000_006),
                )
                .expect("in-memory mutation");
                let mut invalid = commit(106, 6);
                invalid.owner_events[0].envelope_sha256 = [8; 32];
                Ok(invalid)
            })
            .await,
        Err(PersonsPersistenceErrorV1::HashMismatch),
    );
    assert_eq!(
        persistence
            .load_owner(OWNER_A)
            .await
            .expect("hash rollback"),
        before_failure
    );
    assert_eq!(
        persistence
            .apply_command_once(&command(7, OWNER_A, 2), |_| panic!("stale aggregate"))
            .await,
        Err(PersonsPersistenceErrorV1::AggregateConflict),
    );
    assert_eq!(
        persistence
            .apply_command_once(&command(8, OWNER_A, 3), |state| {
                update_owner_profile_v1(
                    state,
                    OWNER_A,
                    PersonIdV1([31; 16]),
                    2,
                    profile("stale Person revision"),
                    time(1_800_000_008),
                )
                .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
                Ok(commit(108, 8))
            })
            .await,
        Err(PersonsPersistenceErrorV1::MutationRejected),
    );
    assert_eq!(
        PersonsPersistenceConformanceV1::durable_command_outbox_counts(&persistence, OWNER_A)
            .await
            .expect("durable counts after failures"),
        durable_before_failure,
        "failed commands leave both inbox and outbox unchanged",
    );

    let reconnected = PersonsPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("reconnect");
    let after_restart = reconnected.load_owner(OWNER_A).await.expect("restart load");
    assert_eq!(after_restart, before_failure);
    let profile_revisions =
        PersonsPersistenceConformanceV1::profile_history_count(&reconnected, OWNER_A, [31; 16])
            .await
            .expect("profile history");
    assert_eq!(
        profile_revisions, 2,
        "two distinct snapshots survive reconnect"
    );
}

#[tokio::test]
#[ignore = "requires MAKOSH_PERSONS_POSTGRES_URL disposable PostgreSQL"]
async fn postgres_sources_tombstones_global_tuple_and_real_rls_fail_closed() {
    let url = database_url();
    let persistence = PersonsPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    PersonsPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    let first = source(41, 41, 1, OWNER_A, "same@example.test");
    let second = source(42, 41, 1, OWNER_A, "same@example.test");
    let first_key = first.key;
    let mut candidate_commit = commit(111, 11);
    candidate_commit.owner_events[0] = envelope(191, b"review-candidate-owner-event");
    persistence
        .apply_command_once(&command(11, OWNER_A, 0), move |state| {
            observe_source_v1(state, first)
                .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            let result = observe_source_v1(state, second)
                .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            assert!(
                !result.review_candidates().is_empty(),
                "candidate, never silent merge"
            );
            Ok(candidate_commit)
        })
        .await
        .expect("two account-isolated sources");
    let pending = persistence
        .load_pending_outbox(OWNER_A)
        .await
        .expect("outbox");
    assert_eq!(
        pending.len(),
        2,
        "terminal result and candidate owner event are both durable"
    );
    assert!(
        pending
            .iter()
            .any(|record| record.record.envelope_bytes == b"terminal")
    );
    assert!(
        pending
            .iter()
            .any(|record| record.record.envelope_bytes == b"review-candidate-owner-event")
    );
    let replay_count = std::cell::Cell::new(0_u8);
    let replay = persistence
        .apply_command_once(&command(11, OWNER_A, 0), |_| {
            replay_count.set(replay_count.get() + 1);
            Ok(commit(199, 99))
        })
        .await
        .expect("exact source replay");
    assert!(replay.replayed);
    assert_eq!(replay_count.get(), 0, "replay closure is never invoked");
    assert_eq!(
        persistence
            .load_pending_outbox(OWNER_A)
            .await
            .expect("outbox replay")
            .len(),
        2
    );
    assert_eq!(
        persistence
            .load_owner(OWNER_A)
            .await
            .expect("load")
            .state
            .persons()
            .count(),
        2
    );

    let loaded = persistence.load_owner(OWNER_A).await.expect("pre-merge");
    let first_person = loaded
        .state
        .source_owner(first_key)
        .expect("first source owner");
    let second_key = source(42, 41, 1, OWNER_A, "same@example.test").key;
    let second_person = loaded
        .state
        .source_owner(second_key)
        .expect("second source owner");
    persistence
        .apply_command_once(&command(12, OWNER_A, 1), move |state| {
            let action = MergePersonsActionV1 {
                logical_owner_id: OWNER_A.to_owned(),
                source_person_id: first_person,
                expected_source_person_revision: revision(state, first_person),
                target_person_id: second_person,
                expected_target_person_revision: revision(state, second_person),
            };
            let digest = merge_persons_action_digest_v1(&action)
                .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            merge_persons_v1(state, action, decision(201, digest, 1_800_000_200))
                .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(commit(112, 12))
        })
        .await
        .expect("merge");
    persistence
        .apply_command_once(&command(13, OWNER_A, 2), move |state| {
            let target_person = state.source_owner(first_key).expect("merged source owner");
            let source_revision = state
                .person(target_person)
                .and_then(|person| person.source_links.get(&first_key))
                .expect("selected source")
                .provenance
                .revision;
            let action = SplitPersonActionV1 {
                logical_owner_id: OWNER_A.to_owned(),
                merged_person_id: first_person,
                expected_merged_person_revision: revision(state, first_person),
                target_person_id: target_person,
                expected_target_person_revision: revision(state, target_person),
                source_selection: vec![SplitSourceSelectionV1 {
                    source: first_key,
                    expected_source_revision: source_revision,
                }],
                profile_fact_selection: Vec::new(),
            };
            let digest = split_person_action_digest_v1(&action)
                .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            split_person_v1(state, action, decision(202, digest, 1_800_000_201))
                .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(commit(113, 13))
        })
        .await
        .expect("selective split");
    let before_decision_restart = persistence
        .load_owner(OWNER_A)
        .await
        .expect("pre-reconnect decision state");
    let decision_restart = PersonsPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("decision reconnect")
        .load_owner(OWNER_A)
        .await
        .expect("decision restart");
    assert_eq!(decision_restart, before_decision_restart);
    let snapshot = decision_restart
        .state
        .snapshot_for_owner_v1(OWNER_A)
        .expect("snapshot");
    assert_eq!(snapshot.lineage.len(), 2);
    assert_eq!(snapshot.decision_receipts.len(), 2);
    assert!(
        snapshot
            .decision_receipts
            .iter()
            .all(|receipt| receipt.outcome.person_revisions.len() == 2)
    );
    let mut expected_merge = vec![(first_person, 2_u64), (second_person, 2_u64)];
    expected_merge.sort();
    let mut expected_split = vec![(first_person, 3_u64), (second_person, 3_u64)];
    expected_split.sort();
    for (decision_id, expected) in [
        ([201_u8; 16], expected_merge),
        ([202_u8; 16], expected_split),
    ] {
        let receipt = snapshot
            .decision_receipts
            .iter()
            .find(|receipt| receipt.decision.decision_id == PublicIdV1(decision_id))
            .expect("exact decision receipt");
        assert_eq!(
            receipt
                .outcome
                .person_revisions
                .iter()
                .map(|entry| (entry.person_id, entry.revision))
                .collect::<Vec<_>>(),
            expected,
        );
    }

    let durable_before_rejections =
        PersonsPersistenceConformanceV1::durable_command_outbox_counts(&persistence, OWNER_A)
            .await
            .expect("durable counts before source/decision rejections");
    assert_eq!(
        persistence
            .apply_command_once(&command(14, OWNER_A, 3), move |state| {
                remove_source_v1(
                    state,
                    OWNER_A,
                    first_key,
                    SourceProvenanceV1 {
                        revision: 1,
                        digest: DigestV1([98; 32]),
                        observed_at: time(1_800_000_202),
                    },
                )
                .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
                Ok(commit(114, 14))
            })
            .await,
        Err(PersonsPersistenceErrorV1::MutationRejected),
        "stale/conflicting source revision rolls back",
    );
    assert_eq!(
        persistence
            .apply_command_once(&command(15, OWNER_A, 3), move |state| {
                let source_person = state.source_owner(second_key).expect("source person");
                let target_person = state.source_owner(first_key).expect("target person");
                let action = MergePersonsActionV1 {
                    logical_owner_id: OWNER_A.to_owned(),
                    source_person_id: source_person,
                    expected_source_person_revision: revision(state, source_person),
                    target_person_id: target_person,
                    expected_target_person_revision: revision(state, target_person),
                };
                let digest = merge_persons_action_digest_v1(&action)
                    .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
                merge_persons_v1(state, action, decision(201, digest, 1_800_000_203))
                    .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
                Ok(commit(115, 15))
            })
            .await,
        Err(PersonsPersistenceErrorV1::MutationRejected),
        "reused decision ID for another action rolls back",
    );
    assert_eq!(
        PersonsPersistenceConformanceV1::durable_command_outbox_counts(&persistence, OWNER_A)
            .await
            .expect("durable counts after source/decision rejections"),
        durable_before_rejections,
    );

    persistence
        .apply_command_once(&command(16, OWNER_A, 3), move |state| {
            remove_source_v1(
                state,
                OWNER_A,
                first_key,
                SourceProvenanceV1 {
                    revision: 2,
                    digest: DigestV1([99; 32]),
                    observed_at: time(1_800_000_300),
                },
            )
            .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(commit(116, 16))
        })
        .await
        .expect("source tombstone");
    let restarted = PersonsPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("reconnect");
    assert_eq!(
        restarted
            .load_owner(OWNER_A)
            .await
            .expect("tombstone restart")
            .aggregate_revision,
        4
    );

    let reused = source(41, 41, 3, OWNER_B, "other@example.test");
    assert_eq!(
        restarted
            .apply_command_once(&command(17, OWNER_B, 0), move |state| {
                observe_source_v1(state, reused)
                    .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
                Ok(commit(117, 17))
            })
            .await,
        Err(PersonsPersistenceErrorV1::StateConflict),
        "global provider tuple cannot be claimed by another owner",
    );

    restarted
        .apply_command_once(&command(18, OWNER_B, 0), |state| {
            create_manual_person_v1(
                state,
                ManualPersonDraftV1 {
                    person_id: PersonIdV1([44; 16]),
                    logical_owner_id: OWNER_B.to_owned(),
                    owner_profile: profile("Grace"),
                    created_at: time(1_800_000_110),
                },
            )
            .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(commit(118, 18))
        })
        .await
        .expect("owner B");
    let evidence = PersonsPersistenceConformanceV1::prove_force_rls(&restarted, OWNER_A, OWNER_B)
        .await
        .expect("actual non-bypass RLS evidence");
    assert_eq!(evidence.visible_owners, vec![OWNER_A.to_owned()]);
    assert_eq!(
        (evidence.cross_owner_updates, evidence.cross_owner_deletes),
        (0, 0)
    );
    assert!(evidence.cross_owner_insert_blocked);
    assert!(evidence.own_profile_update_blocked);
    assert!(evidence.own_profile_delete_blocked);
    assert!(
        PersonsPersistenceConformanceV1::invalid_merged_target_is_blocked(&restarted, OWNER_A)
            .await
            .expect("deferred owner-scoped merged target FK")
    );
    PersonsPersistenceConformanceV1::corrupt_lineage_receipt_linkage(&restarted, OWNER_A)
        .await
        .expect("inject lineage receipt mismatch");
    assert_eq!(
        restarted.load_owner(OWNER_A).await,
        Err(PersonsPersistenceErrorV1::StateConflict),
        "whole-snapshot validation rejects durable lineage/receipt corruption",
    );
}

#[tokio::test]
#[ignore = "requires MAKOSH_PERSONS_POSTGRES_URL disposable PostgreSQL"]
async fn postgres_outbox_pages_are_bounded_and_corrupt_durable_envelopes_fail_closed() {
    let url = database_url();
    let persistence = PersonsPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    PersonsPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    let order_owner = "owner-order";
    let completed_at = 1_800_000_005_000;
    persistence
        .apply_command_once(&command(250, order_owner, 0), |_| {
            Ok(ordered_commit(230, completed_at))
        })
        .await
        .expect("higher command ID at owner revision one");
    persistence
        .apply_command_once(&command(1, order_owner, 1), |_| {
            Ok(ordered_commit(10, completed_at))
        })
        .await
        .expect("lower command ID at owner revision two");
    drop(persistence);
    let persistence = PersonsPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("reconnect for durable order");
    let ordered = persistence
        .load_pending_outbox(order_owner)
        .await
        .expect("durable semantic order after reconnect");
    assert_eq!(
        ordered
            .iter()
            .map(|row| row.resulting_owner_revision)
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 2, 2, 2],
        "owner revision, not timestamp or command hash, is the primary order",
    );
    assert_eq!(
        ordered
            .iter()
            .map(|row| row.outbox_ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 0, 1, 2],
    );
    assert!(ordered.chunks_exact(3).all(|rows| {
        rows[0].semantic_order_key == [0] && rows[1].semantic_order_key < rows[2].semantic_order_key
    }));

    PersonsPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("reset bounded backlog schema");
    let backlog_owner = "owner-c";
    persistence
        .apply_command_once(&command(30, backlog_owner, 0), |state| {
            create_manual_person_v1(
                state,
                ManualPersonDraftV1 {
                    person_id: PersonIdV1([60; 16]),
                    logical_owner_id: backlog_owner.to_owned(),
                    owner_profile: profile("Bounded backlog"),
                    created_at: time(1_800_000_300),
                },
            )
            .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(PersonsCommandCommitV1 {
                terminal_result: envelope(220, b"paged-terminal"),
                owner_events: (0_u16..256).map(indexed_envelope).collect(),
                owner_event_order_keys: (0_u16..256)
                    .map(|index| [vec![1], index.to_be_bytes().to_vec()].concat())
                    .collect(),
                completed_at_unix_millis: 1_800_000_003_000,
            })
        })
        .await
        .expect("257 durable outbox rows");
    let first_page = persistence
        .load_pending_outbox(backlog_owner)
        .await
        .expect("bounded first page");
    assert_eq!(first_page.len(), 256);
    for row in first_page {
        persistence
            .mark_outbox_published(
                backlog_owner,
                row.record.message_id,
                row.record.envelope_sha256,
                1_800_000_004_000,
            )
            .await
            .expect("publish first page");
    }
    assert_eq!(
        persistence
            .load_pending_outbox(backlog_owner)
            .await
            .expect("drain remainder")
            .len(),
        1,
    );

    PersonsPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("reset corruption schema");
    let corruption_owner = "owner-d";
    persistence
        .apply_command_once(&command(31, corruption_owner, 0), |state| {
            create_manual_person_v1(
                state,
                ManualPersonDraftV1 {
                    person_id: PersonIdV1([61; 16]),
                    logical_owner_id: corruption_owner.to_owned(),
                    owner_profile: profile("Corruption sentinel"),
                    created_at: time(1_800_000_310),
                },
            )
            .map_err(|_| PersonsPersistenceErrorV1::MutationRejected)?;
            Ok(commit(131, 31))
        })
        .await
        .expect("corruption fixture");
    PersonsPersistenceConformanceV1::corrupt_terminal_bytes(
        &persistence,
        corruption_owner,
        [31; 16],
    )
    .await
    .expect("corrupt terminal");
    assert_eq!(
        persistence
            .apply_command_once(&command(31, corruption_owner, 0), |_| panic!(
                "corrupt replay"
            ))
            .await,
        Err(PersonsPersistenceErrorV1::HashMismatch),
    );
    let corrupt_outbox_digest = persistence
        .load_pending_outbox(corruption_owner)
        .await
        .expect("uncorrupted outbox")
        .into_iter()
        .find(|row| row.record.message_id == [211; 16])
        .expect("terminal outbox row")
        .record
        .envelope_sha256;
    PersonsPersistenceConformanceV1::corrupt_outbox_bytes(
        &persistence,
        corruption_owner,
        [211; 16],
    )
    .await
    .expect("corrupt outbox");
    assert_eq!(
        persistence.load_pending_outbox(corruption_owner).await,
        Err(PersonsPersistenceErrorV1::HashMismatch),
    );
    assert_eq!(
        persistence
            .mark_outbox_published(
                corruption_owner,
                [211; 16],
                corrupt_outbox_digest,
                1_800_000_400_000,
            )
            .await,
        Err(PersonsPersistenceErrorV1::HashMismatch),
    );
    assert_eq!(
        PersonsPersistenceConformanceV1::outbox_published_at(
            &persistence,
            corruption_owner,
            [211; 16],
        )
        .await
        .expect("corrupt row status"),
        None,
        "hash mismatch leaves corrupt outbox row pending",
    );
    PersonsPersistenceConformanceV1::corrupt_profile_normalization(
        &persistence,
        corruption_owner,
        [61; 16],
    )
    .await
    .expect("corrupt normalized profile ordering");
    assert_eq!(
        persistence.load_owner(corruption_owner).await,
        Err(PersonsPersistenceErrorV1::StateConflict),
        "core reconstitution rejects DB rows that evade aggregate SQL bounds",
    );
}

fn command(seed: u8, owner: &str, expected_revision: u64) -> ApplyPersonsCommandV1 {
    ApplyPersonsCommandV1 {
        logical_owner_id: owner.to_owned(),
        command_message_id: [seed; 16],
        command_envelope_sha256: [seed.wrapping_add(1); 32],
        command_id: [seed.wrapping_add(2); 16],
        command_fingerprint: [seed.wrapping_add(3); 32],
        expected_aggregate_revision: expected_revision,
        received_at_unix_millis: 1_800_000_000_000 + i64::from(seed) * 10,
    }
}

fn commit(seed: u8, clock: i64) -> PersonsCommandCommitV1 {
    PersonsCommandCommitV1 {
        terminal_result: envelope(seed, b"terminal"),
        owner_events: vec![envelope(seed.wrapping_add(80), b"owner-event")],
        owner_event_order_keys: vec![vec![1, seed]],
        completed_at_unix_millis: 1_800_000_000_000 + clock * 100,
    }
}

fn ordered_commit(seed: u8, completed_at_unix_millis: i64) -> PersonsCommandCommitV1 {
    PersonsCommandCommitV1 {
        terminal_result: envelope(seed, b"ordered-terminal"),
        owner_events: vec![
            envelope(seed.wrapping_add(1), b"ordered-person"),
            envelope(seed.wrapping_add(2), b"ordered-source"),
        ],
        owner_event_order_keys: vec![vec![1, seed], vec![3, seed]],
        completed_at_unix_millis,
    }
}

fn envelope(seed: u8, bytes: &[u8]) -> PersonsEnvelopeRecordV1 {
    PersonsEnvelopeRecordV1 {
        message_id: [seed; 16],
        envelope_sha256: Sha256::digest(bytes).into(),
        envelope_bytes: bytes.to_vec(),
    }
}

fn indexed_envelope(index: u16) -> PersonsEnvelopeRecordV1 {
    let bytes = index.to_be_bytes();
    let payload = format!("owner-event-{index}").into_bytes();
    let mut message_id = [1_u8; 16];
    message_id[0] = bytes[0];
    message_id[1] = bytes[1];
    PersonsEnvelopeRecordV1 {
        message_id,
        envelope_sha256: Sha256::digest(&payload).into(),
        envelope_bytes: payload,
    }
}

fn profile(name: &str) -> OwnerProfileV1 {
    OwnerProfileV1 {
        display_name: Some(name.to_owned()),
        given_name: None,
        family_name: None,
        emails: vec!["ada@example.test".to_owned()],
        phones: Vec::new(),
    }
}

fn source(account: u8, source: u8, revision: u64, owner: &str, email: &str) -> SourceObservationV1 {
    SourceObservationV1 {
        logical_owner_id: owner.to_owned(),
        key: SourceLinkKeyV1 {
            integration_public_id: PublicIdV1([9; 16]),
            account_public_id: PublicIdV1([account; 16]),
            provider_source_contact_public_id: PublicIdV1([source; 16]),
        },
        claims: SourceClaimsV1 {
            display_name: Some("Provider Person".to_owned()),
            emails: vec![email.to_owned()],
            phones: Vec::new(),
        },
        provenance: SourceProvenanceV1 {
            revision,
            digest: DigestV1([source; 32]),
            observed_at: time(1_800_000_010 + i64::try_from(revision).expect("revision")),
        },
    }
}

fn time(unix_seconds: i64) -> TimestampV1 {
    TimestampV1 {
        unix_seconds,
        nanos: 0,
    }
}

fn revision(state: &makosh_persons_core::PersonsStateV1, person_id: PersonIdV1) -> u64 {
    state.person(person_id).expect("Person revision").revision
}

fn decision(seed: u8, approved_action_digest: DigestV1, decided_at: i64) -> DecisionProvenanceV1 {
    DecisionProvenanceV1 {
        decision_id: PublicIdV1([seed; 16]),
        review_id: PublicIdV1([77; 16]),
        revision: u64::from(seed),
        decided_by_owner_device_id: PublicIdV1([88; 16]),
        decided_at: time(decided_at),
        approved_action_digest,
    }
}

fn database_url() -> String {
    std::env::var("MAKOSH_PERSONS_POSTGRES_URL").expect("MAKOSH_PERSONS_POSTGRES_URL")
}
