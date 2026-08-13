use makosh_identity_resolution_core::{
    IdentityMatchEvidenceV1, IdentityResolutionCoreErrorV1, IdentityResolutionMatchKindV1,
    validate_identity_match_evidence_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityResolutionEnvelopeRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

impl IdentityResolutionEnvelopeRecordV1 {
    pub fn validate(&self) -> Result<(), IdentityResolutionPersistenceErrorV1> {
        if zero(&self.message_id)
            || zero(&self.envelope_sha256)
            || self.envelope_bytes.is_empty()
            || self.envelope_bytes.len() > 65_536
            || <[u8; 32]>::from(Sha256::digest(&self.envelope_bytes)) != self.envelope_sha256
        {
            Err(IdentityResolutionPersistenceErrorV1::InvalidInput)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyIdentityEvidenceOperationV1 {
    pub input: IdentityResolutionEnvelopeRecordV1,
    pub evidence: IdentityMatchEvidenceV1,
    pub proposal: IdentityResolutionEnvelopeRecordV1,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityResolutionReplayOutcomeV1 {
    Applied(IdentityResolutionEnvelopeRecordV1),
    Replayed(IdentityResolutionEnvelopeRecordV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityResolutionOutboxRecordV1 {
    pub record: IdentityResolutionEnvelopeRecordV1,
    pub candidate_id: [u8; 16],
    pub created_at_unix_millis: i64,
}

pub struct IdentityResolutionOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: IdentityResolutionOutboxRecordV1,
}

impl IdentityResolutionOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &IdentityResolutionOutboxRecordV1 {
        &self.record
    }
    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at: i64,
    ) -> Result<(), IdentityResolutionPersistenceErrorV1> {
        if expected_sha256 != self.record.record.envelope_sha256
            || published_at < self.record.created_at_unix_millis
        {
            return Err(IdentityResolutionPersistenceErrorV1::HashMismatch);
        }
        let affected = sqlx::query("UPDATE makosh_data.identity_resolution_outbox SET published_at_unix_millis=$3 WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 AND published_at_unix_millis IS NULL")
            .bind(&self.logical_owner_id).bind(self.record.record.message_id.as_slice()).bind(published_at).bind(expected_sha256.as_slice()).execute(&mut *self.transaction).await.map_err(|_| storage())?.rows_affected();
        if affected != 1 {
            return Err(IdentityResolutionPersistenceErrorV1::Conflict);
        }
        self.transaction.commit().await.map_err(|_| storage())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityResolutionPersistenceErrorV1 {
    InvalidInput,
    Conflict,
    RevisionConflict,
    NotFound,
    HashMismatch,
    StorageUnavailable,
}

#[derive(Clone)]
pub struct IdentityResolutionPersistenceV1 {
    pool: PgPool,
}

impl IdentityResolutionPersistenceV1 {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg(feature = "conformance-test-support")]
    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        host: &str,
        port: u32,
        password: &str,
    ) -> Result<Self, IdentityResolutionPersistenceErrorV1> {
        if host.is_empty()
            || port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(storage());
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(u16::try_from(port).map_err(|_| storage())?)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|_| storage())?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), IdentityResolutionPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| storage())
    }

    pub async fn replay_if_completed(
        &self,
        owner: &str,
        input: &IdentityResolutionEnvelopeRecordV1,
    ) -> Result<Option<IdentityResolutionReplayOutcomeV1>, IdentityResolutionPersistenceErrorV1>
    {
        validate_owner(owner)?;
        input.validate()?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, owner).await?;
        let replay = replay_in_transaction(&mut tx, owner, input).await?;
        tx.rollback().await.map_err(|_| storage())?;
        Ok(replay)
    }

    pub async fn apply_once(
        &self,
        input: &ApplyIdentityEvidenceOperationV1,
    ) -> Result<IdentityResolutionReplayOutcomeV1, IdentityResolutionPersistenceErrorV1> {
        input.input.validate()?;
        input.proposal.validate()?;
        validate_identity_match_evidence_v1(&input.evidence).map_err(core_error)?;
        if input.proposal.message_id == input.input.message_id
            || input.completed_at_unix_millis < input.evidence.observed_at_unix_millis
        {
            return Err(IdentityResolutionPersistenceErrorV1::InvalidInput);
        }
        let owner = &input.evidence.logical_owner_id;
        validate_owner(owner)?;
        if let Some(replay) = self.replay_if_completed(owner, &input.input).await? {
            return Ok(replay);
        }
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, owner).await?;
        if let Some(replay) = replay_in_transaction(&mut tx, owner, &input.input).await? {
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(replay);
        }
        let current=sqlx::query("SELECT evidence_event_id,observed_at_unix_millis,resulting_owner_revision FROM makosh_data.identity_resolution_candidates WHERE logical_owner_id=$1 AND candidate_id=$2 FOR UPDATE")
            .bind(owner).bind(input.evidence.candidate_id.as_slice()).fetch_optional(&mut *tx).await.map_err(|_|storage())?;
        if let Some(row) = current {
            let revision = u64::try_from(
                row.try_get::<i64, _>("resulting_owner_revision")
                    .map_err(|_| storage())?,
            )
            .map_err(|_| storage())?;
            let observed: i64 = row
                .try_get("observed_at_unix_millis")
                .map_err(|_| storage())?;
            if input.evidence.resulting_owner_revision <= revision
                || input.evidence.observed_at_unix_millis < observed
            {
                return Err(IdentityResolutionPersistenceErrorV1::RevisionConflict);
            }
            update_candidate(&mut tx, input).await?;
        } else {
            insert_candidate(&mut tx, input).await?;
        }
        sqlx::query("INSERT INTO makosh_data.identity_resolution_inbox (logical_owner_id,message_id,envelope_sha256,envelope_bytes,candidate_id,proposal_message_id,completed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7)")
            .bind(owner).bind(input.input.message_id.as_slice()).bind(input.input.envelope_sha256.as_slice()).bind(&input.input.envelope_bytes).bind(input.evidence.candidate_id.as_slice()).bind(input.proposal.message_id.as_slice()).bind(input.completed_at_unix_millis).execute(&mut *tx).await.map_err(|_|storage())?;
        sqlx::query("INSERT INTO makosh_data.identity_resolution_outbox (logical_owner_id,message_id,envelope_sha256,envelope_bytes,candidate_id,created_at_unix_millis,published_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,NULL)")
            .bind(owner).bind(input.proposal.message_id.as_slice()).bind(input.proposal.envelope_sha256.as_slice()).bind(&input.proposal.envelope_bytes).bind(input.evidence.candidate_id.as_slice()).bind(input.completed_at_unix_millis).execute(&mut *tx).await.map_err(|_|storage())?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(IdentityResolutionReplayOutcomeV1::Applied(
            input.proposal.clone(),
        ))
    }

    pub async fn claim_next_pending_outbox(
        &self,
        owner: &str,
    ) -> Result<Option<IdentityResolutionOutboxPublishClaimV1>, IdentityResolutionPersistenceErrorV1>
    {
        validate_owner(owner)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, owner).await?;
        let row=sqlx::query("SELECT message_id,envelope_sha256,envelope_bytes,candidate_id,created_at_unix_millis FROM makosh_data.identity_resolution_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL ORDER BY outbox_sequence FOR UPDATE SKIP LOCKED LIMIT 1")
            .bind(owner).fetch_optional(&mut *tx).await.map_err(|_|storage())?;
        let Some(row) = row else {
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(None);
        };
        let record = IdentityResolutionOutboxRecordV1 {
            record: IdentityResolutionEnvelopeRecordV1 {
                message_id: bytes(&row, "message_id")?,
                envelope_sha256: bytes(&row, "envelope_sha256")?,
                envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
            },
            candidate_id: bytes(&row, "candidate_id")?,
            created_at_unix_millis: row
                .try_get("created_at_unix_millis")
                .map_err(|_| storage())?,
        };
        record.record.validate()?;
        Ok(Some(IdentityResolutionOutboxPublishClaimV1 {
            transaction: tx,
            logical_owner_id: owner.to_owned(),
            record,
        }))
    }
}

async fn replay_in_transaction(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    input: &IdentityResolutionEnvelopeRecordV1,
) -> Result<Option<IdentityResolutionReplayOutcomeV1>, IdentityResolutionPersistenceErrorV1> {
    let row=sqlx::query("SELECT i.envelope_sha256,i.envelope_bytes,o.message_id AS proposal_message_id,o.envelope_sha256 AS proposal_sha256,o.envelope_bytes AS proposal_bytes FROM makosh_data.identity_resolution_inbox i JOIN makosh_data.identity_resolution_outbox o ON o.logical_owner_id=i.logical_owner_id AND o.message_id=i.proposal_message_id WHERE i.logical_owner_id=$1 AND i.message_id=$2 FOR UPDATE OF i,o")
        .bind(owner).bind(input.message_id.as_slice()).fetch_optional(&mut **tx).await.map_err(|_|storage())?;
    let Some(row) = row else {
        return Ok(None);
    };
    if bytes::<32>(&row, "envelope_sha256")? != input.envelope_sha256
        || row
            .try_get::<Vec<u8>, _>("envelope_bytes")
            .map_err(|_| storage())?
            != input.envelope_bytes
    {
        return Err(IdentityResolutionPersistenceErrorV1::Conflict);
    }
    let proposal = IdentityResolutionEnvelopeRecordV1 {
        message_id: bytes(&row, "proposal_message_id")?,
        envelope_sha256: bytes(&row, "proposal_sha256")?,
        envelope_bytes: row.try_get("proposal_bytes").map_err(|_| storage())?,
    };
    proposal.validate()?;
    Ok(Some(IdentityResolutionReplayOutcomeV1::Replayed(proposal)))
}

async fn insert_candidate(
    tx: &mut Transaction<'_, Postgres>,
    input: &ApplyIdentityEvidenceOperationV1,
) -> Result<(), IdentityResolutionPersistenceErrorV1> {
    write_candidate(tx, input, true).await
}
async fn update_candidate(
    tx: &mut Transaction<'_, Postgres>,
    input: &ApplyIdentityEvidenceOperationV1,
) -> Result<(), IdentityResolutionPersistenceErrorV1> {
    write_candidate(tx, input, false).await
}
async fn write_candidate(
    tx: &mut Transaction<'_, Postgres>,
    input: &ApplyIdentityEvidenceOperationV1,
    insert: bool,
) -> Result<(), IdentityResolutionPersistenceErrorV1> {
    let e = &input.evidence;
    let query = if insert {
        sqlx::query(
            "INSERT INTO makosh_data.identity_resolution_candidates (logical_owner_id,candidate_id,evidence_event_id,first_person_id,second_person_id,first_integration_public_id,first_account_public_id,first_source_public_id,second_integration_public_id,second_account_public_id,second_source_public_id,match_kind,observed_at_unix_millis,resulting_owner_revision,proposal_message_id,proposal_sha256,proposal_bytes,updated_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)",
        )
    } else {
        sqlx::query(
            "UPDATE makosh_data.identity_resolution_candidates SET evidence_event_id=$3,first_person_id=$4,second_person_id=$5,first_integration_public_id=$6,first_account_public_id=$7,first_source_public_id=$8,second_integration_public_id=$9,second_account_public_id=$10,second_source_public_id=$11,match_kind=$12,observed_at_unix_millis=$13,resulting_owner_revision=$14,proposal_message_id=$15,proposal_sha256=$16,proposal_bytes=$17,updated_at_unix_millis=$18 WHERE logical_owner_id=$1 AND candidate_id=$2",
        )
    };
    let affected = query
        .bind(&e.logical_owner_id)
        .bind(e.candidate_id.as_slice())
        .bind(e.evidence_event_id.as_slice())
        .bind(e.first_person_id.as_slice())
        .bind(e.second_person_id.as_slice())
        .bind(e.first_source.integration_public_id.as_slice())
        .bind(e.first_source.account_public_id.as_slice())
        .bind(e.first_source.provider_source_contact_public_id.as_slice())
        .bind(e.second_source.integration_public_id.as_slice())
        .bind(e.second_source.account_public_id.as_slice())
        .bind(e.second_source.provider_source_contact_public_id.as_slice())
        .bind(match_kind(e.match_kind))
        .bind(e.observed_at_unix_millis)
        .bind(
            i64::try_from(e.resulting_owner_revision)
                .map_err(|_| IdentityResolutionPersistenceErrorV1::InvalidInput)?,
        )
        .bind(input.proposal.message_id.as_slice())
        .bind(input.proposal.envelope_sha256.as_slice())
        .bind(&input.proposal.envelope_bytes)
        .bind(input.completed_at_unix_millis)
        .execute(&mut **tx)
        .await
        .map_err(|_| storage())?
        .rows_affected();
    if affected != 1 {
        return Err(IdentityResolutionPersistenceErrorV1::Conflict);
    }
    Ok(())
}

async fn set_owner(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<(), IdentityResolutionPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map_err(|_| storage())?;
    Ok(())
}
fn validate_owner(owner: &str) -> Result<(), IdentityResolutionPersistenceErrorV1> {
    if !owner.is_empty()
        && owner.len() <= 128
        && owner.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
    {
        Ok(())
    } else {
        Err(IdentityResolutionPersistenceErrorV1::InvalidInput)
    }
}
fn bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<[u8; N], IdentityResolutionPersistenceErrorV1> {
    row.try_get::<Vec<u8>, _>(name)
        .map_err(|_| storage())?
        .try_into()
        .map_err(|_| storage())
}
const fn match_kind(v: IdentityResolutionMatchKindV1) -> i16 {
    match v {
        IdentityResolutionMatchKindV1::NormalizedEmail => 1,
        IdentityResolutionMatchKindV1::NormalizedPhone => 2,
    }
}
const fn core_error(_: IdentityResolutionCoreErrorV1) -> IdentityResolutionPersistenceErrorV1 {
    IdentityResolutionPersistenceErrorV1::InvalidInput
}
const fn storage() -> IdentityResolutionPersistenceErrorV1 {
    IdentityResolutionPersistenceErrorV1::StorageUnavailable
}
fn zero(v: &[u8]) -> bool {
    v.iter().all(|b| *b == 0)
}
