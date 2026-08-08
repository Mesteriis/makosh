//! Live Storage-runtime revoke over its encrypted inherited Vault route.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeVaultRouteResponseV1, VaultCiphertextResponseV1,
    VaultCiphertextRouteDirectionV1,
    managed_runtime_control_request_v1::Operation as ControlOperation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_storage_postgres::{
    PLATFORM_ADMIN_USERNAME, PostgresAdminConnectorV1, PostgresRuntimeSessionProbeV1,
    StorageRoleSpecV1, read_storage_role_audit,
};
use makosh_storage_protocol::v1::{
    RevokeStorageBindingRequestV1, StorageBindingV1, StorageBundleV1, StorageDeploymentProfileV1,
    StorageEffectiveBudgetsV1, StorageMigrationStepV1, StorageRuntimeConfigurationV1,
    StorageRuntimeControlRequestV1, StorageRuntimeControlResponseV1, StorageRuntimeStateV1,
    StorageRuntimeTopologyV1, storage_runtime_control_request_v1::Operation as StorageOperation,
    storage_runtime_control_response_v1::Result as StorageResult,
};
use makosh_vault_protocol::{
    LeaseAudienceV1, VaultCiphertextFrameV1, VaultResponseRecipientV1, VaultTransportBindingV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, VaultTransportPublicKey, seal,
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::storage_runtime_control::serve_inherited_on_channel;
use crate::tests::fixtures::storage_role_binding_in_database;

const AUTHENTICATED_TEST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_TEST";
const DATABASES_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE";
const AUTH_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_AUTH_FILE";
const PGBOUNCER_PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE";
const PGBOUNCER_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST";
const PGBOUNCER_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT";
const POSTGRES_PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE";
const POSTGRES_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST";
const POSTGRES_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT";
const RUNTIME_PASSWORD: &[u8] = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_runtime_revokes_the_exact_staged_binding_through_vault() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));
    let vault = VaultResponseRecipientV1::generate();
    let (configuration, binding) = configuration(&vault);
    let pgbouncer_password = read_credential(PGBOUNCER_PASSWORD_FILE_ENV);
    let postgres_password = read_credential(POSTGRES_PASSWORD_FILE_ENV);
    let (mut kernel, child) = UnixStream::pair().expect("inherited channel");
    let runtime = std::thread::spawn(move || {
        serve_inherited_on_channel(child, vec![1], Vec::new(), configuration)
    });

    serve_descriptor(&mut kernel);
    respond_platform_credential(
        &mut kernel,
        &vault,
        "storage.control.pgbouncer.admin",
        &pgbouncer_password,
    );
    respond_platform_credential(
        &mut kernel,
        &vault,
        "storage.control.postgres.admin",
        &postgres_password,
    );
    respond_platform_credential(
        &mut kernel,
        &vault,
        "storage.runtime.credential",
        RUNTIME_PASSWORD,
    );
    assert_ready(&mut kernel);
    assert_runtime_role_reaches_pgbouncer(&binding);
    revoke(&mut kernel, &vault, binding.clone());
    assert_reconciling(&mut kernel);
    assert_postgres_role_is_fenced(&binding, &postgres_password);
    drop(kernel);
    assert!(runtime.join().expect("Storage runtime thread").is_err());
}

fn assert_runtime_role_reaches_pgbouncer(binding: &StorageBindingV1) {
    let password = Zeroizing::new(RUNTIME_PASSWORD.to_vec());
    let runtime = tokio::runtime::Runtime::new().expect("PgBouncer runtime probe");
    runtime.block_on(async {
        let mut session = PostgresRuntimeSessionProbeV1::connect_with_password(
            &required(PGBOUNCER_HOST_ENV),
            port(PGBOUNCER_PORT_ENV),
            &binding.pool_alias,
            &binding.runtime_principal,
            &password,
        )
        .await
        .expect("runtime credential reaches PostgreSQL only through PgBouncer");
        assert_eq!(
            session
                .current_principal()
                .await
                .expect("runtime principal"),
            binding.runtime_principal
        );
    });
}

fn assert_postgres_role_is_fenced(binding: &StorageBindingV1, password: &[u8]) {
    let binding = makosh_storage_protocol::validation::storage_binding_from_message(binding)
        .expect("Storage binding model");
    let roles = StorageRoleSpecV1::platform_binding(binding).expect("Storage role specification");
    let password = Zeroizing::new(password.to_vec());
    let runtime = tokio::runtime::Runtime::new().expect("PostgreSQL audit runtime");
    runtime.block_on(async {
        let connector = PostgresAdminConnectorV1::connect_with_password(
            &required(POSTGRES_HOST_ENV),
            port(POSTGRES_PORT_ENV),
            "makosh_storage_authenticated",
            PLATFORM_ADMIN_USERNAME,
            &password,
        )
        .await
        .expect("authenticated PostgreSQL audit connection");
        let audit = read_storage_role_audit(&connector, &roles)
            .await
            .expect("fenced Storage role audit");
        assert!(!audit.can_login, "revoked role must be NOLOGIN");
    });
}

fn configuration(
    vault: &VaultResponseRecipientV1,
) -> (StorageRuntimeConfigurationV1, StorageBindingV1) {
    let suffix = std::process::id();
    let owner = format!("storage_runtime_revoke_{suffix}");
    let bundle_revision = 1;
    let model = storage_role_binding_in_database(
        "makosh_storage_authenticated",
        &owner,
        &format!("storage_runtime_principal_{suffix}"),
    );
    let sql = format!("CREATE TABLE makosh_data.{owner}_{suffix} (id BIGINT PRIMARY KEY);");
    let bundle = StorageBundleV1 {
        major: 1,
        revision: bundle_revision,
        bundle_id: format!("runtime_revoke_{suffix}"),
        owner_id: owner.clone(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: format!("runtime_revoke_{suffix}"),
            forward_sql_utf8: sql.as_bytes().to_vec(),
            sha256: Sha256::digest(sql.as_bytes()).to_vec(),
        }],
    };
    let mut binding = binding_message(&model);
    binding.storage_bundle_revision = u64::from(bundle_revision);
    binding.storage_bundle_digest = Sha256::digest(bundle.encode_to_vec()).to_vec();
    let configuration = StorageRuntimeConfigurationV1 {
        topology: Some(StorageRuntimeTopologyV1 {
            topology_revision: 1,
            storage_generation: 1,
            storage_instance_id: "storage_main".to_owned(),
            database_id: "makosh_storage_authenticated".to_owned(),
            deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
            postgres_artifact_sha256: vec![1; 32],
            pgbouncer_artifact_sha256: vec![2; 32],
            postgres_host: required(POSTGRES_HOST_ENV),
            postgres_port: u32::from(port(POSTGRES_PORT_ENV)),
            pgbouncer_host: required(PGBOUNCER_HOST_ENV),
            pgbouncer_port: u32::from(port(PGBOUNCER_PORT_ENV)),
            pgbouncer_postgres_host: "postgres".to_owned(),
            pgbouncer_postgres_port: 5_432,
        }),
        vault_instance_id: "vault_main".to_owned(),
        vault_runtime_generation: 1,
        vault_hpke_public_key_x25519: vault.public_key().as_bytes().to_vec(),
        desired_bindings: vec![binding.clone()],
        pgbouncer_database_config_path: required(DATABASES_FILE_ENV),
        desired_bundles: vec![bundle],
        pgbouncer_auth_file_path: auth_file_path(),
    };
    (configuration, binding)
}

fn auth_file_path() -> String {
    required(AUTH_FILE_ENV)
}

fn binding_message(binding: &makosh_storage_protocol::StorageBindingV1) -> StorageBindingV1 {
    let identity = binding.identity();
    let fences = binding.fences();
    let access = binding.access();
    StorageBindingV1 {
        storage_instance_id: identity.storage_instance_id().to_owned(),
        storage_generation: fences.storage_generation(),
        database_id: identity.database_id().to_owned(),
        owner: identity.owner().to_owned(),
        registration_id: identity.registration_id().to_owned(),
        runtime_instance_id: identity.runtime_instance_id().to_owned(),
        runtime_generation: fences.runtime_generation(),
        grant_epoch: fences.grant_epoch(),
        role_epoch: fences.role_epoch(),
        runtime_principal: access.runtime_principal().to_owned(),
        pool_alias: access.pool_alias().to_owned(),
        effective_budgets: Some(StorageEffectiveBudgetsV1 {
            max_connections: u32::from(access.effective_budgets().max_connections()),
            statement_timeout_millis: access.effective_budgets().statement_timeout_millis(),
        }),
        credential_lease_revision: fences.credential_lease_revision(),
        storage_bundle_revision: fences.storage_bundle_revision(),
        storage_bundle_digest: access.storage_bundle_digest().to_vec(),
    }
}

fn respond_platform_credential(
    kernel: &mut UnixStream,
    vault: &VaultResponseRecipientV1,
    purpose: &str,
    secret: &[u8],
) {
    let route = read_route(kernel);
    let lease = command(vault, &route).issue_lease(purpose);
    write_response(kernel, &route, lease.as_bytes());
    let route = read_route(kernel);
    assert!(matches!(
        command(vault, &route),
        VaultTransportCommandV1::ResolveLease { .. }
    ));
    write_response(kernel, &route, secret);
}

fn revoke(kernel: &mut UnixStream, vault: &VaultResponseRecipientV1, binding: StorageBindingV1) {
    let request = StorageRuntimeControlRequestV1 {
        operation: Some(StorageOperation::RevokeBinding(
            RevokeStorageBindingRequestV1 {
                binding: Some(binding.clone()),
            },
        )),
    };
    write_frame(kernel, &request.encode_to_vec());
    respond_platform_credential(
        kernel,
        vault,
        "storage.control.pgbouncer.admin",
        &read_credential(PGBOUNCER_PASSWORD_FILE_ENV),
    );
    respond_platform_credential(
        kernel,
        vault,
        "storage.control.postgres.admin",
        &read_credential(POSTGRES_PASSWORD_FILE_ENV),
    );
    let route = read_route(kernel);
    assert!(matches!(
        command(vault, &route),
        VaultTransportCommandV1::RevokeAudience
    ));
    write_response(kernel, &route, &[1]);
    let response = StorageRuntimeControlResponseV1::decode(read_frame(kernel).as_slice())
        .expect("Storage revoke response");
    assert!(
        matches!(response.result, Some(StorageResult::RevokedBinding(actual)) if actual == binding)
    );
}

fn serve_descriptor(kernel: &mut UnixStream) {
    let _ = ManagedRuntimeControlRequestV1::decode(read_frame(kernel).as_slice())
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

fn assert_reconciling(kernel: &mut UnixStream) {
    let request = StorageRuntimeControlRequestV1 {
        operation: Some(StorageOperation::GetStatus(Default::default())),
    };
    write_frame(kernel, &request.encode_to_vec());
    let response = StorageRuntimeControlResponseV1::decode(read_frame(kernel).as_slice())
        .expect("Storage status response");
    assert!(matches!(response.result, Some(StorageResult::Status(value))
        if value.state == StorageRuntimeStateV1::Reconciling as i32 && value.active_bindings.is_empty()));
}

fn command(
    vault: &VaultResponseRecipientV1,
    route: &makosh_runtime_protocol::v1::VaultCiphertextRouteV1,
) -> VaultTransportCommandV1 {
    let frame = VaultCiphertextFrameV1::from_parts(
        route.hpke_encapped_key.clone(),
        route.ciphertext.clone(),
        route.hpke_authentication_tag.clone(),
    )
    .expect("request frame");
    let plaintext = vault
        .open(
            &transport_binding(route, VaultTransportDirectionV1::ToVault),
            &frame,
        )
        .expect("open request");
    VaultTransportCommandV1::decode(plaintext.as_slice()).expect("typed Vault command")
}

fn write_response(
    kernel: &mut UnixStream,
    route: &makosh_runtime_protocol::v1::VaultCiphertextRouteV1,
    plaintext: &[u8],
) {
    let key: [u8; 32] = route
        .response_recipient_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .expect("recipient key");
    let frame = seal(
        &VaultTransportPublicKey::from_bytes(key).expect("recipient public key"),
        &transport_binding(route, VaultTransportDirectionV1::FromVault),
        plaintext,
    )
    .expect("encrypted response");
    let response = ManagedRuntimeVaultRouteResponseV1 {
        response: Some(VaultCiphertextResponseV1 {
            major: 1,
            vault_runtime_generation: route.vault_runtime_generation,
            caller_runtime_generation: route.caller_runtime_generation,
            request_id: route.request_id.clone(),
            operation_digest_sha256: route.operation_digest_sha256.clone(),
            direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
            hpke_encapped_key: frame.encapped_key().to_vec(),
            ciphertext: frame.ciphertext().to_vec(),
            hpke_authentication_tag: frame.tag().to_vec(),
        }),
        error_code: String::new(),
    };
    write_frame(
        kernel,
        &ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::VaultRoute(response)),
            error_code: String::new(),
        }
        .encode_to_vec(),
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
            .expect("response key"),
    )
    .expect("transport binding")
}

fn read_credential(name: &str) -> Vec<u8> {
    let mut value = std::fs::read(required(name)).expect("credential bytes");
    while matches!(value.last(), Some(b'\n' | b'\r')) {
        value.pop();
    }
    value
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("missing {name}"))
}
fn port(name: &str) -> u16 {
    required(name)
        .parse()
        .unwrap_or_else(|_| panic!("invalid {name}"))
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = 0_usize;
    for shift in (0..35).step_by(7) {
        let mut byte = [0; 1];
        stream.read_exact(&mut byte).expect("frame length");
        length |= usize::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let mut bytes = vec![0; length];
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
    stream
        .write_all(&[length as u8])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .expect("write frame");
}

trait LeaseId {
    fn issue_lease(&self, expected_purpose: &str) -> String;
}

impl LeaseId for VaultTransportCommandV1 {
    fn issue_lease(&self, expected_purpose: &str) -> String {
        match self {
            Self::IssueLease { request } if request.purpose().purpose_id() == expected_purpose => {
                if expected_purpose.contains("pgbouncer") {
                    "11111111111111111111111111111111"
                } else {
                    "22222222222222222222222222222222"
                }
                .to_owned()
            }
            _ => panic!("unexpected Vault credential request"),
        }
    }
}
