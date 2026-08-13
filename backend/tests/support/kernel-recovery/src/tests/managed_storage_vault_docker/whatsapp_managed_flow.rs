//! Live signed WhatsApp admission, route-specific grants and revoke fencing.

use super::*;

use makosh_kernel_control_store::{ModuleRegistrationState, PlatformStorageBindingStateV1};
use makosh_whatsapp_api::{
    WhatsAppProviderCommand, WhatsAppPublicClientRequestV1, WhatsAppPublicClientResponseV1,
    client_contract::WhatsAppClientContractV1,
    host_bridge::{
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostBridgeHandshakeV1,
        decode_host_bridge_handshake_accepted, encode_host_bridge_handshake,
    },
    operational::WhatsAppOperationalQueryV1,
    realtime::WhatsAppOperationalReplayRequestV1,
};
use makosh_whatsapp_persistence::WHATSAPP_OWNER_RLS_TABLES_V1;
use makosh_whatsapp_runtime::{
    admission::WHATSAPP_STORAGE_CAPABILITY_ID,
    client_port::{decode_module_response, encode_module_request},
};

use crate::identity::device::signer::DeviceSigner;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, WhatsApp and NATS binaries"]
fn managed_whatsapp_runtime_uses_signed_kernel_admission_and_host_route_fencing() {
    let contour = ManagedWhatsAppContour::start(WhatsAppGrantProfileV1::QueryOnly);
    assert_whatsapp_query_is_admitted(&contour.store, &contour.supervisor, &contour.whatsapp);
    assert_host_route_is_bound(&contour.whatsapp);
    assert_ungranted_whatsapp_command_is_rejected(
        &contour.store,
        &contour.supervisor,
        &contour.whatsapp,
    );
    assert_ungranted_whatsapp_operational_query_is_rejected(
        &contour.store,
        &contour.supervisor,
        &contour.whatsapp,
    );
    assert_ungranted_whatsapp_operational_replay_is_rejected(
        &contour.store,
        &contour.supervisor,
        &contour.whatsapp,
    );
    assert_stale_whatsapp_query_generation_is_rejected(
        &contour.store,
        &contour.supervisor,
        &contour.whatsapp,
    );
    // Admission-grade NOBYPASSRLS proof covers the complete owner-local WhatsApp store.
    assert_owner_rls_tables_v1(
        "makosh_storage_authenticated",
        &WHATSAPP_OWNER_RLS_TABLES_V1,
        "whatsapp_owner_scope",
    );
    let (owner_runtime_dir, owner_control) = start_owner_control(
        &contour.data,
        &contour.store,
        &contour.shutdown,
        &contour.supervisor,
    );
    revoke_whatsapp_runtime(
        &owner_runtime_dir,
        &contour.owner_signer,
        &contour.store,
        &contour.supervisor,
        &contour.whatsapp,
    );

    contour.shutdown_processes();
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    contour.finish();
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, WhatsApp and NATS binaries"]
fn managed_whatsapp_runtime_bootstrap_fails_closed_and_stops_promptly() {
    let contour = ManagedWhatsAppContour::start(WhatsAppGrantProfileV1::QueryOnly);
    assert_whatsapp_query_is_admitted(&contour.store, &contour.supervisor, &contour.whatsapp);
    contour
        .supervisor
        .stop(&contour.whatsapp.registration_id)
        .expect("stop healthy WhatsApp predecessor");
    let runtime_dir = contour.data.join("runtime");
    let mut predecessor = contour.whatsapp.clone();

    for (phase, bootstrap_override) in [
        (
            "missing-settings",
            WhatsAppBootstrapOverrideV1::MissingSettings,
        ),
        (
            "missing-storage",
            WhatsAppBootstrapOverrideV1::MissingStorage,
        ),
        (
            "stale-host-fence",
            WhatsAppBootstrapOverrideV1::StaleHostFence,
        ),
    ] {
        let capture = whatsapp_child_capture_v1(&contour.root, phase);
        predecessor = launch_whatsapp_successor_without_ready_v1(
            &contour.supervisor,
            &contour.store,
            &contour.data,
            &runtime_dir,
            &predecessor,
            bootstrap_override,
            &capture,
        );
        assert_whatsapp_pre_spawn_denied_v1(&contour.supervisor, &predecessor, phase, &capture);
    }

    for (phase, bootstrap_override) in [
        (
            "invalid-settings",
            WhatsAppBootstrapOverrideV1::InvalidSettings,
        ),
        (
            "missing-host-route",
            WhatsAppBootstrapOverrideV1::MissingHostRoute,
        ),
        (
            "stale-event-fence",
            WhatsAppBootstrapOverrideV1::StaleEventFence,
        ),
    ] {
        let capture = whatsapp_child_capture_v1(&contour.root, phase);
        predecessor = launch_whatsapp_successor_without_ready_v1(
            &contour.supervisor,
            &contour.store,
            &contour.data,
            &runtime_dir,
            &predecessor,
            bootstrap_override,
            &capture,
        );
        assert_whatsapp_bounded_runtime_denied_v1(
            &contour.supervisor,
            &predecessor,
            phase,
            &capture,
        );
    }

    for (phase, bootstrap_override) in [
        (
            "stale-storage-fence",
            WhatsAppBootstrapOverrideV1::StaleStorageFence,
        ),
        (
            "stale-vault-fence",
            WhatsAppBootstrapOverrideV1::StaleVaultFence,
        ),
    ] {
        let capture = whatsapp_child_capture_v1(&contour.root, phase);
        predecessor = launch_whatsapp_successor_without_ready_v1(
            &contour.supervisor,
            &contour.store,
            &contour.data,
            &runtime_dir,
            &predecessor,
            bootstrap_override,
            &capture,
        );
        assert_whatsapp_active_until_requested_stop_v1(
            &contour.supervisor,
            &predecessor,
            phase,
            &capture,
        );
    }

    contour.shutdown_processes();
    contour.finish();
}

fn whatsapp_child_capture_v1(root: &Path, phase: &str) -> PathBuf {
    private_directory(root.join(format!("whatsapp-stdio-{phase}")))
}

fn whatsapp_child_capture_paths_v1(directory: &Path) -> Vec<PathBuf> {
    let mut paths = std::fs::read_dir(directory)
        .expect("read WhatsApp child capture directory")
        .map(|entry| entry.expect("read WhatsApp child capture entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn assert_whatsapp_pre_spawn_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedWhatsAppRuntime,
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
            .expect("WhatsApp pre-spawn activity"),
        "{phase} must be denied before child spawn"
    );
    assert!(
        whatsapp_child_capture_paths_v1(capture).is_empty(),
        "{phase} must not create supervised child output"
    );
}

fn assert_whatsapp_bounded_runtime_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedWhatsAppRuntime,
    phase: &str,
    capture: &Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while supervisor
        .is_active(&started.registration_id)
        .expect("WhatsApp bounded denial activity")
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
    let captures = whatsapp_child_capture_paths_v1(capture);
    assert!(
        captures.len() >= 2 && captures.len().is_multiple_of(2),
        "{phase} must have bounded complete supervised child attempts"
    );
}

fn assert_whatsapp_active_until_requested_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedWhatsAppRuntime,
    phase: &str,
    capture: &Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(100);
    while std::time::Instant::now() < deadline {
        assert!(
            supervisor
                .is_active(&started.registration_id)
                .expect("WhatsApp bootstrap activity"),
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
            .expect("request WhatsApp bootstrap stop"),
        "{phase} must own the active child"
    );
    assert!(
        supervisor
            .stop_if_active(&started.registration_id)
            .expect("join WhatsApp bootstrap stop")
    );
    assert!(stopped_at.elapsed() < Duration::from_secs(2));
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("WhatsApp stopped activity"),
        "{phase} must not install a replacement"
    );
    assert_eq!(
        whatsapp_child_capture_paths_v1(capture).len(),
        2,
        "{phase} must spawn exactly one supervised child"
    );
}

fn assert_ungranted_whatsapp_operational_replay_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    whatsapp: &StartedWhatsAppRuntime,
) {
    let request = encode_module_request(
        25,
        &WhatsAppPublicClientRequestV1::OperationalReplay(WhatsAppOperationalReplayRequestV1 {
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            after_sequence: 0,
            limit: 10,
        }),
    )
    .expect("encode ungranted WhatsApp operational replay");
    let route = ManagedCapabilityRouteRequest::new(
        &whatsapp.registration_id,
        &whatsapp.runtime_instance_id,
        whatsapp.runtime_generation,
        whatsapp.grant_epoch,
        WhatsAppClientContractV1::OperationalRealtime.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("ungranted WhatsApp operational replay route"),
        "capability is not granted to this registration"
    );
}

fn assert_ungranted_whatsapp_operational_query_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    whatsapp: &StartedWhatsAppRuntime,
) {
    let request = encode_module_request(
        24,
        &WhatsAppPublicClientRequestV1::OperationalQuery(
            WhatsAppOperationalQueryV1::GetRuntimeStatus {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            },
        ),
    )
    .expect("encode ungranted WhatsApp operational query");
    let route = ManagedCapabilityRouteRequest::new(
        &whatsapp.registration_id,
        &whatsapp.runtime_instance_id,
        whatsapp.runtime_generation,
        whatsapp.grant_epoch,
        WhatsAppClientContractV1::OperationalQuery.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("ungranted WhatsApp operational query route"),
        "capability is not granted to this registration"
    );
}

fn assert_whatsapp_query_is_admitted(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    whatsapp: &StartedWhatsAppRuntime,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&whatsapp.registration_id) != Ok(true) {
        assert!(
            std::time::Instant::now() < deadline,
            "managed WhatsApp runtime did not become ready: {:?}",
            supervisor.last_failure(&whatsapp.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    let request = WhatsAppPublicClientRequestV1::OperationStatus {
        operation_id: "unknown-operation".to_owned(),
    };
    let encoded = encode_module_request(21, &request).expect("encode WhatsApp query");
    loop {
        let route = ManagedCapabilityRouteRequest::new(
            &whatsapp.registration_id,
            &whatsapp.runtime_instance_id,
            whatsapp.runtime_generation,
            whatsapp.grant_epoch,
            WhatsAppClientContractV1::Query.capability_id(),
            &encoded,
        );
        let last_route = match route_managed_client_request(store, &relay, &route) {
            Ok(bytes) => match decode_module_response(WhatsAppClientContractV1::Query, &bytes) {
                Ok((21, WhatsAppPublicClientResponseV1::OperationStatus(None))) => return,
                outcome => format!("unexpected response: {outcome:?}"),
            },
            Err(error) => format!("route error: {error:?}"),
        };
        assert!(
            std::time::Instant::now() < deadline,
            "managed WhatsApp query remained unavailable: {:?}; {last_route}",
            supervisor.last_failure(&whatsapp.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_host_route_is_bound(whatsapp: &StartedWhatsAppRuntime) {
    drop(WhatsAppHostBridgeTestClient::connect(whatsapp));
}

fn assert_ungranted_whatsapp_command_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    whatsapp: &StartedWhatsAppRuntime,
) {
    let request = encode_module_request(
        22,
        &WhatsAppPublicClientRequestV1::Command(WhatsAppProviderCommand::SendText {
            operation_id: "ungranted-whatsapp-command".to_owned(),
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            text: "Kernel rejects this route before WhatsApp receives it".to_owned(),
        }),
    )
    .expect("encode ungranted WhatsApp command");
    let route = ManagedCapabilityRouteRequest::new(
        &whatsapp.registration_id,
        &whatsapp.runtime_instance_id,
        whatsapp.runtime_generation,
        whatsapp.grant_epoch,
        WhatsAppClientContractV1::Command.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("ungranted WhatsApp command route"),
        "capability is not granted to this registration"
    );
}

fn assert_stale_whatsapp_query_generation_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    whatsapp: &StartedWhatsAppRuntime,
) {
    let request = encode_module_request(
        23,
        &WhatsAppPublicClientRequestV1::OperationStatus {
            operation_id: "stale-whatsapp-query".to_owned(),
        },
    )
    .expect("encode stale WhatsApp query");
    let route = ManagedCapabilityRouteRequest::new(
        &whatsapp.registration_id,
        &whatsapp.runtime_instance_id,
        whatsapp.runtime_generation + 1,
        whatsapp.grant_epoch,
        WhatsAppClientContractV1::Query.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("stale WhatsApp query generation"),
        "managed runtime fence is stale"
    );
}

fn revoke_whatsapp_runtime(
    owner_runtime_dir: &Path,
    signer: &FileDeviceSigner,
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    whatsapp: &StartedWhatsAppRuntime,
) {
    let revoked = transition_registration(
        owner_runtime_dir,
        signer,
        &whatsapp.registration_id,
        "revoked",
    );
    assert_eq!(revoked.state, "revoked");
    assert!(
        revoked.grant_epoch > whatsapp.grant_epoch,
        "revoke advances the durable grant epoch before process stop"
    );
    let registration = store
        .module_registration(&whatsapp.registration_id)
        .expect("read revoked WhatsApp registration")
        .expect("revoked WhatsApp registration");
    assert_eq!(registration.state(), ModuleRegistrationState::Revoked);
    let binding = store
        .platform_storage_binding(&whatsapp.registration_id, WHATSAPP_STORAGE_CAPABILITY_ID)
        .expect("read revoked WhatsApp Storage binding")
        .expect("revoked WhatsApp Storage binding");
    assert_eq!(
        binding.state(),
        PlatformStorageBindingStateV1::Revoking,
        "owner transition durably reserves the exact WhatsApp Storage fence"
    );
    assert!(
        !supervisor
            .stop_if_active(&whatsapp.registration_id)
            .expect("observe stopped WhatsApp worker"),
        "owner transition already stopped the exact WhatsApp worker"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("observe Communications worker"),
        "WhatsApp revoke must not stop Communications"
    );
    assert!(
        !whatsapp.host_bridge_socket_path.exists(),
        "managed stop removes only the exact WhatsApp host route socket",
    );
    assert_revoked_whatsapp_query_is_rejected(store, supervisor, whatsapp);
}

fn assert_revoked_whatsapp_query_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    whatsapp: &StartedWhatsAppRuntime,
) {
    let request = encode_module_request(
        24,
        &WhatsAppPublicClientRequestV1::OperationStatus {
            operation_id: "revoked-whatsapp-query".to_owned(),
        },
    )
    .expect("encode revoked WhatsApp query");
    let route = ManagedCapabilityRouteRequest::new(
        &whatsapp.registration_id,
        &whatsapp.runtime_instance_id,
        whatsapp.runtime_generation,
        whatsapp.grant_epoch,
        WhatsAppClientContractV1::Query.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("revoked WhatsApp query route"),
        "module registration is not approved"
    );
}
