use makosh_mail_persons_sync_persistence::{
    MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncPersistenceErrorV1,
    MailPersonsSyncSemanticKindV1, StagedSourceV1, mail_persons_sync_semantic_order_key_v1,
    mail_persons_sync_storage_bundle_v1, validate_page_promotion_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use sha2::{Digest, Sha256};

fn id(seed: u8) -> [u8; 16] {
    [seed; 16]
}

#[test]
fn storage_bundle_is_owner_local_rls_and_content_negative() {
    let bundle = mail_persons_sync_storage_bundle_v1();
    validate_storage_bundle(&bundle).expect("valid storage bundle");
    assert_eq!(bundle.owner_id, "mail_persons_sync");
    let sql = bundle
        .steps
        .iter()
        .flat_map(|step| step.forward_sql_utf8.iter().copied())
        .collect::<Vec<_>>();
    let sql = String::from_utf8(sql).expect("utf8").to_lowercase();
    for table in [
        "mail_persons_sync_runs",
        "mail_persons_sync_scheduler_runs",
        "mail_persons_sync_inbox",
        "mail_persons_sync_pages",
        "mail_persons_sync_sources",
        "mail_persons_sync_outbox",
    ] {
        assert!(sql.contains(table), "{table}");
    }
    assert!(sql.contains("force row level security"));
    for forbidden in [
        "provider_entry_id",
        "provider_etag",
        "continuation_cursor",
        "credential",
        "private_locator",
        "raw_payload",
        "create role",
        "bypassrls",
    ] {
        assert!(!sql.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn exact_envelope_hash_and_semantic_order_fail_closed() {
    let envelope = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: id(1).to_vec(),
            run_id: id(2).to_vec(),
            logical_owner_id: "owner-1".to_owned(),
            account_public_id: id(3).to_vec(),
            page_sequence: 1,
            page_size: 500,
        },
        2,
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-persons-sync-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("durable envelope");
    let bytes = envelope.exact_bytes().to_vec();
    let record = MailPersonsSyncEnvelopeRecordV1::new(*envelope.message_id(), bytes.clone())
        .expect("record");
    assert_eq!(
        record.envelope_sha256,
        <[u8; 32]>::from(Sha256::digest(&bytes))
    );
    let mut corrupted = record.clone();
    corrupted.envelope_bytes.push(0);
    assert_eq!(
        corrupted.validate(),
        Err(MailPersonsSyncPersistenceErrorV1::HashMismatch)
    );
    assert_eq!(
        MailPersonsSyncEnvelopeRecordV1::new(id(9), bytes),
        Err(MailPersonsSyncPersistenceErrorV1::InvalidInput),
        "record ID must equal the embedded durable envelope message ID",
    );

    let source_a = id(3);
    let source_b = id(4);
    let command_a = mail_persons_sync_semantic_order_key_v1(
        2,
        MailPersonsSyncSemanticKindV1::PersonsCommand,
        Some(source_a),
        1,
    )
    .expect("key");
    let command_b = mail_persons_sync_semantic_order_key_v1(
        2,
        MailPersonsSyncSemanticKindV1::PersonsCommand,
        Some(source_b),
        2,
    )
    .expect("key");
    let receipt = mail_persons_sync_semantic_order_key_v1(
        2,
        MailPersonsSyncSemanticKindV1::PageReceipt,
        None,
        3,
    )
    .expect("key");
    assert!(command_a < command_b && command_b < receipt);
}

#[test]
fn page_promotion_requires_exact_bounded_unique_source_set() {
    let staged = vec![
        StagedSourceV1 {
            public_source_id: id(1),
            observed: 1,
            updated: 0,
            removed: 0,
        },
        StagedSourceV1 {
            public_source_id: id(2),
            observed: 0,
            updated: 1,
            removed: 0,
        },
        StagedSourceV1 {
            public_source_id: id(3),
            observed: 0,
            updated: 0,
            removed: 1,
        },
    ];
    let ordered = validate_page_promotion_v1(1, 1, 1, &staged).expect("exact page");
    assert_eq!(
        ordered
            .iter()
            .map(|source| source.public_source_id)
            .collect::<Vec<_>>(),
        vec![id(1), id(2), id(3)]
    );

    assert_eq!(
        validate_page_promotion_v1(2, 1, 1, &staged),
        Err(MailPersonsSyncPersistenceErrorV1::PageIncomplete)
    );
    let mut duplicate = staged;
    duplicate.push(duplicate[0].clone());
    assert_eq!(
        validate_page_promotion_v1(2, 1, 1, &duplicate),
        Err(MailPersonsSyncPersistenceErrorV1::StateConflict)
    );
}
use makosh_mail_address_book_contract::{
    MailAddressBookEnvelopeContextV1, build_fetch_mail_person_source_page_command_v1,
    wire_person_source::FetchMailPersonSourcePageCommandV1,
};
