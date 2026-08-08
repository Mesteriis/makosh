use super::common::*;
use makosh_kernel_control_store::{
    PlatformManagedProcessBinding, PlatformManagedProcessLaunch, PlatformStorageBindingInputV1,
    PlatformStorageBindingV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{ManagedRuntimeVaultRouteRequestV1, VaultCiphertextResponseV1};
use makosh_runtime_protocol::validation::vault::STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1;
use makosh_vault_protocol::VaultTransportCommandV1;
use std::io::{Read, Write};

use crate::runtime::lifecycle::control::{
    ManagedRuntimeVaultRouteHandler, relay_with_vault_routes,
};
use crate::runtime::lifecycle::fence::current_platform_managed_runtime_matches;

#[test]
fn runtime_protocol_keeps_the_revoking_storage_route_fence_exact() {
    assert_eq!(
        STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1,
        VaultTransportCommandV1::RevokeAudience.operation_digest()
    );
}

#[test]
fn platform_managed_runtime_fence_rejects_a_replaced_binding() {
    let root = unique_target_root("makosh-platform-runtime-fence");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .record_platform_managed_process_binding(&platform_binding(1))
        .expect("record platform binding");
    store
        .record_platform_managed_process_launch(&PlatformManagedProcessLaunch::new(
            "storage-control",
            1,
            1,
            4,
            3,
        ))
        .expect("record platform launch");
    assert_eq!(
        current_platform_managed_runtime_matches(
            &store,
            "storage-control",
            "storage-control",
            4,
            3,
        )
        .expect("check current platform runtime"),
        Some(true),
    );

    store
        .record_platform_managed_process_binding(&platform_binding(2))
        .expect("replace platform binding");
    assert_eq!(
        current_platform_managed_runtime_matches(
            &store,
            "storage-control",
            "storage-control",
            4,
            3,
        )
        .expect("check replaced platform runtime"),
        Some(false),
    );

    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn current_storage_can_revoke_an_exact_reserved_binding_after_target_grants_are_fenced() {
    let root = unique_target_root("makosh-storage-revoking-vault-route");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .record_platform_managed_process_binding(&PlatformManagedProcessBinding::new(
            "storage",
            1,
            "distribution-storage",
            "storage-runtime",
            [7; 32],
            [3; 32],
            None,
        ))
        .expect("record Storage process binding");
    store
        .record_platform_managed_process_launch(&PlatformManagedProcessLaunch::new(
            "storage", 1, 1, 4, 3,
        ))
        .expect("record Storage process launch");
    let binding = PlatformStorageBindingV1::new(PlatformStorageBindingInputV1 {
        registration_id: "mail-registration".to_owned(),
        capability_id: "mail.storage.v1".to_owned(),
        owner_id: "mail".to_owned(),
        binding_revision: 1,
        topology_revision: 1,
        storage_generation: 1,
        runtime_instance_id: "mail-runtime".to_owned(),
        runtime_generation: 5,
        grant_epoch: 7,
        role_epoch: 11,
        runtime_principal: "mail_runtime".to_owned(),
        connection_budget: 4,
        statement_timeout_millis: 5_000,
        credential_lease_revision: 13,
        storage_bundle_revision: 1,
        storage_bundle_digest: [8; 32],
    })
    .expect("valid Mail Storage binding");
    store
        .record_platform_storage_binding(&binding)
        .expect("record Mail Storage binding");
    store
        .begin_platform_storage_binding_revocation(
            binding.registration_id(),
            binding.capability_id(),
            binding.binding_revision(),
        )
        .expect("reserve Mail Storage binding revocation");
    let expectation =
        ManagedRuntimeExpectation::new("storage", "storage", "storage", 4, 3, [3; 32], None);
    let route = revoking_storage_route(&binding);
    assert_eq!(
        current_platform_managed_runtime_matches(
            &store,
            expectation.registration_id(),
            expectation.runtime_instance_id(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .expect("check current Storage runtime"),
        Some(true),
    );
    let reserved = store
        .platform_storage_binding(binding.registration_id(), binding.capability_id())
        .expect("read reserved Mail Storage binding")
        .expect("reserved Mail Storage binding");
    assert_eq!(reserved.runtime_instance_id(), route.runtime_instance_id);
    assert_eq!(
        reserved.runtime_generation(),
        route.caller_runtime_generation
    );
    assert_eq!(reserved.grant_epoch(), route.grant_epoch);
    assert_eq!(reserved.role_epoch(), route.storage_role_epoch);
    assert_eq!(
        reserved.credential_lease_revision(),
        route.storage_credential_lease_revision
    );
    assert_eq!(
        reserved.runtime_principal(),
        route.storage_runtime_principal
    );
    assert_eq!(reserved.owner_id(), route.storage_owner_id);
    assert_eq!(
        reserved.state(),
        makosh_kernel_control_store::PlatformStorageBindingStateV1::Revoking
    );
    assert_eq!(
        route.operation_digest_sha256,
        STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1
    );
    assert_eq!(
        store
            .platform_storage_bindings()
            .expect("list exact Storage bindings"),
        vec![reserved]
    );

    crate::platform::vault::managed_route::authorize_storage_delegated_route(
        &store,
        &expectation,
        &route,
    )
    .expect("current Storage may revoke an exact durable reservation");

    let stale_storage =
        ManagedRuntimeExpectation::new("storage", "storage", "storage", 5, 3, [3; 32], None);
    assert!(
        crate::platform::vault::managed_route::authorize_storage_delegated_route(
            &store,
            &stale_storage,
            &route,
        )
        .is_err(),
        "a stale Storage generation cannot use the reserved revoke route"
    );
    let mut non_revoke = route;
    non_revoke.operation_digest_sha256 = vec![9; 32];
    assert!(
        crate::platform::vault::managed_route::authorize_storage_delegated_route(
            &store,
            &expectation,
            &non_revoke,
        )
        .is_err(),
        "a revoking binding authorizes only the exact RevokeAudience operation"
    );

    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn managed_runtime_routes_vault_ciphertext_only_after_descriptor_handshake() {
    let (root, staged, expectation) = route_child_fixture();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicU64::new(0));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    supervisor
        .configure_vault_route_handler(Arc::new(RecordingRouteHandler {
            calls: Arc::clone(&calls),
        }))
        .expect("configure handler before runtime launch");
    supervisor
        .start(
            "storage-control".to_owned(),
            staged,
            expectation,
            ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                .expect("managed execution policy"),
        )
        .expect("start managed runtime");

    assert!(
        wait_for_route(&calls),
        "typed route reaches the Kernel handler"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    std::fs::remove_dir_all(root).expect("remove route fixture");
}

#[test]
fn relay_completes_a_request_after_a_nested_vault_route() {
    let (mut kernel, mut child) = UnixStream::pair().expect("relay channel");
    let (root, _staged, expectation) = route_child_fixture();
    let calls = Arc::new(AtomicU64::new(0));
    let handler = RecordingRouteHandler {
        calls: Arc::clone(&calls),
    };
    let route = valid_route();
    let worker = std::thread::spawn(move || {
        assert_eq!(read_frame(&mut child), b"revoke");
        write_frame(&mut child, &managed_vault_route_request(route));
        let _ = read_frame(&mut child);
        write_bytes(&mut child, b"revoked");
    });

    assert_eq!(
        relay_with_vault_routes(&mut kernel, b"revoke", &expectation, Some(&handler))
            .expect("relay completes"),
        b"revoked"
    );
    assert_eq!(calls.load(Ordering::Acquire), 1);
    worker.join().expect("relay worker");
    std::fs::remove_dir_all(root).expect("remove relay fixture");
}

struct RecordingRouteHandler {
    calls: Arc<AtomicU64>,
}

fn platform_binding(binding_revision: u64) -> PlatformManagedProcessBinding {
    PlatformManagedProcessBinding::new(
        "storage-control",
        binding_revision,
        format!("distribution-{binding_revision}"),
        format!("storage-runtime-{binding_revision}"),
        [7; 32],
        [3; 32],
        None,
    )
}

impl ManagedRuntimeVaultRouteHandler for RecordingRouteHandler {
    fn route_vault_ciphertext(
        &self,
        expectation: &ManagedRuntimeExpectation,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, String> {
        if expectation.registration_id() != "storage-control"
            || route.registration_id != "storage-control"
            || route.caller_runtime_generation != expectation.runtime_generation()
            || route.grant_epoch != expectation.grant_epoch()
        {
            return Err("route fence mismatch".to_owned());
        }
        self.calls.fetch_add(1, Ordering::Release);
        Ok(VaultCiphertextResponseV1 {
            major: 1,
            vault_runtime_generation: route.vault_runtime_generation,
            caller_runtime_generation: route.caller_runtime_generation,
            request_id: route.request_id,
            operation_digest_sha256: route.operation_digest_sha256,
            direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
            hpke_encapped_key: vec![1; 32],
            ciphertext: vec![2],
            hpke_authentication_tag: vec![3; 16],
        })
    }
}

fn route_child_fixture() -> (
    std::path::PathBuf,
    staged_native_artifact::StagedNativeArtifact,
    ManagedRuntimeExpectation,
) {
    let root = unique_target_root("makosh-managed-vault-route");
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "storage".into(),
        owner_id: "platform".into(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let expectation = ManagedRuntimeExpectation::new(
        "storage-control",
        "storage-runtime",
        "storage",
        5,
        7,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let staged = stage_route_child(&root, &descriptor_bytes);
    (root, staged, expectation)
}

fn stage_route_child(
    root: &std::path::Path,
    descriptor_bytes: &[u8],
) -> staged_native_artifact::StagedNativeArtifact {
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes: descriptor_bytes.to_vec(),
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    let route = ManagedRuntimeVaultRouteRequestV1 {
        route: Some(valid_route()),
    };
    let payload = route_child_payload(describe, route);
    let source = root.join("managed-route-child.sh");
    std::fs::create_dir_all(root).expect("create route fixture");
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\nsleep 30\n",
            shell_binary_literal(&payload)
        ),
    )
    .expect("write route child");
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&source).expect("read route child")).into();
    staged_native_artifact::stage(&source, &root.join("launch"), "route-child", &digest)
        .expect("stage route child")
}

fn valid_route() -> VaultCiphertextRouteV1 {
    VaultCiphertextRouteV1 {
        major: 1,
        registration_id: "storage-control".into(),
        runtime_instance_id: "storage-runtime".into(),
        caller_runtime_generation: 5,
        vault_runtime_generation: 3,
        grant_epoch: 7,
        request_id: vec![1; 16],
        operation_digest_sha256: vec![2; 32],
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: vec![3; 32],
        ciphertext: vec![4],
        hpke_authentication_tag: vec![5; 16],
        response_recipient_hpke_public_key_x25519: vec![6; 32],
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    }
}

fn revoking_storage_route(binding: &PlatformStorageBindingV1) -> VaultCiphertextRouteV1 {
    VaultCiphertextRouteV1 {
        major: 1,
        registration_id: binding.registration_id().to_owned(),
        runtime_instance_id: binding.runtime_instance_id().to_owned(),
        caller_runtime_generation: binding.runtime_generation(),
        vault_runtime_generation: 1,
        grant_epoch: binding.grant_epoch(),
        request_id: vec![1; 16],
        operation_digest_sha256: STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1.to_vec(),
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: vec![2; 32],
        ciphertext: vec![3],
        hpke_authentication_tag: vec![4; 16],
        response_recipient_hpke_public_key_x25519: vec![5; 32],
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        storage_role_epoch: binding.role_epoch(),
        storage_credential_lease_revision: binding.credential_lease_revision(),
        storage_runtime_principal: binding.runtime_principal().to_owned(),
        storage_owner_id: binding.owner_id().to_owned(),
    }
}

fn route_child_payload(
    describe: ManagedRuntimeControlRequestV1,
    route: ManagedRuntimeVaultRouteRequestV1,
) -> Vec<u8> {
    [
        frame(&describe.encode_to_vec()),
        frame(
            &ManagedRuntimeControlRequestV1 {
                operation: Some(
                    makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::RouteVaultCiphertext(
                        route,
                    ),
                ),
            }
            .encode_to_vec(),
        ),
    ]
    .concat()
}

fn managed_vault_route_request(route: VaultCiphertextRouteV1) -> ManagedRuntimeControlRequestV1 {
    ManagedRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::RouteVaultCiphertext(
                ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
            ),
        ),
    }
}

fn frame(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len() + 5);
    let mut length = u32::try_from(bytes.len()).expect("bounded route frame");
    while length >= 0x80 {
        result.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    result.push(length as u8);
    result.extend_from_slice(bytes);
    result
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = 0_usize;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).expect("frame prefix");
        length |= usize::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let mut bytes = vec![0; length];
            stream.read_exact(&mut bytes).expect("frame body");
            return bytes;
        }
    }
    panic!("frame prefix exceeds bound");
}

fn write_frame(stream: &mut UnixStream, message: &impl Message) {
    write_bytes(stream, &message.encode_to_vec());
}

fn write_bytes(stream: &mut UnixStream, bytes: &[u8]) {
    stream.write_all(&frame(bytes)).expect("write frame");
    stream.flush().expect("flush frame");
}

fn wait_for_route(calls: &AtomicU64) -> bool {
    // Process spawn and the inherited-FD handshake are intentionally external to
    // this test process. Keep the assertion bounded, but do not turn ordinary
    // scheduler contention from the parallel recovery suite into a route failure.
    for _ in 0..200 {
        if calls.load(Ordering::Acquire) == 1 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}
