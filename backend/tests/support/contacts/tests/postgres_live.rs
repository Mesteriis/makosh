use makosh_contacts_core::{
    ContactProviderKindV1, ContactProviderProvenanceV1, ContactTimestampV1, ContactUpsertDraftV1,
    ContactUpsertOutcomeV1,
};
use makosh_contacts_persistence::{
    ApplyMailEntryCommandV1, BindMailProviderLinkCommandV1, ContactMailEntryRejectCodeV1,
    ContactMutationOutboxV1, ContactProviderLinkBindOutcomeV1, ContactProviderLinkBindRejectCodeV1,
    ContactsOutboxRecordV1, ContactsPersistenceConformanceV1, ContactsPersistenceErrorV1,
    PersistContactMailSyncSourceResultV1, RejectMailEntryCommandV1, ReserveContactMailSyncSourceV1,
};
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires the disposable authenticated PostgreSQL contour"]
async fn postgres_replays_exact_result_and_fences_conflicts() {
    let database_url =
        std::env::var("MAKOSH_CONTACTS_POSTGRES_URL").expect("MAKOSH_CONTACTS_POSTGRES_URL");
    let persistence = ContactsPersistenceConformanceV1::connect_url(&database_url)
        .await
        .expect("connect Contacts persistence");
    ContactsPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install Contacts schema");
    let first = command(1, 1, "ada@example.test", "+34910000001");
    let created = persistence
        .apply_mail_entry(&first, |contact, outcome| {
            mutation(11, contact.contact_id, outcome)
        })
        .await
        .expect("create contact");
    assert_eq!(created.outcome, ContactUpsertOutcomeV1::Created);
    assert_eq!(created.contact_revision, 1);
    assert!(!created.replayed);

    let source_reservation = ReserveContactMailSyncSourceV1 {
        command_message_id: [70; 16],
        command_envelope_sha256: [71; 32],
        operation_id: [72; 16],
        contact_id: created.contact_id,
        expected_contact_revision: created.contact_revision,
        target_mail_account_id: "mail-1".to_owned(),
        logical_owner_id: "owner-1".to_owned(),
        received_at_unix_millis: 1_800_000_000_070,
    };
    assert_eq!(
        persistence
            .reserve_contact_mail_sync_source(&source_reservation)
            .await
            .expect("reserve source command"),
        None,
    );
    let snapshot = persistence
        .contact_mail_sync_source_snapshot(
            "owner-1",
            created.contact_id,
            created.contact_revision,
            "mail-1",
        )
        .await
        .expect("load exact source snapshot");
    assert_eq!(snapshot.display_name, "Ada");
    assert_eq!(
        snapshot
            .target_account_link
            .expect("Mail link")
            .provider_entry_id,
        "people/ada"
    );
    let source_terminal = terminal(73, created.contact_id, ContactUpsertOutcomeV1::Unchanged)
        .expect("source terminal fixture");
    let source_completion = PersistContactMailSyncSourceResultV1 {
        command_message_id: source_reservation.command_message_id,
        command_envelope_sha256: source_reservation.command_envelope_sha256,
        operation_id: source_reservation.operation_id,
        contact_id: source_reservation.contact_id,
        expected_contact_revision: source_reservation.expected_contact_revision,
        target_mail_account_id: source_reservation.target_mail_account_id.clone(),
        logical_owner_id: source_reservation.logical_owner_id.clone(),
        reject_code: None,
        terminal_result: source_terminal.clone(),
        received_at_unix_millis: source_reservation.received_at_unix_millis,
        completed_at_unix_millis: 1_800_000_000_071,
    };
    let completed_source = persistence
        .persist_contact_mail_sync_source_result(&source_completion)
        .await
        .expect("complete source command");
    assert!(!completed_source.replayed);
    let source_replay = persistence
        .reserve_contact_mail_sync_source(&source_reservation)
        .await
        .expect("replay source command")
        .expect("completed source result");
    assert!(source_replay.replayed);
    assert_eq!(source_replay.terminal_result, source_terminal);

    let first_replay = persistence
        .apply_mail_entry(&first, |_, _| {
            panic!("replay must load the persisted exact result")
        })
        .await
        .expect("replay first command");
    assert!(first_replay.replayed);
    assert_eq!(first_replay.contact_revision, 1);
    assert_eq!(first_replay.terminal_result, created.terminal_result);

    let updated_input = command(2, 2, "ada@example.test", "+34910000001");
    let updated = persistence
        .apply_mail_entry(&updated_input, |contact, outcome| {
            mutation(12, contact.contact_id, outcome)
        })
        .await
        .expect("update contact");
    assert_eq!(updated.contact_id, created.contact_id);
    assert_eq!(updated.outcome, ContactUpsertOutcomeV1::Updated);
    assert_eq!(updated.contact_revision, 2);

    let replay_after_update = persistence
        .apply_mail_entry(&first, |_, _| panic!("replay must not rebuild the result"))
        .await
        .expect("replay after later update");
    assert_eq!(replay_after_update.contact_revision, 1);
    assert_eq!(replay_after_update.terminal_result, created.terminal_result);

    let unchanged_input = command(3, 2, "ada@example.test", "+34910000001");
    let unchanged = persistence
        .apply_mail_entry(&unchanged_input, |contact, outcome| {
            mutation(13, contact.contact_id, outcome)
        })
        .await
        .expect("unchanged contact");
    assert_eq!(unchanged.outcome, ContactUpsertOutcomeV1::Unchanged);
    assert_eq!(unchanged.contact_revision, 2);

    let link = BindMailProviderLinkCommandV1 {
        command_message_id: [80; 16],
        command_envelope_sha256: [81; 32],
        command_id: [82; 16],
        logical_owner_id: "owner-1".to_owned(),
        contact_id: created.contact_id,
        expected_contact_revision: 2,
        source_account_id: "mail-1".to_owned(),
        provider_kind: ContactProviderKindV1::Gmail,
        provider_entry_id: "people/ada".to_owned(),
        provider_etag: Some("etag-reconciled".to_owned()),
        received_at_unix_millis: 1_800_000_000_080,
        completed_at_unix_millis: 1_800_000_000_081,
    };
    let linked = persistence
        .bind_mail_provider_link(&link, |_| {
            terminal(83, created.contact_id, ContactUpsertOutcomeV1::Unchanged)
        })
        .await
        .expect("reconcile provider link");
    assert_eq!(
        linked.outcome,
        ContactProviderLinkBindOutcomeV1::Bound {
            contact_revision: 2
        }
    );
    assert!(!linked.replayed);
    assert!(
        persistence
            .bind_mail_provider_link(&link, |_| panic!("link replay must use exact result"))
            .await
            .expect("replay provider link")
            .replayed
    );
    let mut conflicting_link = link.clone();
    conflicting_link.command_message_id = [84; 16];
    conflicting_link.command_envelope_sha256 = [85; 32];
    conflicting_link.command_id = [86; 16];
    conflicting_link.provider_entry_id = "people/different".to_owned();
    let conflict = persistence
        .bind_mail_provider_link(&conflicting_link, |_| {
            terminal(87, created.contact_id, ContactUpsertOutcomeV1::Unchanged)
        })
        .await
        .expect("persist provider-link conflict");
    assert_eq!(
        conflict.outcome,
        ContactProviderLinkBindOutcomeV1::Rejected(
            ContactProviderLinkBindRejectCodeV1::ProviderLinkConflict
        )
    );

    let mut reused_command_id = command(4, 3, "ada@example.test", "+34910000001");
    reused_command_id.command_id = first.command_id;
    assert_eq!(
        persistence
            .apply_mail_entry(&reused_command_id, |contact, outcome| {
                mutation(14, contact.contact_id, outcome)
            })
            .await,
        Err(ContactsPersistenceErrorV1::CommandConflict)
    );

    let second = command(5, 1, "grace@example.test", "+34910000002");
    persistence
        .apply_mail_entry(&second, |contact, outcome| {
            mutation(15, contact.contact_id, outcome)
        })
        .await
        .expect("create second contact");
    let ambiguous = command(6, 4, "ada@example.test", "+34910000002");
    assert_eq!(
        persistence
            .apply_mail_entry(&ambiguous, |contact, outcome| {
                mutation(16, contact.contact_id, outcome)
            })
            .await,
        Err(ContactsPersistenceErrorV1::IdentityAmbiguous)
    );
    let rejected_result = terminal(16, ambiguous.command_id, ContactUpsertOutcomeV1::Unchanged)
        .expect("rejected terminal fixture");
    let rejection = RejectMailEntryCommandV1 {
        command_message_id: ambiguous.command_message_id,
        command_envelope_sha256: ambiguous.command_envelope_sha256,
        command_id: ambiguous.command_id,
        logical_owner_id: ambiguous.draft.logical_owner_id.clone(),
        entry_digest: ambiguous.draft.provenance.entry_digest,
        received_at_unix_millis: ambiguous.received_at_unix_millis,
        completed_at_unix_millis: ambiguous.completed_at_unix_millis,
        code: ContactMailEntryRejectCodeV1::IdentityAmbiguous,
        terminal_result: rejected_result.clone(),
    };
    let rejected = persistence
        .reject_mail_entry(&rejection)
        .await
        .expect("persist rejection");
    assert!(!rejected.replayed);
    assert_eq!(rejected.terminal_result, rejected_result);
    let replayed_rejection = persistence
        .reject_mail_entry(&rejection)
        .await
        .expect("replay rejection");
    assert!(replayed_rejection.replayed);
    assert_eq!(replayed_rejection.terminal_result, rejected_result);
    assert_eq!(
        persistence
            .apply_mail_entry(&ambiguous, |_, _| panic!("rejected command cannot apply"))
            .await,
        Err(ContactsPersistenceErrorV1::IdentityAmbiguous)
    );

    let pending = persistence
        .load_pending_outbox("owner-1")
        .await
        .expect("pending outbox");
    assert_eq!(pending.len(), 11);
}

fn mutation(
    seed: u8,
    contact_id: [u8; 16],
    outcome: ContactUpsertOutcomeV1,
) -> Result<ContactMutationOutboxV1, ContactsPersistenceErrorV1> {
    let terminal_result = terminal(seed, contact_id, outcome)?;
    let changed_event = (outcome != ContactUpsertOutcomeV1::Unchanged).then(|| {
        let mut bytes = b"contacts-changed-v1:".to_vec();
        bytes.extend_from_slice(&contact_id);
        ContactsOutboxRecordV1 {
            message_id: [seed.wrapping_add(100); 16],
            envelope_sha256: Sha256::digest(&bytes).into(),
            envelope_bytes: bytes,
        }
    });
    Ok(ContactMutationOutboxV1 {
        terminal_result,
        changed_event,
    })
}

fn command(seed: u8, source_revision: u64, email: &str, phone: &str) -> ApplyMailEntryCommandV1 {
    ApplyMailEntryCommandV1 {
        command_message_id: [seed; 16],
        command_envelope_sha256: [seed.wrapping_add(20); 32],
        command_id: [seed.wrapping_add(40); 16],
        draft: ContactUpsertDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            display_name: if email.starts_with("ada") {
                "Ada"
            } else {
                "Grace"
            }
            .to_owned(),
            email_addresses: vec![email.to_owned()],
            phone_numbers: vec![phone.to_owned()],
            provenance: ContactProviderProvenanceV1 {
                source_account_id: "mail-1".to_owned(),
                provider_kind: ContactProviderKindV1::Gmail,
                provider_entry_id: if email.starts_with("ada") {
                    "people/ada"
                } else {
                    "people/grace"
                }
                .to_owned(),
                provider_etag: Some(format!("etag-{source_revision}")),
                source_revision,
                entry_digest: [source_revision as u8; 32],
                observed_at: ContactTimestampV1 {
                    unix_seconds: 1_800_000_000 + i64::try_from(source_revision).expect("revision"),
                    nanos: 0,
                },
            },
        },
        received_at_unix_millis: 1_800_000_000_000 + i64::from(seed),
        completed_at_unix_millis: 1_800_000_000_100 + i64::from(seed),
    }
}

fn terminal(
    seed: u8,
    contact_id: [u8; 16],
    outcome: ContactUpsertOutcomeV1,
) -> Result<ContactsOutboxRecordV1, ContactsPersistenceErrorV1> {
    let outcome = match outcome {
        ContactUpsertOutcomeV1::Created => b"created".as_slice(),
        ContactUpsertOutcomeV1::Updated => b"updated".as_slice(),
        ContactUpsertOutcomeV1::Unchanged => b"unchanged".as_slice(),
    };
    let mut bytes = b"contacts-terminal-v1:".to_vec();
    bytes.extend_from_slice(&contact_id);
    bytes.extend_from_slice(outcome);
    Ok(ContactsOutboxRecordV1 {
        message_id: [seed; 16],
        envelope_sha256: Sha256::digest(&bytes).into(),
        envelope_bytes: bytes,
    })
}
