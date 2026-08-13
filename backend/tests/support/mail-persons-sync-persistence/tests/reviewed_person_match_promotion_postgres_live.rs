use makosh_persons_api::{persons_owner_partition_id_v1, wire::PersonCommandSucceededV1};
use makosh_persons_runtime::transport::{
    PersonsEnvelopeContextV1, build_persons_command_succeeded_outbox_record_v1,
};
use makosh_review_person_match_candidate_api::{
    ReviewPersonMatchCandidateEnvelopeContextV1,
    build_review_person_match_candidate_approved_outbox_record_v1,
    wire::{
        AttachPersonSourceReviewActionV1, PersonMatchCandidateApprovedActionV1,
        PersonMatchCandidateApprovedForPromotionV1, PublicPersonSourceIdentityV1,
        person_match_candidate_approved_action_v1::Action,
    },
};
use makosh_review_person_match_candidate_promotion_api::wire::{
    ReviewPersonMatchCandidatePromotionFailureCodeV1, ReviewPersonMatchCandidatePromotionResultV1,
};
use makosh_reviewed_person_match_candidate_promotion_persistence::{
    PersistReviewedPersonMatchApprovalV1, PersistReviewedPersonMatchTerminalV1,
    ReviewedPersonMatchCandidatePromotionCountsV1, ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1,
    ReviewedPersonMatchCandidatePromotionReplayV1,
};
use makosh_reviewed_person_match_candidate_promotion_runtime::{
    ReviewedPersonMatchCandidatePromotionExecutionContextV1,
    process_person_match_candidate_approval_v1, process_persons_terminal_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

const OWNER: &str = "owner-a";
fn envelope(seed: u8) -> ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
    let bytes = vec![seed, seed.wrapping_add(1)];
    ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
        message_id: [seed; 16],
        envelope_sha256: Sha256::digest(&bytes).into(),
        envelope_bytes: bytes,
    }
}
fn approval() -> PersistReviewedPersonMatchApprovalV1 {
    PersistReviewedPersonMatchApprovalV1 {
        logical_owner_id: OWNER.into(),
        approval: envelope(1),
        review_id: [2; 16],
        candidate_id: [3; 16],
        candidate_digest: [4; 32],
        decision_id: [5; 16],
        decision_revision: 2,
        approved_action_digest: [6; 32],
        persons_command_id: [7; 16],
        persons_command_fingerprint: [8; 32],
        persons_command: envelope(7),
        occurred_at_unix_millis: 1_000,
    }
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn promotion_atomic_replay_terminal_restart_and_unknown_ack_ignore() {
    let url = std::env::var("MAKOSH_REVIEWED_PERSON_MATCH_PROMOTION_POSTGRES_URL").expect("URL");
    let persistence =
        ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1::connect_url(&url)
            .await
            .expect("connect");
    ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    let input = approval();
    assert_eq!(
        persistence
            .persist_approval_once(&input)
            .await
            .expect("approval"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Applied
    );
    assert_eq!(
        persistence
            .persist_approval_once(&input)
            .await
            .expect("replay"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Replayed
    );
    assert_eq!(
        ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1::counts(&persistence, OWNER)
            .await
            .expect("counts"),
        ReviewedPersonMatchCandidatePromotionCountsV1 {
            requests: 1,
            result_inbox: 0,
            outbox: 1,
            pending_outbox: 1
        }
    );
    let terminal = PersistReviewedPersonMatchTerminalV1 {
        logical_owner_id: OWNER.into(),
        persons_result: envelope(9),
        persons_command_id: [7; 16],
        review_id: [2; 16],
        candidate_id: [3; 16],
        succeeded: true,
        failure_code: None,
        review_result: envelope(10),
        completed_at_unix_millis: 2_000,
    };
    assert_eq!(
        persistence
            .persist_terminal_once(&terminal)
            .await
            .expect("terminal"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Applied
    );
    assert_eq!(
        persistence
            .persist_terminal_once(&terminal)
            .await
            .expect("terminal replay"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Replayed
    );
    assert_eq!(
        ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1::counts(&persistence, OWNER)
            .await
            .expect("terminal counts"),
        ReviewedPersonMatchCandidatePromotionCountsV1 {
            requests: 1,
            result_inbox: 1,
            outbox: 2,
            pending_outbox: 2
        }
    );
    drop(persistence);
    let restarted =
        ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1::connect_url(&url)
            .await
            .expect("restart");
    assert!(
        restarted
            .load_correlation(OWNER, [7; 16])
            .await
            .expect("correlation")
            .completed
    );
    let unknown = build_persons_command_succeeded_outbox_record_v1(
        [91; 16],
        persons_owner_partition_id_v1(OWNER).expect("owner partition"),
        PersonCommandSucceededV1 {
            command_id: vec![91; 16],
            logical_owner_id: OWNER.into(),
            resulting_owner_revision: 1,
            ..Default::default()
        },
        &PersonsEnvelopeContextV1 {
            module_id: "makosh-persons-runtime".into(),
            runtime_instance_id: "persons-runtime-1".into(),
            runtime_generation: 3,
            recorded_at_unix_seconds: 3,
            recorded_at_nanos: 0,
        },
    )
    .expect("unknown terminal");
    let context = ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
        logical_owner_id: OWNER.into(),
        runtime_instance_id: "promotion-runtime-1".into(),
        runtime_generation: 4,
        now_unix_millis: 3_000,
    };
    assert_eq!(
        process_persons_terminal_v1(&restarted, &unknown, &context)
            .await
            .expect("unknown ignored"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Replayed
    );
    assert_eq!(
        ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1::counts(&restarted, OWNER)
            .await
            .expect("unchanged"),
        ReviewedPersonMatchCandidatePromotionCountsV1 {
            requests: 1,
            result_inbox: 1,
            outbox: 2,
            pending_outbox: 2
        }
    );
    let replay_owner = "owner-d";
    let mut replay_approval = approval();
    replay_approval.logical_owner_id = replay_owner.into();
    replay_approval.approval = envelope(70);
    replay_approval.review_id = [71; 16];
    replay_approval.candidate_id = [72; 16];
    replay_approval.decision_id = [73; 16];
    replay_approval.persons_command_id = [74; 16];
    replay_approval.persons_command = envelope(74);
    restarted
        .persist_approval_once(&replay_approval)
        .await
        .expect("terminal replay approval");
    let replay_terminal = build_persons_command_succeeded_outbox_record_v1(
        [74; 16],
        persons_owner_partition_id_v1(replay_owner).expect("owner partition"),
        PersonCommandSucceededV1 {
            command_id: vec![74; 16],
            logical_owner_id: replay_owner.into(),
            resulting_owner_revision: 1,
            ..Default::default()
        },
        &PersonsEnvelopeContextV1 {
            module_id: "makosh-persons-runtime".into(),
            runtime_instance_id: "persons-runtime-replay".into(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 5,
            recorded_at_nanos: 0,
        },
    )
    .expect("known Persons terminal");
    assert_eq!(
        process_persons_terminal_v1(
            &restarted,
            &replay_terminal,
            &ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
                logical_owner_id: replay_owner.into(),
                runtime_instance_id: "promotion-runtime-replay".into(),
                runtime_generation: 8,
                now_unix_millis: 5_000,
            },
        )
        .await
        .expect("first terminal"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Applied
    );
    assert_eq!(
        process_persons_terminal_v1(
            &restarted,
            &replay_terminal,
            &ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
                logical_owner_id: replay_owner.into(),
                runtime_instance_id: "promotion-runtime-replay".into(),
                runtime_generation: 8,
                now_unix_millis: 4_000,
            },
        )
        .await
        .expect("terminal exact replay before freshness"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Replayed
    );
    let first_claim = restarted
        .claim_next_pending_outbox(OWNER)
        .await
        .expect("claim")
        .expect("pending");
    assert!(
        restarted
            .claim_next_pending_outbox(OWNER)
            .await
            .expect("overlap")
            .is_some(),
        "SKIP LOCKED must make progress to the next durable sequence"
    );
    let first_id = first_claim.record().record.message_id;
    let first_sha = first_claim.record().record.envelope_sha256;
    drop(first_claim);
    let retry = restarted
        .claim_next_pending_outbox(OWNER)
        .await
        .expect("retry")
        .expect("rolled back");
    assert_eq!(retry.record().record.message_id, first_id);
    retry
        .mark_published(first_sha, 3_000)
        .await
        .expect("publish CAS");

    let invalid_owner = "owner-b";
    let invalid_approval = build_review_person_match_candidate_approved_outbox_record_v1(
        PersonMatchCandidateApprovedForPromotionV1 {
            review_id: vec![31; 16],
            candidate_id: vec![32; 16],
            candidate_digest: vec![33; 32],
            decision_id: vec![34; 16],
            decision_revision: 2,
            decided_by_owner_device_id: vec![35; 16],
            decided_at_unix_millis: 4_000,
            approved_action: Some(PersonMatchCandidateApprovedActionV1 {
                action: Some(Action::Attach(AttachPersonSourceReviewActionV1 {
                    from_person_id: vec![36; 16],
                    expected_from_person_revision: 1,
                    to_person_id: vec![37; 16],
                    expected_to_person_revision: 1,
                    source: Some(PublicPersonSourceIdentityV1 {
                        integration_public_id: vec![38; 16],
                        account_public_id: vec![39; 16],
                        provider_source_contact_public_id: vec![40; 16],
                    }),
                    expected_source_revision: 1,
                })),
            }),
            approved_action_digest: vec![41; 32],
            logical_owner_id: invalid_owner.into(),
        },
        &ReviewPersonMatchCandidateEnvelopeContextV1 {
            module_id: "makosh-review-person-match-candidate-runtime".into(),
            runtime_instance_id: "review-runtime-2".into(),
            runtime_generation: 5,
            recorded_at_unix_millis: 4_000,
        },
    )
    .expect("invalid approval envelope remains structurally exact");
    let invalid_context = ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
        logical_owner_id: invalid_owner.into(),
        runtime_instance_id: "promotion-runtime-2".into(),
        runtime_generation: 6,
        now_unix_millis: 4_000,
    };
    assert_eq!(
        process_person_match_candidate_approval_v1(
            &restarted,
            &invalid_approval,
            &invalid_context,
        )
        .await
        .expect("digest mismatch is a bounded terminal"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Applied
    );
    assert_eq!(
        process_person_match_candidate_approval_v1(
            &restarted,
            &invalid_approval,
            &ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
                now_unix_millis: 3_000,
                ..invalid_context.clone()
            },
        )
        .await
        .expect("exact failure replay"),
        ReviewedPersonMatchCandidatePromotionReplayV1::Replayed
    );
    assert_eq!(
        ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1::counts(
            &restarted,
            invalid_owner,
        )
        .await
        .expect("local terminal counts"),
        ReviewedPersonMatchCandidatePromotionCountsV1 {
            requests: 1,
            result_inbox: 0,
            outbox: 1,
            pending_outbox: 1,
        }
    );
    let local_failure = restarted
        .claim_next_pending_outbox(invalid_owner)
        .await
        .expect("local claim")
        .expect("typed Review result");
    assert_eq!(local_failure.record().semantic_kind, 2);
    let result = ReviewPersonMatchCandidatePromotionResultV1::decode(
        makosh_events_protocol::v1::DurableEnvelopeV1::decode(
            local_failure.record().record.envelope_bytes.as_slice(),
        )
        .expect("result envelope")
        .payload
        .as_slice(),
    )
    .expect("result payload");
    assert!(result.persons_command_id.is_none());
    assert_eq!(
        result.failure_code,
        ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeActionDigestMismatch as i32
    );
}
