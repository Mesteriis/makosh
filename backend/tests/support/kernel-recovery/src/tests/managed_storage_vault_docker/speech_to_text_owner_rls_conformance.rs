//! Exact PostgreSQL owner isolation for Speech-to-Text and Whisper private receipts.

use super::*;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

pub(super) fn assert_speech_to_text_whisper_owner_rls_v1(database_id: &str) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build Speech-to-Text owner-RLS runtime")
        .block_on(assert_owner_rls_v1(database_id));
}

async fn assert_owner_rls_v1(database_id: &str) {
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
        .expect("connect Speech-to-Text owner-RLS database");
    sqlx::query(
        "CREATE ROLE makosh_speech_stt_rls_test NOLOGIN NOSUPERUSER NOBYPASSRLS \
         NOCREATEDB NOCREATEROLE NOINHERIT",
    )
    .execute(&admin)
    .await
    .expect("create exact non-bypass Speech-to-Text RLS role");
    sqlx::query("GRANT USAGE ON SCHEMA makosh_data TO makosh_speech_stt_rls_test")
        .execute(&admin)
        .await
        .expect("grant Speech-to-Text schema usage");
    sqlx::query(
        "GRANT SELECT, INSERT, UPDATE, DELETE ON \
         makosh_data.speech_to_text_runs, makosh_data.whisper_stt_runs \
         TO makosh_speech_stt_rls_test",
    )
    .execute(&admin)
    .await
    .expect("grant exact Speech-to-Text table privileges");
    let attributes: (bool, bool) =
        sqlx::query_as("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = $1")
            .bind("makosh_speech_stt_rls_test")
            .fetch_one(&admin)
            .await
            .expect("read Speech-to-Text RLS role attributes");
    assert_eq!(attributes, (false, false));

    let runtime = PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|connection, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE makosh_speech_stt_rls_test")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .expect("connect exact non-bypass Speech-to-Text role");

    for table in ["speech_to_text_runs", "whisper_stt_runs"] {
        let before_count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table} WHERE logical_owner_id = 'owner-1'"
        )))
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read owner1 {table} count: {error}"));
        assert!(before_count > 0, "owner1 {table} fixture must exist");
        let row_json: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT row_to_json(source)::text FROM (SELECT * FROM makosh_data.{table} \
             WHERE logical_owner_id = 'owner-1' LIMIT 1) source"
        )))
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read owner1 {table} fixture: {error}"));

        let mut transaction = runtime
            .begin()
            .await
            .expect("begin owner2 Speech-to-Text transaction");
        sqlx::query("SELECT set_config('makosh.logical_owner_id', 'owner-2', true)")
            .execute(&mut *transaction)
            .await
            .expect("set owner2 Speech-to-Text context");
        let visible: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table} WHERE logical_owner_id = 'owner-1'"
        )))
        .fetch_one(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 SELECT {table}: {error}"));
        assert_eq!(visible, 0, "owner2 must not see owner1 {table}");
        let updated = sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE makosh_data.{table} SET logical_owner_id = logical_owner_id \
             WHERE logical_owner_id = 'owner-1'"
        )))
        .execute(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 UPDATE {table}: {error}"))
        .rows_affected();
        assert_eq!(updated, 0, "owner2 must not update owner1 {table}");
        let deleted = sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM makosh_data.{table} WHERE logical_owner_id = 'owner-1'"
        )))
        .execute(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 DELETE {table}: {error}"))
        .rows_affected();
        assert_eq!(deleted, 0, "owner2 must not delete owner1 {table}");
        transaction
            .commit()
            .await
            .expect("commit invisible Speech-to-Text DML");

        let mut insert = runtime
            .begin()
            .await
            .expect("begin cross-owner Speech-to-Text insert");
        sqlx::query("SELECT set_config('makosh.logical_owner_id', 'owner-2', true)")
            .execute(&mut *insert)
            .await
            .expect("set owner2 Speech-to-Text insert context");
        let error = sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO makosh_data.{table} SELECT \
             (json_populate_record(NULL::makosh_data.{table}, $1::json)).*"
        )))
        .bind(row_json)
        .execute(&mut *insert)
        .await
        .expect_err("owner2 cross-owner Speech-to-Text INSERT must fail");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("42501"),
            "owner2 INSERT into {table} must fail through RLS"
        );
        insert
            .rollback()
            .await
            .expect("rollback denied Speech-to-Text insert");

        let after_count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table} WHERE logical_owner_id = 'owner-1'"
        )))
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read unchanged owner1 {table} count: {error}"));
        assert_eq!(
            after_count, before_count,
            "denied {table} DML changed durable rows"
        );
    }
}
