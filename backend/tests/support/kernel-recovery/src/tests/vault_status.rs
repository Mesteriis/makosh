use super::common::*;
use makosh_runtime_protocol::v1::{
    GetVaultRuntimeStatusRequestV1, ManagedVaultRuntimeControlRequestV1,
    ManagedVaultRuntimeControlResponseV1, VaultRuntimeStateV1, VaultRuntimeStatusV1,
    managed_vault_runtime_control_request_v1::Operation,
    managed_vault_runtime_control_response_v1::Result as ResponseResult,
};

use crate::platform::vault::status::parse_current;

#[test]
fn managed_vault_status_requires_the_exact_live_generation_and_hpke_key() {
    let status = parse_current(ready_response(8), 8).expect("current managed Vault status");

    assert_eq!(status.runtime_generation(), 8);
    assert_eq!(status.hpke_public_key_x25519(), &[3; 32]);
    assert!(parse_current(ready_response(7), 8).is_err());
}

#[test]
fn managed_vault_status_rejects_error_or_nonready_responses() {
    let error = ManagedVaultRuntimeControlResponseV1 {
        result: None,
        error_code: "operation_not_available".to_owned(),
    };
    assert!(parse_current(error, 8).is_err());

    let sealed = ManagedVaultRuntimeControlResponseV1 {
        result: Some(ResponseResult::Status(VaultRuntimeStatusV1 {
            state: VaultRuntimeStateV1::Sealed as i32,
            vault_runtime_generation: 8,
            hpke_public_key_x25519: Vec::new(),
            blocker_code: "vault_sealed".to_owned(),
        })),
        error_code: String::new(),
    };
    assert!(parse_current(sealed, 8).is_err());
}

#[test]
fn kernel_reads_managed_vault_status_over_the_descriptor_bound_channel() {
    let (root, staged, expectation) = status_child_fixture();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    supervisor
        .start(
            "vault".to_owned(),
            staged,
            expectation,
            ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                .expect("managed execution policy"),
        )
        .expect("start managed Vault child");

    let response = supervisor
        .relay("vault", status_request().encode_to_vec())
        .expect("managed status relay");
    let response = ManagedVaultRuntimeControlResponseV1::decode(response.as_slice())
        .expect("typed managed status response");
    assert_eq!(
        parse_current(response, 5)
            .expect("current status")
            .runtime_generation(),
        5
    );

    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed Vault child");
    std::fs::remove_dir_all(root).expect("remove status fixture");
}

fn ready_response(generation: u64) -> ManagedVaultRuntimeControlResponseV1 {
    ManagedVaultRuntimeControlResponseV1 {
        result: Some(ResponseResult::Status(VaultRuntimeStatusV1 {
            state: VaultRuntimeStateV1::Ready as i32,
            vault_runtime_generation: generation,
            hpke_public_key_x25519: vec![3; 32],
            blocker_code: String::new(),
        })),
        error_code: String::new(),
    }
}

fn status_child_fixture() -> (
    std::path::PathBuf,
    staged_native_artifact::StagedNativeArtifact,
    ManagedRuntimeExpectation,
) {
    let root = unique_target_root("makosh-managed-vault-status");
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "vault".into(),
        owner_id: "vault".into(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let expectation = ManagedRuntimeExpectation::new(
        "vault",
        "vault-runtime",
        "vault",
        5,
        7,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let source = root.join("managed-status-child.sh");
    std::fs::create_dir_all(&root).expect("create status fixture");
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes,
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    let request_length = frame(&status_request().encode_to_vec()).len();
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\ndd bs=1 count={request_length} of=/dev/null 2>/dev/null\nprintf '{}' >&0\nsleep 30\n",
            shell_binary_literal(&frame(&describe.encode_to_vec())),
            shell_binary_literal(&frame(&ready_response(5).encode_to_vec())),
        ),
    )
    .expect("write status child");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&source).expect("read status child")).into();
    let staged =
        staged_native_artifact::stage(&source, &root.join("launch"), "status-child", &digest)
            .expect("stage status child");
    (root, staged, expectation)
}

fn status_request() -> ManagedVaultRuntimeControlRequestV1 {
    ManagedVaultRuntimeControlRequestV1 {
        operation: Some(Operation::GetStatus(GetVaultRuntimeStatusRequestV1 {})),
    }
}

fn frame(bytes: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(bytes.len() + 5);
    let mut length = u32::try_from(bytes.len()).expect("bounded status frame");
    while length >= 0x80 {
        frame.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    frame.push(length as u8);
    frame.extend_from_slice(bytes);
    frame
}
