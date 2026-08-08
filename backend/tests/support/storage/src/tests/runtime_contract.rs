use makosh_storage_protocol::{
    v1::{
        ApplyStorageBindingRequestV1, GetStorageRuntimeStatusRequestV1,
        RevokeStorageBindingRequestV1, StorageBindingV1, StorageBundleV1,
        StorageEffectiveBudgetsV1, StorageMigrationStepV1, StorageRuntimeConfigurationV1,
        StorageRuntimeControlRequestV1, StorageRuntimeControlResponseV1, StorageRuntimeStateV1,
        StorageRuntimeStatusV1, storage_runtime_control_request_v1::Operation as RequestOperation,
        storage_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::{
        StorageRuntimeConfigurationErrorV1, StorageRuntimeControlErrorV1,
        StorageRuntimeStatusErrorV1, StorageRuntimeTopologyErrorV1,
        validate_storage_runtime_configuration, validate_storage_runtime_control_request,
        validate_storage_runtime_control_response, validate_storage_runtime_status,
        validate_storage_runtime_topology,
    },
};
use sha2::Digest;

#[test]
fn accepts_unconfigured_and_fenced_ready_storage_statuses() {
    assert_eq!(validate_storage_runtime_status(&unconfigured()), Ok(()));
    assert_eq!(validate_storage_runtime_status(&ready()), Ok(()));
}

#[test]
fn accepts_only_a_complete_non_secret_runtime_topology() {
    assert_eq!(validate_storage_runtime_topology(&topology()), Ok(()));
    let mut malformed = topology();
    malformed.postgres_artifact_sha256 = vec![0; 32];
    assert_eq!(
        validate_storage_runtime_topology(&malformed),
        Err(StorageRuntimeTopologyErrorV1::InvalidArtifactDigest)
    );
    let mut malformed = topology();
    malformed.pgbouncer_port = 0;
    assert_eq!(
        validate_storage_runtime_topology(&malformed),
        Err(StorageRuntimeTopologyErrorV1::InvalidEndpoint)
    );
}

#[test]
fn runtime_configuration_requires_a_current_non_secret_vault_route() {
    assert_eq!(
        validate_storage_runtime_configuration(&configuration()),
        Ok(())
    );
    let mut malformed = configuration();
    malformed.vault_hpke_public_key_x25519 = vec![0; 32];
    assert_eq!(
        validate_storage_runtime_configuration(&malformed),
        Err(StorageRuntimeConfigurationErrorV1::InvalidVaultContext)
    );
}

#[test]
fn runtime_configuration_accepts_only_topology_bound_bindings_and_a_private_path() {
    let mut configured = configuration();
    configured.desired_bindings = vec![binding()];
    configured.pgbouncer_database_config_path = "/private/storage/databases.ini".to_owned();
    configured.pgbouncer_auth_file_path = "/private/storage/users.txt".to_owned();
    assert_eq!(validate_storage_runtime_configuration(&configured), Ok(()));

    configured.pgbouncer_database_config_path = "relative/databases.ini".to_owned();
    assert_eq!(
        validate_storage_runtime_configuration(&configured),
        Err(StorageRuntimeConfigurationErrorV1::InvalidPgBouncerConfigPath)
    );

    let mut mismatched = configuration();
    let mut binding = binding();
    binding.storage_generation = 2;
    mismatched.desired_bindings = vec![binding];
    mismatched.pgbouncer_database_config_path = "/private/storage/databases.ini".to_owned();
    mismatched.pgbouncer_auth_file_path = "/private/storage/users.txt".to_owned();
    assert_eq!(
        validate_storage_runtime_configuration(&mismatched),
        Err(StorageRuntimeConfigurationErrorV1::InvalidBinding)
    );
}

#[test]
fn rejects_ready_status_without_an_exact_storage_binding() {
    let mut status = ready();
    status.active_bindings.clear();
    assert_eq!(
        validate_storage_runtime_status(&status),
        Err(StorageRuntimeStatusErrorV1::InvalidBinding)
    );
}

#[test]
fn rejects_failed_status_without_a_bounded_blocker_code() {
    assert_eq!(
        validate_storage_runtime_status(&StorageRuntimeStatusV1 {
            state: StorageRuntimeStateV1::Failed as i32,
            runtime_generation: 1,
            vault_runtime_generation: 1,
            ..Default::default()
        }),
        Err(StorageRuntimeStatusErrorV1::InvalidBlocker)
    );
}

#[test]
fn control_contract_accepts_status_or_an_exact_binding_revocation() {
    assert_eq!(
        validate_storage_runtime_control_request(&StorageRuntimeControlRequestV1 {
            operation: Some(RequestOperation::GetStatus(
                GetStorageRuntimeStatusRequestV1 {},
            )),
        }),
        Ok(())
    );
    assert_eq!(
        validate_storage_runtime_control_request(&StorageRuntimeControlRequestV1 {
            operation: Some(RequestOperation::ApplyBinding(
                ApplyStorageBindingRequestV1 {
                    binding: Some(binding()),
                    bundle: Some(bundle()),
                }
            )),
        }),
        Ok(())
    );
    assert_eq!(
        validate_storage_runtime_control_request(&StorageRuntimeControlRequestV1 {
            operation: Some(RequestOperation::RevokeBinding(
                RevokeStorageBindingRequestV1 {
                    binding: Some(binding()),
                },
            )),
        }),
        Ok(())
    );
    assert_eq!(
        validate_storage_runtime_control_request(&StorageRuntimeControlRequestV1::default()),
        Err(StorageRuntimeControlErrorV1::MissingOperation)
    );
}

#[test]
fn control_response_cannot_mix_status_and_error_or_omit_both() {
    assert_eq!(
        validate_storage_runtime_control_response(&StorageRuntimeControlResponseV1 {
            result: Some(ResponseResult::RevokedBinding(binding())),
            error_code: String::new(),
        }),
        Ok(())
    );
    assert_eq!(
        validate_storage_runtime_control_response(&StorageRuntimeControlResponseV1 {
            result: Some(ResponseResult::ActiveBinding(binding())),
            error_code: String::new(),
        }),
        Ok(())
    );
    assert_eq!(
        validate_storage_runtime_control_response(&StorageRuntimeControlResponseV1 {
            result: Some(ResponseResult::Status(unconfigured())),
            error_code: String::new(),
        }),
        Ok(())
    );
    assert_eq!(
        validate_storage_runtime_control_response(&StorageRuntimeControlResponseV1::default()),
        Err(StorageRuntimeControlErrorV1::InvalidError)
    );
}

fn unconfigured() -> StorageRuntimeStatusV1 {
    StorageRuntimeStatusV1 {
        state: StorageRuntimeStateV1::Unconfigured as i32,
        runtime_generation: 1,
        ..Default::default()
    }
}

fn ready() -> StorageRuntimeStatusV1 {
    StorageRuntimeStatusV1 {
        state: StorageRuntimeStateV1::Ready as i32,
        storage_generation: 1,
        topology_revision: 1,
        active_bindings: vec![binding()],
        blocker_code: String::new(),
        runtime_generation: 1,
        vault_runtime_generation: 1,
    }
}

fn topology() -> makosh_storage_protocol::v1::StorageRuntimeTopologyV1 {
    makosh_storage_protocol::v1::StorageRuntimeTopologyV1 {
        topology_revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".into(),
        database_id: "makosh".into(),
        deployment_profile:
            makosh_storage_protocol::v1::StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
        postgres_artifact_sha256: vec![1; 32],
        pgbouncer_artifact_sha256: vec![2; 32],
        postgres_host: "127.0.0.1".to_owned(),
        postgres_port: 5_432,
        pgbouncer_host: "127.0.0.1".to_owned(),
        pgbouncer_port: 6_432,
        pgbouncer_postgres_host: "127.0.0.1".to_owned(),
        pgbouncer_postgres_port: 5_432,
    }
}

fn configuration() -> StorageRuntimeConfigurationV1 {
    StorageRuntimeConfigurationV1 {
        topology: Some(topology()),
        vault_instance_id: "vault_main".to_owned(),
        vault_runtime_generation: 1,
        vault_hpke_public_key_x25519: vec![3; 32],
        desired_bindings: Vec::new(),
        pgbouncer_database_config_path: String::new(),
        desired_bundles: Vec::new(),
        pgbouncer_auth_file_path: String::new(),
    }
}

fn binding() -> StorageBindingV1 {
    StorageBindingV1 {
        storage_instance_id: "storage_main".into(),
        storage_generation: 1,
        database_id: "makosh".into(),
        owner: "notes".into(),
        registration_id: "registration_notes".into(),
        runtime_instance_id: "runtime_notes".into(),
        runtime_generation: 1,
        grant_epoch: 1,
        role_epoch: 1,
        runtime_principal: "runtime_notes".into(),
        pool_alias: "runtime_registration_notes_1".into(),
        effective_budgets: Some(StorageEffectiveBudgetsV1 {
            max_connections: 4,
            statement_timeout_millis: 5_000,
        }),
        credential_lease_revision: 1,
        storage_bundle_revision: 1,
        storage_bundle_digest: vec![1; 32],
    }
}

fn bundle() -> StorageBundleV1 {
    let sql = b"CREATE TABLE scheduler_test (id bigint);".to_vec();
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "scheduler_state".to_owned(),
        owner_id: "notes".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "scheduler_initial".to_owned(),
            sha256: sha2::Sha256::digest(&sql).to_vec(),
            forward_sql_utf8: sql,
        }],
    }
}
