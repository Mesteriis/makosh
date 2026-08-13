use makosh_persons_api::{
    persons_owner_partition_id_v1,
    wire::{
        IdentityMatchKindV1, PersonReviewCandidateRaisedEventV1, ProviderSourceIdentityV1,
        TimestampV1,
    },
};
use makosh_persons_runtime::transport::{
    PersonsEnvelopeContextV1, build_persons_review_candidate_outbox_record_v1,
};
use makosh_review_person_match_candidate_core::{
    PersonMatchCandidateApprovedActionV1, PersonMatchCandidateDecisionV1,
    PersonMatchCandidateEvidenceV1, PersonMatchCandidatePromotionStatusV1, PersonMatchKindV1,
    PublicPersonSourceIdentityV1, person_match_candidate_evidence_digest_v1,
};
use makosh_review_person_match_candidate_persistence::{
    DecidePersonMatchCandidateOperationV1, PersistPersonMatchCandidatePromotionResultV1,
    ReviewPersonMatchCandidateDurableCountsV1, ReviewPersonMatchCandidateEnvelopeRecordV1,
    ReviewPersonMatchCandidatePersistenceConformanceV1,
    ReviewPersonMatchCandidatePersistenceErrorV1, ReviewPersonMatchCandidateReplayOutcomeV1,
    SubmitPersonMatchCandidateOperationV1,
};
use makosh_review_person_match_candidate_runtime::{
    ReviewPersonMatchCandidateExecutionContextV1, process_persons_review_candidate_v1,
};
use sha2::{Digest, Sha256};

const OWNER_A: &str = "owner-a";
const OWNER_B: &str = "owner-b";

fn envelope(seed: u8) -> ReviewPersonMatchCandidateEnvelopeRecordV1 {
    let bytes = vec![seed, seed.wrapping_add(1), seed.wrapping_add(2)];
    ReviewPersonMatchCandidateEnvelopeRecordV1 {
        message_id: [seed; 16],
        envelope_sha256: Sha256::digest(&bytes).into(),
        envelope_bytes: bytes,
    }
}

fn source(seed: u8) -> PublicPersonSourceIdentityV1 {
    PublicPersonSourceIdentityV1 {
        integration_public_id: [seed; 16],
        account_public_id: [seed.wrapping_add(1); 16],
        provider_source_contact_public_id: [seed.wrapping_add(2); 16],
    }
}

fn submit(owner: &str, seed: u8) -> SubmitPersonMatchCandidateOperationV1 {
    let mut evidence = PersonMatchCandidateEvidenceV1 {
        evidence_event_id: [seed; 16],
        candidate_id: [seed.wrapping_add(1); 16],
        logical_owner_id: owner.to_owned(),
        first_person_id: [seed.wrapping_add(2); 16],
        second_person_id: [seed.wrapping_add(3); 16],
        first_source: source(seed.wrapping_add(4)),
        second_source: source(seed.wrapping_add(8)),
        match_kind: PersonMatchKindV1::NormalizedEmail,
        observed_at_unix_millis: 1_000,
        resulting_owner_revision: 7,
        candidate_digest: [0; 32],
    };
    evidence.candidate_digest =
        person_match_candidate_evidence_digest_v1(&evidence).expect("candidate digest");
    SubmitPersonMatchCandidateOperationV1 {
        command: envelope(seed.wrapping_add(20)),
        evidence,
        submitted_result: envelope(seed.wrapping_add(40)),
        expected_existing_revision: None,
        received_at_unix_millis: 1_001,
    }
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn review_queue_replay_restart_rls_and_conflict_rollback() {
    let url = std::env::var("MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_POSTGRES_URL")
        .expect("managed disposable URL");
    let persistence = ReviewPersonMatchCandidatePersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    ReviewPersonMatchCandidatePersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");

    let first = submit(OWNER_A, 1);
    let applied = persistence.submit_once(&first).await.expect("submit");
    let review = match applied {
        ReviewPersonMatchCandidateReplayOutcomeV1::Applied(review) => review,
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(_) => panic!("first apply"),
    };
    let before =
        ReviewPersonMatchCandidatePersistenceConformanceV1::durable_counts(&persistence, OWNER_A)
            .await
            .expect("counts");
    assert_eq!(
        before,
        ReviewPersonMatchCandidateDurableCountsV1 {
            reviews: 1,
            inbox: 1,
            outbox: 1,
            pending_outbox: 1,
        }
    );
    assert!(matches!(
        persistence.submit_once(&first).await.expect("exact replay"),
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(replayed) if replayed == review
    ));
    assert_eq!(
        ReviewPersonMatchCandidatePersistenceConformanceV1::durable_counts(&persistence, OWNER_A,)
            .await
            .expect("replay counts"),
        before
    );

    let mut conflict = first.clone();
    conflict.submitted_result = envelope(99);
    assert_eq!(
        persistence.submit_once(&conflict).await,
        Err(ReviewPersonMatchCandidatePersistenceErrorV1::Conflict)
    );
    assert_eq!(
        ReviewPersonMatchCandidatePersistenceConformanceV1::durable_counts(&persistence, OWNER_A,)
            .await
            .expect("rollback counts"),
        before
    );

    let second = submit(OWNER_B, 2);
    persistence.submit_once(&second).await.expect("other owner");
    let rls = ReviewPersonMatchCandidatePersistenceConformanceV1::rls_evidence(
        &persistence,
        OWNER_A,
        OWNER_B,
    )
    .await
    .expect("RLS");
    assert_eq!(rls.visible_owners, vec![OWNER_A.to_owned()]);
    assert_eq!(rls.cross_owner_updates, 0);
    assert_eq!(rls.cross_owner_deletes, 0);
    assert!(rls.cross_owner_insert_blocked);

    drop(persistence);
    let restarted = ReviewPersonMatchCandidatePersistenceConformanceV1::connect_url(&url)
        .await
        .expect("restart");
    assert_eq!(
        restarted
            .load_review(OWNER_A, review.review_id)
            .await
            .expect("review after restart"),
        review
    );
    assert_eq!(
        ReviewPersonMatchCandidatePersistenceConformanceV1::durable_counts(&restarted, OWNER_A,)
            .await
            .expect("restart counts"),
        before
    );

    let first_claim = restarted
        .claim_next_pending_outbox(OWNER_A)
        .await
        .expect("claim")
        .expect("pending");
    assert!(
        restarted
            .claim_next_pending_outbox(OWNER_A)
            .await
            .expect("overlapping generation")
            .is_none()
    );
    let claimed_id = first_claim.record().record.message_id;
    let claimed_sha = first_claim.record().record.envelope_sha256;
    drop(first_claim);
    let retry_claim = restarted
        .claim_next_pending_outbox(OWNER_A)
        .await
        .expect("retry claim")
        .expect("rollback made row retryable");
    assert_eq!(retry_claim.record().record.message_id, claimed_id);
    retry_claim
        .mark_published(claimed_sha, 2_000)
        .await
        .expect("publish CAS");
    assert!(
        restarted
            .claim_next_pending_outbox(OWNER_A)
            .await
            .expect("drained")
            .is_none()
    );

    let local_owner = "owner-c";
    let local_submission = submit(local_owner, 50);
    let local_review = match restarted
        .submit_once(&local_submission)
        .await
        .expect("local submission")
    {
        ReviewPersonMatchCandidateReplayOutcomeV1::Applied(review) => review,
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(_) => panic!("local first apply"),
    };
    let local_decision = DecidePersonMatchCandidateOperationV1 {
        logical_owner_id: local_owner.into(),
        command: envelope(80),
        review_id: local_review.review_id,
        expected_review_revision: 1,
        decision: PersonMatchCandidateDecisionV1::Approve {
            action: PersonMatchCandidateApprovedActionV1::Merge {
                source_person_id: [81; 16],
                expected_source_person_revision: 1,
                target_person_id: [82; 16],
                expected_target_person_revision: 1,
            },
            approved_action_digest: [83; 32],
        },
        decided_by_owner_device_id: [84; 16],
        decided_at_unix_millis: 1_100,
        approved_event: Some(envelope(85)),
        received_at_unix_millis: 1_101,
    };
    let approved = match restarted
        .decide_once(&local_decision)
        .await
        .expect("local approval")
    {
        ReviewPersonMatchCandidateReplayOutcomeV1::Applied(review) => review,
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(_) => panic!("approval first apply"),
    };
    let local_failure = PersistPersonMatchCandidatePromotionResultV1 {
        logical_owner_id: local_owner.into(),
        result: envelope(86),
        review_id: approved.review_id,
        candidate_id: approved.evidence.candidate_id,
        decision_id: local_decision.command.message_id,
        persons_command_id: None,
        expected_review_revision: approved.review_revision,
        succeeded: false,
        completed_at_unix_millis: 1_200,
    };
    let failed = match restarted
        .persist_promotion_result_once(&local_failure)
        .await
        .expect("local terminal")
    {
        ReviewPersonMatchCandidateReplayOutcomeV1::Applied(review) => review,
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(_) => panic!("local first terminal"),
    };
    assert_eq!(
        failed.promotion_status,
        PersonMatchCandidatePromotionStatusV1::Failed
    );
    assert!(matches!(
        restarted
            .persist_promotion_result_once(&local_failure)
            .await
            .expect("local exact replay"),
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(replayed) if replayed == failed
    ));
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn exact_persons_candidate_envelope_is_validated_and_persisted_atomically() {
    let url = std::env::var("MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_POSTGRES_URL")
        .expect("managed disposable URL");
    let persistence = ReviewPersonMatchCandidatePersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    ReviewPersonMatchCandidatePersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    let payload = PersonReviewCandidateRaisedEventV1 {
        event_id: vec![11; 16],
        candidate_id: vec![12; 16],
        logical_owner_id: OWNER_A.to_owned(),
        first_person_id: vec![13; 16],
        second_person_id: vec![14; 16],
        first_source: Some(ProviderSourceIdentityV1 {
            integration_public_id: vec![15; 16],
            account_public_id: vec![16; 16],
            provider_source_contact_public_id: vec![17; 16],
        }),
        second_source: Some(ProviderSourceIdentityV1 {
            integration_public_id: vec![18; 16],
            account_public_id: vec![19; 16],
            provider_source_contact_public_id: vec![20; 16],
        }),
        match_kind: IdentityMatchKindV1::IdentityMatchKindNormalizedEmail as i32,
        observed_at: Some(TimestampV1 {
            unix_seconds: 1,
            nanos: 0,
        }),
        resulting_owner_revision: 9,
    };
    let owner_partition = persons_owner_partition_id_v1(OWNER_A).expect("owner partition");
    let candidate = build_persons_review_candidate_outbox_record_v1(
        [21; 16],
        owner_partition,
        [11; 16],
        owner_partition,
        payload,
        &PersonsEnvelopeContextV1 {
            module_id: "makosh-persons-runtime".to_owned(),
            runtime_instance_id: "persons-runtime-1".to_owned(),
            runtime_generation: 3,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 0,
        },
    )
    .expect("Persons candidate envelope");
    let context = ReviewPersonMatchCandidateExecutionContextV1 {
        logical_owner_id: OWNER_A.to_owned(),
        runtime_instance_id: "review-runtime-1".to_owned(),
        runtime_generation: 4,
        now_unix_millis: 2_000,
    };
    let first = process_persons_review_candidate_v1(&persistence, &candidate, &context)
        .await
        .expect("processed");
    let review = match first {
        ReviewPersonMatchCandidateReplayOutcomeV1::Applied(review) => review,
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(_) => panic!("first apply"),
    };
    assert_eq!(review.evidence.candidate_id, [12; 16]);
    assert!(matches!(
        process_persons_review_candidate_v1(&persistence, &candidate, &context)
            .await
            .expect("exact replay"),
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(replayed) if replayed == review
    ));
    assert_eq!(
        ReviewPersonMatchCandidatePersistenceConformanceV1::durable_counts(&persistence, OWNER_A,)
            .await
            .expect("counts"),
        ReviewPersonMatchCandidateDurableCountsV1 {
            reviews: 1,
            inbox: 1,
            outbox: 1,
            pending_outbox: 1,
        }
    );

    let refreshed_payload = PersonReviewCandidateRaisedEventV1 {
        event_id: vec![31; 16],
        candidate_id: vec![12; 16],
        logical_owner_id: OWNER_A.to_owned(),
        first_person_id: vec![14; 16],
        second_person_id: vec![13; 16],
        first_source: Some(ProviderSourceIdentityV1 {
            integration_public_id: vec![18; 16],
            account_public_id: vec![19; 16],
            provider_source_contact_public_id: vec![20; 16],
        }),
        second_source: Some(ProviderSourceIdentityV1 {
            integration_public_id: vec![15; 16],
            account_public_id: vec![16; 16],
            provider_source_contact_public_id: vec![17; 16],
        }),
        match_kind: IdentityMatchKindV1::IdentityMatchKindNormalizedPhone as i32,
        observed_at: Some(TimestampV1 {
            unix_seconds: 3,
            nanos: 0,
        }),
        resulting_owner_revision: 10,
    };
    let refreshed_candidate = build_persons_review_candidate_outbox_record_v1(
        [32; 16],
        owner_partition,
        [31; 16],
        owner_partition,
        refreshed_payload,
        &PersonsEnvelopeContextV1 {
            module_id: "makosh-persons-runtime".to_owned(),
            runtime_instance_id: "persons-runtime-2".to_owned(),
            runtime_generation: 4,
            recorded_at_unix_seconds: 4,
            recorded_at_nanos: 0,
        },
    )
    .expect("refreshed Persons candidate envelope");
    let refreshed = match process_persons_review_candidate_v1(
        &persistence,
        &refreshed_candidate,
        &ReviewPersonMatchCandidateExecutionContextV1 {
            logical_owner_id: OWNER_A.to_owned(),
            runtime_instance_id: "review-runtime-2".to_owned(),
            runtime_generation: 5,
            now_unix_millis: 4_000,
        },
    )
    .await
    .expect("refresh pending aggregate")
    {
        ReviewPersonMatchCandidateReplayOutcomeV1::Applied(review) => review,
        ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(_) => panic!("refresh must apply"),
    };
    assert_eq!(refreshed.review_id, review.review_id);
    assert_eq!(refreshed.review_revision, 2);
    assert_eq!(refreshed.evidence.evidence_event_id, [31; 16]);
    assert_eq!(refreshed.evidence.first_person_id, [14; 16]);
    assert_eq!(refreshed.evidence.second_person_id, [13; 16]);
    assert_eq!(
        refreshed.evidence.match_kind,
        PersonMatchKindV1::NormalizedPhone
    );
    assert_eq!(refreshed.evidence.resulting_owner_revision, 10);
    drop(persistence);
    let restarted = ReviewPersonMatchCandidatePersistenceConformanceV1::connect_url(&url)
        .await
        .expect("restart after refresh");
    assert_eq!(
        restarted
            .load_review(OWNER_A, review.review_id)
            .await
            .expect("refreshed evidence after restart"),
        refreshed
    );
    assert_eq!(
        ReviewPersonMatchCandidatePersistenceConformanceV1::durable_counts(&restarted, OWNER_A)
            .await
            .expect("refresh counts"),
        ReviewPersonMatchCandidateDurableCountsV1 {
            reviews: 1,
            inbox: 2,
            outbox: 2,
            pending_outbox: 2,
        }
    );
}
