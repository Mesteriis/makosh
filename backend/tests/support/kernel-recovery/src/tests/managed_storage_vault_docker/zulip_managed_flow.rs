//! Live managed Zulip launch through Kernel-owned admission and platform leases.

use super::*;

use makosh_kernel_control_store::{
    ModuleRegistrationState, PlatformStorageBindingStateV1, SettingsApplyState,
};
use makosh_zulip_api::{
    ZulipClientRequestV1, ZulipClientResponseV1, ZulipCommandV1,
    account::{ZulipAccountLifecycleCommandV1, ZulipCredentialBindingStateV1},
    client_contract::ZulipClientContractV1,
    operational::{ZulipOperationalQueryResponseV1, ZulipOperationalQueryV1},
};
use makosh_zulip_persistence::ZULIP_OWNER_RLS_TABLES_V1;
use makosh_zulip_runtime::{
    admission::ZULIP_STORAGE_CAPABILITY_ID,
    client_port::{decode_module_response, encode_module_request},
};

use crate::identity::device::signer::DeviceSigner;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Zulip and NATS binaries"]
fn managed_zulip_runtime_uses_kernel_leases_and_route_specific_admission() {
    let contour = ManagedZulipContour::start(ZulipGrantProfileV1::QueryOnly);
    assert_zulip_query_is_admitted(&contour.store, &contour.supervisor, &contour.zulip);
    assert!(
        contour.fixture.accepted_connections() > 0,
        "managed Zulip runtime must reach the live loopback HTTPS fixture"
    );
    assert_ungranted_zulip_command_is_rejected(&contour.store, &contour.supervisor, &contour.zulip);
    assert_stale_zulip_query_generation_is_rejected(
        &contour.store,
        &contour.supervisor,
        &contour.zulip,
    );
    assert_owner_rls_tables_v1(
        "makosh_storage_authenticated",
        &ZULIP_OWNER_RLS_TABLES_V1,
        "zulip_owner_scope",
    );
    // The shared assertion switches to an effective NOSUPERUSER/NOBYPASSRLS
    // runtime role before exercising every owner-local table.
    let (owner_runtime_dir, owner_control) = start_owner_control(
        &contour.data,
        &contour.store,
        &contour.shutdown,
        &contour.supervisor,
    );
    revoke_zulip_runtime(
        &owner_runtime_dir,
        &contour.owner_signer,
        &contour.store,
        &contour.supervisor,
        &contour.zulip,
    );

    contour.shutdown_processes();
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    contour.finish();
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Zulip and NATS binaries"]
fn managed_zulip_runtime_bootstrap_fails_closed_and_stops_promptly() {
    let contour = ManagedZulipContour::start(ZulipGrantProfileV1::QueryOnly);
    assert_zulip_query_is_admitted(&contour.store, &contour.supervisor, &contour.zulip);
    contour
        .supervisor
        .stop(&contour.zulip.registration_id)
        .expect("stop healthy Zulip predecessor");
    let runtime_dir = contour.root.join("runtime");
    let mut predecessor = contour.zulip.clone();

    for (phase, bootstrap_override) in [
        (
            "missing-settings",
            ZulipBootstrapOverrideV1::MissingSettings,
        ),
        ("missing-storage", ZulipBootstrapOverrideV1::MissingStorage),
        (
            "missing-event-capability",
            ZulipBootstrapOverrideV1::MissingEventCapability,
        ),
        (
            "missing-blob-capability",
            ZulipBootstrapOverrideV1::MissingBlobCapability,
        ),
    ] {
        let capture = zulip_child_capture_v1(&contour.root, phase);
        predecessor = launch_zulip_successor_without_ready_v1(
            &contour.supervisor,
            &contour.store,
            &contour.data,
            &runtime_dir,
            &predecessor,
            contour.fixture.realm_url(),
            bootstrap_override,
            &capture,
        );
        assert_zulip_pre_spawn_denied_v1(&contour.supervisor, &predecessor, phase, &capture);
    }

    for (phase, bootstrap_override) in [
        (
            "invalid-settings",
            ZulipBootstrapOverrideV1::InvalidSettings,
        ),
        (
            "stale-event-fence",
            ZulipBootstrapOverrideV1::StaleEventFence,
        ),
    ] {
        let capture = zulip_child_capture_v1(&contour.root, phase);
        predecessor = launch_zulip_successor_without_ready_v1(
            &contour.supervisor,
            &contour.store,
            &contour.data,
            &runtime_dir,
            &predecessor,
            contour.fixture.realm_url(),
            bootstrap_override,
            &capture,
        );
        assert_zulip_bounded_runtime_denied_v1(&contour.supervisor, &predecessor, phase, &capture);
    }

    for (phase, bootstrap_override) in [
        (
            "stale-storage-fence",
            ZulipBootstrapOverrideV1::StaleStorageFence,
        ),
        (
            "unavailable-vault",
            ZulipBootstrapOverrideV1::StaleVaultFence,
        ),
    ] {
        let capture = zulip_child_capture_v1(&contour.root, phase);
        predecessor = launch_zulip_successor_without_ready_v1(
            &contour.supervisor,
            &contour.store,
            &contour.data,
            &runtime_dir,
            &predecessor,
            contour.fixture.realm_url(),
            bootstrap_override,
            &capture,
        );
        assert_zulip_active_until_requested_stop_v1(
            &contour.supervisor,
            &predecessor,
            phase,
            &capture,
        );
    }

    contour.shutdown_processes();
    contour.finish();
}

fn zulip_child_capture_v1(root: &Path, phase: &str) -> PathBuf {
    private_directory(root.join(format!("zulip-stdio-{phase}")))
}

fn zulip_child_capture_paths_v1(directory: &Path) -> Vec<PathBuf> {
    let mut paths = std::fs::read_dir(directory)
        .expect("read Zulip child capture directory")
        .map(|entry| entry.expect("read Zulip child capture entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn assert_zulip_pre_spawn_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedZulipRuntime,
    phase: &str,
    capture: &Path,
) {
    assert_ne!(
        supervisor.relay_port().is_ready(&started.registration_id),
        Ok(true)
    );
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("Zulip pre-spawn activity"),
        "{phase} must be denied before child spawn"
    );
    assert!(
        zulip_child_capture_paths_v1(capture).is_empty(),
        "{phase} must not create supervised child output"
    );
}

fn assert_zulip_bounded_runtime_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedZulipRuntime,
    phase: &str,
    capture: &Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while supervisor
        .is_active(&started.registration_id)
        .expect("Zulip bounded denial activity")
    {
        assert!(
            std::time::Instant::now() < deadline,
            "{phase} did not terminate"
        );
        assert_ne!(
            supervisor.relay_port().is_ready(&started.registration_id),
            Ok(true),
            "{phase} must not signal Ready"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let captures = zulip_child_capture_paths_v1(capture);
    assert!(
        captures.len() >= 2 && captures.len().is_multiple_of(2),
        "{phase} must have bounded complete supervised child attempts"
    );
}

fn assert_zulip_active_until_requested_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedZulipRuntime,
    phase: &str,
    capture: &Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(100);
    while std::time::Instant::now() < deadline {
        assert!(
            supervisor
                .is_active(&started.registration_id)
                .expect("Zulip bootstrap activity"),
            "{phase} child exited before requested stop"
        );
        assert_ne!(
            supervisor.relay_port().is_ready(&started.registration_id),
            Ok(true)
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let stopped_at = std::time::Instant::now();
    assert!(
        supervisor
            .request_stop_if_active(&started.registration_id)
            .expect("request Zulip bootstrap stop"),
        "{phase} must own the active child"
    );
    assert!(
        supervisor
            .stop_if_active(&started.registration_id)
            .expect("join Zulip bootstrap stop")
    );
    assert!(stopped_at.elapsed() < Duration::from_secs(2));
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("Zulip stopped activity"),
        "{phase} must not install a replacement"
    );
    assert_eq!(
        zulip_child_capture_paths_v1(capture).len(),
        2,
        "{phase} must spawn exactly one supervised child"
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Zulip and NATS binaries"]
fn managed_zulip_account_rotation_and_retirement_use_settings_successors() {
    let contour = ManagedZulipContour::start(ZulipGrantProfileV1::CommandAndQuery);
    let predecessor = contour.zulip.clone();
    assert_zulip_query_is_admitted(&contour.store, &contour.supervisor, &predecessor);
    assert!(
        contour.fixture.accepted_connections() > 0,
        "predecessor Zulip runtime must resolve revision one before Vault rotation",
    );
    let _rotated_credential = rotate_zulip_vault(&contour.vault_dir, &contour.seeded_credential);
    let rotated = bind_zulip_credential(&contour.store, &contour.supervisor, &predecessor, 1, 2);
    assert_eq!(rotated.binding_revision, 2);

    let (owner_runtime_dir, owner_control) = start_owner_control(
        &contour.data,
        &contour.store,
        &contour.shutdown,
        &contour.supervisor,
    );
    let (client, owner_session) =
        open_owner_control_client(&owner_runtime_dir, &contour.owner_signer);
    let revision_two =
        zulip_settings_snapshot(ZULIP_ACCOUNT_ID, 2, contour.fixture.realm_url()).encode_to_vec();
    crate::modules::settings::mutation::commit_after_owner_authorization_for_target(
        &*contour.store,
        &predecessor.registration_id,
        ZULIP_ACCOUNT_ID,
        1,
        &revision_two,
    )
    .expect("commit Zulip Settings revision two");
    let applied = client
        .apply_managed_integration_settings(
            &owner_session,
            &predecessor.registration_id,
            ZULIP_STORAGE_CAPABILITY_ID,
            ZULIP_ACCOUNT_ID,
            2,
            false,
        )
        .expect("apply Zulip credential-rotation successor");
    assert_eq!(
        applied.runtime_generation,
        predecessor.runtime_generation + 1
    );
    assert_eq!(
        contour
            .store
            .settings_configuration_target(&predecessor.registration_id, ZULIP_ACCOUNT_ID)
            .expect("read applied Zulip Settings target")
            .expect("applied Zulip Settings")
            .apply_state(),
        SettingsApplyState::Current,
    );
    assert_eq!(
        contour
            .store
            .settings_configuration_target(&predecessor.registration_id, ZULIP_ACCOUNT_ID)
            .expect("read applied Zulip Settings target")
            .expect("applied Zulip Settings")
            .effective_revision(),
        2,
    );
    let successor = current_zulip_runtime(&contour, &predecessor);
    let status = query_zulip_account_status(&contour, &successor);
    assert_eq!(
        status.credential_state,
        ZulipCredentialBindingStateV1::Active
    );
    assert_eq!(status.credential_revision, Some(2));
    assert_eq!(status.binding_revision, 2);
    assert_eq!(
        status.applied_runtime_generation,
        Some(applied.runtime_generation)
    );
    wait_for_credential_v2_request(&contour);
    assert_stale_zulip_query_generation_is_rejected(
        contour.store.as_ref(),
        &contour.supervisor,
        &predecessor,
    );

    let retired = apply_zulip_account_lifecycle(
        &contour,
        &successor,
        ZulipAccountLifecycleCommandV1::RetireAccount {
            account_id: ZULIP_ACCOUNT_ID.to_owned(),
            expected_binding_revision: 2,
        },
    );
    assert_eq!(retired.binding_revision, 3);
    assert_eq!(retired.state, ZulipCredentialBindingStateV1::Retired);
    wait_for_provider_quiescence(&contour);
    let provider_connections = contour.fixture.accepted_connections();
    let revision_three =
        zulip_settings_snapshot(ZULIP_ACCOUNT_ID, 3, contour.fixture.realm_url()).encode_to_vec();
    crate::modules::settings::mutation::commit_after_owner_authorization_for_target(
        &*contour.store,
        &predecessor.registration_id,
        ZULIP_ACCOUNT_ID,
        2,
        &revision_three,
    )
    .expect("commit Zulip Settings revision three");
    let retired_apply = client
        .apply_managed_integration_settings(
            &owner_session,
            &predecessor.registration_id,
            ZULIP_STORAGE_CAPABILITY_ID,
            ZULIP_ACCOUNT_ID,
            3,
            false,
        )
        .expect("apply configuration-only retired Zulip successor");
    let retired_runtime = current_zulip_runtime(&contour, &successor);
    assert_eq!(
        query_zulip_account_status(&contour, &retired_runtime).credential_state,
        ZulipCredentialBindingStateV1::Retired,
    );
    assert_eq!(
        retired_apply.runtime_generation,
        successor.runtime_generation + 1
    );
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(
        contour.fixture.accepted_connections(),
        provider_connections,
        "retired Zulip successor must remain configuration-only",
    );
    let missing_credential =
        bind_zulip_credential(&contour.store, &contour.supervisor, &retired_runtime, 3, 3);
    assert_eq!(missing_credential.binding_revision, 4);
    let revision_four =
        zulip_settings_snapshot(ZULIP_ACCOUNT_ID, 4, contour.fixture.realm_url()).encode_to_vec();
    crate::modules::settings::mutation::commit_after_owner_authorization_for_target(
        &*contour.store,
        &predecessor.registration_id,
        ZULIP_ACCOUNT_ID,
        3,
        &revision_four,
    )
    .expect("commit Zulip Settings revision four");
    client
        .apply_managed_integration_settings(
            &owner_session,
            &predecessor.registration_id,
            ZULIP_STORAGE_CAPABILITY_ID,
            ZULIP_ACCOUNT_ID,
            4,
            false,
        )
        .expect_err("missing Zulip credential revision must block replacement");
    let blocked = contour
        .store
        .settings_configuration_target(&predecessor.registration_id, ZULIP_ACCOUNT_ID)
        .expect("read blocked Zulip Settings target")
        .expect("blocked Zulip Settings");
    assert_eq!(blocked.desired_revision(), 4);
    assert_eq!(blocked.effective_revision(), 3);
    assert_eq!(blocked.apply_state(), SettingsApplyState::BlockedConfig);
    assert!(
        !contour
            .supervisor
            .is_active(&predecessor.registration_id)
            .expect("read failed Zulip successor state"),
        "failed Zulip successor must not reactivate its predecessor",
    );

    contour.shutdown_processes();
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    contour.finish();
}

fn current_zulip_runtime(
    contour: &ManagedZulipContour,
    predecessor: &StartedZulipRuntime,
) -> StartedZulipRuntime {
    let binding = contour
        .store
        .platform_storage_binding(&predecessor.registration_id, ZULIP_STORAGE_CAPABILITY_ID)
        .expect("read current Zulip Storage binding")
        .expect("current Zulip Storage binding");
    let registration = contour
        .store
        .module_registration(&predecessor.registration_id)
        .expect("read current Zulip registration")
        .expect("current Zulip registration");
    StartedZulipRuntime {
        registration_id: predecessor.registration_id.clone(),
        runtime_instance_id: binding.runtime_instance_id().to_owned(),
        runtime_generation: binding.runtime_generation(),
        grant_epoch: registration.grant_epoch(),
        capability_ids: predecessor.capability_ids.clone(),
    }
}

fn query_zulip_account_status(
    contour: &ManagedZulipContour,
    runtime: &StartedZulipRuntime,
) -> makosh_zulip_api::operational::ZulipAccountStatusV1 {
    let request =
        ZulipClientRequestV1::OperationalQuery(ZulipOperationalQueryV1::GetAccountStatus {
            account_id: ZULIP_ACCOUNT_ID.to_owned(),
        });
    let encoded = encode_module_request(42, &request).expect("encode Zulip account status");
    let response = route_zulip_until_available(
        contour,
        runtime,
        ZulipClientContractV1::OperationalQuery,
        &encoded,
    );
    let (_, response) = decode_module_response(ZulipClientContractV1::OperationalQuery, &response)
        .expect("decode Zulip account status");
    let ZulipClientResponseV1::OperationalQuery(ZulipOperationalQueryResponseV1::AccountStatus(
        status,
    )) = response
    else {
        panic!("Zulip operational query returned the wrong response")
    };
    status
}

fn apply_zulip_account_lifecycle(
    contour: &ManagedZulipContour,
    runtime: &StartedZulipRuntime,
    command: ZulipAccountLifecycleCommandV1,
) -> makosh_zulip_api::account::ZulipAccountLifecycleReceiptV1 {
    let encoded = encode_module_request(43, &ZulipClientRequestV1::AccountLifecycle(command))
        .expect("encode Zulip account lifecycle");
    let response = route_zulip_until_available(
        contour,
        runtime,
        ZulipClientContractV1::AccountLifecycle,
        &encoded,
    );
    let (_, response) = decode_module_response(ZulipClientContractV1::AccountLifecycle, &response)
        .expect("decode Zulip account lifecycle");
    let ZulipClientResponseV1::AccountLifecycle(receipt) = response else {
        panic!("Zulip account lifecycle returned the wrong response")
    };
    receipt
}

fn route_zulip_until_available(
    contour: &ManagedZulipContour,
    runtime: &StartedZulipRuntime,
    contract: ZulipClientContractV1,
    encoded: &[u8],
) -> Vec<u8> {
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let route = ManagedCapabilityRouteRequest::new(
            &runtime.registration_id,
            &runtime.runtime_instance_id,
            runtime.runtime_generation,
            runtime.grant_epoch,
            contract.capability_id(),
            encoded,
        );
        let response = match route_managed_client_request(
            contour.store.as_ref(),
            &contour.supervisor.relay_port(),
            &route,
        ) {
            Ok(response) => response,
            Err(error) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Zulip client route remained unavailable: {error}",
                );
                std::thread::sleep(Duration::from_millis(25));
                continue;
            }
        };
        let envelope =
            makosh_runtime_protocol::v1::ModuleClientResponseV1::decode(response.as_slice())
                .expect("decode Zulip client response envelope");
        if envelope.error_code.is_empty() {
            return response;
        }
        assert!(
            envelope.error_code == "RUNTIME_UNAVAILABLE" && std::time::Instant::now() < deadline,
            "Zulip client request failed: {}",
            envelope.error_code,
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_credential_v2_request(contour: &ManagedZulipContour) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while contour.fixture.credential_v2_requests() == 0 {
        assert!(
            std::time::Instant::now() < deadline,
            "rotated Zulip credential did not reach the provider fixture",
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_provider_quiescence(contour: &ManagedZulipContour) {
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    let mut prior = contour.fixture.accepted_connections();
    loop {
        std::thread::sleep(Duration::from_millis(100));
        let current = contour.fixture.accepted_connections();
        if current == prior {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "retired Zulip provider loop did not quiesce",
        );
        prior = current;
    }
}

fn assert_zulip_query_is_admitted(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&zulip.registration_id) != Ok(true) {
        assert!(
            std::time::Instant::now() < deadline,
            "managed Zulip runtime did not become ready: {:?}",
            supervisor.last_failure(&zulip.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    let request = ZulipClientRequestV1::OperationStatus {
        operation_id: "unknown-operation".to_owned(),
    };
    let encoded = encode_module_request(11, &request).expect("encode Zulip query");
    loop {
        let route = ManagedCapabilityRouteRequest::new(
            &zulip.registration_id,
            &zulip.runtime_instance_id,
            zulip.runtime_generation,
            zulip.grant_epoch,
            ZulipClientContractV1::Query.capability_id(),
            &encoded,
        );
        let last_route = match route_managed_client_request(store, &relay, &route) {
            Ok(bytes) => match decode_module_response(ZulipClientContractV1::Query, &bytes) {
                Ok((11, ZulipClientResponseV1::OperationStatus(None))) => return,
                outcome => format!("unexpected response: {outcome:?}"),
            },
            Err(error) => format!("route error: {error:?}"),
        };
        assert!(
            std::time::Instant::now() < deadline,
            "managed Zulip query remained unavailable: {:?}; {last_route}",
            supervisor.last_failure(&zulip.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_ungranted_zulip_command_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let request = encode_module_request(
        12,
        &ZulipClientRequestV1::Command(ZulipCommandV1::SendStream {
            operation_id: "ungranted-zulip-command".to_owned(),
            account_id: ZULIP_ACCOUNT_ID.to_owned(),
            stream: "operations".to_owned(),
            topic: "admission".to_owned(),
            content: "Kernel must reject this route before Zulip receives it".to_owned(),
        }),
    )
    .expect("encode ungranted Zulip command");
    let route = ManagedCapabilityRouteRequest::new(
        &zulip.registration_id,
        &zulip.runtime_instance_id,
        zulip.runtime_generation,
        zulip.grant_epoch,
        ZulipClientContractV1::Command.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("ungranted Zulip command route"),
        "capability is not granted to this registration"
    );
}

fn assert_stale_zulip_query_generation_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let request = encode_module_request(
        13,
        &ZulipClientRequestV1::OperationStatus {
            operation_id: "stale-zulip-query".to_owned(),
        },
    )
    .expect("encode stale Zulip query");
    let route = ManagedCapabilityRouteRequest::new(
        &zulip.registration_id,
        &zulip.runtime_instance_id,
        zulip.runtime_generation + 1,
        zulip.grant_epoch,
        ZulipClientContractV1::Query.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("stale Zulip query generation"),
        "managed runtime fence is stale"
    );
}

fn revoke_zulip_runtime(
    owner_runtime_dir: &Path,
    signer: &FileDeviceSigner,
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let revoked =
        transition_registration(owner_runtime_dir, signer, &zulip.registration_id, "revoked");
    assert_eq!(revoked.state, "revoked");
    assert!(
        revoked.grant_epoch > zulip.grant_epoch,
        "revoke advances the durable grant epoch before process stop"
    );
    let registration = store
        .module_registration(&zulip.registration_id)
        .expect("read revoked Zulip registration")
        .expect("revoked Zulip registration");
    assert_eq!(registration.state(), ModuleRegistrationState::Revoked);
    let binding = store
        .platform_storage_binding(&zulip.registration_id, ZULIP_STORAGE_CAPABILITY_ID)
        .expect("read revoked Zulip Storage binding")
        .expect("revoked Zulip Storage binding");
    assert_eq!(
        binding.state(),
        PlatformStorageBindingStateV1::Revoking,
        "owner transition durably reserves the exact Zulip Storage fence"
    );
    assert!(
        !supervisor
            .stop_if_active(&zulip.registration_id)
            .expect("observe stopped Zulip worker"),
        "owner transition already stopped the exact Zulip worker"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("observe Communications worker"),
        "Zulip revoke must not stop Communications"
    );
    assert_revoked_zulip_query_is_rejected(store, supervisor, zulip);
}

fn assert_revoked_zulip_query_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let request = encode_module_request(
        14,
        &ZulipClientRequestV1::OperationStatus {
            operation_id: "revoked-zulip-query".to_owned(),
        },
    )
    .expect("encode revoked Zulip query");
    let route = ManagedCapabilityRouteRequest::new(
        &zulip.registration_id,
        &zulip.runtime_instance_id,
        zulip.runtime_generation,
        zulip.grant_epoch,
        ZulipClientContractV1::Query.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("revoked Zulip query route"),
        "module registration is not approved"
    );
}
