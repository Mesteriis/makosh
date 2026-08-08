//! Per-step DDL and canonical ledger commits share one transaction.

use hermes_storage_migrations::admit_storage_bundle;
use hermes_storage_protocol::v1::StorageBundleV1;
use sqlx::{AssertSqlSafe, query, query_as, raw_sql};

use crate::{
    PostgresAdapterErrorV1, PostgresAdminConnectorV1, StorageRoleSpecV1,
    reconcile_owner_data_privileges,
};

pub async fn apply_storage_bundle(
    connector: &PostgresAdminConnectorV1,
    roles: &StorageRoleSpecV1,
    bundle: &StorageBundleV1,
) -> Result<(), PostgresAdapterErrorV1> {
    admit_storage_bundle(bundle).map_err(|error| {
        let digest = match error {
            hermes_storage_migrations::MigrationBundleAdmissionErrorV1::Step {
                revision, ..
            } => bundle
                .steps
                .iter()
                .find(|step| step.revision == revision)
                .map(|step| {
                    step.sha256
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                }),
            _ => None,
        };
        eprintln!(
            "developer_storage_migration_failure=admission error={error:?} digest={}",
            digest.as_deref().unwrap_or("unavailable")
        );
        PostgresAdapterErrorV1::Migration
    })?;
    if bundle.owner_id != roles.owner_id() {
        eprintln!("developer_storage_migration_failure=owner_mismatch");
        return Err(PostgresAdapterErrorV1::Migration);
    }
    for step in &bundle.steps {
        apply_step(connector, roles, bundle, step).await?;
    }
    reconcile_owner_data_privileges(connector, roles)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationPrivileges)
}

async fn apply_step(
    connector: &PostgresAdminConnectorV1,
    roles: &StorageRoleSpecV1,
    bundle: &StorageBundleV1,
    step: &hermes_storage_protocol::v1::StorageMigrationStepV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let mut transaction = connector.pool().begin().await.map_err(|_| {
        eprintln!("developer_storage_migration_failure=transaction_begin");
        PostgresAdapterErrorV1::Migration
    })?;
    let recorded_steps = read_recorded_steps(&mut transaction, bundle, step)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationLedgerRead)?;
    let lineage = classify_recorded_step_lineage(bundle.revision, &step.sha256, &recorded_steps)
        .inspect_err(|_| {
            eprintln!(
                "developer_storage_migration_failure=lineage step_revision={}",
                step.revision
            );
        })?;
    match lineage {
        RecordedStepLineageV1::Exact => {
            transaction
                .commit()
                .await
                .map_err(|_| PostgresAdapterErrorV1::MigrationCommit)?;
            return Ok(());
        }
        RecordedStepLineageV1::Predecessor => {
            record_step(&mut transaction, bundle, step)
                .await
                .map_err(|_| PostgresAdapterErrorV1::MigrationLedgerWrite)?;
            transaction
                .commit()
                .await
                .map_err(|_| PostgresAdapterErrorV1::MigrationCommit)?;
            return Ok(());
        }
        RecordedStepLineageV1::Missing => {}
    }
    execute_step_as_owner(&mut transaction, roles, step).await?;
    record_step(&mut transaction, bundle, step)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationLedgerWrite)?;
    transaction
        .commit()
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationCommit)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecordedStepLineageV1 {
    Missing,
    Exact,
    Predecessor,
}

fn classify_recorded_step_lineage(
    current_bundle_revision: u32,
    expected_digest: &[u8],
    recorded_steps: &[(i32, Vec<u8>)],
) -> Result<RecordedStepLineageV1, PostgresAdapterErrorV1> {
    let mut has_exact = false;
    let mut has_predecessor = false;
    for (recorded_revision, recorded_digest) in recorded_steps {
        let recorded_revision =
            u32::try_from(*recorded_revision).map_err(|_| PostgresAdapterErrorV1::Migration)?;
        if recorded_revision == 0
            || recorded_revision > current_bundle_revision
            || recorded_digest.as_slice() != expected_digest
        {
            return Err(PostgresAdapterErrorV1::Migration);
        }
        if recorded_revision == current_bundle_revision {
            has_exact = true;
        } else {
            has_predecessor = true;
        }
    }
    if has_exact {
        Ok(RecordedStepLineageV1::Exact)
    } else if has_predecessor {
        Ok(RecordedStepLineageV1::Predecessor)
    } else {
        Ok(RecordedStepLineageV1::Missing)
    }
}

async fn read_recorded_steps(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bundle: &StorageBundleV1,
    step: &hermes_storage_protocol::v1::StorageMigrationStepV1,
) -> Result<Vec<(i32, Vec<u8>)>, PostgresAdapterErrorV1> {
    query_as::<_, (i32, Vec<u8>)>(
        "SELECT bundle_revision, step_digest FROM hermes_platform.storage_migration_ledger WHERE owner_id = $1 AND step_revision = $2 ORDER BY bundle_revision",
    )
    .bind(&bundle.owner_id)
    .bind(postgres_revision(step.revision)?)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| PostgresAdapterErrorV1::Migration)
}

async fn execute_step_as_owner(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    roles: &StorageRoleSpecV1,
    step: &hermes_storage_protocol::v1::StorageMigrationStepV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let set_role = format!("SET LOCAL ROLE {}", roles.ddl_owner());
    query(AssertSqlSafe(set_role))
        .execute(&mut **transaction)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationOwnerRole)?;
    let sql = std::str::from_utf8(&step.forward_sql_utf8)
        .map_err(|_| PostgresAdapterErrorV1::MigrationStatement)?;
    raw_sql(AssertSqlSafe(sql.to_owned()))
        .execute(&mut **transaction)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationStatement)?;
    query("RESET ROLE")
        .execute(&mut **transaction)
        .await
        .map_err(|_| PostgresAdapterErrorV1::MigrationResetRole)?;
    Ok(())
}

async fn record_step(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bundle: &StorageBundleV1,
    step: &hermes_storage_protocol::v1::StorageMigrationStepV1,
) -> Result<(), PostgresAdapterErrorV1> {
    query("INSERT INTO hermes_platform.storage_migration_ledger (owner_id, bundle_revision, step_revision, step_digest) VALUES ($1, $2, $3, $4)")
        .bind(&bundle.owner_id)
        .bind(postgres_revision(bundle.revision)?)
        .bind(postgres_revision(step.revision)?)
        .bind(&step.sha256)
        .execute(&mut **transaction)
        .await
        .map_err(|_| PostgresAdapterErrorV1::Migration)?;
    Ok(())
}

fn postgres_revision(revision: u32) -> Result<i32, PostgresAdapterErrorV1> {
    i32::try_from(revision).map_err(|_| PostgresAdapterErrorV1::Migration)
}

#[cfg(test)]
mod tests {
    use super::{RecordedStepLineageV1, classify_recorded_step_lineage};
    use crate::PostgresAdapterErrorV1;

    const EXPECTED_DIGEST: [u8; 32] = [7; 32];

    #[test]
    fn missing_step_requires_execution() {
        assert_eq!(
            classify_recorded_step_lineage(2, &EXPECTED_DIGEST, &[]),
            Ok(RecordedStepLineageV1::Missing)
        );
    }

    #[test]
    fn exact_step_is_a_no_op() {
        assert_eq!(
            classify_recorded_step_lineage(
                2,
                &EXPECTED_DIGEST,
                &[(1, EXPECTED_DIGEST.to_vec()), (2, EXPECTED_DIGEST.to_vec())],
            ),
            Ok(RecordedStepLineageV1::Exact)
        );
    }

    #[test]
    fn exact_predecessor_records_successor_acceptance_without_execution() {
        assert_eq!(
            classify_recorded_step_lineage(2, &EXPECTED_DIGEST, &[(1, EXPECTED_DIGEST.to_vec())],),
            Ok(RecordedStepLineageV1::Predecessor)
        );
    }

    #[test]
    fn digest_drift_is_rejected() {
        assert_eq!(
            classify_recorded_step_lineage(2, &EXPECTED_DIGEST, &[(1, vec![8; 32])]),
            Err(PostgresAdapterErrorV1::Migration)
        );
    }

    #[test]
    fn future_bundle_revision_is_rejected() {
        assert_eq!(
            classify_recorded_step_lineage(2, &EXPECTED_DIGEST, &[(3, EXPECTED_DIGEST.to_vec())],),
            Err(PostgresAdapterErrorV1::Migration)
        );
    }
}
