use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub struct WhatsAppDurablePersistence {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppDurablePersistenceError {
    Database,
    InvalidRow,
    ObservationConflict,
    CommandConflict,
    ConflictingDeliveryRouteLocator,
    ConflictingDeliveryIntentInbox,
    InvalidDeliveryIntentTransition,
    OwnerScopeConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppHostObservationRecordV1 {
    pub account_id: String,
    pub provider_event_id: String,
    pub evidence_kind: i16,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppProviderCommandStateV1 {
    Pending = 1,
    Claimed = 2,
    Succeeded = 3,
    Failed = 4,
}

impl TryFrom<i16> for WhatsAppProviderCommandStateV1 {
    type Error = WhatsAppDurablePersistenceError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Pending),
            2 => Ok(Self::Claimed),
            3 => Ok(Self::Succeeded),
            4 => Ok(Self::Failed),
            _ => Err(WhatsAppDurablePersistenceError::InvalidRow),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppClaimedCommandV1 {
    pub operation_id: String,
    pub account_id: String,
    pub exact_command_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppProviderCommandEnqueueV1 {
    Inserted,
    Existing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppProviderCommandStatusV1 {
    pub operation_id: String,
    pub account_id: String,
    pub state: WhatsAppProviderCommandStateV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
}

impl WhatsAppDurablePersistence {
    #[must_use]
    pub fn delivery_intent_store(&self) -> crate::WhatsAppDeliveryIntentStoreV1 {
        crate::WhatsAppDeliveryIntentStoreV1::new(self.pool.clone())
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, WhatsAppDurablePersistenceError> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        let options = PgConnectOptions::new()
            .statement_cache_capacity(0)
            .host(pgbouncer_host)
            .port(port)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        Ok(Self { pool })
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Claims or verifies the one logical human owner before any WhatsApp
    /// state access. The scope follows the stable fenced Storage-principal
    /// prefix across role epochs and rejects every other registration.
    pub async fn bind_owner_scope(
        &self,
        logical_owner_id: &str,
    ) -> Result<(), WhatsAppDurablePersistenceError> {
        if !valid_logical_owner_id(logical_owner_id) {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        sqlx::query(
            "INSERT INTO makosh_data.whatsapp_owner_scope
                (singleton, logical_owner_id, runtime_principal_prefix)
             SELECT TRUE, $1, regexp_replace(current_user::text, '_[0-9]+$', '')
             WHERE current_user::text ~ '^storage_[a-f0-9]{16}_[1-9][0-9]*$'
             ON CONFLICT (singleton) DO NOTHING",
        )
        .bind(logical_owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        let exact = sqlx::query(
            "SELECT logical_owner_id
             FROM makosh_data.whatsapp_owner_scope
             WHERE singleton = TRUE
               AND logical_owner_id = $1
               AND runtime_principal_prefix =
                   regexp_replace(current_user::text, '_[0-9]+$', '')",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        if exact.is_none() {
            return Err(WhatsAppDurablePersistenceError::OwnerScopeConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn initialize(&self) -> Result<(), WhatsAppDurablePersistenceError> {
        sqlx::raw_sql(crate::WHATSAPP_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        sqlx::raw_sql(crate::WHATSAPP_SCHEMA_V2)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn enqueue_communications_outbox(
        &self,
        record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<(), WhatsAppDurablePersistenceError> {
        sqlx::query("INSERT INTO makosh_data.whatsapp_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(created_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn enqueue_provider_command(
        &self,
        operation_id: &str,
        account_id: &str,
        exact_command_bytes: &[u8],
        requested_at_unix_seconds: i64,
    ) -> Result<WhatsAppProviderCommandEnqueueV1, WhatsAppDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || account_id.trim().is_empty()
            || exact_command_bytes.is_empty()
            || exact_command_bytes.len() > 512 * 1024
            || requested_at_unix_seconds <= 0
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let inserted = sqlx::query("INSERT INTO makosh_data.whatsapp_provider_commands (operation_id, account_id, exact_command_bytes, state, requested_at_unix_seconds) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (operation_id) DO NOTHING")
            .bind(operation_id)
            .bind(account_id)
            .bind(exact_command_bytes)
            .bind(WhatsAppProviderCommandStateV1::Pending as i16)
            .bind(requested_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        if inserted.rows_affected() == 1 {
            return Ok(WhatsAppProviderCommandEnqueueV1::Inserted);
        }
        let existing = sqlx::query(
            "SELECT account_id, exact_command_bytes FROM makosh_data.whatsapp_provider_commands WHERE operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?
        .ok_or(WhatsAppDurablePersistenceError::CommandConflict)?;
        let existing_account_id: String = existing
            .try_get("account_id")
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        let existing_bytes: Vec<u8> = existing
            .try_get("exact_command_bytes")
            .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
        if existing_account_id != account_id || existing_bytes != exact_command_bytes {
            return Err(WhatsAppDurablePersistenceError::CommandConflict);
        }
        Ok(WhatsAppProviderCommandEnqueueV1::Existing)
    }

    pub async fn provider_command_status(
        &self,
        operation_id: &str,
    ) -> Result<Option<WhatsAppProviderCommandStatusV1>, WhatsAppDurablePersistenceError> {
        if operation_id.trim().is_empty() {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "SELECT operation_id, account_id, state, requested_at_unix_seconds, completed_at_unix_seconds FROM makosh_data.whatsapp_provider_commands WHERE operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        row.map(|row| {
            Ok(WhatsAppProviderCommandStatusV1 {
                operation_id: row
                    .try_get("operation_id")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
                account_id: row
                    .try_get("account_id")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
                state: WhatsAppProviderCommandStateV1::try_from(
                    row.try_get::<i16, _>("state")
                        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
                )?,
                requested_at_unix_seconds: row
                    .try_get("requested_at_unix_seconds")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
                completed_at_unix_seconds: row
                    .try_get("completed_at_unix_seconds")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
            })
        })
        .transpose()
    }

    pub async fn claim_provider_commands(
        &self,
        account_id: &str,
        host_claim_id: &str,
        now_unix_seconds: i64,
        lease_seconds: i64,
        limit: i64,
    ) -> Result<Vec<WhatsAppClaimedCommandV1>, WhatsAppDurablePersistenceError> {
        if account_id.trim().is_empty()
            || host_claim_id.trim().is_empty()
            || now_unix_seconds <= 0
            || !(1..=300).contains(&lease_seconds)
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        let lease_expires_at_unix_seconds = now_unix_seconds
            .checked_add(lease_seconds)
            .ok_or(WhatsAppDurablePersistenceError::InvalidRow)?;
        let rows = sqlx::query(
            "WITH candidates AS (SELECT operation_id FROM makosh_data.whatsapp_provider_commands WHERE account_id = $1 AND (state = $2 OR (state = $3 AND lease_expires_at_unix_seconds < $4)) ORDER BY requested_at_unix_seconds ASC, operation_id ASC LIMIT $5 FOR UPDATE SKIP LOCKED) UPDATE makosh_data.whatsapp_provider_commands AS command SET state = $3, host_claim_id = $6, lease_expires_at_unix_seconds = $7 FROM candidates WHERE command.operation_id = candidates.operation_id RETURNING command.operation_id, command.account_id, command.exact_command_bytes",
        )
        .bind(account_id)
        .bind(WhatsAppProviderCommandStateV1::Pending as i16)
        .bind(WhatsAppProviderCommandStateV1::Claimed as i16)
        .bind(now_unix_seconds)
        .bind(limit.clamp(1, 64))
        .bind(host_claim_id)
        .bind(lease_expires_at_unix_seconds)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                Ok(WhatsAppClaimedCommandV1 {
                    operation_id: row
                        .try_get("operation_id")
                        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
                    account_id: row
                        .try_get("account_id")
                        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
                    exact_command_bytes: row
                        .try_get("exact_command_bytes")
                        .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?,
                })
            })
            .collect()
    }

    pub async fn complete_provider_command(
        &self,
        operation_id: &str,
        account_id: &str,
        host_claim_id: &str,
        succeeded: bool,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || account_id.trim().is_empty()
            || host_claim_id.trim().is_empty()
            || completed_at_unix_seconds <= 0
        {
            return Err(WhatsAppDurablePersistenceError::InvalidRow);
        }
        sqlx::query("UPDATE makosh_data.whatsapp_provider_commands SET state = $4, completed_at_unix_seconds = $5 WHERE operation_id = $1 AND account_id = $2 AND host_claim_id = $3 AND state = $6 AND lease_expires_at_unix_seconds >= $5")
            .bind(operation_id)
            .bind(account_id)
            .bind(host_claim_id)
            .bind(if succeeded { WhatsAppProviderCommandStateV1::Succeeded as i16 } else { WhatsAppProviderCommandStateV1::Failed as i16 })
            .bind(completed_at_unix_seconds)
            .bind(WhatsAppProviderCommandStateV1::Claimed as i16)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }

    pub async fn record_host_observation_and_enqueue(
        &self,
        observation: &WhatsAppHostObservationRecordV1,
        record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        self.record_host_observation_projection_and_enqueue(
            observation,
            None,
            Some(record),
            None,
            created_at_unix_seconds,
        )
        .await
    }

    pub async fn pending_communications_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, WhatsAppDurablePersistenceError> {
        let rows = sqlx::query("SELECT exact_envelope_bytes FROM makosh_data.whatsapp_communications_outbox WHERE published_at_unix_seconds IS NULL ORDER BY created_at_unix_seconds ASC, message_id ASC LIMIT $1")
            .bind(limit.clamp(1, 256))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| WhatsAppDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)?;
                OutboxRecordV1::accept(bytes)
                    .map_err(|_| WhatsAppDurablePersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_communications_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDurablePersistenceError> {
        sqlx::query("UPDATE makosh_data.whatsapp_communications_outbox SET published_at_unix_seconds = $2 WHERE message_id = $1 AND published_at_unix_seconds IS NULL")
            .bind(message_id.as_slice())
            .bind(published_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| WhatsAppDurablePersistenceError::Database)
    }
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| match byte {
            b'a'..=b'z' | b'0'..=b'9' => true,
            b'_' | b'-' => index > 0,
            _ => false,
        })
}
