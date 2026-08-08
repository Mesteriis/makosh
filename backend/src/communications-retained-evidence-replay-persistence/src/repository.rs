use makosh_communications_attachment_contract::{
    COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256,
    lifecycle_v1::{AttachmentSafetyStateChangedV1, AttachmentSafetyStateV1},
};
use makosh_events_protocol::{delivery::OutboxRecordV1, validation::envelope::decode_envelope_v1};
use prost::Message;
use sqlx::{PgPool, Row};

const SAFETY_CONTRACT_OWNER: &str = "communications";
const SAFETY_CONTRACT_NAME: &str = "communication_attachment_safety_state_changed";
const SAFETY_CONTRACT_MAJOR: u32 = 1;
const SAFETY_CONTRACT_REVISION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum RetainedCommunicationsReplayPhaseV1 {
    Authorized = 1,
    Published = 2,
    PublishUnavailable = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedCommunicationsEvidenceV1 {
    pub attachment_anchor_id: [u8; 16],
    pub record: OutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedCommunicationsReplayAuditV1 {
    pub operation_id: [u8; 16],
    pub logical_owner_id: String,
    pub owner_device_actor_sha256: [u8; 32],
    pub producer_registration_id: String,
    pub producer_runtime_generation: u64,
    pub producer_grant_epoch: u64,
    pub logical_attempt: u32,
    pub original_message_id: [u8; 16],
    pub original_envelope_sha256: [u8; 32],
    pub phase: RetainedCommunicationsReplayPhaseV1,
    pub recorded_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedCommunicationsReplayErrorV1 {
    InvalidInput,
    InvalidRow,
    WrongContract,
    HashMismatch,
    Conflict,
    NotFound,
    StorageUnavailable,
}

#[derive(Clone)]
pub struct CommunicationsRetainedEvidenceReplayPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl CommunicationsRetainedEvidenceReplayPersistenceV1 {
    #[must_use]
    pub fn from_owner_local_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn verify_storage_ready(&self) -> Result<(), RetainedCommunicationsReplayErrorV1> {
        sqlx::query(
            "SELECT replay.attachment_anchor_id, scan.message_id \
             FROM makosh_data.communications_retained_evidence_replay_index replay, \
                  makosh_data.communications_retained_evidence_replay_scan scan \
             WHERE FALSE",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(storage_error)
    }

    pub async fn index_existing_attachment_safety_events(
        &self,
        limit: i64,
        indexed_at_unix_seconds: i64,
    ) -> Result<usize, RetainedCommunicationsReplayErrorV1> {
        if indexed_at_unix_seconds <= 0 {
            return Err(RetainedCommunicationsReplayErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT outbox.exact_envelope_bytes \
             FROM makosh_data.communications_domain_outbox outbox \
             LEFT JOIN makosh_data.communications_retained_evidence_replay_scan scan \
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
            if is_safe_for_delivery_event(&envelope)? {
                let attachment_anchor_id = id16(&envelope.partition_key)
                    .map_err(|_| RetainedCommunicationsReplayErrorV1::WrongContract)?;
                self.index_verified(attachment_anchor_id, &record, indexed_at_unix_seconds)
                    .await?;
                indexed += 1;
            }
            self.mark_scanned(*record.message_id(), indexed_at_unix_seconds)
                .await?;
        }
        Ok(indexed)
    }

    pub async fn retained_attachment_safety_event(
        &self,
        attachment_anchor_id: [u8; 16],
    ) -> Result<RetainedCommunicationsEvidenceV1, RetainedCommunicationsReplayErrorV1> {
        if zero(&attachment_anchor_id) {
            return Err(RetainedCommunicationsReplayErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT replay.message_id, replay.envelope_sha256, replay.contract_schema_sha256, \
                    outbox.exact_envelope_bytes \
             FROM makosh_data.communications_retained_evidence_replay_index replay \
             JOIN makosh_data.communications_domain_outbox outbox \
               ON outbox.message_id = replay.message_id \
             WHERE replay.attachment_anchor_id = $1 \
               AND replay.contract_owner = $2 \
               AND replay.contract_name = $3 \
               AND replay.contract_major = $4 \
               AND replay.contract_revision = $5",
        )
        .bind(attachment_anchor_id.as_slice())
        .bind(SAFETY_CONTRACT_OWNER)
        .bind(SAFETY_CONTRACT_NAME)
        .bind(i32::try_from(SAFETY_CONTRACT_MAJOR).expect("major fits i32"))
        .bind(i32::try_from(SAFETY_CONTRACT_REVISION).expect("revision fits i32"))
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .ok_or(RetainedCommunicationsReplayErrorV1::NotFound)?;
        let message_id = row_id16(&row, "message_id")?;
        let stored_hash = row_sha256(&row, "envelope_sha256")?;
        let stored_schema = row_sha256(&row, "contract_schema_sha256")?;
        if stored_schema != COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256 {
            return Err(RetainedCommunicationsReplayErrorV1::WrongContract);
        }
        let exact_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
        let record = OutboxRecordV1::accept(exact_bytes).map_err(row_error)?;
        verify_record(&record, attachment_anchor_id, message_id, stored_hash)?;
        Ok(RetainedCommunicationsEvidenceV1 {
            attachment_anchor_id,
            record,
        })
    }

    pub async fn retained_attachment_safety_event_by_message_id(
        &self,
        message_id: [u8; 16],
    ) -> Result<RetainedCommunicationsEvidenceV1, RetainedCommunicationsReplayErrorV1> {
        if zero(&message_id) {
            return Err(RetainedCommunicationsReplayErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT replay.attachment_anchor_id, replay.envelope_sha256, \
                    replay.contract_schema_sha256, outbox.exact_envelope_bytes \
             FROM makosh_data.communications_retained_evidence_replay_index replay \
             JOIN makosh_data.communications_domain_outbox outbox \
               ON outbox.message_id = replay.message_id \
             WHERE replay.message_id = $1 \
               AND replay.contract_owner = $2 \
               AND replay.contract_name = $3 \
               AND replay.contract_major = $4 \
               AND replay.contract_revision = $5",
        )
        .bind(message_id.as_slice())
        .bind(SAFETY_CONTRACT_OWNER)
        .bind(SAFETY_CONTRACT_NAME)
        .bind(i32::try_from(SAFETY_CONTRACT_MAJOR).expect("major fits i32"))
        .bind(i32::try_from(SAFETY_CONTRACT_REVISION).expect("revision fits i32"))
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .ok_or(RetainedCommunicationsReplayErrorV1::NotFound)?;
        let attachment_anchor_id = row_id16(&row, "attachment_anchor_id")?;
        let stored_hash = row_sha256(&row, "envelope_sha256")?;
        let stored_schema = row_sha256(&row, "contract_schema_sha256")?;
        if stored_schema != COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256 {
            return Err(RetainedCommunicationsReplayErrorV1::WrongContract);
        }
        let exact_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
        let record = OutboxRecordV1::accept(exact_bytes).map_err(row_error)?;
        verify_record(&record, attachment_anchor_id, message_id, stored_hash)?;
        Ok(RetainedCommunicationsEvidenceV1 {
            attachment_anchor_id,
            record,
        })
    }

    pub async fn append_audit(
        &self,
        audit: &RetainedCommunicationsReplayAuditV1,
    ) -> Result<bool, RetainedCommunicationsReplayErrorV1> {
        validate_audit(audit)?;
        sqlx::query(
            "INSERT INTO makosh_data.communications_retained_evidence_replay_audit \
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
    ) -> Result<(), RetainedCommunicationsReplayErrorV1> {
        verify_record(
            record,
            attachment_anchor_id,
            *record.message_id(),
            *record.envelope_sha256(),
        )?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communications_retained_evidence_replay_index \
                (attachment_anchor_id, message_id, envelope_sha256, contract_owner, contract_name, \
                 contract_major, contract_revision, contract_schema_sha256, indexed_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (attachment_anchor_id) DO NOTHING",
        )
        .bind(attachment_anchor_id.as_slice())
        .bind(record.message_id().as_slice())
        .bind(record.envelope_sha256().as_slice())
        .bind(SAFETY_CONTRACT_OWNER)
        .bind(SAFETY_CONTRACT_NAME)
        .bind(i32::try_from(SAFETY_CONTRACT_MAJOR).expect("major fits i32"))
        .bind(i32::try_from(SAFETY_CONTRACT_REVISION).expect("revision fits i32"))
        .bind(COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256.as_slice())
        .bind(indexed_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if inserted.rows_affected() == 1 {
            return Ok(());
        }
        let existing = self
            .retained_attachment_safety_event(attachment_anchor_id)
            .await?;
        (existing.record.message_id() == record.message_id()
            && existing.record.envelope_sha256() == record.envelope_sha256())
        .then_some(())
        .ok_or(RetainedCommunicationsReplayErrorV1::Conflict)
    }

    async fn mark_scanned(
        &self,
        message_id: [u8; 16],
        scanned_at_unix_seconds: i64,
    ) -> Result<(), RetainedCommunicationsReplayErrorV1> {
        sqlx::query(
            "INSERT INTO makosh_data.communications_retained_evidence_replay_scan \
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
) -> Result<(), RetainedCommunicationsReplayErrorV1> {
    if record.message_id() != &expected_message_id || record.envelope_sha256() != &expected_hash {
        return Err(RetainedCommunicationsReplayErrorV1::HashMismatch);
    }
    let envelope = decode_envelope_v1(record.exact_bytes()).map_err(row_error)?;
    if !is_safe_for_delivery_event(&envelope)?
        || envelope.partition_key.as_slice() != attachment_anchor_id
    {
        return Err(RetainedCommunicationsReplayErrorV1::WrongContract);
    }
    Ok(())
}

fn is_safe_for_delivery_event(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
) -> Result<bool, RetainedCommunicationsReplayErrorV1> {
    if !is_safety_contract(envelope.contract.as_ref()) {
        return Ok(false);
    }
    let payload =
        AttachmentSafetyStateChangedV1::decode(envelope.payload.as_slice()).map_err(row_error)?;
    Ok(payload.next_state == AttachmentSafetyStateV1::SafeForDelivery as i32)
}

fn is_safety_contract(contract: Option<&makosh_events_protocol::v1::ContractRefV1>) -> bool {
    contract.is_some_and(|contract| {
        contract.owner == SAFETY_CONTRACT_OWNER
            && contract.name == SAFETY_CONTRACT_NAME
            && contract.major == SAFETY_CONTRACT_MAJOR
            && contract.revision == SAFETY_CONTRACT_REVISION
            && contract.schema_sha256 == COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256
    })
}

fn validate_audit(
    audit: &RetainedCommunicationsReplayAuditV1,
) -> Result<(), RetainedCommunicationsReplayErrorV1> {
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
        return Err(RetainedCommunicationsReplayErrorV1::InvalidInput);
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
) -> Result<[u8; 16], RetainedCommunicationsReplayErrorV1> {
    let value: Vec<u8> = row.try_get(column).map_err(row_error)?;
    id16(&value).map_err(|_| RetainedCommunicationsReplayErrorV1::InvalidRow)
}

fn row_sha256(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; 32], RetainedCommunicationsReplayErrorV1> {
    let value: Vec<u8> = row.try_get(column).map_err(row_error)?;
    let value: [u8; 32] = value.as_slice().try_into().map_err(row_error)?;
    (!zero(&value))
        .then_some(value)
        .ok_or(RetainedCommunicationsReplayErrorV1::InvalidRow)
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

fn storage_error(_: sqlx::Error) -> RetainedCommunicationsReplayErrorV1 {
    RetainedCommunicationsReplayErrorV1::StorageUnavailable
}

fn row_error<T>(_: T) -> RetainedCommunicationsReplayErrorV1 {
    RetainedCommunicationsReplayErrorV1::InvalidRow
}

fn input_error<T>(_: T) -> RetainedCommunicationsReplayErrorV1 {
    RetainedCommunicationsReplayErrorV1::InvalidInput
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    };
    use prost::Message;
    use prost_types::Timestamp;

    use super::*;

    fn record(anchor: [u8; 16]) -> OutboxRecordV1 {
        OutboxRecordV1::accept(
            DurableEnvelopeV1 {
                envelope_major: 1,
                envelope_revision: 1,
                message_id: vec![2; 16],
                contract: Some(ContractRefV1 {
                    owner: SAFETY_CONTRACT_OWNER.to_owned(),
                    name: SAFETY_CONTRACT_NAME.to_owned(),
                    major: SAFETY_CONTRACT_MAJOR,
                    revision: SAFETY_CONTRACT_REVISION,
                    schema_sha256: COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256.to_vec(),
                }),
                source: Some(SourceRefV1 {
                    module_id: "communications-runtime".to_owned(),
                    runtime_instance_id: vec![3; 16],
                    runtime_generation: 1,
                }),
                recorded_at: Some(Timestamp {
                    seconds: 1,
                    nanos: 0,
                }),
                partition_key: anchor.to_vec(),
                causation_message_id: vec![4; 16],
                correlation_id: vec![5; 16],
                actor: Some(ActorRefV1 {
                    kind: ActorKindV1::Module as i32,
                    actor_id: b"communications-runtime".to_vec(),
                }),
                trace: None,
                source_fence: Some(SourceFenceV1 {
                    kind: FenceKindV1::RuntimeLease as i32,
                    scope_id: b"communications-runtime".to_vec(),
                    epoch: 1,
                }),
                semantics: Some(Semantics::Event(EventMetadataV1 {
                    occurred_at: Some(Timestamp {
                        seconds: 1,
                        nanos: 0,
                    }),
                })),
                payload: AttachmentSafetyStateChangedV1 {
                    attachment_anchor_id: anchor.to_vec(),
                    expected_state: AttachmentSafetyStateV1::BlobAdmitted as i32,
                    next_state: AttachmentSafetyStateV1::SafeForDelivery as i32,
                    evidence_id: vec![4; 16],
                    observed_at_unix_seconds: 1,
                }
                .encode_to_vec(),
            }
            .encode_to_vec(),
        )
        .expect("record")
    }

    #[test]
    fn verifies_exact_contract_partition_message_and_hash() {
        let anchor = [1; 16];
        let record = record(anchor);
        assert_eq!(
            verify_record(
                &record,
                anchor,
                *record.message_id(),
                *record.envelope_sha256()
            ),
            Ok(())
        );
        assert_eq!(
            verify_record(
                &record,
                [9; 16],
                *record.message_id(),
                *record.envelope_sha256()
            ),
            Err(RetainedCommunicationsReplayErrorV1::WrongContract)
        );
        assert_eq!(
            verify_record(&record, anchor, *record.message_id(), [8; 32]),
            Err(RetainedCommunicationsReplayErrorV1::HashMismatch)
        );
    }

    #[test]
    fn audit_requires_current_nonzero_fences_and_sanitized_identities() {
        let mut audit = RetainedCommunicationsReplayAuditV1 {
            operation_id: [1; 16],
            logical_owner_id: "attachment_preview".to_owned(),
            owner_device_actor_sha256: [2; 32],
            producer_registration_id: "communications-runtime".to_owned(),
            producer_runtime_generation: 3,
            producer_grant_epoch: 4,
            logical_attempt: 1,
            original_message_id: [5; 16],
            original_envelope_sha256: [6; 32],
            phase: RetainedCommunicationsReplayPhaseV1::Authorized,
            recorded_at_unix_seconds: 7,
        };
        assert_eq!(validate_audit(&audit), Ok(()));
        audit.producer_grant_epoch = 0;
        assert_eq!(
            validate_audit(&audit),
            Err(RetainedCommunicationsReplayErrorV1::InvalidInput)
        );
    }
}
