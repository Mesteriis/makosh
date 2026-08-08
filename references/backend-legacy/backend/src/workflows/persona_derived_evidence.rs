use chrono::{DateTime, Utc};
use makosh_events_api::StoredEventEnvelope;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::application::relationship_graph::{
    RelationshipGraphCoordinator, RelationshipGraphCoordinatorError,
};
use crate::domains::obligations::models::entity_kind::ObligationEntityKind;
use crate::domains::obligations::models::evidence::NewObligationEvidence;
use crate::domains::obligations::models::obligation::NewObligation;
use crate::domains::obligations::models::states::ObligationReviewState;
use crate::domains::obligations::ports::{ObligationReviewPort, ObligationReviewPortError};
use crate::domains::personas::core::roles::persona_role_knowledge_id;
use crate::domains::personas::core::roles::{
    PERSONA_ROLE_ASSIGNED_EVENT_TYPE, PERSONA_ROLE_REMOVED_EVENT_TYPE,
};
use crate::domains::personas::enrichment::PERSONA_TRUST_SCORE_CHANGED_EVENT_TYPE;
use crate::domains::personas::trust::promises::PERSONA_PROMISE_CREATED_EVENT_TYPE;
use crate::domains::relationships::{
    ids::relationship_id,
    models::{
        NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewState,
    },
};
use crate::engines::trust::{engine::TrustEngine, errors::TrustEngineError};
use crate::workflows::review_mirror::{
    ReviewMirrorError, relationship::ensure_relationship_review_item,
};
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

pub const PERSONA_DERIVED_EVIDENCE_CONSUMER: &str = "persona_derived_evidence";
const LEGACY_PERSON_ROLE_ASSIGNED_EVENT_TYPE: &str = "person.role.assigned";
const LEGACY_PERSON_ROLE_REMOVED_EVENT_TYPE: &str = "person.role.removed";
const LEGACY_PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE: &str = "person.enrichment.trust_score_changed";
const LEGACY_PERSON_PROMISE_CREATED_EVENT_TYPE: &str = "person.promise.created";

#[derive(Debug, Error)]
pub enum PersonaDerivedEvidenceWorkflowError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    RelationshipGraph(#[from] RelationshipGraphCoordinatorError),

    #[error(transparent)]
    Obligation(#[from] ObligationReviewPortError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error(transparent)]
    Trust(#[from] TrustEngineError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },
}

pub async fn project_persona_derived_evidence_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_persona_derived_evidence_event_inner(&pool, &event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

async fn project_persona_derived_evidence_event_inner(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonaDerivedEvidenceWorkflowError> {
    match event.event.event_type.as_str() {
        PERSONA_ROLE_ASSIGNED_EVENT_TYPE | LEGACY_PERSON_ROLE_ASSIGNED_EVENT_TYPE => {
            materialize_role_assigned(pool, event).await
        }
        PERSONA_ROLE_REMOVED_EVENT_TYPE | LEGACY_PERSON_ROLE_REMOVED_EVENT_TYPE => {
            materialize_role_removed(pool, event).await
        }
        PERSONA_TRUST_SCORE_CHANGED_EVENT_TYPE | LEGACY_PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE => {
            materialize_trust_score(pool, event).await
        }
        PERSONA_PROMISE_CREATED_EVENT_TYPE | LEGACY_PERSON_PROMISE_CREATED_EVENT_TYPE => {
            materialize_promise(pool, event).await
        }
        _ => Ok(()),
    }
}

async fn materialize_role_assigned(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonaDerivedEvidenceWorkflowError> {
    let persona_id = required_persona_id(&event.event.payload)?;
    let role = required_string(&event.event.payload, "role")?;
    let assigned_by = optional_string(&event.event.payload, "assigned_by");
    let role_knowledge_id = optional_string(&event.event.payload, "role_knowledge_id")
        .map(str::to_owned)
        .unwrap_or_else(|| persona_role_knowledge_id(role));

    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSONA_ROLE",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "persona_id": persona_id,
                    "role": role,
                    "assigned_by": assigned_by,
                    "action": "assign",
                }),
                format!("persona://{persona_id}/roles/{role_knowledge_id}"),
            )
            .provenance(json!({
                "captured_by": "persona_derived_evidence.role_assigned",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: persona_id.to_owned(),
        target_entity_kind: RelationshipEntityKind::Knowledge,
        target_entity_id: role_knowledge_id,
        relationship_type: "has_role".to_owned(),
        trust_score: 1.0,
        strength_score: 0.7,
        confidence: 1.0,
        review_state: RelationshipReviewState::UserConfirmed,
        valid_from: None,
        valid_to: None,
        metadata: json!({
            "compatibility_source": "persona_roles",
            "role": role,
            "assigned_by": assigned_by,
        }),
    };
    let evidence = NewRelationshipEvidence::observation(observation.observation_id)
        .excerpt(role)
        .metadata(json!({
            "compatibility_source": "persona_roles",
        }));

    let _ = RelationshipGraphCoordinator::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    Ok(())
}

async fn materialize_role_removed(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonaDerivedEvidenceWorkflowError> {
    let persona_id = required_persona_id(&event.event.payload)?;
    let role = required_string(&event.event.payload, "role")?;
    let role_knowledge_id = optional_string(&event.event.payload, "role_knowledge_id")
        .map(str::to_owned)
        .unwrap_or_else(|| persona_role_knowledge_id(role));
    let relationship_id = relationship_id(
        RelationshipEntityKind::Persona,
        persona_id,
        "has_role",
        RelationshipEntityKind::Knowledge,
        &role_knowledge_id,
    );

    let _ = RelationshipGraphCoordinator::new(pool.clone())
        .set_review_state_with_observation(
            &relationship_id,
            RelationshipReviewState::UserRejected,
            None,
            None,
        )
        .await?;
    Ok(())
}

async fn materialize_trust_score(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonaDerivedEvidenceWorkflowError> {
    let persona_id = required_persona_id(&event.event.payload)?;
    let trust_score = required_i16(&event.event.payload, "trust_score")?;
    let normalized_confidence = f64::from(trust_score.clamp(0, 100)) / 100.0;
    let source_observation_id = optional_string(&event.event.payload, "source_observation_id");
    let evidence_text = format!("trust_score={trust_score}");
    let source_reliability = TrustEngine::source_reliability_signal(
        &format!("persona_enrichment:{persona_id}:trust_score"),
        &evidence_text,
        normalized_confidence,
    )?;

    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSONA_TRUST_SIGNAL",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "persona_id": persona_id,
                    "trust_score": trust_score,
                    "source_observation_id": source_observation_id,
                    "action": "trust_score_enrichment",
                }),
                format!("persona://{persona_id}/trust-score"),
            )
            .confidence(normalized_confidence)
            .provenance(json!({
                "captured_by": "persona_derived_evidence.trust_score",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let Some(owner_person_id) = owner_persona_id(pool, persona_id).await? else {
        return Ok(());
    };

    let relationship_signal = TrustEngine::persona_compatibility_score_signal(trust_score);
    let relationship = NewRelationship::between_personas(
        owner_person_id.clone(),
        persona_id.to_owned(),
        relationship_signal.relationship_type,
        relationship_signal.trust_score,
        relationship_signal.strength_score,
        relationship_signal.confidence,
        RelationshipReviewState::Suggested,
    )
    .metadata(json!({
        "compatibility_source": "personas.trust_score",
        "trust_score": trust_score,
    }));
    let evidence = NewRelationshipEvidence::observation(observation.observation_id.clone())
        .excerpt(evidence_text)
        .metadata(json!({
            "compatibility_source": "personas.trust_score",
            "source_observation_id": source_observation_id,
            "trust_source_reliability": {
                "signal_type": source_reliability.kind.as_str(),
                "affected_source": source_reliability.affected_source,
                "direction": source_reliability.direction.as_str(),
                "confidence": source_reliability.confidence,
            }
        }));
    let relationship = RelationshipGraphCoordinator::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    let _ = ensure_relationship_review_item(
        pool,
        crate::workflows::review_mirror::relationship::RelationshipReviewInput {
            relationship_id: &relationship.relationship_id,
            relationship_type: &relationship.relationship_type,
            source_entity_kind: relationship.source_entity_kind.as_str(),
            source_entity_id: &relationship.source_entity_id,
            target_entity_kind: relationship.target_entity_kind.as_str(),
            target_entity_id: &relationship.target_entity_id,
            confidence: relationship.confidence,
            summary: Some("trust_score enrichment suggests a persona trust relationship"),
            observation_id: &observation.observation_id,
        },
    )
    .await?;

    Ok(())
}

async fn materialize_promise(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonaDerivedEvidenceWorkflowError> {
    let promise_id = required_string(&event.event.payload, "promise_id")?;
    let persona_id = required_persona_id(&event.event.payload)?;
    let description = required_string(&event.event.payload, "description")?;
    let due_at: Option<DateTime<Utc>> = serde_json::from_value(
        event
            .event
            .payload
            .get("due_at")
            .cloned()
            .unwrap_or(Value::Null),
    )?;

    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSONA_PROMISE",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "promise_id": promise_id,
                    "persona_id": persona_id,
                    "description": description,
                    "due_at": &due_at,
                    "action": "create",
                }),
                format!("persona://{persona_id}/promises/{promise_id}"),
            )
            .provenance(json!({
                "captured_by": "persona_derived_evidence.promise_created",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let mut obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        persona_id.to_owned(),
        description.to_owned(),
        1.0,
        ObligationReviewState::UserConfirmed,
    )
    .metadata(json!({
        "compatibility_source": "persona_promises",
        "persona_promise_id": promise_id,
    }));
    if let Some(due_at) = due_at {
        obligation = obligation.due_at(due_at);
    }
    let evidence = NewObligationEvidence::observation(observation.observation_id)
        .quote(description)
        .metadata(json!({
            "compatibility_source": "persona_promises",
            "persona_promise_id": promise_id,
        }));
    let _ = ObligationReviewPort::new(pool.clone())
        .upsert_with_evidence(&obligation, &[evidence])
        .await?;

    Ok(())
}

async fn owner_persona_id(
    pool: &PgPool,
    target_person_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT persona_id
        FROM personas
        WHERE is_self = true
          AND persona_id <> $1
        LIMIT 1
        "#,
    )
    .bind(target_person_id)
    .fetch_optional(pool)
    .await
}

fn required_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, PersonaDerivedEvidenceWorkflowError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(PersonaDerivedEvidenceWorkflowError::MissingPayloadField(
            field,
        ))
}

fn optional_string<'a>(payload: &'a Value, field: &'static str) -> Option<&'a str> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn required_persona_id(payload: &Value) -> Result<&str, PersonaDerivedEvidenceWorkflowError> {
    optional_string(payload, "persona_id")
        .or_else(|| optional_string(payload, "person_id"))
        .ok_or(PersonaDerivedEvidenceWorkflowError::MissingPayloadField(
            "persona_id",
        ))
}

fn required_i16(
    payload: &Value,
    field: &'static str,
) -> Result<i16, PersonaDerivedEvidenceWorkflowError> {
    let value = payload.get(field).and_then(Value::as_i64).ok_or(
        PersonaDerivedEvidenceWorkflowError::MissingPayloadField(field),
    )?;
    i16::try_from(value).map_err(
        |_| PersonaDerivedEvidenceWorkflowError::InvalidPayloadField {
            field,
            value: value.to_string(),
        },
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};

    use super::required_persona_id;

    #[test]
    fn required_persona_id_accepts_legacy_event_payloads() {
        let mut fields = Map::new();
        fields.insert(
            "person_id".to_owned(),
            Value::String("legacy-persona".to_owned()),
        );
        let payload = Value::Object(fields);

        assert_eq!(
            required_persona_id(&payload).expect("legacy Persona identifier must remain readable"),
            "legacy-persona"
        );
    }
}
