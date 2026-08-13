use makosh_organizations_core::{
    OrganizationLifecycleErrorV1, OrganizationRecordV1, OrganizationSourceStateV1,
    OrganizationSourceV1, OrganizationStateV1, OrganizationTimestampV1, add_organization_source_v1,
    create_organization_v1, remove_organization_source_v1, set_organization_state_v1,
    update_organization_v1, validate_organization_record_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    OrganizationLifecycleCommitV1, OrganizationLifecycleMutationV1,
    OrganizationLifecycleOperationOutcomeV1, OrganizationLifecycleOperationV1,
    OrganizationOutboxRecordV1, OrganizationsPersistenceErrorV1,
    model::{i64_value, valid_commit, valid_operation, valid_owner},
};

#[derive(Clone)]
pub struct OrganizationsPersistenceV1 {
    pool: PgPool,
}

pub struct OrganizationOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: OrganizationOutboxRecordV1,
    created_at_unix_millis: i64,
}

impl OrganizationOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &OrganizationOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), OrganizationsPersistenceErrorV1> {
        if expected_sha256 != self.record.envelope_sha256
            || Sha256::digest(&self.record.envelope_bytes).as_slice() != expected_sha256
            || published_at_unix_millis < self.created_at_unix_millis
        {
            return Err(OrganizationsPersistenceErrorV1::OutboxConflict);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.organizations_outbox SET published_at_unix_millis=$3 \
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
            return Err(OrganizationsPersistenceErrorV1::OutboxConflict);
        }
        self.transaction.commit().await.map_err(storage)
    }
}

impl OrganizationsPersistenceV1 {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, OrganizationsPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(OrganizationsPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| OrganizationsPersistenceErrorV1::StorageUnavailable)?,
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

    pub async fn verify_storage_ready(&self) -> Result<(), OrganizationsPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    async fn begin_owner(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, OrganizationsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(OrganizationsPersistenceErrorV1::InvalidInput);
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
    ) -> Result<Option<Vec<u8>>, OrganizationsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || operation_id.iter().all(|byte| *byte == 0)
            || request_sha256.iter().all(|byte| *byte == 0)
            || request_bytes.is_empty()
            || request_bytes.len() > crate::model::ORGANIZATIONS_MAX_CLIENT_MESSAGE_BYTES_V1
            || Sha256::digest(request_bytes).as_slice() != request_sha256
        {
            return Err(OrganizationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let replay = load_operation_replay_raw(
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

    pub async fn apply_lifecycle_operation<F>(
        &self,
        input: OrganizationLifecycleOperationV1,
        build_commit: F,
    ) -> Result<OrganizationLifecycleOperationOutcomeV1, OrganizationsPersistenceErrorV1>
    where
        F: FnOnce(
            &OrganizationRecordV1,
        )
            -> Result<OrganizationLifecycleCommitV1, OrganizationsPersistenceErrorV1>,
    {
        if !valid_operation(&input) {
            return Err(OrganizationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        if let Some(response_bytes) = load_operation_replay_raw(
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
            return Ok(OrganizationLifecycleOperationOutcomeV1::Replayed { response_bytes });
        }

        let creating = matches!(&input.mutation, OrganizationLifecycleMutationV1::Create(_));
        let mut organization = match &input.mutation {
            OrganizationLifecycleMutationV1::Create(draft) => {
                if draft.logical_owner_id != input.logical_owner_id
                    || draft.operation_id != input.operation_id
                {
                    return Err(OrganizationsPersistenceErrorV1::InvalidInput);
                }
                create_organization_v1(draft.clone()).map_err(core_error)?
            }
            mutation => load_organization(
                &mut transaction,
                &input.logical_owner_id,
                mutation
                    .organization_id()
                    .ok_or(OrganizationsPersistenceErrorV1::InvalidInput)?,
                true,
            )
            .await?
            .ok_or(OrganizationsPersistenceErrorV1::NotFound)?,
        };
        apply_mutation(&mut organization, &input.mutation)?;
        validate_organization_record_v1(&organization).map_err(core_error)?;
        persist_organization(&mut transaction, &organization, creating).await?;
        persist_sources(&mut transaction, &organization).await?;

        let commit = build_commit(&organization)?;
        if !valid_commit(&commit) {
            return Err(OrganizationsPersistenceErrorV1::InvalidInput);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.organizations_outbox (logical_owner_id,message_id, \
             envelope_sha256,envelope_bytes,created_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(commit.lifecycle_event.message_id.as_slice())
        .bind(commit.lifecycle_event.envelope_sha256.as_slice())
        .bind(&commit.lifecycle_event.envelope_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if inserted != 1 {
            return Err(OrganizationsPersistenceErrorV1::OutboxConflict);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.organizations_client_operations (logical_owner_id,operation_id, \
             operation_kind,request_sha256,request_bytes,organization_id,organization_revision, \
             response_sha256,response_bytes,received_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(input.mutation.operation_kind())
        .bind(input.request_sha256.as_slice())
        .bind(&input.request_bytes)
        .bind(organization.organization_id.as_slice())
        .bind(i64_value(organization.organization_revision)?)
        .bind(commit.response_sha256.as_slice())
        .bind(&commit.response_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if inserted != 1 {
            return Err(OrganizationsPersistenceErrorV1::OperationConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(OrganizationLifecycleOperationOutcomeV1::Applied {
            organization: Box::new(organization),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_organization(
        &self,
        logical_owner_id: &str,
        organization_id: [u8; 16],
    ) -> Result<Option<OrganizationRecordV1>, OrganizationsPersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let organization =
            load_organization(&mut transaction, logical_owner_id, organization_id, false).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(organization)
    }

    pub async fn list_organizations(
        &self,
        logical_owner_id: &str,
        after_organization_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<OrganizationRecordV1>, OrganizationsPersistenceErrorV1> {
        self.query_organizations(logical_owner_id, None, after_organization_id, limit)
            .await
    }

    pub async fn search_organizations(
        &self,
        logical_owner_id: &str,
        query: &str,
        after_organization_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<OrganizationRecordV1>, OrganizationsPersistenceErrorV1> {
        if query.trim().is_empty()
            || query.chars().count() > 200
            || query.chars().any(char::is_control)
        {
            return Err(OrganizationsPersistenceErrorV1::InvalidInput);
        }
        self.query_organizations(
            logical_owner_id,
            Some(query.trim()),
            after_organization_id,
            limit,
        )
        .await
    }

    async fn query_organizations(
        &self,
        logical_owner_id: &str,
        query: Option<&str>,
        after_organization_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<OrganizationRecordV1>, OrganizationsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || limit == 0 || limit > 201 {
            return Err(OrganizationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = if let Some(query) = query {
            sqlx::query(
                "SELECT organization_id FROM makosh_data.organizations_records \
                 WHERE logical_owner_id=$1 AND ($2::bytea IS NULL OR organization_id>$2) \
                 AND (display_name ILIKE '%' || $3 || '%' OR legal_name ILIKE '%' || $3 || '%' \
                 OR description ILIKE '%' || $3 || '%') \
                 ORDER BY organization_id LIMIT $4",
            )
            .bind(logical_owner_id)
            .bind(after_organization_id.map(|value| value.to_vec()))
            .bind(query)
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
        } else {
            sqlx::query(
                "SELECT organization_id FROM makosh_data.organizations_records \
                 WHERE logical_owner_id=$1 AND ($2::bytea IS NULL OR organization_id>$2) \
                 ORDER BY organization_id LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after_organization_id.map(|value| value.to_vec()))
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
        };
        let mut organizations = Vec::with_capacity(rows.len());
        for row in rows {
            let organization_id = fixed::<16>(row.try_get("organization_id").map_err(storage)?)?;
            organizations.push(
                load_organization(&mut transaction, logical_owner_id, organization_id, false)
                    .await?
                    .ok_or(OrganizationsPersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(organizations)
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<OrganizationOutboxPublishClaimV1>, OrganizationsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(OrganizationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        let row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,created_at_unix_millis \
             FROM makosh_data.organizations_outbox WHERE logical_owner_id=$1 \
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
        let record = OrganizationOutboxRecordV1 {
            message_id: fixed(row.try_get("message_id").map_err(storage)?)?,
            envelope_sha256: fixed(row.try_get("envelope_sha256").map_err(storage)?)?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
        };
        if Sha256::digest(&record.envelope_bytes).as_slice() != record.envelope_sha256 {
            return Err(OrganizationsPersistenceErrorV1::InvalidRow);
        }
        Ok(Some(OrganizationOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record,
            created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(storage)?,
        }))
    }
}

fn apply_mutation(
    organization: &mut OrganizationRecordV1,
    mutation: &OrganizationLifecycleMutationV1,
) -> Result<(), OrganizationsPersistenceErrorV1> {
    match mutation {
        OrganizationLifecycleMutationV1::Create(_) => Ok(()),
        OrganizationLifecycleMutationV1::Update {
            expected_revision,
            display_name,
            legal_name,
            description,
            website,
            industry,
            country_code,
            changed_at,
            ..
        } => update_organization_v1(
            organization,
            *expected_revision,
            display_name.clone(),
            legal_name.clone(),
            description.clone(),
            website.clone(),
            industry.clone(),
            country_code.clone(),
            *changed_at,
        )
        .map_err(core_error),
        OrganizationLifecycleMutationV1::SetState {
            expected_revision,
            state,
            changed_at,
            ..
        } => set_organization_state_v1(organization, *expected_revision, *state, *changed_at)
            .map_err(core_error),
        OrganizationLifecycleMutationV1::AddSource {
            expected_revision,
            source_owner_id,
            source_record_id,
            source_revision,
            evidence_digest,
            changed_at,
            ..
        } => add_organization_source_v1(
            organization,
            *expected_revision,
            source_owner_id.clone(),
            source_record_id.clone(),
            *source_revision,
            *evidence_digest,
            *changed_at,
        )
        .map(|_| ())
        .map_err(core_error),
        OrganizationLifecycleMutationV1::RemoveSource {
            expected_revision,
            source_id,
            changed_at,
            ..
        } => {
            remove_organization_source_v1(organization, *expected_revision, *source_id, *changed_at)
                .map_err(core_error)
        }
    }
}

async fn load_operation_replay_raw(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    request_bytes: &[u8],
    operation_kind: Option<i16>,
) -> Result<Option<Vec<u8>>, OrganizationsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind,request_sha256,request_bytes,response_sha256,response_bytes \
         FROM makosh_data.organizations_client_operations \
         WHERE logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let stored_kind: i16 = row.try_get("operation_kind").map_err(storage)?;
    let stored_request_sha: [u8; 32] = fixed(row.try_get("request_sha256").map_err(storage)?)?;
    let stored_request: Vec<u8> = row.try_get("request_bytes").map_err(storage)?;
    let stored_response_sha: [u8; 32] = fixed(row.try_get("response_sha256").map_err(storage)?)?;
    let stored_response: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
    if operation_kind.is_some_and(|kind| kind != stored_kind)
        || stored_request_sha != request_sha256
        || stored_request != request_bytes
        || Sha256::digest(&stored_request).as_slice() != stored_request_sha
        || Sha256::digest(&stored_response).as_slice() != stored_response_sha
    {
        return Err(OrganizationsPersistenceErrorV1::OperationConflict);
    }
    Ok(Some(stored_response))
}

async fn load_organization(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    organization_id: [u8; 16],
    for_update: bool,
) -> Result<Option<OrganizationRecordV1>, OrganizationsPersistenceErrorV1> {
    let statement = if for_update {
        "SELECT display_name,legal_name,description,website,industry,country_code, \
         organization_state,organization_revision,created_at_unix_seconds,created_at_nanos, \
         updated_at_unix_seconds,updated_at_nanos FROM makosh_data.organizations_records \
         WHERE logical_owner_id=$1 AND organization_id=$2 FOR UPDATE"
    } else {
        "SELECT display_name,legal_name,description,website,industry,country_code, \
         organization_state,organization_revision,created_at_unix_seconds,created_at_nanos, \
         updated_at_unix_seconds,updated_at_nanos FROM makosh_data.organizations_records \
         WHERE logical_owner_id=$1 AND organization_id=$2"
    };
    let row = sqlx::query(statement)
        .bind(logical_owner_id)
        .bind(organization_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let source_rows = sqlx::query(
        "SELECT source_id,source_owner_id,source_record_id,source_revision,evidence_digest, \
         source_state,updated_at_organization_revision FROM makosh_data.organizations_sources \
         WHERE logical_owner_id=$1 AND organization_id=$2 ORDER BY source_id",
    )
    .bind(logical_owner_id)
    .bind(organization_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let mut sources = Vec::with_capacity(source_rows.len());
    for source in source_rows {
        sources.push(OrganizationSourceV1 {
            source_id: fixed(source.try_get("source_id").map_err(storage)?)?,
            source_owner_id: source.try_get("source_owner_id").map_err(storage)?,
            source_record_id: source.try_get("source_record_id").map_err(storage)?,
            source_revision: u64_value(source.try_get("source_revision").map_err(storage)?)?,
            evidence_digest: fixed(source.try_get("evidence_digest").map_err(storage)?)?,
            state: decode_source_state(source.try_get("source_state").map_err(storage)?)?,
            updated_at_organization_revision: u64_value(
                source
                    .try_get("updated_at_organization_revision")
                    .map_err(storage)?,
            )?,
        });
    }
    let organization = OrganizationRecordV1 {
        organization_id,
        logical_owner_id: logical_owner_id.to_owned(),
        display_name: row.try_get("display_name").map_err(storage)?,
        legal_name: row.try_get("legal_name").map_err(storage)?,
        description: row.try_get("description").map_err(storage)?,
        website: row.try_get("website").map_err(storage)?,
        industry: row.try_get("industry").map_err(storage)?,
        country_code: row.try_get("country_code").map_err(storage)?,
        state: decode_state(row.try_get("organization_state").map_err(storage)?)?,
        organization_revision: u64_value(row.try_get("organization_revision").map_err(storage)?)?,
        sources,
        created_at: OrganizationTimestampV1 {
            unix_seconds: row.try_get("created_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("created_at_nanos").map_err(storage)?,
        },
        updated_at: OrganizationTimestampV1 {
            unix_seconds: row.try_get("updated_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("updated_at_nanos").map_err(storage)?,
        },
    };
    validate_organization_record_v1(&organization).map_err(core_error)?;
    Ok(Some(organization))
}

async fn persist_organization(
    transaction: &mut Transaction<'_, Postgres>,
    organization: &OrganizationRecordV1,
    creating: bool,
) -> Result<(), OrganizationsPersistenceErrorV1> {
    let affected = if creating {
        sqlx::query(
            "INSERT INTO makosh_data.organizations_records (logical_owner_id,organization_id,display_name, \
             legal_name,description,website,industry,country_code,organization_state, \
             organization_revision,created_at_unix_seconds,created_at_nanos, \
             updated_at_unix_seconds,updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
        )
        .bind(&organization.logical_owner_id)
        .bind(organization.organization_id.as_slice())
        .bind(&organization.display_name)
        .bind(&organization.legal_name)
        .bind(&organization.description)
        .bind(&organization.website)
        .bind(&organization.industry)
        .bind(&organization.country_code)
        .bind(encode_state(organization.state))
        .bind(i64_value(organization.organization_revision)?)
        .bind(organization.created_at.unix_seconds)
        .bind(organization.created_at.nanos)
        .bind(organization.updated_at.unix_seconds)
        .bind(organization.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?
        .rows_affected()
    } else {
        let previous_revision = organization
            .organization_revision
            .checked_sub(1)
            .ok_or(OrganizationsPersistenceErrorV1::RevisionConflict)?;
        sqlx::query(
            "UPDATE makosh_data.organizations_records SET display_name=$3,legal_name=$4,description=$5, \
             website=$6,industry=$7,country_code=$8,organization_state=$9, \
             organization_revision=$10,updated_at_unix_seconds=$11,updated_at_nanos=$12 \
             WHERE logical_owner_id=$1 AND organization_id=$2 AND organization_revision=$13",
        )
        .bind(&organization.logical_owner_id)
        .bind(organization.organization_id.as_slice())
        .bind(&organization.display_name)
        .bind(&organization.legal_name)
        .bind(&organization.description)
        .bind(&organization.website)
        .bind(&organization.industry)
        .bind(&organization.country_code)
        .bind(encode_state(organization.state))
        .bind(i64_value(organization.organization_revision)?)
        .bind(organization.updated_at.unix_seconds)
        .bind(organization.updated_at.nanos)
        .bind(i64_value(previous_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?
        .rows_affected()
    };
    if affected != 1 {
        return Err(if creating {
            OrganizationsPersistenceErrorV1::OperationConflict
        } else {
            OrganizationsPersistenceErrorV1::RevisionConflict
        });
    }
    Ok(())
}

async fn persist_sources(
    transaction: &mut Transaction<'_, Postgres>,
    organization: &OrganizationRecordV1,
) -> Result<(), OrganizationsPersistenceErrorV1> {
    sqlx::query(
        "DELETE FROM makosh_data.organizations_sources WHERE logical_owner_id=$1 AND organization_id=$2",
    )
    .bind(&organization.logical_owner_id)
    .bind(organization.organization_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    for source in &organization.sources {
        sqlx::query(
            "INSERT INTO makosh_data.organizations_sources (logical_owner_id,organization_id, \
             source_id,source_owner_id,source_record_id,source_revision,evidence_digest, \
             source_state,updated_at_organization_revision) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(&organization.logical_owner_id)
        .bind(organization.organization_id.as_slice())
        .bind(source.source_id.as_slice())
        .bind(&source.source_owner_id)
        .bind(&source.source_record_id)
        .bind(i64_value(source.source_revision)?)
        .bind(source.evidence_digest.as_slice())
        .bind(encode_source_state(source.state))
        .bind(i64_value(source.updated_at_organization_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

fn encode_state(value: OrganizationStateV1) -> i16 {
    match value {
        OrganizationStateV1::Active => 1,
        OrganizationStateV1::Archived => 2,
    }
}

fn decode_state(value: i16) -> Result<OrganizationStateV1, OrganizationsPersistenceErrorV1> {
    match value {
        1 => Ok(OrganizationStateV1::Active),
        2 => Ok(OrganizationStateV1::Archived),
        _ => Err(OrganizationsPersistenceErrorV1::InvalidRow),
    }
}

fn encode_source_state(value: OrganizationSourceStateV1) -> i16 {
    match value {
        OrganizationSourceStateV1::Active => 1,
        OrganizationSourceStateV1::Removed => 2,
    }
}

fn decode_source_state(
    value: i16,
) -> Result<OrganizationSourceStateV1, OrganizationsPersistenceErrorV1> {
    match value {
        1 => Ok(OrganizationSourceStateV1::Active),
        2 => Ok(OrganizationSourceStateV1::Removed),
        _ => Err(OrganizationsPersistenceErrorV1::InvalidRow),
    }
}

fn u64_value(value: i64) -> Result<u64, OrganizationsPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(OrganizationsPersistenceErrorV1::InvalidRow)
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], OrganizationsPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| OrganizationsPersistenceErrorV1::InvalidRow)
}

fn core_error(value: OrganizationLifecycleErrorV1) -> OrganizationsPersistenceErrorV1 {
    match value {
        OrganizationLifecycleErrorV1::InvalidRevision
        | OrganizationLifecycleErrorV1::RevisionOverflow => {
            OrganizationsPersistenceErrorV1::RevisionConflict
        }
        OrganizationLifecycleErrorV1::InvalidOrganizationId
        | OrganizationLifecycleErrorV1::SourceNotFound => OrganizationsPersistenceErrorV1::NotFound,
        _ => OrganizationsPersistenceErrorV1::InvalidInput,
    }
}

fn storage(_: sqlx::Error) -> OrganizationsPersistenceErrorV1 {
    OrganizationsPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_errors_are_bounded_and_claim_keeps_exact_hash() {
        assert_eq!(
            core_error(OrganizationLifecycleErrorV1::InvalidRevision),
            OrganizationsPersistenceErrorV1::RevisionConflict
        );
        assert_eq!(
            core_error(OrganizationLifecycleErrorV1::SourceNotFound),
            OrganizationsPersistenceErrorV1::NotFound
        );
        let bytes = b"event".to_vec();
        let record = OrganizationOutboxRecordV1 {
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
