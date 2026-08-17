use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    ApplyMailPersonsSyncAccountLifecycleV1, BeginMailPersonsSyncRunV1,
    CompleteMailPersonsSyncPageV1, MAIL_PERSONS_SYNC_OUTBOX_READ_LIMIT_V1,
    MailPersonsSyncAccountLifecycleKindV1, MailPersonsSyncEnvelopeRecordV1,
    MailPersonsSyncExpiredRunContextV1, MailPersonsSyncOutboxRecordV1,
    MailPersonsSyncPageContinuationV1, MailPersonsSyncPageFinalizationContextV1,
    MailPersonsSyncPersistenceErrorV1, MailPersonsSyncReplayOutcomeV1, MailPersonsSyncRunContextV1,
    MailPersonsSyncScheduleControlOutboxRecordV1, MailPersonsSyncSemanticKindV1,
    MailPersonsSyncSourceCommandContextV1, MailPersonsSyncStoredRejectCodeV1,
    RecordMailPersonsSyncPersonsTerminalV1, RejectMailPersonsSyncAccountBusyV1,
    StageMailPersonsSyncSourceV1, StagedSourceV1, mail_persons_sync_semantic_order_key_v1,
    validate_page_promotion_v1,
};

#[derive(Clone)]
pub struct MailPersonsSyncPersistenceV1 {
    pool: PgPool,
}

/// Keeps the selected outbox row locked from selection through broker publish.
/// Reclaim therefore either observes a committed publication or supersedes an
/// unlocked unpublished row; it cannot race between load and publication CAS.
pub struct MailPersonsSyncOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: MailPersonsSyncOutboxRecordV1,
}

impl MailPersonsSyncOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &MailPersonsSyncOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        if expected_envelope_sha256 != self.record.record.envelope_sha256
            || published_at_unix_millis < self.record.created_at_unix_millis
        {
            return Err(MailPersonsSyncPersistenceErrorV1::HashMismatch);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.mail_persons_sync_outbox SET published_at_unix_millis=$3 \
             WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 \
             AND published_at_unix_millis IS NULL AND superseded_by_run_id IS NULL",
        )
        .bind(&self.logical_owner_id)
        .bind(self.record.record.message_id.as_slice())
        .bind(published_at_unix_millis)
        .bind(expected_envelope_sha256.as_slice())
        .execute(&mut *self.transaction)
        .await
        .map_err(|_| storage())?
        .rows_affected();
        if affected != 1 {
            return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
        }
        self.transaction.commit().await.map_err(|_| storage())
    }
}

impl MailPersonsSyncPersistenceV1 {
    pub async fn apply_account_lifecycle_once<F>(
        &self,
        input: &ApplyMailPersonsSyncAccountLifecycleV1,
        build_schedule_control: F,
    ) -> Result<MailPersonsSyncReplayOutcomeV1, MailPersonsSyncPersistenceErrorV1>
    where
        F: FnOnce(
            u64,
        )
            -> Result<MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncPersistenceErrorV1>,
    {
        input.validate()?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, &input.logical_owner_id).await?;
        if let Some(row) = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes FROM makosh_data.mail_persons_sync_account_inbox \
             WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.lifecycle.message_id.as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| storage())?
        {
            let stored = MailPersonsSyncEnvelopeRecordV1 {
                message_id: input.lifecycle.message_id,
                envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
                envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
            };
            stored.validate()?;
            if stored != input.lifecycle {
                return Err(MailPersonsSyncPersistenceErrorV1::CommandConflict);
            }
            tx.commit().await.map_err(|_| storage())?;
            return Ok(MailPersonsSyncReplayOutcomeV1 { replayed: true });
        }
        let current = sqlx::query(
            "SELECT integration_public_id,mapping_revision,state,schedule_revision,updated_at_unix_millis \
             FROM makosh_data.mail_persons_sync_account_bindings WHERE logical_owner_id=$1 \
             AND account_public_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| storage())?;
        let schedule_revision = if let Some(row) = &current {
            let current_mapping_revision = u64_value(row, "mapping_revision")?;
            let current_state = row.try_get::<i16, _>("state").map_err(|_| storage())?;
            let exact_stable_mapping_retirement = current_mapping_revision
                == input.mapping_revision
                && current_state == MailPersonsSyncAccountLifecycleKindV1::Ready as i16
                && input.kind == MailPersonsSyncAccountLifecycleKindV1::Retired;
            let higher_revision_ready = current_mapping_revision < input.mapping_revision
                && input.kind == MailPersonsSyncAccountLifecycleKindV1::Ready;
            if bytes::<16>(row, "integration_public_id")? != input.integration_public_id
                || (!exact_stable_mapping_retirement && !higher_revision_ready)
                || row
                    .try_get::<i64, _>("updated_at_unix_millis")
                    .map_err(|_| storage())?
                    > input.processed_at_unix_millis
            {
                return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
            }
            u64_value(row, "schedule_revision")?
                .checked_add(1)
                .ok_or(MailPersonsSyncPersistenceErrorV1::StateConflict)?
        } else {
            if input.kind == MailPersonsSyncAccountLifecycleKindV1::Retired {
                return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
            }
            1
        };
        let schedule_control = build_schedule_control(schedule_revision)?;
        schedule_control.validate()?;
        if schedule_control.message_id == input.lifecycle.message_id {
            return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_account_bindings \
             (logical_owner_id,account_public_id,integration_public_id,mapping_revision,state,schedule_revision,updated_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (logical_owner_id,account_public_id) DO UPDATE SET \
             integration_public_id=EXCLUDED.integration_public_id,mapping_revision=EXCLUDED.mapping_revision, \
             state=EXCLUDED.state,schedule_revision=EXCLUDED.schedule_revision,updated_at_unix_millis=EXCLUDED.updated_at_unix_millis",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.integration_public_id.as_slice())
        .bind(i64::try_from(input.mapping_revision).map_err(|_| invalid())?)
        .bind(input.kind as i16)
        .bind(i64::try_from(schedule_revision).map_err(|_| invalid())?)
        .bind(input.processed_at_unix_millis)
        .execute(&mut *tx).await.map_err(|_| storage())?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_account_inbox \
             (logical_owner_id,message_id,envelope_sha256,envelope_bytes,account_public_id,mapping_revision,semantic_kind,processed_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.lifecycle.message_id.as_slice())
        .bind(input.lifecycle.envelope_sha256.as_slice())
        .bind(&input.lifecycle.envelope_bytes)
        .bind(input.account_public_id.as_slice())
        .bind(i64::try_from(input.mapping_revision).map_err(|_| invalid())?)
        .bind(input.kind as i16)
        .bind(input.processed_at_unix_millis)
        .execute(&mut *tx).await.map_err(|_| storage())?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_schedule_control_outbox \
             (logical_owner_id,message_id,envelope_sha256,envelope_bytes,account_public_id,mapping_revision,schedule_revision,semantic_kind,created_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(&input.logical_owner_id)
        .bind(schedule_control.message_id.as_slice())
        .bind(schedule_control.envelope_sha256.as_slice())
        .bind(&schedule_control.envelope_bytes)
        .bind(input.account_public_id.as_slice())
        .bind(i64::try_from(input.mapping_revision).map_err(|_| invalid())?)
        .bind(i64::try_from(schedule_revision).map_err(|_| invalid())?)
        .bind(input.kind as i16)
        .bind(input.processed_at_unix_millis)
        .execute(&mut *tx).await.map_err(|_| storage())?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncReplayOutcomeV1 { replayed: false })
    }

    pub async fn load_pending_schedule_control(
        &self,
        logical_owner_id: &str,
    ) -> Result<
        Option<MailPersonsSyncScheduleControlOutboxRecordV1>,
        MailPersonsSyncPersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT outbox_sequence,message_id,envelope_sha256,envelope_bytes,account_public_id, \
             mapping_revision,schedule_revision,semantic_kind,created_at_unix_millis \
             FROM makosh_data.mail_persons_sync_schedule_control_outbox WHERE logical_owner_id=$1 \
             AND published_at_unix_millis IS NULL ORDER BY outbox_sequence LIMIT 1",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| storage())?;
        let result = row
            .map(|row| {
                let record = MailPersonsSyncEnvelopeRecordV1 {
                    message_id: bytes::<16>(&row, "message_id")?,
                    envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
                    envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
                };
                record.validate()?;
                Ok(MailPersonsSyncScheduleControlOutboxRecordV1 {
                    record,
                    outbox_sequence: u64_value(&row, "outbox_sequence")?,
                    account_public_id: bytes::<16>(&row, "account_public_id")?,
                    mapping_revision: u64_value(&row, "mapping_revision")?,
                    schedule_revision: u64_value(&row, "schedule_revision")?,
                    kind: match row
                        .try_get::<i16, _>("semantic_kind")
                        .map_err(|_| storage())?
                    {
                        1 => MailPersonsSyncAccountLifecycleKindV1::Ready,
                        2 => MailPersonsSyncAccountLifecycleKindV1::Retired,
                        _ => return Err(MailPersonsSyncPersistenceErrorV1::StateConflict),
                    },
                    created_at_unix_millis: row
                        .try_get("created_at_unix_millis")
                        .map_err(|_| storage())?,
                })
            })
            .transpose()?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(result)
    }

    pub async fn mark_schedule_control_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes,created_at_unix_millis,published_at_unix_millis \
             FROM makosh_data.mail_persons_sync_schedule_control_outbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        ).bind(logical_owner_id).bind(message_id.as_slice()).fetch_one(&mut *tx).await.map_err(|_| storage())?;
        let record = MailPersonsSyncEnvelopeRecordV1 {
            message_id,
            envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
        };
        record.validate()?;
        let created: i64 = row
            .try_get("created_at_unix_millis")
            .map_err(|_| storage())?;
        if record.envelope_sha256 != expected_sha256 || published_at_unix_millis < created {
            return Err(MailPersonsSyncPersistenceErrorV1::HashMismatch);
        }
        if row
            .try_get::<Option<i64>, _>("published_at_unix_millis")
            .map_err(|_| storage())?
            .is_none()
        {
            sqlx::query("UPDATE makosh_data.mail_persons_sync_schedule_control_outbox SET published_at_unix_millis=$3 WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 AND published_at_unix_millis IS NULL")
                .bind(logical_owner_id).bind(message_id.as_slice()).bind(published_at_unix_millis).bind(expected_sha256.as_slice())
                .execute(&mut *tx).await.map_err(|_| storage())?;
        }
        tx.commit().await.map_err(|_| storage())
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        host: &str,
        port: u32,
        password: &str,
    ) -> Result<Self, MailPersonsSyncPersistenceErrorV1> {
        if host.is_empty()
            || port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(MailPersonsSyncPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .statement_cache_capacity(0)
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

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg(feature = "conformance-test-support")]
    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn verify_storage_ready(&self) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| storage())
    }

    pub async fn load_run_context(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<MailPersonsSyncRunContextV1, MailPersonsSyncPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if run_id.iter().all(|byte| *byte == 0) {
            return Err(invalid());
        }
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT r.account_public_id,r.run_id,r.state,r.next_page_sequence,r.processed_pages, \
             r.processed_sources,r.rejection_code,s.scheduler_message_id,s.lease_epoch,s.lease_expires_at_unix_millis \
             FROM makosh_data.mail_persons_sync_runs r JOIN makosh_data.mail_persons_sync_scheduler_runs s \
             USING (logical_owner_id,run_id) WHERE r.logical_owner_id=$1 AND r.run_id=$2",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .ok_or_else(state)?;
        let context = MailPersonsSyncRunContextV1 {
            account_public_id: bytes::<16>(&row, "account_public_id")?,
            run_id: bytes::<16>(&row, "run_id")?,
            state: u8::try_from(row.try_get::<i16, _>("state").map_err(|_| storage())?)
                .map_err(|_| state())?,
            next_page_sequence: u64_value(&row, "next_page_sequence")?,
            processed_pages: u64_value(&row, "processed_pages")?,
            processed_sources: u64_value(&row, "processed_sources")?,
            rejection_code: row
                .try_get::<Option<i16>, _>("rejection_code")
                .map_err(|_| storage())?
                .map(MailPersonsSyncStoredRejectCodeV1::try_from)
                .transpose()?,
            scheduler_message_id: bytes::<16>(&row, "scheduler_message_id")?,
            lease_epoch: u64_value(&row, "lease_epoch")?,
            lease_expires_at_unix_millis: row
                .try_get("lease_expires_at_unix_millis")
                .map_err(|_| storage())?,
        };
        transaction.rollback().await.map_err(|_| storage())?;
        Ok(context)
    }

    pub async fn load_source_command_context(
        &self,
        logical_owner_id: &str,
        persons_command_id: [u8; 16],
    ) -> Result<MailPersonsSyncSourceCommandContextV1, MailPersonsSyncPersistenceErrorV1> {
        self.find_source_command_context(logical_owner_id, persons_command_id)
            .await?
            .ok_or_else(state)
    }

    pub async fn find_source_command_context(
        &self,
        logical_owner_id: &str,
        persons_command_id: [u8; 16],
    ) -> Result<Option<MailPersonsSyncSourceCommandContextV1>, MailPersonsSyncPersistenceErrorV1>
    {
        validate_owner(logical_owner_id)?;
        if persons_command_id.iter().all(|byte| *byte == 0) {
            return Err(invalid());
        }
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT account_public_id,run_id,page_sequence FROM makosh_data.mail_persons_sync_sources \
             WHERE logical_owner_id=$1 AND persons_command_id=$2",
        )
        .bind(logical_owner_id)
        .bind(persons_command_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let context = row
            .as_ref()
            .map(|row| {
                Ok(MailPersonsSyncSourceCommandContextV1 {
                    account_public_id: bytes::<16>(row, "account_public_id")?,
                    run_id: bytes::<16>(row, "run_id")?,
                    page_sequence: u64_value(row, "page_sequence")?,
                })
            })
            .transpose()?;
        transaction.rollback().await.map_err(|_| storage())?;
        Ok(context)
    }

    #[cfg(feature = "conformance-test-support")]
    pub async fn begin_run_for_conformance(
        &self,
        input: &BeginMailPersonsSyncRunV1,
    ) -> Result<MailPersonsSyncReplayOutcomeV1, MailPersonsSyncPersistenceErrorV1> {
        input.validate()?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, &input.logical_owner_id).await?;
        if let Some(row) = sqlx::query(
            "SELECT r.account_public_id, r.run_id, r.run_fingerprint, s.scheduler_envelope_sha256, \
             s.lease_epoch, s.lease_expires_at_unix_millis FROM makosh_data.mail_persons_sync_scheduler_runs s \
             JOIN makosh_data.mail_persons_sync_runs r USING (logical_owner_id, run_id) \
             WHERE s.logical_owner_id = $1 AND s.scheduler_message_id = $2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.scheduler_command.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        {
            let exact = bytes::<16>(&row, "account_public_id")? == input.account_public_id
                && bytes::<16>(&row, "run_id")? == input.run_id
                && bytes::<32>(&row, "run_fingerprint")? == input.run_fingerprint
                && bytes::<32>(&row, "scheduler_envelope_sha256")?
                    == input.scheduler_command.envelope_sha256
                && u64_value(&row, "lease_epoch")? == input.lease_epoch
                && row
                    .try_get::<i64, _>("lease_expires_at_unix_millis")
                    .map_err(|_| storage())?
                    == input.lease_expires_at_unix_millis;
            if !exact {
                return Err(MailPersonsSyncPersistenceErrorV1::CommandConflict);
            }
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(MailPersonsSyncReplayOutcomeV1 { replayed: true });
        }
        let active_account_run: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM makosh_data.mail_persons_sync_runs \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND state IN (1,2))",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        if active_account_run {
            return Err(MailPersonsSyncPersistenceErrorV1::AccountBusy);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_runs \
             (logical_owner_id, account_public_id, run_id, run_fingerprint, state, state_revision, \
              next_page_sequence, processed_pages, processed_sources, rejection_code, \
              created_at_unix_millis, updated_at_unix_millis) \
             VALUES ($1,$2,$3,$4,1,1,1,0,0,NULL,$5,$5)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(input.run_fingerprint.as_slice())
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(begin_run_insert_error)?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_scheduler_runs \
             (logical_owner_id, run_id, scheduler_message_id, scheduler_envelope_sha256, lease_epoch, \
              lease_expires_at_unix_millis, acceptance_queued, terminal_queued) \
             VALUES ($1,$2,$3,$4,$5,$6,FALSE,FALSE)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(input.scheduler_command.message_id.as_slice())
        .bind(input.scheduler_command.envelope_sha256.as_slice())
        .bind(i64::try_from(input.lease_epoch).map_err(|_| invalid())?)
        .bind(input.lease_expires_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            input.run_id,
            0,
            MailPersonsSyncSemanticKindV1::SchedulerAcceptance,
            None,
            0,
            input.scheduler_acceptance.clone(),
            input.received_at_unix_millis,
        )
        .await?;
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            input.run_id,
            0,
            MailPersonsSyncSemanticKindV1::MailFetch,
            None,
            1,
            input.initial_fetch.clone(),
            input.received_at_unix_millis,
        )
        .await?;
        sqlx::query(
            "UPDATE makosh_data.mail_persons_sync_scheduler_runs SET acceptance_queued=TRUE \
             WHERE logical_owner_id=$1 AND run_id=$2 AND acceptance_queued=FALSE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncReplayOutcomeV1 { replayed: false })
    }

    pub async fn begin_run_reclaiming_expired_once<F>(
        &self,
        input: &BeginMailPersonsSyncRunV1,
        build_expired_terminal: F,
    ) -> Result<MailPersonsSyncReplayOutcomeV1, MailPersonsSyncPersistenceErrorV1>
    where
        F: FnOnce(
            MailPersonsSyncExpiredRunContextV1,
        )
            -> Result<MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncPersistenceErrorV1>,
    {
        input.validate()?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, &input.logical_owner_id).await?;
        if let Some(row) = sqlx::query(
            "SELECT r.account_public_id,r.run_id,r.run_fingerprint,s.scheduler_envelope_sha256, \
             s.lease_epoch,s.lease_expires_at_unix_millis FROM makosh_data.mail_persons_sync_scheduler_runs s \
             JOIN makosh_data.mail_persons_sync_runs r USING (logical_owner_id,run_id) \
             WHERE s.logical_owner_id=$1 AND s.scheduler_message_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.scheduler_command.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        {
            let exact = bytes::<16>(&row, "account_public_id")? == input.account_public_id
                && bytes::<16>(&row, "run_id")? == input.run_id
                && bytes::<32>(&row, "run_fingerprint")? == input.run_fingerprint
                && bytes::<32>(&row, "scheduler_envelope_sha256")?
                    == input.scheduler_command.envelope_sha256
                && u64_value(&row, "lease_epoch")? == input.lease_epoch
                && row
                    .try_get::<i64, _>("lease_expires_at_unix_millis")
                    .map_err(|_| storage())?
                    == input.lease_expires_at_unix_millis;
            if !exact {
                return Err(MailPersonsSyncPersistenceErrorV1::CommandConflict);
            }
            transaction.rollback().await.map_err(|_| storage())?;
            return Ok(MailPersonsSyncReplayOutcomeV1 { replayed: true });
        }

        if let Some(active) = sqlx::query(
            "SELECT r.run_id,r.next_page_sequence,s.scheduler_message_id,s.lease_epoch, \
             s.lease_expires_at_unix_millis FROM makosh_data.mail_persons_sync_runs r \
             JOIN makosh_data.mail_persons_sync_scheduler_runs s USING (logical_owner_id,run_id) \
             WHERE r.logical_owner_id=$1 AND r.account_public_id=$2 AND r.state IN (1,2) FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        {
            let expires_at = active
                .try_get::<i64, _>("lease_expires_at_unix_millis")
                .map_err(|_| storage())?;
            if expires_at >= input.received_at_unix_millis {
                return Err(MailPersonsSyncPersistenceErrorV1::AccountBusy);
            }
            let expired = MailPersonsSyncExpiredRunContextV1 {
                logical_owner_id: input.logical_owner_id.clone(),
                account_public_id: input.account_public_id,
                run_id: bytes::<16>(&active, "run_id")?,
                scheduler_message_id: bytes::<16>(&active, "scheduler_message_id")?,
                lease_epoch: u64_value(&active, "lease_epoch")?,
                lease_expires_at_unix_millis: expires_at,
                next_page_sequence: u64_value(&active, "next_page_sequence")?,
            };
            let terminal = build_expired_terminal(expired.clone())?;
            terminal.validate()?;
            sqlx::query(
                "UPDATE makosh_data.mail_persons_sync_outbox \
                 SET superseded_by_run_id=$3,superseded_at_unix_millis=$4 \
                 WHERE logical_owner_id=$1 AND run_id=$2 AND semantic_kind <> 7 \
                 AND published_at_unix_millis IS NULL AND superseded_by_run_id IS NULL",
            )
            .bind(&input.logical_owner_id)
            .bind(expired.run_id.as_slice())
            .bind(input.run_id.as_slice())
            .bind(input.received_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                expired.run_id,
                expired.next_page_sequence,
                MailPersonsSyncSemanticKindV1::SchedulerTerminal,
                None,
                502,
                terminal,
                input.received_at_unix_millis,
            )
            .await?;
            sqlx::query(
                "UPDATE makosh_data.mail_persons_sync_runs SET state=4,state_revision=state_revision+1, \
                 rejection_code=3,updated_at_unix_millis=$3 WHERE logical_owner_id=$1 AND run_id=$2 \
                 AND state IN (1,2) AND updated_at_unix_millis <= $3",
            )
            .bind(&input.logical_owner_id)
            .bind(expired.run_id.as_slice())
            .bind(input.received_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
            sqlx::query(
                "UPDATE makosh_data.mail_persons_sync_scheduler_runs SET terminal_queued=TRUE \
                 WHERE logical_owner_id=$1 AND run_id=$2 AND terminal_queued=FALSE",
            )
            .bind(&input.logical_owner_id)
            .bind(expired.run_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        }

        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_runs \
             (logical_owner_id,account_public_id,run_id,run_fingerprint,state,state_revision, \
              next_page_sequence,processed_pages,processed_sources,rejection_code,created_at_unix_millis,updated_at_unix_millis) \
             VALUES ($1,$2,$3,$4,1,1,1,0,0,NULL,$5,$5)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(input.run_fingerprint.as_slice())
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(begin_run_insert_error)?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_scheduler_runs \
             (logical_owner_id,run_id,scheduler_message_id,scheduler_envelope_sha256,lease_epoch, \
              lease_expires_at_unix_millis,acceptance_queued,terminal_queued) \
             VALUES ($1,$2,$3,$4,$5,$6,FALSE,FALSE)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(input.scheduler_command.message_id.as_slice())
        .bind(input.scheduler_command.envelope_sha256.as_slice())
        .bind(i64::try_from(input.lease_epoch).map_err(|_| invalid())?)
        .bind(input.lease_expires_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            input.run_id,
            0,
            MailPersonsSyncSemanticKindV1::SchedulerAcceptance,
            None,
            0,
            input.scheduler_acceptance.clone(),
            input.received_at_unix_millis,
        )
        .await?;
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            input.run_id,
            0,
            MailPersonsSyncSemanticKindV1::MailFetch,
            None,
            1,
            input.initial_fetch.clone(),
            input.received_at_unix_millis,
        )
        .await?;
        sqlx::query(
            "UPDATE makosh_data.mail_persons_sync_scheduler_runs SET acceptance_queued=TRUE \
             WHERE logical_owner_id=$1 AND run_id=$2 AND acceptance_queued=FALSE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncReplayOutcomeV1 { replayed: false })
    }

    pub async fn record_account_busy_once(
        &self,
        input: &RejectMailPersonsSyncAccountBusyV1,
    ) -> Result<MailPersonsSyncReplayOutcomeV1, MailPersonsSyncPersistenceErrorV1> {
        input.validate()?;
        let begin = &input.begin;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, &begin.logical_owner_id).await?;
        if let Some(row) = sqlx::query(
            "SELECT r.account_public_id,r.run_fingerprint,r.state,s.scheduler_envelope_sha256, \
             s.lease_epoch,s.lease_expires_at_unix_millis,o.envelope_sha256 \
             FROM makosh_data.mail_persons_sync_runs r \
             JOIN makosh_data.mail_persons_sync_scheduler_runs s USING (logical_owner_id,run_id) \
             JOIN makosh_data.mail_persons_sync_outbox o USING (logical_owner_id,run_id) \
             WHERE r.logical_owner_id=$1 AND r.run_id=$2 AND o.message_id=$3 FOR UPDATE",
        )
        .bind(&begin.logical_owner_id)
        .bind(begin.run_id.as_slice())
        .bind(input.scheduler_terminal.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        {
            let exact = bytes::<16>(&row, "account_public_id")? == begin.account_public_id
                && bytes::<32>(&row, "run_fingerprint")? == begin.run_fingerprint
                && row.try_get::<i16, _>("state").map_err(|_| storage())? == 4
                && bytes::<32>(&row, "scheduler_envelope_sha256")?
                    == begin.scheduler_command.envelope_sha256
                && u64_value(&row, "lease_epoch")? == begin.lease_epoch
                && row
                    .try_get::<i64, _>("lease_expires_at_unix_millis")
                    .map_err(|_| storage())?
                    == begin.lease_expires_at_unix_millis
                && bytes::<32>(&row, "envelope_sha256")?
                    == input.scheduler_terminal.envelope_sha256;
            if !exact {
                return Err(MailPersonsSyncPersistenceErrorV1::CommandConflict);
            }
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(MailPersonsSyncReplayOutcomeV1 { replayed: true });
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_runs \
             (logical_owner_id,account_public_id,run_id,run_fingerprint,state,state_revision, \
              next_page_sequence,processed_pages,processed_sources,rejection_code, \
              created_at_unix_millis,updated_at_unix_millis) \
             VALUES ($1,$2,$3,$4,4,1,1,0,0,1,$5,$5)",
        )
        .bind(&begin.logical_owner_id)
        .bind(begin.account_public_id.as_slice())
        .bind(begin.run_id.as_slice())
        .bind(begin.run_fingerprint.as_slice())
        .bind(begin.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailPersonsSyncPersistenceErrorV1::CommandConflict)?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_scheduler_runs \
             (logical_owner_id,run_id,scheduler_message_id,scheduler_envelope_sha256,lease_epoch, \
              lease_expires_at_unix_millis,acceptance_queued,terminal_queued) \
             VALUES ($1,$2,$3,$4,$5,$6,FALSE,TRUE)",
        )
        .bind(&begin.logical_owner_id)
        .bind(begin.run_id.as_slice())
        .bind(begin.scheduler_command.message_id.as_slice())
        .bind(begin.scheduler_command.envelope_sha256.as_slice())
        .bind(i64::try_from(begin.lease_epoch).map_err(|_| invalid())?)
        .bind(begin.lease_expires_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailPersonsSyncPersistenceErrorV1::CommandConflict)?;
        insert_outbox(
            &mut transaction,
            &begin.logical_owner_id,
            begin.run_id,
            0,
            MailPersonsSyncSemanticKindV1::SchedulerTerminal,
            None,
            0,
            input.scheduler_terminal.clone(),
            begin.received_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncReplayOutcomeV1 { replayed: false })
    }

    pub async fn stage_source_once(
        &self,
        input: &StageMailPersonsSyncSourceV1,
    ) -> Result<MailPersonsSyncReplayOutcomeV1, MailPersonsSyncPersistenceErrorV1> {
        input.validate()?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, &input.logical_owner_id).await?;
        let run = sqlx::query(
            "SELECT account_public_id, state FROM makosh_data.mail_persons_sync_runs \
             WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .ok_or(MailPersonsSyncPersistenceErrorV1::StateConflict)?;
        if bytes::<16>(&run, "account_public_id")? != input.account_public_id
            || run.try_get::<i16, _>("state").map_err(|_| storage())? > 2
        {
            return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
        }
        if let Some(row) = sqlx::query(
            "SELECT envelope_sha256, account_public_id, run_id, page_sequence, command_id, command_fingerprint \
             FROM makosh_data.mail_persons_sync_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.observation.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        {
            let exact = bytes::<32>(&row, "envelope_sha256")? == input.observation.envelope_sha256
                && bytes::<16>(&row, "account_public_id")? == input.account_public_id
                && bytes::<16>(&row, "run_id")? == input.run_id
                && u64_value(&row, "page_sequence")? == input.page_sequence
                && optional_bytes::<16>(&row, "command_id")? == Some(input.persons_command_id)
                && optional_bytes::<32>(&row, "command_fingerprint")?
                    == Some(input.persons_command_fingerprint);
            if !exact {
                return Err(MailPersonsSyncPersistenceErrorV1::CommandConflict);
            }
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(MailPersonsSyncReplayOutcomeV1 { replayed: true });
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_pages \
             (logical_owner_id, account_public_id, run_id, page_sequence, page_digest, observed_sources, \
              updated_sources, removed_sources, staged_sources, has_more, completed_message_id, \
              completed_envelope_sha256, receipt_id, receipt_envelope_sha256, \
              receipt_envelope_bytes, completed_at_unix_millis) \
             VALUES ($1,$2,$3,$4,NULL,0,0,0,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL) ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let page = sqlx::query(
            "SELECT completed_message_id IS NOT NULL AS completion_received,continuation_queued, \
             observed_sources,updated_sources,removed_sources FROM makosh_data.mail_persons_sync_pages \
             WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let completion_received = page
            .try_get::<bool, _>("completion_received")
            .map_err(|_| storage())?;
        if page
            .try_get::<bool, _>("continuation_queued")
            .map_err(|_| storage())?
        {
            return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_inbox \
             (logical_owner_id,message_id,envelope_sha256,semantic_kind,account_public_id,run_id, \
              page_sequence,command_id,command_fingerprint,processed_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.observation.message_id.as_slice())
        .bind(input.observation.envelope_sha256.as_slice())
        .bind(i16::from(input.change_kind))
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .bind(input.persons_command_id.as_slice())
        .bind(input.persons_command_fingerprint.as_slice())
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_sources \
             (logical_owner_id,run_id,page_sequence,observation_message_id,observation_envelope_sha256, \
              integration_public_id,account_public_id,provider_source_contact_public_id,change_kind, \
              source_revision,source_digest,persons_command_id,persons_command_fingerprint, \
              persons_command_envelope_sha256,persons_command_envelope_bytes) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .bind(input.observation.message_id.as_slice())
        .bind(input.observation.envelope_sha256.as_slice())
        .bind(input.integration_public_id.as_slice())
        .bind(input.account_public_id.as_slice())
        .bind(input.provider_source_contact_public_id.as_slice())
        .bind(i16::from(input.change_kind))
        .bind(i64::try_from(input.source_revision).map_err(|_| invalid())?)
        .bind(input.source_digest.as_slice())
        .bind(input.persons_command_id.as_slice())
        .bind(input.persons_command_fingerprint.as_slice())
        .bind(input.persons_command.envelope_sha256.as_slice())
        .bind(&input.persons_command.envelope_bytes)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailPersonsSyncPersistenceErrorV1::CommandConflict)?;
        let staged_sources: i32 = sqlx::query_scalar(
            "UPDATE makosh_data.mail_persons_sync_pages SET staged_sources=staged_sources+1 \
             WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3 \
             AND continuation_queued=FALSE AND staged_sources < 500 RETURNING staged_sources",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .ok_or_else(state)?;
        if completion_received {
            let actual: (i64, i64, i64) = sqlx::query_as(
                "SELECT COUNT(*) FILTER (WHERE change_kind=1),COUNT(*) FILTER (WHERE change_kind=2), \
                 COUNT(*) FILTER (WHERE change_kind=3) FROM makosh_data.mail_persons_sync_sources \
                 WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3",
            )
            .bind(&input.logical_owner_id)
            .bind(input.run_id.as_slice())
            .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| storage())?;
            let expected = (
                i64::from(
                    page.try_get::<i32, _>("observed_sources")
                        .map_err(|_| storage())?,
                ),
                i64::from(
                    page.try_get::<i32, _>("updated_sources")
                        .map_err(|_| storage())?,
                ),
                i64::from(
                    page.try_get::<i32, _>("removed_sources")
                        .map_err(|_| storage())?,
                ),
            );
            if actual.0 > expected.0 || actual.1 > expected.1 || actual.2 > expected.2 {
                return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
            }
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                input.run_id,
                input.page_sequence,
                MailPersonsSyncSemanticKindV1::PersonsCommand,
                Some(input.provider_source_contact_public_id),
                u16::try_from(staged_sources).map_err(|_| invalid())?,
                input.persons_command.clone(),
                input.received_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncReplayOutcomeV1 { replayed: false })
    }

    pub async fn complete_page_once(
        &self,
        input: &CompleteMailPersonsSyncPageV1,
    ) -> Result<MailPersonsSyncReplayOutcomeV1, MailPersonsSyncPersistenceErrorV1> {
        input.validate()?;
        let (continuation_kind, next_fetch, run_result, scheduler_terminal) =
            match &input.continuation {
                MailPersonsSyncPageContinuationV1::NextPage { next_fetch } => {
                    (1_i16, Some(next_fetch), None, None)
                }
                MailPersonsSyncPageContinuationV1::Finished {
                    run_result,
                    scheduler_terminal,
                } => (2_i16, None, Some(run_result), Some(scheduler_terminal)),
                MailPersonsSyncPageContinuationV1::AwaitingPersons => (3_i16, None, None, None),
            };
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, &input.logical_owner_id).await?;
        if let Some(row) = sqlx::query(
            "SELECT envelope_sha256, account_public_id, run_id, page_sequence FROM \
             makosh_data.mail_persons_sync_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.completion.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        {
            let exact = bytes::<32>(&row, "envelope_sha256")? == input.completion.envelope_sha256
                && bytes::<16>(&row, "account_public_id")? == input.account_public_id
                && bytes::<16>(&row, "run_id")? == input.run_id
                && u64_value(&row, "page_sequence")? == input.page_sequence;
            if !exact {
                return Err(MailPersonsSyncPersistenceErrorV1::CommandConflict);
            }
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(MailPersonsSyncReplayOutcomeV1 { replayed: true });
        }
        // A page containing no public source changes has no preceding
        // stage_source_once call. Materialize the same owner-local page shell
        // here so its terminal result and continuation commit atomically.
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_pages \
             (logical_owner_id,account_public_id,run_id,page_sequence,observed_sources, \
              updated_sources,removed_sources,staged_sources,continuation_queued) \
             VALUES ($1,$2,$3,$4,0,0,0,0,FALSE) \
             ON CONFLICT (logical_owner_id,run_id,page_sequence) DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let rows = sqlx::query(
            "SELECT provider_source_contact_public_id, change_kind, persons_command_id, \
             persons_command_envelope_sha256, persons_command_envelope_bytes \
             FROM makosh_data.mail_persons_sync_sources \
             WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3 \
             ORDER BY provider_source_contact_public_id FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let staged = rows
            .iter()
            .map(|row| {
                let kind = row
                    .try_get::<i16, _>("change_kind")
                    .map_err(|_| storage())?;
                Ok(StagedSourceV1 {
                    public_source_id: bytes::<16>(row, "provider_source_contact_public_id")?,
                    observed: u32::from(kind == 1),
                    updated: u32::from(kind == 2),
                    removed: u32::from(kind == 3),
                })
            })
            .collect::<Result<Vec<_>, MailPersonsSyncPersistenceErrorV1>>()?;
        let promotion = validate_page_promotion_v1(
            input.observed_sources,
            input.updated_sources,
            input.removed_sources,
            &staged,
        );
        if let Err(error) = promotion
            && error != MailPersonsSyncPersistenceErrorV1::PageIncomplete
        {
            return Err(error);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_inbox \
             (logical_owner_id,message_id,envelope_sha256,semantic_kind,account_public_id,run_id, \
              page_sequence,command_id,command_fingerprint,processed_at_unix_millis) \
             VALUES ($1,$2,$3,4,$4,$5,$6,NULL,NULL,$7)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.completion.message_id.as_slice())
        .bind(input.completion.envelope_sha256.as_slice())
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .bind(input.completed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        for (index, row) in rows.iter().enumerate() {
            let source_id = bytes::<16>(row, "provider_source_contact_public_id")?;
            let message_id = bytes::<16>(row, "persons_command_id")?;
            let digest = bytes::<32>(row, "persons_command_envelope_sha256")?;
            let bytes: Vec<u8> = row
                .try_get("persons_command_envelope_bytes")
                .map_err(|_| storage())?;
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                input.run_id,
                input.page_sequence,
                MailPersonsSyncSemanticKindV1::PersonsCommand,
                Some(source_id),
                u16::try_from(index + 1).map_err(|_| invalid())?,
                MailPersonsSyncEnvelopeRecordV1 {
                    message_id,
                    envelope_sha256: digest,
                    envelope_bytes: bytes,
                },
                input.completed_at_unix_millis,
            )
            .await?;
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_persons_sync_pages SET page_digest=$4, observed_sources=$5, \
             updated_sources=$6, removed_sources=$7, has_more=$8, completed_message_id=$9, \
             completed_envelope_sha256=$10, receipt_id=$11, receipt_envelope_sha256=$12, \
             receipt_envelope_bytes=$13, completed_at_unix_millis=$14, continuation_kind=$15, \
             next_fetch_id=$16,next_fetch_envelope_sha256=$17,next_fetch_envelope_bytes=$18, \
             run_result_id=$19,run_result_envelope_sha256=$20,run_result_envelope_bytes=$21, \
             scheduler_terminal_id=$22,scheduler_terminal_envelope_sha256=$23, \
             scheduler_terminal_envelope_bytes=$24,rejection_code=$25 \
             WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3 AND completed_message_id IS NULL",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .bind(input.page_digest.as_slice())
        .bind(i32::try_from(input.observed_sources).map_err(|_| invalid())?)
        .bind(i32::try_from(input.updated_sources).map_err(|_| invalid())?)
        .bind(i32::try_from(input.removed_sources).map_err(|_| invalid())?)
        .bind(input.has_more)
        .bind(input.completion.message_id.as_slice())
        .bind(input.completion.envelope_sha256.as_slice())
        .bind(input.page_receipt.message_id.as_slice())
        .bind(input.page_receipt.envelope_sha256.as_slice())
        .bind(&input.page_receipt.envelope_bytes)
        .bind(input.completed_at_unix_millis)
        .bind(continuation_kind)
        .bind(next_fetch.map(|record| record.message_id.as_slice()))
        .bind(next_fetch.map(|record| record.envelope_sha256.as_slice()))
        .bind(next_fetch.map(|record| record.envelope_bytes.as_slice()))
        .bind(run_result.map(|record| record.message_id.as_slice()))
        .bind(run_result.map(|record| record.envelope_sha256.as_slice()))
        .bind(run_result.map(|record| record.envelope_bytes.as_slice()))
        .bind(scheduler_terminal.map(|record| record.message_id.as_slice()))
        .bind(scheduler_terminal.map(|record| record.envelope_sha256.as_slice()))
        .bind(scheduler_terminal.map(|record| record.envelope_bytes.as_slice()))
        .bind(input.rejection_code.map(i16::from))
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        if updated.rows_affected() != 1 {
            return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
        }
        if rows.is_empty()
            && input
                .observed_sources
                .checked_add(input.updated_sources)
                .and_then(|count| count.checked_add(input.removed_sources))
                == Some(0)
        {
            finalize_page(
                &mut transaction,
                &input.logical_owner_id,
                input.run_id,
                input.page_sequence,
                input.completed_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncReplayOutcomeV1 { replayed: false })
    }

    pub async fn record_persons_terminal_once(
        &self,
        input: &RecordMailPersonsSyncPersonsTerminalV1,
    ) -> Result<MailPersonsSyncReplayOutcomeV1, MailPersonsSyncPersistenceErrorV1> {
        input.validate()?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, &input.logical_owner_id).await?;
        if let Some(row) = sqlx::query(
            "SELECT envelope_sha256,account_public_id,run_id,page_sequence,command_id FROM \
             makosh_data.mail_persons_sync_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        ).bind(&input.logical_owner_id).bind(input.result.message_id.as_slice())
            .fetch_optional(&mut *transaction).await.map_err(|_| storage())?
        {
            let exact = bytes::<32>(&row,"envelope_sha256")? == input.result.envelope_sha256
                && bytes::<16>(&row,"account_public_id")? == input.account_public_id
                && bytes::<16>(&row,"run_id")? == input.run_id
                && u64_value(&row,"page_sequence")? == input.page_sequence
                && optional_bytes::<16>(&row,"command_id")? == Some(input.persons_command_id);
            if !exact { return Err(MailPersonsSyncPersistenceErrorV1::CommandConflict); }
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(MailPersonsSyncReplayOutcomeV1 { replayed:true });
        }
        let source = sqlx::query(
            "SELECT s.persons_result_message_id,r.state,sr.lease_expires_at_unix_millis \
             FROM makosh_data.mail_persons_sync_sources s \
             JOIN makosh_data.mail_persons_sync_runs r USING (logical_owner_id,run_id) \
             JOIN makosh_data.mail_persons_sync_scheduler_runs sr USING (logical_owner_id,run_id) \
             WHERE s.logical_owner_id=$1 AND s.run_id=$2 AND s.page_sequence=$3 \
             AND s.persons_command_id=$4 FOR UPDATE OF s,r,sr",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .bind(input.persons_command_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .ok_or_else(state)?;
        if source.try_get::<i16, _>("state").map_err(|_| storage())? > 2 {
            transaction.rollback().await.map_err(|_| storage())?;
            return Ok(MailPersonsSyncReplayOutcomeV1 { replayed: true });
        }
        if input.result_completed_at_unix_millis
            > source
                .try_get::<i64, _>("lease_expires_at_unix_millis")
                .map_err(|_| storage())?
        {
            return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
        }
        if source
            .try_get::<Option<Vec<u8>>, _>("persons_result_message_id")
            .map_err(|_| storage())?
            .is_some()
        {
            return Err(MailPersonsSyncPersistenceErrorV1::CommandConflict);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_inbox \
             (logical_owner_id,message_id,envelope_sha256,semantic_kind,account_public_id,run_id,page_sequence,command_id,command_fingerprint,processed_at_unix_millis) \
             VALUES ($1,$2,$3,5,$4,$5,$6,$7,NULL,$8)",
        ).bind(&input.logical_owner_id).bind(input.result.message_id.as_slice())
            .bind(input.result.envelope_sha256.as_slice()).bind(input.account_public_id.as_slice())
            .bind(input.run_id.as_slice()).bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
            .bind(input.persons_command_id.as_slice()).bind(input.received_at_unix_millis)
            .execute(&mut *transaction).await.map_err(|_| storage())?;
        sqlx::query(
            "UPDATE makosh_data.mail_persons_sync_sources SET persons_result_message_id=$5, \
             persons_result_envelope_sha256=$6,outcome=$7 WHERE logical_owner_id=$1 AND run_id=$2 \
             AND page_sequence=$3 AND persons_command_id=$4 AND persons_result_message_id IS NULL",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .bind(input.persons_command_id.as_slice())
        .bind(input.result.message_id.as_slice())
        .bind(input.result.envelope_sha256.as_slice())
        .bind(i16::from(input.outcome))
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let page_status: (i64, bool, i16) = sqlx::query_as(
            "SELECT COUNT(*) FILTER (WHERE s.persons_result_message_id IS NULL), \
             COALESCE(BOOL_OR(s.outcome=2),FALSE),p.continuation_kind \
             FROM makosh_data.mail_persons_sync_pages p JOIN \
             makosh_data.mail_persons_sync_sources s USING (logical_owner_id,run_id,page_sequence) \
             WHERE p.logical_owner_id=$1 AND p.run_id=$2 AND p.page_sequence=$3 \
             GROUP BY p.continuation_kind",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(i64::try_from(input.page_sequence).map_err(|_| invalid())?)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        if page_status == (0, false, 1) {
            finalize_page(
                &mut transaction,
                &input.logical_owner_id,
                input.run_id,
                input.page_sequence,
                input.received_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncReplayOutcomeV1 { replayed: false })
    }

    pub async fn terminal_page_result_is_known(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        run_id: [u8; 16],
        page_sequence: u64,
        fetch_command_id: [u8; 16],
    ) -> Result<bool, MailPersonsSyncPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if account_public_id.iter().all(|byte| *byte == 0)
            || run_id.iter().all(|byte| *byte == 0)
            || !(1..=4_096).contains(&page_sequence)
            || fetch_command_id.iter().all(|byte| *byte == 0)
        {
            return Err(invalid());
        }
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let known = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM makosh_data.mail_persons_sync_runs r \
             JOIN makosh_data.mail_persons_sync_outbox o USING (logical_owner_id,run_id) \
             WHERE r.logical_owner_id=$1 AND r.account_public_id=$2 AND r.run_id=$3 \
             AND r.state > 2 AND o.page_sequence=$4-1 \
             AND ((o.semantic_kind=2 AND $4=1) OR (o.semantic_kind=5 AND $4>1)) \
             AND o.message_id=$5)",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
        .bind(fetch_command_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.rollback().await.map_err(|_| storage())?;
        Ok(known)
    }

    pub async fn load_page_finalization_context(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
        page_sequence: u64,
    ) -> Result<Option<MailPersonsSyncPageFinalizationContextV1>, MailPersonsSyncPersistenceErrorV1>
    {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT p.account_public_id,p.completed_message_id,p.observed_sources,p.updated_sources, \
             p.removed_sources,p.continuation_kind, \
             p.continuation_queued,COUNT(s.persons_command_id) FILTER \
             (WHERE s.persons_result_message_id IS NULL) AS remaining, \
             COALESCE(BOOL_OR(s.outcome=2),FALSE) AS rejected \
             FROM makosh_data.mail_persons_sync_pages p LEFT JOIN \
             makosh_data.mail_persons_sync_sources s USING (logical_owner_id,run_id,page_sequence) \
             WHERE p.logical_owner_id=$1 AND p.run_id=$2 AND p.page_sequence=$3 \
             GROUP BY p.account_public_id,p.completed_message_id,p.observed_sources,p.updated_sources, \
             p.removed_sources,p.continuation_kind,p.continuation_queued",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let context = row
            .filter(|row| {
                matches!(
                    row.try_get::<Option<i16>, _>("continuation_kind")
                        .ok()
                        .flatten(),
                    Some(1 | 3)
                ) && row.try_get::<bool, _>("continuation_queued").ok() == Some(false)
                    && row.try_get::<i64, _>("remaining").ok() == Some(0)
                    && (row
                        .try_get::<Option<i16>, _>("continuation_kind")
                        .ok()
                        .flatten()
                        == Some(3)
                        || row.try_get::<bool, _>("rejected").ok() == Some(true))
            })
            .map(|row| {
                Ok(MailPersonsSyncPageFinalizationContextV1 {
                    account_public_id: bytes::<16>(&row, "account_public_id")?,
                    completion_message_id: bytes::<16>(&row, "completed_message_id")?,
                    observed_sources: row
                        .try_get::<i32, _>("observed_sources")
                        .map_err(|_| storage())?
                        .try_into()
                        .map_err(|_| state())?,
                    updated_sources: row
                        .try_get::<i32, _>("updated_sources")
                        .map_err(|_| storage())?
                        .try_into()
                        .map_err(|_| state())?,
                    removed_sources: row
                        .try_get::<i32, _>("removed_sources")
                        .map_err(|_| storage())?
                        .try_into()
                        .map_err(|_| state())?,
                    rejected: row.try_get::<bool, _>("rejected").map_err(|_| storage())?,
                })
            })
            .transpose()?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(context)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn finalize_finished_page_once(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
        page_sequence: u64,
        run_result: MailPersonsSyncEnvelopeRecordV1,
        scheduler_terminal: MailPersonsSyncEnvelopeRecordV1,
        finalized_at_unix_millis: i64,
    ) -> Result<MailPersonsSyncReplayOutcomeV1, MailPersonsSyncPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        run_result.validate()?;
        scheduler_terminal.validate()?;
        if finalized_at_unix_millis <= 0 {
            return Err(invalid());
        }
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT continuation_kind,continuation_queued,run_result_id,run_result_envelope_sha256, \
             run_result_envelope_bytes,scheduler_terminal_id,scheduler_terminal_envelope_sha256, \
             scheduler_terminal_envelope_bytes,completed_at_unix_millis \
             FROM makosh_data.mail_persons_sync_pages \
             WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .ok_or_else(state)?;
        if row
            .try_get::<bool, _>("continuation_queued")
            .map_err(|_| storage())?
        {
            let stored_run_result = MailPersonsSyncEnvelopeRecordV1 {
                message_id: bytes::<16>(&row, "run_result_id")?,
                envelope_sha256: bytes::<32>(&row, "run_result_envelope_sha256")?,
                envelope_bytes: row
                    .try_get("run_result_envelope_bytes")
                    .map_err(|_| storage())?,
            };
            let stored_scheduler_terminal = MailPersonsSyncEnvelopeRecordV1 {
                message_id: bytes::<16>(&row, "scheduler_terminal_id")?,
                envelope_sha256: bytes::<32>(&row, "scheduler_terminal_envelope_sha256")?,
                envelope_bytes: row
                    .try_get("scheduler_terminal_envelope_bytes")
                    .map_err(|_| storage())?,
            };
            stored_run_result.validate()?;
            stored_scheduler_terminal.validate()?;
            let exact =
                stored_run_result == run_result && stored_scheduler_terminal == scheduler_terminal;
            transaction.commit().await.map_err(|_| storage())?;
            return if exact {
                Ok(MailPersonsSyncReplayOutcomeV1 { replayed: true })
            } else {
                Err(MailPersonsSyncPersistenceErrorV1::CommandConflict)
            };
        }
        let continuation_kind = row
            .try_get::<Option<i16>, _>("continuation_kind")
            .map_err(|_| storage())?;
        if !matches!(continuation_kind, Some(1 | 3))
            || finalized_at_unix_millis
                < row
                    .try_get::<i64, _>("completed_at_unix_millis")
                    .map_err(|_| storage())?
        {
            return Err(state());
        }
        let remaining: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_sources WHERE logical_owner_id=$1 \
             AND run_id=$2 AND page_sequence=$3 AND persons_result_message_id IS NULL",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let rejected: bool = sqlx::query_scalar(
            "SELECT COALESCE(BOOL_OR(outcome=2),FALSE) FROM makosh_data.mail_persons_sync_sources \
             WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        if remaining != 0 || (continuation_kind == Some(1) && !rejected) {
            return Err(MailPersonsSyncPersistenceErrorV1::PageIncomplete);
        }
        sqlx::query(
            "UPDATE makosh_data.mail_persons_sync_pages SET continuation_kind=2,run_result_id=$4, \
             run_result_envelope_sha256=$5,run_result_envelope_bytes=$6,scheduler_terminal_id=$7, \
             scheduler_terminal_envelope_sha256=$8,scheduler_terminal_envelope_bytes=$9, \
             next_fetch_id=NULL,next_fetch_envelope_sha256=NULL,next_fetch_envelope_bytes=NULL,has_more=FALSE \
             WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3 AND continuation_kind IN (1,3) \
             AND continuation_queued=FALSE",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
        .bind(run_result.message_id.as_slice())
        .bind(run_result.envelope_sha256.as_slice())
        .bind(&run_result.envelope_bytes)
        .bind(scheduler_terminal.message_id.as_slice())
        .bind(scheduler_terminal.envelope_sha256.as_slice())
        .bind(&scheduler_terminal.envelope_bytes)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        finalize_page(
            &mut transaction,
            logical_owner_id,
            run_id,
            page_sequence,
            finalized_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncReplayOutcomeV1 { replayed: false })
    }

    pub async fn load_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<MailPersonsSyncOutboxRecordV1>, MailPersonsSyncPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,run_id,page_sequence,semantic_kind, \
             semantic_order_key,source_ordinal,created_at_unix_millis,published_at_unix_millis \
             FROM makosh_data.mail_persons_sync_outbox WHERE logical_owner_id=$1 \
             AND published_at_unix_millis IS NULL AND superseded_by_run_id IS NULL \
             ORDER BY run_id,semantic_order_key,message_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(MAIL_PERSONS_SYNC_OUTBOX_READ_LIMIT_V1)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let kind = semantic_kind(
                row.try_get::<i16, _>("semantic_kind")
                    .map_err(|_| storage())?,
            )?;
            let record = MailPersonsSyncEnvelopeRecordV1 {
                message_id: bytes::<16>(&row, "message_id")?,
                envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
                envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
            };
            record.validate()?;
            out.push(MailPersonsSyncOutboxRecordV1 {
                record,
                run_id: bytes::<16>(&row, "run_id")?,
                page_sequence: u64_value(&row, "page_sequence")?,
                semantic_kind: kind,
                semantic_order_key: row.try_get("semantic_order_key").map_err(|_| storage())?,
                source_ordinal: u16::try_from(
                    row.try_get::<i32, _>("source_ordinal")
                        .map_err(|_| storage())?,
                )
                .map_err(|_| state())?,
                created_at_unix_millis: row
                    .try_get("created_at_unix_millis")
                    .map_err(|_| storage())?,
                published_at_unix_millis: row
                    .try_get("published_at_unix_millis")
                    .map_err(|_| storage())?,
            });
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(out)
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<MailPersonsSyncOutboxPublishClaimV1>, MailPersonsSyncPersistenceErrorV1>
    {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,run_id,page_sequence,semantic_kind, \
             semantic_order_key,source_ordinal,created_at_unix_millis,published_at_unix_millis \
             FROM makosh_data.mail_persons_sync_outbox WHERE logical_owner_id=$1 \
             AND published_at_unix_millis IS NULL AND superseded_by_run_id IS NULL \
             ORDER BY run_id,semantic_order_key,message_id FOR UPDATE SKIP LOCKED LIMIT 1",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let Some(row) = row else {
            transaction.rollback().await.map_err(|_| storage())?;
            return Ok(None);
        };
        let record = MailPersonsSyncEnvelopeRecordV1 {
            message_id: bytes::<16>(&row, "message_id")?,
            envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
        };
        record.validate()?;
        let outbox = MailPersonsSyncOutboxRecordV1 {
            record,
            run_id: bytes::<16>(&row, "run_id")?,
            page_sequence: u64_value(&row, "page_sequence")?,
            semantic_kind: semantic_kind(
                row.try_get::<i16, _>("semantic_kind")
                    .map_err(|_| storage())?,
            )?,
            semantic_order_key: row.try_get("semantic_order_key").map_err(|_| storage())?,
            source_ordinal: u16::try_from(
                row.try_get::<i32, _>("source_ordinal")
                    .map_err(|_| storage())?,
            )
            .map_err(|_| state())?,
            created_at_unix_millis: row
                .try_get("created_at_unix_millis")
                .map_err(|_| storage())?,
            published_at_unix_millis: row
                .try_get("published_at_unix_millis")
                .map_err(|_| storage())?,
        };
        Ok(Some(MailPersonsSyncOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record: outbox,
        }))
    }

    #[cfg(feature = "conformance-test-support")]
    pub async fn mark_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        expected_envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if message_id.iter().all(|byte| *byte == 0)
            || expected_envelope_sha256.iter().all(|byte| *byte == 0)
            || published_at_unix_millis <= 0
        {
            return Err(invalid());
        }
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes,created_at_unix_millis,published_at_unix_millis, \
             superseded_by_run_id \
             FROM makosh_data.mail_persons_sync_outbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .ok_or_else(state)?;
        let record = MailPersonsSyncEnvelopeRecordV1 {
            message_id,
            envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
        };
        record.validate()?;
        let created: i64 = row
            .try_get("created_at_unix_millis")
            .map_err(|_| storage())?;
        if row
            .try_get::<Option<Vec<u8>>, _>("superseded_by_run_id")
            .map_err(|_| storage())?
            .is_some()
        {
            return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
        }
        if record.envelope_sha256 != expected_envelope_sha256 || published_at_unix_millis < created
        {
            return Err(MailPersonsSyncPersistenceErrorV1::HashMismatch);
        }
        if row
            .try_get::<Option<i64>, _>("published_at_unix_millis")
            .map_err(|_| storage())?
            .is_none()
        {
            sqlx::query(
                "UPDATE makosh_data.mail_persons_sync_outbox SET published_at_unix_millis=$3 \
                 WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 \
                 AND published_at_unix_millis IS NULL",
            )
            .bind(logical_owner_id)
            .bind(message_id.as_slice())
            .bind(published_at_unix_millis)
            .bind(expected_envelope_sha256.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(())
    }
}

async fn finalize_page(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    run_id: [u8; 16],
    page_sequence: u64,
    finalized_at: i64,
) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
    let page = sqlx::query(
        "SELECT receipt_id,receipt_envelope_sha256,receipt_envelope_bytes,staged_sources, \
         has_more,completed_at_unix_millis,continuation_kind,next_fetch_id, \
         next_fetch_envelope_sha256,next_fetch_envelope_bytes,run_result_id, \
         run_result_envelope_sha256,run_result_envelope_bytes,scheduler_terminal_id, \
         scheduler_terminal_envelope_sha256,scheduler_terminal_envelope_bytes,rejection_code,continuation_queued \
         FROM makosh_data.mail_persons_sync_pages WHERE logical_owner_id=$1 AND run_id=$2 \
         AND page_sequence=$3 AND completed_message_id IS NOT NULL FOR UPDATE",
    )
    .bind(owner)
    .bind(run_id.as_slice())
    .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| storage())?
    .ok_or_else(state)?;
    if page
        .try_get::<bool, _>("continuation_queued")
        .map_err(|_| storage())?
    {
        return Ok(());
    }
    let staged = page
        .try_get::<i32, _>("staged_sources")
        .map_err(|_| storage())?;
    let base_ordinal = u16::try_from(staged + 1).map_err(|_| state())?;
    let completed_at = page
        .try_get::<i64, _>("completed_at_unix_millis")
        .map_err(|_| storage())?;
    if finalized_at < completed_at {
        return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
    }
    let receipt = MailPersonsSyncEnvelopeRecordV1 {
        message_id: bytes::<16>(&page, "receipt_id")?,
        envelope_sha256: bytes::<32>(&page, "receipt_envelope_sha256")?,
        envelope_bytes: page
            .try_get("receipt_envelope_bytes")
            .map_err(|_| storage())?,
    };
    insert_outbox(
        transaction,
        owner,
        run_id,
        page_sequence,
        MailPersonsSyncSemanticKindV1::PageReceipt,
        None,
        base_ordinal,
        receipt,
        completed_at,
    )
    .await?;
    let has_more = page.try_get::<bool, _>("has_more").map_err(|_| storage())?;
    let continuation_kind = page
        .try_get::<i16, _>("continuation_kind")
        .map_err(|_| storage())?;
    if has_more && continuation_kind == 1 {
        let next_fetch = MailPersonsSyncEnvelopeRecordV1 {
            message_id: bytes::<16>(&page, "next_fetch_id")?,
            envelope_sha256: bytes::<32>(&page, "next_fetch_envelope_sha256")?,
            envelope_bytes: page
                .try_get("next_fetch_envelope_bytes")
                .map_err(|_| storage())?,
        };
        insert_outbox(
            transaction,
            owner,
            run_id,
            page_sequence,
            MailPersonsSyncSemanticKindV1::NextMailFetch,
            None,
            base_ordinal.checked_add(1).ok_or_else(state)?,
            next_fetch,
            completed_at,
        )
        .await?;
    } else if !has_more && continuation_kind == 2 {
        let result = MailPersonsSyncEnvelopeRecordV1 {
            message_id: bytes::<16>(&page, "run_result_id")?,
            envelope_sha256: bytes::<32>(&page, "run_result_envelope_sha256")?,
            envelope_bytes: page
                .try_get("run_result_envelope_bytes")
                .map_err(|_| storage())?,
        };
        let terminal = MailPersonsSyncEnvelopeRecordV1 {
            message_id: bytes::<16>(&page, "scheduler_terminal_id")?,
            envelope_sha256: bytes::<32>(&page, "scheduler_terminal_envelope_sha256")?,
            envelope_bytes: page
                .try_get("scheduler_terminal_envelope_bytes")
                .map_err(|_| storage())?,
        };
        insert_outbox(
            transaction,
            owner,
            run_id,
            page_sequence,
            MailPersonsSyncSemanticKindV1::RunResult,
            None,
            base_ordinal.checked_add(1).ok_or_else(state)?,
            result,
            completed_at,
        )
        .await?;
        insert_outbox(
            transaction,
            owner,
            run_id,
            page_sequence,
            MailPersonsSyncSemanticKindV1::SchedulerTerminal,
            None,
            base_ordinal.checked_add(2).ok_or_else(state)?,
            terminal,
            completed_at,
        )
        .await?;
        sqlx::query(
            "UPDATE makosh_data.mail_persons_sync_scheduler_runs SET terminal_queued=TRUE \
             WHERE logical_owner_id=$1 AND run_id=$2 AND terminal_queued=FALSE",
        )
        .bind(owner)
        .bind(run_id.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(|_| storage())?;
    } else {
        return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
    }
    let rejected: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_sources WHERE logical_owner_id=$1 \
         AND run_id=$2 AND page_sequence=$3 AND outcome=2",
    )
    .bind(owner)
    .bind(run_id.as_slice())
    .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    let page_rejection_code = page
        .try_get::<Option<i16>, _>("rejection_code")
        .map_err(|_| storage())?;
    let run_rejection_code = page_rejection_code.or_else(|| (rejected != 0).then_some(2_i16));
    let run_state = if has_more {
        2_i16
    } else if run_rejection_code.is_some() {
        4_i16
    } else {
        3_i16
    };
    sqlx::query(
        "UPDATE makosh_data.mail_persons_sync_runs SET state=$3,state_revision=state_revision+1, \
         next_page_sequence=$4,processed_pages=processed_pages+1,processed_sources=processed_sources+$5, \
         rejection_code=$6,updated_at_unix_millis=$7 WHERE logical_owner_id=$1 AND run_id=$2 \
         AND updated_at_unix_millis <= $7",
    )
    .bind(owner)
    .bind(run_id.as_slice())
    .bind(run_state)
    .bind(i64::try_from(page_sequence + u64::from(has_more)).map_err(|_| invalid())?)
    .bind(i64::from(staged))
    .bind(run_rejection_code)
    .bind(finalized_at)
    .execute(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    sqlx::query(
        "UPDATE makosh_data.mail_persons_sync_pages SET continuation_queued=TRUE \
         WHERE logical_owner_id=$1 AND run_id=$2 AND page_sequence=$3 AND continuation_queued=FALSE",
    )
    .bind(owner)
    .bind(run_id.as_slice())
    .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
    .execute(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    run_id: [u8; 16],
    page_sequence: u64,
    kind: MailPersonsSyncSemanticKindV1,
    source_id: Option<[u8; 16]>,
    ordinal: u16,
    record: MailPersonsSyncEnvelopeRecordV1,
    created_at: i64,
) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
    record.validate()?;
    let order = mail_persons_sync_semantic_order_key_v1(page_sequence, kind, source_id, ordinal)?;
    sqlx::query(
        "INSERT INTO makosh_data.mail_persons_sync_outbox \
         (logical_owner_id,message_id,envelope_sha256,envelope_bytes,run_id,page_sequence,semantic_kind, \
          semantic_order_key,source_ordinal,created_at_unix_millis,published_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,NULL)",
    )
    .bind(owner)
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(record.envelope_bytes)
    .bind(run_id.as_slice())
    .bind(i64::try_from(page_sequence).map_err(|_| invalid())?)
    .bind(kind as i16)
    .bind(order)
    .bind(i32::from(ordinal))
    .bind(created_at)
    .execute(&mut **transaction)
    .await
    .map_err(|_| MailPersonsSyncPersistenceErrorV1::CommandConflict)?;
    Ok(())
}

async fn set_owner(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
    validate_owner(owner)?;
    sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
        .bind(owner)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(|_| storage())
}

fn validate_owner(value: &str) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
    if !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Ok(())
    } else {
        Err(invalid())
    }
}

fn bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<[u8; N], MailPersonsSyncPersistenceErrorV1> {
    row.try_get::<Vec<u8>, _>(name)
        .map_err(|_| storage())?
        .try_into()
        .map_err(|_| state())
}

fn optional_bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<Option<[u8; N]>, MailPersonsSyncPersistenceErrorV1> {
    row.try_get::<Option<Vec<u8>>, _>(name)
        .map_err(|_| storage())?
        .map(|value| value.try_into().map_err(|_| state()))
        .transpose()
}

fn u64_value(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<u64, MailPersonsSyncPersistenceErrorV1> {
    row.try_get::<i64, _>(name)
        .map_err(|_| storage())?
        .try_into()
        .map_err(|_| state())
}

fn semantic_kind(
    value: i16,
) -> Result<MailPersonsSyncSemanticKindV1, MailPersonsSyncPersistenceErrorV1> {
    match value {
        1 => Ok(MailPersonsSyncSemanticKindV1::SchedulerAcceptance),
        2 => Ok(MailPersonsSyncSemanticKindV1::MailFetch),
        3 => Ok(MailPersonsSyncSemanticKindV1::PersonsCommand),
        4 => Ok(MailPersonsSyncSemanticKindV1::PageReceipt),
        5 => Ok(MailPersonsSyncSemanticKindV1::NextMailFetch),
        6 => Ok(MailPersonsSyncSemanticKindV1::RunResult),
        7 => Ok(MailPersonsSyncSemanticKindV1::SchedulerTerminal),
        _ => Err(state()),
    }
}

const fn invalid() -> MailPersonsSyncPersistenceErrorV1 {
    MailPersonsSyncPersistenceErrorV1::InvalidInput
}
const fn storage() -> MailPersonsSyncPersistenceErrorV1 {
    MailPersonsSyncPersistenceErrorV1::StorageUnavailable
}
const fn state() -> MailPersonsSyncPersistenceErrorV1 {
    MailPersonsSyncPersistenceErrorV1::StateConflict
}

fn begin_run_insert_error(error: sqlx::Error) -> MailPersonsSyncPersistenceErrorV1 {
    if error
        .as_database_error()
        .and_then(|database| database.constraint())
        == Some("mail_persons_sync_one_active_account_run")
    {
        MailPersonsSyncPersistenceErrorV1::AccountBusy
    } else {
        MailPersonsSyncPersistenceErrorV1::CommandConflict
    }
}
