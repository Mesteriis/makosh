//! Test-only direct PostgreSQL diagnostics for the disposable Attachment Security contour.

use makosh_attachment_security_persistence::AttachmentSecurityPersistenceErrorV1;
use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub(super) struct AttachmentSecurityPersistenceConformanceV1 {
    pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AttachmentSecurityPersistenceDiagnosticsV1 {
    pub(super) candidates: i64,
    pub(super) canonical_states: i64,
    pub(super) jobs: i64,
    pub(super) attempts: i64,
    pub(super) target_blob_receipts: i64,
    pub(super) outbox: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AttachmentSecurityScanJobDiagnosticsV1 {
    pub(super) state: i16,
    pub(super) attempt_count: u32,
    pub(super) target_blob_receipt_present: bool,
    pub(super) outbox_message_id_present: bool,
    pub(super) claimed: bool,
}

impl AttachmentSecurityPersistenceConformanceV1 {
    pub(super) async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        database_id: &str,
    ) -> Result<Self, AttachmentSecurityPersistenceErrorV1> {
        if host.trim().is_empty()
            || port == 0
            || username.trim().is_empty()
            || password.is_empty()
            || database_id.trim().is_empty()
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .database(database_id);
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub(super) async fn diagnostics(
        &self,
    ) -> Result<AttachmentSecurityPersistenceDiagnosticsV1, AttachmentSecurityPersistenceErrorV1>
    {
        let row = sqlx::query(
            "SELECT \
             (SELECT count(*) FROM makosh_data.attachment_security_scan_candidates) AS candidates, \
             (SELECT count(*) FROM makosh_data.attachment_security_canonical_states) AS canonical_states, \
             (SELECT count(*) FROM makosh_data.attachment_security_scan_jobs) AS jobs, \
             (SELECT coalesce(sum(attempt_count), 0) FROM makosh_data.attachment_security_scan_jobs) AS attempts, \
             (SELECT count(*) FROM makosh_data.attachment_security_scan_jobs WHERE target_blob_reference_id IS NOT NULL) AS target_blob_receipts, \
             (SELECT count(*) FROM makosh_data.attachment_security_verdict_outbox) AS outbox",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(AttachmentSecurityPersistenceDiagnosticsV1 {
            candidates: row
                .try_get("candidates")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            canonical_states: row
                .try_get("canonical_states")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            jobs: row
                .try_get("jobs")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            attempts: row
                .try_get("attempts")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            target_blob_receipts: row
                .try_get("target_blob_receipts")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            outbox: row
                .try_get("outbox")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        })
    }

    pub(super) async fn scan_job_diagnostics(
        &self,
        attachment_anchor_id: [u8; 16],
    ) -> Result<Option<AttachmentSecurityScanJobDiagnosticsV1>, AttachmentSecurityPersistenceErrorV1>
    {
        let row = sqlx::query(
            "SELECT state, attempt_count, \
             target_blob_reference_id IS NOT NULL AS target_blob_receipt_present, \
             outbox_message_id IS NOT NULL AS outbox_message_id_present, \
             claimed_by IS NOT NULL AS claimed \
             FROM makosh_data.attachment_security_scan_jobs \
             WHERE attachment_anchor_id = $1",
        )
        .bind(attachment_anchor_id.to_vec())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        row.map(|row| {
            let state = row
                .try_get::<i16, _>("state")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
            let attempt_count = u32::try_from(
                row.try_get::<i32, _>("attempt_count")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
            if !(1..=3).contains(&state) {
                return Err(AttachmentSecurityPersistenceErrorV1::InvalidRow);
            }
            Ok(AttachmentSecurityScanJobDiagnosticsV1 {
                state,
                attempt_count,
                target_blob_receipt_present: row
                    .try_get("target_blob_receipt_present")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
                outbox_message_id_present: row
                    .try_get("outbox_message_id_present")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
                claimed: row
                    .try_get("claimed")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            })
        })
        .transpose()
    }

    pub(super) async fn pending_verdict_outbox(
        &self,
        limit: u32,
    ) -> Result<Vec<OutboxRecordV1>, AttachmentSecurityPersistenceErrorV1> {
        if !(1..=256).contains(&limit) {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT exact_envelope_bytes \
             FROM makosh_data.attachment_security_verdict_outbox \
             WHERE published_at_unix_seconds IS NULL \
             ORDER BY created_at_unix_seconds ASC, message_id ASC LIMIT $1",
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter()
            .map(|row| {
                let bytes = row
                    .try_get::<Vec<u8>, _>("exact_envelope_bytes")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
                OutboxRecordV1::accept(bytes)
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)
            })
            .collect()
    }
}
