use makosh_relationships_core::{
    RelationshipCoreErrorV1, RelationshipEvidenceStateV1, RelationshipEvidenceV1,
    RelationshipParticipantKindV1, RelationshipParticipantV1, RelationshipRecordV1,
    RelationshipStateV1, RelationshipTimestampV1, RelationshipTypeV1, add_evidence_v1,
    create_relationship_with_evidence_v1, end_relationship_v1, reactivate_relationship_v1,
    remove_evidence_v1, update_validity_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    RelationshipCommitV1, RelationshipMutationV1, RelationshipOperationOutcomeV1,
    RelationshipOperationV1, RelationshipOutboxRecordV1, RelationshipsPersistenceErrorV1,
    model::{i64_value, nonzero, valid_commit, valid_operation, valid_owner},
};

#[derive(Clone)]
pub struct RelationshipsPersistenceV1 {
    pool: PgPool,
}

pub struct RelationshipOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: RelationshipOutboxRecordV1,
    created_at_unix_millis: i64,
}

impl RelationshipOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &RelationshipOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), RelationshipsPersistenceErrorV1> {
        if expected_sha256 != self.record.envelope_sha256
            || Sha256::digest(&self.record.envelope_bytes).as_slice() != expected_sha256
            || published_at_unix_millis < self.created_at_unix_millis
        {
            return Err(RelationshipsPersistenceErrorV1::OutboxConflict);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.relationships_outbox SET published_at_unix_millis=$3 \
             WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 \
             AND published_at_unix_millis IS NULL",
        )
        .bind(&self.logical_owner_id)
        .bind(self.record.message_id.as_slice())
        .bind(published_at_unix_millis)
        .bind(expected_sha256.as_slice())
        .execute(&mut *self.transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if affected != 1 {
            return Err(RelationshipsPersistenceErrorV1::OutboxConflict);
        }
        self.transaction.commit().await.map_err(storage)
    }
}

impl RelationshipsPersistenceV1 {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        host: &str,
        port: u32,
        password: &str,
    ) -> Result<Self, RelationshipsPersistenceErrorV1> {
        if host.is_empty()
            || port == 0
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(RelationshipsPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(
                u16::try_from(port)
                    .map_err(|_| RelationshipsPersistenceErrorV1::StorageUnavailable)?,
            )
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(storage)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), RelationshipsPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    async fn begin_owner(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, RelationshipsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(RelationshipsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        Ok(transaction)
    }

    pub async fn load_operation_replay(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
        request_sha256: [u8; 32],
        request_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, RelationshipsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !nonzero(&operation_id)
            || !nonzero(&request_sha256)
            || request_bytes.is_empty()
            || request_bytes.len() > crate::model::MAX_CLIENT_MESSAGE_BYTES_V1
            || Sha256::digest(request_bytes).as_slice() != request_sha256
        {
            return Err(RelationshipsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let replay = load_replay(
            &mut transaction,
            logical_owner_id,
            operation_id,
            request_sha256,
            request_bytes,
            None,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(replay)
    }

    pub async fn apply_operation<F>(
        &self,
        input: RelationshipOperationV1,
        build_commit: F,
    ) -> Result<RelationshipOperationOutcomeV1, RelationshipsPersistenceErrorV1>
    where
        F: FnOnce(
            &RelationshipRecordV1,
        ) -> Result<RelationshipCommitV1, RelationshipsPersistenceErrorV1>,
    {
        if !valid_operation(&input) {
            return Err(RelationshipsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        if let Some(response_bytes) = load_replay(
            &mut transaction,
            &input.logical_owner_id,
            input.operation_id,
            input.request_sha256,
            &input.request_bytes,
            Some(input.mutation.operation_kind()),
        )
        .await?
        {
            transaction.commit().await.map_err(storage)?;
            return Ok(RelationshipOperationOutcomeV1::Replayed { response_bytes });
        }

        let creating = matches!(&input.mutation, RelationshipMutationV1::Create { .. });
        let (mut relationship, mut evidence) = if let RelationshipMutationV1::Create {
            operation_id,
            source,
            target,
            relationship_type,
            valid_from,
            valid_until,
            evidence_source_owner_id,
            evidence_source_record_id,
            evidence_source_revision,
            evidence_digest,
            evidence_observed_at,
            created_at,
        } = &input.mutation
        {
            let (relationship, first) = create_relationship_with_evidence_v1(
                input.logical_owner_id.clone(),
                *operation_id,
                *source,
                *target,
                *relationship_type,
                *valid_from,
                *valid_until,
                evidence_source_owner_id.clone(),
                evidence_source_record_id.clone(),
                *evidence_source_revision,
                *evidence_digest,
                *evidence_observed_at,
                *created_at,
            )
            .map_err(core_error)?;
            (relationship, vec![first])
        } else {
            let relationship_id = input
                .mutation
                .relationship_id()
                .ok_or(RelationshipsPersistenceErrorV1::InvalidInput)?;
            let relationship = load_relationship(
                &mut transaction,
                &input.logical_owner_id,
                relationship_id,
                true,
            )
            .await?
            .ok_or(RelationshipsPersistenceErrorV1::NotFound)?;
            let evidence =
                load_evidence(&mut transaction, &input.logical_owner_id, relationship_id).await?;
            (relationship, evidence)
        };
        if !creating {
            apply_mutation(&mut relationship, &mut evidence, &input.mutation)?;
        }
        persist_relationship(&mut transaction, &relationship, creating).await?;
        persist_evidence(&mut transaction, &relationship, &evidence).await?;

        let commit = build_commit(&relationship)?;
        if !valid_commit(&commit) {
            return Err(RelationshipsPersistenceErrorV1::InvalidInput);
        }
        let outbox = sqlx::query(
            "INSERT INTO makosh_data.relationships_outbox (logical_owner_id,message_id,envelope_sha256, \
             envelope_bytes,created_at_unix_millis) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(commit.lifecycle_event.message_id.as_slice())
        .bind(commit.lifecycle_event.envelope_sha256.as_slice())
        .bind(&commit.lifecycle_event.envelope_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction).await.map_err(storage)?.rows_affected();
        if outbox != 1 {
            return Err(RelationshipsPersistenceErrorV1::OutboxConflict);
        }
        let operation = sqlx::query(
            "INSERT INTO makosh_data.relationships_client_operations (logical_owner_id,operation_id, \
             operation_kind,request_sha256,request_bytes,relationship_id,relationship_revision, \
             response_sha256,response_bytes,received_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(input.mutation.operation_kind())
        .bind(input.request_sha256.as_slice())
        .bind(&input.request_bytes)
        .bind(relationship.relationship_id.as_slice())
        .bind(i64_value(relationship.relationship_revision)?)
        .bind(commit.response_sha256.as_slice())
        .bind(&commit.response_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction).await.map_err(storage)?.rows_affected();
        if operation != 1 {
            return Err(RelationshipsPersistenceErrorV1::OperationConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(RelationshipOperationOutcomeV1::Applied {
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_relationship(
        &self,
        logical_owner_id: &str,
        relationship_id: [u8; 16],
    ) -> Result<Option<RelationshipRecordV1>, RelationshipsPersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let value =
            load_relationship(&mut transaction, logical_owner_id, relationship_id, false).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(value)
    }

    pub async fn list_for_participant(
        &self,
        logical_owner_id: &str,
        participant: RelationshipParticipantV1,
        after_relationship_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<RelationshipRecordV1>, RelationshipsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !nonzero(&participant.public_id)
            || limit == 0
            || limit > 201
        {
            return Err(RelationshipsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT relationship_id FROM makosh_data.relationships_records WHERE logical_owner_id=$1 \
             AND ($2::bytea IS NULL OR relationship_id>$2) AND \
             ((source_kind=$3 AND source_public_id=$4) OR (target_kind=$3 AND target_public_id=$4)) \
             ORDER BY relationship_id LIMIT $5",
        )
        .bind(logical_owner_id)
        .bind(after_relationship_id.map(|value| value.to_vec()))
        .bind(encode_participant_kind(participant.kind))
        .bind(participant.public_id.as_slice())
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction).await.map_err(storage)?;
        let mut values = Vec::with_capacity(rows.len());
        for row in rows {
            let id = fixed(row.try_get("relationship_id").map_err(storage)?)?;
            values.push(
                load_relationship(&mut transaction, logical_owner_id, id, false)
                    .await?
                    .ok_or(RelationshipsPersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(values)
    }

    pub async fn list_evidence(
        &self,
        logical_owner_id: &str,
        relationship_id: [u8; 16],
        after_evidence_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<RelationshipEvidenceV1>, RelationshipsPersistenceErrorV1> {
        if limit == 0 || limit > 201 {
            return Err(RelationshipsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let values = load_evidence_page(
            &mut transaction,
            logical_owner_id,
            relationship_id,
            after_evidence_id,
            limit,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(values)
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<RelationshipOutboxPublishClaimV1>, RelationshipsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(RelationshipsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        let row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,created_at_unix_millis \
             FROM makosh_data.relationships_outbox WHERE logical_owner_id=$1 \
             AND published_at_unix_millis IS NULL ORDER BY outbox_sequence \
             LIMIT 1 FOR UPDATE SKIP LOCKED",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let Some(row) = row else {
            transaction.commit().await.map_err(storage)?;
            return Ok(None);
        };
        let record = RelationshipOutboxRecordV1 {
            message_id: fixed(row.try_get("message_id").map_err(storage)?)?,
            envelope_sha256: fixed(row.try_get("envelope_sha256").map_err(storage)?)?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
        };
        if Sha256::digest(&record.envelope_bytes).as_slice() != record.envelope_sha256 {
            return Err(RelationshipsPersistenceErrorV1::InvalidRow);
        }
        Ok(Some(RelationshipOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record,
            created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(storage)?,
        }))
    }
}

fn apply_mutation(
    relationship: &mut RelationshipRecordV1,
    evidence: &mut Vec<RelationshipEvidenceV1>,
    mutation: &RelationshipMutationV1,
) -> Result<(), RelationshipsPersistenceErrorV1> {
    match mutation {
        RelationshipMutationV1::Create { .. } => Ok(()),
        RelationshipMutationV1::UpdateValidity {
            expected_revision,
            valid_from,
            valid_until,
            changed_at,
            ..
        } => update_validity_v1(
            relationship,
            *expected_revision,
            *valid_from,
            *valid_until,
            *changed_at,
        )
        .map_err(core_error),
        RelationshipMutationV1::End {
            expected_revision,
            valid_until,
            changed_at,
            ..
        } => end_relationship_v1(relationship, *expected_revision, *valid_until, *changed_at)
            .map_err(core_error),
        RelationshipMutationV1::Reactivate {
            expected_revision,
            valid_from,
            valid_until,
            changed_at,
            ..
        } => reactivate_relationship_v1(
            relationship,
            *expected_revision,
            *valid_from,
            *valid_until,
            *changed_at,
        )
        .map_err(core_error),
        RelationshipMutationV1::AddEvidence {
            expected_revision,
            source_owner_id,
            source_record_id,
            source_revision,
            evidence_digest,
            observed_at,
            changed_at,
            ..
        } => {
            let added = add_evidence_v1(
                relationship,
                evidence,
                *expected_revision,
                source_owner_id.clone(),
                source_record_id.clone(),
                *source_revision,
                *evidence_digest,
                *observed_at,
                *changed_at,
            )
            .map_err(core_error)?;
            evidence.push(added);
            Ok(())
        }
        RelationshipMutationV1::RemoveEvidence {
            expected_revision,
            evidence_id,
            changed_at,
            ..
        } => {
            let value = evidence
                .iter_mut()
                .find(|value| value.evidence_id == *evidence_id)
                .ok_or(RelationshipsPersistenceErrorV1::NotFound)?;
            remove_evidence_v1(relationship, value, *expected_revision, *changed_at)
                .map_err(core_error)
        }
    }
}

async fn load_replay(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    request_bytes: &[u8],
    operation_kind: Option<i16>,
) -> Result<Option<Vec<u8>>, RelationshipsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind,request_sha256,request_bytes,response_sha256,response_bytes \
         FROM makosh_data.relationships_client_operations WHERE logical_owner_id=$1 \
         AND operation_id=$2 FOR UPDATE",
    )
    .bind(owner)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let kind: i16 = row.try_get("operation_kind").map_err(storage)?;
    let stored_request_sha: [u8; 32] = fixed(row.try_get("request_sha256").map_err(storage)?)?;
    let stored_request: Vec<u8> = row.try_get("request_bytes").map_err(storage)?;
    let response_sha: [u8; 32] = fixed(row.try_get("response_sha256").map_err(storage)?)?;
    let response: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
    if operation_kind.is_some_and(|expected| expected != kind)
        || stored_request_sha != request_sha256
        || stored_request != request_bytes
        || Sha256::digest(&stored_request).as_slice() != stored_request_sha
        || Sha256::digest(&response).as_slice() != response_sha
    {
        return Err(RelationshipsPersistenceErrorV1::OperationConflict);
    }
    Ok(Some(response))
}

async fn load_relationship(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    id: [u8; 16],
    for_update: bool,
) -> Result<Option<RelationshipRecordV1>, RelationshipsPersistenceErrorV1> {
    let statement = if for_update {
        "SELECT source_kind,source_public_id,target_kind,target_public_id,relationship_type, \
         relationship_state,valid_from_unix_seconds,valid_from_nanos,valid_until_unix_seconds, \
         valid_until_nanos,relationship_revision,created_at_unix_seconds,created_at_nanos, \
         updated_at_unix_seconds,updated_at_nanos FROM makosh_data.relationships_records \
         WHERE logical_owner_id=$1 AND relationship_id=$2 FOR UPDATE"
    } else {
        "SELECT source_kind,source_public_id,target_kind,target_public_id,relationship_type, \
         relationship_state,valid_from_unix_seconds,valid_from_nanos,valid_until_unix_seconds, \
         valid_until_nanos,relationship_revision,created_at_unix_seconds,created_at_nanos, \
         updated_at_unix_seconds,updated_at_nanos FROM makosh_data.relationships_records \
         WHERE logical_owner_id=$1 AND relationship_id=$2"
    };
    let row = sqlx::query(statement)
        .bind(owner)
        .bind(id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let until_seconds: Option<i64> = row.try_get("valid_until_unix_seconds").map_err(storage)?;
    let until_nanos: Option<i32> = row.try_get("valid_until_nanos").map_err(storage)?;
    let valid_until = match (until_seconds, until_nanos) {
        (None, None) => None,
        (Some(unix_seconds), Some(nanos)) => Some(RelationshipTimestampV1 {
            unix_seconds,
            nanos,
        }),
        _ => return Err(RelationshipsPersistenceErrorV1::InvalidRow),
    };
    Ok(Some(RelationshipRecordV1 {
        relationship_id: id,
        logical_owner_id: owner.to_owned(),
        source: RelationshipParticipantV1 {
            kind: decode_participant_kind(row.try_get("source_kind").map_err(storage)?)?,
            public_id: fixed(row.try_get("source_public_id").map_err(storage)?)?,
        },
        target: RelationshipParticipantV1 {
            kind: decode_participant_kind(row.try_get("target_kind").map_err(storage)?)?,
            public_id: fixed(row.try_get("target_public_id").map_err(storage)?)?,
        },
        relationship_type: decode_relationship_type(
            row.try_get("relationship_type").map_err(storage)?,
        )?,
        state: decode_state(row.try_get("relationship_state").map_err(storage)?)?,
        valid_from: RelationshipTimestampV1 {
            unix_seconds: row.try_get("valid_from_unix_seconds").map_err(storage)?,
            nanos: row.try_get("valid_from_nanos").map_err(storage)?,
        },
        valid_until,
        relationship_revision: u64_value(row.try_get("relationship_revision").map_err(storage)?)?,
        created_at: RelationshipTimestampV1 {
            unix_seconds: row.try_get("created_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("created_at_nanos").map_err(storage)?,
        },
        updated_at: RelationshipTimestampV1 {
            unix_seconds: row.try_get("updated_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("updated_at_nanos").map_err(storage)?,
        },
    }))
}

async fn load_evidence(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    relationship_id: [u8; 16],
) -> Result<Vec<RelationshipEvidenceV1>, RelationshipsPersistenceErrorV1> {
    load_evidence_page(transaction, owner, relationship_id, None, u16::MAX).await
}

async fn load_evidence_page(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    relationship_id: [u8; 16],
    after: Option<[u8; 16]>,
    limit: u16,
) -> Result<Vec<RelationshipEvidenceV1>, RelationshipsPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT evidence_id,source_owner_id,source_record_id,source_revision,evidence_digest, \
         observed_at_unix_seconds,observed_at_nanos,evidence_state,updated_at_relationship_revision \
         FROM makosh_data.relationships_evidence WHERE logical_owner_id=$1 AND relationship_id=$2 \
         AND ($3::bytea IS NULL OR evidence_id>$3) ORDER BY evidence_id LIMIT $4",
    )
    .bind(owner).bind(relationship_id.as_slice()).bind(after.map(|value| value.to_vec()))
    .bind(i64::from(limit)).fetch_all(&mut **transaction).await.map_err(storage)?;
    rows.into_iter()
        .map(|row| {
            Ok(RelationshipEvidenceV1 {
                evidence_id: fixed(row.try_get("evidence_id").map_err(storage)?)?,
                source_owner_id: row.try_get("source_owner_id").map_err(storage)?,
                source_record_id: row.try_get("source_record_id").map_err(storage)?,
                source_revision: u64_value(row.try_get("source_revision").map_err(storage)?)?,
                evidence_digest: fixed(row.try_get("evidence_digest").map_err(storage)?)?,
                observed_at: RelationshipTimestampV1 {
                    unix_seconds: row.try_get("observed_at_unix_seconds").map_err(storage)?,
                    nanos: row.try_get("observed_at_nanos").map_err(storage)?,
                },
                state: decode_evidence_state(row.try_get("evidence_state").map_err(storage)?)?,
                updated_at_relationship_revision: u64_value(
                    row.try_get("updated_at_relationship_revision")
                        .map_err(storage)?,
                )?,
            })
        })
        .collect()
}

async fn persist_relationship(
    transaction: &mut Transaction<'_, Postgres>,
    value: &RelationshipRecordV1,
    creating: bool,
) -> Result<(), RelationshipsPersistenceErrorV1> {
    let (until_seconds, until_nanos) = value.valid_until.map_or((None, None), |until| {
        (Some(until.unix_seconds), Some(until.nanos))
    });
    let affected = if creating {
        sqlx::query(
            "INSERT INTO makosh_data.relationships_records (logical_owner_id,relationship_id,source_kind, \
             source_public_id,target_kind,target_public_id,relationship_type,relationship_state, \
             valid_from_unix_seconds,valid_from_nanos,valid_until_unix_seconds,valid_until_nanos, \
             relationship_revision,created_at_unix_seconds,created_at_nanos,updated_at_unix_seconds,updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)",
        )
        .bind(&value.logical_owner_id).bind(value.relationship_id.as_slice())
        .bind(encode_participant_kind(value.source.kind)).bind(value.source.public_id.as_slice())
        .bind(encode_participant_kind(value.target.kind)).bind(value.target.public_id.as_slice())
        .bind(encode_relationship_type(value.relationship_type)).bind(encode_state(value.state))
        .bind(value.valid_from.unix_seconds).bind(value.valid_from.nanos)
        .bind(until_seconds).bind(until_nanos).bind(i64_value(value.relationship_revision)?)
        .bind(value.created_at.unix_seconds).bind(value.created_at.nanos)
        .bind(value.updated_at.unix_seconds).bind(value.updated_at.nanos)
        .execute(&mut **transaction).await.map_err(storage)?.rows_affected()
    } else {
        let previous = value
            .relationship_revision
            .checked_sub(1)
            .ok_or(RelationshipsPersistenceErrorV1::RevisionConflict)?;
        sqlx::query(
            "UPDATE makosh_data.relationships_records SET relationship_state=$3,valid_from_unix_seconds=$4, \
             valid_from_nanos=$5,valid_until_unix_seconds=$6,valid_until_nanos=$7,relationship_revision=$8, \
             updated_at_unix_seconds=$9,updated_at_nanos=$10 WHERE logical_owner_id=$1 \
             AND relationship_id=$2 AND relationship_revision=$11",
        )
        .bind(&value.logical_owner_id).bind(value.relationship_id.as_slice()).bind(encode_state(value.state))
        .bind(value.valid_from.unix_seconds).bind(value.valid_from.nanos).bind(until_seconds).bind(until_nanos)
        .bind(i64_value(value.relationship_revision)?).bind(value.updated_at.unix_seconds).bind(value.updated_at.nanos)
        .bind(i64_value(previous)?).execute(&mut **transaction).await.map_err(storage)?.rows_affected()
    };
    if affected != 1 {
        return Err(if creating {
            RelationshipsPersistenceErrorV1::OperationConflict
        } else {
            RelationshipsPersistenceErrorV1::RevisionConflict
        });
    }
    Ok(())
}

async fn persist_evidence(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &RelationshipRecordV1,
    evidence: &[RelationshipEvidenceV1],
) -> Result<(), RelationshipsPersistenceErrorV1> {
    sqlx::query("DELETE FROM makosh_data.relationships_evidence WHERE logical_owner_id=$1 AND relationship_id=$2")
        .bind(&relationship.logical_owner_id).bind(relationship.relationship_id.as_slice())
        .execute(&mut **transaction).await.map_err(storage)?;
    for value in evidence {
        sqlx::query(
            "INSERT INTO makosh_data.relationships_evidence (logical_owner_id,relationship_id,evidence_id, \
             source_owner_id,source_record_id,source_revision,evidence_digest,observed_at_unix_seconds, \
             observed_at_nanos,evidence_state,updated_at_relationship_revision) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
        )
        .bind(&relationship.logical_owner_id).bind(relationship.relationship_id.as_slice())
        .bind(value.evidence_id.as_slice()).bind(&value.source_owner_id).bind(&value.source_record_id)
        .bind(i64_value(value.source_revision)?).bind(value.evidence_digest.as_slice())
        .bind(value.observed_at.unix_seconds).bind(value.observed_at.nanos)
        .bind(encode_evidence_state(value.state)).bind(i64_value(value.updated_at_relationship_revision)?)
        .execute(&mut **transaction).await.map_err(storage)?;
    }
    Ok(())
}

fn encode_participant_kind(value: RelationshipParticipantKindV1) -> i16 {
    match value {
        RelationshipParticipantKindV1::Person => 1,
        RelationshipParticipantKindV1::Organization => 2,
    }
}
fn decode_participant_kind(
    value: i16,
) -> Result<RelationshipParticipantKindV1, RelationshipsPersistenceErrorV1> {
    match value {
        1 => Ok(RelationshipParticipantKindV1::Person),
        2 => Ok(RelationshipParticipantKindV1::Organization),
        _ => Err(RelationshipsPersistenceErrorV1::InvalidRow),
    }
}
fn encode_relationship_type(value: RelationshipTypeV1) -> i16 {
    match value {
        RelationshipTypeV1::Family => 1,
        RelationshipTypeV1::Friend => 2,
        RelationshipTypeV1::Colleague => 3,
        RelationshipTypeV1::ReportsTo => 4,
        RelationshipTypeV1::MemberOf => 5,
        RelationshipTypeV1::Partner => 6,
    }
}
fn decode_relationship_type(
    value: i16,
) -> Result<RelationshipTypeV1, RelationshipsPersistenceErrorV1> {
    match value {
        1 => Ok(RelationshipTypeV1::Family),
        2 => Ok(RelationshipTypeV1::Friend),
        3 => Ok(RelationshipTypeV1::Colleague),
        4 => Ok(RelationshipTypeV1::ReportsTo),
        5 => Ok(RelationshipTypeV1::MemberOf),
        6 => Ok(RelationshipTypeV1::Partner),
        _ => Err(RelationshipsPersistenceErrorV1::InvalidRow),
    }
}
fn encode_state(value: RelationshipStateV1) -> i16 {
    match value {
        RelationshipStateV1::Confirmed => 1,
        RelationshipStateV1::Ended => 2,
    }
}
fn decode_state(value: i16) -> Result<RelationshipStateV1, RelationshipsPersistenceErrorV1> {
    match value {
        1 => Ok(RelationshipStateV1::Confirmed),
        2 => Ok(RelationshipStateV1::Ended),
        _ => Err(RelationshipsPersistenceErrorV1::InvalidRow),
    }
}
fn encode_evidence_state(value: RelationshipEvidenceStateV1) -> i16 {
    match value {
        RelationshipEvidenceStateV1::Active => 1,
        RelationshipEvidenceStateV1::Removed => 2,
    }
}
fn decode_evidence_state(
    value: i16,
) -> Result<RelationshipEvidenceStateV1, RelationshipsPersistenceErrorV1> {
    match value {
        1 => Ok(RelationshipEvidenceStateV1::Active),
        2 => Ok(RelationshipEvidenceStateV1::Removed),
        _ => Err(RelationshipsPersistenceErrorV1::InvalidRow),
    }
}
fn u64_value(value: i64) -> Result<u64, RelationshipsPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(RelationshipsPersistenceErrorV1::InvalidRow)
}
fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], RelationshipsPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| RelationshipsPersistenceErrorV1::InvalidRow)
}
fn core_error(value: RelationshipCoreErrorV1) -> RelationshipsPersistenceErrorV1 {
    match value {
        RelationshipCoreErrorV1::RevisionConflict | RelationshipCoreErrorV1::RevisionOverflow => {
            RelationshipsPersistenceErrorV1::RevisionConflict
        }
        RelationshipCoreErrorV1::StateConflict => RelationshipsPersistenceErrorV1::StateConflict,
        RelationshipCoreErrorV1::EvidenceConflict => {
            RelationshipsPersistenceErrorV1::EvidenceConflict
        }
        RelationshipCoreErrorV1::InvalidInput => RelationshipsPersistenceErrorV1::InvalidInput,
    }
}
fn storage(_: sqlx::Error) -> RelationshipsPersistenceErrorV1 {
    RelationshipsPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_errors_and_exact_outbox_hash() {
        assert_eq!(
            core_error(RelationshipCoreErrorV1::RevisionOverflow),
            RelationshipsPersistenceErrorV1::RevisionConflict
        );
        let bytes = b"event".to_vec();
        let record = RelationshipOutboxRecordV1 {
            message_id: [1; 16],
            envelope_sha256: Sha256::digest(&bytes).into(),
            envelope_bytes: bytes,
        };
        assert_eq!(
            Sha256::digest(&record.envelope_bytes).as_slice(),
            record.envelope_sha256
        );
    }
}
