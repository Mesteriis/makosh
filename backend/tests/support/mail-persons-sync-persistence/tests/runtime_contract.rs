use makosh_events_protocol::validation::envelope::decode_envelope_v1;
use makosh_mail_address_book_contract::{
    mail_person_source_claims_digest_v1, mail_person_source_tombstone_digest_v1,
    wire_person_source::{
        MailPersonSourceClaimsV1, MailPersonSourceIdentityV1, MailPersonSourceObservedV1,
        MailPersonSourceProvenanceV1, MailPersonSourceRemovedV1, MailPersonSourceUpdatedV1,
    },
};
use makosh_mail_persons_sync_runtime::{
    MailPersonSourceInputV1, MailPersonsSyncEnvelopeContextV1,
    build_persons_command_outbox_record_v1, dispatch_mail_person_source_v1,
    mail_persons_sync_module_descriptor_v1, mail_persons_sync_settings_schema_v1,
};
use makosh_persons_api::wire::persons_command_v1::Command;
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use sha2::{Digest, Sha256};

fn source() -> MailPersonSourceIdentityV1 {
    MailPersonSourceIdentityV1 {
        integration_public_id: vec![1; 16],
        account_public_id: vec![2; 16],
        provider_source_contact_public_id: vec![3; 16],
    }
}
fn claims() -> MailPersonSourceClaimsV1 {
    MailPersonSourceClaimsV1 {
        display_name: Some("Ada".to_owned()),
        normalized_emails: vec!["ada@example.test".to_owned()],
        normalized_phones: Vec::new(),
    }
}
fn provenance(digest: [u8; 32], revision: u64) -> MailPersonSourceProvenanceV1 {
    MailPersonSourceProvenanceV1 {
        source_revision: revision,
        source_digest: digest.to_vec(),
        observed_at: Some(prost_types::Timestamp {
            seconds: 10,
            nanos: 0,
        }),
    }
}

#[test]
fn descriptor_is_private_exact_and_has_no_client_surface() {
    let descriptor = mail_persons_sync_module_descriptor_v1("test");
    validate_descriptor_v1(&descriptor).expect("descriptor");
    validate_settings_schema_v1(&mail_persons_sync_settings_schema_v1()).expect("settings");
    assert_eq!(descriptor.owner_id, "mail_persons_sync");
    assert_eq!(descriptor.module_id, "makosh-mail-persons-sync-runtime");
    let ids = descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "mail_persons_sync.mail.account-ready.v1",
            "mail_persons_sync.mail.account-retired.v1",
            "mail_persons_sync.mail.fetch-page.v1",
            "mail_persons_sync.mail.page-completed.v1",
            "mail_persons_sync.mail.page-rejected.v1",
            "mail_persons_sync.mail.source-observed.v1",
            "mail_persons_sync.mail.source-removed.v1",
            "mail_persons_sync.mail.source-updated.v1",
            "mail_persons_sync.page-receipt.v1",
            "mail_persons_sync.persons.command-rejected.v1",
            "mail_persons_sync.persons.command-succeeded.v1",
            "mail_persons_sync.persons.command.v1",
            "mail_persons_sync.run-result.v1",
            "mail_persons_sync.scheduler.receipt.v1",
            "mail_persons_sync.scheduler.v1",
            "mail_persons_sync.scheduler_schedule_command.v1",
            "mail_persons_sync.scheduler_schedule_result.v1",
            "mail_persons_sync.storage.v1",
        ]
    );
    assert!(
        descriptor
            .capabilities
            .iter()
            .flat_map(|capability| &capability.provides)
            .all(
                |surface| surface.client_rpc_route.is_none() && surface.client_blob_route.is_none()
            )
    );
}

#[test]
fn all_three_sanitized_source_variants_dispatch_to_existing_persons_commands() {
    let source = source();
    let claims = claims();
    let observed = MailPersonSourceObservedV1 {
        observation_id: vec![4; 16],
        run_id: vec![5; 16],
        logical_owner_id: "owner-1".to_owned(),
        page_sequence: 1,
        source: Some(source.clone()),
        claims: Some(claims.clone()),
        provenance: Some(provenance(
            mail_person_source_claims_digest_v1(&source, &claims).expect("digest"),
            1,
        )),
    };
    let observe_command =
        dispatch_mail_person_source_v1(MailPersonSourceInputV1::Observed(observed))
            .expect("observe");
    assert!(matches!(
        observe_command.command,
        Some(Command::SourceObserve(_))
    ));
    let record = build_persons_command_outbox_record_v1(
        observe_command,
        20,
        &MailPersonsSyncEnvelopeContextV1 {
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 2,
            recorded_at_unix_seconds: 10,
            recorded_at_nanos: 0,
        },
    )
    .expect("outbox");
    let envelope = decode_envelope_v1(record.exact_bytes()).expect("exact envelope");
    assert_eq!(envelope.contract.expect("contract").owner, "persons");
    assert_eq!(
        envelope.source.expect("source").module_id,
        "makosh-mail-persons-sync-runtime"
    );
    let mut canonical_partition = Sha256::new();
    canonical_partition.update(b"persons-owner-partition-v1");
    canonical_partition.update(("owner-1".len() as u64).to_be_bytes());
    canonical_partition.update(b"owner-1");
    canonical_partition.update(("persons".len() as u64).to_be_bytes());
    canonical_partition.update(b"persons");
    assert_eq!(envelope.partition_key, canonical_partition.finalize()[..16]);
    let updated = MailPersonSourceUpdatedV1 {
        observation_id: vec![6; 16],
        run_id: vec![5; 16],
        logical_owner_id: "owner-1".to_owned(),
        page_sequence: 1,
        source: Some(source.clone()),
        claims: Some(claims.clone()),
        provenance: Some(provenance(
            mail_person_source_claims_digest_v1(&source, &claims).expect("digest"),
            2,
        )),
    };
    assert!(matches!(
        dispatch_mail_person_source_v1(MailPersonSourceInputV1::Updated(updated))
            .expect("update")
            .command,
        Some(Command::SourceUpdate(_))
    ));
    let removed = MailPersonSourceRemovedV1 {
        observation_id: vec![7; 16],
        run_id: vec![5; 16],
        logical_owner_id: "owner-1".to_owned(),
        page_sequence: 1,
        source: Some(source.clone()),
        provenance: Some(provenance(
            mail_person_source_tombstone_digest_v1(&source).expect("tombstone"),
            3,
        )),
    };
    assert!(matches!(
        dispatch_mail_person_source_v1(MailPersonSourceInputV1::Removed(removed))
            .expect("remove")
            .command,
        Some(Command::SourceRemove(_))
    ));
}
