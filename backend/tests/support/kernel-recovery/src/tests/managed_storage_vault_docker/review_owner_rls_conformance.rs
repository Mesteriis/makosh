//! Admission-grade Review owner isolation through an effective non-bypass role.

use super::*;

pub(super) async fn assert_review_owner_rls_v1(role: &str, tables: &[&str]) {
    assert!(
        !role.is_empty()
            && role
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    );
    assert!(!tables.is_empty());
    let admin = authenticated_storage_admin_pool_v1().await;
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE {role} NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOINHERIT; \
         GRANT USAGE ON SCHEMA makosh_data TO {role}; \
         GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA makosh_data TO {role}; \
         GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA makosh_data TO {role};"
    )))
    .execute(&admin)
    .await
    .expect("create exact Review non-bypass role");
    let attributes: (bool, bool) =
        sqlx::query_as("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname=$1")
            .bind(role)
            .fetch_one(&admin)
            .await
            .expect("read Review role attributes");
    assert_eq!(attributes, (false, false));

    for table in tables {
        assert!(
            table
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        );
        let row_json: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT row_to_json(source)::text FROM (SELECT * FROM makosh_data.{table} \
             WHERE logical_owner_id='owner-1' LIMIT 1) source"
        )))
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read owner1 Review fixture {table}: {error}"));

        let mut transaction = admin
            .begin()
            .await
            .expect("begin Review owner2 transaction");
        sqlx::query(sqlx::AssertSqlSafe(format!("SET LOCAL ROLE {role}")))
            .execute(&mut *transaction)
            .await
            .expect("activate Review non-bypass role");
        sqlx::query("SELECT set_config('makosh.logical_owner_id', 'owner-2', true)")
            .execute(&mut *transaction)
            .await
            .expect("set Review owner2 context");
        let visible: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table} WHERE logical_owner_id='owner-1'"
        )))
        .fetch_one(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 SELECT Review table {table}: {error}"));
        assert_eq!(visible, 0, "owner2 must not see owner1 Review {table}");
        let updated = sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE makosh_data.{table} SET logical_owner_id=logical_owner_id \
             WHERE logical_owner_id='owner-1'"
        )))
        .execute(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 UPDATE Review table {table}: {error}"))
        .rows_affected();
        assert_eq!(updated, 0, "owner2 must not update owner1 Review {table}");
        let deleted = sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM makosh_data.{table} WHERE logical_owner_id='owner-1'"
        )))
        .execute(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 DELETE Review table {table}: {error}"))
        .rows_affected();
        assert_eq!(deleted, 0, "owner2 must not delete owner1 Review {table}");
        transaction
            .commit()
            .await
            .expect("commit invisible Review DML");

        let mut insert = admin.begin().await.expect("begin Review owner2 insert");
        sqlx::query(sqlx::AssertSqlSafe(format!("SET LOCAL ROLE {role}")))
            .execute(&mut *insert)
            .await
            .expect("activate Review non-bypass insert role");
        sqlx::query("SELECT set_config('makosh.logical_owner_id', 'owner-2', true)")
            .execute(&mut *insert)
            .await
            .expect("set Review owner2 insert context");
        let error = sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO makosh_data.{table} OVERRIDING SYSTEM VALUE \
             SELECT (json_populate_record(NULL::makosh_data.{table}, $1::json)).*"
        )))
        .bind(row_json)
        .execute(&mut *insert)
        .await
        .expect_err("cross-owner Review INSERT must fail");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("42501"),
            "owner2 INSERT into Review {table} must fail through RLS"
        );
    }
}
