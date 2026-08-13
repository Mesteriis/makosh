//! Exact PostgreSQL owner-isolation evidence for admitted AI private state.

use super::*;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

pub(super) fn assert_ai_inference_owner_rls_v1(database_id: &str) {
    owner_rls_runtime().block_on(assert_owner_rls_v1(
        database_id,
        "makosh_ai_inference_rls_test",
        &[
            "ai_inference_runs",
            "ai_summary_runs",
            "ai_translation_runs",
            "ai_explanation_runs",
            "ai_attachment_translation_runs",
        ],
        AI_FIXTURES,
    ));
}

pub(super) fn assert_ollama_ai_owner_rls_v1(database_id: &str) {
    owner_rls_runtime().block_on(assert_owner_rls_v1(
        database_id,
        "makosh_ollama_ai_rls_test",
        &[
            "ollama_ai_runs",
            "ollama_ai_summary_runs",
            "ollama_ai_translation_runs",
            "ollama_ai_explanation_runs",
        ],
        OLLAMA_FIXTURES,
    ));
}

fn owner_rls_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build AI owner-RLS runtime")
}

async fn assert_owner_rls_v1(
    database_id: &str,
    role: &str,
    tables: &[&str],
    fixtures: &'static str,
) {
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
        .expect("connect AI owner-RLS conformance database");
    sqlx::raw_sql(fixtures)
        .execute(&admin)
        .await
        .expect("insert exact owner1 AI fixtures");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "CREATE ROLE {role} NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOINHERIT; \
         GRANT USAGE ON SCHEMA makosh_data TO {role}; \
         GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA makosh_data TO {role};"
    )))
    .execute(&admin)
    .await
    .expect("create exact non-bypass AI RLS role");
    let attributes: (bool, bool) =
        sqlx::query_as("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = $1")
            .bind(role)
            .fetch_one(&admin)
            .await
            .expect("read AI RLS role attributes");
    assert_eq!(attributes, (false, false));

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .after_connect({
            let role = role.to_owned();
            move |connection, _meta| {
                let role = role.clone();
                Box::pin(async move {
                    sqlx::query(sqlx::AssertSqlSafe(format!("SET ROLE {role}")))
                        .execute(connection)
                        .await?;
                    Ok(())
                })
            }
        })
        .connect_with(options)
        .await
        .expect("connect exact non-bypass AI RLS role");

    for table in tables {
        let row_json: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT row_to_json(source)::text FROM (SELECT * FROM makosh_data.{table} \
             WHERE logical_owner_id = 'owner-1' LIMIT 1) source"
        )))
        .fetch_one(&admin)
        .await
        .unwrap_or_else(|error| panic!("read owner1 {table} fixture: {error}"));

        let mut transaction = pool.begin().await.expect("begin owner2 AI RLS transaction");
        sqlx::query("SELECT set_config('makosh.logical_owner_id', 'owner-2', true)")
            .execute(&mut *transaction)
            .await
            .expect("set owner2 AI RLS context");
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
        transaction.commit().await.expect("commit invisible AI DML");

        let mut insert = pool.begin().await.expect("begin cross-owner AI insert");
        sqlx::query("SELECT set_config('makosh.logical_owner_id', 'owner-2', true)")
            .execute(&mut *insert)
            .await
            .expect("set owner2 AI insert context");
        let error = sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO makosh_data.{table} SELECT \
             (json_populate_record(NULL::makosh_data.{table}, $1::json)).*"
        )))
        .bind(row_json)
        .execute(&mut *insert)
        .await
        .expect_err("owner2 cross-owner AI INSERT must fail");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("42501"),
            "owner2 INSERT into {table} must fail through RLS"
        );
    }
}

const AI_FIXTURES: &str = r#"
INSERT INTO makosh_data.ai_inference_runs
(logical_owner_id,run_id,request_digest,context_id,source_evidence_id,source_evidence_revision,contract_major,contract_revision,contract_schema_sha256,source_reference_id,source_declared_bytes,source_sha256,source_custody_proof,requested_tone,requested_language,subject_policy,maximum_output_bytes,maximum_output_tokens,egress_policy,egress_policy_revision,state_revision,run_state)
VALUES ('owner-1',decode(repeat('11',16),'hex'),decode(repeat('21',32),'hex'),decode(repeat('31',16),'hex'),decode(repeat('41',16),'hex'),1,1,1,decode(repeat('51',32),'hex'),decode(repeat('61',16),'hex'),1,decode(repeat('71',32),'hex'),decode('01','hex'),1,1,1,1,1,1,1,1,1) ON CONFLICT DO NOTHING;
INSERT INTO makosh_data.ai_summary_runs
(logical_owner_id,run_id,request_digest,context_id,source_evidence_id,source_evidence_revision,contract_major,contract_revision,contract_schema_sha256,source_reference_id,source_declared_bytes,source_sha256,source_custody_proof,requested_language,requested_length,maximum_output_bytes,maximum_output_tokens,egress_policy,egress_policy_revision,state_revision,run_state)
VALUES ('owner-1',decode(repeat('12',16),'hex'),decode(repeat('22',32),'hex'),decode(repeat('32',16),'hex'),decode(repeat('42',16),'hex'),1,1,1,decode(repeat('52',32),'hex'),decode(repeat('62',16),'hex'),1,decode(repeat('72',32),'hex'),decode('01','hex'),1,1,1,1,1,1,1,1) ON CONFLICT DO NOTHING;
INSERT INTO makosh_data.ai_translation_runs
(logical_owner_id,run_id,request_digest,context_id,source_evidence_id,source_evidence_revision,contract_major,contract_revision,contract_schema_sha256,source_reference_id,source_declared_bytes,source_sha256,source_custody_proof,requested_target_language,maximum_output_bytes,maximum_output_tokens,egress_policy,egress_policy_revision,state_revision,run_state)
VALUES ('owner-1',decode(repeat('13',16),'hex'),decode(repeat('23',32),'hex'),decode(repeat('33',16),'hex'),decode(repeat('43',16),'hex'),1,1,1,decode(repeat('53',32),'hex'),decode(repeat('63',16),'hex'),1,decode(repeat('73',32),'hex'),decode('01','hex'),1,1,1,1,1,1,1) ON CONFLICT DO NOTHING;
INSERT INTO makosh_data.ai_explanation_runs
(logical_owner_id,run_id,request_digest,context_id,source_evidence_id,source_evidence_revision,contract_major,contract_revision,contract_schema_sha256,source_reference_id,source_declared_bytes,source_sha256,source_custody_proof,maximum_reasons,maximum_reason_text_bytes,maximum_output_tokens,egress_policy,egress_policy_revision,state_revision,run_state)
VALUES ('owner-1',decode(repeat('14',16),'hex'),decode(repeat('24',32),'hex'),decode(repeat('34',16),'hex'),decode(repeat('44',16),'hex'),1,1,1,decode(repeat('54',32),'hex'),decode(repeat('64',16),'hex'),1,decode(repeat('74',32),'hex'),decode('01','hex'),8,512,1,1,1,1,1) ON CONFLICT DO NOTHING;
INSERT INTO makosh_data.ai_attachment_translation_runs
(logical_owner_id,run_id,request_digest,context_id,source_evidence_id,source_evidence_revision,contract_major,contract_revision,contract_schema_sha256,source_reference_id,source_declared_bytes,source_sha256,source_custody_proof,requested_target_language,maximum_output_bytes,maximum_output_tokens,egress_policy,egress_policy_revision,state_revision,run_state)
VALUES ('owner-1',decode(repeat('15',16),'hex'),decode(repeat('25',32),'hex'),decode(repeat('35',16),'hex'),decode(repeat('45',16),'hex'),1,1,1,decode(repeat('55',32),'hex'),decode(repeat('65',16),'hex'),1,decode(repeat('75',32),'hex'),decode('01','hex'),1,1,1,1,1,1,1) ON CONFLICT DO NOTHING;
"#;

const OLLAMA_FIXTURES: &str = r#"
INSERT INTO makosh_data.ollama_ai_runs (logical_owner_id,request_id,request_digest,settings_revision,state_revision,run_state) VALUES ('owner-1',decode(repeat('81',16),'hex'),decode(repeat('91',32),'hex'),1,1,1) ON CONFLICT DO NOTHING;
INSERT INTO makosh_data.ollama_ai_summary_runs (logical_owner_id,request_id,request_digest,settings_revision,state_revision,run_state) VALUES ('owner-1',decode(repeat('82',16),'hex'),decode(repeat('92',32),'hex'),1,1,1) ON CONFLICT DO NOTHING;
INSERT INTO makosh_data.ollama_ai_translation_runs (logical_owner_id,request_id,request_digest,settings_revision,state_revision,run_state) VALUES ('owner-1',decode(repeat('83',16),'hex'),decode(repeat('93',32),'hex'),1,1,1) ON CONFLICT DO NOTHING;
INSERT INTO makosh_data.ollama_ai_explanation_runs (logical_owner_id,request_id,request_digest,settings_revision,state_revision,run_state) VALUES ('owner-1',decode(repeat('84',16),'hex'),decode(repeat('94',32),'hex'),1,1,1) ON CONFLICT DO NOTHING;
"#;
