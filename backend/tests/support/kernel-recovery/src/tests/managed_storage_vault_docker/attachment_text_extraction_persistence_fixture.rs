//! Test-only PostgreSQL diagnostics for the disposable Text Extraction contour.

use super::*;

use sqlx::{
    Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AttachmentTextExtractionDiagnosticsV1 {
    pub(super) candidates: i64,
    pub(super) safety_facts: i64,
    pub(super) custody_requests: i64,
    pub(super) pending_custody_outbox: i64,
    pub(super) custody_results: i64,
    pub(super) jobs: i64,
    pub(super) attempts: i64,
    pub(super) artifacts: i64,
    pub(super) security_delegation_commands: i64,
    pub(super) security_delegation_attempts: i64,
    pub(super) security_delegation_results: i64,
}

pub(super) fn attachment_text_extraction_diagnostics_v1() -> AttachmentTextExtractionDiagnosticsV1 {
    tokio::runtime::Runtime::new()
        .expect("Attachment Text Extraction diagnostics runtime")
        .block_on(async {
            let pool = attachment_text_extraction_diagnostics_pool_v1().await;
            let row = sqlx::query(
                "SELECT \
                 (SELECT count(*) FROM makosh_data.attachment_text_extraction_scan_candidates) AS candidates, \
                 (SELECT count(*) FROM makosh_data.attachment_text_extraction_safety_facts) AS safety_facts, \
                 (SELECT count(*) FROM makosh_data.attachment_text_extraction_custody_outbox) AS custody_requests, \
                 (SELECT count(*) FROM makosh_data.attachment_text_extraction_custody_outbox WHERE published_at_unix_millis IS NULL) AS pending_custody_outbox, \
                 (SELECT count(*) FROM makosh_data.attachment_text_extraction_custody_result_inbox) AS custody_results, \
                 (SELECT count(*) FROM makosh_data.attachment_text_extraction_jobs) AS jobs, \
                 (SELECT coalesce(sum(attempt_count), 0) FROM makosh_data.attachment_text_extraction_jobs) AS attempts, \
                 (SELECT count(*) FROM makosh_data.attachment_text_extraction_artifacts) AS artifacts, \
                 (SELECT count(*) FROM makosh_data.attachment_security_text_extraction_delegation_inbox) AS security_delegation_commands, \
                 (SELECT coalesce(sum(attempt_count), 0) FROM makosh_data.attachment_security_text_extraction_delegation_jobs) AS security_delegation_attempts, \
                 (SELECT count(*) FROM makosh_data.attachment_security_text_extraction_delegation_outbox) AS security_delegation_results",
            )
            .fetch_one(&pool)
            .await
            .expect("read Attachment Text Extraction diagnostics");
            AttachmentTextExtractionDiagnosticsV1 {
                candidates: row.try_get("candidates").expect("candidate count"),
                safety_facts: row.try_get("safety_facts").expect("safety count"),
                custody_requests: row.try_get("custody_requests").expect("custody count"),
                pending_custody_outbox: row
                    .try_get("pending_custody_outbox")
                    .expect("pending custody count"),
                custody_results: row.try_get("custody_results").expect("result count"),
                jobs: row.try_get("jobs").expect("job count"),
                attempts: row.try_get("attempts").expect("attempt count"),
                artifacts: row.try_get("artifacts").expect("artifact count"),
                security_delegation_commands: row
                    .try_get("security_delegation_commands")
                    .expect("security command count"),
                security_delegation_attempts: row
                    .try_get("security_delegation_attempts")
                    .expect("security attempt count"),
                security_delegation_results: row
                    .try_get("security_delegation_results")
                    .expect("security result count"),
            }
        })
}

pub(super) fn replace_attachment_text_parser_identity_v1(
    logical_owner_id: &str,
    run_id: &[u8],
    expected_identity_sha256: [u8; 32],
    replacement_identity_sha256: [u8; 32],
) {
    assert_eq!(run_id.len(), 16);
    assert!(expected_identity_sha256.iter().any(|byte| *byte != 0));
    assert!(replacement_identity_sha256.iter().any(|byte| *byte != 0));
    tokio::runtime::Runtime::new()
        .expect("Attachment Text Extraction parser revision diagnostics runtime")
        .block_on(async {
            let pool = attachment_text_extraction_diagnostics_pool_v1().await;
            let changed = sqlx::query(
                "UPDATE makosh_data.attachment_text_extraction_artifacts SET parser_identity_sha256=$4 WHERE logical_owner_id=$1 AND run_id=$2 AND parser_identity_sha256=$3",
            )
            .bind(logical_owner_id)
            .bind(run_id)
            .bind(expected_identity_sha256.as_slice())
            .bind(replacement_identity_sha256.as_slice())
            .execute(&pool)
            .await
            .expect("replace disposable Text Extraction parser identity")
            .rows_affected();
            assert_eq!(changed, 1, "parser identity replacement must use exact CAS");
        });
}

async fn attachment_text_extraction_diagnostics_pool_v1() -> sqlx::PgPool {
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
        .database("makosh_storage_authenticated")
        .ssl_mode(PgSslMode::Disable);
    PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect Attachment Text Extraction diagnostics")
}
