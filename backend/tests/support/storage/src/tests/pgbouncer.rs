use makosh_storage_control::{
    StorageFenceOutcomeV1, StoragePoolFenceCommandV1, StoragePoolFencePortV1,
};
use makosh_storage_pgbouncer::{
    PgBouncerAdminPortV1, PgBouncerPoolFenceAdapterV1, PgBouncerRuntimeConfigV1, PoolAliasV1,
    PoolConfigErrorV1, PoolLifecycleCommandV1, PoolLifecycleOutcomeV1, PoolRevokePlanV1,
};

use super::fixtures::storage_role_binding;

#[test]
fn fences_a_revoked_pool_before_killing_sessions() {
    assert_eq!(
        PoolRevokePlanV1::commands(),
        [
            PoolLifecycleCommandV1::Pause,
            PoolLifecycleCommandV1::Disable,
            PoolLifecycleCommandV1::Kill
        ]
    );
}

#[test]
fn renders_a_generation_scoped_transaction_pool_without_credentials() {
    let alias = PoolAliasV1::new("registration_notes", 3).expect("valid alias");
    let config = PgBouncerRuntimeConfigV1::new(
        alias.clone(),
        "127.0.0.1".into(),
        5_432,
        "makosh".into(),
        "runtime_notes".into(),
        8,
    )
    .expect("valid pool config");

    let entry = config.render_database_entry();

    assert!(entry.contains("runtime_registration_notes_3"));
    assert!(entry.contains("pool_mode=transaction"));
    assert!(entry.contains("max_db_client_connections=8"));
    assert!(!entry.contains("password="));
    assert_eq!(
        PoolLifecycleCommandV1::Kill.render(&alias),
        Ok("KILL runtime_registration_notes_3".into())
    );
}

#[test]
fn derives_a_safe_alias_for_a_hyphenated_registration_id() {
    let alias = PoolAliasV1::new("communications-runtime", 1).expect("valid alias");

    assert_eq!(
        alias.as_str(),
        makosh_storage_protocol::storage_runtime_pool_alias("communications-runtime", 1),
    );
    assert!(
        alias
            .as_str()
            .bytes()
            .all(|byte| { byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' })
    );
}

#[test]
fn rejects_config_that_cannot_be_safely_rendered() {
    let alias = PoolAliasV1::new("registration_notes", 1).expect("valid alias");
    let result = PgBouncerRuntimeConfigV1::new(
        alias,
        "db.example.com; KILL all".into(),
        5_432,
        "makosh".into(),
        "runtime_notes".into(),
        1,
    );

    assert_eq!(result, Err(PoolConfigErrorV1::Endpoint));
}

#[test]
fn forwards_only_the_fenced_pool_commands_for_the_bound_runtime() {
    let mut adapter = PgBouncerPoolFenceAdapterV1::new(RecordingAdmin::default());
    let binding = storage_role_binding("notes", "runtime_notes");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("test runtime");

    assert_eq!(
        runtime.block_on(adapter.apply_pool_fence(&binding, StoragePoolFenceCommandV1::Pause)),
        StorageFenceOutcomeV1::Applied
    );
    assert_eq!(
        runtime.block_on(adapter.apply_pool_fence(&binding, StoragePoolFenceCommandV1::Disable)),
        StorageFenceOutcomeV1::Applied
    );
    assert_eq!(
        runtime.block_on(adapter.apply_pool_fence(&binding, StoragePoolFenceCommandV1::Kill)),
        StorageFenceOutcomeV1::Applied
    );
    assert_eq!(
        adapter.into_inner().commands,
        [
            "PAUSE runtime_registration_notes_1",
            "DISABLE runtime_registration_notes_1",
            "KILL runtime_registration_notes_1"
        ]
    );
}

#[test]
fn an_already_absent_pool_is_an_idempotent_fence() {
    let mut adapter =
        PgBouncerPoolFenceAdapterV1::for_configuration_state(RecordingAdmin::default(), false);
    let binding = storage_role_binding("notes", "runtime_notes");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("test runtime");

    for command in [
        StoragePoolFenceCommandV1::Pause,
        StoragePoolFenceCommandV1::Disable,
        StoragePoolFenceCommandV1::Kill,
    ] {
        assert_eq!(
            runtime.block_on(adapter.apply_pool_fence(&binding, command)),
            StorageFenceOutcomeV1::Applied
        );
    }
    assert!(adapter.into_inner().commands.is_empty());
}

#[derive(Default)]
struct RecordingAdmin {
    commands: Vec<String>,
}

impl PgBouncerAdminPortV1 for RecordingAdmin {
    fn execute_pool_command(
        &mut self,
        command: &str,
    ) -> impl std::future::Future<Output = PoolLifecycleOutcomeV1> + Send {
        self.commands.push(command.into());
        async { PoolLifecycleOutcomeV1::Applied }
    }
}
