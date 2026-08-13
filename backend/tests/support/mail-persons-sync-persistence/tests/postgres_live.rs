use makosh_mail_address_book_contract::{
    MAIL_RUNTIME_MODULE_ID_V1, MailAddressBookEnvelopeContextV1,
    MailAddressBookResultEnvelopeContextV1, build_fetch_mail_person_source_page_command_v1,
    build_mail_person_source_page_completed_v1, build_mail_person_source_updated_v1,
    mail_person_source_claims_digest_v1,
    wire_person_source::{
        FetchMailPersonSourcePageCommandV1, MailPersonSourceClaimsV1, MailPersonSourceIdentityV1,
        MailPersonSourcePageCompletedV1, MailPersonSourceProvenanceV1, MailPersonSourceUpdatedV1,
    },
};
use makosh_mail_address_book_persistence::{
    MailAddressBookPersistenceConformanceV1, MailPersonSourceAccountMappingV1,
    MailPersonSourceAtomicFetchCommitV1, MailPersonSourceChangeKindV1,
    MailPersonSourceEnvelopeRecordV1, MailPersonSourceFetchOutputV1, MailPersonSourceObservationV1,
    MailPersonSourceRemovalPageCommitV1, MailPersonSourceSnapshotCommitV1,
    mail_person_source_semantic_order_key_v1,
};
use makosh_mail_persons_sync_persistence::{
    ApplyMailPersonsSyncAccountLifecycleV1, BeginMailPersonsSyncRunV1,
    CompleteMailPersonsSyncPageV1, MailPersonsSyncAccountLifecycleKindV1,
    MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncExpiredRunContextV1,
    MailPersonsSyncPageContinuationV1, MailPersonsSyncPersistenceConformanceV1,
    MailPersonsSyncPersistenceErrorV1, MailPersonsSyncSemanticKindV1,
    MailPersonsSyncStoredRejectCodeV1, RecordMailPersonsSyncPersonsTerminalV1,
    RejectMailPersonsSyncAccountBusyV1, StageMailPersonsSyncSourceV1,
};
use makosh_mail_runtime::person_source_producer::{
    MailPersonSourceSyntheticRemovalV1, build_synthetic_removal_page_v1,
    ensure_public_account_mapping_v1, ensure_public_account_ready_v1,
    mail_person_source_fetch_id_v1, record_public_account_retired_v1,
};
use prost_types::Timestamp;
use sha2::Digest;

const OWNER: &str = "owner-a";

fn id(seed: u8) -> [u8; 16] {
    [seed; 16]
}
fn digest(seed: u8) -> [u8; 32] {
    [seed; 32]
}
fn envelope(seed: u8) -> MailPersonsSyncEnvelopeRecordV1 {
    let envelope = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: id(seed).to_vec(),
            run_id: id(250).to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(251).to_vec(),
            page_sequence: 1,
            page_size: 500,
        },
        3,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "persistence-fixture".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("durable envelope");
    MailPersonsSyncEnvelopeRecordV1::new(*envelope.message_id(), envelope.exact_bytes().to_vec())
        .expect("envelope")
}

fn begin() -> BeginMailPersonsSyncRunV1 {
    BeginMailPersonsSyncRunV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        run_fingerprint: digest(3),
        scheduler_command: envelope(4),
        scheduler_acceptance: envelope(5),
        initial_fetch: envelope(6),
        lease_epoch: 1,
        lease_expires_at_unix_millis: 2_000,
        received_at_unix_millis: 1_000,
    }
}

fn source(seed: u8, kind: u8) -> StageMailPersonsSyncSourceV1 {
    StageMailPersonsSyncSourceV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        observation: envelope(seed),
        integration_public_id: id(20),
        provider_source_contact_public_id: id(seed),
        change_kind: kind,
        source_revision: 1,
        source_digest: digest(seed),
        persons_command_id: id(seed + 40),
        persons_command_fingerprint: digest(seed + 50),
        persons_command: envelope(seed + 40),
        received_at_unix_millis: 1_100,
    }
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_account_retirement_requires_the_exact_stable_mapping_revision() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let persistence = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    MailPersonsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    let ready = ApplyMailPersonsSyncAccountLifecycleV1 {
        logical_owner_id: OWNER.to_owned(),
        integration_public_id: id(70),
        account_public_id: id(71),
        mapping_revision: 1,
        kind: MailPersonsSyncAccountLifecycleKindV1::Ready,
        lifecycle: envelope(72),
        processed_at_unix_millis: 1_000,
    };
    persistence
        .apply_account_lifecycle_once(&ready, |_| Ok(envelope(73)))
        .await
        .expect("persist Ready binding");
    let before = MailPersonsSyncPersistenceConformanceV1::account_lifecycle_evidence(
        &persistence,
        OWNER,
        id(71),
    )
    .await
    .expect("Ready evidence");
    assert_eq!((before.inbox_count, before.outbox_count), (1, 1));
    assert_eq!(before.mapping_revision, Some(1));
    assert_eq!(
        before.state,
        Some(MailPersonsSyncAccountLifecycleKindV1::Ready)
    );

    let higher_revision_retired = ApplyMailPersonsSyncAccountLifecycleV1 {
        mapping_revision: 2,
        kind: MailPersonsSyncAccountLifecycleKindV1::Retired,
        lifecycle: envelope(74),
        processed_at_unix_millis: 1_100,
        ..ready.clone()
    };
    assert_eq!(
        persistence
            .apply_account_lifecycle_once(&higher_revision_retired, |_| Ok(envelope(75)))
            .await,
        Err(MailPersonsSyncPersistenceErrorV1::StateConflict),
    );
    assert_eq!(
        MailPersonsSyncPersistenceConformanceV1::account_lifecycle_evidence(
            &persistence,
            OWNER,
            id(71),
        )
        .await
        .expect("post-rejection evidence"),
        before,
        "higher-revision Retired must roll back binding, inbox and schedule outbox",
    );

    let exact_retired = ApplyMailPersonsSyncAccountLifecycleV1 {
        mapping_revision: 1,
        lifecycle: envelope(76),
        ..higher_revision_retired
    };
    persistence
        .apply_account_lifecycle_once(&exact_retired, |_| Ok(envelope(77)))
        .await
        .expect("exact stable-revision Ready to Retired");
    let retired = MailPersonsSyncPersistenceConformanceV1::account_lifecycle_evidence(
        &persistence,
        OWNER,
        id(71),
    )
    .await
    .expect("Retired evidence");
    assert_eq!((retired.inbox_count, retired.outbox_count), (2, 2));
    assert_eq!(retired.mapping_revision, Some(1));
    assert_eq!(
        retired.state,
        Some(MailPersonsSyncAccountLifecycleKindV1::Retired)
    );
    assert_eq!(retired.schedule_revision, Some(2));
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_page_rejection_persists_typed_terminal_run_state() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let persistence = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    MailPersonsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    persistence
        .begin_run_for_conformance(&begin())
        .await
        .expect("begin run");
    let rejected = CompleteMailPersonsSyncPageV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        completion: envelope(12),
        page_digest: digest(13),
        observed_sources: 0,
        updated_sources: 0,
        removed_sources: 0,
        has_more: false,
        page_receipt: envelope(14),
        rejection_code: Some(MailPersonsSyncStoredRejectCodeV1::SourceUnavailable),
        continuation: MailPersonsSyncPageContinuationV1::Finished {
            run_result: envelope(15),
            scheduler_terminal: envelope(16),
        },
        completed_at_unix_millis: 1_200,
    };
    persistence
        .complete_page_once(&rejected)
        .await
        .expect("persist page rejection");
    let run = persistence
        .load_run_context(OWNER, id(2))
        .await
        .expect("rejected run");
    assert_eq!(run.state, 4);
    assert_eq!(
        run.rejection_code,
        Some(MailPersonsSyncStoredRejectCodeV1::SourceUnavailable),
    );
    drop(persistence);
    let restarted = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("reconnect");
    let replayed = restarted
        .load_run_context(OWNER, id(2))
        .await
        .expect("durable rejected run");
    assert_eq!(replayed.state, 4);
    assert_eq!(replayed.rejection_code, run.rejection_code);
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_expired_active_run_is_terminalized_before_atomic_successor() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let persistence = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    MailPersonsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    persistence
        .begin_run_for_conformance(&begin())
        .await
        .expect("active run");
    let staged = source(10, 1);
    persistence
        .stage_source_once(&staged)
        .await
        .expect("predecessor source in flight");
    let in_flight = persistence
        .load_pending_outbox(OWNER)
        .await
        .expect("predecessor pending outbox");
    assert_eq!(in_flight.len(), 2);
    let stale_active = RecordMailPersonsSyncPersonsTerminalV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        persons_command_id: staged.persons_command_id,
        result: envelope(50),
        outcome: 1,
        result_completed_at_unix_millis: 2_001,
        received_at_unix_millis: 2_001,
    };
    assert_eq!(
        persistence
            .record_persons_terminal_once(&stale_active)
            .await,
        Err(MailPersonsSyncPersistenceErrorV1::StateConflict),
        "an active run rejects a result completed after its lease"
    );
    let mut successor = begin();
    successor.run_id = id(22);
    successor.run_fingerprint = digest(23);
    successor.scheduler_command = envelope(24);
    successor.scheduler_acceptance = envelope(25);
    successor.initial_fetch = envelope(26);
    successor.lease_epoch = 2;
    successor.lease_expires_at_unix_millis = 4_000;
    successor.received_at_unix_millis = 2_001;
    let publish_claim = persistence
        .claim_next_pending_outbox(OWNER)
        .await
        .expect("claim predecessor publication")
        .expect("pending predecessor publication");
    assert_eq!(publish_claim.record().run_id, id(2));
    let claimed_message_id = publish_claim.record().record.message_id;
    let claimed_digest = publish_claim.record().record.envelope_sha256;
    let reclaim_persistence = persistence.clone();
    let reclaim_successor = successor.clone();
    let reclaim = tokio::spawn(async move {
        reclaim_persistence
            .begin_run_reclaiming_expired_once(
                &reclaim_successor,
                |expired: MailPersonsSyncExpiredRunContextV1| {
                    assert_eq!(expired.run_id, id(2));
                    assert_eq!(expired.lease_expires_at_unix_millis, 2_000);
                    Ok(envelope(27))
                },
            )
            .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(
        !reclaim.is_finished(),
        "reclaim must wait for the publication row lock"
    );
    publish_claim
        .mark_published(claimed_digest, 2_001)
        .await
        .expect("publish commits before reclaim observes the row");
    let outcome = reclaim
        .await
        .expect("reclaim task")
        .expect("expired predecessor terminalized after publication commit");
    assert!(!outcome.replayed);
    let predecessor = persistence
        .load_run_context(OWNER, id(2))
        .await
        .expect("terminal predecessor");
    assert_eq!(predecessor.state, 4);
    assert!(
        persistence
            .terminal_page_result_is_known(
                OWNER,
                id(1),
                id(2),
                1,
                begin().initial_fetch.message_id,
            )
            .await
            .expect("terminal predecessor page is known before freshness")
    );
    assert_eq!(
        predecessor.rejection_code,
        Some(MailPersonsSyncStoredRejectCodeV1::SourceUnavailable),
    );
    assert_eq!(
        persistence
            .load_run_context(OWNER, id(22))
            .await
            .expect("active successor")
            .state,
        1,
    );
    let pending_after_reclaim = persistence
        .load_pending_outbox(OWNER)
        .await
        .expect("only successor and predecessor terminal remain publishable");
    assert_eq!(pending_after_reclaim.len(), 3);
    assert!(pending_after_reclaim.iter().any(|row| {
        row.run_id == id(2) && row.semantic_kind == MailPersonsSyncSemanticKindV1::SchedulerTerminal
    }));
    assert!(
        pending_after_reclaim
            .iter()
            .filter(|row| row.run_id == id(22))
            .count()
            == 2
    );
    for stale in in_flight
        .iter()
        .filter(|row| row.record.message_id != claimed_message_id)
    {
        assert_eq!(
            persistence
                .mark_outbox_published(
                    OWNER,
                    stale.record.message_id,
                    stale.record.envelope_sha256,
                    2_002,
                )
                .await,
            Err(MailPersonsSyncPersistenceErrorV1::StateConflict),
        );
    }
    let counts_before_late =
        MailPersonsSyncPersistenceConformanceV1::durable_counts(&persistence, OWNER)
            .await
            .expect("counts before late predecessor result");
    let late = RecordMailPersonsSyncPersonsTerminalV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        persons_command_id: staged.persons_command_id,
        result: envelope(51),
        outcome: 1,
        result_completed_at_unix_millis: 2_002,
        received_at_unix_millis: 2_003,
    };
    assert!(
        persistence
            .record_persons_terminal_once(&late)
            .await
            .expect("late predecessor result is acknowledged")
            .replayed
    );
    assert_eq!(
        MailPersonsSyncPersistenceConformanceV1::durable_counts(&persistence, OWNER)
            .await
            .expect("late result does not mutate durable state"),
        counts_before_late,
    );
    drop(persistence);
    let restarted = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("reconnect");
    assert_eq!(
        restarted
            .load_run_context(OWNER, id(2))
            .await
            .expect("durable predecessor")
            .state,
        4,
    );
    assert_eq!(
        restarted
            .load_pending_outbox(OWNER)
            .await
            .expect("restart keeps predecessor work superseded"),
        pending_after_reclaim,
    );
    assert!(
        restarted
            .record_persons_terminal_once(&late)
            .await
            .expect("late predecessor replay after restart")
            .replayed
    );
    assert!(
        restarted
            .begin_run_reclaiming_expired_once(&successor, |_| {
                panic!("exact successor replay must not reclaim again")
            })
            .await
            .expect("exact successor replay")
            .replayed
    );
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_replay_order_hash_cas_restart_and_rls_are_exact() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let persistence = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    MailPersonsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    let mail = MailAddressBookPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("Mail connect");
    MailAddressBookPersistenceConformanceV1::install_person_source_schema(&mail)
        .await
        .expect("dormant Mail schema");
    let proposed = MailPersonSourceAccountMappingV1 {
        integration_public_id: id(80),
        account_public_id: id(81),
        mapping_revision: 1,
    };
    let first_mapping = mail
        .ensure_person_source_account_mapping(OWNER, "private-account-a", proposed.clone(), 900)
        .await
        .expect("first mapping");
    assert_eq!(first_mapping, proposed);
    let replayed_mapping = mail
        .ensure_person_source_account_mapping(
            OWNER,
            "private-account-a",
            MailPersonSourceAccountMappingV1 {
                integration_public_id: id(82),
                account_public_id: id(83),
                mapping_revision: 1,
            },
            901,
        )
        .await
        .expect("stable mapping replay");
    assert_eq!(
        replayed_mapping, proposed,
        "stored random mapping wins after first observation"
    );
    let lifecycle_context = MailAddressBookEnvelopeContextV1 {
        module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
        runtime_instance_id: "mail-runtime-lifecycle".to_owned(),
        runtime_generation: 1,
        recorded_at_unix_seconds: 1,
        recorded_at_nanos: 0,
    };
    ensure_public_account_ready_v1(&mail, OWNER, "private-account-a", &lifecycle_context, 1_000)
        .await
        .expect("durable AccountReady");
    let ready = mail
        .load_pending_person_source_lifecycle_outbox(OWNER)
        .await
        .expect("ready outbox")
        .expect("pending ready");
    assert!(!ready.retired);
    mail.mark_person_source_lifecycle_outbox_published(
        OWNER,
        ready.record.message_id,
        ready.record.envelope_sha256,
        1_001,
    )
    .await
    .expect("publish ready");
    let retired_context = MailAddressBookEnvelopeContextV1 {
        recorded_at_unix_seconds: 2,
        ..lifecycle_context
    };
    record_public_account_retired_v1(&mail, OWNER, "private-account-a", &retired_context, 2_000)
        .await
        .expect("durable AccountRetired");
    let retired = mail
        .load_pending_person_source_lifecycle_outbox(OWNER)
        .await
        .expect("retired outbox")
        .expect("pending retired");
    assert!(retired.retired);
    assert_eq!(retired.account_public_id, proposed.account_public_id);
    assert_eq!(retired.mapping_revision, proposed.mapping_revision);
    record_public_account_retired_v1(
        &mail,
        OWNER,
        "private-account-a",
        &MailAddressBookEnvelopeContextV1 {
            runtime_instance_id: "mail-runtime-successor".to_owned(),
            runtime_generation: 2,
            recorded_at_unix_seconds: 3,
            ..retired_context.clone()
        },
        3_000,
    )
    .await
    .expect("delete-after-retire reuses the one durable retired event");
    assert_eq!(
        mail.load_pending_person_source_lifecycle_outbox(OWNER)
            .await
            .expect("retired exact replay"),
        Some(retired.clone()),
    );
    let reconnected_mail = MailAddressBookPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("Mail reconnect after retirement");
    assert_eq!(
        reconnected_mail
            .load_pending_person_source_lifecycle_outbox(OWNER)
            .await
            .expect("retired replay after reconnect"),
        Some(retired),
    );
    for seed in [84_u8, 85, 86] {
        mail.ensure_person_source_contact_mapping(
            OWNER,
            id(81),
            &[seed],
            id(seed),
            digest(seed),
            1,
            902,
        )
        .await
        .expect("source mapping");
    }
    assert!(
        !persistence
            .begin_run_for_conformance(&begin())
            .await
            .expect("begin")
            .replayed
    );
    assert!(
        persistence
            .begin_run_for_conformance(&begin())
            .await
            .expect("replay")
            .replayed
    );
    let mut conflict = begin();
    conflict.run_fingerprint = digest(99);
    assert_eq!(
        persistence.begin_run_for_conformance(&conflict).await,
        Err(MailPersonsSyncPersistenceErrorV1::CommandConflict)
    );

    let observed = source(10, 1);
    let removed = source(11, 3);
    assert!(
        !persistence
            .stage_source_once(&observed)
            .await
            .expect("observed")
            .replayed
    );
    assert!(
        persistence
            .stage_source_once(&observed)
            .await
            .expect("source replay")
            .replayed
    );
    assert!(
        !persistence
            .stage_source_once(&removed)
            .await
            .expect("removed")
            .replayed
    );
    let completion = CompleteMailPersonsSyncPageV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        completion: envelope(12),
        page_digest: digest(13),
        observed_sources: 1,
        updated_sources: 0,
        removed_sources: 1,
        has_more: false,
        page_receipt: envelope(14),
        rejection_code: None,
        continuation: MailPersonsSyncPageContinuationV1::AwaitingPersons,
        completed_at_unix_millis: 1_200,
    };
    assert!(
        !persistence
            .complete_page_once(&completion)
            .await
            .expect("complete")
            .replayed
    );
    assert!(
        persistence
            .complete_page_once(&completion)
            .await
            .expect("complete replay")
            .replayed
    );
    let commands = persistence
        .load_pending_outbox(OWNER)
        .await
        .expect("commands");
    assert_eq!(
        commands
            .iter()
            .map(|record| record.record.message_id)
            .collect::<Vec<_>>(),
        vec![id(5), id(6), id(50), id(51)]
    );
    for (command_id, result_seed, outcome) in [(id(50), 60_u8, 1_u8), (id(51), 61_u8, 1_u8)] {
        let terminal = RecordMailPersonsSyncPersonsTerminalV1 {
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(1),
            run_id: id(2),
            page_sequence: 1,
            persons_command_id: command_id,
            result: envelope(result_seed),
            outcome,
            result_completed_at_unix_millis: 1_200,
            received_at_unix_millis: 1_250,
        };
        assert!(
            !persistence
                .record_persons_terminal_once(&terminal)
                .await
                .expect("terminal")
                .replayed
        );
        assert!(
            persistence
                .record_persons_terminal_once(&terminal)
                .await
                .expect("terminal replay")
                .replayed
        );
    }
    let ready = persistence
        .load_page_finalization_context(OWNER, id(2), 1)
        .await
        .expect("finalization context")
        .expect("all Persons outcomes received");
    assert!(!ready.rejected);
    persistence
        .finalize_finished_page_once(OWNER, id(2), 1, envelope(15), envelope(16), 1_260)
        .await
        .expect("actual success outputs");
    assert!(
        persistence
            .finalize_finished_page_once(OWNER, id(2), 1, envelope(15), envelope(16), 1_260)
            .await
            .expect("exact terminal output replay")
            .replayed
    );
    let altered_envelope = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: id(15).to_vec(),
            run_id: id(249).to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(251).to_vec(),
            page_sequence: 1,
            page_size: 500,
        },
        2,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "altered-persistence-fixture".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("altered durable envelope");
    let altered_run_result = MailPersonsSyncEnvelopeRecordV1::new(
        *altered_envelope.message_id(),
        altered_envelope.exact_bytes().to_vec(),
    )
    .expect("altered exact ID");
    assert_eq!(
        persistence
            .finalize_finished_page_once(OWNER, id(2), 1, altered_run_result, envelope(16), 1_260,)
            .await,
        Err(MailPersonsSyncPersistenceErrorV1::CommandConflict),
    );
    let pending = persistence
        .load_pending_outbox(OWNER)
        .await
        .expect("pending");
    assert_eq!(pending.len(), 7);
    assert_eq!(
        pending
            .iter()
            .map(|record| record.record.message_id)
            .collect::<Vec<_>>(),
        vec![id(5), id(6), id(50), id(51), id(14), id(15), id(16)]
    );
    assert!(
        pending
            .windows(2)
            .all(|pair| pair[0].semantic_order_key < pair[1].semantic_order_key)
    );

    persistence
        .mark_outbox_published(
            OWNER,
            pending[0].record.message_id,
            pending[0].record.envelope_sha256,
            1_300,
        )
        .await
        .expect("CAS publish");
    let counts = MailPersonsSyncPersistenceConformanceV1::durable_counts(&persistence, OWNER)
        .await
        .expect("counts");
    assert_eq!(counts, (5, 7, 2));

    MailPersonsSyncPersistenceConformanceV1::corrupt_outbox_bytes(
        &persistence,
        OWNER,
        pending[1].record.message_id,
    )
    .await
    .expect("fault injection");
    assert_eq!(
        persistence.load_pending_outbox(OWNER).await,
        Err(MailPersonsSyncPersistenceErrorV1::HashMismatch)
    );
    assert_eq!(
        persistence
            .mark_outbox_published(
                OWNER,
                pending[1].record.message_id,
                pending[1].record.envelope_sha256,
                1_301
            )
            .await,
        Err(MailPersonsSyncPersistenceErrorV1::HashMismatch)
    );

    let rls = MailPersonsSyncPersistenceConformanceV1::rls_evidence(&persistence, OWNER, "owner-b")
        .await
        .expect("RLS");
    assert_eq!(rls.visible_owners, vec![OWNER.to_owned()]);
    assert_eq!(rls.cross_owner_updates, 0);
    assert_eq!(rls.cross_owner_deletes, 0);
    assert!(rls.cross_owner_insert_blocked);

    drop(persistence);
    let restarted = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("reconnect");
    assert_eq!(
        MailPersonsSyncPersistenceConformanceV1::durable_counts(&restarted, OWNER)
            .await
            .expect("restart counts"),
        counts
    );
    let mail_restarted = MailAddressBookPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("Mail reconnect");
    assert_eq!(
        mail_restarted
            .load_person_source_account_mapping(OWNER, "private-account-a")
            .await
            .expect("mapping after reconnect"),
        proposed
    );
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_empty_page_is_materialized_and_finalized_atomically() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let persistence = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    MailPersonsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    assert!(
        !persistence
            .begin_run_for_conformance(&begin())
            .await
            .expect("begin")
            .replayed
    );
    let completion = CompleteMailPersonsSyncPageV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        completion: envelope(12),
        page_digest: digest(13),
        observed_sources: 0,
        updated_sources: 0,
        removed_sources: 0,
        has_more: false,
        page_receipt: envelope(14),
        rejection_code: None,
        continuation: MailPersonsSyncPageContinuationV1::Finished {
            run_result: envelope(15),
            scheduler_terminal: envelope(16),
        },
        completed_at_unix_millis: 1_200,
    };
    assert!(
        !persistence
            .complete_page_once(&completion)
            .await
            .expect("complete empty page")
            .replayed
    );
    assert!(
        persistence
            .complete_page_once(&completion)
            .await
            .expect("replay empty page")
            .replayed
    );
    let pending = persistence
        .load_pending_outbox(OWNER)
        .await
        .expect("pending empty page outputs");
    assert_eq!(
        pending
            .iter()
            .map(|record| record.record.message_id)
            .collect::<Vec<_>>(),
        vec![id(5), id(6), id(14), id(15), id(16)]
    );
    assert_eq!(
        MailPersonsSyncPersistenceConformanceV1::durable_counts(&persistence, OWNER)
            .await
            .expect("empty page durable counts"),
        (1, 5, 0)
    );
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_unknown_terminal_context_and_concurrent_account_run_are_typed() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let persistence = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    MailPersonsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    assert_eq!(
        persistence
            .find_source_command_context(OWNER, id(91))
            .await
            .expect("optional unrelated Persons terminal lookup"),
        None,
    );
    persistence
        .begin_run_for_conformance(&begin())
        .await
        .expect("active run");
    let mut concurrent = begin();
    concurrent.run_id = id(31);
    concurrent.run_fingerprint = digest(32);
    concurrent.scheduler_command = envelope(33);
    concurrent.scheduler_acceptance = envelope(34);
    concurrent.initial_fetch = envelope(35);
    assert_eq!(
        persistence.begin_run_for_conformance(&concurrent).await,
        Err(MailPersonsSyncPersistenceErrorV1::AccountBusy),
    );
    let busy = RejectMailPersonsSyncAccountBusyV1 {
        begin: concurrent,
        scheduler_terminal: envelope(36),
    };
    assert!(
        !persistence
            .record_account_busy_once(&busy)
            .await
            .expect("durable account-busy terminal")
            .replayed
    );
    assert!(
        persistence
            .record_account_busy_once(&busy)
            .await
            .expect("exact busy replay")
            .replayed
    );
    let pending = persistence
        .load_pending_outbox(OWNER)
        .await
        .expect("pending");
    assert!(pending.iter().any(|row| row.record.message_id == id(36)));
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_page_completion_is_durable_before_sources_and_late_sources_roll_back() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let persistence = MailPersonsSyncPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("connect");
    MailPersonsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("schema");
    persistence
        .begin_run_for_conformance(&begin())
        .await
        .expect("begin");
    let completion = CompleteMailPersonsSyncPageV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        completion: envelope(12),
        page_digest: digest(13),
        observed_sources: 1,
        updated_sources: 0,
        removed_sources: 0,
        has_more: false,
        page_receipt: envelope(14),
        rejection_code: None,
        continuation: MailPersonsSyncPageContinuationV1::AwaitingPersons,
        completed_at_unix_millis: 1_200,
    };
    assert!(
        !persistence
            .complete_page_once(&completion)
            .await
            .expect("durably pend completion before source")
            .replayed
    );
    assert_eq!(
        persistence
            .load_pending_outbox(OWNER)
            .await
            .expect("pending before source")
            .len(),
        2,
        "pending completion must not publish receipt/result before its source",
    );
    let observed = source(10, 1);
    persistence
        .stage_source_once(&observed)
        .await
        .expect("late source completes pending dependency");
    let terminal = RecordMailPersonsSyncPersonsTerminalV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        persons_command_id: observed.persons_command_id,
        result: envelope(60),
        outcome: 1,
        result_completed_at_unix_millis: 1_200,
        received_at_unix_millis: 1_250,
    };
    persistence
        .record_persons_terminal_once(&terminal)
        .await
        .expect("final Persons outcome");
    let ready = persistence
        .load_page_finalization_context(OWNER, id(2), 1)
        .await
        .expect("load actual finalization context")
        .expect("page ready after final Persons outcome");
    assert!(!ready.rejected);
    persistence
        .finalize_finished_page_once(OWNER, id(2), 1, envelope(15), envelope(16), 1_260)
        .await
        .expect("store terminal outputs only after actual outcomes");
    let before = MailPersonsSyncPersistenceConformanceV1::durable_counts(&persistence, OWNER)
        .await
        .expect("durable baseline");
    assert_eq!(
        persistence.stage_source_once(&source(11, 1)).await,
        Err(MailPersonsSyncPersistenceErrorV1::StateConflict),
    );
    assert_eq!(
        MailPersonsSyncPersistenceConformanceV1::durable_counts(&persistence, OWNER)
            .await
            .expect("durable rollback evidence"),
        before,
    );
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_dormant_mail_producer_replays_exact_fetch_and_classifies_source_changes() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let mail = MailAddressBookPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("Mail connect");
    MailAddressBookPersistenceConformanceV1::install_schema(&mail)
        .await
        .expect("base Mail schema");
    MailAddressBookPersistenceConformanceV1::install_person_source_schema(&mail)
        .await
        .expect("dormant producer schema");
    mail.ensure_person_source_account_mapping(
        OWNER,
        "private-account-a",
        MailPersonSourceAccountMappingV1 {
            integration_public_id: id(80),
            account_public_id: id(81),
            mapping_revision: 1,
        },
        900,
    )
    .await
    .expect("account mapping");
    let issued = ensure_public_account_mapping_v1(&mail, OWNER, "private-account-random", 900)
        .await
        .expect("internally issued random account mapping");
    assert_eq!(
        ensure_public_account_mapping_v1(&mail, OWNER, "private-account-random", 901)
            .await
            .expect("stable random account replay"),
        issued,
    );
    let first = mail
        .observe_person_source_contact(&MailPersonSourceObservationV1 {
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(81),
            provider_record_key: vec![1],
            provider_record_etag: Some(vec![2]),
            proposed_source_public_id: id(82),
            claims_digest: digest(83),
            observed_at_unix_millis: 901,
        })
        .await
        .expect("new source");
    assert_eq!(first.change_kind, MailPersonSourceChangeKindV1::Observed);
    assert_eq!(first.source_revision, 1);
    let unchanged = mail
        .observe_person_source_contact(&MailPersonSourceObservationV1 {
            proposed_source_public_id: id(99),
            observed_at_unix_millis: 902,
            ..MailPersonSourceObservationV1 {
                logical_owner_id: OWNER.to_owned(),
                account_public_id: id(81),
                provider_record_key: vec![1],
                provider_record_etag: Some(vec![3]),
                proposed_source_public_id: id(82),
                claims_digest: digest(83),
                observed_at_unix_millis: 901,
            }
        })
        .await
        .expect("unchanged source");
    assert_eq!(
        unchanged.change_kind,
        MailPersonSourceChangeKindV1::Unchanged
    );
    assert_eq!(unchanged.provider_source_contact_public_id, id(82));
    assert_eq!(unchanged.source_revision, 1);
    let updated = mail
        .observe_person_source_contact(&MailPersonSourceObservationV1 {
            claims_digest: digest(84),
            observed_at_unix_millis: 903,
            ..MailPersonSourceObservationV1 {
                logical_owner_id: OWNER.to_owned(),
                account_public_id: id(81),
                provider_record_key: vec![1],
                provider_record_etag: Some(vec![4]),
                proposed_source_public_id: id(99),
                claims_digest: digest(83),
                observed_at_unix_millis: 902,
            }
        })
        .await
        .expect("updated source");
    assert_eq!(updated.change_kind, MailPersonSourceChangeKindV1::Updated);
    assert_eq!(updated.provider_source_contact_public_id, id(82));
    assert_eq!(updated.source_revision, 2);
    assert_eq!(
        mail.ensure_person_source_contact_mapping(OWNER, id(81), &[9], id(89), digest(89), 1, 904,)
            .await
            .expect("preexisting source absent from the next full snapshot"),
        id(89),
    );

    let run_id = id(85);
    let command_id = id(86);
    let command = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: command_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(81).to_vec(),
            page_sequence: 1,
            page_size: 500,
        },
        2,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "dormant-producer-test".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("fetch command");
    let source = MailPersonSourceIdentityV1 {
        integration_public_id: id(80).to_vec(),
        account_public_id: id(81).to_vec(),
        provider_source_contact_public_id: id(82).to_vec(),
    };
    let claims = MailPersonSourceClaimsV1 {
        display_name: Some("Public Person".to_owned()),
        normalized_emails: vec!["public@example.test".to_owned()],
        normalized_phones: Vec::new(),
    };
    let source_digest = mail_person_source_claims_digest_v1(&source, &claims).expect("digest");
    let output = build_mail_person_source_updated_v1(
        command_id,
        MailPersonSourceUpdatedV1 {
            observation_id: id(87).to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            page_sequence: 1,
            source: Some(source.clone()),
            claims: Some(claims.clone()),
            provenance: Some(MailPersonSourceProvenanceV1 {
                source_revision: 3,
                source_digest: source_digest.to_vec(),
                observed_at: Some(Timestamp {
                    seconds: 1,
                    nanos: 0,
                }),
            }),
        },
        &MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "dormant-producer-test".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("sanitized output");
    let page_1_completed = build_mail_person_source_page_completed_v1(
        command_id,
        MailPersonSourcePageCompletedV1 {
            command_id: command_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(81).to_vec(),
            page_sequence: 1,
            observed_sources: 0,
            updated_sources: 1,
            removed_sources: 0,
            has_more: true,
            page_digest: digest(88).to_vec(),
            completed_at: Some(Timestamp {
                seconds: 1,
                nanos: 0,
            }),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: "dormant-producer-test".to_owned(),
            runtime_generation: 1,
            completed_at_unix_seconds: 1,
            completed_at_nanos: 0,
            execution_attempt: 1,
        },
    )
    .expect("page 1 completion");
    let unrelated_output = build_mail_person_source_updated_v1(
        command_id,
        MailPersonSourceUpdatedV1 {
            observation_id: id(98).to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: "owner-b".to_owned(),
            page_sequence: 1,
            source: Some(source),
            claims: Some(claims),
            provenance: Some(MailPersonSourceProvenanceV1 {
                source_revision: 3,
                source_digest: source_digest.to_vec(),
                observed_at: Some(Timestamp {
                    seconds: 1,
                    nanos: 0,
                }),
            }),
        },
        &MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "dormant-producer-test".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("valid but unrelated sanitized output");
    let observations = vec![MailPersonSourceObservationV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(81),
        provider_record_key: vec![1],
        provider_record_etag: Some(vec![5]),
        proposed_source_public_id: id(99),
        claims_digest: source_digest,
        observed_at_unix_millis: 1_000,
    }];
    let commit = MailPersonSourceAtomicFetchCommitV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(81),
        run_id,
        page_sequence: 1,
        expected_provider_cursor: None,
        next_provider_cursor: Some(vec![7]),
        public_has_more: true,
        has_more: true,
        command: MailPersonSourceEnvelopeRecordV1::from_outbox(&command),
        processed_at_unix_millis: 1_000,
    };
    assert_eq!(
        mail.commit_person_source_fetch_atomically_once(
            &commit,
            || Ok(observations.clone()),
            |_| {
                Ok(vec![
                    MailPersonSourceFetchOutputV1 {
                        semantic_order_key: mail_person_source_semantic_order_key_v1(1, 1)?,
                        record: MailPersonSourceEnvelopeRecordV1::from_outbox(&unrelated_output),
                    },
                    MailPersonSourceFetchOutputV1 {
                        semantic_order_key: mail_person_source_semantic_order_key_v1(1, 2)?,
                        record: MailPersonSourceEnvelopeRecordV1::from_outbox(&page_1_completed),
                    },
                ])
            },
        )
        .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::InvalidInput),
    );
    assert!(
        mail.load_pending_person_source_fetch_outbox(OWNER)
            .await
            .expect("unrelated output rolls back")
            .is_empty()
    );
    let committed = mail
        .commit_person_source_fetch_atomically_once(
            &commit,
            || Ok(observations.clone()),
            |changes| {
                assert_eq!(changes.len(), 1);
                assert_eq!(
                    changes[0].change_kind,
                    MailPersonSourceChangeKindV1::Updated
                );
                assert_eq!(changes[0].source_revision, 3);
                Ok(vec![
                    MailPersonSourceFetchOutputV1 {
                        semantic_order_key: mail_person_source_semantic_order_key_v1(1, 1)?,
                        record: MailPersonSourceEnvelopeRecordV1::from_outbox(&output),
                    },
                    MailPersonSourceFetchOutputV1 {
                        semantic_order_key: mail_person_source_semantic_order_key_v1(1, 2)?,
                        record: MailPersonSourceEnvelopeRecordV1::from_outbox(&page_1_completed),
                    },
                ])
            },
        )
        .await
        .expect("atomic classify, map, seen and outbox");
    assert!(!committed.replayed);
    let replay = mail
        .commit_person_source_fetch_atomically_once(
            &commit,
            || panic!("exact replay must not prepare entropy or classify"),
            |_| panic!("exact replay must not rebuild outputs"),
        )
        .await
        .expect("exact replay first");
    assert!(replay.replayed);
    assert_eq!(
        replay.processed_at_unix_millis,
        commit.processed_at_unix_millis
    );
    assert_eq!(
        replay.outputs[0].record.envelope_bytes,
        output.exact_bytes()
    );
    drop(mail);
    let mail = MailAddressBookPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("fresh process reconnect before exact replay");
    let mut fresh_process_replay = commit.clone();
    fresh_process_replay.processed_at_unix_millis = 9_999;
    let fresh_replay = mail
        .commit_person_source_fetch_atomically_once(
            &fresh_process_replay,
            || panic!("fresh-process replay must precede entropy and provider classification"),
            |_| panic!("fresh-process replay must use stored output bytes"),
        )
        .await
        .expect("fresh-process exact replay");
    assert!(fresh_replay.replayed);
    assert_eq!(
        fresh_replay.processed_at_unix_millis,
        commit.processed_at_unix_millis
    );
    let mut altered_request = commit.clone();
    altered_request.next_provider_cursor = None;
    altered_request.has_more = false;
    assert_eq!(
        mail.commit_person_source_fetch_atomically_once(
            &altered_request,
            || panic!("conflicting replay must not prepare observations"),
            |_| panic!("conflicting replay must not build outputs"),
        )
        .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict),
    );
    let pending = mail
        .load_pending_person_source_fetch_outbox(OWNER)
        .await
        .expect("pending");
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].record.envelope_bytes, output.exact_bytes());
    for record in &pending {
        mail.mark_person_source_fetch_outbox_published(
            OWNER,
            record.record.message_id,
            record.record.envelope_sha256,
            1_001,
        )
        .await
        .expect("hash CAS publish");
    }
    let command_page_2 = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: id(88).to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(81).to_vec(),
            page_sequence: 2,
            page_size: 500,
        },
        2,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "dormant-producer-test".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("second fetch command");
    let final_page = MailPersonSourceAtomicFetchCommitV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(81),
        run_id,
        page_sequence: 2,
        expected_provider_cursor: Some(vec![7]),
        next_provider_cursor: None,
        public_has_more: true,
        has_more: false,
        command: MailPersonSourceEnvelopeRecordV1::from_outbox(&command_page_2),
        processed_at_unix_millis: 1_001,
    };
    let page_2_completed = build_mail_person_source_page_completed_v1(
        id(88),
        MailPersonSourcePageCompletedV1 {
            command_id: id(88).to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(81).to_vec(),
            page_sequence: 2,
            observed_sources: 0,
            updated_sources: 0,
            removed_sources: 0,
            has_more: true,
            page_digest: digest(89).to_vec(),
            completed_at: Some(Timestamp {
                seconds: 1,
                nanos: 1_000_000,
            }),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: "dormant-producer-test".to_owned(),
            runtime_generation: 1,
            completed_at_unix_seconds: 1,
            completed_at_nanos: 1_000_000,
            execution_attempt: 1,
        },
    )
    .expect("page 2 completion");
    let mut out_of_order = final_page.clone();
    out_of_order.page_sequence = 3;
    assert_eq!(
        mail.commit_person_source_fetch_atomically_once(
            &out_of_order,
            || panic!("out-of-order page must not prepare observations"),
            |_| panic!("out-of-order page must not build outputs"),
        )
        .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::InvalidInput),
    );
    let mut wrong_cursor = final_page.clone();
    wrong_cursor.expected_provider_cursor = Some(vec![8]);
    assert_eq!(
        mail.commit_person_source_fetch_atomically_once(
            &wrong_cursor,
            || panic!("wrong cursor must not prepare observations"),
            |_| panic!("wrong cursor must not build outputs"),
        )
        .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict),
    );
    assert!(
        !mail
            .commit_person_source_fetch_atomically_once(
                &final_page,
                || Ok(Vec::new()),
                |changes| {
                    assert!(changes.is_empty());
                    Ok(vec![MailPersonSourceFetchOutputV1 {
                        semantic_order_key: mail_person_source_semantic_order_key_v1(2, 1)?,
                        record: MailPersonSourceEnvelopeRecordV1::from_outbox(&page_2_completed),
                    }])
                },
            )
            .await
            .expect("final page advances run to awaiting terminal snapshot")
            .replayed
    );
    let durable_removals = mail
        .preview_person_source_removals(OWNER, id(81), &[id(82)])
        .await
        .expect("locked snapshot preview");
    assert_eq!(durable_removals.len(), 1);
    assert_eq!(
        durable_removals[0].provider_source_contact_public_id,
        id(89)
    );
    assert_eq!(durable_removals[0].source_revision, 2);
    let removal_page = build_synthetic_removal_page_v1(
        OWNER,
        run_id,
        3,
        &durable_removals
            .iter()
            .map(|value| MailPersonSourceSyntheticRemovalV1 {
                integration_public_id: value.integration_public_id,
                account_public_id: value.account_public_id,
                provider_source_contact_public_id: value.provider_source_contact_public_id,
                source_revision: value.source_revision,
            })
            .collect::<Vec<_>>(),
        false,
        &MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "dormant-producer-test".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 0,
        },
    )
    .expect("synthetic removal after terminal full snapshot");
    assert_eq!(removal_page.source_records.len(), 1);
    let removal_commit = MailPersonSourceRemovalPageCommitV1 {
        page_sequence: 3,
        source_ids: vec![id(89)],
        outputs: removal_page
            .all_records()
            .into_iter()
            .enumerate()
            .map(|(index, record)| MailPersonSourceFetchOutputV1 {
                semantic_order_key: mail_person_source_semantic_order_key_v1(
                    3,
                    u16::try_from(index + 1).expect("bounded ordinal"),
                )
                .expect("removal semantic key"),
                record: MailPersonSourceEnvelopeRecordV1::from_outbox(record),
            })
            .collect(),
    };
    let snapshot = MailPersonSourceSnapshotCommitV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(81),
        run_id,
        seen_public_source_ids: vec![id(82)],
        expected_removals: durable_removals.clone(),
        removal_pages: vec![removal_commit],
        terminal_command: MailPersonSourceEnvelopeRecordV1::from_outbox(&page_2_completed),
        completed_at_unix_millis: 2_000,
    };
    let mut omitted_seen = snapshot.clone();
    omitted_seen.seen_public_source_ids.clear();
    assert_eq!(
        mail.commit_person_source_snapshot_once(&omitted_seen).await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict),
    );
    let mut extra_seen = snapshot.clone();
    extra_seen.seen_public_source_ids.push(id(99));
    assert_eq!(
        mail.commit_person_source_snapshot_once(&extra_seen).await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict),
    );
    let mut stale_snapshot = snapshot.clone();
    stale_snapshot.expected_removals[0].source_revision += 1;
    assert_eq!(
        mail.commit_person_source_snapshot_once(&stale_snapshot)
            .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::InvalidInput),
    );
    assert_eq!(
        mail.commit_person_source_snapshot_once(&snapshot)
            .await
            .expect("atomic terminal snapshot"),
        durable_removals,
    );
    assert_eq!(
        mail.commit_person_source_snapshot_once(&snapshot)
            .await
            .expect("exact terminal replay after completion"),
        durable_removals,
    );
    let terminal_fetch_replay = mail
        .commit_person_source_fetch_atomically_once(
            &final_page,
            || panic!("terminal fetch replay must not reclassify provider entries"),
            |_| panic!("terminal fetch replay must reuse the durable outbox"),
        )
        .await
        .expect("terminal fetch replay");
    assert!(terminal_fetch_replay.replayed);
    assert!(
        terminal_fetch_replay.terminal_snapshot_succeeded,
        "the fetch worker must be able to ACK an exact replay without rebuilding an already committed terminal snapshot",
    );
    let lookup_replay = mail
        .load_person_source_fetch_replay(
            OWNER,
            id(81),
            run_id,
            2,
            &MailPersonSourceEnvelopeRecordV1::from_outbox(&command_page_2),
        )
        .await
        .expect("pre-provider exact replay lookup")
        .expect("stored replay");
    assert!(lookup_replay.replayed);
    assert!(lookup_replay.terminal_snapshot_succeeded);
    assert_eq!(lookup_replay.outputs, terminal_fetch_replay.outputs);
    let synthetic_fetch_id = mail_person_source_fetch_id_v1(run_id, 3);
    let synthetic_fetch = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: synthetic_fetch_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(81).to_vec(),
            page_sequence: 3,
            page_size: 500,
        },
        3,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "workflow-synthetic-continuation".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 0,
        },
    )
    .expect("canonical workflow fetch for the already-durable synthetic page");
    let synthetic_fetch = MailPersonSourceEnvelopeRecordV1::from_outbox(&synthetic_fetch);
    assert!(
        mail.accept_person_source_synthetic_fetch_continuation_once(
            OWNER,
            id(81),
            run_id,
            3,
            &synthetic_fetch,
            2_000,
        )
        .await
        .expect("the terminal Mail run ACKs the exact synthetic continuation")
    );
    assert!(
        mail.accept_person_source_synthetic_fetch_continuation_once(
            OWNER,
            id(81),
            run_id,
            3,
            &synthetic_fetch,
            9_000,
        )
        .await
        .expect("exact continuation replay is clock-independent")
    );
    let mut changed_synthetic_fetch = synthetic_fetch.clone();
    changed_synthetic_fetch.envelope_bytes = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: synthetic_fetch_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(81).to_vec(),
            page_sequence: 3,
            page_size: 499,
        },
        3,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "workflow-synthetic-continuation".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 0,
        },
    )
    .expect("canonical conflicting continuation")
    .exact_bytes()
    .to_vec();
    changed_synthetic_fetch.envelope_sha256 =
        sha2::Sha256::digest(&changed_synthetic_fetch.envelope_bytes).into();
    assert_eq!(
        mail.accept_person_source_synthetic_fetch_continuation_once(
            OWNER,
            id(81),
            run_id,
            3,
            &changed_synthetic_fetch,
            9_000,
        )
        .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict),
    );
    let changed_command = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: id(88).to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(81).to_vec(),
            page_sequence: 2,
            page_size: 500,
        },
        3,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "dormant-producer-test".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("canonical changed fetch command");
    assert_eq!(
        mail.load_person_source_fetch_replay(
            OWNER,
            id(81),
            run_id,
            2,
            &MailPersonSourceEnvelopeRecordV1::from_outbox(&changed_command),
        )
        .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict),
    );
    let mut conflicting_terminal = snapshot.clone();
    conflicting_terminal.terminal_command = MailPersonSourceEnvelopeRecordV1::from_outbox(
        &build_mail_person_source_page_completed_v1(
            id(88),
            MailPersonSourcePageCompletedV1 {
                command_id: id(88).to_vec(),
                run_id: run_id.to_vec(),
                logical_owner_id: OWNER.to_owned(),
                account_public_id: id(81).to_vec(),
                page_sequence: 2,
                observed_sources: 0,
                updated_sources: 0,
                removed_sources: 0,
                has_more: true,
                page_digest: digest(91).to_vec(),
                completed_at: Some(Timestamp {
                    seconds: 1,
                    nanos: 1_000_000,
                }),
            },
            &MailAddressBookResultEnvelopeContextV1 {
                runtime_instance_id: "dormant-producer-test".to_owned(),
                runtime_generation: 1,
                completed_at_unix_seconds: 1,
                completed_at_nanos: 1_000_000,
                execution_attempt: 1,
            },
        )
        .expect("conflicting terminal result"),
    );
    assert_eq!(
        mail.commit_person_source_snapshot_once(&conflicting_terminal)
            .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict),
    );
    let mut late_page = final_page.clone();
    late_page.command = MailPersonSourceEnvelopeRecordV1::from_outbox(
        &build_fetch_mail_person_source_page_command_v1(
            FetchMailPersonSourcePageCommandV1 {
                command_id: id(90).to_vec(),
                run_id: run_id.to_vec(),
                logical_owner_id: OWNER.to_owned(),
                account_public_id: id(81).to_vec(),
                page_sequence: 2,
                page_size: 500,
            },
            3,
            &MailAddressBookEnvelopeContextV1 {
                module_id: "makosh-mail-persons-sync-runtime".to_owned(),
                runtime_instance_id: "dormant-producer-test".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 2,
                recorded_at_nanos: 0,
            },
        )
        .expect("late page command"),
    );
    late_page.processed_at_unix_millis = 2_001;
    assert_eq!(
        mail.commit_person_source_fetch_atomically_once(
            &late_page,
            || panic!("late page after terminal snapshot must not prepare observations"),
            |_| panic!("late page after terminal snapshot must not build outputs"),
        )
        .await,
        Err(makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict),
    );
    drop(mail);
    let mail = MailAddressBookPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("Mail reconnect after terminal snapshot");
    assert_eq!(
        mail.load_completed_person_source_removals(OWNER, id(81), run_id)
            .await
            .expect("restartable terminal removal plan"),
        durable_removals,
    );
    assert_eq!(
        mail.load_pending_person_source_fetch_outbox(OWNER)
            .await
            .expect("synthetic removal outbox after restart")
            .len(),
        3,
    );
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn postgres_fetch_and_terminal_snapshot_serialize_on_the_exact_run() {
    let url = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_POSTGRES_URL").expect("managed URL");
    let mail = MailAddressBookPersistenceConformanceV1::connect_url(&url)
        .await
        .expect("Mail connect");
    MailAddressBookPersistenceConformanceV1::install_schema(&mail)
        .await
        .expect("base Mail schema");
    MailAddressBookPersistenceConformanceV1::install_person_source_schema(&mail)
        .await
        .expect("dormant Mail schema");
    mail.ensure_person_source_account_mapping(
        OWNER,
        "private-concurrent-account",
        MailPersonSourceAccountMappingV1 {
            integration_public_id: id(91),
            account_public_id: id(92),
            mapping_revision: 1,
        },
        900,
    )
    .await
    .expect("account mapping");
    let run_id = id(93);
    let command = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: id(94).to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(92).to_vec(),
            page_sequence: 1,
            page_size: 500,
        },
        2,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "concurrent-snapshot-test".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("fetch command");
    let completed = build_mail_person_source_page_completed_v1(
        id(94),
        MailPersonSourcePageCompletedV1 {
            command_id: id(94).to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: OWNER.to_owned(),
            account_public_id: id(92).to_vec(),
            page_sequence: 1,
            observed_sources: 0,
            updated_sources: 0,
            removed_sources: 0,
            has_more: false,
            page_digest: digest(95).to_vec(),
            completed_at: Some(Timestamp {
                seconds: 1,
                nanos: 1_000_000,
            }),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: "concurrent-snapshot-test".to_owned(),
            runtime_generation: 1,
            completed_at_unix_seconds: 1,
            completed_at_nanos: 1_000_000,
            execution_attempt: 1,
        },
    )
    .expect("page completion");
    let fetch = MailPersonSourceAtomicFetchCommitV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(92),
        run_id,
        page_sequence: 1,
        expected_provider_cursor: None,
        next_provider_cursor: None,
        public_has_more: false,
        has_more: false,
        command: MailPersonSourceEnvelopeRecordV1::from_outbox(&command),
        processed_at_unix_millis: 1_001,
    };
    let snapshot = MailPersonSourceSnapshotCommitV1 {
        logical_owner_id: OWNER.to_owned(),
        account_public_id: id(92),
        run_id,
        seen_public_source_ids: Vec::new(),
        expected_removals: Vec::new(),
        removal_pages: Vec::new(),
        terminal_command: MailPersonSourceEnvelopeRecordV1::from_outbox(&completed),
        completed_at_unix_millis: 1_001,
    };
    let fetch_mail = mail.clone();
    let snapshot_mail = mail.clone();
    let (fetch_result, snapshot_result) = tokio::join!(
        fetch_mail.commit_person_source_fetch_atomically_once(
            &fetch,
            || Ok(Vec::new()),
            |_| {
                Ok(vec![MailPersonSourceFetchOutputV1 {
                    semantic_order_key: mail_person_source_semantic_order_key_v1(1, 1)?,
                    record: MailPersonSourceEnvelopeRecordV1::from_outbox(&completed),
                }])
            },
        ),
        snapshot_mail.commit_person_source_snapshot_once(&snapshot),
    );
    assert!(!fetch_result.expect("fetch commit").replayed);
    match snapshot_result {
        Ok(removals) => assert!(removals.is_empty()),
        Err(
            makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::NotFound
            | makosh_mail_address_book_persistence::MailAddressBookPersistenceErrorV1::Conflict,
        ) => assert!(
            mail.commit_person_source_snapshot_once(&snapshot)
                .await
                .expect("serialized snapshot retry")
                .is_empty()
        ),
        Err(error) => panic!("unexpected snapshot race result: {error:?}"),
    }
    assert!(
        mail.commit_person_source_fetch_atomically_once(
            &fetch,
            || panic!("exact fetch replay after terminal must not prepare observations"),
            |_| panic!("exact fetch replay after terminal must not rebuild outputs"),
        )
        .await
        .expect("exact fetch replay after terminal")
        .replayed
    );
    assert!(
        mail.load_completed_person_source_removals(OWNER, id(92), run_id)
            .await
            .expect("terminal snapshot state after race")
            .is_empty()
    );
}
