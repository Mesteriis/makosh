use makosh_knowledge_core::{
    KnowledgeLifecycleErrorV1, KnowledgeLifecycleStateV1, KnowledgeNoteOriginV1,
    KnowledgeNoteProvenanceV1, KnowledgeNoteRecordV1, KnowledgeNoteTimestampV1,
    KnowledgeSourceStateV1, KnowledgeSourceV1, add_knowledge_source_v1,
    create_manual_knowledge_note_v1, remove_knowledge_source_v1, set_knowledge_note_state_v1,
    update_knowledge_note_content_v1, validate_knowledge_note_record_v1,
};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    KnowledgeLifecycleCommitV1, KnowledgeLifecycleMutationV1, KnowledgeLifecycleOperationOutcomeV1,
    KnowledgeLifecycleOperationV1, KnowledgePersistenceErrorV1, KnowledgePersistenceV1,
    model::{valid_lifecycle_commit, valid_lifecycle_operation},
};

impl KnowledgePersistenceV1 {
    pub async fn load_lifecycle_operation_replay(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
        request_sha256: [u8; 32],
        request_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, KnowledgePersistenceErrorV1> {
        if !crate::model::valid_identity(logical_owner_id)
            || operation_id.iter().all(|byte| *byte == 0)
            || request_sha256.iter().all(|byte| *byte == 0)
            || request_bytes.is_empty()
            || request_bytes.len() > crate::model::KNOWLEDGE_MAX_CLIENT_MESSAGE_BYTES_V1
            || Sha256::digest(request_bytes).as_slice() != request_sha256
        {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let response = load_operation_replay_raw(
            &mut transaction,
            logical_owner_id,
            operation_id,
            request_sha256,
            request_bytes,
            None,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(response)
    }

    pub async fn apply_lifecycle_operation<F>(
        &self,
        input: KnowledgeLifecycleOperationV1,
        build_commit: F,
    ) -> Result<KnowledgeLifecycleOperationOutcomeV1, KnowledgePersistenceErrorV1>
    where
        F: FnOnce(
            &KnowledgeNoteRecordV1,
        ) -> Result<KnowledgeLifecycleCommitV1, KnowledgePersistenceErrorV1>,
    {
        if !valid_lifecycle_operation(&input) {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        if let Some(response) = load_operation_replay_raw(
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
            return Ok(KnowledgeLifecycleOperationOutcomeV1::Replayed {
                response_bytes: response,
            });
        }

        let mut note = match &input.mutation {
            KnowledgeLifecycleMutationV1::Create(draft) => {
                if draft.logical_owner_id != input.logical_owner_id
                    || draft.operation_id != input.operation_id
                {
                    return Err(KnowledgePersistenceErrorV1::InvalidInput);
                }
                create_manual_knowledge_note_v1(draft.clone()).map_err(core_error)?
            }
            mutation => load_note(
                &mut transaction,
                &input.logical_owner_id,
                mutation_note_id(mutation),
                true,
            )
            .await?
            .ok_or(KnowledgePersistenceErrorV1::NotFound)?,
        };

        match &input.mutation {
            KnowledgeLifecycleMutationV1::Create(_) => {}
            KnowledgeLifecycleMutationV1::Update {
                expected_revision,
                title,
                body,
                changed_at,
                ..
            } => update_knowledge_note_content_v1(
                &mut note,
                *expected_revision,
                title.clone(),
                body.clone(),
                *changed_at,
            )
            .map_err(core_error)?,
            KnowledgeLifecycleMutationV1::SetState {
                expected_revision,
                state,
                changed_at,
                ..
            } => set_knowledge_note_state_v1(&mut note, *expected_revision, *state, *changed_at)
                .map_err(core_error)?,
            KnowledgeLifecycleMutationV1::AddSource {
                expected_revision,
                source_owner_id,
                source_record_id,
                source_revision,
                evidence_digest,
                changed_at,
                ..
            } => {
                add_knowledge_source_v1(
                    &mut note,
                    *expected_revision,
                    source_owner_id.clone(),
                    *source_record_id,
                    *source_revision,
                    *evidence_digest,
                    *changed_at,
                )
                .map_err(core_error)?;
            }
            KnowledgeLifecycleMutationV1::RemoveSource {
                expected_revision,
                source_id,
                changed_at,
                ..
            } => remove_knowledge_source_v1(&mut note, *expected_revision, *source_id, *changed_at)
                .map_err(core_error)?,
        }
        validate_knowledge_note_record_v1(&note).map_err(core_error)?;
        persist_note(
            &mut transaction,
            &note,
            matches!(input.mutation, KnowledgeLifecycleMutationV1::Create(_)),
        )
        .await?;
        persist_sources(&mut transaction, &note).await?;

        let commit = build_commit(&note)?;
        if !valid_lifecycle_commit(&commit) {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        insert_event(
            &mut transaction,
            &input.logical_owner_id,
            &commit,
            input.received_at_unix_millis,
        )
        .await?;
        insert_operation(&mut transaction, &input, &note, &commit).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(KnowledgeLifecycleOperationOutcomeV1::Applied {
            note: Box::new(note),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_lifecycle_note(
        &self,
        logical_owner_id: &str,
        note_id: [u8; 16],
    ) -> Result<Option<KnowledgeNoteRecordV1>, KnowledgePersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let note = load_note(&mut transaction, logical_owner_id, note_id, false).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(note)
    }

    pub async fn list_lifecycle_notes(
        &self,
        logical_owner_id: &str,
        after_note_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<KnowledgeNoteRecordV1>, KnowledgePersistenceErrorV1> {
        self.query_lifecycle_notes(logical_owner_id, None, after_note_id, limit)
            .await
    }

    pub async fn search_lifecycle_notes(
        &self,
        logical_owner_id: &str,
        query: &str,
        after_note_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<KnowledgeNoteRecordV1>, KnowledgePersistenceErrorV1> {
        if query.trim().is_empty()
            || query.chars().count() > 200
            || query.chars().any(char::is_control)
        {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        self.query_lifecycle_notes(logical_owner_id, Some(query.trim()), after_note_id, limit)
            .await
    }

    async fn query_lifecycle_notes(
        &self,
        logical_owner_id: &str,
        query: Option<&str>,
        after_note_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<KnowledgeNoteRecordV1>, KnowledgePersistenceErrorV1> {
        if !crate::model::valid_identity(logical_owner_id) || limit == 0 || limit > 201 {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = if let Some(query) = query {
            sqlx::query(
                "SELECT note_id FROM makosh_data.knowledge_state \
                 WHERE logical_owner_id=$1 AND ($2::bytea IS NULL OR note_id>$2) \
                 AND (title ILIKE '%' || $3 || '%' OR excerpt ILIKE '%' || $3 || '%') \
                 ORDER BY note_id LIMIT $4",
            )
            .bind(logical_owner_id)
            .bind(after_note_id.map(|value| value.to_vec()))
            .bind(query)
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
        } else {
            sqlx::query(
                "SELECT note_id FROM makosh_data.knowledge_state \
                 WHERE logical_owner_id=$1 AND ($2::bytea IS NULL OR note_id>$2) \
                 ORDER BY note_id LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after_note_id.map(|value| value.to_vec()))
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
        };
        let mut notes = Vec::with_capacity(rows.len());
        for row in rows {
            let note_id = fixed::<16>(row.try_get("note_id").map_err(storage)?)?;
            notes.push(
                load_note(&mut transaction, logical_owner_id, note_id, false)
                    .await?
                    .ok_or(KnowledgePersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(notes)
    }

    pub async fn list_lifecycle_sources(
        &self,
        logical_owner_id: &str,
        note_id: [u8; 16],
        after_source_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<KnowledgeSourceV1>, KnowledgePersistenceErrorV1> {
        if !crate::model::valid_identity(logical_owner_id) || limit == 0 || limit > 201 {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT source_id, source_owner_id, source_record_id, source_revision, \
             evidence_digest, source_state, updated_at_note_revision, created_at_unix_seconds, \
             created_at_nanos, updated_at_unix_seconds, updated_at_nanos \
             FROM makosh_data.knowledge_sources WHERE logical_owner_id=$1 AND note_id=$2 \
             AND ($3::bytea IS NULL OR source_id>$3) ORDER BY source_id LIMIT $4",
        )
        .bind(logical_owner_id)
        .bind(note_id.as_slice())
        .bind(after_source_id.map(|value| value.to_vec()))
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let sources = rows
            .iter()
            .map(decode_source)
            .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await.map_err(storage)?;
        Ok(sources)
    }
}

async fn load_operation_replay_raw(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    request_bytes: &[u8],
    operation_kind: Option<i16>,
) -> Result<Option<Vec<u8>>, KnowledgePersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind, request_sha256, request_bytes, response_sha256, response_bytes \
         FROM makosh_data.knowledge_client_operations \
         WHERE logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let stored_kind: i16 = row.try_get("operation_kind").map_err(storage)?;
    let stored_request_sha = fixed::<32>(row.try_get("request_sha256").map_err(storage)?)?;
    let stored_request_bytes: Vec<u8> = row.try_get("request_bytes").map_err(storage)?;
    let response_sha = fixed::<32>(row.try_get("response_sha256").map_err(storage)?)?;
    let response_bytes: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
    if operation_kind.is_some_and(|value| value != stored_kind)
        || stored_request_sha != request_sha256
        || stored_request_bytes != request_bytes
        || Sha256::digest(&response_bytes).as_slice() != response_sha
    {
        return Err(KnowledgePersistenceErrorV1::OperationConflict);
    }
    Ok(Some(response_bytes))
}

async fn load_note(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    note_id: [u8; 16],
    for_update: bool,
) -> Result<Option<KnowledgeNoteRecordV1>, KnowledgePersistenceErrorV1> {
    let sql = if for_update {
        "SELECT title, excerpt, lifecycle_state, origin_kind, note_revision, \
         approved_candidate_id, candidate_digest, source_evidence_id, source_evidence_revision, \
         review_id, decision_revision, decided_by_owner_device_id, created_at_unix_seconds, \
         created_at_nanos, updated_at_unix_seconds, updated_at_nanos \
         FROM makosh_data.knowledge_state WHERE logical_owner_id=$1 AND note_id=$2 FOR UPDATE"
    } else {
        "SELECT title, excerpt, lifecycle_state, origin_kind, note_revision, \
         approved_candidate_id, candidate_digest, source_evidence_id, source_evidence_revision, \
         review_id, decision_revision, decided_by_owner_device_id, created_at_unix_seconds, \
         created_at_nanos, updated_at_unix_seconds, updated_at_nanos \
         FROM makosh_data.knowledge_state WHERE logical_owner_id=$1 AND note_id=$2"
    };
    let Some(row) = sqlx::query(sql)
        .bind(logical_owner_id)
        .bind(note_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
    else {
        return Ok(None);
    };
    let origin = decode_origin(row.try_get("origin_kind").map_err(storage)?)?;
    let reviewed_provenance = decode_provenance(&row, origin)?;
    let source_rows = sqlx::query(
        "SELECT source_id, source_owner_id, source_record_id, source_revision, evidence_digest, \
         source_state, updated_at_note_revision, created_at_unix_seconds, created_at_nanos, \
         updated_at_unix_seconds, updated_at_nanos FROM makosh_data.knowledge_sources \
         WHERE logical_owner_id=$1 AND note_id=$2 ORDER BY source_id",
    )
    .bind(logical_owner_id)
    .bind(note_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let note = KnowledgeNoteRecordV1 {
        note_id,
        logical_owner_id: logical_owner_id.to_owned(),
        title: row.try_get("title").map_err(storage)?,
        body: row.try_get("excerpt").map_err(storage)?,
        state: decode_state(row.try_get("lifecycle_state").map_err(storage)?)?,
        origin,
        note_revision: positive_u64(row.try_get("note_revision").map_err(storage)?)?,
        reviewed_provenance,
        sources: source_rows
            .iter()
            .map(decode_source)
            .collect::<Result<Vec<_>, _>>()?,
        created_at: timestamp(&row, "created_at_unix_seconds", "created_at_nanos")?,
        updated_at: timestamp(&row, "updated_at_unix_seconds", "updated_at_nanos")?,
    };
    validate_knowledge_note_record_v1(&note).map_err(core_error)?;
    Ok(Some(note))
}

async fn persist_note(
    transaction: &mut Transaction<'_, Postgres>,
    note: &KnowledgeNoteRecordV1,
    create: bool,
) -> Result<(), KnowledgePersistenceErrorV1> {
    if create {
        let result = sqlx::query(
            "INSERT INTO makosh_data.knowledge_state (logical_owner_id,note_id,title,excerpt, \
             topic_hints,source_basis,confidence_basis_points,status,note_revision, \
             approved_candidate_id,candidate_digest,source_evidence_id,source_evidence_revision, \
             review_id,decision_revision,decided_by_owner_device_id,created_at_unix_seconds, \
             created_at_nanos,updated_at_unix_seconds,updated_at_nanos,lifecycle_state,origin_kind) \
             VALUES ($1,$2,$3,$4,NULL,NULL,NULL,1,$5,NULL,NULL,NULL,NULL,NULL,NULL,NULL,$6,$7,$8,$9,$10,2)",
        )
        .bind(&note.logical_owner_id)
        .bind(note.note_id.as_slice())
        .bind(&note.title)
        .bind(&note.body)
        .bind(i64_value(note.note_revision)?)
        .bind(note.created_at.unix_seconds)
        .bind(note.created_at.nanos)
        .bind(note.updated_at.unix_seconds)
        .bind(note.updated_at.nanos)
        .bind(encode_state(note.state))
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgePersistenceErrorV1::KnowledgeNoteConflict);
        }
    } else {
        let result = sqlx::query(
            "UPDATE makosh_data.knowledge_state SET title=$3,excerpt=$4,lifecycle_state=$5, \
             note_revision=$6,updated_at_unix_seconds=$7,updated_at_nanos=$8 \
             WHERE logical_owner_id=$1 AND note_id=$2",
        )
        .bind(&note.logical_owner_id)
        .bind(note.note_id.as_slice())
        .bind(&note.title)
        .bind(&note.body)
        .bind(encode_state(note.state))
        .bind(i64_value(note.note_revision)?)
        .bind(note.updated_at.unix_seconds)
        .bind(note.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgePersistenceErrorV1::RevisionConflict);
        }
    }
    Ok(())
}

async fn persist_sources(
    transaction: &mut Transaction<'_, Postgres>,
    note: &KnowledgeNoteRecordV1,
) -> Result<(), KnowledgePersistenceErrorV1> {
    for source in &note.sources {
        sqlx::query(
            "INSERT INTO makosh_data.knowledge_sources (logical_owner_id,note_id,source_id, \
             source_owner_id,source_record_id,source_revision,evidence_digest,source_state, \
             updated_at_note_revision,created_at_unix_seconds,created_at_nanos, \
             updated_at_unix_seconds,updated_at_nanos) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) \
             ON CONFLICT (logical_owner_id,note_id,source_id) DO UPDATE SET \
             source_revision=EXCLUDED.source_revision,evidence_digest=EXCLUDED.evidence_digest, \
             source_state=EXCLUDED.source_state,updated_at_note_revision=EXCLUDED.updated_at_note_revision, \
             updated_at_unix_seconds=EXCLUDED.updated_at_unix_seconds,updated_at_nanos=EXCLUDED.updated_at_nanos \
             WHERE makosh_data.knowledge_sources.source_owner_id=EXCLUDED.source_owner_id \
             AND makosh_data.knowledge_sources.source_record_id=EXCLUDED.source_record_id",
        )
        .bind(&note.logical_owner_id)
        .bind(note.note_id.as_slice())
        .bind(source.source_id.as_slice())
        .bind(&source.source_owner_id)
        .bind(source.source_record_id.as_slice())
        .bind(i64_value(source.source_revision)?)
        .bind(source.evidence_digest.as_slice())
        .bind(encode_source_state(source.state))
        .bind(i64_value(source.updated_at_note_revision)?)
        .bind(source.created_at.unix_seconds)
        .bind(source.created_at.nanos)
        .bind(source.updated_at.unix_seconds)
        .bind(source.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

async fn insert_event(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    commit: &KnowledgeLifecycleCommitV1,
    occurred_at_unix_millis: i64,
) -> Result<(), KnowledgePersistenceErrorV1> {
    let result = sqlx::query(
        "INSERT INTO makosh_data.knowledge_outbox (logical_owner_id,message_id,envelope_sha256, \
         envelope_bytes,created_at_unix_millis) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(commit.lifecycle_event.message_id.as_slice())
    .bind(commit.lifecycle_event.envelope_sha256.as_slice())
    .bind(&commit.lifecycle_event.envelope_bytes)
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(KnowledgePersistenceErrorV1::OperationConflict);
    }
    Ok(())
}

async fn insert_operation(
    transaction: &mut Transaction<'_, Postgres>,
    input: &KnowledgeLifecycleOperationV1,
    note: &KnowledgeNoteRecordV1,
    commit: &KnowledgeLifecycleCommitV1,
) -> Result<(), KnowledgePersistenceErrorV1> {
    let result = sqlx::query(
        "INSERT INTO makosh_data.knowledge_client_operations (logical_owner_id,operation_id, \
         operation_kind,request_sha256,request_bytes,note_id,note_revision,response_sha256, \
         response_bytes,received_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(&input.logical_owner_id)
    .bind(input.operation_id.as_slice())
    .bind(input.mutation.operation_kind())
    .bind(input.request_sha256.as_slice())
    .bind(&input.request_bytes)
    .bind(note.note_id.as_slice())
    .bind(i64_value(note.note_revision)?)
    .bind(commit.response_sha256.as_slice())
    .bind(&commit.response_bytes)
    .bind(input.received_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(KnowledgePersistenceErrorV1::OperationConflict);
    }
    Ok(())
}

fn decode_source(
    row: &sqlx::postgres::PgRow,
) -> Result<KnowledgeSourceV1, KnowledgePersistenceErrorV1> {
    Ok(KnowledgeSourceV1 {
        source_id: fixed(row.try_get("source_id").map_err(storage)?)?,
        source_owner_id: row.try_get("source_owner_id").map_err(storage)?,
        source_record_id: fixed(row.try_get("source_record_id").map_err(storage)?)?,
        source_revision: positive_u64(row.try_get("source_revision").map_err(storage)?)?,
        evidence_digest: fixed(row.try_get("evidence_digest").map_err(storage)?)?,
        state: decode_source_state(row.try_get("source_state").map_err(storage)?)?,
        updated_at_note_revision: positive_u64(
            row.try_get("updated_at_note_revision").map_err(storage)?,
        )?,
        created_at: timestamp(row, "created_at_unix_seconds", "created_at_nanos")?,
        updated_at: timestamp(row, "updated_at_unix_seconds", "updated_at_nanos")?,
    })
}

fn decode_provenance(
    row: &sqlx::postgres::PgRow,
    origin: KnowledgeNoteOriginV1,
) -> Result<Option<KnowledgeNoteProvenanceV1>, KnowledgePersistenceErrorV1> {
    if origin == KnowledgeNoteOriginV1::OwnerAuthored {
        return Ok(None);
    }
    Ok(Some(KnowledgeNoteProvenanceV1 {
        approved_candidate_id: fixed(row.try_get("approved_candidate_id").map_err(storage)?)?,
        candidate_digest: fixed(row.try_get("candidate_digest").map_err(storage)?)?,
        source_evidence_id: fixed(row.try_get("source_evidence_id").map_err(storage)?)?,
        source_evidence_revision: positive_u64(
            row.try_get("source_evidence_revision").map_err(storage)?,
        )?,
        review_id: fixed(row.try_get("review_id").map_err(storage)?)?,
        decision_revision: positive_u64(row.try_get("decision_revision").map_err(storage)?)?,
        decided_by_owner_device_id: fixed(
            row.try_get("decided_by_owner_device_id").map_err(storage)?,
        )?,
    }))
}

fn timestamp(
    row: &sqlx::postgres::PgRow,
    seconds: &str,
    nanos: &str,
) -> Result<KnowledgeNoteTimestampV1, KnowledgePersistenceErrorV1> {
    Ok(KnowledgeNoteTimestampV1 {
        unix_seconds: row.try_get(seconds).map_err(storage)?,
        nanos: row.try_get(nanos).map_err(storage)?,
    })
}

fn mutation_note_id(value: &KnowledgeLifecycleMutationV1) -> [u8; 16] {
    match value {
        KnowledgeLifecycleMutationV1::Create(_) => [0; 16],
        KnowledgeLifecycleMutationV1::Update { note_id, .. }
        | KnowledgeLifecycleMutationV1::SetState { note_id, .. }
        | KnowledgeLifecycleMutationV1::AddSource { note_id, .. }
        | KnowledgeLifecycleMutationV1::RemoveSource { note_id, .. } => *note_id,
    }
}

fn encode_state(value: KnowledgeLifecycleStateV1) -> i16 {
    match value {
        KnowledgeLifecycleStateV1::Active => 1,
        KnowledgeLifecycleStateV1::Archived => 2,
    }
}

fn decode_state(value: i16) -> Result<KnowledgeLifecycleStateV1, KnowledgePersistenceErrorV1> {
    match value {
        1 => Ok(KnowledgeLifecycleStateV1::Active),
        2 => Ok(KnowledgeLifecycleStateV1::Archived),
        _ => Err(KnowledgePersistenceErrorV1::InvalidRow),
    }
}

fn decode_origin(value: i16) -> Result<KnowledgeNoteOriginV1, KnowledgePersistenceErrorV1> {
    match value {
        1 => Ok(KnowledgeNoteOriginV1::ReviewedCandidate),
        2 => Ok(KnowledgeNoteOriginV1::OwnerAuthored),
        _ => Err(KnowledgePersistenceErrorV1::InvalidRow),
    }
}

fn encode_source_state(value: KnowledgeSourceStateV1) -> i16 {
    match value {
        KnowledgeSourceStateV1::Active => 1,
        KnowledgeSourceStateV1::Removed => 2,
    }
}

fn decode_source_state(value: i16) -> Result<KnowledgeSourceStateV1, KnowledgePersistenceErrorV1> {
    match value {
        1 => Ok(KnowledgeSourceStateV1::Active),
        2 => Ok(KnowledgeSourceStateV1::Removed),
        _ => Err(KnowledgePersistenceErrorV1::InvalidRow),
    }
}

fn core_error(error: KnowledgeLifecycleErrorV1) -> KnowledgePersistenceErrorV1 {
    match error {
        KnowledgeLifecycleErrorV1::InvalidRevision => KnowledgePersistenceErrorV1::RevisionConflict,
        KnowledgeLifecycleErrorV1::SourceExists
        | KnowledgeLifecycleErrorV1::SourceNotFound
        | KnowledgeLifecycleErrorV1::InvalidStateTransition => {
            KnowledgePersistenceErrorV1::OperationConflict
        }
        _ => KnowledgePersistenceErrorV1::InvalidInput,
    }
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], KnowledgePersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| KnowledgePersistenceErrorV1::InvalidRow)
}

fn positive_u64(value: i64) -> Result<u64, KnowledgePersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(KnowledgePersistenceErrorV1::InvalidRow)
}

fn i64_value(value: u64) -> Result<i64, KnowledgePersistenceErrorV1> {
    i64::try_from(value).map_err(|_| KnowledgePersistenceErrorV1::InvalidInput)
}

fn storage(_: sqlx::Error) -> KnowledgePersistenceErrorV1 {
    KnowledgePersistenceErrorV1::StorageUnavailable
}
