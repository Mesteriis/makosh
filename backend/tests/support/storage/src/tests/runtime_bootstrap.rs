use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeVaultRouteResponseV1, VaultCiphertextResponseV1,
    VaultCiphertextRouteDirectionV1,
    managed_runtime_control_request_v1::Operation as ControlOperation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_storage_protocol::v1::{
    GetStorageRuntimeStatusRequestV1, StorageDeploymentProfileV1, StorageRuntimeConfigurationV1,
    StorageRuntimeControlRequestV1, StorageRuntimeControlResponseV1, StorageRuntimeStateV1,
    StorageRuntimeTopologyV1, storage_runtime_control_request_v1::Operation as StorageOperation,
    storage_runtime_control_response_v1::Result as StorageResult,
};
use makosh_vault_protocol::{
    LeaseAudienceV1, VaultActionV1, VaultCiphertextFrameV1, VaultResponseRecipientV1,
    VaultTransportBindingV1, VaultTransportCommandV1, VaultTransportDirectionV1,
    VaultTransportPublicKey, seal,
};
use prost::Message;

use crate::storage_runtime_control::{
    serve_credential_bootstrapped_on_channel, serve_inherited_on_channel,
};

const AUTHENTICATED_TEST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_TEST";
const PGBOUNCER_PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE";
const PGBOUNCER_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST";
const PGBOUNCER_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT";
const POSTGRES_PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE";
const POSTGRES_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST";
const POSTGRES_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT";

#[test]
fn inherited_startup_rejects_an_unresponsive_pgbouncer_after_ciphertext_bootstrap() {
    let vault = VaultResponseRecipientV1::generate();
    let postgres = std::net::TcpListener::bind("127.0.0.1:0").expect("PostgreSQL listener");
    let pgbouncer = std::net::TcpListener::bind("127.0.0.1:0").expect("PgBouncer listener");
    let configuration = configuration(&vault, listener_port(&postgres), listener_port(&pgbouncer));
    let (mut kernel, child) = UnixStream::pair().expect("inherited channel");
    let runtime = std::thread::spawn(move || {
        serve_credential_bootstrapped_on_channel(child, vec![1], Vec::new(), configuration)
    });
    serve_descriptor(&mut kernel);
    respond_to_bootstrap(&mut kernel, &vault);
    drop(kernel);
    assert!(runtime.join().expect("Storage runtime thread").is_err());
}

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_inherited_runtime_bootstraps_real_platform_services() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let vault = VaultResponseRecipientV1::generate();
    let configuration = authenticated_configuration(&vault);
    let pgbouncer_password = read_credential(PGBOUNCER_PASSWORD_FILE_ENV);
    let postgres_password = read_credential(POSTGRES_PASSWORD_FILE_ENV);
    let (mut kernel, child) = UnixStream::pair().expect("inherited channel");
    let runtime = std::thread::spawn(move || {
        serve_inherited_on_channel(child, vec![1], Vec::new(), configuration)
    });

    serve_descriptor(&mut kernel);
    respond_to_authenticated_bootstrap(
        &mut kernel,
        &vault,
        &pgbouncer_password,
        &postgres_password,
    );
    assert_ready(&mut kernel);
    write_status_request(&mut kernel);
    assert_reconciling_status(&mut kernel);
    drop(kernel);
    assert!(runtime.join().expect("Storage runtime thread").is_err());
}

fn configuration(
    vault: &VaultResponseRecipientV1,
    postgres_port: u16,
    pgbouncer_port: u16,
) -> StorageRuntimeConfigurationV1 {
    StorageRuntimeConfigurationV1 {
        topology: Some(StorageRuntimeTopologyV1 {
            topology_revision: 1,
            storage_generation: 1,
            storage_instance_id: "storage-main".to_owned(),
            database_id: "makosh".to_owned(),
            deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
            postgres_artifact_sha256: vec![1; 32],
            pgbouncer_artifact_sha256: vec![2; 32],
            postgres_host: "127.0.0.1".to_owned(),
            postgres_port: u32::from(postgres_port),
            pgbouncer_host: "127.0.0.1".to_owned(),
            pgbouncer_port: u32::from(pgbouncer_port),
            pgbouncer_postgres_host: "127.0.0.1".to_owned(),
            pgbouncer_postgres_port: u32::from(postgres_port),
        }),
        vault_instance_id: "vault-main".to_owned(),
        vault_runtime_generation: 7,
        vault_hpke_public_key_x25519: vault.public_key().as_bytes().to_vec(),
        desired_bindings: Vec::new(),
        pgbouncer_database_config_path: String::new(),
        desired_bundles: Vec::new(),
        pgbouncer_auth_file_path: String::new(),
    }
}

fn authenticated_configuration(vault: &VaultResponseRecipientV1) -> StorageRuntimeConfigurationV1 {
    StorageRuntimeConfigurationV1 {
        topology: Some(StorageRuntimeTopologyV1 {
            topology_revision: 1,
            storage_generation: 1,
            storage_instance_id: "storage_main".to_owned(),
            database_id: "makosh_storage_authenticated".to_owned(),
            deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
            postgres_artifact_sha256: vec![1; 32],
            pgbouncer_artifact_sha256: vec![2; 32],
            postgres_host: required(POSTGRES_HOST_ENV),
            postgres_port: port(POSTGRES_PORT_ENV),
            pgbouncer_host: required(PGBOUNCER_HOST_ENV),
            pgbouncer_port: port(PGBOUNCER_PORT_ENV),
            pgbouncer_postgres_host: required(POSTGRES_HOST_ENV),
            pgbouncer_postgres_port: port(POSTGRES_PORT_ENV),
        }),
        vault_instance_id: "vault_main".to_owned(),
        vault_runtime_generation: 1,
        vault_hpke_public_key_x25519: vault.public_key().as_bytes().to_vec(),
        desired_bindings: Vec::new(),
        pgbouncer_database_config_path: String::new(),
        desired_bundles: Vec::new(),
        pgbouncer_auth_file_path: String::new(),
    }
}

fn listener_port(listener: &std::net::TcpListener) -> u16 {
    listener.local_addr().expect("listener address").port()
}

fn respond_to_bootstrap(kernel: &mut UnixStream, vault: &VaultResponseRecipientV1) {
    respond_issue(
        kernel,
        vault,
        VaultActionV1::Resolve,
        b"0123456789abcdef0123456789abcdef",
    );
    respond_denied(kernel, vault);
    respond_issue(
        kernel,
        vault,
        VaultActionV1::Create,
        b"fedcba9876543210fedcba9876543210",
    );
    respond_generated(kernel, vault);
    respond_issue(
        kernel,
        vault,
        VaultActionV1::Resolve,
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    respond_resolved(kernel, vault);
}

fn respond_to_authenticated_bootstrap(
    kernel: &mut UnixStream,
    vault: &VaultResponseRecipientV1,
    pgbouncer_password: &[u8],
    postgres_password: &[u8],
) {
    respond_authenticated_lease(
        kernel,
        vault,
        "storage.control.pgbouncer.admin",
        "11111111111111111111111111111111",
    );
    respond_authenticated_secret(
        kernel,
        vault,
        "11111111111111111111111111111111",
        pgbouncer_password,
    );
    respond_authenticated_lease(
        kernel,
        vault,
        "storage.control.postgres.admin",
        "22222222222222222222222222222222",
    );
    respond_authenticated_secret(
        kernel,
        vault,
        "22222222222222222222222222222222",
        postgres_password,
    );
}

fn respond_authenticated_lease(
    kernel: &mut UnixStream,
    vault: &VaultResponseRecipientV1,
    expected_purpose: &str,
    lease_id: &str,
) {
    let route = read_route(kernel);
    assert!(matches!(
        command_from_route(vault, &route),
        VaultTransportCommandV1::IssueLease { request }
            if request.purpose().purpose_id() == expected_purpose
    ));
    write_vault_response(kernel, &route, lease_id.as_bytes());
}

fn respond_authenticated_secret(
    kernel: &mut UnixStream,
    vault: &VaultResponseRecipientV1,
    expected_lease_id: &str,
    secret: &[u8],
) {
    let route = read_route(kernel);
    assert!(matches!(
        command_from_route(vault, &route),
        VaultTransportCommandV1::ResolveLease { lease_id, .. }
            if lease_id.as_str() == expected_lease_id
    ));
    write_vault_response(kernel, &route, secret);
}

fn respond_issue(
    kernel: &mut UnixStream,
    vault: &VaultResponseRecipientV1,
    expected_action: VaultActionV1,
    plaintext: &[u8],
) {
    let route = read_route(kernel);
    let command = command_from_route(vault, &route);
    assert!(matches!(
        command,
        VaultTransportCommandV1::IssueLease { ref request }
            if request.purpose().actions() == [expected_action]
    ));
    write_vault_response(kernel, &route, plaintext);
}

fn respond_generated(kernel: &mut UnixStream, vault: &VaultResponseRecipientV1) {
    let route = read_route(kernel);
    assert!(matches!(
        command_from_route(vault, &route),
        VaultTransportCommandV1::GenerateOpaqueToken { .. }
    ));
    write_vault_response(kernel, &route, &[7; 16]);
}

fn respond_resolved(kernel: &mut UnixStream, vault: &VaultResponseRecipientV1) {
    let route = read_route(kernel);
    assert!(matches!(
        command_from_route(vault, &route),
        VaultTransportCommandV1::ResolveLease { .. }
    ));
    write_vault_response(kernel, &route, b"resolved-platform-credential");
}

fn write_vault_response(
    kernel: &mut UnixStream,
    route: &makosh_runtime_protocol::v1::VaultCiphertextRouteV1,
    plaintext: &[u8],
) {
    let response = ManagedRuntimeVaultRouteResponseV1 {
        response: Some(encrypt_response(route, plaintext)),
        error_code: String::new(),
    };
    write_control_vault_response(kernel, response);
}

fn respond_denied(kernel: &mut UnixStream, vault: &VaultResponseRecipientV1) {
    let route = read_route(kernel);
    assert!(matches!(
        command_from_route(vault, &route),
        VaultTransportCommandV1::ResolveLease { .. }
    ));
    write_control_vault_response(
        kernel,
        ManagedRuntimeVaultRouteResponseV1 {
            response: None,
            error_code: "missing_credential".to_owned(),
        },
    );
}

fn read_route(kernel: &mut UnixStream) -> makosh_runtime_protocol::v1::VaultCiphertextRouteV1 {
    let request = ManagedRuntimeControlRequestV1::decode(read_frame(kernel).as_slice())
        .expect("managed control request");
    let Some(ControlOperation::RouteVaultCiphertext(request)) = request.operation else {
        panic!("Vault route request");
    };
    request.route.expect("Vault route")
}

fn write_control_vault_response(
    kernel: &mut UnixStream,
    response: ManagedRuntimeVaultRouteResponseV1,
) {
    write_frame(
        kernel,
        &ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::VaultRoute(response)),
            error_code: String::new(),
        }
        .encode_to_vec(),
    );
}

fn command_from_route(
    vault: &VaultResponseRecipientV1,
    route: &makosh_runtime_protocol::v1::VaultCiphertextRouteV1,
) -> VaultTransportCommandV1 {
    let binding = transport_binding(route, VaultTransportDirectionV1::ToVault);
    let frame = VaultCiphertextFrameV1::from_parts(
        route.hpke_encapped_key.clone(),
        route.ciphertext.clone(),
        route.hpke_authentication_tag.clone(),
    )
    .expect("request frame");
    let plaintext = vault.open(&binding, &frame).expect("open request");
    VaultTransportCommandV1::decode(plaintext.as_slice()).expect("typed Vault command")
}

fn encrypt_response(
    route: &makosh_runtime_protocol::v1::VaultCiphertextRouteV1,
    plaintext: &[u8],
) -> VaultCiphertextResponseV1 {
    let key: [u8; 32] = route
        .response_recipient_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .expect("response recipient key");
    let key = VaultTransportPublicKey::from_bytes(key).expect("response recipient public key");
    let frame = seal(
        &key,
        &transport_binding(route, VaultTransportDirectionV1::FromVault),
        plaintext,
    )
    .expect("encrypted response");
    VaultCiphertextResponseV1 {
        major: 1,
        vault_runtime_generation: route.vault_runtime_generation,
        request_id: route.request_id.clone(),
        operation_digest_sha256: route.operation_digest_sha256.clone(),
        direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
        hpke_encapped_key: frame.encapped_key().to_vec(),
        ciphertext: frame.ciphertext().to_vec(),
        hpke_authentication_tag: frame.tag().to_vec(),
        caller_runtime_generation: route.caller_runtime_generation,
    }
}

fn transport_binding(
    route: &makosh_runtime_protocol::v1::VaultCiphertextRouteV1,
    direction: VaultTransportDirectionV1,
) -> VaultTransportBindingV1 {
    VaultTransportBindingV1::new(
        route.vault_runtime_generation,
        LeaseAudienceV1::new(
            route.registration_id.clone(),
            route.runtime_instance_id.clone(),
            route.caller_runtime_generation,
            route.grant_epoch,
        )
        .expect("route audience"),
        route.request_id.as_slice().try_into().expect("request ID"),
        route
            .operation_digest_sha256
            .as_slice()
            .try_into()
            .expect("operation digest"),
        direction,
        route
            .response_recipient_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .expect("response recipient key"),
    )
    .expect("transport binding")
}

fn serve_descriptor(kernel: &mut UnixStream) {
    let _request = ManagedRuntimeControlRequestV1::decode(read_frame(kernel).as_slice())
        .expect("descriptor request");
    let response = ManagedRuntimeControlResponseV1 {
        result: Some(ControlResult::Describe(DescribeManagedRuntimeResponseV1 {
            registration_id: "storage-control".to_owned(),
            runtime_generation: 4,
            grant_epoch: 9,
        })),
        error_code: String::new(),
    };
    write_frame(kernel, &response.encode_to_vec());
}

fn assert_ready(kernel: &mut UnixStream) {
    let request = ManagedRuntimeControlRequestV1::decode(read_frame(kernel).as_slice())
        .expect("Storage ready request");
    assert!(matches!(
        request.operation,
        Some(makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Ready(
            ready
        )) if ready.registration_id == "storage-control"
            && ready.runtime_generation == 4
            && ready.grant_epoch == 9
    ));
}

fn write_status_request(kernel: &mut UnixStream) {
    let request = StorageRuntimeControlRequestV1 {
        operation: Some(StorageOperation::GetStatus(
            GetStorageRuntimeStatusRequestV1 {},
        )),
    };
    write_frame(kernel, &request.encode_to_vec());
}

fn assert_reconciling_status(kernel: &mut UnixStream) {
    let response = StorageRuntimeControlResponseV1::decode(read_frame(kernel).as_slice())
        .expect("Storage status response");
    assert!(matches!(
        response.result,
        Some(StorageResult::Status(value)) if value.state == StorageRuntimeStateV1::Reconciling as i32
    ));
}

fn read_credential(name: &str) -> Vec<u8> {
    let path = required(name);
    let metadata = std::fs::symlink_metadata(&path).expect("credential metadata");
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    let mut credential = std::fs::read(path).expect("credential bytes");
    while matches!(credential.last(), Some(b'\n' | b'\r')) {
        credential.pop();
    }
    assert!(!credential.is_empty());
    credential
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("storage integration test requires {name}"))
}

fn port(name: &str) -> u32 {
    required(name)
        .parse()
        .unwrap_or_else(|_| panic!("storage integration test requires a valid {name}"))
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0; 1];
        stream.read_exact(&mut byte).expect("frame length");
        length |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let mut bytes = vec![0; usize::try_from(length).expect("frame size")];
            stream.read_exact(&mut bytes).expect("frame payload");
            return bytes;
        }
    }
    panic!("frame length is invalid");
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) {
    let mut length = u32::try_from(bytes.len()).expect("frame size");
    while length >= 0x80 {
        stream
            .write_all(&[(length as u8 & 0x7f) | 0x80])
            .expect("frame length");
        length >>= 7;
    }
    stream.write_all(&[length as u8]).expect("frame length");
    stream.write_all(bytes).expect("frame payload");
    stream.flush().expect("frame flush");
}
