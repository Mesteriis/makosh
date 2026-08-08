use makosh_runtime_protocol::v1::SchedulerRuntimeStorageBindingV1;
use makosh_scheduler_persistence::{
    SchedulerPostgresEndpointV1, SchedulerRecoveryDatabaseV1, SchedulerStoreConnectionErrorV1,
    scheduler_storage_binding_from_runtime,
};

#[test]
fn scheduler_postgres_endpoint_rejects_url_or_secret_shaped_hosts() {
    assert_eq!(
        SchedulerPostgresEndpointV1::new("user:password@localhost".to_owned(), 6432),
        Err(SchedulerStoreConnectionErrorV1::InvalidEndpoint)
    );
    assert_eq!(
        SchedulerPostgresEndpointV1::new("127.0.0.1".to_owned(), 0),
        Err(SchedulerStoreConnectionErrorV1::InvalidEndpoint)
    );
}

#[test]
fn scheduler_recovery_database_rejects_secret_or_url_shaped_arguments() {
    assert!(
        SchedulerRecoveryDatabaseV1::new(
            "127.0.0.1".to_owned(),
            5432,
            "makosh".to_owned(),
            "recovery".to_owned(),
            "verify-full",
        )
        .is_ok()
    );
    assert!(
        SchedulerRecoveryDatabaseV1::new(
            "user:secret@localhost".to_owned(),
            5432,
            "makosh".to_owned(),
            "recovery".to_owned(),
            "require",
        )
        .is_err()
    );
    assert!(
        SchedulerRecoveryDatabaseV1::new(
            "127.0.0.1".to_owned(),
            5432,
            "postgres://other".to_owned(),
            "recovery".to_owned(),
            "disable",
        )
        .is_err()
    );
    assert!(
        SchedulerRecoveryDatabaseV1::new(
            "127.0.0.1".to_owned(),
            5432,
            "makosh".to_owned(),
            "recovery".to_owned(),
            "unknown",
        )
        .is_err()
    );
}

#[test]
fn scheduler_storage_binding_matches_authenticated_runtime_fences() {
    let binding = scheduler_storage_binding_from_runtime(
        &configuration(),
        "scheduler_registration".to_owned(),
        "scheduler_runtime".to_owned(),
        4,
        7,
    )
    .expect("complete staged binding");
    assert_eq!(
        binding.identity().registration_id(),
        "scheduler_registration"
    );
    assert_eq!(binding.fences().runtime_generation(), 4);
    assert_eq!(binding.fences().grant_epoch(), 7);
}

fn configuration() -> SchedulerRuntimeStorageBindingV1 {
    SchedulerRuntimeStorageBindingV1 {
        database_id: "makosh_scheduler".to_owned(),
        pgbouncer_host: "127.0.0.1".to_owned(),
        pgbouncer_port: 6432,
        runtime_principal: "scheduler_runtime".to_owned(),
        storage_generation: 5,
        credential_revision: 3,
        storage_instance_id: "storage_main".to_owned(),
        owner: "platform".to_owned(),
        role_epoch: 6,
        pool_alias: "runtime_scheduler_registration_4".to_owned(),
        max_connections: 8,
        statement_timeout_millis: 30_000,
        storage_bundle_revision: 1,
        storage_bundle_digest: vec![9; 32],
    }
}
