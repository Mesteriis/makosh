//! Test-only diagnostics for the disposable retained Preview evidence replay contour.

use std::time::{Duration, Instant};

use super::*;

use sqlx::{
    Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use zeroize::Zeroizing;

pub(super) struct RetainedMailReplayIndexRowV1 {
    attachment_anchor_id: [u8; 16],
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    contract_owner: String,
    contract_name: String,
    contract_major: i32,
    contract_revision: i32,
    contract_schema_sha256: [u8; 32],
    indexed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RetainedPreviewReplayDiagnosticsV1 {
    pub(super) state: i16,
    pub(super) error: i16,
    pub(super) producer_results: i64,
    pub(super) communications_results: i64,
    pub(super) mail_results: i64,
    pub(super) communications_result_published: bool,
    pub(super) mail_result_published: bool,
    pub(super) communications_failure: i16,
    pub(super) mail_failure: i16,
    pub(super) communications_published_audits: i64,
    pub(super) mail_published_audits: i64,
}

pub(super) fn wait_for_retained_preview_evidence_indexes_v1(attachment_anchor_id: [u8; 16]) {
    let runtime = tokio::runtime::Runtime::new().expect("retained Preview diagnostics runtime");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let result = runtime.block_on(async {
            let pool = retained_preview_diagnostics_pool_v1().await;
            let communications = sqlx::query(
                "SELECT message_id FROM makosh_data.communications_retained_evidence_replay_index WHERE attachment_anchor_id=$1",
            )
            .bind(attachment_anchor_id.as_slice())
            .fetch_optional(&pool)
            .await
            .expect("read retained Communications evidence index");
            let mail = sqlx::query(
                "SELECT message_id FROM makosh_data.mail_retained_evidence_replay_index WHERE attachment_anchor_id=$1",
            )
            .bind(attachment_anchor_id.as_slice())
            .fetch_optional(&pool)
            .await
            .expect("read retained Mail evidence index");
            communications
                .zip(mail)
                .map(|(communications, mail)| {
                    let _ = id16(&communications, "message_id");
                    let _ = id16(&mail, "message_id");
                })
        });
        if result.is_some() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "retained Preview producer indexes were not populated"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn wait_for_retained_preview_replay_terminal_v1(
    operation_id: [u8; 16],
) -> RetainedPreviewReplayDiagnosticsV1 {
    let runtime = tokio::runtime::Runtime::new().expect("retained Preview diagnostics runtime");
    // Communications deliberately rotates across fourteen isolated durable
    // consumers. Each idle pull is bounded to 500 ms, so one complete fair
    // turn plus both producer relays and result consumers can exceed 10 s.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let diagnostics = runtime.block_on(async {
            let pool = retained_preview_diagnostics_pool_v1().await;
            let operation = sqlx::query(
                "SELECT state,error FROM makosh_data.attachment_preview_evidence_replay_operations WHERE operation_id=$1",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("read retained Preview replay operation");
            let producer_results: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.attachment_preview_evidence_replay_anchor_result_inbox WHERE operation_id=$1",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("count retained Preview replay producer results");
            let communications_results: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.attachment_preview_evidence_replay_anchor_result_inbox WHERE operation_id=$1 AND producer=1",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("count retained Communications replay results");
            let mail_results: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.attachment_preview_evidence_replay_anchor_result_inbox WHERE operation_id=$1 AND producer=2",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("count retained Mail replay results");
            let communications_result_published: bool = sqlx::query_scalar(
                "SELECT COALESCE((SELECT published_at_unix_seconds IS NOT NULL FROM makosh_data.communications_retained_evidence_replay_result_outbox WHERE operation_id=$1),FALSE)",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("read retained Communications result publish state");
            let mail_result_published: bool = sqlx::query_scalar(
                "SELECT COALESCE((SELECT published_at_unix_seconds IS NOT NULL FROM makosh_data.mail_retained_evidence_replay_result_outbox WHERE operation_id=$1),FALSE)",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("read retained Mail result publish state");
            let communications_failure: i16 = sqlx::query_scalar(
                "SELECT failure FROM makosh_data.attachment_preview_evidence_replay_anchor_producers WHERE operation_id=$1 AND producer=1",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("read retained Communications replay failure");
            let mail_failure: i16 = sqlx::query_scalar(
                "SELECT failure FROM makosh_data.attachment_preview_evidence_replay_anchor_producers WHERE operation_id=$1 AND producer=2",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("read retained Mail replay failure");
            let communications_published_audits: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.communications_retained_evidence_replay_audit WHERE operation_id=$1 AND phase=2",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("count retained Communications replay audits");
            let mail_published_audits: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.mail_retained_evidence_replay_audit WHERE operation_id=$1 AND phase=2",
            )
            .bind(operation_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("count retained Mail replay audits");
            RetainedPreviewReplayDiagnosticsV1 {
                state: operation.try_get("state").expect("replay operation state"),
                error: operation.try_get("error").expect("replay operation error"),
                producer_results,
                communications_results,
                mail_results,
                communications_result_published,
                mail_result_published,
                communications_failure,
                mail_failure,
                communications_published_audits,
                mail_published_audits,
            }
        });
        if (diagnostics.state == 3 || diagnostics.state == 4 || diagnostics.state == 5)
            && diagnostics.producer_results == 2
        {
            return diagnostics;
        }
        assert!(
            Instant::now() < deadline,
            "retained Preview replay operation did not become terminal: operation_id={operation_id:02x?}, diagnostics={diagnostics:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn remove_retained_mail_replay_index_v1(
    attachment_anchor_id: [u8; 16],
) -> RetainedMailReplayIndexRowV1 {
    let runtime = tokio::runtime::Runtime::new().expect("retained Preview fault runtime");
    runtime.block_on(async {
        let pool = retained_preview_diagnostics_pool_v1().await;
        let row = sqlx::query(
            "DELETE FROM makosh_data.mail_retained_evidence_replay_index WHERE attachment_anchor_id=$1 RETURNING attachment_anchor_id,message_id,envelope_sha256,contract_owner,contract_name,contract_major,contract_revision,contract_schema_sha256,indexed_at_unix_seconds",
        )
        .bind(attachment_anchor_id.as_slice())
        .fetch_one(&pool)
        .await
        .expect("remove disposable retained Mail replay index row");
        RetainedMailReplayIndexRowV1 {
            attachment_anchor_id: id16(&row, "attachment_anchor_id"),
            message_id: id16(&row, "message_id"),
            envelope_sha256: id32(&row, "envelope_sha256"),
            contract_owner: row.try_get("contract_owner").expect("contract owner"),
            contract_name: row.try_get("contract_name").expect("contract name"),
            contract_major: row.try_get("contract_major").expect("contract major"),
            contract_revision: row
                .try_get("contract_revision")
                .expect("contract revision"),
            contract_schema_sha256: id32(&row, "contract_schema_sha256"),
            indexed_at_unix_seconds: row
                .try_get("indexed_at_unix_seconds")
                .expect("indexed time"),
        }
    })
}

pub(super) fn restore_retained_mail_replay_index_v1(row: RetainedMailReplayIndexRowV1) {
    let runtime = tokio::runtime::Runtime::new().expect("retained Preview restore runtime");
    runtime.block_on(async {
        let pool = retained_preview_diagnostics_pool_v1().await;
        sqlx::query(
            "INSERT INTO makosh_data.mail_retained_evidence_replay_index (attachment_anchor_id,message_id,envelope_sha256,contract_owner,contract_name,contract_major,contract_revision,contract_schema_sha256,indexed_at_unix_seconds) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(row.attachment_anchor_id.as_slice())
        .bind(row.message_id.as_slice())
        .bind(row.envelope_sha256.as_slice())
        .bind(row.contract_owner)
        .bind(row.contract_name)
        .bind(row.contract_major)
        .bind(row.contract_revision)
        .bind(row.contract_schema_sha256.as_slice())
        .bind(row.indexed_at_unix_seconds)
        .execute(&pool)
        .await
        .expect("restore disposable retained Mail replay index row");
    });
}

fn id16(row: &sqlx::postgres::PgRow, column: &str) -> [u8; 16] {
    row.try_get::<Vec<u8>, _>(column)
        .expect("retained evidence message identifier")
        .try_into()
        .expect("retained evidence message identifier length")
}

fn id32(row: &sqlx::postgres::PgRow, column: &str) -> [u8; 32] {
    row.try_get::<Vec<u8>, _>(column)
        .expect("retained evidence SHA-256")
        .try_into()
        .expect("retained evidence SHA-256 length")
}

async fn retained_preview_diagnostics_pool_v1() -> sqlx::PgPool {
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
        .expect("connect retained Preview replay diagnostics")
}
