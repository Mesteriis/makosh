use makosh_identity_resolution_core::{
    IdentityMatchEvidenceV1, IdentityResolutionMatchKindV1, IdentityResolutionSourceV1,
};
use makosh_identity_resolution_persistence::{
    ApplyIdentityEvidenceOperationV1, IdentityResolutionDurableCountsV1,
    IdentityResolutionEnvelopeRecordV1, IdentityResolutionPersistenceConformanceV1,
    IdentityResolutionPersistenceErrorV1, IdentityResolutionReplayOutcomeV1,
};
use makosh_persons_api::{
    PersonsActionDigestSourceV1, PersonsIdentityMatchKindV1, persons_identity_match_candidate_id_v1,
};
use sha2::{Digest, Sha256};

const OWNER_A: &str = "owner-a";
const OWNER_B: &str = "owner-b";

fn source(seed: u8) -> IdentityResolutionSourceV1 {
    IdentityResolutionSourceV1 {
        integration_public_id: [seed; 16],
        account_public_id: [seed.wrapping_add(1); 16],
        provider_source_contact_public_id: [seed.wrapping_add(2); 16],
    }
}

fn api_source(value: IdentityResolutionSourceV1) -> PersonsActionDigestSourceV1 {
    PersonsActionDigestSourceV1 {
        integration_public_id: value.integration_public_id,
        account_public_id: value.account_public_id,
        provider_source_contact_public_id: value.provider_source_contact_public_id,
    }
}

fn record(seed: u8) -> IdentityResolutionEnvelopeRecordV1 {
    let bytes = vec![seed, seed.wrapping_add(1), seed.wrapping_add(2)];
    IdentityResolutionEnvelopeRecordV1 {
        message_id: [seed; 16],
        envelope_sha256: Sha256::digest(&bytes).into(),
        envelope_bytes: bytes,
    }
}

fn operation(
    owner: &str,
    evidence_seed: u8,
    revision: u64,
    observed_at: i64,
) -> ApplyIdentityEvidenceOperationV1 {
    let first = source(5);
    let second = source(9);
    let candidate_id = persons_identity_match_candidate_id_v1(
        owner,
        api_source(first),
        api_source(second),
        PersonsIdentityMatchKindV1::NormalizedEmail,
    )
    .expect("candidate id");
    ApplyIdentityEvidenceOperationV1 {
        input: record(evidence_seed),
        evidence: IdentityMatchEvidenceV1 {
            evidence_event_id: [evidence_seed; 16],
            candidate_id,
            logical_owner_id: owner.to_owned(),
            first_person_id: [2; 16],
            second_person_id: [3; 16],
            first_source: first,
            second_source: second,
            match_kind: IdentityResolutionMatchKindV1::NormalizedEmail,
            observed_at_unix_millis: observed_at,
            resulting_owner_revision: revision,
        },
        proposal: record(evidence_seed.wrapping_add(80)),
        completed_at_unix_millis: observed_at + 1,
    }
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn identity_resolution_replay_refresh_claim_and_rls_are_durable() {
    let url =
        std::env::var("MAKOSH_IDENTITY_RESOLUTION_POSTGRES_URL").expect("managed disposable URL");
    let persistence = IdentityResolutionPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    IdentityResolutionPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");

    let first = operation(OWNER_A, 1, 7, 1_000);
    assert!(matches!(
        persistence.apply_once(&first).await.expect("first apply"),
        IdentityResolutionReplayOutcomeV1::Applied(ref value) if value == &first.proposal
    ));
    let first_counts = IdentityResolutionDurableCountsV1 {
        candidates: 1,
        inbox: 1,
        outbox: 1,
        pending_outbox: 1,
    };
    assert_eq!(
        IdentityResolutionPersistenceConformanceV1::durable_counts(&persistence, OWNER_A)
            .await
            .expect("first counts"),
        first_counts,
    );
    assert!(matches!(
        persistence.apply_once(&first).await.expect("exact replay"),
        IdentityResolutionReplayOutcomeV1::Replayed(ref value) if value == &first.proposal
    ));

    let second = operation(OWNER_A, 2, 8, 1_100);
    persistence.apply_once(&second).await.expect("refresh");
    assert!(matches!(
        persistence.apply_once(&first).await.expect("old exact replay"),
        IdentityResolutionReplayOutcomeV1::Replayed(ref value) if value == &first.proposal
    ));
    let refreshed_counts = IdentityResolutionDurableCountsV1 {
        candidates: 1,
        inbox: 2,
        outbox: 2,
        pending_outbox: 2,
    };
    assert_eq!(
        IdentityResolutionPersistenceConformanceV1::durable_counts(&persistence, OWNER_A)
            .await
            .expect("refresh counts"),
        refreshed_counts,
    );

    let mut changed_replay = first.clone();
    changed_replay.input.envelope_bytes.push(99);
    changed_replay.input.envelope_sha256 =
        Sha256::digest(&changed_replay.input.envelope_bytes).into();
    assert_eq!(
        persistence.apply_once(&changed_replay).await,
        Err(IdentityResolutionPersistenceErrorV1::Conflict),
    );
    let stale = operation(OWNER_A, 3, 8, 1_200);
    assert_eq!(
        persistence.apply_once(&stale).await,
        Err(IdentityResolutionPersistenceErrorV1::RevisionConflict),
    );
    assert_eq!(
        IdentityResolutionPersistenceConformanceV1::durable_counts(&persistence, OWNER_A)
            .await
            .expect("rollback counts"),
        refreshed_counts,
    );

    let other = operation(OWNER_B, 4, 1, 1_000);
    persistence.apply_once(&other).await.expect("other owner");
    let rls =
        IdentityResolutionPersistenceConformanceV1::rls_evidence(&persistence, OWNER_A, OWNER_B)
            .await
            .expect("RLS");
    assert_eq!(rls.visible_owners, vec![OWNER_A.to_owned()]);
    assert_eq!(rls.cross_owner_updates, 0);
    assert_eq!(rls.cross_owner_deletes, 0);
    assert!(rls.cross_owner_insert_blocked);

    let first_claim = persistence
        .claim_next_pending_outbox(OWNER_A)
        .await
        .expect("first claim")
        .expect("first pending");
    let second_claim = persistence
        .claim_next_pending_outbox(OWNER_A)
        .await
        .expect("overlap claim")
        .expect("second pending progresses");
    assert_ne!(
        first_claim.record().record.message_id,
        second_claim.record().record.message_id
    );
    let first_id = first_claim.record().record.message_id;
    drop(first_claim);
    drop(second_claim);
    let reclaimed = persistence
        .claim_next_pending_outbox(OWNER_A)
        .await
        .expect("reclaim")
        .expect("rollback made row retryable");
    assert_eq!(reclaimed.record().record.message_id, first_id);
    let sha = reclaimed.record().record.envelope_sha256;
    reclaimed
        .mark_published(sha, 2_000)
        .await
        .expect("publish CAS");
}
