//! PostgreSQL inbox and canonical summary storage owned only by Communications.

use hermes_communications_api::{
    AttachmentDispositionV1, AttachmentSafetyStateV1, AttachmentSafetyTransitionDecisionV1,
    CanonicalCommunicationEvidenceKindV1, CanonicalCommunicationProjectionV1,
    CanonicalMessageMutationV1, CommunicationAccountIdV1, CommunicationAccountSummaryV1,
    CommunicationAttachmentAnchorIdV1, CommunicationAttachmentAnchorStateV1,
    CommunicationAttachmentAnchorSummaryV1, CommunicationBodyBlobReferenceV1,
    CommunicationBodyStateV1, CommunicationConversationIdV1, CommunicationConversationSummaryV1,
    CommunicationDirectionV1, CommunicationMessageIdV1, CommunicationMessageLifecycleStateV1,
    CommunicationMessageReferenceKindV1, CommunicationMessageReferenceSummaryV1,
    CommunicationMessageSummaryV1, CommunicationObservationIdV1,
    CommunicationObservedParticipantSummaryV1, CommunicationParticipantIdV1,
    CommunicationProviderProvenanceV1, CommunicationSourceCursorV1, CommunicationSummary,
};
use hermes_events_protocol::delivery::OutboxRecordV1;
use hermes_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    CommunicationsConsumeOutcomeV1, CommunicationsDerivedIndexFailureRecordV1,
    CommunicationsDerivedIndexFailureV1, CommunicationsDerivedIndexJobOperationV1,
    CommunicationsDerivedIndexJobV1, CommunicationsPersistenceError,
    PendingCommunicationsBodyCustodyTransferV1,
};

pub struct CommunicationsDurablePersistence {
    pub(crate) pool: PgPool,
}

/// One canonical owner mutation derived from an admitted ingress record.
pub struct PersistedCommunicationsObservationV1<'a> {
    pub projection: CanonicalCommunicationProjectionV1,
    pub pending_custody_transfer: Option<&'a PendingCommunicationsBodyCustodyTransferV1>,
    pub derived_index_job: Option<&'a CommunicationsDerivedIndexJobV1>,
    pub derived_index_failure: Option<&'a CommunicationsDerivedIndexFailureRecordV1>,
    pub canonical_outbox_record: &'a OutboxRecordV1,
    pub attachment_anchor_outbox_record: Option<&'a OutboxRecordV1>,
    pub created_at_unix_seconds: i64,
}

struct ReusableCanonicalBodyV1 {
    blob: CommunicationBodyBlobReferenceV1,
}

async fn find_reusable_canonical_body_v1(
    transaction: &mut Transaction<'_, Postgres>,
    transfer: &PendingCommunicationsBodyCustodyTransferV1,
) -> Result<Option<ReusableCanonicalBodyV1>, CommunicationsPersistenceError> {
    let row = sqlx::query(
        "SELECT evidence.body_blob_ref, evidence.body_blob_reference_id, evidence.body_blob_declared_bytes, evidence.body_blob_sha256 FROM hermes_data.communications_body_custody_transfers AS source JOIN hermes_data.communications_body_custody_transfer_lifecycle AS lifecycle ON lifecycle.evidence_id = source.evidence_id JOIN hermes_data.communications_evidence_summaries AS evidence ON evidence.observation_id = source.evidence_id WHERE lifecycle.state = 2 AND evidence.body_state = 4 AND source.source_blob_ref = $1 AND source.source_reference_id = $2 AND source.declared_bytes = $3 AND source.plaintext_sha256 = $4 ORDER BY lifecycle.completed_at_unix_seconds DESC NULLS LAST, source.evidence_id DESC LIMIT 1",
    )
    .bind(&transfer.source_blob_ref)
    .bind(transfer.source_reference_id.as_slice())
    .bind(i64::try_from(transfer.declared_bytes).map_err(|_| CommunicationsPersistenceError::InvalidCustodyTransfer)?)
    .bind(transfer.plaintext_sha256.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
    let Some(row) = row else { return Ok(None) };
    let blob_ref: String = row
        .try_get("body_blob_ref")
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
    let reference_id: Vec<u8> = row
        .try_get("body_blob_reference_id")
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
    let declared_bytes: i64 = row
        .try_get("body_blob_declared_bytes")
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
    let sha256: Vec<u8> = row
        .try_get("body_blob_sha256")
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
    Ok(Some(ReusableCanonicalBodyV1 {
        blob: CommunicationBodyBlobReferenceV1 {
            blob_ref,
            reference_id: reference_id
                .try_into()
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?,
            declared_bytes: u64::try_from(declared_bytes)
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?,
            sha256: sha256
                .try_into()
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?,
        },
    }))
}

impl CommunicationsDurablePersistence {
    pub async fn compare_and_set_attachment_safety_state_with_outbox(
        &self,
        decision: AttachmentSafetyTransitionDecisionV1,
        canonical_outbox_record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsPersistenceError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        let result = sqlx::query(
            "UPDATE hermes_data.communications_attachment_anchors SET anchor_state = $2, last_observed_at_unix_seconds = GREATEST(last_observed_at_unix_seconds, $3), last_evidence_id = CASE WHEN $3 >= last_observed_at_unix_seconds THEN $4 ELSE last_evidence_id END WHERE attachment_anchor_id = $1 AND anchor_state = $5",
        )
        .bind(decision.attachment_anchor_id.bytes().as_slice())
        .bind(attachment_safety_state_value(decision.next_state))
        .bind(decision.observed_at_unix_seconds)
        .bind(decision.evidence_id.bytes().as_slice())
        .bind(attachment_safety_state_value(decision.expected_state))
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        if result.rows_affected() != 1 {
            transaction
                .rollback()
                .await
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
            return Ok(false);
        }
        sqlx::query("INSERT INTO hermes_data.communications_domain_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(canonical_outbox_record.message_id().as_slice())
            .bind(canonical_outbox_record.envelope_sha256().as_slice())
            .bind(canonical_outbox_record.exact_bytes())
            .bind(created_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        Ok(true)
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, CommunicationsPersistenceError> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(CommunicationsPersistenceError::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
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
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), CommunicationsPersistenceError> {
        sqlx::query(
            "SELECT 1 FROM hermes_data.communications_event_inbox, hermes_data.communications_evidence_summaries, hermes_data.communications_evidence_audit_lineage, hermes_data.communications_domain_outbox, hermes_data.communications_conversations, hermes_data.communications_accounts, hermes_data.communications_messages, hermes_data.communications_observed_participants, hermes_data.communications_attachment_anchors, hermes_data.communications_message_references, hermes_data.communications_derived_index_projections, hermes_data.communications_derived_index_token_digests, hermes_data.communications_derived_index_tombstones, hermes_data.communications_derived_index_jobs, hermes_data.communications_derived_index_failures, hermes_data.communications_body_custody_transfers, hermes_data.communications_body_custody_transfer_lifecycle LIMIT 0",
        )
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)
    }

    /// Shares the already budgeted owner-local pool with a separately admitted
    /// Communications persistence build unit. The runtime remains the only
    /// composition root; persistence packages do not import each other.
    #[must_use]
    pub fn owner_local_pool_handle(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn persist_consumed_observation(
        &self,
        record: &OutboxRecordV1,
        observation: PersistedCommunicationsObservationV1<'_>,
    ) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsPersistenceError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        let inserted = sqlx::query(
            r#"
            INSERT INTO hermes_data.communications_event_inbox (message_id, envelope_sha256)
            VALUES ($1, $2)
            ON CONFLICT (message_id) DO NOTHING
            RETURNING message_id
            "#,
        )
        .bind(record.message_id().as_slice())
        .bind(record.envelope_sha256().as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        if inserted.is_none() {
            let row = sqlx::query(
                "SELECT envelope_sha256 FROM hermes_data.communications_event_inbox WHERE message_id = $1",
            )
            .bind(record.message_id().as_slice())
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
            let hash: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
            return if hash.as_slice() == record.envelope_sha256() {
                Ok(CommunicationsConsumeOutcomeV1::Duplicate)
            } else {
                Err(CommunicationsPersistenceError::InboxHashConflict)
            };
        }
        let summary = &observation.projection.summary;
        let inserted_summary = sqlx::query(
            r#"
            INSERT INTO hermes_data.communications_evidence_summaries
                (observation_id, source_cursor_sha256, account_cursor_sha256, conversation_cursor_sha256, participant_cursor_sha256, participant_display_label, message_subject, media_cursor_sha256, reply_to_source_cursor_sha256, forward_origin_source_cursor_sha256, provider, direction, evidence_kind, body_state, body_blob_ref, body_blob_reference_id, body_blob_declared_bytes, body_blob_sha256, body_admission_failure, body_media_type, observed_at_unix_seconds)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            ON CONFLICT (observation_id) DO NOTHING
            "#,
        )
        .bind(summary.observation_id.bytes().as_slice())
        .bind(summary.source_cursor.bytes().as_slice())
        .bind(summary.account_cursor.map(|value| value.bytes().to_vec()))
        .bind(summary.conversation_cursor.map(|value| value.bytes().to_vec()))
        .bind(summary.participant_cursor.map(|value| value.bytes().to_vec()))
        .bind(&summary.participant_display_label)
        .bind(&summary.message_subject)
        .bind(summary.media_cursor.map(|value| value.bytes().to_vec()))
        .bind(summary.reply_to_source_cursor.map(|value| value.bytes().to_vec()))
        .bind(summary.forward_origin_source_cursor.map(|value| value.bytes().to_vec()))
        .bind(provider_value(summary.provider))
        .bind(direction_value(summary.direction))
        .bind(kind_value(summary.kind))
        .bind(body_value(summary.body))
        .bind(summary.body_blob.as_ref().map(|value| value.blob_ref.as_str()))
        .bind(summary.body_blob.as_ref().map(|value| value.reference_id.to_vec()))
        .bind(summary.body_blob.as_ref().map(|value| i64::try_from(value.declared_bytes).expect("body byte limit fits i64")))
        .bind(summary.body_blob.as_ref().map(|value| value.sha256.to_vec()))
        .bind(summary.body_admission_failure.map(body_admission_failure_value))
        .bind(&summary.body_media_type)
        .bind(summary.observed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        if inserted_summary.rows_affected() != 1 {
            return Err(CommunicationsPersistenceError::DuplicateOperation);
        }
        sqlx::query(
            "INSERT INTO hermes_data.communications_evidence_audit_lineage (evidence_id, causation_message_id, correlation_id, recorded_at_unix_seconds, recorded_at_nanos) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(summary.evidence_id.bytes().as_slice())
        .bind(summary.causation_message_id.map(|value| value.bytes().to_vec()))
        .bind(summary.correlation_id.bytes().as_slice())
        .bind(summary.recorded_at_unix_seconds)
        .bind(summary.recorded_at_nanos)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        let mut reused_canonical_body = None;
        if let Some(transfer) = observation.pending_custody_transfer {
            if transfer.evidence_id != summary.evidence_id
                || transfer.envelope_sha256.as_slice() != record.envelope_sha256()
                || summary.body != CommunicationBodyStateV1::PendingBlob
                || summary.body_blob.is_some()
                || transfer.source_blob_ref.is_empty()
                || transfer.source_blob_ref.len() > 512
                || !transfer.source_blob_ref.is_ascii()
                || transfer.source_reference_id.iter().all(|byte| *byte == 0)
                || !(1..=64 * 1024 * 1024).contains(&transfer.declared_bytes)
                || transfer.plaintext_sha256.iter().all(|byte| *byte == 0)
                || !(1..=2_048).contains(&transfer.source_custody_proof.len())
            {
                return Err(CommunicationsPersistenceError::InvalidCustodyTransfer);
            }
            if let Some(reusable) =
                find_reusable_canonical_body_v1(&mut transaction, transfer).await?
            {
                let updated = sqlx::query(
                    "UPDATE hermes_data.communications_evidence_summaries SET body_state = 4, body_blob_ref = $2, body_blob_reference_id = $3, body_blob_declared_bytes = $4, body_blob_sha256 = $5 WHERE observation_id = $1 AND body_state = 2",
                )
                .bind(transfer.evidence_id.bytes().as_slice())
                .bind(&reusable.blob.blob_ref)
                .bind(reusable.blob.reference_id.as_slice())
                .bind(i64::try_from(reusable.blob.declared_bytes).map_err(|_| CommunicationsPersistenceError::InvalidCustodyTransfer)?)
                .bind(reusable.blob.sha256.as_slice())
                .execute(&mut *transaction)
                .await
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
                if updated.rows_affected() != 1 {
                    return Err(CommunicationsPersistenceError::InvalidCustodyTransfer);
                }
                reused_canonical_body = Some(reusable);
            } else {
                sqlx::query(
                    "INSERT INTO hermes_data.communications_body_custody_transfers (evidence_id, envelope_sha256, source_blob_ref, source_reference_id, declared_bytes, plaintext_sha256, source_custody_proof, state) VALUES ($1, $2, $3, $4, $5, $6, $7, 1) ON CONFLICT (evidence_id) DO NOTHING",
                )
                .bind(transfer.evidence_id.bytes().as_slice())
                .bind(transfer.envelope_sha256.as_slice())
                .bind(&transfer.source_blob_ref)
                .bind(transfer.source_reference_id.as_slice())
                .bind(i64::try_from(transfer.declared_bytes).map_err(|_| CommunicationsPersistenceError::InvalidCustodyTransfer)?)
                .bind(transfer.plaintext_sha256.as_slice())
                .bind(&transfer.source_custody_proof)
                .execute(&mut *transaction)
                .await
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
                sqlx::query(
                    "INSERT INTO hermes_data.communications_body_custody_transfer_lifecycle (evidence_id, state) VALUES ($1, 1) ON CONFLICT (evidence_id) DO NOTHING",
                )
                .bind(transfer.evidence_id.bytes().as_slice())
                .execute(&mut *transaction)
                .await
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
            }
        } else if summary.body == CommunicationBodyStateV1::PendingBlob {
            return Err(CommunicationsPersistenceError::InvalidCustodyTransfer);
        }
        if let Some(account) = &observation.projection.account {
            sqlx::query(
                "INSERT INTO hermes_data.communications_accounts (account_id, account_cursor_sha256, provider, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id) VALUES ($1, $2, $3, $4, $4, $5) ON CONFLICT (account_id) DO UPDATE SET last_observed_at_unix_seconds = GREATEST(communications_accounts.last_observed_at_unix_seconds, EXCLUDED.last_observed_at_unix_seconds), last_evidence_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds >= communications_accounts.last_observed_at_unix_seconds THEN EXCLUDED.last_evidence_id ELSE communications_accounts.last_evidence_id END",
            )
            .bind(account.account_id.bytes().as_slice())
            .bind(account.account_cursor.bytes().as_slice())
            .bind(provider_value(account.provider))
            .bind(account.observed_at_unix_seconds)
            .bind(summary.evidence_id.bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        if let Some(conversation) = &observation.projection.conversation {
            sqlx::query(
                "INSERT INTO hermes_data.communications_conversations (conversation_id, account_cursor_sha256, conversation_cursor_sha256, provider, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id) VALUES ($1, $2, $3, $4, $5, $5, $6) ON CONFLICT (conversation_id) DO UPDATE SET last_observed_at_unix_seconds = GREATEST(communications_conversations.last_observed_at_unix_seconds, EXCLUDED.last_observed_at_unix_seconds), last_evidence_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds >= communications_conversations.last_observed_at_unix_seconds THEN EXCLUDED.last_evidence_id ELSE communications_conversations.last_evidence_id END",
            )
            .bind(conversation.conversation_id.bytes().as_slice())
            .bind(conversation.account_cursor.bytes().as_slice())
            .bind(conversation.conversation_cursor.bytes().as_slice())
            .bind(provider_value(conversation.provider))
            .bind(conversation.observed_at_unix_seconds)
            .bind(summary.evidence_id.bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        if let Some(message) = &observation.projection.message {
            let projected_body = if reused_canonical_body.is_some() {
                CommunicationBodyStateV1::AdmittedBlob
            } else {
                message.body
            };
            match message.mutation {
                CanonicalMessageMutationV1::Create => {
                    sqlx::query(
                        "INSERT INTO hermes_data.communications_messages (message_id, conversation_id, source_cursor_sha256, body_state, canonical_body_state, direction, lifecycle_state, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id) VALUES ($1, $2, $3, LEAST($4, 3), $4, $5, 1, $6, $6, $7) ON CONFLICT (message_id) DO UPDATE SET body_state = LEAST($4, 3), canonical_body_state = $4, canonical_revision = communications_messages.canonical_revision + 1, last_observed_at_unix_seconds = GREATEST(communications_messages.last_observed_at_unix_seconds, EXCLUDED.last_observed_at_unix_seconds), last_evidence_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds >= communications_messages.last_observed_at_unix_seconds THEN EXCLUDED.last_evidence_id ELSE communications_messages.last_evidence_id END WHERE communications_messages.conversation_id = EXCLUDED.conversation_id AND communications_messages.direction = EXCLUDED.direction",
                    )
                    .bind(message.message_id.bytes().as_slice())
                    .bind(message.conversation_id.bytes().as_slice())
                    .bind(message.source_cursor.bytes().as_slice())
                    .bind(body_value(projected_body))
                    .bind(direction_value(message.direction))
                    .bind(message.observed_at_unix_seconds)
                    .bind(summary.evidence_id.bytes().as_slice())
                    .execute(&mut *transaction)
                    .await
                    .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
                }
                CanonicalMessageMutationV1::Update | CanonicalMessageMutationV1::Delete => {
                    let lifecycle_state = match message.mutation {
                        CanonicalMessageMutationV1::Update => 1_i16,
                        CanonicalMessageMutationV1::Delete => 2_i16,
                        CanonicalMessageMutationV1::Create => unreachable!(),
                    };
                    let updated = sqlx::query(
                        "UPDATE hermes_data.communications_messages SET body_state = LEAST($3, 3), canonical_body_state = $3, canonical_revision = canonical_revision + 1, lifecycle_state = CASE WHEN lifecycle_state = 2 THEN 2 ELSE $4 END, last_observed_at_unix_seconds = GREATEST(last_observed_at_unix_seconds, $5), last_evidence_id = CASE WHEN $5 >= last_observed_at_unix_seconds THEN $6 ELSE last_evidence_id END WHERE message_id = $1 AND conversation_id = $2 AND direction = $7",
                    )
                    .bind(message.message_id.bytes().as_slice())
                    .bind(message.conversation_id.bytes().as_slice())
                    .bind(body_value(projected_body))
                    .bind(lifecycle_state)
                    .bind(message.observed_at_unix_seconds)
                    .bind(summary.evidence_id.bytes().as_slice())
                    .bind(direction_value(message.direction))
                    .execute(&mut *transaction)
                    .await
                    .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
                    if updated.rows_affected() != 1 {
                        return Err(CommunicationsPersistenceError::MissingCanonicalMessage);
                    }
                }
            }
        }
        if let Some(participant) = &observation.projection.participant {
            sqlx::query(
                "INSERT INTO hermes_data.communications_observed_participants (participant_id, conversation_id, participant_cursor_sha256, display_label, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id) VALUES ($1, $2, $3, $4, $5, $5, $6) ON CONFLICT (participant_id) DO UPDATE SET display_label = CASE WHEN EXCLUDED.last_observed_at_unix_seconds >= communications_observed_participants.last_observed_at_unix_seconds THEN COALESCE(EXCLUDED.display_label, communications_observed_participants.display_label) ELSE communications_observed_participants.display_label END, last_observed_at_unix_seconds = GREATEST(communications_observed_participants.last_observed_at_unix_seconds, EXCLUDED.last_observed_at_unix_seconds), last_evidence_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds >= communications_observed_participants.last_observed_at_unix_seconds THEN EXCLUDED.last_evidence_id ELSE communications_observed_participants.last_evidence_id END",
            )
            .bind(participant.participant_id.bytes().as_slice())
            .bind(participant.conversation_id.bytes().as_slice())
            .bind(participant.participant_cursor.bytes().as_slice())
            .bind(&participant.display_label)
            .bind(participant.observed_at_unix_seconds)
            .bind(summary.evidence_id.bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        if let (Some(account), Some(message), Some(participant), Some(sender_id)) = (
            observation.projection.account.as_ref(),
            observation.projection.message.as_ref(),
            observation.projection.participant.as_ref(),
            observation
                .projection
                .participant
                .as_ref()
                .and_then(|participant| participant.sender_id),
        ) && message.direction == CommunicationDirectionV1::Incoming
        {
            sqlx::query(
                "INSERT INTO hermes_data.communications_sender_profiles \
                 (sender_id, display_label, first_observed_at_unix_seconds, \
                  last_observed_at_unix_seconds, last_evidence_id) \
                 VALUES ($1, $2, $3, $3, $4) \
                 ON CONFLICT (sender_id) DO UPDATE SET \
                   display_label = CASE WHEN EXCLUDED.last_observed_at_unix_seconds \
                     >= communications_sender_profiles.last_observed_at_unix_seconds \
                     THEN COALESCE(EXCLUDED.display_label, \
                       communications_sender_profiles.display_label) \
                     ELSE communications_sender_profiles.display_label END, \
                   last_observed_at_unix_seconds = GREATEST(\
                     communications_sender_profiles.last_observed_at_unix_seconds, \
                     EXCLUDED.last_observed_at_unix_seconds), \
                   last_evidence_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds \
                     >= communications_sender_profiles.last_observed_at_unix_seconds \
                     THEN EXCLUDED.last_evidence_id \
                     ELSE communications_sender_profiles.last_evidence_id END",
            )
            .bind(sender_id.bytes().as_slice())
            .bind(&participant.display_label)
            .bind(message.observed_at_unix_seconds)
            .bind(summary.evidence_id.bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
            sqlx::query(
                "INSERT INTO hermes_data.communications_message_sender_facts \
                 (message_id, sender_id, participant_id, account_id, \
                  first_observed_at_unix_seconds, last_observed_at_unix_seconds, \
                  last_evidence_id) \
                 VALUES ($1, $2, $3, $4, $5, $5, $6) \
                 ON CONFLICT (message_id) DO UPDATE SET \
                   sender_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds \
                     >= communications_message_sender_facts.last_observed_at_unix_seconds \
                     THEN EXCLUDED.sender_id \
                     ELSE communications_message_sender_facts.sender_id END, \
                   participant_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds \
                     >= communications_message_sender_facts.last_observed_at_unix_seconds \
                     THEN EXCLUDED.participant_id \
                     ELSE communications_message_sender_facts.participant_id END, \
                   account_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds \
                     >= communications_message_sender_facts.last_observed_at_unix_seconds \
                     THEN EXCLUDED.account_id \
                     ELSE communications_message_sender_facts.account_id END, \
                   last_observed_at_unix_seconds = GREATEST(\
                     communications_message_sender_facts.last_observed_at_unix_seconds, \
                     EXCLUDED.last_observed_at_unix_seconds), \
                   last_evidence_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds \
                     >= communications_message_sender_facts.last_observed_at_unix_seconds \
                     THEN EXCLUDED.last_evidence_id \
                     ELSE communications_message_sender_facts.last_evidence_id END",
            )
            .bind(message.message_id.bytes().as_slice())
            .bind(sender_id.bytes().as_slice())
            .bind(participant.participant_id.bytes().as_slice())
            .bind(account.account_id.bytes().as_slice())
            .bind(message.observed_at_unix_seconds)
            .bind(summary.evidence_id.bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        let mut attachment_anchor_created = false;
        if let Some(anchor) = &observation.projection.attachment_anchor {
            let row = sqlx::query(
                "INSERT INTO hermes_data.communications_attachment_anchors (attachment_anchor_id, message_id, media_cursor_sha256, anchor_state, attachment_filename, attachment_media_type, attachment_declared_bytes, attachment_sha256, attachment_disposition, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id) VALUES ($1, $2, $3, 1, $4, $5, $6, $7, $8, $9, $9, $10) ON CONFLICT (attachment_anchor_id) DO UPDATE SET last_observed_at_unix_seconds = GREATEST(communications_attachment_anchors.last_observed_at_unix_seconds, EXCLUDED.last_observed_at_unix_seconds), last_evidence_id = CASE WHEN EXCLUDED.last_observed_at_unix_seconds >= communications_attachment_anchors.last_observed_at_unix_seconds THEN EXCLUDED.last_evidence_id ELSE communications_attachment_anchors.last_evidence_id END, attachment_filename = COALESCE(communications_attachment_anchors.attachment_filename, EXCLUDED.attachment_filename), attachment_media_type = COALESCE(communications_attachment_anchors.attachment_media_type, EXCLUDED.attachment_media_type), attachment_declared_bytes = COALESCE(communications_attachment_anchors.attachment_declared_bytes, EXCLUDED.attachment_declared_bytes), attachment_sha256 = COALESCE(communications_attachment_anchors.attachment_sha256, EXCLUDED.attachment_sha256), attachment_disposition = COALESCE(communications_attachment_anchors.attachment_disposition, EXCLUDED.attachment_disposition) RETURNING (xmax = 0) AS inserted",
            )
            .bind(anchor.attachment_anchor_id.bytes().as_slice())
            .bind(anchor.message_id.bytes().as_slice())
            .bind(anchor.media_cursor.bytes().as_slice())
            .bind(anchor.descriptor.as_ref().and_then(|value| value.filename()))
            .bind(anchor.descriptor.as_ref().map(|value| value.media_type()))
            .bind(anchor.descriptor.as_ref().map(|value| i64::try_from(value.declared_bytes()).expect("attachment byte limit fits i64")))
            .bind(anchor.descriptor.as_ref().and_then(|value| value.sha256()).map(|value| value.to_vec()))
            .bind(anchor.descriptor.as_ref().map(|value| attachment_disposition_value(value.disposition())))
            .bind(anchor.observed_at_unix_seconds)
            .bind(summary.evidence_id.bytes().as_slice())
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
            attachment_anchor_created = row
                .try_get("inserted")
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        for reference in &observation.projection.message_references {
            let mut reference_id = Sha256::new();
            reference_id.update(reference.source_message_id.bytes());
            reference_id.update(reference_kind_value(reference.kind).to_be_bytes());
            reference_id.update(reference.target_source_cursor.bytes());
            let reference_id: [u8; 32] = reference_id.finalize().into();
            sqlx::query(
                "INSERT INTO hermes_data.communications_message_references (reference_id, source_message_id, reference_kind, target_source_cursor_sha256, observed_at_unix_seconds, evidence_id) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (reference_id) DO NOTHING",
            )
            .bind(reference_id.as_slice())
            .bind(reference.source_message_id.bytes().as_slice())
            .bind(reference_kind_value(reference.kind))
            .bind(reference.target_source_cursor.bytes().as_slice())
            .bind(reference.observed_at_unix_seconds)
            .bind(summary.evidence_id.bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        if let Some(job) = observation.derived_index_job {
            job.validate()
                .map_err(|_| CommunicationsPersistenceError::InvalidDerivedIndexJob)?;
            sqlx::query("INSERT INTO hermes_data.communications_derived_index_jobs (job_id, operation, evidence_id, message_id, conversation_id, blob_ref, blob_reference_id, blob_declared_bytes, blob_sha256, projection_revision, observed_at_unix_seconds, created_at_unix_seconds) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) ON CONFLICT (job_id) DO NOTHING")
                .bind(job.job_id.as_slice())
                .bind(match job.operation { CommunicationsDerivedIndexJobOperationV1::Index => 1_i16, CommunicationsDerivedIndexJobOperationV1::Remove => 2_i16 })
                .bind(job.evidence_id.bytes().as_slice())
                .bind(job.message_id.bytes().as_slice())
                .bind(job.conversation_id.map(|value| value.bytes().to_vec()))
                .bind(job.blob.as_ref().map(|value| value.blob_ref.as_str()))
                .bind(job.blob.as_ref().map(|value| value.reference_id.to_vec()))
                .bind(job.blob.as_ref().map(|value| i64::try_from(value.declared_bytes).expect("body byte limit fits i64")))
                .bind(job.blob.as_ref().map(|value| value.sha256.to_vec()))
                .bind(i32::try_from(job.projection_revision).map_err(|_| CommunicationsPersistenceError::InvalidDerivedIndexJob)?)
                .bind(job.observed_at_unix_seconds)
                .bind(job.created_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        if let Some(failure) = observation.derived_index_failure {
            if failure.failure != CommunicationsDerivedIndexFailureV1::DocumentLimit
                || failure.projection_revision == 0
            {
                return Err(CommunicationsPersistenceError::InvalidDerivedIndexJob);
            }
            sqlx::query("INSERT INTO hermes_data.communications_derived_index_failures (evidence_id, message_id, projection_revision, observed_at_unix_seconds, failure_code, recorded_at_unix_seconds) VALUES ($1, $2, $3, $4, 6, $5) ON CONFLICT (evidence_id) DO NOTHING")
                .bind(failure.evidence_id.bytes().as_slice())
                .bind(failure.message_id.bytes().as_slice())
                .bind(i32::try_from(failure.projection_revision).map_err(|_| CommunicationsPersistenceError::InvalidDerivedIndexJob)?)
                .bind(failure.observed_at_unix_seconds)
                .bind(failure.recorded_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        }
        sqlx::query("INSERT INTO hermes_data.communications_domain_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(observation.canonical_outbox_record.message_id().as_slice())
            .bind(observation.canonical_outbox_record.envelope_sha256().as_slice())
            .bind(observation.canonical_outbox_record.exact_bytes())
            .bind(observation.created_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        if attachment_anchor_created {
            let record = observation
                .attachment_anchor_outbox_record
                .ok_or(CommunicationsPersistenceError::InvalidAttachmentAnchorOutbox)?;
            sqlx::query("INSERT INTO hermes_data.communications_domain_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
                .bind(record.message_id().as_slice())
                .bind(record.envelope_sha256().as_slice())
                .bind(record.exact_bytes())
                .bind(observation.created_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        } else if observation.attachment_anchor_outbox_record.is_some()
            && observation.projection.attachment_anchor.is_none()
        {
            return Err(CommunicationsPersistenceError::InvalidAttachmentAnchorOutbox);
        }
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        Ok(CommunicationsConsumeOutcomeV1::Applied)
    }

    pub async fn pending_domain_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, CommunicationsPersistenceError> {
        let rows = sqlx::query("SELECT exact_envelope_bytes FROM hermes_data.communications_domain_outbox WHERE published_at_unix_seconds IS NULL ORDER BY created_at_unix_seconds ASC, message_id ASC LIMIT $1")
            .bind(limit.clamp(1, 256))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
                OutboxRecordV1::accept(bytes)
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_domain_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsPersistenceError> {
        sqlx::query("UPDATE hermes_data.communications_domain_outbox SET published_at_unix_seconds = $2 WHERE message_id = $1 AND published_at_unix_seconds IS NULL")
            .bind(message_id.as_slice())
            .bind(published_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)
    }

    pub async fn summary(
        &self,
        evidence_id: CommunicationObservationIdV1,
    ) -> Result<Option<CommunicationSummary>, CommunicationsPersistenceError> {
        let row = sqlx::query(
            "SELECT summary.observation_id, lineage.causation_message_id, COALESCE(lineage.correlation_id, summary.observation_id) AS correlation_id, summary.source_cursor_sha256, summary.account_cursor_sha256, summary.conversation_cursor_sha256, summary.participant_cursor_sha256, summary.participant_display_label, summary.message_subject, summary.media_cursor_sha256, summary.reply_to_source_cursor_sha256, summary.forward_origin_source_cursor_sha256, summary.provider, summary.direction, summary.evidence_kind, summary.body_state, summary.body_blob_ref, summary.body_blob_reference_id, summary.body_blob_declared_bytes, summary.body_blob_sha256, summary.body_admission_failure, summary.body_media_type, summary.observed_at_unix_seconds, COALESCE(lineage.recorded_at_unix_seconds, summary.observed_at_unix_seconds) AS recorded_at_unix_seconds, COALESCE(lineage.recorded_at_nanos, 0) AS recorded_at_nanos FROM hermes_data.communications_evidence_summaries summary LEFT JOIN hermes_data.communications_evidence_audit_lineage lineage ON lineage.evidence_id = summary.observation_id WHERE summary.observation_id = $1",
        )
        .bind(evidence_id.bytes().as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        row.map(|row| {
            let observation_id: Vec<u8> = row
                .try_get("observation_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let causation_message_id: Option<Vec<u8>> = row
                .try_get("causation_message_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let correlation_id: Vec<u8> = row
                .try_get("correlation_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let source_cursor: Vec<u8> = row
                .try_get("source_cursor_sha256")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let account_cursor: Option<Vec<u8>> = row
                .try_get("account_cursor_sha256")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let conversation_cursor: Option<Vec<u8>> = row
                .try_get("conversation_cursor_sha256")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let participant_cursor: Option<Vec<u8>> = row
                .try_get("participant_cursor_sha256")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let participant_display_label: Option<String> = row
                .try_get("participant_display_label")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let message_subject: Option<String> = row
                .try_get("message_subject")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let media_cursor: Option<Vec<u8>> = row
                .try_get("media_cursor_sha256")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let reply_to_source_cursor: Option<Vec<u8>> = row
                .try_get("reply_to_source_cursor_sha256")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let forward_origin_source_cursor: Option<Vec<u8>> = row
                .try_get("forward_origin_source_cursor_sha256")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let provider: i16 = row
                .try_get("provider")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let direction: i16 = row
                .try_get("direction")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let kind: i16 = row
                .try_get("evidence_kind")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body: i16 = row
                .try_get("body_state")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body_blob_ref: Option<String> = row
                .try_get("body_blob_ref")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body_blob_reference_id: Option<Vec<u8>> = row
                .try_get("body_blob_reference_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body_blob_declared_bytes: Option<i64> = row
                .try_get("body_blob_declared_bytes")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body_blob_sha256: Option<Vec<u8>> = row
                .try_get("body_blob_sha256")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body_admission_failure: Option<i16> = row
                .try_get("body_admission_failure")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body_media_type: Option<String> = row
                .try_get("body_media_type")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let observed_at_unix_seconds: i64 = row
                .try_get("observed_at_unix_seconds")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let recorded_at_unix_seconds: i64 = row
                .try_get("recorded_at_unix_seconds")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let recorded_at_nanos: i32 = row
                .try_get("recorded_at_nanos")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let observation_id: [u8; 16] = observation_id
                .as_slice()
                .try_into()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let source_cursor: [u8; 32] = source_cursor
                .as_slice()
                .try_into()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let causation_message_id = causation_message_id
                .map(|value| {
                    value
                        .as_slice()
                        .try_into()
                        .map(CommunicationObservationIdV1::new)
                })
                .transpose()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let correlation_id: [u8; 16] = correlation_id
                .as_slice()
                .try_into()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let account_cursor = account_cursor
                .map(|value| {
                    value
                        .as_slice()
                        .try_into()
                        .map(CommunicationSourceCursorV1::new)
                })
                .transpose()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let conversation_cursor = conversation_cursor
                .map(|value| {
                    value
                        .as_slice()
                        .try_into()
                        .map(CommunicationSourceCursorV1::new)
                })
                .transpose()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let participant_cursor = participant_cursor
                .map(|value| {
                    value
                        .as_slice()
                        .try_into()
                        .map(CommunicationSourceCursorV1::new)
                })
                .transpose()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let media_cursor = media_cursor
                .map(|value| {
                    value
                        .as_slice()
                        .try_into()
                        .map(CommunicationSourceCursorV1::new)
                })
                .transpose()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let reply_to_source_cursor = reply_to_source_cursor
                .map(|value| {
                    value
                        .as_slice()
                        .try_into()
                        .map(CommunicationSourceCursorV1::new)
                })
                .transpose()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let forward_origin_source_cursor = forward_origin_source_cursor
                .map(|value| {
                    value
                        .as_slice()
                        .try_into()
                        .map(CommunicationSourceCursorV1::new)
                })
                .transpose()
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body_blob = match (
                body_blob_ref,
                body_blob_reference_id,
                body_blob_declared_bytes,
                body_blob_sha256,
            ) {
                (None, None, None, None) => None,
                (Some(blob_ref), Some(reference_id), Some(declared_bytes), Some(sha256)) => Some(
                    hermes_communications_api::CommunicationBodyBlobReferenceV1 {
                        blob_ref,
                        reference_id: reference_id
                            .as_slice()
                            .try_into()
                            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
                        declared_bytes: u64::try_from(declared_bytes)
                            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
                        sha256: sha256
                            .as_slice()
                            .try_into()
                            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
                    },
                ),
                _ => return Err(CommunicationsPersistenceError::InvalidRow),
            };
            Ok(CommunicationSummary {
                evidence_id: CommunicationObservationIdV1::new(observation_id),
                observation_id: CommunicationObservationIdV1::new(observation_id),
                causation_message_id,
                correlation_id: CommunicationObservationIdV1::new(correlation_id),
                source_cursor: CommunicationSourceCursorV1::new(source_cursor),
                account_cursor,
                conversation_cursor,
                participant_cursor,
                participant_display_label,
                message_subject,
                media_cursor,
                reply_to_source_cursor,
                forward_origin_source_cursor,
                provider: provider_from_value(provider)?,
                direction: direction_from_value(direction)?,
                kind: kind_from_value(kind)?,
                body: body_from_value(body)?,
                body_blob,
                body_media_type,
                body_admission_failure: body_admission_failure
                    .map(body_admission_failure_from_value)
                    .transpose()?,
                attachment_descriptor: None,
                observed_at_unix_seconds,
                recorded_at_unix_seconds,
                recorded_at_nanos,
            })
        })
        .transpose()
    }

    pub async fn conversation(
        &self,
        conversation_id: CommunicationConversationIdV1,
    ) -> Result<Option<CommunicationConversationSummaryV1>, CommunicationsPersistenceError> {
        let row = sqlx::query("SELECT conversation_id, account_cursor_sha256, conversation_cursor_sha256, provider, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id FROM hermes_data.communications_conversations WHERE conversation_id = $1")
            .bind(conversation_id.bytes().as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        row.map(conversation_from_row).transpose()
    }

    pub async fn message_evidence_ids(
        &self,
        message_id: CommunicationMessageIdV1,
        limit: u16,
    ) -> Result<Vec<CommunicationObservationIdV1>, CommunicationsPersistenceError> {
        let rows = sqlx::query("SELECT summary.observation_id FROM hermes_data.communications_evidence_summaries summary INNER JOIN hermes_data.communications_messages message ON message.source_cursor_sha256 = summary.source_cursor_sha256 WHERE message.message_id = $1 ORDER BY summary.observed_at_unix_seconds DESC, summary.observation_id ASC LIMIT $2")
            .bind(message_id.bytes().as_slice()).bind(i64::from(limit)).fetch_all(&self.pool).await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter()
            .map(|row| {
                let value: Vec<u8> = row
                    .try_get("observation_id")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
                id16(&value).map(CommunicationObservationIdV1::new)
            })
            .collect()
    }

    pub async fn conversations(
        &self,
        account_cursor: Option<CommunicationSourceCursorV1>,
        limit: u16,
    ) -> Result<Vec<CommunicationConversationSummaryV1>, CommunicationsPersistenceError> {
        let rows = sqlx::query("SELECT conversation_id, account_cursor_sha256, conversation_cursor_sha256, provider, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id FROM hermes_data.communications_conversations WHERE ($1::bytea IS NULL OR account_cursor_sha256 = $1) ORDER BY last_observed_at_unix_seconds DESC, conversation_id ASC LIMIT $2")
            .bind(account_cursor.map(|value| value.bytes().to_vec()))
            .bind(i64::from(limit.clamp(1, 100)))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter().map(conversation_from_row).collect()
    }

    pub async fn accounts(
        &self,
        limit: u16,
    ) -> Result<Vec<CommunicationAccountSummaryV1>, CommunicationsPersistenceError> {
        let rows = sqlx::query("SELECT account_id, account_cursor_sha256, provider, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id FROM hermes_data.communications_accounts ORDER BY last_observed_at_unix_seconds DESC, account_id ASC LIMIT $1")
            .bind(i64::from(limit.clamp(1, 100)))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter().map(account_from_row).collect()
    }

    pub async fn conversation_messages(
        &self,
        conversation_id: CommunicationConversationIdV1,
        limit: u16,
    ) -> Result<Vec<CommunicationMessageSummaryV1>, CommunicationsPersistenceError> {
        let rows = sqlx::query("SELECT message_id, conversation_id, source_cursor_sha256, canonical_body_state AS body_state, direction, lifecycle_state, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id FROM hermes_data.communications_messages WHERE conversation_id = $1 ORDER BY last_observed_at_unix_seconds DESC, message_id ASC LIMIT $2")
            .bind(conversation_id.bytes().as_slice())
            .bind(i64::from(limit.clamp(1, 100)))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter().map(message_from_row).collect()
    }

    pub async fn conversation_participants(
        &self,
        conversation_id: CommunicationConversationIdV1,
        limit: u16,
    ) -> Result<Vec<CommunicationObservedParticipantSummaryV1>, CommunicationsPersistenceError>
    {
        let rows = sqlx::query("SELECT participant_id, conversation_id, participant_cursor_sha256, display_label, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id FROM hermes_data.communications_observed_participants WHERE conversation_id = $1 ORDER BY last_observed_at_unix_seconds DESC, participant_id ASC LIMIT $2")
            .bind(conversation_id.bytes().as_slice())
            .bind(i64::from(limit.clamp(1, 100)))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter().map(participant_from_row).collect()
    }

    pub async fn message_attachment_anchors(
        &self,
        message_id: CommunicationMessageIdV1,
        limit: u16,
    ) -> Result<Vec<CommunicationAttachmentAnchorSummaryV1>, CommunicationsPersistenceError> {
        let rows = sqlx::query("SELECT attachment_anchor_id, message_id, media_cursor_sha256, anchor_state, attachment_filename, attachment_media_type, attachment_declared_bytes, attachment_sha256, attachment_disposition, first_observed_at_unix_seconds, last_observed_at_unix_seconds, last_evidence_id FROM hermes_data.communications_attachment_anchors WHERE message_id = $1 ORDER BY last_observed_at_unix_seconds DESC, attachment_anchor_id ASC LIMIT $2")
            .bind(message_id.bytes().as_slice())
            .bind(i64::from(limit.clamp(1, 100)))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter().map(anchor_from_row).collect()
    }

    pub async fn message_references(
        &self,
        message_id: CommunicationMessageIdV1,
        limit: u16,
    ) -> Result<Vec<CommunicationMessageReferenceSummaryV1>, CommunicationsPersistenceError> {
        let rows = sqlx::query("SELECT reference.source_message_id, reference.reference_kind, reference.target_source_cursor_sha256, target.message_id AS target_message_id, reference.observed_at_unix_seconds, reference.evidence_id FROM hermes_data.communications_message_references reference LEFT JOIN hermes_data.communications_messages target ON target.source_cursor_sha256 = reference.target_source_cursor_sha256 WHERE reference.source_message_id = $1 ORDER BY reference.observed_at_unix_seconds ASC, reference.reference_kind ASC LIMIT $2")
            .bind(message_id.bytes().as_slice())
            .bind(i64::from(limit.clamp(1, 100)))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        rows.into_iter().map(reference_from_row).collect()
    }
}

pub(crate) fn reference_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationMessageReferenceSummaryV1, CommunicationsPersistenceError> {
    let source_message_id: Vec<u8> = row
        .try_get("source_message_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let reference_kind: i16 = row
        .try_get("reference_kind")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let target_source_cursor: Vec<u8> = row
        .try_get("target_source_cursor_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let target_message_id: Option<Vec<u8>> = row
        .try_get("target_message_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let observed_at_unix_seconds: i64 = row
        .try_get("observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let evidence_id: Vec<u8> = row
        .try_get("evidence_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    Ok(CommunicationMessageReferenceSummaryV1 {
        source_message_id: CommunicationMessageIdV1::new(id16(&source_message_id)?),
        kind: reference_kind_from_value(reference_kind)?,
        target_source_cursor: CommunicationSourceCursorV1::new(id32(&target_source_cursor)?),
        target_message_id: target_message_id
            .map(|value| id16(&value).map(CommunicationMessageIdV1::new))
            .transpose()?,
        observed_at_unix_seconds,
        evidence_id: CommunicationObservationIdV1::new(id16(&evidence_id)?),
    })
}

pub(crate) fn account_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationAccountSummaryV1, CommunicationsPersistenceError> {
    let account_id: Vec<u8> = row
        .try_get("account_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let account_cursor: Vec<u8> = row
        .try_get("account_cursor_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let provider: i16 = row
        .try_get("provider")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let first_observed_at_unix_seconds: i64 = row
        .try_get("first_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_observed_at_unix_seconds: i64 = row
        .try_get("last_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_evidence_id: Vec<u8> = row
        .try_get("last_evidence_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    Ok(CommunicationAccountSummaryV1 {
        account_id: CommunicationAccountIdV1::new(id16(&account_id)?),
        account_cursor: CommunicationSourceCursorV1::new(id32(&account_cursor)?),
        provider: provider_from_value(provider)?,
        first_observed_at_unix_seconds,
        last_observed_at_unix_seconds,
        last_evidence_id: CommunicationObservationIdV1::new(id16(&last_evidence_id)?),
    })
}

pub(crate) fn anchor_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationAttachmentAnchorSummaryV1, CommunicationsPersistenceError> {
    let attachment_anchor_id: Vec<u8> = row
        .try_get("attachment_anchor_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let message_id: Vec<u8> = row
        .try_get("message_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let media_cursor: Vec<u8> = row
        .try_get("media_cursor_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let anchor_state: i16 = row
        .try_get("anchor_state")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let filename: Option<String> = row
        .try_get("attachment_filename")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let media_type: Option<String> = row
        .try_get("attachment_media_type")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let declared_bytes: Option<i64> = row
        .try_get("attachment_declared_bytes")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let sha256: Option<Vec<u8>> = row
        .try_get("attachment_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let disposition: Option<i16> = row
        .try_get("attachment_disposition")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let first_observed_at_unix_seconds: i64 = row
        .try_get("first_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_observed_at_unix_seconds: i64 = row
        .try_get("last_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_evidence_id: Vec<u8> = row
        .try_get("last_evidence_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let state = match anchor_state {
        1 => CommunicationAttachmentAnchorStateV1::DescriptorOnly,
        2 => CommunicationAttachmentAnchorStateV1::BlobPending,
        3 => CommunicationAttachmentAnchorStateV1::BlobAdmitted,
        4 => CommunicationAttachmentAnchorStateV1::Quarantined,
        5 => CommunicationAttachmentAnchorStateV1::SafeForDelivery,
        6 => CommunicationAttachmentAnchorStateV1::Rejected,
        _ => return Err(CommunicationsPersistenceError::InvalidRow),
    };
    let descriptor = match (media_type, declared_bytes, sha256, disposition) {
        (None, None, None, None) => None,
        (Some(media_type), Some(declared_bytes), sha256, Some(disposition)) => {
            let disposition = attachment_disposition_from_value(disposition)?;
            let sha256 = sha256.map(|value| id32(&value)).transpose()?;
            Some(
                hermes_communications_api::AttachmentDescriptorV1::new(
                    filename,
                    media_type,
                    u64::try_from(declared_bytes)
                        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
                    sha256,
                    disposition,
                )
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
            )
        }
        _ => return Err(CommunicationsPersistenceError::InvalidRow),
    };
    Ok(CommunicationAttachmentAnchorSummaryV1 {
        attachment_anchor_id: CommunicationAttachmentAnchorIdV1::new(id16(&attachment_anchor_id)?),
        message_id: CommunicationMessageIdV1::new(id16(&message_id)?),
        media_cursor: CommunicationSourceCursorV1::new(id32(&media_cursor)?),
        state,
        descriptor,
        first_observed_at_unix_seconds,
        last_observed_at_unix_seconds,
        last_evidence_id: CommunicationObservationIdV1::new(id16(&last_evidence_id)?),
    })
}

fn attachment_disposition_value(value: AttachmentDispositionV1) -> i16 {
    match value {
        AttachmentDispositionV1::Attachment => 1,
        AttachmentDispositionV1::Inline => 2,
        AttachmentDispositionV1::Unknown => 3,
    }
}

const fn attachment_safety_state_value(value: AttachmentSafetyStateV1) -> i16 {
    match value {
        AttachmentSafetyStateV1::DescriptorOnly => 1,
        AttachmentSafetyStateV1::BlobPending => 2,
        AttachmentSafetyStateV1::BlobAdmitted => 3,
        AttachmentSafetyStateV1::Quarantined => 4,
        AttachmentSafetyStateV1::SafeForDelivery => 5,
        AttachmentSafetyStateV1::Rejected => 6,
    }
}

fn attachment_disposition_from_value(
    value: i16,
) -> Result<AttachmentDispositionV1, CommunicationsPersistenceError> {
    match value {
        1 => Ok(AttachmentDispositionV1::Attachment),
        2 => Ok(AttachmentDispositionV1::Inline),
        3 => Ok(AttachmentDispositionV1::Unknown),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

pub(crate) fn participant_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationObservedParticipantSummaryV1, CommunicationsPersistenceError> {
    let participant_id: Vec<u8> = row
        .try_get("participant_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let conversation_id: Vec<u8> = row
        .try_get("conversation_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let participant_cursor: Vec<u8> = row
        .try_get("participant_cursor_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let display_label: Option<String> = row
        .try_get("display_label")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let first_observed_at_unix_seconds: i64 = row
        .try_get("first_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_observed_at_unix_seconds: i64 = row
        .try_get("last_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_evidence_id: Vec<u8> = row
        .try_get("last_evidence_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    Ok(CommunicationObservedParticipantSummaryV1 {
        participant_id: CommunicationParticipantIdV1::new(id16(&participant_id)?),
        conversation_id: CommunicationConversationIdV1::new(id16(&conversation_id)?),
        participant_cursor: CommunicationSourceCursorV1::new(id32(&participant_cursor)?),
        display_label,
        first_observed_at_unix_seconds,
        last_observed_at_unix_seconds,
        last_evidence_id: CommunicationObservationIdV1::new(id16(&last_evidence_id)?),
    })
}

pub(crate) fn conversation_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationConversationSummaryV1, CommunicationsPersistenceError> {
    let conversation_id: Vec<u8> = row
        .try_get("conversation_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let account_cursor: Vec<u8> = row
        .try_get("account_cursor_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let conversation_cursor: Vec<u8> = row
        .try_get("conversation_cursor_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let provider: i16 = row
        .try_get("provider")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let first_observed_at_unix_seconds: i64 = row
        .try_get("first_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_observed_at_unix_seconds: i64 = row
        .try_get("last_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_evidence_id: Vec<u8> = row
        .try_get("last_evidence_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    Ok(CommunicationConversationSummaryV1 {
        conversation_id: CommunicationConversationIdV1::new(id16(&conversation_id)?),
        account_cursor: CommunicationSourceCursorV1::new(id32(&account_cursor)?),
        conversation_cursor: CommunicationSourceCursorV1::new(id32(&conversation_cursor)?),
        provider: provider_from_value(provider)?,
        first_observed_at_unix_seconds,
        last_observed_at_unix_seconds,
        last_evidence_id: CommunicationObservationIdV1::new(id16(&last_evidence_id)?),
    })
}

pub(crate) fn message_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationMessageSummaryV1, CommunicationsPersistenceError> {
    let message_id: Vec<u8> = row
        .try_get("message_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let conversation_id: Vec<u8> = row
        .try_get("conversation_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let source_cursor: Vec<u8> = row
        .try_get("source_cursor_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let body: i16 = row
        .try_get("body_state")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let direction: i16 = row
        .try_get("direction")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let lifecycle_state: i16 = row
        .try_get("lifecycle_state")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let first_observed_at_unix_seconds: i64 = row
        .try_get("first_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_observed_at_unix_seconds: i64 = row
        .try_get("last_observed_at_unix_seconds")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let last_evidence_id: Vec<u8> = row
        .try_get("last_evidence_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    Ok(CommunicationMessageSummaryV1 {
        message_id: CommunicationMessageIdV1::new(id16(&message_id)?),
        conversation_id: CommunicationConversationIdV1::new(id16(&conversation_id)?),
        source_cursor: CommunicationSourceCursorV1::new(id32(&source_cursor)?),
        body: body_from_value(body)?,
        direction: direction_from_value(direction)?,
        lifecycle_state: lifecycle_state_from_value(lifecycle_state)?,
        first_observed_at_unix_seconds,
        last_observed_at_unix_seconds,
        last_evidence_id: CommunicationObservationIdV1::new(id16(&last_evidence_id)?),
    })
}

pub(crate) fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsPersistenceError> {
    value
        .try_into()
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsPersistenceError> {
    value
        .try_into()
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)
}

const fn provider_value(value: CommunicationProviderProvenanceV1) -> i16 {
    match value {
        CommunicationProviderProvenanceV1::MailImap => 1,
        CommunicationProviderProvenanceV1::Telegram => 2,
        CommunicationProviderProvenanceV1::WhatsAppWeb => 3,
        CommunicationProviderProvenanceV1::MailSmtp => 4,
        CommunicationProviderProvenanceV1::Zulip => 5,
        CommunicationProviderProvenanceV1::MailGmail => 6,
    }
}

const fn direction_value(value: CommunicationDirectionV1) -> i16 {
    match value {
        CommunicationDirectionV1::Incoming => 1,
        CommunicationDirectionV1::Outgoing => 2,
        CommunicationDirectionV1::Unknown => 3,
    }
}

const fn kind_value(value: CanonicalCommunicationEvidenceKindV1) -> i16 {
    match value {
        CanonicalCommunicationEvidenceKindV1::EmailMessage => 1,
        CanonicalCommunicationEvidenceKindV1::ChatMessage => 2,
        CanonicalCommunicationEvidenceKindV1::MessageEdited => 3,
        CanonicalCommunicationEvidenceKindV1::MessageDeleted => 4,
        CanonicalCommunicationEvidenceKindV1::ReactionChanged => 5,
        CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged => 6,
        CanonicalCommunicationEvidenceKindV1::ConversationStateChanged => 7,
        CanonicalCommunicationEvidenceKindV1::ParticipantChanged => 8,
        CanonicalCommunicationEvidenceKindV1::MediaChanged => 9,
        CanonicalCommunicationEvidenceKindV1::TopicChanged => 10,
        CanonicalCommunicationEvidenceKindV1::TypingChanged => 11,
    }
}

const fn body_value(value: CommunicationBodyStateV1) -> i16 {
    match value {
        CommunicationBodyStateV1::MetadataOnly => 1,
        CommunicationBodyStateV1::PendingBlob => 2,
        CommunicationBodyStateV1::Unavailable => 3,
        CommunicationBodyStateV1::AdmittedBlob => 4,
    }
}

const fn body_admission_failure_value(
    value: hermes_communications_api::CommunicationBodyAdmissionFailureV1,
) -> i16 {
    match value {
        hermes_communications_api::CommunicationBodyAdmissionFailureV1::SourceUnavailable => 1,
        hermes_communications_api::CommunicationBodyAdmissionFailureV1::SizeLimitExceeded => 2,
        hermes_communications_api::CommunicationBodyAdmissionFailureV1::IntegrityMismatch => 3,
        hermes_communications_api::CommunicationBodyAdmissionFailureV1::PolicyRejected => 4,
    }
}

const fn provider_from_value(
    value: i16,
) -> Result<CommunicationProviderProvenanceV1, CommunicationsPersistenceError> {
    match value {
        1 => Ok(CommunicationProviderProvenanceV1::MailImap),
        2 => Ok(CommunicationProviderProvenanceV1::Telegram),
        3 => Ok(CommunicationProviderProvenanceV1::WhatsAppWeb),
        4 => Ok(CommunicationProviderProvenanceV1::MailSmtp),
        5 => Ok(CommunicationProviderProvenanceV1::Zulip),
        6 => Ok(CommunicationProviderProvenanceV1::MailGmail),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

const fn direction_from_value(
    value: i16,
) -> Result<CommunicationDirectionV1, CommunicationsPersistenceError> {
    match value {
        1 => Ok(CommunicationDirectionV1::Incoming),
        2 => Ok(CommunicationDirectionV1::Outgoing),
        3 => Ok(CommunicationDirectionV1::Unknown),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

const fn kind_from_value(
    value: i16,
) -> Result<CanonicalCommunicationEvidenceKindV1, CommunicationsPersistenceError> {
    match value {
        1 => Ok(CanonicalCommunicationEvidenceKindV1::EmailMessage),
        2 => Ok(CanonicalCommunicationEvidenceKindV1::ChatMessage),
        3 => Ok(CanonicalCommunicationEvidenceKindV1::MessageEdited),
        4 => Ok(CanonicalCommunicationEvidenceKindV1::MessageDeleted),
        5 => Ok(CanonicalCommunicationEvidenceKindV1::ReactionChanged),
        6 => Ok(CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged),
        7 => Ok(CanonicalCommunicationEvidenceKindV1::ConversationStateChanged),
        8 => Ok(CanonicalCommunicationEvidenceKindV1::ParticipantChanged),
        9 => Ok(CanonicalCommunicationEvidenceKindV1::MediaChanged),
        10 => Ok(CanonicalCommunicationEvidenceKindV1::TopicChanged),
        11 => Ok(CanonicalCommunicationEvidenceKindV1::TypingChanged),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

const fn body_from_value(
    value: i16,
) -> Result<CommunicationBodyStateV1, CommunicationsPersistenceError> {
    match value {
        1 => Ok(CommunicationBodyStateV1::MetadataOnly),
        2 => Ok(CommunicationBodyStateV1::PendingBlob),
        3 => Ok(CommunicationBodyStateV1::Unavailable),
        4 => Ok(CommunicationBodyStateV1::AdmittedBlob),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

const fn body_admission_failure_from_value(
    value: i16,
) -> Result<
    hermes_communications_api::CommunicationBodyAdmissionFailureV1,
    CommunicationsPersistenceError,
> {
    match value {
        1 => Ok(hermes_communications_api::CommunicationBodyAdmissionFailureV1::SourceUnavailable),
        2 => Ok(hermes_communications_api::CommunicationBodyAdmissionFailureV1::SizeLimitExceeded),
        3 => Ok(hermes_communications_api::CommunicationBodyAdmissionFailureV1::IntegrityMismatch),
        4 => Ok(hermes_communications_api::CommunicationBodyAdmissionFailureV1::PolicyRejected),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

const fn lifecycle_state_from_value(
    value: i16,
) -> Result<CommunicationMessageLifecycleStateV1, CommunicationsPersistenceError> {
    match value {
        1 => Ok(CommunicationMessageLifecycleStateV1::Active),
        2 => Ok(CommunicationMessageLifecycleStateV1::Deleted),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

const fn reference_kind_value(
    value: hermes_communications_api::CommunicationMessageReferenceKindV1,
) -> i16 {
    match value {
        hermes_communications_api::CommunicationMessageReferenceKindV1::Reply => 1,
        hermes_communications_api::CommunicationMessageReferenceKindV1::Forward => 2,
    }
}

const fn reference_kind_from_value(
    value: i16,
) -> Result<CommunicationMessageReferenceKindV1, CommunicationsPersistenceError> {
    match value {
        1 => Ok(CommunicationMessageReferenceKindV1::Reply),
        2 => Ok(CommunicationMessageReferenceKindV1::Forward),
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

#[cfg(test)]
mod tests {
    use super::{provider_from_value, provider_value};
    use hermes_communications_api::CommunicationProviderProvenanceV1;

    #[test]
    fn zulip_provider_value_round_trips() {
        assert_eq!(
            provider_from_value(provider_value(CommunicationProviderProvenanceV1::Zulip)),
            Ok(CommunicationProviderProvenanceV1::Zulip)
        );
    }
}
