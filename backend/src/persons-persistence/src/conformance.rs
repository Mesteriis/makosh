//! Explicit disposable PostgreSQL boundary for Persons persistence conformance only.

use std::str::FromStr;

use sqlx::{
    Executor, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    PersonsEnvelopeRecordV1, PersonsPersistenceErrorV1, PersonsPersistenceV1,
    persons_storage_bundle_v1,
};

pub struct PersonsPersistenceConformanceV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsRlsEvidenceV1 {
    pub visible_owners: Vec<String>,
    pub cross_owner_updates: u64,
    pub cross_owner_deletes: u64,
    pub cross_owner_insert_blocked: bool,
    pub own_profile_update_blocked: bool,
    pub own_profile_delete_blocked: bool,
}

impl PersonsPersistenceConformanceV1 {
    pub async fn connect_url(
        database_url: &str,
    ) -> Result<PersonsPersistenceV1, PersonsPersistenceErrorV1> {
        if database_url.trim().is_empty() {
            return Err(PersonsPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .map_err(|_| PersonsPersistenceErrorV1::StorageUnavailable)?;
        Ok(PersonsPersistenceV1::new(pool))
    }

    pub async fn install_initial_schema(
        persistence: &PersonsPersistenceV1,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        reset_schema(persistence).await?;
        apply_step(persistence, 0).await
    }

    pub async fn upgrade_to_current(
        persistence: &PersonsPersistenceV1,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        for index in 1..persons_storage_bundle_v1().steps.len() {
            apply_step(persistence, index).await?;
        }
        Ok(())
    }

    pub async fn upgrade_to_durable_v2(
        persistence: &PersonsPersistenceV1,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        apply_step(persistence, 1).await
    }

    pub async fn upgrade_outbox_order_v3(
        persistence: &PersonsPersistenceV1,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        apply_step(persistence, 2).await
    }

    pub async fn seed_legacy_v2_pending_outbox(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
        record: &PersonsEnvelopeRecordV1,
        created_at_unix_millis: i64,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        if logical_owner_id.is_empty() || created_at_unix_millis <= 0 {
            return Err(PersonsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query(
            "INSERT INTO makosh_data.persons_outbox
             (logical_owner_id,message_id,envelope_sha256,envelope_bytes,created_at_unix_millis)
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(logical_owner_id)
        .bind(record.message_id.as_slice())
        .bind(record.envelope_sha256.as_slice())
        .bind(&record.envelope_bytes)
        .bind(created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.commit().await.map_err(|_| storage())
    }

    pub async fn install_schema(
        persistence: &PersonsPersistenceV1,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        reset_schema(persistence).await?;
        for index in 0..persons_storage_bundle_v1().steps.len() {
            apply_step(persistence, index).await?;
        }
        Ok(())
    }

    pub async fn seed_initial_source_fixture(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query("INSERT INTO makosh_data.persons_owner_aggregates VALUES ($1,0,1800000000,0)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query("INSERT INTO makosh_data.persons_current (logical_owner_id, person_id, lifecycle, person_revision, current_profile_revision, merged_into_person_id, created_at_unix_seconds, created_at_nanos, updated_at_unix_seconds, updated_at_nanos) VALUES ($1,$2,1,1,NULL,NULL,1800000001,0,1800000001,0)")
            .bind(logical_owner_id)
            .bind([21_u8; 16].as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query("INSERT INTO makosh_data.persons_sources (logical_owner_id,integration_public_id,account_public_id,provider_source_contact_public_id,person_id,removed,display_name,normalized_emails,normalized_phones,source_revision,source_digest,observed_at_unix_seconds,observed_at_nanos) VALUES ($1,$2,$3,$4,$5,FALSE,'Initial',ARRAY['initial@example.test'],ARRAY[]::TEXT[],1,$6,1800000001,0)")
            .bind(logical_owner_id)
            .bind([1_u8; 16].as_slice())
            .bind([2_u8; 16].as_slice())
            .bind([3_u8; 16].as_slice())
            .bind([21_u8; 16].as_slice())
            .bind([7_u8; 32].as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        transaction.commit().await.map_err(|_| storage())
    }

    pub async fn profile_history_count(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
        person_id: [u8; 16],
    ) -> Result<i64, PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.persons_profiles WHERE logical_owner_id = $1 AND person_id = $2",
        )
        .bind(logical_owner_id)
        .bind(person_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.rollback().await.map_err(|_| storage())?;
        Ok(count)
    }

    pub async fn durable_command_outbox_counts(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
    ) -> Result<(i64, i64), PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        let inbox = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.persons_command_inbox WHERE logical_owner_id = $1",
        )
        .bind(logical_owner_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let outbox = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.persons_outbox WHERE logical_owner_id = $1",
        )
        .bind(logical_owner_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.rollback().await.map_err(|_| storage())?;
        Ok((inbox, outbox))
    }

    pub async fn corrupt_terminal_bytes(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
        command_message_id: [u8; 16],
    ) -> Result<(), PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query("UPDATE makosh_data.persons_command_inbox SET terminal_envelope_bytes = 'corrupt' WHERE logical_owner_id = $1 AND command_message_id = $2")
            .bind(logical_owner_id)
            .bind(command_message_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        transaction.commit().await.map_err(|_| storage())
    }

    pub async fn corrupt_outbox_bytes(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
        message_id: [u8; 16],
    ) -> Result<(), PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query("UPDATE makosh_data.persons_outbox SET envelope_bytes = 'corrupt' WHERE logical_owner_id = $1 AND message_id = $2")
            .bind(logical_owner_id)
            .bind(message_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        transaction.commit().await.map_err(|_| storage())
    }

    pub async fn outbox_published_at(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
        message_id: [u8; 16],
    ) -> Result<Option<i64>, PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        let published_at = sqlx::query_scalar(
            "SELECT published_at_unix_millis FROM makosh_data.persons_outbox WHERE logical_owner_id = $1 AND message_id = $2",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.rollback().await.map_err(|_| storage())?;
        Ok(published_at)
    }

    pub async fn invalid_merged_target_is_blocked(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
    ) -> Result<bool, PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        let person_id: Vec<u8> = sqlx::query_scalar(
            "SELECT person_id FROM makosh_data.persons_current WHERE logical_owner_id = $1 ORDER BY person_id LIMIT 1",
        )
        .bind(logical_owner_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let result = sqlx::query("UPDATE makosh_data.persons_current SET lifecycle = 3, merged_into_person_id = $3 WHERE logical_owner_id = $1 AND person_id = $2")
            .bind(logical_owner_id)
            .bind(person_id)
            .bind([250_u8; 16].as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        if result.rows_affected() != 1 {
            return Err(PersonsPersistenceErrorV1::StateConflict);
        }
        Ok(transaction.commit().await.is_err())
    }

    pub async fn corrupt_lineage_receipt_linkage(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        let result = sqlx::query("UPDATE makosh_data.persons_lineage SET review_id = $2 WHERE logical_owner_id = $1 AND lineage_sequence = (SELECT MIN(lineage_sequence) FROM makosh_data.persons_lineage WHERE logical_owner_id = $1)")
            .bind(logical_owner_id)
            .bind([249_u8; 16].as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        if result.rows_affected() != 1 {
            return Err(PersonsPersistenceErrorV1::StateConflict);
        }
        transaction.commit().await.map_err(|_| storage())
    }

    pub async fn corrupt_profile_normalization(
        persistence: &PersonsPersistenceV1,
        logical_owner_id: &str,
        person_id: [u8; 16],
    ) -> Result<(), PersonsPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query(
            "ALTER TABLE makosh_data.persons_profiles DISABLE TRIGGER persons_profiles_immutable",
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        sqlx::query("UPDATE makosh_data.persons_profiles SET normalized_emails = ARRAY['z@example.test','a@example.test'] WHERE logical_owner_id = $1 AND person_id = $2")
            .bind(logical_owner_id)
            .bind(person_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query(
            "ALTER TABLE makosh_data.persons_profiles ENABLE TRIGGER persons_profiles_immutable",
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.commit().await.map_err(|_| storage())
    }

    pub async fn prove_force_rls(
        persistence: &PersonsPersistenceV1,
        visible_owner: &str,
        hidden_owner: &str,
    ) -> Result<PersonsRlsEvidenceV1, PersonsPersistenceErrorV1> {
        sqlx::raw_sql(sqlx::AssertSqlSafe("DROP ROLE IF EXISTS makosh_persons_rls_test; CREATE ROLE makosh_persons_rls_test NOSUPERUSER NOBYPASSRLS NOLOGIN; GRANT USAGE ON SCHEMA makosh_data TO makosh_persons_rls_test; GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA makosh_data TO makosh_persons_rls_test;".to_owned()))
            .execute(persistence.pool()).await.map_err(|_| storage())?;
        let own_profile_update_blocked = profile_history_mutation_blocked(
            persistence,
            hidden_owner,
            "UPDATE makosh_data.persons_profiles SET display_name = display_name WHERE logical_owner_id = $1",
        )
        .await?;
        let own_profile_delete_blocked = profile_history_mutation_blocked(
            persistence,
            hidden_owner,
            "DELETE FROM makosh_data.persons_profiles WHERE logical_owner_id = $1",
        )
        .await?;
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SET LOCAL ROLE makosh_persons_rls_test")
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(visible_owner)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        // This intentionally omits an owner predicate; FORCE RLS must hide the other owner.
        let rows = sqlx::query("SELECT logical_owner_id FROM makosh_data.persons_owner_aggregates ORDER BY logical_owner_id")
            .fetch_all(&mut *transaction).await.map_err(|_| storage())?;
        let visible_owners = rows.iter().map(|row| row.get(0)).collect();
        let cross_owner_updates = sqlx::query("UPDATE makosh_data.persons_owner_aggregates SET aggregate_revision = aggregate_revision WHERE logical_owner_id = $1")
            .bind(hidden_owner).execute(&mut *transaction).await.map_err(|_| storage())?.rows_affected();
        let cross_owner_deletes = sqlx::query(
            "DELETE FROM makosh_data.persons_owner_aggregates WHERE logical_owner_id = $1",
        )
        .bind(hidden_owner)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .rows_affected();
        let cross_owner_insert_blocked = sqlx::query(
            "INSERT INTO makosh_data.persons_owner_aggregates VALUES ($1,0,1800000200,0)",
        )
        .bind(hidden_owner)
        .execute(&mut *transaction)
        .await
        .is_err();
        transaction.rollback().await.map_err(|_| storage())?;
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            "DROP OWNED BY makosh_persons_rls_test; DROP ROLE makosh_persons_rls_test;".to_owned(),
        ))
        .execute(persistence.pool())
        .await
        .map_err(|_| storage())?;
        Ok(PersonsRlsEvidenceV1 {
            visible_owners,
            cross_owner_updates,
            cross_owner_deletes,
            cross_owner_insert_blocked,
            own_profile_update_blocked,
            own_profile_delete_blocked,
        })
    }
}

async fn profile_history_mutation_blocked(
    persistence: &PersonsPersistenceV1,
    logical_owner_id: &str,
    statement: &'static str,
) -> Result<bool, PersonsPersistenceErrorV1> {
    let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
    sqlx::query("SET LOCAL ROLE makosh_persons_rls_test")
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
    sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
        .bind(logical_owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
    let blocked = sqlx::query(statement)
        .bind(logical_owner_id)
        .execute(&mut *transaction)
        .await
        .is_err();
    transaction.rollback().await.map_err(|_| storage())?;
    Ok(blocked)
}

async fn reset_schema(persistence: &PersonsPersistenceV1) -> Result<(), PersonsPersistenceErrorV1> {
    let expected_database = std::env::var("MAKOSH_PERSONS_DISPOSABLE_DATABASE")
        .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
    let expected_sentinel = std::env::var("MAKOSH_PERSONS_DISPOSABLE_SENTINEL")
        .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
    if !expected_database.starts_with("makosh_persons_conformance_")
        || !expected_database
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        || expected_sentinel.len() != 64
        || !expected_sentinel
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(PersonsPersistenceErrorV1::InvalidInput);
    }
    let current_database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(persistence.pool())
        .await
        .map_err(|_| storage())?;
    let sentinel: String = sqlx::query_scalar(
        "SELECT token FROM public.makosh_persons_disposable_sentinel WHERE sentinel_id = 1",
    )
    .fetch_one(persistence.pool())
    .await
    .map_err(|_| storage())?;
    if current_database != expected_database || sentinel != expected_sentinel {
        return Err(PersonsPersistenceErrorV1::InvalidInput);
    }
    persistence
        .pool()
        .execute("DROP SCHEMA IF EXISTS makosh_data CASCADE")
        .await
        .map(|_| ())
        .map_err(|_| PersonsPersistenceErrorV1::StorageUnavailable)?;
    persistence
        .pool()
        .execute("CREATE SCHEMA makosh_data")
        .await
        .map(|_| ())
        .map_err(|_| PersonsPersistenceErrorV1::StorageUnavailable)
}

fn storage() -> PersonsPersistenceErrorV1 {
    PersonsPersistenceErrorV1::StorageUnavailable
}

async fn apply_step(
    persistence: &PersonsPersistenceV1,
    index: usize,
) -> Result<(), PersonsPersistenceErrorV1> {
    let bundle = persons_storage_bundle_v1();
    let step = bundle
        .steps
        .get(index)
        .ok_or(PersonsPersistenceErrorV1::InvalidInput)?;
    let sql = std::str::from_utf8(&step.forward_sql_utf8)
        .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
        .execute(persistence.pool())
        .await
        .map(|_| ())
        .map_err(|_| PersonsPersistenceErrorV1::StorageUnavailable)
}
