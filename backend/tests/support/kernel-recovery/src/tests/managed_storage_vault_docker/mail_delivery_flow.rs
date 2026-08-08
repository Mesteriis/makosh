//! Live accepted-versus-terminal Mail delivery with SMTP and event-only handoff.

use std::time::{Duration, Instant};

use makosh_events_protocol::validation::envelope::decode_envelope_v1;
use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailSendMailRequestV1,
    client_contract::MailClientContractV1,
};

use crate::identity::device::signer::DeviceSigner;

use super::*;

const OPERATION_ID: &str = "managed-mail-smtp-delivery-1";
const PRIVATE_BODY: &str = "private SMTP body must stay outside durable route metadata";
const PRIVATE_RECIPIENT: &str = "recipient@example.test";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Mail, SMTP and NATS"]
fn managed_mail_runtime_accepts_then_completes_smtp_delivery_and_replays_event() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let imap = MailImapFixture::start();
    let smtp = MailSmtpFixture::start();
    let root = unique_target_root("makosh-managed-mail-smtp-delivery");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_mail_vault(&vault_dir);
    let release = installed_communications_mail_release(&root);
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
    let admitted_mail = admit_mail_delivery_runtime(&store);
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
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mail = start_mail_delivery_runtime(
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

    let event_runtime = tokio::runtime::Runtime::new().expect("Mail delivery event runtime");
    let _event_runtime_context = event_runtime.enter();
    let durable = event_runtime.block_on(connect_postgres());
    event_runtime
        .block_on(durable.initialize())
        .expect("initialize Mail persistence");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let (client, mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect Mail delivery observer");
        let observations = client
            .subscribe(MAIL_DELIVERY_OBSERVATION_SUBJECT)
            .await
            .expect("subscribe Mail delivery observations");
        let canonical_events = client
            .subscribe(MAIL_DELIVERY_CANONICAL_EVENT_SUBJECT)
            .await
            .expect("subscribe Communications canonical events");
        client
            .flush()
            .await
            .expect("activate Mail delivery observers");
        (client, observations, canonical_events)
    });

    set_authenticated_nats_container_running(false);
    assert_delivery_accepted(&store, &supervisor, &mail);
    assert_delivery_completed(&store, &supervisor, &mail, OPERATION_ID, 250);
    assert_eq!(smtp.accepted_messages(), 1);
    let message = smtp.last_message();
    assert!(
        message
            .windows(PRIVATE_BODY.len())
            .any(|bytes| bytes == PRIVATE_BODY.as_bytes())
    );
    assert!(
        message
            .windows(PRIVATE_RECIPIENT.len())
            .any(|bytes| { bytes == PRIVATE_RECIPIENT.as_bytes() })
    );

    assert_delivery_accepted(&store, &supervisor, &mail);
    std::thread::sleep(Duration::from_millis(1_200));
    assert_eq!(
        smtp.accepted_messages(),
        1,
        "an exact idempotent command replay must not execute SMTP twice"
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let pending = event_runtime
            .block_on(durable.pending_communications_outbox(4))
            .expect("read Mail delivery outbox");
        if !pending.is_empty() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "accepted SMTP delivery did not persist its neutral observation"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(
        supervisor
            .is_active(&mail.registration_id)
            .expect("read managed Mail state"),
        "NATS outage must not stop Mail after provider completion"
    );

    set_authenticated_nats_container_running(true);
    wait_for_authenticated_nats_reconnect(&event_runtime, &client, "Mail SMTP observer");
    let (observation, canonical) = event_runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .expect("Mail SMTP observation timeout")
            .expect("Mail SMTP observation");
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .expect("Mail SMTP canonical event timeout")
            .expect("Mail SMTP canonical event");
        (observation, canonical)
    });
    let observation_bytes = observation.payload.to_vec();
    assert!(
        !observation_bytes
            .windows(PRIVATE_BODY.len())
            .any(|bytes| bytes == PRIVATE_BODY.as_bytes()),
        "Mail delivery observation must not contain the provider message body"
    );
    assert!(
        !observation_bytes
            .windows(PRIVATE_RECIPIENT.len())
            .any(|bytes| bytes == PRIVATE_RECIPIENT.as_bytes()),
        "Mail delivery observation must not contain the provider recipient"
    );
    let observation =
        decode_envelope_v1(&observation_bytes).expect("Mail SMTP durable observation");
    let canonical =
        decode_envelope_v1(canonical.payload.as_ref()).expect("Communications durable event");
    assert_eq!(canonical.causation_message_id, observation.message_id);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Mail SMTP fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn assert_delivery_accepted(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let response = route_mail_client(
        store,
        supervisor,
        mail,
        MailClientContractV1::Delivery,
        71,
        &MailClientRequestV1::SendMail(MailSendMailRequestV1 {
            operation_id: OPERATION_ID.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            provider_conversation_id: "smtp-conversation-1".to_owned(),
            recipients: vec![PRIVATE_RECIPIENT.to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "managed SMTP delivery".to_owned(),
            text_body: PRIVATE_BODY.to_owned(),
            attachment_anchor_ids: Vec::new(),
        }),
    );
    assert_eq!(
        response,
        MailClientResponseV1::MailAccepted {
            operation_id: OPERATION_ID.to_owned(),
        },
        "provider acceptance must not be folded into the command receipt"
    );
}
