use makosh_mail_address_book_contract::{
    MAIL_RUNTIME_MODULE_ID_V1, MailAddressBookEnvelopeContextV1,
    MailAddressBookResultEnvelopeContextV1, build_mail_address_book_entry_observed_v1,
    build_mail_address_book_page_completed_result_v1,
    wire::{
        MailAddressBookEntryObservedV1, MailAddressBookPageCompletedV1,
        MailAddressBookProviderKindV1,
    },
};
use makosh_mail_address_book_persistence::{
    MailAddressBookCommandInboxOutcomeV1, MailAddressBookDispatchOutcomeV1,
    MailAddressBookFetchAdmissionV1, MailAddressBookFetchInboxOutcomeV1,
    MailAddressBookFetchStoreOutcomeV1, MailAddressBookPersistenceConformanceV1,
    MailAddressBookPersistenceErrorV1, MailAddressBookSnapshotCustodyOutcomeV1,
    MailAddressBookTargetSnapshotReceiptV1, MailAddressBookUpsertAdmissionV1,
};
use makosh_mail_contacts_sync_core::{
    MailContactsSyncDirectionV1, MailContactsSyncDraftV1, MailContactsSyncStateV1,
    MailContactsSyncTriggerV1,
};
use makosh_mail_contacts_sync_persistence::{
    AcceptContactChangedForMailSyncOutcomeV1, AcceptContactChangedForMailSyncV1,
    AcceptScheduledMailContactsSyncDueOutcomeV1, AcceptScheduledMailContactsSyncDueV1,
    AdvanceMailContactsSyncPageV1, CompleteContactMailSyncSourceOutcomeV1,
    CompleteContactMailSyncSourceV1, CompleteContactsProviderLinkOutcomeV1,
    CompleteContactsProviderLinkV1, CompleteMailAddressBookUpsertOutcomeV1,
    CompleteMailAddressBookUpsertV1, CreateMailContactsSyncOutcomeV1, CreateMailContactsSyncRunV1,
    MailContactsSyncAdvanceOutcomeV1, MailContactsSyncContactOutcomeV1,
    MailContactsSyncEntryInputV1, MailContactsSyncEntryOutcomeInputV1,
    MailContactsSyncPageResultInputV1, MailContactsSyncPersistenceConformanceV1,
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceOutcomeV1,
    MailContactsSyncProviderWriteOutcomeV1, MailContactsSyncReverseOperationSeedV1,
    MailContactsSyncScheduledTerminalOutcomeV1, OutboxEnvelopeV1,
    QueueMailContactsSyncScheduledTerminalV1,
};
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires the disposable authenticated PostgreSQL contour"]
async fn address_book_provider_page_is_atomic_replayable_and_conflict_checked() {
    let database_url = std::env::var("MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL")
        .expect("MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL");
    let persistence = MailAddressBookPersistenceConformanceV1::connect_url(&database_url)
        .await
        .expect("connect Mail address-book persistence");
    MailAddressBookPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install Mail address-book schema");
    let admission = MailAddressBookFetchAdmissionV1 {
        command_message_id: [1; 16],
        command_envelope_sha256: [2; 32],
        command_id: [1; 16],
        run_id: [3; 16],
        logical_owner_id: "owner-1".to_owned(),
        account_id: "mail-account-1".to_owned(),
        page_sequence: 2,
        continuation_cursor: Some(b"google-page-v1\0page-2".to_vec()),
        page_size: 50,
    };
    assert_eq!(
        persistence
            .accept_fetch_command(&admission, 1_800_000_000)
            .await
            .expect("accept fetch command"),
        MailAddressBookFetchInboxOutcomeV1::Accepted
    );
    assert_eq!(
        persistence
            .accept_fetch_command(&admission, 1_800_000_001)
            .await
            .expect("replay fetch command"),
        MailAddressBookFetchInboxOutcomeV1::DuplicateAccepted
    );
    assert_eq!(
        persistence.pending_fetches(1).await.expect("pending fetch")[0].admission,
        admission
    );
    let event_context = MailAddressBookEnvelopeContextV1 {
        module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
        runtime_instance_id: "mail-runtime-1".to_owned(),
        runtime_generation: 7,
        recorded_at_unix_seconds: 1_800_000_002,
        recorded_at_nanos: 0,
    };
    let observation = build_mail_address_book_entry_observed_v1(
        admission.command_message_id,
        MailAddressBookEntryObservedV1 {
            observation_id: vec![4; 16],
            run_id: admission.run_id.to_vec(),
            logical_owner_id: admission.logical_owner_id.clone(),
            account_id: admission.account_id.clone(),
            provider_kind: MailAddressBookProviderKindV1::MailAddressBookProviderKindGooglePeople
                as i32,
            provider_entry_id: "people/1".to_owned(),
            provider_etag: Some("etag-1".to_owned()),
            display_name: "Ada".to_owned(),
            email_addresses: vec!["ada@example.test".to_owned()],
            phone_numbers: Vec::new(),
            observed_at: Some(prost_types::Timestamp {
                seconds: 1_800_000_002,
                nanos: 0,
            }),
            source_revision: 5,
            entry_digest: vec![6; 32],
            page_sequence: admission.page_sequence,
        },
        &event_context,
    )
    .expect("observation");
    let completed = build_mail_address_book_page_completed_result_v1(
        admission.command_message_id,
        MailAddressBookPageCompletedV1 {
            command_id: admission.command_id.to_vec(),
            run_id: admission.run_id.to_vec(),
            page_sequence: admission.page_sequence,
            observed_entries: 1,
            next_continuation_cursor: None,
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: "mail-runtime-1".to_owned(),
            runtime_generation: 7,
            completed_at_unix_seconds: 1_800_000_003,
            completed_at_nanos: 0,
            execution_attempt: 1,
        },
    )
    .expect("completed result");
    let records = vec![observation, completed];
    assert_eq!(
        persistence
            .complete_fetch_command(admission.command_id, &records, 1_800_000_003)
            .await
            .expect("complete fetch"),
        MailAddressBookFetchStoreOutcomeV1::Stored
    );
    assert_eq!(
        persistence
            .complete_fetch_command(admission.command_id, &records, 1_800_000_004)
            .await
            .expect("replay complete fetch"),
        MailAddressBookFetchStoreOutcomeV1::AlreadyStored
    );
    let pending = persistence
        .pending_fetch_events(10)
        .await
        .expect("pending page events");
    assert_eq!(pending, records);
    for record in &pending {
        assert!(
            persistence
                .mark_fetch_event_published(*record.message_id(), 1_800_000_005)
                .await
                .expect("mark page event published")
        );
    }
    assert!(
        persistence
            .pending_fetch_events(10)
            .await
            .expect("empty page outbox")
            .is_empty()
    );
}

#[tokio::test]
#[ignore = "requires the disposable authenticated PostgreSQL contour"]
async fn postgres_is_atomic_replayable_and_sse_replayable() {
    let database_url = std::env::var("MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL")
        .expect("MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL");
    let persistence = MailContactsSyncPersistenceConformanceV1::connect_url(&database_url)
        .await
        .expect("connect workflow persistence");
    MailContactsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install workflow schema");

    let create = create_run(1);
    let created = persistence
        .create_run(create.clone())
        .await
        .expect("create run");
    assert!(matches!(
        created,
        CreateMailContactsSyncOutcomeV1::Created(_)
    ));
    let replayed = persistence
        .create_run(create.clone())
        .await
        .expect("replay start");
    assert!(matches!(
        replayed,
        CreateMailContactsSyncOutcomeV1::Existing(_)
    ));

    let pending = persistence
        .unpublished_commands("owner-1", 10)
        .await
        .expect("load initial outbox");
    assert_eq!(pending, create.initial_commands);
    persistence
        .mark_command_published(
            "owner-1",
            &pending[0].message_id,
            &pending[0].envelope_sha256,
            1_800_000_000_100,
        )
        .await
        .expect("mark initial command published");
    assert!(
        persistence
            .unpublished_commands("owner-1", 10)
            .await
            .expect("load empty outbox")
            .is_empty()
    );

    let entry = MailContactsSyncEntryInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        run_id: [1; 16],
        page_sequence: 1,
        observation_message_id: [21; 16],
        observation_envelope_sha256: [22; 32],
        contact_command_id: [23; 16],
        entry_digest: [24; 32],
        contact_command: envelope(23, b"contacts-upsert-command"),
        occurred_at_unix_millis: 1_800_000_000_300,
    };
    assert_eq!(
        persistence
            .accept_provider_entry(&entry)
            .await
            .expect("accept provider entry"),
        MailContactsSyncPersistenceOutcomeV1::Applied
    );
    assert_eq!(
        persistence
            .accept_provider_entry(&entry)
            .await
            .expect("replay provider entry"),
        MailContactsSyncPersistenceOutcomeV1::Duplicate
    );
    let mut conflicting_entry = entry.clone();
    conflicting_entry.observation_envelope_sha256 = [29; 32];
    assert_eq!(
        persistence.accept_provider_entry(&conflicting_entry).await,
        Err(MailContactsSyncPersistenceErrorV1::InboxConflict)
    );
    let outcome = MailContactsSyncEntryOutcomeInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        contact_command_id: [23; 16],
        message_id: [25; 16],
        envelope_sha256: [26; 32],
        outcome: MailContactsSyncContactOutcomeV1::Created,
        occurred_at_unix_millis: 1_800_000_000_400,
    };
    persistence
        .accept_contact_outcome(&outcome)
        .await
        .expect("accept early Contacts result");
    let before_page = persistence
        .load_run("owner-1", &[1; 16])
        .await
        .expect("load before page completion");
    assert_eq!(before_page.status.counters.contacts_created, 0);

    let page = MailContactsSyncPageResultInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        run_id: [1; 16],
        page_sequence: 1,
        message_id: [27; 16],
        envelope_sha256: [28; 32],
        observed_entries: 1,
        next_continuation_cursor: None,
        occurred_at_unix_millis: 1_800_000_000_500,
    };
    persistence
        .accept_provider_page(&page)
        .await
        .expect("complete provider page");
    let after_page = persistence
        .load_run("owner-1", &[1; 16])
        .await
        .expect("load after page completion");
    assert_eq!(
        after_page.status.state,
        MailContactsSyncStateV1::ApplyingContacts
    );
    assert_eq!(after_page.status.counters.provider_entries_seen, 1);
    assert_eq!(after_page.status.counters.contacts_created, 1);
    let progress = persistence
        .page_progress("owner-1", &[1; 16])
        .await
        .expect("load page progress");
    assert_eq!(progress.expected_entries, 1);
    assert_eq!(progress.recorded_entries, 1);
    assert_eq!(progress.accounted_entries, 1);
    assert_eq!(
        persistence
            .advance_ready_page(&AdvanceMailContactsSyncPageV1 {
                logical_owner_id: "owner-1".to_owned(),
                run_id: [1; 16],
                next_page_command: None,
                occurred_at_unix_millis: 1_800_000_000_600,
            })
            .await
            .expect("complete ready run"),
        MailContactsSyncAdvanceOutcomeV1::Applied
    );
    assert_eq!(
        persistence
            .load_run("owner-1", &[1; 16])
            .await
            .expect("load completed run")
            .status
            .state,
        MailContactsSyncStateV1::Completed
    );

    let second = create_run(30);
    persistence
        .create_run(second)
        .await
        .expect("create concurrent-order run");
    persistence
        .accept_provider_entry(&MailContactsSyncEntryInputV1 {
            logical_owner_id: "owner-1".to_owned(),
            run_id: [30; 16],
            page_sequence: 1,
            observation_message_id: [44; 16],
            observation_envelope_sha256: [45; 32],
            contact_command_id: [46; 16],
            entry_digest: [47; 32],
            contact_command: envelope(46, b"second-contacts-upsert"),
            occurred_at_unix_millis: 1_800_000_000_700,
        })
        .await
        .expect("accept concurrent-order entry");
    let concurrent_outcome = MailContactsSyncEntryOutcomeInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        contact_command_id: [46; 16],
        message_id: [48; 16],
        envelope_sha256: [49; 32],
        outcome: MailContactsSyncContactOutcomeV1::Updated,
        occurred_at_unix_millis: 1_800_000_000_800,
    };
    let concurrent_page = MailContactsSyncPageResultInputV1 {
        logical_owner_id: "owner-1".to_owned(),
        run_id: [30; 16],
        page_sequence: 1,
        message_id: [50; 16],
        envelope_sha256: [51; 32],
        observed_entries: 1,
        next_continuation_cursor: None,
        occurred_at_unix_millis: 1_800_000_000_800,
    };
    let (outcome_result, page_result) = tokio::join!(
        persistence.accept_contact_outcome(&concurrent_outcome),
        persistence.accept_provider_page(&concurrent_page),
    );
    outcome_result.expect("concurrent Contacts result");
    page_result.expect("concurrent page completion");
    let concurrent_run = persistence
        .load_run("owner-1", &[30; 16])
        .await
        .expect("load concurrent-order run");
    assert_eq!(concurrent_run.status.counters.contacts_updated, 1);
    assert_eq!(
        persistence
            .advance_ready_page(&AdvanceMailContactsSyncPageV1 {
                logical_owner_id: "owner-1".to_owned(),
                run_id: [30; 16],
                next_page_command: None,
                occurred_at_unix_millis: 1_800_000_000_900,
            })
            .await
            .expect("complete concurrent-order run"),
        MailContactsSyncAdvanceOutcomeV1::Applied
    );

    let realtime = persistence
        .client_realtime_window("owner-1", None, 10)
        .await
        .expect("initial SSE replay");
    assert_eq!(realtime.len(), 6);
    assert_eq!(
        realtime.iter().map(|item| item.state).collect::<Vec<_>>(),
        [
            MailContactsSyncStateV1::FetchingProviderPage,
            MailContactsSyncStateV1::ApplyingContacts,
            MailContactsSyncStateV1::Completed,
            MailContactsSyncStateV1::FetchingProviderPage,
            MailContactsSyncStateV1::ApplyingContacts,
            MailContactsSyncStateV1::Completed,
        ]
    );
    assert_eq!(
        persistence
            .client_realtime_window("owner-1", Some(realtime[0].sequence), 10)
            .await
            .expect("resume SSE replay"),
        realtime[1..].to_vec()
    );

    let scheduled_launch = AcceptScheduledMailContactsSyncDueV1 {
        logical_owner_id: "owner-1".to_owned(),
        command_message_id: [60; 16],
        command_envelope_sha256: [61; 32],
        scheduler_run_id: [62; 16],
        lease_epoch: 1,
        lease_expires_at_unix_millis: 1_800_000_031_000,
        launch: Some(MailContactsSyncDraftV1 {
            run_id: [62; 16],
            operation_id: [62; 16],
            account_id: "mail-account-2".to_owned(),
            direction: MailContactsSyncDirectionV1::ProviderToContacts,
            trigger: MailContactsSyncTriggerV1::Scheduled,
        }),
        durable_messages: vec![
            envelope(63, b"scheduler-acceptance"),
            envelope(64, b"scheduled-fetch"),
        ],
        occurred_at_unix_millis: 1_800_000_001_000,
    };
    assert!(matches!(
        persistence
            .accept_scheduled_due(scheduled_launch.clone())
            .await
            .expect("launch scheduled run"),
        AcceptScheduledMailContactsSyncDueOutcomeV1::Launched(_)
    ));
    assert!(matches!(
        persistence
            .accept_scheduled_due(scheduled_launch)
            .await
            .expect("replay scheduled due"),
        AcceptScheduledMailContactsSyncDueOutcomeV1::Duplicate(Some(_))
    ));
    assert_eq!(
        persistence
            .pending_scheduled_terminal("owner-1")
            .await
            .expect("no premature Scheduler terminal"),
        None
    );
    assert_eq!(
        persistence
            .accept_provider_page(&MailContactsSyncPageResultInputV1 {
                logical_owner_id: "owner-1".to_owned(),
                run_id: [62; 16],
                page_sequence: 1,
                message_id: [66; 16],
                envelope_sha256: [67; 32],
                observed_entries: 0,
                next_continuation_cursor: None,
                occurred_at_unix_millis: 1_800_000_001_010,
            })
            .await
            .expect("complete scheduled provider page"),
        MailContactsSyncPersistenceOutcomeV1::Applied
    );
    assert_eq!(
        persistence
            .advance_ready_page(&AdvanceMailContactsSyncPageV1 {
                logical_owner_id: "owner-1".to_owned(),
                run_id: [62; 16],
                next_page_command: None,
                occurred_at_unix_millis: 1_800_000_001_020,
            })
            .await
            .expect("finish scheduled sync"),
        MailContactsSyncAdvanceOutcomeV1::Applied
    );
    let pending_terminal = persistence
        .pending_scheduled_terminal("owner-1")
        .await
        .expect("scheduled terminal after workflow completion")
        .expect("pending scheduled terminal");
    assert_eq!(pending_terminal.run_id, [62; 16]);
    assert_eq!(pending_terminal.command_message_id, [60; 16]);
    assert_eq!(
        pending_terminal.outcome,
        MailContactsSyncScheduledTerminalOutcomeV1::Succeeded
    );
    let terminal = QueueMailContactsSyncScheduledTerminalV1 {
        logical_owner_id: "owner-1".to_owned(),
        run_id: [62; 16],
        terminal_receipt: envelope(68, b"scheduler-terminal-after-completion"),
        queued_at_unix_millis: 1_800_000_001_030,
    };
    assert!(
        persistence
            .queue_scheduled_terminal(&terminal)
            .await
            .expect("queue scheduled terminal")
    );
    assert!(
        !persistence
            .queue_scheduled_terminal(&terminal)
            .await
            .expect("replay queued scheduled terminal")
    );
    assert_eq!(
        persistence
            .pending_scheduled_terminal("owner-1")
            .await
            .expect("terminal queue drained"),
        None
    );

    let disabled_due = AcceptScheduledMailContactsSyncDueV1 {
        logical_owner_id: "owner-1".to_owned(),
        command_message_id: [70; 16],
        command_envelope_sha256: [71; 32],
        scheduler_run_id: [72; 16],
        lease_epoch: 1,
        lease_expires_at_unix_millis: 1_800_000_031_100,
        launch: None,
        durable_messages: vec![
            envelope(73, b"disabled-scheduler-acceptance"),
            envelope(74, b"disabled-scheduler-terminal"),
        ],
        occurred_at_unix_millis: 1_800_000_001_100,
    };
    assert_eq!(
        persistence
            .accept_scheduled_due(disabled_due.clone())
            .await
            .expect("persist disabled no-op"),
        AcceptScheduledMailContactsSyncDueOutcomeV1::Skipped
    );
    assert_eq!(
        persistence
            .accept_scheduled_due(disabled_due)
            .await
            .expect("replay disabled no-op"),
        AcceptScheduledMailContactsSyncDueOutcomeV1::Duplicate(None)
    );
}

#[tokio::test]
#[ignore = "requires the disposable authenticated PostgreSQL contour"]
async fn address_book_target_custody_is_restart_safe_and_conflict_checked() {
    let database_url = std::env::var("MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL")
        .expect("MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL");
    let persistence = MailAddressBookPersistenceConformanceV1::connect_url(&database_url)
        .await
        .expect("connect Mail address-book persistence");
    MailAddressBookPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install Mail address-book schema");
    let admission = MailAddressBookUpsertAdmissionV1 {
        command_message_id: [1; 16],
        command_envelope_sha256: [2; 32],
        command_id: [3; 16],
        run_id: [4; 16],
        logical_owner_id: "owner-1".to_owned(),
        account_id: "mail-account-1".to_owned(),
        contact_snapshot_reference_id: [5; 16],
        contact_snapshot_sha256: [6; 32],
        expected_contact_revision: 7,
        contact_snapshot_declared_bytes: 128,
        contact_snapshot_custody_source_proof: vec![8; 32],
    };
    assert_eq!(
        persistence
            .accept_upsert_command(&admission, 1_800_000_000)
            .await
            .expect("accept address-book command"),
        MailAddressBookCommandInboxOutcomeV1::Accepted
    );
    assert_eq!(
        persistence
            .pending_upserts(1)
            .await
            .expect("load pre-transfer job")[0]
            .target_snapshot_receipt,
        None
    );

    let receipt = MailAddressBookTargetSnapshotReceiptV1 {
        reference_id: [9; 16],
        receipt_sha256: admission.contact_snapshot_sha256,
    };
    assert_eq!(
        persistence
            .record_target_snapshot_receipt(admission.command_id, receipt, 1_800_000_001)
            .await
            .expect("record target custody"),
        MailAddressBookSnapshotCustodyOutcomeV1::Recorded
    );
    assert_eq!(
        persistence
            .record_target_snapshot_receipt(admission.command_id, receipt, 1_800_000_002)
            .await
            .expect("replay target custody"),
        MailAddressBookSnapshotCustodyOutcomeV1::AlreadyRecorded
    );
    let conflicting = MailAddressBookTargetSnapshotReceiptV1 {
        reference_id: [10; 16],
        ..receipt
    };
    assert_eq!(
        persistence
            .record_target_snapshot_receipt(admission.command_id, conflicting, 1_800_000_003)
            .await,
        Err(MailAddressBookPersistenceErrorV1::Conflict)
    );
    assert_eq!(
        persistence
            .pending_upserts(1)
            .await
            .expect("reload restart-safe pending job")[0]
            .target_snapshot_receipt,
        Some(receipt)
    );
    assert_eq!(
        persistence
            .mark_dispatch_started(admission.command_id, 1_800_000_004)
            .await
            .expect("mark provider dispatch"),
        MailAddressBookDispatchOutcomeV1::Started
    );
    assert_eq!(
        persistence
            .uncertain_upserts(1)
            .await
            .expect("load uncertain job")[0]
            .target_snapshot_receipt,
        Some(receipt)
    );
}

#[tokio::test]
#[ignore = "requires the disposable authenticated PostgreSQL contour"]
async fn reverse_provider_result_is_atomic_restart_safe_and_replayable() {
    let database_url = std::env::var("MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL")
        .expect("MAKOSH_MAIL_CONTACTS_SYNC_POSTGRES_URL");
    let persistence = MailContactsSyncPersistenceConformanceV1::connect_url(&database_url)
        .await
        .expect("connect workflow persistence");
    MailContactsSyncPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install workflow schema");

    let run_id = [80; 16];
    persistence
        .create_run(create_run_with_direction(
            80,
            MailContactsSyncDirectionV1::Bidirectional,
        ))
        .await
        .expect("create bidirectional run");
    persistence
        .accept_provider_entry(&MailContactsSyncEntryInputV1 {
            logical_owner_id: "owner-1".to_owned(),
            run_id,
            page_sequence: 1,
            observation_message_id: [81; 16],
            observation_envelope_sha256: [82; 32],
            contact_command_id: [83; 16],
            entry_digest: [84; 32],
            contact_command: envelope(83, b"reverse-contacts-upsert"),
            occurred_at_unix_millis: 1_800_000_010_100,
        })
        .await
        .expect("accept provider entry");
    persistence
        .accept_contact_outcome(&MailContactsSyncEntryOutcomeInputV1 {
            logical_owner_id: "owner-1".to_owned(),
            contact_command_id: [83; 16],
            message_id: [85; 16],
            envelope_sha256: [86; 32],
            outcome: MailContactsSyncContactOutcomeV1::Created,
            occurred_at_unix_millis: 1_800_000_010_200,
        })
        .await
        .expect("accept Contacts result");
    persistence
        .accept_provider_page(&MailContactsSyncPageResultInputV1 {
            logical_owner_id: "owner-1".to_owned(),
            run_id,
            page_sequence: 1,
            message_id: [87; 16],
            envelope_sha256: [88; 32],
            observed_entries: 1,
            next_continuation_cursor: None,
            occurred_at_unix_millis: 1_800_000_010_300,
        })
        .await
        .expect("complete provider page");
    assert_eq!(
        persistence
            .advance_ready_page(&AdvanceMailContactsSyncPageV1 {
                logical_owner_id: "owner-1".to_owned(),
                run_id,
                next_page_command: None,
                occurred_at_unix_millis: 1_800_000_010_400,
            })
            .await
            .expect("advance bidirectional run"),
        MailContactsSyncAdvanceOutcomeV1::Applied
    );
    assert_eq!(
        persistence
            .load_run("owner-1", &run_id)
            .await
            .expect("load provider-write run")
            .status
            .state,
        MailContactsSyncStateV1::WritingProvider
    );

    let operation_id = [89; 16];
    let changed = AcceptContactChangedForMailSyncV1 {
        logical_owner_id: "owner-1".to_owned(),
        event_message_id: [90; 16],
        event_envelope_sha256: [91; 32],
        operations: vec![MailContactsSyncReverseOperationSeedV1 {
            operation_id,
            configuration_instance_id: "mail-contacts-sync-1".to_owned(),
            account_id: "mail-account-1".to_owned(),
            contact_id: [92; 16],
            contact_revision: 1,
            origin_run_id: Some(run_id),
            source_prepare_command: envelope(89, b"prepare-private-contact-source"),
        }],
        occurred_at_unix_millis: 1_800_000_010_500,
    };
    assert_eq!(
        persistence
            .accept_contact_changed_for_mail_sync(&changed)
            .await
            .expect("accept caused Contact change"),
        AcceptContactChangedForMailSyncOutcomeV1::Applied { operations: 1 }
    );
    assert_eq!(
        persistence
            .accept_contact_changed_for_mail_sync(&changed)
            .await
            .expect("replay caused Contact change"),
        AcceptContactChangedForMailSyncOutcomeV1::Duplicate
    );
    let mail_command = envelope(93, b"mail-provider-upsert-command");
    let source_completed = CompleteContactMailSyncSourceV1 {
        logical_owner_id: "owner-1".to_owned(),
        result_message_id: [94; 16],
        result_envelope_sha256: [95; 32],
        operation_id,
        mail_command: Some(mail_command.clone()),
        rejected: false,
        occurred_at_unix_millis: 1_800_000_010_600,
    };
    assert_eq!(
        persistence
            .complete_contact_mail_sync_source(&source_completed)
            .await
            .expect("queue Mail provider command"),
        CompleteContactMailSyncSourceOutcomeV1::Applied
    );

    drop(persistence);
    let restarted = MailContactsSyncPersistenceConformanceV1::connect_url(&database_url)
        .await
        .expect("restart workflow persistence");
    let provider_result = CompleteMailAddressBookUpsertV1 {
        logical_owner_id: "owner-1".to_owned(),
        result_message_id: [96; 16],
        result_envelope_sha256: [97; 32],
        operation_id,
        mail_command_message_id: mail_command.message_id,
        outcome: MailContactsSyncProviderWriteOutcomeV1::Succeeded,
        contacts_link_command: Some(envelope(108, b"contacts-provider-link-command")),
        occurred_at_unix_millis: 1_800_000_010_700,
    };
    assert_eq!(
        restarted
            .complete_mail_address_book_upsert(&provider_result)
            .await
            .expect("commit provider result after restart"),
        CompleteMailAddressBookUpsertOutcomeV1::Applied
    );
    assert_eq!(
        restarted
            .complete_mail_address_book_upsert(&provider_result)
            .await
            .expect("replay provider result"),
        CompleteMailAddressBookUpsertOutcomeV1::Duplicate
    );
    let awaiting_link = restarted
        .load_reverse_operation("owner-1", operation_id)
        .await
        .expect("load reverse operation awaiting Contacts link");
    assert_eq!(awaiting_link.state, 2);
    assert_eq!(awaiting_link.origin_run_id, Some(run_id));
    assert_eq!(
        awaiting_link.mail_command_message_id,
        Some(mail_command.message_id)
    );
    assert_eq!(
        restarted
            .complete_contacts_provider_link(&CompleteContactsProviderLinkV1 {
                logical_owner_id: "owner-1".to_owned(),
                result_message_id: [109; 16],
                result_envelope_sha256: [110; 32],
                operation_id,
                contacts_command_message_id: [108; 16],
                reject_code: None,
                occurred_at_unix_millis: 1_800_000_010_750,
            })
            .await
            .expect("complete Contacts provider link"),
        CompleteContactsProviderLinkOutcomeV1::Applied
    );
    let operation = restarted
        .load_reverse_operation("owner-1", operation_id)
        .await
        .expect("load completed reverse operation");
    assert_eq!(operation.state, 3);
    let completed = restarted
        .load_run("owner-1", &run_id)
        .await
        .expect("load completed bidirectional run");
    assert_eq!(completed.status.state, MailContactsSyncStateV1::Completed);
    assert_eq!(completed.status.counters.provider_entries_written, 1);
    assert_eq!(completed.status.rejection, None);

    let mut conflicting_result = provider_result;
    conflicting_result.result_envelope_sha256 = [98; 32];
    assert_eq!(
        restarted
            .complete_mail_address_book_upsert(&conflicting_result)
            .await,
        Err(MailContactsSyncPersistenceErrorV1::InboxConflict)
    );

    let late_operation_id = [99; 16];
    restarted
        .accept_contact_changed_for_mail_sync(&AcceptContactChangedForMailSyncV1 {
            logical_owner_id: "owner-1".to_owned(),
            event_message_id: [100; 16],
            event_envelope_sha256: [101; 32],
            operations: vec![MailContactsSyncReverseOperationSeedV1 {
                operation_id: late_operation_id,
                configuration_instance_id: "mail-contacts-sync-1".to_owned(),
                account_id: "mail-account-1".to_owned(),
                contact_id: [102; 16],
                contact_revision: 2,
                origin_run_id: Some(run_id),
                source_prepare_command: envelope(99, b"late-private-contact-source"),
            }],
            occurred_at_unix_millis: 1_800_000_010_800,
        })
        .await
        .expect("accept late caused Contact change");
    restarted
        .complete_contact_mail_sync_source(&CompleteContactMailSyncSourceV1 {
            logical_owner_id: "owner-1".to_owned(),
            result_message_id: [103; 16],
            result_envelope_sha256: [104; 32],
            operation_id: late_operation_id,
            mail_command: Some(envelope(105, b"late-mail-provider-upsert-command")),
            rejected: false,
            occurred_at_unix_millis: 1_800_000_010_900,
        })
        .await
        .expect("queue late Mail provider command");
    restarted
        .complete_mail_address_book_upsert(&CompleteMailAddressBookUpsertV1 {
            logical_owner_id: "owner-1".to_owned(),
            result_message_id: [106; 16],
            result_envelope_sha256: [107; 32],
            operation_id: late_operation_id,
            mail_command_message_id: [105; 16],
            outcome: MailContactsSyncProviderWriteOutcomeV1::Succeeded,
            contacts_link_command: Some(envelope(111, b"late-contacts-provider-link-command")),
            occurred_at_unix_millis: 1_800_000_011_000,
        })
        .await
        .expect("queue late Contacts provider-link reconciliation");
    restarted
        .complete_contacts_provider_link(&CompleteContactsProviderLinkV1 {
            logical_owner_id: "owner-1".to_owned(),
            result_message_id: [112; 16],
            result_envelope_sha256: [113; 32],
            operation_id: late_operation_id,
            contacts_command_message_id: [111; 16],
            reject_code: None,
            occurred_at_unix_millis: 1_800_000_011_050,
        })
        .await
        .expect("terminalize late provider link without rewriting completed run");
    assert_eq!(
        restarted
            .load_reverse_operation("owner-1", late_operation_id)
            .await
            .expect("load late reverse operation")
            .state,
        3
    );
    let still_completed = restarted
        .load_run("owner-1", &run_id)
        .await
        .expect("reload completed run after late result");
    assert_eq!(
        still_completed.status.state,
        MailContactsSyncStateV1::Completed
    );
    assert_eq!(still_completed.status.counters.provider_entries_written, 1);

    let realtime = restarted
        .client_realtime_window("owner-1", None, 16)
        .await
        .expect("load reverse realtime history");
    assert_eq!(
        realtime
            .iter()
            .filter(|transition| transition.run_id == run_id)
            .map(|transition| transition.state)
            .collect::<Vec<_>>(),
        [
            MailContactsSyncStateV1::FetchingProviderPage,
            MailContactsSyncStateV1::ApplyingContacts,
            MailContactsSyncStateV1::WritingProvider,
            MailContactsSyncStateV1::Completed,
        ]
    );
}

fn create_run(seed: u8) -> CreateMailContactsSyncRunV1 {
    create_run_with_direction(seed, MailContactsSyncDirectionV1::ProviderToContacts)
}

fn create_run_with_direction(
    seed: u8,
    direction: MailContactsSyncDirectionV1,
) -> CreateMailContactsSyncRunV1 {
    CreateMailContactsSyncRunV1 {
        logical_owner_id: "owner-1".to_owned(),
        draft: MailContactsSyncDraftV1 {
            run_id: [seed; 16],
            operation_id: [seed.wrapping_add(1); 16],
            account_id: "mail-account-1".to_owned(),
            direction,
            trigger: MailContactsSyncTriggerV1::Manual,
        },
        initial_commands: vec![envelope(seed.wrapping_add(10), b"initial-command")],
        created_at_unix_millis: 1_800_000_000_000,
    }
}

fn envelope(seed: u8, value: &[u8]) -> OutboxEnvelopeV1 {
    OutboxEnvelopeV1 {
        message_id: [seed; 16],
        envelope_sha256: Sha256::digest(value).into(),
        envelope_bytes: value.to_vec(),
    }
}
