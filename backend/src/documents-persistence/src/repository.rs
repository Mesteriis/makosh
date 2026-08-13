use makosh_documents_core::{
    CreateDocumentV1, DocumentBlobBindingV1, DocumentCoreErrorV1, DocumentCustodyStateV1,
    DocumentSourceStateV1, DocumentSourceV1, DocumentStateV1, DocumentV1, add_source_v1,
    remove_source_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    CompleteDocumentBlobOperationV1, DocumentBlobOperationKindV1,
    DocumentBlobOperationStartOutcomeV1, DocumentBlobOperationStartV1, DocumentBoundBlobCustodyV1,
    DocumentLifecycleCommitV1, DocumentLifecycleMutationV1, DocumentLifecycleOperationOutcomeV1,
    DocumentLifecycleOperationV1, DocumentOutboxRecordV1, DocumentsPersistenceErrorV1,
    model::{
        i64_value, valid_blob_start, valid_commit, valid_exact_bytes, valid_operation, valid_owner,
    },
};

#[derive(Clone)]
pub struct DocumentsPersistenceV1 {
    pool: PgPool,
}

pub struct DocumentOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: DocumentOutboxRecordV1,
    created_at_unix_millis: i64,
}

impl DocumentOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &DocumentOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), DocumentsPersistenceErrorV1> {
        if expected_sha256 != self.record.envelope_sha256
            || Sha256::digest(&self.record.envelope_bytes).as_slice() != expected_sha256
            || published_at_unix_millis < self.created_at_unix_millis
        {
            return Err(DocumentsPersistenceErrorV1::OutboxConflict);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.documents_outbox SET published_at_unix_millis=$3 \
             WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 \
             AND published_at_unix_millis IS NULL",
        )
        .bind(&self.logical_owner_id)
        .bind(self.record.message_id.as_slice())
        .bind(published_at_unix_millis)
        .bind(expected_sha256.as_slice())
        .execute(&mut *self.transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if affected != 1 {
            return Err(DocumentsPersistenceErrorV1::OutboxConflict);
        }
        self.transaction.commit().await.map_err(storage)
    }
}

impl DocumentsPersistenceV1 {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, DocumentsPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(DocumentsPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| DocumentsPersistenceErrorV1::StorageUnavailable)?,
            )
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(storage)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), DocumentsPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    async fn begin_owner(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, DocumentsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        Ok(transaction)
    }

    pub async fn load_operation_replay(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
        request_sha256: [u8; 32],
        request_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, DocumentsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || operation_id.iter().all(|byte| *byte == 0)
            || !valid_exact_bytes(request_bytes, &request_sha256)
        {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let replay = load_operation(
            &mut transaction,
            logical_owner_id,
            operation_id,
            request_sha256,
            request_bytes,
            None,
        )
        .await?
        .and_then(|value| value.response_bytes);
        transaction.commit().await.map_err(storage)?;
        Ok(replay)
    }

    pub async fn apply_lifecycle_operation<F>(
        &self,
        input: DocumentLifecycleOperationV1,
        build_commit: F,
    ) -> Result<DocumentLifecycleOperationOutcomeV1, DocumentsPersistenceErrorV1>
    where
        F: FnOnce(&DocumentV1) -> Result<DocumentLifecycleCommitV1, DocumentsPersistenceErrorV1>,
    {
        if !valid_operation(&input) {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        lock_operation(
            &mut transaction,
            &input.logical_owner_id,
            input.operation_id,
        )
        .await?;
        if let Some(stored) = load_operation(
            &mut transaction,
            &input.logical_owner_id,
            input.operation_id,
            input.request_sha256,
            &input.request_bytes,
            Some(input.mutation.operation_kind()),
        )
        .await?
        {
            let response_bytes = stored
                .response_bytes
                .ok_or(DocumentsPersistenceErrorV1::OperationConflict)?;
            transaction.commit().await.map_err(storage)?;
            return Ok(DocumentLifecycleOperationOutcomeV1::Replayed { response_bytes });
        }

        let creating = matches!(input.mutation, DocumentLifecycleMutationV1::Create { .. });
        let mut source_change = None;
        let mut document = match &input.mutation {
            DocumentLifecycleMutationV1::Create {
                title,
                description,
                media_type,
                original_file_name,
                declared_size,
                content_sha256,
                created_at_unix_millis,
            } => DocumentV1::create(CreateDocumentV1 {
                logical_owner_id: input.logical_owner_id.clone(),
                operation_id: input.operation_id,
                title: title.clone(),
                description: description.clone(),
                media_type: media_type.clone(),
                original_file_name: original_file_name.clone(),
                declared_size: *declared_size,
                content_sha256: *content_sha256,
                created_at_unix_millis: *created_at_unix_millis,
            })
            .map_err(core_error)?,
            mutation => load_document(
                &mut transaction,
                &input.logical_owner_id,
                mutation
                    .document_id()
                    .ok_or(DocumentsPersistenceErrorV1::InvalidInput)?,
                true,
            )
            .await?
            .ok_or(DocumentsPersistenceErrorV1::NotFound)?,
        };
        match &input.mutation {
            DocumentLifecycleMutationV1::Create { .. } => {}
            DocumentLifecycleMutationV1::Update {
                expected_revision,
                title,
                description,
                media_type,
                original_file_name,
                changed_at_unix_millis,
                ..
            } => document
                .update_metadata(
                    *expected_revision,
                    title.clone(),
                    description.clone(),
                    media_type.clone(),
                    original_file_name.clone(),
                    *changed_at_unix_millis,
                )
                .map_err(core_error)?,
            DocumentLifecycleMutationV1::SetState {
                expected_revision,
                state,
                changed_at_unix_millis,
                ..
            } => document
                .set_state(*expected_revision, *state, *changed_at_unix_millis)
                .map_err(core_error)?,
            DocumentLifecycleMutationV1::AddSource {
                expected_revision,
                source_owner_id,
                source_record_id,
                source_revision,
                evidence_digest,
                changed_at_unix_millis,
                ..
            } => {
                source_change = Some(
                    add_source_v1(
                        &mut document,
                        *expected_revision,
                        source_owner_id.clone(),
                        source_record_id.clone(),
                        *source_revision,
                        *evidence_digest,
                        *changed_at_unix_millis,
                    )
                    .map_err(core_error)?,
                );
            }
            DocumentLifecycleMutationV1::RemoveSource {
                expected_revision,
                source_id,
                changed_at_unix_millis,
                ..
            } => {
                let mut source = load_source(
                    &mut transaction,
                    &input.logical_owner_id,
                    document.document_id,
                    *source_id,
                    true,
                )
                .await?
                .ok_or(DocumentsPersistenceErrorV1::NotFound)?;
                remove_source_v1(
                    &mut document,
                    &mut source,
                    *expected_revision,
                    *changed_at_unix_millis,
                )
                .map_err(core_error)?;
                source_change = Some(source);
            }
        }
        persist_document(&mut transaction, &document, creating).await?;
        if let Some(source) = &source_change {
            persist_source(&mut transaction, &document, source).await?;
        }
        let commit = build_commit(&document)?;
        if !valid_commit(&commit) {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        persist_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &commit.lifecycle_event,
            input.received_at_unix_millis,
        )
        .await?;
        persist_completed_operation(
            &mut transaction,
            &input,
            document.document_id,
            expected_revision(&input.mutation),
            &commit,
            input.received_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(DocumentLifecycleOperationOutcomeV1::Applied {
            document: Box::new(document),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn start_blob_operation(
        &self,
        input: DocumentBlobOperationStartV1,
    ) -> Result<DocumentBlobOperationStartOutcomeV1, DocumentsPersistenceErrorV1> {
        if !valid_blob_start(&input) {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        lock_operation(
            &mut transaction,
            &input.logical_owner_id,
            input.operation_id,
        )
        .await?;
        if let Some(stored) = load_operation(
            &mut transaction,
            &input.logical_owner_id,
            input.operation_id,
            input.client_request_sha256,
            &input.client_request_bytes,
            Some(input.kind.code()),
        )
        .await?
        {
            validate_stored_blob(&mut transaction, &input).await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(match stored.response_bytes {
                Some(response_bytes) => {
                    DocumentBlobOperationStartOutcomeV1::Replayed { response_bytes }
                }
                None => DocumentBlobOperationStartOutcomeV1::Pending,
            });
        }
        let document = load_document(
            &mut transaction,
            &input.logical_owner_id,
            input.document_id,
            true,
        )
        .await?
        .ok_or(DocumentsPersistenceErrorV1::NotFound)?;
        if document.document_revision != input.expected_revision {
            return Err(DocumentsPersistenceErrorV1::RevisionConflict);
        }
        match input.kind {
            DocumentBlobOperationKindV1::Attach => {
                if document.state != DocumentStateV1::Active
                    || document
                        .custody
                        .as_ref()
                        .is_some_and(|value| value.state == DocumentCustodyStateV1::Bound)
                    || input.declared_size != Some(document.declared_size)
                    || input.content_sha256 != Some(document.content_sha256)
                {
                    return Err(DocumentsPersistenceErrorV1::StateConflict);
                }
            }
            DocumentBlobOperationKindV1::Release => {
                if !document.custody.as_ref().is_some_and(|value| {
                    value.state == DocumentCustodyStateV1::Bound
                        && value.blob_reference_id == input.blob_reference_id
                }) {
                    return Err(DocumentsPersistenceErrorV1::StateConflict);
                }
            }
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.documents_client_operations (logical_owner_id,operation_id,operation_kind, \
             request_sha256,request_bytes,document_id,expected_document_revision,operation_state,received_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,1,$8)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(input.kind.code())
        .bind(input.client_request_sha256.as_slice())
        .bind(&input.client_request_bytes)
        .bind(input.document_id.as_slice())
        .bind(i64_value(input.expected_revision)?)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if inserted != 1 {
            return Err(DocumentsPersistenceErrorV1::OperationConflict);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.documents_blob_operations (logical_owner_id,operation_id,document_id, \
             operation_kind,expected_document_revision,blob_reference_id,declared_size,content_sha256, \
             changed_at_unix_millis,custody_source_proof,source_evidence_id,source_evidence_envelope_sha256, \
             provider_request_sha256,provider_request_bytes) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(input.document_id.as_slice())
        .bind(input.kind.code())
        .bind(i64_value(input.expected_revision)?)
        .bind(input.blob_reference_id.as_slice())
        .bind(input.declared_size.map(i64::try_from).transpose().map_err(|_| DocumentsPersistenceErrorV1::InvalidInput)?)
        .bind(input.content_sha256.map(|value| value.to_vec()))
        .bind(input.changed_at_unix_millis)
        .bind(&input.custody_source_proof)
        .bind(input.source_evidence_id.map(|value| value.to_vec()))
        .bind(input.source_evidence_envelope_sha256.map(|value| value.to_vec()))
        .bind(input.provider_request_sha256.as_slice())
        .bind(&input.provider_request_bytes)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if inserted != 1 {
            return Err(DocumentsPersistenceErrorV1::OperationConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(DocumentBlobOperationStartOutcomeV1::Pending)
    }

    pub async fn load_bound_blob_custody(
        &self,
        logical_owner_id: &str,
        document_id: [u8; 16],
    ) -> Result<Option<DocumentBoundBlobCustodyV1>, DocumentsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || document_id.iter().all(|byte| *byte == 0) {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT b.resolved_blob_reference_id,b.declared_size,b.content_sha256,b.custody_source_proof \
             FROM makosh_data.documents_blob_operations b JOIN makosh_data.documents_client_operations c \
             ON c.logical_owner_id=b.logical_owner_id AND c.operation_id=b.operation_id \
             JOIN makosh_data.documents_records d ON d.logical_owner_id=b.logical_owner_id AND d.document_id=b.document_id \
             WHERE b.logical_owner_id=$1 AND b.document_id=$2 AND b.operation_kind=4 AND c.operation_state=2 \
             AND d.custody_state=2 AND d.blob_reference_id=b.resolved_blob_reference_id \
             ORDER BY c.completed_at_unix_millis DESC LIMIT 1",
        )
        .bind(logical_owner_id)
        .bind(document_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let result = row
            .map(|row| -> Result<_, DocumentsPersistenceErrorV1> {
                Ok(DocumentBoundBlobCustodyV1 {
                    blob_reference_id: fixed(
                        row.try_get("resolved_blob_reference_id").map_err(storage)?,
                    )?,
                    declared_size: u64_value(row.try_get("declared_size").map_err(storage)?)?,
                    content_sha256: fixed(row.try_get("content_sha256").map_err(storage)?)?,
                    custody_source_proof: row.try_get("custody_source_proof").map_err(storage)?,
                })
            })
            .transpose()?;
        transaction.commit().await.map_err(storage)?;
        Ok(result)
    }

    pub async fn complete_blob_operation<F>(
        &self,
        input: CompleteDocumentBlobOperationV1,
        build_commit: F,
    ) -> Result<DocumentLifecycleOperationOutcomeV1, DocumentsPersistenceErrorV1>
    where
        F: FnOnce(&DocumentV1) -> Result<DocumentLifecycleCommitV1, DocumentsPersistenceErrorV1>,
    {
        let CompleteDocumentBlobOperationV1 {
            logical_owner_id,
            operation_id,
            provider_receipt_sha256,
            provider_receipt_bytes,
            resolved_blob_reference_id,
            completed_at_unix_millis,
        } = input;
        if !valid_owner(&logical_owner_id)
            || operation_id.iter().all(|byte| *byte == 0)
            || !valid_exact_bytes(&provider_receipt_bytes, &provider_receipt_sha256)
            || resolved_blob_reference_id.iter().all(|byte| *byte == 0)
            || completed_at_unix_millis <= 0
        {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&logical_owner_id).await?;
        lock_operation(&mut transaction, &logical_owner_id, operation_id).await?;
        let operation = load_blob_operation(&mut transaction, &logical_owner_id, operation_id)
            .await?
            .ok_or(DocumentsPersistenceErrorV1::NotFound)?;
        let client = load_operation_by_id(&mut transaction, &logical_owner_id, operation_id)
            .await?
            .ok_or(DocumentsPersistenceErrorV1::NotFound)?;
        if let Some(response_bytes) = client.operation.response_bytes {
            if operation.provider_receipt_sha256 != Some(provider_receipt_sha256)
                || operation.provider_receipt_bytes.as_deref()
                    != Some(provider_receipt_bytes.as_slice())
                || operation.resolved_blob_reference_id != Some(resolved_blob_reference_id)
            {
                return Err(DocumentsPersistenceErrorV1::OperationConflict);
            }
            transaction.commit().await.map_err(storage)?;
            return Ok(DocumentLifecycleOperationOutcomeV1::Replayed { response_bytes });
        }
        let mut document = load_document(
            &mut transaction,
            &logical_owner_id,
            operation.document_id,
            true,
        )
        .await?
        .ok_or(DocumentsPersistenceErrorV1::NotFound)?;
        match operation.kind {
            DocumentBlobOperationKindV1::Attach => document
                .attach_blob(
                    operation.expected_revision,
                    resolved_blob_reference_id,
                    operation
                        .declared_size
                        .ok_or(DocumentsPersistenceErrorV1::InvalidRow)?,
                    operation
                        .content_sha256
                        .ok_or(DocumentsPersistenceErrorV1::InvalidRow)?,
                    operation.changed_at_unix_millis,
                )
                .map_err(core_error)?,
            DocumentBlobOperationKindV1::Release => document
                .release_blob(
                    operation.expected_revision,
                    resolved_blob_reference_id,
                    operation.changed_at_unix_millis,
                )
                .map_err(core_error)?,
        }
        persist_document(&mut transaction, &document, false).await?;
        let commit = build_commit(&document)?;
        if !valid_commit(&commit)
            || completed_at_unix_millis < client.operation.received_at_unix_millis
        {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        persist_outbox(
            &mut transaction,
            &logical_owner_id,
            &commit.lifecycle_event,
            completed_at_unix_millis,
        )
        .await?;
        let affected = sqlx::query(
            "UPDATE makosh_data.documents_blob_operations SET provider_receipt_sha256=$3,provider_receipt_bytes=$4, \
             resolved_blob_reference_id=$5 \
             WHERE logical_owner_id=$1 AND operation_id=$2 AND provider_receipt_sha256 IS NULL",
        )
        .bind(&logical_owner_id)
        .bind(operation_id.as_slice())
        .bind(provider_receipt_sha256.as_slice())
        .bind(provider_receipt_bytes)
        .bind(resolved_blob_reference_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if affected != 1 {
            return Err(DocumentsPersistenceErrorV1::OperationConflict);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.documents_client_operations SET response_sha256=$3,response_bytes=$4, \
             operation_state=2,completed_at_unix_millis=$5 WHERE logical_owner_id=$1 AND operation_id=$2 \
             AND operation_state=1",
        )
        .bind(&logical_owner_id)
        .bind(operation_id.as_slice())
        .bind(commit.response_sha256.as_slice())
        .bind(&commit.response_bytes)
        .bind(completed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if affected != 1 {
            return Err(DocumentsPersistenceErrorV1::OperationConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(DocumentLifecycleOperationOutcomeV1::Applied {
            document: Box::new(document),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_document(
        &self,
        logical_owner_id: &str,
        document_id: [u8; 16],
    ) -> Result<Option<DocumentV1>, DocumentsPersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let result = load_document(&mut transaction, logical_owner_id, document_id, false).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(result)
    }

    pub async fn list_documents(
        &self,
        logical_owner_id: &str,
        query: Option<&str>,
        after_document_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<DocumentV1>, DocumentsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || limit == 0
            || limit > 201
            || query.is_some_and(|value| {
                value.trim().is_empty() || value.len() > 200 || value.chars().any(char::is_control)
            })
        {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = if let Some(query) = query {
            sqlx::query(
                "SELECT document_id FROM makosh_data.documents_records WHERE logical_owner_id=$1 \
                 AND ($2::bytea IS NULL OR document_id>$2) AND (title ILIKE '%' || $3 || '%' \
                 OR description ILIKE '%' || $3 || '%' OR original_file_name ILIKE '%' || $3 || '%') \
                 ORDER BY document_id LIMIT $4",
            )
            .bind(logical_owner_id)
            .bind(after_document_id.map(|value| value.to_vec()))
            .bind(query.trim())
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
        } else {
            sqlx::query(
                "SELECT document_id FROM makosh_data.documents_records WHERE logical_owner_id=$1 \
                 AND ($2::bytea IS NULL OR document_id>$2) ORDER BY document_id LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after_document_id.map(|value| value.to_vec()))
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
        };
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let document_id = fixed(row.try_get("document_id").map_err(storage)?)?;
            result.push(
                load_document(&mut transaction, logical_owner_id, document_id, false)
                    .await?
                    .ok_or(DocumentsPersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(result)
    }

    pub async fn list_sources(
        &self,
        logical_owner_id: &str,
        document_id: [u8; 16],
        after_source_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<DocumentSourceV1>, DocumentsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || document_id.iter().all(|byte| *byte == 0)
            || limit == 0
            || limit > 201
        {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT source_id FROM makosh_data.documents_sources WHERE logical_owner_id=$1 AND document_id=$2 \
             AND ($3::bytea IS NULL OR source_id>$3) ORDER BY source_id LIMIT $4",
        )
        .bind(logical_owner_id)
        .bind(document_id.as_slice())
        .bind(after_source_id.map(|value| value.to_vec()))
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let source_id = fixed(row.try_get("source_id").map_err(storage)?)?;
            result.push(
                load_source(
                    &mut transaction,
                    logical_owner_id,
                    document_id,
                    source_id,
                    false,
                )
                .await?
                .ok_or(DocumentsPersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(result)
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<DocumentOutboxPublishClaimV1>, DocumentsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(DocumentsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        let row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,created_at_unix_millis \
             FROM makosh_data.documents_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL \
             ORDER BY outbox_sequence LIMIT 1 FOR UPDATE SKIP LOCKED",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let Some(row) = row else {
            transaction.commit().await.map_err(storage)?;
            return Ok(None);
        };
        let record = DocumentOutboxRecordV1 {
            message_id: fixed(row.try_get("message_id").map_err(storage)?)?,
            envelope_sha256: fixed(row.try_get("envelope_sha256").map_err(storage)?)?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
        };
        if !valid_exact_bytes(&record.envelope_bytes, &record.envelope_sha256) {
            return Err(DocumentsPersistenceErrorV1::InvalidRow);
        }
        Ok(Some(DocumentOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record,
            created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(storage)?,
        }))
    }
}

struct StoredOperationV1 {
    response_bytes: Option<Vec<u8>>,
    received_at_unix_millis: i64,
}

struct StoredBlobOperationV1 {
    kind: DocumentBlobOperationKindV1,
    document_id: [u8; 16],
    expected_revision: u64,
    blob_reference_id: [u8; 16],
    declared_size: Option<u64>,
    content_sha256: Option<[u8; 32]>,
    changed_at_unix_millis: i64,
    provider_receipt_sha256: Option<[u8; 32]>,
    provider_receipt_bytes: Option<Vec<u8>>,
    custody_source_proof: Vec<u8>,
    source_evidence_id: Option<[u8; 16]>,
    source_evidence_envelope_sha256: Option<[u8; 32]>,
    resolved_blob_reference_id: Option<[u8; 16]>,
}

async fn lock_operation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> Result<(), DocumentsPersistenceErrorV1> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
        .bind(logical_owner_id)
        .bind(operation_id.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    Ok(())
}

async fn load_operation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    request_bytes: &[u8],
    expected_kind: Option<i16>,
) -> Result<Option<StoredOperationV1>, DocumentsPersistenceErrorV1> {
    let stored = load_operation_by_id(transaction, logical_owner_id, operation_id).await?;
    let Some((stored, kind, stored_sha, stored_bytes)) = stored.map(|value| {
        let StoredOperationRowV1 {
            operation,
            kind,
            request_sha256,
            request_bytes,
        } = value;
        (operation, kind, request_sha256, request_bytes)
    }) else {
        return Ok(None);
    };
    if expected_kind.is_some_and(|value| value != kind)
        || stored_sha != request_sha256
        || stored_bytes != request_bytes
        || !valid_exact_bytes(&stored_bytes, &stored_sha)
    {
        return Err(DocumentsPersistenceErrorV1::OperationConflict);
    }
    Ok(Some(stored))
}

struct StoredOperationRowV1 {
    operation: StoredOperationV1,
    kind: i16,
    request_sha256: [u8; 32],
    request_bytes: Vec<u8>,
}

async fn load_operation_by_id(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> Result<Option<StoredOperationRowV1>, DocumentsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind,request_sha256,request_bytes,response_sha256,response_bytes,received_at_unix_millis \
         FROM makosh_data.documents_client_operations WHERE logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let request_sha256 = fixed(row.try_get("request_sha256").map_err(storage)?)?;
    let request_bytes: Vec<u8> = row.try_get("request_bytes").map_err(storage)?;
    let response_sha: Option<Vec<u8>> = row.try_get("response_sha256").map_err(storage)?;
    let response_bytes: Option<Vec<u8>> = row.try_get("response_bytes").map_err(storage)?;
    match (&response_sha, &response_bytes) {
        (Some(sha), Some(bytes)) if valid_exact_bytes(bytes, &fixed::<32>(sha.clone())?) => {}
        (None, None) => {}
        _ => return Err(DocumentsPersistenceErrorV1::InvalidRow),
    }
    Ok(Some(StoredOperationRowV1 {
        operation: StoredOperationV1 {
            response_bytes,
            received_at_unix_millis: row.try_get("received_at_unix_millis").map_err(storage)?,
        },
        kind: row.try_get("operation_kind").map_err(storage)?,
        request_sha256,
        request_bytes,
    }))
}

async fn validate_stored_blob(
    transaction: &mut Transaction<'_, Postgres>,
    input: &DocumentBlobOperationStartV1,
) -> Result<(), DocumentsPersistenceErrorV1> {
    let stored = load_blob_operation(transaction, &input.logical_owner_id, input.operation_id)
        .await?
        .ok_or(DocumentsPersistenceErrorV1::InvalidRow)?;
    if stored.kind != input.kind
        || stored.document_id != input.document_id
        || stored.expected_revision != input.expected_revision
        || stored.blob_reference_id != input.blob_reference_id
        || stored.declared_size != input.declared_size
        || stored.content_sha256 != input.content_sha256
        || stored.changed_at_unix_millis != input.changed_at_unix_millis
        || stored.custody_source_proof != input.custody_source_proof
        || stored.source_evidence_id != input.source_evidence_id
        || stored.source_evidence_envelope_sha256 != input.source_evidence_envelope_sha256
    {
        return Err(DocumentsPersistenceErrorV1::OperationConflict);
    }
    let row = sqlx::query(
        "SELECT provider_request_sha256,provider_request_bytes FROM makosh_data.documents_blob_operations \
         WHERE logical_owner_id=$1 AND operation_id=$2",
    )
    .bind(&input.logical_owner_id)
    .bind(input.operation_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage)?;
    let sha: [u8; 32] = fixed(row.try_get("provider_request_sha256").map_err(storage)?)?;
    let bytes: Vec<u8> = row.try_get("provider_request_bytes").map_err(storage)?;
    if sha != input.provider_request_sha256
        || bytes != input.provider_request_bytes
        || !valid_exact_bytes(&bytes, &sha)
    {
        return Err(DocumentsPersistenceErrorV1::OperationConflict);
    }
    Ok(())
}

async fn load_blob_operation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> Result<Option<StoredBlobOperationV1>, DocumentsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind,document_id,expected_document_revision,blob_reference_id,declared_size, \
         content_sha256,changed_at_unix_millis,custody_source_proof,source_evidence_id, \
         source_evidence_envelope_sha256,provider_receipt_sha256,provider_receipt_bytes, \
         resolved_blob_reference_id \
         FROM makosh_data.documents_blob_operations WHERE logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let kind = match row.try_get::<i16, _>("operation_kind").map_err(storage)? {
        4 => DocumentBlobOperationKindV1::Attach,
        5 => DocumentBlobOperationKindV1::Release,
        _ => return Err(DocumentsPersistenceErrorV1::InvalidRow),
    };
    let receipt_sha = row
        .try_get::<Option<Vec<u8>>, _>("provider_receipt_sha256")
        .map_err(storage)?
        .map(fixed::<32>)
        .transpose()?;
    let receipt_bytes: Option<Vec<u8>> = row.try_get("provider_receipt_bytes").map_err(storage)?;
    match (&receipt_sha, &receipt_bytes) {
        (Some(sha), Some(bytes)) if valid_exact_bytes(bytes, sha) => {}
        (None, None) => {}
        _ => return Err(DocumentsPersistenceErrorV1::InvalidRow),
    }
    Ok(Some(StoredBlobOperationV1 {
        kind,
        document_id: fixed(row.try_get("document_id").map_err(storage)?)?,
        expected_revision: u64_value(row.try_get("expected_document_revision").map_err(storage)?)?,
        blob_reference_id: fixed(row.try_get("blob_reference_id").map_err(storage)?)?,
        declared_size: row
            .try_get::<Option<i64>, _>("declared_size")
            .map_err(storage)?
            .map(u64_value)
            .transpose()?,
        content_sha256: row
            .try_get::<Option<Vec<u8>>, _>("content_sha256")
            .map_err(storage)?
            .map(fixed::<32>)
            .transpose()?,
        changed_at_unix_millis: row.try_get("changed_at_unix_millis").map_err(storage)?,
        provider_receipt_sha256: receipt_sha,
        provider_receipt_bytes: receipt_bytes,
        custody_source_proof: row.try_get("custody_source_proof").map_err(storage)?,
        source_evidence_id: row
            .try_get::<Option<Vec<u8>>, _>("source_evidence_id")
            .map_err(storage)?
            .map(fixed::<16>)
            .transpose()?,
        source_evidence_envelope_sha256: row
            .try_get::<Option<Vec<u8>>, _>("source_evidence_envelope_sha256")
            .map_err(storage)?
            .map(fixed::<32>)
            .transpose()?,
        resolved_blob_reference_id: row
            .try_get::<Option<Vec<u8>>, _>("resolved_blob_reference_id")
            .map_err(storage)?
            .map(fixed::<16>)
            .transpose()?,
    }))
}

async fn persist_completed_operation(
    transaction: &mut Transaction<'_, Postgres>,
    input: &DocumentLifecycleOperationV1,
    document_id: [u8; 16],
    expected_revision: Option<u64>,
    commit: &DocumentLifecycleCommitV1,
    completed_at_unix_millis: i64,
) -> Result<(), DocumentsPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.documents_client_operations (logical_owner_id,operation_id,operation_kind, \
         request_sha256,request_bytes,document_id,expected_document_revision,response_sha256,response_bytes, \
         operation_state,received_at_unix_millis,completed_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,2,$10,$11)",
    )
    .bind(&input.logical_owner_id)
    .bind(input.operation_id.as_slice())
    .bind(input.mutation.operation_kind())
    .bind(input.request_sha256.as_slice())
    .bind(&input.request_bytes)
    .bind(document_id.as_slice())
    .bind(expected_revision.map(i64_value).transpose()?)
    .bind(commit.response_sha256.as_slice())
    .bind(&commit.response_bytes)
    .bind(input.received_at_unix_millis)
    .bind(completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?
    .rows_affected();
    if inserted != 1 {
        return Err(DocumentsPersistenceErrorV1::OperationConflict);
    }
    Ok(())
}

fn expected_revision(value: &DocumentLifecycleMutationV1) -> Option<u64> {
    match value {
        DocumentLifecycleMutationV1::Create { .. } => None,
        DocumentLifecycleMutationV1::Update {
            expected_revision, ..
        }
        | DocumentLifecycleMutationV1::SetState {
            expected_revision, ..
        }
        | DocumentLifecycleMutationV1::AddSource {
            expected_revision, ..
        }
        | DocumentLifecycleMutationV1::RemoveSource {
            expected_revision, ..
        } => Some(*expected_revision),
    }
}

async fn persist_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    record: &DocumentOutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<(), DocumentsPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.documents_outbox (logical_owner_id,message_id,envelope_sha256,envelope_bytes,created_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(&record.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?
    .rows_affected();
    if inserted != 1 {
        return Err(DocumentsPersistenceErrorV1::OutboxConflict);
    }
    Ok(())
}

async fn persist_document(
    transaction: &mut Transaction<'_, Postgres>,
    document: &DocumentV1,
    creating: bool,
) -> Result<(), DocumentsPersistenceErrorV1> {
    let (custody_state, blob_reference_id, custody_revision) = match &document.custody {
        None => (1_i16, None, None),
        Some(value) => (
            encode_custody_state(value.state),
            Some(value.blob_reference_id.to_vec()),
            Some(i64_value(value.updated_at_document_revision)?),
        ),
    };
    let affected = if creating {
        sqlx::query(
            "INSERT INTO makosh_data.documents_records (logical_owner_id,document_id,title,description,media_type, \
             original_file_name,declared_size,content_sha256,document_state,custody_state,blob_reference_id, \
             custody_updated_document_revision,document_revision,created_at_unix_millis,updated_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)",
        )
        .bind(&document.logical_owner_id)
        .bind(document.document_id.as_slice())
        .bind(&document.title)
        .bind(&document.description)
        .bind(&document.media_type)
        .bind(&document.original_file_name)
        .bind(i64_value(document.declared_size)?)
        .bind(document.content_sha256.as_slice())
        .bind(encode_document_state(document.state))
        .bind(custody_state)
        .bind(blob_reference_id)
        .bind(custody_revision)
        .bind(i64_value(document.document_revision)?)
        .bind(document.created_at_unix_millis)
        .bind(document.updated_at_unix_millis)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?
        .rows_affected()
    } else {
        let previous_revision = document
            .document_revision
            .checked_sub(1)
            .ok_or(DocumentsPersistenceErrorV1::RevisionConflict)?;
        sqlx::query(
            "UPDATE makosh_data.documents_records SET title=$3,description=$4,media_type=$5,original_file_name=$6, \
             document_state=$7,custody_state=$8,blob_reference_id=$9,custody_updated_document_revision=$10, \
             document_revision=$11,updated_at_unix_millis=$12 WHERE logical_owner_id=$1 AND document_id=$2 \
             AND document_revision=$13",
        )
        .bind(&document.logical_owner_id)
        .bind(document.document_id.as_slice())
        .bind(&document.title)
        .bind(&document.description)
        .bind(&document.media_type)
        .bind(&document.original_file_name)
        .bind(encode_document_state(document.state))
        .bind(custody_state)
        .bind(blob_reference_id)
        .bind(custody_revision)
        .bind(i64_value(document.document_revision)?)
        .bind(document.updated_at_unix_millis)
        .bind(i64_value(previous_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?
        .rows_affected()
    };
    if affected != 1 {
        return Err(if creating {
            DocumentsPersistenceErrorV1::OperationConflict
        } else {
            DocumentsPersistenceErrorV1::RevisionConflict
        });
    }
    Ok(())
}

async fn persist_source(
    transaction: &mut Transaction<'_, Postgres>,
    document: &DocumentV1,
    source: &DocumentSourceV1,
) -> Result<(), DocumentsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.documents_sources (logical_owner_id,document_id,source_id,source_owner_id, \
         source_record_id,source_revision,evidence_digest,source_state,updated_at_document_revision) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (logical_owner_id,document_id,source_id) \
         DO UPDATE SET source_revision=EXCLUDED.source_revision,evidence_digest=EXCLUDED.evidence_digest, \
         source_state=EXCLUDED.source_state,updated_at_document_revision=EXCLUDED.updated_at_document_revision",
    )
    .bind(&document.logical_owner_id)
    .bind(document.document_id.as_slice())
    .bind(source.source_id.as_slice())
    .bind(&source.source_owner_id)
    .bind(&source.source_record_id)
    .bind(i64_value(source.source_revision)?)
    .bind(source.evidence_digest.as_slice())
    .bind(encode_source_state(source.state))
    .bind(i64_value(source.updated_at_document_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn load_document(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    document_id: [u8; 16],
    for_update: bool,
) -> Result<Option<DocumentV1>, DocumentsPersistenceErrorV1> {
    let sql = if for_update {
        "SELECT * FROM makosh_data.documents_records WHERE logical_owner_id=$1 AND document_id=$2 FOR UPDATE"
    } else {
        "SELECT * FROM makosh_data.documents_records WHERE logical_owner_id=$1 AND document_id=$2"
    };
    let row = sqlx::query(sql)
        .bind(logical_owner_id)
        .bind(document_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let stored_id: [u8; 16] = fixed(row.try_get("document_id").map_err(storage)?)?;
    let declared_size = u64_value(row.try_get("declared_size").map_err(storage)?)?;
    let content_sha256 = fixed(row.try_get("content_sha256").map_err(storage)?)?;
    let custody_state = decode_custody_state(row.try_get("custody_state").map_err(storage)?)?;
    let blob_reference: Option<Vec<u8>> = row.try_get("blob_reference_id").map_err(storage)?;
    let custody_revision: Option<i64> = row
        .try_get("custody_updated_document_revision")
        .map_err(storage)?;
    let custody = match (custody_state, blob_reference, custody_revision) {
        (DocumentCustodyStateV1::Unbound, None, None) => None,
        (
            state @ (DocumentCustodyStateV1::Bound | DocumentCustodyStateV1::Released),
            Some(reference),
            Some(revision),
        ) => Some(DocumentBlobBindingV1 {
            blob_reference_id: fixed(reference)?,
            declared_size,
            content_sha256,
            state,
            updated_at_document_revision: u64_value(revision)?,
        }),
        _ => return Err(DocumentsPersistenceErrorV1::InvalidRow),
    };
    Ok(Some(DocumentV1 {
        document_id: stored_id,
        logical_owner_id: row.try_get("logical_owner_id").map_err(storage)?,
        title: row.try_get("title").map_err(storage)?,
        description: row.try_get("description").map_err(storage)?,
        media_type: row.try_get("media_type").map_err(storage)?,
        original_file_name: row.try_get("original_file_name").map_err(storage)?,
        declared_size,
        content_sha256,
        state: decode_document_state(row.try_get("document_state").map_err(storage)?)?,
        custody,
        document_revision: u64_value(row.try_get("document_revision").map_err(storage)?)?,
        created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(storage)?,
        updated_at_unix_millis: row.try_get("updated_at_unix_millis").map_err(storage)?,
    }))
}

async fn load_source(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    document_id: [u8; 16],
    source_id: [u8; 16],
    for_update: bool,
) -> Result<Option<DocumentSourceV1>, DocumentsPersistenceErrorV1> {
    let sql = if for_update {
        "SELECT * FROM makosh_data.documents_sources WHERE logical_owner_id=$1 AND document_id=$2 AND source_id=$3 FOR UPDATE"
    } else {
        "SELECT * FROM makosh_data.documents_sources WHERE logical_owner_id=$1 AND document_id=$2 AND source_id=$3"
    };
    let row = sqlx::query(sql)
        .bind(logical_owner_id)
        .bind(document_id.as_slice())
        .bind(source_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    Ok(Some(DocumentSourceV1 {
        source_id: fixed(row.try_get("source_id").map_err(storage)?)?,
        source_owner_id: row.try_get("source_owner_id").map_err(storage)?,
        source_record_id: row.try_get("source_record_id").map_err(storage)?,
        source_revision: u64_value(row.try_get("source_revision").map_err(storage)?)?,
        evidence_digest: fixed(row.try_get("evidence_digest").map_err(storage)?)?,
        state: decode_source_state(row.try_get("source_state").map_err(storage)?)?,
        updated_at_document_revision: u64_value(
            row.try_get("updated_at_document_revision")
                .map_err(storage)?,
        )?,
    }))
}

fn encode_document_state(value: DocumentStateV1) -> i16 {
    match value {
        DocumentStateV1::Active => 1,
        DocumentStateV1::Archived => 2,
    }
}

fn decode_document_state(value: i16) -> Result<DocumentStateV1, DocumentsPersistenceErrorV1> {
    match value {
        1 => Ok(DocumentStateV1::Active),
        2 => Ok(DocumentStateV1::Archived),
        _ => Err(DocumentsPersistenceErrorV1::InvalidRow),
    }
}

fn encode_custody_state(value: DocumentCustodyStateV1) -> i16 {
    match value {
        DocumentCustodyStateV1::Unbound => 1,
        DocumentCustodyStateV1::Bound => 2,
        DocumentCustodyStateV1::Released => 3,
    }
}

fn decode_custody_state(value: i16) -> Result<DocumentCustodyStateV1, DocumentsPersistenceErrorV1> {
    match value {
        1 => Ok(DocumentCustodyStateV1::Unbound),
        2 => Ok(DocumentCustodyStateV1::Bound),
        3 => Ok(DocumentCustodyStateV1::Released),
        _ => Err(DocumentsPersistenceErrorV1::InvalidRow),
    }
}

fn encode_source_state(value: DocumentSourceStateV1) -> i16 {
    match value {
        DocumentSourceStateV1::Active => 1,
        DocumentSourceStateV1::Removed => 2,
    }
}

fn decode_source_state(value: i16) -> Result<DocumentSourceStateV1, DocumentsPersistenceErrorV1> {
    match value {
        1 => Ok(DocumentSourceStateV1::Active),
        2 => Ok(DocumentSourceStateV1::Removed),
        _ => Err(DocumentsPersistenceErrorV1::InvalidRow),
    }
}

fn u64_value(value: i64) -> Result<u64, DocumentsPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DocumentsPersistenceErrorV1::InvalidRow)
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], DocumentsPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| DocumentsPersistenceErrorV1::InvalidRow)
}

fn core_error(value: DocumentCoreErrorV1) -> DocumentsPersistenceErrorV1 {
    match value {
        DocumentCoreErrorV1::RevisionConflict | DocumentCoreErrorV1::RevisionOverflow => {
            DocumentsPersistenceErrorV1::RevisionConflict
        }
        DocumentCoreErrorV1::StateConflict => DocumentsPersistenceErrorV1::StateConflict,
        DocumentCoreErrorV1::InvalidInput => DocumentsPersistenceErrorV1::InvalidInput,
    }
}

fn storage(_: sqlx::Error) -> DocumentsPersistenceErrorV1 {
    DocumentsPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_core_errors_and_exact_outbox_hash_are_preserved() {
        assert_eq!(
            core_error(DocumentCoreErrorV1::RevisionOverflow),
            DocumentsPersistenceErrorV1::RevisionConflict
        );
        assert_eq!(
            core_error(DocumentCoreErrorV1::StateConflict),
            DocumentsPersistenceErrorV1::StateConflict
        );
        let bytes = b"event".to_vec();
        let record = DocumentOutboxRecordV1 {
            message_id: [1; 16],
            envelope_sha256: Sha256::digest(&bytes).into(),
            envelope_bytes: bytes,
        };
        assert!(valid_exact_bytes(
            &record.envelope_bytes,
            &record.envelope_sha256
        ));
    }
}
