//! Idempotent role reconciliation with no credential handling.

use sqlx::{AssertSqlSafe, query, query_scalar};
use zeroize::Zeroizing;

use crate::{PostgresAdapterErrorV1, PostgresAdminConnectorV1};

use super::{StorageRoleSpecV1, ensure_role_ledger_binding};

pub async fn ensure_storage_roles(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    ensure_role_ledger_binding(connector, spec).await?;
    ensure_role(connector, spec.ddl_owner(), no_login_sql(spec.ddl_owner())).await?;
    ensure_role(connector, spec.runtime_principal(), runtime_login_sql(spec)).await?;
    set_runtime_limits(connector, spec).await?;
    grant_schema_access(connector, spec).await
}

pub async fn set_runtime_role_password(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
    password: &Zeroizing<Vec<u8>>,
) -> Result<(), PostgresAdapterErrorV1> {
    let password = std::str::from_utf8(password).map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    if password.is_empty() || password.len() > 4 * 1024 {
        return Err(PostgresAdapterErrorV1::Bootstrap);
    }
    let statement = password_statement(connector, spec.runtime_principal(), password).await?;
    execute_bootstrap_statement(connector, &statement).await
}

pub async fn read_runtime_role_scram_verifier(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<Zeroizing<String>, PostgresAdapterErrorV1> {
    let verifier = query_scalar::<_, Option<String>>(
        "SELECT rolpassword FROM pg_authid WHERE rolname = $1 AND rolcanlogin",
    )
    .bind(spec.runtime_principal())
    .fetch_one(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::Query)?
    .ok_or(PostgresAdapterErrorV1::Query)?;
    verifier
        .starts_with("SCRAM-SHA-256$")
        .then_some(Zeroizing::new(verifier))
        .ok_or(PostgresAdapterErrorV1::Query)
}

async fn password_statement(
    connector: &PostgresAdminConnectorV1,
    runtime_principal: &str,
    password: &str,
) -> Result<String, PostgresAdapterErrorV1> {
    query_scalar("SELECT format('ALTER ROLE %I PASSWORD %L', $1, $2)")
        .bind(runtime_principal)
        .bind(password)
        .fetch_one(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::Bootstrap)
}

async fn grant_schema_access(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let ddl_grant = format!(
        "GRANT USAGE, CREATE ON SCHEMA {} TO {}",
        spec.storage_schema(),
        spec.ddl_owner(),
    );
    execute_bootstrap_statement(connector, &ddl_grant).await?;
    let runtime_grant = format!(
        "GRANT USAGE ON SCHEMA {} TO {}",
        spec.storage_schema(),
        spec.runtime_principal(),
    );
    execute_bootstrap_statement(connector, &runtime_grant).await
}

async fn execute_bootstrap_statement(
    connector: &PostgresAdminConnectorV1,
    statement: &str,
) -> Result<(), PostgresAdapterErrorV1> {
    query(AssertSqlSafe(statement))
        .execute(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    Ok(())
}

async fn ensure_role(
    connector: &PostgresAdminConnectorV1,
    role_name: &str,
    create_sql: String,
) -> Result<(), PostgresAdapterErrorV1> {
    let exists =
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname = $1)")
            .bind(role_name)
            .fetch_one(connector.pool())
            .await
            .map_err(|_| PostgresAdapterErrorV1::Query)?;
    if !exists {
        query(AssertSqlSafe(create_sql))
            .execute(connector.pool())
            .await
            .map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    }
    Ok(())
}

async fn set_runtime_limits(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let statement = runtime_role_settings_sql(spec.runtime_principal(), spec.max_connections());
    query(AssertSqlSafe(statement))
        .execute(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    let timeout = format!(
        "ALTER ROLE {} SET statement_timeout = {}",
        spec.runtime_principal(),
        spec.statement_timeout_millis(),
    );
    query(AssertSqlSafe(timeout))
        .execute(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    let search_path = format!(
        "ALTER ROLE {} SET search_path = pg_catalog",
        spec.runtime_principal()
    );
    query(AssertSqlSafe(search_path))
        .execute(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    Ok(())
}

fn no_login_sql(role_name: &str) -> String {
    format!(
        "CREATE ROLE {role_name} NOLOGIN NOINHERIT NOCREATEDB NOCREATEROLE NOSUPERUSER NOBYPASSRLS"
    )
}

fn runtime_login_sql(spec: &StorageRoleSpecV1) -> String {
    format!(
        "CREATE ROLE {} LOGIN NOINHERIT NOCREATEDB NOCREATEROLE NOSUPERUSER NOBYPASSRLS CONNECTION LIMIT {}",
        spec.runtime_principal(),
        spec.max_connections(),
    )
}

fn runtime_role_settings_sql(runtime_principal: &str, max_connections: u16) -> String {
    format!(
        "ALTER ROLE {runtime_principal} LOGIN NOINHERIT NOCREATEDB NOCREATEROLE NOSUPERUSER NOBYPASSRLS CONNECTION LIMIT {max_connections}"
    )
}

#[cfg(test)]
mod tests {
    use super::runtime_role_settings_sql;

    #[test]
    fn authorized_runtime_role_settings_restore_login_after_a_restart_fence() {
        let statement = runtime_role_settings_sql("storage_runtime_1", 4);

        assert_eq!(
            statement,
            "ALTER ROLE storage_runtime_1 LOGIN NOINHERIT NOCREATEDB NOCREATEROLE NOSUPERUSER NOBYPASSRLS CONNECTION LIMIT 4"
        );
    }
}
