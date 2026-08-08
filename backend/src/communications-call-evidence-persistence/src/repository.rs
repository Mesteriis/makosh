use makosh_communications_call_evidence_core::{
    CallDirectionV1, CallEvidenceApplyOutcomeV1, CallEvidenceCoreErrorV1, CallEvidenceProjectionV1,
    CallLifecycleStateV1, CallMediaKindV1, CallProviderProvenanceV1, CallTerminalDispositionV1,
    RecordCallEvidenceV1, apply_call_evidence_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};

const INBOX_APPLIED: i16 = 1;
const INBOX_DUPLICATE: i16 = 2;
const INBOX_STALE: i16 = 3;
const INBOX_REJECTED: i16 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidenceRejectionCodeV1 {
    InvalidPayload,
    IdentityConflict,
    RevisionConflict,
    StateRegression,
    TerminalConflict,
}

impl CallEvidenceRejectionCodeV1 {
    const fn code(self) -> i16 {
        match self {
            Self::InvalidPayload => 1,
            Self::IdentityConflict => 2,
            Self::RevisionConflict => 3,
            Self::StateRegression => 4,
            Self::TerminalConflict => 5,
        }
    }

    fn from_code(value: i16) -> Result<Self, CallEvidencePersistenceErrorV1> {
        match value {
            1 => Ok(Self::InvalidPayload),
            2 => Ok(Self::IdentityConflict),
            3 => Ok(Self::RevisionConflict),
            4 => Ok(Self::StateRegression),
            5 => Ok(Self::TerminalConflict),
            _ => Err(CallEvidencePersistenceErrorV1::InvalidRow),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidenceConsumeOutcomeV1 {
    Applied {
        canonical_revision: u64,
        realtime_sequence: u64,
    },
    Duplicate,
    Stale,
    Rejected(CallEvidenceRejectionCodeV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallEvidenceRealtimeRecordV1 {
    pub sequence: u64,
    pub call_evidence_id: [u8; 16],
    pub canonical_revision: u64,
    pub state: CallLifecycleStateV1,
    pub terminal_disposition: Option<CallTerminalDispositionV1>,
    pub observed_at_unix_seconds: i64,
    pub participant_display_label: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CallEvidenceListFilterV1 {
    pub provider: Option<CallProviderProvenanceV1>,
    pub direction: Option<CallDirectionV1>,
    pub media_kind: Option<CallMediaKindV1>,
    pub state: Option<CallLifecycleStateV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallEvidencePageV1 {
    pub items: Vec<CallEvidenceProjectionV1>,
    pub next_cursor: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidencePersistenceErrorV1 {
    InvalidInput,
    InboxHashConflict,
    InvalidRow,
    StorageUnavailable,
}

#[derive(Clone)]
pub struct CommunicationsCallEvidencePersistenceV1 {
    pool: PgPool,
}

impl CommunicationsCallEvidencePersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, CallEvidencePersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(CallEvidencePersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| CallEvidencePersistenceErrorV1::StorageUnavailable)?;
        let options = PgConnectOptions::new()
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
            .map_err(storage_error)?;
        Ok(Self { pool })
    }

    #[must_use]
    pub fn from_owner_local_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn verify_storage_ready(&self) -> Result<(), CallEvidencePersistenceErrorV1> {
        sqlx::query(
            "SELECT call_evidence_id \
             FROM makosh_data.communications_call_evidence_projection \
             WHERE FALSE",
        )
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    pub async fn consume(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        incoming: RecordCallEvidenceV1,
        observed_at_unix_seconds: i64,
        consumed_at_unix_seconds: i64,
    ) -> Result<CallEvidenceConsumeOutcomeV1, CallEvidencePersistenceErrorV1> {
        validate_consume_input(
            logical_owner_id,
            &message_id,
            &envelope_sha256,
            observed_at_unix_seconds,
            consumed_at_unix_seconds,
        )?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        if let Some(outcome) = existing_inbox_outcome(
            &mut transaction,
            logical_owner_id,
            &message_id,
            &envelope_sha256,
        )
        .await?
        {
            transaction.commit().await.map_err(storage_error)?;
            return Ok(outcome);
        }

        let current = load_projection_for_update(
            &mut transaction,
            logical_owner_id,
            &incoming.call_evidence_id,
        )
        .await?;
        let applied = apply_call_evidence_v1(current.as_ref(), incoming.clone());
        let (projection, outcome) = match applied {
            Ok(value) => value,
            Err(error) => {
                let rejection = rejection_code(error);
                insert_inbox(
                    &mut transaction,
                    logical_owner_id,
                    &message_id,
                    &envelope_sha256,
                    &incoming.call_evidence_id,
                    INBOX_REJECTED,
                    Some(rejection.code()),
                    None,
                    None,
                    consumed_at_unix_seconds,
                )
                .await?;
                transaction.commit().await.map_err(storage_error)?;
                return Ok(CallEvidenceConsumeOutcomeV1::Rejected(rejection));
            }
        };

        if outcome != CallEvidenceApplyOutcomeV1::Applied {
            let inbox_outcome = match outcome {
                CallEvidenceApplyOutcomeV1::Duplicate => INBOX_DUPLICATE,
                CallEvidenceApplyOutcomeV1::Stale => INBOX_STALE,
                CallEvidenceApplyOutcomeV1::Applied => unreachable!("handled above"),
            };
            insert_inbox(
                &mut transaction,
                logical_owner_id,
                &message_id,
                &envelope_sha256,
                &incoming.call_evidence_id,
                inbox_outcome,
                None,
                None,
                None,
                consumed_at_unix_seconds,
            )
            .await?;
            transaction.commit().await.map_err(storage_error)?;
            return Ok(match outcome {
                CallEvidenceApplyOutcomeV1::Duplicate => CallEvidenceConsumeOutcomeV1::Duplicate,
                CallEvidenceApplyOutcomeV1::Stale => CallEvidenceConsumeOutcomeV1::Stale,
                CallEvidenceApplyOutcomeV1::Applied => unreachable!("handled above"),
            });
        }

        persist_projection(
            &mut transaction,
            logical_owner_id,
            &projection,
            consumed_at_unix_seconds,
        )
        .await?;
        insert_history(
            &mut transaction,
            logical_owner_id,
            &projection,
            &message_id,
            &envelope_sha256,
            observed_at_unix_seconds,
        )
        .await?;
        let sequence = next_realtime_sequence(&mut transaction, logical_owner_id).await?;
        insert_realtime_frame(
            &mut transaction,
            logical_owner_id,
            sequence,
            &projection,
            observed_at_unix_seconds,
        )
        .await?;
        insert_inbox(
            &mut transaction,
            logical_owner_id,
            &message_id,
            &envelope_sha256,
            &projection.evidence.call_evidence_id,
            INBOX_APPLIED,
            None,
            Some(projection.canonical_revision),
            Some(sequence),
            consumed_at_unix_seconds,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(CallEvidenceConsumeOutcomeV1::Applied {
            canonical_revision: projection.canonical_revision,
            realtime_sequence: sequence,
        })
    }

    pub async fn get(
        &self,
        logical_owner_id: &str,
        call_evidence_id: [u8; 16],
    ) -> Result<Option<CallEvidenceProjectionV1>, CallEvidencePersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !valid_id(&call_evidence_id) {
            return Err(CallEvidencePersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT source_call_cursor_sha256, account_cursor_sha256, \
                    conversation_cursor_sha256, participant_cursor_sha256, provider, direction, \
                    media_kind, lifecycle_state, terminal_disposition, source_revision, \
                    canonical_revision, started_at_unix_seconds, connected_at_unix_seconds, \
                    ended_at_unix_seconds, duration_seconds, participant_display_label, \
                    payload_sha256 \
             FROM makosh_data.communications_call_evidence_projection \
             WHERE logical_owner_id = $1 AND call_evidence_id = $2",
        )
        .bind(logical_owner_id)
        .bind(call_evidence_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?;
        row.map(|row| projection_from_row(logical_owner_id, call_evidence_id, &row))
            .transpose()
    }

    pub async fn list(
        &self,
        logical_owner_id: &str,
        filter: CallEvidenceListFilterV1,
        limit: u16,
        cursor: &[u8],
    ) -> Result<CallEvidencePageV1, CallEvidencePersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || limit == 0 || limit > 100 {
            return Err(CallEvidencePersistenceErrorV1::InvalidInput);
        }
        let cursor = decode_list_cursor(cursor)?;
        let cursor_revision = cursor.map(|value| value.0);
        let cursor_id = cursor.map(|value| value.1.to_vec());
        let rows = sqlx::query(
            "SELECT call_evidence_id, source_call_cursor_sha256, account_cursor_sha256, \
                    conversation_cursor_sha256, participant_cursor_sha256, provider, direction, \
                    media_kind, lifecycle_state, terminal_disposition, source_revision, \
                    canonical_revision, started_at_unix_seconds, connected_at_unix_seconds, \
                    ended_at_unix_seconds, duration_seconds, participant_display_label, \
                    payload_sha256 \
             FROM makosh_data.communications_call_evidence_projection \
             WHERE logical_owner_id = $1 \
               AND ($2::SMALLINT IS NULL OR provider = $2) \
               AND ($3::SMALLINT IS NULL OR direction = $3) \
               AND ($4::SMALLINT IS NULL OR media_kind = $4) \
               AND ($5::SMALLINT IS NULL OR lifecycle_state = $5) \
               AND ($6::BIGINT IS NULL OR \
                    (canonical_revision, call_evidence_id) < ($6, $7)) \
             ORDER BY canonical_revision DESC, call_evidence_id DESC \
             LIMIT $8",
        )
        .bind(logical_owner_id)
        .bind(filter.provider.map(provider_code))
        .bind(filter.direction.map(direction_code))
        .bind(filter.media_kind.map(media_kind_code))
        .bind(filter.state.map(state_code))
        .bind(cursor_revision)
        .bind(cursor_id)
        .bind(i64::from(limit) + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        let mut items = rows
            .iter()
            .map(|row| {
                let call_evidence_id = bytes16(
                    row.try_get::<Vec<u8>, _>("call_evidence_id")
                        .map_err(storage_error)?,
                )?;
                projection_from_row(logical_owner_id, call_evidence_id, row)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > usize::from(limit);
        if has_more {
            items.pop();
        }
        let next_cursor = if has_more {
            items.last().map(encode_list_cursor).unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(CallEvidencePageV1 { items, next_cursor })
    }

    pub async fn replay(
        &self,
        logical_owner_id: &str,
        after_sequence: u64,
        limit: u32,
    ) -> Result<Vec<CallEvidenceRealtimeRecordV1>, CallEvidencePersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || limit == 0 || limit > 256 {
            return Err(CallEvidencePersistenceErrorV1::InvalidInput);
        }
        let after = i64::try_from(after_sequence)
            .map_err(|_| CallEvidencePersistenceErrorV1::InvalidInput)?;
        let rows = sqlx::query(
            "SELECT sequence, call_evidence_id, canonical_revision, lifecycle_state, \
                    terminal_disposition, observed_at_unix_seconds, participant_display_label \
             FROM makosh_data.communications_call_evidence_realtime_frames \
             WHERE logical_owner_id = $1 AND sequence > $2 \
             ORDER BY sequence ASC LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(after)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.iter().map(realtime_from_row).collect()
    }
}

async fn existing_inbox_outcome(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    message_id: &[u8; 16],
    envelope_sha256: &[u8; 32],
) -> Result<Option<CallEvidenceConsumeOutcomeV1>, CallEvidencePersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, outcome, rejection_code, canonical_revision, realtime_sequence \
         FROM makosh_data.communications_call_evidence_inbox \
         WHERE logical_owner_id = $1 AND message_id = $2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .fetch_optional(transaction.as_mut())
    .await
    .map_err(storage_error)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let existing_hash: Vec<u8> = row
        .try_get("envelope_sha256")
        .map_err(|_| CallEvidencePersistenceErrorV1::InvalidRow)?;
    if existing_hash.as_slice() != envelope_sha256 {
        return Err(CallEvidencePersistenceErrorV1::InboxHashConflict);
    }
    let outcome: i16 = row
        .try_get("outcome")
        .map_err(|_| CallEvidencePersistenceErrorV1::InvalidRow)?;
    match outcome {
        INBOX_APPLIED => Ok(Some(CallEvidenceConsumeOutcomeV1::Applied {
            canonical_revision: positive_u64(row.try_get("canonical_revision")?)?,
            realtime_sequence: positive_u64(row.try_get("realtime_sequence")?)?,
        })),
        INBOX_DUPLICATE => Ok(Some(CallEvidenceConsumeOutcomeV1::Duplicate)),
        INBOX_STALE => Ok(Some(CallEvidenceConsumeOutcomeV1::Stale)),
        INBOX_REJECTED => {
            let code: i16 = row
                .try_get("rejection_code")
                .map_err(|_| CallEvidencePersistenceErrorV1::InvalidRow)?;
            Ok(Some(CallEvidenceConsumeOutcomeV1::Rejected(
                CallEvidenceRejectionCodeV1::from_code(code)?,
            )))
        }
        _ => Err(CallEvidencePersistenceErrorV1::InvalidRow),
    }
}

async fn load_projection_for_update(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    call_evidence_id: &[u8; 16],
) -> Result<Option<CallEvidenceProjectionV1>, CallEvidencePersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT source_call_cursor_sha256, account_cursor_sha256, \
                conversation_cursor_sha256, participant_cursor_sha256, provider, direction, \
                media_kind, lifecycle_state, terminal_disposition, source_revision, \
                canonical_revision, started_at_unix_seconds, connected_at_unix_seconds, \
                ended_at_unix_seconds, duration_seconds, participant_display_label, \
                payload_sha256 \
         FROM makosh_data.communications_call_evidence_projection \
         WHERE logical_owner_id = $1 AND call_evidence_id = $2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(call_evidence_id.as_slice())
    .fetch_optional(transaction.as_mut())
    .await
    .map_err(storage_error)?;
    row.map(|row| projection_from_row(logical_owner_id, *call_evidence_id, &row))
        .transpose()
}

fn projection_from_row(
    logical_owner_id: &str,
    call_evidence_id: [u8; 16],
    row: &sqlx::postgres::PgRow,
) -> Result<CallEvidenceProjectionV1, CallEvidencePersistenceErrorV1> {
    let canonical_revision = positive_u64(row.try_get("canonical_revision")?)?;
    Ok(CallEvidenceProjectionV1 {
        evidence: RecordCallEvidenceV1 {
            call_evidence_id,
            logical_owner_id: logical_owner_id.to_owned(),
            source_call_cursor_sha256: bytes32(row.try_get("source_call_cursor_sha256")?)?,
            account_cursor_sha256: bytes32(row.try_get("account_cursor_sha256")?)?,
            conversation_cursor_sha256: optional_bytes32(
                row.try_get("conversation_cursor_sha256")?,
            )?,
            participant_cursor_sha256: optional_bytes32(row.try_get("participant_cursor_sha256")?)?,
            provider: provider_from_code(row.try_get("provider")?)?,
            direction: direction_from_code(row.try_get("direction")?)?,
            media_kind: media_kind_from_code(row.try_get("media_kind")?)?,
            state: state_from_code(row.try_get("lifecycle_state")?)?,
            terminal_disposition: terminal_from_code(row.try_get("terminal_disposition")?)?,
            source_revision: positive_u64(row.try_get("source_revision")?)?,
            started_at_unix_seconds: row.try_get("started_at_unix_seconds")?,
            connected_at_unix_seconds: row.try_get("connected_at_unix_seconds")?,
            ended_at_unix_seconds: row.try_get("ended_at_unix_seconds")?,
            duration_seconds: optional_u64(row.try_get("duration_seconds")?)?,
            participant_display_label: row.try_get("participant_display_label")?,
            payload_sha256: bytes32(row.try_get("payload_sha256")?)?,
        },
        canonical_revision,
    })
}

async fn persist_projection(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    projection: &CallEvidenceProjectionV1,
    updated_at_unix_seconds: i64,
) -> Result<(), CallEvidencePersistenceErrorV1> {
    let evidence = &projection.evidence;
    sqlx::query(
        "INSERT INTO makosh_data.communications_call_evidence_projection ( \
             logical_owner_id, call_evidence_id, source_call_cursor_sha256, \
             account_cursor_sha256, conversation_cursor_sha256, participant_cursor_sha256, \
             provider, direction, media_kind, lifecycle_state, terminal_disposition, \
             source_revision, canonical_revision, started_at_unix_seconds, \
             connected_at_unix_seconds, ended_at_unix_seconds, duration_seconds, \
             participant_display_label, payload_sha256, updated_at_unix_seconds \
         ) VALUES ( \
             $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, \
             $17, $18, $19, $20 \
         ) \
         ON CONFLICT (logical_owner_id, call_evidence_id) DO UPDATE SET \
             conversation_cursor_sha256 = EXCLUDED.conversation_cursor_sha256, \
             participant_cursor_sha256 = EXCLUDED.participant_cursor_sha256, \
             lifecycle_state = EXCLUDED.lifecycle_state, \
             terminal_disposition = EXCLUDED.terminal_disposition, \
             source_revision = EXCLUDED.source_revision, \
             canonical_revision = EXCLUDED.canonical_revision, \
             started_at_unix_seconds = EXCLUDED.started_at_unix_seconds, \
             connected_at_unix_seconds = EXCLUDED.connected_at_unix_seconds, \
             ended_at_unix_seconds = EXCLUDED.ended_at_unix_seconds, \
             duration_seconds = EXCLUDED.duration_seconds, \
             participant_display_label = EXCLUDED.participant_display_label, \
             payload_sha256 = EXCLUDED.payload_sha256, \
             updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds",
    )
    .bind(logical_owner_id)
    .bind(evidence.call_evidence_id.as_slice())
    .bind(evidence.source_call_cursor_sha256.as_slice())
    .bind(evidence.account_cursor_sha256.as_slice())
    .bind(
        evidence
            .conversation_cursor_sha256
            .as_ref()
            .map(<[u8; 32]>::as_slice),
    )
    .bind(
        evidence
            .participant_cursor_sha256
            .as_ref()
            .map(<[u8; 32]>::as_slice),
    )
    .bind(provider_code(evidence.provider))
    .bind(direction_code(evidence.direction))
    .bind(media_kind_code(evidence.media_kind))
    .bind(state_code(evidence.state))
    .bind(evidence.terminal_disposition.map(terminal_code))
    .bind(positive_i64(evidence.source_revision)?)
    .bind(positive_i64(projection.canonical_revision)?)
    .bind(evidence.started_at_unix_seconds)
    .bind(evidence.connected_at_unix_seconds)
    .bind(evidence.ended_at_unix_seconds)
    .bind(optional_i64(evidence.duration_seconds)?)
    .bind(evidence.participant_display_label.as_deref())
    .bind(evidence.payload_sha256.as_slice())
    .bind(updated_at_unix_seconds)
    .execute(transaction.as_mut())
    .await
    .map_err(storage_error)?;
    Ok(())
}

async fn insert_history(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    projection: &CallEvidenceProjectionV1,
    message_id: &[u8; 16],
    envelope_sha256: &[u8; 32],
    observed_at_unix_seconds: i64,
) -> Result<(), CallEvidencePersistenceErrorV1> {
    let evidence = &projection.evidence;
    sqlx::query(
        "INSERT INTO makosh_data.communications_call_evidence_history ( \
             logical_owner_id, call_evidence_id, canonical_revision, source_revision, \
             message_id, envelope_sha256, lifecycle_state, terminal_disposition, \
             observed_at_unix_seconds \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(logical_owner_id)
    .bind(evidence.call_evidence_id.as_slice())
    .bind(positive_i64(projection.canonical_revision)?)
    .bind(positive_i64(evidence.source_revision)?)
    .bind(message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(state_code(evidence.state))
    .bind(evidence.terminal_disposition.map(terminal_code))
    .bind(observed_at_unix_seconds)
    .execute(transaction.as_mut())
    .await
    .map_err(storage_error)?;
    Ok(())
}

async fn next_realtime_sequence(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
) -> Result<u64, CallEvidencePersistenceErrorV1> {
    let sequence: i64 = sqlx::query_scalar(
        "INSERT INTO makosh_data.communications_call_evidence_realtime_sequence ( \
             logical_owner_id, next_sequence \
         ) VALUES ($1, 2) \
         ON CONFLICT (logical_owner_id) DO UPDATE SET \
             next_sequence = makosh_data.communications_call_evidence_realtime_sequence.next_sequence + 1 \
         RETURNING next_sequence - 1",
    )
    .bind(logical_owner_id)
    .fetch_one(transaction.as_mut())
    .await
    .map_err(storage_error)?;
    positive_u64(sequence)
}

async fn insert_realtime_frame(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    sequence: u64,
    projection: &CallEvidenceProjectionV1,
    observed_at_unix_seconds: i64,
) -> Result<(), CallEvidencePersistenceErrorV1> {
    let evidence = &projection.evidence;
    sqlx::query(
        "INSERT INTO makosh_data.communications_call_evidence_realtime_frames ( \
             logical_owner_id, sequence, call_evidence_id, canonical_revision, \
             lifecycle_state, terminal_disposition, observed_at_unix_seconds, \
             participant_display_label \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(logical_owner_id)
    .bind(positive_i64(sequence)?)
    .bind(evidence.call_evidence_id.as_slice())
    .bind(positive_i64(projection.canonical_revision)?)
    .bind(state_code(evidence.state))
    .bind(evidence.terminal_disposition.map(terminal_code))
    .bind(observed_at_unix_seconds)
    .bind(evidence.participant_display_label.as_deref())
    .execute(transaction.as_mut())
    .await
    .map_err(storage_error)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_inbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    message_id: &[u8; 16],
    envelope_sha256: &[u8; 32],
    call_evidence_id: &[u8; 16],
    outcome: i16,
    rejection_code: Option<i16>,
    canonical_revision: Option<u64>,
    realtime_sequence: Option<u64>,
    consumed_at_unix_seconds: i64,
) -> Result<(), CallEvidencePersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.communications_call_evidence_inbox ( \
             logical_owner_id, message_id, envelope_sha256, call_evidence_id, outcome, \
             rejection_code, canonical_revision, realtime_sequence, consumed_at_unix_seconds \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(call_evidence_id.as_slice())
    .bind(outcome)
    .bind(rejection_code)
    .bind(optional_positive_i64(canonical_revision)?)
    .bind(optional_positive_i64(realtime_sequence)?)
    .bind(consumed_at_unix_seconds)
    .execute(transaction.as_mut())
    .await
    .map_err(storage_error)?;
    Ok(())
}

fn realtime_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<CallEvidenceRealtimeRecordV1, CallEvidencePersistenceErrorV1> {
    Ok(CallEvidenceRealtimeRecordV1 {
        sequence: positive_u64(row.try_get("sequence")?)?,
        call_evidence_id: bytes16(row.try_get("call_evidence_id")?)?,
        canonical_revision: positive_u64(row.try_get("canonical_revision")?)?,
        state: state_from_code(row.try_get("lifecycle_state")?)?,
        terminal_disposition: terminal_from_code(row.try_get("terminal_disposition")?)?,
        observed_at_unix_seconds: row.try_get("observed_at_unix_seconds")?,
        participant_display_label: row.try_get("participant_display_label")?,
    })
}

fn validate_consume_input(
    logical_owner_id: &str,
    message_id: &[u8; 16],
    envelope_sha256: &[u8; 32],
    observed_at_unix_seconds: i64,
    consumed_at_unix_seconds: i64,
) -> Result<(), CallEvidencePersistenceErrorV1> {
    if !valid_owner(logical_owner_id)
        || !valid_id(message_id)
        || envelope_sha256.iter().all(|byte| *byte == 0)
        || !valid_timestamp(observed_at_unix_seconds)
        || !valid_timestamp(consumed_at_unix_seconds)
    {
        return Err(CallEvidencePersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn valid_owner(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}

fn valid_id(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_timestamp(value: i64) -> bool {
    (-62_135_596_800..=253_402_300_799).contains(&value)
}

fn decode_list_cursor(
    cursor: &[u8],
) -> Result<Option<(i64, [u8; 16])>, CallEvidencePersistenceErrorV1> {
    if cursor.is_empty() {
        return Ok(None);
    }
    if cursor.len() != 24 {
        return Err(CallEvidencePersistenceErrorV1::InvalidInput);
    }
    let canonical_revision = u64::from_be_bytes(
        cursor[..8]
            .try_into()
            .map_err(|_| CallEvidencePersistenceErrorV1::InvalidInput)?,
    );
    let call_evidence_id = cursor[8..]
        .try_into()
        .map_err(|_| CallEvidencePersistenceErrorV1::InvalidInput)?;
    if canonical_revision == 0 || !valid_id(&call_evidence_id) {
        return Err(CallEvidencePersistenceErrorV1::InvalidInput);
    }
    Ok(Some((positive_i64(canonical_revision)?, call_evidence_id)))
}

fn encode_list_cursor(projection: &CallEvidenceProjectionV1) -> Vec<u8> {
    let mut cursor = Vec::with_capacity(24);
    cursor.extend_from_slice(&projection.canonical_revision.to_be_bytes());
    cursor.extend_from_slice(&projection.evidence.call_evidence_id);
    cursor
}

fn bytes16(value: Vec<u8>) -> Result<[u8; 16], CallEvidencePersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CallEvidencePersistenceErrorV1::InvalidRow)
}

fn bytes32(value: Vec<u8>) -> Result<[u8; 32], CallEvidencePersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CallEvidencePersistenceErrorV1::InvalidRow)
}

fn optional_bytes32(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; 32]>, CallEvidencePersistenceErrorV1> {
    value.map(bytes32).transpose()
}

fn positive_i64(value: u64) -> Result<i64, CallEvidencePersistenceErrorV1> {
    i64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(CallEvidencePersistenceErrorV1::InvalidInput)
}

fn positive_u64(value: i64) -> Result<u64, CallEvidencePersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(CallEvidencePersistenceErrorV1::InvalidRow)
}

fn optional_i64(value: Option<u64>) -> Result<Option<i64>, CallEvidencePersistenceErrorV1> {
    value
        .map(|value| i64::try_from(value).map_err(|_| CallEvidencePersistenceErrorV1::InvalidInput))
        .transpose()
}

fn optional_positive_i64(
    value: Option<u64>,
) -> Result<Option<i64>, CallEvidencePersistenceErrorV1> {
    value.map(positive_i64).transpose()
}

fn optional_u64(value: Option<i64>) -> Result<Option<u64>, CallEvidencePersistenceErrorV1> {
    value
        .map(|value| u64::try_from(value).map_err(|_| CallEvidencePersistenceErrorV1::InvalidRow))
        .transpose()
}

const fn provider_code(value: CallProviderProvenanceV1) -> i16 {
    match value {
        CallProviderProvenanceV1::Telegram => 1,
        CallProviderProvenanceV1::WhatsAppWeb => 2,
        CallProviderProvenanceV1::Zoom => 3,
        CallProviderProvenanceV1::YandexTelemost => 4,
    }
}

fn provider_from_code(
    value: i16,
) -> Result<CallProviderProvenanceV1, CallEvidencePersistenceErrorV1> {
    match value {
        1 => Ok(CallProviderProvenanceV1::Telegram),
        2 => Ok(CallProviderProvenanceV1::WhatsAppWeb),
        3 => Ok(CallProviderProvenanceV1::Zoom),
        4 => Ok(CallProviderProvenanceV1::YandexTelemost),
        _ => Err(CallEvidencePersistenceErrorV1::InvalidRow),
    }
}

const fn direction_code(value: CallDirectionV1) -> i16 {
    match value {
        CallDirectionV1::Incoming => 1,
        CallDirectionV1::Outgoing => 2,
        CallDirectionV1::Unknown => 3,
    }
}

fn direction_from_code(value: i16) -> Result<CallDirectionV1, CallEvidencePersistenceErrorV1> {
    match value {
        1 => Ok(CallDirectionV1::Incoming),
        2 => Ok(CallDirectionV1::Outgoing),
        3 => Ok(CallDirectionV1::Unknown),
        _ => Err(CallEvidencePersistenceErrorV1::InvalidRow),
    }
}

const fn media_kind_code(value: CallMediaKindV1) -> i16 {
    match value {
        CallMediaKindV1::OneToOneAudio => 1,
        CallMediaKindV1::Meeting => 2,
    }
}

fn media_kind_from_code(value: i16) -> Result<CallMediaKindV1, CallEvidencePersistenceErrorV1> {
    match value {
        1 => Ok(CallMediaKindV1::OneToOneAudio),
        2 => Ok(CallMediaKindV1::Meeting),
        _ => Err(CallEvidencePersistenceErrorV1::InvalidRow),
    }
}

const fn state_code(value: CallLifecycleStateV1) -> i16 {
    match value {
        CallLifecycleStateV1::Observed => 1,
        CallLifecycleStateV1::Ringing => 2,
        CallLifecycleStateV1::Connecting => 3,
        CallLifecycleStateV1::Active => 4,
        CallLifecycleStateV1::Ended => 5,
    }
}

fn state_from_code(value: i16) -> Result<CallLifecycleStateV1, CallEvidencePersistenceErrorV1> {
    match value {
        1 => Ok(CallLifecycleStateV1::Observed),
        2 => Ok(CallLifecycleStateV1::Ringing),
        3 => Ok(CallLifecycleStateV1::Connecting),
        4 => Ok(CallLifecycleStateV1::Active),
        5 => Ok(CallLifecycleStateV1::Ended),
        _ => Err(CallEvidencePersistenceErrorV1::InvalidRow),
    }
}

const fn terminal_code(value: CallTerminalDispositionV1) -> i16 {
    match value {
        CallTerminalDispositionV1::Completed => 1,
        CallTerminalDispositionV1::Missed => 2,
        CallTerminalDispositionV1::Declined => 3,
        CallTerminalDispositionV1::Disconnected => 4,
        CallTerminalDispositionV1::Failed => 5,
        CallTerminalDispositionV1::Canceled => 6,
    }
}

fn terminal_from_code(
    value: Option<i16>,
) -> Result<Option<CallTerminalDispositionV1>, CallEvidencePersistenceErrorV1> {
    value
        .map(|value| match value {
            1 => Ok(CallTerminalDispositionV1::Completed),
            2 => Ok(CallTerminalDispositionV1::Missed),
            3 => Ok(CallTerminalDispositionV1::Declined),
            4 => Ok(CallTerminalDispositionV1::Disconnected),
            5 => Ok(CallTerminalDispositionV1::Failed),
            6 => Ok(CallTerminalDispositionV1::Canceled),
            _ => Err(CallEvidencePersistenceErrorV1::InvalidRow),
        })
        .transpose()
}

const fn rejection_code(error: CallEvidenceCoreErrorV1) -> CallEvidenceRejectionCodeV1 {
    match error {
        CallEvidenceCoreErrorV1::InvalidPayload => CallEvidenceRejectionCodeV1::InvalidPayload,
        CallEvidenceCoreErrorV1::IdentityConflict => CallEvidenceRejectionCodeV1::IdentityConflict,
        CallEvidenceCoreErrorV1::RevisionConflict => CallEvidenceRejectionCodeV1::RevisionConflict,
        CallEvidenceCoreErrorV1::StateRegression => CallEvidenceRejectionCodeV1::StateRegression,
        CallEvidenceCoreErrorV1::TerminalConflict => CallEvidenceRejectionCodeV1::TerminalConflict,
    }
}

fn storage_error(_: sqlx::Error) -> CallEvidencePersistenceErrorV1 {
    CallEvidencePersistenceErrorV1::StorageUnavailable
}

impl From<sqlx::Error> for CallEvidencePersistenceErrorV1 {
    fn from(_: sqlx::Error) -> Self {
        Self::InvalidRow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_codes_are_closed_and_provider_neutral() {
        for (code, expected) in [
            (1, CallLifecycleStateV1::Observed),
            (2, CallLifecycleStateV1::Ringing),
            (3, CallLifecycleStateV1::Connecting),
            (4, CallLifecycleStateV1::Active),
            (5, CallLifecycleStateV1::Ended),
        ] {
            assert_eq!(state_from_code(code), Ok(expected));
            assert_eq!(state_code(expected), code);
        }
        assert_eq!(
            state_from_code(6),
            Err(CallEvidencePersistenceErrorV1::InvalidRow)
        );
    }

    #[test]
    fn input_validation_rejects_unbounded_or_zero_identity() {
        assert_eq!(
            validate_consume_input("owner-1", &[1; 16], &[2; 32], 1_700_000_000, 1_700_000_001),
            Ok(())
        );
        assert_eq!(
            validate_consume_input("", &[1; 16], &[2; 32], 1_700_000_000, 1_700_000_001),
            Err(CallEvidencePersistenceErrorV1::InvalidInput)
        );
        assert_eq!(
            validate_consume_input("owner-1", &[0; 16], &[2; 32], 1_700_000_000, 1_700_000_001),
            Err(CallEvidencePersistenceErrorV1::InvalidInput)
        );
    }

    #[test]
    fn list_cursor_is_exact_opaque_and_round_trips() {
        let projection = CallEvidenceProjectionV1 {
            evidence: RecordCallEvidenceV1 {
                call_evidence_id: [7; 16],
                logical_owner_id: "owner-1".to_owned(),
                source_call_cursor_sha256: [1; 32],
                account_cursor_sha256: [2; 32],
                conversation_cursor_sha256: None,
                participant_cursor_sha256: None,
                provider: CallProviderProvenanceV1::Telegram,
                direction: CallDirectionV1::Incoming,
                media_kind: CallMediaKindV1::OneToOneAudio,
                state: CallLifecycleStateV1::Ringing,
                terminal_disposition: None,
                source_revision: 1,
                started_at_unix_seconds: Some(41),
                connected_at_unix_seconds: None,
                ended_at_unix_seconds: None,
                duration_seconds: None,
                participant_display_label: None,
                payload_sha256: [8; 32],
            },
            canonical_revision: 42,
        };
        let cursor = encode_list_cursor(&projection);
        assert_eq!(cursor.len(), 24);
        assert_eq!(decode_list_cursor(&cursor), Ok(Some((42, [7; 16]))));
        assert_eq!(
            decode_list_cursor(&cursor[..23]),
            Err(CallEvidencePersistenceErrorV1::InvalidInput)
        );
    }
}
