use chrono::{DateTime, Duration, Utc};
use makosh_communications_api::commands::{
    ProviderCommandQueuePortError, ProviderRuntimeLease as LeaseContract,
    ProviderRuntimeLeaseFuture, ProviderRuntimeLeasePort,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct ProviderRuntimeLeaseStore {
    pool: PgPool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRuntimeLease {
    pub provider: String,
    pub account_id: String,
    pub topology: String,
    pub holder: String,
    pub epoch: i64,
    pub state: String,
    pub expires_at: DateTime<Utc>,
}

impl ProviderRuntimeLeaseStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Acquire is the fencing point: every acquire, including topology rollback,
    /// increments epoch and invalidates all older completions.
    pub async fn acquire(
        &self,
        provider: &str,
        account_id: &str,
        topology: &str,
        holder: &str,
        ttl: Duration,
    ) -> Result<ProviderRuntimeLease, sqlx::Error> {
        let expires_at = Utc::now() + ttl;
        sqlx::query_as::<_, ProviderRuntimeLeaseRow>(
            r#"
            INSERT INTO provider_runtime_leases
                (provider, account_id, topology, holder, epoch, state, expires_at)
            VALUES ($1, $2, $3, $4, 1, 'active', $5)
            ON CONFLICT (provider, account_id) DO UPDATE SET
                topology = EXCLUDED.topology,
                holder = EXCLUDED.holder,
                epoch = provider_runtime_leases.epoch + 1,
                state = 'active',
                expires_at = EXCLUDED.expires_at,
                updated_at = now()
            RETURNING provider, account_id, topology, holder, epoch, state, expires_at
            "#,
        )
        .bind(provider.trim())
        .bind(account_id.trim())
        .bind(topology.trim())
        .bind(holder.trim())
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map(ProviderRuntimeLease::from)
    }

    pub async fn revoke(
        &self,
        provider: &str,
        account_id: &str,
        holder: &str,
    ) -> Result<Option<ProviderRuntimeLease>, sqlx::Error> {
        sqlx::query_as::<_, ProviderRuntimeLeaseRow>(
            r#"
            UPDATE provider_runtime_leases
            SET state = 'revoked', epoch = epoch + 1, updated_at = now()
            WHERE provider = $1 AND account_id = $2 AND holder = $3
            RETURNING provider, account_id, topology, holder, epoch, state, expires_at
            "#,
        )
        .bind(provider.trim())
        .bind(account_id.trim())
        .bind(holder.trim())
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(ProviderRuntimeLease::from))
    }
}

#[derive(sqlx::FromRow)]
struct ProviderRuntimeLeaseRow {
    provider: String,
    account_id: String,
    topology: String,
    holder: String,
    epoch: i64,
    state: String,
    expires_at: DateTime<Utc>,
}

impl From<ProviderRuntimeLeaseRow> for ProviderRuntimeLease {
    fn from(row: ProviderRuntimeLeaseRow) -> Self {
        Self {
            provider: row.provider,
            account_id: row.account_id,
            topology: row.topology,
            holder: row.holder,
            epoch: row.epoch,
            state: row.state,
            expires_at: row.expires_at,
        }
    }
}

impl ProviderRuntimeLeasePort for ProviderRuntimeLeaseStore {
    fn acquire<'a>(
        &'a self,
        provider: &'a str,
        account_id: &'a str,
        topology: &'a str,
        holder: &'a str,
        ttl: Duration,
    ) -> ProviderRuntimeLeaseFuture<'a, LeaseContract> {
        Box::pin(async move {
            self.acquire(provider, account_id, topology, holder, ttl)
                .await
                .map(|lease| LeaseContract {
                    provider: lease.provider,
                    account_id: lease.account_id,
                    topology: lease.topology,
                    holder: lease.holder,
                    epoch: lease.epoch,
                    state: lease.state,
                    expires_at: lease.expires_at,
                })
                .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn revoke<'a>(
        &'a self,
        provider: &'a str,
        account_id: &'a str,
        holder: &'a str,
    ) -> ProviderRuntimeLeaseFuture<'a, Option<LeaseContract>> {
        Box::pin(async move {
            self.revoke(provider, account_id, holder)
                .await
                .map(|lease| {
                    lease.map(|lease| LeaseContract {
                        provider: lease.provider,
                        account_id: lease.account_id,
                        topology: lease.topology,
                        holder: lease.holder,
                        epoch: lease.epoch,
                        state: lease.state,
                        expires_at: lease.expires_at,
                    })
                })
                .map_err(ProviderCommandQueuePortError::new)
        })
    }
}
