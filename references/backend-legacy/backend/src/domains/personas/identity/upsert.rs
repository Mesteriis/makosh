use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};
use uuid::Uuid;

use super::errors::PersonaIdentityError;
use super::models::{
    PersonaIdentityCandidateKind, PersonaIdentityCandidatePayload, PersonaIdentityReviewState,
};
use makosh_events_postgres::store::EventStore;

const PERSONA_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE: &str = "persona_identity.candidate.detected";
const LEGACY_PERSON_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE: &str =
    "person_identity.candidate.detected";

pub(super) async fn upsert_candidate(
    pool: &PgPool,
    payload: &PersonaIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonaIdentityReviewState,
) -> Result<(), PersonaIdentityError> {
    let mut transaction = pool.begin().await?;
    upsert_candidate_in_transaction(
        &mut transaction,
        payload,
        identity_candidate_id,
        review_state,
    )
    .await?;
    transaction.commit().await?;

    Ok(())
}

pub(crate) fn persona_identity_candidate_detected_event_type() -> &'static str {
    PERSONA_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE
}

pub(crate) fn is_persona_identity_candidate_detected_event_type(event_type: &str) -> bool {
    matches!(
        event_type,
        PERSONA_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE
            | LEGACY_PERSON_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE
    )
}

pub(super) async fn upsert_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonaIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonaIdentityReviewState,
) -> Result<(), PersonaIdentityError> {
    let stored_review_state: String = sqlx::query_scalar(
        r#"
        INSERT INTO persona_identity_candidates (
            identity_candidate_id,
            candidate_kind,
            left_persona_id,
            right_persona_id,
            email_address,
            evidence_summary,
            confidence,
            review_state,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL, NULL)
        ON CONFLICT (identity_candidate_id)
        DO UPDATE SET
            candidate_kind = EXCLUDED.candidate_kind,
            left_persona_id = EXCLUDED.left_persona_id,
            right_persona_id = EXCLUDED.right_persona_id,
            email_address = EXCLUDED.email_address,
            evidence_summary = EXCLUDED.evidence_summary,
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN persona_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN persona_identity_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            event_id = CASE
                WHEN persona_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN persona_identity_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN persona_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN persona_identity_candidates.actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN persona_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN persona_identity_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        RETURNING review_state
        "#,
    )
    .bind(&identity_candidate_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.left_persona_id)
    .bind(&payload.right_persona_id)
    .bind(&payload.email_address)
    .bind(&payload.evidence_summary)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .fetch_one(&mut **transaction)
    .await?;

    append_candidate_detected_event(
        transaction,
        payload,
        &identity_candidate_id,
        &stored_review_state,
    )
    .await?;

    Ok(())
}

async fn append_candidate_detected_event(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonaIdentityCandidatePayload,
    identity_candidate_id: &str,
    review_state: &str,
) -> Result<(), PersonaIdentityError> {
    let event_instance_id = Uuid::now_v7();
    let event = NewEventEnvelope::builder(
        format!(
            "persona_identity_candidate_detected:{identity_candidate_id}:{}",
            event_instance_id
        ),
        PERSONA_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "persona_identity",
            "provider": "makosh",
            "source_id": format!("{identity_candidate_id}:{event_instance_id}"),
        }),
        json!({
            "kind": "persona_identity_candidate",
            "identity_candidate_id": identity_candidate_id,
        }),
    )
    .payload(json!({
        "identity_candidate_id": identity_candidate_id,
        "candidate_kind": payload.candidate_kind.as_str(),
        "left_persona_id": &payload.left_persona_id,
        "right_persona_id": &payload.right_persona_id,
        "email_address": &payload.email_address,
        "evidence_summary": &payload.evidence_summary,
        "confidence": payload.confidence,
        "review_state": review_state,
    }))
    .build()?;

    match EventStore::append_in_transaction(transaction, &event).await {
        Ok(_) => Ok(()),
        Err(error) if error.is_unique_violation() => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub(crate) async fn load_identity_candidate_payload(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
) -> Result<PersonaIdentityCandidatePayload, PersonaIdentityError> {
    let row = sqlx::query(
        r#"
        SELECT
            candidate_kind,
            left_persona_id,
            right_persona_id,
            email_address,
            evidence_summary,
            confidence::float8 AS confidence
        FROM persona_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_optional(&mut **transaction)
    .await?;

    let row = row.ok_or(PersonaIdentityError::IdentityCandidateNotFound)?;
    let candidate_kind = match row.try_get::<String, _>("candidate_kind")?.as_str() {
        "merge_personas" | "merge_persons" => PersonaIdentityCandidateKind::MergePersonas,
        "attach_email_address" => PersonaIdentityCandidateKind::AttachEmailAddress,
        "split_persona" | "split_person" => PersonaIdentityCandidateKind::SplitPersona,
        other => return Err(PersonaIdentityError::InvalidCandidateKind(other.to_owned())),
    };

    Ok(PersonaIdentityCandidatePayload {
        candidate_kind,
        left_persona_id: row.try_get("left_persona_id")?,
        right_persona_id: row.try_get("right_persona_id")?,
        email_address: row.try_get("email_address")?,
        evidence_summary: row.try_get("evidence_summary")?,
        confidence: row.try_get("confidence")?,
    })
}

pub(crate) fn parse_persona_identity_candidate_kind(
    value: &str,
) -> Result<PersonaIdentityCandidateKind, PersonaIdentityError> {
    match value {
        "merge_personas" | "merge_persons" => Ok(PersonaIdentityCandidateKind::MergePersonas),
        "attach_email_address" => Ok(PersonaIdentityCandidateKind::AttachEmailAddress),
        "split_persona" | "split_person" => Ok(PersonaIdentityCandidateKind::SplitPersona),
        other => Err(PersonaIdentityError::InvalidCandidateKind(other.to_owned())),
    }
}

pub(crate) fn parse_persona_identity_review_state(
    value: &str,
) -> Result<PersonaIdentityReviewState, PersonaIdentityError> {
    match value {
        "suggested" => Ok(PersonaIdentityReviewState::Suggested),
        "user_confirmed" => Ok(PersonaIdentityReviewState::UserConfirmed),
        "user_rejected" => Ok(PersonaIdentityReviewState::UserRejected),
        other => Err(PersonaIdentityError::InvalidReviewState(other.to_owned())),
    }
}
