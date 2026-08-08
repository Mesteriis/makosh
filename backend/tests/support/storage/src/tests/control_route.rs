use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeVaultRouteResponseV1, VaultCiphertextResponseV1,
    VaultCiphertextRouteV1, managed_runtime_control_request_v1::Operation as ControlOperation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_storage_protocol::v1::{
    GetStorageRuntimeStatusRequestV1, StorageBindingV1, StorageEffectiveBudgetsV1,
    StorageRuntimeConfigurationV1, StorageRuntimeControlRequestV1, StorageRuntimeControlResponseV1,
    StorageRuntimeStateV1, StorageRuntimeTopologyV1,
    storage_runtime_control_request_v1::Operation as StorageOperation,
    storage_runtime_control_response_v1::Result as StorageResult,
};
use prost::Message;

use crate::storage_runtime_control::{
    InheritedVaultRoutePortV1, describe, describe_on_channel, serve_inherited, serve_on_channel,
};
use crate::storage_runtime_vault::StorageVaultRoutePortV1;

#[test]
fn inherited_storage_port_uses_only_the_typed_managed_vault_route() {
    let (server, client) = UnixStream::pair().expect("inherited channel");
    let server = std::thread::spawn(move || serve_one_route(server));
    let mut port = InheritedVaultRoutePortV1::new(client);

    let response = run(port.route_vault_ciphertext(VaultCiphertextRouteV1 {
        major: 1,
        registration_id: "storage-control".into(),
        caller_runtime_generation: 1,
        ..Default::default()
    }))
    .expect("managed route response");

    assert_eq!(response.request_id, [7; 16]);
    server.join().expect("route server");
}

#[test]
fn inherited_storage_handshake_requires_a_fenced_descriptor_response() {
    let _entrypoint: fn(Vec<u8>, Vec<u8>) -> Result<UnixStream, String> = describe;
    let (server, client) = UnixStream::pair().expect("inherited channel");
    let server = std::thread::spawn(move || {
        let mut server = server;
        serve_descriptor(&mut server);
    });

    let channel =
        describe_on_channel(client, vec![1], Vec::new()).expect("accepted managed descriptor");

    drop(channel);
    server.join().expect("descriptor server");
}

#[test]
fn inherited_storage_runtime_reports_only_unconfigured_before_a_binding_exists() {
    let _entrypoint: fn(Vec<u8>, Vec<u8>, StorageRuntimeConfigurationV1) -> Result<(), String> =
        serve_inherited;
    let postgres = std::net::TcpListener::bind("127.0.0.1:0").expect("postgres listener");
    let pgbouncer = std::net::TcpListener::bind("127.0.0.1:0").expect("pgbouncer listener");
    let (mut kernel, child) = UnixStream::pair().expect("inherited channel");
    let runtime = std::thread::spawn(move || {
        serve_on_channel(
            child,
            vec![1],
            Vec::new(),
            configuration(port(&postgres), port(&pgbouncer)),
        )
    });
    serve_descriptor(&mut kernel);
    write_frame(
        &mut kernel,
        &StorageRuntimeControlRequestV1 {
            operation: Some(StorageOperation::GetStatus(
                GetStorageRuntimeStatusRequestV1 {},
            )),
        }
        .encode_to_vec(),
    );
    let response = StorageRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
        .expect("storage status response");
    assert_eq!(response.error_code, "");
    assert!(matches!(
        response.result,
        Some(StorageResult::Status(status))
            if status.state == StorageRuntimeStateV1::Reconciling as i32
                && status.storage_generation == 1
                && status.runtime_generation == 1
                && status.topology_revision == 1
                && status.vault_runtime_generation == 1
                && status.active_bindings.is_empty()
    ));
    write_frame(
        &mut kernel,
        &StorageRuntimeControlRequestV1::default().encode_to_vec(),
    );
    let refusal = StorageRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
        .expect("storage refusal response");
    assert!(refusal.result.is_none());
    assert_eq!(refusal.error_code, "invalid_request");
    drop(kernel);
    assert!(runtime.join().expect("runtime thread").is_err());
}

#[test]
fn test_channel_cannot_report_ready_for_an_unapplied_staged_binding() {
    let postgres = std::net::TcpListener::bind("127.0.0.1:0").expect("postgres listener");
    let pgbouncer = std::net::TcpListener::bind("127.0.0.1:0").expect("pgbouncer listener");
    let (mut kernel, child) = UnixStream::pair().expect("inherited channel");
    let runtime = std::thread::spawn(move || {
        serve_on_channel(
            child,
            vec![1],
            Vec::new(),
            bound_configuration(port(&postgres), port(&pgbouncer)),
        )
    });

    serve_descriptor(&mut kernel);
    write_frame(
        &mut kernel,
        &StorageRuntimeControlRequestV1 {
            operation: Some(StorageOperation::GetStatus(
                GetStorageRuntimeStatusRequestV1 {},
            )),
        }
        .encode_to_vec(),
    );
    let response = StorageRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
        .expect("storage status response");
    assert!(matches!(
        response.result,
        Some(StorageResult::Status(status))
            if status.state == StorageRuntimeStateV1::Reconciling as i32
                && status.active_bindings.is_empty()
    ));
    drop(kernel);
    assert!(runtime.join().expect("runtime thread").is_err());
}

#[test]
fn inherited_storage_runtime_fails_closed_when_an_endpoint_is_unreachable() {
    let postgres = std::net::TcpListener::bind("127.0.0.1:0").expect("postgres listener");
    let unavailable = std::net::TcpListener::bind("127.0.0.1:0").expect("unavailable port");
    let unavailable_port = port(&unavailable);
    drop(unavailable);
    let (mut kernel, child) = UnixStream::pair().expect("inherited channel");
    let runtime = std::thread::spawn(move || {
        serve_on_channel(
            child,
            vec![1],
            Vec::new(),
            configuration(port(&postgres), unavailable_port),
        )
    });

    serve_descriptor(&mut kernel);
    write_frame(
        &mut kernel,
        &StorageRuntimeControlRequestV1 {
            operation: Some(StorageOperation::GetStatus(
                GetStorageRuntimeStatusRequestV1 {},
            )),
        }
        .encode_to_vec(),
    );
    let response = StorageRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
        .expect("storage status response");

    assert!(matches!(
        response.result,
        Some(StorageResult::Status(status))
            if status.state == StorageRuntimeStateV1::Failed as i32
                && status.blocker_code == "storage_endpoint_unavailable"
                && status.active_bindings.is_empty()
    ));
    drop(kernel);
    assert!(runtime.join().expect("runtime thread").is_err());
}

#[test]
fn inherited_storage_runtime_rechecks_endpoints_after_startup() {
    let postgres = std::net::TcpListener::bind("127.0.0.1:0").expect("postgres listener");
    let pgbouncer = std::net::TcpListener::bind("127.0.0.1:0").expect("pgbouncer listener");
    let postgres_port = port(&postgres);
    let pgbouncer_port = port(&pgbouncer);
    let (mut kernel, child) = UnixStream::pair().expect("inherited channel");
    let runtime = std::thread::spawn(move || {
        serve_on_channel(
            child,
            vec![1],
            Vec::new(),
            configuration(postgres_port, pgbouncer_port),
        )
    });

    serve_descriptor(&mut kernel);
    drop(pgbouncer);
    write_frame(
        &mut kernel,
        &StorageRuntimeControlRequestV1 {
            operation: Some(StorageOperation::GetStatus(
                GetStorageRuntimeStatusRequestV1 {},
            )),
        }
        .encode_to_vec(),
    );
    let response = StorageRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
        .expect("storage status response");

    assert!(matches!(
        response.result,
        Some(StorageResult::Status(status))
            if status.state == StorageRuntimeStateV1::Failed as i32
                && status.blocker_code == "storage_endpoint_unavailable"
    ));
    drop(kernel);
    assert!(runtime.join().expect("runtime thread").is_err());
}

fn topology(postgres_port: u16, pgbouncer_port: u16) -> StorageRuntimeTopologyV1 {
    StorageRuntimeTopologyV1 {
        topology_revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".into(),
        database_id: "makosh".into(),
        deployment_profile:
            makosh_storage_protocol::v1::StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
        postgres_artifact_sha256: vec![1; 32],
        pgbouncer_artifact_sha256: vec![2; 32],
        postgres_host: "127.0.0.1".to_owned(),
        postgres_port: u32::from(postgres_port),
        pgbouncer_host: "127.0.0.1".to_owned(),
        pgbouncer_port: u32::from(pgbouncer_port),
        pgbouncer_postgres_host: "127.0.0.1".to_owned(),
        pgbouncer_postgres_port: u32::from(postgres_port),
    }
}

fn configuration(postgres_port: u16, pgbouncer_port: u16) -> StorageRuntimeConfigurationV1 {
    StorageRuntimeConfigurationV1 {
        topology: Some(topology(postgres_port, pgbouncer_port)),
        vault_instance_id: "vault_main".to_owned(),
        vault_runtime_generation: 1,
        vault_hpke_public_key_x25519: vec![3; 32],
        desired_bindings: Vec::new(),
        pgbouncer_database_config_path: String::new(),
        desired_bundles: Vec::new(),
        pgbouncer_auth_file_path: String::new(),
    }
}

fn bound_configuration(postgres_port: u16, pgbouncer_port: u16) -> StorageRuntimeConfigurationV1 {
    let mut configuration = configuration(postgres_port, pgbouncer_port);
    configuration.desired_bindings = vec![StorageBindingV1 {
        storage_instance_id: "storage_main".to_owned(),
        storage_generation: 1,
        database_id: "makosh".to_owned(),
        owner: "notes".to_owned(),
        registration_id: "registration_notes".to_owned(),
        runtime_instance_id: "runtime_notes".to_owned(),
        runtime_generation: 1,
        grant_epoch: 1,
        role_epoch: 1,
        runtime_principal: "runtime_notes".to_owned(),
        pool_alias: "runtime_registration_notes_1".to_owned(),
        effective_budgets: Some(StorageEffectiveBudgetsV1 {
            max_connections: 4,
            statement_timeout_millis: 5_000,
        }),
        credential_lease_revision: 1,
        storage_bundle_revision: 1,
        storage_bundle_digest: vec![1; 32],
    }];
    configuration.pgbouncer_database_config_path = "/private/storage/databases.ini".to_owned();
    configuration.pgbouncer_auth_file_path = "/private/storage/users.txt".to_owned();
    configuration
}

fn port(listener: &std::net::TcpListener) -> u16 {
    listener.local_addr().expect("listener address").port()
}

fn serve_one_route(mut stream: UnixStream) {
    let request = ManagedRuntimeControlRequestV1::decode(read_frame(&mut stream).as_slice())
        .expect("managed control request");
    let Some(ControlOperation::RouteVaultCiphertext(request)) = request.operation else {
        panic!("managed route request");
    };
    assert_eq!(
        request.route.expect("route").registration_id,
        "storage-control"
    );
    let response = ManagedRuntimeControlResponseV1 {
        result: Some(ControlResult::VaultRoute(
            ManagedRuntimeVaultRouteResponseV1 {
                response: Some(VaultCiphertextResponseV1 {
                    request_id: vec![7; 16],
                    caller_runtime_generation: 1,
                    ..Default::default()
                }),
                error_code: String::new(),
            },
        )),
        error_code: String::new(),
    };
    write_frame(&mut stream, &response.encode_to_vec());
}

fn serve_descriptor(stream: &mut UnixStream) {
    let request = ManagedRuntimeControlRequestV1::decode(read_frame(stream).as_slice())
        .expect("descriptor request");
    assert!(request.operation.is_some());
    let response = ManagedRuntimeControlResponseV1 {
        result: Some(ControlResult::Describe(DescribeManagedRuntimeResponseV1 {
            registration_id: "storage-control".into(),
            runtime_generation: 1,
            grant_epoch: 1,
        })),
        error_code: String::new(),
    };
    write_frame(stream, &response.encode_to_vec());
}

fn run<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("test runtime")
        .block_on(future)
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0; 1];
        stream.read_exact(&mut byte).expect("frame length");
        length |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            break;
        }
    }
    let mut bytes = vec![0; usize::try_from(length).expect("frame size")];
    stream.read_exact(&mut bytes).expect("frame bytes");
    bytes
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) {
    let mut length = u32::try_from(bytes.len()).expect("frame length");
    while length >= 0x80 {
        stream
            .write_all(&[(length as u8 & 0x7f) | 0x80])
            .expect("frame prefix");
        length >>= 7;
    }
    stream.write_all(&[length as u8]).expect("frame prefix");
    stream.write_all(bytes).expect("frame payload");
}
