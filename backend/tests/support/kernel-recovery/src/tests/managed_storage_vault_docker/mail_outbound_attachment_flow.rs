//! Live event-gated Mail attachment delivery through Communications and Blob custody.

use std::time::{Duration, Instant};

use super::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use makosh_communications_api::{
    AttachmentSafetyStateV1 as CanonicalAttachmentSafetyStateV1,
    AttachmentSafetyTransitionDecisionV1, CommunicationAttachmentAnchorIdV1,
    CommunicationObservationIdV1,
};
use makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1;
use makosh_communications_runtime::canonical_outbox::{
    CanonicalEventContextV1, build_attachment_safety_state_changed_outbox_v1,
};
use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailSendMailRequestV1,
    client_contract::MailClientContractV1,
};
use makosh_mail_core::rfc822::{attachment_metadata, extract_attachment_part};
use makosh_mail_persistence::{
    MailAttachmentSafetyStateV1 as PersistedAttachmentSafetyStateV1, MailDurablePersistenceError,
};
use makosh_mail_runtime::attachment_safety_projection::{
    MailAttachmentSafetyProjectionErrorV1, project_attachment_safety_state_changed_v1,
};

use super::attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture;
use crate::identity::device::signer::DeviceSigner;
const OPERATION_ID: &str = "managed-mail-smtp-attachment-delivery-1";
const PRIVATE_BODY: &str = "private attachment delivery body";
const PRIVATE_RECIPIENT: &str = "attachment-recipient@example.test";
const ATTACHMENT_BYTES: &[u8] = b"clean-room-attachment";
const AMBIGUOUS_OPERATION_ID: &str = "managed-mail-smtp-attachment-ambiguous-1";
const GMAIL_OPERATION_ID: &str = "managed-mail-gmail-attachment-delivery-1";
const GMAIL_ACCESS_TOKEN: &str = "managed-mail-gmail-access-token";
const GMAIL_ATTACHMENT_BYTES: &[u8] = b"gmail-clean-room-attachment";

#[test]
#[ignore = "requires disposable authenticated PostgreSQL, NATS, PgBouncer and live managed runtimes"]
fn managed_mail_delivers_only_canonical_safe_attachment_from_its_blob_custody() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let imap = MailImapFixture::start();
    let smtp = MailSmtpFixture::start();
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("makosh-managed-mail-outbound-attachment");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_mail_vault(&vault_dir);
    let release = installed_communications_mail_attachment_security_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim initial owner");
    let admitted_mail = admit_mail_attachment_delivery_runtime(&store);
    let admitted_attachment_security = admit_attachment_security_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start signed Blob runtime"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    let admitted_mail = prepare_mail_runtime(&supervisor, &store, admitted_mail);
    let admitted_attachment_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_attachment_security);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mut mail = start_mail_delivery_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        imap.port(),
        MailSmtpFixtureSettingsV1 {
            port: smtp.port(),
            ca_certificate_pem: smtp.ca_certificate_pem().to_owned(),
        },
    );
    wait_for_mail_ready(&supervisor, &mail);

    let unknown_attachment = MailClientRequestV1::SendMail(MailSendMailRequestV1 {
        operation_id: "managed-mail-unknown-attachment-rejection".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        provider_conversation_id: "smtp-attachment-thread".to_owned(),
        recipients: vec![PRIVATE_RECIPIENT.to_owned()],
        cc_recipients: Vec::new(),
        bcc_recipients: Vec::new(),
        subject: "unknown attachment must fail closed".to_owned(),
        text_body: PRIVATE_BODY.to_owned(),
        attachment_anchor_ids: vec![[0x7f; 16]],
    });
    let error = route_mail_client_once(
        &store,
        &supervisor,
        &mail,
        MailClientContractV1::Delivery,
        69,
        &unknown_attachment,
    )
    .expect_err("unknown attachment must be rejected");
    assert_eq!(error, "Mail route runtime error");
    assert_eq!(smtp.accepted_messages(), 0);
    assert!(
        supervisor
            .is_active(&mail.registration_id)
            .expect("read Mail after rejected attachment"),
        "an unsafe application command must not stop the Mail runtime"
    );

    let attachment_anchor_id = assert_mail_attachment_lifecycle(&store, &supervisor, &mail);
    let attachment_security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_attachment_security,
        clamav.port(),
    );
    assert!(
        supervisor
            .is_active(&attachment_security.registration_id)
            .expect("read Attachment Security process state")
    );
    wait_for_attachment_state_value(
        &store,
        &supervisor,
        attachment_anchor_id,
        AttachmentSafetyStateV1::SafeForDelivery as u32,
    );
    wait_for_mail_attachment_state_value(
        attachment_anchor_id,
        PersistedAttachmentSafetyStateV1::SafeForDelivery,
    );
    assert_mail_rejects_stale_and_unknown_safety_events(attachment_anchor_id);

    let delivery = MailClientRequestV1::SendMail(MailSendMailRequestV1 {
        operation_id: OPERATION_ID.to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        provider_conversation_id: "smtp-attachment-thread".to_owned(),
        recipients: vec![PRIVATE_RECIPIENT.to_owned()],
        cc_recipients: Vec::new(),
        bcc_recipients: Vec::new(),
        subject: "managed SMTP attachment delivery".to_owned(),
        text_body: PRIVATE_BODY.to_owned(),
        attachment_anchor_ids: vec![attachment_anchor_id],
    });
    let response = route_mail_client(
        &store,
        &supervisor,
        &mail,
        MailClientContractV1::Delivery,
        70,
        &delivery,
    );
    assert_eq!(
        response,
        MailClientResponseV1::MailAccepted {
            operation_id: OPERATION_ID.to_owned(),
        }
    );
    assert_delivery_completed(&store, &supervisor, &mail, OPERATION_ID, 250);
    assert_eq!(smtp.accepted_messages(), 1);

    let message = smtp.last_message();
    let metadata = attachment_metadata(&message);
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].filename.as_deref(), Some("evidence.pdf"));
    assert_eq!(metadata[0].media_type, "application/pdf");
    assert_eq!(
        extract_attachment_part(&message, metadata[0].part_id),
        Ok(ATTACHMENT_BYTES.to_vec())
    );
    assert_eq!(
        route_mail_client(
            &store,
            &supervisor,
            &mail,
            MailClientContractV1::Delivery,
            71,
            &delivery,
        ),
        MailClientResponseV1::MailAccepted {
            operation_id: OPERATION_ID.to_owned(),
        }
    );
    std::thread::sleep(Duration::from_millis(1_200));
    assert_eq!(
        smtp.accepted_messages(),
        1,
        "an exact attachment command replay must not read Blob or execute SMTP twice"
    );

    mail = restart_mail_delivery_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        mail,
        imap.port(),
        MailSmtpFixtureSettingsV1 {
            port: smtp.port(),
            ca_certificate_pem: smtp.ca_certificate_pem().to_owned(),
        },
    );
    wait_for_mail_ready(&supervisor, &mail);
    assert_eq!(
        route_mail_client(
            &store,
            &supervisor,
            &mail,
            MailClientContractV1::Delivery,
            74,
            &delivery,
        ),
        MailClientResponseV1::MailAccepted {
            operation_id: OPERATION_ID.to_owned(),
        }
    );
    std::thread::sleep(Duration::from_millis(1_200));
    assert_eq!(
        smtp.accepted_messages(),
        1,
        "runtime restart must retain the exact terminal delivery and Blob-read authority"
    );

    let ambiguous_smtp = MailSmtpFixture::start_outcome_unknown();
    assert!(ambiguous_smtp.disconnects_after_data());
    mail = restart_mail_delivery_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        mail,
        imap.port(),
        MailSmtpFixtureSettingsV1 {
            port: ambiguous_smtp.port(),
            ca_certificate_pem: ambiguous_smtp.ca_certificate_pem().to_owned(),
        },
    );
    wait_for_mail_ready(&supervisor, &mail);
    let ambiguous_delivery = MailClientRequestV1::SendMail(MailSendMailRequestV1 {
        operation_id: AMBIGUOUS_OPERATION_ID.to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        provider_conversation_id: "smtp-attachment-thread".to_owned(),
        recipients: vec![PRIVATE_RECIPIENT.to_owned()],
        cc_recipients: Vec::new(),
        bcc_recipients: Vec::new(),
        subject: "ambiguous SMTP attachment delivery".to_owned(),
        text_body: PRIVATE_BODY.to_owned(),
        attachment_anchor_ids: vec![attachment_anchor_id],
    });
    assert_eq!(
        route_mail_client(
            &store,
            &supervisor,
            &mail,
            MailClientContractV1::Delivery,
            75,
            &ambiguous_delivery,
        ),
        MailClientResponseV1::MailAccepted {
            operation_id: AMBIGUOUS_OPERATION_ID.to_owned(),
        }
    );
    assert_delivery_outcome_unknown(&store, &supervisor, &mail, AMBIGUOUS_OPERATION_ID);
    assert_eq!(ambiguous_smtp.accepted_messages(), 1);
    assert_eq!(
        route_mail_client(
            &store,
            &supervisor,
            &mail,
            MailClientContractV1::Delivery,
            76,
            &ambiguous_delivery,
        ),
        MailClientResponseV1::MailAccepted {
            operation_id: AMBIGUOUS_OPERATION_ID.to_owned(),
        }
    );
    std::thread::sleep(Duration::from_millis(1_200));
    assert_eq!(
        ambiguous_smtp.accepted_messages(),
        1,
        "provider ambiguity must remain terminal across exact command replay"
    );

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Mail attachment fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

#[test]
#[ignore = "requires disposable authenticated PostgreSQL, NATS, PgBouncer and live managed runtimes"]
fn managed_gmail_materializes_then_delivers_canonical_safe_attachment() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let gmail = MailGmailFixture::start();
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("makosh-managed-gmail-outbound-attachment");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    let seeded_gmail = seed_mail_vault(&vault_dir);
    let release = installed_communications_mail_attachment_security_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim initial owner");
    let admitted_mail = admit_mail_gmail_attachment_delivery_runtime(&store);
    let admitted_attachment_security = admit_attachment_security_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start signed Blob runtime"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    let admitted_mail = prepare_mail_runtime(&supervisor, &store, admitted_mail);
    let admitted_attachment_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_attachment_security);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mail = start_mail_gmail_delivery_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        MailGmailFixtureSettingsV1 {
            port: gmail.port(),
            ca_certificate_pem: gmail.ca_certificate_pem().to_owned(),
            oauth: None,
        },
    );
    wait_for_mail_ready(&supervisor, &mail);
    let runtime = tokio::runtime::Runtime::new().expect("Gmail attachment persistence runtime");
    let durable = runtime.block_on(connect_postgres());
    runtime
        .block_on(durable.initialize())
        .expect("initialize Mail persistence");
    runtime
        .block_on(durable.store_gmail_oauth_credential_binding(
            MAIL_ACCOUNT_ID,
            &seeded_gmail.binding(),
            1,
        ))
        .expect("store Mail-owned Gmail credential binding");

    let attachment_anchor_id = assert_mail_attachment_lifecycle(&store, &supervisor, &mail);
    let accepted_reads = gmail.accepted_reads();
    assert_mail_sync_replay_and_health(
        &store,
        &supervisor,
        &mail,
        "managed-mail-attachment-replay",
        1,
        190,
    );
    assert_eq!(
        gmail.accepted_reads(),
        accepted_reads,
        "an exact replayed Gmail sync operation must not reach the provider twice"
    );
    assert!(
        gmail.accepted_reads() >= 4,
        "Gmail materialization must use bounded list/history and exact raw-message reads"
    );
    let attachment_security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_attachment_security,
        clamav.port(),
    );
    assert!(
        supervisor
            .is_active(&attachment_security.registration_id)
            .expect("read Attachment Security process state")
    );
    wait_for_attachment_state_value(
        &store,
        &supervisor,
        attachment_anchor_id,
        AttachmentSafetyStateV1::SafeForDelivery as u32,
    );
    wait_for_mail_attachment_state_value(
        attachment_anchor_id,
        PersistedAttachmentSafetyStateV1::SafeForDelivery,
    );

    let response = route_mail_client(
        &store,
        &supervisor,
        &mail,
        MailClientContractV1::Delivery,
        80,
        &MailClientRequestV1::SendMail(MailSendMailRequestV1 {
            operation_id: GMAIL_OPERATION_ID.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            provider_conversation_id: "gmail-attachment-thread".to_owned(),
            recipients: vec![PRIVATE_RECIPIENT.to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "managed Gmail attachment delivery".to_owned(),
            text_body: PRIVATE_BODY.to_owned(),
            attachment_anchor_ids: vec![attachment_anchor_id],
        }),
    );
    assert_eq!(
        response,
        MailClientResponseV1::MailAccepted {
            operation_id: GMAIL_OPERATION_ID.to_owned(),
        }
    );
    assert_delivery_completed(&store, &supervisor, &mail, GMAIL_OPERATION_ID, 200);
    assert_eq!(gmail.accepted_mutations(), 1);
    let request = gmail.last_request();
    assert_eq!(
        request.authorization,
        format!("Bearer {GMAIL_ACCESS_TOKEN}")
    );
    let message = URL_SAFE_NO_PAD
        .decode(request.raw)
        .expect("decode Gmail outbound RFC822");
    let metadata = attachment_metadata(&message);
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].filename.as_deref(), Some("gmail-evidence.txt"));
    assert_eq!(metadata[0].media_type, "text/plain");
    assert_eq!(
        extract_attachment_part(&message, metadata[0].part_id),
        Ok(GMAIL_ATTACHMENT_BYTES.to_vec())
    );

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Gmail attachment fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn wait_for_attachment_state_value(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    attachment_anchor_id: [u8; 16],
    expected_state: u32,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if wait_for_attachment_state(store, supervisor, attachment_anchor_id) == expected_state {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Communications attachment did not reach the expected safety state"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_mail_attachment_state_value(
    attachment_anchor_id: [u8; 16],
    expected_state: PersistedAttachmentSafetyStateV1,
) {
    let runtime = tokio::runtime::Runtime::new().expect("Mail safety projection test runtime");
    let durable = runtime.block_on(connect_postgres());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if runtime
            .block_on(durable.attachment_safety_state(attachment_anchor_id))
            .expect("read Mail attachment safety projection")
            == Some(expected_state)
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Mail did not project the canonical attachment safety state"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_mail_rejects_stale_and_unknown_safety_events(attachment_anchor_id: [u8; 16]) {
    let runtime = tokio::runtime::Runtime::new().expect("Mail safety rejection runtime");
    let durable = runtime.block_on(connect_postgres());
    let context = CanonicalEventContextV1 {
        runtime_instance_id: "communications-runtime-mail-safety-negative".to_owned(),
        runtime_generation: 1,
        recorded_at_unix_seconds: 1_783_110_100,
        recorded_at_nanos: 0,
    };
    let stale = build_attachment_safety_state_changed_outbox_v1(
        AttachmentSafetyTransitionDecisionV1 {
            attachment_anchor_id: CommunicationAttachmentAnchorIdV1::new(attachment_anchor_id),
            expected_state: CanonicalAttachmentSafetyStateV1::BlobAdmitted,
            next_state: CanonicalAttachmentSafetyStateV1::Quarantined,
            evidence_id: CommunicationObservationIdV1::new([0x31; 16]),
            observed_at_unix_seconds: 1_783_110_100,
        },
        [0x32; 16],
        [0x33; 16],
        &context,
    )
    .expect("build stale canonical safety event");
    assert_eq!(
        runtime.block_on(project_attachment_safety_state_changed_v1(
            &durable,
            stale.exact_bytes(),
            1_783_110_101,
        )),
        Err(MailAttachmentSafetyProjectionErrorV1::Persistence(
            MailDurablePersistenceError::ConflictingAttachmentSafetyProjection,
        ))
    );

    let unknown = build_attachment_safety_state_changed_outbox_v1(
        AttachmentSafetyTransitionDecisionV1 {
            attachment_anchor_id: CommunicationAttachmentAnchorIdV1::new([0x41; 16]),
            expected_state: CanonicalAttachmentSafetyStateV1::DescriptorOnly,
            next_state: CanonicalAttachmentSafetyStateV1::BlobPending,
            evidence_id: CommunicationObservationIdV1::new([0x42; 16]),
            observed_at_unix_seconds: 1_783_110_102,
        },
        [0x43; 16],
        [0x44; 16],
        &context,
    )
    .expect("build unknown-anchor canonical safety event");
    assert_eq!(
        runtime.block_on(project_attachment_safety_state_changed_v1(
            &durable,
            unknown.exact_bytes(),
            1_783_110_103,
        )),
        Err(MailAttachmentSafetyProjectionErrorV1::Persistence(
            MailDurablePersistenceError::MissingSourceObservation,
        ))
    );
    assert_eq!(
        runtime
            .block_on(durable.attachment_safety_state(attachment_anchor_id))
            .expect("read Mail safety state after rejected events"),
        Some(PersistedAttachmentSafetyStateV1::SafeForDelivery)
    );
}
