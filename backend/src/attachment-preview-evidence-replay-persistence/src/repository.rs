use makosh_attachment_preview_evidence_replay_api::wire::{
    AttachmentPreviewEvidenceReplayErrorV1, AttachmentPreviewEvidenceReplayStateV1,
};
use makosh_attachment_preview_evidence_replay_core::{
    AuthenticatedReplayOperationRequestV1, ReplayFailureV1, ReplayOperationStateV1,
    ReplayProducerOutcomeV1, ReplayProducerResultV1, ReplayProducerV1,
    accepted_replay_operation_v1, observe_producer_result_v1, plan_replay_operation_v1,
    replay_operation_status_v1,
};
use makosh_events_protocol::{delivery::OutboxRecordV1, validation::envelope::decode_envelope_v1};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentPreviewEvidenceReplayPersistenceV1,
    model::{
        PersistedReplayOperationV1, decode_command_v1, decode_result_v1, id16, id32,
        request_fingerprint_v1,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    WrongContract,
    Conflict,
    NotFound,
    StorageUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCommandOutboxRecordV1 {
    pub producer: ReplayProducerV1,
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
}

impl ReplayCommandOutboxRecordV1 {
    pub fn accept(
        producer: ReplayProducerV1,
        exact_envelope_bytes: Vec<u8>,
    ) -> Result<Self, ReplayPersistenceErrorV1> {
        let record = OutboxRecordV1::accept(exact_envelope_bytes)
            .map_err(|_| ReplayPersistenceErrorV1::WrongContract)?;
        Ok(Self {
            producer,
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            exact_envelope_bytes: record.exact_bytes().to_vec(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayResultInboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
    pub operation_id: [u8; 16],
}

impl ReplayResultInboxRecordV1 {
    pub fn accept(exact_envelope_bytes: Vec<u8>) -> Result<Self, ReplayPersistenceErrorV1> {
        let envelope = decode_envelope_v1(&exact_envelope_bytes)
            .map_err(|_| ReplayPersistenceErrorV1::WrongContract)?;
        Ok(Self {
            message_id: id16(&envelope.message_id)?,
            envelope_sha256: Sha256::digest(&exact_envelope_bytes).into(),
            exact_envelope_bytes,
            operation_id: id16(&envelope.correlation_id)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayOperationCreateOutcomeV1 {
    Created(PersistedReplayOperationV1),
    Replayed(PersistedReplayOperationV1),
    OperationCollision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayResultAcceptOutcomeV1 {
    Applied(PersistedReplayOperationV1),
    Replayed(PersistedReplayOperationV1),
}

impl AttachmentPreviewEvidenceReplayPersistenceV1 {
    pub async fn create_operation(
        &self,
        request: &AuthenticatedReplayOperationRequestV1,
        commands: [ReplayCommandOutboxRecordV1; 2],
        accepted_at_unix_seconds: i64,
    ) -> Result<ReplayOperationCreateOutcomeV1, ReplayPersistenceErrorV1> {
        plan_replay_operation_v1(request.clone())
            .map_err(|_| ReplayPersistenceErrorV1::InvalidInput)?;
        if accepted_at_unix_seconds <= 0 {
            return Err(ReplayPersistenceErrorV1::InvalidInput);
        }
        let communications = command_for(&commands, ReplayProducerV1::Communications)?;
        let mail = command_for(&commands, ReplayProducerV1::Mail)?;
        verify_command_record(communications, request)?;
        verify_command_record(mail, request)?;

        let fingerprint = request_fingerprint_v1(request);
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_preview_evidence_replay_operations (operation_id,attachment_anchor_id,logical_owner_id,owner_device_actor_sha256,request_fingerprint,state,error,state_revision,accepted_at_unix_seconds,completed_at_unix_seconds) VALUES ($1,$2,$3,$4,$5,$6,$7,1,$8,NULL) ON CONFLICT (operation_id) DO NOTHING",
        )
        .bind(request.operation_id.as_slice())
        .bind(request.attachment_anchor_id.as_slice())
        .bind(&request.logical_owner_id)
        .bind(request.owner_device_actor_sha256.as_slice())
        .bind(fingerprint.as_slice())
        .bind(state_code(AttachmentPreviewEvidenceReplayStateV1::AwaitingProducers))
        .bind(error_code(AttachmentPreviewEvidenceReplayErrorV1::Unspecified))
        .bind(accepted_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() == 0 {
            let existing = load_operation(&mut transaction, request.operation_id, true)
                .await?
                .ok_or(ReplayPersistenceErrorV1::InvalidRow)?;
            transaction.commit().await.map_err(storage_unavailable)?;
            return if request_fingerprint_v1(&existing.persisted.request) == fingerprint {
                Ok(ReplayOperationCreateOutcomeV1::Replayed(existing.persisted))
            } else {
                Ok(ReplayOperationCreateOutcomeV1::OperationCollision)
            };
        }

        insert_producer(
            &mut transaction,
            request.operation_id,
            ReplayProducerV1::Communications,
        )
        .await?;
        insert_producer(
            &mut transaction,
            request.operation_id,
            ReplayProducerV1::Mail,
        )
        .await?;
        insert_command(
            &mut transaction,
            request.operation_id,
            communications,
            accepted_at_unix_seconds,
        )
        .await?;
        insert_command(
            &mut transaction,
            request.operation_id,
            mail,
            accepted_at_unix_seconds,
        )
        .await?;

        let created = load_operation(&mut transaction, request.operation_id, true)
            .await?
            .ok_or(ReplayPersistenceErrorV1::InvalidRow)?
            .persisted;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(ReplayOperationCreateOutcomeV1::Created(created))
    }

    pub async fn operation_status(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
    ) -> Result<Option<PersistedReplayOperationV1>, ReplayPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || !nonzero(&operation_id) {
            return Err(ReplayPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let loaded = load_operation(&mut transaction, operation_id, false).await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        match loaded {
            Some(value) if value.persisted.request.logical_owner_id == logical_owner_id => {
                Ok(Some(value.persisted))
            }
            Some(_) => Err(ReplayPersistenceErrorV1::NotFound),
            None => Ok(None),
        }
    }

    pub async fn pending_commands(
        &self,
        limit: u32,
    ) -> Result<Vec<ReplayCommandOutboxRecordV1>, ReplayPersistenceErrorV1> {
        if limit == 0 || limit > 64 {
            return Err(ReplayPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let rows = sqlx::query(
            "SELECT message_id,envelope_sha256,exact_envelope_bytes,operation_id,producer FROM makosh_data.attachment_preview_evidence_replay_anchor_command_outbox WHERE published_at_unix_seconds IS NULL ORDER BY created_at_unix_seconds,message_id LIMIT $1",
        )
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        let mut pending = Vec::with_capacity(rows.len());
        for row in rows {
            let operation_id = id16(row.try_get("operation_id").map_err(invalid_row)?)?;
            let producer = producer_from_code(row.try_get("producer").map_err(invalid_row)?)?;
            let loaded = load_operation(&mut transaction, operation_id, false)
                .await?
                .ok_or(ReplayPersistenceErrorV1::InvalidRow)?;
            let record = ReplayCommandOutboxRecordV1 {
                producer,
                message_id: id16(row.try_get("message_id").map_err(invalid_row)?)?,
                envelope_sha256: id32(row.try_get("envelope_sha256").map_err(invalid_row)?)?,
                exact_envelope_bytes: row.try_get("exact_envelope_bytes").map_err(invalid_row)?,
            };
            verify_command_record(&record, &loaded.persisted.request)?;
            pending.push(record);
        }
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(pending)
    }

    pub async fn mark_command_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<(), ReplayPersistenceErrorV1> {
        if !nonzero(&message_id) || published_at_unix_seconds <= 0 {
            return Err(ReplayPersistenceErrorV1::InvalidInput);
        }
        let result = sqlx::query(
            "UPDATE makosh_data.attachment_preview_evidence_replay_anchor_command_outbox SET published_at_unix_seconds=$1 WHERE message_id=$2 AND published_at_unix_seconds IS NULL",
        )
        .bind(published_at_unix_seconds)
        .bind(message_id.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_unavailable)?;
        if result.rows_affected() == 1 {
            return Ok(());
        }
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM makosh_data.attachment_preview_evidence_replay_anchor_command_outbox WHERE message_id=$1 AND published_at_unix_seconds IS NOT NULL)",
        )
        .bind(message_id.as_slice())
        .fetch_one(&self.pool)
        .await
        .map_err(storage_unavailable)?;
        exists
            .then_some(())
            .ok_or(ReplayPersistenceErrorV1::NotFound)
    }

    pub async fn accept_producer_result(
        &self,
        producer: ReplayProducerV1,
        record: &ReplayResultInboxRecordV1,
        accepted_at_unix_seconds: i64,
    ) -> Result<ReplayResultAcceptOutcomeV1, ReplayPersistenceErrorV1> {
        if accepted_at_unix_seconds <= 0 {
            return Err(ReplayPersistenceErrorV1::InvalidInput);
        }
        verify_result_record(record)?;
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let loaded = load_operation(&mut transaction, record.operation_id, true)
            .await?
            .ok_or(ReplayPersistenceErrorV1::NotFound)?;
        let command_message_id =
            load_command_message_id(&mut transaction, record.operation_id, producer).await?;
        let result = decode_result_v1(
            producer,
            &record.exact_envelope_bytes,
            record.operation_id,
            command_message_id,
        )?;
        let inbox = inspect_result_inbox(&mut transaction, producer, record).await?;
        match inbox {
            ResultInboxStateV1::Duplicate => {
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(ReplayResultAcceptOutcomeV1::Replayed(loaded.persisted));
            }
            ResultInboxStateV1::Conflict => return Err(ReplayPersistenceErrorV1::Conflict),
            ResultInboxStateV1::New => {}
        }
        let mut core_state = loaded.core_state;
        observe_producer_result_v1(&mut core_state, result.clone())
            .map_err(|_| ReplayPersistenceErrorV1::Conflict)?;
        insert_result_inbox(&mut transaction, producer, record, accepted_at_unix_seconds).await?;
        update_producer_result(&mut transaction, record.operation_id, &result).await?;
        let (next_state, next_error) = replay_operation_status_v1(&core_state);
        let next_revision = loaded
            .persisted
            .state_revision
            .checked_add(1)
            .ok_or(ReplayPersistenceErrorV1::InvalidRow)?;
        let completed_at = terminal(next_state).then_some(accepted_at_unix_seconds);
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_preview_evidence_replay_operations SET state=$1,error=$2,state_revision=$3,completed_at_unix_seconds=$4 WHERE operation_id=$5 AND state_revision=$6",
        )
        .bind(state_code(next_state))
        .bind(error_code(next_error))
        .bind(i64::try_from(next_revision).map_err(invalid_input)?)
        .bind(completed_at)
        .bind(record.operation_id.as_slice())
        .bind(i64::try_from(loaded.persisted.state_revision).map_err(invalid_input)?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if updated.rows_affected() != 1 {
            return Err(ReplayPersistenceErrorV1::Conflict);
        }
        let persisted = PersistedReplayOperationV1 {
            request: core_state.request,
            state: next_state,
            error: next_error,
            state_revision: next_revision,
            accepted_at_unix_seconds: loaded.persisted.accepted_at_unix_seconds,
            completed_at_unix_seconds: completed_at,
        };
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(ReplayResultAcceptOutcomeV1::Applied(persisted))
    }
}

struct LoadedOperationV1 {
    persisted: PersistedReplayOperationV1,
    core_state: ReplayOperationStateV1,
}

async fn load_operation(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: [u8; 16],
    for_update: bool,
) -> Result<Option<LoadedOperationV1>, ReplayPersistenceErrorV1> {
    let row = if for_update {
        sqlx::query(
            "SELECT operation_id,attachment_anchor_id,logical_owner_id,owner_device_actor_sha256,state,error,state_revision,accepted_at_unix_seconds,completed_at_unix_seconds FROM makosh_data.attachment_preview_evidence_replay_operations WHERE operation_id=$1 FOR UPDATE",
        )
        .bind(operation_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_unavailable)?
    } else {
        sqlx::query(
            "SELECT operation_id,attachment_anchor_id,logical_owner_id,owner_device_actor_sha256,state,error,state_revision,accepted_at_unix_seconds,completed_at_unix_seconds FROM makosh_data.attachment_preview_evidence_replay_operations WHERE operation_id=$1",
        )
        .bind(operation_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_unavailable)?
    };
    let Some(row) = row else {
        return Ok(None);
    };
    let producers = load_producers(transaction, operation_id).await?;
    if producers.len() != 2
        || !producers
            .iter()
            .any(|value| value.producer == ReplayProducerV1::Communications)
        || !producers
            .iter()
            .any(|value| value.producer == ReplayProducerV1::Mail)
    {
        return Err(ReplayPersistenceErrorV1::InvalidRow);
    }
    let request = AuthenticatedReplayOperationRequestV1 {
        operation_id: id16(row.try_get("operation_id").map_err(invalid_row)?)?,
        attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(invalid_row)?,
        owner_device_actor_sha256: id32(
            row.try_get("owner_device_actor_sha256")
                .map_err(invalid_row)?,
        )?,
    };
    let mut core_state = accepted_replay_operation_v1(request.clone())
        .map_err(|_| ReplayPersistenceErrorV1::InvalidRow)?;
    for producer in producers {
        if let Some(result) = producer.result {
            observe_producer_result_v1(&mut core_state, result)
                .map_err(|_| ReplayPersistenceErrorV1::InvalidRow)?;
        }
    }
    let persisted = PersistedReplayOperationV1 {
        request,
        state: state_from_code(row.try_get("state").map_err(invalid_row)?)?,
        error: error_from_code(row.try_get("error").map_err(invalid_row)?)?,
        state_revision: u64::try_from(
            row.try_get::<i64, _>("state_revision")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        accepted_at_unix_seconds: row
            .try_get("accepted_at_unix_seconds")
            .map_err(invalid_row)?,
        completed_at_unix_seconds: row
            .try_get("completed_at_unix_seconds")
            .map_err(invalid_row)?,
    };
    if persisted.accepted_at_unix_seconds <= 0
        || persisted.state_revision == 0
        || persisted
            .completed_at_unix_seconds
            .is_some_and(|value| value <= 0)
        || replay_operation_status_v1(&core_state) != (persisted.state, persisted.error)
    {
        return Err(ReplayPersistenceErrorV1::InvalidRow);
    }
    Ok(Some(LoadedOperationV1 {
        persisted,
        core_state,
    }))
}

struct LoadedProducerV1 {
    producer: ReplayProducerV1,
    result: Option<ReplayProducerResultV1>,
}

async fn load_producers(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: [u8; 16],
) -> Result<Vec<LoadedProducerV1>, ReplayPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT producer,outcome,failure FROM makosh_data.attachment_preview_evidence_replay_anchor_producers WHERE operation_id=$1 ORDER BY producer",
    )
    .bind(operation_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        let producer = producer_from_code(row.try_get("producer").map_err(invalid_row)?)?;
        let ids = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT original_message_id FROM makosh_data.attachment_preview_evidence_replay_anchor_result_messages WHERE operation_id=$1 AND producer=$2 ORDER BY ordinal",
        )
        .bind(operation_id.as_slice())
        .bind(producer_code(producer))
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage_unavailable)?
        .into_iter()
        .map(|value| id16(&value))
        .collect::<Result<Vec<_>, _>>()?;
        let outcome_code: i16 = row.try_get("outcome").map_err(invalid_row)?;
        let failure_code_value: i16 = row.try_get("failure").map_err(invalid_row)?;
        let result = if outcome_code == 0 {
            if failure_code_value != 0 || !ids.is_empty() {
                return Err(ReplayPersistenceErrorV1::InvalidRow);
            }
            None
        } else {
            Some(ReplayProducerResultV1 {
                producer,
                original_message_ids: ids.clone(),
                outcome: outcome_from_code(outcome_code)?,
                failure: failure_from_code(failure_code_value)?,
            })
        };
        values.push(LoadedProducerV1 { producer, result });
    }
    Ok(values)
}

async fn insert_producer(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: [u8; 16],
    producer: ReplayProducerV1,
) -> Result<(), ReplayPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_preview_evidence_replay_anchor_producers (operation_id,producer,outcome,failure) VALUES ($1,$2,0,0)",
    )
    .bind(operation_id.as_slice())
    .bind(producer_code(producer))
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    Ok(())
}

async fn insert_command(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: [u8; 16],
    command: &ReplayCommandOutboxRecordV1,
    created_at_unix_seconds: i64,
) -> Result<(), ReplayPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_preview_evidence_replay_anchor_command_outbox (message_id,envelope_sha256,exact_envelope_bytes,operation_id,producer,created_at_unix_seconds,published_at_unix_seconds) VALUES ($1,$2,$3,$4,$5,$6,NULL)",
    )
    .bind(command.message_id.as_slice())
    .bind(command.envelope_sha256.as_slice())
    .bind(&command.exact_envelope_bytes)
    .bind(operation_id.as_slice())
    .bind(producer_code(command.producer))
    .bind(created_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    Ok(())
}

async fn load_command_message_id(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: [u8; 16],
    producer: ReplayProducerV1,
) -> Result<[u8; 16], ReplayPersistenceErrorV1> {
    let value = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT message_id FROM makosh_data.attachment_preview_evidence_replay_anchor_command_outbox WHERE operation_id=$1 AND producer=$2",
    )
    .bind(operation_id.as_slice())
    .bind(producer_code(producer))
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(ReplayPersistenceErrorV1::InvalidRow)?;
    id16(&value)
}

enum ResultInboxStateV1 {
    New,
    Duplicate,
    Conflict,
}

async fn inspect_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    producer: ReplayProducerV1,
    record: &ReplayResultInboxRecordV1,
) -> Result<ResultInboxStateV1, ReplayPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT message_id,envelope_sha256,operation_id,producer FROM makosh_data.attachment_preview_evidence_replay_anchor_result_inbox WHERE message_id=$1 OR (operation_id=$2 AND producer=$3) FOR UPDATE",
    )
    .bind(record.message_id.as_slice())
    .bind(record.operation_id.as_slice())
    .bind(producer_code(producer))
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    if rows.is_empty() {
        return Ok(ResultInboxStateV1::New);
    }
    if rows.len() != 1 {
        return Ok(ResultInboxStateV1::Conflict);
    }
    let row = &rows[0];
    let exact = id16(row.try_get("message_id").map_err(invalid_row)?)? == record.message_id
        && id32(row.try_get("envelope_sha256").map_err(invalid_row)?)? == record.envelope_sha256
        && id16(row.try_get("operation_id").map_err(invalid_row)?)? == record.operation_id
        && producer_from_code(row.try_get("producer").map_err(invalid_row)?)? == producer;
    Ok(if exact {
        ResultInboxStateV1::Duplicate
    } else {
        ResultInboxStateV1::Conflict
    })
}

async fn insert_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    producer: ReplayProducerV1,
    record: &ReplayResultInboxRecordV1,
    accepted_at_unix_seconds: i64,
) -> Result<(), ReplayPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_preview_evidence_replay_anchor_result_inbox (message_id,envelope_sha256,operation_id,producer,accepted_at_unix_seconds) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(record.operation_id.as_slice())
    .bind(producer_code(producer))
    .bind(accepted_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    Ok(())
}

async fn update_producer_result(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: [u8; 16],
    result: &ReplayProducerResultV1,
) -> Result<(), ReplayPersistenceErrorV1> {
    let updated = sqlx::query(
        "UPDATE makosh_data.attachment_preview_evidence_replay_anchor_producers SET outcome=$1,failure=$2 WHERE operation_id=$3 AND producer=$4 AND outcome=0 AND failure=0",
    )
    .bind(outcome_code(result.outcome))
    .bind(failure_code(result.failure))
    .bind(operation_id.as_slice())
    .bind(producer_code(result.producer))
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    if updated.rows_affected() != 1 {
        return Err(ReplayPersistenceErrorV1::Conflict);
    }
    for (ordinal, message_id) in result.original_message_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO makosh_data.attachment_preview_evidence_replay_anchor_result_messages (operation_id,producer,ordinal,original_message_id) VALUES ($1,$2,$3,$4)",
        )
        .bind(operation_id.as_slice())
        .bind(producer_code(result.producer))
        .bind(i16::try_from(ordinal).map_err(invalid_input)?)
        .bind(message_id.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(storage_unavailable)?;
    }
    Ok(())
}

fn verify_command_record(
    record: &ReplayCommandOutboxRecordV1,
    request: &AuthenticatedReplayOperationRequestV1,
) -> Result<(), ReplayPersistenceErrorV1> {
    let accepted = OutboxRecordV1::accept(record.exact_envelope_bytes.clone())
        .map_err(|_| ReplayPersistenceErrorV1::WrongContract)?;
    if accepted.message_id() != &record.message_id
        || accepted.envelope_sha256() != &record.envelope_sha256
    {
        return Err(ReplayPersistenceErrorV1::Conflict);
    }
    let decoded_message_id =
        decode_command_v1(record.producer, &record.exact_envelope_bytes, request)?;
    (decoded_message_id == record.message_id)
        .then_some(())
        .ok_or(ReplayPersistenceErrorV1::Conflict)
}

fn verify_result_record(
    record: &ReplayResultInboxRecordV1,
) -> Result<(), ReplayPersistenceErrorV1> {
    let envelope = decode_envelope_v1(&record.exact_envelope_bytes)
        .map_err(|_| ReplayPersistenceErrorV1::WrongContract)?;
    let exact = id16(&envelope.message_id)? == record.message_id
        && id16(&envelope.correlation_id)? == record.operation_id
        && Sha256::digest(&record.exact_envelope_bytes).as_slice() == record.envelope_sha256;
    exact
        .then_some(())
        .ok_or(ReplayPersistenceErrorV1::Conflict)
}

fn command_for(
    commands: &[ReplayCommandOutboxRecordV1; 2],
    producer: ReplayProducerV1,
) -> Result<&ReplayCommandOutboxRecordV1, ReplayPersistenceErrorV1> {
    let matching = commands
        .iter()
        .filter(|value| value.producer == producer)
        .collect::<Vec<_>>();
    (matching.len() == 1)
        .then_some(matching[0])
        .ok_or(ReplayPersistenceErrorV1::InvalidInput)
}

fn producer_code(value: ReplayProducerV1) -> i16 {
    value as i16
}
fn producer_from_code(value: i16) -> Result<ReplayProducerV1, ReplayPersistenceErrorV1> {
    match value {
        1 => Ok(ReplayProducerV1::Communications),
        2 => Ok(ReplayProducerV1::Mail),
        _ => Err(ReplayPersistenceErrorV1::InvalidRow),
    }
}
fn outcome_code(value: ReplayProducerOutcomeV1) -> i16 {
    match value {
        ReplayProducerOutcomeV1::Published => 1,
        ReplayProducerOutcomeV1::AlreadyPublished => 2,
        ReplayProducerOutcomeV1::Rejected => 3,
        ReplayProducerOutcomeV1::Unavailable => 4,
    }
}
fn outcome_from_code(value: i16) -> Result<ReplayProducerOutcomeV1, ReplayPersistenceErrorV1> {
    match value {
        1 => Ok(ReplayProducerOutcomeV1::Published),
        2 => Ok(ReplayProducerOutcomeV1::AlreadyPublished),
        3 => Ok(ReplayProducerOutcomeV1::Rejected),
        4 => Ok(ReplayProducerOutcomeV1::Unavailable),
        _ => Err(ReplayPersistenceErrorV1::InvalidRow),
    }
}
fn failure_code(value: ReplayFailureV1) -> i16 {
    match value {
        ReplayFailureV1::None => 0,
        ReplayFailureV1::NotFound => 1,
        ReplayFailureV1::HashMismatch => 2,
        ReplayFailureV1::WrongContract => 3,
        ReplayFailureV1::StaleRuntimeFence => 4,
        ReplayFailureV1::StaleGrantFence => 5,
        ReplayFailureV1::OwnerMismatch => 6,
        ReplayFailureV1::PublishUnavailable => 7,
    }
}
fn failure_from_code(value: i16) -> Result<ReplayFailureV1, ReplayPersistenceErrorV1> {
    match value {
        0 => Ok(ReplayFailureV1::None),
        1 => Ok(ReplayFailureV1::NotFound),
        2 => Ok(ReplayFailureV1::HashMismatch),
        3 => Ok(ReplayFailureV1::WrongContract),
        4 => Ok(ReplayFailureV1::StaleRuntimeFence),
        5 => Ok(ReplayFailureV1::StaleGrantFence),
        6 => Ok(ReplayFailureV1::OwnerMismatch),
        7 => Ok(ReplayFailureV1::PublishUnavailable),
        _ => Err(ReplayPersistenceErrorV1::InvalidRow),
    }
}
fn state_code(value: AttachmentPreviewEvidenceReplayStateV1) -> i16 {
    value as i16
}
fn state_from_code(
    value: i16,
) -> Result<AttachmentPreviewEvidenceReplayStateV1, ReplayPersistenceErrorV1> {
    AttachmentPreviewEvidenceReplayStateV1::try_from(i32::from(value))
        .ok()
        .filter(|value| *value != AttachmentPreviewEvidenceReplayStateV1::Unspecified)
        .ok_or(ReplayPersistenceErrorV1::InvalidRow)
}
fn error_code(value: AttachmentPreviewEvidenceReplayErrorV1) -> i16 {
    value as i16
}
fn error_from_code(
    value: i16,
) -> Result<AttachmentPreviewEvidenceReplayErrorV1, ReplayPersistenceErrorV1> {
    AttachmentPreviewEvidenceReplayErrorV1::try_from(i32::from(value))
        .map_err(|_| ReplayPersistenceErrorV1::InvalidRow)
}
fn terminal(value: AttachmentPreviewEvidenceReplayStateV1) -> bool {
    matches!(
        value,
        AttachmentPreviewEvidenceReplayStateV1::Completed
            | AttachmentPreviewEvidenceReplayStateV1::Unavailable
            | AttachmentPreviewEvidenceReplayStateV1::Rejected
    )
}
fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}
fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
fn storage_unavailable(_: sqlx::Error) -> ReplayPersistenceErrorV1 {
    ReplayPersistenceErrorV1::StorageUnavailable
}
fn invalid_row<T>(_: T) -> ReplayPersistenceErrorV1 {
    ReplayPersistenceErrorV1::InvalidRow
}
fn invalid_input<T>(_: T) -> ReplayPersistenceErrorV1 {
    ReplayPersistenceErrorV1::InvalidInput
}

#[cfg(test)]
mod tests {
    use makosh_communications_retained_evidence_replay_contract::{
        CommunicationsReplayCommandEnvelopeContextV1,
        build_communications_replay_command_outbox_v1, wire::ReplayCommunicationsEvidenceCommandV1,
    };

    use super::*;

    #[test]
    fn exact_command_record_is_bound_to_authenticated_request() {
        let request = request();
        let command = ReplayCommunicationsEvidenceCommandV1 {
            operation_id: request.operation_id.to_vec(),
            logical_owner_id: request.logical_owner_id.clone(),
            owner_device_actor_sha256: request.owner_device_actor_sha256.to_vec(),
            attachment_anchor_id: request.attachment_anchor_id.to_vec(),
        };
        let outbox = build_communications_replay_command_outbox_v1(
            command,
            &CommunicationsReplayCommandEnvelopeContextV1 {
                runtime_instance_id: "replay-workflow-1".to_owned(),
                runtime_generation: 5,
                recorded_at_unix_seconds: 1_700_000_000,
                recorded_at_nanos: 0,
                deadline_unix_seconds: 1_700_000_300,
                logical_attempt: 1,
            },
        )
        .expect("outbox");
        let record = ReplayCommandOutboxRecordV1::accept(
            ReplayProducerV1::Communications,
            outbox.exact_bytes().to_vec(),
        )
        .expect("record");
        verify_command_record(&record, &request).expect("exact");
        let mut wrong_owner = request;
        wrong_owner.logical_owner_id = "owner-2".to_owned();
        assert_eq!(
            verify_command_record(&record, &wrong_owner),
            Err(ReplayPersistenceErrorV1::WrongContract)
        );
    }

    fn request() -> AuthenticatedReplayOperationRequestV1 {
        AuthenticatedReplayOperationRequestV1 {
            operation_id: [1; 16],
            attachment_anchor_id: [2; 16],
            logical_owner_id: "owner-1".to_owned(),
            owner_device_actor_sha256: [9; 32],
        }
    }
}
