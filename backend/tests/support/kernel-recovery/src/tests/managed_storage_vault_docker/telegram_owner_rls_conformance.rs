//! Exact PostgreSQL owner isolation for the complete owner-local Telegram store.

use super::*;

use makosh_telegram_persistence::TELEGRAM_OWNER_RLS_TABLES_V1;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

const CROSS_OWNER_ROLE: &str = "storage_ffffffffffffffff_1";

pub(super) fn assert_telegram_owner_rls_v1(database_id: &str) {
    assert_owner_rls_tables_v1(
        database_id,
        &TELEGRAM_OWNER_RLS_TABLES_V1,
        "telegram_owner_scope",
    );
}

pub(super) fn assert_owner_rls_tables_v1(
    database_id: &str,
    tables: &[&str],
    owner_scope_table: &str,
) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build owner-RLS runtime")
        .block_on(assert_owner_rls_v1(database_id, tables, owner_scope_table));
}

async fn assert_owner_rls_v1(database_id: &str, tables: &[&str], owner_scope_table: &str) {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    let options = PgConnectOptions::new()
        .host(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
        .port(
            required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                .parse()
                .expect("valid PostgreSQL port"),
        )
        .username("makosh_postgres_admin")
        .password(password.as_str())
        .database(database_id)
        .ssl_mode(PgSslMode::Disable);
    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options.clone())
        .await
        .expect("connect owner-RLS conformance database");

    let owner_scope: (i64, String) = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT COUNT(*), MIN(runtime_principal_prefix) FROM makosh_data.{owner_scope_table}"
    )))
    .fetch_one(&admin)
    .await
    .expect("read exact Telegram owner scope");
    assert_eq!(owner_scope.0, 1);
    assert_ne!(owner_scope.1, "storage_ffffffffffffffff");

    sqlx::raw_sql(
        "CREATE ROLE storage_ffffffffffffffff_1 \
         NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOINHERIT; \
         GRANT USAGE ON SCHEMA makosh_data TO storage_ffffffffffffffff_1; \
         GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA makosh_data \
         TO storage_ffffffffffffffff_1; \
         GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA makosh_data \
         TO storage_ffffffffffffffff_1;",
    )
    .execute(&admin)
    .await
    .expect("create exact non-bypass Telegram RLS role");
    let attributes: (bool, bool, bool) = sqlx::query_as(
        "SELECT rolcanlogin, rolsuper, rolbypassrls FROM pg_roles WHERE rolname = $1",
    )
    .bind(CROSS_OWNER_ROLE)
    .fetch_one(&admin)
    .await
    .expect("read Telegram RLS role attributes");
    assert_eq!(attributes, (false, false, false));

    let runtime = PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|connection, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE storage_ffffffffffffffff_1")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .expect("connect exact non-bypass Telegram role");

    for table in tables {
        let flags: (bool, bool) = sqlx::query_as(
            "SELECT class.relrowsecurity, class.relforcerowsecurity \
             FROM pg_class AS class \
             JOIN pg_namespace AS namespace ON namespace.oid = class.relnamespace \
             WHERE namespace.nspname = 'makosh_data' AND class.relname = $1",
        )
        .bind(table)
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read Telegram RLS flags for {table}: {error}"));
        assert_eq!(flags, (true, true), "exact FORCE RLS flags for {table}");
        let policy_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pg_policies \
             WHERE schemaname = 'makosh_data' AND tablename = $1 \
               AND policyname = $2 AND cmd = 'ALL' \
               AND qual IS NOT NULL AND with_check IS NOT NULL",
        )
        .bind(table)
        .bind(format!("{table}_owner_isolation_v1"))
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read Telegram RLS policy for {table}: {error}"));
        assert_eq!(policy_count, 1, "one exact owner policy for {table}");

        let owner_count_before: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table}"
        )))
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read Telegram owner rows for {table}: {error}"));
        let visible: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table}"
        )))
        .fetch_one(&runtime)
        .await
        .unwrap_or_else(|error| panic!("cross-owner SELECT {table}: {error}"));
        assert_eq!(visible, 0, "cross-owner role must not see {table}");

        let update_column: String = sqlx::query_scalar(
            "SELECT column_name FROM information_schema.columns \
             WHERE table_schema = 'makosh_data' AND table_name = $1 \
               AND is_generated = 'NEVER' \
               AND COALESCE(identity_generation, '') <> 'ALWAYS' \
             ORDER BY ordinal_position LIMIT 1",
        )
        .bind(table)
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("select update column for {table}: {error}"));
        let updated = sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE makosh_data.{table} SET {update_column} = {update_column}"
        )))
        .execute(&runtime)
        .await
        .unwrap_or_else(|error| panic!("cross-owner UPDATE {table}: {error}"))
        .rows_affected();
        assert_eq!(updated, 0, "cross-owner role must not update {table}");
        let deleted = sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM makosh_data.{table}"
        )))
        .execute(&runtime)
        .await
        .unwrap_or_else(|error| panic!("cross-owner DELETE {table}: {error}"))
        .rows_affected();
        assert_eq!(deleted, 0, "cross-owner role must not delete {table}");

        let mut insert = runtime
            .begin()
            .await
            .unwrap_or_else(|error| panic!("begin cross-owner INSERT {table}: {error}"));
        let error = sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO makosh_data.{table} DEFAULT VALUES"
        )))
        .execute(&mut *insert)
        .await
        .expect_err("cross-owner Telegram INSERT must fail");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("42501"),
            "cross-owner INSERT into {table} must fail through RLS"
        );
        insert
            .rollback()
            .await
            .unwrap_or_else(|error| panic!("rollback denied INSERT {table}: {error}"));

        let owner_count_after: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table}"
        )))
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read unchanged Telegram rows for {table}: {error}"));
        assert_eq!(
            owner_count_after, owner_count_before,
            "denied cross-owner DML changed {table}"
        );
    }

    runtime.close().await;
    sqlx::raw_sql(
        "DROP OWNED BY storage_ffffffffffffffff_1; \
         DROP ROLE storage_ffffffffffffffff_1;",
    )
    .execute(&admin)
    .await
    .expect("remove exact Telegram RLS test role");
}
