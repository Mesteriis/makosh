use makosh_mail_address_book_contract::{
    MailAddressBookEnvelopeContextV1, build_fetch_mail_person_source_page_command_v1,
    wire_person_source::FetchMailPersonSourcePageCommandV1,
};
use makosh_mail_persons_sync_persistence::{
    BeginMailPersonsSyncRunV1, CompleteMailPersonsSyncPageV1, MailPersonsSyncEnvelopeRecordV1,
    MailPersonsSyncPageContinuationV1, MailPersonsSyncPersistenceErrorV1,
    MailPersonsSyncStoredRejectCodeV1, RecordMailPersonsSyncPersonsTerminalV1,
    StageMailPersonsSyncSourceV1,
};

fn id(seed: u8) -> [u8; 16] {
    [seed; 16]
}

#[test]
fn persons_terminal_binds_command_result_and_typed_outcome() {
    let valid = RecordMailPersonsSyncPersonsTerminalV1 {
        logical_owner_id: "owner-1".to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        persons_command_id: id(3),
        result: envelope(4),
        outcome: 1,
        result_completed_at_unix_millis: 1_100,
        received_at_unix_millis: 1_200,
    };
    valid.validate().expect("terminal");
    let mut invalid = valid;
    invalid.outcome = 3;
    assert_eq!(
        invalid.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
}

fn digest(seed: u8) -> [u8; 32] {
    [seed; 32]
}

fn envelope(seed: u8) -> MailPersonsSyncEnvelopeRecordV1 {
    let envelope = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: id(seed).to_vec(),
            run_id: id(250).to_vec(),
            logical_owner_id: "owner-1".to_owned(),
            account_public_id: id(251).to_vec(),
            page_sequence: 1,
            page_size: 500,
        },
        2,
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

#[test]
fn begin_run_binds_owner_account_scheduler_and_lease() {
    let valid = BeginMailPersonsSyncRunV1 {
        logical_owner_id: "owner-1".to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        run_fingerprint: digest(3),
        scheduler_command: envelope(4),
        scheduler_acceptance: envelope(5),
        initial_fetch: envelope(6),
        lease_epoch: 1,
        lease_expires_at_unix_millis: 2_000,
        received_at_unix_millis: 1_000,
    };
    valid.validate().expect("valid run");
    let mut expired = valid.clone();
    expired.lease_expires_at_unix_millis = expired.received_at_unix_millis;
    assert_eq!(
        expired.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
    let mut foreign_owner = valid.clone();
    foreign_owner.logical_owner_id = "OWNER:1".to_owned();
    assert_eq!(
        foreign_owner.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
    let mut duplicate_outbox_ids = valid;
    duplicate_outbox_ids.initial_fetch = duplicate_outbox_ids.scheduler_acceptance.clone();
    assert_eq!(
        duplicate_outbox_ids.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
}

#[test]
fn staged_source_binds_exact_public_tuple_and_persons_command() {
    let valid = StageMailPersonsSyncSourceV1 {
        logical_owner_id: "owner-1".to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        observation: envelope(3),
        integration_public_id: id(4),
        provider_source_contact_public_id: id(5),
        change_kind: 1,
        source_revision: 1,
        source_digest: digest(6),
        persons_command_id: id(7),
        persons_command_fingerprint: digest(8),
        persons_command: envelope(9),
        received_at_unix_millis: 1_000,
    };
    valid.validate().expect("valid source");
    let mut invalid_kind = valid.clone();
    invalid_kind.change_kind = 4;
    assert_eq!(
        invalid_kind.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
    let mut wrong_account = valid;
    wrong_account.account_public_id = [0; 16];
    assert_eq!(
        wrong_account.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
}

#[test]
fn page_completion_is_bounded_and_binds_exact_receipt() {
    let valid = CompleteMailPersonsSyncPageV1 {
        logical_owner_id: "owner-1".to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        completion: envelope(3),
        page_digest: digest(4),
        observed_sources: 250,
        updated_sources: 125,
        removed_sources: 125,
        has_more: false,
        page_receipt: envelope(5),
        rejection_code: None,
        continuation: MailPersonsSyncPageContinuationV1::Finished {
            run_result: envelope(6),
            scheduler_terminal: envelope(7),
        },
        completed_at_unix_millis: 1_000,
    };
    valid.validate().expect("bounded page");
    let mut overflow = valid.clone();
    overflow.removed_sources = 126;
    assert_eq!(
        overflow.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
    let mut zero_digest = valid;
    zero_digest.page_digest = [0; 32];
    assert_eq!(
        zero_digest.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
}

#[test]
fn rejected_page_requires_a_typed_terminal_rejection_code() {
    let mut rejected = CompleteMailPersonsSyncPageV1 {
        logical_owner_id: "owner-1".to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        completion: envelope(3),
        page_digest: digest(4),
        observed_sources: 0,
        updated_sources: 0,
        removed_sources: 0,
        has_more: false,
        page_receipt: envelope(5),
        rejection_code: Some(MailPersonsSyncStoredRejectCodeV1::SourceUnavailable),
        continuation: MailPersonsSyncPageContinuationV1::Finished {
            run_result: envelope(6),
            scheduler_terminal: envelope(7),
        },
        completed_at_unix_millis: 1_000,
    };
    rejected.validate().expect("typed rejected page");

    rejected.has_more = true;
    rejected.continuation = MailPersonsSyncPageContinuationV1::NextPage {
        next_fetch: envelope(8),
    };
    assert_eq!(
        rejected.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput),
    );
}

#[test]
fn page_continuation_exactly_matches_has_more() {
    let mut page = CompleteMailPersonsSyncPageV1 {
        logical_owner_id: "owner-1".to_owned(),
        account_public_id: id(1),
        run_id: id(2),
        page_sequence: 1,
        completion: envelope(3),
        page_digest: digest(4),
        observed_sources: 0,
        updated_sources: 0,
        removed_sources: 0,
        has_more: true,
        page_receipt: envelope(5),
        rejection_code: None,
        continuation: MailPersonsSyncPageContinuationV1::NextPage {
            next_fetch: envelope(6),
        },
        completed_at_unix_millis: 1_000,
    };
    page.validate().expect("next page");
    page.continuation = MailPersonsSyncPageContinuationV1::Finished {
        run_result: envelope(7),
        scheduler_terminal: envelope(8),
    };
    assert_eq!(
        page.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
    );
}
