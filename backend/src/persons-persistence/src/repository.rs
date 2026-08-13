use std::collections::{BTreeMap, BTreeSet};

use makosh_persons_core::{
    ConfirmedActionOutcomeV1, ConfirmedActionStatusV1, DecisionProvenanceV1, DecisionReceiptV1,
    DigestV1, LineageChangeKindV1, LineageRecordV1, OwnerProfileV1, PersonIdV1, PersonLifecycleV1,
    PersonRevisionV1, PersonV1, PersonsOwnerSnapshotV1, PersonsStateV1, PublicIdV1,
    RemovedSourceV1, SourceClaimsV1, SourceLinkKeyV1, SourceLinkV1, SourceProvenanceV1,
    SplitProfileFactKindV1, TimestampV1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    ApplyPersonsCommandOutcomeV1, ApplyPersonsCommandV1, LoadedPersonsOwnerV1,
    PERSONS_MAX_ENVELOPE_BYTES_V1, PERSONS_OUTBOX_READ_LIMIT_V1, PERSONS_RECOVERY_ROW_LIMIT_V1,
    PersonsCommandCommitV1, PersonsEnvelopeRecordV1, PersonsOutboxRecordV1,
    PersonsPersistenceErrorV1,
};

#[derive(Clone)]
pub struct PersonsPersistenceV1 {
    pool: PgPool,
}

impl PersonsPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, PersonsPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(PersonsPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(u16::try_from(pgbouncer_port).map_err(|_| storage())?)
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

    pub async fn verify_storage_ready(&self) -> Result<(), PersonsPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| storage())
    }

    pub async fn apply_command_once<F>(
        &self,
        input: &ApplyPersonsCommandV1,
        mutation: F,
    ) -> Result<ApplyPersonsCommandOutcomeV1, PersonsPersistenceErrorV1>
    where
        F: FnOnce(&mut PersonsStateV1) -> Result<PersonsCommandCommitV1, PersonsPersistenceErrorV1>,
    {
        validate_command(input)?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner_context(&mut transaction, &input.logical_owner_id).await?;
        ensure_owner_aggregate(&mut transaction, input).await?;
        reserve_command(&mut transaction, input).await?;
        let inbox = lock_command(
            &mut transaction,
            &input.logical_owner_id,
            input.command_message_id,
        )
        .await?;
        validate_reserved_command(&inbox, input)?;
        if inbox.completed {
            let terminal_result = inbox
                .terminal_result
                .ok_or(PersonsPersistenceErrorV1::StateConflict)?;
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(ApplyPersonsCommandOutcomeV1 {
                replayed: true,
                aggregate_revision: inbox
                    .resulting_aggregate_revision
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
                terminal_result,
            });
        }

        let loaded = load_owner_locked(&mut transaction, &input.logical_owner_id).await?;
        if loaded.aggregate_revision != input.expected_aggregate_revision {
            return Err(PersonsPersistenceErrorV1::AggregateConflict);
        }
        let mut state = loaded.state;
        let commit = mutation(&mut state).map_err(|error| {
            if error == PersonsPersistenceErrorV1::StorageUnavailable {
                PersonsPersistenceErrorV1::MutationRejected
            } else {
                error
            }
        })?;
        validate_commit(&commit)?;
        let snapshot = state
            .snapshot_for_owner_v1(&input.logical_owner_id)
            .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
        let round_trip = PersonsStateV1::reconstitute_owner_v1(snapshot.clone())
            .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
        if round_trip != state {
            return Err(PersonsPersistenceErrorV1::StateConflict);
        }
        persist_snapshot(&mut transaction, &snapshot).await?;

        let resulting_revision = input
            .expected_aggregate_revision
            .checked_add(1)
            .ok_or(PersonsPersistenceErrorV1::AggregateConflict)?;
        update_aggregate(
            &mut transaction,
            &input.logical_owner_id,
            input.expected_aggregate_revision,
            resulting_revision,
            commit.completed_at_unix_millis,
        )
        .await?;
        complete_command(&mut transaction, input, resulting_revision, &commit).await?;
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            input.command_message_id,
            resulting_revision,
            &commit,
        )
        .await?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(ApplyPersonsCommandOutcomeV1 {
            replayed: false,
            aggregate_revision: resulting_revision,
            terminal_result: commit.terminal_result,
        })
    }

    pub async fn load_owner(
        &self,
        logical_owner_id: &str,
    ) -> Result<LoadedPersonsOwnerV1, PersonsPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner_context(&mut transaction, logical_owner_id).await?;
        let loaded = load_owner_locked(&mut transaction, logical_owner_id).await?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(loaded)
    }

    pub async fn load_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<PersonsOutboxRecordV1>, PersonsPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner_context(&mut transaction, logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes, command_message_id, \
             resulting_owner_revision, outbox_ordinal, semantic_order_key, created_at_unix_millis, \
             published_at_unix_millis FROM makosh_data.persons_outbox \
             WHERE logical_owner_id = $1 AND published_at_unix_millis IS NULL \
             ORDER BY resulting_owner_revision, created_at_unix_millis, command_message_id, semantic_order_key, \
             outbox_ordinal, message_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(PERSONS_OUTBOX_READ_LIMIT_V1)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let record = PersonsOutboxRecordV1 {
                record: PersonsEnvelopeRecordV1 {
                    message_id: bytes::<16>(&row, "message_id")?,
                    envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
                    envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
                },
                command_message_id: bytes::<16>(&row, "command_message_id")?,
                resulting_owner_revision: row
                    .try_get::<i64, _>("resulting_owner_revision")
                    .map_err(|_| storage())?
                    .try_into()
                    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?,
                outbox_ordinal: row
                    .try_get::<i32, _>("outbox_ordinal")
                    .map_err(|_| storage())?
                    .try_into()
                    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?,
                semantic_order_key: row.try_get("semantic_order_key").map_err(|_| storage())?,
                created_at_unix_millis: row
                    .try_get("created_at_unix_millis")
                    .map_err(|_| storage())?,
                published_at_unix_millis: row
                    .try_get("published_at_unix_millis")
                    .map_err(|_| storage())?,
            };
            validate_envelope_record(&record.record)?;
            validate_loaded_outbox_order(&record)?;
            records.push(record);
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(records)
    }

    pub async fn mark_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        expected_envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), PersonsPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if zero(&message_id) || zero(&expected_envelope_sha256) || published_at_unix_millis <= 0 {
            return Err(PersonsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner_context(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT envelope_sha256, envelope_bytes, created_at_unix_millis, \
             published_at_unix_millis FROM makosh_data.persons_outbox \
             WHERE logical_owner_id = $1 AND message_id = $2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .ok_or(PersonsPersistenceErrorV1::StateConflict)?;
        let stored = PersonsEnvelopeRecordV1 {
            message_id,
            envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
        };
        validate_envelope_record(&stored)?;
        if stored.envelope_sha256 != expected_envelope_sha256 {
            return Err(PersonsPersistenceErrorV1::HashMismatch);
        }
        let existing_published_at: Option<i64> = row
            .try_get("published_at_unix_millis")
            .map_err(|_| storage())?;
        if let Some(existing) = existing_published_at {
            if existing != published_at_unix_millis {
                return Err(PersonsPersistenceErrorV1::StateConflict);
            }
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(());
        }
        let result = sqlx::query(
            "UPDATE makosh_data.persons_outbox SET published_at_unix_millis = $4 \
             WHERE logical_owner_id = $1 AND message_id = $2 \
             AND envelope_sha256 = $3 AND published_at_unix_millis IS NULL",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(expected_envelope_sha256.as_slice())
        .bind(published_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        if result.rows_affected() != 1 {
            return Err(PersonsPersistenceErrorV1::StateConflict);
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(())
    }
}

#[derive(Debug)]
struct InboxRecord {
    command_envelope_sha256: [u8; 32],
    command_id: [u8; 16],
    command_fingerprint: [u8; 32],
    expected_aggregate_revision: u64,
    completed: bool,
    resulting_aggregate_revision: Option<u64>,
    terminal_result: Option<PersonsEnvelopeRecordV1>,
}

async fn set_owner_context(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
) -> Result<(), PersonsPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
        .bind(logical_owner_id)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(|_| storage())
}

async fn ensure_owner_aggregate(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ApplyPersonsCommandV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.persons_owner_aggregates \
         (logical_owner_id, aggregate_revision, updated_at_unix_seconds, updated_at_nanos) \
         VALUES ($1, 0, $2, 0) ON CONFLICT (logical_owner_id) DO NOTHING",
    )
    .bind(&input.logical_owner_id)
    .bind(input.received_at_unix_millis / 1_000)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| storage())
}

async fn reserve_command(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ApplyPersonsCommandV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.persons_command_inbox \
         (logical_owner_id, command_message_id, command_envelope_sha256, command_id, \
          command_fingerprint, expected_aggregate_revision, received_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT DO NOTHING",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(input.command_envelope_sha256.as_slice())
    .bind(input.command_id.as_slice())
    .bind(input.command_fingerprint.as_slice())
    .bind(i64_value(input.expected_aggregate_revision)?)
    .bind(input.received_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| storage())
}

async fn lock_command(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    command_message_id: [u8; 16],
) -> Result<InboxRecord, PersonsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT command_envelope_sha256, command_id, command_fingerprint, \
         expected_aggregate_revision, completed, resulting_aggregate_revision, \
         terminal_message_id, terminal_envelope_sha256, terminal_envelope_bytes \
         FROM makosh_data.persons_command_inbox \
         WHERE logical_owner_id = $1 AND command_message_id = $2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(command_message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| PersonsPersistenceErrorV1::CommandConflict)?;
    let completed: bool = row.try_get("completed").map_err(|_| storage())?;
    let terminal_result = if completed {
        let terminal = PersonsEnvelopeRecordV1 {
            message_id: optional_bytes::<16>(&row, "terminal_message_id")?
                .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            envelope_sha256: optional_bytes::<32>(&row, "terminal_envelope_sha256")?
                .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            envelope_bytes: row
                .try_get::<Option<Vec<u8>>, _>("terminal_envelope_bytes")
                .map_err(|_| storage())?
                .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
        };
        validate_envelope_record(&terminal)?;
        Some(terminal)
    } else {
        None
    };
    Ok(InboxRecord {
        command_envelope_sha256: bytes(&row, "command_envelope_sha256")?,
        command_id: bytes(&row, "command_id")?,
        command_fingerprint: bytes(&row, "command_fingerprint")?,
        expected_aggregate_revision: u64_value(
            row.try_get("expected_aggregate_revision")
                .map_err(|_| storage())?,
        )?,
        completed,
        resulting_aggregate_revision: row
            .try_get::<Option<i64>, _>("resulting_aggregate_revision")
            .map_err(|_| storage())?
            .map(u64_value)
            .transpose()?,
        terminal_result,
    })
}

fn validate_reserved_command(
    record: &InboxRecord,
    input: &ApplyPersonsCommandV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    if record.command_envelope_sha256 != input.command_envelope_sha256
        || record.command_id != input.command_id
        || record.command_fingerprint != input.command_fingerprint
        || (!record.completed
            && record.expected_aggregate_revision != input.expected_aggregate_revision)
    {
        Err(PersonsPersistenceErrorV1::CommandConflict)
    } else {
        Ok(())
    }
}

async fn load_owner_locked(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
) -> Result<LoadedPersonsOwnerV1, PersonsPersistenceErrorV1> {
    let aggregate_revision = sqlx::query_scalar::<_, i64>(
        "SELECT aggregate_revision FROM makosh_data.persons_owner_aggregates \
         WHERE logical_owner_id = $1 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| storage())?
    .unwrap_or(0);
    let person_rows = sqlx::query(
        "SELECT person_id, lifecycle, person_revision, current_profile_revision, \
         merged_into_person_id, created_at_unix_seconds, created_at_nanos, \
         updated_at_unix_seconds, updated_at_nanos FROM makosh_data.persons_current \
         WHERE logical_owner_id = $1 ORDER BY person_id LIMIT $2",
    )
    .bind(logical_owner_id)
    .bind(PERSONS_RECOVERY_ROW_LIMIT_V1 + 1)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    bounded(&person_rows)?;

    let mut persons = BTreeMap::<PersonIdV1, PersonV1>::new();
    for row in person_rows {
        let person_id = PersonIdV1(bytes(&row, "person_id")?);
        let profile_revision: Option<i64> = row
            .try_get("current_profile_revision")
            .map_err(|_| storage())?;
        let owner_profile = if let Some(profile_revision) = profile_revision {
            Some(load_profile(transaction, logical_owner_id, person_id, profile_revision).await?)
        } else {
            None
        };
        let lifecycle = match row.try_get::<i16, _>("lifecycle").map_err(|_| storage())? {
            1 => PersonLifecycleV1::Provisional,
            2 => PersonLifecycleV1::Active,
            3 => PersonLifecycleV1::Merged,
            4 => PersonLifecycleV1::Archived,
            _ => return Err(PersonsPersistenceErrorV1::StateConflict),
        };
        let person = PersonV1 {
            person_id,
            logical_owner_id: logical_owner_id.to_owned(),
            lifecycle,
            revision: u64_value(row.try_get("person_revision").map_err(|_| storage())?)?,
            owner_profile,
            source_links: BTreeMap::new(),
            merged_into: optional_bytes(&row, "merged_into_person_id")?.map(PersonIdV1),
            created_at: timestamp(&row, "created_at_unix_seconds", "created_at_nanos")?,
            updated_at: timestamp(&row, "updated_at_unix_seconds", "updated_at_nanos")?,
        };
        if persons.insert(person_id, person).is_some() {
            return Err(PersonsPersistenceErrorV1::StateConflict);
        }
    }

    let source_rows = sqlx::query(
        "SELECT integration_public_id, account_public_id, provider_source_contact_public_id, \
         person_id, removed, display_name, normalized_emails, normalized_phones, source_revision, \
         source_digest, observed_at_unix_seconds, observed_at_nanos, last_decision_id, \
         last_review_id, last_decision_revision, last_decided_by_owner_device_id, \
         last_decided_at_unix_seconds, last_decided_at_nanos, last_approved_action_digest \
         FROM makosh_data.persons_sources WHERE logical_owner_id = $1 \
         ORDER BY integration_public_id, account_public_id, provider_source_contact_public_id \
         LIMIT $2",
    )
    .bind(logical_owner_id)
    .bind(PERSONS_RECOVERY_ROW_LIMIT_V1 + 1)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    bounded(&source_rows)?;
    let mut removed_sources = Vec::new();
    for row in source_rows {
        let key = source_key(&row)?;
        let provenance = SourceProvenanceV1 {
            revision: u64_value(row.try_get("source_revision").map_err(|_| storage())?)?,
            digest: DigestV1(bytes(&row, "source_digest")?),
            observed_at: timestamp(&row, "observed_at_unix_seconds", "observed_at_nanos")?,
        };
        if row.try_get::<bool, _>("removed").map_err(|_| storage())? {
            removed_sources.push((
                key,
                RemovedSourceV1 {
                    logical_owner_id: logical_owner_id.to_owned(),
                    provenance,
                },
            ));
            continue;
        }
        let person_id = PersonIdV1(
            optional_bytes(&row, "person_id")?.ok_or(PersonsPersistenceErrorV1::StateConflict)?,
        );
        let last_decision = decode_optional_decision(&row)?;
        let source = SourceLinkV1 {
            key,
            claims: SourceClaimsV1 {
                display_name: row.try_get("display_name").map_err(|_| storage())?,
                emails: row.try_get("normalized_emails").map_err(|_| storage())?,
                phones: row.try_get("normalized_phones").map_err(|_| storage())?,
            },
            provenance,
            last_decision,
        };
        persons
            .get_mut(&person_id)
            .ok_or(PersonsPersistenceErrorV1::StateConflict)?
            .source_links
            .insert(key, source);
    }

    let lineage = load_lineage(transaction, logical_owner_id).await?;
    let decision_receipts = load_decisions(transaction, logical_owner_id).await?;
    let snapshot = PersonsOwnerSnapshotV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        persons: persons.into_values().collect(),
        removed_sources,
        lineage,
        decision_receipts,
    };
    let state = PersonsStateV1::reconstitute_owner_v1(snapshot)
        .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    Ok(LoadedPersonsOwnerV1 {
        aggregate_revision: u64_value(aggregate_revision)?,
        state,
    })
}

async fn load_profile(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    person_id: PersonIdV1,
    revision: i64,
) -> Result<OwnerProfileV1, PersonsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT display_name, given_name, family_name, normalized_emails, normalized_phones \
         FROM makosh_data.persons_profiles WHERE logical_owner_id = $1 AND person_id = $2 \
         AND profile_revision = $3",
    )
    .bind(owner)
    .bind(person_id.0.as_slice())
    .bind(revision)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    Ok(OwnerProfileV1 {
        display_name: row.try_get("display_name").map_err(|_| storage())?,
        given_name: row.try_get("given_name").map_err(|_| storage())?,
        family_name: row.try_get("family_name").map_err(|_| storage())?,
        emails: row.try_get("normalized_emails").map_err(|_| storage())?,
        phones: row.try_get("normalized_phones").map_err(|_| storage())?,
    })
}

async fn load_lineage(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<Vec<LineageRecordV1>, PersonsPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT lineage_sequence, change_kind, source_person_id, target_person_id, \
         preserved_display_name, preserved_given_name, preserved_family_name, preserved_emails, \
         preserved_phones, selected_profile_fact_kinds, decision_id, review_id, decision_revision, \
         decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos, \
         approved_action_digest FROM makosh_data.persons_lineage WHERE logical_owner_id = $1 \
         ORDER BY lineage_sequence LIMIT $2",
    )
    .bind(owner)
    .bind(PERSONS_RECOVERY_ROW_LIMIT_V1 + 1)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    bounded(&rows)?;
    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        let sequence: i64 = row.try_get("lineage_sequence").map_err(|_| storage())?;
        let source_rows = sqlx::query(
            "SELECT integration_public_id, account_public_id, provider_source_contact_public_id \
             FROM makosh_data.persons_lineage_sources WHERE logical_owner_id = $1 \
             AND lineage_sequence = $2 ORDER BY source_sequence LIMIT $3",
        )
        .bind(owner)
        .bind(sequence)
        .bind(PERSONS_RECOVERY_ROW_LIMIT_V1 + 1)
        .fetch_all(&mut **transaction)
        .await
        .map_err(|_| storage())?;
        bounded(&source_rows)?;
        let selected: Vec<i16> = row
            .try_get("selected_profile_fact_kinds")
            .map_err(|_| storage())?;
        let mut unique = BTreeSet::new();
        let mut profile_fact_selection = Vec::with_capacity(selected.len());
        for value in selected {
            let fact = match value {
                1 => SplitProfileFactKindV1::DisplayName,
                2 => SplitProfileFactKindV1::GivenName,
                3 => SplitProfileFactKindV1::FamilyName,
                4 => SplitProfileFactKindV1::Emails,
                5 => SplitProfileFactKindV1::Phones,
                _ => return Err(PersonsPersistenceErrorV1::StateConflict),
            };
            if !unique.insert(fact) {
                return Err(PersonsPersistenceErrorV1::StateConflict);
            }
            profile_fact_selection.push(fact);
        }
        let profile = OwnerProfileV1 {
            display_name: row
                .try_get("preserved_display_name")
                .map_err(|_| storage())?,
            given_name: row.try_get("preserved_given_name").map_err(|_| storage())?,
            family_name: row
                .try_get("preserved_family_name")
                .map_err(|_| storage())?,
            emails: row.try_get("preserved_emails").map_err(|_| storage())?,
            phones: row.try_get("preserved_phones").map_err(|_| storage())?,
        };
        records.push(LineageRecordV1 {
            change_kind: match row
                .try_get::<i16, _>("change_kind")
                .map_err(|_| storage())?
            {
                1 => LineageChangeKindV1::Merge,
                2 => LineageChangeKindV1::Split,
                _ => return Err(PersonsPersistenceErrorV1::StateConflict),
            },
            source_person_id: PersonIdV1(bytes(&row, "source_person_id")?),
            target_person_id: PersonIdV1(bytes(&row, "target_person_id")?),
            moved_sources: source_rows
                .iter()
                .map(source_key)
                .collect::<Result<Vec<_>, _>>()?,
            preserved_source_profile: if profile.is_empty() {
                None
            } else {
                Some(profile)
            },
            profile_fact_selection,
            decision: DecisionProvenanceV1 {
                decision_id: PublicIdV1(bytes(&row, "decision_id")?),
                review_id: PublicIdV1(bytes(&row, "review_id")?),
                revision: u64_value(row.try_get("decision_revision").map_err(|_| storage())?)?,
                decided_by_owner_device_id: PublicIdV1(bytes(&row, "decided_by_owner_device_id")?),
                decided_at: timestamp(&row, "decided_at_unix_seconds", "decided_at_nanos")?,
                approved_action_digest: DigestV1(bytes(&row, "approved_action_digest")?),
            },
        });
    }
    Ok(records)
}

async fn load_decisions(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<Vec<DecisionReceiptV1>, PersonsPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT decision_id, action_digest, review_id, decision_revision, \
         decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos \
         FROM makosh_data.persons_decision_receipts WHERE logical_owner_id = $1 \
         ORDER BY decision_id LIMIT $2",
    )
    .bind(owner)
    .bind(PERSONS_RECOVERY_ROW_LIMIT_V1 + 1)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    bounded(&rows)?;
    let mut receipts = Vec::with_capacity(rows.len());
    for row in rows {
        let decision_id = PublicIdV1(bytes(&row, "decision_id")?);
        let outcome_rows = sqlx::query(
            "SELECT person_id, resulting_person_revision \
             FROM makosh_data.persons_decision_outcomes WHERE logical_owner_id = $1 \
             AND decision_id = $2 ORDER BY person_id LIMIT $3",
        )
        .bind(owner)
        .bind(decision_id.0.as_slice())
        .bind(PERSONS_RECOVERY_ROW_LIMIT_V1 + 1)
        .fetch_all(&mut **transaction)
        .await
        .map_err(|_| storage())?;
        bounded(&outcome_rows)?;
        let action_digest = DigestV1(bytes(&row, "action_digest")?);
        receipts.push(DecisionReceiptV1 {
            logical_owner_id: owner.to_owned(),
            action_digest,
            decision: DecisionProvenanceV1 {
                decision_id,
                review_id: PublicIdV1(bytes(&row, "review_id")?),
                revision: u64_value(row.try_get("decision_revision").map_err(|_| storage())?)?,
                decided_by_owner_device_id: PublicIdV1(bytes(&row, "decided_by_owner_device_id")?),
                decided_at: timestamp(&row, "decided_at_unix_seconds", "decided_at_nanos")?,
                approved_action_digest: action_digest,
            },
            outcome: ConfirmedActionOutcomeV1 {
                status: ConfirmedActionStatusV1::Applied,
                person_revisions: outcome_rows
                    .iter()
                    .map(|outcome| {
                        Ok(PersonRevisionV1 {
                            person_id: PersonIdV1(bytes(outcome, "person_id")?),
                            revision: u64_value(
                                outcome
                                    .try_get("resulting_person_revision")
                                    .map_err(|_| storage())?,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, PersonsPersistenceErrorV1>>()?,
            },
        });
    }
    Ok(receipts)
}

async fn persist_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    snapshot: &PersonsOwnerSnapshotV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    for person in &snapshot.persons {
        let profile_plan = profile_plan(transaction, person).await?;
        upsert_person(transaction, person, profile_plan.revision).await?;
        if profile_plan.append {
            insert_profile_snapshot(
                transaction,
                person,
                person
                    .owner_profile
                    .as_ref()
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
                profile_plan
                    .revision
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            )
            .await?;
        }
    }
    for person in &snapshot.persons {
        for source in person.source_links.values() {
            upsert_active_source(transaction, person, source).await?;
        }
    }
    for (key, removed) in &snapshot.removed_sources {
        upsert_removed_source(transaction, *key, removed).await?;
    }
    for (index, lineage) in snapshot.lineage.iter().enumerate() {
        insert_lineage(transaction, &snapshot.logical_owner_id, index + 1, lineage).await?;
    }
    for receipt in &snapshot.decision_receipts {
        insert_decision(transaction, receipt).await?;
    }
    Ok(())
}

struct ProfilePlan {
    revision: Option<i64>,
    append: bool,
}

async fn profile_plan(
    transaction: &mut Transaction<'_, Postgres>,
    person: &PersonV1,
) -> Result<ProfilePlan, PersonsPersistenceErrorV1> {
    let existing_revision = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT current_profile_revision FROM makosh_data.persons_current \
         WHERE logical_owner_id = $1 AND person_id = $2",
    )
    .bind(&person.logical_owner_id)
    .bind(person.person_id.0.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| storage())?
    .flatten();
    let max_revision = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT MAX(profile_revision) FROM makosh_data.persons_profiles \
         WHERE logical_owner_id = $1 AND person_id = $2",
    )
    .bind(&person.logical_owner_id)
    .bind(person.person_id.0.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    let Some(profile) = &person.owner_profile else {
        return Ok(ProfilePlan {
            revision: None,
            append: false,
        });
    };
    if let Some(revision) = existing_revision {
        let existing = load_profile(
            transaction,
            &person.logical_owner_id,
            person.person_id,
            revision,
        )
        .await?;
        if existing == *profile {
            return Ok(ProfilePlan {
                revision: Some(revision),
                append: false,
            });
        }
        return Ok(ProfilePlan {
            revision: Some(
                max_revision
                    .unwrap_or(revision)
                    .checked_add(1)
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            ),
            append: true,
        });
    }
    Ok(ProfilePlan {
        revision: Some(
            max_revision
                .unwrap_or(0)
                .checked_add(1)
                .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
        ),
        append: true,
    })
}

async fn upsert_person(
    transaction: &mut Transaction<'_, Postgres>,
    person: &PersonV1,
    profile_revision: Option<i64>,
) -> Result<(), PersonsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.persons_current \
         (logical_owner_id, person_id, lifecycle, person_revision, current_profile_revision, \
          merged_into_person_id, created_at_unix_seconds, created_at_nanos, \
          updated_at_unix_seconds, updated_at_nanos) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) \
         ON CONFLICT (logical_owner_id, person_id) DO UPDATE SET lifecycle = EXCLUDED.lifecycle, \
         person_revision = EXCLUDED.person_revision, current_profile_revision = EXCLUDED.current_profile_revision, \
         merged_into_person_id = EXCLUDED.merged_into_person_id, \
         updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds, \
         updated_at_nanos = EXCLUDED.updated_at_nanos \
         WHERE makosh_data.persons_current.person_revision < EXCLUDED.person_revision",
    )
    .bind(&person.logical_owner_id)
    .bind(person.person_id.0.as_slice())
    .bind(lifecycle(person.lifecycle))
    .bind(i64_value(person.revision)?)
    .bind(profile_revision)
    .bind(person.merged_into.map(|id| id.0.to_vec()))
    .bind(person.created_at.unix_seconds)
    .bind(person.created_at.nanos)
    .bind(person.updated_at.unix_seconds)
    .bind(person.updated_at.nanos)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)
}

async fn insert_profile_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    person: &PersonV1,
    profile: &OwnerProfileV1,
    profile_revision: i64,
) -> Result<(), PersonsPersistenceErrorV1> {
    let result = sqlx::query(
        "INSERT INTO makosh_data.persons_profiles \
         (logical_owner_id, person_id, display_name, given_name, family_name, normalized_emails, \
          normalized_phones, profile_revision) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) \
         ON CONFLICT (logical_owner_id, person_id, profile_revision) DO NOTHING",
    )
    .bind(&person.logical_owner_id)
    .bind(person.person_id.0.as_slice())
    .bind(&profile.display_name)
    .bind(&profile.given_name)
    .bind(&profile.family_name)
    .bind(&profile.emails)
    .bind(&profile.phones)
    .bind(profile_revision)
    .execute(&mut **transaction)
    .await
    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(PersonsPersistenceErrorV1::StateConflict)
    }
}

async fn upsert_active_source(
    transaction: &mut Transaction<'_, Postgres>,
    person: &PersonV1,
    source: &SourceLinkV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    let decision = source.last_decision.as_ref();
    let result = sqlx::query(
        "INSERT INTO makosh_data.persons_sources \
         (logical_owner_id, integration_public_id, account_public_id, provider_source_contact_public_id, \
          person_id, removed, display_name, normalized_emails, normalized_phones, source_revision, \
          source_digest, observed_at_unix_seconds, observed_at_nanos, last_decision_id, last_review_id, \
          last_decision_revision, last_decided_by_owner_device_id, last_decided_at_unix_seconds, \
          last_decided_at_nanos, last_approved_action_digest) \
         VALUES ($1,$2,$3,$4,$5,FALSE,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19) \
         ON CONFLICT (integration_public_id, account_public_id, provider_source_contact_public_id) \
         DO UPDATE SET person_id = EXCLUDED.person_id, removed = FALSE, display_name = EXCLUDED.display_name, \
         normalized_emails = EXCLUDED.normalized_emails, normalized_phones = EXCLUDED.normalized_phones, \
         source_revision = EXCLUDED.source_revision, source_digest = EXCLUDED.source_digest, \
         observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, observed_at_nanos = EXCLUDED.observed_at_nanos, \
         last_decision_id = EXCLUDED.last_decision_id, last_review_id = EXCLUDED.last_review_id, \
         last_decision_revision = EXCLUDED.last_decision_revision, \
         last_decided_by_owner_device_id = EXCLUDED.last_decided_by_owner_device_id, \
         last_decided_at_unix_seconds = EXCLUDED.last_decided_at_unix_seconds, \
         last_decided_at_nanos = EXCLUDED.last_decided_at_nanos, \
         last_approved_action_digest = EXCLUDED.last_approved_action_digest \
         WHERE makosh_data.persons_sources.logical_owner_id = EXCLUDED.logical_owner_id \
         AND makosh_data.persons_sources.source_revision <= EXCLUDED.source_revision",
    )
    .bind(&person.logical_owner_id)
    .bind(source.key.integration_public_id.0.as_slice())
    .bind(source.key.account_public_id.0.as_slice())
    .bind(source.key.provider_source_contact_public_id.0.as_slice())
    .bind(person.person_id.0.as_slice())
    .bind(&source.claims.display_name)
    .bind(&source.claims.emails)
    .bind(&source.claims.phones)
    .bind(i64_value(source.provenance.revision)?)
    .bind(source.provenance.digest.0.as_slice())
    .bind(source.provenance.observed_at.unix_seconds)
    .bind(source.provenance.observed_at.nanos)
    .bind(decision.map(|value| value.decision_id.0.to_vec()))
    .bind(decision.map(|value| value.review_id.0.to_vec()))
    .bind(decision.map(|value| i64_value(value.revision)).transpose()?)
    .bind(decision.map(|value| value.decided_by_owner_device_id.0.to_vec()))
    .bind(decision.map(|value| value.decided_at.unix_seconds))
    .bind(decision.map(|value| value.decided_at.nanos))
    .bind(decision.map(|value| value.approved_action_digest.0.to_vec()))
    .execute(&mut **transaction)
    .await
    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(PersonsPersistenceErrorV1::StateConflict)
    }
}

async fn upsert_removed_source(
    transaction: &mut Transaction<'_, Postgres>,
    key: SourceLinkKeyV1,
    removed: &RemovedSourceV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    let result = sqlx::query(
        "INSERT INTO makosh_data.persons_sources \
         (logical_owner_id, integration_public_id, account_public_id, provider_source_contact_public_id, \
          person_id, removed, display_name, normalized_emails, normalized_phones, source_revision, \
          source_digest, observed_at_unix_seconds, observed_at_nanos) \
         VALUES ($1,$2,$3,$4,NULL,TRUE,NULL,'{}','{}',$5,$6,$7,$8) \
         ON CONFLICT (integration_public_id, account_public_id, provider_source_contact_public_id) \
         DO UPDATE SET person_id = NULL, removed = TRUE, display_name = NULL, \
         normalized_emails = '{}', normalized_phones = '{}', source_revision = EXCLUDED.source_revision, \
         source_digest = EXCLUDED.source_digest, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, \
         observed_at_nanos = EXCLUDED.observed_at_nanos, last_decision_id = NULL, last_review_id = NULL, \
         last_decision_revision = NULL, last_decided_by_owner_device_id = NULL, \
         last_decided_at_unix_seconds = NULL, last_decided_at_nanos = NULL, \
         last_approved_action_digest = NULL \
         WHERE makosh_data.persons_sources.logical_owner_id = EXCLUDED.logical_owner_id \
         AND makosh_data.persons_sources.source_revision <= EXCLUDED.source_revision",
    )
    .bind(&removed.logical_owner_id)
    .bind(key.integration_public_id.0.as_slice())
    .bind(key.account_public_id.0.as_slice())
    .bind(key.provider_source_contact_public_id.0.as_slice())
    .bind(i64_value(removed.provenance.revision)?)
    .bind(removed.provenance.digest.0.as_slice())
    .bind(removed.provenance.observed_at.unix_seconds)
    .bind(removed.provenance.observed_at.nanos)
    .execute(&mut **transaction)
    .await
    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(PersonsPersistenceErrorV1::StateConflict)
    }
}

async fn insert_lineage(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    sequence: usize,
    lineage: &LineageRecordV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    let profile = lineage.preserved_source_profile.as_ref();
    let facts = lineage
        .profile_fact_selection
        .iter()
        .map(|fact| match fact {
            SplitProfileFactKindV1::DisplayName => 1_i16,
            SplitProfileFactKindV1::GivenName => 2,
            SplitProfileFactKindV1::FamilyName => 3,
            SplitProfileFactKindV1::Emails => 4,
            SplitProfileFactKindV1::Phones => 5,
        })
        .collect::<Vec<_>>();
    sqlx::query(
        "INSERT INTO makosh_data.persons_lineage \
         (logical_owner_id, lineage_sequence, change_kind, source_person_id, target_person_id, \
          preserved_display_name, preserved_given_name, preserved_family_name, preserved_emails, \
          preserved_phones, selected_profile_fact_kinds, decision_id, review_id, decision_revision, \
          decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos, approved_action_digest) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18) \
         ON CONFLICT (logical_owner_id, lineage_sequence) DO NOTHING",
    )
    .bind(owner)
    .bind(i64::try_from(sequence).map_err(|_| PersonsPersistenceErrorV1::StateConflict)?)
    .bind(match lineage.change_kind { LineageChangeKindV1::Merge => 1_i16, LineageChangeKindV1::Split => 2 })
    .bind(lineage.source_person_id.0.as_slice())
    .bind(lineage.target_person_id.0.as_slice())
    .bind(profile.and_then(|value| value.display_name.as_ref()))
    .bind(profile.and_then(|value| value.given_name.as_ref()))
    .bind(profile.and_then(|value| value.family_name.as_ref()))
    .bind(profile.map_or(&[][..], |value| value.emails.as_slice()))
    .bind(profile.map_or(&[][..], |value| value.phones.as_slice()))
    .bind(&facts)
    .bind(lineage.decision.decision_id.0.as_slice())
    .bind(lineage.decision.review_id.0.as_slice())
    .bind(i64_value(lineage.decision.revision)?)
    .bind(lineage.decision.decided_by_owner_device_id.0.as_slice())
    .bind(lineage.decision.decided_at.unix_seconds)
    .bind(lineage.decision.decided_at.nanos)
    .bind(lineage.decision.approved_action_digest.0.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    for (index, source) in lineage.moved_sources.iter().enumerate() {
        sqlx::query(
            "INSERT INTO makosh_data.persons_lineage_sources \
             (logical_owner_id, lineage_sequence, source_sequence, integration_public_id, \
              account_public_id, provider_source_contact_public_id) VALUES ($1,$2,$3,$4,$5,$6) \
             ON CONFLICT (logical_owner_id, lineage_sequence, source_sequence) DO NOTHING",
        )
        .bind(owner)
        .bind(i64::try_from(sequence).map_err(|_| PersonsPersistenceErrorV1::StateConflict)?)
        .bind(i32::try_from(index + 1).map_err(|_| PersonsPersistenceErrorV1::StateConflict)?)
        .bind(source.integration_public_id.0.as_slice())
        .bind(source.account_public_id.0.as_slice())
        .bind(source.provider_source_contact_public_id.0.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    }
    Ok(())
}

async fn insert_decision(
    transaction: &mut Transaction<'_, Postgres>,
    receipt: &DecisionReceiptV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.persons_decision_receipts \
         (logical_owner_id, decision_id, action_digest, review_id, decision_revision, \
          decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (logical_owner_id, decision_id) DO NOTHING",
    )
    .bind(&receipt.logical_owner_id)
    .bind(receipt.decision.decision_id.0.as_slice())
    .bind(receipt.action_digest.0.as_slice())
    .bind(receipt.decision.review_id.0.as_slice())
    .bind(i64_value(receipt.decision.revision)?)
    .bind(receipt.decision.decided_by_owner_device_id.0.as_slice())
    .bind(receipt.decision.decided_at.unix_seconds)
    .bind(receipt.decision.decided_at.nanos)
    .execute(&mut **transaction)
    .await
    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    for outcome in &receipt.outcome.person_revisions {
        sqlx::query(
            "INSERT INTO makosh_data.persons_decision_outcomes \
             (logical_owner_id, decision_id, person_id, resulting_person_revision) \
             VALUES ($1,$2,$3,$4) ON CONFLICT (logical_owner_id, decision_id, person_id) DO NOTHING",
        )
        .bind(&receipt.logical_owner_id)
        .bind(receipt.decision.decision_id.0.as_slice())
        .bind(outcome.person_id.0.as_slice())
        .bind(i64_value(outcome.revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    }
    Ok(())
}

async fn update_aggregate(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    expected: u64,
    resulting: u64,
    completed_at_unix_millis: i64,
) -> Result<(), PersonsPersistenceErrorV1> {
    let result = sqlx::query(
        "UPDATE makosh_data.persons_owner_aggregates SET aggregate_revision = $3, \
         updated_at_unix_seconds = $4, updated_at_nanos = $5 \
         WHERE logical_owner_id = $1 AND aggregate_revision = $2 \
         AND (updated_at_unix_seconds, updated_at_nanos) <= ($4, $5)",
    )
    .bind(owner)
    .bind(i64_value(expected)?)
    .bind(i64_value(resulting)?)
    .bind(completed_at_unix_millis / 1_000)
    .bind(
        i32::try_from((completed_at_unix_millis % 1_000) * 1_000_000)
            .map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?,
    )
    .execute(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(PersonsPersistenceErrorV1::AggregateConflict)
    }
}

async fn complete_command(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ApplyPersonsCommandV1,
    resulting_revision: u64,
    commit: &PersonsCommandCommitV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    let result = sqlx::query(
        "UPDATE makosh_data.persons_command_inbox SET completed = TRUE, \
         resulting_aggregate_revision = $3, terminal_message_id = $4, terminal_envelope_sha256 = $5, \
         terminal_envelope_bytes = $6, completed_at_unix_millis = $7 \
         WHERE logical_owner_id = $1 AND command_message_id = $2 AND NOT completed",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(i64_value(resulting_revision)?)
    .bind(commit.terminal_result.message_id.as_slice())
    .bind(commit.terminal_result.envelope_sha256.as_slice())
    .bind(&commit.terminal_result.envelope_bytes)
    .bind(commit.completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(PersonsPersistenceErrorV1::CommandConflict)
    }
}

async fn insert_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    command_message_id: [u8; 16],
    resulting_owner_revision: u64,
    commit: &PersonsCommandCommitV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    insert_outbox_record(
        transaction,
        owner,
        &commit.terminal_result,
        command_message_id,
        resulting_owner_revision,
        0,
        &[0],
        commit.completed_at_unix_millis,
    )
    .await?;
    for (index, (event, order_key)) in commit
        .owner_events
        .iter()
        .zip(&commit.owner_event_order_keys)
        .enumerate()
    {
        insert_outbox_record(
            transaction,
            owner,
            event,
            command_message_id,
            resulting_owner_revision,
            u16::try_from(index + 1).map_err(|_| PersonsPersistenceErrorV1::InvalidInput)?,
            order_key,
            commit.completed_at_unix_millis,
        )
        .await?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_outbox_record(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    event: &PersonsEnvelopeRecordV1,
    command_message_id: [u8; 16],
    resulting_owner_revision: u64,
    ordinal: u16,
    semantic_order_key: &[u8],
    created_at_unix_millis: i64,
) -> Result<(), PersonsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.persons_outbox \
         (logical_owner_id, message_id, envelope_sha256, envelope_bytes, command_message_id, \
          resulting_owner_revision, outbox_ordinal, semantic_order_key, created_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(owner)
    .bind(event.message_id.as_slice())
    .bind(event.envelope_sha256.as_slice())
    .bind(&event.envelope_bytes)
    .bind(command_message_id.as_slice())
    .bind(i64_value(resulting_owner_revision)?)
    .bind(i32::from(ordinal))
    .bind(semantic_order_key)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| PersonsPersistenceErrorV1::StateConflict)?;
    Ok(())
}

fn validate_command(input: &ApplyPersonsCommandV1) -> Result<(), PersonsPersistenceErrorV1> {
    validate_owner(&input.logical_owner_id)?;
    if zero(&input.command_message_id)
        || zero(&input.command_envelope_sha256)
        || zero(&input.command_id)
        || zero(&input.command_fingerprint)
        || input.received_at_unix_millis <= 0
    {
        return Err(PersonsPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_commit(commit: &PersonsCommandCommitV1) -> Result<(), PersonsPersistenceErrorV1> {
    if commit.completed_at_unix_millis <= 0
        || commit.owner_events.len() > PERSONS_OUTBOX_READ_LIMIT_V1 as usize
        || commit.owner_event_order_keys.len() != commit.owner_events.len()
    {
        return Err(PersonsPersistenceErrorV1::InvalidInput);
    }
    let mut ids = BTreeSet::new();
    for envelope in std::iter::once(&commit.terminal_result).chain(&commit.owner_events) {
        validate_envelope_record(envelope)?;
        if !ids.insert(envelope.message_id) {
            return Err(PersonsPersistenceErrorV1::InvalidInput);
        }
    }
    let mut previous: Option<&[u8]> = None;
    for key in &commit.owner_event_order_keys {
        if key.is_empty()
            || key.len() > 128
            || key[0] == 0
            || previous.is_some_and(|old| old >= key.as_slice())
        {
            return Err(PersonsPersistenceErrorV1::InvalidInput);
        }
        previous = Some(key);
    }
    Ok(())
}

fn validate_loaded_outbox_order(
    record: &PersonsOutboxRecordV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    if zero(&record.command_message_id)
        || record.semantic_order_key.is_empty()
        || record.semantic_order_key.len() > 128
        || (record.outbox_ordinal == 0 && record.semantic_order_key != [0])
        || (record.outbox_ordinal > 0 && record.semantic_order_key[0] == 0)
    {
        Err(PersonsPersistenceErrorV1::StateConflict)
    } else {
        Ok(())
    }
}

fn validate_envelope_record(
    envelope: &PersonsEnvelopeRecordV1,
) -> Result<(), PersonsPersistenceErrorV1> {
    let actual: [u8; 32] = Sha256::digest(&envelope.envelope_bytes).into();
    if zero(&envelope.message_id)
        || zero(&envelope.envelope_sha256)
        || envelope.envelope_bytes.is_empty()
        || envelope.envelope_bytes.len() > PERSONS_MAX_ENVELOPE_BYTES_V1
        || actual != envelope.envelope_sha256
    {
        Err(PersonsPersistenceErrorV1::HashMismatch)
    } else {
        Ok(())
    }
}

fn validate_owner(owner: &str) -> Result<(), PersonsPersistenceErrorV1> {
    if owner.is_empty()
        || owner.len() > 128
        || !owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Err(PersonsPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}

fn decode_optional_decision(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<DecisionProvenanceV1>, PersonsPersistenceErrorV1> {
    let decision_id = optional_bytes::<16>(row, "last_decision_id")?;
    match decision_id {
        None => Ok(None),
        Some(decision_id) => Ok(Some(DecisionProvenanceV1 {
            decision_id: PublicIdV1(decision_id),
            review_id: PublicIdV1(
                optional_bytes(row, "last_review_id")?
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            ),
            revision: u64_value(
                row.try_get::<Option<i64>, _>("last_decision_revision")
                    .map_err(|_| storage())?
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            )?,
            decided_by_owner_device_id: PublicIdV1(
                optional_bytes(row, "last_decided_by_owner_device_id")?
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            ),
            decided_at: TimestampV1 {
                unix_seconds: row
                    .try_get::<Option<i64>, _>("last_decided_at_unix_seconds")
                    .map_err(|_| storage())?
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
                nanos: row
                    .try_get::<Option<i32>, _>("last_decided_at_nanos")
                    .map_err(|_| storage())?
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            },
            approved_action_digest: DigestV1(
                optional_bytes(row, "last_approved_action_digest")?
                    .ok_or(PersonsPersistenceErrorV1::StateConflict)?,
            ),
        })),
    }
}

fn source_key(row: &sqlx::postgres::PgRow) -> Result<SourceLinkKeyV1, PersonsPersistenceErrorV1> {
    Ok(SourceLinkKeyV1 {
        integration_public_id: PublicIdV1(bytes(row, "integration_public_id")?),
        account_public_id: PublicIdV1(bytes(row, "account_public_id")?),
        provider_source_contact_public_id: PublicIdV1(bytes(
            row,
            "provider_source_contact_public_id",
        )?),
    })
}

fn timestamp(
    row: &sqlx::postgres::PgRow,
    seconds: &str,
    nanos: &str,
) -> Result<TimestampV1, PersonsPersistenceErrorV1> {
    Ok(TimestampV1 {
        unix_seconds: row.try_get(seconds).map_err(|_| storage())?,
        nanos: row.try_get(nanos).map_err(|_| storage())?,
    })
}

fn lifecycle(value: PersonLifecycleV1) -> i16 {
    match value {
        PersonLifecycleV1::Provisional => 1,
        PersonLifecycleV1::Active => 2,
        PersonLifecycleV1::Merged => 3,
        PersonLifecycleV1::Archived => 4,
    }
}

fn bounded<T>(rows: &[T]) -> Result<(), PersonsPersistenceErrorV1> {
    if rows.len() > usize::try_from(PERSONS_RECOVERY_ROW_LIMIT_V1).unwrap_or(0) {
        Err(PersonsPersistenceErrorV1::StateConflict)
    } else {
        Ok(())
    }
}

fn bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; N], PersonsPersistenceErrorV1> {
    let value: Vec<u8> = row.try_get(column).map_err(|_| storage())?;
    value
        .try_into()
        .map_err(|_| PersonsPersistenceErrorV1::StateConflict)
}

fn optional_bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<[u8; N]>, PersonsPersistenceErrorV1> {
    row.try_get::<Option<Vec<u8>>, _>(column)
        .map_err(|_| storage())?
        .map(|value| {
            value
                .try_into()
                .map_err(|_| PersonsPersistenceErrorV1::StateConflict)
        })
        .transpose()
}

fn i64_value(value: u64) -> Result<i64, PersonsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| PersonsPersistenceErrorV1::InvalidInput)
}

fn u64_value(value: i64) -> Result<u64, PersonsPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| PersonsPersistenceErrorV1::StateConflict)
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

fn storage() -> PersonsPersistenceErrorV1 {
    PersonsPersistenceErrorV1::StorageUnavailable
}
