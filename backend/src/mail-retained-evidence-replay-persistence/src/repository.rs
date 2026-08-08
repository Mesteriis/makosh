use makosh_attachment_security_contract::{
    ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256,
    admission::{
        ATTACHMENT_SECURITY_CONTRACT_MAJOR, ATTACHMENT_SECURITY_CONTRACT_OWNER,
        ATTACHMENT_SECURITY_CONTRACT_REVISION, ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME,
    },
};
use makosh_events_protocol::{delivery::OutboxRecordV1, validation::envelope::decode_envelope_v1};
use sqlx::{PgPool, Row};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum RetainedMailReplayPhaseV1 {
    Authorized = 1,
    Published = 2,
    PublishUnavailable = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedMailEvidenceV1 {
    pub attachment_anchor_id: [u8; 16],
    pub record: OutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedMailReplayAuditV1 {
    pub operation_id: [u8; 16],
    pub logical_owner_id: String,
    pub owner_device_actor_sha256: [u8; 32],
    pub producer_registration_id: String,
    pub producer_runtime_generation: u64,
    pub producer_grant_epoch: u64,
    pub logical_attempt: u32,
    pub original_message_id: [u8; 16],
    pub original_envelope_sha256: [u8; 32],
    pub phase: RetainedMailReplayPhaseV1,
    pub recorded_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedMailReplayErrorV1 {
    InvalidInput,
    InvalidRow,
    WrongContract,
    HashMismatch,
    Conflict,
    NotFound,
    StorageUnavailable,
}

#[derive(Clone)]
pub struct MailRetainedEvidenceReplayPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl MailRetainedEvidenceReplayPersistenceV1 {
    #[must_use]
    pub fn from_owner_local_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn verify_storage_ready(&self) -> Result<(), RetainedMailReplayErrorV1> {
        sqlx::query(
            "SELECT replay.attachment_anchor_id, scan.message_id \
             FROM makosh_data.mail_retained_evidence_replay_index replay, \
                  makosh_data.mail_retained_evidence_replay_scan scan \
             WHERE FALSE",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(storage_error)
    }

    pub async fn index_existing_scan_candidates(
        &self,
        limit: i64,
        indexed_at_unix_seconds: i64,
    ) -> Result<usize, RetainedMailReplayErrorV1> {
        if indexed_at_unix_seconds <= 0 {
            return Err(RetainedMailReplayErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT outbox.exact_envelope_bytes \
             FROM makosh_data.mail_attachment_security_outbox outbox \
             LEFT JOIN makosh_data.mail_retained_evidence_replay_scan scan \
               ON scan.message_id = outbox.message_id \
             WHERE scan.message_id IS NULL \
             ORDER BY outbox.created_at_unix_seconds ASC, outbox.message_id ASC \
             LIMIT $1",
        )
        .bind(limit.clamp(1, 256))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        let mut indexed = 0;
        for row in rows {
            let exact_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
            let record = OutboxRecordV1::accept(exact_bytes).map_err(row_error)?;
            let envelope = decode_envelope_v1(record.exact_bytes()).map_err(row_error)?;
            if !is_scan_candidate_contract(envelope.contract.as_ref()) {
                self.mark_scanned(*record.message_id(), indexed_at_unix_seconds)
                    .await?;
                continue;
            }
            let attachment_anchor_id = id16(&envelope.partition_key)
                .map_err(|_| RetainedMailReplayErrorV1::WrongContract)?;
            self.index_verified(attachment_anchor_id, &record, indexed_at_unix_seconds)
                .await?;
            self.mark_scanned(*record.message_id(), indexed_at_unix_seconds)
                .await?;
            indexed += 1;
        }
        Ok(indexed)
    }

    pub async fn retained_scan_candidate(
        &self,
        attachment_anchor_id: [u8; 16],
    ) -> Result<RetainedMailEvidenceV1, RetainedMailReplayErrorV1> {
        if zero(&attachment_anchor_id) {
            return Err(RetainedMailReplayErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT replay.message_id, replay.envelope_sha256, replay.contract_schema_sha256, \
                    outbox.exact_envelope_bytes \
             FROM makosh_data.mail_retained_evidence_replay_index replay \
             JOIN makosh_data.mail_attachment_security_outbox outbox \
               ON outbox.message_id = replay.message_id \
             WHERE replay.attachment_anchor_id = $1 \
               AND replay.contract_owner = $2 \
               AND replay.contract_name = $3 \
               AND replay.contract_major = $4 \
               AND replay.contract_revision = $5",
        )
        .bind(attachment_anchor_id.as_slice())
        .bind(ATTACHMENT_SECURITY_CONTRACT_OWNER)
        .bind(ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME)
        .bind(i32::try_from(ATTACHMENT_SECURITY_CONTRACT_MAJOR).expect("major fits i32"))
        .bind(i32::try_from(ATTACHMENT_SECURITY_CONTRACT_REVISION).expect("revision fits i32"))
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .ok_or(RetainedMailReplayErrorV1::NotFound)?;
        let message_id = row_id16(&row, "message_id")?;
        let stored_hash = row_sha256(&row, "envelope_sha256")?;
        let stored_schema = row_sha256(&row, "contract_schema_sha256")?;
        if stored_schema != ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256 {
            return Err(RetainedMailReplayErrorV1::WrongContract);
        }
        let exact_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
        let record = OutboxRecordV1::accept(exact_bytes).map_err(row_error)?;
        verify_record(&record, attachment_anchor_id, message_id, stored_hash)?;
        Ok(RetainedMailEvidenceV1 {
            attachment_anchor_id,
            record,
        })
    }

    pub async fn retained_scan_candidate_by_message_id(
        &self,
        message_id: [u8; 16],
    ) -> Result<RetainedMailEvidenceV1, RetainedMailReplayErrorV1> {
        if zero(&message_id) {
            return Err(RetainedMailReplayErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT replay.attachment_anchor_id, replay.envelope_sha256, \
                    replay.contract_schema_sha256, outbox.exact_envelope_bytes \
             FROM makosh_data.mail_retained_evidence_replay_index replay \
             JOIN makosh_data.mail_attachment_security_outbox outbox \
               ON outbox.message_id = replay.message_id \
             WHERE replay.message_id = $1 \
               AND replay.contract_owner = $2 \
               AND replay.contract_name = $3 \
               AND replay.contract_major = $4 \
               AND replay.contract_revision = $5",
        )
        .bind(message_id.as_slice())
        .bind(ATTACHMENT_SECURITY_CONTRACT_OWNER)
        .bind(ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME)
        .bind(i32::try_from(ATTACHMENT_SECURITY_CONTRACT_MAJOR).expect("major fits i32"))
        .bind(i32::try_from(ATTACHMENT_SECURITY_CONTRACT_REVISION).expect("revision fits i32"))
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .ok_or(RetainedMailReplayErrorV1::NotFound)?;
        let attachment_anchor_id = row_id16(&row, "attachment_anchor_id")?;
        let stored_hash = row_sha256(&row, "envelope_sha256")?;
        let stored_schema = row_sha256(&row, "contract_schema_sha256")?;
        if stored_schema != ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256 {
            return Err(RetainedMailReplayErrorV1::WrongContract);
        }
        let exact_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
        let record = OutboxRecordV1::accept(exact_bytes).map_err(row_error)?;
        verify_record(&record, attachment_anchor_id, message_id, stored_hash)?;
        Ok(RetainedMailEvidenceV1 {
            attachment_anchor_id,
            record,
        })
    }

    pub async fn append_audit(
        &self,
        audit: &RetainedMailReplayAuditV1,
    ) -> Result<bool, RetainedMailReplayErrorV1> {
        validate_audit(audit)?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_retained_evidence_replay_audit \
                (operation_id, logical_owner_id, owner_device_actor_sha256, producer_registration_id, \
                 producer_runtime_generation, producer_grant_epoch, logical_attempt, \
                 original_message_id, original_envelope_sha256, phase, recorded_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (operation_id, original_message_id, logical_attempt, phase) DO NOTHING",
        )
        .bind(audit.operation_id.as_slice())
        .bind(&audit.logical_owner_id)
        .bind(audit.owner_device_actor_sha256.as_slice())
        .bind(&audit.producer_registration_id)
        .bind(i64::try_from(audit.producer_runtime_generation).map_err(input_error)?)
        .bind(i64::try_from(audit.producer_grant_epoch).map_err(input_error)?)
        .bind(i32::try_from(audit.logical_attempt).map_err(input_error)?)
        .bind(audit.original_message_id.as_slice())
        .bind(audit.original_envelope_sha256.as_slice())
        .bind(audit.phase as i16)
        .bind(audit.recorded_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(storage_error)
    }

    async fn index_verified(
        &self,
        attachment_anchor_id: [u8; 16],
        record: &OutboxRecordV1,
        indexed_at_unix_seconds: i64,
    ) -> Result<(), RetainedMailReplayErrorV1> {
        verify_record(
            record,
            attachment_anchor_id,
            *record.message_id(),
            *record.envelope_sha256(),
        )?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_retained_evidence_replay_index \
                (attachment_anchor_id, message_id, envelope_sha256, contract_owner, contract_name, \
                 contract_major, contract_revision, contract_schema_sha256, indexed_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (attachment_anchor_id) DO NOTHING",
        )
        .bind(attachment_anchor_id.as_slice())
        .bind(record.message_id().as_slice())
        .bind(record.envelope_sha256().as_slice())
        .bind(ATTACHMENT_SECURITY_CONTRACT_OWNER)
        .bind(ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME)
        .bind(i32::try_from(ATTACHMENT_SECURITY_CONTRACT_MAJOR).expect("major fits i32"))
        .bind(i32::try_from(ATTACHMENT_SECURITY_CONTRACT_REVISION).expect("revision fits i32"))
        .bind(ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256.as_slice())
        .bind(indexed_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if inserted.rows_affected() == 1 {
            return Ok(());
        }
        let existing = self.retained_scan_candidate(attachment_anchor_id).await?;
        (existing.record.message_id() == record.message_id()
            && existing.record.envelope_sha256() == record.envelope_sha256())
        .then_some(())
        .ok_or(RetainedMailReplayErrorV1::Conflict)
    }

    async fn mark_scanned(
        &self,
        message_id: [u8; 16],
        scanned_at_unix_seconds: i64,
    ) -> Result<(), RetainedMailReplayErrorV1> {
        sqlx::query(
            "INSERT INTO makosh_data.mail_retained_evidence_replay_scan \
                (message_id, scanned_at_unix_seconds) VALUES ($1, $2) \
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(message_id.as_slice())
        .bind(scanned_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(storage_error)
    }
}

fn verify_record(
    record: &OutboxRecordV1,
    attachment_anchor_id: [u8; 16],
    expected_message_id: [u8; 16],
    expected_hash: [u8; 32],
) -> Result<(), RetainedMailReplayErrorV1> {
    if record.message_id() != &expected_message_id || record.envelope_sha256() != &expected_hash {
        return Err(RetainedMailReplayErrorV1::HashMismatch);
    }
    let envelope = decode_envelope_v1(record.exact_bytes()).map_err(row_error)?;
    if !is_scan_candidate_contract(envelope.contract.as_ref())
        || envelope.partition_key.as_slice() != attachment_anchor_id
    {
        return Err(RetainedMailReplayErrorV1::WrongContract);
    }
    Ok(())
}

fn is_scan_candidate_contract(
    contract: Option<&makosh_events_protocol::v1::ContractRefV1>,
) -> bool {
    contract.is_some_and(|contract| {
        contract.owner == ATTACHMENT_SECURITY_CONTRACT_OWNER
            && contract.name == ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME
            && contract.major == ATTACHMENT_SECURITY_CONTRACT_MAJOR
            && contract.revision == ATTACHMENT_SECURITY_CONTRACT_REVISION
            && contract.schema_sha256 == ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256
    })
}

fn validate_audit(audit: &RetainedMailReplayAuditV1) -> Result<(), RetainedMailReplayErrorV1> {
    if zero(&audit.operation_id)
        || !valid_identity(&audit.logical_owner_id)
        || zero(&audit.owner_device_actor_sha256)
        || !valid_identity(&audit.producer_registration_id)
        || audit.producer_runtime_generation == 0
        || audit.producer_grant_epoch == 0
        || !(1..=1024).contains(&audit.logical_attempt)
        || zero(&audit.original_message_id)
        || zero(&audit.original_envelope_sha256)
        || audit.recorded_at_unix_seconds <= 0
    {
        return Err(RetainedMailReplayErrorV1::InvalidInput);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], ()> {
    let value: [u8; 16] = value.try_into().map_err(|_| ())?;
    (!zero(&value)).then_some(value).ok_or(())
}

fn row_id16(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; 16], RetainedMailReplayErrorV1> {
    let value: Vec<u8> = row.try_get(column).map_err(row_error)?;
    id16(&value).map_err(|_| RetainedMailReplayErrorV1::InvalidRow)
}

fn row_sha256(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; 32], RetainedMailReplayErrorV1> {
    let value: Vec<u8> = row.try_get(column).map_err(row_error)?;
    let value: [u8; 32] = value.as_slice().try_into().map_err(row_error)?;
    (!zero(&value))
        .then_some(value)
        .ok_or(RetainedMailReplayErrorV1::InvalidRow)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

fn storage_error(_: sqlx::Error) -> RetainedMailReplayErrorV1 {
    RetainedMailReplayErrorV1::StorageUnavailable
}

fn row_error<T>(_: T) -> RetainedMailReplayErrorV1 {
    RetainedMailReplayErrorV1::InvalidRow
}

fn input_error<T>(_: T) -> RetainedMailReplayErrorV1 {
    RetainedMailReplayErrorV1::InvalidInput
}

#[cfg(test)]
mod tests {
    use makosh_attachment_security_contract::{
        AttachmentSecurityObservationContextV1, AttachmentSecurityScanCandidateFactV1,
        build_attachment_security_scan_candidate_outbox_record_v1,
    };

    use super::*;

    fn record(anchor: [u8; 16]) -> OutboxRecordV1 {
        build_attachment_security_scan_candidate_outbox_record_v1(
            &AttachmentSecurityScanCandidateFactV1 {
                attachment_anchor_id: anchor,
                blob_reference_id: [2; 16],
                declared_size: 1,
                blob_receipt_sha256: [3; 32],
                custody_transfer_source_proof: vec![4],
                source_observation_id: [5; 16],
                correlation_id: [6; 16],
                observed_at_unix_seconds: 1,
            },
            &AttachmentSecurityObservationContextV1 {
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 1,
                module_id: "makosh-mail-runtime".to_owned(),
                recorded_at_unix_seconds: 1,
                recorded_at_nanos: 0,
            },
        )
        .expect("candidate record")
    }

    #[test]
    fn exact_scan_candidate_is_anchor_bound() {
        let anchor = [1; 16];
        let record = record(anchor);
        assert_eq!(
            verify_record(
                &record,
                anchor,
                *record.message_id(),
                *record.envelope_sha256(),
            ),
            Ok(())
        );
        assert_eq!(
            verify_record(
                &record,
                [9; 16],
                *record.message_id(),
                *record.envelope_sha256(),
            ),
            Err(RetainedMailReplayErrorV1::WrongContract)
        );
    }

    #[test]
    fn audit_requires_current_nonzero_fences() {
        let mut audit = RetainedMailReplayAuditV1 {
            operation_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            owner_device_actor_sha256: [2; 32],
            producer_registration_id: "mail-registration-1".to_owned(),
            producer_runtime_generation: 1,
            producer_grant_epoch: 1,
            logical_attempt: 1,
            original_message_id: [3; 16],
            original_envelope_sha256: [4; 32],
            phase: RetainedMailReplayPhaseV1::Authorized,
            recorded_at_unix_seconds: 1,
        };
        assert_eq!(validate_audit(&audit), Ok(()));
        audit.producer_runtime_generation = 0;
        assert_eq!(
            validate_audit(&audit),
            Err(RetainedMailReplayErrorV1::InvalidInput)
        );
    }
}
