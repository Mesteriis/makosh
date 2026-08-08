use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobContractBindingV1, JobKindV1, OpaqueScheduleScopeV1, ScheduleIdV1,
    SchedulePolicyV1, ScheduleRevisionV1, ScheduleSpecV1,
};
use sqlx::{Row, postgres::PgRow};

use super::request::{
    SchedulerDueScheduleV1, SchedulerScheduleStoreErrorV1, SchedulerScheduleUpsertV1,
};

pub(crate) struct PersistedScheduleRowV1 {
    schedule_id: Vec<u8>,
    revision: i64,
    job_owner: String,
    job_name: String,
    job_major: i32,
    contract_name: String,
    contract_revision: Option<i32>,
    contract_schema_sha256: Vec<u8>,
    scope_id: String,
    concurrency_key: String,
    max_parallelism: i32,
    enabled: bool,
    policy_bytes: Vec<u8>,
    next_due_at: i64,
}

impl PersistedScheduleRowV1 {
    pub(crate) fn from_row(row: PgRow) -> Result<Self, SchedulerScheduleStoreErrorV1> {
        Ok(Self {
            schedule_id: field(&row, "schedule_id")?,
            revision: field(&row, "schedule_revision")?,
            job_owner: field(&row, "job_owner")?,
            job_name: field(&row, "job_name")?,
            job_major: field(&row, "job_major")?,
            contract_name: field(&row, "contract_name")?,
            contract_revision: field(&row, "contract_revision")?,
            contract_schema_sha256: field(&row, "contract_schema_sha256")?,
            scope_id: field(&row, "scope_id")?,
            concurrency_key: field(&row, "concurrency_key")?,
            max_parallelism: field(&row, "max_parallelism")?,
            enabled: field(&row, "enabled")?,
            policy_bytes: field(&row, "policy_bytes")?,
            next_due_at: field(&row, "next_due_at_unix_ms")?,
        })
    }

    pub(super) fn matches(&self, change: &SchedulerScheduleUpsertV1) -> bool {
        let spec = change.spec();
        i64::try_from(spec.revision().value()).is_ok_and(|revision| self.revision == revision)
            && self.schedule_id == spec.schedule_id().bytes()
            && self.job_owner == spec.binding().job_kind().owner()
            && self.job_name == spec.binding().job_kind().name()
            && self.job_major == i32::from(spec.binding().job_kind().major())
            && self.contract_name == spec.binding().contract_name()
            && i32::try_from(spec.binding().contract_revision()).ok() == self.contract_revision
            && self.contract_schema_sha256 == spec.binding().schema_sha256()
            && self.scope_id == spec.scope().value()
            && self.concurrency_key == spec.concurrency_key().value()
            && self.max_parallelism == i32::from(spec.policy().max_parallelism())
            && self.enabled == spec.enabled()
            && self.policy_bytes == spec.policy().canonical_bytes()
            && self.next_due_at == change.next_due_at().value()
    }

    pub(super) const fn revision(&self) -> i64 {
        self.revision
    }

    pub(crate) fn into_due(self) -> Result<SchedulerDueScheduleV1, SchedulerScheduleStoreErrorV1> {
        let schedule_id = ScheduleIdV1::new(fixed(self.schedule_id)?)
            .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?;
        let revision = ScheduleRevisionV1::new(
            u64::try_from(self.revision)
                .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?,
        )
        .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?;
        let job_kind = JobKindV1::new(
            self.job_owner,
            self.job_name,
            u16::try_from(self.job_major)
                .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?,
        )
        .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?;
        let binding = JobContractBindingV1::new(
            job_kind,
            self.contract_name,
            u32::try_from(
                self.contract_revision
                    .ok_or(SchedulerScheduleStoreErrorV1::CorruptState)?,
            )
            .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?,
            fixed(self.contract_schema_sha256)?,
        )
        .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?;
        let scope = OpaqueScheduleScopeV1::new(self.scope_id)
            .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?;
        let key = ConcurrencyKeyV1::new(self.concurrency_key)
            .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?;
        let policy = SchedulePolicyV1::from_canonical_bytes(&self.policy_bytes)
            .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?;
        if self.max_parallelism != i32::from(policy.max_parallelism()) {
            return Err(SchedulerScheduleStoreErrorV1::CorruptState);
        }
        Ok(SchedulerDueScheduleV1::new(
            ScheduleSpecV1::new(
                schedule_id,
                revision,
                binding,
                scope,
                key,
                self.enabled,
                policy,
            ),
            UtcMillisV1::new(self.next_due_at),
        ))
    }
}

fn field<T>(row: &PgRow, name: &str) -> Result<T, SchedulerScheduleStoreErrorV1>
where
    T: for<'r> sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    row.try_get(name)
        .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], SchedulerScheduleStoreErrorV1> {
    value
        .try_into()
        .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)
}
