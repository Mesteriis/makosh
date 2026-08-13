use makosh_mail_address_book_contract::{
    MailAddressBookEnvelopeContextV1,
    wire_person_source::{MailPersonSourceClaimsV1, MailPersonSourceIdentityV1},
};
use makosh_mail_address_book_persistence::{
    MAIL_ADDRESS_BOOK_PERSON_SOURCE_SCHEMA_V1,
    MAIL_ADDRESS_BOOK_PERSON_SOURCE_STORAGE_BUNDLE_REVISION_V1,
    MAIL_ADDRESS_BOOK_STORAGE_BUNDLE_REVISION_V1, terminal_snapshot_tombstones_v1,
};
use makosh_mail_runtime::person_source_producer::{
    MailPersonSourcePublicChangeInputV1, MailPersonSourcePublicChangeV1,
    MailPersonSourceSyntheticRemovalV1, build_public_source_change_v1,
    build_synthetic_removal_page_v1, derive_public_account_mapping_v1,
    derive_public_source_contact_id_v1, mail_person_source_fetch_id_v1,
    plan_synthetic_removal_pages_v1,
};
use prost_types::Timestamp;

fn id(seed: u8) -> [u8; 16] {
    [seed; 16]
}

#[test]
fn random_seed_derivation_is_owner_scoped_stable_and_public_only() {
    let first =
        derive_public_account_mapping_v1("owner-a", "mail-account-a", [7; 32]).expect("mapping");
    assert_eq!(
        first,
        derive_public_account_mapping_v1("owner-a", "mail-account-a", [7; 32]).expect("stable")
    );
    assert_ne!(
        first,
        derive_public_account_mapping_v1("owner-b", "mail-account-a", [7; 32])
            .expect("owner isolated")
    );
    assert_ne!(first.account_public_id, first.integration_public_id);
    assert!(first.account_public_id.iter().any(|byte| *byte != 0));
    assert!(first.integration_public_id.iter().any(|byte| *byte != 0));
}

#[test]
fn tombstones_exist_only_after_successful_terminal_full_snapshot() {
    let active = vec![id(3), id(1), id(2)];
    let seen = vec![id(2)];
    assert!(
        terminal_snapshot_tombstones_v1(false, &active, &seen)
            .expect("partial")
            .is_empty()
    );
    assert_eq!(
        terminal_snapshot_tombstones_v1(true, &active, &seen).expect("terminal"),
        vec![id(1), id(3)]
    );
    assert!(terminal_snapshot_tombstones_v1(true, &[id(1), id(1)], &seen).is_err());
}

#[test]
fn terminal_snapshot_has_no_alternate_non_outboxed_repository_bypass() {
    let repository = include_str!(
        "../../../../src/mail-address-book-persistence/src/person_source_repository.rs"
    );
    assert!(repository.contains("pub async fn commit_person_source_snapshot_once"));
    assert!(!repository.contains("pub async fn complete_person_source_snapshot"));
}

#[test]
fn additive_mail_schema_is_private_dormant_and_forward_only() {
    assert_eq!(
        MAIL_ADDRESS_BOOK_STORAGE_BUNDLE_REVISION_V1, 28,
        "admitted Mail bundle stays unchanged"
    );
    assert_eq!(
        MAIL_ADDRESS_BOOK_PERSON_SOURCE_STORAGE_BUNDLE_REVISION_V1,
        29
    );
    let sql = std::str::from_utf8(MAIL_ADDRESS_BOOK_PERSON_SOURCE_SCHEMA_V1)
        .expect("utf8")
        .to_lowercase();
    for table in [
        "mail_address_book_person_source_accounts",
        "mail_address_book_person_sources",
        "mail_address_book_person_source_runs",
        "mail_address_book_person_source_seen",
        "mail_address_book_person_source_fetch_inbox",
        "mail_address_book_person_source_fetch_outbox",
    ] {
        assert!(sql.contains(table), "{table}");
    }
    assert!(sql.contains("force row level security"));
    assert!(!sql.contains("mail_contacts_sync_"));
}

#[test]
fn random_source_ids_are_owner_account_and_private_record_scoped() {
    let first = derive_public_source_contact_id_v1("owner-a", id(1), b"private-record-a", [7; 32])
        .expect("source public ID");
    assert_eq!(
        first,
        derive_public_source_contact_id_v1("owner-a", id(1), b"private-record-a", [7; 32])
            .expect("stable test issuer"),
    );
    assert_ne!(
        first,
        derive_public_source_contact_id_v1("owner-a", id(2), b"private-record-a", [7; 32])
            .expect("account isolation"),
    );
    assert_ne!(
        first,
        derive_public_source_contact_id_v1("owner-b", id(1), b"private-record-a", [7; 32])
            .expect("owner isolation"),
    );
    assert!(derive_public_source_contact_id_v1("owner-a", id(1), b"", [7; 32]).is_err());
}

#[test]
fn synthetic_removals_are_sorted_and_bounded_to_500_per_page() {
    let mut removals = (1_u16..=1_201)
        .rev()
        .map(|value| {
            let mut id = [0_u8; 16];
            id[..2].copy_from_slice(&value.to_be_bytes());
            id
        })
        .collect::<Vec<_>>();
    let pages = plan_synthetic_removal_pages_v1(id(9), 7, &removals).expect("removal pages");
    assert_eq!(
        pages
            .iter()
            .map(|page| page.source_ids.len())
            .collect::<Vec<_>>(),
        vec![500, 500, 201]
    );
    assert_eq!(
        pages
            .iter()
            .map(|page| page.page_sequence)
            .collect::<Vec<_>>(),
        vec![7, 8, 9]
    );
    assert_eq!(
        pages,
        plan_synthetic_removal_pages_v1(id(9), 7, &removals).expect("deterministic replay"),
    );
    assert!(
        pages
            .iter()
            .all(|page| page.source_ids.windows(2).all(|pair| pair[0] < pair[1]))
    );
    removals.push(removals[0]);
    assert!(plan_synthetic_removal_pages_v1(id(9), 7, &removals).is_err());
}

#[test]
fn dormant_producer_emits_only_public_new_update_and_bounded_removal_records() {
    let source = MailPersonSourceIdentityV1 {
        integration_public_id: id(1).to_vec(),
        account_public_id: id(2).to_vec(),
        provider_source_contact_public_id: id(3).to_vec(),
    };
    let claims = MailPersonSourceClaimsV1 {
        display_name: Some("Public Person".to_owned()),
        normalized_emails: vec!["public@example.test".to_owned()],
        normalized_phones: Vec::new(),
    };
    let context = MailAddressBookEnvelopeContextV1 {
        module_id: "makosh-mail-runtime".to_owned(),
        runtime_instance_id: "dormant-producer-test".to_owned(),
        runtime_generation: 1,
        recorded_at_unix_seconds: 1_800_000_000,
        recorded_at_nanos: 0,
    };
    let input = MailPersonSourcePublicChangeInputV1 {
        command_message_id: mail_person_source_fetch_id_v1(id(6), 1),
        observation_id: id(5),
        run_id: id(6),
        logical_owner_id: "owner-a".to_owned(),
        page_sequence: 1,
        source: source.clone(),
        claims: claims.clone(),
        source_revision: 1,
        observed_at: Timestamp {
            seconds: 1_800_000_000,
            nanos: 0,
        },
        context: context.clone(),
    };
    assert!(
        build_public_source_change_v1(&input, MailPersonSourcePublicChangeV1::Unchanged)
            .expect("unchanged")
            .is_none()
    );
    assert!(
        build_public_source_change_v1(&input, MailPersonSourcePublicChangeV1::Observed)
            .expect("observed")
            .is_some()
    );
    assert!(
        build_public_source_change_v1(
            &MailPersonSourcePublicChangeInputV1 {
                source_revision: 2,
                ..input
            },
            MailPersonSourcePublicChangeV1::Updated
        )
        .expect("updated")
        .is_some()
    );
    let removals = (0_u16..500)
        .map(|index| {
            let mut public_id = [0_u8; 16];
            public_id[..2].copy_from_slice(&(index + 1).to_be_bytes());
            MailPersonSourceSyntheticRemovalV1 {
                integration_public_id: id(1),
                account_public_id: id(2),
                provider_source_contact_public_id: public_id,
                source_revision: 2,
            }
        })
        .collect::<Vec<_>>();
    let page = build_synthetic_removal_page_v1("owner-a", id(6), 2, &removals, false, &context)
        .expect("bounded synthetic removal page");
    assert_eq!(page.source_records.len(), 500);
    assert_eq!(page.source_records.len() + 1, page.all_records().len());
    let exact = page
        .all_records()
        .into_iter()
        .flat_map(|record| record.exact_bytes())
        .copied()
        .collect::<Vec<_>>();
    for forbidden in [
        b"provider_record_key".as_slice(),
        b"provider_record_etag",
        b"private-account",
        b"credential",
    ] {
        assert!(
            !exact
                .windows(forbidden.len())
                .any(|window| window == forbidden)
        );
    }
}
