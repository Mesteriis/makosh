use makosh_runtime_protocol::{
    v1::{
        BlobRuntimeConfigurationV1, BlobRuntimeControlResponseV1, BlobRuntimeStateV1,
        BlobRuntimeStatusV1, blob_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::blob::{
        validate_blob_runtime_configuration, validate_blob_runtime_control_response,
    },
};

#[test]
fn blob_managed_runtime_contract_excludes_content_and_requires_current_vault_fence() {
    let configuration = BlobRuntimeConfigurationV1 {
        data_dir: "/private/makosh/blob".to_owned(),
        maximum_blob_bytes: 64 * 1024 * 1024,
        vault_instance_id: "instance_blob".to_owned(),
        vault_runtime_generation: 9,
        vault_hpke_public_key_x25519: vec![3; 32],
        data_socket_path: "/private/makosh/runtime/blob/data.sock".to_owned(),
        kernel_instance_id: "instance_blob".to_owned(),
        kernel_authorization_public_key_sec1: vec![4; 65],
        custody_release_grace_period_ms: 24 * 60 * 60 * 1_000,
    };
    assert!(validate_blob_runtime_configuration(&configuration).is_ok());
    let status = BlobRuntimeStatusV1 {
        state: BlobRuntimeStateV1::Ready as i32,
        runtime_generation: 4,
        vault_runtime_generation: 9,
        maximum_blob_bytes: 64 * 1024 * 1024,
        blocker_code: String::new(),
    };
    assert!(
        validate_blob_runtime_control_response(&BlobRuntimeControlResponseV1 {
            result: Some(ResponseResult::Status(status)),
            error_code: String::new(),
        })
        .is_ok()
    );
}

#[test]
fn blob_runtime_rejects_unbounded_configuration_or_status() {
    let configuration = BlobRuntimeConfigurationV1 {
        data_dir: "relative".to_owned(),
        maximum_blob_bytes: 64 * 1024 * 1024 + 1,
        vault_instance_id: String::new(),
        vault_runtime_generation: 0,
        vault_hpke_public_key_x25519: vec![3; 31],
        data_socket_path: "relative.sock".to_owned(),
        kernel_instance_id: String::new(),
        kernel_authorization_public_key_sec1: vec![4; 64],
        custody_release_grace_period_ms: 0,
    };
    assert!(validate_blob_runtime_configuration(&configuration).is_err());
    let status = BlobRuntimeStatusV1 {
        state: BlobRuntimeStateV1::Ready as i32,
        runtime_generation: 1,
        vault_runtime_generation: 1,
        maximum_blob_bytes: 1,
        blocker_code: "not_empty".to_owned(),
    };
    assert!(
        validate_blob_runtime_control_response(&BlobRuntimeControlResponseV1 {
            result: Some(ResponseResult::Status(status)),
            error_code: String::new(),
        })
        .is_err()
    );
}
