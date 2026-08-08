//! Live signed admission smoke gate for retained Preview evidence replay.

use super::*;

use super::{
    attachment_preview_evidence_replay_persistence_fixture::{
        remove_retained_mail_replay_index_v1, restore_retained_mail_replay_index_v1,
        wait_for_retained_preview_evidence_indexes_v1,
        wait_for_retained_preview_replay_terminal_v1,
    },
    attachment_preview_gateway_fixture::{
        attachment_preview_gateway_v1, get_attachment_preview_v1, post_attachment_preview_proto_v1,
        read_attachment_preview_blob_v1, read_terminal_attachment_preview_sse_event_v1,
        wait_for_ready_attachment_preview_v1,
    },
    attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture,
    mail_attachment_flow::assert_mail_attachment_lifecycle,
};

use crate::identity::device::signer::DeviceSigner;
use hyper::StatusCode;
use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1, ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
    wire::{
        AttachmentPreviewErrorCodeV1, AttachmentPreviewStateV1,
        IssueAttachmentPreviewReadRequestV1, IssueAttachmentPreviewReadResponseV1,
        StartAttachmentPreviewRequestV1, StartAttachmentPreviewResponseV1,
    },
};
use makosh_attachment_preview_evidence_replay_api::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONNECT_PATH_V1,
    wire::{
        AttachmentPreviewEvidenceReplayErrorV1, AttachmentPreviewEvidenceReplayStateV1,
        StartAttachmentPreviewEvidenceReplayRequestV1,
        StartAttachmentPreviewEvidenceReplayResponseV1,
    },
};

const SAFETY_STATE_SUBJECT_V1: &str =
    "makosh.event.v1.communications.communication_attachment_safety_state_changed.v1";
const SCAN_CANDIDATE_SUBJECT_V1: &str = "makosh.observation.v1.attachment_security.\
    attachment_security_scan_candidate_observed.v1";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and replay workflow binaries"]
fn managed_attachment_preview_evidence_replay_runtime_starts_with_exact_signed_contracts() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-attachment-preview-evidence-replay");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_attachment_preview_replay_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim retained Preview evidence replay logical owner");

    let _admitted_mail = admit_mail_runtime(&store);
    let admitted_replay = admit_attachment_preview_evidence_replay_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
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
    let admitted_replay =
        prepare_attachment_preview_evidence_replay_runtime_v1(&supervisor, &store, admitted_replay);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let replay = start_attachment_preview_evidence_replay_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_replay,
    );
    assert!(
        supervisor
            .is_active(&replay.registration_id)
            .expect("read retained Preview evidence replay process state")
    );
    assert_eq!(replay.runtime_generation, 1);
    assert!(replay.grant_epoch > 0);
    assert!(!replay.runtime_instance_id.is_empty());

    let previous_runtime_instance_id = replay.runtime_instance_id.clone();
    supervisor
        .stop(&replay.registration_id)
        .expect("stop retained Preview evidence replay predecessor");
    let replay = restart_attachment_preview_evidence_replay_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        replay,
    );
    assert_eq!(replay.runtime_generation, 2);
    assert_ne!(replay.runtime_instance_id, previous_runtime_instance_id);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove retained Preview evidence replay fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

#[test]
#[ignore = "requires disposable Docker plus the complete retained Preview evidence replay managed ensemble"]
fn managed_attachment_preview_evidence_replay_restores_expired_sources_to_browser_preview() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let imap = MailImapFixture::start();
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("makosh-managed-attachment-preview-evidence-recovery");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_mail_vault(&vault_dir);
    let release = installed_attachment_preview_replay_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim retained Preview recovery logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
    );

    let admitted_mail = admit_mail_runtime(&store);
    let admitted_security = admit_attachment_security_runtime(&store);
    let admitted_preview = admit_attachment_preview_runtime_v1(&store);
    let admitted_replay = admit_attachment_preview_evidence_replay_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_attachment_preview_realtime_v1(&supervisor, &store, realtime.clone());
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
    let admitted_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_security);
    let admitted_preview =
        prepare_attachment_preview_runtime_v1(&supervisor, &store, admitted_preview);
    let admitted_replay =
        prepare_attachment_preview_evidence_replay_runtime_v1(&supervisor, &store, admitted_replay);
    configure_communications_jetstream_for_retained_replay_test(&store);
    let _communications_generation =
        start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mail = start_mail_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        imap.port(),
    );
    let mut replay_runtime = start_attachment_preview_evidence_replay_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_replay,
    );

    let attachment_anchor_id = assert_mail_attachment_lifecycle(&store, &supervisor, &mail);
    let _security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_security,
        clamav.port(),
    );
    wait_for_retained_preview_attachment_state_v1(
        &store,
        &supervisor,
        attachment_anchor_id,
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32,
    );
    wait_for_retained_preview_evidence_indexes_v1(attachment_anchor_id);
    wait_for_communications_jetstream_subject_expiry(&store, SAFETY_STATE_SUBJECT_V1);
    wait_for_communications_jetstream_subject_expiry(&store, SCAN_CANDIDATE_SUBJECT_V1);

    let _preview = start_attachment_preview_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_preview,
    );
    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = attachment_preview_gateway_v1(&store, &supervisor, &root, &data, realtime);
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let preview = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        StartAttachmentPreviewRequestV1 {
            protocol_major: 1,
            operation_id: vec![0xD1; 16],
            attachment_anchor_id: attachment_anchor_id.to_vec(),
        },
    );
    assert_eq!(
        preview.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    assert_eq!(preview.state, AttachmentPreviewStateV1::Accepted as i32);
    assert_eq!(
        get_attachment_preview_v1(&router, &gateway_runtime, &cookie, &preview.run_id,).state,
        AttachmentPreviewStateV1::Accepted as i32
    );

    let replay_operation_id = [0xD2; 16];
    let replay = post_retained_preview_replay_v1(
        &router,
        &gateway_runtime,
        &cookie,
        replay_operation_id,
        attachment_anchor_id,
    );
    assert_eq!(
        replay.error,
        AttachmentPreviewEvidenceReplayErrorV1::Unspecified as i32
    );
    assert!(
        replay.state == AttachmentPreviewEvidenceReplayStateV1::Accepted as i32
            || replay.state == AttachmentPreviewEvidenceReplayStateV1::AwaitingProducers as i32,
        "replay start must return an accepted or already-dispatched operation"
    );
    let replay_diagnostics = wait_for_retained_preview_replay_terminal_v1(replay_operation_id);
    assert_eq!(
        replay_diagnostics.state,
        AttachmentPreviewEvidenceReplayStateV1::Completed as i16
    );
    assert_eq!(replay_diagnostics.error, 0);
    assert_eq!(replay_diagnostics.producer_results, 2);
    assert_eq!(replay_diagnostics.communications_failure, 0);
    assert_eq!(replay_diagnostics.mail_failure, 0);
    assert_eq!(replay_diagnostics.communications_published_audits, 1);
    assert_eq!(replay_diagnostics.mail_published_audits, 1);

    let ready = wait_for_ready_attachment_preview_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &preview.run_id,
        "retained evidence recovery",
    );
    assert_eq!(ready.state, AttachmentPreviewStateV1::Ready as i32);
    let event = read_terminal_attachment_preview_sse_event_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &preview.run_id,
    );
    assert!(!event.payload.is_empty());
    let ticket = post_attachment_preview_proto_v1::<_, IssueAttachmentPreviewReadResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
        IssueAttachmentPreviewReadRequestV1 {
            protocol_major: 1,
            run_id: preview.run_id,
        },
    );
    assert_eq!(
        ticket.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    let (status, body) = read_attachment_preview_blob_v1(
        &router,
        &gateway_runtime,
        Some(&cookie),
        ticket.opaque_read_ticket,
    );
    assert_eq!(status, StatusCode::OK);
    assert!(!body.is_empty());
    assert_replay_control_payload_is_private(event.payload.as_slice());
    assert_replay_control_payload_is_private(replay.encode_to_vec().as_slice());

    let duplicate_operation_id = [0xD3; 16];
    let duplicate = post_retained_preview_replay_v1(
        &router,
        &gateway_runtime,
        &cookie,
        duplicate_operation_id,
        attachment_anchor_id,
    );
    assert_eq!(
        duplicate.error,
        AttachmentPreviewEvidenceReplayErrorV1::Unspecified as i32
    );
    let duplicate_diagnostics =
        wait_for_retained_preview_replay_terminal_v1(duplicate_operation_id);
    assert_eq!(
        duplicate_diagnostics.state,
        AttachmentPreviewEvidenceReplayStateV1::Completed as i16
    );
    assert_eq!(duplicate_diagnostics.producer_results, 2);
    assert_eq!(duplicate_diagnostics.communications_published_audits, 1);
    assert_eq!(duplicate_diagnostics.mail_published_audits, 1);
    assert_eq!(
        get_attachment_preview_v1(&router, &gateway_runtime, &cookie, &ready.run_id).state,
        AttachmentPreviewStateV1::Ready as i32
    );

    let partial_operation_id = [0xD4; 16];
    let removed_mail_index = remove_retained_mail_replay_index_v1(attachment_anchor_id);
    let partial = post_retained_preview_replay_v1(
        &router,
        &gateway_runtime,
        &cookie,
        partial_operation_id,
        attachment_anchor_id,
    );
    assert_eq!(
        partial.error,
        AttachmentPreviewEvidenceReplayErrorV1::Unspecified as i32
    );
    let partial_diagnostics = wait_for_retained_preview_replay_terminal_v1(partial_operation_id);
    assert_eq!(
        partial_diagnostics.state,
        AttachmentPreviewEvidenceReplayStateV1::Unavailable as i16
    );
    assert_eq!(
        partial_diagnostics.error,
        AttachmentPreviewEvidenceReplayErrorV1::ProducerUnavailable as i16
    );
    assert_eq!(partial_diagnostics.producer_results, 2);
    assert_eq!(partial_diagnostics.communications_failure, 0);
    assert_eq!(partial_diagnostics.mail_failure, 1);
    assert_eq!(partial_diagnostics.communications_published_audits, 1);
    assert_eq!(partial_diagnostics.mail_published_audits, 0);
    assert_replay_control_payload_is_private(partial.encode_to_vec().as_slice());
    restore_retained_mail_replay_index_v1(removed_mail_index);

    let event_endpoint = store
        .platform_event_hub_topology()
        .expect("read replay outage Event Hub topology")
        .expect("replay outage Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let outage_observer = gateway_runtime
        .block_on(async_nats::connect(event_endpoint))
        .expect("connect replay outage observer");
    super::nats_outage_fixture::set_authenticated_nats_container_running(false);
    let outage_operation_id = [0xD6; 16];
    let outage = post_retained_preview_replay_v1(
        &router,
        &gateway_runtime,
        &cookie,
        outage_operation_id,
        attachment_anchor_id,
    );
    assert_eq!(
        outage.error,
        AttachmentPreviewEvidenceReplayErrorV1::Unspecified as i32
    );
    let replay_successor = reserve_attachment_preview_evidence_replay_successor_v1(
        &supervisor,
        &store,
        replay_runtime,
    );
    super::nats_outage_fixture::set_authenticated_nats_container_running(true);
    super::nats_outage_fixture::wait_for_authenticated_nats_reconnect(
        &gateway_runtime,
        &outage_observer,
        "retained evidence replay observer",
    );
    replay_runtime = start_attachment_preview_evidence_replay_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        replay_successor,
    );
    assert_eq!(replay_runtime.runtime_generation, 2);
    let outage_diagnostics = wait_for_retained_preview_replay_terminal_v1(outage_operation_id);
    assert_eq!(
        outage_diagnostics.state,
        AttachmentPreviewEvidenceReplayStateV1::Completed as i16
    );
    assert_eq!(outage_diagnostics.producer_results, 2);
    assert_eq!(outage_diagnostics.communications_published_audits, 1);
    assert_eq!(outage_diagnostics.mail_published_audits, 1);
    assert_replay_control_payload_is_private(outage.encode_to_vec().as_slice());

    let mismatched_mail = restart_mail_runtime_without_smtp_for_human_owner(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        mail,
        imap.port(),
        "owner-2",
    );
    let wrong_owner_operation_id = [0xD7; 16];
    assert!(!mismatched_mail.registration_id.is_empty());
    let wrong_owner = post_retained_preview_replay_v1(
        &router,
        &gateway_runtime,
        &cookie,
        wrong_owner_operation_id,
        attachment_anchor_id,
    );
    assert_eq!(
        wrong_owner.error,
        AttachmentPreviewEvidenceReplayErrorV1::Unspecified as i32
    );
    let wrong_owner_diagnostics =
        wait_for_retained_preview_replay_terminal_v1(wrong_owner_operation_id);
    assert_eq!(
        wrong_owner_diagnostics.state,
        AttachmentPreviewEvidenceReplayStateV1::Rejected as i16
    );
    assert_eq!(
        wrong_owner_diagnostics.error,
        AttachmentPreviewEvidenceReplayErrorV1::StaleProducerFence as i16
    );
    assert_eq!(wrong_owner_diagnostics.producer_results, 2);
    assert_eq!(wrong_owner_diagnostics.communications_failure, 0);
    assert_eq!(wrong_owner_diagnostics.mail_failure, 6);
    assert_eq!(wrong_owner_diagnostics.communications_published_audits, 1);
    assert_eq!(wrong_owner_diagnostics.mail_published_audits, 0);
    assert_replay_control_payload_is_private(wrong_owner.encode_to_vec().as_slice());

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove retained Preview recovery fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn post_retained_preview_replay_v1(
    router: &attachment_preview_gateway_fixture::AttachmentPreviewGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    operation_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
) -> StartAttachmentPreviewEvidenceReplayResponseV1 {
    post_attachment_preview_proto_v1(
        router,
        runtime,
        cookie,
        ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONNECT_PATH_V1,
        StartAttachmentPreviewEvidenceReplayRequestV1 {
            protocol_major: 1,
            operation_id: operation_id.to_vec(),
            attachment_anchor_id: attachment_anchor_id.to_vec(),
        },
    )
}

fn assert_replay_control_payload_is_private(bytes: &[u8]) {
    for private_marker in [
        b"managed Mail body".as_slice(),
        b"Y2xlYW4tcm9vbS1hdHRhY2htZW50".as_slice(),
    ] {
        assert!(
            !bytes
                .windows(private_marker.len())
                .any(|window| window == private_marker),
            "retained evidence replay control payload exposed private source content"
        );
    }
}

fn wait_for_retained_preview_attachment_state_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    attachment_anchor_id: [u8; 16],
    expected_state: u32,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if wait_for_attachment_state(store, supervisor, attachment_anchor_id) == expected_state {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "retained Preview attachment did not reach the expected safety state"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
