//! PostgreSQL-only portion of the runtime revoke sequence.

use makosh_storage_control::{StorageFenceOutcomeV1, StoragePostgresFencePortV1};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{AssertSqlSafe, query, query_scalar};
use std::future::Future;

use crate::{PostgresAdapterErrorV1, PostgresAdminConnectorV1};

use super::{StorageRoleSpecV1, revoke_owner_data_privileges};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PostgresRuntimeFenceV1 {
    terminated_backend_count: u64,
}

pub struct PostgresRuntimeFenceAdapterV1<'a> {
    connector: &'a PostgresAdminConnectorV1,
    spec: &'a StorageRoleSpecV1,
}

impl<'a> PostgresRuntimeFenceAdapterV1<'a> {
    pub const fn new(connector: &'a PostgresAdminConnectorV1, spec: &'a StorageRoleSpecV1) -> Self {
        Self { connector, spec }
    }
}

impl StoragePostgresFencePortV1 for PostgresRuntimeFenceAdapterV1<'_> {
    fn fence_runtime_role(
        &mut self,
        binding: &StorageBindingV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send {
        let matches_binding = self.spec.binding() == binding;
        async move {
            if !matches_binding {
                return StorageFenceOutcomeV1::Rejected;
            }
            fence_postgres_runtime_role(self.connector, self.spec)
                .await
                .map(|_| StorageFenceOutcomeV1::Applied)
                .unwrap_or(StorageFenceOutcomeV1::Unavailable)
        }
    }
}

impl PostgresRuntimeFenceV1 {
    pub const fn terminated_backend_count(&self) -> u64 {
        self.terminated_backend_count
    }
}

pub async fn fence_postgres_runtime_role(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<PostgresRuntimeFenceV1, PostgresAdapterErrorV1> {
    if !runtime_role_exists(connector, spec).await? {
        return Ok(PostgresRuntimeFenceV1 {
            terminated_backend_count: 0,
        });
    }
    disable_new_logins(connector, spec)
        .await
        .map_err(|_| PostgresAdapterErrorV1::LoginFence)?;
    revoke_schema_access(connector, spec)
        .await
        .map_err(|_| PostgresAdapterErrorV1::SchemaFence)?;
    revoke_owner_data_privileges(connector, spec)
        .await
        .map_err(|_| PostgresAdapterErrorV1::OwnerPrivilegeFence)?;
    let terminated_backend_count = terminate_runtime_backends(connector, spec)
        .await
        .map_err(|_| PostgresAdapterErrorV1::BackendTerminationFence)?;
    ensure_no_runtime_backends(connector, spec)
        .await
        .map_err(|_| PostgresAdapterErrorV1::BackendDrainFence)?;
    Ok(PostgresRuntimeFenceV1 {
        terminated_backend_count,
    })
}

async fn runtime_role_exists(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<bool, PostgresAdapterErrorV1> {
    query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname = $1)")
        .bind(spec.runtime_principal())
        .fetch_one(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::Query)
}

async fn disable_new_logins(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let statement = format!("ALTER ROLE {} NOLOGIN", spec.runtime_principal());
    query(AssertSqlSafe(statement))
        .execute(connector.pool())
        .await
        .map_err(login_fence_error)?;
    Ok(())
}

fn login_fence_error(error: sqlx::Error) -> PostgresAdapterErrorV1 {
    match error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .as_deref()
    {
        Some("42501") => PostgresAdapterErrorV1::LoginFenceUnauthorized,
        Some("42704") => PostgresAdapterErrorV1::LoginFenceMissingRole,
        _ => PostgresAdapterErrorV1::LoginFence,
    }
}

async fn revoke_schema_access(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let statement = format!(
        "REVOKE USAGE ON SCHEMA {} FROM {}",
        spec.storage_schema(),
        spec.runtime_principal(),
    );
    execute_fence_statement(connector, &statement).await
}

async fn terminate_runtime_backends(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<u64, PostgresAdapterErrorV1> {
    let outcomes = query_scalar::<_, bool>(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
         WHERE usename = $1 AND pid <> pg_backend_pid()",
    )
    .bind(spec.runtime_principal())
    .fetch_all(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::SessionFence)?;
    Ok(outcomes
        .into_iter()
        .filter(|terminated| *terminated)
        .count() as u64)
}

async fn ensure_no_runtime_backends(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let active_count = query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM pg_stat_activity WHERE usename = $1 AND pid <> pg_backend_pid()",
    )
    .bind(spec.runtime_principal())
    .fetch_one(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::SessionFence)?;
    if active_count != 0 {
        return Err(PostgresAdapterErrorV1::SessionFence);
    }
    Ok(())
}

async fn execute_fence_statement(
    connector: &PostgresAdminConnectorV1,
    statement: &str,
) -> Result<(), PostgresAdapterErrorV1> {
    query(AssertSqlSafe(statement))
        .execute(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::SessionFence)?;
    Ok(())
}
