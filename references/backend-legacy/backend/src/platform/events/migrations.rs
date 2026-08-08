use sqlx::postgres::PgPool;

use makosh_events_postgres::errors::EventStoreError;

pub async fn run_migrations(pool: &PgPool) -> Result<(), EventStoreError> {
    makosh_schema::apply(pool).await?;
    Ok(())
}

pub fn expected_migration_summary() -> MigrationSummary {
    let mut count = 0;
    let mut latest_version = 0;

    for migration in makosh_schema::MIGRATOR.iter() {
        count += 1;
        latest_version = latest_version.max(migration.version);
    }

    MigrationSummary {
        count,
        latest_version,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationSummary {
    pub count: i64,
    pub latest_version: i64,
}
