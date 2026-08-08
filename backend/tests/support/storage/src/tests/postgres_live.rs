use makosh_storage_postgres::{
    PostgresAdapterErrorV1, PostgresAdminConnectorV1, PostgresRuntimeSessionProbeV1,
    StorageRoleSpecV1, apply_storage_bundle, ensure_platform_schemas, ensure_storage_roles,
    fence_postgres_runtime_role, read_readiness, read_storage_data_privilege_audit,
    read_storage_role_audit,
};
use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use sha2::{Digest, Sha256};

use super::fixtures::storage_role_binding;

const DATABASE_URL_ENV: &str = "MAKOSH_STORAGE_TEST_DATABASE_URL";

#[test]
#[ignore = "requires the disposable development PostgreSQL contour"]
fn reconciles_bootstrap_and_runtime_role_against_postgres() {
    let database_url = std::env::var(DATABASE_URL_ENV)
        .expect("storage integration test requires MAKOSH_STORAGE_TEST_DATABASE_URL");
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async move {
        let connector = PostgresAdminConnectorV1::connect(&database_url)
            .await
            .expect("connect development PostgreSQL");
        ensure_platform_schemas(&connector)
            .await
            .expect("reconcile platform schemas");
        let spec = StorageRoleSpecV1::from_binding(
            "storage_ddl_privilege_probe".into(),
            storage_role_binding("storage_privilege_probe", "storage_runtime_privilege_probe"),
        )
        .expect("valid role specification");
        ensure_storage_roles(&connector, &spec)
            .await
            .expect("reconcile storage roles");
        assert_role_ledger_rejects_reused_principals(&connector).await;
        assert_postgres_runtime_fence(&database_url, &connector).await;

        let readiness = read_readiness(&connector).await.expect("readiness");
        assert_eq!(readiness.database_id(), "makosh_development");

        let audit = read_storage_role_audit(&connector, &spec)
            .await
            .expect("audit runtime role");
        assert!(audit.can_login);
        assert!(!audit.inherits_privileges);
        assert!(!audit.can_create_database);
        assert!(!audit.can_create_roles);
        assert!(!audit.is_superuser);
        assert!(!audit.bypasses_row_security);
        assert_eq!(audit.connection_limit, 8);
        assert!(audit.search_path_isolated);

        assert_ledger_reconciliation(&connector, &spec).await;
        assert_successor_bundle_inherits_exact_steps(&connector).await;
        assert_role_ledger_accepts_fenced_successor(&connector, &spec).await;
        assert_owner_data_privileges(&connector, &spec).await;
    });
}

#[test]
#[ignore = "requires the disposable authenticated PostgreSQL contour"]
fn successor_bundle_inherits_exact_steps_without_ddl_replay() {
    let database_url = std::env::var(DATABASE_URL_ENV)
        .expect("storage integration test requires MAKOSH_STORAGE_TEST_DATABASE_URL");
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async move {
        let connector = PostgresAdminConnectorV1::connect(&database_url)
            .await
            .expect("connect disposable PostgreSQL");
        ensure_platform_schemas(&connector)
            .await
            .expect("reconcile platform schemas");
        assert_successor_bundle_inherits_exact_steps(&connector).await;
    });
}

async fn assert_postgres_runtime_fence(database_url: &str, connector: &PostgresAdminConnectorV1) {
    let spec = revocation_role_spec();
    ensure_storage_roles(connector, &spec)
        .await
        .expect("create runtime principal for fencing");
    let runtime_url = development_runtime_url(database_url, spec.runtime_principal());
    let mut runtime_connection = PostgresRuntimeSessionProbeV1::connect(&runtime_url)
        .await
        .expect("open disposable runtime connection");
    let current_user = runtime_connection
        .current_principal()
        .await
        .expect("authenticate runtime principal");
    assert_eq!(current_user, spec.runtime_principal());

    let fence = fence_postgres_runtime_role(connector, &spec)
        .await
        .expect("fence PostgreSQL runtime role");
    assert!(fence.terminated_backend_count() >= 1);
    assert!(!runtime_connection.remains_connected().await);

    let audit = read_storage_role_audit(connector, &spec)
        .await
        .expect("audit fenced role");
    assert!(!audit.can_login);
}

fn revocation_role_spec() -> StorageRoleSpecV1 {
    let process_id = std::process::id();
    StorageRoleSpecV1::from_binding(
        format!("storage_ddl_revoke_{process_id}"),
        storage_role_binding(
            &format!("storage_revoke_probe_{process_id}"),
            &format!("storage_runtime_revoke_{process_id}"),
        ),
    )
    .expect("valid dynamic revocation role specification")
}

fn development_runtime_url(database_url: &str, runtime_principal: &str) -> String {
    let (_, endpoint) = database_url
        .split_once('@')
        .expect("development database URL has an authority separator");
    format!("postgres://{runtime_principal}@{endpoint}")
}

async fn assert_role_ledger_rejects_reused_principals(connector: &PostgresAdminConnectorV1) {
    let reused_ddl = StorageRoleSpecV1::from_binding(
        "storage_ddl_privilege_probe".into(),
        storage_role_binding("storage_other_probe", "storage_runtime_other_probe"),
    )
    .expect("valid reused DDL role specification");
    let reused_runtime = StorageRoleSpecV1::from_binding(
        "storage_ddl_yet_another_probe".into(),
        storage_role_binding(
            "storage_yet_another_probe",
            "storage_runtime_privilege_probe",
        ),
    )
    .expect("valid reused runtime role specification");

    assert_eq!(
        ensure_storage_roles(connector, &reused_ddl).await,
        Err(PostgresAdapterErrorV1::RoleBinding)
    );
    assert_eq!(
        ensure_storage_roles(connector, &reused_runtime).await,
        Err(PostgresAdapterErrorV1::RoleBinding)
    );
}

async fn assert_ledger_reconciliation(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) {
    let bundle = ledger_bundle();
    let mismatched_roles = StorageRoleSpecV1::from_binding(
        "storage_ddl_different_owner".into(),
        storage_role_binding("different_owner", "storage_runtime_different_owner"),
    )
    .expect("valid mismatched role specification");
    assert!(
        apply_storage_bundle(connector, &mismatched_roles, &bundle)
            .await
            .is_err()
    );
    apply_storage_bundle(connector, spec, &bundle)
        .await
        .expect("apply admitted bundle");
    apply_storage_bundle(connector, spec, &bundle)
        .await
        .expect("reapply exact ledgered bundle");
    let mut conflicting = bundle.clone();
    conflicting.steps[0].forward_sql_utf8 =
        b"CREATE TABLE makosh_data.storage_privilege_probe_conflict (probe_id uuid);".to_vec();
    conflicting.steps[0].sha256 = Sha256::digest(&conflicting.steps[0].forward_sql_utf8).to_vec();
    assert!(
        apply_storage_bundle(connector, spec, &conflicting)
            .await
            .is_err()
    );
}

async fn assert_successor_bundle_inherits_exact_steps(connector: &PostgresAdminConnectorV1) {
    let suffix = std::process::id();
    let owner = format!("storage_lineage_{suffix}");
    let runtime_principal = format!("storage_runtime_lineage_{suffix}");
    let spec = StorageRoleSpecV1::from_binding(
        format!("storage_ddl_lineage_{suffix}"),
        storage_role_binding(&owner, &runtime_principal),
    )
    .expect("valid successor lineage role specification");
    ensure_storage_roles(connector, &spec)
        .await
        .expect("reconcile successor lineage roles");

    let predecessor = lineage_bundle(&owner, 1, false);
    apply_storage_bundle(connector, &spec, &predecessor)
        .await
        .expect("apply predecessor bundle");

    let successor = lineage_bundle(&owner, 2, true);
    apply_storage_bundle(connector, &spec, &successor)
        .await
        .expect("inherit predecessor step and apply successor step");
    apply_storage_bundle(connector, &spec, &successor)
        .await
        .expect("reapply exact successor bundle");

    assert_eq!(
        apply_storage_bundle(connector, &spec, &predecessor).await,
        Err(PostgresAdapterErrorV1::Migration)
    );

    let mut drifted = successor;
    drifted.revision = 3;
    drifted.steps[0].forward_sql_utf8 =
        format!("CREATE TABLE makosh_data.{owner}_drifted (probe_id uuid);").into_bytes();
    drifted.steps[0].sha256 = Sha256::digest(&drifted.steps[0].forward_sql_utf8).to_vec();
    assert_eq!(
        apply_storage_bundle(connector, &spec, &drifted).await,
        Err(PostgresAdapterErrorV1::Migration)
    );
}

fn lineage_bundle(
    owner: &str,
    bundle_revision: u32,
    include_successor_step: bool,
) -> StorageBundleV1 {
    let initial_sql =
        format!("CREATE TABLE makosh_data.{owner}_initial (probe_id uuid);").into_bytes();
    let mut steps = vec![StorageMigrationStepV1 {
        revision: 1,
        migration_id: "create_initial".into(),
        sha256: Sha256::digest(&initial_sql).to_vec(),
        forward_sql_utf8: initial_sql,
    }];
    if include_successor_step {
        let successor_sql =
            format!("CREATE TABLE makosh_data.{owner}_successor (probe_id uuid);").into_bytes();
        steps.push(StorageMigrationStepV1 {
            revision: 2,
            migration_id: "create_successor".into(),
            sha256: Sha256::digest(&successor_sql).to_vec(),
            forward_sql_utf8: successor_sql,
        });
    }
    StorageBundleV1 {
        major: 1,
        revision: bundle_revision,
        bundle_id: format!("{owner}_bundle"),
        owner_id: owner.to_owned(),
        steps,
    }
}

async fn assert_role_ledger_accepts_fenced_successor(
    connector: &PostgresAdminConnectorV1,
    predecessor: &StorageRoleSpecV1,
) {
    let identity = predecessor.binding().identity();
    let successor_binding = StorageBindingV1::new(
        StorageBindingIdentityV1::new(
            identity.storage_instance_id().to_owned(),
            identity.database_id().to_owned(),
            identity.owner().to_owned(),
            identity.registration_id().to_owned(),
            format!("{}_successor", identity.runtime_instance_id()),
        )
        .expect("valid successor identity"),
        StorageBindingFencesV1::new(
            predecessor.binding().fences().storage_generation(),
            predecessor.binding().fences().runtime_generation() + 1,
            predecessor.binding().fences().grant_epoch(),
            predecessor.binding().fences().role_epoch() + 1,
            predecessor.binding().fences().credential_lease_revision() + 1,
            predecessor.binding().fences().storage_bundle_revision(),
        )
        .expect("valid successor fences"),
        StorageBindingAccessV1::new(
            format!("{}_successor", predecessor.runtime_principal()),
            format!(
                "runtime_{}_{}",
                identity.registration_id(),
                predecessor.binding().fences().runtime_generation() + 1,
            ),
            StorageEffectiveBudgetsV1::new(
                predecessor.max_connections(),
                predecessor.statement_timeout_millis(),
            )
            .expect("valid successor budgets"),
            [2; 32],
        )
        .expect("valid successor access"),
    )
    .expect("valid successor binding");
    let successor =
        StorageRoleSpecV1::from_binding(predecessor.ddl_owner().to_owned(), successor_binding)
            .expect("valid successor role specification");

    ensure_storage_roles(connector, &successor)
        .await
        .expect("admit fenced successor role");
    assert_eq!(
        ensure_storage_roles(connector, predecessor).await,
        Err(PostgresAdapterErrorV1::RoleBinding)
    );
    ensure_storage_roles(connector, &successor)
        .await
        .expect("reapply exact successor role");
}

async fn assert_owner_data_privileges(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) {
    let audit = read_storage_data_privilege_audit(connector, spec)
        .await
        .expect("audit owner data privileges");

    assert!(audit.owner_table_count > 0);
    assert!(audit.owner_tables_owned_by_ddl);
    assert!(audit.owner_tables_have_dml);
    assert_eq!(audit.foreign_tables_with_dml, 0);
}

fn ledger_bundle() -> StorageBundleV1 {
    let sql = b"CREATE TABLE makosh_data.storage_privilege_probe_ledger (probe_id uuid);".to_vec();
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "storage_privilege_probe_bundle".into(),
        owner_id: "storage_privilege_probe".into(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "create_privilege_probe".into(),
            sha256: Sha256::digest(&sql).to_vec(),
            forward_sql_utf8: sql,
        }],
    }
}
