//! Test-only PostgreSQL diagnostics for the disposable Preview contour.

use super::*;

use sha2::{Digest, Sha256};
use sqlx::{
    Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AttachmentPreviewDiagnosticsV1 {
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

pub(super) fn attachment_preview_diagnostics_v1() -> AttachmentPreviewDiagnosticsV1 {
    tokio::runtime::Runtime::new()
        .expect("Attachment Preview diagnostics runtime")
        .block_on(async {
            let pool = attachment_preview_diagnostics_pool_v1().await;
            let row = sqlx::query(
                "SELECT \
                 (SELECT count(*) FROM makosh_data.attachment_preview_scan_candidates) AS candidates, \
                 (SELECT count(*) FROM makosh_data.attachment_preview_safety_facts) AS safety_facts, \
                 (SELECT count(*) FROM makosh_data.attachment_preview_custody_outbox) AS custody_requests, \
                 (SELECT count(*) FROM makosh_data.attachment_preview_custody_outbox WHERE published_at_unix_millis IS NULL) AS pending_custody_outbox, \
                 (SELECT count(*) FROM makosh_data.attachment_preview_custody_result_inbox) AS custody_results, \
                 (SELECT count(*) FROM makosh_data.attachment_preview_jobs) AS jobs, \
                 (SELECT coalesce(sum(attempt_count), 0) FROM makosh_data.attachment_preview_jobs) AS attempts, \
                 (SELECT count(*) FROM makosh_data.attachment_preview_artifacts) AS artifacts, \
                 (SELECT count(*) FROM makosh_data.attachment_security_preview_delegation_inbox) AS security_delegation_commands, \
                 (SELECT coalesce(sum(attempt_count), 0) FROM makosh_data.attachment_security_preview_delegation_jobs) AS security_delegation_attempts, \
                 (SELECT count(*) FROM makosh_data.attachment_security_preview_delegation_outbox) AS security_delegation_results",
            )
            .fetch_one(&pool)
            .await
            .expect("read Attachment Preview diagnostics");
            AttachmentPreviewDiagnosticsV1 {
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
                    .expect("security delegation command count"),
                security_delegation_attempts: row
                    .try_get("security_delegation_attempts")
                    .expect("security delegation attempt count"),
                security_delegation_results: row
                    .try_get("security_delegation_results")
                    .expect("security delegation result count"),
            }
        })
}

pub(super) fn replace_attachment_preview_renderer_identity_v1(
    logical_owner_id: &str,
    run_id: &[u8],
    expected_identity_sha256: [u8; 32],
    replacement_identity_sha256: [u8; 32],
) {
    assert_eq!(run_id.len(), 16);
    assert!(expected_identity_sha256.iter().any(|byte| *byte != 0));
    assert!(replacement_identity_sha256.iter().any(|byte| *byte != 0));
    attachment_preview_fixture_runtime_v1().block_on(async {
        let pool = attachment_preview_diagnostics_pool_v1().await;
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_preview_artifacts SET renderer_identity_sha256=$4 WHERE logical_owner_id=$1 AND run_id=$2 AND renderer_identity_sha256=$3",
        )
        .bind(logical_owner_id)
        .bind(run_id)
        .bind(expected_identity_sha256.as_slice())
        .bind(replacement_identity_sha256.as_slice())
        .execute(&pool)
        .await
        .expect("replace disposable Preview renderer identity")
        .rows_affected();
        assert_eq!(changed, 1, "renderer identity replacement must use exact CAS");
    });
}

pub(super) fn replace_attachment_preview_state_revision_v1(
    logical_owner_id: &str,
    run_id: &[u8],
    expected_revision: u64,
    replacement_revision: u64,
) {
    assert_eq!(run_id.len(), 16);
    assert!(expected_revision > 0);
    assert!(replacement_revision > 0);
    attachment_preview_fixture_runtime_v1().block_on(async {
        let pool = attachment_preview_diagnostics_pool_v1().await;
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_preview_runs SET state_revision=$4 WHERE logical_owner_id=$1 AND run_id=$2 AND state_revision=$3",
        )
        .bind(logical_owner_id)
        .bind(run_id)
        .bind(i64::try_from(expected_revision).expect("bounded expected Preview revision"))
        .bind(i64::try_from(replacement_revision).expect("bounded replacement Preview revision"))
        .execute(&pool)
        .await
        .expect("replace disposable Preview state revision")
        .rows_affected();
        assert_eq!(changed, 1, "state revision replacement must use exact CAS");
    });
}

pub(super) fn expire_attachment_preview_ticket_v1(
    logical_owner_id: &str,
    run_id: &[u8],
    opaque_ticket: &[u8],
) {
    assert_eq!(run_id.len(), 16);
    assert_eq!(opaque_ticket.len(), 32);
    let ticket_sha256: [u8; 32] = Sha256::digest(opaque_ticket).into();
    attachment_preview_fixture_runtime_v1().block_on(async {
        let pool = attachment_preview_diagnostics_pool_v1().await;
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_preview_read_tickets SET expires_at_unix_seconds=created_at_unix_seconds+1 WHERE logical_owner_id=$1 AND run_id=$2 AND ticket_sha256=$3 AND used_at_unix_seconds IS NULL AND expires_at_unix_seconds>created_at_unix_seconds+1",
        )
        .bind(logical_owner_id)
        .bind(run_id)
        .bind(ticket_sha256.as_slice())
        .execute(&pool)
        .await
        .expect("expire disposable Preview tickets")
        .rows_affected();
        assert_eq!(changed, 1, "exactly one disposable Preview ticket must expire");
    });
}

pub(super) fn replace_attachment_preview_job_source_receipt_v1(
    logical_owner_id: &str,
    run_id: &[u8],
    expected_receipt_sha256: [u8; 32],
    replacement_receipt_sha256: [u8; 32],
) {
    assert_eq!(run_id.len(), 16);
    assert!(expected_receipt_sha256.iter().any(|byte| *byte != 0));
    assert!(replacement_receipt_sha256.iter().any(|byte| *byte != 0));
    assert_ne!(expected_receipt_sha256, replacement_receipt_sha256);
    attachment_preview_fixture_runtime_v1().block_on(async {
        let pool = attachment_preview_diagnostics_pool_v1().await;
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_preview_jobs SET source_receipt_sha256=$4 WHERE logical_owner_id=$1 AND run_id=$2 AND source_receipt_sha256=$3",
        )
        .bind(logical_owner_id)
        .bind(run_id)
        .bind(expected_receipt_sha256.as_slice())
        .bind(replacement_receipt_sha256.as_slice())
        .execute(&pool)
        .await
        .expect("replace disposable Preview job source receipt")
        .rows_affected();
        assert_eq!(changed, 1, "source receipt replacement must use exact CAS");
    });
}

pub(super) fn expire_attachment_preview_job_lease_v1(logical_owner_id: &str, run_id: &[u8]) {
    assert_eq!(run_id.len(), 16);
    attachment_preview_fixture_runtime_v1().block_on(async {
        let pool = attachment_preview_diagnostics_pool_v1().await;
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_preview_jobs SET lease_expires_at_unix_millis=updated_at_unix_millis+1 WHERE logical_owner_id=$1 AND run_id=$2 AND state=2 AND lease_expires_at_unix_millis>updated_at_unix_millis+1",
        )
        .bind(logical_owner_id)
        .bind(run_id)
        .execute(&pool)
        .await
        .expect("expire disposable Preview job lease")
        .rows_affected();
        assert_eq!(changed, 1, "exactly one active Preview job lease must expire");
    });
}

fn attachment_preview_fixture_runtime_v1() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("Attachment Preview diagnostics runtime")
}

async fn attachment_preview_diagnostics_pool_v1() -> sqlx::PgPool {
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
        .expect("connect Attachment Preview diagnostics")
}
