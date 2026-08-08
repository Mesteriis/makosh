use makosh_communication_delivery_intent_core::{
    CommunicationConversationIdV1, CommunicationDeliveryRouteV1, CommunicationMessageIdV1,
    CommunicationProviderProvenanceV1, CommunicationSourceCursorV1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CommunicationDeliveryIntentPersistenceV1, valid_bounded_identity, valid_id16, valid_id32,
    valid_timestamp,
};

const STATE_ACCEPTED: i16 = 1;
pub(crate) const STATE_RESOLVING_ROUTE: i16 = 2;
pub(crate) const STATE_SUBMITTED_TO_PROVIDER: i16 = 3;
pub(crate) const STATE_PROVIDER_CONFIRMED: i16 = 4;
pub(crate) const STATE_REJECTED: i16 = 5;
const MAX_BODY_BYTES: u64 = 64 * 1024;
const MAX_CUSTODY_SOURCE_PROOF_BYTES: usize = 2_048;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryIntentBodyBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CreateDeliveryIntentV1 {
    pub logical_owner_id: String,
    pub intent_id: [u8; 16],
    pub canonical_conversation_id: CommunicationConversationIdV1,
    pub canonical_reply_message_id: Option<CommunicationMessageIdV1>,
    pub route: CommunicationDeliveryRouteV1,
    pub body_receipt: DeliveryIntentBodyBlobReceiptV1,
    pub request_fingerprint: [u8; 32],
    pub created_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryIntentStateV1 {
    Accepted,
    ResolvingRoute,
    SubmittedToProvider,
    ProviderConfirmed,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryIntentStatusRecordV1 {
    pub intent_id: [u8; 16],
    pub state: DeliveryIntentStateV1,
    pub state_revision: u64,
    pub provider_operation_id: Option<Vec<u8>>,
    pub rejection_code: Option<u16>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DeliveryIntentClaimV1 {
    pub logical_owner_id: String,
    pub intent_id: [u8; 16],
    pub canonical_conversation_id: CommunicationConversationIdV1,
    pub canonical_reply_message_id: Option<CommunicationMessageIdV1>,
    pub route: CommunicationDeliveryRouteV1,
    pub body_receipt: DeliveryIntentBodyBlobReceiptV1,
    pub worker_id: String,
    pub claim_epoch: u64,
    pub lease_expires_at_unix_seconds: i64,
}

#[derive(Clone, PartialEq, Eq)]
pub enum CreateDeliveryIntentOutcomeV1 {
    Created(DeliveryIntentStatusRecordV1),
    Existing(DeliveryIntentStatusRecordV1),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryIntentPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    Conflict,
    ClaimLost,
}

impl CommunicationDeliveryIntentPersistenceV1 {
    pub async fn create_intent(
        &self,
        command: &CreateDeliveryIntentV1,
    ) -> Result<CreateDeliveryIntentOutcomeV1, DeliveryIntentPersistenceErrorV1> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        let outcome = create_intent_in_transaction(&mut transaction, command).await?;
        transaction
            .commit()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(outcome)
    }

    pub async fn claim_next(
        &self,
        logical_owner_id: &str,
        worker_id: &str,
        now_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<Option<DeliveryIntentClaimV1>, DeliveryIntentPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_bounded_identity(worker_id)
            || !valid_timestamp(now_unix_seconds)
            || lease_expires_at_unix_seconds <= now_unix_seconds
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "WITH candidate AS (
               SELECT logical_owner_id, intent_id
               FROM makosh_data.communication_delivery_intent_jobs
               WHERE logical_owner_id = $1
                 AND (
                   state = $2
                   OR (state = $3 AND lease_expires_at_unix_seconds < $4)
                 )
               ORDER BY updated_at_unix_seconds, intent_id
               FOR UPDATE SKIP LOCKED
               LIMIT 1
             )
             UPDATE makosh_data.communication_delivery_intent_jobs AS jobs
             SET state = $3,
                 state_revision = jobs.state_revision + 1,
                 claimed_by = $5,
                 claim_epoch = jobs.claim_epoch + 1,
                 lease_expires_at_unix_seconds = $6,
                 updated_at_unix_seconds = $4
             FROM candidate
             WHERE jobs.logical_owner_id = candidate.logical_owner_id
               AND jobs.intent_id = candidate.intent_id
             RETURNING jobs.*",
        )
        .bind(logical_owner_id)
        .bind(STATE_ACCEPTED)
        .bind(STATE_RESOLVING_ROUTE)
        .bind(now_unix_seconds)
        .bind(worker_id)
        .bind(lease_expires_at_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        let Some(row) = row else {
            transaction
                .commit()
                .await
                .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
            return Ok(None);
        };
        let intent_id = id16(row.try_get("intent_id").map_err(row_error)?)?;
        let revision = positive_u64(row.try_get("state_revision").map_err(row_error)?)?;
        insert_transition(
            &mut transaction,
            logical_owner_id,
            &intent_id,
            revision,
            STATE_RESOLVING_ROUTE,
            None,
            now_unix_seconds,
        )
        .await?;
        let claim = claim_from_row(&row)?;
        transaction
            .commit()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(Some(claim))
    }

    pub async fn mark_submitted(
        &self,
        claim: &DeliveryIntentClaimV1,
        provider_operation_id: &[u8],
        now_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
        if !valid_claim(claim)
            || provider_operation_id.is_empty()
            || provider_operation_id.len() > 256
            || !valid_timestamp(now_unix_seconds)
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        self.claim_transition(
            claim,
            STATE_SUBMITTED_TO_PROVIDER,
            Some(provider_operation_id),
            None,
            false,
            now_unix_seconds,
        )
        .await
    }

    pub async fn reject_claim(
        &self,
        claim: &DeliveryIntentClaimV1,
        rejection_code: u16,
        now_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
        if !valid_claim(claim)
            || !(1..=32).contains(&rejection_code)
            || !valid_timestamp(now_unix_seconds)
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        self.claim_transition(
            claim,
            STATE_REJECTED,
            None,
            Some(rejection_code),
            true,
            now_unix_seconds,
        )
        .await
    }

    pub async fn mark_provider_confirmed(
        &self,
        logical_owner_id: &str,
        intent_id: [u8; 16],
        now_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
        self.terminal_provider_transition(
            logical_owner_id,
            intent_id,
            STATE_PROVIDER_CONFIRMED,
            None,
            now_unix_seconds,
        )
        .await
    }

    pub async fn mark_provider_rejected(
        &self,
        logical_owner_id: &str,
        intent_id: [u8; 16],
        rejection_code: u16,
        now_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
        if !(1..=32).contains(&rejection_code) {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        self.terminal_provider_transition(
            logical_owner_id,
            intent_id,
            STATE_REJECTED,
            Some(rejection_code),
            now_unix_seconds,
        )
        .await
    }

    pub async fn status(
        &self,
        logical_owner_id: &str,
        intent_id: [u8; 16],
    ) -> Result<Option<DeliveryIntentStatusRecordV1>, DeliveryIntentPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id) || !valid_id16(&intent_id) {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT intent_id, state, state_revision,
                    provider_operation_id, rejection_code
             FROM makosh_data.communication_delivery_intent_jobs
             WHERE logical_owner_id = $1 AND intent_id = $2",
        )
        .bind(logical_owner_id)
        .bind(intent_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .as_ref()
        .map(status_from_row)
        .transpose()
    }

    async fn claim_transition(
        &self,
        claim: &DeliveryIntentClaimV1,
        target_state: i16,
        provider_operation_id: Option<&[u8]>,
        rejection_code: Option<u16>,
        _clear_body: bool,
        now_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
        let claim_epoch = i64::try_from(claim.claim_epoch)
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "UPDATE makosh_data.communication_delivery_intent_jobs
             SET state = $1,
                 state_revision = state_revision + 1,
                 provider_operation_id = $2,
                 rejection_code = $3,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL,
                 updated_at_unix_seconds = $4
             WHERE logical_owner_id = $5 AND intent_id = $6
               AND state = $7 AND claimed_by = $8 AND claim_epoch = $9
               AND lease_expires_at_unix_seconds >= $4
             RETURNING intent_id, state, state_revision,
                       provider_operation_id, rejection_code",
        )
        .bind(target_state)
        .bind(provider_operation_id)
        .bind(
            rejection_code
                .map(i16::try_from)
                .transpose()
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?,
        )
        .bind(now_unix_seconds)
        .bind(&claim.logical_owner_id)
        .bind(claim.intent_id.as_slice())
        .bind(STATE_RESOLVING_ROUTE)
        .bind(&claim.worker_id)
        .bind(claim_epoch)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .ok_or(DeliveryIntentPersistenceErrorV1::ClaimLost)?;
        let status = status_from_row(&row)?;
        insert_transition(
            &mut transaction,
            &claim.logical_owner_id,
            &status.intent_id,
            status.state_revision,
            target_state,
            rejection_code,
            now_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(status)
    }

    async fn terminal_provider_transition(
        &self,
        logical_owner_id: &str,
        intent_id: [u8; 16],
        target_state: i16,
        rejection_code: Option<u16>,
        now_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_id16(&intent_id)
            || !valid_timestamp(now_unix_seconds)
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "UPDATE makosh_data.communication_delivery_intent_jobs
             SET state = $1,
                 state_revision = state_revision + 1,
                 rejection_code = $2,
                 updated_at_unix_seconds = $3
             WHERE logical_owner_id = $4 AND intent_id = $5 AND state = $6
             RETURNING intent_id, state, state_revision,
                       provider_operation_id, rejection_code",
        )
        .bind(target_state)
        .bind(
            rejection_code
                .map(i16::try_from)
                .transpose()
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?,
        )
        .bind(now_unix_seconds)
        .bind(logical_owner_id)
        .bind(intent_id.as_slice())
        .bind(STATE_SUBMITTED_TO_PROVIDER)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .ok_or(DeliveryIntentPersistenceErrorV1::Conflict)?;
        let status = status_from_row(&row)?;
        insert_transition(
            &mut transaction,
            logical_owner_id,
            &status.intent_id,
            status.state_revision,
            target_state,
            rejection_code,
            now_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(status)
    }
}

pub(crate) async fn create_intent_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command: &CreateDeliveryIntentV1,
) -> Result<CreateDeliveryIntentOutcomeV1, DeliveryIntentPersistenceErrorV1> {
    if !valid_create(command) {
        return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
    }
    let reply_message_id = command
        .canonical_reply_message_id
        .map(|id| id.bytes().to_vec());
    let reply_source_cursor = command
        .route
        .reply_to_source_cursor
        .map(|cursor| cursor.bytes().to_vec());
    let declared_bytes = i64::try_from(command.body_receipt.declared_bytes)
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_delivery_intent_jobs (
           intent_id, logical_owner_id, request_fingerprint,
           canonical_conversation_id, canonical_reply_message_id,
           provider_kind, account_cursor, conversation_cursor,
           reply_source_cursor, body_reference_id, body_declared_bytes,
           body_sha256, body_custody_source_proof,
           state, state_revision, created_at_unix_seconds,
           updated_at_unix_seconds
         ) VALUES (
           $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
           $13, $14, 1, $15, $15
         ) ON CONFLICT (logical_owner_id, intent_id) DO NOTHING",
    )
    .bind(command.intent_id.as_slice())
    .bind(&command.logical_owner_id)
    .bind(command.request_fingerprint.as_slice())
    .bind(command.canonical_conversation_id.bytes().as_slice())
    .bind(reply_message_id)
    .bind(provider_code(command.route.provider))
    .bind(command.route.account_cursor.bytes().as_slice())
    .bind(command.route.conversation_cursor.bytes().as_slice())
    .bind(reply_source_cursor)
    .bind(command.body_receipt.reference_id.as_slice())
    .bind(declared_bytes)
    .bind(command.body_receipt.sha256.as_slice())
    .bind(&command.body_receipt.custody_transfer_source_proof)
    .bind(STATE_ACCEPTED)
    .bind(command.created_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
    .rows_affected()
        == 1;

    if inserted {
        insert_transition(
            transaction,
            &command.logical_owner_id,
            &command.intent_id,
            1,
            STATE_ACCEPTED,
            None,
            command.created_at_unix_seconds,
        )
        .await?;
    }
    let row = sqlx::query(
        "SELECT intent_id, logical_owner_id, request_fingerprint, state,
                state_revision, provider_operation_id, rejection_code
         FROM makosh_data.communication_delivery_intent_jobs
         WHERE logical_owner_id = $1 AND intent_id = $2",
    )
    .bind(&command.logical_owner_id)
    .bind(command.intent_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
    let stored_owner: String = row
        .try_get("logical_owner_id")
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    let stored_fingerprint: Vec<u8> = row
        .try_get("request_fingerprint")
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    if stored_owner != command.logical_owner_id
        || stored_fingerprint.as_slice() != command.request_fingerprint
    {
        return Err(DeliveryIntentPersistenceErrorV1::Conflict);
    }
    let status = status_from_row(&row)?;
    Ok(if inserted {
        CreateDeliveryIntentOutcomeV1::Created(status)
    } else {
        CreateDeliveryIntentOutcomeV1::Existing(status)
    })
}

pub(crate) async fn insert_transition(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    intent_id: &[u8; 16],
    state_revision: u64,
    state: i16,
    rejection_code: Option<u16>,
    occurred_at_unix_seconds: i64,
) -> Result<(), DeliveryIntentPersistenceErrorV1> {
    let revision = i64::try_from(state_revision)
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
    sqlx::query(
        "INSERT INTO makosh_data.communication_delivery_intent_transitions (
           logical_owner_id, intent_id, state_revision, state,
           occurred_at_unix_seconds, rejection_code
         ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(logical_owner_id)
    .bind(intent_id.as_slice())
    .bind(revision)
    .bind(state)
    .bind(occurred_at_unix_seconds)
    .bind(
        rejection_code
            .map(i16::try_from)
            .transpose()
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?,
    )
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)
}

fn valid_create(command: &CreateDeliveryIntentV1) -> bool {
    valid_bounded_identity(&command.logical_owner_id)
        && valid_id16(&command.intent_id)
        && valid_id16(&command.canonical_conversation_id.bytes())
        && command
            .canonical_reply_message_id
            .is_none_or(|id| valid_id16(&id.bytes()))
        && valid_id32(&command.route.account_cursor.bytes())
        && valid_id32(&command.route.conversation_cursor.bytes())
        && command
            .route
            .reply_to_source_cursor
            .is_none_or(|cursor| valid_id32(&cursor.bytes()))
        && valid_id16(&command.body_receipt.reference_id)
        && (1..=MAX_BODY_BYTES).contains(&command.body_receipt.declared_bytes)
        && valid_id32(&command.body_receipt.sha256)
        && !command
            .body_receipt
            .custody_transfer_source_proof
            .is_empty()
        && command.body_receipt.custody_transfer_source_proof.len()
            <= MAX_CUSTODY_SOURCE_PROOF_BYTES
        && valid_id32(&command.request_fingerprint)
        && valid_timestamp(command.created_at_unix_seconds)
}

pub(crate) fn valid_claim(claim: &DeliveryIntentClaimV1) -> bool {
    valid_bounded_identity(&claim.logical_owner_id)
        && valid_bounded_identity(&claim.worker_id)
        && valid_id16(&claim.intent_id)
        && claim.claim_epoch > 0
        && valid_timestamp(claim.lease_expires_at_unix_seconds)
}

fn claim_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<DeliveryIntentClaimV1, DeliveryIntentPersistenceErrorV1> {
    let provider = provider_from_code(row.try_get("provider_kind").map_err(row_error)?)?;
    let reply_message_id = optional_id16(
        row.try_get::<Option<Vec<u8>>, _>("canonical_reply_message_id")
            .map_err(row_error)?,
    )?
    .map(CommunicationMessageIdV1::new);
    let reply_source_cursor = optional_id32(
        row.try_get::<Option<Vec<u8>>, _>("reply_source_cursor")
            .map_err(row_error)?,
    )?
    .map(CommunicationSourceCursorV1::new);
    Ok(DeliveryIntentClaimV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        intent_id: id16(row.try_get("intent_id").map_err(row_error)?)?,
        canonical_conversation_id: CommunicationConversationIdV1::new(id16(
            row.try_get("canonical_conversation_id")
                .map_err(row_error)?,
        )?),
        canonical_reply_message_id: reply_message_id,
        route: CommunicationDeliveryRouteV1 {
            provider,
            account_cursor: CommunicationSourceCursorV1::new(id32(
                row.try_get("account_cursor").map_err(row_error)?,
            )?),
            conversation_cursor: CommunicationSourceCursorV1::new(id32(
                row.try_get("conversation_cursor").map_err(row_error)?,
            )?),
            reply_to_source_cursor: reply_source_cursor,
        },
        body_receipt: DeliveryIntentBodyBlobReceiptV1 {
            reference_id: id16(row.try_get("body_reference_id").map_err(row_error)?)?,
            declared_bytes: positive_u64(row.try_get("body_declared_bytes").map_err(row_error)?)?,
            sha256: id32(row.try_get("body_sha256").map_err(row_error)?)?,
            custody_transfer_source_proof: row
                .try_get("body_custody_source_proof")
                .map_err(row_error)?,
        },
        worker_id: row.try_get("claimed_by").map_err(row_error)?,
        claim_epoch: positive_u64(row.try_get("claim_epoch").map_err(row_error)?)?,
        lease_expires_at_unix_seconds: row
            .try_get("lease_expires_at_unix_seconds")
            .map_err(row_error)?,
    })
}

pub(crate) fn status_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
    let state_code: i16 = row.try_get("state").map_err(row_error)?;
    let rejection_code = row
        .try_get::<Option<i16>, _>("rejection_code")
        .map_err(row_error)?
        .map(u16::try_from)
        .transpose()
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    Ok(DeliveryIntentStatusRecordV1 {
        intent_id: id16(row.try_get("intent_id").map_err(row_error)?)?,
        state: state_from_code(state_code)?,
        state_revision: positive_u64(row.try_get("state_revision").map_err(row_error)?)?,
        provider_operation_id: row.try_get("provider_operation_id").map_err(row_error)?,
        rejection_code,
    })
}

pub(crate) const fn provider_code(provider: CommunicationProviderProvenanceV1) -> i16 {
    match provider {
        CommunicationProviderProvenanceV1::MailImap => 1,
        CommunicationProviderProvenanceV1::Telegram => 2,
        CommunicationProviderProvenanceV1::WhatsAppWeb => 3,
        CommunicationProviderProvenanceV1::MailSmtp => 4,
        CommunicationProviderProvenanceV1::Zulip => 5,
        CommunicationProviderProvenanceV1::MailGmail => 6,
    }
}

fn provider_from_code(
    provider: i16,
) -> Result<CommunicationProviderProvenanceV1, DeliveryIntentPersistenceErrorV1> {
    match provider {
        1 => Ok(CommunicationProviderProvenanceV1::MailImap),
        2 => Ok(CommunicationProviderProvenanceV1::Telegram),
        3 => Ok(CommunicationProviderProvenanceV1::WhatsAppWeb),
        4 => Ok(CommunicationProviderProvenanceV1::MailSmtp),
        5 => Ok(CommunicationProviderProvenanceV1::Zulip),
        6 => Ok(CommunicationProviderProvenanceV1::MailGmail),
        _ => Err(DeliveryIntentPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn state_from_code(
    state: i16,
) -> Result<DeliveryIntentStateV1, DeliveryIntentPersistenceErrorV1> {
    match state {
        STATE_ACCEPTED => Ok(DeliveryIntentStateV1::Accepted),
        STATE_RESOLVING_ROUTE => Ok(DeliveryIntentStateV1::ResolvingRoute),
        STATE_SUBMITTED_TO_PROVIDER => Ok(DeliveryIntentStateV1::SubmittedToProvider),
        STATE_PROVIDER_CONFIRMED => Ok(DeliveryIntentStateV1::ProviderConfirmed),
        STATE_REJECTED => Ok(DeliveryIntentStateV1::Rejected),
        _ => Err(DeliveryIntentPersistenceErrorV1::InvalidRow),
    }
}

fn id16(bytes: Vec<u8>) -> Result<[u8; 16], DeliveryIntentPersistenceErrorV1> {
    bytes
        .try_into()
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)
}

fn id32(bytes: Vec<u8>) -> Result<[u8; 32], DeliveryIntentPersistenceErrorV1> {
    bytes
        .try_into()
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)
}

fn optional_id16(
    bytes: Option<Vec<u8>>,
) -> Result<Option<[u8; 16]>, DeliveryIntentPersistenceErrorV1> {
    bytes.map(id16).transpose()
}

fn optional_id32(
    bytes: Option<Vec<u8>>,
) -> Result<Option<[u8; 32]>, DeliveryIntentPersistenceErrorV1> {
    bytes.map(id32).transpose()
}

fn positive_u64(value: i64) -> Result<u64, DeliveryIntentPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DeliveryIntentPersistenceErrorV1::InvalidRow)
}

fn row_error(_: sqlx::Error) -> DeliveryIntentPersistenceErrorV1 {
    DeliveryIntentPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_codes_are_exact_and_reversible() {
        for provider in [
            CommunicationProviderProvenanceV1::MailImap,
            CommunicationProviderProvenanceV1::Telegram,
            CommunicationProviderProvenanceV1::WhatsAppWeb,
            CommunicationProviderProvenanceV1::MailSmtp,
            CommunicationProviderProvenanceV1::Zulip,
            CommunicationProviderProvenanceV1::MailGmail,
        ] {
            assert_eq!(provider_from_code(provider_code(provider)), Ok(provider));
        }
        assert_eq!(
            provider_from_code(7),
            Err(DeliveryIntentPersistenceErrorV1::InvalidRow)
        );
    }

    #[test]
    fn blob_receipt_validation_rejects_unbound_material() {
        let command = CreateDeliveryIntentV1 {
            logical_owner_id: "owner:test".to_owned(),
            intent_id: [1; 16],
            canonical_conversation_id: CommunicationConversationIdV1::new([2; 16]),
            canonical_reply_message_id: None,
            route: CommunicationDeliveryRouteV1 {
                provider: CommunicationProviderProvenanceV1::Telegram,
                account_cursor: CommunicationSourceCursorV1::new([3; 32]),
                conversation_cursor: CommunicationSourceCursorV1::new([4; 32]),
                reply_to_source_cursor: None,
            },
            body_receipt: DeliveryIntentBodyBlobReceiptV1 {
                reference_id: [5; 16],
                declared_bytes: 12,
                sha256: [6; 32],
                custody_transfer_source_proof: vec![7; 48],
            },
            request_fingerprint: [8; 32],
            created_at_unix_seconds: 1,
        };
        assert!(valid_create(&command));
        let mut invalid = command;
        invalid.body_receipt.custody_transfer_source_proof.clear();
        assert!(!valid_create(&invalid));
    }

    #[test]
    fn persistence_create_contract_contains_no_plaintext_body() {
        let source = include_str!("intents.rs");
        assert!(!source.contains(concat!("pub ", "plan:")));
        assert!(!source.contains(concat!("pub ", "body_utf8:")));
        assert!(!source.contains(concat!("Planned", "DeliveryIntentV1")));
        assert!(source.contains("ON CONFLICT (logical_owner_id, intent_id)"));
        assert!(source.contains("jobs.logical_owner_id = candidate.logical_owner_id"));
    }
}
