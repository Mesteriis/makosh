//! Live signed managed Engine admission before attachment verdict scenarios.

use super::*;

use super::attachment_security_clamav_fixture::{
    AttachmentSecurityClamAvFixture, ClamAvFixtureOutcomeV1,
};
use crate::identity::device::signer::DeviceSigner;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS and Attachment Security binaries"]
fn managed_attachment_security_engine_starts_with_exact_signed_contracts() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("makosh-managed-attachment-security");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
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
    let _admitted_mail = admit_mail_runtime(&store);
    let admitted_attachment_security = admit_attachment_security_runtime(&store);
    let mut blob_source = AttachmentSecurityBlobSourceFixture::admit(&store);
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
    let admitted_attachment_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_attachment_security);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mut attachment_security = start_attachment_security_runtime(
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
    assert_eq!(attachment_security.runtime_generation, 1);
    assert!(attachment_security.grant_epoch > 0);
    assert!(!attachment_security.runtime_instance_id.is_empty());
    let plaintext =
        b"clean attachment payload visible only to Blob and the loopback scanner fixture";
    let blob = blob_source.write(&store, &supervisor, &data, [81; 16], plaintext);
    let attachment = prepare_communications_attachment_for_scan(
        &store,
        "clean",
        blob.declared_size,
        blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(&store, &attachment, &blob, &clamav, plaintext);
    assert_attachment_security_source_blob_read_is_denied(
        &store,
        &supervisor,
        &data,
        &attachment_security,
        &blob,
    );
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );
    assert_stale_attachment_security_verdict_cas_is_rejected(&store, &attachment);
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );

    let outage_plaintext =
        b"attachment fixture-held-clean payload retained through NATS outage and relay restart";
    let outage_blob = blob_source.write(&store, &supervisor, &data, [86; 16], outage_plaintext);
    let outage_attachment = prepare_communications_attachment_for_scan(
        &store,
        "outage-restart",
        outage_blob.declared_size,
        outage_blob.receipt_sha256,
    );
    let previous_runtime_instance_id = attachment_security.runtime_instance_id.clone();
    attachment_security = assert_attachment_security_outbox_replays_after_nats_outage_and_restart(
        &store,
        &outage_attachment,
        &outage_blob,
        &clamav,
        || {
            supervisor
                .stop(&attachment_security.registration_id)
                .expect("stop Attachment Security runtime with pending verdict");
        },
        || {
            supervisor
                .stop(COMMUNICATIONS_REGISTRATION)
                .expect("stop Communications before deterministic NATS recovery");
            assert_eq!(
                restart_communications_domain(&supervisor, &store, &root.join("runtime")),
                2
            );
        },
        || {
            restart_attachment_security_runtime(
                &supervisor,
                &store,
                &root.join("runtime"),
                &attachment_security,
                clamav.port(),
            )
        },
    );
    assert_eq!(attachment_security.runtime_generation, 2);
    assert_ne!(
        attachment_security.runtime_instance_id,
        previous_runtime_instance_id
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            outage_attachment.attachment_anchor_id
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );

    let threat_plaintext =
        b"attachment fixture-threat payload visible only to Blob and scanner fixture";
    let threat_blob = blob_source.write(&store, &supervisor, &data, [82; 16], threat_plaintext);
    let threat_attachment = prepare_communications_attachment_for_scan(
        &store,
        "threat",
        threat_blob.declared_size,
        threat_blob.receipt_sha256,
    );
    assert_threat_attachment_security_verdict_flow(
        &store,
        &threat_attachment,
        &threat_blob,
        &clamav,
        threat_plaintext,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            threat_attachment.attachment_anchor_id
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::Quarantined
            as u32
    );

    for (scenario_id, blob_id, plaintext, scanner_outcome) in [
        (
            "malformed",
            [83; 16],
            b"attachment fixture-malformed response scenario".as_slice(),
            ClamAvFixtureOutcomeV1::Malformed,
        ),
        (
            "disconnect",
            [84; 16],
            b"attachment fixture-disconnect I/O scenario".as_slice(),
            ClamAvFixtureOutcomeV1::Disconnect,
        ),
        (
            "timeout",
            [85; 16],
            b"attachment fixture-timeout response scenario".as_slice(),
            ClamAvFixtureOutcomeV1::Timeout,
        ),
    ] {
        let failure_blob = blob_source.write(&store, &supervisor, &data, blob_id, plaintext);
        let failure_attachment = prepare_communications_attachment_for_scan(
            &store,
            scenario_id,
            failure_blob.declared_size,
            failure_blob.receipt_sha256,
        );
        assert_attachment_security_scanner_failure_is_fail_closed(
            &store,
            &failure_attachment,
            &failure_blob,
            &clamav,
            scanner_outcome,
        );
        assert_eq!(
            wait_for_attachment_state(
                &store,
                &supervisor,
                failure_attachment.attachment_anchor_id
            ),
            makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::BlobAdmitted
                as u32
        );
    }
    assert_eq!(clamav.outcome_count(ClamAvFixtureOutcomeV1::Clean), 1);
    assert_eq!(clamav.outcome_count(ClamAvFixtureOutcomeV1::Threat), 1);
    assert_eq!(clamav.outcome_count(ClamAvFixtureOutcomeV1::HeldClean), 1);

    let successor_source_plaintext =
        b"fixture-custody source generation successor before target-bound transfer";
    let successor_source_blob = blob_source.write(
        &store,
        &supervisor,
        &data,
        [87; 16],
        successor_source_plaintext,
    );
    let successor_source_attachment = prepare_communications_attachment_for_scan(
        &store,
        "source-successor",
        successor_source_blob.declared_size,
        successor_source_blob.receipt_sha256,
    );
    blob_source.advance_runtime_generation(&store, "72727272727272727272727272727272");
    assert_clean_attachment_security_verdict_flow(
        &store,
        &successor_source_attachment,
        &successor_source_blob,
        &clamav,
        successor_source_plaintext,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            successor_source_attachment.attachment_anchor_id
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );

    let revoked_source_blob = blob_source.write(
        &store,
        &supervisor,
        &data,
        [88; 16],
        b"fixture-custody source revoked before target-bound transfer",
    );
    let revoked_source_attachment = prepare_communications_attachment_for_scan(
        &store,
        "source-revoked",
        revoked_source_blob.declared_size,
        revoked_source_blob.receipt_sha256,
    );
    blob_source.revoke(&store);
    assert_attachment_security_custody_failure_is_fail_closed(
        &store,
        &revoked_source_attachment,
        &revoked_source_blob,
        &clamav,
        ClamAvFixtureOutcomeV1::CustodyProbe,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            revoked_source_attachment.attachment_anchor_id
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::BlobAdmitted
            as u32
    );

    let authority_blob_source = AttachmentSecurityBlobSourceFixture::admit_authority_source(&store);
    let vault_outage_blob = authority_blob_source.write(
        &store,
        &supervisor,
        &data,
        [89; 16],
        b"fixture-vault-outage Vault unavailable payload",
    );
    let vault_outage_attachment = prepare_communications_attachment_for_scan(
        &store,
        "vault-outage",
        vault_outage_blob.declared_size,
        vault_outage_blob.receipt_sha256,
    );
    let blob_outage_blob = authority_blob_source.write(
        &store,
        &supervisor,
        &data,
        [90; 16],
        b"fixture-blob-outage Blob unavailable payload",
    );
    let blob_outage_attachment = prepare_communications_attachment_for_scan(
        &store,
        "blob-outage",
        blob_outage_blob.declared_size,
        blob_outage_blob.receipt_sha256,
    );
    let target_revoked_blob = authority_blob_source.write(
        &store,
        &supervisor,
        &data,
        [92; 16],
        b"fixture-target-revoked before target-bound transfer",
    );
    let target_revoked_attachment = prepare_communications_attachment_for_scan(
        &store,
        "target-revoked",
        target_revoked_blob.declared_size,
        target_revoked_blob.receipt_sha256,
    );

    supervisor
        .stop("vault")
        .expect("stop Vault for Attachment Security custody outage");
    assert_attachment_security_custody_failure_is_fail_closed(
        &store,
        &vault_outage_attachment,
        &vault_outage_blob,
        &clamav,
        ClamAvFixtureOutcomeV1::VaultOutageProbe,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            vault_outage_attachment.attachment_anchor_id
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::BlobAdmitted
            as u32
    );
    supervisor
        .stop(&attachment_security.registration_id)
        .expect("stop Attachment Security before rebinding successor Storage");
    supervisor
        .stop("blob")
        .expect("stop Blob before rebinding successor Vault");
    supervisor
        .stop("storage")
        .expect("stop Storage before rebinding successor Vault");
    assert_eq!(start_vault(&supervisor, &store, &data, release.kernel()), 2);
    assert_eq!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory(),
        ),
        2
    );
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("restart signed Blob runtime after Vault outage"),
        2
    );
    assert_eq!(
        restart_communications_domain(&supervisor, &store, &root.join("runtime")),
        3
    );
    attachment_security = restart_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        &attachment_security,
        clamav.port(),
    );
    assert_eq!(attachment_security.runtime_generation, 3);

    supervisor
        .stop("blob")
        .expect("stop Blob for Attachment Security custody outage");
    assert_attachment_security_custody_failure_is_fail_closed(
        &store,
        &blob_outage_attachment,
        &blob_outage_blob,
        &clamav,
        ClamAvFixtureOutcomeV1::BlobOutageProbe,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            blob_outage_attachment.attachment_anchor_id
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::BlobAdmitted
            as u32
    );
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("restart signed Blob runtime after Blob outage"),
        3
    );

    store
        .transition_module_registration(
            &attachment_security.registration_id,
            ModuleRegistrationState::Revoked,
        )
        .expect("revoke Attachment Security target registration");
    assert_attachment_security_custody_failure_is_fail_closed(
        &store,
        &target_revoked_attachment,
        &target_revoked_blob,
        &clamav,
        ClamAvFixtureOutcomeV1::TargetRevokedProbe,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            target_revoked_attachment.attachment_anchor_id
        ),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::BlobAdmitted
            as u32
    );

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}
